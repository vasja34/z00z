---
phase: 019-gaps-1
plan: 03
subsystem: wallet
tags: [backup, wallet-export-pack, rpc, restore, legacy-compatibility]
requires:
  - phase: 019-02
    provides: report-first receive contract and caller migration needed for final phase gate closure
provides:
  - WalletExportPack-backed active public backup contract
  - legacy BackupPayloadV1 readability on restore input
  - dispatcher-visible backup semantics aligned to the guaranteed restore set
affects: [phase-019-closeout, wallet-backups, rpc-backup, restore-contract]
tech-stack:
  added: []
  patterns: [WalletExportPack-backed backup contract, explicit guaranteed restore set, legacy-read-only fallback]
key-files:
  created: []
  modified:
    - crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs
    - crates/z00z_wallets/src/core/backup/backup_importer.rs
    - crates/z00z_wallets/src/core/backup/backup_importer_impl.rs
    - crates/z00z_wallets/src/core/wallet/snapshot.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/backup.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/backup_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/types/backup.rs
    - crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs
key-decisions:
  - "Converge the active public backup format on WalletExportPack instead of evolving a second full-restore schema."
  - "Keep BackupPayloadV1 readable only as legacy input and make the active-format restore guarantee explicit at service and RPC boundaries."
patterns-established:
  - "Active backup create and restore reuse the strongest existing wallet export/import contract."
  - "Public RPC wording and response surfaces describe only the guaranteed restore set, leaving journals and caches to post-restore rescan territory."
requirements-completed: [PH19-BACKUP]
duration: unknown
completed: 2026-03-25
---

# Phase 019 Plan 03: Backup Convergence Summary

WalletExportPack-backed public backup restore contract with legacy V1 readability and dispatcher-visible backup semantics.

## Performance

- **Duration:** unknown
- **Started:** 2026-03-25T16:11:40Z
- **Completed:** 2026-03-25T17:35:55Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Confirmed the active public backup path restores root secret material, wallet snapshot state, and versioned restore metadata through `WalletExportPack`.
- Preserved `BackupPayloadV1` readability as a legacy restore input instead of breaking existing backup files during migration.
- Fixed the legacy V1 restore contract so V1 input no longer reports success without restored wallet state and now fails explicitly as unsupported restore scope.
- Expanded dispatcher roundtrip coverage to exercise backup create, list, and restore on the public JSON-RPC transport path instead of treating non-backup RPC calls as sufficient proxy coverage.
- Revalidated backup create and restore behavior across direct backup impl tests, runtime response serialization, legacy V1 rejection coverage, dispatcher roundtrip coverage, and release-profile backup gates.

## Task Commits

Current branch history does not expose an isolated 019-03 task commit on top of `origin/z00z-simul`; this closure pass validated the active backup contract and completed the missing phase artifacts.

1. **Task 1: Introduce a WalletExportPack-backed public backup V2 with V1 fallback read path** - validated current branch state (no isolated local task commit)
2. **Task 2: Validate backup semantics, legacy compatibility, and final phase gates** - validated current branch state (no isolated local task commit)

**Plan metadata:** pending docs commit for summary and state reconciliation

## Files Created/Modified

- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs` - Emits the active backup payload on top of `WalletExportPack`.
- `crates/z00z_wallets/src/core/backup/backup_importer.rs` - Carries imported backup data including the converged restore payload.
- `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` - Reads the active format and keeps legacy V1 compatibility.
- `crates/z00z_wallets/src/core/wallet/snapshot.rs` - Defines the shared `WalletExportPack` restore contract.
- `crates/z00z_wallets/src/services/wallet_service.rs` - Aligns public backup creation and restoration to the shared export/import contract and rejects legacy V1 restore as unsupported instead of returning false success.
- `crates/z00z_wallets/src/adapters/rpc/methods/backup.rs` - Keeps the public JSON-RPC contract aligned to the guaranteed restore set.
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl.rs` - Exercises public backup create, list, and restore semantics.
- `crates/z00z_wallets/src/adapters/rpc/types/backup.rs` - Preserves explicit runtime restore response semantics for the active format.
- `crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs` - Confirms dispatcher-bound backup create, list, and restore calls preserve the same semantics as the direct implementation path.

## Decisions Made

- Reuse the existing encrypted wallet export/import path as the active public backup contract instead of drifting into a parallel schema.
- Keep legacy V1 readable and explicit about its narrower guarantees so operator-visible behavior does not over-promise restore breadth.

## Deviations from Plan

None - the final fix stayed inside the planned backup service and dispatcher verification boundary.

## Issues Encountered

- A later independent review invalidated the earlier assumption that the branch already satisfied the backup contract: legacy `BackupPayloadV1` restore could report success even though no `WalletExportPack` state was restored, and the dispatcher roundtrip test did not actually exercise backup RPC methods.
- The contract gap was resolved by making legacy V1 restore fail explicitly at the public service boundary and by extending dispatcher coverage to real backup create, list, and restore calls.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Backup create and restore semantics are validated on direct, runtime-response, dispatcher, legacy V1 rejection, and release-profile surfaces.
- The old `test_verify_backup_wrong_wallet` mismatch is resolved and now passes on the current branch state.
- Phase 019 phase-level closeout is still blocked by an unrelated compile failure in `crates/z00z_wallets/src/lib.rs`, so this summary should be treated as plan-local green with workspace-level closeout still pending.

## Self-Check: PASSED

- Found summary target: `.planning/phases/019-gaps-1/019-03-SUMMARY.md`
- Found debug validation: `test_export_import_wallet_payload`, `test_backup_create_list_restore`, `test_runtime_restore_backup_response`, `test_verify_backup_wrong_wallet`, `local_rpc_dispatcher_can_call`
- Found release validation: `legacy_v1_restore_fails`, `test_runtime_restore_backup_response`, `local_rpc_dispatcher_can_call`

---
_Phase: 019-gaps-1_
_Completed: 2026-03-25_
