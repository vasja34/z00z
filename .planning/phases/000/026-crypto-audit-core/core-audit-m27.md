# Z00Z Core Cryptographic Security Audit Report

**Report ID:** core-audit-m27  
**Date:** 2026-03-26  
**Auditor:** GitHub Copilot (MiniMax M2)  
**Scope:** `crates/z00z_core/src/` (excluding Tari vendor code)  
**Files Reviewed:** 29 source files

---

## 1. Executive Verdict

**Verdict:** **Risky but Salvageable**

The z00z_core crate implements reasonable cryptographic primitives using Tari's audited crypto library. Domain separation is well-designed with network-specific domains preventing cross-network replay. However, several S2 findings and undefined/nullifier semantics in the transaction layer require clarification before production deployment. The codebase demonstrates good security hygiene overall (overflow protection, canonical encoding for scalars) but has a few areas where timing side-channels or encoding issues could emerge.

**Confidence Level:** High (95%) — Files read in full; findings based on direct code analysis. Some transaction validation semantics (nullifier construction, fee binding in proofs) are deferred to upper layers not reviewed here.

---

## 2. Scope & Files Reviewed

| File | Lines | Purpose |
|------|-------|---------|
| `lib.rs` | ~100 | Public API facade, hash type aliases |
| `hashing.rs` | ~70 | Hash domain aliases |
| `domains.rs` | ~50 | Domain separation tag definitions |
| `assets/blinding.rs` | ~120 | Blinding factor generation |
| `assets/commitment.rs` | ~20 | Commitment construction/verification |
| `assets/secret.rs` | ~80 | Asset secret generation |
| `assets/nonce.rs` | ~350 | Nonce derivation strategies |
| `assets/serial_id.rs` | ~90 | Serial ID serialization |
| `assets/definition.rs` | ~300 | Asset definition structure |
| `assets/registry.rs` | ~400 | Global asset registry |
| `assets/amount.rs` | ~10 | Amount type alias |
| `assets/gas.rs` | ~250 | Gas calculation with overflow protection |
| `assets/policy_flags.rs` | ~120 | Policy flag constants |
| `assets/snapshot.rs` | ~150 | Registry versioning |
| `assets/assets.rs` | ~400 | Core Asset struct |
| `assets/leaf.rs` | ~200 | AssetLeaf + AssetPackPlain |
| `assets/wire.rs` | ~400 | Wire format serialization |
| `assets/wire_pkg.rs` | ~350 | JSON DTO encoding/decoding |
| `assets/wire_tests.rs` | ~200 | Wire format tests |
| `assets/version.rs` | ~80 | AssetPackVersion detection |
| `assets/assets_config.rs` | ~200 | Config parsing helpers |
| `genesis/genesis.rs` | ~500 | Core genesis generation |
| `genesis/genesis_config.rs` | ~250 | YAML config parsing |
| `genesis/asset_std.rs` | ~100 | Devnet test utilities |
| `genesis/validator.rs` | ~400 | Genesis validation + batch proofs |
| `genesis/serde.rs` | ~200 | Genesis serialization |
| `genesis/mod.rs` | ~50 | Module facade |
| `state/mod.rs` | ~1 | State module (empty) |

**Excluded:** `tari/` vendor directory, `tx/` transaction module (not in scope for this audit phase).

---

## 3. Security Goals Assumed

The z00z_core crate is intended to provide:

1. **Confidential multi-asset transactions** via Pedersen commitments hiding amounts
2. **Range proof soundness** via Bulletproofs+ proving amount ∈ [0, 2^64)
3. **Unique nonces** preventing transaction linkage/traceability
4. **Deterministic genesis** reproducible across all nodes
5. **Fee binding** preventing fee malleability attacks
6. **Asset policy enforcement** (mintable, burnable, fungible flags)
7. **Cross-network isolation** via domain-separated hash functions
8. **DoS resilience** via gas metering and overflow protection

---

## 4. Threat Model Summary

| Adversary | Capability | Threat |
|-----------|------------|--------|
| Network observer | View on-chain data | Link transactions via nonce reuse, weak blinding |
| Malicious validator | Control block production | Accept invalid proofs, censor specific assets |
| Cartel of >50% stake | Double-spend, fork choice manipulation | Replay genesis state, reorder transactions |
| Side-channel attacker | Timing measurement | Extract commitment openings via timing |
| RNG failure attacker | Predict or manipulate RNG | Predict blinding factors, forge range proofs |
| Malicious config loader | Supply invalid YAML | Bypass policy flags, create unsound genesis |
| Prover (transaction creator) | Create proofs | Forge proof for amount outside range |
| Light client | Limited verification | Accept invalid commitments or proofs |

**Trust Boundaries:**
- Tari crypto library (vendored) — assumed secure
- System RNG (`SystemRngProvider`) — assumed cryptographically secure
- Genesis seed — validated against M1 entropy requirements
- Upper layers (transaction validation, state management) — not in scope

---

## 5. S0/S1 Findings

**No S0 or S1 (CRITICAL/HIGH) findings in the reviewed files.**

Rationale: The crate delegates crypto to `z00z_crypto` (Tari) which is assumed audited. No custom crypto, no weak RNG, no nonce reuse patterns, no missing domain separation. All arithmetic has overflow protection.

---

## 6. S2/S3/S4 Findings

### S2 — Commitment Verification Not Constant-Time

**File:** `assets/commitment.rs:17`  
**Problem:** `verify_commitment_opening` uses `==` byte comparison:

```rust
pub fn verify_commitment_opening(
    commitment: &Z00ZCommitment,
    amount: u64,
    blinding: &Z00ZScalar,
) -> Result<bool, CryptoError> {
    let expected = create_commitment(amount, blinding)?;
    Ok(expected.as_bytes() == commitment.as_bytes())
}
```

**Impact:** Timing side-channel could leak whether a commitment opening is correct. An attacker with precise timing measurements could determine if a specific amount/blinding pair matches a commitment.

**Fix:**
```rust
use subtle::ConstantTimeEq;
// ...
Ok(expected.as_bytes().ct_eq(commitment.as_bytes()).into())
```

---

### S2 — Unvalidated Asset ID in Definition Constructor

**File:** `assets/definition.rs:180` (`AssetDefinition::new`)  
**Problem:** The `id` field is accepted as a parameter without validation that it was derived correctly from domain || name || symbol || version.

**Impact:** A caller could create an `AssetDefinition` with an arbitrary `id` that doesn't match the asset's metadata, breaking the asset identity guarantee. This is mitigated if `id` is always derived by the caller using proper domain-separated hashing, but there's no enforcement.

**Fix:** Add a static validator or derive `id` internally:
```rust
impl AssetDefinition {
    /// Validate that id is correctly derived from asset metadata
    pub fn validate_id(&self) -> Result<(), AssetError> {
        let expected = Self::derive_id(/* ... */)?;
        if self.id != expected {
            return Err(AssetError::InvalidAsset(
                "id does not match derived value".into()
            ));
        }
        Ok(())
    }
}
```

---

### S2 — Signature Scalar Parsing Without Canonicality Check

**File:** `assets/wire_pkg.rs:80-90` (`sig_from_hex`)  
**Problem:** When deserializing `KernelSignature`, the scalar bytes are parsed using `Z00ZScalar::from_canonical_bytes`. However, Ed25519 signatures permit non-canonical scalar encodings (scalar + multiple of group order). If the underlying crypto library doesn't reject non-canonical encodings, this could enable signature malleability.

**Impact:** A malleable signature could be transformed to another valid signature on the same message, potentially enabling replay or fork merging attacks.

**Fix:** Verify the crypto library (Tari) rejects non-canonical scalar encodings. If not, add explicit check:
```rust
fn scalar_from_canonical(bytes: [u8; 32]) -> Result<Z00ZScalar, AssetError> {
    Z00ZScalar::from_canonical_bytes(&bytes)
        .ok_or_else(|| AssetError::InvalidSignature("non-canonical scalar".into()))
}
```

---

### S3 — AssetPackPlain::decode_strict Doesn't Validate Blinding Canonicality

**File:** `assets/leaf.rs:90-110`  
**Problem:** `decode_strict` only checks length, not whether blinding bytes are a canonical scalar encoding. The `decode_checked` method exists for canonicality validation but `from_bytes` calls `decode_strict` directly.

**Impact:** Non-canonical blinding factors could enter the system. If the Pedersen commitment code doesn't validate canonicality, this could cause inconsistent behavior.

**Fix:**
```rust
pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
    Self::decode_checked(bytes).ok()
}
```

Or rename `decode_strict` to `decode_unchecked` and make `decode_checked` the default.

---

### S3 — LE Serial ID Encoding Convention Not Prominently Documented

**File:** `assets/serial_id.rs:25-30`  
**Problem:** Serial ID uses little-endian byte encoding (`serial_id.to_le_bytes()`). This is the correct choice for cross-platform consistency but is not explicitly documented.

**Impact:** Developers might assume BE encoding (common in crypto protocols) and create incompatible implementations.

**Fix:** Add doc comment:
```rust
/// 4-byte serial ID in LITTLE-ENDIAN byte order (cross-platform standard).
/// This ensures consistent serialization across architectures.
pub const SERIAL_ID_BYTE_LEN: usize = 4;
```

---

### S3 — Lock Ordering Assertions Only in Debug Builds

**File:** `assets/registry.rs:120-140` (lock ordering comments)  
**Problem:** The lock ordering rule (`definitions` → `version`) is documented with `#[cfg(debug_assertions)]` checks only.

**Impact:** In release builds, lock ordering violations could cause deadlocks with no warning.

**Fix:** Consider adding runtime validation in debug and optionally in release with a feature flag:
```rust
#[cfg(not(debug_assertions))]
const LOCK_ORDER_CHECKS: bool = false;
```

---

### S4 — Range Proof RNG Entropy Not Documented

**File:** `assets/assets.rs` (Asset::new range proof generation)  
**Problem:** The range proof generation uses `OsRng` (or caller-provided RNG). The security of Bulletproofs+ depends on cryptographic RNG quality, but there's no documentation of minimum entropy requirements.

**Impact:** Low entropy RNG could produce weakened range proofs that are still accepted by validators.

**Fix:** Document the requirement:
```rust
/// Generates a range proof for the committed amount.
///
/// # Security Note
/// Requires cryptographically secure RNG with minimum 128 bits entropy.
/// Using a weak RNG (e.g., `rand::thread_rng()` without OS entropy source)
/// could produce unsound proofs.
pub fn generate_range_proof(&mut self, rng: impl CryptoRng) -> Result<(), AssetError> { ... }
```

---

### S4 — Deterministic RNG for Genesis Not Audited for Cryptographic Suitability

**File:** `genesis/asset_std.rs:30-40` (`DeterministicRngProvider`)  
**Problem:** Genesis uses a deterministic RNG seeded from `genesis_seed || asset_id || serial_id`. While this is designed for reproducibility, the suitability of `DeterministicRngProvider` for cryptographic operations (particularly range proof generation) is not verified.

**Impact:** If the deterministic RNG has biases, genesis range proofs could be weakened.

**Fix:** Document that `DeterministicRngProvider` passes statistical tests and is suitable for genesis-only use (not for wallet operations).

---

### S4 — Gas Calculation Implicitly Assumes Fee Asset is Coin

**File:** `assets/gas.rs:180-200` (`GasAsset` validation)  
**Problem:** The `GasAsset` validation checks `is_gas` flag but the actual fee calculation (`calculate_fee`) appears to be an addendum to the transaction rather than bound within the proof statement.

**Impact:** Fee could be malleated post-proof unless the fee amount is explicitly included in the range proof statement at a higher layer.

**Note:** This is partially mitigated if the transaction layer binds fees before proof generation. Requires confirmation from transaction layer review.

---

## 7. Open Ambiguities

| # | Ambiguity | Impact | Evidence Needed |
|---|-----------|--------|----------------|
| 1 | **Nullifier construction semantics** | Cannot verify nullifier uniqueness without knowing how nullifiers are derived from (asset_id, serial_id, amount, nonce) | Nullifier derivation function from tx layer |
| 2 | **Fee binding in range proofs** | Cannot confirm fees are committed in proof statement vs. plaintext addenda | Proof statement construction from tx layer |
| 3 | **State management** | `state/mod.rs` is empty — state tree implementation is in storage layer | Confirm state tree uses proper hash domain |
| 4 | **Bulletproofs+ parameter binding** | Cannot verify range check maximum matches `MAX_AMOUNT` from genesis config | Confirmation that `RANGE_PROOF_BITS_V1` matches amount range |
| 5 | **Transaction validation ordering** | Unknown if semantic validation (serial bounds, nonce uniqueness) happens before or after proof verification | Transaction validation flow diagram |

---

## 8. Concrete Fixes

### Fix 1: Constant-Time Commitment Verification

**File:** `assets/commitment.rs`

```rust
// OLD (line 17):
Ok(expected.as_bytes() == commitment.as_bytes())

// NEW:
use subtle::ConstantTimeEq;
Ok(expected.as_bytes().ct_eq(commitment.as_bytes()).into())
```

Add `subtle` crate dependency if not already present.

---

### Fix 2: Validate Asset ID Derivation

**File:** `assets/definition.rs`

```rust
impl AssetDefinition {
    /// Derive asset ID from metadata using domain-separated hashing.
    pub fn derive_id(
        domain: [u8; 32],
        name: &str,
        symbol: &str,
        version: u8,
    ) -> [u8; 32] {
        let mut hasher = DomainHasher::<AssetIdHashDomain>::new();
        hasher.update(&domain);
        hasher.update(name.as_bytes());
        hasher.update(symbol.as_bytes());
        hasher.update(&[version]);
        let hash = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash.as_ref()[..32]);
        id
    }

    /// Validate that id matches derived value.
    pub fn validate_id(&self) -> Result<(), AssetError> {
        let expected = Self::derive_id(
            [self.class.class_byte()],
            &self.name,
            &self.symbol,
            self.version,
        );
        if self.id != expected {
            return Err(AssetError::Integrity(
                "AssetDefinition id does not match derived value".into()
            ));
        }
        Ok(())
    }
}
```

---

### Fix 3: Canonical Scalar Validation for Signatures

**File:** `assets/wire_pkg.rs`

```rust
fn scalar_from_canonical(bytes: [u8; 32]) -> Result<Z00ZScalar, AssetError> {
    Z00ZScalar::from_canonical_bytes(&bytes).ok_or_else(|| {
        AssetError::InvalidSignature(
            "non-canonical scalar encoding in signature".into()
        )
    })
}
```

Replace direct `from_canonical_bytes` calls with this wrapper.

---

### Fix 4: Make decode_checked the Default

**File:** `assets/leaf.rs`

```rust
/// Decode payload and validate blinding encoding.
pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
    Self::decode_checked(bytes).ok()
}
```

Rename `decode_strict` to `decode_length_only` and make the semantics clearer in comments.

---

### Fix 5: Document Serial ID Byte Order

**File:** `assets/serial_id.rs`

```rust
/// Serial ID byte length (4 bytes).
///
/// # Byte Order
/// Serial IDs are encoded in **little-endian** byte order for consistent
/// cross-platform serialization. This is different from some blockchain
/// protocols that use big-endian; ensure compatibility when integrating
/// with external systems.
pub const SERIAL_ID_BYTE_LEN: usize = 4;
```

---

## 9. Test Plan

### Positive Tests (Should Pass)

| Test | Purpose | Location |
|------|---------|----------|
| `test_commitment_roundtrip` | Verify `commit_amount` → `verify_commitment_opening` succeeds for valid pairs | `commitment.rs` |
| `test_blinding_uniqueness` | 100 batch blinding factors are all distinct | `blinding.rs` |
| `test_hidden_wrapping` | `Hidden::zeroize()` actually zeros memory | `blinding.rs` |
| `test_nonce_derivation_deterministic` | Same inputs produce same nonce | `nonce.rs` |
| `test_serial_id_roundtrip` | serialize → deserialize is identity | `serial_id.rs` |
| `test_asset_pack_encoding` | 72-byte format encodes/decodes correctly | `leaf.rs` |
| `test_genesis_reproducibility` | Same seed produces identical genesis | `genesis.rs` |
| `test_gas_overflow_rejected` | Exceeding MAX_INPUTS/MAX_OUTPUTS returns error | `gas.rs` |

### Negative Tests (Should Fail)

| Test | Purpose | Expected |
|------|---------|----------|
| `test_zero_blinding_rejected` | All-zero blinding factor rejected | `blinding.rs` |
| `test_zero_nonce_rejected` | All-zero nonce detected by validator | `nonce.rs` |
| `test_serial_out_of_bounds` | serial_id ≥ serials rejected | `serial_id.rs` |
| `test_malformed_blinding` | Non-canonical blinding rejected | `leaf.rs` |
| `test_low_entropy_seed_rejected` | Weak genesis seed rejected | `genesis.rs` |
| `test_non_fungible_decimal_nonzero` | NFT/Void with decimals > 0 rejected | `definition.rs` |

### Misuse Tests (Should Handle Gracefully)

| Test | Scenario | Expected |
|------|----------|----------|
| N/A for core | Concurrent registry access | RwLock prevents data races |
| N/A for core | Double-spend nonce | Detected at tx validation layer |
| N/A for core | Negative amount | u64 prevents; range proof enforces |

### Wycheproof-equivalent Tests

| Test | Standard | Source |
|------|----------|--------|
| `test_scalar_canonical` | RFC 8032 §5.1 | Verify Ed25519 library rejects non-canonical scalars |
| `test_commitment_binding` | Custom | Verify commitment opening fails for wrong (amount, blinding) |
| `test_domain_collision` | Custom | Verify different domains produce different outputs for same input |

---

## 10. Confidence Levels

| Claim | Confidence | Evidence |
|-------|------------|----------|
| Domain separation is correct | 95% | All domains are unique strings with version suffixes; network-specific domains for genesis |
| Blinding factors are cryptographically random | 90% | Uses `Z00ZScalar::random` with CSPRNG fallback after 64 attempts |
| Serial ID encoding is canonical | 95% | LE encoding is explicit; bounds check enforced |
| Overflow protection is complete | 95% | All gas calculations use `checked_add`/`checked_mul`; pre-checks on limits |
| Range proof generation is sound | 85% | Delegates to Tari Bulletproofs+; RNG quality not independently verified |
| Genesis is deterministic | 90% | Seed derivation uses domain separation; M1 validation present |
| Commitment verification is timing-safe | 70% | Uses `==` not constant-time (S2 finding) |
| Nullifier uniqueness | Unknown | Nullifier construction not visible in this scope |
| Fee binding in proofs | Unknown | Gas calculations present; fee in proof statement not verified |

---

## 11. Final Decision

| Status | Condition |
|--------|-----------|
| ✅ **Execution Ready** | S2 findings fixed (commitment timing, signature canonicality, ID validation) |
| ⏸️ **Blocked** | Nullifier semantics clarified and confirmed |
| ⏸️ **Blocked** | Fee binding in range proof statements confirmed |
| ⏸️ **Blocked** | State module implementation reviewed (currently empty) |

**Required before production:**
1. Fix S2 commitment timing issue (add `subtle::ConstantTimeEq`)
2. Add asset ID self-validation or document caller responsibility
3. Verify Ed25519 library rejects non-canonical scalars
4. Clarify nullifier derivation in transaction layer documentation
5. Confirm fee amount is bound in range proof statement (not plaintext addendum)

**Advisory (address before next audit):**
- Document serial ID byte order convention
- Add blinding canonicality check to `from_bytes`
- Consider runtime lock ordering validation for release builds

---

## 12. Summary Table

| Severity | Count | Findings |
|----------|-------|----------|
| S0 CRITICAL | 0 | None |
| S1 HIGH | 0 | None |
| S2 MEDIUM | 3 | Commitment timing, unvalidated ID, signature canonicality |
| S3 LOW | 3 | Blinding check gap, LE docs, lock ordering |
| S4 INFO | 3 | RNG docs, genesis RNG, fee binding ambiguity |
| **Total** | **12** | — |

---

*End of Report*
