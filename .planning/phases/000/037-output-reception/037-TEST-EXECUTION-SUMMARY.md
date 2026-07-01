---
phase: 037-output-reception
artifact: test-execution-summary
status: partial
updated: 2026-04-23
---

# Phase 037 Test Execution Summary

## ✅ Scope

This artifact records only the currently executed Phase 037 test slice from
`037-TEST-SPEC.md` and `037-TESTS-TASKS.md`.

It does not claim full closure of the entire test backlog.

## ✅ Landed Slice

The execution-backed slice currently covers:

- Wave T1 deterministic request ordering and expiry-aware request registration
- the narrow current Wave T5 severity contract where `ReceiveReject::NotMine`
  stays non-alerting while `InvalidInput`, `InvalidProof`, and `RuntimeFail`
  remain actionable

The remaining assisted-receive, wrapper-parity, residual RPC expansion, and
final backlog-sweep waves remain open.

## ✅ Repository Changes

- `crates/z00z_wallets/src/core/address/stealth_scanner/types_tag_cache.rs`
  now uses deterministic active-request ordering and skips expired requests
  during registration.
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` now keeps the
  request-less fallback explicit and last in candidate materialization.
- `crates/z00z_wallets/src/core/address/test_stealth_scan_support_suite.rs`
  now covers fallback-last ordering, first-win short-circuit, and a
  request-bound owned scan path.
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` now
  prunes expired requests before scanner registration.
- `crates/z00z_wallets/src/core/address/stealth_scanner/types_receive.rs`,
  `crates/z00z_wallets/src/core/address/stealth_scanner/test_stealth_scanner.rs`,
  and `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
  now hold the non-alerting `NotMine` severity split.
- `.planning/phases/037-output-reception/037-TEST-SPEC.md`,
  `.planning/phases/037-output-reception/037-TESTS-TASKS.md`, and
  `.planning/phases/037-output-reception/037-TEST-PLAN.md` were rebased to stay
  within what the landed code and tests actually prove.

## ✅ Validation

- Mandatory bootstrap gate passed:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Focused Phase 037 anchors passed:
  - `cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scan_support::tests::ordered_request_candidates_puts_fallback_last --lib -- --exact`
  - `cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scan_support::tests::scan_cached_keys_first_win --lib -- --exact`
  - `cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scan_support::tests::scan_owned_matches_request_bound_output --lib -- --exact`
  - `cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scanner::types::tests::test_active_requests_are_sorted_and_skip_expired --lib -- --exact`
  - `cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scanner::tests::test_recv_reject_map --lib -- --exact`
  - `cargo test -p z00z_wallets --release --features test-fast adapters::rpc::methods::asset_impl::asset_impl_tests::asset_receive_api_sync --lib -- --exact`
- Required broader workspace command is now green alongside this slice:
  - `cargo test --release --features test-fast --features wallet_debug_dump`
  - the fresh rerun passed, including the Tari doctest surface that earlier
    interim notes treated as an external blocker.

## ✅ Review Loop

The required YOLO review loop ran five narrow passes against the landed slice.

1. Pass 1 was clean.
2. Pass 2 found a planning/test-anchor drift and added `scan_cached_keys_first_win`.
3. Pass 3 found that the new test still did not prove short-circuit directly and
   strengthened it with a panic-guard iterator.
4. Pass 4 was clean.
5. Pass 5 was clean.

The last two consecutive review passes reported no significant in-scope issues.

## ✅ Current Boundary

This artifact closes only the landed T1 plus narrow current T5 slice of the
Phase 037 test backlog.

It does not claim that Wave T2, Wave T3, Wave T4, Wave T6, or the broader
Task 9 backlog are fully closed.
