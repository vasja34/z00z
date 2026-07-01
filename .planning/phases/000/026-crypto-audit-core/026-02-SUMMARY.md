---
phase: 026-crypto-audit-core
plan: "02"
subsystem: core
tags: [rust, assets, registry, snapshot, integrity, concurrency]
requires:
  - phase: 026-crypto-audit-core
    provides: canonical asset-definition identity and validated definition payloads
provides:
  - full-payload registry snapshot hashing over canonical DefinitionWire framing
  - snapshot emit and apply paths bound to one ordered payload-plus-version contract
  - version-safe registry publication across local inserts and concurrent snapshot updates
affects: [026-03, 026-04, 026-05, z00z_core-assets, z00z_core-genesis]
tech-stack:
  added: []
  patterns: [canonical-wire-payload-hash, version-bound-snapshot-digest, atomic-registry-publication]
key-files:
  created:
    - .planning/phases/026-crypto-audit-core/026-02-SUMMARY.md
  modified:
    - crates/z00z_core/src/assets/wire.rs
    - crates/z00z_core/src/assets/snapshot.rs
    - crates/z00z_core/src/assets/registry.rs
    - crates/z00z_core/tests/assets/test_integration_assets_test12.rs
key-decisions:
  - "Hash registry snapshots over ordered canonical DefinitionWire payload bytes plus snapshot version, not only definition ids."
  - "Keep snapshot export read-only and make version monotonicity explicit on every accepted local or remote registry state change."
  - "Recheck downgrade prevention under commit locks so concurrent writers cannot publish an older accepted snapshot after a newer one wins."
patterns-established:
  - "Registry integrity follows canonical payload framing: emit, verify, and replay tests must reuse the same bytes."
  - "Versioned state publication is atomic: definitions and version advance together, and local inserts participate in the same monotonic contract as snapshot apply."
requirements-completed: [PH26-REGISTRY]
duration: multi-session
completed: 2026-03-28
---

# Phase 026 Plan 02 Summary

📌 **Registry snapshots now bind versioned full DefinitionWire payloads, reject stable-id payload tampering, and publish atomically under concurrent updates**

## Performance

- **Duration:** multi-session
- **Started:** 2026-03-28T09:03:21Z
- **Completed:** 2026-03-28T12:30:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- ✅ Replaced id-only registry hashing with canonical ordered `DefinitionWire` payload hashing and bound the snapshot version into the digest.
- ✅ Unified snapshot create and snapshot apply around the same hash contract, including duplicate-id rejection and stable-id payload-tamper rejection.
- ✅ Hardened registry publication so local inserts bump version, snapshot export stays read-only, and concurrent snapshot writers cannot overwrite a newer committed version.

## Task Commits

📌 This execution closed the plan from validated working-tree state.

1. **Task 1: Redefine the registry hash over canonical definition payloads** - not separately committed in this execution
2. **Task 2: Rewire snapshot creation and update to the new hash contract** - not separately committed in this execution

**Plan metadata:** not committed in this execution; repo-owned git/versioning checkpoint remains deferred.

## Files Created/Modified

- `crates/z00z_core/src/assets/wire.rs` - canonical payload framing helper for registry hashing
- `crates/z00z_core/src/assets/snapshot.rs` - version-bound full-payload registry digest contract and payload-drift tests
- `crates/z00z_core/src/assets/registry.rs` - read-only snapshot export, duplicate-id rejection, version-safe local inserts, and atomic snapshot apply
- `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` - concurrent updater and export/apply roundtrip coverage against the new hash contract
- `.planning/phases/026-crypto-audit-core/026-02-SUMMARY.md` - phase execution summary and verification record

## Decisions Made

- 📌 `RegistryVersion::compute_hash(...)` now consumes `version + ordered DefinitionWire payload bytes`, so registry integrity changes when canonical payload content changes even if ids stay stable.
- 📌 `create_snapshot()` now exports the current registry state without mutating it, and local insert paths advance the stored version so export and apply obey one monotonicity rule.
- 📌 `update_from_snapshot()` now validates hash, duplicate ids, and downgrade rules before commit and then rechecks version under commit locks to prevent concurrent stale publication.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Snapshot acceptance still permitted duplicate definition ids before canonical verification**

- **Found during:** Task 2 review and concurrency follow-up
- **Issue:** Rebuilding the registry from a keyed map could collapse repeated ids and leave part of the incoming payload list non-authoritative.
- **Fix:** Sorted the incoming wire list, rejected duplicate ids before registry rebuild, and added a dedicated regression test.
- **Files modified:** `crates/z00z_core/src/assets/registry.rs`
- **Verification:** `cargo test -p z00z_core --test assets_tests -- --nocapture`
- **Committed in:** not committed in this execution

**2. [Rule 1 - Bug] Snapshot export advanced registry version on read-only export**

- **Found during:** Task 2 review
- **Issue:** Exporting a snapshot mutated local version state, which broke emit/apply symmetry and made repeated exports produce artificial version drift.
- **Fix:** Kept snapshot export read-only and exported the current committed version verbatim.
- **Files modified:** `crates/z00z_core/src/assets/registry.rs`
- **Verification:** `cargo test -p z00z_core --test assets_tests -- --nocapture`
- **Committed in:** not committed in this execution

**3. [Rule 2 - Missing Critical] Local registry mutations did not advance version monotonicity**

- **Found during:** Task 2 review and concurrent snapshot testing
- **Issue:** Registries built through `insert()` or `insert_batch()` could change content without changing version, making later snapshot replication ambiguous.
- **Fix:** Added explicit version bumps for accepted local inserts and batch inserts and updated metrics accordingly.
- **Files modified:** `crates/z00z_core/src/assets/registry.rs`
- **Verification:** `cargo test -p z00z_core --test assets_tests -- --nocapture`; `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **Committed in:** not committed in this execution

**4. [Rule 1 - Bug] Concurrent snapshot writers could still publish an older accepted version after a newer one won**

- **Found during:** Task 2 review and strengthened integration testing
- **Issue:** Pre-commit downgrade checks alone were insufficient under concurrent writers because a later commit section could still race after validation.
- **Fix:** Rechecked monotonicity inside the commit section under the definitions-plus-version lock order and added a latest-version concurrency regression test.
- **Files modified:** `crates/z00z_core/src/assets/registry.rs`, `crates/z00z_core/tests/assets/test_integration_assets_test12.rs`
- **Verification:** `cargo test -p z00z_core --test assets_tests -- --nocapture`
- **Committed in:** not committed in this execution

**5. [Rule 1 - Bug] The new registry digest initially did not bind version metadata**

- **Found during:** Final review cycle
- **Issue:** A payload-only digest left snapshot version metadata outside the integrity contract.
- **Fix:** Included snapshot version bytes in `RegistryVersion::compute_hash(...)` and updated all call sites and regression tests.
- **Files modified:** `crates/z00z_core/src/assets/snapshot.rs`, `crates/z00z_core/src/assets/registry.rs`, `crates/z00z_core/tests/assets/test_integration_assets_test12.rs`
- **Verification:** `cargo test -p z00z_core --test assets_tests -- --nocapture`
- **Committed in:** not committed in this execution

**6. [Rule 1 - Bug] Version-zero handling was too permissive for non-empty snapshots**

- **Found during:** Final review cycle
- **Issue:** The empty-registry roundtrip case and arbitrary version-zero payload snapshots were treated too similarly.
- **Fix:** Kept the empty roundtrip path for version `0` but rejected non-empty version-zero snapshots explicitly.
- **Files modified:** `crates/z00z_core/src/assets/registry.rs`
- **Verification:** `cargo test -p z00z_core --test assets_tests -- --nocapture`
- **Committed in:** not committed in this execution

---

**Total deviations:** 6 auto-fixed (5 bug, 1 missing critical)
**Impact on plan:** All deviations were required to make `PH26-REGISTRY` authoritative under realistic concurrency and replay-style mutation scenarios. Scope stayed inside the registry and snapshot boundary.

## Issues Encountered

- ⚠️ Review-driven hardening surfaced additional monotonicity and duplicate-handling gaps after the first green test pass; the plan stayed open until those deeper state-publication issues were fixed.
- ⚠️ Codacy MCP analyze tools were not available in this environment during closure, so no Codacy post-edit scan could be completed for the final state.
- ⚠️ Snapshot acceptance is still integrity-only rather than authenticity-bound. That provenance gap remains outside the local `PH26-REGISTRY` scope and should be treated as a follow-up security slice.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ✅ `PH26-REGISTRY` is now closed with emit/apply symmetry, stable-id tamper rejection, duplicate-id rejection, and concurrency coverage.
- ✅ `026-04` can reuse the same canonical `DefinitionWire` framing and validated conversion expectations for untrusted wire boundaries.
- ✅ `026-05` inherits a stricter registry version contract for any asset-policy checks that depend on synchronized registry state.
- ⚠️ `026-03` remains independent but should keep the same fail-closed discipline when it touches genesis-owned registry or definition publication paths.

## Validation Evidence

- ✅ `cargo test -p z00z_core --test assets_tests -- --nocapture` -> `254 passed; 0 failed; 7 ignored`
- ✅ `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` -> passed for the scoped bootstrap suite used during closure

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/026-crypto-audit-core/026-02-SUMMARY.md`
- ✅ `PH26-REGISTRY` closure evidence recorded against the final tested working tree
- ✅ No commit hashes were claimed because git checkpointing was not performed in this execution

---

*Phase: 026-crypto-audit-core*
*Completed: 2026-03-28*
