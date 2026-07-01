---
phase: 053-HJMT-Backend
plan: 053-09
status: complete
completed_at: 2026-06-02
next_plan: 053-10
requirements:
  - PH53-09
summary_artifact_for: .planning/phases/053-HJMT-Backend/053-09-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 053-09 Summary: Occupancy Privacy Evidence

## Completed Scope

`053-09` is complete for privacy-bounded adaptive occupancy evidence.

The live settlement surface now carries `BucketOccupancyEvidence` as the
proof-visible adaptive metadata object for split, merge, and policy-transition
proofs. The evidence is versioned, bound to scope and occupancy class, and
commits to bucket or pair or set state through a policy-bound digest instead of
exposing raw `leaf_count`, exact deltas, or timing fields. Exact counts remain
local-only diagnostics through `BucketOccupancyMetric` and are not exported in
the adaptive proof payloads.

The closeout extends the existing settlement-owned seams instead of creating a
parallel authority layer. `AdaptiveBucket`, `SplitProof`, `MergeProof`, and
`PolicyTransitionProof` now carry bounded occupancy evidence, validation rejects
tampered evidence with `AdaptiveProofErr::OccupancyDrift`, and transition
evidence is recomputed from canonical live bucket contents so interrupted reload
or recovery paths cannot drift proof-visible bindings.

Privacy coverage is now explicit. The new occupancy suites prove that exact
counts stay visible only in local metrics, encoded proof payloads block raw
counter field names, split and merge proofs do not reveal unrelated bucket
distributions when the bounded class is unchanged, and repeated adaptive proofs
do not turn occupancy metadata into an activity feed beyond the authorized
policy class.

## Scoped Boundary

This summary closes the occupancy-privacy slice only. It does not claim forest
cache layers, scheduler productionization, downstream generalized settlement
integration, documentation closeout beyond the occupancy contract updates, or
legacy purge work.

## Review Loop

The required `GSD-Review-Tasks-Execution` loop completed for `053-09`.

- Review pass 1 reopened two correctness issues: privacy tests had a brittle
  exact-count assumption, and policy-transition occupancy binding could drift
  after recovery because it depended on internal root-row shape instead of the
  canonical live bucket contents. The tests were made class-based, and
  transition evidence now binds against the current sorted bucket key set.
- Review pass 2 reran the task on the updated tree and found no significant
  remaining issues.
- Review pass 3 reran the same task after full validation and found no
  significant remaining issues.

Two consecutive post-fix review passes were clean.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on the final tree.
- `cargo test -p z00z_storage --release --features test-fast` passed on the final tree.
- `cargo test -p z00z_storage --release --features test-fast --test test_occupancy_evidence --test test_occupancy_privacy` passed.
- `cargo test --release --features test-fast --features wallet_debug_dump` passed on the final tree.

## Result

`053-09` is complete. Phase 053 advances to `053-10-PLAN.md` for the private
forest-cache slice with full release validation green on the final tree.
