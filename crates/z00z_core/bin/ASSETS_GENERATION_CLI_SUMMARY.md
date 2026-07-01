# Asset Generation CLI Summary

`assets_generation_cli` is the live bulk-generation binary for registry-backed
asset exports.

## Target Shape

- Cargo target kind: `[[bin]]`
- Target name: `assets_generation_cli`
- Source path: `crates/z00z_core/bin/assets_generation_cli.rs`
- Default catalog constant:
  `z00z_core::config_paths::DEVNET_ASSETS_CONFIG_REL`
- Default output root: `crates/z00z_core/outputs/assets`
- Feature boundary: `cli` + `deterministic-rng`

## Release Commands

```bash
cargo run --release --bin assets_generation_cli -- --help
cargo run --release --bin assets_generation_cli -- --format json --verbose
cargo run --release --bin assets_generation_cli -- --format bincode --config <catalog-path>
```

## Behaviour

- loads a secondary registry catalog through
  `AssetDefinitionRegistry::load_catalog_from_yaml`
- generates release-mode asset corpora with deterministic test parameters
- verifies commitments, proofs, and signatures before export
- writes JSON or Bincode bundles plus report files under the output root

## Guardrails

- this target does not replace `z00z_core::genesis` as bootstrap authority
- docs must not use obsolete `--example assets_generation_cli` commands
- when the catalog location changes, update the config-path constant instead of
  scattering new literal paths through docs
