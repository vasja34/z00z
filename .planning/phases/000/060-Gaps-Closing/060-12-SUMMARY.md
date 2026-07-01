---
phase: 060-Gaps-Closing
plan: 060-12
status: complete
completed_at: 2026-06-22
next_plan: 060-13
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-12-PLAN.md
---

# 060-12 Summary: HJMT Core Storage Shard Truth Closure

## Completed Scope

`060-12` is complete for the supplemental HJMT core-storage reopen.

This slice closes the three reopened storage seams on the live tree without
introducing a second authority path. Generation-1 root-of-shard-roots is now
the storage-owned live settlement-root truth, the durable journal and recovery
export now bind shard route identity and lineage instead of collapsing to one
bucket-centric store version, and the shared publication-route checker is live
on storage, simulator, rollup-node preflight, validator, and watcher
surfaces.

The closeout is now backed by full current-tree release evidence instead of
targeted-only claims. The mandatory bootstrap rerun is green, the targeted
`060-12` release packet remains green on the current tree with the corrected
`test_live_recovery` selector and preserved `test_hjmt_migrate` target, and
the full `cargo test --release` rerun is green on the same tree. The long
default-feature `scenario_1` tail that previously kept this slice open now
completes inside the broad workspace gate:
`z00z_simulator/tests/scenario_1/main.rs` finished with
`252 passed; 0 failed; 1 ignored; finished in 1920.59s`.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-12-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Boundary Kept

- No second route-table codec, second publication checker, or second
  settlement-root authority plane was introduced.
- `aggregator_owned` remains the HJMT production default; this slice only
  closes storage truth and shared publication acceptance.
- The existing `.planning/phases/060-Gaps-Closing/` folder remains the only
  Phase 060 authority path.
- No edits were made under `crates/z00z_crypto/tari/**`.

## Review Loop

Manual workspace-first review was used instead of repeated slash-prompt
execution because `/GSD-Review-Tasks-Execution` is not a callable tool in this
environment.

- Pass 1 rechecked the `060-12` contract against the current tree and kept the
  plan packet honest by preserving the real `test_hjmt_migrate` target and the
  corrected `test_live_recovery` lib-test selector.
- Pass 2 reran the mandatory bootstrap gate and confirmed the current tree was
  still clean enough for broad validation.
- Pass 3 completed the full `cargo test --release` rerun and verified that the
  long `scenario_1` tail now closes green instead of leaving `060-12` open.
- Pass 4 synchronized `STATE` and `ROADMAP` so the canonical active lane now
  moves to `060-13-PLAN.md` with no conflicting status text.

Two consecutive clean review passes were achieved on passes 3 and 4 after the
final broad release gate completed green.

## Validation

- Mandatory bootstrap gate passed:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- The targeted `060-12` release packet remained green on the current tree via
  the verify commands frozen in `060-12-PLAN.md`, including the corrected
  `test_live_recovery` selector and the preserved
  `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_migrate -- --nocapture`
  target.
- Broad workspace release validation passed:
  `cargo test --release`
- The long default-feature simulator tail now closes inside the broad workspace
  gate:
  `z00z_simulator/tests/scenario_1/main.rs`
  finished with `252 passed; 0 failed; 1 ignored; finished in 1920.59s`.

## Result

`060-12` is complete. Phase 060 now advances to `060-13-PLAN.md` for the
wallet/object reject-path closeout packet, while future full
`z00z-verification-orchestrator` reruns remain operator-owned manual work.
