---
phase: 028-crypto-audit-storage
plan: "05"
subsystem: storage
tags: [nullifier, replay, redb, root-mode, hardening]
requires:
  - phase: 028-crypto-audit-storage
    provides: truthful checkpoint semantics, explicit root binding, and typed checkpoint identity from plans 01-04
provides:
  - canonical binary nullifier replay keying owned by z00z_storage
  - explicit legacy parity decode for old text-key claim-nullifier rows during RedB reload
  - typed invalid root-mode failures instead of panic on storage default paths
  - explicit debug-gated RedB commit fault injection instead of ambient production-default env behavior
affects: [phase-028-closeout, scenario_1, redb, replay-defense, storage]
tech-stack:
  added: []
  patterns: [binary-nullifier-key, explicit-legacy-parity, typed-env-failure, debug-gated-fault-hook]
key-files:
  created: [.planning/phases/028-crypto-audit-storage/028-05-SUMMARY.md]
  modified:
    - crates/z00z_storage/src/assets/store.rs
    - crates/z00z_storage/src/assets/mod.rs
    - crates/z00z_storage/src/assets/store_internal/tx_plan.rs
    - crates/z00z_storage/src/assets/store_internal/redb_backend.rs
    - crates/z00z_simulator/src/claim_pkg_consumer.rs
    - crates/z00z_simulator/tests/test_claim_tx_pipeline.rs
    - crates/z00z_storage/tests/test_redb_rehydrate.rs
    - crates/z00z_storage/tests/test_redb_mutation.rs
key-decisions:
  - "Replay protection is keyed by a storage-owned ClaimNullifier byte wrapper, while chain_id remains metadata and validation context instead of key scope."
  - "Legacy text-key replay rows remain readable only through one explicit key-and-value parity path during RedB reload; new writes always use binary nullifier keys."
  - "Unsupported Z00Z_ASSET_ROOT_MODE values now return typed backend failures, and RedB commit fault injection requires an explicit debug acknowledgement env instead of ambient default-path activation."
patterns-established:
  - "Storage-owned canonical boundary: simulator converts upstream claim-package nullifier hex into ClaimNullifier before replay checks or store writes."
  - "Compatibility pattern: legacy replay rows are decoded through explicit V0 helper structs hidden behind serde renames, not through public canonical surface types."
requirements-completed: [PH28-NULLIFIER, PH28-TRUST-HOOK]
duration: multi-session
completed: 2026-03-30
---

# Phase 028 Plan 05: Binary Nullifier Closeout Summary

Claim replay defense now uses canonical binary nullifier state, legacy RedB replay rows are accepted only through one explicit parity path, and the remaining storage default-path hazards fail typed or stay behind a debug-only contract.

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-30T03:32:12Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Replaced storage-side replay keying based on lower-hex strings with a storage-owned `ClaimNullifier([u8; 32])` wrapper and switched in-memory replay maps plus RedB keys onto canonical bytes.
- Moved the simulator storage boundary onto canonical nullifier bytes by parsing claim-package hex at the consumer seam instead of letting stringly replay state leak into storage internals.
- Added an explicit legacy replay parity path so RedB reload accepts old text-key rows only when both key and row content resolve to the same canonical nullifier.
- Replaced `RootMode::load()` panics with typed `AssetStoreError::Backend(...)` failures and required an explicit debug acknowledgement env before RedB commit fault injection can trip the write path.
- Closed the phase on a green full release-style workspace gate plus the release `scenario_1` binary.

## Task Commits

1. **Task 1: Move replay protection onto canonical binary nullifier state without a crate-cycle** - not yet committed in this execution
2. **Task 2: Remove the remaining production-default hardening hazards and freeze the closeout gates** - not yet committed in this execution

**Plan metadata:** not yet committed in this execution.

## Files Created/Modified

- `crates/z00z_storage/src/assets/store.rs` - introduced `ClaimNullifier`, keyed replay state by canonical bytes, removed owner metadata from canonical replay rows, and made root-mode loading return typed errors.
- `crates/z00z_storage/src/assets/mod.rs` - exported the new storage-owned binary nullifier type.
- `crates/z00z_storage/src/assets/store_internal/tx_plan.rs` - propagated binary replay-key ownership through store snapshots and typed root-mode loading.
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` - wrote binary nullifier keys, decoded legacy text-key rows through one explicit compatibility path, and gated commit fault injection behind an explicit debug mode.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` - converted claim-package nullifier hex into `ClaimNullifier` before replay checks and storage commits.
- `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs` - updated storage replay assertions to query by canonical binary nullifier.
- `crates/z00z_storage/tests/test_redb_rehydrate.rs` - added parity coverage for legacy text-key replay rows under the new binary-key contract.
- `crates/z00z_storage/tests/test_redb_mutation.rs` - added invalid root-mode coverage and explicit debug-gate coverage for RedB commit injection.

## Decisions Made

- Canonical replay uniqueness is now global over validated nullifier bytes, not over normalized text formatting.
- `chain_id` remains recorded as metadata but does not participate in replay key scope because canonical nullifier derivation already binds it upstream.
- Legacy replay rows are a compatibility-only decode surface; the production write path no longer emits text-key replay rows.
- Storage default paths must not panic on unsupported root-mode env values or surprise the write path through an ambient injection env alone.

## Deviations from Plan

None. The plan executed as written.

## Issues Encountered

- `.planning/REQUIREMENTS.md` still lacks explicit `PH28-*` entries, so automatic requirement checkbox closure remains a planning follow-up rather than something this execution can truthfully update in place.

## User Setup Required

None.

## Next Phase Readiness

- Phase 028 is now complete: checkpoint proof semantics, execution transcript authenticity, root binding, typed checkpoint identity, canonical link binding, binary nullifier replay state, and local storage hardening hazards all closed on a green release-style workspace gate.
- The remaining follow-up is process-only: commit/version-manager closeout if desired.

## Validation Evidence

- ✅ `cargo test -p z00z_storage --release --test test_redb_mutation -- --nocapture`
- ✅ `cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture`
- ✅ `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage3_nullifier_store -- --nocapture`
- ✅ `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline -- --nocapture`
- ✅ `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture`
- ✅ `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture`
- ✅ `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
- ✅ `cargo test --release --features test-fast --features wallet_debug_dump` (exit `0`)
- ✅ Legacy text-key detector returned clean against canonical storage and simulator surfaces.

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/028-crypto-audit-storage/028-05-SUMMARY.md`
- ✅ Full workspace release gate completed with exit `0`
- ✅ Release `scenario_1` binary completed successfully
- ✅ No commit hashes are claimed yet in this execution

---

*Phase: 028-crypto-audit-storage*
*Completed: 2026-03-30*
