# 🔐 Cryptographic Audit Report — `z00z_crypto` Crate

**Auditor:** GLM-5 Turbo (Crypto Architect skill)  
**Date:** 2026-03-26  
**Scope:** All `*.rs` source files under `crates/z00z_crypto/src/`, excluding `tari/` vendor  
**Branch:** `z00z-simul`  
**Files audited:** 23 Rust source files (18 top-level + 5 claim submodule)

---

## Executive Verdict

**`Risky but salvageable`** — No S0 (fundamental) findings. One S1 finding in the ZkPack
custom cipher construction. Several S2 findings requiring documented remediation. The
overall architecture is sound: domain separation is thorough, backend abstraction is clean,
and established primitives (Bulletproofs+, Pedersen, XChaCha20-Poly1305, Argon2id, HKDF)
are used correctly. The primary risk area is the hand-rolled ZkPack stream cipher/MAC
which lacks formal security proof.

---

## 1. Input Classification

**Type:** Implementation review of a cryptographic library crate providing:
- Pedersen commitments + Bulletproofs+ range proofs
- Stealth ECDH (Ristretto25519)
- AEAD encryption (XChaCha20-Poly1305)
- KDF (Argon2id, HKDF-SHA256)
- Poseidon2 hashing (Goldilocks field, ZK-circuit-friendly)
- Blake2b/SHA-256 hashing with domain separation
- Claim proof placeholder system
- ZkPack custom encryption (stream cipher + MAC)

---

## 2. Security Goals

| Goal | Status |
|------|--------|
| Confidentiality of transaction amounts | ✅ Pedersen + Bulletproofs+ |
| Binding of commitments | ✅ Computational binding via Pedersen |
| Stealth address unlinkability | ✅ Ephemeral ECDH with identity rejection |
| Key secrecy (zeroization) | ✅ `Zeroize` on `Z00ZScalar`, `SecretBytes`, `SecretBytes32` |
| Domain separation | ✅ Comprehensive `hash_domain!` registry |
| AEAD confidentiality + integrity | ✅ XChaCha20-Poly1305 via RustCrypto |
| Replay resistance (nullifiers) | ⚠️ Nullifier in claim statement, but no global uniqueness enforcement in this crate |
| Forward secrecy | ❌ Not provided (static ECDH, no ratchet) |
| ZkPack confidentiality | ⚠️ Custom construction, no formal proof |

---

## 3. Threat Model Summary

**Adversaries considered:**
- Passive chain observer (linking transactions)
- Active network attacker (tampering with envelopes, proofs)
- Malicious prover (forging range proofs, claims)
- Malicious verifier (extracting witness data)
- Side-channel attacker (timing, memory)

**Trust boundaries:**
- `z00z_crypto` ↔ Tari vendor code (trusted, audited)
- `z00z_crypto` ↔ external callers (untrusted inputs validated)
- ZkPack internal construction (self-contained, no external deps)

**Failure assumptions:**
- RNG failure → operations halt (fail-fast)
- Identity point in ECDH → rejected
- Zero scalar → rejected
- Nonce reuse in AEAD → catastrophic (documented, random 192-bit nonce)

---

## 4. Findings

### S1 — HIGH

#### S1-1: ZkPack Uses Custom Stream Cipher + MAC Without Formal Security Proof

| Field | Content |
|-------|---------|
| **Severity** | S1 |
| **Component** | `aead.rs` → `pub mod zkpack` |
| **Problem** | ZkPack implements a custom encryption scheme: XOR stream cipher from Poseidon2-based XOF + separate Poseidon2-based MAC tag. This is a **hand-rolled authenticated encryption** construction that has not been formally analyzed. The MAC is computed as `H_zk(pack_key, nonce, leaf_ad, ciphertext, len)` — a single hash without HMAC-style nested construction or encrypt-then-MAC composition proof. |
| **Impact** | If the MAC construction is not a secure PRF, an attacker who can query the decryption oracle may forge valid ciphertext+tag pairs. The XOR stream cipher provides no confidentiality guarantee without a proven PRF property for the XOF. In the worst case, this could allow asset pack content forgery or decryption. |
| **Fix** | **Recommended:** Replace ZkPack encryption with the already-available XChaCha20-Poly1305 `seal()`/`open()` functions, deriving the key from the same `k_dh` via existing KDF. This reuses an audited AEAD construction. **Alternative:** If Poseidon2-based encryption is required for ZK-circuit compatibility, commission a formal security proof of the encrypt-then-hash-MAC composition before production use. At minimum, switch the MAC to an HMAC-style nested construction: `tag = H_zk(key, H_zk(key, nonce || ad || ct))`. |

**Confidence:** HIGH — The construction is clearly custom and lacks any reference to a security proof or standard. Evidence that would change confidence: a published security analysis or reduction proof for this exact construction.

---

### S2 — MEDIUM

#### S2-1: `h2scalar_zk` Fallback to `Z00ZScalar::one()` on Failure

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `kdf.rs` → `h2scalar_zk()`, `hash.rs` → deprecated `h2scalar_zk()` |
| **Problem** | When `h2s_zk` fails (returns `Err`), the function silently falls back to `Z00ZScalar::one()`. A scalar of 1 is a valid blinding factor but is **deterministic and predictable**. If the failure path is reachable in production (e.g., edge case in Poseidon2 output), all affected blinding factors become `1`, breaking the hiding property of Pedersen commitments. |
| **Impact** | If triggered, commitments become `C = amount*G + 1*H`, making amount recovery trivial via precomputation. |
| **Fix** | Propagate the error instead of falling back: `h2scalar_zk` should return `Result<Z00ZScalar, CryptoError>`. All callers must handle the error. The fallback to `one()` is only acceptable in test code with explicit documentation. |

**Confidence:** MEDIUM — The failure path depends on `Z00ZScalar::from_uniform_bytes` rejecting valid Poseidon2 output, which is unlikely but not impossible (e.g., if the 64-byte hash happens to be ≥ field order). Evidence that would change confidence: proof that Poseidon2 output over Goldilocks field always produces a valid Ristretto scalar when interpreted as 64 bytes.

#### S2-2: `generate_hedged_r` Does Not Mix System Entropy

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `kdf.rs` → `generate_hedged_r()` |
| **Problem** | The function derives a hedged ephemeral scalar from `secret_ctx || message || add_entropy` via `h2scalar_zk`. If `add_entropy` is predictable or low-entropy, the output is deterministic. The function name suggests hedging against bad system RNG, but it does not actually mix system CSPRNG output. |
| **Impact** | If `add_entropy` is a counter or timestamp, the ephemeral scalar `r` becomes predictable, breaking stealth address unlinkability. |
| **Fix** | Either: (a) rename to `derive_r_from_entropy()` to clarify it requires high-entropy input, or (b) add system RNG mixing: `r = h2scalar_zk(b"hedged_r", &[secret_ctx, message, add_entropy, &system_rng_bytes])`. |

**Confidence:** MEDIUM — Depends on how callers provide `add_entropy`. If callers always use CSPRNG output, this is benign.

#### S2-3: `BlindingFactorGenerator::new_deterministic` Uses Predictable Seed

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `commitments.rs` → `BlindingFactorGenerator` |
| **Problem** | Deterministic blinding factor generation from `seed || asset_idx` via `h2scalar_zk(b"blind", &[&seed, &idx_bytes])`. If the seed is known or guessable, all blinding factors for all assets are computable, breaking hiding. |
| **Impact** | Anyone who learns the seed can compute all commitment openings, recovering all amounts. |
| **Fix** | Document clearly that the seed MUST be high-entropy (≥128 bits from CSPRNG). Consider adding a runtime assertion or log warning if the seed appears to be low-entropy (e.g., all zeros, sequential). Add a `#[doc(hidden)]` attribute or `cfg(test)` gate if this is only for testing. |

**Confidence:** MEDIUM — The function is likely used only in tests/simulations, but the public API does not communicate this restriction.

#### S2-4: `ClaimAuthoritySig` Is Not a Real Signature

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `claim/proof.rs` → `ClaimAuthoritySig` |
| **Problem** | `ClaimAuthoritySig::from_statement()` computes `H(ClaimSigDomain, H(ClaimStmtDomain, stmt_bytes))` — this is a **hash-derived token**, not a cryptographic signature. Anyone who knows the statement can compute the same "signature." There is no secret key involved. |
| **Impact** | The claim authority verification is currently a placeholder that provides zero authentication. Any party can forge a valid authority signature for any statement. |
| **Fix** | This is documented as a "placeholder" in the code. Before production use, replace with an actual signature scheme (e.g., Schnorr signature over the statement hash using an authority secret key). Track as a known TODO with explicit `// TODO(S1): Replace with real signature` comment. |

**Confidence:** HIGH — The code explicitly acknowledges this is a placeholder.

#### S2-5: `kdf_from_dh` Truncates 32-byte Hash to 12-byte Nonce

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `kdf.rs` → `derive_pack_nonce()` |
| **Problem** | `derive_pack_nonce` derives 32 bytes via `kdf_consensus` and truncates to 12 bytes (`&derived[..12]`). While 12 bytes (96 bits) provides sufficient nonce space for a counter-based scheme, truncation of a hash output is not equivalent to generating a random 12-byte nonce — the truncated bytes may have biased distribution depending on the hash function's output structure. |
| **Impact** | If the 12-byte nonce has biased bits, the effective nonce space is reduced, increasing collision probability. |
| **Fix** | Use HKDF-Expand with explicit 12-byte output length instead of hash truncation: `hkdf_expand(k_dh, b"asset_pack/nonce", 12)`. This is the standard approach for deriving fixed-length nonces. |

**Confidence:** LOW — For BLAKE2b/SHA-256, truncation is generally considered safe. For Poseidon2 over Goldilocks, the bias analysis is less clear.

---

### S3 — LOW

#### S3-1: `SecretBytes::dangerous_clone` Creates Uncontrolled Copies

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `secret.rs` → `SecretBytes::dangerous_clone()` |
| **Problem** | The method name correctly signals danger, but there is no usage tracking or audit trail. Multiple copies of secret data in memory increase the attack surface for cold-boot or memory-scraping attacks. |
| **Impact** | Increased window for memory disclosure attacks. |
| **Fix** | Add a `#[track_caller]` attribute and consider logging (in debug builds) when `dangerous_clone` is called, to enable auditing of secret copy sites. |

#### S3-2: `Hidden<T>` Wrapper Does Not Implement `Drop` With Explicit Zeroization

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `hidden.rs` → `Hidden<T>` |
| **Problem** | The custom `Hidden<T>` wraps `tari_crypto::tari_utilities::Hidden<T>`. Zeroization depends on `T: Zeroize`. If `T` does not implement `Zeroize`, the inner data is not wiped on drop. The trait bound `T: Zeroize` is present, which is correct, but the wrapper adds no additional protection. |
| **Impact** | If a user creates `Hidden<Vec<u8>>` where `Vec<u8>` has a blanket `Zeroize` impl, the zeroization is correct. No current vulnerability, but the design relies on downstream `Zeroize` correctness. |
| **Fix** | Add a compile-time assertion or documentation note that `T` must have a working `Zeroize` implementation. Consider adding a `Drop` impl that explicitly calls `zeroize()` on the inner value as defense-in-depth. |

#### S3-3: `ecdh_stealth.rs` Uses `subtle::ConstantTimeEq` for Owner Tag Comparison but Not for All Comparisons

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `ecdh_stealth.rs` → `LeafOwner::owner_handle()` comparison |
| **Problem** | The `LeafOwner` trait uses `subtle::ConstantTimeEq` for owner handle comparison, which is correct. However, other comparison sites in the crate (e.g., `GenesisClaimStatement::PartialEq`) use standard equality. |
| **Impact** | Standard equality on statement hashes is acceptable (they are public data). No current timing leak for secret data. |
| **Fix** | No action needed — document that constant-time comparison is only required for secret-derived values (owner tags, k_dh), not for public hashes. |

#### S3-4: `poseidon2_hash` Uses Addition-Based Absorption Instead of XOR

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `hash.rs` → `policy::poseidon2_hash()` → `WordPacker` |
| **Problem** | The Poseidon2 sponge uses `state[rate_idx] += word` (addition in Goldilocks field) rather than the standard XOR-based absorption. While this is a valid sponge construction over prime fields, it differs from the reference Poseidon2 specification which typically uses XOR for the rate portion. |
| **Impact** | If the ZK circuit implementation uses XOR-based absorption while the native Rust code uses addition-based absorption, the two will produce different hashes, breaking cross-implementation compatibility. |
| **Fix** | Verify that the ZK circuit (when implemented) uses the same absorption mode. Document the choice explicitly. If the reference Poseidon2 spec uses addition, this is correct; if it uses XOR, this must be changed. |

**Confidence:** MEDIUM — Without access to the ZK circuit code, cross-compatibility cannot be verified.

---

### S4 — INFO

#### S4-1: Two Parallel Domain Separation Systems Exist

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `domains.rs` (new `hash_domain!` system) vs `hash.rs` → `domains` (legacy `&[u8]` constants) |
| **Problem** | Two domain separation systems coexist: (1) type-safe `hash_domain!` macro in `domains.rs` with `"z00z.*.v1"` naming, and (2) legacy `CONS_DOMAINS` / `WALLET_DOMAINS` byte-string constants in `hash.rs` with `"Z00Z/*"` naming. The consensus hash path uses the legacy system, while newer code uses the macro system. |
| **Impact** | Maintenance burden and potential for confusion. No current security issue since the two systems use different domain strings. |
| **Fix** | Plan migration of legacy domains to the `hash_domain!` macro system. Ensure no domain string collisions between the two systems during migration. |

#### S4-2: `Z00ZScalar::from_hash` Silently Falls Back to `one()`

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `types.rs` → `Z00ZScalar::from_hash()` |
| **Problem** | If `from_uniform_bytes` fails, returns `Z00ZScalar::one()` silently. Same issue as S2-1 but in a different location. |
| **Fix** | Return `Result` or document the fallback behavior prominently. |

#### S4-3: `blake2b_256_simple` / `sha256_256_simple` Exist Without Domain Separation

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `hash.rs` → `blake2b_256_simple()`, `sha256_256_simple()` |
| **Problem** | These functions exist for "non-cryptographic purposes" but are public. A caller might accidentally use them for security-sensitive operations. |
| **Fix** | Mark as `#[doc(hidden)]` or move to a `pub(crate)` module. |

#### S4-4: `hmac_sha256` Returns All-Zeros on Key Error

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `hash.rs` → `hmac_sha256()` |
| **Problem** | If `HmacSha256::new_from_slice(key)` fails (key too long for HMAC), the function returns `[0u8; 32]` instead of an error. HMAC-SHA256 accepts any key length, so this path is only reachable with an empty key, but the silent failure is still undesirable. |
| **Fix** | Return `Result<[u8; 32], CryptoError>` or at minimum `panic!("BUG: HMAC key creation failed")` since this should never happen with valid input. |

---

## 5. Composition Review

### 5.1 Domain Separation — ✅ STRONG

The domain separation system is comprehensive and well-organized:

- **22+ consensus domains** declared via `hash_domain!` macro with type safety
- **Versioned domains** (`.v1` suffix) enable future algorithm migration
- **Test verifies uniqueness** of all domain strings
- **Two-tier system**: `Z00Z/*` for consensus (Poseidon2), `z00z.*.v1` for wallet (Blake2b)
- **HKDF info constants** in `kdf_domains.rs` provide additional separation for key derivation

### 5.2 Transcript Binding — ✅ ADEQUATE

- Claim statement hash includes: genesis_root, asset_id, commitment, claim_id, chain_id, scenario_scope, recipient_binding, nullifier, owner_bind_digest, output_leaf_hashes
- ECDH k_dh derivation uses domain-separated hash over compressed point bytes
- AEAD AAD includes domain + context binding

### 5.3 Nonce Policy — ✅ CORRECT

- XChaCha20-Poly1305 uses 192-bit random nonces (safe for random generation)
- Nonce generation failure is treated as critical error (`CryptoError::RngFailure`)
- ZkPack derives nonces deterministically from k_dh + asset_id + serial_id (acceptable for deterministic encryption)

### 5.4 Key Separation — ✅ GOOD

- Different KDF salts for consensus (`z00z/consensus/kdf/v1`) vs wallet (`z00z/wallet/kdf/v1`)
- HKDF info strings are distinct per purpose
- ECDH-derived keys use separate derivation paths for pack key, pack nonce, symmetric key

### 5.5 Error Handling — ✅ GOOD (with caveats)

- No sensitive data in error messages (documented policy)
- Generic error descriptions prevent information leakage
- `#[source]` preserves debug context
- **Caveat:** S2-1 (silent fallback to `one()`) and S4-4 (silent all-zeros return) violate this principle

### 5.6 Serialization — ✅ CANONICAL

- All encodings use fixed-size fields or length-prefixed variable fields
- Little-endian throughout (consistent)
- Statement serialization rejects trailing bytes
- Point encoding uses canonical Ristretto compressed format

---

## 6. Implementation Safety

### 6.1 Constant-Time Discipline — ✅ GOOD

- `Z00ZScalar::is_zero()` uses `subtle::ConstantTimeEq`
- `SecretBytes32::ct_eq()` uses `subtle::ConstantTimeEq`
- `ZkPack` tag verification uses `subtle::ConstantTimeEq`
- Identity point detection uses `subtle::ConstantTimeEq`
- Scalar comparison via `ct_eq()` method

### 6.2 Secret Lifecycle — ✅ GOOD

- `Z00ZScalar`: `#[zeroize(drop)]`, no `Clone`, no `Debug`
- `SecretBytes`: `#[zeroize(drop)]`, explicit `wipe()` method
- `SecretBytes32`: `#[zeroize(drop)]`, no `Clone`, no `PartialEq`, custom `Debug` that hides contents
- `Hidden<T>`: wraps Tari's `Hidden<T>` with `Zeroize` bound

### 6.3 Randomness — ✅ CORRECT

- Production: `SystemRngProvider` (via `z00z_utils::rng`)
- Tests: `MockRngProvider` with explicit seeds
- WASM: `getrandom::getrandom()`
- Nonce generation failure → explicit error propagation

### 6.4 Input Validation — ✅ THOROUGH

- Identity point rejection in ECDH (sender + receiver)
- Zero scalar rejection for ephemeral keys
- Point size validation (exactly 32 bytes)
- Canonical scalar decoding
- Proof size limits (DoS prevention)
- Batch size limits (MAX_BATCH_PROOF_COUNT = 1000)
- AAD size limits (8 KB normal, 64 KB extended)
- Plaintext size limits (8 MB)
- Argon2id parameter validation with overflow protection

### 6.5 Library Usage — ✅ SOUND

- `chacha20poly1305` (RustCrypto) — audited AEAD
- `blake2` (RustCrypto) — audited hash
- `sha2` (RustCrypto) — audited hash
- `hmac` (RustCrypto) — audited MAC
- `hkdf` (RustCrypto) — audited KDF
- `argon2` — reference Argon2 implementation
- `subtle` — constant-time operations
- `zeroize` — secure memory zeroing
- `tari_crypto` — vendor crypto (Bulletproofs+, Pedersen, Ristretto)
- `p3_goldilocks` / `p3_poseidon2` — Plonky3 Poseidon2 (ZK-friendly hash)

---

## 7. Open Ambiguities

| # | Question | Impact | Owner |
|---|----------|--------|-------|
| 1 | Is ZkPack required to be ZK-circuit-compatible, or can it use XChaCha20-Poly1305? | Determines whether S1-1 requires formal proof or can be resolved by switching to standard AEAD | Protocol design |
| 2 | Does the ZK circuit use addition-based or XOR-based Poseidon2 absorption? | Determines whether S3-4 is a real cross-compatibility issue | ZK circuit team |
| 3 | What is the production use case for `BlindingFactorGenerator::new_deterministic`? | Determines whether S2-3 requires API restriction | Wallet team |
| 4 | Is `generate_hedged_r` called with CSPRNG-sourced `add_entropy` in all production paths? | Determines whether S2-2 is exploitable | Wallet team |
| 5 | When will `ClaimAuthoritySig` be replaced with a real signature? | S2-4 tracking | Protocol design |

---

## 8. Concrete Fixes (Priority Order)

### P0 — Before Production

1. **S1-1:** Replace ZkPack encryption with XChaCha20-Poly1305, or obtain formal security proof
2. **S2-1:** Make `h2scalar_zk` return `Result` instead of falling back to `one()`
3. **S2-4:** Add explicit `// TODO(S1)` tracking comment on `ClaimAuthoritySig`

### P1 — Before Mainnet

4. **S2-2:** Clarify `generate_hedged_r` entropy requirements or add system RNG mixing
5. **S2-3:** Document or restrict `BlindingFactorGenerator::new_deterministic`
6. **S2-5:** Use HKDF-Expand for 12-byte nonce derivation instead of hash truncation
7. **S3-4:** Verify Poseidon2 absorption mode matches ZK circuit implementation

### P2 — Maintenance

8. **S4-1:** Plan migration from legacy domain constants to `hash_domain!` macro
9. **S4-2:** Fix `Z00ZScalar::from_hash` silent fallback
10. **S4-3:** Hide simple hash functions from public API
11. **S4-4:** Fix `hmac_sha256` silent all-zeros return

---

## 9. Test Plan

### Required (Not Yet Verified)

| Category | Tests Needed |
|----------|-------------|
| **Wycheproof vectors** | HMAC-SHA256, XChaCha20-Poly1305, HKDF-SHA256, Argon2id |
| **Negative tests** | Identity point in ECDH, zero scalar, malformed proofs, oversized inputs |
| **Property tests** | Commitment homomorphism, range proof round-trip, ECDH symmetry |
| **Fuzzing targets** | `GenesisClaimStatement::from_bytes`, `ZkPackEncrypted::from_bytes`, `open()` envelope parsing, `poseidon2_hash` input |
| **Cross-implementation** | Poseidon2 output vs reference implementation (Plonky3) |
| **Domain uniqueness** | ✅ Already exists (`consensus_domains_unique` test) |
| **Nonce uniqueness** | ✅ Already exists (`test_nonce_uniqueness` in transport tests) |
| **Tamper detection** | ✅ Already exists (`test_tamper_detection` in transport tests) |

### Existing Good Test Coverage

- Claim proof round-trip (prove → verify)
- AEAD encrypt/decrypt round-trip
- Wrong key/AAD rejection
- Nonce uniqueness
- Tamper detection
- Domain uniqueness
- Argon2id determinism
- HKDF domain separation

---

## 10. Confidence Assessment

| Claim | Confidence | Evidence That Would Change It |
|-------|-----------|-------------------------------|
| Pedersen commitments are binding and hiding | HIGH | Tari library audit report |
| Bulletproofs+ soundness | HIGH | Tari library audit report |
| XChaCha20-Poly1305 AEAD security | HIGH | RFC 7539, RustCrypto audit |
| Domain separation prevents cross-protocol attacks | HIGH | Formal uniqueness test exists |
| Poseidon2 hash output is unbiased | MEDIUM | Reference implementation comparison |
| ZkPack construction is secure | LOW | Formal security proof |
| Stealth ECDH provides unlinkability | HIGH | Standard construction, identity rejection verified |
| Argon2id parameters resist brute-force | HIGH | Standard construction, parameter validation exists |
| HKDF key separation is sufficient | HIGH | RFC 5869 compliant, distinct info strings |

---

## 11. Final Decision

**`Blocked`** — One S1 finding (ZkPack custom cipher) must be resolved before production deployment. The fix is straightforward: either switch to XChaCha20-Poly1305 or obtain a formal proof. All other findings are S2 or below and can be addressed on a normal development timeline.

**Open decisions requiring resolution:**
1. ZkPack encryption strategy (S1-1) — Protocol design team
2. Poseidon2 absorption mode verification (S3-4) — ZK circuit team
3. ClaimAuthoritySig real signature timeline (S2-4) — Protocol design team

---

## Appendix A: File Inventory

| File | Lines (approx) | Role |
|------|---------------|------|
| `lib.rs` | 700 | Public API facade, re-exports, backend dispatch |
| `types.rs` | 350 | Z00ZScalar, Z00ZCommitment, protocol constants |
| `error.rs` | 200 | CryptoError enum (unified error handling) |
| `hash.rs` | 1100 | Blake2b, SHA-256, HMAC, Poseidon2, framing, domain separation |
| `domains.rs` | 200 | `hash_domain!` declarations (22+ domains) |
| `kdf.rs` | 700 | Argon2id, HKDF, consensus/wallet KDF, stealth derivation |
| `kdf_domains.rs` | 50 | HKDF info string constants |
| `aead.rs` | 1700 | XChaCha20-Poly1305, envelope format, transport, ZkPack |
| `ecdh.rs` | 200 | Stealth ECDH (sender/receiver), identity validation |
| `ecdh_stealth.rs` | 150 | Legacy bytes-oriented ECDH compatibility layer |
| `validation.rs` | 200 | ECC point/scalar validation, constant-time identity check |
| `commitments.rs` | 150 | Pedersen commitment wrapper, blinding factor generation |
| `range_proofs.rs` | 150 | Bulletproofs+ wrapper, batch verification |
| `secret.rs` | 100 | SecretBytes (zeroizing byte buffer) |
| `hidden.rs` | 100 | Enhanced Hidden<T> wrapper |
| `zkpack.rs` | 100 | ZkPack encrypted wire format |
| `backend.rs` | 150 | CryptoBackend trait (private) |
| `backend_tari.rs` | 200 | TariCryptoBackend implementation (private) |
| `claim/mod.rs` | 10 | Claim submodule re-exports |
| `claim/statement.rs` | 200 | GenesisClaimStatement serialization + hashing |
| `claim/proof.rs` | 200 | GenesisClaimProof, ClaimAuthoritySig containers |
| `claim/prover.rs` | 80 | prove_genesis_claim placeholder |
| `claim/verifier.rs` | 200 | verify_genesis_claim, verify_claim_authority_sig |

**Total:** ~5,900 lines of Rust (excluding tests and comments)

---

## Appendix B: Dependency Graph

```
z00z_crypto
├── tari_crypto (vendor, audited)
│   ├── Ristretto25519 (curve25519-dalek)
│   ├── Bulletproofs+ (dalek-cryptography)
│   ├── Pedersen commitments
│   ├── Schnorr signatures
│   └── Domain-separated hashing
├── blake2 (RustCrypto)
├── sha2 (RustCrypto)
├── hmac (RustCrypto)
├── hkdf (RustCrypto)
├── chacha20poly1305 (RustCrypto)
├── argon2 (reference impl)
├── subtle (constant-time)
├── zeroize (memory zeroing)
├── p3_goldilocks / p3_poseidon2 (Plonky3)
├── rand (RNG traits)
├── getrandom (entropy source)
├── once_cell (lazy statics)
├── thiserror (error derive)
├── serde (serialization, ZkPack only)
└── z00z_utils (RNG provider abstraction)
```

---

*Report generated by GLM-5 Turbo using the Crypto Architect skill. This report is a code review and does not constitute a formal security audit. Production deployment requires independent third-party audit.*
