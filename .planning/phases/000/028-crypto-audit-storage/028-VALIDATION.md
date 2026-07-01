---
phase: 028
slug: crypto-audit-storage
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-30
---

# Phase 028 — Validation Strategy

> 📌 Reconstructed from executed Phase 028 plans, summaries, test contract, verification logs, and existing Rust test seams because no prior validation file existed in this phase directory.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust `cargo test` plus release-style simulator and workspace commands |
| **Config file** | `Cargo.toml` workspace manifest plus crate manifests under `crates/*/Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | release-mode multi-command Rust suite |

## Sampling Rate

- 📌 After every task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` plus the task-specific targeted storage or simulator command.
- 📌 After every plan wave: run `cargo test --release --features test-fast --features wallet_debug_dump`.
- 📌 Before `/gsd-verify-work`: keep the full workspace release gate green and rerun the authoritative `scenario_1` release command when the wave touches simulator acceptance.
- 📌 Max feedback latency: bounded to one bootstrap cycle plus one targeted release test or one wave-level full gate.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | --------- | ----------------- | ----------- | ------ |
| 028-01-01 | 01 | 1 | PH28-CHK-PROOF | integration | `cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture` | ✅ | ✅ green |
| 028-01-02 | 01 | 1 | PH28-TRUST-HOOK | integration | `cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture` | ✅ | ✅ green |
| 028-02-01 | 02 | 2 | PH28-EXEC-PROOF | integration | `cargo test -p z00z_storage --release --test test_checkpoint_replay_inputs -- --nocapture` | ✅ | ✅ green |
| 028-02-02 | 02 | 2 | PH28-TRUST-HOOK | persistence | `cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture` | ✅ | ✅ green |
| 028-03-01 | 03 | 3 | PH28-ROOT-BIND | integration | `cargo test -p z00z_storage --release --test test_checkpoint_root_binding -- --nocapture` | ✅ | ✅ green |
| 028-03-02 | 03 | 3 | PH28-ROOT-BIND | consumer-compat | `cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture` | ✅ | ✅ green |
| 028-04-01 | 04 | 3 | PH28-ID-BIND | integration | `cargo test -p z00z_storage --release --test test_checkpoint_ids -- --nocapture` | ✅ | ✅ green |
| 028-04-02 | 04 | 3 | PH28-ID-BIND | persistence | `cargo test -p z00z_storage --release --test test_checkpoint_link_injective -- --nocapture` | ✅ | ✅ green |
| 028-05-01 | 05 | 4 | PH28-NULLIFIER | simulator-release | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage3_nullifier_store -- --nocapture` | ✅ | ✅ green |
| 028-05-02 | 05 | 4 | PH28-TRUST-HOOK, PH28-NULLIFIER | closeout-gate | `cargo test -p z00z_storage --release --test test_redb_mutation -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture && cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump` | ✅ | ✅ green |

📌 Status legend: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky

## Requirement Coverage Summary

| Requirement | Status | Evidence |
| ----------- | ------ | -------- |
| PH28-CHK-PROOF | COVERED | 📌 `test_checkpoint_finalization`, `test_checkpoint_draft_final`, and `test_checkpoint_store_api` prove truthful opaque semantics, explicit legacy handling, and honest store-facing sealing behavior. |
| PH28-EXEC-PROOF | COVERED | 📌 `test_checkpoint_replay_inputs`, `test_checkpoint_draft_build`, and the green simulator claim pipeline prove canonical exec artifacts preserve real upstream `tx_proof` bytes. |
| PH28-TRUST-HOOK | COVERED | 📌 Checkpoint build and store trust-boundary behavior is exercised by `test_checkpoint_store_api`, `test_redb_rehydrate`, `test_redb_mutation`, the simulator closeout gates, and the green release `scenario_1` run. |
| PH28-ROOT-BIND | COVERED | 📌 `test_checkpoint_root_binding` and `test_claim_source_proof` prove explicit semantic-root/backend-root binding plus fail-closed consumer compatibility. |
| PH28-ID-BIND | COVERED | 📌 `test_checkpoint_ids`, `test_checkpoint_link_injective`, and RedB reload validation prove typed artifact IDs, canonical link binding, and mixed-era rejection. |
| PH28-NULLIFIER | COVERED | 📌 `test_stage3_nullifier_store`, `test_claim_tx_pipeline`, `test_redb_rehydrate`, and `test_redb_mutation` prove canonical binary nullifier replay state, migration parity, and hardened default-path behavior. |

📌 Gap analysis result: all declared Phase 028 requirements are `COVERED`; no `PARTIAL` or `MISSING` automated requirement references were found.

## Wave 0 Requirements

📌 Existing infrastructure covers all phase requirements. No Wave 0 scaffolding is needed.

## Manual-Only Verifications

All phase behaviors have automated verification.

## Reconstruction Notes

- `028-01` through `028-05` each have summary-backed closure artifacts.
- `028-TEST-SPEC.md` is verification-backed and defines the canonical Phase 028 test seams and release-style commands.
- `028-VERIFICATION.md` records the executed green verification bundle and requirement-level evidence.
- The strongest automated seams for this phase are:
  - `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
  - `crates/z00z_storage/tests/test_checkpoint_draft_final.rs`
  - `crates/z00z_storage/tests/test_checkpoint_draft_build.rs`
  - `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
  - `crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs`
  - `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
  - `crates/z00z_storage/tests/test_checkpoint_ids.rs`
  - `crates/z00z_storage/tests/test_checkpoint_link_injective.rs`
  - `crates/z00z_storage/tests/test_redb_rehydrate.rs`
  - `crates/z00z_storage/tests/test_redb_mutation.rs`
  - `crates/z00z_storage/tests/test_claim_source_proof.rs`
  - `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs`
  - `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`
  - `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`
  - `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`

📌 The phase-local structural scans were also validated:

- the legacy text-key nullifier detector returned zero matches on canonical storage and simulator surfaces;
- the old placeholder replay-emitter oracle now also returns zero matches, which is the correct post-`028-02` invariant because that emitter was intentionally removed from production paths.

## Gap Audit

| Requirement | Coverage | Evidence |
| ----------- | -------- | -------- |
| PH28-CHK-PROOF | COVERED | `028-VERIFICATION.md` plus checkpoint finalization, draft-final, and store API regressions |
| PH28-EXEC-PROOF | COVERED | `028-VERIFICATION.md` plus replay-input, draft-build, and claim pipeline regressions |
| PH28-TRUST-HOOK | COVERED | `028-VERIFICATION.md` plus store API, rehydrate, redb mutation, and release-style simulator gates |
| PH28-ROOT-BIND | COVERED | `028-VERIFICATION.md` plus root-binding and claim-source proof regressions |
| PH28-ID-BIND | COVERED | `028-VERIFICATION.md` plus checkpoint IDs, link injectivity, and reload regressions |
| PH28-NULLIFIER | COVERED | `028-VERIFICATION.md` plus nullifier-store, claim-pipeline, rehydrate, mutation, and structural absence scan evidence |

📌 No Nyquist gaps were found during reconstruction, so no additional test files were required.

## Validation Sign-Off

- [x] All tasks have `<automated>` verify coverage or existing infrastructure coverage.
- [x] Sampling continuity is preserved; no three consecutive tasks rely on missing automated verification.
- [x] Wave 0 is not required because existing infrastructure covers all phase requirements.
- [x] No watch-mode flags are part of the phase validation contract.
- [x] Feedback latency remains bounded to bootstrap plus targeted release validation loops.
- [x] `nyquist_compliant: true` is set in frontmatter.

📌 Approval: approved 2026-03-30
