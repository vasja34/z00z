---
post_title: "029 Crypto Audit Wallets Storage Audit GPT54"
author1: "GitHub Copilot"
post_slug: "029-crypto-audit-wallets-storage-audit-gpt54"
microsoft_alias: "copilot"
featured_image: ""
categories:
  - engineering
tags:
  - rust
  - crypto-audit
  - wallets
  - storage
  - z00z
ai_note: "AI-assisted report generated from Rust source review only."
summary: "Deep storage-focused crypto audit of the z00z_wallets Rust implementation, covering seed handling, .wlt persistence, backup cryptography, metadata exposure, and logging redaction."
post_date: "2026-03-26"
---

# 029 Crypto Audit Wallets Storage Audit GPT54

## Scope

📌 This audit reviewed only Rust implementation files under `crates/z00z_wallets/src/**/*.rs`.

📌 The review excluded markdown, YAML, README files, non-Rust documentation, and Tari vendor code.

📌 The audit focus was storage-relevant cryptography and secret lifecycle handling:

- `.wlt` creation, open, migration, and secret persistence
- password KDF and master-key wrapping
- seed phrase at-rest representation and reveal flow
- backup export/import cryptography
- auxiliary key storage
- RPC logging/redaction around high-risk wallet methods

📌 The audit did not attempt a full transaction, ZK, or network protocol review outside the storage boundary.

## Threat Model

🎯 Assets reviewed:

- user password material
- persisted `MASTER_KEY`
- derived DATA / INDEX / INTEGRITY keys
- BIP-39 seed entropy and reconstructed mnemonic
- receiver secret material
- encrypted backup payloads and backup metadata

🎯 Adversaries considered:

- attacker with offline access to `.wlt` or backup files
- attacker supplying malformed wallet files to trigger excessive KDF work or parser faults
- local operator or malware reading process memory, crash dumps, or temporary files
- observability / logging paths that could leak secret or privacy-sensitive material

🎯 Security goals:

- no plaintext seed persistence on disk
- bounded offline password attack surface
- authenticated storage records with unambiguous AAD
- deterministic, versioned migration behavior
- minimized secret lifetime in memory
- minimized metadata leakage for a privacy-oriented wallet

## Executive Verdict

✅ No S0 or S1 issue was identified in the reviewed storage path.

✅ The `.wlt` path is materially stronger than the backup and auxiliary key-store paths.

⚠️ The strongest design is the native `.wlt` container: it versions KDF/AAD/HKDF semantics, validates untrusted KDF metadata before expensive work, stores seed entropy instead of mnemonic text, and avoids plaintext-on-disk by round-tripping through `/dev/shm`.

⚠️ The weakest design is the backup and auxiliary secret-storage edge: backup KDF semantics are under-specified, plaintext metadata is intentionally exposed, and one public key-store API holds password bytes in an ordinary cloneable heap buffer.

## Strengths Observed

✅ `.wlt` create/open enforces zstd-wrapped storage and uses `/dev/shm` work files instead of writing a plaintext RedB database to persistent disk.

✅ Temporary work files are permission-hardened through the I/O boundary in `crates/z00z_wallets/src/db/wlt_io.rs` and the open/create flows in `crates/z00z_wallets/src/db/redb_wallet_store.rs`.

✅ Seed storage is stronger than a naive wallet design: `store_seed_secret()` converts the mnemonic into 32 bytes of entropy plus language metadata before AEAD encryption instead of storing the phrase text directly.

✅ `verify_password_for_session()` re-derives the persisted master key and compares it with the unlocked session master key using `ConstantTimeEq`.

✅ `.wlt` persistence explicitly versions KDF, secret AAD format, and HKDF info derivation, and it contains migration helpers for legacy formats.

✅ RPC logging for `wallet.session.show_seed_phrase`, `wallet.session.unlock_wallet`, `app.wallet.create_wallet`, and other sensitive methods is intentionally summarized and redacted in `crates/z00z_wallets/src/adapters/rpc/logging/summarize.rs`.

## Findings

### S2 Medium - Backup KDF Semantics Are Not Self-Describing

🚨 Evidence:

- `crates/z00z_wallets/src/core/backup/wallet_backup.rs::WalletBackupCrypto::derive_key()` selects KDF behavior implicitly from compile-time configuration: `Argon2Params::test_fast()` under `feature = "test-fast"`, otherwise `Argon2Params::moderate()`.
- The same function expands the 16-byte backup salt to 32 bytes by repetition instead of a versioned, explicit normalization rule.
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs` stores only `algorithm`, `kdf`, `salt`, and `nonce` in `BackupEncryptionV1`.
- No persisted field records Argon2 memory, iterations, parallelism, or a backup-KDF version.

💥 Why this matters:

- Backup files are not fully self-describing.
- Decryption compatibility depends on current build behavior and hidden library defaults rather than only on bytes stored in the backup.
- A future change in Argon2 parameters, salt handling, or an accidental build/profile mismatch can make historical backups undecryptable.
- This is recovery-critical and inconsistent with the stronger `.wlt` design, which explicitly versions KDF/AAD/HKDF semantics.

🔑 Exploit / failure mode:

- This is primarily an availability and recoverability failure, not a direct confidentiality break.
- A user can successfully export backups today and later lose the ability to restore them after a build-profile or implementation drift.

⚙️ Recommendation:

- Persist a backup KDF version and explicit Argon2 parameters in the backup container.
- Persist the salt-normalization rule as part of the backup crypto contract.
- Reject unknown KDF versions rather than silently interpreting them with current defaults.
- Align backup format governance with the `.wlt` versioning discipline.

👁️‍🗨️ Confidence: 96%

### S2 Medium - `FileKeyStore` Keeps Its Encryption Password As Cloneable Plain Heap Bytes

🚨 Evidence:

- `crates/z00z_wallets/src/core/storage/file_key_store.rs` defines `EncryptionScheme::Password(Vec<u8>)`.
- `EncryptionScheme` derives `Clone`, `Debug`, `PartialEq`, and `Eq`.
- `FileKeyStore` itself derives `Clone` and stores `encryption: EncryptionScheme`.
- The password bytes are passed directly into `ReceiverSecret::to_encrypted()` and `ReceiverSecret::from_encrypted()` with no `Hidden`, `SafePassword`, `Zeroize`, or zeroize-on-drop semantics.

💥 Why this matters:

- Password material can be duplicated by routine cloning of `FileKeyStore` or `EncryptionScheme`.
- The secret can remain in allocator-managed heap memory after logical use.
- This materially weakens the otherwise consistent secret-handling model used in the `.wlt` path.

🔑 Exploit / failure mode:

- This is a local secret-lifecycle issue rather than a remote exploit.
- Memory inspection, crash dumps, swap leakage, or accidental cloning can expose the password protecting receiver-secret files.

⚙️ Recommendation:

- Replace `Vec<u8>` with `SafePassword`, `Hidden<SecretBytes>`, or a zeroizing secret wrapper.
- Remove `Debug`, `PartialEq`, and `Eq` from secret-bearing enums unless they are strictly required.
- Avoid `Clone` for secret-bearing storage schemes unless the clone semantics are explicit and defensible.

⚠️ Scope note:

- No in-crate production caller was observed in the reviewed source.
- The issue still matters because the API is public and re-exported.

👁️‍🗨️ Confidence: 95%

### S3 Low - Seed Phrase Handling Repeatedly Escapes Into Ordinary `String` Boundaries

🚨 Evidence:

- `crates/z00z_wallets/src/services/app_service.rs::generate_seed_phrase_24()` converts `Hidden<String>` into a plain `String` for higher layers.
- `crates/z00z_wallets/src/services/wallet_service.rs::persist_wlt_snapshot()` clones `seed_phrase` into an owned `String` before passing it into the blocking `.wlt` persistence worker.
- `crates/z00z_wallets/src/services/wallet_service.rs::show_seed_phrase()` receives a plaintext mnemonic `String` from `reveal_seed_phrase()`, then re-encrypts it for the RPC response.

💥 Why this matters:

- At-rest storage is strong, but runtime secret lifetime is weaker than it appears.
- Mnemonic text can linger in ordinary heap allocations with no zeroization guarantees.
- This increases exposure to local memory scraping, core dumps, post-crash inspection, and accidental secret retention during refactors.

🔑 Exploit / failure mode:

- This is a local hardening gap rather than a direct disk or network compromise.
- The most likely effect is mnemonic recovery from process memory under local compromise conditions.

⚙️ Recommendation:

- Carry seed material as `Hidden<_>`, `Zeroizing<String>`, or entropy objects for as long as possible.
- Defer conversion to plain UTF-8 strings until the final API boundary that absolutely requires it.
- Explicitly wipe temporary plaintext mnemonic buffers after packaging RPC or backup responses.

👁️‍🗨️ Confidence: 89%

### S3 Low - Backup Container Exposes Wallet Metadata In Plaintext

🚨 Evidence:

- `crates/z00z_wallets/src/core/backup/backup_exporter.rs` defines `BackupMetadata` with plaintext `created_at`, `wallet_id`, and `network` fields.
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs` embeds `metadata` in `BackupContainerV1`, which is serialized as outer JSON.
- The payload itself is encrypted, but the metadata remains visible without the password.

💥 Why this matters:

- A stolen backup still leaks stable wallet identifier, network, and creation timing.
- For a privacy-focused wallet, metadata disclosure is part of the threat surface even when ciphertext confidentiality is intact.

🔑 Exploit / failure mode:

- This is a privacy leak, not a direct decryption break.
- A passive adversary can correlate multiple backups or tie a backup file to a specific wallet identifier.

⚙️ Recommendation:

- Encrypt metadata, or split the format into a minimal public header plus an encrypted metadata block.
- If plaintext metadata is required for UX, document it as an explicit tradeoff.

👁️‍🗨️ Confidence: 93%

## Hardening Notes

⚠️ `crates/z00z_wallets/src/db/redb_wallet_store.rs` contains a `wallet_debug_tools` feature-gated debug path that can export decrypted secret material, including `plaintext_b64` for the master key. It is properly feature-gated in source, so it is not treated as an active production finding in this report.

⚠️ `crates/z00z_wallets/src/core/key/key_manager.rs` has a `verbose-logging` feature-gated path that logs derivation paths. The code comments explicitly warn against using it in production. That warning is appropriate because BIP-44 paths are privacy-sensitive metadata.

⚠️ `crates/z00z_wallets/src/core/backup/wallet_backup.rs` adds an explicit checksum on top of AEAD authentication. This is not inherently wrong, but it increases long-term format complexity and should stay versioned and documented if retained.

## What Looks Good In The `.wlt` Path

✅ `store_seed_secret()` stores entropy plus language metadata under AEAD, not mnemonic text.

✅ `read_wallet_meta_header()` treats persisted KDF parameters as untrusted input and validates them before expensive derivation.

✅ `open_wlt_with_deps()` and discovery/open code reject non-zstd `.wlt` content and insist on `/dev/shm` for plaintext work files.

✅ `store_seed_revealed_at_secret()` keeps reveal audit state encrypted under the master key instead of writing plaintext markers.

✅ `reveal_seed_phrase()` and `reveal_seed_phrase_once()` wipe decrypted secret buffers after reconstructing the return value.

✅ `verify_password_for_session()` performs a bounded authentication check against the persisted master-key record instead of relying only on cached state.

## Remediation Order

🚩 Priority 1:

- make backup decryption semantics self-describing and versioned

🚩 Priority 2:

- remove plain cloneable password bytes from `FileKeyStore`

🚩 Priority 3:

- tighten service-layer secret lifetimes for mnemonic text

🚩 Priority 4:

- decide whether plaintext backup metadata is an accepted privacy tradeoff or an implementation gap

## Confidence Assessment

📌 Confidence is high for the `.wlt`, backup, and logging conclusions because the relevant storage and reveal paths were read directly in source.

📌 Confidence is lower for feature-profile exposure beyond what is visible in `#[cfg(...)]` usage because this audit intentionally did not inspect non-Rust configuration files.

📌 Overall confidence: 90% for the storage-focused claims in this report.
