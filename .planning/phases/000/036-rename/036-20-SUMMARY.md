---
phase: 036-rename
plan: 20
status: partial
updated: 2026-04-20
---

# 036-20 Summary

## Scope

This summary records the current execution and validation state for `036-20-PLAN.md`, the self-contained A4 shim-removal follow-on rooted in `036-a4-shims-spec.md`.

## Outcome

`036-20` is not complete. The plan is now summary-backed partial.

The landed slice removed the explicit old-name caller shims that were blocking honest closeout on the wallet and simulator owner surfaces:

- removed deprecated `TimeProvider::unix_timestamp*()` wrappers in favor of explicit `try_*` or `compat_*` calls
- deleted `crates/z00z_crypto/src/ecdh_stealth.rs` and `crates/z00z_crypto/src/test_ecdh_stealth_suite.rs`
- removed `decode_ristretto_pk` from the wallet stealth owner surface and normalized callers onto `decode_public_key()` or `decode_card_public_key()`
- renamed `build_stealth_sender_leaf`, `build_stealth_bundle`, and `build_stealth_bundle_with_rng` to `build_card_stealth_leaf`, `build_output_bundle`, and `build_output_bundle_with_rng`, then migrated the live caller inventory
- renamed `derive_address()` to `derive_spend_key()` across the address-manager surface and renamed live `recv_one()` callers to `scan_asset_report()` or simulator-local `scan_asset()`
- moved the touched production commitment and point decode seams onto `Commitment::from_bytes(...)`, `try_from_bytes(...)`, or equivalent explicit canonical decode paths where this wave actually owned the call sites
- repaired the post-migration signer verification seam so wrapper-based decode still feeds the Tari verification path correctly

Focused validation for the old-name shim slice is clean: the residual scan for `recv_one(`, `decode_ristretto_pk`, `build_stealth_sender_leaf`, `build_stealth_bundle*`, and `derive_address` still returns zero matches on repeated reruns, and this continuation completed a fresh green bootstrap plus the broader `cargo test --release --features test-fast --features wallet_debug_dump` gate after the constructor cleanup.

This is not a full `036-20` closeout. The broader constructor, storage, and simulator sweep remains open. The current broad repository scan reports 71 live residual hits: 54 `from_canonical_bytes`, 13 `CompatRoot`, and 4 `SeqSecureRngProvider`. Not every remaining `from_canonical_bytes` hit is necessarily a compatibility bug, and a narrower non-test source sweep across the active phase crates now reduces that tail to a src-local test helper plus one rustdoc example, but the plan-level deletion and survivor reclassification work is still not complete enough to claim full shim removal.

## Repository Changes

Representative owned seams updated in this slice include:

- `crates/z00z_utils/src/time/traits.rs`
- `crates/z00z_crypto/src/commitments.rs`
- `crates/z00z_crypto/src/types.rs`
- `crates/z00z_core/src/assets/wire_pkg_serde_impls.rs`
- `crates/z00z_wallets/src/core/tx/claim_tx.rs`
- `crates/z00z_wallets/src/core/address/stealth_request_crypto.rs`
- `crates/z00z_wallets/src/core/stealth/encoding.rs`
- `crates/z00z_wallets/src/core/stealth/mod.rs`
- `crates/z00z_wallets/src/core/stealth/output.rs`
- `crates/z00z_wallets/src/core/address/address_manager/address_manager_trait.rs`
- `crates/z00z_wallets/src/core/address/address_manager/address_manager_impl_snapshot_io.rs`
- `crates/z00z_wallets/src/core/address/stealth_card_codec.rs`
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`
- `crates/z00z_wallets/src/core/address/z00z_address/z00z_single_address.rs`
- `crates/z00z_wallets/src/core/address/z00z_address/z00z_dual_address.rs`
- `crates/z00z_wallets/src/core/tx/output_flow.rs`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/signer.rs`
- `crates/z00z_wallets/src/core/tx/prover.rs`
- `crates/z00z_wallets/src/core/tx/witness_gate.rs`
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/test_tx_lane_runtime_suite.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/test_tx_lane_runtime_support.rs`

## Validation

- `rg -n "recv_one\(|decode_ristretto_pk|build_stealth_sender_leaf|build_stealth_bundle|build_stealth_bundle_with_rng|derive_address\b" crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'`: passed with no matches on two consecutive reruns
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed in the current continuation after the constructor cleanup
- `cargo test --release --features test-fast --features wallet_debug_dump`: passed in the current continuation after the constructor cleanup
- `rg -n "from_canonical_bytes\b|CompatRoot\b|SeqSecureRngProvider" crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'`: reports 71 current residual hits total (`54` + `13` + `4`), so the open gap is retained truthfully
- `find crates/z00z_core/src crates/z00z_storage/src crates/z00z_wallets/src crates/z00z_simulator/src crates/z00z_utils/src -type f -name '*.rs' ! -name 'test_*' -print0 | xargs -0 rg -n "from_canonical_bytes\("`: narrows the active non-test source tail to a src-local test helper and one rustdoc example instead of a new production runtime lane

## Review Loop

The focused old-name shim slice still has convergence on the old-name boundary: repeated residual scans stay clean, and this continuation added a fresh green bootstrap plus the broader release-style cargo test gate after the decode-lane migration.

That does not upgrade the whole plan to completed. The broader constructor, storage, and simulator tail still has open inventory, so the stronger "two consecutive no significant issues" condition is still claimed only for the focused old-name caller slice, not for the entire embedded shim table.

As with the nearby continuation slices, `/GSD-Review-Tasks-Execution` was not exposed as a direct CLI entrypoint in this environment, so the recorded review evidence is the repo-backed substitute: exact-context rereads, deterministic scans, and repeated bootstrap reruns.

## Canonical Artifact Sync

- `.planning/phases/036-rename/036-20-PLAN.md`
- `.planning/phases/036-rename/036-20-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Current Boundary

`036-20` remains the live execution pointer. `036-20.A` and `036-20.B` are partially executed, and `036-20.C` now has a truth-backed survivor inventory reread but still no deletion proof. The next honest closure step is the explicit boundary decision for `CompatRoot` and `SeqSecureRngProvider`, plus any remaining constructor cleanup that survives after that decision.
