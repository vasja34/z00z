# Z00Z Core Benches

Live Criterion targets now sit directly under `crates/z00z_core/benches/`.

## Run

```bash
cargo bench --release -p z00z_core --no-run
cargo bench -p z00z_core --bench genesis_bench
cargo bench -p z00z_core --bench metadata_validation_bench
```

## Targets

- `commitment_properties_bench`
- `gas_calculation_bench`
- `genesis_bench`
- `metadata_ops_bench`
- `metadata_validation_bench`
- `registry_bench`

## Guardrails

- `bench_helpers.rs` is a local helper module, not a separate benchmark target
- compare regressions in release mode only
- treat current benchmark output as the authority, not pasted historical numbers
- larger genesis semantics and operator notes belong to
  `crates/z00z_core/docs/GENESIS_DOCUMENTATION.md`
