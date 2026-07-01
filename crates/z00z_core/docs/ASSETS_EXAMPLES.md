# Assets Live Surfaces

This document records the live assets-facing examples and operator commands
after the Phase 063 path normalization work.

## Choose the Right Entry Point

- `assets_generation_cli` is the release-mode binary for bulk asset generation
  and export from a secondary registry catalog.
- `asset_config_loading` is the smallest typed catalog-loading example.
- `asset_registry_basic` shows registry insertion and lookup.
- `asset_registry_with_metrics` adds logger and metrics wiring.
- `asset_snapshot` demonstrates snapshot export and import flow.

## Release Commands

```bash
cargo run --release --bin assets_generation_cli -- --help
cargo run --release --bin assets_generation_cli -- --format json --verbose
cargo run --release --bin assets_generation_cli -- --format bincode --config <catalog-path>

cargo run --release --example asset_config_loading
cargo run --release --example asset_registry_basic
cargo run --release --example asset_registry_with_metrics
cargo run --release --example asset_snapshot
```

## Canonical Config Inputs

- Bootstrap authority: `configs/devnet_genesis_config.yaml`
- Secondary registry catalog constant:
  `z00z_core::config_paths::DEVNET_ASSETS_CONFIG_REL`
- The registry catalog is not a second bootstrap path. All bootstrap authority
  stays under `z00z_core::genesis::GenesisConfig`.

## Minimal Registry Load

```rust
use std::path::Path;
use std::sync::Arc;
use z00z_core::assets::registry::AssetDefinitionRegistry;
use z00z_core::config_paths::DEVNET_ASSETS_CONFIG_REL;
use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};

let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
    Path::new(DEVNET_ASSETS_CONFIG_REL),
    Arc::new(NoopLogger),
    Arc::new(NoopMetrics),
    Arc::new(SystemTimeProvider),
)?;
```

## Output Contract

- The default output root is `crates/z00z_core/outputs/assets`.
- JSON and Bincode files are secondary artifacts for tooling, examples, and
  tests.
- Registry snapshots remain the sync surface for definitions.
- Live bootstrap generation still belongs to `z00z_core::genesis` and
  `genesis_cli`.

## Guardrails

- Use `cargo run --release --bin assets_generation_cli`, not
  `cargo run --example assets_generation_cli`.
- Avoid example-local YAML paths and stale nested references in new docs.
- Prefer `z00z_core::config_paths::*` constants over scattering literal config
  strings through examples.
