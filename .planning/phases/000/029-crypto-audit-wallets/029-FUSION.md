---
post_title: "029 Crypto Audit Wallets Fusion"
author1: "GitHub Copilot"
post_slug: "029-crypto-audit-wallets-fusion"
microsoft_alias: "copilot"
featured_image: ""
categories:
  - engineering
tags:
  - rust
  - crypto-audit
  - wallets
  - fusion
  - security
ai_note: "AI-assisted fusion of three wallet crypto audit reports into one canonical source."
summary: "Canonical fusion of three crypto audit reports for z00z_wallets, preserving source coverage, deduplicating overlaps, and consolidating remediation priorities, tests, and open questions."
post_date: "2026-03-26"
---

# 029 Crypto Audit Wallets Fusion

## FUS-01 Canonical Verdict

📌 This document fuses three source audits into one canonical wallet crypto review for
`crates/z00z_wallets`.

📌 The combined verdict is `Risky but salvageable`, with one important normalization:
`wallets-audit-m27.md` identified two service-layer `S1` crash findings, while the
storage-focused and primitive-focused reports did not rate those paths as `S1` because they
did not center their review on the same code surfaces.

📌 Production readiness is `BLOCKED` until these issues are resolved:

1. Service-layer panic paths using `expect()` and `unwrap()` in wallet and chain services.
2. View-key canonicalization risk between unversioned and versioned derivation functions.
3. KDF and encryption-path fragmentation across `WalletEncryption`, RedB V1/V2 migration,
   and backup export/import formats.

📌 After those blockers are resolved, the crate can move to controlled integration and
testnet verification, but it still requires follow-through on secret-lifecycle hardening,
backup format governance, and digest-framing cleanup.

## FUS-02 Source Set and Review Boundaries

📌 The fusion input set is exactly these files:

| Source | Date | Primary Emphasis | Highest Severity Observed |
| --- | ---: | --- | --- |
| `wallets-audit-m27.md` | 2026-03-26 | Service layer, key manager, entropy, cache, open ambiguities | `S1` |
| `storage-audit-gpt54.md` | 2026-03-26 | `.wlt` storage, backups, secret lifetimes, logging, metadata | `S2` |
| `wallets-audit-sonet46.md` | 2025-07-17 | Primitive selection, composition review, protocol framing | `S2` |

📌 All three sources reviewed Rust implementation files under
`crates/z00z_wallets/src/**/*.rs` and excluded vendor Tari code.

📌 The merged boundary covers:

1. Wallet encryption and RedB persistence.
2. Backup export/import cryptography and metadata exposure.
3. Key derivation, stealth key usage, view-key derivation, and cache behavior.
4. Transaction digest framing and proof verification helper behavior.
5. Service-layer error propagation, entropy signaling, and secret-lifecycle handling.

📌 The merged boundary does not claim a complete review of non-Rust documentation,
deployment configuration, or the full network protocol outside wallet-owned code paths.

## FUS-03 Assets, Goals, and Threat Model

### FUS-03.1 Assets and Security Goals

📌 The source reports consistently treat these as protected assets:

| Asset | Why It Matters |
| --- | --- |
| BIP-39 seed and seed entropy | Root wallet secret; compromise means total loss or full recovery loss |
| Receiver secret and derived view key material | Privacy boundary for stealth scanning |
| Wallet encryption key and persisted master-key material | At-rest protection for `.wlt` state |
| Ephemeral stealth scalar `r` | Prevents output linkability |
| Transaction blinding factors and signatures | Prevents amount leakage and unauthorized spending |
| Backup containers and metadata | Recovery path plus privacy-sensitive identifiers |

📌 The security goals assumed across the sources are confidentiality, integrity,
authentication on wallet unlock, key separation between roles, non-malleable signing,
high-quality entropy, bounded offline password resistance, versioned persistence semantics,
and minimal metadata leakage.

### FUS-03.2 Attackers and Failure Modes

📌 The fused threat model includes:

| Actor or Failure | Merged Assumption |
| --- | --- |
| Offline attacker with stolen `.wlt` or backup file | Must face authenticated encryption plus Argon2id cost and versioned parsing rules |
| Malicious wallet file or crafted metadata | Must not trigger unbounded CPU or memory work before validation |
| Passive network observer | Should not learn recipient ownership or secret payloads |
| Active output-level adversary | Should not exploit ECDH, view-key mismatch, or framing ambiguity |
| Local memory scraper or crash-dump reader | Benefits if secrets escape `Hidden<T>` and zeroizing boundaries |
| Process crash or service timeout | Must not turn into silent wallet termination in real-funds paths |

📌 The fused failure model also includes gap-limit invariant breaks, stale key caches,
legacy KDF compatibility drift, backup recovery drift across build profiles, and protocol
ambiguity around canonical view-key derivation.

## FUS-04 Verified Strengths and Sound Constructions

### FUS-04.1 Primitive Selection and Construction Choices

📌 The primitive set is broadly sound: Ristretto255, XChaCha20-Poly1305, Argon2id,
Bulletproofs+, Poseidon2-Goldilocks, BIP-32/BIP-44 bridging, and length-framed hashing are
appropriate for the wallet domain when used under the documented constraints.

📌 The strongest positive findings preserved from the source reports are:

1. Ristretto255 use is cofactor-safe and paired with identity-point rejection in the ECDH
   path.
2. XChaCha20-Poly1305 is a good at-rest choice, and the RedB path documents AAD and KDF
   semantics better than the older wallet-encryption path.
3. `RANGE_PROOF_BITS_V1 = 64` correctly covers the entire `u64` value range.
4. `hash_zk` and Poseidon2 packing use item counts and per-item length framing, which rules
   out concatenation ambiguity in the reviewed hash inputs.
5. Hedged ephemeral derivation ties randomness to RNG output plus transaction-local data,
   reducing single-source RNG failure risk.

### FUS-04.2 Domain Separation, ECDH, and Secret Handling Positives

📌 Domain separation is a net strength. The source reports agree that the crate defines many
distinct domains and does not show a reused domain string across clearly separate protocol
roles.

📌 Secret-bearing types generally follow a strong pattern with `Hidden<T>`, zeroizing
behavior, and constant-time equality for sensitive comparisons.

📌 ECDH composition is structurally sound once sender and receiver use the same view-key
derivation path. The sender-side and receiver-side DH computations, public-key binding, and
owner-tag derivations were all assessed positively.

### FUS-04.3 Strengths of the Native `.wlt` Path

📌 The storage-focused review found the `.wlt` path materially stronger than the backup and
auxiliary key-store edges.

📌 Preserved `.wlt` strengths include:

1. Seed storage as encrypted entropy plus language metadata instead of raw mnemonic text.
2. Validation of persisted KDF parameters before expensive work.
3. Rejection of non-zstd wallet content and use of `/dev/shm` for plaintext work files.
4. Encrypted audit state for seed reveal timestamps.
5. Explicit wiping of decrypted secret buffers in reveal flows.
6. Password verification against persisted master-key material rather than cached state only.

## FUS-05 Blocking Service-Layer Reliability Findings

📌 These findings are preserved as the highest-severity blockers because they can crash the
wallet process instead of returning typed errors.

| Severity | Location | Canonical Finding | Required Action |
| --- | --- | --- | --- |
| `S1` | `services/wallet_service.rs` | Backup export and crypto-adjacent flows use `expect()` for serialization, compression, key derivation, encryption, conversions, and asset construction. | Replace with `WalletResult<T>` and `?`, preserving underlying context in `WalletError`. |
| `S1` | `services/chain_service.rs` | Async network calls use `.await.unwrap()`, allowing timeouts or remote failures to terminate the process. | Propagate errors with `?` and surface them to the caller. |

📌 The source fusion keeps the original remediation pattern because it carries unique
operational value:

```rust
fn to_wallet_result<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> WalletResult<T> {
    result.map_err(|error| WalletError::CryptoError(format!("{}: {}", context, error)))
}
```

📌 No other report contradicted these findings; the apparent severity difference comes from
scope, not from evidence that the panic paths are safe.

## FUS-06 KDF, Storage, and Backup Contract Findings

### FUS-06.1 Encryption-Path Divergence

📌 The reports converge on a single theme: encryption and KDF behavior is not governed by one
canonical, self-describing contract across wallet encryption, RedB migration, and backup
containers.

| Path | Effective Parameters | Salt Handling | Canonical Issue |
| --- | --- | --- | --- |
| `core/security/encryption.rs` | Argon2id `128 MiB / 3 iter / 6 parallel` | 16-byte salt repeated into 32 bytes | Legacy behavior is weaker and non-standard; must be versioned and migrated |
| `db/redb_wallet_crypto.rs` V2 | Argon2id `128 MiB / 5 iter / 8 parallel` | Zero-padded 16-32 byte salt | Strongest reviewed path; should become the canonical baseline |
| `core/backup/wallet_backup.rs` + backup exporters | Compile-time behavior, with no persisted full Argon2 parameter set | 16-byte salt repetition without explicit format governance | Backup files are not fully self-describing and may become undecryptable across drift |

📌 The zero-pad versus repetition-pad distinction is operationally important and therefore
preserved explicitly:

```rust
// Legacy behavior in WalletEncryption.
salt32[..16].copy_from_slice(salt);
salt32[16..].copy_from_slice(salt);

// Preferred behavior aligned with RedB V2.
let len = salt.len().min(32);
salt32[..len].copy_from_slice(&salt[..len]);
```

### FUS-06.2 Backup Format Self-Description

📌 Backup encryption is currently weaker as a contract than `.wlt` storage.

📌 The fused report preserves these storage-specific findings:

1. Backup containers do not persist the complete Argon2 memory, iteration, and parallelism
   parameters used to derive the backup key.
2. Backup salt normalization is not recorded as a versioned rule.
3. Recovery correctness can therefore depend on build profile or future implementation drift.
4. Plaintext backup metadata currently exposes `wallet_id`, `network`, and `created_at`.

📌 The required direction is to version backup KDF semantics explicitly, reject unknown KDF
versions, and decide whether plaintext metadata is an accepted privacy tradeoff or an
implementation gap.

📌 The existing codebase already provides the preferred implementation route, so this is not a
greenfield redesign:

1. Reuse the persisted `KdfParams` contract from `db/redb_wallet_crypto.rs` as the model for the
   backup KDF record, including `version`, `algorithm`, `salt`, memory cost, time cost,
   parallelism, and explicit salt-padding semantics.
2. Evolve `core/backup/backup_exporter_impl.rs` from its current `BackupEncryptionV1` shape into a
   versioned backup-encryption header that carries a nested KDF description instead of only a salt
   and KDF name.
3. Reuse the already existing compatibility pattern in
   `BackupExporterImpl::resolve_aad_bytes()` so new readers try the newest backup header/AAD
   contract first and then fall back only to explicitly supported legacy variants.
4. Reject unknown backup KDF versions before any expensive derivation, matching the defensive
   `validate_untrusted_persisted()` posture already used by the RedB wallet path.
5. Keep only a minimal public header if backup discoverability is required; otherwise move
   `wallet_id`, `network`, and `created_at` into the encrypted payload and treat them as private
   recovery metadata.

### FUS-06.3 Upgrade and Deterministic-Salt Questions

📌 RedB V1 wallets can still be opened without forced upgrade to the stronger V2 KDF path,
which leaves old wallets on weaker semantics indefinitely.

📌 One report also flagged `compute_seed_salt(wallet_id)` as a possible deterministic-salt risk
if `wallet_id` is derivable from public data. That risk remains conditional until the wallet-id
derivation path is fully audited.

📌 The migration path is also already partially implemented in the workspace and should be
completed rather than reinvented:

1. `db/redb_wallet_store.rs` already calls `migrate_kdf_if_needed(...)` during open/unlock and
   `core/key/key_manager_redb.rs` already exposes `migrate_kdf_v1_to_v2(...)`, which decrypts V1
   records and re-wraps them under V2 semantics.
2. The missing governance step is to make successful unlock of a V1 wallet contingent on writing
   the migrated record back to the `.wlt` container instead of treating migration as a best-effort
   side effect.
3. New wallet writes should stop deriving seed salt from `compute_seed_salt(wallet_id)` and store
   a fresh random 16-byte wallet-owned salt in wallet metadata; the deterministic function should
   remain legacy-read fallback only until old wallets are migrated.
4. This turns the deterministic-salt question from a long-lived ambiguity into a bounded legacy
   compatibility concern.

## FUS-07 Key Derivation, View Keys, and Cache Findings

### FUS-07.1 Canonical View-Key Derivation Must Be Singular

📌 This is the highest-priority protocol finding below the service-layer `S1` issues.

📌 Two functions derive different view-key scalars from the same `ReceiverSecret`:

```rust
derive_view_secret_key(receiver_secret)
derive_view_key_versioned(receiver_secret, version)
```

📌 If sender and scanner call different derivation paths, outputs can be silently classified as
`NotMine`, which is a direct recoverability and fund-detection failure.

📌 The fusion therefore requires one canonical derivation function, immediate call-graph audit,
deprecation of the superseded path, and regression tests that prove sender and scanner remain in
lock-step.

📌 The implementation-ready resolution is:

1. Treat `derive_view_secret_key(receiver_secret)` as the live protocol path because it is the
   simpler unversioned derivation and is already the baseline primitive used by the current
   receiver flow.
2. Restrict `derive_view_key_versioned(receiver_secret, version)` to explicit rotation or
   historical-recovery use sites only, instead of allowing it to appear in sender or scanner hot
   paths.
3. Rename or re-scope the versioned helper so the API itself advertises that it is not the default
   spend/scan derivation path.
4. Audit all sender, scanner, and card-generation call sites against this rule and add a regression
   test that proves both sides derive the same key for the same `ReceiverSecret` in the default
   protocol path.

### FUS-07.2 Gap-Limit and Cache Invariants

📌 The key-manager findings converge on one principle: state invariants should fail loudly, and
cache metadata should be enforced instead of merely recorded.

| Topic | Canonical Risk | Canonical Action |
| --- | --- | --- |
| Gap-limit math | `saturating_sub` can hide an impossible `last_used_plus1 > next_index` invariant break | Use `checked_sub()` and return a state-corruption or gap-limit error |
| Cached public keys | TTL field exists but is not enforced on reads | Treat expired entries as cache misses and re-derive |
| Spot-check ordering | `Relaxed` ordering is probably acceptable but under-documented | Keep the behavior if desired, but document why it is advisory only |

📌 The final fusion keeps the original invariant-preserving sketch because it is the strongest
actionable form found in the sources:

```rust
let gap = next_index
    .checked_sub(last_used_plus1)
    .ok_or(KeyManagerError::StateCorrupted)?;
```

### FUS-07.3 Receiver-Secret Validation and View-Key Convention Notes

📌 `ReceiverSecret::from_bytes()` currently rejects zero bytes but defers some unusable-key
conditions until later derivation. The fused recommendation is to validate at creation and at
load time with a `validate_usable()` helper.

📌 The concrete fix is straightforward and does not need new dependencies:

1. Add `ReceiverSecret::validate_usable(&self) -> Result<(), StealthKeyError>` that runs the same
   canonical derivation path used in production, starting with `derive_view_secret_key(self)`.
2. Call `validate_usable()` from `from_raw()`, `from_encrypted()`, and therefore transitively from
   `from_bytes()` and `load()`.
3. Keep the object-level invariant simple: once a `ReceiverSecret` instance exists, callers should
   be able to assume it can derive a valid live view key.

📌 The audit also preserves the lower-severity governance note that
`VIEW_KEY_ACCOUNT_OFFSET = 100_000` is a convention, not a cryptographic enforcement boundary.
That convention should stay documented as a policy boundary rather than a hard isolation claim.

## FUS-08 Secret Lifecycle, Privacy, and API Hygiene Findings

### FUS-08.1 Secret-Bearing Buffers and Wrappers

📌 The strongest exception to the otherwise solid `Hidden<T>` pattern is the public
`FileKeyStore` API, which stores password material as cloneable `Vec<u8>` bytes and derives
`Clone`, `Debug`, `PartialEq`, and `Eq` on a secret-bearing enum.

📌 The canonical remediation is to move to `SafePassword`, `Hidden<_>`, or another zeroizing
wrapper and to remove convenience traits that do not belong on secret-bearing state.

📌 The codebase-backed implementation path is to align `FileKeyStore` with the already dominant
password-handling pattern used by backup and RedB code:

1. Change `EncryptionScheme::Password(Vec<u8>)` in `core/storage/file_key_store.rs` to
   `EncryptionScheme::Password(SafePassword)`.
2. Update `ReceiverSecret::{to_encrypted, from_encrypted, store, load}` in
   `core/key/stealth_keys.rs` to accept `&SafePassword` instead of raw byte slices.
3. Remove `Clone`, `Debug`, `PartialEq`, and `Eq` from secret-bearing enums or structs that would
   otherwise copy or expose password state implicitly.
4. Keep the conversion boundary at the outer API layer, where plaintext passwords already become
   `SafePassword` in several wallet entrypoints.

### FUS-08.2 Mnemonic and Secret Exposure Boundaries

📌 The storage audit established that mnemonic material repeatedly escapes into ordinary
`String` allocations on some service paths.

📌 The fused position is:

1. The at-rest design is strong.
2. The runtime secret lifetime is weaker than it needs to be.
3. Plain UTF-8 mnemonic strings should be postponed until the last API boundary that truly
   requires them.
4. Temporary plaintext buffers should be explicitly wiped.

📌 The fused report also preserves two related API-hygiene observations:

1. `ReceiverKeys::reveal_receiver_secret()` creates misuse risk by exposing the master secret
   reference.
2. Entropy validation warnings are currently log-only and do not surface through a distinct
   API result type.

📌 The current workspace already suggests the preferred mnemonic hardening path:

1. Keep `SeedPhrase24` or raw seed entropy as the source of truth inside wallet-core flows, rather
   than materializing plain `String` early.
2. Reuse the existing `services/seed_phrase.rs` contract, which already returns `Hidden<String>`,
   only at the outer edge where text must actually be shown or exported.
3. Where an owned text buffer is temporarily unavoidable, wrap it in `Zeroizing<String>` and keep
   its scope as short as possible.
4. Apply the same boundary rule to restore and backup code paths so mnemonic text does not bounce
   through intermediate service structs longer than necessary.

### FUS-08.3 Privacy and Backup Metadata

📌 Plaintext backup metadata is preserved as a real privacy finding, even though it is not a
direct decryption break.

📌 If metadata remains public for UX reasons, the tradeoff must be documented explicitly. If the
wallet claims stronger privacy for backup artifacts, metadata should move behind encryption or a
minimal public header.

📌 The recommended default is a minimal public header plus encrypted private metadata:

1. Keep only the fields required for format dispatch and password-verification routing in the clear.
2. Move user-identifying metadata into the encrypted payload that is already authenticated by the
   backup AAD and checksum flow.
3. If product requirements insist on plaintext discoverability, rename those fields as explicit
   `public_hint` metadata so the privacy tradeoff is visible in both code and documentation.

## FUS-09 Protocol, Documentation, and Performance Hardening

### FUS-09.1 Transaction Digest Framing

📌 `build_tx_package_digest()` currently concatenates multiple variable-length string fields
without separators. The underlying hash primitive is not the issue; the semantic framing is.

📌 The preserved recommendation is to add explicit length prefixes for every variable-length
field:

```rust
let len = |value: &str| (value.len() as u32).to_le_bytes();
hasher.update(len(kind));
hasher.update(kind.as_bytes());
```

📌 This is immediately implementable with helpers that already exist in
`core/hashing.rs`:

```rust
hasher.update(frame_bytes(b"z00z.tx.pkg.digest.v1"));
hasher.update(frame_str(kind));
hasher.update(frame_str(package_type));
hasher.update(frame_bytes(&[version]));
hasher.update(frame_u32_le(chain_id));
hasher.update(frame_str(chain_type));
hasher.update(frame_str(chain_name));
hasher.update(frame_bytes(&tx_json));
```

📌 That route is preferable to a local ad hoc framing helper because it keeps transaction digest
semantics aligned with the wallet's existing one-source-of-truth hashing utilities.

### FUS-09.2 Documentation and Naming Hardening

📌 The following lower-severity but legitimate issues are preserved:

1. `derive_s_out` is re-exported without a facade-level security contract.
2. `generate_identity_keypair()` is public despite comments warning that it should not be used
   for recoverable wallet identity.
3. `RistrettoBridgeDomain` should be checked for versioning consistency with the rest of the
   domain macro usage.

### FUS-09.3 Informational Notes and Feature-Gated Exposure

📌 The fused report keeps these informational items because they carry operational meaning:

1. Sequential proof verification in `ProverImpl` is correct but slower than true batch verify.
2. The legacy Argon2 preset is below the stronger V2 baseline and should remain backward
   compatibility only.
3. Debug and verbose-logging feature gates can expose sensitive material or privacy-sensitive
   metadata if enabled in the wrong environment.
4. A `NonZeroUsize::new(...).unwrap()` on a constant cache size is low risk but still a brittle
   constructor pattern.

## FUS-10 Open Ambiguities and Manual-Review Items

📌 These questions were not safely closed by the source set and remain explicit follow-up items:

1. Whether `wallet_id` is publicly derivable and therefore whether deterministic seed salting is
   a real offline-attack amplifier.
2. Whether ChaCha20 nonce truncation from a 32-byte derivation to 12 bytes exactly matches the
   expected external protocol or circuit contract.
3. Whether `CipherSeedContainer` bytes need an additional versioned wrapper inside wallet-owned
   persistence structures.
4. Whether backup metadata disclosure is an explicit privacy tradeoff or a gap that should be
   closed.
5. Whether any remaining sender path already uses the versioned view-key function.

📌 No unresolved source-to-source contradiction remained after scope normalization. Where two
reports appeared to disagree, the disagreement was resolved as either a scope difference or a
layering difference rather than incompatible facts.

## FUS-11 Consolidated Remediation Plan

📌 The merged remediation order is:

1. Remove `expect()` and `unwrap()` from wallet and chain services.
2. Audit every call site of both view-key derivation functions and choose one canonical path.
3. Migrate `WalletEncryption` away from repetition-padded salt and align it with RedB V2.
4. Make backup KDF semantics fully self-describing and versioned.
5. Force or strongly stage V1-to-V2 wallet re-encryption after successful unlock.
6. Enforce cache TTL and invariant-failing gap-limit math in the key manager.
7. Replace cloneable plaintext password bytes in `FileKeyStore` with a zeroizing wrapper.
8. Tighten mnemonic and receiver-secret exposure boundaries.
9. Add field framing to `build_tx_package_digest()`.
10. Preserve and document the acceptable feature-gated debug surface.

### FUS-11.1 Implementation-Ready Solution Paths

📌 The remaining high-value fixes now have concrete execution paths inside the current workspace:

| Finding | Codebase-backed execution path |
| --- | --- |
| Backup KDF is not self-describing | Copy the RedB `KdfParams` model into the backup header contract, version it explicitly, reject unknown versions before derivation, and keep legacy compatibility in the same staged style as `resolve_aad_bytes()`. |
| V1 wallets remain on weaker semantics | Keep using `migrate_kdf_if_needed()` plus `migrate_kdf_v1_to_v2()`, but make persisted rewrite part of the successful unlock contract instead of a soft upgrade. |
| `FileKeyStore` holds cloneable plaintext password bytes | Replace `Vec<u8>` with `SafePassword`, switch `ReceiverSecret` persistence helpers to `&SafePassword`, and remove trait derives from secret-bearing state. |
| Mnemonic text lives too long in ordinary strings | Keep `SeedPhrase24` or entropy internally, emit `Hidden<String>` only at the outer service boundary, and use `Zeroizing<String>` for unavoidable temporary text buffers. |
| View-key derivation is ambiguous | Designate `derive_view_secret_key()` as canonical live path, confine `derive_view_key_versioned()` to rotation/history only, and lock this down with sender/scanner regression tests. |
| Receiver secrets are validated too late | Add `validate_usable()` and call it from `from_raw()` and decrypt/load constructors so invalid secrets never survive object creation. |
| Deterministic seed salt may be too predictable | Persist a fresh random wallet-owned salt for new writes and keep `compute_seed_salt(wallet_id)` as legacy compatibility fallback only. |
| Transaction digest fields are concatenated ambiguously | Replace raw concatenation in `build_tx_package_digest()` with existing `frame_str`, `frame_u32_le`, and `frame_bytes` helpers from `core/hashing.rs`. |

## FUS-12 Consolidated Test Plan

### FUS-12.1 Regression and Positive Tests

📌 The merged regression plan includes:

| Area | Test |
| --- | --- |
| Service errors | Backup export failures and chain-service failures return typed errors instead of panicking |
| KDF migration | Legacy wallet decrypt still succeeds while new writes use zero-pad and stronger params |
| View-key correctness | Unversioned and versioned paths are never mixed in sender/scanner flows |
| Gap-limit invariants | Impossible counter order returns an error instead of silently producing zero gap |
| Cache TTL | Expired cache entries become misses |
| Receiver secret validity | Identity-producing or unusable secrets are rejected at construction/load time |
| Transaction digest framing | `("AB","C")` and `("A","BC")` produce different digests |
| Backup format | Unknown backup KDF version is rejected deterministically |

### FUS-12.2 Negative and Fuzz Tests

📌 The fused negative and fuzz plan preserves these cases:

1. Malformed BIP-44 paths.
2. Malformed encrypted seed containers and tampered AEAD envelopes.
3. Pathological KDF metadata intended to trigger excessive work.
4. Wrong-password receiver-secret decrypt attempts.
5. Backup and wallet files carrying unsupported version markers.

## FUS-13 Confidence and Release Gate

📌 Confidence is highest on the structural crypto claims that were confirmed by multiple
sources: domain separation, Poseidon framing, `.wlt` design quality, and the existence of KDF
fragmentation.

📌 Confidence is medium on questions that require wider call-graph or protocol-surface proof,
especially view-key path usage and whether `wallet_id` is public enough to strengthen the
deterministic-salt concern.

| Gate | Status | Reason |
| --- | --- | --- |
| Source-section coverage | `PASS` | Every H1-H4 section from all three inputs is mapped in the audit artifact |
| Provision coverage | `PASS` | Every extracted provision is mapped to a canonical destination section |
| Duplicate elimination | `PASS` | Repeated verdicts, scope notes, KDF overlap, and positive findings were fused into single canonical statements |
| Unresolved semantic conflicts | `PASS` | No unresolved source-to-source contradiction remained after normalization |
| Doublecheck review | `BLOCKED` | The one-shot external reviewer could not independently read local files in its execution context, so completion could not be established externally |
| Release readiness | `FAIL` | Service panic paths, view-key canonicalization, and KDF governance still block production |

📌 Canonical release stance: `development-ready only after blocker remediation`, `not safe for
production deployment today`, and `fusion completion externally blocked until Doublecheck can
read the local files`.
