---
phase: 063-Core-Update
plan: 063-11
status: complete
completed_at: 2026-06-29
next_plan: 063-12
summary_artifact_for: .planning/phases/063-Core-Update/063-11-PLAN.md
---

# 063-11 Summary: Test Ownership And Naming Flattening

## Outcome

`063-11` is complete. `PLAN-063-G11` closes `REC-063-P2-01` by flattening
`crates/z00z_core/tests/`, removing `*_suite.rs` naming from the owned
`z00z_core` test surface, and keeping one canonical root for integration test
entrypoints and module wiring.

The flat tree now lives directly under `crates/z00z_core/tests/` with
`tests/fixtures/` preserved as the only allowed subdirectory. Source-owned
unit test includes were rewritten to canonical `test_*.rs` filenames across
`assets` and `genesis`, the root integration mod files now point at
meaningfully named top-level test files instead of numbered nested
`test_integration_assets_testN.rs` paths, and the root still exposes explicit
rights or vouchers coverage through `test_rights_config.rs`,
`test_genesis_rights.rs`, `test_genesis_policies.rs`, and
`test_genesis_vouchers.rs`.

`crates/z00z_core/Cargo.toml` now disables Cargo autodiscovery with
`autotests = false` and lists the owned test entrypoints explicitly. That
keeps the flattened layout from becoming a second authority where Cargo
silently compiles submodule files as accidental standalone test crates.

The first broad workspace rerun exposed one downstream stale authority guard
in `crates/z00z_wallets/tests/test_rename_guards.rs`: it still expected the
retired `z00z_core` `*_suite.rs` paths. The same slice updated that guard to
the live `test_asset.rs`, `test_definition.rs`, `test_leaf.rs`,
`test_nonce.rs`, `test_registry.rs`, `test_wire_compat.rs`,
`test_genesis.rs`, and `test_validator.rs` vocabulary.

The slice also truth-restored the Phase 063 authority docs around manifest
golden verification. `063-11-PLAN.md` and `063-TESTS-TASKS.md` now use exact
`cargo test --release -p z00z_core --test ...` target invocations instead of
the misleading filter form that can exit green without actually running the
named manifest-golden crate.

## Files Changed

- `crates/z00z_core/Cargo.toml`
- `crates/z00z_core/src/assets/{assets.rs,definition.rs,leaf.rs,mod.rs,nonce.rs,registry.rs,wire.rs}`
- `crates/z00z_core/src/assets/{test_asset.rs,test_definition.rs,test_leaf.rs,test_nonce.rs,test_registry.rs}`
- `crates/z00z_core/src/genesis/{genesis.rs,validator.rs,test_genesis.rs,test_validator.rs}`
- `crates/z00z_core/tests/test_assets.rs`
- `crates/z00z_core/tests/test_assets_mod.rs`
- `crates/z00z_core/tests/test_genesis.rs`
- `crates/z00z_core/tests/test_genesis_mod.rs`
- flattened root test files under `crates/z00z_core/tests/test_*.rs`
- `crates/z00z_core/tests/{test_genesis_readme.md,test_genesis_rights_manifest.md}`
- `crates/z00z_wallets/tests/test_rename_guards.rs`
- `.planning/phases/063-Core-Update/{063-11-PLAN.md,063-TESTS-TASKS.md}`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core --tests`
- `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`
- `find crates/z00z_core/tests -mindepth 2 -type f ! -path '*/tests/fixtures/*'`
- `rg -n "test_integration_assets_test|_suite\\.rs|tests/(assets|genesis|rights|vauchers|vectors)|#\\[path = \"\\.\\./assets" crates/z00z_core/Cargo.toml crates/z00z_core/tests crates/z00z_core/src`
- `cargo test --release -p z00z_wallets --test test_rename_guards`
- `git diff --check -- .planning/phases/063-Core-Update/063-11-PLAN.md .planning/phases/063-Core-Update/063-TESTS-TASKS.md crates/z00z_core/Cargo.toml crates/z00z_core/src/assets crates/z00z_core/src/genesis crates/z00z_core/tests crates/z00z_wallets/tests/test_rename_guards.rs`
- `cargo test --release`

- Result:
  - The mandatory bootstrap gate passed before the slice work and again after
    the core layout or manifest rewrites.
  - The focused `z00z_core` release test packet passed with the flattened root
    and explicit manifest wiring.
  - The flat-tree scan stayed empty outside `tests/fixtures`, and the stale
    layout scan over `Cargo.toml`, `src`, and `tests` stayed clean.
  - The manifest wiring audit confirmed `autotests = false` plus one explicit
    owned test-entrypoint set for `assets`, `genesis`, manifest goldens or
    refs, and the direct signature or stealth guards.
  - The first broad workspace rerun exposed stale `z00z_wallets`
    rename-guard expectations for the retired `z00z_core` `*_suite.rs`
    surface; the guard was updated in the same slice and rerun green.
  - The downstream wallet rename guard rerun passed with `11 passed; 0 failed`.
  - The final broad workspace `cargo test --release` gate passed end to end on
    the current tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times
against this slice in YOLO mode, but the available runtime still produced no
review output:

- Attempt 1
  - `timeout 60s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-11-PLAN.md current_task="Flatten core test ownership and canonicalize test naming" --yolo'`
  - Result: timed out with exit `124` and no output
- Attempt 2
  - `timeout 60s gsd --extension .github --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-11-PLAN.md current_task="Flatten core test ownership and canonicalize test naming" --yolo'`
  - Result: timed out with exit `124` and no output
- Attempt 3
  - `timeout 60s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-11-PLAN.md current_task="Flatten core test ownership and canonicalize test naming" --yolo'`
  - Result: timed out with exit `124` and no output

Equivalent review passes were executed manually under the prompt contract and
the repository `doublecheck` expectations.

- Pass 1
  - Rechecked the flat-tree invariant, stale layout or stale naming scan, and
    diff hygiene across the touched core or wallet files
  - Result: clean
- Pass 2
  - Audited `Cargo.toml` test ownership, verified `autotests = false`, and
    confirmed rights or vouchers coverage remains visible from the flat root
  - Result: clean
- Pass 3
  - Rechecked the broader stale retired-core-name scan and reran the wallet
    rename guard against the live `z00z_core` path vocabulary
  - Result: clean
- Pass 4
  - Rechecked the Phase 063 authority-doc verify commands after the `--test`
    truth-restoration patch and kept diff hygiene green through closeout
  - Result: clean

Passes 1 and 2 were consecutive clean review passes after the downstream guard
fix, and Passes 3 and 4 kept the slice clean through final closeout.

## Completion Notes

- `063-11-SUMMARY.md` closes `PLAN-063-G11` and advances the active execution
  lane to `063-12-PLAN.md`.
- `crates/z00z_core/tests/` is now flat except for `tests/fixtures/`.
- The owned `z00z_core` test surface no longer uses `*_suite.rs` filenames.
- `crates/z00z_core/Cargo.toml` is the only authority for the flattened test
  entrypoint set, and the touched wallet rename guard now tracks the same live
  path vocabulary.
