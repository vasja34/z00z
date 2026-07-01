---
phase: 037-output-reception
artifact: tests-tasks
status: partial
source: 037-TEST-SPEC.md
updated: 2026-04-23
---

# Phase 037 Tests Tasks

## Purpose

📌 This document translates `037-TEST-SPEC.md` into one concrete
implementation order for test work.

📌 The execution order stays subordinate to the numbered planning chain. In
particular, gap-only expansion remains deferred to Task 9 in `037-09-PLAN.md`
after the later decision gates settle.

## Scope Inputs

- `037-TEST-SPEC.md`
- `037-CONTEXT.md`
- `037-TODO.md`
- `037-01-PLAN.md` through `037-10-PLAN.md`
- live code and existing receive-path test anchors in `crates/z00z_wallets`

## Current Execution Slice

- `037-TEST-EXECUTION-SUMMARY.md` records the currently landed T1 deterministic
  request-ordering slice and the narrow current T5 severity-contract slice.
- `crates/z00z_wallets/tests/test_phase037_output_reception.rs` also exists as
  a narrow duplicate-surface quarantine guard for the live Phase 037 tree. It
  is phase-local validation support, not proof that the broader Task 9 waves
  are closed.
- Later waves in this file remain open unless that summary or a future
  execution artifact says otherwise.

## Execution Strategy

- Freeze reuse anchors first so implementation does not duplicate already-green
  restart, route-gate, exact-`asset_id`, or strict prefilter coverage.
- Land deterministic request-ordering coverage before inbox-assisted or other
  service-boundary additions, because later receive helpers must inherit the
  same candidate-selection truth.
- Keep service-boundary assisted-receive tests ahead of wrapper or RPC
  reinforcement so canonical ownership and persistence semantics are pinned
  before optional surfaces are checked.
- Treat observability, callbacks, and any new integration file as conditional;
  add them only when code exists.
- Keep `ScanEngineImpl` and orphan duplicate seams quarantined unless a real
  implementation change rewires them.

## Task Waves

### Wave T0: Harness And Reuse Lock-In

- files to inspect:
  - `crates/z00z_wallets/src/services/test_wallet_service_suite.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs`
  - `crates/z00z_wallets/src/core/address/test_stealth_scan_support_suite.rs`
  - `crates/z00z_wallets/tests/test_stealth_scanner_prefilter.rs`
  - `crates/z00z_wallets/src/core/address/optimized_scanner.rs`
  - `crates/z00z_wallets/src/core/chain/scan_engine_impl.rs`
- deliverables:
  - confirmed reuse inventory for R1-R4 and P1
  - explicit note that `037-ARCHITECTURE.md` now exists as executed Task 0
    evidence, but does not by itself prove later numbered-plan closure
  - explicit confirmation that Task 9 remains deferred behind the later decision
    tasks
- completion gate:
  - no proposed new test duplicates baseline restart, persistence, or exact-id
    coverage

### Wave T1: Deterministic Request Ordering

- priority: highest new executable gap because later request-bound helpers must
  inherit the same candidate-resolution truth
- why now:
  - Task 15 changes the lowest-level request candidate seam
  - later assisted-receive scenarios are not trustworthy until order and expiry
    rules are pinned
- files to extend:
  - `crates/z00z_wallets/src/core/address/stealth_scanner/types_tag_cache.rs`
  - `crates/z00z_wallets/src/core/address/test_stealth_scan_support_suite.rs`
- files to create:
  - none
- implementation tasks:
  - add expiry-pruning coverage
  - add stable order coverage across repeated runs
  - add explicit `req_id = None` fallback-last coverage
  - add first-success short-circuit coverage
- success conditions:
  - repeated runs pick the same winner
  - expired request ids never win
  - fallback path runs only after explicit request candidates fail
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scan_support::tests::ordered_request_candidates_puts_fallback_last --lib -- --exact`
  - `cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scan_support::tests::scan_cached_keys_first_win --lib -- --exact`
  - `cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scan_support::tests::scan_owned_matches_request_bound_output --lib -- --exact`
  - `cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scanner::types::tests::test_active_requests_are_sorted_and_skip_expired --lib -- --exact`

### Wave T2: Service-Boundary Assisted Receive

- priority: second because inbox-assisted receive must prove it never becomes a
  second ownership or persistence engine
- why now:
  - Task 6 is service-boundary work, not detector work
  - it depends on deterministic request ordering when helper routing is
    request-bound
  - do not start this wave until Task 15 has closed deterministic ordered
    non-expired candidate selection or the inbox-assisted branch has been
    explicitly deferred
- files to extend:
  - `crates/z00z_wallets/src/services/test_wallet_service_suite.rs`
- files to create:
  - none unless the service boundary introduces a genuinely new seam later
- implementation tasks:
  - add hint hit coverage
  - add hint miss coverage
  - add false-positive hint coverage
  - prove hints never persist by themselves
  - prove helper routing remains request-bound, not stable-identity-bound
  - prove hinted and non-hinted range scans converge to the same claimed set and
    cursor
  - add partial-success alignment coverage only if mixed-result handling changes
- success conditions:
  - hints narrow candidates only
  - persistence still flows only through canonical `recv_range(...)` plus
    `recv_route(...)`
  - claimed asset set remains canonical
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast test_recv_route_gate -- --nocapture`

### Wave T3: Optional Wrapper Parity

- priority: third and conditional on `OptimizedScanner` remaining optional
- why now:
  - wrapper parity must not be used to imply canonicality
- files to extend:
  - `crates/z00z_wallets/src/core/address/optimized_scanner.rs`
- files to create:
  - none
- implementation tasks:
  - add parity checks over mine, not-mine, malformed, and request-bound leaves
  - document the parity oracle if order is intentionally parallelized
- success conditions:
  - no extra mine classification
  - no missing reject classification
  - no separate persistence or ownership behavior emerges
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast optimized_scanner -- --nocapture`

### Wave T4: Public RPC Receive Reinforcement

- priority: fourth because adapter reinforcement should follow service and
  wrapper truth, not lead it
- why now:
  - public surface must stay exact-`asset_id` only and aligned to
    `scan_asset_report(...)`
- files to extend:
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs`
- files to create:
  - none
- implementation tasks:
  - reuse `asset_receive_api_sync` and `asset_receive_path_parity`
  - add cases only if refactor drift changes outward status or invalid-proof
    handling
  - preserve definition-id rejection and non-persisting invalid-proof behavior
- success conditions:
  - outward status codes remain stable
  - exact `asset_id` contract stays intact
  - invalid-proof path does not persist claims
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast asset_receive_exact_asset_id_survives_definition_collision -- --nocapture`

### Wave T5: Observability And Callback Gates

- priority: conditional only
- why now:
  - Task 16 and Task 17 are explicitly branch-sensitive; tests must not invent
    hooks or callback surfaces that do not exist
  - the live RPC receive path keeps `ReceiveStatus::NotMine` non-alerting,
    while `InvalidInput`, `InvalidProof`, and `RuntimeFail` remain actionable
    warning-level failures
- files to extend:
  - closest implementation seam if observability or callback code exists
- files to create:
  - none by default
- implementation tasks:
  - if actionable observability lands, test `InvalidInput`, `InvalidProof`, and
    `RuntimeFail` hooks
  - explicitly prove `NotMine` stays non-alerting while actionable reject
    classes remain warning-level
  - if a callback or progress seam lands, test monotonic order and parity with
    persisted cursor progression
- success conditions:
  - no observability or callback tests are added for non-existent seams
  - any added hook remains bounded to actionable failures only
  - `NotMine` stays excluded from operator-facing alert paths
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast --lib --tests -- --nocapture`

### Wave T6: Final Residual Gap Sweep

- priority: last, because this is the actual Task 9 add-tests closure wave
- why now:
  - residual-gap tests must be computed after the decision gates settle, not
    before
- files to extend:
  - whichever existing anchor still owns the surviving gap
- files to create:
  - `crates/z00z_wallets/tests/test_phase037_output_reception.rs` only if a
    new integration seam or additional duplicate-surface quarantine invariant
    cannot truthfully fit any existing anchor
- implementation tasks:
  - diff the post-Task-17 codebase against `037-TEST-SPEC.md`
  - implement only the surviving unresolved scenarios
  - sync `037-TEST-SPEC.md`, `037-TESTS-TASKS.md`, and `037-TEST-PLAN.md` to
    match what was actually added
- success conditions:
  - no duplicate restart, route-gate, exact-id, or prefilter tests were added
  - only real residual gaps are covered
  - planning artifacts stay synchronized with implemented coverage
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast --lib --tests -- --nocapture`

## Review Checklist Per Wave

- [ ] Does the wave extend an existing anchor before proposing a new file?
- [ ] Does the scenario prove a real Phase 037 gap instead of re-proving an
      already-green baseline behavior?
- [ ] Does the assertion map to the live canonical seam named in
      `037-TEST-SPEC.md`?
- [ ] Does the test stay truthful about optional, conditional, or stub-only
      surfaces?
- [ ] Does the scenario preserve one canonical range receive lane, one
      persistence gate, and one public single-asset RPC surface?

## Done Condition

- the `reuse` scenarios remain explicit reference anchors
- the `extend` scenarios have concrete tests or a written blocker
- the `conditional` scenarios are implemented only when their code seam exists
- no test falsely promotes `ScanEngineImpl`, orphan duplicates, or compatibility
  lanes into canonical receive authority
- the resulting test set still proves one canonical range receive lane, one
  persistence gate, and one public single-asset RPC receive surface
