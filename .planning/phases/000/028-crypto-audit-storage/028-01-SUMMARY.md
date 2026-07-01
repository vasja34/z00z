---
phase: 028-crypto-audit-storage
plan: "01"
subsystem: storage
tags: [checkpoint, storage, attestation, simulator]
requires: []
provides:
  - truthful opaque checkpoint attestation statements
  - explicit legacy opaque compatibility reads
  - store sealing gated on snapshot and exec-bound attestation statements
affects: [checkpoint, simulator, stage8, phase-028]
tech-stack:
  added: []
  patterns: [legacy-opaque-compat, statement-bound-attestation, explicit-store-trust-boundary]
key-files:
  created: [.planning/phases/028-crypto-audit-storage/028-01-SUMMARY.md]
  modified:
    - crates/z00z_storage/src/checkpoint/artifact.rs
    - crates/z00z_storage/src/checkpoint/codec.rs
    - crates/z00z_storage/src/checkpoint/store.rs
    - crates/z00z_storage/src/assets/store_internal/redb_backend.rs
    - crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs
    - crates/z00z_simulator/src/scenario_1/stage_12.rs
key-decisions:
  - "Treat legacy opaque checkpoint bytes as compatibility-only artifacts and keep new truthful payloads on a versioned attestation statement."
  - "Require the store seal path to accept only snapshot and exec-bound opaque attestations, not detached legacy opaque payloads."
patterns-established:
  - "Checkpoint statement pattern: bind storage semantics and replay ids explicitly before sealing opaque bytes."
  - "Compatibility-read pattern: load legacy bytes explicitly, but never silently reuse them as stronger verified semantics."
requirements-completed: [PH28-CHK-PROOF, PH28-TRUST-HOOK]
duration: current-session
completed: 2026-03-29
---

# Phase 028 Plan 01: Honest Checkpoint Attestations Summary

Versioned opaque checkpoint attestations with explicit legacy compatibility reads and store sealing that binds snapshot and exec ids.

## Performance

- **Duration:** current-session
- **Started:** 2026-03-29T22:45:38Z
- **Completed:** 2026-03-29T23:44:33Z
- **Tasks:** 2
- **Files modified:** 16

## Accomplishments

- Added `CheckpointStmtV1` and explicit `CheckpointStatement` semantics so new opaque checkpoint payloads bind truthful checkpoint state plus replay ids.
- Preserved backward reads for legacy opaque checkpoint bytes through explicit compatibility handling instead of silent reinterpretation.
- Hardened the store and RedB sealing path so production-facing checkpoint persistence only accepts snapshot and exec-bound attestation statements.
- Aligned simulator and storage integration tests with the new truthful checkpoint contract.

## Task Commits

1. **Task 1 + Task 2: shared checkpoint semantics and sealing surface** - `b7b9315b` (feat)

## Files Created/Modified

- `crates/z00z_storage/src/checkpoint/artifact.rs` - Added versioned attestation statements, proof-system split, and truthful artifact finalization behavior.
- `crates/z00z_storage/src/checkpoint/codec.rs` - Added explicit legacy opaque decode fallback and compatibility validation.
- `crates/z00z_storage/src/checkpoint/store.rs` - Rejected detached legacy opaque payloads on the seal path unless snapshot and exec ids are bound.
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` - Switched durable checkpoint writes onto the attestation constructor.
- `crates/z00z_storage/src/checkpoint/build.rs` - Documented the external verifier and spent-index trust boundary explicitly.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` - Bound scenario checkpoint proofs to snapshot and exec ids.
- `crates/z00z_simulator/src/scenario_1/stage_12.rs` - Sealed stage 12 checkpoints through the attestation path.
- `crates/z00z_storage/tests/test_checkpoint_finalization.rs` - Covered truthful attestation behavior and legacy opaque compatibility reads.
- `crates/z00z_storage/tests/test_checkpoint_store_api.rs` - Covered explicit seal-path rejection of legacy opaque payloads.
- `crates/z00z_storage/tests/test_checkpoint_link_injective.rs` - Updated link injectivity tests to use statement-bound attestations.

## Decisions Made

- Reused the existing checkpoint artifact shape for new attestation writes and added replay-id fields at the tail, while explicit codec fallback keeps legacy opaque bytes readable.
- Kept `CheckpointProof::new(...)` as a compatibility constructor for legacy opaque payloads, but moved truthful production-facing writes onto `CheckpointProof::new_attest(...)`.
- Fixed simulator and checkpoint-id regression tests instead of weakening the new store seal policy.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added explicit legacy artifact decode fallback**

- **Found during:** Task 1
- **Issue:** Old opaque checkpoint bytes failed to deserialize once attestation-only replay-id fields were added to the artifact shell.
- **Fix:** Added explicit legacy decode fallback in the checkpoint codec and regression coverage for preserved legacy opaque reads.
- **Files modified:** `crates/z00z_storage/src/checkpoint/codec.rs`, `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
- **Verification:** `cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture`
- **Committed in:** `b7b9315b`

**2. [Rule 1 - Bug] Updated runtime and regression callsites that still built detached legacy opaque proofs**

- **Found during:** Task 2 verification
- **Issue:** The release-style gate exposed simulator stage 12 and checkpoint-link tests that still used the legacy opaque constructor, which the new store seal path now correctly rejects.
- **Fix:** Switched those callsites to `CheckpointProof::new_attest(...)` with bound snapshot and exec ids, and aligned checkpoint-id shell fixtures with the current artifact layout.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`, `crates/z00z_simulator/src/scenario_1/stage_12.rs`, `crates/z00z_storage/tests/test_checkpoint_link_injective.rs`, `crates/z00z_storage/tests/test_checkpoint_ids.rs`, `crates/z00z_storage/src/checkpoint/ids.rs`
- **Verification:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture` and `cargo test --release --features test-fast --features wallet_debug_dump`
- **Committed in:** `b7b9315b`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both fixes were required to make the new truthful checkpoint contract viable across existing runtime and compatibility surfaces. No architectural scope change was introduced.

## Issues Encountered

- The initial compatibility approach assumed trailing optional fields would deserialize from old bincode bytes; explicit legacy codec fallback was required instead.
- The release-style workspace gate exposed remaining runtime and integration-test callsites still using detached legacy opaque proofs.
- `requirements mark-complete` could not update `PH28-CHK-PROOF` and `PH28-TRUST-HOOK` because those ids are referenced by the plan but not yet present in `.planning/REQUIREMENTS.md`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 028 now starts from an honest checkpoint artifact contract and an explicit store trust boundary.
- Wave 2 can focus on authoritative execution transcripts without carrying forward legacy opaque semantics drift.

## Self-Check: PASSED

- Found summary file: `.planning/phases/028-crypto-audit-storage/028-01-SUMMARY.md`
- Verified code commit: `b7b9315b`

---
*Phase: 028-crypto-audit-storage*
*Completed: 2026-03-29*
