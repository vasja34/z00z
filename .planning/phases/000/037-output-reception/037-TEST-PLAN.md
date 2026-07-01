---
phase: 037-output-reception
artifact: test-plan
status: compatibility-alias
source: 037-TESTS-TASKS.md
updated: 2026-04-23
---

# Phase 037 Test Plan

## 🎯 Purpose

This file is the compatibility handoff for workflows that explicitly request
`037-TEST-PLAN.md`.

The canonical planning artifacts remain:

- `037-TEST-SPEC.md`
- `037-TESTS-TASKS.md`

This document does not introduce a third divergent plan. It summarizes the
approved planning state, the classification result, and the exact next gate for
future test implementation.

## ⚠️ Workflow Status

Phase 037 test planning is complete at the description level only.

- planning artifacts were derived from `037-CONTEXT.md`, `037-TODO.md`, live
  receive-path code, and existing test anchors
- numbered `037-01-PLAN.md` through `037-10-PLAN.md` artifacts now exist and
  govern execution order for the phase
- Phase 037 remains `fallback-ready`, not `verification-backed`
- `037-ARCHITECTURE.md` now exists as executed Task 0 evidence for the frozen
  baseline, but the broader phase still remains planning-led and not
  verification-backed
- `037-TEST-EXECUTION-SUMMARY.md` now records the landed T1 deterministic
  request-ordering slice and the narrow current T5 severity slice without
  claiming full backlog closure
- `crates/z00z_wallets/tests/test_phase037_output_reception.rs` already exists
  as a narrow duplicate-surface quarantine guard and must not be misread as
  evidence that the broader residual Task 9 backlog is closed
- deterministic request-ordering coverage now has live execution-backed deltas
  in `crates/z00z_wallets`
- non-alerting receive semantics are live in code and covered by reject-map
  assertions, but Phase 037 still has no dedicated log-capture or hook-count
  test anchor for those observability branches
- the broader residual add-tests wave remains governed by `037-09-PLAN.md`
  after the later decision-gated tasks settle

## 📦 Classification Outcome

### TDD And Integration Targets

- `crates/z00z_wallets/src/services/test_wallet_service_suite.rs`
  Canonical service-boundary receive path. Owns restart, persistence, inbox
  hint, and mixed-result range assertions.
- `crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs`
  Public single-asset receive adapter seam. Owns exact `asset_id`, outward
  status mapping, and definition-id rejection behavior.
- `crates/z00z_wallets/src/core/address/test_stealth_scan_support_suite.rs`
  Lowest-level deterministic request-ordering seam. Owns candidate iteration,
  expiry pruning, fallback ordering, and first-win behavior against the live
  `stealth_scan_support.rs` helper boundary.
- `crates/z00z_wallets/src/core/address/stealth_scanner/types_tag_cache.rs`
  Inline low-level request-registration seam. Owns sorted active-request
  storage and expiry-aware request registration.
- `crates/z00z_wallets/src/core/address/optimized_scanner.rs`
  Optional batching wrapper seam. Eligible only for parity-style unit tests.
- `crates/z00z_wallets/src/core/chain/scan_engine_impl.rs`
  Stub-truth seam. Eligible only for non-overclaim guard tests unless the stub
  is replaced.

### E2E Targets

- No browser E2E target is currently in scope.
- For Phase 037, end-to-end means realistic Rust integration flows across the
  wallet service, scanner, persistence cursor, and RPC adapter boundaries.

### Skip Targets

- `WalletService::receive_asset(...)`
  Compatibility-only reachability surface; not the canonical privacy lane.
- orphan duplicate surfaces such as `wallet_service_actions_runtime.rs` and
  `asset_impl_tests.rs`
  Cleanup/documentation concerns only unless they are explicitly rewired.
- speculative `ScanConfig`, `DoSMitigationConfig`, or richer callback facades
  that do not exist in live code

## ✅ Existing Anchors To Reuse

- `test_recv_range_restart`
- `test_claimed_asset_restart`
- `test_ex4_restart_resume`
- `test_recv_route_gate`
- `test_claimed_asset_rejects_invalid`
- `test_stays_live_post_rotate`
- `test_recv_ver_explicit`
- `test_recv_ver_save`
- `asset_receive_api_sync`
- `asset_receive_path_parity`
- `asset_receive_exact_asset_id_survives_definition_collision`
- `asset_receive_rejects_definition_id_query`
- `test_phase7_req_flow`
- `test_phase7_fast_reject`
- `test_phase7_collision`
- `test_phase9_bad_rpub`
- `test_phase9_tampered_tag16`
- `test_phase9_missing_fields`
- `test_active_requests_are_sorted_and_skip_expired`

These anchors are baseline evidence and must be extended only where Phase 037
introduces a real gap.

## 🧪 Planned Scenario Groups

### Group 1: Deterministic Request Candidate Resolution

What it proves:

- expired requests are skipped
- active request iteration is stable and deterministic
- `req_id = None` fallback is explicit and last
- first successful candidate terminates evaluation

Primary file:

- `crates/z00z_wallets/src/core/address/test_stealth_scan_support_suite.rs`
- `crates/z00z_wallets/src/core/address/stealth_scanner/types_tag_cache.rs`

### Group 2: Assisted Receive Without New Ownership Surface

What it proves:

- inbox hint hit, miss, and false-positive cases remain notify-only or
  candidate-local
- canonical receive ownership stays in `recv_range(...)`
- persistence still flows only through `recv_route(..., PersistClaim)`
- helper routing stays request-bound instead of depending on stable receiver
  identity
- hinted and non-hinted scans converge to the same persisted claim result

Primary file:

- `crates/z00z_wallets/src/services/test_wallet_service_suite.rs`

### Group 3: Optional Wrapper Parity

What it proves:

- `OptimizedScanner` does not invent new ownership logic
- wrapper output classification matches `StealthOutputScanner`
- malformed and request-bound leaves keep the same pass/fail semantics

Primary file:

- `crates/z00z_wallets/src/core/address/optimized_scanner.rs`

### Group 4: Public RPC Boundary Stability

What it proves:

- outward receive path still maps to `scan_asset_report(...)`
- exact `asset_id` semantics survive refactor drift
- definition-id queries remain rejected
- invalid proof paths remain non-persisting and outwardly stable

Primary file:

- `crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs`

### Group 5: Conditional Observability And Callback Hooks

What it proves when implemented:

- actionable failures have stable hooks
- the live RPC receive path keeps `ReceiveStatus::NotMine` non-alerting,
  while `InvalidInput`, `InvalidProof`, and `RuntimeFail` remain actionable
  warning-level failures
- callback order, if added, matches checkpoint progression

Primary seam:

- nearest implementation surface only if actual instrumentation or callback code
  lands in Phase 037

## 🔐 Critical Invariants To Preserve

- one canonical range receive lane: `WalletService::recv_range(...)`
- one canonical single-asset report lane: `WalletService::scan_asset_report(...)`
- one persistence decision gate: `WalletService::recv_route(...)`
- exact `asset_id` as public RPC lookup rule
- no claim persistence on malformed, tampered, or invalid-proof inputs
- monotonic cursor progression across chunked scans and restart
- no silent promotion of optional wrappers or stub engines into ownership truth

## 🛠️ Commands For Future Implementation Phase

```bash
cargo test -p z00z_wallets --release --features test-fast test_recv_range_restart -- --nocapture
cargo test -p z00z_wallets --release --features test-fast asset_receive_api_sync -- --nocapture
cargo test -p z00z_wallets --release --features test-fast asset_receive_exact_asset_id_survives_definition_collision -- --nocapture
cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scan_support::tests::ordered_request_candidates_puts_fallback_last --lib -- --exact
cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scan_support::tests::scan_cached_keys_first_win --lib -- --exact
cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scan_support::tests::scan_owned_matches_request_bound_output --lib -- --exact
cargo test -p z00z_wallets --release --features test-fast core::address::stealth_scanner::types::tests::test_active_requests_are_sorted_and_skip_expired --lib -- --exact
cargo test -p z00z_wallets --release --features test-fast test_phase7_req_flow -- --nocapture
cargo test -p z00z_wallets --release --features test-fast optimized_scanner -- --nocapture
cargo test -p z00z_wallets --release --features test-fast --lib --tests -- --nocapture
```

## 🚫 Current Stop Gate

This compatibility handoff remains planning-first, but it now coexists with
execution-backed deltas for deterministic request ordering and with live
code-backed non-alerting receive severity semantics.

The next residual add-tests stage is still allowed only when someone
explicitly wants to move from these landed deltas into broader gap closure and
the numbered plan chain has reached `037-09-PLAN.md` Task 9. Until then, the
phase-local deliverables are:

- `037-TEST-SPEC.md`
- `037-TESTS-TASKS.md`
- `037-TEST-PLAN.md`

## 🔗 Canonical References

- `037-TEST-SPEC.md`
- `037-TESTS-TASKS.md`
- `037-CONTEXT.md`
- `037-TODO.md`
