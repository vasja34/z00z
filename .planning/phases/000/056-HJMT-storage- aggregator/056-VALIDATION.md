---
phase: 056
slug: hjmt-storage-aggregator
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-12
---

# Phase 056 — Validation Strategy

> Audited Nyquist validation contract for Phase 056 against the executed plan
> chain, numbered summaries, current live rerun evidence, the verified security
> register, and the phase-local test packet. All phase requirements now have
> green automated verification on the current live tree.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` plus `cargo bench --no-run` compile gates |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release` |
| **Estimated runtime** | ~600 seconds |

---

## Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run the phase-local release matrix for the touched owner homes plus any required bench `--no-run` gates
- **Before `/gsd-verify-work`:** Rerun the focused simulator matrix and `cargo test --release`; both are currently green on the live tree
- **Max feedback latency:** ~600 seconds

---

## Evidence Basis

- **Current rerun on 2026-06-12:** this validation pass relies on the same
  live-tree reruns completed in the current working session after the Stage 13
  cache stabilization fix: bootstrap, the broad workspace `cargo test --release`
  gate, the focused simulator regressions (`test_pipeline_genesis_tx`,
  `test_scenario_settlement`, `test_scenario1_stage_surface`), and both
  required storage bench `--no-run` compile gates.
- **Current broad-gate truth:** `cargo test --release` completed green on the
  live tree after the shared Stage 13 cache stabilization fix. The earlier RedB
  lock and persisted-claim-store failures are no longer reproducible in the
  current rerun set.
- **Current phase-local simulator truth:** `test_scenario_settlement` and
  `test_scenario1_stage_surface` both completed green on the live tree, and the
  targeted `test_pipeline_genesis_tx` regression also passed after the cache
  stabilization fix. `056-06-SUMMARY.md` already records a green
  `test_hjmt_runtime_config` rerun in the same stabilized state.
- **Execution-backed closeout evidence from 2026-06-12:** `056-06-SUMMARY.md`
  and `056-07-SUMMARY.md` record green focused simulator commands, a green
  workspace `cargo test --release`, and the required
  `/GSD-Review-Tasks-Execution` manual fallback loop with at least three passes
  and two consecutive clean results.
- **Truthfulness rule:** `✅ green` below is reserved for current-session live
  reruns or execution-backed summary evidence that remains consistent with the
  current green workspace state. No phase row below relies on superseded red
  evidence.

---

## Current Evidence Commands

Current reruns in this validation pass:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh

cargo test --release

cargo test -p z00z_simulator --release --features test-params-fast \
  --features wallet_debug_tools --test test_pipeline_genesis_tx -- --nocapture

cargo test -p z00z_simulator --release --features test-params-fast \
  --features wallet_debug_tools --test test_scenario_settlement -- --nocapture

cargo test -p z00z_simulator --release --features test-params-fast \
  --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture

cargo bench -p z00z_storage --bench settlement_shard --no-run
cargo bench -p z00z_storage --bench settlement_hjmt --no-run
```

Execution-backed focused commands retained from the numbered closeout
summaries:

```bash
cargo test -p z00z_aggregators --release --features test-params-fast \
  --test test_hjmt_planner \
  --test test_hjmt_shard_routing \
  --test test_hjmt_failover_same_lineage \
  --test test_hjmt_split_brain_fencing \
  --test test_live_guardrails \
  -- --nocapture

cargo test -p z00z_rollup_node --release --features test-params-fast \
  --test test_hjmt_topology \
  --test test_hjmt_process \
  --test test_hjmt_node_lifecycle \
  --test test_hjmt_preflight \
  -- --nocapture

cargo test -p z00z_storage --release --features test-params-fast \
  --test test_hjmt_scope_birth \
  --test test_live_guardrails \
  --test test_bench_lanes \
  -- --nocapture
```

---

## Requirement Coverage Summary

| Requirement | Status | Evidence |
|-------------|--------|----------|
| `056-G1` runtime topology contract | COVERED | Current green workspace reruns plus `056-01-SUMMARY.md` keep `test_hjmt_topology.rs` and the `SIM-5A7S` manifest aligned on the live tree. |
| `056-G2` OS-process lifecycle contract | COVERED | Current green workspace reruns plus `056-01-SUMMARY.md` keep `test_hjmt_process.rs` and `test_hjmt_node_lifecycle.rs` green on the live tree. |
| `056-G3` YAML-backed runtime home | COVERED | Current green workspace reruns, bootstrap, and the numbered summaries keep `test_hjmt_preflight.rs` and the disk-backed config home live. |
| `056-G4` route and planner ownership | COVERED | Current green workspace reruns plus `056-02-SUMMARY.md` keep `test_hjmt_shard_routing.rs`, `test_hjmt_planner.rs`, and runtime guardrails green. |
| `056-G5` semantic-only runtime-to-storage handoff | COVERED | Current green workspace reruns plus `056-03-SUMMARY.md` keep `test_hjmt_scope_birth.rs` and storage guardrails green on the live tree. |
| `056-G6` first-scope birth and restart safety | COVERED | Current green workspace reruns plus closeout evidence keep `test_hjmt_scope_birth.rs` and `test_live_recovery` green for the executed phase. |
| `056-G7` lawful failover path | COVERED | Current green workspace reruns plus `056-04-SUMMARY.md` keep `test_hjmt_failover_same_lineage.rs` and `test_hjmt_split_brain_fencing.rs` green. |
| `056-G8` journal and lineage fencing | COVERED | Current green workspace reruns plus `056-04-SUMMARY.md` and `056-07-SUMMARY.md` keep the failover, recovery, and journal-fencing owner homes green. |
| `056-G9` startup fail-closed preflight | COVERED | Current green workspace reruns plus `056-05-SUMMARY.md` keep `test_hjmt_preflight.rs` green on the live tree. |
| `056-G10` simulator runtime observability and trace closure | COVERED | Current live reruns passed `test_pipeline_genesis_tx`, `test_scenario_settlement`, `test_scenario1_stage_surface`, and the broad `cargo test --release` gate after Stage 13 cache stabilization; `056-06-SUMMARY.md` also records a green `test_hjmt_runtime_config` rerun in the stabilized state. |

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `056-TT-01` | `01` | `1` | `056-G1` | `T-056-01`, `T-056-03`, `T-056-13` | `SIM-5A7S` stays a canonical fixture only, topology remains generic, and malformed runtime shapes reject. | e2e | `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_topology -- --nocapture` | ✅ | ✅ green |
| `056-TT-02` | `01` | `1` | `056-G2` | `T-056-01`, `T-056-03` | The accepted runtime path stays OS-process based with separate lifecycle control, paths, ports, and startup references. | e2e | `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_process --test test_hjmt_node_lifecycle -- --nocapture` | ✅ | ✅ green |
| `056-TT-03` | `02` | `2` | `056-G4` | `T-056-04`, `T-056-05` | Route-table bytes, digest, and generation stay runtime-owned and tamper rows reject fail-closed. | fixture + integration | `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_shard_routing -- --nocapture` | ✅ | ✅ green |
| `056-TT-04` | `02` | `2` | `056-G4` | `T-056-05`, `T-056-06` | Central and per-aggregator planners agree on accepted digest and reject semantics. | integration | `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_planner -- --nocapture` | ✅ | ✅ green |
| `056-TT-05` | `03` | `3` | `056-G5`, `056-G6` | `T-056-07`, `T-056-08`, `T-056-09` | Runtime stays semantic-only, storage remains subtree and proof owner, and first-scope birth stays restart-safe. | integration | `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_scope_birth --test test_live_guardrails -- --nocapture` | ✅ | ✅ green |
| `056-TT-06` | `04` | `4` | `056-G7`, `056-G8` | `T-056-10`, `T-056-11`, `T-056-12` | Same-lineage takeover is the only legal path, and wrong-lineage or stale-restart states reject explicitly. | integration | `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage --test test_hjmt_split_brain_fencing -- --nocapture` | ✅ | ✅ green |
| `056-TT-07` | `05` | `5` | `056-G3`, `056-G9` | `T-056-02`, `T-056-13`, `T-056-14`, `T-056-15` | Config surfaces remain disk-backed, behavior-changing, and fail-closed before live work starts. | e2e | `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture` | ✅ | ✅ green |
| `056-TT-08` | `06` | `6` | `056-G10` | `T-056-16`, `T-056-17`, `T-056-18` | The simulator must prove the live runtime plane, trace-pack linkage, and design/runtime sync without detached or stale evidence. | e2e | `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement --test test_scenario1_stage_surface --test test_hjmt_runtime_config -- --nocapture` | ✅ | ✅ green |
| `056-TT-09` | `07` | `7` | `056-G1`..`056-G9` closeout benches | `T-056-19`, `T-056-20`, `T-056-21` | Required shard and cache-edge bench lanes stay in the existing storage bench homes only. | bench-guard + bench compile | `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture && cargo bench -p z00z_storage --bench settlement_shard --no-run && cargo bench -p z00z_storage --bench settlement_hjmt --no-run` | ✅ | ✅ green |
| `056-TT-10` | `07` | `7` | `056-G10` closeout guardrails | `T-056-05`, `T-056-07`, `T-056-12`, `T-056-16` | Duplicate planner authority, duplicate semantic storage authority, shared WAL truth, and a second simulator evidence lane must all fail closed. | source-shape + integration | `cargo test -p z00z_aggregators --release --features test-params-fast --test test_live_guardrails -- --nocapture && cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails -- --nocapture && cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

No new framework installation, shared fixture bootstrap, or test-home scaffolding
is missing for Phase 056. No Wave 0 validation gaps remain.

---

## Manual-Only Verifications

All Phase 056 behaviors have automated verification homes.

No manual-only validation rows are required on the current live tree.

---

## Open Gaps And Watchpoints

No open Nyquist coverage gaps remain on the current live tree.

The only noteworthy regression watchpoint is historical: the simulator lane had
previous Stage 13 cache drift and lock-related instability, but the stabilized
cache-root flow and refreshed content fingerprint closed those failures in the
current rerun set. This is now a maintenance watchpoint, not a validation gap.

---

## Validation Audit 2026-06-12

| Metric | Count |
|--------|-------|
| Plan auto-tasks audited | 10 |
| Current-green tasks | 10 |
| Flaky or red tasks | 0 |
| Workspace-wide red gates | 0 |
| Manual-only rows | 0 |
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

---

## Validation Sign-Off

- [x] All tasks have automated verification or execution-backed release evidence
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all missing references because no missing references remain
- [x] No watch-mode flags are used
- [x] Feedback latency stays within the documented phase-local budget
- [x] Phase-local simulator evidence lane is stable on the current live tree
- [x] Full workspace `cargo test --release` is green on the current live tree
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-06-12

---

## Reconstruction Notes

This file was originally reconstructed under validate-phase State B and then
updated under State A from:

- `.planning/phases/056-HJMT-storage- aggregator/056-TEST-SPEC.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-TESTS-TASKS.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-01-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-02-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-03-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-04-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-05-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-06-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-07-PLAN.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-01-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-02-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-03-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-04-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-05-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-06-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-07-SUMMARY.md`
- `.planning/phases/056-HJMT-storage- aggregator/056-SECURITY.md`
- the current live outputs of the rerun commands listed above and the
  stabilized green reruns completed in the current working session

Generated test files in this validation pass:

- none — existing Phase 056 owner homes already exist, and no new Nyquist gaps
  required additional test files.
