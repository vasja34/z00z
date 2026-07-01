---
phase: 053
slug: hjmt-backend
status: audited
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-06
updated: 2026-06-08
---

# Phase 053 - Validation Strategy

Reconstructed Nyquist validation contract for Phase 053 from completed plan,
summary, security, and live test artifacts.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust workspace `cargo test` plus repository bootstrap checks and packet-truth source guards |
| **Config file** | Workspace [Cargo.toml](/home/vadim/Projects/z00z/Cargo.toml) |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | ~1800 seconds, workspace and cache dependent |

## Sampling Rate

- After every Rust or test-affecting task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- After every plan wave: run the narrow task-local cargo tests from the owning `053-XX-PLAN.md`, then `cargo test --release --features test-fast --features wallet_debug_dump`
- Before `/gsd-verify-work`: full suite must be green
- Max feedback latency: bounded by bootstrap plus the targeted phase-local crate tests for the active slice

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `T053-01` | 01 | 1 | `PH53-01` | `T-053-01 / 02 / 03 / 25` | Live settlement exports remain required, physical layout stays non-authoritative, and stale aliases cannot impersonate live roots. | integration + source guard | `cargo test -p z00z_storage --release --features test-fast --test test_live_guardrails --test test_default_gate -- --nocapture` | ✅ | ✅ green |
| `T053-02` | 02 | 2 | `PH53-02` | `T-053-05 / 17` | Settlement roots, generations, and checkpoints remain semantically bound and reject mismatched root or generation substitutions. | integration | `cargo test -p z00z_storage --release --features test-fast --test test_settlement_root --test test_checkpoint_store_api -- --nocapture` | ✅ | ✅ green |
| `T053-03` | 03 | 3 | `PH53-03` | `T-053-02 / 04 / 05` | `SettlementPath` and `RightLeaf` stay on one semantic surface while family confusion and wrong proof binding reject. | integration | `cargo test -p z00z_storage --release --features test-fast --test test_store_api --test test_hjmt_live_proof_families --test test_genesis_ingestion -- --nocapture` | ✅ | ✅ green |
| `T053-04` | 04 | 4 | `PH53-04` | `T-053-15` | Fee support remains separate from rights meaning and replay rejection survives reload and recovery. | integration | `cargo test -p z00z_storage --release --features test-fast --test test_fee_envelope --test test_fee_replay -- --nocapture` | ✅ | ✅ green |
| `T053-05` | 05 | 5 | `PH53-05` | `T-053-02 / 18 / 25` | HJMT remains the only live storage mode and downstream callers must stay on semantic APIs instead of backend-private ids. | integration + source guard | `cargo test -p z00z_storage --release --features test-fast --test test_store_api --test test_downstream_guardrails --test test_default_gate -- --nocapture` | ✅ | ✅ green |
| `T053-06` | 06 | 6 | `PH53-06` | `T-053-19` | Canonical rights config and genesis inputs generate deterministic mixed settlement corpus and storage ingests the produced rights without drift. | integration | `cargo test -p z00z_core --release --features test-fast --test assets_tests --test genesis_tests -- --nocapture && cargo test -p z00z_storage --release --features test-fast --test test_genesis_ingestion -- --nocapture` | ✅ | ✅ green |
| `T053-07` | 07 | 7 | `PH53-07` | `T-053-04 / 05 / 06` | Inclusion, deletion, and non-existence proof families stay root-, family-, and journal-bound and reject rebinding. | integration | `cargo test -p z00z_storage --release --features test-fast --test test_hjmt_live_proof_families --test test_settlement_root -- --nocapture` | ✅ | ✅ green |
| `T053-08` | 08 | 8 | `PH53-08` | `T-053-09 / 23` | Adaptive split, merge, and policy-transition proofs stay on live storage APIs and reject stale or policy-invalid transitions. | integration + bench compile | `cargo test -p z00z_storage --release --features test-fast --test test_hjmt_adaptive_policy_proofs --test test_bench_lanes -- --nocapture` | ✅ | ✅ green |
| `T053-09` | 09 | 9 | `PH53-09` | `T-053-07 / 08 / 09` | Occupancy evidence stays threshold-bounded and proof-visible payloads never expose raw counts or underreported policy state. | integration | `cargo test -p z00z_storage --release --features test-fast --test test_occupancy_privacy --test test_occupancy_evidence -- --nocapture` | ✅ | ✅ green |
| `T053-10` | 10 | 10 | `PH53-10` | `—` | Cache reuse stays a private optimization only and recomputation fails closed on cache mismatch. | integration | `cargo test -p z00z_storage --release --features test-fast --test test_forest_cache --test test_cache_recompute -- --nocapture` | ✅ | ✅ green |
| `T053-11` | 11 | 11 | `PH53-11` | `T-053-10 / 11 / 12 / 24` | Scheduler ordering stays deterministic, bounded, backpressured, and rollback-safe under cancellation. | integration | `cargo test -p z00z_storage --release --features test-fast --test test_async_scheduler -- --nocapture` | ✅ | ✅ green |
| `T053-12` | 12 | 12 | `PH53-12` | `T-053-13 / 14 / 15` | Journal recovery rejects missing or stale rows and resolves crashes to either previous or complete next state only. | integration | `cargo test -p z00z_storage --release --features test-fast --test test_hjmt_adaptive_policy_proofs --test test_redb_reload -- --nocapture` | ✅ | ✅ green |
| `T053-13` | 13 | 13 | `PH53-13` | `T-053-13 / 14` | RedB reload reproduces semantic state and historical-proof lanes reject stale or unsupported durable shapes. | integration | `cargo test -p z00z_storage --release --features test-fast --test test_redb_reload --test test_bench_lanes -- --nocapture` | ✅ | ✅ green |
| `T053-14` | 14 | 14 | `PH53-14` | `T-053-16 / 17 / 18` | Downstream consumers verify through semantic roots and storage-owned proof blobs and reject `RightLeaf` as wallet asset inventory. | cross-crate integration | `cargo test -p z00z_storage --release --features test-fast --test test_downstream_guardrails -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --test test_stage7_jmt_wallet_scan --test test_s7_examples -- --nocapture` | ✅ | ✅ green |
| `T053-15` | 15 | 15 | `PH53-15` | `T-053-16 / 17` | Scenario output remains rights-aware, mixed-family, and replay-verifiable across Stage 1, 4, 6, 11, and 13 paths. | cross-crate integration | `cargo test -p z00z_simulator --release --features test-fast --test test_scenario_settlement --test test_scenario1_stage_surface --test test_s7_examples -- --nocapture && cargo test -p z00z_wallets --release --features test-fast --test test_s7_examples -- --nocapture` | ✅ | ✅ green |
| `T053-16` | 16 | 16 | `PH53-16` | `T-053-19 / 20 / 21` | Golden corpus, property lanes, and malformed-proof fuzz seeds stay deterministic, storage-owned, and panic-safe. | integration + property + fuzz compile | `cargo test -p z00z_storage --release --features test-fast --test test_golden_corpus --test test_property_corpus --test test_fuzz_seeds -- --nocapture && cargo test -p z00z_core --release --features test-fast --test genesis_tests test_settlement_corpus -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --test test_scenario_settlement -- --nocapture` | ✅ | ✅ green |
| `T053-17` | 17 | 17 | `PH53-17` | `T-053-22 / 23 / 24` | Bench lanes compile against live HJMT APIs and metrics remain bounded diagnostics rather than proof authority. | integration + bench compile | `cargo test -p z00z_storage --release --features test-fast --test test_bench_lanes --test test_metrics -- --nocapture` | ✅ | ✅ green |
| `T053-18` | 18 | 18 | `PH53-18` | `T-053-03 / 27` | README and API examples stay executable and docs cannot drift back to future-only or compatibility wording. | doc guard + integration | `cargo test -p z00z_storage --release --features test-fast --test test_readme_examples --test test_live_guardrails -- --nocapture` | ✅ | ✅ green |
| `T053-19` | 19 | 19 | `PH53-19` | `T-053-25 / 26 / 27` | Unset backend mode resolves to HJMT, stale aliases reject, and the broad workspace release gate remains the closeout proof. | integration + full suite | `cargo test -p z00z_storage --release --features test-fast --test test_default_gate -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| `T053-20` | 20 | 20 | `PH53-20` | `T-053-25 / 26 / 27` | Compatibility/simple-JMT runtime tails stay deleted from the live tree and the phase packet remains truthful about the surviving live test owners. | source guard + integration | `cargo test -p z00z_storage --release --features test-fast --test test_live_guardrails -- --nocapture` | ✅ | ✅ green |

Status legend: `⬜ pending` · `✅ green` · `❌ red` · `⚠️ flaky`

## Wave 0 Requirements

Existing infrastructure covers all Phase 053 requirements.

No new framework install, fixture bootstrap, or helper harness was required
beyond the repository-standard bootstrap gate, targeted crate tests, and the
workspace release gate.

## Manual-Only Verifications

All Phase 053 behaviors have automated verification.

The original `/GSD-Review-Tasks-Execution` prompt runner is not callable in
this executor environment, but the executed phase packet already records the
required review-loop evidence and the current validation rerun rechecked the
live command-backed proof surface directly.

## Validation Audit 2026-06-06

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

Gap resolved:

- Removed stale closeout bullets in
  [053-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/053-HJMT-Backend/053-SUMMARY.md)
  and
  [053-20-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/053-HJMT-Backend/053-20-SUMMARY.md)
  that incorrectly treated
  `crates/z00z_storage/tests/test_golden_corpus.rs` as deleted legacy
  scaffolding after it had already become the live `PH53-16` corpus owner.

## Execution Evidence 2026-06-06

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_storage --release --features test-fast --test test_default_gate --test test_golden_corpus --test test_live_guardrails -- --nocapture`
- `cargo test -p z00z_core --release --features test-fast --test assets_tests --test genesis_tests --no-run`
- `cargo test -p z00z_simulator --release --features test-fast --test test_scenario_settlement -- --nocapture`
- First `cargo test --release --features test-fast --features wallet_debug_dump` rerun failed on unrelated `z00z_core` perf threshold test `genesis::batch_verification::test_bench_vs_batch_100` with observed speedup `1.68x`
- `cargo test -p z00z_core --release --features test-fast --test genesis_tests genesis::batch_verification::test_bench_vs_batch_100 -- --nocapture` reran green with observed speedup `3.15x`
- Second `cargo test --release --features test-fast --features wallet_debug_dump` rerun passed green after the warmed exact-test pass
- `git diff --check -- .planning/phases/053-HJMT-Backend/053-SUMMARY.md .planning/phases/053-HJMT-Backend/053-20-SUMMARY.md .planning/phases/053-HJMT-Backend/053-VALIDATION.md`

## Reopen Validation 2026-06-08

- Narrow Phase 053 reopen scope: live operator env-var truth drift in
  `docs/tech-papers/Z00Z-HJMT-Design.md` plus a matching regression guard in
  `crates/z00z_storage/tests/test_live_guardrails.rs`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` — passed
- `cargo test -p z00z_storage --release --test test_live_guardrails -- --nocapture` — passed
- Exact required broad command
  `cargo test --release --features test-fast --features wallet_debug_dump` —
  failed immediately because the current workspace no longer exposes
  `test-fast` or `wallet_debug_dump`
- Live-equivalent touched-owner sweep
  `cargo test -p z00z_storage --release --features test-params-fast` — passed
- Manual `/GSD-Review-Tasks-Execution` equivalent review loop ran 3 passes on
  the reopen scope; pass 1 confirmed the live doc-truth drift, passes 2 and 3
  were clean after the fix
- `git diff --check` — passed

## Validation Sign-Off

- [x] All tasks have automated verify coverage or existing infrastructure coverage
- [x] Sampling continuity: no 3 consecutive tasks without automated verify evidence
- [x] Wave 0 covers all missing references
- [x] No watch-mode flags
- [x] Feedback latency stays bounded by bootstrap plus targeted phase-local checks
- [x] `nyquist_compliant: true` set in frontmatter

Approval: approved 2026-06-06

## Reconstruction Notes

This file was reconstructed under validate-phase State B because
`053-VALIDATION.md` was missing and `gsd-sdk` is unavailable in this executor
environment.

Inputs used for the reconstruction:

- `053-01-PLAN.md` through `053-20-PLAN.md`
- `053-01-SUMMARY.md` through `053-20-SUMMARY.md`
- `053-SUMMARY.md`
- `053-TEST-SPEC.md`
- `053-TESTS-TASKS.md`
- `053-SOURCE-AUDIT.md`
- `053-SECURITY.md`

Gap audit result: `0` missing automated requirement-to-test coverage gaps and
`1` packet-truth drift gap, resolved in scope.
