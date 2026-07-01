---
phase: 065-Attack-Surface
plan: 065-02
status: complete
completed_at: 2026-07-01
next_plan: 065-03
summary_artifact_for: .planning/phases/065-Attack-Surface/065-02-PLAN.md
---

# 065-02 Summary: Canonical Checkpoint Persistence

## Outcome

`065-02` is complete.

The live tree now has one canonical final-checkpoint birth path. Public
checkpoint persistence no longer exposes `save_artifact()` as a peer API beside
`seal_artifact()`. Canonical final artifacts, links, and audits are admitted
only through the seal path, while the post-tx publication view uses an explicit
noncanonical export surface with a persisted final-lane marker that blocks
canonical reloads.

Write-time checkpoint-link validation is now fail-closed. `save_link()` and the
seal path both require the snapshot row, exec-input row, statement binding, and
root coherence to exist and match before the link persists. Compatibility or
draft proof bytes are no longer named or shaped like the canonical final
artifact proof contract: the draft-side payload is now explicitly
`attest_payload_bytes`, while the final sealed artifact remains the only owner
of `cp_proof()`.

## Files Changed

- `.planning/phases/065-Attack-Surface/065-02-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `crates/z00z_simulator/src/scenario_1/stage_12/finalize_flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4/storage_view.rs`
- `crates/z00z_simulator/src/scenario_1/stage_9/exec_input_builder.rs`
- `crates/z00z_simulator/src/scenario_1/support/checkpoint_shared_cases.rs`
- `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs`
- `crates/z00z_simulator/tests/scenario_1/test_stage6_checkpoint_storage_bridge.rs`
- `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`
- `crates/z00z_storage/src/checkpoint/store.rs`
- `crates/z00z_storage/src/checkpoint/store_fs.rs`
- `crates/z00z_storage/tests/test_checkpoint_draft_final.rs`
- `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
- `crates/z00z_storage/tests/test_checkpoint_link_injective.rs`
- `crates/z00z_storage/tests/test_checkpoint_store.rs`

## Landed Changes

- `crates/z00z_storage/src/checkpoint/store.rs`
  - removed the public raw checkpoint `save_artifact()` lane from the
    `CheckpointStore` contract
  - added internal persisted-artifact and persisted-audit helpers so canonical
    and noncanonical flows use the same low-level bytes but not the same public
    authority lane
  - added explicit noncanonical export and reload APIs:
    `export_noncanonical_final_bundle(...)`,
    `load_noncanonical_artifact(...)`,
    `load_noncanonical_link(...)`, and
    `load_noncanonical_audit(...)`
  - made `save_link()` validate snapshot existence, exec-row existence, link
    statement binding, replay membership, and root coherence before write
  - made `seal_artifact()` persist the final artifact only after the same
    evidence checks succeed and then mark the lane as canonical
- `crates/z00z_storage/src/checkpoint/store_fs.rs`
  - added `final_lane.marker` handling with explicit `canonical_seal` and
    `noncanonical_export` ownership
  - canonical loads now reject the noncanonical export lane, and noncanonical
    reload APIs require the noncanonical marker
- `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`
  - renamed the draft-side proof field from `cp_proof` to
    `attest_payload_bytes`
  - kept the final sealed artifact proof contract separate so downstream code
    cannot confuse draft compatibility bytes with the canonical theorem-bearing
    payload
- `crates/z00z_simulator/src/scenario_1/stage_12/finalize_flow.rs`
  - stage 12 now builds `build_attest_proof(...)` and seals only through
    `seal_artifact(...)`
- `crates/z00z_simulator/src/scenario_1/stage_4/storage_view.rs`
  - post-tx export now uses `export_noncanonical_final_bundle(...)`
  - the summary surface explicitly records `"final_lane": "noncanonical_export"`
- checkpoint regression coverage
  - added negative write-time tests for missing exec rows, missing snapshot
    rows, and root drift
  - added rejection tests proving noncanonical post-tx exports cannot be loaded
    through canonical final-artifact APIs
  - added naming-split tests proving draft proof bytes stay separate from the
    final sealed artifact `cp_proof()` contract
  - added draft-only simulator guard coverage proving no final-lane marker is
    published before real finalization
- fixture refresh
  - bumped the stage-12 shared cache keys to `*_v3` so the current tree uses
    fresh fixtures aligned with the new lane-marker contract

## Validation

Commands green on the current tree:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_storage --test test_checkpoint_store -- --nocapture`
- `cargo test --release -p z00z_storage --test test_checkpoint_finalization -- --nocapture`
- `cargo test --release -p z00z_storage --test test_checkpoint_link_injective -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_checkpoint_acceptance -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_stage6_checkpoint_storage_bridge -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 small_medium_stay_deterministic -- --nocapture`
- `cargo test --release --quiet`
- `git diff --check -- crates/z00z_storage/src/checkpoint/store.rs crates/z00z_storage/src/checkpoint/store_fs.rs crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs crates/z00z_simulator/src/scenario_1/stage_12/finalize_flow.rs crates/z00z_simulator/src/scenario_1/stage_4/storage_view.rs crates/z00z_simulator/src/scenario_1/stage_9/exec_input_builder.rs crates/z00z_storage/tests/test_checkpoint_store.rs crates/z00z_storage/tests/test_checkpoint_finalization.rs crates/z00z_storage/tests/test_checkpoint_link_injective.rs crates/z00z_storage/tests/test_checkpoint_draft_final.rs crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs crates/z00z_simulator/tests/scenario_1/test_stage6_checkpoint_storage_bridge.rs crates/z00z_simulator/src/scenario_1/support/checkpoint_shared_cases.rs .planning/phases/065-Attack-Surface/065-02-SUMMARY.md .planning/STATE.md .planning/ROADMAP.md`
- `rg -n "fn save_artifact\\(|export_noncanonical_final_bundle|load_noncanonical_artifact|load_noncanonical_link|load_noncanonical_audit|check_link_evidence|persist_final_lane|reject_noncanonical_final_lane|require_noncanonical_final_lane|reject_canonical_final_lane" crates/z00z_storage/src/checkpoint`
- `rg -n "build_attest_proof|seal_artifact\\(|export_noncanonical_final_bundle|final_lane.marker|noncanonical_export|load_noncanonical_" crates/z00z_simulator/src/scenario_1 crates/z00z_simulator/tests/scenario_1`

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted multiple times,
but the current runtime still does not provide a reliable callable review path
for this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-02-PLAN.md current_task="Canonical Checkpoint Persistence"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 30s gsd --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-02-PLAN.md current_task="Canonical Checkpoint Persistence" --yolo'`
  - Result: exited with code `1` and produced no review output
- Attempt 3
  - `timeout 30s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-02-PLAN.md current_task="Canonical Checkpoint Persistence" --yolo'`
  - Result: exited with code `1` and produced no review output
- Attempt 4
  - `timeout 30s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-02-PLAN.md current_task="Canonical Checkpoint Persistence" --yolo'`
  - Result: exited with code `1` and produced no review output

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `065-02-PLAN.md`, `065-TODO.md`, and the checkpoint store/store-fs
    authority seams.
  - Result: clean. No public checkpoint `save_artifact()` lane remains, the
    canonical path is `seal_artifact()`, and the only remaining raw lane is the
    explicitly named noncanonical export surface.
- Pass 2
  - Re-read the stage-12 producer path, the stage-4 post-tx export path, and
    the post-tx consumer tests.
  - Result: clean. Stage 12 now uses `build_attest_proof(...)` plus
    `seal_artifact(...)`, while post-tx export uses
    `export_noncanonical_final_bundle(...)` and can only reload through the
    explicit noncanonical APIs.
- Pass 3
  - Re-checked negative coverage and anti-placeholder evidence through source
    grep and the targeted checkpoint tests.
  - Result: clean. The tree now has explicit rejection coverage for missing
    exec rows, missing snapshot rows, root drift, post-tx canonical-load
    misuse, conflicting exec reuse, and draft-vs-final proof naming drift.
- Pass 4
  - Re-checked diff hygiene and the broad release concern after an earlier
    `cargo test --release --quiet` run had surfaced
    `small_medium_stay_deterministic` once during a prior pass.
  - Result: clean. The isolated rerun of
    `cargo test --release -p z00z_simulator --test scenario_1 small_medium_stay_deterministic -- --nocapture`
    passed, and the final broad `cargo test --release --quiet` rerun also
    completed green on the current tree.

Passes 2 and 3 were consecutive clean manual review runs for the actual
checkpoint-authority closure slice before the final broad release rerun.

## Closeout

`065-02` closes `WS-02` by making the seal path the only canonical checkpoint
authority, quarantining post-tx publication as an explicit noncanonical export
lane, and separating draft attestation payload bytes from the final artifact
proof contract. The active Phase 065 lane moves to `065-03-PLAN.md`.
