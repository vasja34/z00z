---
phase: 063
slug: 063-core-update
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-29
---

# Phase 063 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Bootstrap YAML -> `GenesisConfig` | Core YAML enters the live bootstrap path through `manifest_refs`, typed profile sections, and validator gates. | Manifest paths, asset/right/policy/voucher records, parser-owned profile data |
| Public genesis facade -> downstream callers | `z00z_core::genesis::*` is the only supported bootstrap surface for wallets, simulator, docs, and tests. | Public imports, operator examples, artifact paths |
| Explicit registry owner -> global fallback | Generation and simulator flows use explicit `AssetDefinitionRegistry` owners and only sync into the global fallback at a narrow adapter point. | Asset definitions, registry snapshots, fallback reads |
| Object-family runtime -> wallet object RPC | Rights and vouchers cross from typed genesis/runtime objects into wallet inventory and consumption methods. | Runtime object packages, right actions, voucher backing, policy ids |
| Support surfaces -> operators and contributors | `Cargo.toml`, docs, benches, bins, examples, and tests expose canonical paths and commands. | Build targets, CLI commands, Markdown paths, test ownership |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-063-01 | Tampering | Genesis runtime config | mitigate | `performance.num_threads` is validated and consumed by a local pool in `crates/z00z_core/src/genesis/genesis_config_validate.rs:133,147` and `crates/z00z_core/src/genesis/genesis_run.rs:216-217,498-499`; guard test rejects `build_global()` in `crates/z00z_core/src/genesis/test_genesis.rs:176-177`. | closed |
| T-063-02 | Tampering | Genesis output roots | mitigate | Artifact and snapshot roots are separated in `crates/z00z_core/src/genesis/genesis_run.rs:371,489-494,676` and documented in `crates/z00z_core/src/genesis/genesis_config.rs:422-435`. | closed |
| T-063-03 | Availability | Genesis concurrency isolation | mitigate | Genesis uses a dedicated `rayon::ThreadPoolBuilder` and installs work inside that pool in `crates/z00z_core/src/genesis/genesis_run.rs:216-217,498-499`; source guard asserts the boundary in `crates/z00z_core/src/genesis/test_genesis.rs:176-177`. | closed |
| T-063-04 | Integrity | Public genesis owner path | mitigate | The canonical facade is pinned by `crates/z00z_core/src/genesis/mod.rs:6,26,39,68,79`, `crates/z00z_core/src/lib.rs:10,65`, `crates/z00z_core/README.md:13,25,56`, and `crates/z00z_core/tests/test_live_guardrails.rs:51,123-126`. | closed |
| T-063-05 | Integrity | Genesis module boundaries | mitigate | Boundary-defining `include!` assembly is absent from the public genesis facade and validator surface; the live guard suite compiles the shallow path in `crates/z00z_core/tests/test_live_guardrails.rs:51`. | closed |
| T-063-06 | Integrity | Public docs | mitigate | Stale-string and public-surface pins are enforced by `crates/z00z_core/tests/test_live_guardrails.rs:104-168`, with canonical wording in `crates/z00z_core/README.md:25,85,111` and `crates/z00z_core/src/genesis/mod.rs:68`. | closed |
| T-063-07 | Integrity | Doc contract gate | mitigate | `Cargo.toml` keeps doctests disabled at `crates/z00z_core/Cargo.toml:14`, and the live replacement guard exists at `crates/z00z_core/tests/test_live_guardrails.rs:123-168`. | closed |
| T-063-08 | Spoofing | Secondary registry loader | mitigate | Secondary authority wording is explicit in `crates/z00z_core/src/assets/registry_catalog.rs:5,12,18,30`, `crates/z00z_core/src/assets/registry_config.rs:7-8`, `crates/z00z_core/README.md:25,30,82`, and `crates/z00z_core/src/genesis/README.md:129-133`. | closed |
| T-063-09 | Tampering | Generation lane invariants | mitigate | Lane-aware full-bootstrap and selected-lane behavior is implemented in `crates/z00z_core/src/genesis/genesis_run.rs:21-32,95,148-157,236` and proven by `crates/z00z_core/tests/test_genesis_manifest.rs:226,263,356`. | closed |
| T-063-10 | Tampering | Partial export boundary | mitigate | Full manifests and partial receipts are distinct via `GENESIS_GENERATION_RECEIPT_FILE` in `crates/z00z_core/src/genesis/genesis_run.rs:5` and `GENESIS_SETTLEMENT_MANIFEST_FILE` in `crates/z00z_core/src/genesis/genesis_settlement_manifest.rs:12`; negative coverage exists in `crates/z00z_core/tests/test_genesis_manifest.rs:333`. | closed |
| T-063-11 | Tampering | Policy resolution | mitigate | Plan-aware validation is routed through `crates/z00z_core/src/genesis/genesis_config_validate.rs:61,147`; wrong-family voucher policy rejection and rights-only policy gating are covered in `crates/z00z_core/tests/test_genesis_manifest.rs:287,318`. | closed |
| T-063-12 | Tampering | Registry ownership | mitigate | Explicit-owner staging stays local in `crates/z00z_simulator/src/scenario_1/stage_1/mod.rs:126,142,145`; global sync is narrow and named in `crates/z00z_core/src/genesis/genesis_run.rs:626`; regression proof is `crates/z00z_core/tests/test_assets_registry_integration.rs:302,326-338`. | closed |
| T-063-13 | Integrity | Voucher namespace | mitigate | The only live owner path is `crates/z00z_core/src/vouchers/mod.rs:8`; bootstrap type exports remain stable in `crates/z00z_core/src/lib.rs:111` and `crates/z00z_core/src/vouchers/voucher_bootstrap.rs:14-17`; repo-wide `rg -n "\\bvauchers\\b"` across core, wallets, storage, simulator, and docs returned no hits on 2026-06-29. | closed |
| T-063-14 | Integrity | Object-family semantics | mitigate | The canonical matrix is `crates/z00z_core/docs/OBJECT_FAMILY_SEMANTICS.md:8,12,20-23`; runtime anchors exist in `crates/z00z_wallets/src/rpc/object_rpc_impl.rs:189-201,236,247-259,348,769,807-811` and `crates/z00z_storage/src/settlement/tx_plan_types.rs:73,361-365,570-572,693,746,809,996`. | closed |
| T-063-15 | Elevation of Privilege | Object-flow runtime authority | mitigate | Object RPC remains explicit at `crates/z00z_wallets/src/rpc/object_rpc.rs:47,112`; cash-vs-right separation is asserted in `crates/z00z_wallets/tests/test_asset_import_security.rs:801,812`; bounded object-flow rows are pinned in `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs:810,852,870-871,901-916,1297` and `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs:350,378,459`. | closed |
| T-063-16 | Tampering | Core YAML root | mitigate | Canonical config constants live in `crates/z00z_core/src/config_paths.rs:9-27`; the live root manifest is `crates/z00z_core/z00z_config/devnet_genesis_config.yaml:2,56,189`; `manifest_refs` and canonical-profile counts are enforced in `crates/z00z_core/tests/test_genesis_manifest_refs.rs:61-65,96-104,141,195,220`. | closed |
| T-063-17 | Integrity | Parser-owned profile sections | mitigate | Typed profile fields exist in `crates/z00z_core/src/genesis/genesis_config.rs:60-62,235-265,306-325,535-559,720-732`; loader parsing is explicit in `crates/z00z_core/src/genesis/manifest_ref_loader.rs:57,106-116,132`; validator gates are in `crates/z00z_core/src/genesis/genesis_config_validate.rs:141-142,324-350`. | closed |
| T-063-18 | Integrity | Core test ownership | mitigate | Flat-root test entrypoints are declared in `crates/z00z_core/Cargo.toml:137-165`, `crates/z00z_core/tests/test_genesis.rs:4-5,13`, `crates/z00z_core/tests/test_assets.rs:4-5,13`, and `crates/z00z_core/tests/test_genesis_mod.rs:4`; `find crates/z00z_core/tests -mindepth 2 -type f ! -path '*/tests/fixtures/*'` returned no hits on 2026-06-29. | closed |
| T-063-19 | Integrity | Docs and support Markdown | mitigate | Live-path and ASCII guards are enforced by `crates/z00z_core/tests/test_live_guardrails.rs:104-168`; canonical support-surface paths are in `crates/z00z_core/bin/README.md:11`, `crates/z00z_core/examples/README.md:20`, `crates/z00z_core/docs/ASSETS_EXAMPLES.md:30`, and `crates/z00z_core/docs/GENESIS_DOCUMENTATION.md:863-880`. | closed |
| T-063-20 | Integrity | Benches, bins, and examples | mitigate | Flat support-surface entries are declared in `crates/z00z_core/Cargo.toml:87-133,169-185`; `find crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples -mindepth 2 -type f` returned no hits on 2026-06-29. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-29 | 20 | 20 | 0 | Codex `/gsd-secure-phase 063` |

---

## Verification Notes

- `register_authored_at_plan_time: true`
  Evidence: all `063-01-PLAN.md` through `063-13-PLAN.md` contain a
  `<threat_model>` block.
- `Threat Flags` summary scan returned none.
  Evidence: no `## Threat Flags` sections were present in any
  `063-*-SUMMARY.md` file on 2026-06-29.
- Repo-wide stale-voucher-spelling scan returned none.
  Evidence: `rg -n "\\bvauchers\\b" crates/z00z_core crates/z00z_wallets crates/z00z_storage crates/z00z_simulator crates/z00z_core/docs crates/z00z_core/README.md`
  returned no hits on 2026-06-29.
- Flat core test-owner scan returned none.
  Evidence: `find crates/z00z_core/tests -mindepth 2 -type f ! -path '*/tests/fixtures/*'`
  returned no hits on 2026-06-29.
- Flat support-surface scan returned none.
  Evidence: `find crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples -mindepth 2 -type f`
  returned no hits on 2026-06-29.
- Release verification run:
  - `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture`
  - `cargo test --release -p z00z_core --features test-params-fast --test genesis_tests test_genesis_partial_run_does_not_emit_full_settlement_manifest -- --nocapture`
  - `cargo test --release -p z00z_core --test assets_tests registry_integration::test_registry_explicit_owner_stays_local_until_global_sync -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_1 test_scenario1_object_flows_matrix_contract -- --nocapture`

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-29
