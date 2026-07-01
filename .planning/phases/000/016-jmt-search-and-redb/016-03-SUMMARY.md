---
phase: 016-jmt-search-and-redb
plan: 03
subsystem: api
tags: [search, pagination, asset-store, canonical-order, redb]
requires:
  - phase: 016-02
    provides: durable active-version rehydrate and canonical RedB reload semantics
provides:
  - public typed search contract for exact lookup and deterministic listing
  - canonical-order pagination semantics derived from AssetPath ordering
  - regression tests and boundary docs that keep search subordinate to canonical path ownership
affects: [z00z_storage, downstream wallets, future read-model work]
tech-stack:
  added: []
  patterns: [canonical-order pagination, convenience-only search indexes, deterministic list replay]
key-files:
  created:
    - crates/z00z_storage/tests/search_api.rs
  modified:
    - crates/z00z_storage/src/assets/README.MD
    - crates/z00z_storage/src/assets/mod.rs
    - crates/z00z_storage/src/assets/model.rs
    - crates/z00z_storage/src/assets/store.rs
    - crates/z00z_storage/src/assets/types.rs
key-decisions:
  - "Expose typed search requests and responses from the storage crate while keeping AssetPath as the only canonical contract."
  - "Derive ordering and pagination tokens from canonical model path order rather than backend iteration order."
patterns-established:
  - "Convenience-only search: asset_id lookup and list APIs may accelerate reads but never redefine roots or path ownership."
  - "Pagination replay follows definition_id -> serial_id -> asset_id ordering and resumes after the last returned canonical path."
requirements-completed: [STREDB-03, STREDB-04]
duration: 40m
completed: 2026-03-24
---

# Phase 016 Plan 03: Deterministic Search Summary

📌 `z00z_storage` now exposes a typed deterministic search surface for exact lookup, scoped listing, and canonical-order pagination without elevating convenience indexes into a new source of truth.

## Performance

- ⏰ **Duration:** 40m
- ⏰ **Started:** 2026-03-24T01:05:00+02:00
- ✅ **Completed:** 2026-03-24T01:45:56+02:00
- ✅ **Tasks:** 3
- ✅ **Files modified:** 6

## Accomplishments

- ✅ Added public search-facing storage types for exact lookup, scoped list requests, page results, and pagination tokens.
- ✅ Implemented deterministic exact lookup and canonical-order listing APIs in `AssetStore`, including stable pagination replay.
- ✅ Added integration tests and storage-boundary docs proving root invariance after reload and clarifying that search indexes remain convenience-only.

## Task Commits

1. **Task 1: Define the public search contract per D-01 through D-09** - `e545d606` (feat)
2. **Task 2: Implement deterministic search and pagination over canonical ordering** - `e545d606` (feat)
3. **Task 3: Add deterministic search tests and boundary documentation** - `e545d606` (feat)

## Files Created/Modified

- `crates/z00z_storage/src/assets/types.rs` - Adds `AssetLookup`, `AssetScope`, `AssetListReq`, `AssetPage`, and `AssetPageTok`.
- `crates/z00z_storage/src/assets/mod.rs` - Re-exports the typed search contract from the storage facade.
- `crates/z00z_storage/src/assets/store.rs` - Implements exact lookup, deterministic list queries, and canonical-order pagination.
- `crates/z00z_storage/src/assets/model.rs` - Exposes canonical model path traversal used for deterministic search ordering.
- `crates/z00z_storage/src/assets/README.MD` - Documents the search surface as convenience-only and subordinate to canonical path semantics.
- `crates/z00z_storage/tests/search_api.rs` - Proves exact lookup, scoped listing, pagination replay, and reload-stable root behavior.

## Decisions Made

- ✅ Kept `AssetPath` primary and exposed `asset_id` lookup plus list APIs only as typed convenience helpers.
- ✅ Derived page tokens from the last returned canonical path instead of backend-specific iterator position.
- ✅ Used canonical `AssetModel` traversal for ordered listing so search results do not depend on `path_by_id` mutation history or backend row iteration quirks.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed pagination replay to resume after the last returned canonical item**

- **Found during:** Task 2 (deterministic pagination validation)
- **Issue:** The next-page token initially pointed at the first unreturned path, which caused the following page to skip or collapse results.
- **Fix:** Changed token derivation to use the last returned `StoreItem` path and tightened canonical path filtering around `after` semantics.
- **Files modified:** `crates/z00z_storage/src/assets/store.rs`
- **Verification:** `cargo test -p z00z_storage --test search_api -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_storage --test search_api -- --nocapture`
- **Committed in:** `e545d606`

**2. [Rule 1 - Bug] Fixed ordered listing to traverse the canonical model instead of convenience map history**

- **Found during:** Task 2 (reload and ordering regression tests)
- **Issue:** Listing based on `path_by_id` could diverge from canonical committed order after reload and multi-version mutation history.
- **Fix:** Added canonical path traversal to `AssetModel` and made `AssetStore::sorted_paths()` derive ordered results from model state, with stable `sort_unstable()` normalization.
- **Files modified:** `crates/z00z_storage/src/assets/model.rs`, `crates/z00z_storage/src/assets/store.rs`
- **Verification:** `cargo test -p z00z_storage --test search_api -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_storage --test search_api -- --nocapture`
- **Committed in:** `e545d606`

---

✅ **Total deviations:** 2 auto-fixed (2 bug fixes)
📌 **Impact on plan:** Both deviations were necessary to make pagination and ordered listing deterministic under reload and multi-version state; no scope creep was introduced.

## Issues Encountered

- ⚠️ Search determinism depended on the durable reload semantics finalized in the overlapping `AssetStore` path, so the final green validation required the active-version rehydrate fix already recorded in plan 02.

## User Setup Required

✅ None - no external service configuration required.

## Next Phase Readiness

- ✅ Phase 016 now has both durable live-state reload and a deterministic search surface, so the roadmap can treat the RedB/search phase as complete.
- ✅ Future storage work can extend read-model helpers without changing canonical ordering or durable root ownership contracts.

## Self-Check: PASSED

- ✅ Found `.planning/phases/016-jmt-search-and-redb/016-03-SUMMARY.md`.
- ✅ Verified implementation commit `e545d606` exists in git history.

---
*Phase: 016-jmt-search-and-redb*
*Completed: 2026-03-24*
