# 036-03 Summary

## Scope

This summary records the completion state for `036-03-PLAN.md`, covering task
`036-07 Claim-V2 Protocol Surface Hold`, task
`036-08 Address-V2 Future Activation Hold`, and task
`036-09 Test-Only Row Review After Production Steps`.

## Outcome

Plan 03 is closed for the Phase 036 slice.

Phase 036 now keeps the outer claim-v2 transport contract and the inner
`ClaimProofVer::V1` source-proof boundary explicitly separated, keeps the
address-v2 helpers dormant and future-reserved, and resolves the test-only row
ledger truthfully: rows `1` and `2` were renamed as pure suffix-noise cleanup,
rows `3` through `28` remain kept as live evidence-bearing tests, and the
wallet-KDF retirement rows were re-resolved onto the current fail-closed proof
surfaces before closeout.

## Repository Changes

- `crates/z00z_wallets/src/core/tx/claim_wire_types.rs` now documents that
  `CLAIM_PROOF_V2` is only the outer transport tag.
- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl.rs` and
  `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` now make
  the outer-tag versus inner-proof-version boundary explicit at the verifier
  seam.
- `crates/z00z_storage/src/assets/store_internal/store_query.rs` now documents
  that storage remains the authoritative issuer of `ClaimProofVer::V1`
  semantics until a separate protocol migration changes that contract.
- `crates/z00z_core/src/assets/leaf_tests.rs` renamed `test_value_endian_v1`
  to `test_value_endian` and `test_offsets_v1` to `test_offsets` as the only
  pure suffix-noise cleanup performed in the `036-09` review wave.
- `036-a1-suffixes-spec.md` and `036-TODO-1.md` now re-resolve the test-only
  row ledger to the live proof surfaces for rows `1`, `2`, `5`, and `8`.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test -p z00z_storage --release --features test-fast --test test_claim_source_proof -- --nocapture`: passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_redb_wlt_open test_open_rejects_kdf_v1 -- --exact --nocapture`: passed
- `cargo test -p z00z_wallets --release --features test-fast --lib core::key::key_manager_redb::tests::unwrap_rejects_non_current_record_params -- --exact --nocapture`: passed
- `cargo test -p z00z_core --release --lib assets::leaf::tests::test_value_endian -- --exact --nocapture`: passed
- `cargo test -p z00z_core --release --lib assets::leaf::tests::test_offsets -- --exact --nocapture`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`: failed outside Plan 03 scope in `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` with `test_phase034_doc_allowlist_tracks_active_closure_truth` and `test_phase034_closeout_artifacts_reconcile_semantic_chain`

## Review Loop

The review loop closed with these repository-backed findings:

1. `036-09` initially carried stale raw-row names for the wallet-KDF test
   surfaces after fail-closed retirement, so the canonical phase-local sources
   were corrected before test-only execution continued
2. only rows `1` and `2` qualified as pure suffix-noise rename candidates;
   the remaining rows continued to carry live compatibility or versioned-proof
   evidence and were kept
3. two consecutive manual source-backed review passes after the row-drift and
   suffix-cleanup edits found no remaining material issues in the active Plan 03
   slice

## Current Boundary

This summary closes the Plan 03 Phase 036 slice truthfully, but the final broad
workspace release gate still reports an unrelated Phase 034 simulator-document
closeout failure outside the files changed for this plan.
