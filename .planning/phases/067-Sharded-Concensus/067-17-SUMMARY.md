---
phase: 067
plan: 067-17
status: complete
completed_at: 2026-07-06
next_plan: 067-18
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-17-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-17 Summary: Structured Evidence Registry

## Outcome

`067-17` is complete.

`VERDICT-LCS-08` now closes on one canonical structured evidence path. The
runtime evidence schema, the simulator fault matrix, and the replay-vote report
surface now all bind the same digest-backed evidence identifiers and artifact
digests for the Gate 14 safety cases instead of relying on filename strings or
log-only wording.

The closeout also removes the last placeholder edge inside the reviewed scope:
`scenario_11` now emits `evidence_registry.json`, the structured fault rows use
real registry ids for equivocation or payload withholding or missing blob or
wrong root or wrong route digest or stale member or split-brain cases, and the
vote-level evidence rows are forced to resolve back to that same canonical
registry. `067-18` is now the next canonical execution lane.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/067-Sharded-Concensus/067-17-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-17-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `crates/z00z_runtime/aggregators/src/evidence.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/service.rs`
- `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs`
- `crates/z00z_runtime/aggregators/tests/test_structured_evidence_registry.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/src/scenario_11/report.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`

## Landed Changes

- Canonical evidence registry path
  - `EvidenceRecord` now covers the required Gate 14 evidence kinds behind one
    digest-bound schema, and the dedicated registry test exercises both the
    positive kind coverage and the malformed-record rejection path.
  - The runtime exports one canonical machine-auditable evidence path instead
    of mixing structured records with string-only placeholders.
- Real structured simulator evidence
  - `scenario_11` now emits `evidence_registry.json` alongside
    `fault_matrix.json`.
  - The fault matrix uses real registry ids and artifact digests for
    equivocation, payload withholding, missing blob, wrong root, wrong route
    digest, stale member, and split-brain rows.
  - The replay-vote report now carries `evidence_id_hex`, `evidence_kind`, and
    `artifact_digests_hex` when structured evidence is emitted.
- Review-driven invariant tightening
  - Manual review found one remaining registry-integrity gap: vote-level
    evidence rows were not yet fail-closed against the emitted registry.
  - The final fix made `scenario_11` reject vote evidence rows that reference a
    non-registry id, omit the evidence kind, or omit artifact digests, and the
    simulator test now asserts that vote evidence ids, kinds, and artifact
    digests exactly match the registry entries.
- Canonical planning/status sync
  - Phase `067` coverage, verdict, state, and roadmap artifacts now record
    `067-17` as complete and move the active lane to `067-18`.

## Validation

Commands green during the `067-17` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_structured_evidence_registry -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_equivocation_evidence -- --nocapture`
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_celestia_local_binding -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `cargo test --release`
- `bash scripts/audit/audit_release_feature_guards.sh`

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --no-session --extension .github --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-17-PLAN.md current_task="067-17-T1" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-17-PLAN.md current_task="067-17-T1" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`.
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-17-PLAN.md current_task="Structured Evidence Registry" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.

Equivalent workspace-first manual review was executed with the
`/code-reviewer` checklist and `/doublecheck` three-layer posture against the
same scope.

- Pass 1
  - Re-read the touched runtime or simulator or test surfaces against
    `067-17-PLAN.md` and the Gate 14 requirement set.
  - Result: found one material invariant gap. Vote-level structured evidence
    rows were not yet forced to resolve back to the canonical evidence
    registry. Fixed in `scenario_11` runtime validation and simulator tests.
- Pass 2
  - Re-ran anchored grep for the Gate 14 evidence kinds
    `equivocation`, `payload_withholding`, `missing_blob`, `wrong_root`,
    `wrong_route_digest`, `stale_member`, and `split_brain` across the touched
    aggregator, simulator, and test surfaces.
  - Result: clean. All required evidence kinds remain present on one canonical
    runtime/test path.
- Pass 3
  - Re-ran anchored grep for the canonical registry fields
    `evidence_registry.json`, `EvidenceRegistryReport`, `evidence_id_hex`,
    `evidence_kind`, and `artifact_digests_hex` across the touched simulator
    report, runtime, and test surfaces.
  - Result: clean. Fault-matrix rows and vote-level evidence rows bind to the
    same registry vocabulary and artifact surface.
- Pass 4
  - Ran `git diff --check` across the touched `067-17` code and planning
    artifacts after the final status sync.
  - Result: clean.
- Pass 5
  - Re-read `067-17-PLAN.md`, `067-17-SUMMARY.md`, `067-COVERAGE.md`,
    `067-verdict.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md` after
    the final status sync.
  - Result: clean.

Passes 4 and 5 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-17` closes `VERDICT-LCS-08` by making the safety evidence path
machine-auditable end to end: runtime schema, simulator registry, fault matrix,
and vote-evidence rows now all converge on the same digest-bound identifiers
and artifact digests.

`067-18` is now the next canonical execution lane.
