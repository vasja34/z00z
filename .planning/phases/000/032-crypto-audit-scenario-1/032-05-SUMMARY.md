---
phase: 032-crypto-audit-scenario-1
plan: "05"
subsystem: crypto-audit
tags: [scenario1, checkpoint, stage11, fail-closed, simulator, storage]
requires:
  - phase: 032-crypto-audit-scenario-1
    plan: "03"
    provides: storage-owned claim-root and proof continuity consumed by later checkpoint handoff
  - phase: 032-crypto-audit-scenario-1
    plan: "04"
    provides: current-stack public spend-contract verifier and persisted stage-4 proof/auth contract
provides:
  - placeholder-free checkpoint acceptance for accepted Scenario 1 flows
  - stage-7 and stage-11 handoff validation bound to the persisted stage-4 package proof, input refs, and bridge outputs
  - regression coverage that rejects tampered package proof continuity before checkpoint persistence
affects: [032-06, 032-07, scenario1, checkpoint-acceptance]
tech-stack:
  added: []
  patterns:
    - stage-11 apply fails closed when the persisted exec input drifts from the accepted stage-4 tx package or stage-6 bridge outputs
    - replay spent-state checks stay bound to authoritative exec input refs instead of placeholder success paths
    - checkpoint acceptance truth is enforced at the accepted handoff boundary before storage publication
key-files:
  created:
    - .planning/phases/032-crypto-audit-scenario-1/032-05-SUMMARY.md
  modified:
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_11_apply.rs
    - crates/z00z_simulator/tests/test_checkpoint_acceptance.rs
key-decisions:
  - "Accepted checkpoint apply now reuses the persisted stage-4 proof bytes, input refs, and stage-6 bridge outputs as the authoritative handoff contract instead of non-empty proof bytes or placeholder spent-state success."
  - "`CheckpointReplaySpentIndex` fails closed on unknown roots and unknown input ids so replay-style spent-state drift cannot silently downgrade into success."
  - "No additional checkpoint storage schema rewrite was introduced in this wave because the accepted boundary gap was the simulator handoff verifier, not the already statement-bound persistence format."
patterns-established:
  - "`PassProof` and `NoSpent` are no longer part of accepted Scenario 1 checkpoint acceptance paths."
  - "Checkpoint handoff verification must compare persisted exec proof bytes, canonical input refs, and canonical bridge outputs against the accepted stage-4 package before apply."
requirements-completed: [PH32-CHECKPOINT]
duration: continuation session
completed: 2026-04-05
---

# Phase 032 Plan 05: Checkpoint Truthfulness Summary

Scenario 1 checkpoint acceptance no longer succeeds through placeholder proof or spent-state semantics: Stage 11 now replays the accepted stage-4 proof contract into the checkpoint handoff and rejects mismatched proof bytes, input refs, outputs, or replay-style spent-state drift before persistence.

## Performance

- **Duration:** continuation session
- **Started:** carried over from prior Phase 032 execution state
- **Completed:** 2026-04-05T23:59:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Replaced placeholder checkpoint acceptance in [crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs) with `CheckpointPackageProofVerifier` and `CheckpointReplaySpentIndex`, binding checkpoint proof acceptance to the persisted stage-4 package contract and canonical exec-input refs.
- Added reusable canonical handoff helpers in [crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs) and re-exported them through [crates/z00z_simulator/src/scenario_1/stage_6_utils/mod.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/mod.rs) so stage-11 apply compares the same canonical refs and outputs that stage 6 persisted.
- Hardened [crates/z00z_simulator/src/scenario_1/stage_11_apply.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_11_apply.rs) so apply fails closed when the exec tx proof drifts from the stage-4 package, when input refs diverge, when bridge outputs diverge, or when the stage-4 public spend contract itself no longer verifies.
- Extended [crates/z00z_simulator/tests/test_checkpoint_acceptance.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_checkpoint_acceptance.rs) with stage-11 package-proof tamper coverage so checkpoint artifacts are not emitted after proof-contract drift.

## Task Commits

No task commit was created in this execution pass.

The repository-required `/z00z-git-versioning` flow is version-tag oriented and stages the full dirty worktree. Because this phase is still executing inside a shared worktree with unrelated pending changes, this plan was closed summary-first and the next explicit version-managed sync remains deferred to a deliberate user-approved release point.

## Files Created/Modified

- [crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs) - Removed accepted-path placeholder proof and spent-state logic in favor of canonical package and replay checks.
- [crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs) - Added canonical input-ref and exec-output builders shared by checkpoint creation and apply.
- [crates/z00z_simulator/src/scenario_1/stage_6_utils/mod.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/mod.rs) - Re-exported the new canonical handoff helpers and checkpoint verification types.
- [crates/z00z_simulator/src/scenario_1/stage_11_apply.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_11_apply.rs) - Added fail-closed stage-7 handoff verification before checkpoint draft creation.
- [crates/z00z_simulator/tests/test_checkpoint_acceptance.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_checkpoint_acceptance.rs) - Added regression coverage for tampered stage-4 package proof continuity.

## Decisions Made

- The accepted checkpoint truth boundary is the stage-4 package plus stage-6 canonical bridge outputs, not a later storage reload guess based on non-empty proof bytes.
- Stage-11 apply must reject drift before checkpoint draft creation instead of relying on downstream artifact presence or storage loaders to discover the mismatch later.
- Existing checkpoint persistence artifacts remained untouched because this wave closed the accepted-path truth gap without requiring a new statement schema.

## Review Passes

- **Pass 1:** Prompt-equivalent review against [032-05-PLAN.md](/home/vadim/Projects/z00z/.planning/phases/032-crypto-audit-scenario-1/032-05-PLAN.md) and the Wave 4 threat register confirmed that accepted flows no longer reach `PassProof` or `NoSpent`, and that the new stage-11 handoff compares proof bytes, input refs, and outputs against authoritative upstream artifacts. Clean.
- **Pass 2:** Crypto/security review under the `crypto-architect` and `security-audit` criteria found no remaining accepted-path bypass after confirming stage-4 public-contract revalidation, fail-closed replay spent-index behavior, and rejection of malformed package-proof continuity. Clean.
- **Pass 3:** Doublecheck-style evidence review against the executed release tests and the broader simulator release suite stayed clean. Clean.

Slash prompt execution is not directly available in this agent runtime, so the `GSD-Review-Tasks-Execution` rubric was applied manually for three passes using the same plan context and review criteria. The last two review passes were consecutive clean runs.

## Deviations from Plan

### Auto-fixed Issues

None.

### Execution Notes

- The plan named checkpoint storage files as possible touch points, but the accepted-path defect was fully closed at the simulator handoff boundary. Existing checkpoint persistence artifacts were already statement-bound enough that widening this wave into redundant storage edits was unnecessary.

## Issues Encountered

- Verification output from the interactive terminal truncated during the broader suite run, so the closeout relies on the successful `exit code 0` release run plus targeted release reruns instead of pretending the full textual log remained intact.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump test_checkpoint_acceptance -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`

## User Setup Required

None.

## Next Phase Readiness

- Wave 6 can now describe checkpoint truthfulness honestly because accepted Scenario 1 checkpoint apply is tied to the persisted stage-4 contract instead of placeholder proof or spent-state success semantics.
- Wave 7 can reference Stage 11 checkpoint acceptance as current-stack honest behavior without implying a broader proof backend than the code actually delivers.

## Threat Flags

None.

## Self-Check: PASSED

- Verified [.planning/phases/032-crypto-audit-scenario-1/032-05-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/032-crypto-audit-scenario-1/032-05-SUMMARY.md) exists.
- Verified [crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs) exists.
- Verified [crates/z00z_simulator/src/scenario_1/stage_11_apply.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_11_apply.rs) exists.
- Verified [crates/z00z_simulator/tests/test_checkpoint_acceptance.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_checkpoint_acceptance.rs) exists.

---
*Phase: 032-crypto-audit-scenario-1*
*Completed: 2026-04-05*
