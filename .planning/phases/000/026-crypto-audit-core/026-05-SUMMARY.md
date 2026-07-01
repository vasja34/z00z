---
phase: 026-crypto-audit-core
plan: "05"
subsystem: crypto
tags: [assets, ownership, stealth, fee, nonce, range-proof]
requires:
  - phase: 026-01
    provides: canonical asset-definition identity seam
provides:
  - owner signature domain binds owner_pub and stealth-critical fields
  - native fee validation uses one canonical native fee definition accessor
  - public nonce helpers fail closed on time-provider errors
  - amount policy is tied to RANGE_PROOF_BITS_V1 and enforced in asset validation
affects: [z00z_core, z00z_wallets, z00z_simulator, fee-validation, ownership-checks]
tech-stack:
  added: []
  patterns: [canonical owner message binding, fail-closed nonce helpers, canonical native fee accessor]
key-files:
  created: []
  modified:
    - crates/z00z_core/src/assets/assets.rs
    - crates/z00z_core/src/assets/gas.rs
    - crates/z00z_core/src/assets/nonce.rs
    - crates/z00z_core/src/assets/amount.rs
    - crates/z00z_core/src/assets/mod.rs
    - crates/z00z_core/src/assets/test_wire.rs
    - crates/z00z_core/src/assets/test_wire_phase26.rs
    - crates/z00z_core/tests/assets/asset_signature_domain.rs
    - crates/z00z_core/tests/assets/test_integration_crypto.rs
    - crates/z00z_core/tests/assets/test_integration_owner_signature_security.rs
key-decisions:
  - "Bind owner_pub into the owner signature message so authority identity is part of canonical signed state."
  - "Treat native fee identity as one authoritative reconstructed AssetDefinition seam in assets/mod.rs and reuse that seam in gas tests."
  - "Keep zero-fallback nonce helpers crate-visible only and export fail-closed Result-returning helpers on the public path."
patterns-established:
  - "Canonical state first: verify_owner_signature validates canonical definition and stealth consistency before signature verification."
  - "Fee identity first: gas validation accepts only the exact canonical native fee definition, not tuple-adjacent impostors."
requirements-completed: [PH26-AUTH, PH26-NONCE-FEE]
duration: multi-session
completed: 2026-03-28
---

# Phase 026 Plan 05: Owner authority, fee identity, and fail-closed nonce policy summary

📌 Owner-signature semantics now bind owner_pub plus the full stealth tuple, gas validation accepts only the canonical native fee definition, and the public nonce path fails closed on time-provider errors while amount policy stays tied to the configured proof width.

## Performance

- **Duration:** multi-session
- **Started:** multi-session
- **Completed:** 2026-03-28T20:06:52Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Bound `owner_pub`, `r_pub`, `owner_tag`, `enc_pack`, `tag16`, and `leaf_ad_id` into the canonical owner-signature message and extended negative tamper coverage.
- Replaced class-only fee acceptance with a canonical native fee accessor and added negative tests for class-only, flag-only, and near-canonical impostors.
- Moved the public nonce path to Result-returning helpers, kept `_or_zero` helpers crate-visible only, and enforced proof-width-aware amount policy in asset validation.

## Task Commits

📌 Each task was committed atomically:

1. **Task 1: Bind owner and stealth verification to authoritative state and native fee identity** - `3b3f055f` (feat)
2. **Task 2: Make nonce and amount policy fail closed** - `2a60a545` (fix)
3. **Review-driven hardening across both tasks** - `f3d48836` (fix)

**Plan metadata:** `2cee0ef7` (docs)

## Files Created/Modified

- `crates/z00z_core/src/assets/assets.rs` - extended canonical owner message, enforced amount-policy validation, and updated task-local test helper.
- `crates/z00z_core/src/assets/gas.rs` - consumed the canonical native fee accessor and added missing negative fee-identity coverage.
- `crates/z00z_core/src/assets/nonce.rs` - converted public nonce helpers to fail-closed `Result` APIs and narrowed `_or_zero` helpers to crate-visible compatibility.
- `crates/z00z_core/src/assets/amount.rs` - tied `MAX_AMOUNT` and helper checks to `RANGE_PROOF_BITS_V1` semantics.
- `crates/z00z_core/src/assets/mod.rs` - exported only fail-closed nonce helpers and introduced the authoritative canonical native fee definition accessor.
- `crates/z00z_core/src/assets/test_wire.rs` - adapted test helpers to the new fail-closed nonce API.
- `crates/z00z_core/src/assets/test_wire_phase26.rs` - adapted phase-26 wire tests to the new fail-closed nonce API.
- `crates/z00z_core/tests/assets/asset_signature_domain.rs` - added owner_pub rebinding negative coverage and stealth-field tamper coverage.
- `crates/z00z_core/tests/assets/test_integration_crypto.rs` - adapted integration tests to the new fail-closed nonce API.
- `crates/z00z_core/tests/assets/test_integration_owner_signature_security.rs` - adapted owner-signature integration tests to the new fail-closed nonce API.

## Decisions Made

- Used the canonical owner message seam instead of inventing a second verification path for stealth and authority fields.
- Kept the native fee identity seam in `assets/mod.rs` so `gas.rs` does not hardcode unverifiable IDs or duplicate logic.
- Enforced proof-width-aware amount policy at asset validation boundaries while preserving the crypto layer’s current 64-bit proof-width configuration.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Public nonce helpers still failed open**

- **Found during:** Task 2 review pass
- **Issue:** `_or_zero` behavior was still available through the public nonce surface.
- **Fix:** Converted `get_timestamp_micros`, `derive_nonce_simple`, and `derive_nonce_minimal` to Result-returning APIs, removed `_or_zero` from root exports, and narrowed compatibility helpers to `pub(crate)`.
- **Files modified:** `crates/z00z_core/src/assets/nonce.rs`, `crates/z00z_core/src/assets/mod.rs`, `crates/z00z_core/src/assets/test_wire.rs`, `crates/z00z_core/src/assets/test_wire_phase26.rs`, `crates/z00z_core/tests/assets/test_integration_crypto.rs`, `crates/z00z_core/tests/assets/test_integration_owner_signature_security.rs`
- **Verification:** `cargo test -p z00z_core --tests --lib -- --nocapture`; `cargo test -p z00z_core --lib before_epoch_fails_closed -- --nocapture`
- **Committed in:** `f3d48836`

**2. [Rule 2 - Missing Critical] Native fee and owner authority seams were still incomplete under review**

- **Found during:** Task 1 review pass
- **Issue:** Native fee validation still used a partial tuple seam and owner authority was missing `owner_pub` in the canonical owner message.
- **Fix:** Added the authoritative `native_fee_def()` accessor, made `is_native_fee_def()` compare against the full canonical definition, and bound `owner_pub` into `to_owner_message()` with dedicated negative coverage.
- **Files modified:** `crates/z00z_core/src/assets/mod.rs`, `crates/z00z_core/src/assets/gas.rs`, `crates/z00z_core/src/assets/assets.rs`, `crates/z00z_core/tests/assets/asset_signature_domain.rs`
- **Verification:** `cargo test -p z00z_core --test assets_tests asset_signature_domain -- --nocapture`; `cargo test -p z00z_core --lib native_fee_asset_matcher -- --nocapture`
- **Committed in:** `f3d48836`

---

**Total deviations:** 2 auto-fixed (2 missing critical)
**Impact on plan:** Both deviations were required to close review-discovered gaps without widening scope outside `z00z_core`.

## Issues Encountered

- The required bootstrap suite passed cleanly.
- The required full workspace command `cargo test --release --features test-fast --features wallet_debug_dump` still hits a pre-existing doctest failure in `crates/z00z_crypto/tari/crypto` caused by duplicate `tari_utilities` trait resolution; this was out of scope for plan 026-05 and unchanged by the current work.
- Early review passes raised authority-binding and fee-identity concerns; both were fixed and two consecutive later review passes reported a clean pass for plan closure.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `z00z_core` is ready for downstream consumers to rely on canonical owner binding, canonical fee identity, and fail-closed nonce helpers.
- The pre-existing `z00z_crypto/tari/crypto` doctest failure remains a separate workspace gate issue if the phase requires a fully green all-crates doctest run.

## Self-Check: PASSED

- Summary file created at `.planning/phases/026-crypto-audit-core/026-05-SUMMARY.md`.
- Task commits `3b3f055f`, `2a60a545`, and `f3d48836` exist in `git log`.

---
*Phase: 026-crypto-audit-core*
*Completed: 2026-03-28*
