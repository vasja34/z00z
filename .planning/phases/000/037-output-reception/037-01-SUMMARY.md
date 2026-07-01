# 037-01 Summary

## Scope

This summary records the completion state for `037-01-PLAN.md`, covering task
`Task 0. Freeze the implemented Phase 037 baseline before extending it` and
task `Task 1. Keep \`WalletService::recv_range(...)\` as the canonical receive path`.

## Outcome

Plan 01 is closed for the implemented baseline-freeze slice.

Phase 037 now has an explicit receive-architecture ledger anchored to live
code truth, the canonical receive lane is frozen to
`WalletService::recv_range(...)`, compatibility-only receive surfaces are
named as such, and `ScanEngineImpl` is documented as a non-parity stub-only
seam instead of live functionality.

## Repository Changes

- `037-ARCHITECTURE.md` now records the implemented receive baseline around
  `recv_range(...)`, `StealthOutputScanner`, `recv_route(...)`, wallet-native
  claimed persistence, `ScanStatePayload`, request-aware helpers, and
  `receiver_keys(...)`.
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` now
  documents `scan_asset_report(...)` and `receive_asset(...)` as
  compatibility-only single-asset lanes while freezing `recv_range(...)` as
  the canonical Phase 037 receive path.
- `crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs` now
  documents `receiver_keys(...)` as the canonical live receiver-key boundary
  for Phase 037 service and RPC receive flows.
- `crates/z00z_wallets/src/core/chain/scan_engine_impl.rs` now documents the
  scan engine seam as stub-only and explicitly non-parity while the canonical
  receive authority remains `WalletService::recv_range(...)`.
- `crates/z00z_wallets/src/adapters/rpc/methods/asset.rs` now marks the
  outward `wallet.asset.receive_asset` RPC as a compatibility-only
  single-asset lane rather than a canonical privacy receive surface.
- `037-CONTEXT.md`, `037-TEST-PLAN.md`, `037-TEST-SPEC.md`, and
  `037-TESTS-TASKS.md` now treat `037-ARCHITECTURE.md` as executed Task 0
  evidence instead of a planned-only future artifact.
- `037-01-SUMMARY.md` now exists as the required plan-closeout artifact.

## Validation

- Diagnostics for `037-01-PLAN.md`, `037-ARCHITECTURE.md`,
  `wallet_service_actions_receive.rs`,
  `wallet_service_actions_receiver.rs`, and `scan_engine_impl.rs`: clean.
- Diagnostics for `037-CONTEXT.md`, `037-TEST-PLAN.md`, `037-TEST-SPEC.md`,
  `037-TESTS-TASKS.md`, and `037-01-SUMMARY.md`: clean after the truth-repair
  pass.
- Focused request-flow regression guard after the Task 1 wording repair:
  `cargo test -p z00z_wallets --test test_e2e_req_flow --release --features test-fast --features wallet_debug_dump`
  passed clean.
- Repository-path recheck across the Phase 037 planning surfaces: the earlier
  `planned-only` claims for `037-ARCHITECTURE.md` are removed from the active
  Task 0 support artifacts.

## Review Loop

The repeated bounded review of the Task 0 slice found and resolved these
truthful-completion gaps:

1. `037-CONTEXT.md` still described `037-ARCHITECTURE.md` as planned-only.
2. `037-TEST-PLAN.md`, `037-TEST-SPEC.md`, and `037-TESTS-TASKS.md` still
   described `037-ARCHITECTURE.md` as non-executed future evidence.
3. The required `037-01-SUMMARY.md` artifact was missing.
4. The outward `wallet.asset.receive_asset` RPC docstring still left semantic
  slack around its compatibility-only status.

After those fixes, the bounded Plan 01 support surface matches the implemented
baseline-freeze story without reopening the receive architecture as
greenfield work.

## Current Boundary

This summary closes only the Plan 01 baseline-freeze slice for Task 0 and
Task 1. It does not claim later Phase 037 task execution, verification-backed
phase status, or closure of the later decision-gated receive work.
