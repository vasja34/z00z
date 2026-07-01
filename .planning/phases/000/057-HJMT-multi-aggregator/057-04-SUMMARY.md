---
phase: 057-HJMT-multi-aggregator
plan: 057-04
status: complete
completed_at: 2026-06-13
next_plan: 057-05
requirements-completed:
  - 057-G6
  - 057-G7
  - 057-G8
summary_artifact_for: .planning/phases/057-HJMT-multi-aggregator/057-04-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 057-04 Summary: Join, Transfer, Carry-Forward, And Crash Recovery

## Completed Scope

`057-04` is complete for the live Phase 057 lawful-transition slice.

The repository now ships one executable lawful-transition packet on the
existing Phase 056 lineage instead of a future-only design promise. The live
aggregator evidence now distinguishes standby join from owner activation,
binds ownership change to route generation `N+1` plus one activation
checkpoint, proves transfer both to a remaining aggregator and to a new
aggregator, and records the carry-forward or crash vectors with exact public
root and carried-forward-leaf bytes in the checked-in failover corpus.

This closeout also fixes the honest release-only gaps that blocked a green
workspace verification pass after the lawful-transition code landed. The final
review loop hardened the same canonical packet instead of adding another lane:
the shared crash-injection environment is now serialized across all recovery
helpers, the durable-crash storage test now proves one exact prior root and one
exact durable successor root, and the positive `FOV-G-002` through `FOV-G-004`
tests now stay aligned with one checked-in fixture vocabulary.

Future-only wording in the referenced HJMT packet stayed live scope authority
for this slice, but the implementation remained on the existing runtime,
storage, and rollup-node seams rather than creating a second transition engine.

## Files Changed

- `.planning/phases/057-HJMT-multi-aggregator/057-04-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/README.md`
- `crates/z00z_runtime/aggregators/tests/fixtures/failover_v1/manifest.json`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
- `crates/z00z_runtime/aggregators/tests/test_recovery_common.rs`
- `crates/z00z_storage/src/settlement/test_live_recovery.rs`

## Boundary Kept Intact

- Standby join mirrors lineage only. It does not create a second public
  authority path or silently replace the current owner.
- Owner activation is lawful only after the committed next route generation
  and one activation checkpoint; checkpoint rollback stays fail-closed.
- Carry-forward reuses the failed shard leaf byte-for-byte. It does not
  recompute a semantic approximation under partial failure.
- Crash recovery exposes only the prior visible public root or the exact later
  durable root; route migration during crash never makes a partial migration
  root visible.
- The recovery-helper hardening stays on the same crash-injection and fixture
  corpus seams; it does not create a second recovery or publication contract.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found one significant issue: `durable_commit_publication_case()`
  serialized the shared crash-injection environment, but
  `live_recovery_state()` did not, so concurrent recovery helpers could read
  the injected `"parents"` stage unexpectedly. The shared
  `storage_injection_lock()` guard was added to both helper paths.
- Pass 2 found one significant issue: the new storage crash-lawfulness test
  allowed either root via a tautological check instead of proving the exact
  prior visible root and exact durable successor root. The test now asserts the
  exact golden root hex values.
- Pass 3 found one significant issue: the new positive-lawfulness test names
  drifted from the canonical fixture vocabulary. The plan-owned tests were
  renamed to the concise canonical ids while the checked-in manifest kept the
  authoritative fixture ids.
- Pass 4 re-audited the final lawful-transition diff after the targeted
  release reruns. No significant issues remained.
- Pass 5 repeated the same audit after the fresh bootstrap rerun, the fresh
  green workspace `cargo test --release`, and the final planning-state sync.
  No significant issues remained.

Two consecutive clean review passes were achieved on passes 4 and 5.

## Validation

All Rust validation for this slice is green on the final code path.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed as
  the mandatory fail-fast gate before broader validation and was rerun green
  for the final closeout pass.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_join -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_migrate -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_split_brain_fencing -- --nocapture`
  passed.
- `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast settlement::store::test_live_recovery::test_parent_crash_exact_root -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --test test_stage4_selection -- --nocapture`
  passed.
- `cargo test --release` passed for the workspace on the final code tree.
- `git diff --check` is clean.
- No `cargo doc --no-deps` rerun was required for this slice because the
  closeout changed no public runtime/node API contract or normative public
  documentation surface.

## Result

`057-04` is complete. Phase 057 now advances to `057-05-PLAN.md` for the
validator, watcher, scope-continuity, and scenario-sync slice.

This summary does not claim validator or watcher closeout, first-seen
scope-continuity closeout, scenario-sync closeout, or final fixture and
benchmark closeout; those remain owned by `057-05` and `057-06`.
