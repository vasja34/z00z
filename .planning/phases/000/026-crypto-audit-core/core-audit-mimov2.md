# Crypto Audit Report: `z00z_core` Implementation

**Date:** 2026-03-26  
**Scope:** `crates/z00z_core/src/**/*.rs` (implementation only, excluding `tari/` vendor)  
**Model:** Xiaomi MiMo-V2-Pro  
**Skill:** crypto-architect  

---

## Executive Verdict

**`Risky but salvageable`** — S1 findings present; concrete fixes exist and are documented below. No S0 (fundamentally broken) findings, but several S1 issues require remediation before production deployment.

---

## Input Type and Scope

- **Input type:** Source code — implementation review mode
- **Scope:** All `.rs` files in `crates/z00z_core/src/` including `assets/`, `genesis/`, `hashing.rs`, `domains.rs`, `lib.rs`, and `state/mod.rs`
- **Exclusions:** `crates/z00z_crypto/tari/` (vendor code), documentation files, non-Rust artifacts

---

## Security Goals Assumed

| Goal | Status | Evidence |
| --- | --- | --- |
| Confidentiality of amounts | ✅ | Pedersen commitments + Bulletproofs+ range proofs |
| Binding (commitment integrity) | ✅ | Pedersen binding property |
| Nonce uniqueness (unlinkability) | ⚠️ | Enforced at validation layer, not construction |
| Domain separation | ✅ | Comprehensive `hash_domain!` usage |
| Secret lifecycle (zeroization) | ✅ | `Hidden<T>`, `SecretBytes32`, `Zeroize` derives |
| Owner signature integrity | ✅ | Schnorr over canonical message |
| Stealth address privacy | ⚠️ | Partial — see S1 findings |
| Fee privacy | ❌ | Fees not included in commitment proof statement |

---

## Threat Model Summary

**Adversaries:**

- Passive chain observer (amount/linkability analysis)
- Malicious sender (forged commitments, replayed nonces)
- Compromised wallet (key extraction, nonce reuse)
- Front-runner (partial transaction data leakage)

**Trust boundaries:**

- `z00z_core` → `z00z_crypto` (backend abstraction)
- `z00z_core` → `z00z_utils` (I/O, time, RNG abstractions)
- Genesis seed → deterministic asset generation
- Wallet seed → nonce derivation

**Failure assumptions:**

- Backend (Tari) primitives are correctly implemented
- System RNG is available and cryptographically secure
- Database transactions persist nonce counters atomically

---

## Critical and High Findings (S0/S1)

### S1-01: Fee Amount Not Included in Proof Statement

| Field | Content |
| --- | --- |
| Severity | S1 HIGH |
| Component | `crates/z00z_core/src/assets/gas.rs`, `crates/z00z_core/src/assets/assets.rs` |
| Problem | The `GasSchedule` calculates fees as abstract `GasUnit` values, but there is no evidence that fee amounts are committed within the Pedersen commitment or included in the range proof statement. The `Asset` struct has no dedicated fee field, and `verify_complete()` does not verify fee binding. |
| Impact | An attacker could declare a fee amount that differs from the actual fee consumed, enabling fee theft or inflation attacks. In confidential transaction systems, fees MUST be part of the proof statement to prevent amount malleability. |
| Fix | Add a `fee_amount` field to the transaction proof statement. Ensure the range proof covers `amount + fee` or that a separate fee commitment is verified homomorphically. Reference: Mimblewimble/Tari protocol fee commitment model. |

**Confidence:** High — code inspection confirms no fee binding in commitment or proof paths.

---

### S1-02: Nonce Zero Check Only in Production Builds

| Field | Content |
| --- | --- |
| Severity | S1 HIGH |
| Component | `crates/z00z_core/src/assets/assets.rs` (lines ~1250-1260) |
| Problem | The zero-nonce check in `validate()` is gated behind `#[cfg(not(test))]`. While this is intentional for test convenience, it means that any code compiled with `test` feature (including integration tests that may ship) will accept zero nonces. The `test-fast` feature is used in CI and could mask production bugs. |
| Impact | Zero nonces break unlinkability — all assets with zero nonce become linkable by chain observers. If a test build leaks into production or if `test-fast` is accidentally enabled, privacy is compromised. |
| Fix | Move the zero-nonce check to `Asset::new()` constructor (unconditional), or add a runtime assertion that fires in all builds. Keep the `#[cfg(not(test))]` only for the error message variant, not the check itself. |

**Confidence:** High — code inspection confirms the conditional compilation gate.

---

### S1-03: Stealth Address — Missing View Tag Validation

| Field | Content |
| --- | --- |
| Severity | S1 HIGH |
| Component | `crates/z00z_core/src/assets/leaf.rs`, `crates/z00z_core/src/assets/assets.rs` |
| Problem | The `AssetLeaf` struct contains `tag16: u16` for stealth filtering, but there is no validation that `tag16` is correctly derived from the ECDH shared secret. The `validate_stealth_consistency()` method checks presence/absence of fields but does not verify the cryptographic binding between `tag16` and the ECDH output. |
| Impact | A malicious sender could provide an arbitrary `tag16` value that doesn't match the ECDH derivation, causing the recipient to either miss the output (denial of service) or accept a misdirected output. The view tag must be a deterministic function of the shared secret to prevent metadata leakage. |
| Fix | Add a `verify_tag16(&self, view_sk: &Z00ZScalar)` method that recomputes `tag16` from the ECDH shared secret (`k_dh = view_sk * R_pub`) and compares it to the stored value. Reference: EIP-5564 stealth address specification. |

**Confidence:** Medium — the tag16 derivation logic may exist in `z00z_wallets` but is not validated at the core layer.

---

### S1-04: `derive_asset_secret` Uses Poseidon2 Without Domain Tag Verification

| Field | Content |
| --- | --- |
| Severity | S1 MEDIUM-HIGH |
| Component | `crates/z00z_core/src/assets/assets.rs` (line ~830) |
| Problem | `derive_asset_secret(s_out)` calls `poseidon2_hash(b"Z00Z/ASSET_SECRET", &[s_out])`. The domain tag `"Z00Z/ASSET_SECRET"` is a raw byte string, not a `hash_domain!` macro invocation. This bypasses the domain separation framework and could collide with other Poseidon2 usages if the same tag is reused elsewhere. |
| Impact | Cross-context collision if another module uses the same Poseidon2 tag for a different purpose. While Poseidon2 is ZK-friendly, its security relies on proper domain separation. |
| Fix | Define a dedicated `hash_domain!` for Poseidon2-based derivations, or use the existing `hash_zk` module which provides domain-separated Poseidon2 hashing. Replace the raw byte string with a domain-separated call. |

**Confidence:** High — code inspection confirms raw byte string usage.

---

## Medium and Low Findings (S2/S3/S4)

### S2-01: `NonceCounter::increment_unsafe` Debug Warning Only

| Field | Content |
| --- | --- |
| Severity | S2 MEDIUM |
| Component | `crates/z00z_core/src/assets/nonce.rs` |
| Problem | The `increment_unsafe` method prints a warning to stderr in debug builds but has no compile-time or runtime enforcement that it's called within a database transaction. The `#[must_use]` attribute helps but doesn't prevent misuse. |
| Impact | If called outside a database transaction, a wallet restart could cause nonce reuse, breaking privacy. |
| Fix | Consider a transaction-scoped API: `fn increment_within_tx(&mut self, tx: &DbTransaction) -> Result<u64, AssetError>` that ties the counter increment to the transaction lifecycle. |

---

### S2-02: `AssetMetadata::compute_hash` Uses `DomainHasher` Without Length Prefixing

| Field | Content |
| --- | --- |
| Severity | S2 MEDIUM |
| Component | `crates/z00z_core/src/assets/assets.rs` (lines ~370-390) |
| Problem | `compute_hash` iterates over `custom_fields` and calls `hasher.update(key.as_bytes())` and `hasher.update(value.as_bytes())` in a loop. While `DomainSeparatedHasher` adds length prefixes automatically, the iteration order depends on `BTreeMap` ordering (which is deterministic), but there's no explicit separator between key-value pairs. |
| Impact | Low — `BTreeMap` ordering is deterministic and `DomainSeparatedHasher` handles length prefixing. However, if the hasher implementation changes, metadata hash stability could break. |
| Fix | Add an explicit separator byte between key-value pairs, or document that `DomainSeparatedHasher.update()` provides sufficient separation. |

---

### S2-03: `MAX_AMOUNT` Set to `u64::MAX`

| Field | Content |
| --- | --- |
| Severity | S2 LOW |
| Component | `crates/z00z_core/src/assets/amount.rs` |
| Problem | `MAX_AMOUNT` is set to `u64::MAX` (18.4 quintillion). While range proofs cover this range, the protocol nominal values are much smaller (e.g., 100,000,000 for coins). Allowing `u64::MAX` as a valid amount could enable overflow attacks in homomorphic operations. |
| Impact | If two assets with amounts near `u64::MAX` are added homomorphically, the result overflows. The range proof prevents individual overflow, but aggregate operations could still overflow. |
| Fix | Set `MAX_AMOUNT` to a protocol-defined limit (e.g., `serials * nominal * 2` for safety margin) and enforce it in `validate_construction_params`. |

---

### S2-04: `Asset::new_confidential` Returns `Hidden<Z00ZScalar>` Without Usage Guidance

| Field | Content |
| --- | --- |
| Severity | S2 LOW |
| Component | `crates/z00z_core/src/assets/assets.rs` |
| Problem | `new_confidential` generates a random blinding and returns it as `Hidden<Z00ZScalar>`. The caller is responsible for persisting this blinding for later spending. If the caller discards the blinding, the asset becomes unspendable. |
| Impact | User funds locked permanently if blinding is lost. |
| Fix | Add documentation emphasizing that the returned `Hidden<Z00ZScalar>` MUST be persisted in the wallet database before the asset is broadcast. |

---

### S3-01: `state/mod.rs` Is Empty

| Field | Content |
| --- | --- |
| Severity | S3 INFO |
| Component | `crates/z00z_core/src/state/mod.rs` |
| Problem | The `state` module is declared in `lib.rs` but contains only whitespace. This suggests incomplete implementation. |
| Impact | None currently — module is unused. |
| Fix | Either implement the epoch-based state management described in `lib.rs` documentation, or remove the module declaration. |

---

### S3-02: `AssetError` Has Both String-Based and Structured Variants

| Field | Content |
| --- | --- |
| Severity | S3 INFO |
| Component | `crates/z00z_core/src/assets/assets.rs` |
| Problem | `AssetError` contains both legacy string-based errors (`InvalidCommitment(Cow<'static, str>)`) and new structured errors (`InvalidSerialIdStructured { ... }`). This creates API inconsistency. |
| Impact | Callers may match on string-based variants and miss structured context. |
| Fix | Migrate all string-based error variants to structured forms in a future major version. |

---

## Open Ambiguities

1. **Fee commitment model:** The exact mechanism for binding fees to the proof statement is not visible in `z00z_core`. This may be handled in `z00z_wallets` or `z00z_storage`, but the core layer should at least define the fee commitment interface.

2. **Nullifier construction:** The `z00z_core` crate does not contain nullifier generation or verification logic. This is likely in `z00z_storage` or `z00z_wallets`, but the core layer should define the nullifier derivation formula for consistency.

3. **Batch verification scope:** `verify_genesis_assets` uses batch verification, but it's unclear whether transaction-level batch verification is implemented elsewhere.

---

## Concrete Fixes

### Fix S1-01: Fee Binding

```rust
// In Asset struct, add:
pub fee_commitment: Option<Commitment>,

// In verify_complete(), add:
if let Some(fee_commit) = &self.fee_commitment {
    // Verify fee commitment is valid Pedersen commitment
    // Verify fee is included in proof statement
}
```

### Fix S1-02: Unconditional Nonce Check

```rust
// In Asset::new(), add BEFORE creating crypto components:
if nonce == [0u8; 32] {
    return Err(AssetError::InvalidAsset(Cow::Borrowed(
        "Zero nonce forbidden. Use derive_nonce() to generate secure nonces."
    )));
}
```

### Fix S1-03: View Tag Verification

```rust
// In AssetLeaf or Asset, add:
pub fn verify_tag16(&self, view_sk: &Z00ZScalar) -> Result<bool, AssetError> {
    let r_pub = safe_decompress_point(&self.r_pub)?;
    let k_dh = compute_stealth_dh_sender(view_sk, &r_pub)?;
    let expected_tag = derive_tag16(&k_dh);
    Ok(expected_tag == self.tag16)
}
```

### Fix S1-04: Domain-Separated Poseidon2

```rust
// Replace:
z00z_crypto::hash::poseidon2_hash(b"Z00Z/ASSET_SECRET", &[s_out])

// With:
use z00z_crypto::hash_zk::hash_zk;
hash_zk("Z00Z/ASSET_SECRET", &[s_out])
```

---

## Implementation Guidance

### Safe Architecture

1. **Fee model:** Adopt the Mimblewimble fee commitment model where fees are committed in a separate Pedersen commitment and verified homomorphically against the transaction balance equation.

2. **Nonce policy:** Enforce nonce uniqueness at construction time (not just validation time) to prevent accidental reuse in test builds.

3. **Stealth validation:** Move tag16 verification to the core layer so that all stealth outputs are validated consistently regardless of the wallet implementation.

4. **Domain separation:** Use `hash_domain!` macro for all new hash derivations, including Poseidon2-based ZK hashes.

### Test Plan

| Test Type | Coverage |
| --- | --- |
| Positive | Asset creation, commitment verification, range proof generation/verification, owner signature |
| Negative | Zero nonce rejection, invalid serial_id, tampered range proof, wrong owner signature |
| Misuse | Nonce reuse detection, fee malleability detection, partial stealth field rejection |
| Property | Commitment homomorphism, asset_id determinism, blinding factor uniqueness |
| Wycheproof | Ed25519/Ristretto signature verification against known test vectors |

---

## Confidence Level

| Claim | Confidence | Evidence That Would Change It |
| --- | --- | --- |
| Fee not bound in proof statement | High | Documentation showing fee commitment in `z00z_wallets` or `z00z_storage` |
| Zero nonce accepted in test builds | High | Code inspection confirms `#[cfg(not(test))]` gate |
| Stealth tag16 not validated | Medium | Tag derivation may exist in `z00z_wallets` |
| Poseidon2 domain separation weak | High | Code inspection confirms raw byte string |
| No nullifier logic in core | High | `grep` confirms no nullifier references in `z00z_core` |

---

## Final Decision

**`Blocked: [S1-01 fee binding model, S1-02 nonce enforcement, S1-03 tag16 validation, S1-04 Poseidon2 domain separation]`**

All S1 findings have concrete fixes documented above. The implementation is structurally sound with good domain separation, proper zeroization, and comprehensive error handling. The primary gaps are in fee privacy, nonce enforcement scope, and stealth validation completeness.

---

**Report generated:** 2026-03-26  
**Next review:** After S1 fixes are applied
