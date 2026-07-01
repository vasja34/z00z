---
phase: 047-wallet-redesign
plan: 6
status: complete
completed_at: 2026-05-18
next_plan: 047-07
---

# Phase 047-06 Summary

## Completed Scope

`047-06` is complete for the tx lifecycle and asset-view cutover slice.
Live `wallet.tx.*` build, reservation, cancel, import, and reconcile paths now
mutate or read through owned-asset authority, and user-facing asset balance or
pending semantics no longer depend on Snapshot-owned claim vectors or the old
JSONL-only reservation inference path.

## Files Changed

- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
- `crates/z00z_wallets/src/services/wallet_service_tests.rs`

## Landed Changes

- Rewired `wallet.tx.build_transaction` onto
  `list_spendable_asset_rows(...)` plus
  `reserve_claimed_asset_inputs(...)`, keeping TOFU and payment-request checks
  on the live build path while preventing pending inputs from being selected
  twice.
- Hardened pending tx persistence ordering so tx journal writes happen before
  in-memory tx-byte cache updates, and build-time reservation rollback now runs
  fail-closed if tx journal persistence fails.
- Reworked cancel flow to verify pending status first, write cancelled status to
  the tx journal before asset release, and roll the tx status back to `Pending`
  if the reservation release path fails.
- Rewired import and reconcile onto owned-asset authority through
  `import_claimed_assets(...)`, `collect_owned_assets(...)`, and
  `confirm_claimed_asset_spend(...)`, so wallet-owned outputs are inserted or
  confirmed without replacing a full claimed-asset vector.
- Updated asset balance pending math to derive from live tx reservations instead
  of the old broad `has_pending_owner(...)` shortcut in the balance path.
- Added fail-closed regression coverage for build rollback, pending-input
  exclusion, import output insertion, evidence-mismatch reconcile rejection, and
  the receive restart assertion drift that surfaced on the broad workspace gate.

## Boundary Kept Intact

- `047-06` does not claim backup, restore, export, or Snapshot bridge retirement
  is complete. That remains explicit `047-07` scope.
- `047-06` keeps tx history as the explicit journal sidecar plane; it does not
  move live tx history into `.wlt`.
- `wallet.asset.send_asset`, `split_asset`, `merge_assets`, `stake_assets`,
  `swap_assets`, and `unstake_assets` still remain non-canonical compatibility
  surfaces rather than a second live asset authority plane.

## Review Passes

- Pass 1: Rechecked reconcile persistence against the live `TxStorage`
  semantics and confirmed that `record_confirmation_evidence(...)` still writes
  `Confirmed` status into the tx journal rather than leaving a hidden pending
  drift.
- Pass 2: Rechecked the remaining `claim_registry` usage in compatibility
  import flow and confirmed it is still a transient finalize or retry seam, not
  a parallel live owned-asset authority.
- Pass 3: The first broad workspace rerun exposed a real `test_recv_range_restart`
  regression because the test was identifying the resumed asset by shared
  `definition.id` instead of unique `asset_id`. The test was corrected and the
  mandatory verify order was restarted honestly from bootstrap.
- Pass 4: Rechecked the final write-set after the restart fix with
  `git diff --check`, owned-asset seam searches, and the focused
  `test_recv_range_restart` rerun. No significant issues found.
- Pass 5: Rechecked the final closeout claims against the green bootstrap gate,
  the green full workspace `cargo test` gate, and the final `047-06` write-set.
  No significant issues found.

Two consecutive clean passes were achieved on passes 4 and 5.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the final tree after the `test_recv_range_restart` fix.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed on the final tree after the same fix.
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_recv_range_restart -- --nocapture`
  passed on the final tree.
