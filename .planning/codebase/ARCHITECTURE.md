# Architecture

**Analysis Date:** 2026-05-06

## System Overview

```text
Workspace root
├── Foundation
│   ├── `crates/z00z_utils/`
│   └── `crates/z00z_crypto/`
├── Protocol and state
│   ├── `crates/z00z_core/`
│   └── `crates/z00z_storage/`
├── Wallet, runtime, and rollup
│   ├── `crates/z00z_wallets/`
│   ├── `crates/z00z_runtime/*`
│   └── `crates/z00z_rollup_node/`
└── Transport and replay
    ├── `crates/z00z_networks/*`
    └── `crates/z00z_simulator/`
```

## Component Responsibilities

| Component | Responsibility | File |
| --- | --- | --- |
| Workspace manifest | Declares active workspace members and shared dependency policy | `Cargo.toml` |
| `z00z_utils` | Cross-cutting config, codec, I/O, logging, metrics, RNG, time, and compression | `crates/z00z_utils/src/lib.rs` |
| `z00z_crypto` | Public crypto facade over the read-only Tari backend | `crates/z00z_crypto/src/lib.rs` |
| `z00z_core` | Protocol domain for assets, genesis, domains, and hashing | `crates/z00z_core/src/lib.rs` |
| `z00z_storage` | Checkpoints, snapshots, serialization, and durable storage contracts | `crates/z00z_storage/src/lib.rs` |
| `z00z_wallets` | Wallet ownership, receiver, tx, services, persistence, RPC adapters, and wasm | `crates/z00z_wallets/src/lib.rs` |
| `z00z_aggregators` | Aggregation and publication policies | `crates/z00z_runtime/aggregators/src/lib.rs` |
| `z00z_validators` | Checkpoint, claim, spend, and tx validation policies | `crates/z00z_runtime/validators/src/lib.rs` |
| `z00z_watchers` | Operational watches and evidence/reporting | `crates/z00z_runtime/watchers/src/lib.rs` |
| `z00z_networks_rpc` | Transport-only RPC dispatch and local/WASM client plumbing | `crates/z00z_networks/rpc/src/lib.rs` |
| `onionnet` | Placeholder privacy-overlay boundary | `crates/z00z_networks/onionnet/src/lib.rs` |
| `z00z_rollup_node` | Settlement verification and rollup-facing limits | `crates/z00z_rollup_node/src/lib.rs` |
| `z00z_simulator` | Scenario orchestration and integration replay | `crates/z00z_simulator/src/lib.rs` |
| `z00z_extensions` | Empty boundary crate for future extension surfaces | `crates/z00z_extensions/src/lib.rs` |
| `z00z_telemetry` | Empty boundary crate for future telemetry surfaces | `crates/z00z_telemetry/src/lib.rs` |

## Pattern Overview

**Overall:** layered Rust workspace with narrow crate facades and private implementation subtrees.

**Key characteristics:**

- Public crates expose small root facades in `src/lib.rs`.
- Wallet ownership is split through `crates/z00z_wallets/src/core.rs` into `app_owned` and `wallet_owned` imports.
- `crates/z00z_crypto/tari/` is the read-only vendor boundary.
- Runtime policy is split into three small crates rather than one broad service crate.
- `z00z_networks/rpc` stays transport-only and does not own peer, auth, or retry policy.
- `z00z_simulator` runs staged scenarios rather than ad hoc integration logic.

## Layers

### Foundation

- Purpose: shared utilities and cryptographic primitives.
- Location: `crates/z00z_utils/`, `crates/z00z_crypto/`
- Depends on: standard Rust ecosystem crates and vendored Tari sources under `crates/z00z_crypto/tari/`
- Used by: core, storage, wallets, runtime, rollup, and simulator crates

### Protocol and state

- Purpose: canonical protocol types and durable state contracts.
- Location: `crates/z00z_core/src/`, `crates/z00z_storage/src/`
- Contains: `assets/`, `genesis/`, `domains.rs`, `hashing.rs`, `checkpoint/`, `serialization/`, `snapshot/`
- Depends on: `z00z_utils` and `z00z_crypto`
- Used by: wallets, runtime, rollup, and simulator

### Wallet application

- Purpose: wallet ownership boundaries, service orchestration, persistence, and user-facing adapters.
- Location: `crates/z00z_wallets/src/`
- Contains: `app/`, `backup/`, `chain/`, `claim/`, `core.rs`, `db/`, `domains/`, `key/`, `network/`, `persistence/`, `receiver/`, `security/`, `services/`, `stealth/`, `tx/`, `wallet/`, `adapters/`, `wasm/`, `egui_views/`
- Depends on: `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_utils`, and `z00z_networks_rpc`
- Used by: wallet binaries, simulator scenarios, and rollup settlement checks

### Runtime and rollup

- Purpose: small policy crates and settlement verification.
- Location: `crates/z00z_runtime/*`, `crates/z00z_rollup_node/`
- Contains: `aggregators/`, `validators/`, `watchers/`, and rollup node modules in `config.rs`, `da_adapter.rs`, `lifecycle.rs`, `mode.rs`, `rpc.rs`, `status.rs`
- Depends on: storage, wallet, crypto, utils, and runtime policy crates
- Used by: orchestration, validation, and settlement flows

### Transport and simulation

- Purpose: transport plumbing and staged replay.
- Location: `crates/z00z_networks/`, `crates/z00z_simulator/`
- Contains: `rpc/`, `onionnet/`, `actors.rs`, `context.rs`, `design.rs`, `event.rs`, `result.rs`, `rng_mode.rs`, `scenario_1/`
- Depends on: wallet, storage, core, crypto, utils, and runtime surfaces
- Used by: transport consumers and integration-style simulation runs

## Data Flow

### Primary request path

1. `crates/z00z_utils/src/lib.rs` provides shared abstractions for config, I/O, time, logging, RNG, codec, and compression.
2. `crates/z00z_crypto/src/lib.rs` exposes the public crypto facade and hides Tari behind the vendor boundary.
3. `crates/z00z_core/src/lib.rs` defines protocol types, domain hashing, and deterministic genesis.
4. `crates/z00z_storage/src/lib.rs` persists checkpoints, snapshots, and serialization artifacts.
5. `crates/z00z_wallets/src/lib.rs` composes receiver, tx, service, persistence, and adapter modules.
6. `crates/z00z_runtime/*`, `crates/z00z_rollup_node/src/lib.rs`, and `crates/z00z_simulator/src/lib.rs` consume those lower layers for validation, settlement, and replay.

### Transport path

1. `crates/z00z_networks/rpc/src/lib.rs` exposes `RpcTransport`, `RpcDispatcher`, and `LocalRpcTransport`.
2. Wallet adapters and simulator code compose that transport without taking ownership of peer identity or retry policy.
3. `crates/z00z_networks/onionnet/src/lib.rs` remains reserved for node-owned privacy overlay work.

### State management

- Canonical durable state lives in `crates/z00z_storage/`.
- Wallet-private state lives in `crates/z00z_wallets/src/db/`, `crates/z00z_wallets/src/persistence/`, and `crates/z00z_wallets/src/wasm/`.
- Simulator-local state lives in `crates/z00z_simulator/src/context.rs` and `crates/z00z_simulator/src/scenario_1/outputs/`.

## Key Abstractions

**Wallet ownership boundary:**

- Purpose: keep app-owned and wallet-owned imports separate.
- Examples: `crates/z00z_wallets/src/core.rs`
- Pattern: `app_owned` and `wallet_owned` facades, with helper facades for cache, ecdh, kdf, scan, and zkpack.

**Protocol and genesis model:**

- Purpose: define assets, domains, and chain identity once and reuse them everywhere else.
- Examples: `crates/z00z_core/src/assets/`, `crates/z00z_core/src/genesis/`
- Pattern: shared domain types flow from `z00z_core` into storage, wallet, simulator, and rollup code.

**Checkpoint and snapshot contracts:**

- Purpose: preserve replayable state transitions and durable artifacts.
- Examples: `crates/z00z_storage/src/checkpoint/`, `crates/z00z_storage/src/snapshot/`
- Pattern: state writes are mediated by storage contracts rather than ad hoc files.

**Transport-only RPC:**

- Purpose: provide reusable request dispatch without taking over higher-level network policy.
- Examples: `crates/z00z_networks/rpc/src/lib.rs`
- Pattern: transport and local test helpers stay separate from authentication, peer, and lifecycle policy.

**Scenario staging:**

- Purpose: keep integration scenarios deterministic and replayable.
- Examples: `crates/z00z_simulator/src/scenario_1/runner.rs`, `crates/z00z_simulator/src/scenario_1/stage_*.rs`
- Pattern: explicit stage files make scenario flow easy to inspect and update.

## Entry Points

- `Cargo.toml`: workspace membership and dependency policy.
- `crates/z00z_core/src/lib.rs`: protocol facade.
- `crates/z00z_crypto/src/lib.rs`: crypto facade.
- `crates/z00z_storage/src/lib.rs`: storage facade.
- `crates/z00z_wallets/src/lib.rs`: wallet facade.
- `crates/z00z_wallets/src/core.rs`: wallet ownership boundary.
- `crates/z00z_networks/rpc/src/lib.rs`: transport facade.
- `crates/z00z_rollup_node/src/lib.rs`: rollup verification facade.
- `crates/z00z_simulator/src/lib.rs`: simulator facade.
- `crates/z00z_simulator/bin/scenario_1.rs`: scenario entrypoint.

## Architectural Constraints

- `crates/z00z_crypto/tari/` is read-only vendor code.
- `z00z_utils` is the shared abstraction layer for config, time, I/O, logging, RNG, and codec.
- `z00z_networks/rpc` is transport-only.
- `crates/z00z_wallets/src/core.rs` separates app-owned and wallet-owned imports.
- `z00z_runtime` is intentionally split into `aggregators`, `validators`, and `watchers`.
- `z00z_extensions` and `z00z_telemetry` are currently empty boundary crates.
- `crates/z00z-offline/` exists under `crates/` but is not declared in the root workspace members.

## Anti-Patterns

### Bypassing shared utilities

**What happens:** config, time, I/O, logging, RNG, or codec code goes straight to low-level libraries from business crates.
**Why it's wrong:** it fragments policy and makes cross-cutting behavior harder to audit.
**Do this instead:** use `crates/z00z_utils/src/lib.rs` and its public abstractions.

### Flattening wallet logic into the facade

**What happens:** new wallet behavior is added directly to `crates/z00z_wallets/src/lib.rs` or `crates/z00z_wallets/src/core.rs`.
**Why it's wrong:** it obscures ownership boundaries and makes the wallet surface harder to maintain.
**Do this instead:** keep receiver work in `crates/z00z_wallets/src/receiver/`, transaction work in `crates/z00z_wallets/src/tx/`, and orchestration in `crates/z00z_wallets/src/services/`.

### Treating transport crates as business logic

**What happens:** RPC or onion transport code starts owning wallet or protocol state transitions.
**Why it's wrong:** it mixes concerns and makes the transport layer harder to test and evolve.
**Do this instead:** keep state transitions in wallet, storage, or runtime layers and keep `crates/z00z_networks/*` transport-only.

### Editing vendored crypto directly

**What happens:** changes land under `crates/z00z_crypto/tari/`.
**Why it's wrong:** it breaks the vendor boundary and couples the workspace to unsupported upstream changes.
**Do this instead:** add Z00Z-owned wrappers or adapters in `crates/z00z_crypto/src/`.

## Error Handling

**Strategy:** use crate-local typed errors, validate at the lowest sensible layer, and normalize failures at boundaries.

**Patterns:**

- `z00z_crypto` owns crypto-specific errors.
- `z00z_storage` owns checkpoint and serialization errors in `crates/z00z_storage/src/error.rs`.
- `z00z_wallets` keeps wallet-facing errors near `crates/z00z_wallets/src/core.rs` and wallet-owned modules.
- `z00z_networks/rpc` uses transport errors in `crates/z00z_networks/rpc/src/error.rs`.
- `z00z_rollup_node` returns explicit settlement errors for theorem, link, root, and inclusion checks.
- `z00z_simulator` returns structured scenario results from `crates/z00z_simulator/src/result.rs`.

## Cross-Cutting Concerns

**Logging:** structured logging goes through `z00z_utils` helpers.

**Validation:** validation is layered: crypto validates primitives, core validates protocol and genesis data, storage validates persisted artifacts, wallet code validates ownership and session invariants, and rollup code validates public settlement bundles.

**Configuration:** YAML-backed configuration lives beside the code that consumes it, such as `crates/z00z_wallets/src/wallet_config.yaml` and `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`. `z00z_core` keeps its canonical live config root under `crates/z00z_core/configs/`: `crates/z00z_core/configs/devnet_genesis_config.yaml` is the bootstrap manifest, while `crates/z00z_core/configs/devnet_assets_config.yaml` is secondary registry data.

**Observability:** runtime watchers in `crates/z00z_runtime/watchers/` provide reporting and health surfaces for orchestration and validation flows.

---

Architecture analysis date: 2026-05-06
