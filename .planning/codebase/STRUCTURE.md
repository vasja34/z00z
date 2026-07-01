# Codebase Structure

**Analysis Date:** 2026-05-06

## Directory Layout

```text
z00z/
├── Cargo.toml
├── assets/
├── config/
├── crates/
│   ├── z00z_core/
│   ├── z00z_crypto/
│   ├── z00z_extensions/
│   ├── z00z_networks/
│   │   ├── onionnet/
│   │   └── rpc/
│   ├── z00z_rollup_node/
│   ├── z00z_runtime/
│   │   ├── aggregators/
│   │   ├── validators/
│   │   └── watchers/
│   ├── z00z_simulator/
│   ├── z00z_storage/
│   ├── z00z_telemetry/
│   ├── z00z_utils/
│   └── z00z_wallets/
├── data/
├── deploy/
├── docker/
├── docs/
├── logs/
├── outputs/
├── reports/
├── scripts/
├── tools/
├── target/
└── website/
```

## Directory Purposes

**`crates/`:** workspace source root.

- Contains the active first-party Rust crates listed in the root `Cargo.toml`.
- Also contains `z00z-offline/`, which is present in the tree but not listed as a workspace member.

**`crates/z00z_core/`:** protocol-domain crate.

- Contains asset models, deterministic genesis logic, hashing, and protocol-facing domain types.
- Key files: `crates/z00z_core/src/lib.rs`, `crates/z00z_core/src/assets/`, `crates/z00z_core/src/genesis/`, `crates/z00z_core/src/domains.rs`, `crates/z00z_core/src/hashing.rs`.

**`crates/z00z_crypto/`:** public crypto facade with vendor-isolated backend.

- Contains the public crypto API and the private Tari-backed implementation boundary.
- Key files: `crates/z00z_crypto/src/lib.rs`, `crates/z00z_crypto/src/aead.rs`, `crates/z00z_crypto/src/hash.rs`, `crates/z00z_crypto/src/kdf.rs`, `crates/z00z_crypto/src/protocol/`, `crates/z00z_crypto/src/vendor.rs`, `crates/z00z_crypto/tari/`.

**`crates/z00z_storage/`:** canonical persistence and replay crate.

- Contains storage contracts for assets, checkpoints, serialization, snapshots, and inspection helpers.
- Key files: `crates/z00z_storage/src/lib.rs`, `crates/z00z_storage/src/assets/`, `crates/z00z_storage/src/checkpoint/`, `crates/z00z_storage/src/serialization/`, `crates/z00z_storage/src/snapshot/`, `crates/z00z_storage/src/vizualization/`.

**`crates/z00z_wallets/`:** wallet ownership and orchestration crate.

- Contains the facade in `crates/z00z_wallets/src/core.rs`, plus `receiver/`, `tx/`, `services/`, `persistence/`, `db/`, `adapters/rpc/`, `wasm/`, and the wallet-owned domain trees.
- Key files: `crates/z00z_wallets/src/lib.rs`, `crates/z00z_wallets/src/core.rs`, `crates/z00z_wallets/src/receiver/`, `crates/z00z_wallets/src/tx/`, `crates/z00z_wallets/src/services/`, `crates/z00z_wallets/src/db/`, `crates/z00z_wallets/src/wasm/`.

**`crates/z00z_networks/`:** transport crates.

- `rpc/` hosts request dispatch and transport-only plumbing.
- `onionnet/` hosts the privacy-overlay placeholder boundary.

**`crates/z00z_runtime/`:** policy crates.

- `aggregators/`, `validators/`, and `watchers/` each own a narrow policy or reporting slice.
- Keep these crates small and boundary-focused.

**`crates/z00z_rollup_node/`:** rollup verification crate.

- Contains proof verification and settlement-facing logic.
- Key files: `crates/z00z_rollup_node/src/lib.rs`, `crates/z00z_rollup_node/src/config.rs`, `crates/z00z_rollup_node/src/da_adapter.rs`, `crates/z00z_rollup_node/src/lifecycle.rs`, `crates/z00z_rollup_node/src/rpc.rs`, `crates/z00z_rollup_node/src/status.rs`.

**`crates/z00z_simulator/`:** staged integration harness.

- Contains scenario config, design, event, result, and `scenario_1/` stage orchestration.
- `crates/z00z_simulator/src/scenario_1/outputs/` stores scenario-local artifacts.

**`crates/z00z_utils/`:** shared utilities.

- Contains the cross-cutting abstractions for config, codec, I/O, logging, metrics, RNG, time, compression, and OS hardening.

**`crates/z00z_extensions/`:** empty boundary crate.

- The root crate is currently empty.
- Future manifests live under `crates/z00z_extensions/.todo/`.

**`crates/z00z_telemetry/`:** empty boundary crate.

- The root crate currently exposes an empty `src/lib.rs` boundary.

**Repo-level directories:**

- `assets/` contains colors, fonts, icons, images, logos, and sounds.
- `config/` contains repo-wide configuration such as `config/z00z_blockchain_config.yaml`.
- `docs/` contains long-form design, research, and protocol notes.
- `scripts/` contains maintenance and validation helpers such as `scripts/play_tone.sh`.
- `tools/` contains developer tooling and browser automation helpers.
- `reports/` contains generated analysis and verification output.
- `outputs/` contains generated artifacts and backups.
- `website/` contains website content and snapshots.
- `target/` is Cargo build output and should not contain source files.

## Key File Locations

**Entry points:**

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

**Configuration:**

- `versions.yaml`: version inventory used by repo tooling.
- `webdriver.json`: browser automation configuration.
- `config/z00z_blockchain_config.yaml`: blockchain configuration.
- `crates/z00z_core/configs/devnet_assets_config.yaml`: secondary asset-registry catalog for examples, fixtures, and compatibility flows.
- `crates/z00z_core/configs/devnet_genesis_config.yaml`: canonical devnet genesis manifest, with referenced rights, policies, and vouchers subfiles.
- `crates/z00z_wallets/src/wallet_config.yaml`: wallet configuration.
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`: simulator scenario config.
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`: simulator stage design.

**Core logic:**

- `crates/z00z_core/src/assets/`: asset model, registry, validation, and wire formats.
- `crates/z00z_core/src/genesis/`: deterministic genesis generation and validation.
- `crates/z00z_crypto/src/`: hash, KDF, AEAD, claim, secret, protocol, and vendor boundary code.
- `crates/z00z_storage/src/checkpoint/`: checkpoint artifact and build flow.
- `crates/z00z_storage/src/serialization/`: storage serialization and restoration.
- `crates/z00z_storage/src/snapshot/`: snapshot construction and restoration.
- `crates/z00z_wallets/src/receiver/`: inbound receiver logic.
- `crates/z00z_wallets/src/tx/`: transaction assembly, verification, and state flow.
- `crates/z00z_wallets/src/services/`: orchestration services.
- `crates/z00z_wallets/src/db/`: native persistence backend.
- `crates/z00z_wallets/src/wasm/`: browser-compatible storage and client boundaries.
- `crates/z00z_simulator/src/scenario_1/`: staged integration flow.

**Testing:**

- `crates/z00z_core/tests/`: protocol and genesis tests.
- `crates/z00z_crypto/tests/`: crypto tests and benchmarks.
- `crates/z00z_storage/tests/`: storage and checkpoint tests.
- `crates/z00z_wallets/tests/`: wallet tests.
- `crates/z00z_simulator/tests/`: scenario and pipeline tests.

## Naming Conventions

**Files:**

- Rust modules use `snake_case.rs` or `mod.rs` inside a directory boundary.
- File-backed facades use a single root file such as `core.rs` in `crates/z00z_wallets/src/`.
- Scenario stages use `stage_*.rs` names, such as `stage_1.rs` and `stage_11.rs`.

**Directories:**

- Feature and domain directories are grouped by responsibility rather than by technical artifact type.
- Public crate facades live at `src/lib.rs`; deeper subdomains live under folders such as `assets/`, `checkpoint/`, `receiver/`, `tx/`, or `scenario_1/`.

## Where to Add New Code

**New protocol logic:**

- Primary code: `crates/z00z_core/src/assets/` or `crates/z00z_core/src/genesis/`
- Tests: `crates/z00z_core/tests/`

**New crypto helper:**

- Primary code: `crates/z00z_crypto/src/`
- Tests: `crates/z00z_crypto/tests/`
- Never add backend code under `crates/z00z_crypto/tari/`.

**New storage model or checkpoint flow:**

- Primary code: `crates/z00z_storage/src/assets/`, `crates/z00z_storage/src/checkpoint/`, `crates/z00z_storage/src/serialization/`, or `crates/z00z_storage/src/snapshot/`
- Tests: `crates/z00z_storage/tests/`

**New wallet workflow:**

- Primary code: `crates/z00z_wallets/src/receiver/` for inbound flows or `crates/z00z_wallets/src/tx/` for transaction logic.
- Shared orchestration belongs in `crates/z00z_wallets/src/services/`.
- Tests: `crates/z00z_wallets/tests/`

**New wallet persistence backend:**

- Native backend: `crates/z00z_wallets/src/db/`
- Persistence abstraction: `crates/z00z_wallets/src/persistence/`
- Wasm backend: `crates/z00z_wallets/src/wasm/`

**New RPC or transport code:**

- Wallet-facing RPC: `crates/z00z_wallets/src/adapters/rpc/`
- Transport plumbing: `crates/z00z_networks/rpc/`
- Privacy transport: `crates/z00z_networks/onionnet/`

**New runtime policy:**

- `crates/z00z_runtime/aggregators/`, `crates/z00z_runtime/validators/`, or `crates/z00z_runtime/watchers/`

**New rollup behavior:**

- `crates/z00z_rollup_node/src/`

**New simulator stage or scenario helper:**

- `crates/z00z_simulator/src/scenario_1/`
- Keep stage-specific logic in separate `stage_*.rs` files.

**New shared utility:**

- `crates/z00z_utils/`

## Special Directories

**`crates/z00z_crypto/tari/`:**

- Purpose: vendored Tari cryptography implementation.
- Generated: No.
- Committed: Yes.
- Handling rule: read-only vendor boundary.

**`crates/z00z_wallets/src/core.rs`:**

- Purpose: file-backed wallet ownership boundary and facade.
- Generated: No.
- Committed: Yes.
- Handling rule: keep app-owned and wallet-owned imports separated here.

**`crates/z00z_storage/src/vizualization/`:**

- Purpose: inspection-oriented storage view boundary.
- Generated: No.
- Committed: Yes.
- Handling rule: keep it separate from storage mutation paths.

**`crates/z00z_extensions/.todo/`:**

- Purpose: planned extension sub-crates.
- Generated: No.
- Committed: Yes.
- Handling rule: treat as future work, not active workspace surface.

**`target/`:**

- Purpose: Cargo build output.
- Generated: Yes.
- Committed: No.
- Handling rule: do not add source files here.

---

Structure analysis date: 2026-05-06
