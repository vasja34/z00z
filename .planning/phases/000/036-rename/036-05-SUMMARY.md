# 036-05 Summary

## Scope

This summary records the completion state for `036-05-PLAN.md`, covering task
`036-02 Preserve Compatibility Shims And Compatibility Read-Import Lanes`.

## Outcome

Plan 05 is closed for the active Phase 036 slice.

Phase 036 now keeps the Step 1 compatibility rows explicit and untouched:
`spend_v1`, `events_v1`, `multi_v1`, `ProofBlobV0`, and `ClaimNullRecV0`
remain live at their current spellings, each tied to an explicit retirement
precondition rather than premature rename or delete work.

## Repository Changes

- `crates/z00z_wallets/src/core/tx/spending.rs` now marks `spend_v1` and
  `events_v1` as Step 1 compatibility shims and adds direct guard tests proving
  the legacy public shim lanes remain reachable.
- `crates/z00z_wallets/src/core/tx/mod.rs` now documents that the public
  `multi_v1`, `events_v1`, and `spend_v1` re-export lanes remain intentional
  compatibility holds.
- `crates/z00z_wallets/src/core/tx/asset_selector.rs` now records the
  retirement precondition for the `multi_v1` selector lane, and
  `crates/z00z_wallets/src/core/tx/asset_selector_tests.rs` now proves both
  shim import paths still resolve to the same live helpers.
- `crates/z00z_storage/src/assets/proof.rs` now records that `ProofBlobV0`
  decode remains intentionally live until explicit persisted-proof retirement
  evidence exists.
- `crates/z00z_storage/src/assets/store_internal/redb_backend_state.rs` and
  `crates/z00z_storage/src/assets/store_internal/redb_backend_helpers.rs` now
  document that `ClaimNullRecV0` and its decode fallback remain live until the
  persisted rehydrate lane is proven retired.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`:
  passed
- `cargo test -p z00z_storage --release --features test-fast --test test_redb_rehydrate`:
  passed
- `cargo test -p z00z_storage --release --features test-fast test_proof_blob_decode_legacy_v0_upgrades_root_bind`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::tx::asset_selector::tests::test_multi_v1_shim_stays_public_until_cutover -- --exact --nocapture`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::tx::spending::events_v1::tests::test_events_v1_shim_stays_public_until_cutover -- --exact --nocapture`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::tx::spending::events_v1::tests::test_spend_v1_shim_stays_public_until_cutover -- --exact --nocapture`:
  passed
- `rg -n "spend_v1|events_v1|ProofBlobV0|ClaimNullRecV0|multi_v1" crates/z00z_wallets/src crates/z00z_storage/src crates/z00z_storage/tests`:
  passed

## Review Loop

The review loop closed truthfully on a mixed automated and manual path:

1. direct review-subagent attempts for the full code-review and crypto-review
   passes hit environment rate limits and could not be used as the authoritative
   closeout path
2. one security-style review pass completed and reported no material issue in
   the Step 1 compatibility slice, with residual risk limited to the deliberate
   continued compatibility exposure until a later retirement wave proves the
   cutover
3. manual source-backed re-review of the active files, task wording, and test
   ownership found no remaining material issue after the shim guard tests and
   retirement-precondition comments were added

## Current Boundary

This summary closes only Plan 05 of Phase 036. It does not claim execution of
the later rename or retirement waves now queued under `036-06-PLAN.md` through
`036-10-PLAN.md`.
