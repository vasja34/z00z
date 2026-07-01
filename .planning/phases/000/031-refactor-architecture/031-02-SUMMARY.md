---
phase: 031-refactor-architecture
plan: "02"
subsystem: core
tags: [rust, facade, assets, genesis, json, bounds]
requires:
  - phase: 031-01
    provides: Wave 0 caller inventory and Gate G-00 import-graph proof for root-facade narrowing.
provides:
  - Curated `z00z_core` root facade without wildcard stable exports.
  - Explicit asset-owned `AssetPkgWire` JSON payload ceiling with fail-closed oversized decode rejection.
  - Oversized asset package integration coverage and updated crate boundary documentation.
affects: [031-03, z00z_core, assets, genesis]
tech-stack:
  added: []
  patterns: [explicit root facade ownership, asset-owned fail-closed json boundary]
key-files:
  created: [crates/z00z_core/tests/assets/test_wire_pkg_bounds.rs]
  modified:
    [crates/z00z_core/src/lib.rs, crates/z00z_core/src/genesis/mod.rs, crates/z00z_core/src/assets/gas.rs, crates/z00z_core/src/assets/mod.rs, crates/z00z_core/src/assets/wire.rs, crates/z00z_core/src/assets/wire_pkg.rs, crates/z00z_core/src/assets/wire_pkg_serde_impls.rs, crates/z00z_core/src/assets/wire_pkg_serde_parse.rs, crates/z00z_core/tests/assets/mod.rs, crates/z00z_core/README.md]
key-decisions:
  - "Keep the `z00z_core` crate root limited to curated runtime contracts and remove wildcard stable exports."
  - "Make the asset package JSON seam itself own the explicit 64 KiB fail-closed payload ceiling instead of leaving hardening implicit in callers."
patterns-established:
  - "Curated root facades expose only proven runtime contracts; helper/config entrypoints stay under owning modules."
  - "Public JSON wire seams reject oversized payloads before secret probing or deserialization."
requirements-completed: [PH31-CORE]
duration: 1m 26s
completed: 2026-04-04
---

# Phase 031 Plan 02: Core Facade Narrowing Summary

**Curated `z00z_core` runtime facade with asset-owned 64 KiB fail-closed `AssetPkgWire` JSON boundary**

## Performance

- **Duration:** 1m 26s
- **Started:** 2026-04-04T14:12:46Z
- **Completed:** 2026-04-04T14:14:12Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Removed broad wildcard stable exports from the `z00z_core` root and replaced them with one explicit runtime contract set.
- Replaced the wildcard `z00z_core::genesis` facade with a curated export list while keeping higher-level helpers under owning modules.
- Added an explicit 64 KiB payload ceiling for `AssetPkgWire` JSON decode and secret probing, documented the ownership seam, and covered oversized rejection with integration tests.

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace broad root exports with an explicit `z00z_core` facade** - `fcb067e5` (refactor)
2. **Task 2: Preserve the bounded JSON and upstream-cap ownership while the facade narrows** - `c60ca012` (fix)

## Files Created/Modified

- `crates/z00z_core/src/lib.rs` - removed broad root wildcard exports and published a curated runtime root facade.
- `crates/z00z_core/src/genesis/mod.rs` - replaced wildcard genesis re-exports with an explicit facade.
- `crates/z00z_core/src/assets/gas.rs` - repaired internal imports after root facade narrowing.
- `crates/z00z_core/src/assets/wire_pkg.rs` - defined the asset-owned `ASSET_PKG_JSON_MAX_BYTES` ceiling and documented seam ownership.
- `crates/z00z_core/src/assets/wire_pkg_serde_impls.rs` - added fail-closed payload size enforcement before decode.
- `crates/z00z_core/src/assets/wire_pkg_serde_parse.rs` - bounded secret probing before JSON parse.
- `crates/z00z_core/src/assets/wire.rs` - exported the asset package JSON ceiling through the wire facade.
- `crates/z00z_core/src/assets/mod.rs` - exported the asset package JSON ceiling through the public assets facade.
- `crates/z00z_core/README.md` - documented the curated root, owning module entrypoints, and the chosen JSON hardening branch.
- `crates/z00z_core/tests/assets/mod.rs` - registered the new bounds coverage module.
- `crates/z00z_core/tests/assets/test_wire_pkg_bounds.rs` - added oversized payload rejection coverage for decode and secret-probe paths.

## Decisions Made

- Kept the stable `z00z_core` crate root limited to active runtime contracts: `Asset`, `AssetClass`, `AssetDefinition`, `AssetDefinitionRegistry`, `AssetLeaf`, `AssetMetadata`, `AssetPkgWire`, `AssetWire`, `BlindingFactor`, `Commitment`, and `ChainType`.
- Chose the explicit fail-closed hardening branch for shared asset JSON import paths by adding a type-owned bounded decode seam and oversize rejection tests at the owning core boundary.
- Kept genesis file import hardening documented as upstream-cap-owned through bounded `z00z_utils::io::read_to_string` instead of widening that responsibility into a second core-owned file-import layer.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed internal `gas.rs` imports after root facade narrowing**

- **Found during:** Task 1 (Replace broad root exports with an explicit `z00z_core` facade)
- **Issue:** `crates/z00z_core/src/assets/gas.rs` still imported `Amount` and `AssetError` from the old broad crate root, which broke `assets_tests` once the root facade was narrowed.
- **Fix:** Switched `gas.rs` to import `Amount` and `AssetError` from `crate::assets`, keeping ownership on the assets facade instead of restoring compatibility aliases.
- **Files modified:** `crates/z00z_core/src/assets/gas.rs`
- **Verification:** `cargo test -p z00z_core --release --test assets_tests -- --nocapture`
- **Committed in:** `fcb067e5`

**2. [Rule 3 - Blocking] Completed the public re-export chain for the new asset JSON ceiling**

- **Found during:** Task 2 (Preserve the bounded JSON and upstream-cap ownership while the facade narrows)
- **Issue:** The new `ASSET_PKG_JSON_MAX_BYTES` constant was defined at the wire seam but not yet exported through the existing `assets::wire` and `assets` facades.
- **Fix:** Added explicit re-exports in `crates/z00z_core/src/assets/wire.rs` and `crates/z00z_core/src/assets/mod.rs`.
- **Files modified:** `crates/z00z_core/src/assets/wire.rs`, `crates/z00z_core/src/assets/mod.rs`
- **Verification:** `cargo test -p z00z_core --release --test assets_tests -- --nocapture`
- **Committed in:** `c60ca012`

**3. [Rule 3 - Blocking] Reworked oversize tests to avoid feature-gated dev constructors**

- **Found during:** Task 2 (Preserve the bounded JSON and upstream-cap ownership while the facade narrows)
- **Issue:** The initial bounds tests used `asset_from_dev_class`, which fails without the `deterministic-rng` feature in the normal `assets_tests` target.
- **Fix:** Rebuilt the test fixture using shared runtime helpers and direct `Asset::new(...)` construction so the bounds coverage works under the normal release-style integration target.
- **Files modified:** `crates/z00z_core/tests/assets/test_wire_pkg_bounds.rs`
- **Verification:** `cargo test -p z00z_core --release --test assets_tests -- --nocapture`
- **Committed in:** `c60ca012`

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking)
**Impact on plan:** All deviations were required to keep the narrowed facade buildable and to make the selected fail-closed hardening branch compile and validate under the release-style test targets.

## Issues Encountered

- The README update initially carried hard-tab indentation that surfaced as markdown formatting noise and was normalized before final validation.
- The first task-level validation pass surfaced internal imports that still assumed the old root facade shape; the fix stayed inside the owning assets module rather than widening the root again.

## Auth Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `z00z_core` now exposes one deliberate stable root surface, so downstream facade-narrowing waves can migrate other callers without binding to new wildcard aliases.
- The asset JSON import hardening branch is now explicit and summary-backed, so later phases can reference a single fail-closed seam instead of proving implicit caller behavior again.

## Known Stubs

None.

## Self-Check: PASSED

- Found `.planning/phases/031-refactor-architecture/031-02-SUMMARY.md`
- Found commit `fcb067e5`
- Found commit `c60ca012`
