# 037-04 Summary

## Scope

This summary records the completion state for `037-04-PLAN.md`, covering task
`Task 6. Implement inbox-assisted receive only at the service or adapter boundary`
and task
`Task 7. Resolve \`ScanEngineImpl\` by either de-scoping it or making it a thin delegate`.

## Outcome

Plan 04 is closed for the inbox-defer and scan-engine de-scope slice.

Phase 037 now states explicitly that inbox-assisted receive remains future-only
in this phase because no concrete live inbox or hint source is verified in the
codebase and Task 15 has not yet closed deterministic ordered non-expired
candidate selection. No speculative
`wallet_service_actions_receive_inbox.rs` module was added, and no second
receive, detect, or store lane was introduced.

Phase 037 also now states explicitly that `ScanEngineImpl` remains a
proposed-only stub seam in this phase. It no longer implies live parity with
`recv_range(...)`, and its runtime stub behavior now returns one explicit
deferred-not-implemented error contract instead of sitting in a half-claimed
state.

## Repository Changes

- `.planning/phases/037-output-reception/037-TODO.md` now records the selected
  Phase 037 branch for Task 6 as explicit defer-only and the selected Phase 037
  branch for Task 7 as explicit de-scope-only.
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` now ties
  any future inbox-assisted receive opening to both a concrete live inbox or
  hint source and Task 15 closure on deterministic ordered non-expired
  candidate selection.
- `crates/z00z_wallets/src/core/chain/scan_engine_impl.rs` now states the seam
  as explicitly stub-only for Phase 037 and routes `scan()` plus `scan_range()`
  through one shared deferred-not-implemented constant.
- `.planning/phases/037-output-reception/037-ARCHITECTURE.md` remains aligned
  with the live code: inbox-assisted receive is future-only and
  `ScanEngineImpl` remains proposed-only rather than an implemented parity
  surface.
- `.planning/phases/037-output-reception/037-04-REVIEW.md` and
  `docs/code-review/2026-04-22-phase-037-plan-04-task-6-review-pass-4.md`
  capture the converged review evidence for the Task 6 and Task 7 closeout.

## Validation

- Mandatory bootstrap gate rerun:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  clean.
- Focused scan-engine de-scope regression:
  `cargo test -p z00z_wallets --lib scan_range_not_implemented --release --features test-fast --features wallet_debug_dump`
  passed clean.
- Focused receive-gate regression after the Task 6 wording fix:
  `cargo test -p z00z_wallets --lib recv_route_gate --release --features test-fast --features wallet_debug_dump`
  passed clean.
- Required broader release suite rerun:
  `cargo test --release --features test-fast --features wallet_debug_dump`
  passed clean.
- One intermediate broader rerun hit a transient simulator claim-store RedB
  lock in
  `stage4_rejects_bundle_omission_against_persisted_store`. The exact isolated
  repro of that test passed clean, and the required broader release rerun then
  passed clean, so the blocker classified as a transient suite-level lock flake
  rather than a deterministic Plan 04 regression.

## Review Loop

The required repeated `/GSD-Review-Tasks-Execution` runs converged on bounded
spec drift and then closed cleanly.

1. Task 6 review ran four times. Pass 2 found wording drift in
   `wallet_service_actions_receive.rs`; that fix made the future-only inbox
   boundary depend explicitly on both a live hint source and Task 15 closure.
   Passes 3 and 4 were consecutive clean runs with no significant issues.
2. Task 7 review ran three times. Pass 1 found backlog drift in `037-TODO.md`;
   that fix recorded the explicit de-scope branch for `ScanEngineImpl`. Passes 2
   and 3 were consecutive clean runs with no significant issues.

## Current Boundary

This summary closes only the Plan 04 inbox-defer and scan-engine de-scope slice
for Task 6 and Task 7. It does not claim closure of later Phase 037 API,
observability, RPC-alignment, or any future inbox-assisted receive
implementation work.
