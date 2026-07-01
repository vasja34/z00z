# Coding Conventions

**Analysis Date:** 2026-05-06

## Naming Patterns

**Files:**

- Rust source files use `snake_case` names for modules and implementation files, for example `crates/z00z_utils/src/logger/mod.rs`, `crates/z00z_wallets/src/adapters/rpc/logging/config.rs`, and `crates/z00z_core/src/lib.rs`.
- Integration test files are usually prefixed with `test_`, especially in `crates/z00z_simulator/tests/` and `crates/z00z_wallets/tests/`.
- Some crates also use suite entry points instead of the `test_` prefix, for example `crates/z00z_core/tests/assets/test_assets.rs`, `crates/z00z_core/tests/genesis/test_genesis.rs`, `crates/z00z_storage/tests/test_assets_suite.rs`, and `crates/z00z_storage/tests/test_snapshot_suite.rs`.

**Functions:**

- Functions and methods use `snake_case`, including helpers with explicit domain intent such as `wallet_env_lock`, `rpc_test_tee_logger`, `patch_cfg_paths`, `build_tx_stealth_output`, and `test_rpc_logging_sink_jsonl`.
- Test names are descriptive and behavior-focused rather than generic, for example `test_batch_verification_performance` in `crates/z00z_crypto/tests/test_perf_guards.rs` and `stage2_keys_json_no_secrets` in `crates/z00z_simulator/tests/test_wallet_integration.rs`.
- Helper function names are short but semantic, for example `test_path`, `test_leaf`, `assert_blob_err`, `root_dir`, and `wallet_env_lock`.

**Variables:**

- Local variables generally use domain names instead of single letters: `wallet_id`, `proof`, `cfg_patched`, `wallet_ids_large`, `owner_handle`, and `definition_id`.
- Constants use `SCREAMING_SNAKE_CASE`, for example `MAX_BATCH_VERIFY_MS` in `crates/z00z_crypto/tests/test_perf_guards.rs` and `ACTORS` in `crates/z00z_simulator/tests/test_wallet_integration.rs`.
- Type aliases preserve domain vocabulary and often end with `Result`, `Error`, `Id`, or `Params`, for example `CheckResult`, `WalletResult`, `JsonHandlerFuture`, and `WalletIdPasswordParams`.

**Types:**

- Public types use `PascalCase`, for example `AssetDefinitionRegistry` in `crates/z00z_core/src/assets/registry.rs`, `ScenarioCfg` in `crates/z00z_simulator/src/lib.rs`, `CheckpointError` in `crates/z00z_storage/src/lib.rs`, and `WalletEnvGuard` in `crates/z00z_wallets/tests/test_common/mod.rs`.
- Domain-specific wrapper types are preferred over primitive values where the API boundary matters, for example `AssetId`, `DefinitionId`, `SerialId`, `PersistWalletId`, `SessionToken`, and `ReceiverSecret`.

## Code Style

**Formatting:**

- The workspace targets Rust 2021 and centralizes lint policy in `Cargo.toml` at the repository root.
- Crates use crate-level attributes to make style rules explicit. `crates/z00z_core/src/lib.rs` sets `#![forbid(unsafe_code)]` plus `#![warn(rust_2018_idioms, unused_qualifications, unreachable_pub)]`. `crates/z00z_storage/src/lib.rs` and `crates/z00z_wallets/src/lib.rs` also forbid unsafe code.
- `crates/z00z_wallets/src/lib.rs` adds `#![warn(missing_docs)]`, so public APIs in that crate are expected to carry documentation.
- Source files often use dense rustdoc and section banners for large modules, especially `crates/z00z_core/src/assets/registry.rs` and `crates/z00z_wallets/src/lib.rs`.

**Linting:**

- Workspace lint policy is defined in `Cargo.toml` under `[workspace.lints.clippy]`.
- Vendor crates under `crates/z00z_crypto/tari/` are treated as read-only, and a small set of Clippy lints is explicitly allowed there so workspace-wide verification can still pass.
- The strongest observed gate is `cargo clippy --workspace --release --all-targets --all-features -- -D warnings`, defined in `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh`.
- `crates/z00z_crypto/README.md` also documents crate-scoped Clippy gates for both native and WASM targets.

## Import Organization

**Order:**

1. External crates first, often grouped by crate, for example `use criterion::{criterion_group, criterion_main, Criterion};` in `crates/z00z_wallets/benches/wallet_service_bench.rs`.
2. Standard library imports next, often grouped with braces, for example `use std::{path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};` in `crates/z00z_simulator/tests/test_claim_persist.rs`.
3. Internal crate imports last, usually grouped by subsystem, for example `use z00z_wallets::core::{...};` in `crates/z00z_wallets/tests/test_e2e_req_flow.rs`.

**Path Aliases:**

- No custom Rust path alias system is used. Crates import each other by published crate name such as `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_utils`, and `z00z_wallets`.
- Internal module imports typically use `super::...` inside implementation modules, for example `crates/z00z_core/src/assets/registry.rs`.

## Error Handling

**Patterns:**

- Typed error enums using `thiserror::Error` are the default pattern across the target crates. This is widespread in `crates/z00z_crypto/src/error.rs`, `crates/z00z_storage/src/error.rs`, `crates/z00z_simulator/src/design.rs`, and many modules under `crates/z00z_wallets/src/core/`.
- Result aliases are localized per subsystem, for example `CheckResult<T>` in `crates/z00z_storage/src/error.rs`, `WalletResult<T>` in `crates/z00z_wallets/src/core/wallet/errors.rs`, `SignerResult<T>` in `crates/z00z_wallets/src/core/tx/signer.rs`, and `BackupImporterResult<T>` in `crates/z00z_wallets/src/core/backup/backup_importer.rs`.
- `anyhow::Result` appears only at implementation seams where third-party traits require it, for example JMT integration in `crates/z00z_storage/src/assets/store.rs`. Public APIs still expose typed errors.
- `expect` and `unwrap` are common inside tests, examples, benches, and invariant-heavy helpers, but production-facing crates rely on `Result`-based propagation.

## Logging

**Framework:** `tracing`

**Patterns:**

- Logging is selective and mostly concentrated in `z00z_wallets`, especially persistence, RPC, and scanning code such as `crates/z00z_wallets/src/services/wallet_service.rs`, `crates/z00z_wallets/src/adapters/rpc/logging/config.rs`, and `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs`.
- `z00z_core` uses very little live logging in the target surface; the visible `tracing` reference in `crates/z00z_core/src/assets/gas.rs` is documentation-oriented.
- Test code uses dedicated logger helpers rather than ad hoc stdout. `crates/z00z_wallets/tests/test_common/mod.rs` builds a tee logger that writes to both a file sink and `VecLogger`.
- Logging is treated as a controlled surface with configuration objects and middleware, not as direct `println!` calls.
- Logging is selective and mostly concentrated in `z00z_wallets`, especially persistence, RPC, and scanning code such as `crates/z00z_wallets/src/services/wallet_service.rs`, `crates/z00z_wallets/src/adapters/rpc/logging/config.rs`, and `crates/z00z_wallets/tests/test_rpc_logging_file_sink.rs`.

## Comments

**When to Comment:**

- Large modules begin with architectural rustdoc that explains responsibility boundaries, invariants, and related files. `crates/z00z_core/src/assets/registry.rs` is the clearest example.
- Security-sensitive or test-sensitive behavior is documented inline, for example the process-global environment warning in `crates/z00z_wallets/tests/test_common/mod.rs` and the release-mode gate in `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`.
- Comments are frequently used to mark feature intent and operational risk, such as `wallet_debug_dump` in `crates/z00z_simulator/Cargo.toml` and `wallet_debug_tools` in `crates/z00z_wallets/Cargo.toml`.

**JSDoc/TSDoc:**

- Not applicable. The target subsystems are Rust crates using rustdoc (`//!` and `///`).
- `#![doc = include_str!("../README.md")]` is used in `crates/z00z_crypto/src/lib.rs` and `crates/z00z_storage/src/lib.rs` to treat the crate README as the top-level API document.

## Function Design

**Size:**

- Small utility modules use concise functions, but domain orchestrators can be large. Examples include `crates/z00z_wallets/src/services/wallet_service.rs` and `crates/z00z_wallets/src/db/redb_wallet_store.rs`.
- When modules grow large, the codebase compensates with helper types, support modules, and heavy documentation rather than aggressive file splitting.

**Parameters:**

- APIs prefer typed wrappers and structured params over positional primitives at boundaries, for example `WalletIdPasswordParams` and `WalletShowSeedPhraseParams` in `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs`.
- Shared state is often passed as `Arc<T>` when crossing async or service boundaries, for example service setup in `crates/z00z_wallets/benches/wallet_service_bench.rs`.

**Return Values:**

- Public and subsystem APIs return typed `Result` aliases wherever failures are part of normal flow.
- Serialization boundaries often return `serde_json::Value` only inside RPC or simulator plumbing, for example `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs` and `crates/z00z_simulator/src/scenario_1/stage_4.rs`.
- Validation helpers commonly return booleans only after all typed failure paths have already been converted or checked.

## Module Design

**Exports:**

- Each crate exposes a curated facade from `src/lib.rs`, then re-exports stable domain modules and selected types. This is visible in all five target crates.
- `crates/z00z_crypto/src/lib.rs` is intentionally a facade over internal backend modules, with `backend` and `backend_tari` kept private.
- `crates/z00z_wallets/src/lib.rs` re-exports `core`, `adapters`, `services`, feature-gated modules, and stable address or stealth types to keep callers off internal file paths.

**Barrel Files:**

- Barrel-style `mod.rs` files are used heavily for submodule boundaries, for example `crates/z00z_storage/src/assets/mod.rs`, `crates/z00z_wallets/src/services/mod.rs`, and `crates/z00z_simulator/src/scenario_1/mod.rs`.
- Integration test suites also use barrel entry points. `crates/z00z_core/tests/assets/test_assets.rs` and `crates/z00z_core/tests/genesis/test_genesis.rs` delegate to sibling `mod.rs` files.

## Architectural Rules

**Observed rules in code:**

- Use `z00z_utils` abstractions for I/O, config, codec, logging, and time instead of direct low-level APIs. This is stated in crate manifests for `crates/z00z_core/Cargo.toml` and `crates/z00z_wallets/Cargo.toml`, used throughout tests and services, and enforced by regression tests such as `crates/z00z_wallets/tests/test_wallet_persist_nostd_fs.rs`.
- Treat `crates/z00z_crypto/tari/` as read-only vendor code. The workspace lint configuration explicitly preserves this boundary.
- Keep backend internals hidden behind facades. `crates/z00z_crypto/src/lib.rs` exposes public crypto primitives while keeping backend modules private.
- Enforce compile-time feature exclusivity where security policy depends on configuration. `crates/z00z_wallets/src/lib.rs` uses `compile_error!` to require exactly one ownership policy feature unless the dual-override feature is set.
- Separate native-only and WASM-only code with explicit `cfg` blocks instead of runtime branching, especially in `crates/z00z_wallets/src/lib.rs`, `crates/z00z_wallets/Cargo.toml`, and `crates/z00z_simulator/Cargo.toml`.
- Use release-mode or gated tests for heavyweight end-to-end checks. `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs` returns early in debug builds and writes artifact files under `crates/z00z_simulator/src/scenario_1/outputs/tests/`.
- Preserve deterministic behavior through domain-separated derivation, golden vectors, and explicit regression tests across crypto and wallet code.

## Project-Specific Practices

**Security and persistence:**

- Wallet persistence code is guarded by literal source-inspection tests to prevent reintroduction of forbidden filesystem calls, for example `crates/z00z_wallets/tests/test_wallet_persist_nostd_fs.rs`.
- Simulator and wallet tests explicitly scan output JSON for forbidden secret fields, for example `stage2_keys_json_no_secrets` in `crates/z00z_simulator/tests/test_wallet_integration.rs`.
- Debug-only features that could leak sensitive information are explicitly marked as unsafe for production in `crates/z00z_simulator/Cargo.toml` and `crates/z00z_wallets/Cargo.toml`.

**Test support conventions:**

- Shared integration helpers live in dedicated support files instead of being duplicated, for example `crates/z00z_wallets/tests/test_common/mod.rs`, `crates/z00z_wallets/tests/support/`, and `crates/z00z_storage/tests/checkpoint/fixtures.rs`.
- Tests that touch process-global state serialize access with locks and RAII guards rather than assuming single-threaded execution.

---

Convention analysis date: 2026-05-06
