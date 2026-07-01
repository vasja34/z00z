# z00z_core

`z00z_core` is the live protocol crate for assets, canonical genesis bootstrap,
policies, rights, and vouchers.

Use the curated crate root for the stable runtime surface:

- `z00z_core::{Asset, AssetClass, AssetDefinition, AssetDefinitionRegistry, AssetLeaf, AssetPkgWire, AssetWire, ChainType}`

Use owner modules for higher-level helpers and typed config surfaces:

- `z00z_core::assets::...` for asset-only helpers, registry, snapshots, gas, and policy flags
- `z00z_core::genesis::...` for canonical bootstrap orchestration and typed generation artifacts
- `z00z_core::actions::...` for action-pool contracts
- `z00z_core::policies::...` for policy descriptor contracts
- `z00z_core::rights::...` for rights config and right semantics
- `z00z_core::vouchers::...` for voucher config and voucher semantics

The removed compatibility shims under
`z00z_core::assets::{action_pool,policy_descriptor,right_config,voucher_config}`
must not return.

## Bootstrap Authority

`z00z_core::genesis` is the single canonical bootstrap authority for assets,
rights, policies, and vouchers. `GenesisConfig` is the canonical typed manifest
for live bootstrap.

`z00z_core::config_paths::DEVNET_ASSETS_CONFIG_REL` and
`AssetDefinitionRegistry::load_catalog_from_yaml()` remain secondary
registry-data inputs for examples, fixtures, and compatibility flows only.
They do not compete with `GenesisConfig`, and they must not be described as a
second bootstrap path.

## Canonical Object-Family Semantics

Use [`docs/OBJECT_FAMILY_SEMANTICS.md`](docs/OBJECT_FAMILY_SEMANTICS.md) as the
single live matrix for assets, rights, policies, and vouchers.

- `z00z_core::ObjectFamily` is the canonical caller-visible family vocabulary.
  `z00z_core::assets::ObjectFamily` remains a compatibility facade only.
- `VoucherBootstrapEntryV1` is bootstrap input only; runtime vouchers persist
  as `VoucherLeaf`.
- Rights are zero-value authority objects and must not be described as
  carrying value.
- `mintable` is a definition/catalog flag on the current tree, not a proven
  generic post-genesis asset-mint API.

## Quick Start

```rust,no_run
# use std::path::Path;
# use std::sync::Arc;
use z00z_core::assets::registry::AssetDefinitionRegistry;
use z00z_core::config_paths::{DEVNET_ASSETS_CONFIG_REL, DEVNET_GENESIS_CONFIG_REL};
use z00z_core::genesis::genesis_config::load_genesis_config;
use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let genesis = load_genesis_config(DEVNET_GENESIS_CONFIG_REL)?;
println!("chain={} assets={}", genesis.chain.name, genesis.assets.len());

let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
    Path::new(DEVNET_ASSETS_CONFIG_REL),
    Arc::new(NoopLogger),
    Arc::new(NoopMetrics),
    Arc::new(SystemTimeProvider),
)?;
println!(
    "registry version={} definitions={}",
    registry.get_version()?,
    registry.len()?
);
# Ok(())
# }
```

## Live Config Surfaces

- `configs/devnet_genesis_config.yaml` - canonical root manifest for the
  live split devnet config
- `DEVNET_ASSETS_CONFIG_REL` - secondary registry-data catalog constant for
  examples, fixtures, and compatibility flows

Split manifest files are allowed through `manifest_refs`, but they must
rehydrate into one `GenesisConfig` before validation or generation.

## Registry Snapshot Flow

`AssetDefinitionRegistry` owns the live registry snapshot contract through
`create_snapshot()` and `update_from_snapshot()`. That lane is for registry
synchronization, not for replacing genesis authority.

## Genesis Outputs

`run_genesis()` writes a timestamped export directory rooted at
`GenesisConfig.outputs.assets_export_path`, per-asset JSON/Bincode exports such
as `genesis_<SYMBOL>.json` and `genesis_<SYMBOL>.bin`, and the typed bootstrap
artifacts:

- `genesis_rights.json`
- `genesis_policies.json`
- `genesis_vouchers.json`
- `genesis_settlement_manifest.json`

Snapshot ZIP exports remain rooted at
`GenesisConfig.outputs.snapshot_export_path`.

## Documentation Guardrails

`Cargo.toml` keeps `doctest = false`, so the doc contract is pinned by
`tests/test_live_guardrails.rs` and release-mode validation scans instead of
rustdoc execution alone.
