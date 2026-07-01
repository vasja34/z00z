---
phase: 046-wallet-addons
plan: 4
status: complete
completed_at: 2026-05-15
next_plan: paused
---

<!-- markdownlint-disable-file MD022 MD031 MD032 MD041 MD047 MD060 -->

# Phase 046-04 Summary

## Completed Scope

`046-04` is complete for the payment-request / TOFU, session-hardening, and rotate-master-key boundary slice.

The implementation keeps the security boundary inside wallet RPC and wallet session management:

- Payment-request RPC validation now has direct negative coverage for TOFU first-use pinning, second-use approval, chain mismatch, expired request, invalid signature, malformed request payloads, and malformed receiver-card rejection.
- Sensitive wallet.tx RPC actions now have stale-session regression coverage for build, broadcast, cancel, reconcile, import, and export.
- `wallet.key.rotate_master_key` no longer calls the old reachability placeholder shim and returns through the live `finish_rotate(...)` / `rotate_master_key_in_memory(...)` boundary.
- RPC logging risk coverage now includes wrong unlock password, wrong show-seed password, raw bad session token, rotate, and lifecycle-lock failure paths, with assertions that secrets and raw tokens do not leak into logs.
- The stale test name for the live rotate flow was renamed away from `stub` wording.

## Files Changed

- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_admin.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_rpc.rs`
- `crates/z00z_wallets/src/services/wallet_service.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl/tests/mod.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs`

## Boundary Notes

- `rotate_master_key` remains an audited, rate-limited, in-memory rederive flow. It does not claim durable seed rotation or persisted master-key rewrite.
- The empty `wallet_service_actions_rpc.rs` include remains as a stable extension point only; sensitive wallet RPC actions must route through the live session/runtime guards rather than a reachability placeholder.
- Stale-session tx coverage accepts the live guard outcomes used by the runtime boundary: session expired, session invalid, or wallet locked after auto-lock.

## Review Passes

The local `/GSD-Review-Tasks-Execution` prompt was applied manually in YOLO mode three times against `.planning/phases/046-wallet-addons/046-04-PLAN.md`; no standalone slash-command runner was present in the repository.

- Pass 1: Found stale `stub` wording in the rotate-master-key test name after removing the reachability placeholder path. Fixed by renaming the test to `test_rotate_master_rederive`.
- Pass 2: Rechecked payment-request, session, logging, and rotate boundaries against the plan and implementation. No significant issues found.
- Pass 3: Rechecked validation evidence, scope boundaries, and the user stop condition. No significant issues found.

Two consecutive clean passes were achieved on passes 2 and 3.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed before broader validation.
- `cargo fmt` passed; rustfmt emitted the existing warnings about nightly-only formatter options.
- `cargo test -p z00z_wallets key_impl::tests::test_validate_req_ --features test-fast --features wallet_debug_dump` passed.
- `cargo test -p z00z_wallets test_tx_sensitive_ops_reject_stale_session --features test-fast --features wallet_debug_dump` passed.
- `cargo test -p z00z_wallets --test test_rpc_logging_risk_policy --features test-fast --features wallet_debug_dump` passed.
- `cargo test -p z00z_wallets key_impl::tests::test_ --features test-fast --features wallet_debug_dump` passed.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed again after fixes.
- `cargo test --release --features test-fast --features wallet_debug_dump` passed on the final tree.

## Stop State

Per user instruction, execution stops after `046-04`.

`046-05` remains pending and was not executed. `046-06` is paused pending rewrite so it can be reconciled with the wallet `.wlt` redesign direction before any tamper/release-smoke closeout work begins.
