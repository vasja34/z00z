---
phase: 020
slug: refactor-scenario-1
status: completed
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-25
updated: 2026-03-27
---

# Phase 020 - Validation Strategy

📌 Per-phase validation contract for Scenario 1 stage-splitting work, deeper
YAML synchronization, broader scenario_1 cleanup, and release-style
anti-regression gates.

📌 The target post-refactor shape is one explicit 12-stage Scenario 1 map:
`genesis_init`, `wallet_create`, `claim_prepare`, `claim_publish`, `tx_plan`,
`tx_prepare`, `transfer_receive`, `transfer_claim`, `bundle_build`,
`bundle_publish`, `checkpoint_apply_storage`, `checkpoint_finalize`.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_scenario1_stage_surface -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_chain_path -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage5_receive_bridge -- --nocapture` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --tests -- --nocapture` |
| **Estimated runtime** | ~180 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task-local `<automated>` command from
  the owning PLAN file, including the release-flagged simulator command when
  the touched target is under `z00z_simulator`.
- **After every plan wave:** Run the relevant release quick-run gate plus the
  full suite command for the simulator test surface.
- **Before `/gsd-verify-work`:** `cargo test --release --features test-fast
  --features wallet_debug_dump -p z00z_simulator --tests -- --nocapture` and
  `cargo run --release -p z00z_simulator --bin scenario_1 --features
  wallet_debug_dump` must be green.
- **Max feedback latency:** ~180 seconds.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | --------- | ----------------- | ----------- | ------ |
| 020-01-01 | 01 | 1 | SCN1-04 | source-structure | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |
| 020-01-02 | 01 | 1 | SCN1-04 | integration | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_claim_acceptance -- --nocapture` | ✅ | ✅ green |
| 020-02-01 | 02 | 2 | SCN1-04 | source-structure | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_source_shape -- --nocapture` | ✅ | ✅ green |
| 020-02-02 | 02 | 2 | SCN1-04 | integration | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_chain_path -- --nocapture` | ✅ | ✅ green |
| 020-03-01 | 03 | 3 | SCN1-03 | source-structure | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage5_source_shape -- --nocapture` | ✅ | ✅ green |
| 020-03-02 | 03 | 3 | SCN1-03 | integration | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint_storage_bridge -- --nocapture` | ✅ | ✅ green |
| 020-04-01 | 04 | 4 | SCN1-05 | integration/source-structure | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage8_proof_path -- --nocapture` | ✅ | ✅ green |
| 020-04-02 | 04 | 4 | SCN1-05 | phase gate | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |

📌 Status legend: `⬜ pending` · `✅ green` · `❌ red` · `⚠️ flaky`

---

## Wave 0 Requirements

- [x] `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` — explicit
  guardrail for the explicit 12-stage Scenario 1 stage map.
- [x] `crates/z00z_simulator/tests/test_stage4_source_shape.rs` — Stage 4
  stage-split regression for `SCN1-04`.
- [x] `crates/z00z_simulator/tests/test_stage5_source_shape.rs` — Stage 5
  stage-split regression for `SCN1-03`.

## Current 020-01 Evidence

- Release guard passed: `test_scenario1_stage_surface`.
- Release claim-lane set passed: `test_claim_acceptance`, `test_claim_emit`,
  `test_claim_tx_pipeline`, `test_claim_snapshot`, `test_claim_persist`,
  `test_claim_integration`.
- Stage 3 helper seam is now materialized under
  `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` and
  `crates/z00z_simulator/src/scenario_1/stage_3_utils/audit.rs` while the
  public Stage 3 API remains stable.

## Current 020-02 Evidence

- Release guard passed again: `test_scenario1_stage_surface`.
- Release tx-lane structure passed: `test_stage4_source_shape`.
- Release tx-lane failure-path guard passed: `test_stage4_tamper`.
- Release path remap guard passed: `test_stage4_cfg_paths`.
- Release continuity guard passed: `test_stage4_chain_path`.
- Release cross-stage pipeline guard passed: `test_pipeline_genesis_tx`.
- The tx-lane split remains bound to the preferred entrypoints
  `stage_4::run_tx_plan` and `stage_4::run_tx_prepare`.
- The implementation stayed aligned with the existing
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/` tree instead of
  materializing every proposed helper filename listed in `020-02-PLAN.md`.
- Release runs were green across the full `020-02` command set with no test
  failures; the only residual output was a non-blocking `dead_code` warning for
  `Stage4ResolvedPaths::logger_path` in `stage_4.rs`.

## Current 020-03 Evidence

- Release structural guard passed: `test_scenario1_stage_surface`.
- Release transfer-lane structure passed: `test_stage5_source_shape`.
- Release transfer-lane continuity passed: `test_stage5_receive_bridge`.
- Release Stage 6 bridge and checkpoint guards passed: `test_stage6_checkpoint_storage_bridge` and `test_stage6_checkpoint_final_gate`.
- Release downstream continuity passed: `test_stage7_jmt_wallet_scan`, `test_stage8_proof_path`, and `test_scenario1_unified_gate`.
- The Stage 5 split was closed through `stage_5_utils/mod.rs` with a small receive handoff artifact, while the Stage 6 split was closed through `stage_6_utils/mod.rs` with explicit build or publish orchestration.
- Broader `scenario_paths.rs` or `scenario_logging.rs` helpers were not materialized because the remaining duplication did not justify them after the Stage 5/6 split.

## Current 020-04 Evidence

- Release structural guard passed again: `test_scenario1_stage_surface`.
- Release YAML or continuity guards passed: `test_stage4_cfg_paths` and `test_stage4_chain_path`.
- Release acceptance anchors passed: `test_claim_acceptance`, `test_stage5_receive_bridge`, `test_stage6_checkpoint_storage_bridge`, `test_stage6_checkpoint_final_gate`, `test_stage7_jmt_wallet_scan`, `test_stage8_proof_path`, `test_pipeline_genesis_tx`, and `test_scenario1_unified_gate`.
- Additional stale test contracts exposed by the full release suite were aligned to the explicit 12-stage map, including `test_s7_examples`, shared stage4 lookup helpers, and the stage4-era negative or happy-path fixture set.
- The full release suite passed: `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`.
- The release binary passed: `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Audit 2026-03-27

| Metric     | Count |
| ---------- | ----- |
| Gaps found | 0     |
| Resolved   | 0     |
| Escalated  | 0     |

📌 Audit result: Phase 020 remains Nyquist-compliant. All mapped requirements keep automated verification, and no additional test generation was required in this validation pass.

---

## Validation Sign-Off

- [x] All planned tasks have `<automated>` verify or explicit Wave 0
  dependencies.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all MISSING references.
- [x] No watch-mode flags.
- [x] Feedback latency remains phase-local and bounded.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** completed 2026-03-26
