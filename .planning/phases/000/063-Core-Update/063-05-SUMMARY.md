---
phase: 063-Core-Update
plan: 063-05
status: complete
completed_at: 2026-06-28
next_plan: 063-06
summary_artifact_for: .planning/phases/063-Core-Update/063-05-PLAN.md
---

# 063-05 Summary: Explicit Generation Lanes Under One Bootstrap Authority

## Outcome

`063-05` is complete. `PLAN-063-G05` now closes `REC-063-P1-02` by making
selected-family genesis generation explicit without creating a second
bootstrap authority path.

The live canonical full-bootstrap entrypoint remains `run_genesis()`. The new
explicit lane path lives under `GenesisGenerationPlan` plus
`run_genesis_with_plan(...)`, and plan-aware validation now flows through
`validate_genesis_config_for(...)` instead of one unconditional whole-config
gate. Partial exports no longer pretend to be canonical settlement manifests:
they emit `genesis_generation_receipt.json`, while
`genesis_settlement_manifest.json` remains reserved for full bootstrap only.

The slice also preserves full-bootstrap parity. `full_bootstrap()` reproduces
the legacy full manifest path, while `assets_only()`, `rights_only()`,
`vouchers_only()`, and explicit `selected(...)` lane sets stay typed,
negative-tested, and subordinate to one canonical `GenesisConfig`.

## Files Changed

- `crates/z00z_core/src/genesis/genesis_run.rs`
- `crates/z00z_core/src/genesis/genesis_config_validate.rs`
- `crates/z00z_core/src/genesis/genesis.rs`
- `crates/z00z_core/src/genesis/mod.rs`
- `crates/z00z_core/src/genesis/validator.rs`
- `crates/z00z_core/tests/genesis/test_genesis_manifest.rs`
- `.planning/phases/063-Core-Update/063-05-PLAN.md`
- `.planning/phases/063-Core-Update/063-TEST-SPEC.md`
- `.planning/phases/063-Core-Update/063-TESTS-TASKS.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`
- `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests genesis_rights -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests genesis_vouchers -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_full_bootstrap_matches_legacy_run -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_assets_only_skips_rights_validation -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_rights_only_requires_policy_resolution_when_needed -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_vouchers_only_rejects_non_voucher_policy -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_partial_run_does_not_emit_full_settlement_manifest -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_selected_lanes_preserve_terminal_collision_checks -- --nocapture`
- `cargo test --release`
- `rg -n "FullBootstrap|Selected|lane_manifest|generation_receipt|validate_genesis_config_for" crates/z00z_core/src/genesis crates/z00z_core/tests`
- Result: green

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted repeatedly, but
the available runtime paths are tool-blocked on this tree:

- global `gsd --mode json --print ...` resolves a runtime that does not expose
  the repo-local `z00z-chat-init` skill
- repo-local `.github/gsd-core/bin/gsd-tools.cjs` aborts immediately on the
  missing `.github/package.json` runtime dependency

Equivalent review passes were executed manually against the same slice.

- Pass 1
  - Reviewed `genesis_run.rs`, `genesis_config_validate.rs`, `genesis.rs`,
    `mod.rs`, `validator.rs`, and `test_genesis_manifest.rs` against
    `063-05-PLAN.md`, `063-TODO.md`, and `063-core-examples.md`
  - Result: no substantive phase-local correctness or canonical-path defects
    remained after the lane-plan implementation landed
- Pass 2
  - `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`
  - `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests genesis_rights -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests genesis_vouchers -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_full_bootstrap_matches_legacy_run -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_assets_only_skips_rights_validation -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_rights_only_requires_policy_resolution_when_needed -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_vouchers_only_rejects_non_voucher_policy -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_partial_run_does_not_emit_full_settlement_manifest -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_selected_lanes_preserve_terminal_collision_checks -- --nocapture`
  - `rg -n "FullBootstrap|Selected|lane_manifest|generation_receipt|validate_genesis_config_for" crates/z00z_core/src/genesis crates/z00z_core/tests`
  - Result: clean
- Pass 3
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test --release`
  - Result: clean

Passes 2 and 3 were consecutive clean runs.

## Completion Notes

- `063-05-SUMMARY.md` closes `PLAN-063-G05` and advances the execution lane to
  `063-06-PLAN.md`.
- `run_genesis()` remains the one canonical full-bootstrap path.
- `validate_genesis_config_for(...)` is now the one canonical plan-aware
  validation path.
- Partial lane exports now have one canonical receipt surface:
  `genesis_generation_receipt.json`.
