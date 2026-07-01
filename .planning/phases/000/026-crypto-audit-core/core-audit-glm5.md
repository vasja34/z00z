# Z00Z Core Cryptographic Audit Report

**Auditor:** GLM-5 (Crypto Architect)  
**Date:** 2026-03-26  
**Scope:** `crates/z00z_core/src/**/*.rs` (excluding `z00z_crypto/tari/` vendor)  
**Input Type:** Implementation review (source code)  
**Total LOC:** ~14,225 Rust source lines across 33 files  

---

## 1. Executive Verdict

**`Risky but salvageable`** — No S0 (fundamental) findings. Two S1 findings require
attention before production deployment. The remaining findings are S2/S3/S4 and
documented with concrete fixes.

---

## 2. Scope and Security Goals

### 2.1 Security Goals Assumed

| Goal | Status | Notes |
|------|--------|-------|
| Confidentiality of transaction amounts | ✅ Active | Pedersen commitments + Bulletproofs+ |
| Commitment binding (amount ↔ blinding) | ✅ Active | Standard Mimblewimble/Tari construction |
| Range proof soundness (amount ∈ [0, 2^64)) | ✅ Active | Bulletproofs+ via z00z_crypto |
| Owner signature integrity | ✅ Active | Schnorr over domain-separated transcript |
| Nonce uniqueness (output privacy) | ⚠️ Partial | Derivation is correct; enforcement is deferred |
| Genesis determinism | ✅ Active | Domain-separated Blake2b derivations |
| Domain separation (cross-context isolation) | ✅ Active | 16 distinct hash domains |
| Replay protection (genesis state) | ✅ Active | State hash + consensus verification |
| Stealth address consistency | ✅ Active | All-or-nothing tuple validation |

### 2.2 Threat Model Summary

| Adversary | Capability | Mitigated |
|-----------|-----------|-----------|
| Passive chain observer | Sees commitments, proofs, nonces | ✅ Pedersen hiding |
| Malicious client | Submits crafted assets | ⚠️ Partial (see S1-1) |
| Malicious validator | Alters genesis state | ✅ State hash consensus |
| Cross-network attacker | Replays devnet assets on mainnet | ✅ Domain separation |
| Fee dodger | Omits or underpays fees | ✅ Gas schedule enforcement |

### 2.3 Trust Boundaries

- **z00z_core** trusts `z00z_crypto` for all primitive operations (commitments, proofs, signatures, hashing)
- **z00z_core** trusts `z00z_utils` for RNG, time, I/O, and codec abstractions
- **z00z_core** does NOT trust incoming `AssetWire` payloads from network
- **z00z_core** does NOT trust YAML configuration without validation

---

## 3. Findings

### 3.1 S1 — HIGH Severity

#### S1-1: `AssetWire::to_asset()` accepts `secret` field from network

| Field | Content |
|-------|---------|
| **Severity** | S1 |
| **Component** | `assets/wire.rs` → `AssetWire::to_asset()` |
| **Problem** | `AssetWire` struct includes `secret: Option<[u8; 32]>` and `to_asset()` copies it directly into the resulting `Asset` without filtering. The doc comment in `assets.rs` says "secret must never be accepted from incoming wire payloads; that restriction is enforced by import path gate before to_asset()", but no such gate exists in the reviewed code. Any caller of `to_asset()` that receives an `AssetWire` from the network will silently accept the `secret` field. |
| **Impact** | A malicious node or MITM can inject arbitrary `s_out` values into wallet state. If the wallet uses `s_out` for any cryptographic derivation (e.g., `Asset::derive_asset_secret(s_out)`), the attacker controls the derived secret. This breaks stealth address security. |
| **Fix** | Strip `secret` in `to_asset()`: |
| | ```rust |
| | pub fn to_asset(self) -> Result<Asset, AssetError> { |
| |     let arc_def = GLOBAL_ASSET_REGISTRY.insert(self.definition)?; |
| |     let asset = Asset { |
| |         // ...existing fields... |
| |         secret: None,  // NEVER accept from wire |
| |         // ...rest... |
| |     }; |
| |     asset.validate()?; |
| |     Ok(asset) |
| | } |
| | ``` |
| | Additionally, add a `validate()` check in `Asset` that rejects `secret: Some(_)` unless the asset was created locally (e.g., via `Asset::new()`). |

#### S1-2: `derive_nonce_simple` and `derive_nonce_minimal` silently fall back to timestamp=0

| Field | Content |
|-------|---------|
| **Severity** | S1 |
| **Component** | `assets/nonce.rs` → `derive_nonce_simple()`, `derive_nonce_minimal()` |
| **Problem** | Both functions call `try_get_timestamp_micros(time_provider).unwrap_or(0)` which silently replaces a time error with zero. If `TimeProvider` fails (e.g., system clock before Unix epoch, mocked environment), the nonce is derived with `timestamp=0`. Two calls at different times with the same seed+counter would produce the same nonce if both fail to get time. The fallible variants (`try_derive_nonce_simple`, `try_derive_nonce_minimal`) exist but are not the default path. |
| **Impact** | Nonce reuse if time provider fails consistently. Nonce reuse = total privacy loss for affected outputs. An attacker who can trigger time provider failures can force nonce reuse. |
| **Fix** | 1. Deprecate the infallible variants or mark them `#[deprecated]` with guidance to use `try_*` variants. 2. In production builds (`#[cfg(not(test))]`), make the infallible variants panic or return `Result` instead of silently falling back. 3. Document that `derive_nonce_simple` and `derive_nonce_minimal` are test-only APIs. |

### 3.2 S2 — MEDIUM Severity

#### S2-1: `Asset::asset_id()` does not include `amount` in the hash

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `assets/assets.rs` → `Asset::asset_id()` |
| **Problem** | `asset_id` is derived from `nonce || commitment || definition_id || serial_id`. Since `commitment` already binds `amount` via Pedersen commitment, this is technically sound. However, `asset_id` is used for "spend tracking" and if two assets share the same nonce, commitment, definition, and serial_id but differ in mutable state (e.g., `is_burned`), they produce the same `asset_id`. This could cause confusion in spend tracking if burned and unburned variants of the same output coexist. |
| **Impact** | Potential spend-tracking ambiguity for outputs that differ only in mutable state flags. |
| **Fix** | Include `is_burned` in the `asset_id` derivation, or document that `asset_id` is defined over immutable fields only and that spend tracking must also check mutable state. |

#### S2-2: `Asset::to_owner_message()` encodes `lock_height: None` as `0u64`

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `assets/assets.rs` → `Asset::to_owner_message()` |
| **Problem** | `self.lock_height.unwrap_or(0).to_le_bytes()` means `None` and `Some(0)` produce identical signature messages. An asset with `lock_height = None` can be modified to `lock_height = Some(0)` without invalidating the owner signature. |
| **Impact** | A party holding a signed asset with `lock_height = None` can set it to `Some(0)` and the signature remains valid. Since `Some(0)` means "spendable immediately" (same as `None`), the practical impact is limited, but this violates the principle that signature should cover all mutable state. |
| **Fix** | Use a discriminant byte: encode `None` as `0xFF` prefix + 8 zero bytes, and `Some(v)` as `0x00` prefix + `v.to_le_bytes()`. This is a consensus-critical change — coordinate with any existing signed assets. |

#### S2-3: `generate_blinding()` has a 64-iteration rejection loop

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `assets/blinding.rs` → `generate_blinding()` |
| **Problem** | The function tries `Z00ZScalar::try_from_bytes()` up to 64 times with the provided RNG, then falls back to `SystemRngProvider`. If the provided RNG is deterministic (e.g., `ChaCha20Rng`), the fallback introduces non-determinism. The 64-iteration limit is arbitrary and may not be sufficient for biased RNGs. |
| **Impact** | Non-deterministic blinding in tests that use deterministic RNGs with high rejection rates. In production, the fallback is correct behavior. |
| **Fix** | 1. Document that `generate_blinding()` may fall back to system RNG. 2. Consider removing the fallback and returning an error instead, forcing callers to handle RNG failure explicitly. 3. If deterministic behavior is required, use `BlindingFactor::random()` directly with a known-good RNG. |

#### S2-4: `verify_genesis_consensus()` has placeholder `None` hashes

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `genesis/validator.rs` → `verify_genesis_consensus()` |
| **Problem** | `MAINNET_GENESIS_STATE_HASH`, `TESTNET_GENESIS_STATE_HASH`, and `DEVNET_GENESIS_STATE_HASH` are all `None`. This means genesis state verification is a no-op for all networks. The C2 security enhancement (genesis state integrity) is not enforced. |
| **Impact** | Before mainnet/testnet launch, this must be populated. Currently, any genesis state is accepted, enabling chain split attacks. |
| **Fix** | Populate the constants after final genesis generation. Add a compile-time or runtime check that fails if mainnet/testnet hashes are `None` in release builds. |

#### S2-5: `AssetMetadata::compute_hash()` does not include `metadata_hash` field itself

| Field | Content |
|-------|---------|
| **Severity** | S2 |
| **Component** | `assets/assets.rs` → `AssetMetadata::compute_hash()` |
| **Problem** | The hash covers `custom_fields` (sorted) and `timestamp`, but not the `metadata_hash` field itself. This is correct by design (hash of hash would be redundant), but the field name is misleading — it suggests the hash includes itself. |
| **Impact** | No direct security impact, but could confuse developers who expect `metadata_hash` to be included in its own computation. |
| **Fix** | Rename to `compute_metadata_hash()` or add a doc comment clarifying that `metadata_hash` is the output, not an input. |

### 3.3 S3 — LOW Severity

#### S3-1: `AssetDebug` implementation shows `nonce` in hex

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `assets/assets.rs` → `impl fmt::Debug for Asset` |
| **Problem** | Nonce is displayed in hex in Debug output. While nonces are not secret keys, they are privacy-relevant: knowing the nonce for an output could help an observer link transactions if the derivation scheme is known. |
| **Impact** | Minor privacy leak through debug logs. |
| **Fix** | Redact nonce in Debug output: `debug_struct.field("nonce", &"<present, 32 bytes>")`. |

#### S3-2: `Asset::new()` uses blinding factor as both commitment key and owner key

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `assets/assets.rs` → `Asset::new()` |
| **Problem** | The same `blinding` scalar is used for: (1) Pedersen commitment `C = amount·G + blinding·H`, and (2) owner public key `owner_pub = blinding·G`. This is documented as intentional Mimblewimble design. However, it means that anyone who can verify the commitment opening (i.e., who knows `blinding`) automatically has the owner key. This is standard Tari/Grin behavior but worth noting. |
| **Impact** | Standard Mimblewimble trade-off — not a vulnerability, but a design constraint. |
| **Fix** | No fix needed. Document the security implication clearly. Consider whether future versions should separate commitment blinding from ownership keys. |

#### S3-3: `NonceCounter` uses `eprintln!` for safety warning

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `assets/nonce.rs` → `NonceCounter::increment_unsafe()` |
| **Problem** | The debug assertion uses `eprintln!` which is not structured logging and may be missed in production log aggregation. |
| **Impact** | Warning may not reach monitoring systems. |
| **Fix** | Use `tracing::warn!()` or the `Logger` trait from `z00z_utils`. |

#### S3-4: `AssetPackPlain::from_bytes()` uses `Option` return instead of `Result`

| Field | Content |
|-------|---------|
| **Severity** | S3 |
| **Component** | `assets/leaf.rs` → `AssetPackPlain::from_bytes()` |
| **Problem** | `from_bytes()` returns `Option<Self>`, discarding the specific rejection reason. The `decode_strict()` and `decode_checked()` methods exist with proper error types, but `from_bytes()` is the primary API. |
| **Impact** | Callers cannot distinguish between "wrong length" and "malformed blinding" errors. |
| **Fix** | Deprecate `from_bytes()` in favor of `decode_checked()`. Update all call sites. |

### 3.4 S4 — INFO

#### S4-1: `MAX_AMOUNT` is `u64::MAX`

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `assets/amount.rs` |
| **Problem** | `MAX_AMOUNT = u64::MAX` means the range proof must prove `amount ∈ [0, 2^64)`. This is the full 64-bit range, which is correct for Bulletproofs+ with `RANGE_PROOF_BITS_V1 = 64`. However, it means there is no protocol-level cap on individual output amounts. |
| **Impact** | No direct security issue, but economic policy may want to cap individual amounts. |
| **Fix** | Consider whether a protocol-level amount cap (e.g., `u64::MAX / 2`) is needed for economic reasons. |

#### S4-2: `AssetDefinition::from_decimal()` uses `f64` arithmetic

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `assets/definition.rs` → `AssetDefinition::from_decimal()` |
| **Problem** | `f64` has ~15 decimal digits of precision. For assets with >15 decimal digits of precision in their smallest unit, this can lose precision. |
| **Impact** | Negligible for practical use (max 32 decimals, but f64 precision is sufficient for any realistic amount). |
| **Fix** | No fix needed for current use cases. Document the precision limitation. |

#### S4-3: Genesis blinding derivation uses `from_uniform_bytes` with 64-byte hash

| Field | Content |
|-------|---------|
| **Severity** | S4 |
| **Component** | `genesis/genesis.rs` → `derive_genesis_blinding()` |
| **Problem** | Uses Blake2b-512 output (64 bytes) for wide reduction to Ristretto255 scalar field. The modulo bias is < 2^-128, which is negligible. This is correct and well-documented. |
| **Impact** | None — this is an advisory note confirming correct implementation. |
| **Fix** | No fix needed. |

---

## 4. Open Ambiguities

| # | Question | Owner | Evidence Needed |
|---|----------|-------|-----------------|
| A1 | Is `AssetWire::to_asset()` the only import path, or are there other code paths that construct `Asset` from network data? | z00z_wallets / z00z_networks | Audit all `Asset` construction sites |
| A2 | What is the planned `crypto_version = 2` migration path? | Protocol team | Migration specification |
| A3 | Are there plans to separate commitment blinding from owner keys in a future version? | Protocol team | Design decision document |
| A4 | When will `MAINNET_GENESIS_STATE_HASH` be populated? | Genesis team | Final genesis config |
| A5 | Is `Asset::derive_asset_secret(s_out)` used in any spending protocol? | z00z_wallets | Usage audit |

---

## 5. Positive Observations

### 5.1 Domain Separation (EXCELLENT)

16 distinct `hash_domain!` declarations in `domains.rs` with:
- Network-aware separation (devnet/testnet/mainnet) for genesis operations
- Purpose-specific domains (metadata, owner_signature, nonce_derivation, etc.)
- Version tags (`.v1`) for forward compatibility
- No domain reuse across different semantic purposes

### 5.2 Pedersen Commitment + Bulletproofs+ (CORRECT)

- Standard Mimblewimble construction via `z00z_crypto`
- 64-bit range proofs with `MIN_VALUE_PROMISE`
- Batch verification support (O(log n))
- Commitment verification available

### 5.3 Owner Signature (WELL-DESIGNED)

- Schnorr signature over canonical message with domain separation
- Covers all critical fields (definition_id, serial_id, amount, commitment, nonce, lock_height, range_proof, state flags)
- Field ordering documented as consensus-critical
- Public key derived from blinding factor (standard Mimblewimble)

### 5.4 Genesis Security (STRONG)

- Shannon entropy validation (200 bits production, 140 bits devnet)
- Known test seed rejection
- Sequential pattern detection
- Deterministic derivation with network isolation
- State hash consensus verification (C2 — pending population)

### 5.5 Stealth Address Validation (CORRECT)

- All-or-nothing tuple enforcement (r_pub + owner_tag + enc_pack)
- tag16 requires full stealth fields
- leaf_ad_id requires full stealth fields
- secret field validation (import path gate — needs hardening per S1-1)

### 5.6 Error Handling (GOOD)

- Typed errors with context (structured error variants)
- No `unwrap()` in production paths
- `thiserror` for error definitions
- No silent fallback on crypto errors (except nonce timestamp — see S1-2)

### 5.7 Secret Lifecycle (GOOD)

- `Hidden<Z00ZScalar>` for blinding factors
- `Zeroize` support in tests
- Custom `Debug` that redacts sensitive fields
- No logging of secrets

### 5.8 Gas Calculation (CORRECT)

- Overflow protection with checked arithmetic
- Protocol limits (MAX_INPUTS, MAX_OUTPUTS, MAX_PROOF_BITS)
- Native coin enforcement for fee payment

---

## 6. Concrete Fixes Summary

| Priority | ID | Fix | Effort |
|----------|----|-----|--------|
| **P0** | S1-1 | Strip `secret` in `AssetWire::to_asset()` | S |
| **P0** | S1-2 | Deprecate infallible nonce functions or add `#[cfg(test)]` guard | S |
| P1 | S2-1 | Document `asset_id` scope or include `is_burned` | S |
| P1 | S2-2 | Add discriminant byte for `lock_height` encoding | M (consensus change) |
| P1 | S2-3 | Document fallback behavior in `generate_blinding()` | S |
| P1 | S2-4 | Populate genesis state hashes before mainnet | M |
| P2 | S3-1 | Redact nonce in Debug output | S |
| P2 | S3-3 | Replace `eprintln!` with structured logging | S |
| P2 | S3-4 | Deprecate `from_bytes()` in favor of `decode_checked()` | S |

---

## 7. Test Plan

### 7.1 Required Tests (Missing)

| Test | Type | Priority |
|------|------|----------|
| `AssetWire::to_asset()` strips `secret` field | Unit | P0 |
| Nonce derivation with failing time provider returns error | Unit | P0 |
| `Asset::asset_id()` differs for burned vs unburned same output | Unit | P1 |
| `to_owner_message()` differs for `None` vs `Some(0)` lock_height | Unit | P1 |
| Genesis blinding uniqueness across 50k assets | Property | P1 |
| Batch range proof verification with empty set | Unit | P1 |
| `AssetPackPlain` fuzzing with 10k random inputs | Fuzz | P2 |
| `AssetMetadata::compute_hash()` with empty custom_fields | Unit | P2 |
| `RegistryVersion::compute_hash()` with empty ID list | Unit | P2 |
| Nonce derivation collision resistance (100k nonces) | Property | P2 |

### 7.2 Existing Test Coverage (Adequate)

- ✅ Blinding factor uniqueness (100 samples)
- ✅ Nonce determinism and uniqueness (10k samples)
- ✅ Asset creation and validation
- ✅ Stealth field consistency
- ✅ Serial ID bounds checking
- ✅ Genesis seed validation (entropy, patterns)
- ✅ Asset pack serialization roundtrip
- ✅ Gas calculation overflow protection
- ✅ Policy flag validation
- ✅ Registry version hashing

### 7.3 Recommended Additions

- **Wycheproof vectors**: Not applicable (no custom primitives)
- **Cross-implementation tests**: Verify commitment/proof compatibility with Tari reference implementation
- **Negative tests**: Malformed `AssetWire` with invalid commitments, truncated proofs, zero nonces
- **Property tests**: `prop_assert!(!nonce_collision)` for 100k derivations with same seed
- **Fuzzing**: `AssetPackPlain::from_bytes()`, `AssetWire` deserialization

---

## 8. Confidence Levels

| Claim | Confidence | Evidence That Would Change It |
|-------|-----------|-------------------------------|
| Domain separation is correct | **High (95%)** | Finding a domain collision or reuse |
| Pedersen commitment binding is correct | **High (95%)** | Tari audit finding on commitment implementation |
| Bulletproofs+ soundness | **High (90%)** | Tari audit finding or proof system paper correction |
| Owner signature covers all critical fields | **Medium (80%)** | Discovery of a field that should be signed but isn't |
| Nonce derivation is collision-resistant | **High (90%)** | Blake2b collision or domain confusion |
| Genesis determinism is correct | **High (95%)** | Finding non-deterministic code path in genesis |
| Stealth address security | **Medium (75%)** | Full protocol review including spending flow |
| Fee privacy | **Low (50%)** | Fee is plaintext `u64` — no commitment on fee amount |

---

## 9. Final Decision

**`Execution-ready with conditions`**

### Conditions Before Production:

1. **S1-1 MUST be fixed**: Strip `secret` in `AssetWire::to_asset()` — this is a privacy vulnerability
2. **S1-2 MUST be addressed**: Either deprecate infallible nonce functions or restrict them to `#[cfg(test)]`
3. **S2-4 MUST be resolved**: Populate genesis state hashes before mainnet/testnet launch
4. **A1 MUST be investigated**: Audit all `Asset` construction paths for `secret` field injection

### Recommended Next Steps:

1. Fix S1-1 and S1-2 (estimated: 1 hour)
2. Investigate A1 (estimated: 2 hours)
3. Add missing tests from §7.1 (estimated: 4 hours)
4. Address S2-2 (lock_height encoding) in next protocol version
5. Populate genesis state hashes (S2-4) before mainnet launch
6. Schedule independent audit of stealth address spending flow (A5)

---

## Appendix A: Files Reviewed

| File | LOC | Crypto-Relevant |
|------|-----|-----------------|
| `assets/assets.rs` | 2771 | ✅ Commitments, proofs, signatures, asset_id |
| `assets/nonce.rs` | 1024 | ✅ Nonce derivation, counter management |
| `assets/definition.rs` | 806 | ⚠️ Asset ID (passed in, not derived here) |
| `assets/registry.rs` | 1775 | ❌ Storage/lookup only |
| `assets/leaf.rs` | 427 | ✅ Asset pack encoding, Poseidon2 usage |
| `assets/wire_pkg.rs` | 578 | ⚠️ Wire format (secret field) |
| `assets/wire.rs` | 223 | ⚠️ Import path (secret field) |
| `assets/blinding.rs` | 97 | ✅ Blinding factor generation |
| `assets/secret.rs` | 102 | ✅ Asset secret generation |
| `assets/gas.rs` | 269 | ⚠️ Fee calculation (no crypto) |
| `assets/policy_flags.rs` | 226 | ❌ Bit flags only |
| `assets/serial_id.rs` | 93 | ❌ Serialization only |
| `assets/version.rs` | 81 | ❌ Version detection only |
| `assets/snapshot.rs` | 232 | ⚠️ Registry hash |
| `assets/commitment.rs` | 14 | ✅ Commitment wrapper |
| `assets/amount.rs` | 3 | ❌ Type alias |
| `assets/mod.rs` | 300 | ❌ Re-exports |
| `genesis/genesis.rs` | 2024 | ✅ Blinding/nonce/RNG derivation |
| `genesis/validator.rs` | 1056 | ✅ Seed validation, state hash |
| `genesis/genesis_config.rs` | 283 | ❌ Config parsing |
| `genesis/serde.rs` | 367 | ❌ Serialization |
| `genesis/mod.rs` | 196 | ❌ Re-exports |
| `genesis/asset_std.rs` | 101 | ❌ Config helpers |
| `domains.rs` | 48 | ✅ Domain separation |
| `hashing.rs` | 45 | ✅ Hash type aliases |
| `lib.rs` | 108 | ❌ Crate facade |
| `state/mod.rs` | 1 | ❌ Empty |

**Total crypto-relevant LOC:** ~7,500 (53% of crate)

---

## Appendix B: Dependency Chain

```text
z00z_core
  ├── z00z_crypto (commitments, proofs, signatures, hashing, domains)
  │     └── tari/ (vendor — READ-ONLY, audited)
  └── z00z_utils (RNG, time, I/O, codec, config, logger, metrics)
```

No direct cryptographic library dependencies in `z00z_core` — all crypto is delegated to `z00z_crypto`. This is correct architecture.

---

## Appendix C: Standards Compliance

| Standard | Status | Notes |
|----------|--------|-------|
| RFC 8032 §5.1.3 (wide reduction) | ✅ Compliant | Blake2b-512 → scalar |
| NIST SP 800-107 (domain separation) | ✅ Compliant | 16 distinct domains |
| Mimblewimble/Tari commitment scheme | ✅ Compliant | Standard construction |
| Bulletproofs+ range proofs | ✅ Compliant | Via z00z_crypto |
| BIP-32/SLIP-0010 key derivation | N/A | Not used in core (wallets crate) |
| RFC 9380 (hash-to-curve) | N/A | Not used in core |
