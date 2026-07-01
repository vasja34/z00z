---
phase: 021
slug: refactor-continue
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 021 - Validation Strategy

> 📌 Reconstructed from executed Phase 021 plan and summary artifacts because no prior validation file existed in this phase directory.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust release-mode integration tests via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | release-mode, workspace-dependent |

## Sampling Rate

- 📌 After every task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` plus the task-specific targeted release test.
- 📌 After every plan wave: run `cargo test --release --features test-fast --features wallet_debug_dump` or the strongest phase-local release subset when unrelated protected-vendor doctests block the workspace suite.
- 📌 Before `/gsd-verify-work`: run the targeted release-stage closure stack for Scenario 1.
- 📌 Max feedback latency: bounded to one targeted release validation cycle on the maintainer machine.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | --------- | ----------------- | ----------- | ------ |
| 021-01-01 | 01 | 1 | SCN1-06 | source-shape | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture` | ✅ existing | ✅ green |
| 021-02-01 | 02 | 2 | SCN1-06 | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_integration -- --nocapture` | ✅ existing | ✅ green |
| 021-02-02 | 02 | 2 | SCN1-06 | regression | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline -- --nocapture` | ✅ existing | ✅ green |
| 021-03-01 | 03 | 2 | SCN1-06 | split-surface | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_split -- --nocapture` | ✅ existing | ✅ green |
| 021-03-02 | 03 | 2 | SCN1-06 | guard | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_digest -- --nocapture` | ✅ existing | ✅ green |
| 021-04-01 | 04 | 2 | SCN1-06 | bridge | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_storage_bridge -- --nocapture` | ✅ existing | ✅ green |
| 021-04-02 | 04 | 2 | SCN1-06 | downstream | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage8_proof_path -- --nocapture` | ✅ existing | ✅ green |
| 021-05-01 | 05 | 3 | SCN1-07, SCN1-08 | contract | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture` | ✅ existing | ✅ green |
| 021-05-02 | 05 | 3 | SCN1-07, SCN1-08 | end-to-end | `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump` | ✅ existing | ✅ green |

📌 Status legend: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky

## Requirement Coverage Summary

| Requirement | Status | Evidence |
| ----------- | ------ | -------- |
| SCN1-06 | COVERED | 📌 Wave 01 through Wave 04 summaries record canonical stage ownership closure, and targeted release tests cover claim, tx, transfer, bundle, and checkpoint boundaries. |
| SCN1-07 | COVERED | 📌 Wave 05 hardens `scenario_design.yaml`, shared design parsing, and runner-side contract validation; `test_scenario1_stage_surface` and the release `scenario_1` binary prove descriptive YAML fidelity. |
| SCN1-08 | COVERED | 📌 Wave 05 plus source-shape guards prove final `rust_entry` ownership, truthful stage-id metadata, and the canonical 12-stage runner surface. |

📌 Gap analysis result: no `PARTIAL` or `MISSING` requirement gaps were found, so no Nyquist auditor run was required for this reconstruction pass.

## Wave 0 Requirements

📌 Existing infrastructure covers all phase requirements.

## Manual-Only Verifications

📌 All phase behaviors have automated verification.

## External Blockers Outside Phase Scope

- ⚠️ The workspace-wide release suite still reaches unrelated protected-vendor doctest failures under `crates/z00z_crypto/tari/crypto/`.
- ⚠️ That blocker does not remove automated verification coverage for `SCN1-06`, `SCN1-07`, or `SCN1-08`, because the phase-local release gates and Scenario 1 closure commands are already green.

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or existing infrastructure coverage.
- [x] Sampling continuity is preserved; no three consecutive tasks rely on missing automated verification.
- [x] Wave 0 is not required because existing infrastructure covers all phase requirements.
- [x] No watch-mode flags are part of the phase validation contract.
- [x] Feedback latency remains bounded to targeted release validation loops.
- [x] `nyquist_compliant: true` is set in frontmatter.

📌 Approval: approved 2026-03-27
