---
phase: 057-HJMT-multi-aggregator
plan: 057-01
status: complete
completed_at: 2026-06-13
next_plan: 057-02
requirements-completed:
  - 057-G1
  - 057-G2
  - 057-G3
summary_artifact_for: .planning/phases/057-HJMT-multi-aggregator/057-01-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 057-01 Summary: Root Generation, Shard Leaf, And Checkpoint Publication Contracts

## Completed Scope

`057-01` is complete for the live Phase 057 publication-contract slice.

The repository now ships one executable publication byte path on the existing
storage and runtime seams: `PolicySetCommitmentV1`, `ShardRootLeafV1`, and
`CheckpointPublicationV1` live in `z00z_storage::settlement`, carry exact
canonical bytes and digests, and fail closed on generation confusion,
route-binding drift, shard-order drift, duplicate shards, reserved transition
bits, prior-root mismatch, and monotonicity violations. The first bridge from
the current visible settlement root into root-of-shard-roots publication is
now explicit and test-backed on the live owner seams instead of remaining a
paper-only contract.

This slice also hardens fixture authority so the checked-in publication corpus
matches the Phase 057 checklist exactly where it claims coverage. `SRL-T-002`
now means stale `route_table_digest`, `CPP-T-001` now means reordered shard
leaves, and the runtime-owned publish lane now binds publication metadata to a
live `ShardRouteTable::digest()` path instead of a synthetic digest literal.
Policy-set digest drift remains covered, but it no longer squats on a
checklist-owned fixture id.

The closeout keeps the user-requested scope rule intact: future-only wording
from the referenced HJMT design packet was treated as live execution authority
for this slice, and the implementation extended the current storage/runtime
surfaces in place instead of opening a parallel publication subsystem.

## Files Changed

- `.planning/phases/057-HJMT-multi-aggregator/057-01-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/proof.rs`
- `crates/z00z_storage/src/settlement/proof_batch.rs`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/snapshot/store.rs`
- `crates/z00z_storage/tests/test_settlement_root.rs`
- `crates/z00z_storage/tests/test_hjmt_root_generation.rs`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/README.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/README.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json`

## Boundary Kept Intact

- Phase 057 still publishes Phase 056 lineage; it did not reopen runtime
  routing truth, planner truth, or storage semantic truth.
- The publication layer stayed on one canonical ordered shard-leaf set plus
  one canonical checkpoint digest story; no second publication authority path
  was introduced.
- The runtime-owned route-table contract remained the only source of
  `route_table_digest` truth; this slice consumed `ShardRouteTable::digest()`
  instead of duplicating digest logic.
- Fixture ids were brought back into alignment with the HJMT checklist instead
  of inventing a parallel local numbering scheme.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found two significant issues: fixture ids drifted from the Phase 057
  checklist (`SRL-T-002`, `CPP-T-001`, and `CPP-T-002` were misassigned), and
  the runtime publish test still used a synthetic publication route-table
  digest instead of a live `ShardRouteTable::digest()` path. Both were fixed,
  the manifests were regenerated from live code, and the route-binding helper
  was added on `ShardRootLeafV1`.
- Pass 2 re-audited the checked-in fixture corpus, README authority text, and
  the runtime publish lane against `057-TODO.md` plus
  `Z00Z-HJMT-Fixture-Checklist.md`. No significant issues remained.
- Pass 3 repeated the same audit after the final validation wave, including
  diff hygiene, checklist-id alignment, and the absence of synthetic
  publication-digest literals on the runtime-owned test seam. No significant
  issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation

All Rust validation for this slice is green on the final code path.

- `cargo fmt --all` completed.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate and was rerun green after the final review
  fixes.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_settlement_root -- --nocapture`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_publish -- --nocapture`
  passed.
- `cargo test --release` passed for the workspace on the final code tree.
- `cargo doc --no-deps` passed. It reported pre-existing rustdoc warnings in
  untouched crates (`z00z_crypto`, `z00z_core`, `z00z_wallets`,
  `z00z_simulator`) outside the `057-01` slice.
- `git diff --check` is clean.

## Result

`057-01` is complete. Phase 057 now advances to `057-02-PLAN.md` for the
two-layer proof-composition and historical-compatibility slice.

This summary does not claim `SIM-5A7S-PUB` integration, join/transfer/carry-
forward execution, validator/watcher binding, scenario synchronization, or
phase closeout evidence; those remain owned by `057-02` through `057-06`.
