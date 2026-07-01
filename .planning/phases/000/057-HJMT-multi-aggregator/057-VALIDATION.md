---
phase: 057
slug: hjmt-multi-aggregator
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-14
validated_at: 2026-06-14
---

# Phase 057 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## 🧪 Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` with bench-home compile probes |
| **Config file** | Workspace `Cargo.toml` files plus `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release` |
| **Estimated runtime** | ~1300 seconds for the full Phase 057 targeted matrix |

## 📡 Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run the owning targeted Phase 057 command set from the Per-Task Verification Map
- **Before `/gsd-verify-work`:** The full targeted Phase 057 matrix must be green; `cargo test --release` remains the broad workspace lane
- **Max feedback latency:** 972 seconds on the current long-running `test_scenario1_stage_surface` lane

## ✅ Coverage Summary

- Automated coverage exists for every execution-backed Phase 057 test task
  `057-TT-01` through `057-TT-09`.
- No Wave 0 stubs or framework-install work is needed; the repository already
  contains the live test homes, fixture homes, simulator packet, and bench
  anchors required by `057-TEST-SPEC.md` and `057-TESTS-TASKS.md`.
- No manual-only Phase 057 behaviors remain. The phase is Nyquist-compliant on
  the current live tree.

## 🗺️ Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `057-TT-01` | `01` | `1` | `057-SC-01`, `057-G1`, `057-G2` | `T-057-01` | Root-generation stays explicit, fail-closed, and byte-canonical on the storage-owned seam. | unit / integration | `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture` | ✅ | ✅ green |
| `057-TT-02` | `01` | `1` | `057-SC-02`, `057-G2`, `057-G3` | `T-057-02`, `T-057-03`, `T-057-08` | Checkpoint publication keeps one ordered leaf set, one prior-root story, and one runtime route-binding story. | integration | `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_publish -- --nocapture` | ✅ | ✅ green |
| `057-TT-03` | `02` | `2` | `057-SC-03`, `057-SC-04`, `057-G4` | `T-057-04`, `T-057-05`, `T-057-06` | Public proof composition stays layered above shard-local proof truth and rejects wrong-lineage or cross-shard rows. | integration | `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture` | ✅ | ✅ green |
| `057-TT-04` | `03` | `3` | `057-SC-05`, `057-SC-06`, `057-G5`, `057-G11` | `T-057-07`, `T-057-08`, `T-057-09`, `T-057-15`, `T-057-17` | `SIM-5A7S-PUB` stays YAML-driven, topology-generic, lineage-linked, and trace-complete. | integration / E2E | `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_hjmt_runtime_config -- --nocapture`; `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement -- --nocapture`; `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |
| `057-TT-05` | `04` | `4` | `057-SC-07`, `057-SC-08`, `057-G6`, `057-G7` | `T-057-10` | Standby join and owner activation remain separate protocol states and fail closed before lawful activation. | integration / E2E | `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_join -- --nocapture`; `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture` | ✅ | ✅ green |
| `057-TT-06` | `04-05` | `4-5` | `057-SC-09`, `057-SC-10`, `057-SC-11`, `057-G7`, `057-G8`, `057-G9`, `057-G10` | `T-057-11`, `T-057-12`, `T-057-13`, `T-057-14`, `T-057-20` | Route-generation transfer, carry-forward, crash recovery, downstream digest sameness, and first-scope continuity all stay on one canonical publication contract. | integration / E2E | `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_migrate -- --nocapture`; `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_scope_birth -- --nocapture`; `cargo test -p z00z_validators --release`; `cargo test -p z00z_watchers --release` | ✅ | ✅ green |
| `057-TT-07` | `06` | `6` | `057-SC-12`, `057-G11` | `T-057-16`, `T-057-17`, `T-057-18`, `T-057-21` | Guardrails fail if a second publication, proof, validator, watcher, or simulator authority lane appears. | integration / guardrail | `cargo test -p z00z_aggregators --release --features test-params-fast --test test_live_guardrails -- --nocapture`; `cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails -- --nocapture` | ✅ | ✅ green |
| `057-TT-08` | `06` | `6` | `057-SC-13`, `057-G11` | `T-057-16`, `T-057-17`, `T-057-18` | Bench lanes stay on accepted storage homes only, and reserved profile claims remain honest. | integration / bench probe | `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture` | ✅ | ✅ green |
| `057-TT-09` | `07` | `7` | `057-SC-12`, `057-SC-13`, `057-G11` | `T-057-19`, `T-057-20`, `T-057-21` | Continuation code, traces, and planning packet stay aligned on one shared `bind_publication_contract(...)` path. | integration / guardrail | `cargo test -p z00z_aggregators --release --features test-params-fast --test test_live_guardrails -- --nocapture`; `cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails -- --nocapture`; `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## 🌊 Wave 0 Requirements

Existing infrastructure covers all phase requirements.

## 🛠️ Manual-Only Verifications

All phase behaviors have automated verification.

## 🧾 Validation Audit Trail

| Audit Date | Gaps Found | Resolved | Escalated | Run By |
|------------|------------|----------|-----------|--------|
| 2026-06-14 | 0 | 0 | 0 | Codex `gsd-validate-phase` |

## 🔬 Verification Evidence

- Workspace-first discovery confirmed State B for this workflow: phase
  summaries existed, while `057-VALIDATION.md` did not.
- Cross-read of `057-01-PLAN.md` through `057-07-PLAN.md`,
  `057-TEST-SPEC.md`, and `057-TESTS-TASKS.md` showed that every Phase 057
  scenario and gate already maps to one live test or bench home.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  on the live tree in this validation pass.
- `cargo test -p z00z_storage --release --features test-params-fast` passed on
  the live tree in this validation pass, including `test_bench_lanes`,
  `test_hjmt_root_generation`, `test_hjmt_historical_proofs`,
  `test_hjmt_live_proof_families`, `test_hjmt_scope_birth`, and
  `test_live_guardrails`.
- `cargo test -p z00z_aggregators --release --features test-params-fast`
  passed on the live tree in this validation pass, including
  `test_hjmt_publish`, `test_hjmt_join`, `test_hjmt_migrate`,
  `test_hjmt_failover_same_lineage`, `test_hjmt_split_brain_fencing`, and
  `test_live_guardrails`.
- `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture`
  passed on the live tree in this validation pass.
- `cargo test -p z00z_validators --release` passed on the live tree in this
  validation pass.
- `cargo test -p z00z_watchers --release` passed on the live tree in this
  validation pass.
- `cargo test -p z00z_simulator --release --features test-params-fast
  --features wallet_debug_tools --test test_hjmt_runtime_config
  -- --nocapture` passed on the live tree in this validation pass.
- `cargo test -p z00z_simulator --release --features test-params-fast
  --features wallet_debug_tools --test test_scenario_settlement
  -- --nocapture` passed on the live tree in this validation pass.
- `cargo test -p z00z_simulator --release --features test-params-fast
  --features wallet_debug_tools --test test_scenario1_stage_surface
  -- --nocapture` passed on the live tree in this validation pass with
  `22 passed; 0 failed` and a measured runtime of `971.98s`.

## ✅ Validation Sign-Off

- [x] All tasks have automated verify coverage or existing live infrastructure
- [x] Sampling continuity: no three consecutive tasks lack an automated lane
- [x] Wave 0 is not needed because no MISSING references remain
- [x] No watch-mode flags are required for Phase 057 verification
- [x] Feedback latency is known and bounded by the current long-running
  simulator lane
- [x] `nyquist_compliant: true` is set in frontmatter

**Approval:** verified 2026-06-14
