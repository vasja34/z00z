# 🔐 Crypto-Architect Audit: `z00z_wallets` Crate

**Model:** Claude Sonnet 4.6  
**Date:** 2025-07-17  
**Scope:** `crates/z00z_wallets/src/**/*.rs` — implementation only; `tari/` vendor excluded  
**Protocol:** Crypto-Architect Skill — Phases 1–7

---

## ⭐ Executive Verdict

> **Risky but Salvageable.** No S0/S1 critical breaks found. The codebase demonstrates
> above-average cryptographic hygiene for a production wallet (correct primitives, systematic
> domain separation, ZeroizeOnDrop everywhere, constant-time comparisons, hedged ephemeral keys).
> Three **S2 issues** must be resolved before mainnet: duplicate salt padding that survives in
> `WalletEncryption` (not yet migrated to zero-pad V2), dual view-key derivation functions with
> different domains creating scanner ambiguity, and un-forced KDF parameter upgrade for legacy V1
> wallets. All three have documented mitigations and existing V2 counterparts in the codebase.

---

## 📌 Phase 1 — Input Classification

| Attribute | Value |
|-----------|-------|
| Curve | Ristretto255 (cofactor-safe, via `curve25519-dalek` / Tari RistrettoPoint) |
| AEAD | XChaCha20-Poly1305 (`z00z_crypto::aead`, 192-bit nonce) |
| KDF | Argon2id → HKDF-SHA256 (wallet encryption); standalone Argon2id (stealth receiver secret, redb) |
| Signatures | RistrettoSchnorr, random nonce per-signature, Z00Z domain challenge |
| Range Proofs | Bulletproofs+ (`BulletproofsPlusService`), `RANGE_PROOF_BITS_V1 = 64` |
| HD Keys | BIP-32/BIP-44 (coin type 1337, hardened accounts) → Ristretto bridge |
| ZK hash | Poseidon2-Goldilocks (width=8, rate=7), length-framed absorption |
| Wallet hash | BLAKE2b-512, truncated to 32 bytes, domain+length-framed |
| RNG | `SystemRngProvider` wrapping `rand::SystemRandom`; `MockRngProvider` for tests |
| Trust boundary | `z00z_crypto/tari/` is read-only vendor; all custom code above it |

---

## 🎯 Phase 2 — Scope & Threat Model

### 🔑 Assets under protection

1. **BIP-39 seed** (`SeedPhrase24`) — wallet root secret; loss = total loss
2. **ReceiverSecret** (`[u8; 32]`) — stealth scan key; compromise = privacy break (receiver linkability)
3. **Wallet encryption key** (derived via Argon2id from user password) — at-rest encryption
4. **Ephemeral scalar `r`** — per-output stealth key; reuse = output linkability
5. **Transaction blinding scalar** — Pedersen commitment; leak = amount revelation
6. **Schnorr signing key** — authorization; compromise = fund theft

### 🚨 Attacker models considered

| Model | Description |
|-------|-------------|
| AT-1 | Offline attacker with stolen `.wlt` file |
| AT-2 | Passive network observer (blockchain + P2P) |
| AT-3 | Active relay / output-level adversary |
| AT-4 | Malicious `.wlt` file (pathological KDF params) |
| AT-5 | Compromised `SystemRngProvider` (weak RNG) |
| AT-6 | Protocol-level downgrade via V1 wallet open |

---

## ⚙️ Phase 3 — Construction Selection Review

### Ristretto255

✅ **Correct choice.** Cofactor-1 point arithmetic eliminates subgroup malleability.
Active use of `from_canonical_bytes` + identity-point rejection guards all ECDH inputs.

### XChaCha20-Poly1305

✅ **Correct for at-rest encryption.** 192-bit nonce eliminates nonce-reuse risk for randomized encryption. Tag-before-release pattern enforced by `aead::seal`. AAD policy documented in `redb_wallet_crypto.rs` with explicit length framing for variable fields.

### Argon2id

✅ **Correct choice for PBKDF.** Production params in redb V2 path:
128 MiB / 5 iter / 8 parallelism — meets OWASP 2023 recommendations.
⚠️ See F1 and F5 for parameter divergence in legacy paths.

### Bulletproofs+ with `RANGE_PROOF_BITS_V1 = 64`

✅ **Confirmed correct.** 64-bit range covers the full u64 value space `[0, 2^64 - 1]`.
`MIN_VALUE_PROMISE = 0` (no lower bound constraint).
`AGGREGATION_FACTOR = 1` (no batch aggregation; safe but leaves performance on the table).

### Poseidon2-Goldilocks for ZK hashing

✅ **Appropriate for ZK-circuit compatibility** (Goldilocks field, ~10-50 constraints/hash).
**Input framing is correct** (see analysis below — F2 initially feared, definitively resolved).

### BIP-32/44 → Ristretto bridge

✅ **Pragmatic but non-standard.** BIP-32 operates on secp256k1, requiring a one-way bridge
via `RistrettoBridgeDomain` to produce Ristretto scalars. Domain separation prevents cross-protocol
reuse. Non-hardened `change`/`index` is intentional for HW-wallet interop and is documented.

---

## 💥 Phase 4 — Composition Review

### 4.1 Wallet Encryption (encryption.rs) — **TWO KDF PATHS**

```rust
// Path A: WalletEncryption (encryption.rs)
// 128 MiB / 3 iter / 6 parallelism — Argon2id
// Salt: 16 bytes, DOUBLED by repetition → salt32[0..16] = salt; salt32[16..] = salt
let salt32 = expand_salt_v1(salt);  // repetition padding — S2 ISSUE

// Path B: redb_wallet_crypto.rs (V2)
// 128 MiB / 5 iter / 8 parallelism — Argon2id
// Salt: 16–32 bytes, ZERO-PADDED to 32 → standard behavior
let salt32 = pad_salt32_zero(params.salt.as_slice());  // CORRECT
```

The V2 migration in `redb_wallet_crypto.rs` is complete and correct. However, `encryption.rs`
(`WalletEncryption`) still uses the legacy repetition-pad pattern. These are separate code paths
serving different wallet layers — both must be migrated.

### 4.2 hash_zk Input Framing — **CONFIRMED SAFE**

```rust
pub fn poseidon2_hash(domain: &[u8], data: &[&[u8]]) -> [u8; 32] {
    packer.push_frame_bytes(domain);   // length-prefixed domain
    packer.push_u64_le(data.len());    // ITEM COUNT as u64 LE
    for item in data {
        packer.push_frame_bytes(item); // length-prefixed per item
    }
    // ...
}
```

`push_frame_bytes` prepends a 4-byte LE length before each chunk. Additionally, `data.len()`
is encoded as a u64. This means:

- `derive_k_dh(dh)` → chunks[2]: `""`, `dh` — count=2 encoded
- `derive_k_dh_with_req(dh, req_id)` → chunks[3]: `""`, `dh`, `req_id` — count=3 encoded

**No concatenation ambiguity possible.** F2 (initially suspected shared-domain collision) is
**NOT a vulnerability**. The item count + length framing per item makes all distinct input
tuples structurally distinguishable.

### 4.3 Stealth ECDH Composition

```text
r ← SystemRng + sender_salt + tx_digest + out_index   (hedged)
R_pub = r·G
dh_sender  = r · view_pk    (Ristretto255, cofactor-safe)
dh_receiver = view_sk · R_pub
k_dh = H_poseidon2<KdhDomain>("", [dh])
s_out = H_poseidon2<SOutProdDomain>("Z00Z/S_OUT", [k_dh, r_pub, serial_le])
asset_id = H_poseidon2<AssetIdDomain>("", [s_out])
owner_tag = H_poseidon2<OwnerTagDomain>([owner_handle, k_dh])
```

✅ Full domain separation at each step. Zero-scalar/identity-point rejection at ECDH level.
All inputs to `s_out` are bound: k_dh (ECDH), r_pub (uniqueness across outputs to same recipient),
serial_le (uniqueness within packet).

### 4.4 Dual View Key Functions — **PROTOCOL AMBIGUITY**

Two different functions derive a view key scalar from the same `ReceiverSecret`:

```rust
// UNVERSIONED (ViewKeyDomain)
pub fn derive_view_secret_key(receiver_secret: &ReceiverSecret) -> Z00ZScalar {
    hash_to_scalar_zk::<ViewKeyDomain>("", &[receiver_secret.as_bytes()])
}

// VERSIONED (WalletViewKeyHashProdDomain)
pub fn derive_view_key_versioned(receiver_secret, version: u32) -> Z00ZScalar {
    hash_to_scalar_zk::<WalletViewKeyHashProdDomain>("VIEW_V", &[receiver_secret.as_bytes(), &version.to_le_bytes()])
}
```

`verify_owner_two_factor` (output.rs) calls `derive_view_secret_key` (unversioned). The
spender-side ECDH receiver path also calls `derive_view_secret_key`. The versioned path
appears to be an intended upgrade path, but if any **sender** encodes against the versioned
view key while the **scanner uses the unversioned key**, outputs will be permanently undetectable
(silent fund loss). This must be resolved with a single canonical function.

---

## 🔑 Phase 5 — ZK Circuit Review

### Poseidon2 field arithmetic

The implementation uses Goldilocks field `p = 2^64 - 2^32 + 1`. The `WordPacker` encodes bytes
as LE u64 values and calls `Goldilocks::new(val)`. Values where `val ≥ p` are legal — the
Goldilocks implementation reduces them modulo p during arithmetic. No panic risk for inputs with
`val > p` since addition overflows into reduction. **No bias introduced** for the 32-byte output
(4 × u64 LE from state).

### hash_to_scalar_zk — Rejection Sampling

```rust
// Uses counter-based Fiat-Shamir: H(base || ctr=0) || H(base || ctr=1) → 64 bytes
// → Z00ZScalar::from_uniform_bytes (wide reduction)
```

✅ Wide reduction from 512 bits eliminates bias for Ristretto255 (252-bit order group).
Zero-scalar rejection at call sites (stealth_keys.rs, ephemeral.rs). Correct.

---

## 🐞 Phase 6 — Implementation Review (Findings)

---

### F1 — S2 ♨️ — Salt Repetition in `WalletEncryption` Not Yet Migrated

**Location:** `src/core/security/encryption.rs`

**Code:**
```rust
let mut salt32 = [0u8; 32];
salt32[..16].copy_from_slice(salt);
salt32[16..].copy_from_slice(salt);   // ← both halves identical
let intermediate_key = derive_key_argon2id_32(password.reveal(), &salt32, &params)?;
```

**Impact (AT-1):** For a 16-byte salt `s`, the Argon2id input salt is `[s || s]`. Any 16-byte
collision in `s` also collides in `[s || s]`. Effective uniqueness is 128 bits — meets the NIST
SP800-132 minimum of exactly 128 bits but non-standard. A dictionary attacker with a known wallet
can exploit this since preprocessing over 128-bit space covers the same workload as a proper
32-byte salt.

**Comparison:** `redb_wallet_crypto.rs` V2 path (`pad_salt32_zero`) correctly zero-pads. The
fix is one line.

**Fix:**
```rust
// Replace repetition with zero-pad (mirrors redb V2 behavior):
let mut salt32 = [0u8; 32];
let len = salt.len().min(32);
salt32[..len].copy_from_slice(&salt[..len]);
// salt32[len..] already zeroed — do NOT copy salt again
```

**Test requirement:** Existing wallets encrypted with the old padding must continue to decrypt
(add version byte to `EncryptedWalletContainer.algorithm` field or introduce a new enum value
before changing the derivation).

---

### F2 — S2 🔔 — Dual View Key Functions: Scanner/Spender Key Mismatch Risk

**Location:** `src/core/key/stealth_keys.rs`, `src/core/stealth/output.rs`

**Code:**
```rust
// stealth_keys.rs — called by receiver ECDH paths
pub fn derive_view_secret_key(rs: &ReceiverSecret) -> Result<Z00ZScalar, _> {
    hash_to_scalar_zk::<ViewKeyDomain>("", &[rs.as_bytes()])
}

// stealth_keys.rs — upgrade path (versioned)
pub fn derive_view_key_versioned(rs: &ReceiverSecret, version: u32) -> Result<Z00ZScalar, _> {
    hash_to_scalar_zk::<WalletViewKeyHashProdDomain>("VIEW_V", &[rs.as_bytes(), &version.to_le_bytes()])
}
```

The unversioned and versioned functions produce **different scalar values** for the same
`ReceiverSecret`. If outputs are sent to `derive_view_key_versioned(version=0)` but scanned
via `derive_view_secret_key`, ECDH will fail silently — outputs appear as `NotMine`. This is a
**silent fund loss** risk if call sites diverge.

**Current state:** The scan path (`verify_owner_two_factor`, `verify_owner_tag_with_req`) uses
`derive_view_secret_key`. The versioned path exists without confirmed usage in the sender's
`ReceiverCard` construction. This must be traced and resolved.

**Required action:**
1. Audit all call sites: `grep -rn "derive_view_key_versioned\|derive_view_secret_key" crates/`
2. Designate ONE canonical function. If `derive_view_key_versioned` is the intended future,
   deprecate `derive_view_secret_key` and migrate all call sites.
3. Add `#[deprecated]` to the superseded function immediately.

---

### F3 — S2 ⚠️ — V1 Wallets: No Forced Re-encryption on Upgrade

**Location:** `src/db/redb_wallet_crypto.rs`

**Code:**
```rust
let out = match params.version {
    KdfParams::VERSION_V1 => derive_key_v1_repetition_padding(password, params, &argon2_params)?,
    KdfParams::VERSION_V2 => derive_key_v2_zero_padding(password, params, &argon2_params)?,
    _ => return Err(...)
};
```

The application will open V1 wallets using the repetition-padding path indefinitely. There is
no forced upgrade on successful unlock. AT-1 with a V1 wallet file has a weaker salt than
intended by V2.

**Fix:** On successful unlock of a V1 wallet, re-derive and re-encrypt the master key record
using V2 params (transparent migration). Document the migration in the release notes.

```rust
// After successful unlock with VERSION_V1:
if params.version == KdfParams::VERSION_V1 {
    let v2_params = KdfParams::enhanced_argon2id_with_salt(generate_random_salt_32());
    store.upgrade_kdf_params(v2_params, unlocked_key)?;  // requires transaction
}
```

---

### F4 — S3 — Deterministic Argon2id Salt from `wallet_id`

**Location:** `src/core/hashing.rs → compute_seed_salt()`

**Code:**
```rust
pub fn compute_seed_salt(wallet_id: &WalletId) -> [u8; 16] {
    let hash = domain_hash::<SeedSaltDomain>(wallet_id.as_bytes());
    hash[..16].try_into().unwrap()
}
```

If `wallet_id` is a public identifier (e.g., derived from a public key or stored unencrypted),
an attacker with the wallet file and wallet_id can target the Argon2id computation with a
known salt, enabling offline dictionary attacks.

**Note:** Random salts are preferred. If `wallet_id` derives from the master public key, this
represents a salt-predictability oracle for AT-1.

**Fix:** Generate a cryptographically random 16-or-32-byte salt at wallet creation time and persist it separately from wallet_id.

---

### F5 — S3 — KDF Parameter Divergence Between Encryption Paths

**Location:** `src/core/security/encryption.rs` vs `src/db/redb_wallet_crypto.rs`

| Path | Memory | Iterations | Parallelism |
|------|--------|------------|-------------|
| `WalletEncryption` (encryption.rs) | 128 MiB | **3** | **6** |
| `redb_wallet_crypto` V2 (redb path) | 128 MiB | **5** | **8** |

Two different production KDF parameter sets exist in the same application. The `encryption.rs`
path provides less resistance to AT-1 (offline attack). Should be aligned to the stronger V2 params.

---

### F6 — S3 — `build_tx_package_digest` Missing Field Separators

**Location:** `src/core/tx/tx_verifier.rs → build_tx_package_digest()`

```rust
let mut hasher = Hasher::new();
hasher.update(b"z00z.tx.pkg.digest.v1.");
hasher.update(kind.as_bytes());           // variable-length, no separator
hasher.update(package_type.as_bytes());   // variable-length, no separator
hasher.update(&[version]);
hasher.update(&chain_id.to_le_bytes());   // fixed 4 bytes ✓
hasher.update(chain_type.as_bytes());     // variable-length, no separator
hasher.update(chain_name.as_bytes());     // variable-length, no separator
hasher.update(&tx_json);
```

The string fields `kind`, `package_type`, `chain_type`, `chain_name` are concatenated without
length prefixes. While these values are currently protocol-defined constants, the design
allows `kind="TxPackageregular_tx"` + `package_type=""` to produce the same hash as
`kind="TxPackage"` + `package_type="regular_tx"`.

**Note:** BLAKE3's streaming API is used correctly (no length-extension attack). The
vulnerability is only at the semantic level of field boundary conflation.

**Fix:** Add 4-byte LE length prefixes for all variable-length string fields:
```rust
let len_bytes = |s: &str| (s.len() as u32).to_le_bytes();
hasher.update(len_bytes(kind));
hasher.update(kind.as_bytes());
// ... etc.
```

---

### F7 — S3 — `RistrettoBridgeDomain` Macro Inconsistency

**Location:** `src/core/domains.rs`

```rust
// All other domains use version parameter:
hash_domain!(WalletTag16HashProdDomain, "z00z.wallet.tag16.v1", 1);

// RistrettoBridgeDomain does NOT:
hash_domain!(RistrettoBridgeDomain, "z00z.wallet.bip32_to_ristretto.v1");
```

Requires verification that the `hash_domain!` macro provides a safe default for the missing
version argument, or that this is an intentional single-version domain with no future migration
concern. Cosmetic if the macro default is semantically equivalent.

---

### F8 — INFO — Sequential Batch Verification in `ProverImpl`

**Location:** `src/core/tx/prover.rs`

```rust
pub fn verify_batch_proofs(&self, proofs: &[...]) -> Result<_, _> {
    for proof in proofs {
        self.verify_proof(proof)?;   // sequential
    }
    Ok(())
}
```

True Bulletproofs+ batch verification uses multi-exponentiation for ~30% speedup per additional
proof in the batch and also fails atomically. The current loop fails after the first invalid
proof. **Security-correct** but suboptimal for throughput.

---

### F9 — INFO — LEGACY Argon2id Preset Below OWASP 2023

**Location:** `src/core/key/seed.rs`

`Argon2idParamsV1::LEGACY` = 64 MiB / 3 iter / 4 parallelism.
OWASP 2023 minimum: 128 MiB. Gated as "backward compat only" — use of this preset should
be prohibited for new wallet creation.

---

## ✅ Phase 7 — Positive Findings

| # | Component | Finding |
|---|-----------|---------|
| P1 | `hash_zk` / `poseidon2_hash` | **Proper length framing**: count + per-item LE-length prefix. No concatenation ambiguity possible across all `hash_zk` call sites. |
| P2 | `stealth/ecdh.rs` | ECDH zero-scalar + identity-point double rejection. Both `compute_dh_sender` and `compute_dh_receiver` verify outputs. |
| P3 | `stealth/ephemeral.rs` | Hedged ephemeral scalar: `H(rng_bytes || sender_salt || tx_digest || out_index)`. RNG failure degrades gracefully to determinism but not to zero. Retry mechanism with `RetryDigestDomain` produces independent scalars per retry index. |
| P4 | `RANGE_PROOF_BITS_V1 = 64` | Confirmed full u64 coverage. No amount overflow possible in range proof statement. |
| P5 | `redb_wallet_crypto.rs` V2 | Zero-pad salt migration already implemented. V2 is the default for all new wallets. |
| P6 | `password.rs` | Bloom filter denylist + entropy + pattern + keyboard walk + leet-normalization. 14-char minimum with strength guidance. Above-average for wallet UX. |
| P7 | All secret types | Consistent `Hidden<T>` + `ZeroizeOnDrop` + `subtle::ConstantTimeEq`. Secrets never hit `Debug` output. |
| P8 | `redb_wallet_crypto.rs` | DoS protection validated: `validate_untrusted_persisted()` enforces hard caps on KDF params before any computation (AT-4 defended). |
| P9 | Domain separation | 40+ named domains in `domains.rs`, all `v1`/`v2` versioned, `hash_domain!()` macro. No domain reuse detected across distinct protocol roles. |
| P10 | `facade_kdf.rs` nonce | AEAD nonce `derive_nonce(leaf_ad, r_pub, asset_id, serial_id)` — fully deterministic from public leaf fields, prevents nonce reuse even in stateless contexts. |

---

## ❓ Open Ambiguities

| ID | Question | Risk if unresolved |
|----|----------|--------------------|
| OA-1 | Which view key function is canonical (ViewKeyDomain vs WalletViewKeyHashProdDomain)? Needs full call-graph audit. | Silent scan failure = fund loss |
| OA-2 | Is `wallet_id` derivable from public on-chain data? If yes, F4 becomes S2. | Targeted offline attack |
| OA-3 | Is `tx_digest_hex` in `TxPackage` covered by any signature in the signing flow? If not, digest substitution attack is possible. | Transaction malleability |
| OA-4 | What is the wallet unlock flow for upgrading V1 → V2 KDF? Is there any UI/UX warning? | Users remain on weaker V1 indefinitely |

---

## 🛑 Test Plan

### Required new tests

| Test | File | Coverage |
|------|------|----------|
| `test_wallet_encrypt_zero_pad_v2` | `encryption.rs` tests | Verify `WalletEncryption` with V2 salt produces different ciphertext from V1 but correct decrypt |
| `test_view_key_unversioned_vs_versioned_differ` | `stealth_keys.rs` tests | Assert `derive_view_secret_key` ≠ `derive_view_key_versioned(version=0)` for same ReceiverSecret |
| `test_v1_wallet_upgrade_on_unlock` | `redb_wallet_crypto.rs` tests | Open V1 wallet, verify KDF params upgrade to V2 on close |
| `test_tx_digest_field_boundaries` | `tx_verifier.rs` tests | Assert `build_tx_package_digest("AB","C",...) ≠ build_tx_package_digest("A","BC",...)` |
| `test_kdf_params_alignment` | `encryption.rs` / `redb_wallet_crypto.rs` | Assert production Argon2 parameters ≥ OWASP 2023 in both paths |

---

## 🎯 Confidence Levels

| Section | Confidence | Basis |
|---------|-----------|-------|
| hash_zk framing safety | **99%** | Source code fully read; poseidon2_hash definitively length-framing |
| ECDH correctness | **97%** | Full ecdh.rs read; standard Ristretto DH with proper rejection |
| Range proof coverage | **99%** | RANGE_PROOF_BITS_V1 = 64 confirmed in types.rs |
| Salt padding analysis | **95%** | encryption.rs and redb V1/V2 both fully read |
| View key dual-path risk | **90%** | Call sites partially traced; output.rs uses unversioned, full graph needs audit |
| Deterministic salt (F4) | **85%** | wallet_id derivation path not fully traced |
| Tx digest field boundary | **90%** | build_tx_package_digest fully read |
| Password entropy | **95%** | Full password.rs read |
| Key manager LRU safety | **88%** | Structure read; spot-check mechanism and TTL observed |

---

## 🚩 Final Decision

**Execution-ready with conditions:**

1. ✅ **Before any testnet with real funds**: Fix F1 (salt repetition in `WalletEncryption`)
2. ✅ **Before any testnet**: Resolve OA-1 (canonical view key) — potential silent fund loss
3. ✅ **Before mainnet**: Fix F3 (forced V1→V2 upgrade on unlock)
4. ⚠️ **Recommended before mainnet**: Fix F6 (tx digest field separators), F5 (KDF alignment)
5. 📌 **Backlog**: F8 (batch verify performance), F9 (LEGACY preset guard)

The core cryptographic construction (Ristretto255 + Bulletproofs+ + Poseidon2 + Argon2id) is
sound. No fundamental redesign required.

---

## 📋 Appendix: Files Read

| File | Status |
|------|--------|
| `src/core/key/bip32.rs` | Full (600+ lines) |
| `src/core/key/seed.rs` | Partial (200 lines) |
| `src/core/key/key_manager.rs` | Partial (200 lines) |
| `src/core/key/stealth_keys.rs` | Full (300 lines) |
| `src/core/security/encryption.rs` | Full (300 lines) |
| `src/core/security/password.rs` | Full (200 lines) |
| `src/core/stealth/ecdh.rs` | Full |
| `src/core/stealth/ephemeral.rs` | Full (300 lines) |
| `src/core/stealth/facade_kdf.rs` | Full |
| `src/core/stealth/tag.rs` | Full (250 lines) |
| `src/core/stealth/output.rs` | Full (200 lines) |
| `src/core/tx/prover.rs` | Full (155 lines) |
| `src/core/tx/signer.rs` | Partial (200 lines) |
| `src/core/tx/tx_verifier.rs` | Partial (300 lines) |
| `src/core/tx/builder.rs` | Full (200 lines) |
| `src/core/hashing.rs` | Partial (200 lines) |
| `src/core/domains.rs` | Full (200 lines) |
| `src/db/redb_wallet_crypto.rs` | Full (500 lines) |
| `crates/z00z_crypto/src/hash.rs` (poseidon2_hash) | Critical section |
| `crates/z00z_crypto/src/types.rs` (constants) | Key constants |

**NOT read** (outside scope or time-bounded): `stealth_scanner.rs`, `stealth_trust.rs`,
`leaf_scan.rs`, `session.rs`, `wallet.rs`, `key_service.rs`, `backup_service.rs`,
`seed_phrase.rs`, full `signer.rs` (multi_sign), `tx_id.rs`, `spending.rs`, `witness_gate.rs`.
These are recommended for a follow-up Phase 2 audit.
