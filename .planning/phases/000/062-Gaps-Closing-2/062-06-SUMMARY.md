---
phase: 062-Gaps-Closing-2
plan: 062-06
status: complete
completed_at: 2026-06-25
next_plan: 062-07
summary_artifact_for: .planning/phases/062-Gaps-Closing-2/062-06-PLAN.md
---

# 062-06 Summary: Typed Import And Verify Error Taxonomy

## Outcome

`062-06` is complete. The grouped plan contract `PLAN-062-G06` now resolves
through the renamed `062-06-PLAN.md` packet with one wallet authority lane
for tx-package decode, verify, import, and runtime error projection.

Wallet tx import and verify now expose stable machine-readable reject
semantics instead of string-only outcomes. `RuntimeTxLifecycle`,
`RuntimeTxErrorCode`, and `RuntimeTxRpcErrorData` are threaded through
pre-import verify, import, and runtime lifecycle projection so callers can
distinguish invalid payloads, unsupported versions, digest mismatches,
wrong-chain packages, not-import-ready payloads, no-owned-output reports,
and already-spent conflicts without inventing a parallel interpretation
layer.

Portable package parsing and export are also converged on the live runtime
taxonomy. Portable decode maps invalid wrapper or encoding, unsupported
version, metadata or tx digest mismatch, invalid inner package, and wrong
chain onto explicit typed classes, while export rewrites the inner package
status from the durable runtime lifecycle before wrapping. The live import
path now treats admitted packages as import-ready, preserves verify as
pre-mutation only, and rejects already-spent conflicts through the same
public typed error lane.

`062-06` also closes the missing same-package idempotence proof. Re-importing
the exact same portable tx package now remains `Imported`, does not duplicate
claimed assets or tx-history rows, and keeps imported output identities
stable across retries.

The phase evidence packet now records the converged docs authority story for
unsupported receive-version taxonomy and offline tx-package verify/import
hardening without creating a second wallet authority plane or a docs-only
closure.

## Files Changed

- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_wallets/src/rpc/error_mapping.rs`
- `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`
- `crates/z00z_wallets/src/rpc/tx_types.rs`
- `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- `.planning/phases/062-Gaps-Closing-2/062-06-SUMMARY.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_wallets --test test_payment_request --test test_s5_input_gate --test test_tx_interop --test test_import_error_taxonomy --test test_asset_import_security --test test_asset_replay_protection --test test_direct_tx_receive --test test_tx_store_integration --test test_tx_serial --test test_output_reception --test test_runtime_validation_result`
- `cargo test --release -p z00z_wallets --lib 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::' -- --nocapture`
- `cargo test --release`
- Result: green

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
required review loop was executed manually against the same scope.

- Pass 1
  - `git diff -- .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_wallets/src/rpc/error_mapping.rs crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs crates/z00z_wallets/src/rpc/tx_rpc_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_server.rs crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs crates/z00z_wallets/src/rpc/tx_rpc_support.rs crates/z00z_wallets/src/rpc/tx_types.rs crates/z00z_wallets/tests/test_direct_tx_receive.rs`
  - `cargo test --release -p z00z_wallets --lib 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::' -- --nocapture`
  - Result: found missing same-package portable import idempotence proof and a stale nonexistent test target reference; repaired with an explicit idempotent re-import test on the live tx import suite and phase-local validation against current workspace test surfaces.
- Pass 2
  - `cargo test --release -p z00z_wallets --test test_payment_request --test test_s5_input_gate --test test_tx_interop --test test_import_error_taxonomy --test test_asset_import_security --test test_asset_replay_protection --test test_direct_tx_receive --test test_tx_store_integration --test test_tx_serial --test test_output_reception --test test_runtime_validation_result`
  - `git diff --check -- .planning/phases/Z00Z-IMPL-PHASES.md .planning/phases/062-Gaps-Closing-2/062-06-SUMMARY.md crates/z00z_wallets/src/rpc/error_mapping.rs crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs crates/z00z_wallets/src/rpc/tx_rpc_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_server.rs crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs crates/z00z_wallets/src/rpc/tx_rpc_support.rs crates/z00z_wallets/src/rpc/tx_types.rs crates/z00z_wallets/tests/test_direct_tx_receive.rs`
  - Result: clean
- Pass 3
  - `cargo test --release`
  - `git diff --check -- .planning/phases/Z00Z-IMPL-PHASES.md .planning/phases/062-Gaps-Closing-2/062-06-SUMMARY.md .planning/STATE.md .planning/ROADMAP.md crates/z00z_wallets/src/rpc/error_mapping.rs crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs crates/z00z_wallets/src/rpc/tx_rpc_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_server.rs crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs crates/z00z_wallets/src/rpc/tx_rpc_support.rs crates/z00z_wallets/src/rpc/tx_types.rs crates/z00z_wallets/tests/test_direct_tx_receive.rs`
  - Result: clean

Passes 2 and 3 were consecutive clean runs.

## Completion Notes

Completion:
- Date: 2026-06-25
- Task: TASK-019
- Files changed:
  - `crates/z00z_wallets/src/rpc/error_mapping.rs`
  - `crates/z00z_wallets/src/rpc/tx_types.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_direct_tx_receive` -> passed
  - `cargo test --release -p z00z_wallets --test test_import_error_taxonomy` -> passed
  - `cargo test --release -p z00z_wallets --lib 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::' -- --nocapture` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/rpc/error_mapping.rs`
  - `crates/z00z_wallets/src/rpc/tx_types.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-020
- Files changed:
  - `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`
  - `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_direct_tx_receive` -> passed
  - `cargo test --release -p z00z_wallets --lib 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::' -- --nocapture` -> passed
  - `cargo test --release` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`
  - `crates/z00z_wallets/tests/test_direct_tx_receive.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-021
- Files changed:
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_asset_import_security` -> passed
  - `cargo test --release -p z00z_wallets --test test_asset_replay_protection` -> passed
  - `cargo test --release -p z00z_wallets --lib 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::' -- --nocapture` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-025
- Files changed:
  - `crates/z00z_wallets/src/rpc/error_mapping.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_import_error_taxonomy` -> passed
  - `cargo test --release -p z00z_wallets --test test_payment_request` -> passed
  - `cargo test --release -p z00z_wallets --lib 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::' -- --nocapture` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/rpc/error_mapping.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-029
- Files changed:
  - `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
  - `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_import_error_taxonomy` -> passed
  - `cargo test --release -p z00z_wallets --test test_tx_interop` -> passed
  - `cargo test --release -p z00z_wallets --lib 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::' -- --nocapture` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
  - `crates/z00z_wallets/tests/test_direct_tx_receive.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-035
- Files changed:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_wallets/src/rpc/error_mapping.rs`
  - `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_payment_request` -> passed
  - `cargo test --release -p z00z_wallets --test test_direct_tx_receive` -> passed
  - `cargo test --release` -> passed
- Closeout evidence:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_wallets/src/rpc/error_mapping.rs`
  - `crates/z00z_wallets/tests/test_direct_tx_receive.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-037
- Files changed:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
  - `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_asset_import_security` -> passed
  - `cargo test --release -p z00z_wallets --test test_asset_replay_protection` -> passed
  - `cargo test --release -p z00z_wallets --test test_direct_tx_receive` -> passed
  - `cargo test --release` -> passed
- Closeout evidence:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
  - `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
