---
phase: 030
plan: 21
subsystem: z00z_core assets and genesis cli
summary: Reduce the remaining oversized z00z_core asset-domain, genesis-output, and CLI roots below the continuation band while preserving the shallow asset and genesis surfaces.
tags:
  - phase-030
  - z00z-core
  - assets
  - genesis
  - cli
  - seams
requirements-completed:
  - PH30-SEAMS
  - PH30-FACADE
  - PH30-VERIFY
affects:
  - crates/z00z_core/src/assets
  - crates/z00z_core/src/genesis
  - crates/z00z_core/bin/assets
  - crates/z00z_core/bin/genesis
provides:
  - Thin stable asset-domain roots with sibling construction, crypto, serde, formatting, loader, and extracted test seams
  - Thin stable genesis and CLI roots with support modules under the continuation band
  - Verified shallow asset and genesis caller paths after the structural split
key_files:
  created:
    - crates/z00z_core/src/assets/asset_arc_serde.rs
    - crates/z00z_core/src/assets/asset_construction.rs
    - crates/z00z_core/src/assets/asset_crypto.rs
    - crates/z00z_core/src/assets/asset_tests.rs
    - crates/z00z_core/src/assets/assets_config_load.rs
    - crates/z00z_core/src/assets/definition_format.rs
    - crates/z00z_core/src/assets/definition_tests.rs
    - crates/z00z_core/src/assets/leaf_tests.rs
    - crates/z00z_core/src/assets/nonce_docs.md
    - crates/z00z_core/src/assets/nonce_tests.rs
    - crates/z00z_core/src/assets/registry_tests.rs
    - crates/z00z_core/src/assets/wire_pkg_serde.rs
    - crates/z00z_core/bin/assets/assets_generation_cli_phase.rs
    - crates/z00z_core/bin/assets/assets_generation_cli_report.rs
    - crates/z00z_core/bin/genesis/assets_extractor_cli_ops.rs
    - crates/z00z_core/bin/genesis/assets_extractor_cli_args.rs
    - crates/z00z_core/bin/genesis/assets_analyzer_cli_ops.rs
    - crates/z00z_core/bin/genesis/assets_analyzer_cli_args.rs
    - crates/z00z_core/src/genesis/genesis_output_support.rs
  modified:
    - crates/z00z_core/src/assets/assets.rs
    - crates/z00z_core/src/assets/registry.rs
    - crates/z00z_core/src/assets/definition.rs
    - crates/z00z_core/src/assets/nonce.rs
    - crates/z00z_core/src/assets/leaf.rs
    - crates/z00z_core/src/assets/wire_pkg.rs
    - crates/z00z_core/src/assets/assets_config.rs
    - crates/z00z_core/src/genesis/genesis_output.rs
    - crates/z00z_core/bin/assets/assets_generation_cli.rs
    - crates/z00z_core/bin/genesis/assets_extractor_cli.rs
    - crates/z00z_core/bin/genesis/assets_analyzer_cli.rs
decisions:
  - Keep `assets.rs`, `registry.rs`, `definition.rs`, `nonce.rs`, and `genesis_output.rs` as stable shallow roots while sibling files own extracted helper and test logic.
  - Keep binary entry roots shallow and route moved helpers through sibling support modules instead of rewriting CLI behavior.
  - Treat stale plan test aliases (`test_assets`, `test_genesis`) as verification drift and map them to the live integration targets (`assets_tests`, `genesis_tests`) rather than widening scope into test renames.
metrics:
  duration: current-session
  completed_at: 2026-04-03
  tasks_completed: 2/2
---

# Phase 030 Plan 21: Core Continuation Split Summary

Closed the `z00z_core` continuation wave for the remaining asset-domain and genesis or CLI roots. The asset roots now stay as thin stable facades over sibling seams, the core CLI and genesis-output roots were reduced below the continuation band, and the shallow caller story stayed intact through release-style verification.

## Outcomes

- Task 1 closed the remaining oversized asset-domain roots:
  - `assets.rs` now stays at `382` lines while sibling files own Arc serde, construction, crypto helpers, and extracted tests
  - `registry.rs` now stays at `331` lines with unit coverage moved into `registry_tests.rs`
  - `definition.rs` now stays at `325` lines while formatting and conversion helpers moved into `definition_format.rs`
  - `nonce.rs` became a thin documented facade over `nonce_counter`, `nonce_derivation`, `nonce_type`, and `nonce_tests.rs`
  - `leaf.rs`, `wire_pkg.rs`, and `assets_config.rs` each dropped below the continuation band through extracted helper or test seams
- Task 2 closed the remaining core CLI and genesis-output residue:
  - `assets_generation_cli.rs` now stays at `317` lines while `assets_generation_cli_phase.rs` and `assets_generation_cli_report.rs` own the moved logic
  - `assets_extractor_cli.rs` now stays at `102` lines while args and ops live in dedicated sibling files
  - `assets_analyzer_cli.rs` now stays at `101` lines with dedicated args and ops seams
  - `genesis_output.rs` now stays at `54` lines while `genesis_output_support.rs` owns the moved support flow at `397` lines
- The shallow surface remained stable:
  - asset-facing callers still enter through the original `assets::*` roots
  - genesis support still resolves through `z00z_core::genesis::*` without path widening in callers
  - binary entrypoints remain the same while their heavy helper bodies moved into sibling files

## Verification

- File-level diagnostics were clean for the touched asset, genesis, and CLI roots plus their new support files.
- Live post-split line counts for plan targets:
  - `crates/z00z_core/src/assets/assets.rs`: `382`
  - `crates/z00z_core/src/assets/registry.rs`: `331`
  - `crates/z00z_core/src/assets/definition.rs`: `325`
  - `crates/z00z_core/src/assets/nonce.rs`: `302`
  - `crates/z00z_core/src/assets/leaf.rs`: `189`
  - `crates/z00z_core/src/assets/wire_pkg.rs`: `61`
  - `crates/z00z_core/src/assets/assets_config.rs`: `232`
  - `crates/z00z_core/bin/assets/assets_generation_cli.rs`: `317`
  - `crates/z00z_core/bin/genesis/assets_extractor_cli.rs`: `102`
  - `crates/z00z_core/bin/genesis/assets_analyzer_cli.rs`: `101`
  - `crates/z00z_core/src/genesis/genesis_output.rs`: `54`
  - `crates/z00z_core/src/genesis/genesis_output_support.rs`: `397`
- Executed verification commands:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_core --release --test assets_tests -- --nocapture`
  - `cargo test -p z00z_core --release --test genesis_tests -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_genesis_integration -- --nocapture`
  - `cargo test --release --features test-fast --features wallet_debug_dump`
  - `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- Final max-safe gate passed with `planned=313 skipped=21 failed=0`.

## Deviations from Plan

### Auto-fixed Issues

1. `[Rule 3 - Blocking issue]` `state begin-phase` in the repository GSD implementation still hard-reset Phase 030 to `Plan 1`, which caused repeated stalls before work on `030-21` began. The live repository `begin-phase` path was fixed so execution resumes from the first incomplete plan.
2. `[Rule 3 - Blocking issue]` The plan's verification commands still referenced stale `z00z_core` test target names (`test_assets`, `test_genesis`). Verification was mapped to the live targets `assets_tests` and `genesis_tests` after confirming the current `cargo test -- --list` surface.
3. `[Rule 1 - Bug]` The first support-file extraction left moved `genesis_output` and CLI helper functions private, which broke sibling callers during release compilation. The extracted helpers were reopened only to `pub(super)` or `pub(crate)` where required.
4. `[Rule 3 - Blocking issue]` The first mechanical extraction left a truncated getter in `asset_crypto.rs`, a malformed insertion in `asset_construction.rs`, and stray leading blank lines in extracted `*_tests.rs` files. These were repaired before the final verification reruns.

## Residual Risk

- The plan closed the targeted roots and support seams below the continuation band, but later continuation plans may still choose to reduce extracted support modules further if the repo-wide residue audit wants stricter limits than the root-focused closure used here.

## Self-Check: PASSED

- Summary file created at `.planning/phases/030-refactor-long-files/030-21-SUMMARY.md`
- Targeted asset, genesis, simulator, broad release, and max-safe verification all passed after the split
- Every root named in `030-21-PLAN.md` is now below the continuation band
- The final max-safe run reported `planned=313 skipped=21 failed=0`
