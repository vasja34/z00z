---
phase: 029-crypto-audit-wallets
plan: "03"
subsystem: wallets
tags: [wallets, kdf, backup, migration, redb]
requires:
  - phase: 029-crypto-audit-wallets
    provides: 029-RECONCILIATION.md scope freeze and 029-02-SUMMARY.md live-view-key contract
provides:
  - RedB V2-shaped self-describing backup KDF metadata with explicit compatibility handling
  - persisted V1-to-V2 `.wlt` rewrite on accepted legacy unlock
  - reopen proof for rewritten wallet containers and in-session KDF continuity after migration
affects: [029-04, 029-05, 029-06]
tech-stack:
  added: []
  patterns: [versioned KDF contract, explicit compatibility path, rewrite-on-accept migration]
key-files:
  created:
    - .planning/phases/029-crypto-audit-wallets/029-03-SUMMARY.md
    - crates/z00z_wallets/tests/test_wallet_kdf_migration.rs
  modified:
    - crates/z00z_wallets/src/core/backup/mod.rs
    - crates/z00z_wallets/src/core/backup/wallet_backup.rs
    - crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs
    - crates/z00z_wallets/src/core/backup/backup_importer_impl.rs
    - crates/z00z_wallets/src/db/redb_wallet_store.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/tests/test_backup_kdf_contract.rs
    - crates/z00z_wallets/tests/test_redb_wlt_open.rs
    - crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs
key-decisions:
  - "Model new backup headers on RedB V2 `KdfParams` instead of keeping a parallel implicit backup KDF contract."
  - "Reject unknown backup KDF versions before expensive derivation or decrypt work starts."
  - "Treat accepted V1 wallet unlock as a persisted rewrite boundary and update the live session to the migrated V2 KDF immediately."
patterns-established:
  - "One governed KDF contract: backup export and RedB wallet persistence now describe KDF policy through explicit versioned metadata."
  - "Compatibility stays explicit: legacy reads remain bounded helper paths instead of ambient write-policy drift."
requirements-completed: [PH29-KDF, PH29-BACKUP]
duration: checkpointed
completed: 2026-03-30
---

# Phase 029 Plan 03: KDF Governance Summary

One explicit KDF contract now governs wallet persistence and backup export, and accepted legacy `.wlt` unlock rewrites persist to canonical V2 with reopen proof.

## Performance

- **Duration:** checkpointed
- **Started:** not recorded in resumed session
- **Completed:** 2026-03-30T12:00:00Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- 📌 Reworked backup export and import onto a self-describing RedB V2-shaped KDF header that records explicit algorithm, salt, Argon2 cost fields, and salt-padding semantics.
- 📌 Rejected unknown backup KDF versions before expensive derivation and kept legacy backup reads behind one explicit compatibility path.
- 📌 Turned legacy wallet KDF migration into a persisted rewrite-on-open flow and proved clean reopen from the rewritten `.wlt` container.
- 📌 Fixed the live session so in-memory KDF state matches the migrated on-disk V2 state immediately after accepted unlock.

## Task Commits

1. **Task 1: Make backup KDF semantics self-describing and aligned with the RedB V2 contract** - `df2f25b0` (feat)
2. **Task 2: Turn V1 wallet migration into a persisted rewrite with reopen proof** - `88122ce0` (feat)

**Plan metadata:** recorded in the final docs commit for Plan 03 closure

## Files Created Or Modified

- `crates/z00z_wallets/src/core/backup/mod.rs` - re-exported the backup KDF contract so backup code paths share one typed surface.
- `crates/z00z_wallets/src/core/backup/wallet_backup.rs` - defined the versioned backup KDF payload and explicit salt-padding semantics.
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs` - wrote self-describing backup headers aligned to RedB V2 `KdfParams`.
- `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` - enforced eager unknown-version rejection and explicit compatibility handling for legacy reads.
- `crates/z00z_wallets/src/db/redb_wallet_store.rs` - persisted legacy V1-to-V2 wallet migration and kept the active session on the migrated KDF contract.
- `crates/z00z_wallets/src/services/wallet_service.rs` - kept legacy backup fixture behavior aligned with the explicit compatibility contract.
- `crates/z00z_wallets/tests/test_backup_kdf_contract.rs` - added contract coverage for explicit backup KDF metadata, legacy compatibility, and unknown-version rejection.
- `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs` - preserved the wallet save or load or export or import roundtrip anchor under the new backup contract.
- `crates/z00z_wallets/tests/test_redb_wlt_open.rs` - extended the `.wlt` open regression to prove reopen after persisted migration.
- `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs` - added focused persisted rewrite and reopen coverage for legacy wallet KDF migration.

## Decisions Made

- 📌 New backup writes now persist explicit KDF metadata instead of relying on the old repeat-the-16-byte-salt rule as an implicit contract.
- 📌 The RedB V2 `KdfParams` shape is the baseline model for both backup governance and wallet persistence policy.
- 📌 Legacy compatibility remains allowed only through explicit read-time helpers; it no longer defines new-write behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Synced live session KDF state to the persisted migrated V2 contract**

- **Found during:** Task 2 review follow-up
- **Issue:** A legacy wallet could open and rewrite on disk, but the live session still kept pre-migration V1 `kdf_params`, which broke password-gated in-session operations until reopen.
- **Fix:** Changed `migrate_kdf_if_needed(...)` to return the active migrated `KdfParams` and updated `open_wlt_with_deps(...)` to replace the live session KDF state immediately after persisted rewrite.
- **Files modified:** `crates/z00z_wallets/src/db/redb_wallet_store.rs`, `crates/z00z_wallets/tests/test_redb_wlt_open.rs`, `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs`
- **Verification:** `cargo test -p z00z_wallets --release --test test_wallet_kdf_migration -- --nocapture`, `cargo test -p z00z_wallets --release --test test_redb_wlt_open -- --nocapture`, `cargo test -p z00z_wallets --release --test test_wallet_persistence_backup_service -- --nocapture`
- **Committed in:** `88122ce0`

---

**Total deviations:** 1 auto-fixed bug
**Impact on plan:** The fix stayed inside the plan-owned migration boundary and was required to make rewrite-on-accept transparent to operator-facing password flows.

## Deferred Issues

- 📌 No plan-owned deferred blocker remains in the backup identity path. The later Phase 029 review pass closed restore chain preservation by extending the encrypted backup identity contract to carry both `network` and `chain`.

## User Setup Required

None.

## Next Phase Readiness

- 📌 Phase 029 can proceed to the panic, seed-salt, and secret-boundary waves on top of one explicit KDF and backup contract.
- 📌 Legacy wallet unlock now proves one persisted rewrite plus one successful reopen under canonical V2 semantics.
- 📌 Backup governance no longer has an open chain-identity follow-up; the plan-owned KDF contract and migration gates remain closed under the later `v4` restore-identity hardening.

## Self-Check

PASSED

- Verified summary exists: `.planning/phases/029-crypto-audit-wallets/029-03-SUMMARY.md`
- Verified task commit exists: `df2f25b0`
- Verified task commit exists: `88122ce0`
- Verified created test exists: `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs`

---
*Phase: 029-crypto-audit-wallets*
*Completed: 2026-03-30*
