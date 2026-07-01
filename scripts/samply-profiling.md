## 🎯 Samply Profiling

`cargo-samply` is installed for this workspace and standardized through
`./scripts/profile_samply.sh`.

The workspace defines a dedicated Cargo profile:

```toml
[profile.samply]
inherits = "release"
debug = 1
strip = "none"
```

This keeps release optimizations while preserving enough symbols for readable
profiles.

### ✅ List supported Z00Z targets

```bash
./scripts/profile_samply.sh list
```

### ✅ Profile a binary

```bash
./scripts/profile_samply.sh --package z00z_simulator --bin scenario_1
```

### ✅ Profile a benchmark

```bash
./scripts/profile_samply.sh --package z00z_wallets --bench tx_perf_bench -- --bench
```

### ✅ Profile a test target

```bash
./scripts/profile_samply.sh --package z00z_storage --test test_bench_lanes -- --nocapture
```

### ⚙️ Notes

- The wrapper accepts only workspace `z00z_*` packages.
- App arguments must go after `--`, exactly like `cargo samply`.
- If you need raw `cargo samply` flags, pass them through the wrapper.
