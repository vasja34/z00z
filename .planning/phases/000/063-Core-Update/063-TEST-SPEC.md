---
phase: 063-Core-Update
artifact: test-spec
status: complete
source: live code and tests, 063-TODO.md, 063-CONTEXT.md, 063-core-examples.md, 063-01-PLAN.md..063-13-PLAN.md
updated: 2026-06-29
---

<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD041 MD047 -->

# Phase 063 Test Spec

## 🎯 Purpose

This file is the live Phase 063 test authority. It is no longer a
`fallback-ready` or future-only planning artifact.

Use these inputs in this order:

1. `063-TODO.md` for scope and acceptance language.
2. `063-01-PLAN.md` through `063-13-PLAN.md` for scenario ownership.
3. `063-core-examples.md` for mandatory example anchors and parser-owned
   profile sections.
4. Live code and live tests for the implementation truth.

Future-oriented design wording from older drafts is retired here. If a term,
path, module, or test home is listed below, it is expected to match the live
repository now.

## 📌 Live Status

- The canonical core YAML root is `crates/z00z_core/z00z_config/**`.
- The canonical core test tree is flat under `crates/z00z_core/tests/` with
  `tests/fixtures/` as the only owned nested support directory.
- The canonical vouchers namespace is `z00z_core::vouchers`.
- Lane-selection coverage is live in
  `crates/z00z_core/tests/test_genesis_manifest.rs`.
- Explicit-registry ownership coverage is live in
  `crates/z00z_core/tests/test_assets_registry_integration.rs`.
- Documentation guard coverage is live in
  `crates/z00z_core/tests/test_live_guardrails.rs`.
- Bounded object-family simulator coverage is live in
  `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs` and
  `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`.

## 🔑 Canonical Roots

| Area | Canonical path or owner |
| --- | --- |
| Core YAML root | `crates/z00z_core/z00z_config/**` |
| Core assets entry | `crates/z00z_core/tests/test_assets.rs` -> `tests/test_assets_mod.rs` |
| Core genesis entry | `crates/z00z_core/tests/test_genesis.rs` -> `tests/test_genesis_mod.rs` |
| Doc guard entry | `crates/z00z_core/tests/test_live_guardrails.rs` |
| Genesis manifest references | `crates/z00z_core/tests/test_genesis_manifest_refs.rs` |
| Genesis manifest goldens | `crates/z00z_core/tests/test_genesis_manifest_goldens.rs` |
| Lane-selection and partial-manifest coverage | `crates/z00z_core/tests/test_genesis_manifest.rs` |
| Registry ownership coverage | `crates/z00z_core/tests/test_assets_registry_integration.rs` |
| Rights config coverage | `crates/z00z_core/tests/test_rights_config.rs` via `assets_tests` |
| Voucher config coverage | `crates/z00z_core/src/vouchers/test_voucher_config.rs` |
| Simulator object-family coverage | `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs` |
| Simulator right lifecycle coverage | `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs` |
| Wallet object RPC authority | `crates/z00z_wallets/src/rpc/object_rpc.rs` and `object_rpc_impl.rs` |
| Support surfaces | `crates/z00z_core/benches/*`, `crates/z00z_core/bin/*`, `crates/z00z_core/examples/*` |

## 🔒 Required Invariants

- `z00z_core::genesis::*` stays the only public bootstrap caller path.
- `GenesisConfig` stays the only bootstrap authority.
- `performance.num_threads` is live behavior, not dead config.
- Output semantics remain singular and deterministic.
- Partial lanes must emit lane-scoped receipts and must not emit the full
  settlement manifest.
- `GLOBAL_ASSET_REGISTRY` is fallback only; explicit registries remain the
  authoritative write owner inside generation and simulator flows.
- `vauchers` must stay absent from live code, docs, tests, and examples.
- `VoucherBootstrapEntryV1` must remain bootstrap-only input, not runtime
  voucher state.
- `wallet.object.list_rights` and `wallet.object.consume_right` remain typed
  right inventory and right-consumption surfaces only.
- Broad unsupported claims stay absent from live docs and APIs.
- `wallet_profiles[]` and `policy_profiles[]` remain typed, validated, and
  parser-owned live sections when present.
- Bench, bin, example, and test trees stay flat and canonical.

## ✅ Scenario Matrix

| Scenario | Plan / requirement | Live proof homes | What must stay true | What must fail closed |
| --- | --- | --- | --- | --- |
| `063-S01` | `PLAN-063-G01` / `REC-063-P0-01` | `src/genesis/genesis_run.rs`, `tests/test_genesis_manifest_refs.rs`, `tests/test_genesis_manifest_goldens.rs`, `tests/test_genesis_validation.rs` | Parsed runtime settings drive live genesis behavior and emitted artifacts. | Drift between parsed config, output roots, and emitted manifest artifacts. |
| `063-S02` | `PLAN-063-G02` / `REC-063-P0-02` | `src/genesis/mod.rs`, `src/genesis/genesis.rs`, `src/genesis/validator.rs`, `tests/test_live_guardrails.rs`, repo-wide scans | Only the shallow `z00z_core::genesis::*` path remains canonical. | Any return of deep public owner paths or boundary-defining `include!` assembly. |
| `063-S03` | `PLAN-063-G03` / `REC-063-P0-03` | `README.md`, `src/lib.rs`, `src/assets/mod.rs`, `src/genesis/mod.rs`, `tests/test_live_guardrails.rs` | Public docs describe the live crate surface truthfully. | Stale strings, stale modules, or examples that only look valid because doctests are disabled. |
| `063-S04` | `PLAN-063-G04` / `REC-063-P1-01` | `src/assets/registry_config.rs`, `src/genesis/genesis_config.rs`, `tests/test_assets_config_integration.rs`, `README.md`, `src/genesis/README.md` | Secondary registry vocabulary stays visibly subordinate to `GenesisConfig`. | Any code or docs that make registry YAML look like co-equal bootstrap authority. |
| `063-S05` | `PLAN-063-G05` / `REC-063-P1-02` | `tests/test_genesis_manifest.rs`, `tests/test_genesis_manifest_refs.rs`, `tests/test_genesis_manifest_goldens.rs`, `src/genesis/*` | Full bootstrap parity and partial-lane behavior coexist under one authority plane. | Partial lanes emitting `genesis_settlement_manifest.json`, wrong-family voucher policies, or skipped collision checks. |
| `063-S06` | `PLAN-063-G06` / `REC-063-P1-03` | `tests/test_assets_registry_integration.rs`, `src/genesis/genesis_run.rs`, `z00z_simulator/src/scenario_1/stage_1/mod.rs` | Explicit registry owners stay authoritative until a narrow global sync boundary. | Co-equal dual writes or hidden simulator dependence on global side effects. |
| `063-S07` | `PLAN-063-G07` / `REC-063-P1-04` | `src/vouchers/mod.rs`, `src/vouchers/test_voucher_config.rs`, `tests/test_genesis_vouchers.rs`, downstream imports across wallets, storage, and simulator | `z00z_core::vouchers` is the only live namespace and voucher bootstrap meaning is unchanged. | Any surviving `vauchers` caller, alias, shim, or semantic drift between bootstrap and runtime voucher roles. |
| `063-S08` | `PLAN-063-G08` / `REC-063-P1-05` | `docs/OBJECT_FAMILY_SEMANTICS.md`, `src/genesis/README.md`, `z00z_wallets/src/rpc/object_rpc_impl.rs`, `z00z_storage/src/settlement/tx_plan_types.rs`, `test_scenario1_object_flows.rs` | Assets, rights, vouchers, policies, and `FeeEnvelope` keep one canonical semantics matrix with live anchors. | Ambiguous lifecycle wording, value-role drift, or collapsing vouchers into `asset + right`. |
| `063-S09` | `PLAN-063-G09` / `REC-063-P1-06` | `test_scenario1_object_flows.rs`, `test_hjmt_e2e.rs`, `z00z_wallets/src/rpc/object_rpc.rs`, `z00z_wallets/tests/test_asset_import_security.rs` | Positive and negative object-family flows remain explicit and bounded. | Cash authority leakage into object RPC or live claims for unsupported broad capability scope. |
| `063-S10` | `PLAN-063-G10` / `REC-063-P1-07` | `z00z_config/*`, `src/config_paths.rs`, `src/genesis/manifest_ref_loader.rs`, `tests/test_genesis_manifest_refs.rs`, `tests/test_genesis_manifest_goldens.rs` | One root-owned YAML authority exists and parser-owned profile sections remain live. | Old YAML roots, stale filenames, or silently ignored live fields. |
| `063-S11` | `PLAN-063-G11` / `REC-063-P2-01` | `Cargo.toml`, `tests/test_genesis.rs`, `tests/test_genesis_mod.rs`, `tests/test_assets.rs`, `tests/test_assets_mod.rs`, `tests/test_rights_config.rs`, `tests/test_genesis_rights.rs`, `tests/test_genesis_vouchers.rs` | Core integration tests stay flat and canonically named. | Nested non-fixture test owners, `*_suite.rs`, or stale cross-owner path wiring. |
| `063-S12` | `PLAN-063-G12` / `REC-063-P2-02` | `docs/**`, `README.md`, `tests/test_live_guardrails.rs`, `docs/GENESIS_DOCUMENTATION.md` | Docs and support Markdown stay ASCII English and point only at live paths and APIs. | Stale path references, nested support-surface drift, or non-ASCII repo artifacts. |
| `063-S13` | `PLAN-063-G13` / `REC-063-P2-03` | `Cargo.toml`, `benches/*`, `bin/*`, `examples/*`, `z00z_config/*` | Benches, bins, examples, and CLI/export feature boundaries stay flat and explicit. | Nested support trees, example-local YAML, or hidden feature-boundary drift. |

## 🧪 Canonical Commands

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`
- `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`
- `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_full_bootstrap_matches_legacy_run -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_partial_run_does_not_emit_full_settlement_manifest -- --nocapture`
- `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_selected_lanes_preserve_terminal_collision_checks -- --nocapture`
- `cargo test --release -p z00z_core --test assets_tests registry_integration::test_registry_explicit_owner_stays_local_until_global_sync -- --nocapture`
- `cargo test --release -p z00z_core --test assets_tests rights_config::test_rights_config_loads -- --nocapture`
- `cargo test --release -p z00z_core test_voucher_config_validates_shape -- --nocapture`
- `cargo test --release -p z00z_wallets rpc::methods::asset_rpc_impl::test_asset_impl::test_object_rpc_lists_inventory -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_asset_import_security test_rejects_cash_inventory_write -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_scenario1_object_flows_matrix_contract -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_rights_business_entitlement_lifecycle_local -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_agentic_right_lifecycle_local -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_machine_capability_lifecycle_local -- --nocapture`
- `cargo doc --release -p z00z_core --no-deps`
- `cargo bench -p z00z_core --no-run`
- `cargo build --release -p z00z_core --bins`
- `cargo test --release -p z00z_core --examples`
- `cargo test --release`

## 🔎 Canonical Scans

- `find crates/z00z_core -path '*/target' -prune -o \( -name '*.yaml' -o -name '*.yml' \) -type f | sort`
- `find crates/z00z_core/tests -mindepth 2 -type f ! -path '*/tests/fixtures/*'`
- `find crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples -mindepth 2 -type f`
- `rg -n "\bvauchers\b" crates/z00z_core crates/z00z_wallets crates/z00z_storage crates/z00z_simulator crates/z00z_core/docs crates/z00z_core/README.md`
- `rg -n "include!\(|z00z_core::genesis::genesis::" crates/z00z_core/src/genesis crates/z00z_core/tests crates/z00z_core/docs crates/z00z_core/README.md`
- `rg -n "tests/(genesis|assets|rights|vauchers)|test_.*_suite\.rs" crates/z00z_core/docs crates/z00z_core/src crates/z00z_core/README.md`
- `rg -n "examples/.+/.+|benches/.+/.+|bin/.+/.+|src/assets/.*\.ya?ml|src/genesis/.*\.ya?ml|crates/z00z_core/configs|tests/vectors" crates/z00z_core/docs crates/z00z_core/README.md crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples crates/z00z_core/tests -g '*.md'`
- `rg -n "[А-Яа-я]" crates/z00z_core/docs crates/z00z_core/README.md crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples crates/z00z_core/tests -g '*.md'`
- `rg -n "wallet_profiles\\[\\]|policy_profiles\\[\\]" .planning/phases/063-Core-Update/063-core-examples.md crates/z00z_core/src crates/z00z_core/tests crates/z00z_core/docs crates/z00z_core/README.md`
- `rg -n "useful[-_]work|live cross-chain|linked liability|live external enforcement|full-wallet spend|broad controller|second cash authority|universal private VM" crates/z00z_core crates/z00z_core/docs crates/z00z_core/README.md wiki/03-core-protocol`

<verify>

1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
   first. If it fails, stop, fix, and rerun it before any broader validation.
2. Re-run packet-consistency checks:
   `bash -lc 'test "$(rg -n "^### P[0-2]\\." .planning/phases/063-Core-Update/063-TODO.md | wc -l)" -eq 13'`
   `bash -lc 'test "$(rg --no-filename -o "REC-063-[A-Z0-9-]+" .planning/phases/063-Core-Update/063-CONTEXT.md .planning/phases/063-Core-Update/063-TEST-SPEC.md .planning/phases/063-Core-Update/063-TESTS-TASKS.md | sort -u | wc -l)" -eq 13'`
   `bash -lc 'test "$(rg --no-filename -o "PLAN-063-G[0-9]{2}" .planning/phases/063-Core-Update/063-TEST-SPEC.md .planning/phases/063-Core-Update/063-TESTS-TASKS.md | sort -u | wc -l)" -eq 13'`
   `bash -lc 'test "$(rg --no-filename -o "063-S(0[1-9]|1[0-3])" .planning/phases/063-Core-Update/063-TEST-SPEC.md | sort -u | wc -l)" -eq 13'`
3. Run the scenario-owned narrow tests and scans listed above, then run
   `cargo test --release` when Rust, tests, docs, config loaders, simulator
   behavior, or verification scripts changed.
4. Run `./.github/prompts/gsd-review-tasks-execution.prompt.md`
   (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times against the
   touched scenario or the full Phase 063 packet. Fix all issues and warnings
   and continue until at least `2` consecutive runs show no significant issues.
5. If a commit is required after verification, use `/z00z-git-versioning`.

## ✅ Completion Criteria

- All `PLAN-063-G01` through `PLAN-063-G13` remain represented by live
  `063-S01` through `063-S13` proof homes.
- All `REC-063-*` requirements remain mapped to executable tests, scans, or
  release build gates.
- No stale canonical-path strings survive in the Phase 063 test packet.
- No parallel bootstrap, YAML, vouchers, object-RPC, or support-surface
  authority is introduced by the test packet.
