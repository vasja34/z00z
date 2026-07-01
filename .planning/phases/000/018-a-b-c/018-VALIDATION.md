---
phase: 018
slug: a-b-c
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-25
---

# Phase 018 - Validation Strategy

📌 Per-phase validation contract reconstructed from executed Phase 018 artifacts
and current green release tests for the Scenario 1 storage-backed continuity,
Stage 7 committed-state wallet scan, and finalized Stage 8 checkpoint lane.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_root_support -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_chain_path -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint_storage_bridge -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage7_jmt_wallet_scan -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint_final_gate -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage8_proof_path -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_scenario1_unified_gate -- --nocapture` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --tests -- --nocapture` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task-local `<automated>` command from
  the owning PLAN file.
- **After every plan wave:** Run the relevant release quick-run gate for the
  changed Scenario 1 seams before handoff.
- **Before `/gsd-verify-work`:** `cargo test --release --features test-fast
  --features wallet_debug_dump -p z00z_simulator --tests -- --nocapture` must
  be green.
- **Max feedback latency:** ~90 seconds.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | --------- | ----------------- | ----------- | ------ |
| 018-01-01 | 01 | 1 | SCN1-03, SCN1-04 | integration | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_root_support -- --nocapture` | ✅ | ✅ green |
| 018-01-02 | 01 | 1 | SCN1-04 | integration | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_chain_path -- --nocapture` | ✅ | ✅ green |
| 018-02-01 | 02 | 2 | SCN1-03 | integration | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage7_jmt_wallet_scan -- --nocapture` | ✅ | ✅ green |
| 018-02-02 | 02 | 2 | SCN1-03, SCN1-04 | integration | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint_storage_bridge -- --nocapture` | ✅ | ✅ green |
| 018-03-01 | 03 | 3 | SCN1-05 | integration | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint_final_gate -- --nocapture` | ✅ | ✅ green |
| 018-03-02 | 03 | 3 | SCN1-04, SCN1-05 | integration | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage8_proof_path -- --nocapture` | ✅ | ✅ green |
| 018-03-03 | 03 | 3 | SCN1-03, SCN1-04, SCN1-05 | phase gate | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_scenario1_unified_gate -- --nocapture` | ✅ | ✅ green |

📌 Status legend: `⬜ pending` · `✅ green` · `❌ red` · `⚠️ flaky`

---

## Wave 0 Requirements

✅ Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

✅ All phase behaviors have automated verification.

---

## Validation Evidence

📌 Reconstructed from these phase artifacts:

- `.planning/phases/018-a-b-c/018-01-PLAN.md`
- `.planning/phases/018-a-b-c/018-02-PLAN.md`
- `.planning/phases/018-a-b-c/018-03-PLAN.md`
- `.planning/phases/018-a-b-c/018-01-SUMMARY.md`
- `.planning/phases/018-a-b-c/018-02-SUMMARY.md`
- `.planning/phases/018-a-b-c/018-03-SUMMARY.md`
- `.planning/phases/018-a-b-c/018-TEST-SPEC.md`

📌 Current audit reran and confirmed:

- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_root_support -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage4_chain_path -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint_storage_bridge -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage7_jmt_wallet_scan -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage6_checkpoint_final_gate -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage8_proof_path -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_scenario1_unified_gate -- --nocapture`

📌 The Phase 018 summaries additionally record a green full-suite release gate:

- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --tests -- --nocapture`

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all MISSING references.
- [x] No watch-mode flags.
- [x] Feedback latency remains short for phase-local checks.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-03-25
