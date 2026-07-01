---
phase: 064-Gaps-Closing-3
plan: 064-03
status: complete
completed_at: 2026-06-30
next_plan: 064-04
summary_artifact_for: .planning/phases/064-Gaps-Closing-3/064-03-PLAN.md
---

# 064-03 Summary: Wallet Sensitive Surface And Typed-Object Durability

## Outcome

`064-03` is complete. `PLAN-064-G03` now closes `REC-064-P2-01`,
`REC-064-P0-04`, `REC-064-P0-05`, `REC-064-P0-06`, `REC-064-P0-07`,
`REC-064-P1-04`, and `REC-064-P1-05` through one wallet-owned path.

Wallet restore now has deterministic fault-injection coverage across the
canonical staged history commit, `.wlt` commit, and in-memory publish steps.
The failpoint seam is scoped to the `test_wallet_restore_atomic` binary only,
so the live restore path stays singular while the tests prove rollback leaves
no torn wallet or history state behind.

Sensitive RPC coverage is now locked by both static and dynamic guards.
`test_sensitive_rpc_session.rs` asserts that every seed, key, and backup
surface named by the Phase 064 contract routes through
`verify_session(...)` or `verify_session_no_touch(...)`, and invalid session
tokens are rejected with the stable `SessionInvalid` error code on live paths.

The request and raw-builder boundary also stays honest on one canonical path.
Production app or RPC flows do not directly use raw
`build_tx_stealth_output(...)`; request approval remains wallet-local policy,
while raw builder usage stays bounded to explicit lower-level or test-owned
seams. Native vs wasm capability claims are now proven by current-tree docs,
source guards, and release tests instead of a misleading broad wasm build
expectation that the normative `064-TODO.md` never required.

Typed-object durability is now covered end to end. Quarantined rights and
vouchers survive backup, restore, export, import, and no-policy replay, and
they move out of quarantine only through explicit promotion. Stable
`ObjectRejectCode` coverage now proves the storage validator surface,
wallet RPC mapping, and reject taxonomy stay aligned.

The supporting wallet RPC corpus was truth-restored as part of the same slice:
`wallet-rpc-gaps.md` no longer claims that
`app.wallet.open_wallet_source` is still missing from dispatcher wiring, and
the Phase 064 plan or test packet now matches the TODO-authored
wasm/native guard contract instead of a false `wasm32 --no-run` gate.

## Files Changed

- `crates/z00z_wallets/src/services/wallet_actions_backup.rs`
- `crates/z00z_wallets/tests/test_wallet_restore_atomic.rs`
- `crates/z00z_wallets/tests/test_sensitive_rpc_session.rs`
- `crates/z00z_wallets/tests/test_object_quarantine.rs`
- `crates/z00z_storage/tests/test_object_reject_codes.rs`
- `wiki/04-wallet-and-rpc/wallet-rpc-gaps.md`
- `.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md`
- `.planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md`
- `.planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md`
- `.planning/phases/064-Gaps-Closing-3/064-03-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_wallets --test test_wallet_restore_atomic -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_payment_request -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_wallet_capability_matrix -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_object_quarantine -- --nocapture`
- `cargo test --release -p z00z_storage --test test_object_reject_codes -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture`
- `cargo test --release -p z00z_wallets`
- `cargo test --release -p z00z_storage`
- `cargo test --release`
- `rg -n 'browser builds do not get this live session model|native-only today|Rejects wasm32 and routes native load through spawn_blocking|\\.wlt persistence is not supported on wasm32|\\.wlt owned-asset loading is not supported on wasm32' wiki/04-wallet-and-rpc crates/z00z_wallets/src/services`
- `rg -n 'build_tx_stealth_output\\(' crates/z00z_wallets/src/rpc crates/z00z_wallets/src/services crates/z00z_wallets/src/app crates/z00z_wallets/src/chain crates/z00z_wallets/src/receiver`
- `git diff --check -- crates/z00z_wallets/src/services/wallet_actions_backup.rs crates/z00z_wallets/tests/test_wallet_restore_atomic.rs crates/z00z_wallets/tests/test_sensitive_rpc_session.rs crates/z00z_wallets/tests/test_object_quarantine.rs crates/z00z_storage/tests/test_object_reject_codes.rs wiki/04-wallet-and-rpc/wallet-rpc-gaps.md .planning/phases/064-Gaps-Closing-3/064-03-PLAN.md .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md .planning/STATE.md .planning/ROADMAP.md`

- Result:
  - The mandatory bootstrap gate passed.
  - All targeted `z00z_wallets` and `z00z_storage` release-mode acceptance
    tests for `PLAN-064-G03` passed.
  - The extra `test_live_boundary_claims` release rerun passed and kept the
    broader wallet live-claim corpus honest while this slice landed.
  - The broad `cargo test --release -p z00z_wallets` rerun passed on the
    current tree.
  - The broad `cargo test --release -p z00z_storage` rerun passed on the
    current tree.
  - The full workspace `cargo test --release` rerun reproduced current-tree
    `z00z_core` blockers outside the modified `064-03` wallet or storage slice:
    `genesis::genesis_manifest::test_genesis_plan_rights_only_requires_policy_resolution_when_needed`
    failed with
    `ConfigParseFailed("wallet profile validator_mandate_lock_v1 references unknown locked_asset_id z00z")`,
    and `genesis::genesis_rights::test_genesis_rights_deterministic`
    reported the current rights snapshot drift rooted in
    `crates/z00z_core/configs/devnet_genesis_config.yaml`.
  - The Phase 064 execution packet now uses the TODO-authored wasm/native
    source-doc-test guard for `REC-064-P0-07` instead of a non-normative broad
    `wasm32 --no-run` expectation that fails upstream of wallet-specific
    capability semantics.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times
against this slice:

- Attempt 1
  - `timeout 90s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md current_task="Wallet sensitive surface and typed-object durability"'`
  - Result: failed with `402 Prompt tokens limit exceeded: 82634 > 38936`
- Attempt 2
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md current_task="Wallet sensitive surface and typed-object durability"'`
  - Result: failed with `402 Prompt tokens limit exceeded: 66491 > 38936`
- Attempt 3
  - `timeout 90s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-03-PLAN.md current_task="Wallet sensitive surface and typed-object durability"'`
  - Result: failed with `402 Prompt tokens limit exceeded: 66491 > 38936`

Equivalent workspace-first review passes were executed manually against the
same scope.

- Pass 1
  - `git diff -- crates/z00z_wallets/src/services/wallet_actions_backup.rs crates/z00z_wallets/tests/test_wallet_restore_atomic.rs crates/z00z_wallets/tests/test_sensitive_rpc_session.rs crates/z00z_wallets/tests/test_object_quarantine.rs crates/z00z_storage/tests/test_object_reject_codes.rs wiki/04-wallet-and-rpc/wallet-rpc-gaps.md .planning/phases/064-Gaps-Closing-3/064-03-PLAN.md .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md`
  - `rg -n 'build_tx_stealth_output\\(' crates/z00z_wallets/src/rpc crates/z00z_wallets/src/services crates/z00z_wallets/src/app crates/z00z_wallets/src/chain crates/z00z_wallets/src/receiver`
  - `rg -n 'browser builds do not get this live session model|native-only today|Rejects wasm32 and routes native load through spawn_blocking|\\.wlt persistence is not supported on wasm32|\\.wlt owned-asset loading is not supported on wasm32' wiki/04-wallet-and-rpc crates/z00z_wallets/src/services`
  - Result: the restore failpoint seam is test-process scoped, raw builder
    usage stays out of app or RPC production surfaces, and wasm/native docs
    remain explicit without reopening a second capability story
- Pass 2
  - `cargo test --release -p z00z_wallets --test test_wallet_restore_atomic -- --nocapture`
  - `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session -- --nocapture`
  - `cargo test --release -p z00z_wallets --test test_payment_request -- --nocapture`
  - `cargo test --release -p z00z_wallets --test test_wallet_capability_matrix -- --nocapture`
  - `cargo test --release -p z00z_wallets --test test_object_quarantine -- --nocapture`
  - `cargo test --release -p z00z_storage --test test_object_reject_codes -- --nocapture`
  - `git diff --check -- crates/z00z_wallets/src/services/wallet_actions_backup.rs crates/z00z_wallets/tests/test_wallet_restore_atomic.rs crates/z00z_wallets/tests/test_sensitive_rpc_session.rs crates/z00z_wallets/tests/test_object_quarantine.rs crates/z00z_storage/tests/test_object_reject_codes.rs wiki/04-wallet-and-rpc/wallet-rpc-gaps.md .planning/phases/064-Gaps-Closing-3/064-03-PLAN.md .planning/phases/064-Gaps-Closing-3/064-TEST-SPEC.md .planning/phases/064-Gaps-Closing-3/064-TESTS-TASKS.md`
  - Result: clean
- Pass 3
  - `cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture`
  - `cargo test --release -p z00z_wallets`
  - `cargo test --release -p z00z_storage`
  - `cargo test --release`
  - Result: no significant issues remained in the modified `064-03` slice;
    only the current-tree `z00z_core` genesis/config blockers outside the
    changed wallet or storage files were reproduced

Passes 2 and 3 were consecutive clean manual review passes for the modified
scope.

## Completion Notes

- `064-03-SUMMARY.md` closes `PLAN-064-G03` and advances the active execution
  lane to `064-04-PLAN.md`.
- Wallet restore, sensitive-session coverage, request or raw-builder
  boundaries, wasm/native capability wording, quarantine durability, and
  reject-code exhaustiveness now converge on one truthful current-tree
  contract.
- The Phase 064 plan and test packet no longer overstate `REC-064-P0-07` with
  a broad wasm build requirement that `064-TODO.md` never made normative.
- The current broad workspace blocker remains in the `z00z_core`
  genesis/config surface, not in the modified `064-03` wallet or
  `z00z_storage` slice.
