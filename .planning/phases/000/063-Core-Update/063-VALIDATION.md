---
phase: 063
slug: 063-core-update
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-29
---

# Phase 063 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` / rustdoc / cargo bench / cargo build |
| **Config file** | `crates/z00z_core/Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release` |
| **Estimated runtime** | ~3000 seconds |

---

## Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run the Phase 063 narrow release packet plus `cargo test --release` when Rust, tests, docs, config loaders, simulator behavior, or verification scripts change
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 900 seconds for the narrow packet

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `063-S01` | `PLAN-063-G01` | 1 | `REC-063-P0-01` | `T-063-01`, `T-063-02`, `T-063-03` | Thread-count and output-root contracts stay live and deterministic. | integration | `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`<br>`cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture` | ✅ | ✅ green |
| `063-S02` | `PLAN-063-G02` | 1 | `REC-063-P0-02` | `T-063-04`, `T-063-05` | Only `z00z_core::genesis::*` remains the canonical public bootstrap path. | integration + scan | `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`<br>`rg -n "include!\\(|z00z_core::genesis::genesis::" crates/z00z_core/src/genesis crates/z00z_core/tests crates/z00z_core/docs crates/z00z_core/README.md` | ✅ | ✅ green |
| `063-S03` | `PLAN-063-G03` | 1 | `REC-063-P0-03` | `T-063-06`, `T-063-07` | Public docs stay aligned with the live crate surface. | doc + integration | `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`<br>`rg -n "assets_generator|utils_traits|max_supply|genesis_Z00Z|state, tx, and validation" crates/z00z_core` | ✅ | ✅ green |
| `063-S04` | `PLAN-063-G04` | 2 | `REC-063-P1-01` | `T-063-08` | Secondary registry YAML stays visibly subordinate to `GenesisConfig`. | integration | `cargo test --release -p z00z_core --test assets_tests config_integration -- --nocapture`<br>`cargo test --release -p z00z_core --test assets_tests rights_config::test_rights_config_loads -- --nocapture` | ✅ | ✅ green |
| `063-S05` | `PLAN-063-G05` | 2 | `REC-063-P1-02` | `T-063-09`, `T-063-10`, `T-063-11` | Full bootstrap parity and partial-lane fail-closed behavior stay under one authority plane. | integration | `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_full_bootstrap_matches_legacy_run -- --nocapture`<br>`cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_partial_run_does_not_emit_full_settlement_manifest -- --nocapture`<br>`cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_selected_lanes_preserve_terminal_collision_checks -- --nocapture` | ✅ | ✅ green |
| `063-S06` | `PLAN-063-G06` | 2 | `REC-063-P1-03` | `T-063-12` | Explicit registry owners remain authoritative until the narrow global sync boundary. | integration | `cargo test --release -p z00z_core --test assets_tests registry_integration::test_registry_explicit_owner_stays_local_until_global_sync -- --nocapture`<br>`cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_scenario1_object_flows_matrix_contract -- --nocapture` | ✅ | ✅ green |
| `063-S07` | `PLAN-063-G07` | 2 | `REC-063-P1-04` | `T-063-13` | `z00z_core::vouchers` stays the only live vouchers namespace. | unit + scan | `cargo test --release -p z00z_core test_voucher_config_validates_shape -- --nocapture`<br>`rg -n "\\bvauchers\\b" crates/z00z_core crates/z00z_wallets crates/z00z_storage crates/z00z_simulator crates/z00z_core/docs crates/z00z_core/README.md` | ✅ | ✅ green |
| `063-S08` | `PLAN-063-G08` | 2 | `REC-063-P1-05` | `T-063-14` | The object-family semantics matrix remains pinned to live runtime anchors. | doc + unit + scan | `cargo test --release -p z00z_wallets rpc::methods::asset_rpc_impl::test_asset_impl::test_object_rpc_lists_inventory -- --nocapture`<br>`rg -n "VoucherBootstrapEntryV1|FeeEnvelope|mintable|required_signatures|primary_family|backing" crates/z00z_core crates/z00z_wallets crates/z00z_storage` | ✅ | ✅ green |
| `063-S09` | `PLAN-063-G09` | 2 | `REC-063-P1-06` | `T-063-15` | Object-family positive and negative flows stay bounded with no cash leakage into object RPC. | integration + scan | `cargo test --release -p z00z_wallets --test test_asset_import_security test_rejects_cash_inventory_write -- --nocapture`<br>`cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_scenario1_object_flows_matrix_contract -- --nocapture` | ✅ | ✅ green |
| `063-S10` | `PLAN-063-G10` | 3 | `REC-063-P1-07` | `T-063-16`, `T-063-17` | `z00z_config/` remains the only core YAML root and parser-owned profiles stay live. | integration + scan | `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`<br>`find crates/z00z_core -path '*/target' -prune -o \\( -name '*.yaml' -o -name '*.yml' \\) -type f | sort` | ✅ | ✅ green |
| `063-S11` | `PLAN-063-G11` | 3 | `REC-063-P2-01` | `T-063-18` | Core integration tests stay flat and canonically named. | integration + scan | `cargo test --release -p z00z_core --tests`<br>`find crates/z00z_core/tests -mindepth 2 -type f ! -path '*/tests/fixtures/*'` | ✅ | ✅ green |
| `063-S12` | `PLAN-063-G12` | 4 | `REC-063-P2-02` | `T-063-19` | Docs and support Markdown stay ASCII English and point only to live paths. | doc + scan | `cargo doc --release -p z00z_core --no-deps`<br>`rg -n "[А-Яа-я]" crates/z00z_core/docs crates/z00z_core/README.md crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples crates/z00z_core/tests -g '*.md'` | ✅ | ✅ green |
| `063-S13` | `PLAN-063-G13` | 4 | `REC-063-P2-03` | `T-063-20` | Benches, bins, examples, and feature boundaries stay flat and buildable. | build + scan | `cargo bench -p z00z_core --no-run`<br>`cargo build --release -p z00z_core --bins`<br>`cargo test --release -p z00z_core --examples` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Audit 2026-06-29

| Metric | Count |
|--------|-------|
| Gaps found | 2 |
| Resolved | 2 |
| Escalated | 0 |

### Gap Resolution

- Replaced the invalid support-surface bench invocation with the correct `cargo bench -p z00z_core --no-run` command in `063-TEST-SPEC.md` and `063-TESTS-TASKS.md`.
- Tightened the wallet object-flow proof from an over-broad cargo filter to the exact unit and integration commands now recorded in `063-TEST-SPEC.md` and `063-TESTS-TASKS.md`.

---

## Verification Notes

- `gsd-tools` hook discovery returned no active `validate-phase` `verify:post` hook in the local GSD config, so the Nyquist audit was executed directly from repository artifacts and live release-mode commands.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed before any broader validation.
- `cargo test --release` passed on the current tree after the validation packet rerun.
- Packet-consistency checks stayed green with all `13` plans, `13` scenarios, and `13` `REC-063-*` references present.
- Canonical scans stayed clean:
  - nested non-fixture core tests: `0`
  - nested support-surface files: `0`
  - `vauchers` hits: `0`
  - deep genesis owner-path / `include!` hits: `0`
  - stale test-path hits: `0`
  - stale doc-path hits: `0`
  - non-ASCII doc hits: `0`
  - unsupported broad-claim hits: `0`
- Core YAML roots remained limited to the `8` files under `crates/z00z_core/z00z_config/`.
- Supporting release commands that passed during this audit included:
  - `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`
  - `cargo test --release -p z00z_core --test test_genesis_manifest_refs -- --nocapture`
  - `cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_full_bootstrap_matches_legacy_run -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_assets_only_skips_rights_validation -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_rights_only_requires_policy_resolution_when_needed -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_plan_vouchers_only_rejects_non_voucher_policy -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_partial_run_does_not_emit_full_settlement_manifest -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_selected_lanes_preserve_terminal_collision_checks -- --nocapture`
  - `cargo test --release -p z00z_core --test assets_tests config_integration -- --nocapture`
  - `cargo test --release -p z00z_core --test assets_tests rights_config::test_rights_config_loads -- --nocapture`
  - `cargo test --release -p z00z_core --test assets_tests registry_integration::test_registry_explicit_owner_stays_local_until_global_sync -- --nocapture`
  - `cargo test --release -p z00z_wallets rpc::methods::asset_rpc_impl::test_asset_impl::test_object_rpc_lists_inventory -- --nocapture`
  - `cargo test --release -p z00z_wallets --test test_asset_import_security test_rejects_cash_inventory_write -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_scenario1_object_flows_matrix_contract -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_rights_business_entitlement_lifecycle_local -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_agentic_right_lifecycle_local -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_machine_capability_lifecycle_local -- --nocapture`
  - `cargo test --release -p z00z_core --tests`
  - `cargo doc --release -p z00z_core --no-deps`
  - `cargo bench -p z00z_core --no-run`
  - `cargo build --release -p z00z_core --bins`
  - `cargo test --release -p z00z_core --examples`

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or existing infrastructure coverage
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency is bounded by the phase packet and full-suite gates
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-06-29
