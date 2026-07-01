# z00z_core Deep Recommendation Review

[TOC]



**Date:** 2026-06-25  
**Scope:** `crates/z00z_core` with downstream impact checks in `z00z_wallets`, `z00z_storage`, and `z00z_simulator`

## Method

- I used `.planning/graphs` for topology only, not as a source of concrete truth.
- Concrete findings were confirmed against live code in `z00z_core` and selected downstream consumers.
- I cross-checked the recommendations against the repository design rules in [Z00Z_DESIGN_FOUNDATION.md](/home/vadim/Projects/z00z/.github/requirements/Z00Z_DESIGN_FOUNDATION.md:821).

## Topology Context

The graph report shows a large corpus with high structural density: [GRAPH_REPORT.md](/home/vadim/Projects/z00z/.planning/graphs/GRAPH_REPORT.md:1) reports `97392` nodes and `208146` edges. A local scan over `.planning/graphs/graph.json` shows that `z00z_core` alone contributes `1298` graph nodes across `145` source files and spans multiple active communities. The same scan shows heavy community overlap with `z00z_wallets`, `z00z_storage`, and `z00z_simulator`.

That topology matches the code:

- `z00z_simulator` imports genesis helpers and also writes into the global asset registry in [stage_1/mod.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_1/mod.rs:123).
- `z00z_wallets` depends on runtime object-family semantics that originate in `z00z_core`, including voucher issue and right creation flows in [object_rpc_impl.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/rpc/object_rpc_impl.rs:637).
- `z00z_storage` depends on core asset and voucher leaf contracts through `SettlementActionV1` and related leaf types in [tx_plan_types.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/settlement/tx_plan_types.rs:64).

The practical consequence is simple: path drift, ownership drift, and config drift inside `z00z_core` do not stay local. They propagate into wallet, storage, simulator, and test infrastructure quickly.

## Executive Summary

The deepest problems in `z00z_core` are not low-level crypto defects. The stronger risks are:

1. public-contract drift between docs, config, and runtime behavior;
2. boundary confusion inside `genesis`;
3. dual authority patterns around bootstrap data and the global registry;
4. orchestration coupling between object-family generation lanes;
5. naming and test-layout debt that makes the crate harder to trust and evolve.

The crate already has strong pieces:

- a curated root facade in [lib.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/lib.rs:103);
- a clear statement that `genesis` is the single canonical bootstrap authority in [README.md](/home/vadim/Projects/z00z/crates/z00z_core/README.md:20);
- deterministic typed artifact generation in [src/genesis/README.md](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/README.md:13).

The recommendations below focus on removing contradictions around those strengths.

## Prioritized Recommendations

### P0. Fix the genesis execution contract drift

**Problem**

`GenesisConfig` exposes runtime controls and output fields that look authoritative, but the implementation does not fully honor that contract.

**Evidence**

- `performance.num_threads` is parsed in [genesis_config.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config.rs:203) and validated in [genesis_config_validate.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_validate.rs:70).
- `run_genesis()` only logs `rayon::current_num_threads()` and never applies the configured thread count in [genesis_run.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_run.rs:24).
- `snapshot_export_path` is validated and prepared in [genesis_run.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_run.rs:11), but the live typed artifacts and the snapshot ZIP are written under the timestamped `assets_export_path` output directory in [genesis_run.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_run.rs:15), [genesis_run.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_run.rs:166), and [genesis_output_support.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_output_support.rs:94).
- The manifest file itself is always `output_dir / genesis_settlement_manifest.json` in [genesis_settlement_manifest.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_settlement_manifest.rs:487).

**Recommendation**

- Apply `performance.num_threads` through a dedicated local Rayon pool installed around generation and verification work.
- Do not use `build_global()` unless the repository intentionally wants process-global Rayon behavior.
- Define one exact meaning for `snapshot_export_path`:
  - either it owns snapshot ZIP outputs,
  - or it is removed/renamed because the actual output contract is `assets_export_path`.
- Add regression tests that assert both thread-config behavior and artifact-path behavior.

**Why this is the right fix**

This is not only a performance concern. It is a contract-integrity concern. A parsed and validated config field that does nothing is more dangerous than an absent field because operators trust it.

**Guardrails**

- Use a local pool to avoid surprising `z00z_simulator` or test processes that also use Rayon.
- Keep output semantics deterministic and document them in one canonical place.

### P0. Collapse `genesis` to one canonical public owner path

**Problem**

The current `genesis` surface exposes too many ownership layers for the same capability.

**Evidence**

- `genesis/mod.rs` exports a public `genesis` submodule and also re-exports many of the same items at the shallower `z00z_core::genesis::*` path in [genesis/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/mod.rs:189) and [genesis/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/mod.rs:198).
- The boundary-defining implementation module is assembled from `include!` fragments in [genesis/genesis.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis.rs:35).
- The same pattern is repeated in the public validator surface, where `validator.rs` assembles its implementation from `include!` fragments in [validator.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/validator.rs:16).
- `genesis_output.rs` is only a timestamp helper plus a thin forwarding wrapper into `genesis_output_support.rs` in [genesis_output.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_output.rs:1) and [genesis_output.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_output.rs:52).
- The design foundation explicitly says boundary-defining modules must not be assembled from `include!` fragments and recommends one thin facade with explicit submodules in [Z00Z_DESIGN_FOUNDATION.md](/home/vadim/Projects/z00z/.github/requirements/Z00Z_DESIGN_FOUNDATION.md:917).

**Recommendation**

- Make `z00z_core::genesis::*` the only canonical caller path.
- Convert the deep implementation from `include!` assembly to explicit internal submodules.
- Remove or internalize the extra public owner layer `z00z_core::genesis::genesis::*` after consumer inventory and migration.
- Merge `genesis_output.rs` into its real owner module or make the support module the direct internal implementation.

**Why this is the right fix**

This preserves the good part of the existing design: a shallow public facade. It removes the contradictory part: a second deep public owner for the same contracts.

**Guardrails**

- Do not change caller-visible paths and deep module names in the same first wave.
- Inventory downstream imports before hiding deep paths.
- Keep public re-exports explicit and narrow, which aligns with [Z00Z_DESIGN_FOUNDATION.md](/home/vadim/Projects/z00z/.github/requirements/Z00Z_DESIGN_FOUNDATION.md:829).

### P0. Run a documentation truth-restoration wave and add contract checks

**Problem**

The public documentation surface has drifted far enough that it now creates false mental models.

**Evidence**

- `lib.rs` still documents `state`, `tx`, and `validation` as if they were core exported modules and uses a nonexistent `z00z_core::genesis::assets_generator` example in [lib.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/lib.rs:20) and [lib.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/lib.rs:67).
- `assets/mod.rs` points to `tests/assets/fixtures.rs`, references a nonexistent `utils_traits` feature, and includes stale example APIs such as `insert_wire`, `to_snapshot`, and `list_asset_ids` in [assets/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/mod.rs:34), [assets/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/mod.rs:57), [assets/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/mod.rs:80), [assets/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/mod.rs:105), and [assets/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/mod.rs:182).
- The old `assets/assets_config.rs` doc drift is now closed by the live
  `registry_catalog.rs` surface, which documents the canonical `serials` and
  `nominal` contract instead of the removed `max_supply` wording in
  [registry_catalog.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/registry_catalog.rs:1).
- `genesis/mod.rs` still describes output as `genesis_Z00Z.json` and `genesis_Z00Z.bin` and frames genesis primarily as asset generation in [genesis/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/mod.rs:8) and [genesis/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/mod.rs:69).
- Doctests are disabled for the crate in [Cargo.toml](/home/vadim/Projects/z00z/crates/z00z_core/Cargo.toml:8), which reduces protection against example rot.

**Recommendation**

- Pick one canonical truth source for each public area:
  - crate root overview in `README.md` and `lib.rs`,
  - asset-owner contract in `assets/mod.rs`,
  - bootstrap-owner contract in `src/genesis/README.md` and `genesis/mod.rs`.
- Remove duplicated legacy architecture narratives that no longer match the code.
- Add lightweight contract checks for public examples:
  - compile-only doc examples where practical,
  - or equivalent small tests that validate the documented path and type names.
- Treat stale docs as API debt, not editorial debt.

**Why this is the right fix**

For `z00z_core`, documentation is part of the protocol surface. Wallet, simulator, and storage developers infer semantics from it. False docs create concept drift even when the code is correct.

### P1. Enforce the single bootstrap authority in code shape, not only in prose

**Problem**

The repository already says that `z00z_core::genesis` is the only canonical bootstrap authority, but the surrounding public surfaces still make secondary YAML loading look too similar to a competing authority path.

**Evidence**

- The crate README correctly says `src/assets/assets_config.yaml` is not a second genesis authority in [README.md](/home/vadim/Projects/z00z/crates/z00z_core/README.md:26).
- The old public-facing `AssetDefinitionRegistry::load_from_config()` ambiguity
  is now closed by the explicit
  `AssetDefinitionRegistry::load_catalog_from_yaml()` contract in
  [registry_config.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/registry_config.rs:6).
- `genesis_config.rs` explicitly says the `assets` parse path uses the same logic as the registry loader in [genesis_config.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config.rs:223).

**Recommendation**

- Keep shared parsing logic, but separate authority vocabulary from registry-data vocabulary.
- Rename or relocate the secondary YAML lane so that it reads like legacy registry-data input, not bootstrap input.
- If the path must remain public, document it as a secondary lane in the type and function names, not only in surrounding prose.
- Preserve `GenesisConfig` as the only typed authority for bootstrap.

**Why this is the right fix**

This recommendation does not split genesis into multiple sources of truth. It does the opposite: it removes the accidental symmetry between canonical bootstrap input and secondary registry data.

### P1. Decouple object-family generation lanes without splitting bootstrap authority

**Problem**

Generation is already partially split at the helper level, but the public config and orchestration path still force a mostly full-corpus mental model.

**Evidence**

- `run_genesis()` always loads one config, builds asset definitions, derives policies, and then constructs one shared settlement corpus in [genesis_run.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_run.rs:7), [genesis_run.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_run.rs:26), [genesis_run.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_run.rs:69), and [genesis_run.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_run.rs:72).
- The code already has separable generators:
  - asset definitions via `create_asset_definition`;
  - assets via `generate_all_genesis_assets`;
  - rights via `generate_genesis_rights_with_policies`;
  - vouchers via `generate_genesis_vouchers`;
  - full aggregation via `generate_genesis_settlement_corpus`;
  as visible in [genesis_derivation.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_derivation.rs:307), [genesis_rights.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_rights.rs:295), and [genesis_vouchers.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_vouchers.rs:77).
- Despite that, config parsing still hard-requires `assets` in [genesis_config.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config.rs:241) and validation hard-requires non-empty `rights` in [genesis_config_validate.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_validate.rs:153).
- Voucher generation depends on policy lookup, and full-corpus generation always runs terminal collision checks across families in [genesis_derivation.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_derivation.rs:325) and [genesis_derivation.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_derivation.rs:470).

**Recommendation**

- Keep one canonical `GenesisConfig` authority, but introduce explicit generation lanes on top of it.
- Separate the API into:
  - shared prerequisites: seed, chain context, policy resolution, collision rules;
  - family lanes: definitions/assets, rights, vouchers;
  - full-bootstrap aggregation: corpus assembly, cross-family collision checks, manifest export.
- Add a typed selector, for example `GenesisGenerationPlan` or `GenesisSelection`, that can express:
  - `full_bootstrap`;
  - `assets_only`;
  - `rights_only`;
  - `vouchers_only`;
  - `policies_only`;
  - `selected_families`.
- Relax validation so it is mode-aware instead of globally unconditional.
  - Example: `rights` should be mandatory only for plans that request right generation or full bootstrap.
  - Example: `assets` should be mandatory only for plans that request asset generation or policies derived from asset-owned context.
- Keep `run_genesis()` as the consensus-safe full-bootstrap default, but add a second explicit lane for partial generation and artifact refresh.

**Why this is the right fix**

This is the clean middle path:

- it does **not** create parallel bootstrap authorities;
- it does **not** weaken deterministic full genesis;
- it does allow modular regeneration of specific object families when the repository needs targeted artifact rebuilds, fixture refreshes, or staged evolution.

**Guardrails**

- Partial generation must still consume the same canonical config format unless there is a strong reason to introduce narrower typed wrappers.
- Cross-family invariants such as terminal collision checks must remain mandatory whenever more than one family is emitted together.
- Voucher generation must continue to require resolved voucher-family policy binding; do not silently fall back to asset or generic policy semantics.

#### Concrete implementation plan

**Target outcome**

Keep the current consensus-safe full bootstrap path intact, but add an explicit and audited partial-generation path for selected object families.

**Proposed public API shape**

Add these public contracts under `z00z_core::genesis`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum GenesisLane {
    Policies,
    Assets,
    Rights,
    Vouchers,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GenesisSelection {
    FullBootstrap,
    PoliciesOnly,
    AssetsOnly,
    RightsOnly,
    VouchersOnly,
    Selected(BTreeSet<GenesisLane>),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GenesisExportKind {
    FullBootstrap,
    PartialLaneSet,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GenesisGenerationPlan {
    pub selection: GenesisSelection,
    pub export_kind: GenesisExportKind,
    pub verify_proofs: bool,
    pub export_artifacts: bool,
}

#[derive(Clone, Debug)]
pub struct GenesisResolvedContext {
    pub config: GenesisConfig,
    pub seed: GenesisSeed,
    pub chain_id: u32,
    pub network_type: ChainType,
    pub policies: Vec<GenesisPolicyRecord>,
    pub policy_lookup: BTreeMap<String, GenesisPolicyRecord>,
}

#[derive(Clone, Debug, Default)]
pub struct GenesisLaneOutputs {
    pub assets: Option<GenesisAssetAccumulator>,
    pub rights: Option<Vec<GenesisRightRecord>>,
    pub vouchers: Option<Vec<GenesisVoucherRecord>>,
    pub policies: Option<Vec<GenesisPolicyRecord>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GenesisGenerationReceipt {
    pub selection: GenesisSelection,
    pub output_kind: GenesisExportKind,
    pub network: String,
    pub emitted_files: Vec<String>,
    pub full_manifest_file: Option<String>,
    pub lane_manifest_file: Option<String>,
}
```

**Why this shape**

- `GenesisSelection` makes the mode explicit and reviewable.
- `GenesisResolvedContext` centralizes shared prerequisites once instead of re-deriving them in each lane.
- `GenesisLaneOutputs` avoids forcing a fake full corpus when only one lane was requested.
- `GenesisGenerationReceipt` gives partial-generation runs an auditable output contract without pretending they are full settlement manifests.

**Proposed new entrypoints**

Keep existing stable functions and add new ones:

```rust
pub fn load_genesis_context(
    config_path: &str,
    plan: &GenesisGenerationPlan,
) -> Result<GenesisResolvedContext, GenesisError>;

pub fn validate_genesis_config_for(
    config: &GenesisConfig,
    plan: &GenesisGenerationPlan,
) -> Result<(), GenesisError>;

pub fn generate_genesis_lanes(
    ctx: &GenesisResolvedContext,
    plan: &GenesisGenerationPlan,
    logger: Arc<dyn Logger>,
    metrics: Arc<dyn MetricsSink>,
) -> Result<GenesisLaneOutputs, GenesisError>;

pub fn run_genesis_with_plan(
    config_path: &str,
    plan: &GenesisGenerationPlan,
    cli_command: Option<&str>,
) -> Result<GenesisGenerationReceipt, GenesisError>;
```

Then preserve the current public contract by rewriting:

```rust
pub fn run_genesis(config_path: &str, cli_command: Option<&str>) -> Result<(), GenesisError> {
    let plan = GenesisGenerationPlan::full_bootstrap();
    run_genesis_with_plan(config_path, &plan, cli_command).map(|_| ())
}
```

**Validation refactor**

Split validation into three layers:

1. `validate_common_config(config)`
   - chain id, chain type, magic bytes, outputs, performance, seed.

2. `validate_lanes_requested(config, plan)`
   - require `assets` only when assets or asset-derived policies are requested;
   - require `rights` only when rights are requested;
   - require `vouchers` only when vouchers are requested;
   - require `policies` only when vouchers or rights depend on them.

3. `validate_cross_lane_constraints(ctx, plan, outputs)`
   - terminal collisions;
   - voucher policy-family binding;
   - any full-manifest-only invariants.

This avoids the current unconditional constraints such as mandatory non-empty `rights` in all modes.

**Policy handling rule**

Policies remain a shared prerequisite lane, not a parallel authority:

- vouchers always require resolved voucher-family policies;
- rights may require policy resolution when policy ids are derived or checked;
- asset-only generation may still need policies when native cash detection or canonical gas-policy binding is part of the requested output.

That means `Policies` is not just another artifact bucket. It is also a dependency lane.

**Export contract**

Keep current artifact names and semantics for `FullBootstrap`.

For partial generation:

- do **not** emit `genesis_settlement_manifest.json`;
- emit selected typed artifacts only;
- emit a new lane-scoped receipt file, for example:
  - `genesis_generation_receipt.json`
  - or `genesis_lane_manifest.json`

Reason: a partial run is not the same thing as the full deterministic settlement manifest, and the naming must not blur that distinction.

**Refactor waves**

**Wave 1: add mode types without behavior change**

- add `GenesisLane`, `GenesisSelection`, `GenesisGenerationPlan`;
- add constructor helpers such as `full_bootstrap()`, `assets_only()`, `selected(...)`;
- keep `run_genesis()` behavior byte-for-byte unchanged.

Success criteria:

- no downstream callers change;
- all existing genesis tests still pass unchanged.

**Wave 2: split validation by responsibility**

- extract `validate_common_config`;
- add `validate_genesis_config_for`;
- keep current `validate_config_schema` as a full-bootstrap wrapper.

Success criteria:

- full-bootstrap fixtures behave exactly as before;
- new unit tests prove that `rights_only` and `vouchers_only` plans no longer fail because unrelated sections are empty or absent, when those sections are not semantically required.

**Wave 3: introduce shared resolved context**

- add `GenesisResolvedContext`;
- centralize `load_genesis_config`, seed derivation, chain/network resolution, and policy resolution;
- stop recomputing the same prerequisites across lane paths.

Success criteria:

- `run_genesis()` still delegates through the new context and produces the same outputs;
- lower-level lane generators can be called independently in tests.

**Wave 4: split execution into lane executors**

- route asset generation through `generate_all_genesis_assets`;
- route rights through `generate_genesis_rights_with_policies`;
- route vouchers through `generate_genesis_vouchers`;
- keep `generate_genesis_settlement_corpus` as the full-bootstrap aggregator, not as the only public execution shape.

Success criteria:

- partial lane tests can generate exactly one family at a time;
- cross-family collision checks still run in full-bootstrap and multi-lane modes.

**Wave 5: add partial export path**

- add `run_genesis_with_plan`;
- keep `run_genesis` as a full-bootstrap wrapper;
- add lane receipt export;
- keep `genesis_settlement_manifest.json` reserved for full bootstrap only.

Success criteria:

- `assets_only` can export only asset artifacts;
- `rights_only` can export only rights artifacts;
- `vouchers_only` can export only voucher artifacts;
- full bootstrap still emits the current manifest contract.

**Wave 6: optional CLI and downstream adoption**

- add optional CLI selectors such as `--selection full_bootstrap|assets_only|rights_only|vouchers_only|selected`;
- or `--lane assets --lane rights`.
- leave existing CLI behavior unchanged when no selector is passed.
- optionally simplify simulator/test fixture refresh flows to use partial lanes where appropriate.

Success criteria:

- no existing scripts break when they call the current CLI form;
- partial artifact refresh becomes possible without replaying the entire full-bootstrap path.

**Verification gates**

Add these focused tests:

- `test_genesis_plan_full_bootstrap_matches_legacy_run`
- `test_genesis_plan_assets_only_skips_rights_validation`
- `test_genesis_plan_rights_only_requires_policy_resolution_when_needed`
- `test_genesis_plan_vouchers_only_rejects_non_voucher_policy`
- `test_genesis_partial_run_does_not_emit_full_settlement_manifest`
- `test_genesis_selected_lanes_preserve_terminal_collision_checks`

Add these regression checks:

- full-bootstrap digest parity before and after refactor;
- manifest byte-stability for the canonical devnet fixture;
- simulator stage-1 parity when using full-bootstrap mode.

**Main risks**

- accidentally weakening full-bootstrap invariants while enabling partial modes;
- allowing partial modes to emit files that look canonical when they are not;
- letting policy resolution semantics diverge between full and partial paths.

**Risk controls**

- keep `run_genesis()` as the stable wrapper until the new path is proven;
- reserve the existing settlement manifest for full bootstrap only;
- gate every new partial mode with explicit tests and receipt semantics.

### P1. Reduce dual-authority writes around `GLOBAL_ASSET_REGISTRY`

**Problem**

The global registry is allowed by the design foundation, but current write patterns make ownership too implicit.

**Evidence**

- The global singleton is defined in [registry.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/registry.rs:177).
- `run_genesis()` writes asset definitions into `GLOBAL_ASSET_REGISTRY` in [genesis_run.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_run.rs:33).
- The simulator stage writes into both `GLOBAL_ASSET_REGISTRY` and `ctx.registry` in the same flow in [stage_1/mod.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_1/mod.rs:123).
- The design foundation allows global singletons, but warns against mutable state without clear ownership in [Z00Z_DESIGN_FOUNDATION.md](/home/vadim/Projects/z00z/.github/requirements/Z00Z_DESIGN_FOUNDATION.md:2491).

**Recommendation**

- Treat the global registry as a process-wide read-mostly fallback, not as a second write owner.
- Push primary ownership toward explicitly passed `AssetDefinitionRegistry` instances for generation, simulator contexts, and runtime services.
- Narrow global writes to small adapter boundaries rather than core flows.
- Keep test-only reset helpers under `#[cfg(test)]`, which already exists in [registry_snapshot.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/registry_snapshot.rs:140).

**Why this is the right fix**

The issue is not that a global exists. The issue is that two registries can both look authoritative in the same flow. That complicates reproducibility, test isolation, and future refactors.

### P1. Fix the public namespace debt around `vauchers`

**Problem**

The misspelled `vauchers` namespace has escaped into the stable surface and multiple downstream crates.

**Evidence**

- Before the normalization wave, the public module was `pub mod vauchers;`; the live canonical root is now `pub mod vouchers;` in [lib.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/lib.rs:89).
- The live canonical owner module is now [vouchers/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/vouchers/mod.rs:1), and the old `vauchers` path must stay dead with no alias or shim.
- Downstream crates must stay on that canonical spelling; any regression back to `vauchers` fails this slice.

**Recommendation**

- Introduce `z00z_core::vouchers` as the new canonical public path.
- Rewrite every internal and downstream caller to `z00z_core::vouchers` in the same normalization wave.
- Remove the misspelled `z00z_core::vauchers` public lane in that wave. Do not keep an alias, shim, or compatibility module.
- Before removal, produce a caller inventory for storage, wallets, tests, docs, and examples, then update that full inventory in one coordinated patch set.

**Why this is the right fix**

The safety mechanism here is caller inventory plus same-wave rewrite, not a dual public surface. Keeping the misspelled lane would preserve known namespace debt and make new code carry legacy spelling forward.

**Important non-recommendation**

I do **not** recommend renaming `VoucherBootstrapEntryV1` away from its current concept. The type name is semantically correct for a manifest/bootstrap entry. The better fix is clearer rustdoc, not type-concept drift.

**Voucher bootstrap semantics clarification**

- `VoucherBootstrapEntryV1` is a manifest/bootstrap declaration, not the canonical runtime or wire-level voucher object.
- `materialize(policy_id, action_pool_id)` converts that declaration into `VoucherConfigEntry`, and genesis derivation then deterministically recomputes `replay_nonce` and terminal bindings before emitting final voucher records.
- The semantic model is hybrid but not reducible to `asset + right`: vouchers carry value and backing claims, while rights encode control semantics and do not carry value.

### P1. Publish an authoritative object-family semantics matrix

**Problem**

The repository now supports more than one object family, but the lifecycle story is not equally explicit for all of them.

**Evidence**

- `src/genesis/README.md` clearly frames genesis as a shared typed bootstrap boundary for assets, rights, policies, and vouchers in [src/genesis/README.md](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/README.md:13).
- Runtime voucher issue and runtime right creation are explicit in [object_rpc_impl.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/rpc/object_rpc_impl.rs:637) and [object_rpc_impl.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/rpc/object_rpc_impl.rs:660).
- `SettlementActionV1` has a generic `AssetMutation` lane in [tx_plan_types.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/settlement/tx_plan_types.rs:64).
- Asset definitions expose a `mintable` concept in [definition.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/definition.rs:176), but the post-genesis asset lifecycle is not documented as explicitly as the voucher/right paths.

**Recommendation**

Add one canonical matrix that states, for each object family:

- whether genesis can create it;
- whether runtime can create or mutate it;
- whether its bootstrap-manifest shape differs from its canonical runtime or wire shape;
- whether it carries value;
- what policy binding is required;
- whether explicit backing is required;
- what the canonical caller path is.

That matrix should explicitly record that `VoucherBootstrapEntryV1` is a bootstrap input rather than a runtime voucher object, and that vouchers carry value/backing claims while rights do not carry value.

If post-genesis asset minting is intentionally unsupported today, document that clearly and narrow the wording around `mintable`. If it is supported, document and test the exact runtime contract.

**Why this is the right fix**

This avoids a future semantic split where wallet, storage, and simulator each guess the asset-family lifecycle differently.

### P1. Preserve bounded object-family scenario coverage without widening authority

**Problem**

The old local implementation packet contains object-family closeout requirements that belong in this core cleanup, but only after trimming broad claims that the current code does not support.

**Evidence**

- `RightLeaf`, `VoucherLeaf`, and `FeeEnvelope` are concrete settlement record types in [record.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/settlement/record.rs:266), [record.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/settlement/record.rs:303), and [record.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/settlement/record.rs:541).
- `wallet.object.*` is the typed object inventory and package surface in [object_rpc.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/rpc/object_rpc.rs:19).
- `test_scenario1_object_flows_matrix_contract` already locks voucher, right, fee-supported transition, and fail-closed object-flow scenario ids in [test_scenario1_object_flows.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs:812).
- `test_scenario1_object_flows_wallet_inventory_for_alice_bob_charlie` verifies typed wallet object inventory, and wallet import security checks that cash import does not appear in `wallet.object.list_rights`.
- Rights business entitlement, agentic right, and machine capability local lifecycle tests exist in [test_hjmt_e2e.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs:350), [test_hjmt_e2e.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs:378), and [test_hjmt_e2e.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs:459).
- `063-core-examples.md` names the required examples for `machine_compute_capability`, `confidential_data_access`, `service_entitlement`, `validator_mandate`, `one_time_agent_action`, voucher policies, and right policies.

**Recommendation**

- Preserve the bounded internal object-family scope: `Asset`, `VoucherLeaf`, `RightLeaf`, `FeeEnvelope`, the object policy registry, wallet object inventory, and validator fail-closed enforcement.
- Keep `FeeEnvelope` as processing support only. It must not become right meaning, ownership evidence, or cash authority.
- Keep `wallet.object.*` as typed object inventory only. It must not become a second cash authority plane.
- Preserve object-flow coverage for `voucher_issue_offer`, `voucher_accept`, `voucher_transfer`, `voucher_redeem_full`, `voucher_redeem_partial`, `voucher_reject_refund`, `voucher_expiry`, `right_grant`, `right_delegate`, `right_consume`, `right_revoke`, `right_expiry`, `right_challenge`, `right_gated_voucher_action`, `fee_supported_transition`, and the matching missing-right, expired-right, revoked-right, replay, wrong-family, voucher-as-cash, and right-as-value rejection cases.
- Preserve the Section 32 wallet-extension coverage explicitly: rights listing must stay on `wallet.object.list_rights`, right consumption must stay on `wallet.object.consume_right`, validator locked-asset spend rejection must remain fail-closed, and cash/object separation after restart must remain a tested property rather than an implied side effect of generic inventory.
- Treat payroll and B2B entitlement coverage as local `RightLeaf` plus voucher policy coverage only; do not introduce live service authority or bridge authority.
- Treat agent budget, service entitlement, data-access, and machine capability profiles as local deterministic right/profile evidence only.
- Keep machine capability evidence capability-scoped: one-time use, expiry, wrong object rejection, wrong action rejection, and reuse rejection must not grant full-wallet spend or broad controller privilege.
- Keep external chain trust tiers, linked liability carryover, live cross-chain settlement, live bridge rollout, live external enforcement, and useful-work attestation or oracle evidence out of 063 core.
- If `063-core-examples.md` contains profile names or fields that are not yet consumed by runtime materialization, keep them in live YAML anyway, but require the live loader to parse and validate them through typed `GenesisConfig` sections. Do not allow silently ignored YAML fields.

**Verification gates**

- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_object_flows_matrix_contract -- --nocapture` must pass.
- `cargo test --release -p z00z_simulator --test scenario_1 test_rights_business_entitlement_lifecycle_local -- --nocapture`, `cargo test --release -p z00z_simulator --test scenario_1 test_agentic_right_lifecycle_local -- --nocapture`, and `cargo test --release -p z00z_simulator --test scenario_1 test_machine_capability_lifecycle_local -- --nocapture` must pass.
- `rg -n "voucher_issue_offer|voucher_accept|voucher_transfer|voucher_redeem_full|voucher_redeem_partial|voucher_reject_refund|voucher_expiry|right_grant|right_delegate|right_consume|right_revoke|right_expiry|right_challenge|right_gated_voucher_action|fee_supported_transition" crates/z00z_simulator/tests .planning/phases/063-Core-Update/063-core-examples.md` must prove every positive transferred scenario id is either covered by the simulator matrix or represented as a required example.
- `rg -n "right_missing_for_voucher_action|right_expired_for_voucher_action|right_revoked_for_voucher_action|right_replay_reject|wrong_family_proof_reject|voucher_as_cash_reject|right_as_value_reject|voucher_invalid_backing|voucher_non_transferable_transfer_reject|voucher_forced_acceptance|voucher_double_redeem|voucher_expired_use_reject" crates/z00z_simulator/tests` must prove every negative transferred scenario id is covered by the simulator matrix or a targeted simulator test.
- `rg -n "wallet\\.object\\.list_rights|wallet\\.object\\.consume_right" crates/z00z_wallets/src/rpc` and `rg -n "validator_lock_unlock_after_expiry|validator_lock_unlock_without_right_delta_reject|validator_lock_unlock_replay_reject|test_scenario1_object_flows_wallet_inventory_for_alice_bob_charlie|cash import must not appear in wallet\\.object\\.list_rights" crates/z00z_simulator/tests crates/z00z_wallets/tests` must prove Section 32 wallet-extension coverage is still explicit.
- `rg -n "machine_compute_capability|confidential_data_access|service_entitlement|validator_mandate|one_time_agent_action|voucher_transferable_policy|right_delegate_policy" .planning/phases/063-Core-Update/063-core-examples.md crates/z00z_core/src/genesis` must prove the transferred profile/policy examples are represented in the canonical example source or current config inputs.
- `rg -n "useful[-_]work|live cross-chain|linked liability|live external enforcement|full-wallet spend|broad controller|second cash authority|universal private VM" crates/z00z_core crates/z00z_core/docs crates/z00z_core/README.md wiki/03-core-protocol` must return zero live-scope claims after the cleanup. Explicit exclusion language belongs in planning files, not in live API/docs as a promised capability.

### P1. Make `z00z_core/configs` the only core YAML authority

**Problem**

`z00z_core` now has more than one YAML location, and that makes the real config authority hard to audit.

**Evidence**

- The crate now uses `crates/z00z_core/z00z_config/devnet_genesis_config.yaml`, `devnet_assets_config.yaml`, `devnet_rights_config.yaml`, `devnet_policies_config.yaml`, and `devnet_vouchers_config.yaml` as the live config catalog.
- Live code, docs, and tests still point to YAML under `src/assets`, `src/genesis`, `examples/genesis`, and `tests/vectors`, including [assets_config.yaml](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/assets_config.yaml:1), [genesis_config_devnet.yaml](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_devnet.yaml:1), [genesis_config_devnet_small.yaml](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml:1), [genesis_config_devnet_test.yaml](/home/vadim/Projects/z00z/crates/z00z_core/examples/genesis/genesis_config_devnet_test.yaml:1), and [asset_pack_vectors.yaml](/home/vadim/Projects/z00z/crates/z00z_core/tests/vectors/asset_pack_vectors.yaml:1).
- `063-core-examples.md` is no longer just illustrative; it defines the required example envelope and payload families for assets, rights, policies, vouchers, wallet profiles, and policy profiles.

**Recommendation**

- Create `crates/z00z_core/z00z_config/` as the only YAML root for `z00z_core`.
- Move every core-owned `.yaml` and `.yml` file under that root.
- Remove the legacy `crates/z00z_core/configs/` path as a parallel authority after migration; do not keep both `configs/` and `z00z_config/`.
- Preserve the existing `_config.yaml` filenames from the new catalog, including `devnet_genesis_config.yaml`, `devnet_assets_config.yaml`, `devnet_rights_config.yaml`, `devnet_policies_config.yaml`, and `devnet_vouchers_config.yaml`.
- Replace the legacy `genesis_config_devnet_small.yaml` path with `devnet_genesis_config.yaml` in one coordinated migration. Do not keep an alias, shim, compatibility path, or second live filename.
- Treat `devnet` as the small configuration by default. Do not introduce a second `devnet_small_*_config.yaml` family unless a non-small devnet profile is added with explicit code support.
- Use `063-core-examples.md` as a mandatory implementation source, not as optional sample text.

**Config materialization requirements**

- Every `assets[]`, `rights[]`, `policies[]`, and `vouchers[]` field from `063-core-examples.md` must be represented in real Rust config types, parsed through the live loader, validated, and used by materialization or export code.
- Every live asset and right `domain_name` derived from `063-core-examples.md` must use the canonical pattern `z00z.core.<family>.<name>.devnet.v1`, for example `z00z.core.assets.coin.devnet.v1`.
- `wallet_profiles[]` and `policy_profiles[]` must be real typed live-config sections in `GenesisConfig`, parsed and validated by the live loader even when some fields remain parser-owned before runtime materialization uses them.
- Actions must remain inside `policies[]` as shown by `063-core-examples.md`. Do not keep a standalone `devnet_actions_config.yaml` unless the same wave adds a real parser, validation, and live use path for it.
- Add schema, golden, and roundtrip tests proving that the live `z00z_config` files parse and drive the same object-family generation paths that production uses.

**Migration conflict resolution**

- Treat `z00z_core/z00z_config/` as the target location, not as permission to change the `GenesisConfig` authority model. The loader must still produce one canonical `GenesisConfig`.
- Preserve the current manifest contract unless a later design explicitly replaces it: `manifest_refs` may fan out only `assets`, `rights`, `policies`, and `vouchers`; `chain`, `outputs`, and `performance` stay root-owned.
- Preserve the self-contained small-fixture behavior by moving that behavior into `devnet_genesis_config.yaml`. The old `genesis_config_devnet_small.yaml` file name must not remain live after the migration.
- Update every code path, CLI default, example, test, README, crate doc, and wiki page that currently names `src/genesis/genesis_config_devnet.yaml` or `genesis_config_devnet_small.yaml` in the same wave as the code move. The target-state docs must say `crates/z00z_core/z00z_config/...`.
- Treat tarballs or archived config snapshots under the old legacy paths as migration inputs only. They must not remain in `crates/z00z_core` as a second YAML authority after the move.

**Migration pitfalls**

- Moving YAML before changing `load_genesis_config`, tests, CLI defaults, examples, and docs will strand live callers on missing paths.
- Renaming `genesis_config_devnet_small.yaml` without updating every caller in the same wave will break simulator and release-mode fixture tests. The mitigation is a complete caller rewrite, not a compatibility alias.
- Collapsing the current split root manifest into one file would lose the production-scale `manifest_refs` contract. The location move and the ref semantics are separate concerns.
- Keeping `devnet_actions_config.yaml` in the live config root without parser and validator support would contradict the current loader's explicit rejection of top-level actions lanes.
- Adding `wallet_profiles[]` or `policy_profiles[]` into live genesis YAML without typed parser and validation support would create dead configuration fields. Adding them with live parser ownership is required scope.

**Verification gates**

- `find crates/z00z_core -path '*/target' -prune -o \( -name '*.yaml' -o -name '*.yml' \) -type f | sort` must show only `crates/z00z_core/z00z_config/...` paths.
- `perl -ne 'print if /domain_name:/ && !/domain_name: z00z\.core\.[a-z_]+\.[a-z0-9_]+\.devnet\.v1$/' .planning/phases/063-Core-Update/063-core-examples.md` must print nothing, and equivalent parser or fixture checks must enforce the same pattern for live `z00z_config` asset and right entries.
- `rg -n "src/(assets|genesis)/.*\\.ya?ml|examples/.+\\.ya?ml|tests/vectors|crates/z00z_core/configs" crates/z00z_core` must return zero stale references after the migration.
- `cargo test -p z00z_core test_genesis_manifest_refs -- --nocapture` and the canonical config parser tests must pass against `z00z_config`.
- `rg -n "src/genesis/genesis_config_devnet|genesis_config_devnet_small|crates/z00z_core/configs|devnet_actions_config" wiki crates/z00z_core/docs crates/z00z_core/README.md crates/z00z_core` must return zero stale target-state references.

**Doublecheck note**

The evidence for this recommendation was checked against the current file tree, `Cargo.toml`, direct YAML references in code/docs/tests, wiki manifest-ref pages, the Phase 062 YAML decision note, and `063-core-examples.md`. The main risk is over-moving generated or third-party YAML; keep the migration scoped to `crates/z00z_core` unless a later phase explicitly widens it.

### P2. Flatten test ownership and align names with the repository standard

**Problem**

Tests are spread across owner-misaligned paths, nested suites, and object-family gaps.

**Evidence**

- `policies/mod.rs` pulls its tests from `../assets/test_policy_descriptor.rs` in [policies/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/policies/mod.rs:17).
- `vouchers/mod.rs` still pulls its tests from `../assets/test_voucher_config.rs` in [vouchers/mod.rs](/home/vadim/Projects/z00z/crates/z00z_core/src/vouchers/mod.rs:16).
- `src/assets` and `src/genesis` still contain files such as `test_asset_suite.rs`, `test_definition_suite.rs`, `test_leaf_suite.rs`, `test_genesis_suite.rs`, and `test_validator_suite.rs`.
- Integration tests are nested under `tests/assets` and `tests/genesis`, while `tests/rights` and `tests/vauchers` exist without object-family test files.
- The design foundation says all Rust test files must use the `test_*.rs` shape and rejects `*_suite.rs` naming in [Z00Z_DESIGN_FOUNDATION.md](/home/vadim/Projects/z00z/.github/requirements/Z00Z_DESIGN_FOUNDATION.md:3482).
- Existing rename-guard coverage already encodes some of the current suite filenames, so any cleanup wave must update those guards at the same time.

**Recommendation**

- Flatten `crates/z00z_core/tests` so Rust integration tests live directly under `tests/`.
- Keep `crates/z00z_core/tests/fixtures` as the only allowed subdirectory under `tests/`.
- Move YAML vectors out of `tests/vectors` into `z00z_config`; tests should load them from the canonical config root or use Rust fixtures.
- Rename `*_suite.rs` files to canonical `test_*` names.
- Rename numeric asset files from the current noisy shape to the compact owner shape. Example: `test_integration_assets_test1.rs` becomes `test_assets_1.rs`.
- Prefer meaningful names when the file has a clear behavior theme, for example `test_assets_nonce_domains.rs` instead of `test_assets_9.rs`.
- Add explicit object-family test files for rights and vouchers under the flattened test root.
- Use `vouchers` in new and renamed test file names. Do not add legacy-lane assertions for `vauchers`; delete or rewrite old `vauchers` references during the canonical caller rewrite.
- Remove `#[path = "../assets/..."]` cross-owner test wiring once the files are moved.
- Update rename-guard tests in the same wave so the repository does not fail on intentional normalization.

**Required coverage**

- Rights tests must cover config parsing, terminal id determinism, policy binding, lifecycle windows, payload commitments, and zero-value authority semantics.
- Voucher tests must cover bootstrap materialization, policy binding, replay nonce derivation, lifecycle state, backing, refund target, partial redemption, and expired voucher rejection.
- Asset tests must keep the existing coverage but move under clear flattened names.
- Genesis tests must keep full-bootstrap parity and manifest golden coverage after path changes.

**Verification gates**

- `find crates/z00z_core/tests -mindepth 2 -type f ! -path '*/tests/fixtures/*'` must return zero files.
- `rg -n "test_integration_assets_test|_suite\\.rs|tests/(assets|genesis|rights|vauchers|vectors)" crates/z00z_core/Cargo.toml crates/z00z_core/tests crates/z00z_core/src` must return zero stale owned-test paths after migration.
- `cargo test -p z00z_core --tests` must pass after the move.

**Why this is the right fix**

This reduces reader confusion, makes object-family coverage visible, and brings the crate closer to the repository's own test-layout constitution.

### P2. Restore `z00z_core/docs` against live code

**Problem**

The `z00z_core/docs` directory is no longer a reliable live-code guide unless it is audited in the same wave as config and layout changes.

**Evidence**

- `z00z_core/docs` contains asset and genesis documents that still reference old config paths such as `assets_config.yaml`, `src/assets/assets_config.yaml`, and `config/genesis_config_devnet_small.yaml`.
- The docs directory also contains a non-ASCII claim document filename, which conflicts with the repository's English-only artifact policy.

**Recommendation**

- Audit every file in `crates/z00z_core/docs` against the live code, `Cargo.toml`, examples, tests, and the new `z00z_config` layout.
- Replace stale paths with `crates/z00z_core/z00z_config/...`.
- Remove or rewrite examples that refer to nonexistent APIs, disabled doctest-only paths, old config names, or old binary/example paths.
- Rename documentation files to English ASCII names and keep headings/content in English.
- Keep docs focused on real public APIs and live behavior; when a Phase 063 field is parser-owned live scope before full runtime use exists, document that state explicitly instead of hiding the field as planning-only.
- During the same layout wave, audit support-surface Markdown under `crates/z00z_core/bin`, `benches`, `examples`, and `tests`. Those files currently carry old nested paths and can reintroduce drift even if `docs/` itself is cleaned.

**Verification gates**

- `rg -n "src/assets/.*\\.ya?ml|src/genesis/.*\\.ya?ml|config/genesis|assets_config\\.yaml|examples/(assets|genesis)|benches/(assets|genesis)|bin/(assets|genesis)" crates/z00z_core/docs crates/z00z_core/README.md` must return zero stale references.
- `rg -n "src/assets/.*\\.ya?ml|src/genesis/.*\\.ya?ml|config/genesis|assets_config\\.yaml|examples/(assets|genesis)|benches/(assets|genesis)|bin/(assets|genesis)|[А-Яа-я]" crates/z00z_core/bin crates/z00z_core/benches crates/z00z_core/examples crates/z00z_core/tests -g '*.md'` must return zero support-documentation drift after the flatten wave.
- `cargo doc -p z00z_core --no-deps` must build without new documentation warnings when public APIs or rustdoc examples change.

### P2. Flatten benches, binaries, examples, and crate responsibility

**Problem**

`z00z_core` currently carries protocol contracts, registry/config loaders, genesis export tooling, ZIP packaging, multiple binaries, examples, benches, and large test inventory under one crate boundary, and several of those support surfaces are nested by historical owner names instead of current workflow names.

**Evidence**

- The manifest includes multiple binaries and benches in [Cargo.toml](/home/vadim/Projects/z00z/crates/z00z_core/Cargo.toml:75) and [Cargo.toml](/home/vadim/Projects/z00z/crates/z00z_core/Cargo.toml:105).
- Default features also pull in CLI and export-related capabilities in [Cargo.toml](/home/vadim/Projects/z00z/crates/z00z_core/Cargo.toml:43).
- Bench paths are nested under `benches/assets` and `benches/genesis`.
- Binary paths are nested under `bin/assets` and `bin/genesis`.
- Example paths are nested under `examples/assets` and `examples/genesis`, and one example carries its own YAML config.

**Recommendation**

- Keep protocol contracts, deterministic generation semantics, and stable domain types in `z00z_core`.
- Move operator-facing CLIs and heavyweight export packaging toward a dedicated tooling crate or at least stricter feature gating.
- Revisit whether `clap` and export-oriented features should be part of the default library feature set.
- Flatten `crates/z00z_core/benches`: put bench files directly under `benches/`, keep bench names subject-first, and update every `[[bench]]` path in `Cargo.toml`.
- Flatten `crates/z00z_core/bin`: put binary entrypoints and their support modules directly under `bin/`, keep `Cargo.toml` `[[bin]]` paths explicit, and avoid nested `assets` or `genesis` folders.
- Flatten `crates/z00z_core/examples`: put `.rs` examples directly under `examples/`, remove example-local YAML, load all example config from `z00z_config`, and update every `[[example]]` path.
- Keep one README per support surface when useful, but do not use nested README files to preserve old directory ownership.

**Verification gates**

- `find crates/z00z_core/benches -mindepth 2 -type f`, `find crates/z00z_core/bin -mindepth 2 -type f`, and `find crates/z00z_core/examples -mindepth 2 -type f` must return zero files after flattening.
- `rg -n "path = \"(benches|bin|examples)/.*/" crates/z00z_core/Cargo.toml` must return zero nested support paths.
- `cargo bench -p z00z_core --no-run`, `cargo build -p z00z_core --bins`, and `cargo test -p z00z_core --examples` must pass after the move.

**Why this is the right fix**

This recommendation is weaker than the P0/P1 semantic issues, but it is still useful. Because `z00z_core` sits near the center of the workspace dependency graph, unnecessary responsibility inside this crate increases downstream rebuild cost and broadens the blast radius of routine changes.

## Recommended Execution Order

1. **Truth-restoration wave**
   - fix `performance.num_threads` behavior or remove the false promise;
   - define artifact-path semantics precisely;
   - clean public docs that directly contradict live code.

2. **Boundary-normalization wave**
   - make `z00z_core::genesis::*` the only canonical public owner;
   - remove `include!` assembly from the boundary module;
   - reduce global-registry write ownership.

3. **Migration and hygiene wave**
   - introduce `vouchers` as canonical path;
   - preserve bounded voucher, right, fee-envelope, and machine-capability scenario coverage while keeping broad claims excluded;
   - clean test ownership and naming;
   - begin compile-surface extraction if build-time pressure remains material.

4. **Config and support-layout wave**
   - move core YAML into `crates/z00z_core/z00z_config`;
   - materialize the required `063-core-examples.md` fields in code before putting them into live config;
   - flatten `benches`, `bin`, `examples`, and `tests`;
   - sync `z00z_core/docs` with the new code and config paths.

## Doublecheck Verdict

I checked this recommendation set against the most likely concept-drift traps:

- I did **not** use `.planning/graphs` as proof of concrete behavior; graph data was used only to rank downstream blast radius.
- I did **not** recommend splitting bootstrap authority away from `GenesisConfig`; the recommendation is to reinforce that authority.
- I did **not** treat `GLOBAL_ASSET_REGISTRY` as forbidden by principle; the recommendation is to clarify ownership and reduce dual-write behavior.
- I did **not** claim that post-genesis asset minting already has a fully explicit runtime contract; I recommend documenting or constraining that area because it is currently less explicit than voucher/right runtime flows.
- I scoped the "all YAML under `z00z_config`" recommendation to `crates/z00z_core`, because this document is a `z00z_core` recommendation and the wider repository contains unrelated YAML owners.
- I found a real migration conflict with the current wiki and Phase 062 notes: they treat `src/genesis/genesis_config_devnet.yaml` plus `manifest_refs` and the self-contained `genesis_config_devnet_small.yaml` fixture as live anchors. This TODO resolves that by making `z00z_config` the target-state path, preserving `GenesisConfig` and `manifest_refs` semantics, and rejecting old file names as live aliases or shims.
- `wallet_profiles[]` and `policy_profiles[]` from `063-core-examples.md` are live genesis sections for parser and validation scope now. If a field is not yet consumed by runtime materialization, that gap must be explicit, but the field must still be parsed and validated through live code.
- I checked `063-core-examples.md` `domain_name` entries as a separate config-consistency gate: target-state examples use `z00z.core.<family>.<name>.devnet.v1`, and live config migration must enforce the same pattern for asset and right entries.
- I moved only the verified core-relevant object-family provisions from the old local implementation packet into this TODO: bounded assets, vouchers, rights, fee envelope support, wallet object inventory, agentic rights, entitlement scenarios, and machine capability scenarios. I did **not** move local adapter, live bridge, linked liability, useful-work attestation, or broad production-matrix claims into 063 core.
- I checked the flatten recommendations against the current `Cargo.toml` support paths and the live nested `benches`, `bin`, `examples`, and `tests` directories.
- I checked the rights and voucher test recommendation against the current empty `tests/rights` and `tests/vauchers` directories plus the existing genesis-level rights and voucher tests.
- I checked the docs recommendation against the current `crates/z00z_core/docs` file list and stale path references in docs and README files.

## Final Assessment

`z00z_core` is structurally important and conceptually close to a good shape, but it currently leaks too much historical layering. The best next work is not a broad rewrite. It is a sequence of targeted cleanup waves that restore one source of truth for:

- public paths;
- bootstrap ownership;
- runtime configuration behavior;
- object-family semantics;
- core YAML authority;
- support layout;
- docs and test ownership.

That sequence should improve correctness, maintainability, reviewability, and build hygiene without changing the protocol model unnecessarily.
