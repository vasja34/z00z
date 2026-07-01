---
phase: 016-jmt-search-and-redb
plan: 01
subsystem: database
tags: [redb, jmt, storage, durability, asset-store]
requires:
  - phase: 015-03
    provides: deterministic artifact build, persistence helpers, and storage-owned inspection boundaries
provides:
  - private RedB durability backend for live AssetStore mutations
  - synchronous mutation commit seam that fails atomically on durable write errors
  - integration coverage for rollback, reopen, batch commit, and root-safe secondary rows
affects: [016-02, 016-03, z00z_storage]
tech-stack:
  added: [redb]
  patterns: [storage-owned durable backend, single commit seam, root-safe secondary indexes]
key-files:
  created:
    - crates/z00z_storage/src/assets/store_internal/redb_backend.rs
    - crates/z00z_storage/tests/redb_mutation.rs
  modified:
    - Cargo.lock
    - crates/z00z_storage/Cargo.toml
    - crates/z00z_storage/src/assets/store.rs
    - crates/z00z_storage/src/assets/store_internal/tx_plan.rs
    - crates/z00z_storage/src/error.rs
key-decisions:
  - "Keep all RedB types inside a private backend module and expose only storage-owned backend failures at the AssetStore boundary."
  - "Bind durable persistence to the existing mutation commit path so a committed AssetStateRoot is never returned before the durable write succeeds."
  - "Rehydrate persisted rows through one apply_ops batch and collapse history maps to the active durable version to preserve canonical root semantics after reopen."
patterns-established:
  - "Durable commit before success: live-state mutations must complete one storage-owned backend transaction before returning a committed root."
  - "Secondary indexes are convenience-only: persisted lookup rows may accelerate reads but cannot redefine canonical roots or path ownership."
requirements-completed: [STREDB-01, STREDB-04]
duration: 1h 52m
completed: 2026-03-23
---

# Phase 016 Plan 01: RedB Mutation Boundary Summary

## Outcome

Private RedB-backed durability for live `AssetStore` mutations, wired through one synchronous commit seam with rollback-safe reopen coverage.

## Performance

- **Duration:** 1h 52m
- **Started:** 2026-03-23T21:28:29Z
- **Completed:** 2026-03-23T23:20:18Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Added a private RedB backend module that owns table layout, key encoding, and durable write/load helpers for asset state.
- Wired the durable write into the existing mutation boundary so `put_item`, `del_item`, and `apply_ops` cannot report a committed root after a failed backend transaction.
- Added integration coverage for persistence, rollback, secondary rows, reopen-after-commit, delete/apply_ops behavior, and reopen-after-batch regressions.

## Task Commits

Implementation landed in one atomic code commit because the backend contract, mutation seam, and regression coverage had to evolve together to keep the storage boundary compiling through the review loop:

1. **Tasks 1-3: Private RedB backend, mutation seam, and regression coverage** - `25b9896e` (feat)

## Files Created/Modified

- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` - Private RedB backend with table families, metadata, row encoding, durable sync, and durable load helpers.
- `crates/z00z_storage/tests/redb_mutation.rs` - End-to-end mutation durability coverage using real store operations and temporary RedB roots.
- `crates/z00z_storage/src/assets/store.rs` - Backend wiring, durable reopen path, and canonical rehydrate logic.
- `crates/z00z_storage/src/assets/store_internal/tx_plan.rs` - Commit-path durable sync and full rollback snapshot restore.
- `crates/z00z_storage/src/error.rs` - Storage-owned backend error path without leaking RedB-specific types.
- `crates/z00z_storage/Cargo.toml` - Added the `redb` dependency.
- `Cargo.lock` - Locked RedB dependency resolution for the workspace.

## Decisions Made

- Kept RedB isolated behind `store_internal/redb_backend.rs` so the public `z00z_storage` surface remains storage-owned.
- Inserted the durable write exactly at the mutation commit boundary instead of introducing a second flush path or deferred background sync.
- Rebuilt persisted rows with one `apply_ops` batch during rehydrate to preserve canonical path/root semantics across reopen and batch commits.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed RedB slice writes for fixed-size root bytes**

- **Found during:** Task 1 (private backend implementation)
- **Issue:** RedB insert calls were using fixed-size arrays where byte slices were required, blocking compilation.
- **Fix:** Converted persisted root and artifact writes to the byte slice forms RedB expects.
- **Files modified:** `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`
- **Verification:** `cargo test -p z00z_storage --lib -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_storage --lib -- --nocapture`
- **Committed in:** `25b9896e`

**2. [Rule 3 - Blocking] Restored stable database reuse for repeated mutation calls**

- **Found during:** Task 2 (durable mutation seam wiring)
- **Issue:** Reopening the same RedB file repeatedly caused `DatabaseAlreadyOpen` failures and later hangs in integration runs.
- **Fix:** Reintroduced cached database initialization with a stable `OnceLock`-owned handle per backend instance.
- **Files modified:** `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`
- **Verification:** `cargo test -p z00z_storage --test redb_mutation -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_storage --test redb_mutation -- --nocapture`
- **Committed in:** `25b9896e`

**3. [Rule 1 - Bug] Fixed reopen rehydrate semantics after multi-row commits**

- **Found during:** Task 2 (durable load verification)
- **Issue:** Replaying persisted rows with per-item `put_item` created phantom version history and broke reopen-after-batch semantics.
- **Fix:** Batched rehydrate through one `apply_ops` call and reset version-history maps to only the active durable version.
- **Files modified:** `crates/z00z_storage/src/assets/store.rs`
- **Verification:** `cargo test -p z00z_storage --test redb_mutation -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_storage --test redb_mutation -- --nocapture`
- **Committed in:** `25b9896e`

**4. [Rule 1 - Bug] Fixed env-scoped integration test cleanup ordering**

- **Found during:** Task 3 (integration regression coverage)
- **Issue:** Tests could release the shared lock before clearing `Z00Z_STORAGE_REDB_ROOT`, leaving a race for subsequent runs.
- **Fix:** Cleared env overrides before unlocking the shared test guard in RedB integration tests.
- **Files modified:** `crates/z00z_storage/tests/redb_mutation.rs`
- **Verification:** `cargo test -p z00z_storage --test redb_mutation -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_storage --test redb_mutation -- --nocapture`
- **Committed in:** `25b9896e`

---

**Total deviations:** 4 auto-fixed (3 bug fixes, 1 blocking issue)
**Impact on plan:** All deviations were required for correctness of the RedB mutation seam and did not widen scope beyond the plan objective.

## Issues Encountered

- One review pass interpreted canonical artifact persistence more broadly than the `016-01` mutation-boundary scope. The implemented seam persists the storage-owned artifacts required by the current durable commit path, and `016-02` will extend that persisted load surface further.
- The repository still carries an unrelated working-tree diff in `.codacy/codacy.yaml`; it was left untouched because it is outside `016-01` scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `016-02` can build on an established RedB ownership boundary and durable reopen path instead of inventing a second persistence seam.
- `016-03` can rely on persisted convenience rows remaining root-safe while adding deterministic search APIs.

## Self-Check: PASSED

- Found `.planning/phases/016-jmt-search-and-redb/016-01-SUMMARY.md`.
- Verified implementation commit `25b9896e` exists in git history.

---
*Phase: 016-jmt-search-and-redb*
*Completed: 2026-03-23*
