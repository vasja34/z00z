---
phase: 057-HJMT-multi-aggregator
plan: 057-05
status: complete
completed_at: 2026-06-14
next_plan: 057-06
requirements-completed:
  - 057-G9
  - 057-G10
  - 057-G11
summary_artifact_for: .planning/phases/057-HJMT-multi-aggregator/057-05-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 057-05 Summary: Validator, Watcher, Scope-Continuity, And Scenario Sync

## Completed Scope

`057-05` is complete for the live Phase 057 downstream-consumer and
scope-continuity slice.

The repository now binds validator acceptance, watcher evidence, and simulator
publication traces to one canonical publication digest story instead of letting
each consumer derive a local interpretation. `PublicationBinding` is now the
shared contract for the published batch, checkpoint id, route-table digest,
and `pub_in` digest; validators construct and verify that binding once,
watchers consume the same binding through verdicts and checked snapshots, and
simulator traces export the same publication and binding digests through
`val_flow.json` and `watch_flow.json`.

This closeout also fixes the last live scenario drift inside the `057-05`
scope. The shared runtime-observability packet previously assumed that Stage 8
publication was always already accepted, which broke the honest shared
`draft_only` publication lane used by the checked-in live scenario. The final
implementation now reuses the saved Stage 7 draft to build one incomplete
publication-binding story for `draft_only` runs, keeps `checkpoint_id_hex`
empty when publication is not sealed yet, and proves that validator and
watcher traces still stay aligned on the same digest contract.

Future-only wording in the referenced HJMT design packet stayed live scope
authority for this slice, but the implementation remained on the existing
aggregator, validator, watcher, simulator, and storage seams rather than
creating a second publication registry, watcher-local digest, or detached
scenario narrative.

## Files Changed

- `.planning/phases/057-HJMT-multi-aggregator/057-05-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/service.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/validators/src/checkpoint.rs`
- `crates/z00z_runtime/validators/src/engine.rs`
- `crates/z00z_runtime/validators/src/verdict.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/watchers/src/evidence_export.rs`
- `crates/z00z_runtime/watchers/src/engine.rs`
- `crates/z00z_runtime/watchers/src/lib.rs`
- `crates/z00z_runtime/watchers/src/publication.rs`
- `crates/z00z_runtime/watchers/src/status.rs`
- `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_simulator/Cargo.toml`
- `crates/z00z_simulator/src/config.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
- `crates/z00z_simulator/src/test_support/stage13_shared_cases.rs`
- `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`

## Boundary Kept Intact

- Validators, watchers, and simulator traces now share one
  `PublicationBinding`; no second digest authority path was introduced.
- First-seen scope birth remains storage-owned. Publication continuity is
  proven from committed shard-local results and saved drafts or artifacts, not
  from a new public registry or synthetic leaf fixture lane.
- `draft_only` publication evidence reuses the saved Stage 7 draft and marks
  the result honestly as `incomplete`; it does not pretend a sealed checkpoint
  exists.
- `val_flow.json` and `watch_flow.json` remain evidentiary only. They resolve
  back to the inherited Phase 056 lineage packet instead of replacing route,
  journal, scope, or process truth.
- `057-06` still owns the final gate matrix, bench closure, and full
  planning-state phase closeout. This summary does not claim full Phase 057
  completion.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.
The review used the default `code-reviewer` lens, the
`alert-concept-drift` lens against `057-TODO.md` and `057-CONTEXT.md`, and a
workspace-first `doublecheck` pass on the factual closure claims.

- Pass 1 found one significant issue: the simulator publication evidence path
  assumed Stage 8 status was always `ok`, so the checked-in live shared lane
  failed when the honest publication state was `draft_only`. The fix now loads
  the saved Stage 7 draft, derives the same publication binding from that
  draft, emits `checkpoint_id_hex: null`, and marks the validator or watcher
  publication story as `incomplete` or `draft_only`.
- Pass 2 re-audited `057-G9` through `057-G11` against
  `PublicationBinding`, `CheckpointFlow`, watcher checked snapshots, the YAML
  scenario packet, and the simulator trace assertions. The workspace-first
  `doublecheck` verified the material closure claims directly against the code
  and tests. No significant issues remained.
- Pass 3 repeated the same audit after the fresh bootstrap rerun, targeted
  release reruns, green `cargo test --release`, green `cargo doc --no-deps`,
  and clean `git diff --check`. No significant issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation

All Rust validation for this slice is green on the final code path.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate before broader validation and was rerun green
  for the final closeout pass.
- `cargo test -p z00z_validators --release` passed.
- `cargo test -p z00z_watchers --release` passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture`
  passed.
- `cargo test --release` passed for the workspace on the final code tree.
- `cargo doc --no-deps` passed with only pre-existing rustdoc warnings outside
  the `057-05` scope.
- `git diff --check` is clean.

## Result

`057-05` is complete. Phase 057 now advances to `057-06-PLAN.md` for the
final fixture, benchmark, validation, and planning-state closeout slice.

This summary does not claim final Phase 057 closure, full gate-matrix closure,
or final benchmark-home closure; those remain owned by `057-06`.
