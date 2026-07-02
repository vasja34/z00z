---
phase: 065-Attack-Surface
plan: 065-05
status: complete
completed_at: 2026-07-01
next_plan: 065-06
summary_artifact_for: .planning/phases/065-Attack-Surface/065-05-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 065-05 Summary: Capability-Typed Privileged Wallet Paths

## 🎯 Outcome

`065-05` is complete.

Privileged wallet routes now require typed verified capability objects instead
of relying on each handler to remember a raw session guard. The privileged
route registry is explicit and auditable, the dispatcher audit fails closed on
guard mismatches, raw stealth builders are visibly noncanonical through
`*_unchecked` naming, and native or wasm capability truth is explicit in code
and tests.

## 📦 Files Changed

- `.planning/phases/065-Attack-Surface/065-05-SUMMARY.md`
- `.planning/phases/065-Attack-Surface/065-TODO.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `crates/z00z_wallets/scripts/audit_rpc_method_wiring.py`
- `crates/z00z_wallets/src/lib.rs`
- `crates/z00z_wallets/src/rpc/backup_rpc_impl.rs`
- `crates/z00z_wallets/src/rpc/dispatcher_handlers.rs`
- `crates/z00z_wallets/src/rpc/key_rpc_impl.rs`
- `crates/z00z_wallets/src/rpc/key_rpc_server.rs`
- `crates/z00z_wallets/src/rpc/key_rpc_server_admin.rs`
- `crates/z00z_wallets/src/rpc/key_rpc_server_derive.rs`
- `crates/z00z_wallets/src/rpc/key_rpc_server_requests.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`
- `crates/z00z_wallets/src/rpc/wallet_dispatcher_routes.rs`
- `crates/z00z_wallets/src/rpc/wallet_dispatcher_wiring.rs`
- `crates/z00z_wallets/src/rpc/wallet_rpc_impl.rs`
- `crates/z00z_wallets/src/services/mod.rs`
- `crates/z00z_wallets/src/services/wallet_service.rs`
- `crates/z00z_wallets/src/services/wallet_session_guards_inactive.rs`
- `crates/z00z_wallets/src/services/wallet_session_runtime_limits.rs`
- `crates/z00z_wallets/src/stealth/mod.rs`
- `crates/z00z_wallets/src/stealth/output.rs`
- `crates/z00z_wallets/tests/test_rpc_route_coverage.rs`
- `crates/z00z_wallets/tests/test_sensitive_rpc_session.rs`
- `crates/z00z_wallets/tests/test_stealth_output.rs`
- `crates/z00z_wallets/tests/test_wallet_capability_matrix.rs`
- `wiki/04-wallet-and-rpc/receiver-request-flow.md`
- `wiki/04-wallet-and-rpc/wallet-session-locks.md`

## 🔧 Landed Changes

- Capability-typed session surfaces
  - `WalletService::verify_session(...)` now returns `VerifiedSession`.
  - `WalletService::verify_session_no_touch(...)` now returns
    `VerifiedSessionNoTouch`.
  - `SessionCapKind`, `SessionCapState`, `SessionCapRow`,
    `SESSION_CAP_MATRIX`, `TOUCH_CAP_ERR`, and `NO_TOUCH_CAP_ERR` make
    native or wasm capability truth explicit and fail closed on wasm.
- Central privileged route ownership
  - `wallet_dispatcher_routes.rs` now carries `PRIV_ROUTE_SPECS` plus
    `PrivRouteGuard`.
  - Sensitive dispatcher routes register through `typed_handler_cap(...)`
    and must pass `verify_touch_cap(...)`, `verify_no_touch_cap(...)`, or
    `verify_rotate_cap(...)` before reaching `*_checked` owners.
  - `wallet_rpc_impl.rs`, `backup_rpc_impl.rs`, `key_rpc_impl.rs`, and the
    split key server modules expose `pub(crate)` checked entrypoints so raw
    session wrappers stay thin and auditable.
- Canonical module path and raw-builder demotion
  - `crate::services::{VerifiedSession, VerifiedSessionNoTouch}` is the
    canonical import path for privileged capability objects.
  - raw stealth builders were renamed to `build_tx_output_unchecked(...)`
    and `build_tx_output_serial_unchecked(...)`; validated builders remain
    the clear production path.
  - wallet wiki docs plus `065-TODO.md` were updated to match the live
    capability and builder contract.
- Audit and regression hardening
  - `audit_rpc_method_wiring.py` now understands split dispatcher
    registration blocks, records `guard_kind`, parses `PRIV_ROUTE_SPECS`,
    and treats `verify_*` plus `*_checked` helpers as legitimate ownership
    paths.
  - source-based tests now anchor on dispatcher registration blocks instead
    of accidentally matching `PRIV_ROUTE_SPECS`.
- Compliance cleanup
  - fixed the remaining WS-05 identifier-length violations by renaming
    `build_validated_output_bundle_with_rng` to
    `finish_output_bundle_rng`,
    `show_seed_phrase_rejects_invalid_session` to
    `show_seed_phrase_bad_session`, and
    `export_public_material_rejects_invalid_session` to
    `export_material_bad_session`.

## ✅ Validation

Commands green on the current tree:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `bash crates/z00z_wallets/scripts/audit_rpc_method_wiring.sh`
  - JSON proof artifact:
    `crates/z00z_wallets/outputs/audit_rpc/audit_rpc_methods.json`
  - recorded result: `errors = 0`, `warnings = 0`
- `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session --test test_wallet_capability_matrix --test test_stealth_output --test test_rpc_route_coverage -- --nocapture`
- `cargo test --release -p z00z_wallets`

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still does not provide a callable review path for this
slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-05-PLAN.md current_task="Capability-Typed Privileged Wallet Paths"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-05-PLAN.md current_task="Capability-Typed Privileged Wallet Paths" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66676 > 38936`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-05-PLAN.md current_task="Capability-Typed Privileged Wallet Paths" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 82819 > 38936`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `065-05-PLAN.md`, `065-TODO.md`, the privileged dispatcher
    routes, and the route-audit script.
  - Result: found the split-route audit and source tests still anchored on
    the first RPC string match instead of the real registration block. Fixed
    the audit parser to recover `guard_kind` from `typed_handler_cap(...)`
    registrations and fixed the source tests to anchor on dispatcher
    registration blocks.
- Pass 2
  - Ran an identifier-length scan across the touched WS-05 code and tests.
  - Result: found three remaining V2 naming violations and fixed them:
    `build_validated_output_bundle_with_rng` ->
    `finish_output_bundle_rng`,
    `show_seed_phrase_rejects_invalid_session` ->
    `show_seed_phrase_bad_session`,
    `export_public_material_rejects_invalid_session` ->
    `export_material_bad_session`.
- Pass 3
  - Re-ran the identifier scan over touched `fn`, `struct`, `enum`, `trait`,
    and `const` surfaces and re-checked for retired raw builder naming.
  - Result: clean. No touched WS-05 identifiers over five words remain, and
    the old canonical-looking raw stealth builder names are absent.
- Pass 4
  - Re-ran the narrow release gate and re-read the audit JSON plus the
    capability-matrix and dispatcher-route test anchors.
  - Result: clean. The privileged route audit reports zero errors and zero
    warnings, capability truth stays explicit, and the narrow release tests
    remain green.

Passes 3 and 4 were consecutive clean manual review runs after the last
in-scope findings were fixed.

## 🧾 Closeout

`065-05` closes `WS-05` by making privileged wallet entry impossible without
typed capability objects, centralizing guard truth in one auditable dispatcher
route registry, publishing explicit native or wasm capability truth, and
pushing raw stealth builders off the canonical API path. The active Phase 065
lane moves to `065-06-PLAN.md`.
