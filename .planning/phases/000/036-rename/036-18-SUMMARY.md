---
phase: 036-rename
plan: 18
status: completed
updated: 2026-04-19
---

# 036-18 Summary

## Scope

This summary records the Wave 8 cleanup and closure for `036-18-PLAN.md` on the active `036-a3_for_remane-spec.md` authority chain.

## Outcome

`036-18` closes the a3 follow-on rename sweep truthfully. The storage, core, simulator, wallet, support, and bench helpers listed in the exact reference table were renamed to the suggested semantic names, and the final workspace residual scan over the scoped files returned zero matches. The summary closes the last a3-backed helper set without changing runtime semantics or touching Tari vendor code.

Representative closures include:

- `root_for` -> `build_root_from_path`
- `asset_batches_for` -> `select_asset_batches`
- `serial_batches_for` -> `select_serial_batches`
- `claim_rows_for` -> `build_claim_rows`
- `expected_hash_for` -> `expected_genesis_hash`
- `claim_leaf_for` -> `decrypt_claim_leaf`
- `wlt_path_for` -> `require_wallet_file_path`
- `lease_for` -> `create_nullifier_lease`
- `sender_create_output_for` -> `build_sender_output_leaf`
- `build_tx_stealth_output_for` -> `build_tx_stealth_output_with_serial`
- `label_for` -> `lookup_address_label`
- `seed_for` -> `derive_wallet_mark_seed`

This closeout is intentionally scoped to the a3 reference table. It does not claim unrelated repository-wide cleanup beyond the rows proven in scope.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --quiet`: passed
- `cargo test -p z00z_core --release --features test-fast --features wallet_debug_dump --quiet`: passed
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --quiet`: passed
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed
- scoped residual scan across the a3 files: passed with no matches
- appendix note verification on `036-a1-versioning-spec-V3.md`: passed after the zero-versioning closure note was appended

## Review Loop

The required review passes were completed in YOLO mode during the live cleanup, and the final deterministic reruns plus residual scan did not introduce any material issue.

## Canonical Artifact Sync

- `.planning/phases/036-rename/036-a1-versioning-spec-V3.md`
- `.planning/phases/036-rename/036-18-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Current Boundary

`036-18` is summary-backed complete. This summary closes Plan 18 only; later summaries continue the broader Phase 036 chain.
