---
phase: 063-Core-Update
artifact: tests-tasks
status: complete
source: live code and tests, 063-TEST-SPEC.md, 063-TODO.md, 063-CONTEXT.md, 063-core-examples.md, 063-01-PLAN.md..063-13-PLAN.md
updated: 2026-06-29
---

<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD041 MD047 -->

# Phase 063 Test Tasks

## 🎯 Purpose

This file is the executable verification checklist for the live Phase 063 test
packet. It is not a future-work queue anymore.

Use it to verify the implementation directly from code, tests, file trees, and
release-mode commands. Do not treat summaries as proof.

## 📌 Execution Rules

- Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  first as the mandatory fail-fast gate.
- Run cargo validation only in `--release` mode.
- Treat `063-core-examples.md` as live requirement input wherever examples,
  `wallet_profiles[]`, or `policy_profiles[]` are involved.
- Keep one canonical path for each module, function family, test owner, YAML
  root, and vouchers namespace.
- Do not introduce parallel legacy shims while verifying or repairing the
  packet.

## ✅ Ordered Verification Tasks

| Step | Scenario / requirement | Canonical proof homes | Required commands or scans | Closure signal |
| --- | --- | --- | --- | --- |
| `063-00` | Packet integrity | `063-TODO.md`, `063-CONTEXT.md`, `063-TEST-SPEC.md`, `063-TESTS-TASKS.md`, `063-01-PLAN.md..063-13-PLAN.md` | `bash -lc 'test "$(rg -n "^### P[0-2]\\." .planning/phases/063-Core-Update/063-TODO.md | wc -l)" -eq 13'`; `bash -lc 'test "$(rg --no-filename -o "REC-063-[A-Z0-9-]+" .planning/phases/063-Core-Update/063-CONTEXT.md .planning/phases/063-Core-Update/063-TEST-SPEC.md .planning/phases/063-Core-Update/063-TESTS-TASKS.md | sort -u | wc -l)" -eq 13'`; `bash -lc 'test "$(rg --no-filename -o "PLAN-063-G[0-9]{2}" .planning/phases/063-Core-Update/063-TEST-SPEC.md .planning/phases/063-Core-Update/063-TESTS-TASKS.md | sort -u | wc -l)" -eq 13'`; `bash -lc 'test "$(rg --no-filename -o "063-S(0[1-9]|1[0-3])" .planning/phases/063-Core-Update/063-TEST-SPEC.md | sort -u | wc -l)" -eq 13'`; `rg -n "063-core-examples\\.md" .planning/phases/063-Core-Update/063-TEST-SPEC.md .planning/phases/063-Core-Update/063-TESTS-TASKS.md` | 13 plans, 13 scenarios, 13 `REC-063-*` entries, and one canonical example corpus attachment remain intact. |
| `063-01` | `063-S01` / `PLAN-063-G01` / `REC-063-P0-01` | `tests/test_genesis_manifest_refs.rs`, `tests/test_genesis_manifest_goldens.rs`, `tests/test_genesis_validation.rs`, `src/genesis/genesis_run.rs` | `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`; `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`; `cargo test --release -p z00z_core --test genesis_tests validation -- --nocapture`; `rg -n "num_threads|snapshot_export_path|assets_export_path" crates/z00z_core/src/genesis crates/z00z_core/tests` | Parsed runtime knobs, output semantics, and manifest stability match live behavior. |
| `063-02` | `063-S02` / `PLAN-063-G02` / `REC-063-P0-02` | `src/genesis/mod.rs`, `src/genesis/genesis.rs`, `src/genesis/validator.rs`, `tests/test_live_guardrails.rs` | `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`; `rg -n "include!\(|z00z_core::genesis::genesis::" crates/z00z_core/src/genesis crates/z00z_core/tests crates/z00z_core/docs crates/z00z_core/README.md` | The shallow public genesis path is canonical and no boundary-defining `include!` or deep public owner path has returned. |
| `063-03` | `063-S03` / `PLAN-063-G03` / `REC-063-P0-03` | `README.md`, `src/lib.rs`, `src/assets/mod.rs`, `src/genesis/mod.rs`, `tests/test_live_guardrails.rs` | `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`; `rg -n "assets_generator|utils_traits|max_supply|genesis_Z00Z|state, tx, and validation" crates/z00z_core` | Live docs and guard tests agree on the current public surface. |
| `063-04` | `063-S04` / `PLAN-063-G04` / `REC-063-P1-01` | `src/assets/registry_config.rs`, `src/genesis/genesis_config.rs`, `tests/test_assets_config_integration.rs`, `src/genesis/README.md` | `cargo test --release -p z00z_core --test assets_tests config_integration -- --nocapture`; `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`; `rg -n "AssetDefinitionRegistry::load_catalog_from_yaml|load_catalog_from_yaml|registry_config|GenesisConfig" crates/z00z_core/src crates/z00z_core/tests crates/z00z_core/README.md` | Secondary registry vocabulary is clearly subordinate and `GenesisConfig` remains the sole bootstrap authority. |
| `063-05` | `063-S05` / `PLAN-063-G05` / `REC-063-P1-02` | `tests/test_genesis_manifest.rs`, `tests/test_genesis_manifest_refs.rs`, `tests/test_genesis_manifest_goldens.rs`, `src/genesis/*` | `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_full_bootstrap_matches_legacy_run -- --nocapture`; `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_assets_only_skips_rights_validation -- --nocapture`; `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_rights_only_requires_policy_resolution_when_needed -- --nocapture`; `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_vouchers_only_rejects_non_voucher_policy -- --nocapture`; `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_partial_run_does_not_emit_full_settlement_manifest -- --nocapture`; `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_selected_lanes_preserve_terminal_collision_checks -- --nocapture` | Full-bootstrap parity, lane-scoped receipts, and fail-closed partial generation are proven on live code. |
| `063-06` | `063-S06` / `PLAN-063-G06` / `REC-063-P1-03` | `tests/test_assets_registry_integration.rs`, `src/genesis/genesis_run.rs`, `z00z_simulator/src/scenario_1/stage_1/mod.rs` | `cargo test --release -p z00z_core --test assets_tests registry_integration::test_registry_explicit_owner_stays_local_until_global_sync -- --nocapture`; `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_scenario1_object_flows_matrix_contract -- --nocapture`; `rg -n "GLOBAL_ASSET_REGISTRY|ctx\\.registry" crates/z00z_core crates/z00z_simulator` | Explicit registry ownership stays authoritative and the global sync boundary stays narrow. |
| `063-07` | `063-S07` / `PLAN-063-G07` / `REC-063-P1-04` | `src/vouchers/mod.rs`, `src/vouchers/test_voucher_config.rs`, `tests/test_genesis_vouchers.rs`, downstream imports | `cargo test --release -p z00z_core test_voucher_config_validates_shape -- --nocapture`; `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests genesis_vouchers -- --nocapture`; `rg -n "VoucherBootstrapEntryV1" crates/z00z_core crates/z00z_wallets crates/z00z_storage crates/z00z_simulator`; `rg -n "\\bvauchers\\b" crates/z00z_core crates/z00z_wallets crates/z00z_storage crates/z00z_simulator crates/z00z_core/docs crates/z00z_core/README.md` | Only `z00z_core::vouchers` remains live and bootstrap-vs-runtime voucher meaning is unchanged. |
| `063-08` | `063-S08` / `PLAN-063-G08` / `REC-063-P1-05` | `docs/OBJECT_FAMILY_SEMANTICS.md`, `src/genesis/README.md`, `object_rpc_impl.rs`, `tx_plan_types.rs`, `test_scenario1_object_flows.rs` | `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`; `cargo test --release -p z00z_wallets rpc::methods::asset_rpc_impl::test_asset_impl::test_object_rpc_lists_inventory -- --nocapture`; `rg -n "VoucherBootstrapEntryV1|FeeEnvelope|mintable|required_signatures|primary_family|backing" crates/z00z_core crates/z00z_wallets crates/z00z_storage` | Every family row stays mapped to live anchors and no ambiguous semantics return. |
| `063-09` | `063-S09` / `PLAN-063-G09` / `REC-063-P1-06` | `test_scenario1_object_flows.rs`, `test_hjmt_e2e.rs`, `object_rpc.rs`, `test_asset_import_security.rs` | `cargo test --release -p z00z_wallets --test test_asset_import_security test_rejects_cash_inventory_write -- --nocapture`; `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_scenario1_object_flows_matrix_contract -- --nocapture`; `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_rights_business_entitlement_lifecycle_local -- --nocapture`; `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_agentic_right_lifecycle_local -- --nocapture`; `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_machine_capability_lifecycle_local -- --nocapture`; `rg -n "wallet\\.object\\.list_rights|wallet\\.object\\.consume_right" crates/z00z_wallets/src/rpc`; `rg -n "cash import must not appear in wallet\\.object\\.list_rights|validator_lock_unlock_after_expiry|validator_lock_unlock_without_right_delta_reject|validator_lock_unlock_replay_reject" crates/z00z_wallets/tests crates/z00z_simulator/tests/scenario_1`; `rg -n "useful[-_]work|live cross-chain|linked liability|live external enforcement|full-wallet spend|broad controller|second cash authority|universal private VM" crates/z00z_core crates/z00z_core/docs crates/z00z_core/README.md wiki/03-core-protocol` | Positive and negative object-family proofs stay live while unsupported broad claims remain absent. |
| `063-10` | `063-S10` / `PLAN-063-G10` / `REC-063-P1-07` | `z00z_config/*`, `src/config_paths.rs`, `src/genesis/manifest_ref_loader.rs`, `tests/test_genesis_manifest_refs.rs`, `tests/test_genesis_manifest_goldens.rs` | `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`; `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`; `cargo test --release -p z00z_core --test assets_tests rights_config::test_rights_config_loads -- --nocapture`; `cargo test --release -p z00z_core test_voucher_config_validates_shape -- --nocapture`; `find crates/z00z_core -path '*/target' -prune -o \\( -name '*.yaml' -o -name '*.yml' \\) -type f | sort`; `rg -n "src/(assets|genesis)/.*\\.ya?ml|crates/z00z_core/configs|tests/vectors" crates/z00z_core`; `rg -n "genesis_config_devnet_small|devnet_actions_config|wallet_profiles|policy_profiles" crates/z00z_core/docs crates/z00z_core/README.md crates/z00z_core/src crates/z00z_core/tests` | Only `z00z_config` remains live and parser-owned profile sections stay typed and validated. |
| `063-11` | `063-S11` / `PLAN-063-G11` / `REC-063-P2-01` | `Cargo.toml`, `tests/test_genesis.rs`, `tests/test_genesis_mod.rs`, `tests/test_assets.rs`, `tests/test_assets_mod.rs`, flat root test files | `cargo test --release -p z00z_core --tests`; `find crates/z00z_core/tests -mindepth 2 -type f ! -path '*/tests/fixtures/*'`; `rg -n "test_.*_suite\\.rs|tests/(assets|genesis|rights|vauchers|vectors)|\\#\\[path = \"\\.\\./assets/" crates/z00z_core/Cargo.toml crates/z00z_core/tests crates/z00z_core/src` | The core integration tree is flat, canonically named, and free of stale owner-path strings. |
| `063-12` | `063-S12` / `PLAN-063-G12` / `REC-063-P2-02` | `docs/**`, `README.md`, `tests/test_live_guardrails.rs`, support Markdown under `benches`, `bin`, `examples`, `tests` | `cargo doc --release -p z00z_core --no-deps`; `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`; `rg -n "examples/.+/.+|benches/.+/.+|bin/.+/.+|src/assets/.*\\.ya?ml|src/genesis/.*\\.ya?ml|crates/z00z_core/configs|tests/vectors" crates/z00z_core/docs crates/z00z_core/README.md crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples crates/z00z_core/tests -g '*.md'`; `rg -n "[А-Яа-я]" crates/z00z_core/docs crates/z00z_core/README.md crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples crates/z00z_core/tests -g '*.md'` | Public and support-surface docs stay truthful, flat-rooted, and ASCII English. |
| `063-13` | `063-S13` / `PLAN-063-G13` / `REC-063-P2-03` | `Cargo.toml`, `benches/*`, `bin/*`, `examples/*`, `z00z_config/*` | `cargo bench -p z00z_core --no-run`; `cargo build --release -p z00z_core --bins`; `cargo test --release -p z00z_core --examples`; `find crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples -mindepth 2 -type f`; `rg -n "path = \"(benches|bin|examples)/.*/" crates/z00z_core/Cargo.toml`; `rg -n "z00z_config|\\.ya?ml" crates/z00z_core/examples crates/z00z_core/bin crates/z00z_core/benches` | Support surfaces are flat, buildable, and keep one explicit config and feature-boundary story. |

## 🔁 Shared Validation Rules

| Rule | Requirement |
| --- | --- |
| Bootstrap fail-fast | Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first. If it fails, stop, fix, rerun, and only then continue. |
| Release-only cargo | Use `--release` for every cargo command in this packet. |
| Broad Rust gate | Run `cargo test --release` after Rust, tests, docs guards, config loaders, simulator behavior, or verification scripts change. |
| Review repetition | Run `./.github/prompts/gsd-review-tasks-execution.prompt.md` (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times and continue until at least `2` consecutive runs show no significant issues. |
| Commit discipline | If a commit is required after verification, use `/z00z-git-versioning`. |
| Evidence discipline | Preserve the exact command, scan, or file listing that proves the closure signal. |

## 🚫 Reject Conditions

| Condition | Why it fails |
| --- | --- |
| A second bootstrap, YAML, vouchers, object-RPC, or support-surface authority appears | It breaks the Phase 063 canonical-path contract. |
| `vauchers` reappears anywhere live | The namespace normalization is no longer complete. |
| A partial lane emits the full settlement manifest | It makes non-canonical output look canonical. |
| `wallet_profiles[]` or `policy_profiles[]` become untyped or silently ignored | It reintroduces dead live config. |
| Nested non-fixture test files return under `crates/z00z_core/tests/` | It breaks the flat test-owner contract. |
| Stale YAML roots or stale doc paths return | It breaks the single-root and single-path story. |

<verify>

1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
   first.
2. Run the step-owned narrow commands and scans above.
3. Run `cargo test --release` when the touched slice changes Rust, tests, docs,
   config loaders, simulator behavior, or verification scripts.
4. Run `./.github/prompts/gsd-review-tasks-execution.prompt.md`
   (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times. Fix all
   issues and warnings and continue until at least `2` consecutive runs show
   no significant issues.
5. If a commit is required after verification, use `/z00z-git-versioning`.

## ✅ Exit Conditions

- All `PLAN-063-G01` through `PLAN-063-G13` remain matched to live
  `063-S01` through `063-S13` verification work.
- All `REC-063-*` items remain backed by executable commands, scans, or build
  gates.
- The next verifier can rerun the packet without guessing test targets, file
  homes, YAML roots, or canonical-path strings.
