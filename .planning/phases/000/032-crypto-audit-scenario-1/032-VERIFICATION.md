---
phase: 032-crypto-audit-scenario-1
artifact: verification
status: review-backed
updated: 2026-04-06
---

# Phase 032 Verification

## 🎯 Scope

📌 This artifact records the executed verification evidence for the current
Phase 032 test-spec contract.

📌 It is intentionally narrower than a full phase closeout.

📌 The broader original `PH32-SPEND` and `PH32-CLAIM-TRUST` requirements
remain open in `.planning/REQUIREMENTS.md`, and this file must not be used to
claim that the phase closed either the missing nullifier-semantics portion of
`PH32-SPEND` or the persisted storage-backed continuity portion of
`PH32-CLAIM-TRUST`.

## ✅ Required Verification Order

The required sign-off order from `032-HONEST-CLOSEOUT.md` was preserved.

1. Bootstrap fail-fast gate ran first.
2. Required release-style targeted tests ran next.
3. Broader release-style reruns were recorded.
4. The manual equivalent of `/GSD-Review-Tasks-Execution` was executed until
   two consecutive clean runs were reached.

## 🔔 Executed Command Evidence

### Bootstrap And Required Release-Style Tests

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - result: PASS
  - log: `.planning/phases/032-crypto-audit-scenario-1/.logs/032-bootstrap.log`
- `cargo test -p z00z_wallets --release --features test-fast --test test_spend_witness_gate -- --nocapture`
  - result: PASS
  - log: `.planning/phases/032-crypto-audit-scenario-1/.logs/032-wallet-spend-witness.log`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`
  - result: PASS
  - log: `.planning/phases/032-crypto-audit-scenario-1/.logs/032-simulator-spend-gate.log`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture`
  - result: PASS
  - log: `.planning/phases/032-crypto-audit-scenario-1/.logs/032-simulator-checkpoint.log`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage2_secret_artifacts -- --nocapture`
  - result: PASS
  - log: `.planning/phases/032-crypto-audit-scenario-1/.logs/032-simulator-stage2-secret.log`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_transport_rng_boundaries -- --nocapture`
  - result: PASS
  - log: `.planning/phases/032-crypto-audit-scenario-1/.logs/032-simulator-transport-rng.log`

### Broader Release-Style Reruns

- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
  - historical result set: mixed / stale
  - logs:
    - `.planning/phases/032-crypto-audit-scenario-1/.logs/032-simulator-release.log`
    - `.planning/phases/032-crypto-audit-scenario-1/.logs/032-simulator-release-rerun.log`
- `cargo test --release --features test-fast --features wallet_debug_dump`
  - historical manifest-backed result: FAIL
  - log: `.planning/phases/032-crypto-audit-scenario-1/.logs/032-workspace-release.log`
  - note: later checked-in manifests still record `RESULT[18]=FAIL`, so this command is not treated as clean closeout evidence in the current review state

### Fresh 2026-04-05 Test-Spec Rerun

- Canonical command matrix from `032-TEST-SPEC.md`
  - targeted result set: PASS through `RESULT[17]`
  - manifest: `.planning/phases/032-crypto-audit-scenario-1/.logs/032-test-spec-rerun-20260405T172235Z/manifest.txt`
- `cargo test --release --features test-fast --features wallet_debug_dump`
  - manifest-backed result in the later rerun set: FAIL
  - evidence: `.planning/phases/032-crypto-audit-scenario-1/.logs/032-test-spec-rerun-20260405T182809Z/manifest.current.txt`

### Targeted Follow-Up Validation During Review Loop

These runs were not substitutes for the required release evidence above. They
were targeted checks used to validate fixes uncovered by the review loop.

- `cargo test -p z00z_simulator --release --features test-fast --lib witness_gate_ok -- --nocapture`
  - result: PASS
- `cargo test -p z00z_simulator --release --features test-fast stage4_witness_gate_ok -- --nocapture`
  - result: PASS
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture`
  - result: PASS
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint -- --nocapture`
  - result: PASS
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture`
  - result: PASS
- `cargo test --release --features test-fast --features wallet_debug_dump`
  - current-session result: BLOCKED
  - blocker: host filesystem full (`No space left on device`); not treated as clean PASS evidence

## 🔎 Review-Loop Record

The required manual equivalent of `/GSD-Review-Tasks-Execution` was executed in
independent passes.

### Pass 1

- result: ISSUES
- finding: `witness_gate_ok` passed through an unrealistic positive fixture that
  manually mutated output fields instead of rebuilding a self-consistent output
  bundle.
- fix:
  - added a test-only wrapper around the canonical output builder seam
  - added `mk_balanced_out_from_input(...)`
  - updated `witness_gate_ok` to rebuild a canonical output from the input
    output's blinding instead of mutating `leaf` fields directly

### Pass 2

- result: ISSUES
- findings:
  - verification manifests lacked explicit `RESULT:` lines for the failing
    pre-fix rerun and the broader workspace rerun
  - `032-TEST-SPEC.md` had future-dated metadata and wording that could
    overstate phase closure
- fix:
  - added explicit `RESULT: FAIL` and `RESULT: PASS` entries to the manifests
  - corrected the test-spec date
  - added explicit wording that the test-spec does not close `PH32-SPEND`

### Pass 3

- result: ISSUES
- findings:
  - simulator-local naming, tests, and checked-in log artifacts still used
    `spend_public_contract` wording for the narrower S4-10 witness-gate step
  - adjacent simulator-local error strings were too imprecise next to the
    witness-gate/current-stack verifier split
- fix:
  - renamed the simulator-local helper to `verify_spend_witness_gate`
  - updated S4-10 runtime log event/detail wording to `spend_witness_gate`
  - updated `test_stage4_gates.rs` to assert the witness-gate wording
  - updated the checked-in `src/scenario_1/outputs/logs/logger.json` artifact
  - narrowed adjacent error strings to `current-stack tx public spend ...`

### Clean Run 1

- result: PASS
- scope: simulator-local truth surface for S4-10 and adjacent current-stack
  verifier wording

### Clean Run 2

- result: PASS
- scope: same as Clean Run 1
- stop condition: satisfied

### Fresh Rerun Pass 1

- result: ISSUES
- finding: `032-TEST-SPEC.md` still claimed that `032-VERIFICATION.md` did not
  exist, which was no longer true after the earlier verification closeout.
- fix:
  - updated the test-spec workflow-status wording to acknowledge that
    `032-VERIFICATION.md` now exists
  - kept the honest caveat that the verification artifact is narrower than full
    phase closure and does not close `PH32-SPEND`

### Fresh Clean Run 1

- result: PASS
- scope: semantic freeze surfaces, request/card validation boundaries, and the
  current-stack spend witness/public verifier seam

### Fresh Clean Run 2

- result: PASS
- scope: checkpoint acceptance, claim-source proof continuity, and transport RNG
  boundary coverage
- stop condition: satisfied

### Follow-Up Review Correction

- result: ISSUES
- findings:
  - stale verification artifacts still claimed broad-suite PASS even though later checked-in manifests recorded `RESULT[18]=FAIL`
  - the latest current-session full-suite rerun could not complete because the host filesystem had no free space left
- fix:
  - narrowed this artifact to targeted clean evidence plus explicit broad-suite blocker language
  - removed any remaining claim that the broad workspace release suite is currently clean closeout evidence

## ⚙️ Files Adjusted During Verification Repairs

- `crates/z00z_simulator/src/scenario_1/stage_4_utils/output_construction.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_test_support.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
- `crates/z00z_simulator/tests/test_stage4_gates.rs`
- `crates/z00z_simulator/src/scenario_1/outputs/logs/logger.json`
- `.planning/phases/032-crypto-audit-scenario-1/032-TEST-SPEC.md`
- `.planning/phases/032-crypto-audit-scenario-1/.logs/032-test-spec-rerun-20260405T172235Z/manifest.txt`
- `.planning/phases/032-crypto-audit-scenario-1/.logs/032-verification-manifest-20260405T152950Z.txt`
- `.planning/phases/032-crypto-audit-scenario-1/.logs/032-verification-rerun-manifest-20260405T154419Z.txt`
- `docs/code-review/2026-04-05-leaf-ad-asset-id-spend-claim-review.md`

## ⛔ Open Requirement Caveat

This verification artifact does not change the honest requirement status.

- `PH32-SPEND` remains open.
- The current tree now proves a narrower, review-backed current-stack boundary
  for the accepted spend witness gate and adjacent verifier paths.
- The current tree does not yet prove the broader original spend requirement
  with nullifier semantics.

## ⭐ Verification Outcome

Phase 032 test-spec execution is now backed by:

- bootstrap-first evidence
- required release-style targeted tests
- broader rerun evidence with explicit pass/fail manifests, including unresolved broad-suite failure state
- targeted follow-up checks for the repaired simulator surfaces
- a completed review loop with two consecutive targeted clean runs

That is sufficient to treat the Phase 032 verification contract as
review-backed.

It is not sufficient to claim full closure of the broader original
`PH32-SPEND` requirement, full closure of `PH32-CLAIM-TRUST`, or a clean broad workspace release suite.
