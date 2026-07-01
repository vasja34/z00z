---
phase: 028-crypto-audit-storage
plan: "04"
subsystem: storage
tags: [checkpoint, identity, link-binding, redb, compatibility]
requires:
  - phase: 028-crypto-audit-storage
    provides: authoritative replay bytes and explicit proof-root binding from plans 02 and 03
provides:
  - type-separated checkpoint draft, artifact, and exec-input identities
  - canonical checkpoint-link binding that rejects tuple tamper and legacy-artifact link persistence
  - fail-closed malformed-shell rejection on encode, id derivation, store load, and redb rehydrate paths
  - explicit redb compatibility rules for mixed checkpoint-id eras and legacy link-bearing bundles
affects: [028-05, checkpoint, redb, simulator, storage]
tech-stack:
  added: []
  patterns: [typed-checkpoint-id, statement-bound-link, fail-closed-compatibility-gate]
key-files:
  created: [.planning/phases/028-crypto-audit-storage/028-04-SUMMARY.md]
  modified:
    - crates/z00z_storage/src/assets/store_internal/redb_backend.rs
    - crates/z00z_storage/src/checkpoint/artifact.rs
    - crates/z00z_storage/src/checkpoint/codec.rs
    - crates/z00z_storage/src/checkpoint/ids.rs
    - crates/z00z_storage/src/checkpoint/store.rs
    - crates/z00z_storage/tests/test_checkpoint_finalization.rs
    - crates/z00z_storage/tests/test_checkpoint_ids.rs
    - crates/z00z_storage/tests/test_checkpoint_link_injective.rs
    - crates/z00z_storage/tests/test_checkpoint_store_api.rs
    - crates/z00z_storage/tests/test_redb_rehydrate.rs
key-decisions:
  - "Keep CheckpointId statement-based and proof-byte-independent while making malformed shells and unsupported proof systems fail closed before hashing or encoding."
  - "Require filesystem and RedB checkpoint links to match the persisted V1 checkpoint statement exactly instead of accepting loose tuple compatibility."
  - "Reject mixed checkpoint-id eras, missing snapshot rows, and legacy checkpoint bundles carrying persisted link metadata during RedB reload."
patterns-established:
  - "Checkpoint identity pattern: typed domain separation is enforced at derivation time, not inferred from raw byte shape."
  - "Checkpoint link pattern: canonical tuple binding is checked against the stored statement before link persistence or reload acceptance."
requirements-completed: [PH28-ID-BIND]
duration: multi-session
completed: 2026-03-30
---

# Phase 028 Plan 04: Typed Checkpoint Identity Summary

Type-separated checkpoint identities and canonical checkpoint-link binding now fail closed on malformed shells, tuple tamper, and mixed-era RedB reload state.

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-30T03:08:18Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added typed, domain-separated checkpoint identity derivation for draft, final artifact, and exec-input artifacts while keeping final `CheckpointId` bound to the attested statement rather than opaque proof bytes.
- Hardened encode, derive, filesystem-store, and RedB reload paths so malformed checkpoint shells, proofless artifacts, partial statement pairs, unsupported proof systems, and mixed compatibility eras fail closed.
- Bound persisted checkpoint links to the canonical V1 statement and rejected legacy checkpoint artifacts or legacy link-bearing bundles on both filesystem and RedB paths.
- Closed the full release-style workspace gate after aligning one outdated store API regression to the stricter link-binding invariant.

## Task Commits

1. **Task 1: Move checkpoint IDs onto one typed, domain-separated derivation helper** - not yet committed in this execution
2. **Task 2: Bind checkpoint links canonically and keep RedB compatibility explicit** - not yet committed in this execution

**Plan metadata:** not yet committed in this execution.

## Files Created/Modified

- `crates/z00z_storage/src/checkpoint/ids.rs` - moved artifact identities onto typed domain-separated derivation and added stricter compatibility checks before hashing.
- `crates/z00z_storage/src/checkpoint/codec.rs` - made artifact compatibility validation reusable and enforced it on encode paths.
- `crates/z00z_storage/src/checkpoint/artifact.rs` - exposed partial-statement-shell detection so malformed shells cannot collapse silently into legacy semantics.
- `crates/z00z_storage/src/checkpoint/store.rs` - required checkpoint-link tuple fields to match the persisted V1 statement and rejected legacy artifact link persistence.
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` - hardened reload validation for snapshot-row presence, mixed-era checkpoint ids, and legacy link-bearing bundles.
- `crates/z00z_storage/tests/test_checkpoint_ids.rs` - added fail-closed regressions for malformed attest shells, proofless shells, partial statement pairs, and typed identity separation.
- `crates/z00z_storage/tests/test_checkpoint_link_injective.rs` - pinned canonical link persistence and legacy-artifact rejection semantics.
- `crates/z00z_storage/tests/test_redb_rehydrate.rs` - added RedB reload regressions for mixed checkpoint-id eras, missing snapshot rows, and legacy bundle rejection.
- `crates/z00z_storage/tests/test_checkpoint_finalization.rs` - updated the final-id contract to assert proof-byte deltas do not change statement-based checkpoint identity.
- `crates/z00z_storage/tests/test_checkpoint_store_api.rs` - aligned the roundtrip surface test to the stricter link-binding invariant by using a proof and link tuple that match the saved exec input.

## Decisions Made

- `CheckpointId` remains statement-based and intentionally ignores opaque proof-byte deltas when the attested statement is unchanged.
- Legacy link bytes stay supported only through explicit decode-upgrade paths; persisted new-era link semantics must match the checkpoint statement exactly.
- RedB compatibility is now judged at bundle level, not only by isolated row decode success.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Encode and id-derivation paths still allowed malformed checkpoint shells that load paths would later reject**

- **Found during:** Task 1 review and targeted verification
- **Issue:** malformed attest shells, proofless artifacts, and partial `(prep_snapshot_id, exec_input_id)` pairs could still reach canonical encode or hash paths.
- **Fix:** enforced compatibility checks during encode and id derivation, added explicit partial-shell detection, and pinned the behavior in checkpoint-id regressions.
- **Files modified:** `crates/z00z_storage/src/checkpoint/artifact.rs`, `crates/z00z_storage/src/checkpoint/codec.rs`, `crates/z00z_storage/src/checkpoint/ids.rs`, `crates/z00z_storage/tests/test_checkpoint_ids.rs`
- **Verification:** targeted release tests for `test_checkpoint_ids` and the full rerun workspace gate.
- **Committed in:** not yet committed in this execution

**2. [Rule 2 - Missing Critical] Filesystem and RedB paths still accepted legacy or mixed-era link state too loosely**

- **Found during:** Task 2 review and reload-hardening pass
- **Issue:** store and RedB validation checked artifact rows in isolation but still needed explicit statement-bound link checks, missing snapshot-row validation, and legacy bundle rejection.
- **Fix:** required checkpoint-link tuple equality against the stored V1 statement, rejected bound links on legacy checkpoint artifacts, validated snapshot-row presence on reload, and rejected legacy checkpoint bundles carrying persisted link metadata.
- **Files modified:** `crates/z00z_storage/src/checkpoint/store.rs`, `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`, `crates/z00z_storage/tests/test_checkpoint_link_injective.rs`, `crates/z00z_storage/tests/test_redb_rehydrate.rs`
- **Verification:** targeted release tests for `test_checkpoint_link_injective`, `test_redb_rehydrate`, and the full rerun workspace gate.
- **Committed in:** not yet committed in this execution

**3. [Rule 1 - Bug] One store API regression still built a link that no longer matched the stricter checkpoint statement binding**

- **Found during:** final full workspace release rerun
- **Issue:** `test_file_store_keeps_surfaces_separate` finalized an artifact with one exec identity and then attempted to persist a link built from a different saved exec-input id, which the stricter store invariant now correctly rejects as `LinkMix`.
- **Fix:** rebuilt the test artifact proof from the saved exec input and used the same prep-snapshot and exec ids for the persisted link roundtrip.
- **Files modified:** `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
- **Verification:** `cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture` and the green rerun `cargo test --release --features test-fast --features wallet_debug_dump`.
- **Committed in:** not yet committed in this execution

---

**Total deviations:** 3 auto-fixed (2 missing critical, 1 bug)
**Impact on plan:** all deviations were required to make typed identity, canonical link binding, and RedB compatibility rules internally consistent across encode, store, reload, and workspace-level regression surfaces.

## Issues Encountered

- The initial full workspace gate surfaced one stale storage regression after all targeted checkpoint suites were already green.
- `.planning/REQUIREMENTS.md` still lacks explicit `PH28-*` entries, so requirement closeout automation is expected to need manual follow-up or to fail until the phase requirement inventory is added.

## User Setup Required

None.

## Next Phase Readiness

- Phase 028 now has explicit typed checkpoint identity and link-binding contracts, so `028-05` can build canonical binary nullifier storage without inheriting mixed-era checkpoint ambiguity.
- The full release-style workspace gate is green after the final storage test alignment, so plan 04 is ready for state and roadmap closeout.

## Validation Evidence

- ✅ `cargo test -p z00z_storage --release --test test_checkpoint_ids -- --nocapture`
- ✅ `cargo test -p z00z_storage --release --test test_checkpoint_link_injective -- --nocapture`
- ✅ `cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture`
- ✅ `cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture`
- ✅ `cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture`
- ✅ `cargo test --release --features test-fast --features wallet_debug_dump` (rerun exit `0`)
- ✅ Two consecutive clean scoped review passes were obtained before the final workspace rerun.

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/028-crypto-audit-storage/028-04-SUMMARY.md`
- ✅ Final rerun workspace gate recorded green with exit `0`
- ✅ No commit hashes are claimed yet in this execution

---

*Phase: 028-crypto-audit-storage*
*Completed: 2026-03-30*
