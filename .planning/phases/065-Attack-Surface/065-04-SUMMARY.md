---
phase: 065-Attack-Surface
plan: 065-04
status: complete
completed_at: 2026-07-01
next_plan: 065-05
summary_artifact_for: .planning/phases/065-Attack-Surface/065-04-PLAN.md
---

# 065-04 Summary: Draft And Debug Simulator Evidence Truth

## Outcome

`065-04` is complete.

Draft-only stage-12 runs now emit a private-only checkpoint summary class and
can no longer feed public publication evidence. Public `pub_flow`, `val_flow`,
and `watch_flow` packets now require a real finalized `checkpoint_id_hex`,
while draft runs fail closed before those public artifacts are written. The
default public lane remains secret-free, and the live stage-8 design contract
now documents the new `evidence_class` boundary explicitly.

Broad release validation also surfaced an unrelated but phase-blocking wallet
test-hook race. The receive-persist failpoint hook is now scoped per wallet id
instead of through one global singleton, so parallel release tests fail closed
deterministically.

## Files Changed

- `.planning/phases/065-Attack-Surface/065-04-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_design_orig.yaml`
- `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs`
- `crates/z00z_simulator/tests/scenario_1/test_stage6_checkpoint_final_gate.rs`
- `crates/z00z_simulator/tests/scenario_1/test_wallet_integration.rs`
- `crates/z00z_wallets/src/services/test_wallet_service.rs`
- `crates/z00z_wallets/src/services/wallet_actions_receive.rs`

## Landed Changes

- `crates/z00z_simulator/src/config.rs`
  - added explicit stage-12 evidence classes for draft-private and
    final-public lanes
  - added `Stage6ProofMode::stage12_evidence_class()` and
    `allows_public_checkpoint_evidence()`
- `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs`
  - stage-12 summary now carries `evidence_class`
  - draft-only runs stop before final checkpoint/publication export
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
  - public trace structs now require `checkpoint_id_hex: String`
  - `load_stage12_publication_binding(...)` accepts only the final-public
    evidence class and rejects draft-only synthetic publication binding
- `crates/z00z_simulator/tests/scenario_1/test_stage6_checkpoint_final_gate.rs`
  - added runtime rejection coverage for draft publication evidence
  - renamed the new regression test to `test_draft_publication_rejected` to
    satisfy the five-word identifier rule
- `crates/z00z_simulator/tests/scenario_1/test_wallet_integration.rs`
  - added public-lane regression coverage that proves no plaintext wallet
    secret artifact reaches public output paths and that accepted public traces
    keep real checkpoint ids
  - renamed the new regression test to `test_public_lane_secret_free` to
    satisfy the five-word identifier rule
- `crates/z00z_simulator/src/scenario_1/scenario_design_orig.yaml`
  - updated the live stage-8 summary contract to include `evidence_class`
    and the final-public vs private-only post-conditions
- `.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - aligned wallet release bootstrap coverage with the live feature matrix
    after the release-lane hardening from `065-03`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
  - removed a debug-only scheduler hook from the release bench seed path so
    bootstrap compile checks stay honest
- `crates/z00z_wallets/src/services/wallet_actions_receive.rs`,
  `crates/z00z_wallets/src/services/test_wallet_service.rs`
  - replaced the global receive-persist test hook singleton with a per-wallet
    map to remove the parallel release-test race found by the full workspace
    gate

## Validation

Commands green on the current tree:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture`
- `cargo test --release -p z00z_storage --test test_checkpoint_finalization -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_stage6_checkpoint_final_gate::test_draft_publication_rejected -- --exact --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_wallet_integration::test_public_lane_secret_free -- --exact --nocapture`
- `cargo test --release --quiet`
  - final proof file: `/tmp/phase065_cargo_release.exit`
  - recorded result: `0`
- `git diff --check -- crates/z00z_simulator/tests/scenario_1/test_stage6_checkpoint_final_gate.rs crates/z00z_simulator/tests/scenario_1/test_wallet_integration.rs crates/z00z_simulator/src/scenario_1/scenario_design_orig.yaml`

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted multiple times,
but the current runtime still does not provide a callable review path for this
slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-04-PLAN.md current_task="Draft And Debug Simulator Evidence Truth"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-04-PLAN.md current_task="Draft And Debug Simulator Evidence Truth" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-04-PLAN.md current_task="Draft And Debug Simulator Evidence Truth" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`
- Attempt 4
  - repeated the `gsd --no-session --extension .github -p ...` invocation after
    the compliance rename pass
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `065-04-PLAN.md`, `065-TODO.md`, and the stage-12/publication
    code anchors.
  - Result: found two V2 identifier-length violations in the newly added
    stage-12 test names. Fixed by renaming them to
    `test_draft_publication_rejected` and `test_public_lane_secret_free`.
- Pass 2
  - Re-ran the identifier-length grep on the touched test files and re-read the
    final stage-12 summary/publication route.
  - Result: clean. No remaining `fn test_*` identifiers over five words remain
    in the touched `WS-04` test files, and the public flow still routes through
    `load_stage12_publication_binding(...)`.
- Pass 3
  - Re-read the live design contract plus the public trace schema anchors in
    `runtime_observability.rs`.
  - Result: clean. The live stage-8 design now names `evidence_class`,
    public traces require a real `checkpoint_id_hex`, and draft-only summaries
    fail closed before public publication evidence is built.
- Pass 4
  - Re-checked current-tree validation evidence after the rename fix:
    bootstrap, the two exact renamed regression tests, and the full
    `cargo test --release --quiet` rerun with `/tmp/phase065_cargo_release.exit = 0`.
  - Result: clean.

Passes 2 through 4 were consecutive clean manual review runs after the only
in-scope review finding was fixed.

## Closeout

`065-04` closes `WS-04` by making draft/debug checkpoint truth explicit at the
schema boundary, removing synthetic publication binding from the public lane,
keeping public wallet-secret artifacts absent, aligning the live stage-8 design
doc, and preserving determinism under release validation. The active Phase 065
lane moves to `065-05-PLAN.md`.
