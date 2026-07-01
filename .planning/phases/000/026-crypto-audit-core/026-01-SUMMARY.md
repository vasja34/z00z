---
phase: 026-crypto-audit-core
plan: "01"
subsystem: core
tags: [rust, assets, registry, genesis, config, canonical-id]
requires:
  - phase: 025-crypto-audit-crypto
    provides: fail-closed crypto review discipline and narrowed production-facing trust seams
provides:
  - canonical framed asset-definition identity derivation
  - config and registry acceptance paths that validate canonical definition payloads
  - checked versus prechecked construction split for deferred genesis-runtime hardening
affects: [026-02, 026-03, 026-04, 026-05, z00z_core-assets, z00z_core-genesis]
tech-stack:
  added: []
  patterns: [canonical-framed-identity, checked-prechecked-seams, fail-closed-config-parsing]
key-files:
  created:
    - .planning/phases/026-crypto-audit-core/026-01-SUMMARY.md
  modified:
    - crates/z00z_core/src/assets/definition.rs
    - crates/z00z_core/src/assets/assets_config.rs
    - crates/z00z_core/src/assets/registry.rs
    - crates/z00z_core/src/assets/assets.rs
    - crates/z00z_core/src/genesis/genesis.rs
    - crates/z00z_core/src/genesis/asset_std.rs
    - crates/z00z_core/src/genesis/genesis_config.rs
key-decisions:
  - "Centralize asset-definition identity in one framed AssetIdHasher-based seam and stop trusting caller-supplied ids on production paths."
  - "Keep the network-aware genesis compatibility seam local to runtime genesis until plan 03 instead of leaking it through public helpers."
  - "Harden config and registry acceptance paths to fail closed on malformed metadata, malformed policy flags, non-canonical definitions, and reserved policy bits."
patterns-established:
  - "Canonical identity first: validate or derive authoritative ids before insert, rehydrate, or public asset construction."
  - "Trusted seams must be explicit and narrow: use checked public helpers and local prechecked runtime-only seams when later plans still own a compatibility path."
requirements-completed: [PH26-ASSET-ID]
duration: multi-session
completed: 2026-03-28
---

# Phase 026 Plan 01 Summary

📌 **Canonical framed asset-definition identity with fail-closed config, registry, and public asset construction paths in `z00z_core`**

## Performance

- **Duration:** multi-session
- **Started:** 2026-03-28T06:29:24Z
- **Completed:** 2026-03-28T08:59:23Z
- **Tasks:** 2
- **Files modified:** 19

## Accomplishments

- ✅ Centralized `AssetDefinition` identity on one framed `AssetIdHasher` seam that covers class, names, numeric policy fields, versions, flags, and ordered metadata.
- ✅ Routed config loading, registry insertion, snapshot rehydration, public asset construction, and public genesis helper paths through canonical validation instead of trusting caller-controlled ids.
- ✅ Kept the deferred runtime genesis compatibility seam local to `genesis.rs` while hardening public helper surfaces and fail-closing metadata or policy parsing.

## Task Commits

📌 This execution closed the plan from validated working-tree state.

1. **Task 1: Create one canonical asset-definition identity seam** - not separately committed in this execution
2. **Task 2: Route config and trusted construction paths through the canonical identity seam** - not separately committed in this execution

**Plan metadata:** not committed in this execution; repo-owned git/versioning checkpoint remains deferred.

## Files Created/Modified

- `crates/z00z_core/src/assets/definition.rs` - canonical framed id derivation, field validation split, canonical-id verification, and new regression tests
- `crates/z00z_core/src/assets/assets_config.rs` - canonical config-driven id derivation, fail-closed metadata parsing, and stricter policy flag parsing
- `crates/z00z_core/src/assets/registry.rs` - validated insert and snapshot rehydration plus a narrow `insert_prechecked` seam for deferred runtime genesis
- `crates/z00z_core/src/assets/assets.rs` - checked public asset construction and authoritative definition validation inside asset validation
- `crates/z00z_core/src/genesis/genesis.rs` - localized prechecked runtime genesis seam plus checked public generation paths
- `crates/z00z_core/src/genesis/asset_std.rs` - canonical public helper construction so public dev helpers no longer leak the genesis compatibility seam
- `crates/z00z_core/src/genesis/genesis_config.rs` - aligned fail-closed policy or metadata parsing and explicit policy-flag semantics
- `crates/z00z_core/tests/assets/test_registry_integration.rs` - registry tests updated for canonical ids and validated acceptance behavior
- `crates/z00z_core/tests/genesis/test_cross_network_isolation.rs` - genesis test fixtures aligned with expanded policy config defaults

## Decisions Made

- 📌 `AssetDefinition::new(...)` now derives the authoritative id internally and validates the full canonical payload instead of trusting the incoming id bytes.
- 📌 Runtime genesis keeps a local prechecked seam until plan 03 because existing cross-network public behavior still depends on that compatibility path.
- 📌 Shared assets and genesis config parsing now reject malformed metadata containers, malformed `policy.flags`, unknown nested flag keys, invalid bool flag values, oversized numeric casts, and reserved `policy_flags` bits.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Canonical metadata was missing from production config-driven identity derivation**

- **Found during:** Task 2
- **Issue:** Config-loaded definitions could ignore metadata when deriving ids, allowing canonical identity drift between constructor and loader paths.
- **Fix:** Reused canonical `AssetDefinition::derive_id(...)` from config loading and added fail-closed metadata parsing.
- **Files modified:** `crates/z00z_core/src/assets/assets_config.rs`, `crates/z00z_core/src/assets/definition.rs`
- **Verification:** `cargo test -p z00z_core --lib -- --nocapture`; `cargo test -p z00z_core --release --features test-fast --test assets_tests -- --nocapture`
- **Committed in:** not committed in this execution

**2. [Rule 1 - Bug] Registry snapshot rehydration accepted non-canonical definitions**

- **Found during:** Task 2 closure review
- **Issue:** A snapshot payload could rehydrate a definition whose fields no longer matched its id.
- **Fix:** Validated definitions on normal insert, batch insert, and snapshot update before registry acceptance.
- **Files modified:** `crates/z00z_core/src/assets/registry.rs`
- **Verification:** `cargo test -p z00z_core --lib -- --nocapture`; targeted snapshot regression added in registry tests
- **Committed in:** not committed in this execution

**3. [Rule 2 - Missing Critical] Public asset construction still bypassed authoritative definition validation**

- **Found during:** Task 2 closure review
- **Issue:** Even with canonical definitions available, public asset constructors and validators could still accept unchecked definitions.
- **Fix:** Added checked `Asset::new(...)` behavior, explicit `new_prechecked(...)`, and authoritative definition validation inside `Asset::validate()`.
- **Files modified:** `crates/z00z_core/src/assets/assets.rs`
- **Verification:** `cargo test -p z00z_core --lib -- --nocapture`; `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_conservation -- --nocapture`
- **Committed in:** not committed in this execution

**4. [Rule 2 - Missing Critical] Public genesis helpers leaked the deferred runtime compatibility seam**

- **Found during:** Task 2 closure review
- **Issue:** Public dev helper paths still constructed definitions through the network-aware runtime seam that plan 03 intentionally owns.
- **Fix:** Added checked public helper construction in `asset_std.rs` and kept the prechecked seam local to runtime genesis internals.
- **Files modified:** `crates/z00z_core/src/genesis/asset_std.rs`, `crates/z00z_core/src/genesis/genesis.rs`
- **Verification:** `cargo test -p z00z_core --test genesis_tests test_asset_cross_network_id -- --nocapture`
- **Committed in:** not committed in this execution

**5. [Rule 1 - Bug] Config parsing failed open on malformed flags and truncating numeric casts**

- **Found during:** Task 2 closure review
- **Issue:** Invalid `policy.flags` shapes, invalid bool types, unknown nested keys, and oversized `u64` numeric fields could pass silently or truncate.
- **Fix:** Added fail-closed nested flag parsing, alias handling, explicit override detection, and `try_into()` numeric bounds checks.
- **Files modified:** `crates/z00z_core/src/assets/assets_config.rs`, `crates/z00z_core/src/genesis/genesis_config.rs`
- **Verification:** `cargo test -p z00z_core --lib -- --nocapture`; `cargo test -p z00z_core --release --features test-fast --test assets_tests -- --nocapture`
- **Committed in:** not committed in this execution

**6. [Rule 2 - Missing Critical] Reserved `policy_flags` bits were not rejected by the canonical validation seam**

- **Found during:** Final closure review
- **Issue:** Canonical identity validation still accepted reserved policy bits even after config hardening.
- **Fix:** Rejected reserved bits in `validate_fields()` and added a dedicated regression test.
- **Files modified:** `crates/z00z_core/src/assets/definition.rs`
- **Verification:** `cargo test -p z00z_core --lib -- --nocapture`
- **Committed in:** not committed in this execution

---

**Total deviations:** 6 auto-fixed (3 bug, 3 missing critical)
**Impact on plan:** All deviations were required to make the canonical identity seam authoritative across consumer paths. Scope stayed inside the planned `PH26-ASSET-ID` boundary.

## Issues Encountered

- ⚠️ Historical tests across `z00z_core` assumed placeholder ids such as `[1u8; 32]` survived constructor calls; those expectations had to be rewritten to follow canonical ids.
- ⚠️ The broader workspace release gate still has an unrelated `z00z_wallets` blocker documented in `deferred-items.md`; it does not block this plan’s `z00z_core` closure evidence.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ✅ `PH26-ASSET-ID` is implemented and validated for assets-layer construction, config loading, public helper paths, and registry acceptance.
- ✅ `026-02` can now reuse canonical definition payloads for full-payload registry hashing without inventing a parallel framing rule.
- ✅ `026-03` still owns the remaining runtime genesis compatibility seam and protected-network hardening.
- ✅ `026-04` can now require validated `DefinitionWire` rehydration against the canonical identity seam.

## Validation Evidence

- ✅ `cargo test -p z00z_core --lib -- --nocapture` -> `203 passed; 0 failed`
- ✅ `cargo test -p z00z_core --release --features test-fast --test assets_tests -- --nocapture` -> `252 passed; 0 failed; 7 ignored`
- ✅ `cargo test -p z00z_core --test genesis_tests test_asset_cross_network_id -- --nocapture` -> `1 passed; 0 failed`
- ✅ `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_conservation -- --nocapture` -> `3 passed; 0 failed`
- ✅ Final scoped review pass 1 -> `NO_SIGNIFICANT_ISSUES`
- ✅ Final scoped review pass 2 -> `NO_SIGNIFICANT_ISSUES`

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/026-crypto-audit-core/026-01-SUMMARY.md`
- ✅ Validation evidence recorded and matched the final tested working tree
- ✅ No commit hashes were claimed because git checkpointing was not performed in this execution

---

*Phase: 026-crypto-audit-core*
*Completed: 2026-03-28*
