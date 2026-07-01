---
phase: 062-Gaps-Closing-2
plan: 062-05
status: complete
completed_at: 2026-06-25
next_plan: 062-06
summary_artifact_for: .planning/phases/062-Gaps-Closing-2/062-05-PLAN.md
---

# 062-05 Summary: Wallet Lifecycle DTOs And Durable Tx History

## Outcome

`062-05` is complete. The grouped plan contract `PLAN-062-G05` now resolves
through the renamed `062-05-PLAN.md` packet with one append-only wallet
tx-history authority lane and no parallel lifecycle model.

Public tx lifecycle projection now comes from durable tx-history rows plus the
folded current record on the existing wallet JSONL/store path.
`RuntimeTxLifecycle` is threaded through send/details/import/reconcile
responses, and restart rebuilds the same lifecycle from durable rows instead
of an in-memory pending view.

The append-only row taxonomy is broader but still canonical. `Conflicted` and
`AlreadySpent` rows now preserve coarse `TxStatus::Failed` storage semantics
while exposing precise runtime lifecycle states. The DTO carrier for
`RuntimeTxErrorCode` is now present, but the full import/verify error-taxonomy
closeout remains owned by `062-06`.

Rollback and failpoint recovery now preserve durable history semantics. When a
mutation must be reverted, the tx store appends a compensating snapshot row
through `restore_snapshot(...)` instead of delete-and-recreate, so admitted,
imported, exported, and submitted lineage stays reconstructible after restart.

The phase-local evidence packet now records the converged authority story:
`wallet.asset.*` remains cash-only, `wallet.object.*` keeps voucher/right
semantics, and no second wallet authority plane or docs-only closure path was
introduced.

## Files Changed

- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_wallets/src/backup/backup_wire.rs`
- `crates/z00z_wallets/src/persistence/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`
- `crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs`
- `crates/z00z_wallets/src/rpc/test_tx_impl.rs`
- `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
- `crates/z00z_wallets/src/rpc/test_tx_send_suite.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_send.rs`
- `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
- `crates/z00z_wallets/src/rpc/tx_types.rs`
- `crates/z00z_wallets/src/wallet/stub_defaults_tx.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- `.planning/phases/062-Gaps-Closing-2/062-05-SUMMARY.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_wallets --test test_tx_store_integration`
- `cargo test --release -p z00z_wallets --test test_direct_tx_receive`
- `cargo test --release -p z00z_wallets 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::test_failpoint_restores_pending_state' -- --exact --nocapture`
- `cargo test --release -p z00z_wallets 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::test_failpoint_restores_tx_state' -- --exact --nocapture`
- `cargo test --release -p z00z_wallets 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::test_tx_lifecycle_projection_survives_restart' -- --exact`
- `cargo test --release -p z00z_wallets`
- `cargo test --release`
- Result: green

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
required review loop was executed manually against the same scope.

- Pass 1
  - `git diff -- .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_wallets/src/backup/backup_wire.rs crates/z00z_wallets/src/persistence/tx_storage.rs crates/z00z_wallets/src/persistence/tx_storage_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs crates/z00z_wallets/src/rpc/tx_rpc_server_send.rs crates/z00z_wallets/src/rpc/tx_runtime_state.rs crates/z00z_wallets/src/rpc/tx_types.rs crates/z00z_wallets/src/wallet/stub_defaults_tx.rs crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs crates/z00z_wallets/src/rpc/test_tx_impl.rs crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs crates/z00z_wallets/src/rpc/test_tx_send_suite.rs crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - `cargo test --release -p z00z_wallets --test test_tx_store_integration`
  - `cargo test --release -p z00z_wallets --test test_direct_tx_receive`
  - `cargo test --release -p z00z_wallets 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::test_failpoint_restores_pending_state' -- --exact --nocapture`
  - `cargo test --release -p z00z_wallets 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::test_failpoint_restores_tx_state' -- --exact --nocapture`
  - Result: found that rollback used delete-and-recreate semantics and lost durable lifecycle lineage; repaired with compensating snapshot restore rows.
- Pass 2
  - `rg -n 'RuntimeTxLifecycle|RuntimeTxErrorCode|Conflicted|AlreadySpent|restore_snapshot|wallet\\.asset\\.\\*' .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_wallets/src/backup/backup_wire.rs crates/z00z_wallets/src/persistence/tx_storage.rs crates/z00z_wallets/src/persistence/tx_storage_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs crates/z00z_wallets/src/rpc/tx_types.rs crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
  - `git diff --check -- .planning/phases/Z00Z-IMPL-PHASES.md .planning/phases/062-Gaps-Closing-2/062-05-SUMMARY.md crates/z00z_wallets/src/backup/backup_wire.rs crates/z00z_wallets/src/persistence/tx_storage.rs crates/z00z_wallets/src/persistence/tx_storage_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs crates/z00z_wallets/src/rpc/tx_rpc_server_send.rs crates/z00z_wallets/src/rpc/tx_runtime_state.rs crates/z00z_wallets/src/rpc/tx_types.rs crates/z00z_wallets/src/wallet/stub_defaults_tx.rs crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs crates/z00z_wallets/src/rpc/test_tx_impl.rs crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs crates/z00z_wallets/src/rpc/test_tx_send_suite.rs crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - Result: clean
- Pass 3
  - `cargo test --release -p z00z_wallets --test test_tx_store_integration --test test_direct_tx_receive`
  - `git diff --check -- .planning/phases/Z00Z-IMPL-PHASES.md .planning/phases/062-Gaps-Closing-2/062-05-SUMMARY.md .planning/STATE.md .planning/ROADMAP.md crates/z00z_wallets/src/backup/backup_wire.rs crates/z00z_wallets/src/persistence/tx_storage.rs crates/z00z_wallets/src/persistence/tx_storage_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_impl.rs crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs crates/z00z_wallets/src/rpc/tx_rpc_server_send.rs crates/z00z_wallets/src/rpc/tx_runtime_state.rs crates/z00z_wallets/src/rpc/tx_types.rs crates/z00z_wallets/src/wallet/stub_defaults_tx.rs crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs crates/z00z_wallets/src/rpc/test_tx_impl.rs crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs crates/z00z_wallets/src/rpc/test_tx_send_suite.rs crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - Result: clean

Passes 2 and 3 were consecutive clean runs.

## Completion Notes

Completion:
- Date: 2026-06-25
- Task: TASK-014
- Files changed:
  - `crates/z00z_wallets/src/persistence/tx_storage.rs`
  - `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`
  - `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_tx_store_integration` -> passed
  - `cargo test --release -p z00z_wallets --test test_direct_tx_receive` -> passed
  - `cargo test --release -p z00z_wallets` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/persistence/tx_storage.rs`
  - `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`
  - `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-015
- Files changed:
  - `crates/z00z_wallets/src/rpc/tx_types.rs`
  - `crates/z00z_wallets/src/backup/backup_wire.rs`
  - `crates/z00z_wallets/src/persistence/tx_storage.rs`
  - `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_tx_store_integration` -> passed
  - `cargo test --release -p z00z_wallets` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/rpc/tx_types.rs`
  - `crates/z00z_wallets/src/backup/backup_wire.rs`
  - `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-016
- Files changed:
  - `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::test_tx_lifecycle_projection_survives_restart' -- --exact` -> passed
  - `cargo test --release -p z00z_wallets` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-017
- Files changed:
  - `crates/z00z_wallets/src/backup/backup_wire.rs`
  - `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`
  - `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_tx_store_integration` -> passed
  - `cargo test --release -p z00z_wallets` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/backup/backup_wire.rs`
  - `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`
  - `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-018
- Files changed:
  - `crates/z00z_wallets/src/rpc/tx_types.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_send.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_direct_tx_receive` -> passed
  - `cargo test --release -p z00z_wallets` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/rpc/tx_types.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_send.rs`
  - `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-022
- Files changed:
  - `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_send_suite.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::test_tx_lifecycle_projection_survives_restart' -- --exact` -> passed
  - `cargo test --release -p z00z_wallets` -> passed
- Closeout evidence:
  - `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_send_suite.rs`

Completion:
- Date: 2026-06-25
- Task: TASK-038
- Files changed:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
  - `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- Tests run:
  - `cargo test --release -p z00z_wallets --test test_tx_store_integration` -> passed
  - `cargo test --release -p z00z_wallets 'rpc::methods::tx_rpc_impl::tests::test_tx_impl_suite::test_tx_lifecycle_projection_survives_restart' -- --exact` -> passed
  - `cargo test --release -p z00z_wallets` -> passed
- Closeout evidence:
  - `.planning/phases/Z00Z-IMPL-PHASES.md`
  - `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`
  - `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
  - `crates/z00z_wallets/tests/test_tx_store_integration.rs`
