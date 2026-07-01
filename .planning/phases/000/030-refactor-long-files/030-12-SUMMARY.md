---
phase: 030-refactor-long-files
plan: 12
subsystem: crypto-support-normalization
tags: [rust, crypto, utils, simulator, facade, support-surface, verification]
requires:
  - phase: 030-03
    provides: stable crypto facade after protected-surface extraction
  - phase: 030-04
    provides: stable simulator helper split and `z00z_utils::io` facade baseline
provides:
  - preserved approved crypto support namespaces on the shallow `z00z_crypto` surface
  - final simulator helper routing through `stage_4_utils` and `stage_6_utils`
  - closure of Phase 030 on a green max-safe workspace gate
affects: [z00z_crypto, z00z_utils, z00z_simulator, planning]
tech-stack:
  added: []
  patterns: [shallow support-surface preservation, utility-module routing, explicit helper prelude, release-style verification]
key-files:
  created:
    - .planning/phases/030-refactor-long-files/030-12-SUMMARY.md
  modified:
    - crates/z00z_simulator/src/scenario_1/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/persistence.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_capture.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_diff.rs
    - crates/z00z_simulator/tests/test_stage4_source_shape.rs
    - reports/full_verify-report-long-running-tests.txt
key-decisions:
  - Preserve the approved shallow crypto namespaces on `z00z_crypto::lib.rs` and the shallow `z00z_utils::io` facade rather than collapsing them into a new replacement surface during the normalization wave.
  - Treat `stage_4_utils/mod.rs` as the explicit helper prelude and caller facade for stage 4 helpers; sibling helper files must not rely on recursive nested module loading from `tx_lane_impl.rs`.
patterns-established:
  - "Support-surface closeout: preserve the canonical shallow facade, route simulator callers through utility modules, and fix any helper-scope leaks by making the utility prelude explicit."
requirements-completed: [PH30-NORMALIZE, PH30-SYNC, PH30-VERIFY]
completed: 2026-04-01
---

# Phase 030 Plan 12 Summary

📌 Phase 030 closes with simulator stage callers routed through the intended
utility modules, while the shallow crypto and I/O support facades remain the
canonical caller-visible surfaces.

## Accomplishments

- 📌 Kept the approved shallow crypto namespaces intact on
  `crates/z00z_crypto/src/lib.rs`, including `domains`, `kdf_domains`,
  `aead_transport`, `hash_policy`, `hash_types`, `kdf_consensus`, and
  `kdf_extended`, instead of collapsing support callers onto a replacement
  surface.
- 📌 Kept `crates/z00z_utils/src/io/mod.rs` as the shallow I/O facade over
  `fs`, with callers consuming `z00z_utils::io::*` instead of deep
  `z00z_utils::io::fs::*` imports.
- 📌 Finalized simulator stage ownership so the scenario root publishes
  `stage_4_lane` and `stage_6_lane`, while stage callers route through
  `stage_4_utils` and `stage_6_utils` instead of file-specific lane imports.
- 📌 Repaired the late blocker in stage 4 by removing recursive module loading,
  adding the missing collection imports, and turning
  `stage_4_utils/mod.rs` into the explicit helper prelude for the stage-4
  utility directory.
- 📌 Strengthened the stage-4 source-shape regression so it now checks that
  stage callers reference the utility-module surfaces and not the legacy
  file-specific lane paths.
- 📌 Closed the plan on a green max-safe workspace gate with the long-running
  test report regenerated.

## Task Commits

📌 No git commit was created in this closeout. The workspace contains unrelated
local changes, and the repo rules require the owned Z00Z git-versioning flow
instead of ad hoc `git commit` usage.

## Files Created/Modified

- 📌 `crates/z00z_simulator/src/scenario_1/mod.rs` now owns the stable
  `stage_4_lane` and `stage_6_lane` seams while callers consume the utility
  modules.
- 📌 `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs` is now the
  explicit prelude and facade for stage-4 helper submodules.
- 📌 `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
  dropped the recursive `mod.rs` reload and now consumes the shared helper
  surface through `super::stage_4_utils`.
- 📌 `crates/z00z_simulator/src/scenario_1/stage_4_utils/persistence.rs`,
  `reports_capture.rs`, and `reports_diff.rs` now declare the collection types
  they actually use instead of inheriting them from the old nested-module scope.
- 📌 `crates/z00z_simulator/tests/test_stage4_source_shape.rs` now asserts that
  the stage callers route through `stage_4_utils` and `stage_6_utils` rather
  than `tx_lane_impl` or `bundle_lane_impl`.
- 📌 `reports/full_verify-report-long-running-tests.txt` was regenerated during
  the final clean workspace verification pass.

## Decisions Made

- 📌 Preserve the crypto support namespaces already approved on the public
  `z00z_crypto` surface instead of widening this normalization wave into a new
  support-facade redesign.
- 📌 Fix the stage-4 helper leak at the module boundary, not with another layer
  of compatibility imports. The utility `mod.rs` file is now the only intended
  shared scope for sibling stage-4 helper files.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed recursive stage-4 helper loading**

- 📌 Found during: final `full_verify --max-safe-run` rerun for Phase 030 closeout
- 📌 Issue: `tx_lane_impl.rs` still loaded `stage_4_utils/mod.rs` recursively,
  creating duplicate helper ownership and compile failures in the stage-4
  utility tree.
- 📌 Fix: removed the nested `mod stage_4_utils;` load, moved the lane seam onto
  `super::stage_4_utils`, and published the stable stage-lane ownership from
  `scenario_1/mod.rs`.
- 📌 Files modified: `crates/z00z_simulator/src/scenario_1/mod.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`

**2. [Rule 1 - Bug] Restored explicit helper imports for stage-4 support files**

- 📌 Found during: the first post-fix compile of the stage-4 helper tree
- 📌 Issue: helper siblings had been implicitly inheriting names like
  `BTreeMap` and `BTreeSet` from the old nested-module scope.
- 📌 Fix: added the missing collection imports directly to the helper files and
  made `stage_4_utils/mod.rs` the explicit shared prelude for sibling helper
  modules.
- 📌 Files modified: `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/persistence.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_capture.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/reports_diff.rs`

**3. [Rule 1 - Bug] Hardened the stage-4 source-shape guard against legacy lane imports**

- 📌 Found during: final regression coverage review for the utility-surface closeout
- 📌 Issue: the existing source-shape test covered the stage-4 split, but it did
  not assert that downstream stage callers stopped reaching into
  `tx_lane_impl` and `bundle_lane_impl` directly.
- 📌 Fix: extended `test_stage4_source_shape.rs` with route assertions over the
  affected stage source files and runner.
- 📌 Files modified: `crates/z00z_simulator/tests/test_stage4_source_shape.rs`

## Caller Inventory Audit

📌 Exact simulator and support residue checks:

```text
rg -n "stage_4_utils::tx_lane_impl|stage_6_utils::bundle_lane_impl" crates -g '*.rs'
EXIT=1

rg -n "z00z_utils::io::fs::" crates -g '*.rs'
EXIT=1
```

📌 Interpretation: no remaining Rust callers reach into the file-specific
simulator lane modules or the deep `z00z_utils::io::fs` path.

📌 The shallow crypto support surface remains intentionally exported from
`crates/z00z_crypto/src/lib.rs`, including `domains`, `kdf_domains`,
`aead_transport`, `hash_policy`, `hash_types`, `kdf_consensus`, and
`kdf_extended`.

## Known Stubs

📌 None detected in the touched Plan 12 support-surface closeout files.

## User Setup Required

📌 None. This plan changed only Rust module wiring, regression coverage, and
planning artifacts.

## Phase Readiness

- 📌 Phase 030 is now fully summary-backed and ready to be marked complete in the
  roadmap and state artifacts.

## Verification

- 📌 `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_crypto --release --test test_hash_policy -- --nocapture`
- 📌 `cargo test -p z00z_crypto --release --test test_domain_separation -- --nocapture`
- 📌 `cargo test -p z00z_crypto --release --test test_public_surface -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_kdf -- --nocapture`
- 📌 `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_split -- --nocapture`
- 📌 `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`
- 📌 `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_source_shape -- --nocapture`
- 📌 `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
- 📌 `cargo test -p z00z_utils --release --features test-fast --all-targets -- --nocapture`
- 📌 `cargo test -p z00z_crypto --release --features test-fast -- --nocapture`
- 📌 `rg -n "stage_4_utils::tx_lane_impl|stage_6_utils::bundle_lane_impl" crates -g '*.rs'`
- 📌 `rg -n "z00z_utils::io::fs::" crates -g '*.rs'`
- 📌 `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run` reached a clean summary of `313 planned, 21 skipped, 0 failed` and regenerated the long-running test inventory.

## Self-Check

📌 PASSED: `030-12-SUMMARY.md` exists, and the accompanying roadmap and state
updates now mark all 12 Phase 030 plans as closed.
