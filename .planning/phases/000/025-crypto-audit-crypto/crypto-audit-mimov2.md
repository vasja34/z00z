# Crypto Architect Audit — `z00z_crypto` Crate (MIMO-V2)

**Date:** 2026-03-26  
**Scope:** `crates/z00z_crypto/src/**/*.rs` excluding `tari/` vendor directory  
**Input Type:** Source code — implementation review mode  
**Auditor:** Crypto Architect skill (MIMO-V2)

---

## Executive Verdict

**`Risky but salvageable`** — S1 findings present; concrete fixes exist and are documented below.

The crate demonstrates strong architectural discipline: domain separation is comprehensive, error messages avoid sensitive data leakage, `Hidden<T>` and `SecretBytes32` wrappers enforce zeroization, and the backend abstraction cleanly isolates Tari. However, several S1 and S2 issues require remediation before production deployment.

---

## 1. Input Type and Scope

| Property | Value |
|----------|-------|
| **Type** | Rust source code — cryptographic library |
| **Files reviewed** | 23 `*.rs` files in `src/` and `src/claim/` |
| **Excluded** | `tari/` vendor directory (read-only boundary) |
| **Crate purpose** | Unified crypto API: commitments, range proofs, ECDH stealth, AEAD, KDF, hashing, claim proofs |

---

## 2. Security Goals Assumed

| Goal | Status | Evidence |
|------|--------|----------|
| **Confidentiality** | ✅ | Pedersen commitments hide amounts; XChaCha20-Poly1305 for AEAD |
| **Binding** | ✅ | Bulletproofs+ range proofs bind to commitments |
| **Unlinkability** | ⚠️ | Stealth ECDH implemented; view tag construction not yet visible |
| **Non-malleability** | ✅ | Canonical encoding enforced; identity point rejection |
| **Replay resistance** | ⚠️ | Claim nullifier present; tx-level replay not visible in this crate |
| **Zeroization** | ✅ | `Hidden<T>`, `SecretBytes32`, `Z00ZScalar` all `#[zeroize(drop)]` |
| **Domain separation** | ✅ | 30+ `hash_domain!` declarations; HKDF info constants |

---

## 3. Threat Model Summary

| Adversary | Mitigation |
|-----------|------------|
| Passive chain observer | Pedersen commitments + Bulletproofs+ hide amounts |
| Malicious prover | Range proof verification; claim proof statement binding |
| Replay attacker | Claim nullifier; domain-separated transcripts |
| Side-channel (timing) | `subtle::ConstantTimeEq` used for secret comparisons |
| DoS via large inputs | AEAD size limits; batch proof count limits; Argon2 cost caps |
| Identity point attacker | Explicit rejection in `ecdh.rs`, `validation.rs` |
| Zero scalar attacker | Explicit rejection for ephemeral `r` in ECDH flows |

---

## 4. Critical and High Findings (S0/S1)

### S1-01: `hmac_sha256` Returns `[0u8; 32]` on Key Error — Silent MAC Forgery Risk

| Field | Content |
|-------|---------|
| **Severity** | S1 — HIGH |
| **Component** | `hash.rs` — `hmac_sha256()` and `hmac_sha256_raw()` |
| **Problem** | When `HmacSha256::new_from_slice(key)` fails (e.g., key > 2^64 bytes), the function returns `[0u8; 32]` — a valid-looking but completely insecure MAC. The same pattern appears in `verify_hmac()`, which returns `false` on key error instead of propagating the failure. |
| **Impact** | A caller receiving `[0u8; 32]` may use it as a legitimate MAC tag. An attacker who knows the key schedule failed can forge messages that match the all-zero tag. In `verify_hmac`, a key error silently rejects valid messages (availability issue) but does not accept forgeries (safety preserved). |
| **Fix** | Return `Result<[u8; 32], CryptoError>` from `hmac_sha256()` and `hmac_sha256_raw()`. Propagate `HmacInvalidKeyLength` as `CryptoError::InvalidParameters { param: "hmac_key" }`. For `verify_hmac`, return `Result<bool, CryptoError>` and let callers decide whether to treat key errors as verification failures. |

### S1-02: `h2scalar_zk` Falls Back to `Z00ZScalar::one()` on Hash Failure

| Field | Content |
|-------|---------|
| **Severity** | S1 — HIGH |
| **Component** | `kdf.rs` — `h2scalar_zk()` |
| **Problem** | The function uses `.unwrap_or_else(\|_\| Z00ZScalar::one())` when `h2s_zk` fails. This silently replaces a cryptographic failure with a deterministic scalar value (`1`). If `h2s_zk` ever fails due to a bug or field arithmetic issue, all callers receive the same scalar `1`, breaking unlinkability and potentially enabling key recovery. |
| **Impact** | If `h2s_zk` fails for any reason (field reduction bug, domain collision, memory corruption), every call to `h2scalar_zk` returns the same scalar. This collapses all hedged ephemeral keys, view keys, and blinding factors to a single known value. An attacker who triggers the failure path can predict all derived secrets. |
| **Fix** | Return `Result<Z00ZScalar, CryptoError>` from `h2scalar_zk()`. Update all callers to propagate the error. The fallback to `Z00ZScalar::one()` must be removed entirely — a hash-to-scalar failure is a security-critical event that must halt the operation. |

### S1-03: `Z00ZScalar::from_hash` Falls Back to `Self::one()` on Failure

| Field | Content |
|-------|---------|
| **Severity** | S1 — HIGH |
| **Component** | `types.rs` — `Z00ZScalar::from_hash()` |
| **Problem** | Same pattern as S1-02: `from_uniform_bytes` failure is silently replaced with `Z00ZScalar::one()`. This function is used for deterministic scalar derivation from hash outputs. |
| **Impact** | Identical to S1-02: scalar collapse to known value `1`. |
| **Fix** | Return `Result<Self, CryptoError>` and propagate errors. Remove the `one()` fallback. |

### S1-04: `Z00ZScalar::random` Has Infinite Loop Fallback

| Field | Content |
|-------|---------|
| **Severity** | S1 — HIGH |
| **Component** | `types.rs` — `Z00ZScalar::random()` |
| **Problem** | If `random_from_rng` fails after 16 tries, the function enters an infinite `loop` that calls `from_uniform_bytes` until it produces a non-zero scalar. If the RNG is broken or returns deterministic zeros, this loop never terminates. |
| **Impact** | Application hang (liveness failure). In a blockchain context, this blocks transaction processing, consensus participation, and wallet operations. A broken RNG should fail fast, not spin forever. |
| **Fix** | Remove the infinite loop. Return `Err(CryptoError::RngFailure)` after the retry limit. Callers that need backward compatibility can use `random_secure()` with explicit error handling. |

---

## 5. Medium and Low Findings (S2/S3/S4)

### S2-01: `ecdh_stealth` Module Exposes Byte-Oriented API Without Point Validation

| Field | Content |
|-------|---------|
| **Severity** | S2 — MEDIUM |
| **Component** | `ecdh_stealth.rs` — `ecdh_shared_secret()`, `compute_stealth_dh()` |
| **Problem** | These compatibility wrappers accept `Z00ZRistrettoPoint` but do not explicitly validate identity points before ECDH. They delegate to `compute_stealth_dh_sender` which does validate, but the wrapper layer adds no additional defense. If the canonical function's validation is ever removed or bypassed, the compatibility layer provides no safety net. |
| **Impact** | Defense-in-depth degradation. Currently safe because the underlying function validates, but the wrapper's error mapping (`map_crypto_err`) could mask future validation failures. |
| **Fix** | Add explicit `validate_stealth_point()` call in each wrapper before delegating. This is cheap (~1 comparison) and provides defense-in-depth. |

### S2-02: `ClaimAuthoritySig` Uses Hash-Based Placeholder — Not a Real Signature

| Field | Content |
|-------|---------|
| **Severity** | S2 — MEDIUM |
| **Component** | `claim/proof.rs` — `ClaimAuthoritySig::from_statement()` |
| **Problem** | The authority signature is constructed as `SIG_TAG || Blake2b-256(ClaimSigDomain, "sig", stmt_hash)`. This is a deterministic hash, not a cryptographic signature. Anyone who can compute the statement hash can forge the authority signature. The `verify_claim_authority_sig` function checks the hash matches, providing no asymmetric verification. |
| **Impact** | If this placeholder is deployed to production without replacement, any party can forge claim authority signatures. The claim flow provides no authentication of the authority. |
| **Fix** | Replace with actual Schnorr or Ed25519 signature using a known authority public key. The placeholder is acceptable for testing/staging but MUST be flagged as non-production in code comments and documentation. Add a `#[cfg(not(feature = "production"))]` gate or equivalent warning. |

### S2-03: `GenesisClaimProof` Is a Deterministic Placeholder — No ZK Soundness

| Field | Content |
|-------|---------|
| **Severity** | S2 — MEDIUM |
| **Component** | `claim/prover.rs` — `prove_genesis_claim()` |
| **Problem** | The proof is `PROOF_TAG || Blake2b-256(ClaimProofDomain, "proof", stmt_hash)`. This is a deterministic hash, not a zero-knowledge proof. The `GenesisClaimWitness` parameter is ignored (`_witness`). There is no soundness guarantee — anyone with the statement can produce a valid "proof". |
| **Impact** | Same as S2-02: no cryptographic binding between the proof and the witness. Acceptable for current testing phase but must be replaced before production. |
| **Fix** | Document clearly as placeholder. Plan migration to Bulletproofs+ or PLONK proof for production claim verification. |

### S2-04: `ZkPackEncrypted` Uses `serde` Without Canonical Encoding Guarantee

| Field | Content |
|-------|---------|
| **Severity** | S2 — MEDIUM |
| **Component** | `zkpack.rs` — `ZkPackEncrypted` derive macros |
| **Problem** | The struct derives `serde::Serialize` and `serde::Deserialize`. If these are used for consensus-critical serialization (e.g., transaction encoding), non-canonical encodings could cause consensus splits. The manual `to_bytes()`/`from_bytes()` methods are canonical, but the serde derives provide an alternative path that may be used accidentally. |
| **Impact** | Potential consensus divergence if serde serialization is used instead of canonical wire format. |
| **Fix** | Remove `serde::Serialize` and `serde::Deserialize` derives from `ZkPackEncrypted`, or add a `#[serde(skip)]` annotation with a comment explaining that canonical encoding must use `to_bytes()`/`from_bytes()`. Alternatively, implement a custom `Serialize` that delegates to the canonical format. |

### S2-05: `blake2b_256` / `blake2b_512` DST Construction Uses Manual Length Prefixing

| Field | Content |
|-------|---------|
| **Severity** | S2 — MEDIUM |
| **Component** | `hash.rs` — `dst()`, `chain_len_prefixed()` |
| **Problem** | The DST format `z00z.hash.v1\0<domain_len:u64le><domain><label_len:u64le><label>` is manually constructed. While the format is documented as stable, the manual construction is error-prone. A future refactor could break the length-prefixing logic without detection. |
| **Impact** | If DST construction changes, all existing hashes, commitments, and derived keys become invalid. This is a consensus-breaking change. |
| **Fix** | Add a frozen test vector that locks the DST format: `assert_eq!(dst("test", "label"), EXPECTED_BYTES)`. This prevents accidental format changes. Consider migrating to `hash_domain!` macro for new code paths. |

### S3-01: `SecretBytes32::into_inner` Bypasses Automatic Zeroization

| Field | Content |
|-------|---------|
| **Severity** | S3 — LOW |
| **Component** | `kdf.rs` — `SecretBytes32::into_inner()` |
| **Problem** | The method uses `std::mem::take` to extract the inner `[u8; 32]`, which replaces the original with zeros (good), but the returned bytes are not automatically zeroized. The caller is responsible for zeroizing the returned value. |
| **Impact** | If a caller forgets to zeroize the returned bytes, secret material may remain in memory after use. |
| **Fix** | Add a `#[must_use]` annotation and a prominent doc comment warning. Consider returning a `Zeroizing<[u8; 32]>` wrapper instead of raw bytes. |

### S3-02: `Z00ZScalar::reveal()` Exposes Inner Scalar Publicly

| Field | Content |
|-------|---------|
| **Severity** | S3 — LOW |
| **Component** | `types.rs` — `Z00ZScalar::reveal()` |
| **Problem** | The `reveal()` method is `pub` and returns `&RistrettoSecretKey`. This allows any downstream crate to access the raw Tari scalar, defeating the backend abstraction. |
| **Impact** | Code hygiene issue. Downstream crates may depend on Tari types through `reveal()`, creating tight coupling. Not a direct security vulnerability but undermines the abstraction layer. |
| **Fix** | Change visibility to `pub(crate)` or `pub(super)`. If external callers need the bytes, they should use `as_bytes()` or `to_bytes()` which return opaque `[u8; 32]`. |

### S3-03: `Z00ZRistrettoPoint::Debug` Leaks Point Representation

| Field | Content |
|-------|---------|
| **Severity** | S3 — LOW |
| **Component** | `types.rs` — `Z00ZRistrettoPoint` Debug impl |
| **Problem** | The `Debug` implementation calls `write!(f, "Z00ZRistrettoPoint({})", self.0)`, which prints the Ristretto point's hex representation. While public keys are not secret, logging them in debug output can aid chain analysis. |
| **Impact** | Minor privacy concern. Public keys logged in debug traces could be correlated with transactions. |
| **Fix** | Consider redacting the point bytes in Debug output (e.g., `Z00ZRistrettoPoint(..)`) for consistency with `SecretBytes32` and `Hidden<T>` patterns. |

### S4-01: `Z00ZScalar::from_hash` Silently Collapses to `one()` — Duplicate of S1-03

*(Covered in S1-03 above. Listed here for completeness.)*

### S4-02: `zkpack.rs` Has No Encryption Logic — Only Wire Format

| Field | Content |
|-------|---------|
| **Severity** | S4 — INFO |
| **Component** | `zkpack.rs` |
| **Problem** | The module defines `ZkPackEncrypted` wire format but contains no encryption/decryption functions. The actual encryption logic (Poseidon2-based sponge) is not visible in this crate. |
| **Impact** | Cannot audit the encryption flow without finding the actual implementation. |
| **Fix** | Document where the encryption/decryption logic lives (likely in `z00z_core` or a separate module). Add a doc comment pointing to the actual implementation. |

---

## 6. Open Ambiguities

| # | Ambiguity | Impact on Confidence | Evidence Needed |
|---|-----------|---------------------|-----------------|
| 1 | **Poseidon2 sponge implementation location** — `ZkPackEncrypted` has wire format but no encrypt/decrypt. Where is the actual sponge? | Cannot verify ZkPack soundness | Locate and audit the sponge implementation |
| 2 | **View tag construction** — ECDH stealth is implemented but view tag derivation is not visible in this crate | Cannot verify unlinkability | Find view tag derivation code |
| 3 | **Nullifier construction** — Claim nullifier is a 32-byte field but derivation logic is not in this crate | Cannot verify replay resistance | Locate nullifier derivation |
| 4 | **Bulletproofs+ transcript binding** — The `backend_tari.rs` delegates to Tari's `BulletproofsPlusService` but does not verify transcript construction | Cannot verify Fiat-Shamir soundness | Audit Tari's transcript construction or add explicit binding check |
| 5 | **Production claim proof plan** — Current claim proof/signature are deterministic placeholders | S2-02, S2-03 block production use | Define production proof system (Bulletproofs+, PLONK, etc.) |

---

## 7. Concrete Fixes

### Fix 1: HMAC Error Propagation (S1-01)

```rust
// BEFORE (hash.rs)
pub fn hmac_sha256(key: &[u8], domain: &str, label: &str, msg: &[u8]) -> [u8; 32] {
    let mut mac = match HmacSha256::new_from_slice(key) {
        Ok(mac) => mac,
        Err(_) => return [0u8; 32],  // ← SILENT FAILURE
    };
    // ...
}

// AFTER
pub fn hmac_sha256(key: &[u8], domain: &str, label: &str, msg: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| CryptoError::InvalidParameters { param: "hmac_key" })?;
    // ...
    Ok(mac.finalize().into_bytes().into())
}
```

### Fix 2: Remove `h2scalar_zk` Fallback (S1-02)

```rust
// BEFORE (kdf.rs)
pub fn h2scalar_zk(domain: &[u8], data: &[&[u8]]) -> Z00ZScalar {
    let encoded = encode_h2s_input(domain, data);
    let scalar = h2s_zk::<HashToScalarDomain>(H2S_LABEL, &[&encoded])
        .unwrap_or_else(|_| Z00ZScalar::one());  // ← SILENT FALLBACK
    if scalar.is_zero() { return Z00ZScalar::one(); }
    scalar
}

// AFTER
pub fn h2scalar_zk(domain: &[u8], data: &[&[u8]]) -> Result<Z00ZScalar, CryptoError> {
    let encoded = encode_h2s_input(domain, data);
    let scalar = h2s_zk::<HashToScalarDomain>(H2S_LABEL, &[&encoded])
        .map_err(|_| CryptoError::CryptoOperationFailed)?;
    if scalar.is_zero() {
        return Err(CryptoError::ZeroScalar);
    }
    Ok(scalar)
}
```

### Fix 3: Remove `Z00ZScalar::from_hash` Fallback (S1-03)

```rust
// BEFORE (types.rs)
pub fn from_hash(hash: &[u8; 64]) -> Self {
    match RistrettoSecretKey::from_uniform_bytes(hash) {
        Ok(key) => Self(key),
        Err(_) => Self::one(),  // ← SILENT FALLBACK
    }
}

// AFTER
pub fn from_hash(hash: &[u8; 64]) -> Result<Self, CryptoError> {
    RistrettoSecretKey::from_uniform_bytes(hash)
        .map(Self)
        .map_err(|_| CryptoError::InvalidScalar)
}
```

### Fix 4: Remove Infinite Loop in `Z00ZScalar::random` (S1-04)

```rust
// BEFORE (types.rs)
pub fn random<R: rand::CryptoRng + rand::RngCore>(rng: &mut R) -> Self {
    match Self::random_from_rng(rng) {
        Ok(scalar) => scalar,
        Err(_) => loop {  // ← INFINITE LOOP
            let mut bytes = [0u8; 64];
            rng.fill_bytes(&mut bytes);
            if let Ok(scalar) = Self::from_uniform_bytes(&bytes) {
                if !scalar.is_zero() { break scalar; }
            }
        },
    }
}

// AFTER
pub fn random<R: rand::CryptoRng + rand::RngCore>(rng: &mut R) -> Result<Self, CryptoError> {
    Self::random_from_rng(rng)
}
```

### Fix 5: Add Frozen DST Test Vector (S2-05)

```rust
#[test]
fn dst_format_frozen() {
    // LOCK: This test vector locks the DST format.
    // If this test fails, ALL existing hashes are invalidated.
    let dst_bytes = dst("test.domain", "label");
    let expected: &[u8] = &[
        0x7a, 0x30, 0x30, 0x7a, 0x2e, 0x68, 0x61, 0x73,  // "z00z.has"
        0x68, 0x2e, 0x76, 0x31, 0x00,                      // "h.v1\0"
        0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,    // domain_len=11
        0x74, 0x65, 0x73, 0x74, 0x2e, 0x64, 0x6f, 0x6d,    // "test.dom"
        0x61, 0x69, 0x6e,                                    // "ain"
        0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,    // label_len=5
        0x6c, 0x61, 0x62, 0x65, 0x6c,                      // "label"
    ];
    assert_eq!(dst_bytes, expected, "DST format changed — CONSENSUS BREAKING");
}
```

---

## 8. Implementation Guidance

### What's Done Well

| Aspect | Assessment |
|--------|------------|
| **Domain separation** | ✅ Excellent — 30+ `hash_domain!` declarations with versioned labels |
| **Error message hygiene** | ✅ Excellent — no sensitive data in error messages |
| **Zeroization** | ✅ Excellent — `Hidden<T>`, `SecretBytes32`, `Z00ZScalar` all zeroize on drop |
| **Constant-time comparisons** | ✅ Good — `subtle::ConstantTimeEq` used for secret comparisons |
| **Backend abstraction** | ✅ Good — `CryptoBackend` trait cleanly isolates Tari |
| **Identity point rejection** | ✅ Good — explicit checks in ECDH and validation modules |
| **AEAD size limits** | ✅ Good — DoS protection via `MAX_AEAD_PLAINTEXT_SIZE`, `MAX_AEAD_ENVELOPE_SIZE` |
| **Argon2 cost caps** | ✅ Good — `MAX_ARGON2_TOTAL_COST` prevents overflow attacks |
| **HKDF salt enforcement** | ✅ Good — low-entropy IKM requires salt |

### What Needs Improvement

| Aspect | Issue | Priority |
|--------|-------|----------|
| **Silent fallbacks** | `h2scalar_zk`, `from_hash`, `hmac_sha256` all have silent failure paths | S1 — Fix immediately |
| **Infinite loops** | `Z00ZScalar::random` can hang on broken RNG | S1 — Fix immediately |
| **Placeholder proofs** | Claim proof/signature are deterministic hashes, not ZK proofs | S2 — Plan replacement |
| **Serde on crypto types** | `ZkPackEncrypted` has serde derives that bypass canonical encoding | S2 — Remove or gate |
| **Public `reveal()`** | `Z00ZScalar::reveal()` leaks backend type | S3 — Restrict visibility |

---

## 9. Test Plan

### Positive Tests (Existing — Good Coverage)

- ✅ ECDH roundtrip symmetry (sender/receiver derive same `k_dh`)
- ✅ Domain separation uniqueness test (all 22 consensus domains verified distinct)
- ✅ Commitment determinism (same inputs → same output)
- ✅ Range proof generation/verification (zero, typical, max values)
- ✅ Batch verification
- ✅ Argon2id determinism
- ✅ HKDF version separation
- ✅ HMAC determinism
- ✅ Claim proof roundtrip

### Negative Tests (Existing — Good Coverage)

- ✅ Identity point rejection
- ✅ Zero scalar rejection
- ✅ Tampered proof rejection
- ✅ Invalid proof size rejection
- ✅ Empty HKDF info rejection
- ✅ Low-entropy IKM without salt rejection
- ✅ Malicious Argon2 params rejection

### Missing Tests (Recommended Additions)

| Test | Purpose |
|------|---------|
| `test_hmac_key_error_propagation` | Verify HMAC returns error on invalid key length |
| `test_h2scalar_zk_no_fallback` | Verify `h2scalar_zk` returns error on hash failure |
| `test_from_hash_no_fallback` | Verify `from_hash` returns error on failure |
| `test_random_no_infinite_loop` | Verify `random` returns error after retry limit |
| `test_dst_format_frozen` | Lock DST format with test vector |
| `test_zkpack_no_serde_path` | Verify serde serialization is not used for consensus |
| `test_ecdh_stealth_explicit_validation` | Verify compatibility wrappers validate points |

### Wycheproof Integration (Recommended)

Run Wycheproof test vectors for:
- AES-GCM (if used — currently XChaCha20-Poly1305)
- ECDH (Ristretto255)
- EdDSA (if used for signatures)
- HMAC-SHA256

---

## 10. Confidence Level

| Claim | Confidence | Evidence That Would Change It |
|-------|------------|-------------------------------|
| Domain separation is correct | **95%** | Frozen test vector would raise to 99% |
| Error messages are safe | **98%** | Automated scan for `secret`, `key`, `password` in error strings |
| Zeroization is complete | **90%** | Memory dump analysis after drop would raise to 99% |
| Silent fallbacks are the only S1 issues | **85%** | Full call-graph analysis of `h2scalar_zk` and `from_hash` callers |
| Claim placeholders are clearly marked | **70%** | `#[cfg(not(feature = "production"))]` gate would raise to 95% |
| ZkPack encryption is sound | **40%** | Cannot verify without locating the sponge implementation |

---

## 11. Final Decision

**`Blocked: 4 open decisions with owner`**

| # | Decision | Owner | Required Evidence |
|---|----------|-------|-------------------|
| 1 | Replace `h2scalar_zk` / `from_hash` silent fallbacks with `Result` | z00z_crypto maintainer | Updated call sites compile; tests pass |
| 2 | Remove infinite loop in `Z00ZScalar::random` | z00z_crypto maintainer | `random()` returns `Result`; callers updated |
| 3 | Define production claim proof system (replace S2-02, S2-03) | Protocol designer | Proof system selected; implementation plan |
| 4 | Locate and audit ZkPack Poseidon2 sponge implementation | z00z_crypto maintainer | Sponge code identified; separate audit pass |

---

## Appendix A: File Inventory

| File | Lines | Purpose | Findings |
|------|-------|---------|----------|
| `lib.rs` | ~200 | Public facade, re-exports | Clean |
| `aead.rs` | ~500 | XChaCha20-Poly1305 envelope | S2-04 (serde) |
| `backend.rs` | ~200 | `CryptoBackend` trait | Clean |
| `backend_tari.rs` | ~200 | Tari backend impl | Clean |
| `claim/mod.rs` | ~20 | Claim module facade | Clean |
| `claim/proof.rs` | ~200 | Proof/sig containers | S2-02, S2-03 |
| `claim/prover.rs` | ~100 | Placeholder proof gen | S2-03 |
| `claim/statement.rs` | ~200 | Statement encoding | Clean |
| `claim/verifier.rs` | ~200 | Proof/sig verification | Clean |
| `commitments.rs` | ~200 | Pedersen commitment helpers | Clean |
| `domains.rs` | ~200 | 30+ hash domain declarations | Clean |
| `ecdh.rs` | ~300 | Stealth ECDH primitives | Clean |
| `ecdh_stealth.rs` | ~200 | Compatibility ECDH wrappers | S2-01 |
| `error.rs` | ~300 | Crypto error types | Clean |
| `hash.rs` | ~500 | Blake2b/SHA-256/HMAC facade | S1-01, S2-05 |
| `hidden.rs` | ~100 | `Hidden<T>` wrapper | Clean |
| `kdf.rs` | ~800 | Argon2id/HKDF/consensus KDF | S1-02, S3-01 |
| `kdf_domains.rs` | ~50 | HKDF info constants | Clean |
| `range_proofs.rs` | ~150 | Range proof wrappers | Clean |
| `secret.rs` | ~100 | `SecretBytes` wrapper | Clean |
| `types.rs` | ~500 | Scalar/point wrappers, constants | S1-03, S1-04, S3-02, S3-03 |
| `validation.rs` | ~150 | ECC point/scalar validation | Clean |
| `zkpack.rs` | ~100 | ZkPack wire format | S2-04, S4-02 |

---

## Appendix B: Severity Summary

| Severity | Count | Findings |
|----------|-------|----------|
| **S0** | 0 | — |
| **S1** | 4 | S1-01, S1-02, S1-03, S1-04 |
| **S2** | 5 | S2-01, S2-02, S2-03, S2-04, S2-05 |
| **S3** | 3 | S3-01, S3-02, S3-03 |
| **S4** | 2 | S4-01, S4-02 |

**Total: 14 findings (4 high, 5 medium, 3 low, 2 info)**
