<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD041 MD047 -->
# Phase 063: Core Update - Context

**Gathered:** 2026-06-28  
**Status:** Planned from the current-tree Phase 063 corpus on the existing
`.planning/phases/063-Core-Update/` directory only  
**Source:** PRD Express Path (`063-TODO.md`, `063-core-examples.md`, the
referenced `z00z_core` docs and YAML corpus, and the live `z00z_core`
workspace anchors)

<domain>
## 🎯 Phase Boundary

Phase 063 converts the current `z00z_core` recommendation audit into an
executable cleanup packet for bootstrap authority, genesis execution contracts,
object-family semantics, YAML ownership, test ownership, docs truth, and
support-surface layout.

`063-TODO.md` is normative, not advisory. `063-core-examples.md` is a
mandatory implementation source. The existing
`.planning/phases/063-Core-Update/` folder is the only canonical Phase 063
root. Graph outputs may be used for topology context only; they are not
implementation evidence.

### Current-Tree Planning Rule

- `063-TODO.md` contains 13 recommendation headings and zero `TASK-NNN` rows.
- `.planning/GSD-Workflow.md` still carries stale Phase 062 wording about
  `125` tasks and `27` grouped plans.
- Phase 063 planning therefore uses the 13 current recommendation headings as
  the canonical execution inventory.
- `REC-063-*` labels in this planning packet are local traceability ids only.
  They are not a second task namespace and they do not pretend that missing
  `TASK-NNN` rows exist.

### Phase 063 Delivers

1. `063-CONTEXT.md` with a recommendation-to-plan transfer table and
   current-tree conflict resolution.
2. Ordered executable plans `063-01-PLAN.md` through `063-13-PLAN.md`.
3. One canonical cleanup packet for `z00z_core` with no duplicate bootstrap,
   config, docs, test, or namespace authorities.

### Phase 063 Does Not Deliver

- No invented `TASK-NNN` inventory.
- No duplicate bootstrap or YAML authority lane.
- No graph-derived implementation claims.
- No compatibility alias for `z00z_core::vauchers`.
- No silently ignored `wallet_profiles[]` or `policy_profiles[]` genesis
  sections. When present, they must be typed, parsed, and validated through
  live `GenesisConfig` code.

</domain>

<decisions>
## ⚙️ Locked Decisions

- **D-01:** `063-TODO.md` remains the single canonical planning authority for
  Phase 063.
- **D-02:** The 13 recommendation headings map one-to-one onto 13 ordered
  numbered plans: `063-01` through `063-13`.
- **D-03:** `REC-063-*` labels are planning-local coverage ids only and must
  never be promoted into fake canonical task ids.
- **D-04:** `GenesisConfig` remains the only canonical bootstrap authority even
  when generation lanes, receipts, or YAML layout change.
- **D-05:** `z00z_core::genesis::*` remains the canonical public bootstrap
  caller path.
- **D-06:** `063-core-examples.md` is mandatory implementation input for live
  asset, right, policy, and voucher config materialization.
- **D-07:** `wallet_profiles[]` and `policy_profiles[]` are live parser-owned
  `GenesisConfig` sections. They must stay typed and validated even when some
  fields remain parser-owned before runtime materialization uses them.
- **D-08:** `crates/z00z_core/z00z_config/` is the target-state core YAML root;
  `configs/`, `src/assets/*.yaml`, `src/genesis/*.yaml`, example-local YAML,
  and `tests/vectors/*.yaml` must not remain parallel live authorities.
- **D-09:** `vauchers` namespace debt is removed in one coordinated rewrite;
  Phase 063 must not keep an alias or shim.
- **D-10:** Every plan `<verify>` block starts with
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, runs
  slice-specific checks, runs `cargo test --release` when Rust or tests are
  affected, runs `/GSD-Review-Tasks-Execution` at least 3 times until 2
  consecutive clean runs, and uses `/z00z-git-versioning` if a commit is
  needed.
- **D-11:** Phase 063 is a cleanup and truth-restoration packet, not a broad
  rewrite. New abstractions are allowed only when they reduce drift without
  creating a parallel authority plane.

</decisions>

<canonical_refs>
## 📚 Canonical References

### Planning Authority

- `.planning/phases/063-Core-Update/063-TODO.md`
- `.planning/phases/063-Core-Update/063-core-examples.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.github/copilot-instructions.md`
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`

### Live `z00z_core` Contract Anchors

- `crates/z00z_core/Cargo.toml`
- `crates/z00z_core/README.md`
- `crates/z00z_core/src/lib.rs`
- `crates/z00z_core/src/genesis/README.md`
- `crates/z00z_core/src/genesis/mod.rs`
- `crates/z00z_core/src/genesis/genesis.rs`
- `crates/z00z_core/src/genesis/validator.rs`
- `crates/z00z_core/src/genesis/genesis_run.rs`
- `crates/z00z_core/src/genesis/genesis_config.rs`
- `crates/z00z_core/src/genesis/genesis_config_validate.rs`
- `crates/z00z_core/src/genesis/genesis_derivation.rs`
- `crates/z00z_core/src/genesis/genesis_output.rs`
- `crates/z00z_core/src/genesis/genesis_output_support.rs`
- `crates/z00z_core/src/genesis/genesis_settlement_manifest.rs`
- `crates/z00z_core/src/assets/assets_config.yaml`
- `crates/z00z_core/src/genesis/genesis_config_devnet.yaml`
- `crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml`
- `crates/z00z_core/examples/genesis/genesis_config_devnet_test.yaml`
- `crates/z00z_core/tests/vectors/asset_pack_vectors.yaml`

### Downstream Impact Anchors

- `crates/z00z_simulator/src/scenario_1/stage_1/mod.rs`
- `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs`
- `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`
- `crates/z00z_wallets/src/rpc/object_rpc.rs`
- `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`
- `crates/z00z_storage/src/settlement/record.rs`
- `crates/z00z_storage/src/settlement/tx_plan_types.rs`

</canonical_refs>

<normative_mirror>
## 🧭 Inventory And Coverage Answer

| Inventory item | Count | Decision |
| --- | ---: | --- |
| Canonical recommendation headings in `063-TODO.md` | 13 | Phase 063 execution inventory |
| Canonical `TASK-NNN` rows in `063-TODO.md` | 0 | Must not be invented |
| Ordered numbered plans in this packet | 13 | `063-01` through `063-13` |
| Mandatory example sidecar sources | 1 | `063-core-examples.md` |

## ✅ Recommendation Transfer Table

| Coverage id | `063-TODO.md` heading | Locked execution contract | Primary refs | Target plan |
| --- | --- | --- | --- | --- |
| `REC-063-P0-01` | `P0. Fix the genesis execution contract drift` | Make `performance.num_threads` real, define one exact `snapshot_export_path` contract, and add regression tests for thread/config and artifact-path behavior. | `063-TODO.md`, `src/genesis/genesis_run.rs`, `genesis_config*.rs`, `genesis_output_support.rs`, `genesis_settlement_manifest.rs` | `063-01-PLAN.md` |
| `REC-063-P0-02` | P0. Collapse `genesis` to one canonical public owner path | Keep `z00z_core::genesis::*` as the only public owner path, remove boundary `include!` assembly, and internalize extra owner layers. | `063-TODO.md`, `src/genesis/mod.rs`, `genesis.rs`, `validator.rs`, `genesis_output.rs`, Design Foundation | `063-02-PLAN.md` |
| `REC-063-P0-03` | `P0. Run a documentation truth-restoration wave and add contract checks` | Restore truthful crate/genesis/assets docs, remove stale API narratives, and add compile-checked or test-backed doc contract gates. | `063-TODO.md`, `README.md`, `src/lib.rs`, `src/assets/mod.rs`, `src/assets/registry_catalog.rs`, `src/genesis/mod.rs`, `Cargo.toml` | `063-03-PLAN.md` |
| `REC-063-P1-01` | `P1. Enforce the single bootstrap authority in code shape, not only in prose` | Keep shared parsing logic while renaming or relocating secondary registry YAML vocabulary so it cannot look like a competing bootstrap path. | `063-TODO.md`, `README.md`, `src/assets/registry_config.rs`, `src/genesis/genesis_config.rs` | `063-04-PLAN.md` |
| `REC-063-P1-02` | `P1. Decouple object-family generation lanes without splitting bootstrap authority` | Add explicit generation-plan types, mode-aware validation, lane outputs, and partial receipts while keeping `run_genesis()` as the full-bootstrap wrapper. | `063-TODO.md`, `src/genesis/genesis_run.rs`, `genesis_derivation.rs`, `genesis_rights.rs`, `genesis_vouchers.rs`, `063-core-examples.md` | `063-05-PLAN.md` |
| `REC-063-P1-03` | P1. Reduce dual-authority writes around `GLOBAL_ASSET_REGISTRY` | Treat the global registry as a read-mostly fallback, narrow global writes to adapter boundaries, make explicit registries the primary write owner for generation, simulator, and runtime flows, and keep reset helpers test-only. | `063-TODO.md`, `src/assets/registry.rs`, `src/assets/registry_snapshot.rs`, `src/genesis/genesis_run.rs`, `z00z_simulator/stage_1/mod.rs` | `063-06-PLAN.md` |
| `REC-063-P1-04` | P1. Fix the public namespace debt around `vauchers` | Rename all callers to `z00z_core::vouchers`, remove `vauchers` in the same wave, keep a full caller inventory, preserve `VoucherBootstrapEntryV1` bootstrap semantics, and avoid reducing vouchers to `asset + right`. | `063-TODO.md`, `src/lib.rs`, `src/vouchers/mod.rs`, downstream imports, `README.md` | `063-07-PLAN.md` |
| `REC-063-P1-05` | `P1. Publish an authoritative object-family semantics matrix` | Create one canonical matrix that records whether genesis can create each family, whether runtime can create or mutate it, whether bootstrap shape differs from runtime shape, whether it carries value, what policy binding is required, whether explicit backing is required, and what the caller path is. | `063-TODO.md`, `src/genesis/README.md`, `z00z_wallets/src/rpc/object_rpc_impl.rs`, `z00z_storage/src/settlement/tx_plan_types.rs`, `063-core-examples.md` | `063-08-PLAN.md` |
| `REC-063-P1-06` | `P1. Preserve bounded object-family scenario coverage without widening authority` | Keep bounded asset/voucher/right/fee-object coverage, wallet object inventory, validator fail-closed enforcement, and explicit exclusions for unsupported broad claims. | `063-TODO.md`, `063-core-examples.md`, `z00z_simulator/tests/scenario_1/*`, `z00z_wallets/src/rpc/object_rpc.rs`, `z00z_storage/src/settlement/record.rs` | `063-09-PLAN.md` |
| `REC-063-P1-07` | P1. Make `z00z_core/z00z_config` the only core YAML authority | Move all core-owned YAML into `z00z_config`, preserve one `GenesisConfig` authority, keep catalog `_config.yaml` names with `devnet_genesis_config.yaml` as the canonical small devnet root, keep actions inside `policies[]`, parse and validate every `063-core-examples.md` section including `wallet_profiles[]` and `policy_profiles[]`, and treat archived tarballs as migration inputs rather than parallel authority. | `063-TODO.md`, `063-core-examples.md`, `crates/z00z_core/configs/*.yaml`, `src/genesis/*.yaml`, `src/assets/*.yaml`, example/test YAML | `063-10-PLAN.md` |
| `REC-063-P2-01` | `P2. Flatten test ownership and align names with the repository standard` | Flatten integration tests, rename `*_suite.rs`, add rights/vouchers coverage under the canonical test root, and update rename guards in the same wave. | `063-TODO.md`, `Cargo.toml`, `crates/z00z_core/tests/**`, `src/*/test_*_suite.rs` | `063-11-PLAN.md` |
| `REC-063-P2-02` | P2. Restore `z00z_core/docs` against live code | Rewrite docs to the live APIs and `z00z_config` paths, remove stale examples, and rename docs to English ASCII filenames. | `063-TODO.md`, `crates/z00z_core/docs/**`, `README.md`, support-surface Markdown | `063-12-PLAN.md` |
| `REC-063-P2-03` | `P2. Flatten benches, binaries, examples, and crate responsibility` | Flatten support-surface paths, remove example-local YAML, tighten feature ownership, update `Cargo.toml` to the new direct paths, and make an explicit decision between a dedicated tooling crate versus stricter feature gating for operator-facing CLI and export-heavy support. | `063-TODO.md`, `Cargo.toml`, `crates/z00z_core/benches/**`, `bin/**`, `examples/**`, `configs/**` | `063-13-PLAN.md` |

## 🔎 Meta Guidance Transfer

| `063-TODO.md` section | Planning transfer |
| --- | --- |
| `Recommended Execution Order` | Preserved as waves `W1` through `W4` below and as explicit `depends_on` chains in `063-01` through `063-13`. |
| `Doublecheck Verdict` | Preserved as current-tree guardrails: no graph-derived proof, no split bootstrap authority, no unsupported broad capability claims, and no live profile fields without parser/runtime support. |
| `Final Assessment` | Preserved as the phase-level no-broad-rewrite rule: Phase 063 executes targeted truth-restoration and ownership normalization waves rather than a speculative redesign. |

## 🧩 High-Risk Sub-Bullet Ledger

| Dense `063-TODO.md` sub-bullet cluster | Explicit plan transfer |
| --- | --- |
| Local Rayon pool only; no process-global side effects on simulator or unrelated tests | `063-01-PLAN.md` |
| Downstream import inventory before deep owner-path removal; `genesis_output.rs` must merge or lose boundary status | `063-02-PLAN.md` |
| Stale docs are API debt, not editorial debt | `063-03-PLAN.md` |
| Shared prerequisites, family lanes, full-bootstrap aggregation, policy dependency lane, partial receipt naming, exact lane tests, digest parity, manifest byte-stability, and simulator stage-1 parity | `063-05-PLAN.md` |
| Global registry stays read-mostly fallback, explicit owners take primary writes, and reset helpers remain test-only | `063-06-PLAN.md` |
| Full caller inventory; preserve `VoucherBootstrapEntryV1`; vouchers are not reducible to `asset + right` | `063-07-PLAN.md`, `063-08-PLAN.md` |
| Matrix rows must capture genesis-create vs runtime-create-or-mutate, explicit backing requirements, `VoucherBootstrapEntryV1` as bootstrap input rather than runtime voucher shape, and `mintable` wording that is narrowed or proven | `063-08-PLAN.md` |
| Full positive/negative object-flow grep gates, Section 32 wallet-extension coverage, validator fail-closed checks, and explicit unsupported-claim exclusion scans | `063-09-PLAN.md` |
| Canonical `domain_name` enforcement, preserved `_config.yaml` catalog naming with `devnet_genesis_config.yaml` as the canonical small devnet root, actions staying inside `policies[]`, `wallet_profiles[]` and `policy_profiles[]` parsed and validated as live parser-owned sections, `manifest_refs` root-owned boundary, archived tarballs as migration inputs only, zero stale target-state references, and migration scope limited to `crates/z00z_core` | `063-10-PLAN.md` |
| Meaningful test filenames when behavior theme is clear; required rights and vouchers coverage bullets | `063-11-PLAN.md` |
| Unsupported or non-authoritative ideas move back to planning artifacts instead of live docs | `063-12-PLAN.md` |
| Operator-facing CLI and heavy export packaging boundary, dedicated-tooling-crate versus stricter-feature-gating decision, and default `clap` feature review | `063-13-PLAN.md` |

## 🪜 Execution Waves

| Wave | Plans | Scope |
| --- | --- | --- |
| `W1` | `063-01` .. `063-03` | Runtime contract restoration and public truth restoration |
| `W2` | `063-04` .. `063-09` | Bootstrap authority, lane semantics, registry ownership, namespace normalization, and bounded scenario coverage |
| `W3` | `063-10` .. `063-11` | YAML authority migration and test ownership normalization |
| `W4` | `063-12` .. `063-13` | Docs truth restoration, support-surface flattening, and crate-boundary hygiene |

## 🚫 No-Drift Guardrails

- No new bootstrap authority beside `GenesisConfig`.
- No alias or shim for `vauchers`.
- No second YAML root under `configs/`, `src/assets`, `src/genesis`,
  `examples/genesis`, or `tests/vectors` after migration.
- No live `wallet_profiles[]` or `policy_profiles[]` fields may be left
  unparsed or unvalidated.
- No broad claims about useful-work attestation, live cross-chain settlement,
  linked liability carryover, live external enforcement, universal private VM,
  or second cash authority.

</normative_mirror>
