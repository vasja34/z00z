---
phase: 028-crypto-audit-storage
plan: "03"
subsystem: storage
tags: [proof, binding, storage, snapshot, checkpoint]
requires:
  - phase: 028-crypto-audit-storage
    provides: canonical replay bytes preserve truthful tx-proof payloads and generic store commits stop fabricating canonical exec artifacts
provides:
  - explicit versioned semantic-root and backend-root binding in ProofBlob
  - fail-closed witness verification before branch proof acceptance
  - legacy proof-blob decode uplift for pre-binding persisted storage witnesses
  - store-generated proof emitters and claim-source consumers aligned to one binding contract
affects: [checkpoint, snapshot, storage, claim-proof, phase-028]
tech-stack:
  added: []
  patterns: [versioned-root-binding, fail-closed-witness-check, legacy-proof-uplift]
key-files:
  created: [.planning/phases/028-crypto-audit-storage/028-03-SUMMARY.md]
  modified:
    - crates/z00z_storage/src/assets/proof.rs
    - crates/z00z_storage/src/assets/store_internal/proof_help.rs
    - crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs
    - crates/z00z_storage/src/assets/store_internal/test_whitebox_state.rs
    - crates/z00z_storage/src/snapshot/store.rs
    - crates/z00z_storage/tests/test_checkpoint_root_binding.rs
    - crates/z00z_storage/tests/test_claim_source_proof.rs
key-decisions:
  - "Require every decoded ProofBlob to validate one explicit versioned semantic-root/backend-root commitment before branch proof acceptance."
  - "Preserve upgrade compatibility by uplifting legacy pre-binding witness bytes into the new binding contract at decode time instead of silently accepting missing bindings."
patterns-established:
  - "Witness binding pattern: semantic root and backend root are joined by one domain-separated commitment carried inside ProofBlob."
  - "Compatibility pattern: legacy storage-owned witness bytes decode through one explicit uplift path, but freshly decoded blobs always operate on the versioned binding contract."
requirements-completed: [PH28-ROOT-BIND]
duration: current-session
completed: 2026-03-30
---

# Phase 028 Plan 03: Proof Root Binding Summary

Versioned proof blobs now bind the semantic storage root to the backend JMT root explicitly, fail closed on cross-root tampering, and uplift legacy persisted witness bytes into the new contract.

## Performance

- **Duration:** current-session
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added explicit `root_bind_ver` and `root_bind` fields to `ProofBlob` and verified the binding before any definition, serial, or asset branch proof is accepted.
- Extended storage-owned proof coverage with root-binding tamper, version-mix, and legacy decode-upgrade tests.
- Kept store-generated proof blobs and claim-source proof consumers compatible under the new format by emitting and asserting the binding contract consistently.
- Mapped root-binding failures into snapshot witness validation so cross-root mismatches remain a hard storage-boundary failure.
- Re-ran the wave 3 automated verify sequence and finished with two consecutive clean review passes after fixing the only significant compatibility issue.

## Task Commits

1. **Task 1: Introduce a versioned root-binding field in `ProofBlob` and verify it before branch proofs** - `2e9e4449` (feat)
2. **Task 2: Re-emit proof blobs with the new root-binding field and keep proof consumers compatible** - `0ec05a14` (feat)

## Files Created/Modified

- `crates/z00z_storage/src/assets/proof.rs` - Added versioned root binding, fail-closed verification, and legacy witness decode uplift.
- `crates/z00z_storage/src/assets/store_internal/proof_help.rs` - Kept store proof emission anchored on `ProofBlob::new(...)` as the canonical binding seam.
- `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` - Added codec, tamper, version-mix, and legacy-upgrade coverage for proof blobs.
- `crates/z00z_storage/src/assets/store_internal/test_whitebox_state.rs` - Added parity coverage proving store-generated blobs emit the explicit binding field.
- `crates/z00z_storage/src/snapshot/store.rs` - Treated binding failures as root-boundary witness failures during snapshot validation.
- `crates/z00z_storage/tests/test_checkpoint_root_binding.rs` - Added integration coverage for rejecting tampered proof root bindings before branch acceptance.
- `crates/z00z_storage/tests/test_claim_source_proof.rs` - Added consumer coverage asserting emitted claim-source proof blobs carry the versioned binding.

## Decisions Made

- Freshly decoded proof blobs now require the explicit versioned root-binding contract instead of relying on an implicit semantic/backend root relationship.
- Legacy pre-binding proof bytes remain readable only through one explicit decode-upgrade path that computes the new binding immediately.
- Snapshot witness validation treats binding mismatches as root-boundary failures, not as late branch-proof errors.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added legacy witness decode-upgrade for pre-binding proof blobs**

- **Found during:** Task 1 review gate
- **Issue:** The initial root-binding implementation made the new binding mandatory for every decoded blob, which would have broken persisted pre-wave-3 witness bytes during snapshot reload or claim-proof decoding.
- **Fix:** Added an explicit legacy `ProofBlobV0` decode path that uplifts old witness bytes into the versioned root-binding contract, then added whitebox coverage for that upgrade path.
- **Files modified:** `crates/z00z_storage/src/assets/proof.rs`, `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs`
- **Verification:** Re-ran bootstrap, targeted release tests for checkpoint root binding, claim-source proof, proof blob codec/replay/root parity, full release workspace gate, and two consecutive clean review passes.
- **Committed in:** `2e9e4449`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The fix kept the stronger fail-closed contract while making the format transition explicit instead of silently breaking persisted storage witnesses.

## Issues Encountered

- The first review pass surfaced a real upgrade-compatibility gap: current-format binding checks were correct for new blobs but incomplete for previously persisted witness bytes.
- `requirements mark-complete` is still expected to fail for phase 028 ids because the mapped `PH28-*` entries remain absent from `.planning/REQUIREMENTS.md`.

## User Setup Required

None.

## Next Phase Readiness

- Phase 028 can now build the next binding-hardening wave on one explicit proof contract: producers, consumers, snapshot validation, and claim-source export all agree on the versioned semantic/backend root commitment.
- Wave 4 can extend checkpoint and identifier trust logic without inheriting an implicit or format-ambiguous proof root relationship.

## Self-Check: PASSED

- Found summary file: `.planning/phases/028-crypto-audit-storage/028-03-SUMMARY.md`
- Verified task commit: `2e9e4449`
- Verified task commit: `0ec05a14`

---
*Phase: 028-crypto-audit-storage*
*Completed: 2026-03-30*
