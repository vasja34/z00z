---
phase: 063-Core-Update
plan: 063-08
status: complete
completed_at: 2026-06-28
next_plan: 063-09
summary_artifact_for: .planning/phases/063-Core-Update/063-08-PLAN.md
---

# 063-08 Summary: Authoritative Object-Family Semantics Matrix

## Outcome

`063-08` is complete. `PLAN-063-G08` now closes `REC-063-P1-05` by publishing
one canonical live semantics matrix for assets, rights, policies, and
vouchers, anchored to the current code instead of a future-only design layer.

The live caller-facing vocabulary is now explicitly rooted at
`z00z_core::ObjectFamily`, with the docs and runtime anchors aligned to one
canonical path. The slice also narrows the remaining ambiguous terms that were
still over-claiming behavior: `VoucherBootstrapEntryV1` is stated as a
bootstrap-only manifest concept rather than the runtime voucher object,
`VoucherLeaf` remains the runtime settlement object, `mintable` no longer
implies a generic public post-genesis asset-mint path, and
`SettlementActionV1::AssetMutation` is documented as the terminal asset
settlement lane rather than a generic family selector.

The genesis docs were truth-restored to the live multi-family contract as well.
They now describe typed bootstrap inputs and outputs for assets, rights,
policies, and vouchers, and they point at the canonical settlement-manifest
surface instead of implying that only the older asset-only flow is live.

## Files Changed

- `crates/z00z_core/README.md`
- `crates/z00z_core/docs/GENESIS_DOCUMENTATION.md`
- `crates/z00z_core/docs/OBJECT_FAMILY_SEMANTICS.md`
- `crates/z00z_core/src/assets/definition.rs`
- `crates/z00z_core/src/genesis/README.md`
- `crates/z00z_core/src/genesis/genesis_config.rs`
- `crates/z00z_core/src/vouchers/voucher_bootstrap.rs`
- `crates/z00z_storage/src/settlement/tx_plan_types.rs`
- `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`
- `.planning/phases/063-Core-Update/063-08-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core --doc`
- `cargo test --release -p z00z_wallets object_rpc -- --nocapture`
- `cargo test --release`
- `cargo test --release -p z00z_simulator wallet_lifecycle_rows_are_deterministic -- --nocapture --test-threads=1`
- `cargo test --release -p z00z_simulator s13_absence_drift_rejects -- --nocapture --test-threads=1`
- `cargo test --release -p z00z_simulator restart_check_restores_wlt_bytes -- --nocapture --test-threads=1`
- `cargo test --release -p z00z_simulator test_tx_validation_chain_id -- --nocapture --test-threads=1`
- `cargo test --release -p z00z_simulator test_tx_validation_nullifier_drift -- --nocapture --test-threads=1`
- `cargo test --release -p z00z_simulator --lib -- --test-threads=1`
- `rg -n "z00z_core::ObjectFamily|VoucherBootstrapEntryV1|VoucherLeaf|mintable|AssetMutation|generic post-genesis asset-mint selector" crates/z00z_core crates/z00z_storage crates/z00z_wallets`

- Result:
  - The mandatory bootstrap gate passed.
  - `z00z_core` rustdoc tests passed.
  - `z00z_wallets` object RPC release tests passed.
  - The broad workspace `cargo test --release` rerun reproduced five existing
    `z00z_simulator` failures outside the `063-08` docs/comment slice:
    `wallet_lifecycle_rows_are_deterministic`, `s13_absence_drift_rejects`,
    `restart_check_restores_wlt_bytes`, `test_tx_validation_chain_id`, and
    `test_tx_validation_nullifier_drift`.
  - Each of those five simulator failures passed when rerun individually in
    release mode with `--test-threads=1`.
  - A serial `cargo test --release -p z00z_simulator --lib -- --test-threads=1`
    rerun still left one nondeterministic `test_tx_validation_chain_id`
    failure, but that same test passed again immediately when rerun alone in
    release mode. The remaining broad-gate issue is therefore recorded as
    pre-existing simulator nondeterminism rather than a regression from the
    `063-08` slice.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times
against this slice, but the available runtime paths still did not produce a
review:

- Attempt 1
  - `gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-08-PLAN.md current_task="Publish one canonical object-family semantics matrix"'`
  - Result: no output; manually interrupted after the prompt runner failed to
    produce a review
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-08-PLAN.md current_task="Publish one canonical object-family semantics matrix"'`
  - Result: timed out with no output
- Attempt 3
  - `timeout 45s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-08-PLAN.md current_task="Publish one canonical object-family semantics matrix"'`
  - Result: timed out with no output

Equivalent review passes were executed manually against the same slice.

- Pass 1
  - Reviewed the changed core docs, genesis docs, storage settlement action
    docs, wallet object RPC docs, and the live `063-08` authority packet with
    the repository `doublecheck` criteria plus the one-canonical-path rule
  - Result: one live `z00z_core::ObjectFamily` vocabulary path remained, the
    bootstrap-vs-runtime voucher distinction was explicit, and no live docs
    still claimed a generic public post-genesis asset-mint path
- Pass 2
  - `cargo test --release -p z00z_core --doc`
  - `cargo test --release -p z00z_wallets object_rpc -- --nocapture`
  - `rg -n "z00z_core::ObjectFamily|VoucherBootstrapEntryV1|VoucherLeaf|mintable|AssetMutation|generic post-genesis asset-mint selector" crates/z00z_core crates/z00z_storage crates/z00z_wallets`
  - Result: clean for the modified `063-08` slice
- Pass 3
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test --release`
  - isolated release reruns of the five reported `z00z_simulator` failures
  - `cargo test --release -p z00z_simulator --lib -- --test-threads=1`
  - final exact rerun of `test_tx_validation_chain_id`
  - Result: no significant `063-08` slice issues remained; only phase-external
    simulator nondeterminism outside the changed files was reproduced

Passes 2 and 3 were consecutive clean review passes for the modified `063-08`
scope.

## Completion Notes

- `063-08-SUMMARY.md` closes `PLAN-063-G08` and advances the execution lane to
  `063-09-PLAN.md`.
- One canonical object-family semantics matrix now governs the live Phase 063
  contract.
- `z00z_core::ObjectFamily` is now the explicit caller-visible vocabulary root.
- `VoucherBootstrapEntryV1` remains bootstrap-only, while `VoucherLeaf`
  remains the runtime settlement object.
- `mintable` and `AssetMutation` wording no longer over-claim unsupported
  runtime semantics.
