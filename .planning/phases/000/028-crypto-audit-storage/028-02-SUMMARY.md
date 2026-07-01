---
phase: 028-crypto-audit-storage
plan: "02"
subsystem: storage
tags: [checkpoint, storage, redb, replay]
requires: []
provides:
  - canonical replay bytes preserved through checkpoint build and verifier handoff
  - explicit generic-vs-canonical RedB persistence split for checkpoint exec artifacts
  - atomic store mutation path that persists authoritative replay rows only when real tx proof bytes are supplied
affects: [checkpoint, storage, redb, simulator, phase-028]
tech-stack:
  added: []
  patterns: [canonical-replay-only, generic-store-no-fake-exec, atomic-apply-with-exec]
key-files:
  created: [.planning/phases/028-crypto-audit-storage/028-02-SUMMARY.md]
  modified:
    - crates/z00z_storage/src/checkpoint/exec_input.rs
    - crates/z00z_storage/src/assets/store.rs
    - crates/z00z_storage/src/assets/store_internal/tx_plan.rs
    - crates/z00z_storage/src/assets/store_internal/redb_backend.rs
    - crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs
    - crates/z00z_storage/tests/test_checkpoint_draft_build.rs
    - crates/z00z_storage/tests/test_redb_rehydrate.rs
    - crates/z00z_storage/tests/test_redb_mutation.rs
key-decisions:
  - "Keep CheckpointExecInput canonical-only and stop generic AssetStore mutation commits from synthesizing replay artifacts into RedB."
  - "Persist canonical RedB checkpoint exec artifacts only through one atomic apply_ops_with_exec path that receives authoritative tx rows with real proof bytes."
patterns-established:
  - "Canonical replay pattern: exact tx proof bytes survive exec encoding, draft build, verifier handoff, and RedB persistence without reconstruction."
  - "Generic mutation pattern: state-only commits may persist snapshots, but they do not publish canonical exec, draft, checkpoint, or link artifacts."
requirements-completed: [PH28-EXEC-PROOF, PH28-TRUST-HOOK]
duration: current-session
completed: 2026-03-30
---

# Phase 028 Plan 02: Canonical Replay Persistence Summary

Canonical replay artifacts now preserve real tx proof bytes end to end, while generic storage commits no longer fabricate checkpoint exec rows in RedB.

## Performance

- **Duration:** current-session
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Proved that canonical `CheckpointExecInput` bytes preserve the exact upstream `tx_proof` payload through codec roundtrip and `build_cp_draft(...)` verifier handoff.
- Removed the placeholder `vec![(index as u8).saturating_add(1)]` replay emitter from the storage mutation path.
- Split generic store mutation from canonical replay persistence so normal `put_item` and `apply_ops` commits stop publishing fake exec, draft, checkpoint, and link artifacts.
- Added `apply_ops_with_exec(...)` as one atomic storage-owned path for committing state together with authoritative replay rows backed by real proof bytes.
- Updated RedB rehydrate and mutation tests to distinguish canonical replay persistence from snapshot-only generic state commits.

## Task Commits

1. **Task 1: canonical replay proof preservation** - `07dd4bbb` (test)
2. **Task 2: RedB canonical-vs-generic split** - `deaba962` (feat)

## Files Created/Modified

- `crates/z00z_storage/src/checkpoint/exec_input.rs` - Clarified that canonical exec rows preserve exact upstream proof bytes.
- `crates/z00z_storage/src/assets/store.rs` - Added `apply_ops_with_exec(...)`, removed placeholder exec synthesis, and enforced replay rows matching store ops.
- `crates/z00z_storage/src/assets/store_internal/tx_plan.rs` - Threaded optional canonical exec rows through the atomic commit pipeline.
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` - Made checkpoint exec, draft, artifact, and link persistence conditional on one canonical exec bundle.
- `crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs` - Added exact-proof-byte replay roundtrip coverage.
- `crates/z00z_storage/tests/test_checkpoint_draft_build.rs` - Added verifier-side proof-byte preservation coverage.
- `crates/z00z_storage/tests/test_redb_rehydrate.rs` - Added canonical RedB replay persistence and generic mutation non-publication coverage.
- `crates/z00z_storage/tests/test_redb_mutation.rs` - Updated generic RedB mutation expectations for the new snapshot-only default path.

## Decisions Made

- Generic storage commits without authoritative tx rows now persist only snapshot and state history, not production-looking replay artifacts.
- Canonical RedB replay persistence is atomic with state mutation only when callers supply authoritative `CheckpointExecTx` rows.
- Storage-side validation for `apply_ops_with_exec(...)` compares exec inputs and outputs against the concrete store ops before persisting canonical replay bytes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] RedB mutation tests assumed absent checkpoint tables would still exist as zero-row tables**

- **Found during:** Task 2 verification
- **Issue:** After the generic-vs-canonical split, `test_redb_mutation` panicked on `TableDoesNotExist` instead of treating omitted checkpoint tables as empty.
- **Fix:** Changed the test helper to interpret a missing RedB table as `0` rows for generic state-only commits.
- **Files modified:** `crates/z00z_storage/tests/test_redb_mutation.rs`
- **Verification:** `cargo test -p z00z_storage --release --test test_redb_mutation -- --nocapture`
- **Committed in:** `deaba962`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The fix kept the intended persistence split and removed only a stale test assumption.

## Issues Encountered

- The old RedB mutation assertions encoded an implicit architectural assumption that generic state commits must always emit checkpoint exec artifacts.
- `requirements mark-complete` is still expected to fail for phase 028 ids because the mapped `PH28-*` entries remain absent from `.planning/REQUIREMENTS.md`.

## User Setup Required

None.

## Next Phase Readiness

- Phase 028 can now build on one honest replay contract: canonical replay bytes come from real tx proofs, and generic state persistence no longer pollutes RedB with placeholder checkpoint exec rows.
- Wave 3 can bind semantic and backend roots without inheriting fake replay artifacts from storage mutation helpers.

## Self-Check: PASSED

- Found summary file: `.planning/phases/028-crypto-audit-storage/028-02-SUMMARY.md`
- Verified task commit: `07dd4bbb`
- Verified task commit: `deaba962`

---
*Phase: 028-crypto-audit-storage*
*Completed: 2026-03-30*
