---
phase: 030
plan: 19
subsystem: z00z_wallets wallet-rpc
summary: Reduce the remaining wallet RPC method, dispatcher, logging, and DTO roots below the continuation band while preserving the wallet-visible transport contract.
tags:
  - phase-030
  - wallet-rpc
  - dispatcher
  - logging
  - dto
  - refactor
  - wallet
requirements-completed:
  - PH30-SEAMS
  - PH30-FACADE
  - PH30-SYNC
affects:
  - crates/z00z_wallets/src/adapters/rpc/methods
  - crates/z00z_wallets/src/adapters/rpc/logging
  - crates/z00z_wallets/src/adapters/rpc/types
  - crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs
  - crates/z00z_wallets/tests/test_rpc_scenarios_minimal.rs
  - crates/z00z_wallets/tests/test_import_error_taxonomy.rs
provides:
  - Wallet RPC method, dispatcher, logging, and DTO roots below the >400 continuation band
  - Stable shallow wallet RPC facades over extracted server, support, register, impl, and test seams
  - Synchronized transport, logging, and DTO handling for session-specific and rate-limit wallet RPC failures
key_files:
  created:
    - crates/z00z_wallets/src/adapters/rpc/logging/middleware_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/logging/summarize_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_support_assets.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_support_claims.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_support_state.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/backup_impl_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support_tail.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/storage_impl_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_tests_body.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/types/common_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/types/events_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/types/events_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/types/key_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/types/security_tests.rs
    - crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs
    - docs/code-review/2026-04-02-phase-030-plan-19-wallet-rpc-review.md
  modified:
    - crates/z00z_networks/rpc/src/error.rs
    - crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs
    - crates/z00z_wallets/src/adapters/rpc/logging/middleware.rs
    - crates/z00z_wallets/src/adapters/rpc/logging/summarize.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/backup_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/key.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/key_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/storage_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/types/common.rs
    - crates/z00z_wallets/src/adapters/rpc/types/events.rs
    - crates/z00z_wallets/src/adapters/rpc/types/key.rs
    - crates/z00z_wallets/src/adapters/rpc/types/security.rs
    - crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs
    - crates/z00z_wallets/src/services/wallet_service_session_build.rs
    - crates/z00z_wallets/src/services/wallet_service_session_construction.rs
    - crates/z00z_wallets/src/services/wallet_service_session_derivation.rs
    - crates/z00z_wallets/src/services/wallet_service_session_password.rs
    - crates/z00z_wallets/src/services/wallet_service_session_rotation.rs
    - crates/z00z_wallets/src/services/wallet_service_session_runtime.rs
    - crates/z00z_wallets/src/services/wallet_service_types_core.rs
    - crates/z00z_wallets/tests/test_import_error_taxonomy.rs
    - crates/z00z_wallets/tests/test_rpc_scenarios_minimal.rs
    - reports/full_verify-report-long-running-tests.txt
decisions:
  - Keep the wallet RPC caller surface stable by turning oversized method, dispatcher, logging, and DTO roots into thin facades over extracted seam files.
  - Preserve session and rate-limit failure taxonomy end-to-end by extending the generic RPC transport error surface instead of collapsing wallet security failures into AuthFailed.
  - Document Phase 1 rotate and asset or tx placeholder behavior explicitly in wallet RPC docs and summaries instead of masking it behind synthetic success semantics.
metrics:
  duration: current-session
  completed_at: 2026-04-02
  tasks_completed: 2/2
---

# Phase 030 Plan 19: Wallet RPC Residue Split Summary

Reduced the remaining oversized wallet RPC method, dispatcher, logging, and DTO roots below the continuation band while preserving the visible wallet RPC surface and tightening rotate, audit, and transport error behavior.

## Outcomes

- Task 1 finished with the remaining wallet RPC method and dispatcher roots reduced below the `>400` continuation band by extracting server, support, register, and test seams:
  - `asset_impl.rs`: facade over `asset_impl_server.rs`, `asset_impl_support_assets.rs`, `asset_impl_support_claims.rs`, `asset_impl_support_state.rs`, and `asset_impl_tests.rs`
  - `tx_impl.rs`: facade over `tx_impl_server.rs`, `tx_impl_tests.rs`, and `tx_impl_tests_body.rs`
  - `wallet_impl.rs`: facade over `wallet_impl_tests.rs`
  - `storage_impl.rs`: facade over `storage_impl_tests.rs`
  - `backup_impl.rs`: facade over `backup_impl_tests.rs`
  - `key_impl.rs`: facade over `server.rs`, `support.rs`, `support_tail.rs`, and `test_key_impl.rs`
  - `wallet_dispatcher_wiring.rs`: facade over `wallet_dispatcher_wiring_register.rs`
- Task 2 finished with the logging and typed-surface residue reduced and synchronized:
  - extracted focused logging tests into `middleware_tests.rs` and `summarize_tests.rs`
  - split `events.rs` into a stable type root plus `events_impl.rs` and `events_tests.rs`
  - moved DTO regression coverage into `common_tests.rs`, `key_tests.rs`, and `security_tests.rs`
  - aligned `key.rs`, `types/key.rs`, and `types/security.rs` with the real post-split wallet RPC transport behavior
- Review-driven hardening closed three wallet RPC correctness gaps that became visible after the split:
  - restored backward-compatible `id` alias handling for wallet-id params in dispatcher parsing
  - restored rotate password re-auth, added `verify_session_no_touch(...)`, and enforced the documented per-wallet rotate rate limit
  - preserved `RateLimited`, `SessionExpired`, and `SessionInvalid` transport categories through dispatcher mapping and bounded logging instead of collapsing them into generic auth failure
- Logging and audit behavior now retain wallet attribution and failure class more accurately:
  - rotate summaries prefer nested `session.wallet_id`, keep top-level compatibility fallback, and only emit redacted confirmation when present
  - rotate audit paths now emit denied, rate-limited, expired, invalid, and finish-failure outcomes with service-time timestamps
- Final validation closed green after one last scenario-test sync fix for the new transport contract, and the fresh max-safe gate passed with `planned=313 skipped=21 failed=0`.

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_wallets --release --test test_asset_ownership_security -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_tx_assetpack -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_runtime_validation_result -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_rpc_scenarios_minimal`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- Wallet RPC review loop completed with two consecutive clean review passes after all rotate, audit, transport, and logging findings were addressed.
- Codacy file analysis completed clean on the edited wallet RPC, service, and transport files after the final edits and formatting pass.

## Deviations from Plan

### Auto-fixed Issues

1. `[Rule 2 - Missing critical functionality]` Restored legacy wallet-id compatibility by accepting both `wallet_id` and legacy `id` in dispatcher param DTOs so older wallet RPC callers did not break after the transport split.
2. `[Rule 2 - Missing critical functionality]` Reintroduced rotate password re-auth, no-touch session validation, explicit rotate rate limiting, and audit coverage for denied and runtime-failure paths after review exposed that the split had weakened destructive-key-operation safety.
3. `[Rule 1 - Bug]` Preserved session-specific and rate-limit transport taxonomy by adding `RpcError::RateLimited`, `RpcError::SessionExpired`, and `RpcError::SessionInvalid`, then updating dispatcher and logging middleware to keep those classes visible.
4. `[Rule 1 - Bug]` Updated `test_rpc_scenarios_minimal` after the fresh max-safe gate found that the old lifecycle-lock scenario still assumed the pre-split `AuthFailed` collapse and did not accept the new session-specific transport errors.
5. `[Rule 3 - Blocking issue]` Cleared the first fresh max-safe rerun by running `cargo fmt`, then reran the full gate to reach the real semantic regression in the scenario test instead of stopping on formatting-only fallout.

## Known Stubs

- Wallet RPC asset, tx, storage, and several wallet flows still intentionally expose Phase 1 placeholder behavior and `stub_default`-style responses in operations outside the rotate, dispatcher, logging, and DTO hardening slice.
- `wallet.key.rotate_master_key` now documents its current Phase 1 behavior honestly: it rebuilds in-memory derivation state and returns the current key-material fingerprint, not a new persistent seed lifecycle.

## Deferred Issues

- The worktree still contains unrelated user-side changes outside Plan 030-19 scope, including repository instruction and skill-tree edits under `.github/`; they were left untouched and excluded from this plan summary.
- The long-running inventory report at `reports/full_verify-report-long-running-tests.txt` was regenerated by the final max-safe gate and retained as a verification artifact.

## Self-Check: PASSED

- Summary file created at `.planning/phases/030-refactor-long-files/030-19-SUMMARY.md`
- Plan requirements `PH30-SEAMS`, `PH30-FACADE`, and `PH30-SYNC` are reflected in the delivered seam split and transport/DTO synchronization outcomes
- Fresh `full_verify.sh --max-safe-run` passed with `planned=313 skipped=21 failed=0`
- Review loop closed with two consecutive clean passes after the final wallet RPC hardening fixes
- Scenario regression found by the first fresh max-safe rerun was fixed and revalidated before closeout
