---
phase: 016-jmt-search-and-redb
plan: 02
subsystem: database
tags: [redb, jmt, storage, rehydration, checkpoint, snapshot]
requires:
  - phase: 016-01
    provides: private RedB mutation boundary and synchronous durable commit seam
provides:
  - canonical snapshot and checkpoint blob persistence keyed by existing content ids
  - durable AssetStore load path that rebuilds canonical rows and committed root state
  - rehydration regression coverage for durable roundtrip and blob-id stability
affects: [016-03, z00z_storage]
tech-stack:
  added: []
  patterns: [canonical blob reuse, durable active-version rehydrate, storage-owned reload]
key-files:
  created:
    - crates/z00z_storage/tests/redb_rehydrate.rs
  modified:
    - crates/z00z_storage/src/assets/store.rs
    - crates/z00z_storage/src/assets/store_internal/redb_backend.rs
    - crates/z00z_storage/src/assets/store_internal/tx_plan.rs
    - crates/z00z_storage/src/snapshot/mod.rs
    - crates/z00z_storage/src/snapshot/store.rs
    - crates/z00z_storage/tests/redb_mutation.rs
key-decisions:
  - "Persist canonical snapshot and checkpoint bytes directly in RedB instead of inventing a second artifact schema."
  - "Rehydrate the durable store through one committed batch at the persisted active version so canonical lookups survive process restart."
patterns-established:
  - "Canonical ids stay primary: RedB keys for persisted artifacts follow existing PrepSnapshotId and checkpoint id derivation rules."
  - "Durable load is storage-owned: AssetStore rebuilds committed state without wallet persistence helpers or lazy replay side effects."
requirements-completed: [STREDB-01, STREDB-02]
duration: 45m
completed: 2026-03-24
---

# Phase 016 Plan 02: Durable Rehydration Summary

📌 Canonical snapshot and checkpoint blobs now persist through RedB, and durable reload rebuilds a usable `AssetStore` with the same canonical root and path semantics after restart.

## Performance

- ⏰ **Duration:** 45m
- ⏰ **Started:** 2026-03-24T01:00:00+02:00
- ✅ **Completed:** 2026-03-24T01:45:56+02:00
- ✅ **Tasks:** 3
- ✅ **Files modified:** 7

## Accomplishments

- ✅ Reused existing snapshot and checkpoint codecs to persist canonical blob bytes and ids in dedicated RedB tables.
- ✅ Added a storage-owned durable `AssetStore::load(...)` path that rehydrates committed rows, roots, and lookup semantics from RedB-backed state.
- ✅ Added roundtrip regression tests proving root equivalence, canonical path lookup equivalence, and blob-id stability after durable reload.

## Task Commits

📌 Implementation and final correctness landed across one main plan commit plus one shared follow-up in the overlapping `store.rs` reload path:

1. **Task 1: Reuse canonical snapshot and checkpoint bytes for RedB blob persistence per D-11 and D-12** - `5c657101` (feat)
2. **Task 2: Implement full durable AssetStore rehydration per D-13 and Gate V2** - `5c657101`, `e545d606` (feat)
3. **Task 3: Add durable roundtrip and rehydration tests** - `5c657101` (feat)

## Files Created/Modified

- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` - Persists canonical snapshot, draft, checkpoint, exec-input, and link blobs by their canonical ids.
- `crates/z00z_storage/src/assets/store.rs` - Adds durable load and active-version rehydrate logic for committed state.
- `crates/z00z_storage/src/assets/store_internal/tx_plan.rs` - Carries preplanned artifact inputs into the durable commit boundary.
- `crates/z00z_storage/src/snapshot/store.rs` - Exposes crate-private snapshot byte helper reused by the backend.
- `crates/z00z_storage/src/snapshot/mod.rs` - Re-exports the snapshot byte helper inside the crate.
- `crates/z00z_storage/tests/redb_mutation.rs` - Verifies canonical artifact tables participate in atomic durable writes.
- `crates/z00z_storage/tests/redb_rehydrate.rs` - Proves durable roundtrip and blob-id stability after reload.

## Decisions Made

- ✅ Kept artifact persistence byte-compatible with the existing snapshot and checkpoint codecs so content-derived ids stay stable.
- ✅ Preserved a storage-owned reload path in `AssetStore` instead of routing rehydrate through wallet or filesystem persistence code.
- ✅ Treated the persisted active version as authoritative during rehydrate so path-index lookups and ordered reads survive process restart.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed durable reload to rebuild at the persisted active version**

- **Found during:** Task 2 (durable rehydration verification)
- **Issue:** Rehydration rebuilt in-memory state on a synthetic next version, which left canonical search and path resolution empty after reload on multi-version RedB state.
- **Fix:** Replaced `apply_ops()`-based replay with `plan_ops()` + `commit_plan(..., state.version)` so the rebuilt flat tree, path index, and model state all align with the persisted active version.
- **Files modified:** `crates/z00z_storage/src/assets/store.rs`
- **Verification:** `cargo test -p z00z_storage --test redb_rehydrate -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_storage --test redb_rehydrate -- --nocapture`, `cargo test -p z00z_storage --test search_api test_search_root_stays_stable_after_reload -- --nocapture`
- **Committed in:** `e545d606`

---

✅ **Total deviations:** 1 auto-fixed (1 bug fix)
📌 **Impact on plan:** The deviation was required for correctness of durable reload semantics and did not widen scope beyond the planned RedB rehydration contract.

## Issues Encountered

- ⚠️ Durable artifact persistence and durable reload share the same `AssetStore` core path, so the final active-version rehydrate fix landed in a shared follow-up commit that also finalized the search plan.

## User Setup Required

✅ None - no external service configuration required.

## Next Phase Readiness

- ✅ `016-03` can now expose public search APIs on top of a verified durable reload path instead of relying on transient in-memory state.
- ✅ Canonical ids, durable blob tables, and active-version rehydrate semantics are stable inputs for deterministic search pagination.

## Self-Check: PASSED

- ✅ Found `.planning/phases/016-jmt-search-and-redb/016-02-SUMMARY.md`.
- ✅ Verified implementation commits `5c657101` and `e545d606` exist in git history.

---
*Phase: 016-jmt-search-and-redb*
*Completed: 2026-03-24*
