# 🔐 Z00Z Crypto Audit — Deep Review of `z00z_crypto` Crate

**Auditor:** GitHub Copilot (Claude Sonnet 4.6)  
**Scope:** `crates/z00z_crypto/src/**/*.rs` — excludes `tari/` vendor subtree  
**Date:** 2025-01-09  
**Branch:** `z00z-simul`

---

## 📌 Executive Verdict

> **BLOCKED for production.** The `claim` subsystem presents **two S0 (critical) vulnerabilities** that invalidate the genesis-claim security model entirely. The ECDH / stealth-address layer has one S1 (high) privacy gap (no view tags). Remaining items (S2–S4) are implementation hygiene issues. Non-claim cryptography (AEAD, KDF, range proofs, commitments) is **structurally sound** pending the S2 fixes listed below.

---

## 🎯 Phase 0 — Input Classification

| Property | Value |
|----------|-------|
| **Artifact type** | Mixed: protocol design + implementation |
| **Primary language** | Rust 2021 edition |
| **Crypto backend** | Tari Crypto — Ristretto255, Bulletproofs+ |
| **AEAD** | XChaCha20-Poly1305 (RustCrypto `chacha20poly1305`) |
| **KDF** | Argon2id (`argon2` crate) + HKDF-HMAC-SHA256 (`hkdf` crate) |
| **Hash** | Blake2b-256/512 (`blake2`), SHA-256 (`sha2`), HMAC-SHA256 (`hmac`) |
| **Secret handling** | `zeroize` + `subtle`, `Hidden<T>` wrapper |
| **RNG** | `z00z_utils::rng::SystemRngProvider` / OS entropy (`getrandom`) |
| **Files audited** | `aead.rs`, `backend.rs`, `backend_tari.rs`, `claim/`, `commitments.rs`, `domains.rs`, `ecdh.rs`, `ecdh_stealth.rs`, `error.rs`, `hash.rs`, `hidden.rs`, `kdf.rs`, `kdf_domains.rs`, `lib.rs`, `range_proofs.rs`, `secret.rs`, `types.rs`, `validation.rs`, `zkpack.rs` |
| **Total non-vendor Rust lines** | ~4 500 (est.) |

---

## ☢️ Phase 1 — Threat Model

### 🔑 Security Goals (from code + spec commentary)

1. **Amount hiding** — Pedersen commitments keep transaction amounts private.
2. **Range validity** — Bulletproofs+ prove amounts are non-negative without revealing them.
3. **Stealth addressing** — Senders derive one-time recipient addresses via ECDH; only the recipient can identify outputs destined for them.
4. **Authenticated encryption** — XChaCha20-Poly1305 protects transaction payloads against tampering and eavesdropping.
5. **Claim integrity** — Genesis claims bind a recipient to their initial allocation in a cryptographically unforgeable manner.
6. **Key lifecycle security** — Blinding factors, view keys, and ephemeral keys are generated securely and zeroised when no longer needed.

### 👁️‍🗨️ Adversary Model

| Adversary | Capability |
|-----------|-----------|
| **Network adversary** | Passive / active MitM on P2P layer |
| **Malicious full node** | Sees all transaction data, can inject / replay |
| **Forger** | Attempts to create valid claims or proofs without witness |
| **Scanner** | Passively monitors outputs to link recipients |
| **DoS attacker** | Supplies crafted large or malformed proofs/AADs |

---

## 💥 Phase 2 — Critical Findings (S0/S1)

### 🚨 F-01 — CRITICAL: Genesis Claim Proof Is a Hash, Not a ZK Proof

**File:** [crates/z00z_crypto/src/claim/prover.rs](crates/z00z_crypto/src/claim/prover.rs), [crates/z00z_crypto/src/claim/verifier.rs](crates/z00z_crypto/src/claim/verifier.rs)  
**Severity:** S0 (Critical)  
**CVSS-like:** Integrity=Complete, Authentication=None  

**Problem:**  
`prove_genesis_claim` ignores the witness entirely and returns:
```rust
// GenesisClaimWitness is an empty struct — literally:
pub struct GenesisClaimWitness;

// prove_genesis_claim body:
let proof_bytes =
    PROOF_TAG || DomainHasher256::<ClaimProofDomain>::new_with_label("proof")
        .chain(stmt_hash)
        .finalize();
```
This is a deterministic hash of the statement. **Any party** who knows the `ClaimProofDomain` string and the statement can produce an identical "proof" with zero knowledge of any witness. The proof carries no binding to a secret possessed exclusively by the legitimate claimant.

`verify_genesis_claim` reconstructs the same hash and performs a plain equality check — confirming the system is a self-consistent hash-equality circuit, not a ZK proof system.

**Impact:** The genesis claim security guarantee is broken. An attacker who observes a valid claim can re-create an identical proof for any statement. There is no cryptographic binding of the claim to a unique claimant secret.

**Fix:** Replace with a genuine ZK proof or at minimum a Schnorr signature:
- **Short-term:** Schnorr proof-of-knowledge over a secret seed (`GenesisClaimWitness` must hold a `Z00ZScalar` secret, and proof must demonstrate knowledge of it against a public commitment in the statement).
- **Long-term:** Integrate a ZK proof circuit (e.g., Bulletproof+-based DLEQ) that binds the recipient blinding factor to the genesis root.

---

### 🚨 F-02 — CRITICAL: ClaimAuthoritySig Is Not a Signature

**File:** [crates/z00z_crypto/src/claim/proof.rs](crates/z00z_crypto/src/claim/proof.rs) (`ClaimAuthoritySig::from_statement`)  
**Severity:** S0 (Critical)  
**CVSS-like:** Integrity=Complete, Authentication=None  

**Problem:**
```rust
pub fn from_statement(stmt_hash: &[u8; 32]) -> Self {
    let bytes = DomainHasher256::<ClaimSigDomain>::new_with_label("sig")
        .chain(stmt_hash)
        .finalize();
    Self { bytes: bytes.into_array() }
}
```
The "authority signature" is a deterministic, public hash of the statement hash with a public domain label. There is **no private key material** anywhere in this derivation. Any party with access to the statement can recompute an identical `ClaimAuthoritySig`.

**Impact:** The authority signature provides no authentication. An adversary can forge a valid `ClaimAuthoritySig` for any statement they construct.

**Fix:** Replace with a genuine asymmetric signature:
- Use `RistrettoSchnorr` (Tari) or equivalent to sign `stmt_hash` with an authority keypair.
- Authority `PublicKey` must be embedded in the verifier and validated at verification time.
- **Minimum:** `ClaimAuthoritySig { r_pub: [u8;32], s: [u8;32] }` — Schnorr signature pair.

---

### ⚠️ F-03 — HIGH: Non-Constant-Time Comparison in Claim Verifier

**File:** [crates/z00z_crypto/src/claim/verifier.rs](crates/z00z_crypto/src/claim/verifier.rs)  
**Severity:** S1 (High)  
**CWE:** CWE-208 (Observable Timing Discrepancy)  

**Problem:**
```rust
// verify_genesis_claim (approx. L55)
if got != want {
    return Err(ClaimProofErr::DigestMismatch);
}

// verify_claim_authority_sig (approx. L90)
if got_sig != expected_sig {
    return Err(ClaimProofErr::SigMismatch);
}
```
Slice inequality via `!=` is Rust's default comparison which is **not constant-time**. It short-circuits on the first differing byte. An adversary on the same system (or via remote timing) can determine how many leading bytes of their forged proof match the expected value, enabling prefix-guided forgery attempts.

Note: This is a secondary issue; F-01 and F-02 make this comparison moot in terms of security, but fixing the CT issue is still required before deploying any future real proof/signature check.

**Fix:**
```rust
use subtle::ConstantTimeEq;
if got.ct_eq(&want).unwrap_u8() == 0 {
    return Err(ClaimProofErr::DigestMismatch);
}
```

---

### ⚠️ F-04 — HIGH: No View Tag in Stealth Address Scanning

**File:** [crates/z00z_crypto/src/ecdh.rs](crates/z00z_crypto/src/ecdh.rs), [crates/z00z_crypto/src/ecdh_stealth.rs](crates/z00z_crypto/src/ecdh_stealth.rs)  
**Severity:** S1 (High)  
**Protocol Gap:** Privacy (sender-unlinkability) + performance  

**Problem:**  
`scan_asset_leaf_owner` performs a 32-byte constant-time comparison of `owner_handle` values from each output leaf. There is **no short view tag** to enable fast filtering before the expensive full ECDH recovery.

Per protocol best-practice (EIP-5564, Monero view tags): recipients should be able to reject ~`255/256` of outputs in O(1) using a single-byte probabilistic tag, then perform full ECDH only for the ~1/256 survivors.

Without a view tag:
- Every wallet scan requires a full Ristretto point multiplication per output leaf.
- On a chain with 10M outputs and 50M leaf scans, this is ~50M scalar multiplications.

Additionally, `derive_pack_key(k_dh)` flows through `kdf_consensus("asset_pack/key", k_dh)` without producing any tag material — meaning the existing `Tag16Domain` declared in `domains.rs` is declared but not wired up anywhere in the implementation.

**Fix:**
1. In `compute_stealth_dh_sender` / `recover_stealth_dh_receiver`, compute a `Tag16` value:
   ```rust
   pub fn compute_view_tag(k_dh: &[u8; 32]) -> u8 {
       hash_zk::<Tag16Domain>("", &[k_dh])[0]
   }
   ```
2. Store tag alongside `owner_tag` in the leaf associated data (via `derive_leaf_ad`).
3. In `scan_asset_leaf_owner`, filter on tag byte mismatch before computing owner_handle.

---

## ⚙️ Phase 3 — Medium Findings (S2)

### F-05 — MEDIUM: Range Proof Lacks Asset-ID / Chain Binding

**File:** [crates/z00z_crypto/src/range_proofs.rs](crates/z00z_crypto/src/range_proofs.rs)  
**Severity:** S2 (Medium)  

**Problem:**  
`AssetRangeProof::new(value, blinding)` creates a proof over `(value, blinding)` only. The proof does not include `asset_id`, `chain_id`, or output commitment context in its Bulletproof++ witness/statement.

In a multi-asset system, a range proof valid for asset A on chain A is valid for the same `(value, blinding)` pair on asset B on chain B. This enables **cross-asset proof replay**: if two outputs share the same blinding factor and value (possible in deterministic test fixtures), their proofs are interchangeable.

**Mitigation scope:** Tari's Bulletproofs+ statement includes a commitment but not a domain-aware context. The protection against cross-asset replay must be enforced at the transaction composition layer (e.g., by including a domain commitment in AAD, or by deriving blinding factors that incorporate asset_id). Ensure this is explicitly documented and enforced at the caller layer rather than relying on the proof system.

**Fix:**  
- Document at `AssetRangeProof::new`'s doc comment that callers MUST derive blinding factors from a seed that includes `asset_id` and `chain_id`.
- Consider adding a `binding_context: Option<&[u8]>` parameter that gets hashed into the minimum_value_promise field or an auxiliary witness entry.

---

### F-06 — MEDIUM: ZkPackEncrypted Stores AEAD Tag Separately

**File:** [crates/z00z_crypto/src/zkpack.rs](crates/z00z_crypto/src/zkpack.rs)  
**Severity:** S2 (Medium)  

**Problem:**
```rust
pub struct ZkPackEncrypted {
    pub version: u8,
    pub ciphertext: Vec<u8>,   // AEAD ciphertext WITHOUT tag
    pub tag: [u8; 16],          // tag stored separately
}
```

Standard AEAD interfaces return `ciphertext || tag` as a single opaque blob. Splitting them:
1. Enables callers to pass `ciphertext` alone to decryption routines that expect `ciphertext || tag` format — silently stripping authentication.
2. Creates a divergence from the canonical `seal()` envelope format used everywhere else in the crate.
3. Is inconsistent with `seal()` which returns `algo_id || nonce || ciphertext_with_tag` as a single `Vec<u8>`.

As of audit time, the encryption logic that populates this struct is not visible; risk depends on callsite discipline.

**Fix:**  
- Merge tag into ciphertext before storage: `inner: Vec<u8>` containing `ciphertext || tag`.  
- Or add assertion/compile-time check ensuring tag is always appended to ciphertext before any crypto opaque boundary.

---

### F-07 — MEDIUM: `hmac_sha256` Silently Returns `[0u8;32]` on Key Error

**File:** [crates/z00z_crypto/src/hash.rs](crates/z00z_crypto/src/hash.rs)  
**Severity:** S2 (Medium)  

**Problem:**
```rust
pub fn hmac_sha256(key: &[u8], domain: &str, label: &str, msg: &[u8]) -> [u8; 32] {
    let mut mac = match HmacSha256::new_from_slice(key) {
        Ok(mac) => mac,
        Err(_) => return [0u8; 32], // ← silent failure
    };
    // ...
}
```
If `new_from_slice` fails (only possible if the HMAC implementation imposes a length constraint; RustCrypto HMAC accepts any length, but this could change), the function silently returns the all-zero value. A caller checking `hmac_sha256(...) == expected` without knowing the key failed would get an inconsistent security guarantee. The all-zero bytes could coincidentally match a crafted tag.

In practice, RustCrypto's `HmacSha256::new_from_slice` accepts any key length, so this code path is unreachable. However, the infallible return type masks the failure mode.

**Fix:**  
- Change signature to `Result<[u8; 32], CryptoError>` and propagate the error.  
- Or, since `new_from_slice` is actually infallible for HMAC-SHA256, replace with `.expect("HMAC accepts any key length")` with a comment justifying the unwrap.

---

### F-08 — MEDIUM: `Z00ZScalar::from_hash` Falls Back to Scalar One

**File:** [crates/z00z_crypto/src/types.rs](crates/z00z_crypto/src/types.rs)  
**Severity:** S2 (Medium)  

**Problem:**
```rust
pub fn from_hash(hash: &[u8; 64]) -> Self {
    match RistrettoSecretKey::from_uniform_bytes(hash) {
        Ok(key) => Self(key),
        Err(_) => Self::one(), // ← fallback to known constant
    }
}
```
`RistrettoSecretKey::from_uniform_bytes` implements wide scalar reduction over 64 bytes, which **always succeeds** for any 64-byte input (the Ristretto group has order ~2^252, and wide reduction modulo a ~252-bit prime cannot fail for a 512-bit input). The `Err(_)` branch is unreachable.

However, if a future compiler/library change makes it reachable, falling back silently to the constant scalar `1` is a **critical security issue**: using blinding factor = 1 reveals amounts via commitment lookup tables.

**Fix:**  
- Replace with `.expect("wide scalar reduction over 64 bytes is infallible")` with a comment.  
- Alternatively change to `Result<Self, CryptoError>` and propagate.

---

## ⭐ Phase 4 — Low / Informational Findings (S3–S4)

### F-09 — LOW: `ct_eq()` Returns `bool`

**File:** [crates/z00z_crypto/src/types.rs](crates/z00z_crypto/src/types.rs), [crates/z00z_crypto/src/kdf.rs](crates/z00z_crypto/src/kdf.rs)  
**Severity:** S3 (Low)  

**Problem:**
```rust
pub fn ct_eq(&self, other: &Self) -> bool {
    use subtle::ConstantTimeEq;
    self.0.as_bytes().ct_eq(other.0.as_bytes()).into()
}
```
The internal `ConstantTimeEq` is correct. However, converting to `bool` before returning means the caller receives a `bool` that, if used in a branch condition, could be subject to compiler branch prediction optimizations. This is a well-known subtle API footgun.

**Fix:** Return `subtle::Choice` rather than `bool`. Let the caller explicitly call `.into()` when a `bool` is needed, making the implicit timing risk visible at call sites.

---

### F-10 — LOW: `BlindingFactorGenerator` Uses Short Domain Label `b"blind"`

**File:** [crates/z00z_crypto/src/commitments.rs](crates/z00z_crypto/src/commitments.rs)  
**Severity:** S3 (Low)  

**Problem:**
```rust
// In BlindingFactorGenerator::generate(asset_idx):
h2scalar_zk(b"blind", &[&seed, &idx_bytes])
```
The domain label `b"blind"` is 5 bytes with no namespace. The `h2scalar_zk` function feeds this into `HashToScalarDomain` with label `H2S`. If other code calls `h2scalar_zk(b"blind", ...)` with different semantics, the domain separation is broken.

**Fix:**  
- Promote to a fully-qualified domain constant: `const BLIND_GEN_LABEL: &[u8] = b"z00z/commit/blinding/v1";`
- Or declare a dedicated `hash_domain!(BlindGenDomain, "z00z.commit.blinding.v1", 1)` and use it directly.

---

### F-11 — LOW: Two Competing Hash Domain Systems

**File:** [crates/z00z_crypto/src/hash.rs](crates/z00z_crypto/src/hash.rs) (custom `dst()`) and [crates/z00z_crypto/src/domains.rs](crates/z00z_crypto/src/domains.rs) (Tari `hash_domain!`)  
**Severity:** S3 (Low)  

**Problem:**  
The crate exposes two separate domain-separated hash mechanisms:
1. **Custom `dst()` system** (`hash.rs`): `"z00z.hash.v1\0" || len_le(domain) || domain || len_le(label) || label` — used by `blake2b_256`, `sha256_256`, `hmac_sha256`.
2. **Tari `DomainSeparatedHasher`**: `hash_domain!` macro produces typed hashers — used by `domains.rs`, `claim/statement.rs`, `kdf.rs` (`h2s_zk`/`hash_zk`).

These two systems produce different hash outputs for the same logical domain name. If a future developer inadvertently mixes them (e.g., switching `blake2b_256("z00z.tx.hash.v1", "tx", ...)` to `DomainHasher::<TxHashDomain>::new_with_label("tx")`), the outputs will silently diverge. This has already caused documented breakage (comment in `hmac_sha256` about all existing tags being invalid after migration to RustCrypto).

**Fix (informational):**  
Document clearly which system is authoritative for each use case:
- Tari `DomainSeparatedHasher` → all consensus-level ZK hashes (`hash_zk`, `h2s_zk`)
- Custom `dst()` → HMAC, SHA-256 uses (where Tari Blake2b is not used)
- Add a one-sentence top-level comment in both `hash.rs` and `domains.rs` indicating which contexts use which system and why both exist.

---

### F-12 — INFO: `ClaimVerifyReport::owner_bind_checked` Is Hardcoded `true`

**File:** [crates/z00z_crypto/src/claim/proof.rs](crates/z00z_crypto/src/claim/proof.rs)  
**Severity:** S4 (Informational)  

**Problem:**
```rust
// ClaimVerifyReport construction in verify_genesis_claim
ClaimVerifyReport {
    owner_bind_checked: true, // hardcoded, no actual binding done
    // ...
}
```
The field name implies a verifier checked owner binding. No such check exists. Downstream consumers reading `report.owner_bind_checked == true` will be misled about the security properties actually verified.

**Fix:** Either remove the field until owner binding is implemented, or rename to `owner_bind_placeholder: bool` with `#[deprecated]` so callers cannot use it without noticing.

---

### F-13 — INFO: `derive_pack_nonce` Uses Heap Allocation via Concatenation

**File:** [crates/z00z_crypto/src/kdf.rs](crates/z00z_crypto/src/kdf.rs)  
**Severity:** S4 (Informational)  

**Problem:**
```rust
let derived = kdf_consensus(b"asset_pack/nonce", &[k_dh.as_slice(), &seq_bytes].concat())?;
```
`.concat()` allocates a `Vec<u8>` for what is a 40-byte fixed-size input (`32 + 8`). This is in a potentially hot path.

**Fix:**
```rust
let mut input = [0u8; 40];
input[..32].copy_from_slice(k_dh);
input[32..].copy_from_slice(&seq_bytes);
let derived = kdf_consensus(b"asset_pack/nonce", &input)?;
```

---

### F-14 — INFO: `EphemeralScalarDomain` String Uses Different Case Convention

**File:** [crates/z00z_crypto/src/domains.rs](crates/z00z_crypto/src/domains.rs)  
**Severity:** S4 (Informational)  

**Problem:**  
Consensus domains use `"Z00Z/..."` (uppercase) while wallet/generic domains use `"z00z...."` (lowercase dot-separated). Mixed case is intentional per spec (Section §2.2.2.1) but creates an audit/maintenance hazard: a developer writing `"z00z/dh"` instead of `"Z00Z/DH"` would produce a different, non-error-checked domain.

**Fix:** Add a compile-time uniqueness test that also canonicalizes case — or add a CI lint check that validates all domain strings against the two accepted formats.

---

## 🔍 Phase 5 — Open Ambiguities

| # | Question | Risk |
|---|----------|------|
| A-1 | `hash_zk` and `h2s_zk` are called in `ecdh.rs` and `kdf.rs` but not declared in `lib.rs` module list. Are they re-exported from `backend_tari.rs`? If so, the abstraction barrier is incomplete. | Medium |
| A-2 | `ZkPackEncrypted` size constant `ZKPACK_V1_CT_LEN=72` suggests a fixed-size plaintext. What is the plaintext format? If it includes blinding-factor-adjacent material, separation of tag from ciphertext (F-06) gains severity. | High |
| A-3 | `Tag16Domain` is declared in `domains.rs` but no implementation of `compute_view_tag` exists. Is this planned or forgotten? | Medium |
| A-4 | `ecdh_stealth.rs` is `#[cfg(test)] pub(crate)` only. Is the production stealth-address path through `ecdh.rs` directly, or is there a consumer layer not in scope? | Low |
| A-5 | `generate_hedged_r` combines `(secret_ctx, message, add_entropy)` via `h2scalar_zk`. The `add_entropy` is `&[u8; 32]` — what is its source? If it's `[0u8;32]` in any caller, the hedging fails. | Medium |

---

## 🛠️ Phase 6 — Concrete Remediation Plan

### Priority 1 — S0 Fixes (Must Fix Before First Use of Claim System)

#### P1-A: Real Claim Proof (addresses F-01, F-02)

**Step 1:** Add secret witness to `GenesisClaimWitness`:
```rust
// claim/proof.rs
pub struct GenesisClaimWitness {
    /// Secret binding scalar known only to the legitimate claimant.
    pub binding_secret: Hidden<Z00ZScalar>,
}
```

**Step 2:** Add a public commitment to `GenesisClaimStatement.recipient_binding`:
This field already exists (`recipient_binding: [u8;32]`) — it must be the Ristretto encoding of `binding_secret * G`.

**Step 3:** Replace `prove_genesis_claim` with a Schnorr proof:
```rust
/// prove_genesis_claim: Schnorr proof-of-knowledge of binding_secret
pub fn prove_genesis_claim(
    statement: &GenesisClaimStatement,
    witness: &GenesisClaimWitness,
) -> Result<GenesisClaimProof, ClaimProofErr> {
    // 1. Derive challenge c = H(stmt_hash || R_pub)
    // 2. Produce Schnorr response: s = r + c * binding_secret
    // 3. Return proof = (R_pub, s)
    todo!("implement Schnorr proof-of-knowledge")
}
```

**Step 4:** In `GenesisClaimStatement`, verify that recipient_binding == witness.binding_secret * G before proof construction.

#### P1-B: Real Authority Signature (addresses F-02)

Replace the hash-MAC with an asymmetric Schnorr signature using an authority keypair stored in protocol configuration.

```rust
pub struct ClaimAuthoritySig {
    // Schnorr signature from authority key over stmt_hash
    pub sig: [u8; 64], // (R_pub 32 || s 32)
}
```

Verifier must hold `AUTHORITY_PUBLIC_KEY: Z00ZRistrettoPoint` as a compile-time or genesis-config constant.

---

### Priority 2 — S1 Fixes

#### P2-A: Fix CT comparison (addresses F-03)

In `claim/verifier.rs`, replace all `!=` / `==` on secret digests with `subtle::ConstantTimeEq`:
```rust
use subtle::ConstantTimeEq;
if got.ct_eq(&want).unwrap_u8() == 0 {
    return Err(ClaimProofErr::DigestMismatch);
}
```

#### P2-B: Implement view tags (addresses F-04)

```rust
// In kdf.rs - new function
pub fn compute_view_tag_byte(k_dh: &[u8; 32]) -> u8 {
    hash_zk::<Tag16Domain>("", &[k_dh])[0]
}
```
Wire into leaf creation and scanning paths. Update `derive_leaf_ad` signature to include `view_tag: u8`.

---

### Priority 3 — S2 Fixes

| Finding | Fix Summary |
|---------|-------------|
| F-05 | Document that blinding factors MUST incorporate asset_id in seed derivation. Add `#[must_use]` sentinel or assertion at range proof creation. |
| F-06 | Merge `ciphertext` and `tag` in `ZkPackEncrypted`: change to `inner: Vec<u8>` with a private accessor that splits at `len - 16`. |
| F-07 | Change `hmac_sha256` to `Result<[u8;32], CryptoError>` or add `.expect()` with justification. |
| F-08 | Add `.expect("wide scalar reduction is infallible")` to `from_hash` or change to `Result`. |

---

## 🔑 Phase 7 — Non-Claim Cryptography Assessment

This section covers production-quality code outside the claim subsystem.

### ✅ AEAD Module (`aead.rs`)

| Property | Status |
|----------|--------|
| Algorithm choice | ✅ XChaCha20-Poly1305 — correct |
| Nonce generation | ✅ Random 192-bit via OS entropy |
| Nonce caller supply | ✅ Callers cannot supply nonces (`seal()` is authoritative) |
| Plaintext size limit | ✅ 8 MB hard limit, checked before allocation |
| AAD size limit | ✅ 8 KB / 64 KB extended, overflow-checked |
| Error genericness | ✅ No oracle: `AeadError::Crypto` only |
| Envelope format | ✅ `algo_id(1) || nonce(24) || ct_with_tag` — stable, versioned |
| Low-level visibility | ✅ `xchacha20poly1305_encrypt/decrypt` are `pub(crate)` |
| WASM path | ✅ Separate `getrandom` path — correct |
| **Gap** | ⚠️ `ZkPackEncrypted` diverges from envelope format (F-06) |

### ✅ KDF Module (`kdf.rs`)

| Property | Status |
|----------|--------|
| Argon2id parameters | ✅ `moderate` (128MB/3iter), `strong` (256MB/5iter) — appropriate |
| Argon2id untrusted validation | ✅ Bounds check + total-cost overflow check |
| HKDF enforcement | ✅ `info` must be non-empty; salt required for low-entropy IKM |
| SecretBytes32 | ✅ Zeroize-on-drop, no `PartialEq`, CT-compare |
| `ct_eq` return type | ⚠️ Returns `bool` — see F-09 |
| `from_hash` fallback | ⚠️ Falls back to `one()` — see F-08 |
| Domain salts (`KDF_CONS_SALT`, `KDF_WLT_SALT`) | ✅ Distinct, non-empty, namespace-qualified |
| `generate_hedged_r` | ✅ Combines secret context + message + entropy |
| `derive_pack_nonce` | ⚠️ Heap allocation on hot path (F-13) |

### ✅ Hash Module (`hash.rs`)

| Property | Status |
|----------|--------|
| DST format | ✅ `"z00z.hash.v1\0" || len_le(domain) || domain || len_le(label) || label` |
| Length prefixing | ✅ Prevents part-boundary ambiguity (confirmed by test) |
| HMAC implementation | ✅ RustCrypto audited HMAC-SHA256 |
| `hmac_sha256` error | ⚠️ Silent `[0u8;32]` fallback — see F-07 |
| Dual hash systems | ⚠️ Maintenance hazard — see F-11 |

### ✅ Commitment & Range Proof Modules

| Property | Status |
|----------|--------|
| Blinding factor rejection | ✅ Loop up to 32 tries, reject zero scalar |
| Commitment comparison | ✅ `subtle::ConstantTimeEq` in `verify()` |
| Bulletproofs+ parameters | ✅ `RANGE_PROOF_BITS_V1=64`, `AGGREGATION_FACTOR=1` |
| DoS protection | ✅ `MAX_PROOF_SIZE=10KB`, `MAX_BATCH_PROOF_COUNT=1000`, `MAX_BATCH_MEMORY=8MB` |
| Single-use service cache | ✅ Lazy static, fail-fast init |
| Range proof asset-binding | ⚠️ No asset_id in proof statement — see F-05 |

### ✅ ECDH / Stealth Address Module

| Property | Status |
|----------|--------|
| Identity rejection | ✅ Both sender and receiver reject identity inputs |
| Zero scalar rejection | ✅ Both sides check `is_zero()` |
| `derive_k_dh` | ✅ Domain-separated hash via `KdhDomain` |
| DLEQ proof | ❌ Missing — ephemeral key `R_pub` not bound to shared secret in verifiable way |
| View tag | ❌ Missing (F-04) — `Tag16Domain` declared but not wired |

### ✅ Types, Secret, Hidden Modules

| Property | Status |
|----------|--------|
| `Z00ZScalar` zeroize | ✅ `ZeroizeOnDrop` via `#[repr(transparent)]` over `RistrettoSecretKey` |
| `Z00ZScalar::random_from_rng` | ✅ Rejects zero scalar, max 16 tries, returns `CryptoError::RngFailure` |
| `Hidden<T>` Debug/Display | ✅ Print `[REDACTED]`, do not reveal contents |
| `dangerous_clone()` | ✅ `#[must_use]` annotation, explicit naming |
| `SecretBytes` | ✅ Vec-backed, zeroize-on-drop, no PartialEq |

### ✅ Validation Module

| Property | Status |
|----------|--------|
| Identity pre-check | ✅ `[0u8;32]` compare before decompression |
| Identity post-check | ✅ Checks compressed bytes of decoded point |
| `is_identity_compressed` | ✅ Uses `subtle::ConstantTimeEq` |
| Non-canonical encoding | ✅ `from_canonical_bytes` rejects |

---

## 📋 Consolidated Finding Table

| ID | Severity | Component | Short Description | Fix Priority |
|----|----------|-----------|-------------------|--------------|
| F-01 | **S0 Critical** | `claim/prover.rs`, `claim/verifier.rs` | Claim proof is hash-MAC, not ZK | P1 (block release) |
| F-02 | **S0 Critical** | `claim/proof.rs` | Authority "signature" is hash, any party can forge | P1 (block release) |
| F-03 | S1 High | `claim/verifier.rs` | Non-CT comparison → timing oracle | P2 |
| F-04 | S1 High | `ecdh.rs`, `ecdh_stealth.rs` | No view tag → O(n) scan, `Tag16Domain` dead code | P2 |
| F-05 | S2 Medium | `range_proofs.rs` | Range proof no asset_id/chain_id binding | P3 (doc + caller contract) |
| F-06 | S2 Medium | `zkpack.rs` | AEAD tag stored separately, non-standard | P3 |
| F-07 | S2 Medium | `hash.rs` | `hmac_sha256` silent zero return on key error | P3 |
| F-08 | S2 Medium | `types.rs` | `from_hash` falls back to scalar one silently | P3 |
| F-09 | S3 Low | `types.rs`, `kdf.rs` | `ct_eq()` returns `bool` not `Choice` | P4 |
| F-10 | S3 Low | `commitments.rs` | Short domain label `b"blind"` in blinding generator | P4 |
| F-11 | S3 Low | `hash.rs`, `domains.rs` | Two hash domain systems — maintenance hazard | P4 |
| F-12 | S4 Info | `claim/proof.rs` | `owner_bind_checked: true` hardcoded | P4 |
| F-13 | S4 Info | `kdf.rs` | `derive_pack_nonce` heap-allocs 40-byte concat | P5 |
| F-14 | S4 Info | `domains.rs` | Mixed uppercase/lowercase domain string convention | P5 |

---

## 🐞 Test Plan Additions

The following tests are **missing** and should be added to close validation gaps:

| Test | Location | What It Validates |
|------|----------|-------------------|
| `claim_proof_binds_witness_secret` | `claim/prover.rs` tests | Proves that changing `binding_secret` changes the proof (once F-01 is fixed) |
| `claim_forgery_rejected` | `claim/verifier.rs` tests | A proof produced without witness is rejected |
| `authority_sig_requires_private_key` | `claim/proof.rs` tests | Constructing `ClaimAuthoritySig` without the private key fails (once F-02 fixed) |
| `claim_verifier_ct_timing` | `claim/verifier.rs` tests | Measures timing of `verify_genesis_claim` across differing prefix lengths (regression guard) |
| `view_tag_filters_non_owner` | `ecdh.rs` tests | 255/256 outputs are rejected at tag stage |
| `view_tag_passes_owner` | `ecdh.rs` tests | Owner output passes tag filter and full ECDH |
| `zkpack_tag_always_appended` | `zkpack.rs` tests | Serialised bytes always end with the 16-byte tag |
| `blinding_gen_domain_separation` | `commitments.rs` tests | `BlindingFactorGenerator` with different seeds → different scalars; different asset_idx → different scalars |
| `hmac_sha256_returns_error_on_failure` | `hash.rs` tests | Validates the (rare) key-error path doesn't silently return zeros |

---

## 🚩 Final Decision

| Layer | Status |
|-------|--------|
| AEAD (`aead.rs`) | ✅ Production-ready |
| KDF (`kdf.rs`) | ✅ Production-ready (minor S2/S3 cleanup needed) |
| Commitments + Range Proofs | ✅ Production-ready (document caller contract for F-05) |
| ECDH / Stealth (`ecdh.rs`) | ⚠️ Production with known limitation (add view tags before mainnet) |
| Claim system (`claim/`) | 🛑 **BLOCKED — S0 vulnerabilities present, do not deploy** |

> ‼️ The `claim/` module must not be used in any production flow until F-01 and F-02 are resolved with real asymmetric cryptography. All other modules are safe to ship pending the S2–S3 cleanups at the pace of normal development.

---

*Report produced by: GitHub Copilot (Claude Sonnet 4.6) — 2025-01-09*
