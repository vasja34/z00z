---
phase: 063-Core-Update
plan: 063-10
status: complete
completed_at: 2026-06-29
next_plan: 063-11
summary_artifact_for: .planning/phases/063-Core-Update/063-10-PLAN.md
---

# 063-10 Summary: Canonical z00z_config YAML Root

## Outcome

`063-10` is complete. `PLAN-063-G10` closes `REC-063-P1-07` by making
`crates/z00z_core/z00z_config/` the only live core YAML root and by removing
the old `src/genesis`, `src/assets`, `configs`, example-local, and
vector-fixture YAML authority paths from live callers.

The canonical path vocabulary is now explicit in `z00z_core::config_paths`:
all live devnet YAML filenames and path helpers are defined once and
downstream callers use those helpers instead of hardcoded legacy strings. That
keeps `GenesisConfig` as the only bootstrap authority while preserving the
manifest-ref boundary: `assets`, `rights`, `policies`, and `vouchers` may fan
out, while `chain`, `outputs`, and `performance` stay root-owned.

The slice also truth-restored downstream callers exposed by the release
reruns. Simulator Stage 1 and Stage 13 fixtures no longer reach back to
`../z00z_core/src/genesis/genesis_config_devnet_small.yaml`, wallet
profile-contract checks now bind to the live rights authority in
`devnet_rights_config.yaml`, and the wallet docs plus supporting techpaper
notes now use the live right anchors `machine_compute_capability` and
`one_time_agent_action` instead of the retired names. One residual stale
authority note in `docs/tech-papers/done/Z00Z-HJMT-Design.md` was also updated
so the repository no longer carries a second path vocabulary for the same live
config roots.

## Files Changed

- `crates/z00z_core/z00z_config/**`
- `crates/z00z_core/src/config_paths.rs`
- `crates/z00z_core/src/lib.rs`
- `crates/z00z_core/src/genesis/genesis_config.rs`
- `crates/z00z_core/src/genesis/manifest_ref_loader.rs`
- `.planning/phases/063-Core-Update/063-TODO.md`
- `.planning/phases/063-Core-Update/063-core-examples.md`
- `crates/z00z_simulator/src/config/config_accessors.rs`
- `crates/z00z_simulator/src/config/config_defaults.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- `crates/z00z_simulator/src/scenario_1/stage_1/support.rs`
- `crates/z00z_simulator/src/scenario_1/support/scenario_support.rs`
- `crates/z00z_simulator/src/scenario_1/support/claim_shared_cases.rs`
- `crates/z00z_simulator/src/scenario_1/support/checkpoint_shared_cases.rs`
- `crates/z00z_simulator/examples/scenario_1/simulator_interop_support.inc`
- `crates/z00z_simulator/tests/scenario_1/test_claim_acceptance.rs`
- `crates/z00z_simulator/tests/scenario_1/test_claim_integration.rs`
- `crates/z00z_simulator/tests/scenario_1/test_claim_persist.rs`
- `crates/z00z_simulator/tests/scenario_1/test_genesis_integration.rs`
- `crates/z00z_simulator/tests/scenario_1/test_genesis_unit.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs`
- `crates/z00z_simulator/tests/scenario_1/test_stage4_cfg_guards.rs`
- `crates/z00z_simulator/tests/scenario_1/test_tx_handoff_integration.rs`
- `crates/z00z_simulator/tests/scenario_1/test_wallet_claim_replay.rs`
- `crates/z00z_wallets/src/redb_store/test_store_suite.rs`
- `crates/z00z_wallets/docs/WALLET-GUIDE.md`
- `docs/tech-papers/TODO-Wallet-idea.md`
- `docs/tech-papers/done/Z00Z-HJMT-Design.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`
- `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`
- `cargo test --release -p z00z_core --test test_rights_config -- --nocapture`
- `cargo test --release -p z00z_core --test test_voucher_config -- --nocapture`
- `cargo test --release -p z00z_simulator s13_verify_live_outputs_ok -- --nocapture`
- `cargo test --release -p z00z_simulator s13_ -- --nocapture`
- `cargo test --release -p z00z_wallets test_wallet_profile_catalog_contract -- --nocapture`
- `find crates/z00z_core -path '*/target' -prune -o \( -name '*.yaml' -o -name '*.yml' \) -type f | sort`
- `perl -ne 'print if /domain_name:/ && !/domain_name: z00z\.core\.[a-z_]+\.[a-z0-9_]+\.devnet\.v1$/' .planning/phases/063-Core-Update/063-core-examples.md`
- `rg -n "src/(assets|genesis)/.*\.ya?ml|examples/.+\.ya?ml|tests/vectors|crates/z00z_core/configs|genesis_config_devnet_small|devnet_actions_config" crates/z00z_core wiki -g '!**/target/**'`
- `rg -n "crates/z00z_core/src/genesis/genesis_config_devnet|genesis_config_devnet_small|crates/z00z_core/configs|devnet_actions_config|crates/z00z_core/src/assets/assets_config.yaml" crates/z00z_core crates/z00z_simulator crates/z00z_wallets docs/tech-papers wiki -g '!**/target/**'`
- `rg -n "machine_capability|one_time_use|machine_compute_capability|one_time_agent_action|devnet_rights_config.yaml|devnet_assets_config.yaml" crates/z00z_wallets/docs/WALLET-GUIDE.md docs/tech-papers/TODO-Wallet-idea.md crates/z00z_wallets/src/redb_store/test_store_suite.rs`
- `git diff --check -- docs/tech-papers/done/Z00Z-HJMT-Design.md crates/z00z_wallets/docs/WALLET-GUIDE.md docs/tech-papers/TODO-Wallet-idea.md crates/z00z_wallets/src/redb_store/test_store_suite.rs crates/z00z_simulator/src/config/config_accessors.rs crates/z00z_simulator/src/config/config_defaults.rs crates/z00z_simulator/src/scenario_1/stage_1/support.rs crates/z00z_simulator/src/scenario_1/runner_verify.rs crates/z00z_simulator/src/scenario_1/support/scenario_support.rs crates/z00z_simulator/src/scenario_1/support/claim_shared_cases.rs crates/z00z_simulator/src/scenario_1/support/checkpoint_shared_cases.rs crates/z00z_simulator/examples/scenario_1/simulator_interop_support.inc crates/z00z_simulator/tests/scenario_1/test_claim_acceptance.rs crates/z00z_simulator/tests/scenario_1/test_claim_integration.rs crates/z00z_simulator/tests/scenario_1/test_claim_persist.rs crates/z00z_simulator/tests/scenario_1/test_genesis_integration.rs crates/z00z_simulator/tests/scenario_1/test_genesis_unit.rs crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs crates/z00z_simulator/tests/scenario_1/test_stage4_cfg_guards.rs crates/z00z_simulator/tests/scenario_1/test_tx_handoff_integration.rs crates/z00z_simulator/tests/scenario_1/test_wallet_claim_replay.rs crates/z00z_core/src/config_paths.rs`
- `cargo test --release -q`

- Result:
  - The mandatory bootstrap gate passed.
  - The focused core YAML-root and manifest-ref release tests passed.
  - The first broad workspace rerun exposed real downstream stale callers in
    the simulator Stage 13 path setup and the wallet profile catalog contract;
    both were fixed in the same slice and rerun green in release mode.
  - The post-fix simulator Stage 13 release selector packet passed with
    `26 passed; 0 failed`.
  - The post-migration core YAML file list contains exactly the eight files
    under `crates/z00z_core/z00z_config/`.
  - The `domain_name` regex check over `063-core-examples.md` stayed clean.
  - The stale-root scans for retired core YAML paths and retired wallet right
    anchors stayed clean after the final doc cleanup.
  - The final broad workspace `cargo test --release -q` gate passed end to
    end on the current tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times
against this slice, but the available runtime path still did not produce a
review:

- Attempt 1
  - `timeout 180s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-10-PLAN.md current_task="Move all core-owned live YAML under z00z_config" --yolo'`
  - Result: timed out with exit `124` and no output
- Attempt 2
  - `timeout 180s gsd --extension .github --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-10-PLAN.md current_task="Move all core-owned live YAML under z00z_config" --yolo'`
  - Result: timed out with exit `124` and no output
- Attempt 3
  - `timeout 180s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-10-PLAN.md current_task="Move all core-owned live YAML under z00z_config" --yolo'`
  - Result: timed out with exit `124` and no output

Equivalent review passes were executed manually against the same scope under
the prompt's review contract and the repository `doublecheck` expectations.

- Pass 1
  - Rechecked the post-migration core YAML file list, the example
    `domain_name` pattern contract, and the Phase 063 core/wiki stale-root
    scans
  - Result: clean for the core authority surface
- Pass 2
  - Reviewed the downstream simulator and wallet diffs plus a wider stale-path
    scan across core, simulator, wallets, and docs
  - Result: found one residual stale authority note in
    `docs/tech-papers/done/Z00Z-HJMT-Design.md`; the note was updated to the
    canonical `z00z_config` paths
- Pass 3
  - Re-ran the wider stale-path scan after the doc fix
  - Result: clean
- Pass 4
  - Rechecked the wallet live-anchor vocabulary scan and diff-hygiene gate
  - Result: clean

Passes 3 and 4 were consecutive clean review passes for the modified `063-10`
scope.

## Completion Notes

- `063-10-SUMMARY.md` closes `PLAN-063-G10` and advances the execution lane to
  `063-11-PLAN.md`.
- `crates/z00z_core/z00z_config/` is now the sole live core YAML root.
- `z00z_core::config_paths` is now the canonical helper surface for live core
  YAML path and filename access.
- The simulator, wallet docs, wallet catalog contract, and touched techpaper
  notes now share the same live core config vocabulary.
