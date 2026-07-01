# Z00Z Core Binaries

Live support binaries now sit directly under `crates/z00z_core/bin/`.

## Canonical Bootstrap Owner

`genesis_cli.rs` is the only live generator entry point in this directory.

```bash
cargo run --release --bin genesis_cli -- \
  --config configs/devnet_genesis_config.yaml
```

It wraps `z00z_core::genesis::run_genesis` and writes:

- per-asset `genesis_<SYMBOL>.json` and `genesis_<SYMBOL>.bin`
- `genesis_rights.json`
- `genesis_policies.json`
- `genesis_vouchers.json`
- `genesis_settlement_manifest.json`

## Other Live Tools

- `assets_analyzer_cli.rs` inspects generated artifact directories or specific
  files
- `assets_extractor_cli.rs` filters generated per-asset exports
- `assets_generation_cli.rs` is the registry-catalog bulk export tool
- `assets_allocator_cli.rs` is an experimental scaffold source file and is not
  a registered `[[bin]]` target or a second bootstrap authority

## Feature Boundary

- `assets_generation_cli` stays inside `z00z_core` for Phase 063, but its
  operator CLI dependency is named explicitly through the `cli` feature
- JSON/Bincode and snapshot ZIP exports remain crate-owned because
  `run_genesis()` and related support tools emit them directly

## Support Docs

- `ASSETS_GENERATION_CLI_SUMMARY.md` is the canonical command summary for the
  bulk asset export tool
- there is no old generator alias in this directory
