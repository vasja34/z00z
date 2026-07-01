---
phase: 030-refactor-long-files
plan: 01
subsystem: database
tags: [redb, wallet-store, facade, persistence, migrations]
requires: []
provides:
  - semantic wallet-store seam modules behind the stable redb wallet facade
  - green wallet open, KDF migration, and persistence-boundary anchors for the split
  - shallow crate-local db re-exports for wallet identity and store backend consumers
affects: [wallet-service, phase-030-02, phase-030-wallet-store-followups]
tech-stack:
  added: []
  patterns: [semantic seam extraction, facade-preserving split, crate-local db re-export cleanup]
key-files:
  created:
    - crates/z00z_wallets/src/db/redb_wallet_store_backup.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_codecs.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_migrations.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_objects.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_queries.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_session.rs
    - crates/z00z_wallets/src/db/redb_wallet_store_tables.rs
  modified:
    - crates/z00z_wallets/src/db/mod.rs
    - crates/z00z_wallets/src/db/redb_wallet_crypto.rs
    - crates/z00z_wallets/src/db/redb_wallet_store.rs
    - crates/z00z_wallets/src/services/app_service.rs
    - crates/z00z_wallets/src/services/wallet_paths.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/tests/test_wallet_persistence_no_std.rs
key-decisions:
  - Keep `redb_wallet_store.rs` as the stable caller-visible facade while moving homogeneous seams into sibling modules.
  - Centralize steady-state zstd flush ownership in the session seam and keep persisted secret-AAD version markers owned by `redb_wallet_crypto.rs`.
  - Keep service callers on shallow `crate::db` re-exports instead of direct `wlt_store` paths when no public API expansion is required.
patterns-established:
  - Semantic seam extraction: tables, codecs, migrations, session, queries, objects, backup, and crypto helpers move behind one facade instead of numeric file shards.
  - Boundary-safe test guards: source-shape anchors validate facade behavior and forbidden boundary calls across the full seam set rather than one file.
requirements-completed: [PH30-SEAMS, PH30-PROTECTED, PH30-VERIFY]
duration: 1h 42m
completed: 2026-03-31
---

# Phase 030 Plan 01 Summary

**RedB wallet-store split into semantic persistence seams behind the stable `redb_wallet_store` facade with green open, migration, and boundary anchors**

## Performance

- **Duration:** 1h 42m
- **Started:** 2026-03-30T22:57:03Z
- **Completed:** 2026-03-31T00:38:42Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments

- Split the wallet-store monolith into dedicated tables, codecs, migrations, session, objects, queries, backup, and crypto seam modules without breaking the caller-visible `crate::db::redb_wallet_store::*` boundary.
- Preserved wallet-open and migration behavior through green release anchors for `test_redb_wlt_open`, `test_wallet_kdf_migration`, and the updated persistence-boundary guardrail test.
- Tightened boundary ownership after review by centralizing secret-AAD version markers, moving steady-state flush ownership into the session seam, and routing service callers through shallow `crate::db` re-exports.

## Task Commits

This plan is prepared for one consolidated repository-managed version-manager commit after artifact creation. Per-task hashes were not materialized during execution because the repo workflow requires the final staged file set to flow through `version-manager.sh`.

## Files Created/Modified

- `crates/z00z_wallets/src/db/redb_wallet_store.rs` - Stable facade and orchestration root for the split wallet-store surface.
- `crates/z00z_wallets/src/db/redb_wallet_store_tables.rs` - Payload types, table definitions, and semantic index update types.
- `crates/z00z_wallets/src/db/redb_wallet_store_codecs.rs` - Bincode, payload framing, seed decode, object-id, and bounded decode helpers.
- `crates/z00z_wallets/src/db/redb_wallet_store_migrations.rs` - Wallet KDF, AAD, HKDF-info, and index-format migration flows.
- `crates/z00z_wallets/src/db/redb_wallet_store_session.rs` - File locking, session lifetime, and steady-state `.wlt` flush ownership.
- `crates/z00z_wallets/src/db/redb_wallet_store_objects.rs` - Encrypted object writes and index manifest updates.
- `crates/z00z_wallets/src/db/redb_wallet_store_queries.rs` - Object and metadata read paths.
- `crates/z00z_wallets/src/db/redb_wallet_store_backup.rs` - Snapshot object persistence surface.
- `crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops.rs` - Store-side crypto operations, integrity updates, and seed reveal helpers.
- `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` - Canonical persisted-wallet crypto contract now also owns secret-AAD version markers.
- `crates/z00z_wallets/src/db/mod.rs` - Shallow crate-local re-exports for `WalletIdentity`, `RedbWltStore`, `WltStore`, and `Z00ZWalletIo`.
- `crates/z00z_wallets/src/services/app_service.rs` - Uses shallow `crate::db::WalletIdentity` import.
- `crates/z00z_wallets/src/services/wallet_paths.rs` - Uses shallow `crate::db::WalletIdentity` import.
- `crates/z00z_wallets/src/services/wallet_service.rs` - Uses shallow `crate::db` backend imports for in-crate store consumers.
- `crates/z00z_wallets/tests/test_wallet_persistence_no_std.rs` - Guards the full seam set for forbidden std boundary calls and locates integrity ownership without hardcoding one deep internal file.

## Decisions Made

- Keep the root wallet-store file as the compatibility facade even though it remains larger than the preferred end-state; this plan closes the highest-risk mixed responsibilities first.
- Treat flush ownership and object-record decode ownership as boundary concerns and move them to the session and codec seams respectively after review found drift.
- Treat broader service-layer dependence on RedB-specific helper flows as out of scope for `030-01` and defer it to later Phase 030 waves instead of widening this plan into a wallet-service refactor.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Boundary Ownership] Removed helper ownership drift after the initial seam split**

- **Found during:** Task 1 and Task 2 review passes
- **Issue:** Steady-state `.wlt` flush logic and object-record decode helpers ended up owned by migration and backup seams, while persisted secret-AAD version markers still lived in the root facade.
- **Fix:** Moved steady-state flush ownership into the session seam, moved bounded object-record decode into the codecs seam, and centralized secret-AAD version markers in `redb_wallet_crypto.rs`.
- **Files modified:** `crates/z00z_wallets/src/db/redb_wallet_crypto.rs`, `crates/z00z_wallets/src/db/redb_wallet_store.rs`, `crates/z00z_wallets/src/db/redb_wallet_store_backup.rs`, `crates/z00z_wallets/src/db/redb_wallet_store_codecs.rs`, `crates/z00z_wallets/src/db/redb_wallet_store_migrations.rs`, `crates/z00z_wallets/src/db/redb_wallet_store_session.rs`
- **Verification:** `cargo check -p z00z_wallets --lib --tests`, `cargo test -p z00z_wallets --release --test test_redb_wlt_open -- --nocapture`, `cargo test -p z00z_wallets --release --test test_wallet_kdf_migration -- --nocapture`
- **Committed in:** Pending consolidated version-manager commit

**2. [Rule 1 - Validation Drift] Restored persistence-boundary proof coverage after the split**

- **Found during:** Task 2 validation and review passes
- **Issue:** The source-shape guardrail was still tied either to the old root-only layout or to one deep internal seam, which no longer proved the full no-raw-std-I/O boundary across the split seam set.
- **Fix:** Updated `test_wallet_persistence_no_std.rs` to scan the full wallet-store seam set for forbidden std calls and to locate `update_wallet_integrity` by ownership rather than a single hardcoded file path.
- **Files modified:** `crates/z00z_wallets/tests/test_wallet_persistence_no_std.rs`
- **Verification:** `cargo test -p z00z_wallets --release --test test_wallet_persistence_no_std -- --nocapture`
- **Committed in:** Pending consolidated version-manager commit

**3. [Rule 2 - Facade Consistency] Removed shallow-caller drift for wallet identity and store backend imports**

- **Found during:** Task 2 review passes
- **Issue:** Service consumers still depended on `crate::db::wlt_store::*` for `WalletIdentity` and in-crate backend types even though `db/mod.rs` could provide a shallower crate-local surface.
- **Fix:** Added crate-local re-exports in `db/mod.rs` and switched service consumers to `crate::db::{...}` imports.
- **Files modified:** `crates/z00z_wallets/src/db/mod.rs`, `crates/z00z_wallets/src/services/app_service.rs`, `crates/z00z_wallets/src/services/wallet_paths.rs`, `crates/z00z_wallets/src/services/wallet_service.rs`
- **Verification:** `cargo check -p z00z_wallets --lib --tests`
- **Committed in:** Pending consolidated version-manager commit

---

**Total deviations:** 3 auto-fixed (2 boundary or validation fixes, 1 missing facade-consistency fix)
**Impact on plan:** All deviations were required to keep the split behavior-preserving and to make the verification contract match the post-split architecture. No unrelated scope was pulled into the plan.

## Issues Encountered

- The source-shape persistence test initially failed because it still asserted the pre-split root layout rather than the new seam-based ownership model.
- Codacy CLI activation succeeded, but the chat tool routing became unreliable during the final patch set, so static analysis evidence is strongest for the earlier edited files while the final validation relied on clean compile and targeted release tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The wallet-store seam split is stable enough for the next Phase 030 waves.
- Follow-up items that remain intentionally deferred for later long-file waves are recorded in `deferred-items.md`.

## Deferred Issues

- `wallet_service.rs` still reaches into RedB-specific helper and `.wlt` lifecycle flows beyond the stable persistence facade; this is deferred to the later wallet-service split waves in Phase 030.
- Some service-level tests still assert concrete backend details rather than only observable service behavior; that normalization is deferred with the wallet-service waves.

## Self-Check: PASSED

- Found `.planning/phases/030-refactor-long-files/030-01-SUMMARY.md`
- Found `.planning/phases/030-refactor-long-files/deferred-items.md`

---
*Phase: 030-refactor-long-files*
*Completed: 2026-03-31*
