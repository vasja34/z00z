---
phase: 053-HJMT-Backend
plan: 053-13
status: complete
completed_at: 2026-06-02
next_plan: 053-14
requirements:
  - PH53-13
summary_artifact_for: .planning/phases/053-HJMT-Backend/053-13-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 053-13 Summary: RedB Reload And Historical Proofs

## Completed Scope

`053-13` is complete for the live RedB reload and historical-proof slice.

Live HJMT reload now rejects legacy simple-JMT persistence with a typed
unsupported-generation error instead of falling back to compatibility-shaped
rows or journal absence by key presence alone. The reload path now accepts only
validated HJMT settlement rows, typed fee replay metadata, and journal-backed
history before state becomes visible.

The slice also closes durable historical-proof coverage on the live settlement
seam. Close or reopen now preserves generalized asset leaves, right leaves,
deletion proof history, fee replay rows, adaptive bucket epochs, retained root
rows, and split or merge or policy-transition proof metadata strongly enough
for proof validation to survive reload and later unrelated writes. Reload
rejects flat-root drift, bucket drift, settlement-path drift, overwrite-history
corruption, fee replay drift, and legacy row generation drift before cache
warmup or proof access can succeed.

Phase-owned evidence is now explicit in the live tests. The slice covers
settlement-row-only reload, right-leaf roundtrip, deletion-proof replay after
reload, fee replay row tamper rejection, path-index rebuild rejection, typed
legacy-row rejection, and historical split or merge or policy-transition proof
validation under retained roots and epochs.

## Scoped Boundary

This summary closes the reload-and-historical-proof slice only. It does not
claim downstream checkpoint or wallet or validator integration, documentation
closeout beyond the slice authority sync, or the later legacy-purge work owned
by subsequent numbered plans.

## Review Loop

The required `GSD-Review-Tasks-Execution` loop completed for `053-13`.

- Review pass 1 found a live reload contract gap: the HJMT path still accepted
  missing journal metadata as a generic backend failure instead of a typed
  unsupported-generation rejection for legacy simple-JMT state.
- Review pass 2 found incomplete legacy evidence in `test_redb_reload.rs`: the
  seeded legacy-only fixture opened old tables but did not persist real legacy
  asset/path rows, so the rejection path was not proven against actual stale
  rows.
- Review pass 3 found a historical reload test bug in `test_redb_reload.rs`:
  unrelated post-reload writes reused asset ids that could collide with earlier
  inserted items and trigger `PathAssetMix` nondeterministically.
- Review pass 4 found the same collision pattern still present in
  `test_hjmt_proofs.rs`, so the reload-historical tests there were repaired to
  use unique post-reload asset ids as well.
- Review pass 5 reran the task on the post-fix tree and found no significant
  remaining issues.
- Review pass 6 reran the same task after full validation and found no
  significant remaining issues.

Two consecutive post-fix review passes were clean.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the final tree.
- `cargo test -p z00z_storage --release --features test-fast --test test_redb_reload --test test_hjmt_proofs --test test_fee_replay`
  passed.
- `cargo test -p z00z_storage --release --features test-fast` passed on the
  final tree.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed on the final tree.

## Result

`053-13` is complete for the owned slice. Phase 053 advances to
`053-14-PLAN.md` for downstream checkpoint, snapshot, wallet, and validator
integration with full release validation green on the final tree.
