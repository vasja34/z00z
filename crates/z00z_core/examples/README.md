# Z00Z Core Examples

This directory keeps small runnable examples around the live `z00z_core`
public API. All example entrypoints now live directly under
`crates/z00z_core/examples/`.

## Live Targets

```bash
cargo run --release --bin assets_generation_cli -- --help
cargo run --release --example asset_config_loading
cargo run --release --example asset_registry_basic
cargo run --release --example asset_registry_with_metrics
cargo run --release --example asset_snapshot
cargo run --release --example genesis_example
```

## Canonical Inputs

- `genesis_example` uses `configs/devnet_genesis_config.yaml`.
- `assets_generation_cli` defaults to
  `z00z_core::config_paths::DEVNET_ASSETS_CONFIG_REL`.
- The registry catalog is a secondary asset-definition input only. Bootstrap
  authority stays under `z00z_core::genesis::GenesisConfig`.
- `asset_config_loading` synthesizes temporary YAML under `tempfile`; there is
  no live example-local YAML checked into this support surface.

## Notes

- `assets_generation_cli` is a binary target, not an example target.
- Example code in this directory demonstrates narrow API slices only.
- Historical nested owner folders are gone; the flat root is the only
  canonical example path.
- Operator workflows and larger architecture notes belong to the crate README
  and the documents under `docs/`.
