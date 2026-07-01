# Testing Patterns

**Analysis Date:** 2026-05-06

## Test Framework

**Runner:**

- Rust built-in test harness via `cargo test` across the workspace.
- `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` is the canonical repo-wide gate.
- Criterion benches are registered in `crates/z00z_core/Cargo.toml`, `crates/z00z_crypto/Cargo.toml`, `crates/z00z_storage/Cargo.toml`, and `crates/z00z_wallets/Cargo.toml`.
- `.cargo/config.toml` defines `cargo t` and `cargo rt` aliases and enables the `test_fast` cfg for local runs.

**Assertion Library:**

- Standard Rust assertions are the norm: `assert!`, `assert_eq!`, `assert_ne!`, `matches!`, `expect`, and `unwrap_err`.
- Property assertions use Proptest macros such as `prop_assert_eq!` and `prop_assert_ne!` in `crates/z00z_core/tests/assets/test_property_based.rs` and `crates/z00z_core/tests/genesis/test_security_validation.rs`.

**Run Commands:**

```bash
cargo test --workspace
cargo test --workspace --release --all-targets --all-features
cargo test --workspace --release --all-features --doc
cargo bench --workspace --all-features --no-run
./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh
```

## Test File Organization

**Location:**

- `z00z_core` mixes unit tests inside source files with integration suites under `crates/z00z_core/tests/assets/` and `crates/z00z_core/tests/genesis/`.
- `z00z_crypto` primarily uses flat integration tests under `crates/z00z_crypto/tests/` plus Criterion benches under `crates/z00z_crypto/benches/`.
- `z00z_simulator` relies heavily on integration tests under `crates/z00z_simulator/tests/` and uses fixture data in `crates/z00z_simulator/tests/fixtures/`.
- `z00z_storage` uses both unit-style test modules in source files such as `crates/z00z_storage/src/assets/model_tests.rs` and integration suites under `crates/z00z_storage/tests/`.
- `z00z_wallets` has the broadest integration surface under `crates/z00z_wallets/tests/`, plus targeted unit tests embedded inside source modules.

**Naming:**

- Flat integration tests are usually named `test_<behavior>.rs`, especially in `crates/z00z_simulator/tests/` and `crates/z00z_wallets/tests/`.
- Suite entry points use functional names rather than `test_`, for example `crates/z00z_core/tests/assets/test_assets.rs`, `crates/z00z_storage/tests/test_assets_suite.rs`, and `crates/z00z_storage/tests/test_snapshot_suite.rs`.
- Benchmark files describe the hot path they measure, for example `crates/z00z_wallets/benches/tx_perf_bench.rs` and `crates/z00z_core/benches/assets/metadata_validation_bench.rs`.

**Structure:**

```text
crates/z00z_core/tests/
├── assets/
│   ├── mod.rs
│   ├── test_assets.rs
│   ├── test_fixtures.rs
│   └── test_property_based.rs
├── genesis/
│   ├── mod.rs
│   ├── test_genesis.rs
│   └── test_security_validation.rs

crates/z00z_storage/tests/
├── assets/
│   ├── mod.rs
│   └── test_store_api.rs
├── checkpoint/
│   └── test_fixtures.rs
├── snapshot/
│   ├── mod.rs
│   └── test_versions.rs
├── test_assets_suite.rs
└── test_snapshot_suite.rs
```

## Test Structure

**Suite Organization:**

```rust
// crates/z00z_core/tests/assets/test_assets.rs
#[path = "mod.rs"]
mod assets;
```

```rust
// crates/z00z_storage/tests/test_assets_suite.rs
mod assets;
```

```rust
// crates/z00z_wallets/tests/test_common/mod.rs
pub struct WalletEnvGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Drop for WalletEnvGuard {
    fn drop(&mut self) {
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");
    }
}
```

**Patterns:**

- Integration suites frequently expose a single entry file that includes a sibling `mod.rs` and lets Cargo treat the file as the test crate.
- Shared test helpers are factored into reusable support modules, for example `crates/z00z_wallets/tests/test_common/mod.rs`, `crates/z00z_simulator/tests/support/test_stage4_support.rs`, and `crates/z00z_storage/tests/checkpoint/test_fixtures.rs`.
- Test functions are concrete and scenario-focused. Examples include `test_store_public_api_roundtrip` in `crates/z00z_storage/tests/assets/test_store_api.rs`, `test_h2scalar_golden_vectors` in `crates/z00z_crypto/tests/test_golden_vectors.rs`, and `stage2_wallet_ids_unique` in `crates/z00z_simulator/tests/test_wallet_integration.rs`.
- Release-only integration gates are encoded in tests rather than external tooling. `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs` exits early on debug builds and writes pass artifacts when running in release mode.

## Mocking

**Framework:**

- No dedicated mocking framework is detected.
- The codebase prefers real domain objects, temporary directories, helper constructors, and in-memory implementations.

**Patterns:**

```rust
// crates/z00z_wallets/tests/test_common/mod.rs
pub struct WalletEnvGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Drop for WalletEnvGuard {
    fn drop(&mut self) {
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");
    }
}
```

```rust
// crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs
let (logger, vec_logger) = common::rpc_test_tee_logger();
let transport = LoggedRpcTransport::new(base, config, logger, time, rng);
```

**What to Mock:**

- Prefer fakeable environment and filesystem boundaries with temp directories, helper guards, or structured fixtures.
- Prefer `z00z_utils` abstractions and deterministic inputs over external service mocks.

**What NOT to Mock:**

- Cryptographic derivation, proof generation, and canonical encoding paths are usually exercised directly with golden vectors or parity tests.
- Storage proofs and path validation are tested against real store operations, as shown in `crates/z00z_storage/tests/assets/test_store_api.rs`.

## Fixtures and Factories

**Test Data:**

```rust
// crates/z00z_storage/tests/assets/test_store_api.rs
fn test_item(path: AssetPath, value: u64) -> StoreItem {
    StoreItem::new(path, test_leaf(path.asset_id, path.serial_id.get(), value)).expect("item")
}
```

```rust
// crates/z00z_simulator/tests/test_wallet_integration.rs
fn patch_cfg_paths(cfg_path: &Path) -> PathBuf {
    let mut cfg = ScenarioCfg::from_file(...).expect("load scenario_config.yaml");
    cfg.stage1_genesis.get_or_insert_with(Default::default).genesis_config = ...;
    ...
}
```

**Location:**

- Simulator fixtures live under `crates/z00z_simulator/tests/fixtures/`.
- Wallet fixtures live under `crates/z00z_wallets/tests/fixtures/` and `crates/z00z_wallets/tests/test_common/`.
- Checkpoint-specific fixtures live under `crates/z00z_storage/tests/checkpoint/test_fixtures.rs`.
- Some suites generate temporary patched configs or output artifacts under `target/tmp` or crate-local output directories instead of relying only on checked-in fixture files.

## Coverage

**Requirements:**

- No explicit numeric coverage threshold is detected in the target crates.
- Quality gates emphasize breadth of suites, deterministic vectors, property tests, performance checks, and full-workspace verification rather than a coverage percentage.

**View Coverage:**

```bash
Not detected in the target crates.
```

## Test Types

**Unit Tests:**

- Embedded module tests are common for validation-heavy code, for example `crates/z00z_core/src/genesis/validator.rs`, `crates/z00z_storage/src/assets/model_tests.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`, and many modules under `crates/z00z_wallets/src/core/`.
- These tests target invariants, parser round-trips, internal state transitions, and small helper behavior.

**Integration Tests:**

- This is the dominant pattern for `z00z_simulator` and `z00z_wallets`.
- Integration tests often cross crate boundaries, exercising `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_utils`, and `z00z_wallets` together.
- Security regression tests are first-class integration tests, for example `crates/z00z_wallets/tests/test_wallet_persist_nostd_fs.rs`, `crates/z00z_simulator/tests/test_claim_persist.rs`, and `crates/z00z_simulator/tests/test_stealth_scanner_cache.rs`.

**E2E Tests:**

- E2E-style tests exist but remain inside the Rust test harness rather than a separate framework.
- Examples include `crates/z00z_wallets/tests/test_e2e_req_flow.rs`, `crates/z00z_simulator/tests/test_claim_persist.rs`, and the artifact-producing unified gate in `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`.

## Common Patterns

**Async Testing:**

```rust
// Bench and service tests use a real Tokio runtime.
let rt = Runtime::new().expect("tokio runtime");
rt.block_on(async {
    let wallets = service.list_wallets_in_memory().await.expect("list_wallets_in_memory must succeed");
    criterion::black_box(wallets);
})
```

- Native-only async testing is common in `z00z_wallets`, which depends on Tokio in both implementation and dev dependencies.
- Tests also use `#[tokio::test]` for async RPC and service flows when a dedicated runtime is enough.

**Error Testing:**

```rust
let err = chk_blob(bytes, root, path, wrong_def, proof.ser_leaf(), leaf)
    .expect_err("wrong definition leaf must fail");
assert_eq!(err, ProofChkErr::DefMix);
```

```rust
let err = parse_params::<WalletIdParams>(json!({"id": "wallet-abc"})).unwrap_err();
assert!(matches!(err, RpcError::InvalidParams(_)));
```

- Error-path assertions are strongly typed and often check exact enum variants rather than only checking `is_err()`.
- Several tests validate negative security cases by mutating serialized JSON or proof bytes and asserting the correct reject class.

**Property Testing:**

```rust
proptest! {
    #[test]
    fn test_rebuild_idx_keeps_root(
        marks in proptest::collection::vec((1u8..4, 1u32..4, 1u8..8, 1u8..16), 1..8)
    ) {
        ...
        prop_assert_eq!(root_many(&left), root_many(&right));
    }
}
```

- Proptest is actively used in `crates/z00z_core/tests/assets/test_property_based.rs`, `crates/z00z_core/tests/genesis/test_security_validation.rs`, `crates/z00z_storage/src/assets/model_tests.rs`, `crates/z00z_storage/src/assets/store_internal/whitebox_state.rs`, and `crates/z00z_wallets/src/core/key/bip32.rs`.

**Performance Guards:**

```rust
const MAX_BATCH_VERIFY_MS: u128 = 500;
assert!(elapsed.as_millis() < MAX_BATCH_VERIFY_MS);
```

- Hard performance thresholds appear inside tests, not only in benchmarks. `crates/z00z_crypto/tests/test_perf_guards.rs` is the clearest example.
- Criterion benchmarks are present for hot paths in core, crypto, storage, and wallets, and `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` runs `cargo bench --workspace --all-features --no-run` as part of the global gate.

## Quality Gates

**Observed gates across the target crates:**

- `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` enforces `cargo fmt --check`, workspace `clippy` with denied warnings, release builds for all targets, release test runs for all targets, doctests, benchmark compilation, runnable target sweeps, and slow-test reporting.
- `crates/z00z_core/bin/run_fast_tests.sh` defines a curated fast lane for library, genesis, determinism, proof, snapshot, and security tests.

---

Testing analysis: 2026-05-06
