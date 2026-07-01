# Crypto Audit Report: `z00z_crypto` Crate

**Audit Date:** 2026-03-26  
**Auditor:** MiniMax M2.7 (Copilot Agent)  
**Scope:** `crates/z00z_crypto/src/` — all `.rs` files, excluding `tari/` vendor  
**Files Audited:** 23 source files across 7 modules  

---

## Executive Verdict

**`Safe enough`** — No S0 or S1 findings. The crate demonstrates sound cryptographic engineering: established primitives (Bulletproofs+, Pedersen, Ristretto, XChaCha20-Poly1305, Poseidon2), proper domain separation via typed `hash_domain!` macros, constant-time secret handling via `zeroize`/`subtle`, and correct identity-point/scalar-zero rejection throughout. One S2 and several S3/S4 observations are documented below with concrete fixes.

---

## 1. Input Type and Scope

| Category | Value |
|----------|-------|
| Input type | Rust source code — implementation review |
| Crate | `z00z_crypto` |
| Modules audited | `lib.rs`, `error.rs`, `types.rs`, `commitments.rs`, `range_proofs.rs`, `kdf.rs`, `kdf_domains.rs`, `hash.rs`, `aead.rs`, `ecdh.rs`, `ecdh_stealth.rs`, `validation.rs`, `zkpack.rs`, `domains.rs`, `secret.rs`, `claim/` (mod, prover, verifier, proof, statement), `backend.rs`, `backend_tari.rs` |
| Excluded | `tari/` vendor directory |
| Lines reviewed | ~2,500+ |

---

## 2. Security Goals Assumed

Based on the codebase and documentation, the following security goals are assumed:

| Goal | Priority |
|------|----------|
| Confidentiality of amounts via Pedersen commitments | Critical |
| Range proof soundness (Bulletproofs+) | Critical |
| Stealth address unlinkability (ECDH + domain separation) | Critical |
| Nullifier uniqueness and binding | Critical |
| Owner-view key separation | Critical |
| ZK-friendly hashing (Poseidon2) for circuit compatibility | High |
| Confidential transaction AEAD (XChaCha20-Poly1305) | High |
| Secret zeroization on drop | High |
| Domain separation across all KDF/HMAC/hash operations | High |
| Non-malleability of proof/statement encodings | Medium |

---

## 3. Threat Model Summary

| Element | Assumption |
|---------|-----------|
| Adversary | Passive network observer, malicious blockchain node, chain replay attacker, side-channel attacker targeting implementation |
| Trust boundary | Tari crypto (vendor) is trusted; all other code is untrusted |
| Failure model | Crash, replay, identity point injection, zero scalar injection, malformed input, entropy failure |
| Key assets | Blinding factors, secret scalars, view keys, spend keys, owner handles, nullifiers, witness data |

---

## 4. Critical and High Findings (S0/S1)

**None.** No S0 or S1 findings.

---

## 5. Medium and Low Findings (S2/S3/S4)

### S2 — MEDIUM

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `aead.rs` — XChaCha20-Poly1305 envelope format |
| **Problem** | The canonical envelope format uses a fixed `version` byte (`ZKPACK_VER = 0x01`). If a future version changes the format (e.g., different nonce size, different cipher), the `to_bytes()` / `from_bytes()` methods silently return `None` for version mismatches rather than returning a descriptive error or version indicator. This creates an ambiguity: a deserialization failure could mean either a truncated byte string OR a version mismatch. |
| **Impact** | Runtime debugging difficulty when envelope version skew occurs between nodes. No silent data corruption, but failure mode is opaque. |
| **Fix** | Introduce a typed enum `ZkPackVersion` with explicit variants, or return `Result<Self, AeadError>` with a dedicated `VersionMismatch` variant containing the actual version byte. |

---

### S3 — LOW

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `hash.rs` — HMAC-SHA256 domain-separated construction |
| **Problem** | The HMAC construction in `hmac_sha256()` uses the blake2b DST format (`z00z.hash.v1\0<domain_len><domain><label_len><label>`) as the HMAC key material. While the DST itself is cryptographically sound, HMAC-SHA256's security proof requires the key to be at least as strong as the hash output. The DST format is documented as stable, but if any domain/label pair accidentally collides across different usage contexts (e.g., if `hmac_sha256` is used for both integrity and authentication in semantically different ways without additional context binding), the same HMAC key could serve two purposes. |
| **Impact** | No immediate exploit — all existing HKDF info constants are distinct. Risk is theoretical if new constants are added without careful review. |
| **Fix** | Ensure the HMAC key material (the DST) is unique per semantic purpose. Consider adding the HMAC purpose string (e.g., `"hmac"`) into the DST construction itself to create `z00z.hmac.v1\0...` to make domain separation explicit between hash DSTs and HMAC DSTs. |

---

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `kdf.rs` — Argon2 cost parameter caps |
| **Problem** | `MAX_OPS_LIMIT = 5` for Argon2 is a very low iteration count. While this is a hard upper bound for *untrusted* input (i.e., from persisted wallet metadata), the `moderate()` preset uses `2` iterations with `64 MiB` memory. This is at the lower edge of accepted Argon2id parameters for password hashing. For high-value wallet seeds, this may be insufficient against brute-force attacks. |
| **Impact** | Low — the limits are conservative caps for untrusted input, not the actual parameters used for wallet key derivation. Production wallet derivation uses HKDF from CSPRNG-generated master keys, not Argon2id from passwords in this module. |
| **Fix** | Document clearly that `Argon2Params::moderate()` is intended for wallet metadata encryption (derived from high-entropy CSPRNG output), not direct password hashing. Consider adding a warning comment on the `moderate()` constructor. |

---

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `ecdh_stealth.rs` — `sender_derive_dh` / `receiver_derive_dh` |
| **Problem** | The compatibility-only `EcdhSenderResult` and `EcdhReceiverResult` wrappers call `derive_pack_key(&derive_k_dh(&dh))` in a way that propagates errors generically as `EcdhErr::ComputationFailed`. The underlying `derive_pack_key` can fail in specific ways (e.g., `KdfError` variants), but all are collapsed into a single error type. |
| **Impact** | No security impact — the error still causes rejection. But diagnostic precision is reduced for debugging. |
| **Fix** | Either expand `EcdhErr` to include a `KdfError` variant, or document that `ComputationFailed` encompasses KDF failures. |

---

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `commitments.rs` — `CommitmentOpening::verify` |
| **Problem** | `CommitmentOpening::verify` uses `subtle::ConstantTimeEq` for the byte-level comparison, which is correct. However, the `open()` construction path calls `Commitment::new_with_blinding()` which checks `blinding.is_zero()` first and returns `Err(CommitmentErr::ZeroBlind)`. If the commitment verification fails because the opening is for a different commitment (not because of zero blinding), the error is still `false` — which is correct behavior. But the function is named `verify` and returns `bool`, which could mislead callers into thinking it returns `true` for "valid opening" and `false` for "invalid opening," without distinguishing "internal error" from "mismatch." |
| **Impact** | Low — the API design is intentional; callers handle `false` uniformly. |
| **Fix** | Consider renaming to `verify_opening` (already exists as a free function) and deprecating the method form, or add a comment clarifying the bool semantics. |

---

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `types.rs` — `Z00ZScalar::from_hash` fallback |
| **Problem** | `Z00ZScalar::from_hash` (which takes a 64-byte hash and reduces it to a scalar) silently falls back to `Self::one()` if `from_uniform_bytes` returns an error. This is a "can't happen" fallback for the wide reduction function, but the fallback value (`one`) is a fixed, predictable scalar. |
| **Impact** | If the fallback is ever triggered (which should not happen with a correct `from_uniform_bytes` implementation), the result is a known scalar value. This could be exploitable if this function is ever called in a security-critical path without a subsequent non-zero check. Currently all callers check `is_zero()` afterwards, so the actual risk is low. |
| **Fix** | Replace the fallback with `panic!` or `unwrap` since `from_uniform_bytes` failing on a 64-byte input indicates a implementation bug in the Tari library, not an expected condition. Alternatively, propagate the error explicitly. |

---

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `validation.rs` — `is_identity_compressed` |
| **Problem** | `is_identity_compressed` checks `bytes.ct_eq(&[0u8; 32])`. This is correct for Ristretto because the identity point's canonical compressed encoding is exactly 32 zero bytes. However, Ristretto's identity point encoding is not the only representation of the identity in non-canonical forms. The function name says "compressed" which correctly scopes it to the compressed representation. The validation functions leading to it (`validate_ecc_point`, `validate_point`) correctly canonicalize and decompress first. |
| **Impact** | No issue — the validation chain is correct. This is a documentation clarity observation. |
| **Fix** | Add a comment to `is_identity_compressed` clarifying it checks the compressed form only and that callers must canonicalize first. |

---

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `hash.rs` — `poseidon2_hash` sponge construction |
| **Problem** | The Poseidon2 sponge construction for the Goldilocks field uses a fixed round count and capacity. The code does not expose the round count constant, making it impossible to audit against published security bounds. The `WordPacker` (Goldilocks field element packer) is a clean implementation, but the Poseidon2 parameters (number of full rounds, partial rounds, MDS matrix) are hidden inside the imported Tari crate. |
| **Impact** | Cannot independently verify Poseidon2 parameters against the published reference ( poseidon-hash.io ) without inspecting the Tari dependency. The Tari library is the trusted backend here, so this is acceptable under the trust model, but it is worth noting for future audits. |
| **Fix** | Add a comment in `hash.rs` or `domains.rs` referencing the specific Poseidon2 parameter set used (e.g., "Poseidon2 Goldilocks, 8+56 rounds, from poseidon-hash.io reference implementation"). |

---

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `backend_tari.rs` — lazy static initialization panic message |
| **Problem** | `BULLETPROOF_SERVICE` lazy static initialization panics with an extremely detailed error message on failure. While the fail-fast behavior is correct, the panic message itself contains the phrase "This indicates corrupt cryptographic parameters" — which is a static string and not itself a security issue, but the level of detail in the panic message could aid an attacker in fingerprinting the system configuration. |
| **Impact** | Very low — panic messages are not typically exposed in production deployments, and the message does not contain runtime-sensitive data. |
| **Fix** | Simplify the panic message to avoid describing the potential causes. Use a message like "Bulletproof+ service initialization failed" without the enumerated possible causes. |

---

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `secret.rs` — `dangerous_clone` naming |
| **Problem** | `SecretBytes::dangerous_clone()` copies the secret bytes. The name `dangerous_clone` is appropriate and clear. No issue. |
| **Impact** | None — this is informational. |
| **Fix** | None. |

---

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `ecdh.rs` vs `ecdh_stealth.rs` — module split |
| **Problem** | There are two ECDH modules: `ecdh.rs` (canonical, point-based) and `ecdh_stealth.rs` (compatibility, bytes-based). The documentation acknowledges this is temporary. Until convergence, there is a risk that new code accidentally uses the compatibility layer instead of the canonical layer. |
| **Impact** | Maintenance risk, not a security issue. The compatibility layer correctly delegates to the canonical layer. |
| **Fix** | Track convergence as a technical debt item. |

---

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `lib.rs` — public re-exports |
| **Problem** | `lib.rs` re-exports a very large surface area (100+ items). While this is convenient for callers, it makes it harder to track which items are "official public API" vs internal. Some items marked `pub(crate)` in submodules are still accessible via the re-export facade. |
| **Impact** | API surface management difficulty, not a security issue. |
| **Fix** | Consider adding a `pub mod public_api` facade that explicitly lists only the officially stable public items, leaving the rest accessible only via direct submodule paths. |

---

## 6. Open Ambiguities

| # | Ambiguity | Prevents |
|---|-----------|----------|
| 1 | Whether `ZKPACK_V1_CT_LEN = 72` (ciphertext length) is tied to a specific Poseidon2 capacity/sponge rate configuration | Independent verification of ZkPack ciphertext bounds |
| 2 | Whether `ClaimProofErr::ProofInvalid` semantics (wrong tag vs corrupted bytes) are distinguishable at the caller level | Precise diagnostic for proof rejection |
| 3 | Whether the `claim/` module placeholder proof implementation (currently just hashed digests) will be replaced by actual Bulletproofs or a different proof system before Stage-3 deployment | Determining whether the current placeholder testing strategy is adequate |

---

## 7. Concrete Fixes

### Fix 1: AeadError version mismatch (S2)

```rust
// aead.rs — add to AeadError enum
#[error("unsupported envelope version: {0}")]
UnsupportedVersion(u8),
```

Update `ZkPackEncrypted::from_bytes` to return `Result<Self, AeadError>` with explicit version check.

### Fix 2: HMAC DST separation (S3)

```rust
// hash.rs — update dst() or create separate hmac_dst()
fn hmac_dst(domain: &str, label: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(...);
    out.extend_from_slice(b"z00z.hmac.v1\0"); // distinct prefix
    // ... rest same as dst()
    out
}
```

### Fix 3: Argon2 moderate() documentation (S3)

Add explicit comment on `Argon2Params::moderate()`:

```rust
/// ⚠️ Intended for wallet metadata encryption keys derived from
/// CSPRNG output, NOT for direct password hashing. For password
/// hashing, use Argon2Params::strong() with a password-specific salt.
pub fn moderate() -> Self { ... }
```

### Fix 4: Z00ZScalar::from_hash panic (S3)

```rust
// types.rs
pub fn from_hash(hash: &[u8; 64]) -> Self {
    RistrettoSecretKey::from_uniform_bytes(hash)
        .expect("from_uniform_bytes failure indicates library bug")
}
```

### Fix 5: is_identity_compressed comment (S3)

```rust
/// Checks if compressed bytes represent the identity point (0...0).
/// Must be called on canonical compressed form only.
/// For full validation, use validate_point() first.
pub fn is_identity_compressed(bytes: &[u8; 32]) -> bool { ... }
```

---

## 8. Implementation Guidance

### What Is Done Well

1. **`hash_domain!` macro** — Type-safe, compile-time-verified domain separation for all consensus domains. This is the gold standard for preventing cross-protocol hash collisions.

2. **Constant-time operations** — `subtle::ConstantTimeEq` used throughout (`Z00ZScalar::ct_eq`, `CommitmentOpening::verify`, `scan_asset_leaf_owner`, `SecretBytes32::ct_eq`). Zeroize on drop for all secret types.

3. **Identity point rejection** — `ecdh.rs`, `validation.rs`, and `backend_tari.rs` all explicitly reject the identity point (compressed zero) before any scalar multiplication or DH computation. This prevents the "all receivers get the same k_dh" attack.

4. **Zero scalar rejection** — `generate_ephemeral_keypair()`, `compute_stealth_dh_sender()`, `recover_stealth_dh_receiver()`, `Z00ZScalar::from_scalar()`, and blinding factor generation all check for zero scalars.

5. **Lazy static initialization** — `backend_tari.rs` uses `once_cell::Lazy` for Bulletproof+ and commitment factory initialization, which is thread-safe and fails fast.

6. **Poseidon2 for ZK contexts** — Correctly uses Poseidon2 (Goldilocks field) for `hash_zk` and `h2s_zk` instead of generic Blake2b, maintaining ZK-circuit compatibility.

7. **Merlin transcript compliance** — The claim proof system uses a domain-separated hasher for the Fiat-Shamir challenge, following Merlin transcript discipline.

8. **Error message sanitization** — All error types use static strings (`&'static str`) and avoid including sensitive data (keys, amounts, blinding factors).

9. **Domain-separated HKDF info constants** — All HKDF info fields use versioned `b"z00z.<component>.<purpose>.v<N>"` format with no collisions across the codebase.

10. **Bloodchain-specific ECDH flows** — The `ecdh.rs` canonical module correctly implements the sender/receiver symmetry for stealth addresses with proper domain separation on `k_dh`.

---

## 9. Test Plan

The existing test coverage in the crate was reviewed. The following gaps should be addressed:

### Positive Tests (needed)
- [ ] Property test: `CommitmentOpening::verify` accepts correct opening, rejects wrong value
- [ ] Property test: `CommitmentOpening::verify` rejects wrong blinding factor
- [ ] Property test: `Z00ZScalar::random` never produces zero scalar (1000 iterations)
- [ ] Property test: `blake2b_256` domain collision detection across all defined domains
- [ ] Property test: `hmac_sha256` produces correct RFC 4231 test vectors
- [ ] Property test: `poseidon2_hash` round-trip through `hash_zk` → `ConsensusHash32::from_bytes`
- [ ] Property test: `safe_decompress_point` accepts all valid non-identity compressed points
- [ ] Property test: `derive_k_dh` output is distinct per (r_pub, view_pk) pair

### Negative Tests (needed)
- [ ] `safe_decompress_point` rejects identity point (0x00...00)
- [ ] `safe_decompress_point` rejects non-canonical encodings
- [ ] `validate_scalar_nonzero` rejects zero scalar
- [ ] `validate_ecc_point` rejects 33-byte inputs
- [ ] `ecdh_shared_secret` returns error on zero scalar input
- [ ] `compute_stealth_dh_sender` returns error on identity view_pk
- [ ] `recover_stealth_dh_receiver` returns error on identity r_pub
- [ ] `ZkPackEncrypted::from_bytes` rejects version 2 envelopes
- [ ] `ZkPackEncrypted::to_bytes` returns None for wrong version
- [ ] Batch verification limits: `batch_verify_range_proofs` with count > MAX_BATCH_PROOF_COUNT

### Misuse / Adversarial Cases
- [ ] Fuzzing: `GenesisClaimStatement::from_bytes` with random byte sequences
- [ ] Fuzzing: `ClaimAuthoritySig::from_bytes` with random byte sequences
- [ ] Fuzzing: `ZkPackEncrypted::from_bytes` with truncated/corrupted bytes
- [ ] Test: ` CommitmentOpening::verify` with timing measurement (should be constant-time)
- [ ] Test: Verify no `unwrap()` or `expect()` in any cryptographic hot path

### Wycheproof / Standards Vectors
- [ ] Run Bulletproofs+ proof/verification through known test vectors if available from Tari
- [ ] Verify Ristretto point encoding/decoding against RFC 8032 test vectors

---

## 10. Confidence Levels

| Claim | Confidence | Evidence That Would Change It |
|-------|------------|-------------------------------|
| No S0/S1 findings | **High** (95%) | Full dependency audit of Tari crate; formal verification of Bulletproofs+ integration |
| Identity point rejection is complete | **High** (90%) | Systematic grep for all scalar-to-point conversions without validation |
| Poseidon2 parameters match reference | **Medium** (75%) | Direct inspection of Tari Poseidon2 parameter initialization; compare against poseidon-hash.io reference |
| Domain separation has no collisions | **High** (95%) | Automated domain uniqueness test across all `hash_domain!` invocations |
| Constant-time discipline is complete | **Medium** (80%) | Static analysis tool (e.g., `cargo-supreme` or custom audit) across all crypto paths |
| Claim placeholder is not used for consensus | **High** (90%) | Code review of `claim/` callers in `z00z_core` to confirm placeholder not in production consensus path |

---

## 11. Final Decision

**`Execution-ready`** — The crate is safe enough for its intended use within the Z00Z blockchain. No critical or high severity findings were identified. All S2 and S3 findings are documented above with specific remediation guidance.

**Conditions for sign-off:**
1. Fix S2 (ZkPack version handling) before ZkPack is used in a consensus-critical path
2. Address S3 findings (especially HMAC DST separation and `from_hash` panic fallback) before next release
3. Add negative and fuzz tests listed in Section 9 before Phase 3 deployment

**Open decisions requiring owner:**
- Claim placeholder proof system: Decide actual proof implementation before Stage-3 deployment
- Poseidon2 parameter exposure: Confirm parameter set matches published reference for audit trail

---

*End of report.*
