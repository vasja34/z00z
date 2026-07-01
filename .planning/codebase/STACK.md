# Technology Stack

**Analysis Date:** 2026-05-06

## Languages

**Primary:**
- Rust 2021 - Workspace baseline for `crates/z00z_core`, `crates/z00z_storage`, `crates/z00z_utils`, `crates/z00z_wallets`, `crates/z00z_networks/rpc`, `crates/z00z_networks/onionnet`, `crates/z00z_runtime/aggregators`, `crates/z00z_runtime/validators`, `crates/z00z_runtime/watchers`, `crates/z00z_rollup_node`, `crates/z00z_simulator`, `crates/z00z_extensions`, and `crates/z00z_telemetry` from the root `Cargo.toml`.
- Rust 2018 - `crates/z00z_crypto` remains on edition 2018 while exporting the shared cryptographic surface used by the rest of the workspace.
- TypeScript 5 - Website application code under `website/website_2025-09-30/`.

**Secondary:**
- JavaScript / JSX - Next.js config, React components, and build-time helper code in `website/website_2025-09-30/`.
- Bash - Local build and utility scripts in `scripts/cargo_build.sh` and `scripts/play_tone.sh`.
- Python 3 - Build orchestration in `scripts/cargo_build.py`.
- YAML and JSON - Repository configuration and runtime metadata in `versions.yaml`, `config/z00z_blockchain_config.yaml`, `webdriver.json`, `crates/z00z_wallets/webdriver.json`, and `website/website_2025-09-30/package-lock.json`.

## Runtime

**Environment:**
- Native Rust runtime across the workspace, with `rust-version = 1.90.0` declared at the root.
- Browser/WASM runtime for wallet and RPC surfaces through target-specific dependencies in `crates/z00z_wallets/Cargo.toml`, `crates/z00z_networks/rpc/Cargo.toml`, and `crates/z00z_crypto/Cargo.toml`.
- Node.js runtime for the website subtree, with the active container build pinned to `node:18-alpine` in `website/website_2025-09-30/Dockerfile`.
- Tokio async runtime in the RPC, wallet, and simulator crates.
- Headless Firefox for WebDriver-based browser automation via `webdriver.json` and `crates/z00z_wallets/webdriver.json`.

**Package Manager:**
- Cargo - Primary package manager and workspace build tool.
- npm - Website package manager under `website/website_2025-09-30/`.
- Lockfiles present - `Cargo.lock` at the workspace root and `website/website_2025-09-30/package-lock.json` for the web app.

## Frameworks

**Core Rust:**
- Workspace resolver 2 - Declared in the root `Cargo.toml`.
- `z00z_utils` abstractions - Shared codec, RNG, time, compression, and metrics boundary used across the Rust crates.
- `tokio` 1 - Async runtime for `crates/z00z_networks/rpc`, `crates/z00z_wallets`, and `crates/z00z_simulator`.
- `jsonrpsee` 0.26 - JSON-RPC transport in `crates/z00z_networks/rpc` and `crates/z00z_wallets/src/adapters/rpc/`.
- `redb` - Embedded wallet and storage backends in `crates/z00z_wallets/src/db/` and `crates/z00z_storage/src/assets/store_internal/`.
- `jmt` 0.12.0 - Jellyfish Merkle Tree storage engine in `crates/z00z_storage/src/assets/store.rs`.
- `tracing` - Logging and instrumentation in the Rust workspace.
- `criterion` - Benchmarks in `crates/z00z_core`, `crates/z00z_crypto`, `crates/z00z_storage`, and `crates/z00z_wallets`.
- `proptest` - Property-based tests in the Rust crates.

**Cryptography:**
- Vendored Tari keep-set - `tari_crypto`, `tari_bulletproofs_plus`, and `tari_utilities` are path dependencies exposed through `crates/z00z_crypto`.
- `argon2`, `hkdf`, `hmac`, `sha2`, `blake2`, `blake3`, `chacha20poly1305`, `bip39`, `bip32`, `zeroize`, and the `p3-*` field and Poseidon2 crates - The main cryptographic stack used by `crates/z00z_crypto` and `crates/z00z_wallets`.

**Web / Frontend:**
- Next.js 15.5.4 - Website runtime and app framework in `website/website_2025-09-30/package.json`.
- React 19 - UI runtime for the website.
- `next-intl` - Locale and translation support in the website app.
- `next-auth` 5 beta - Authentication layer in the website app.
- Tailwind CSS 4 via `@tailwindcss/postcss` and `@tailwindcss/typography` - Styling pipeline for the website.
- `framer-motion`, `@monaco-editor/react`, `@tiptap/react`, `mermaid`, `markdown-it`, `reactflow`, `react-apexcharts`, and `react-simple-maps` - Major UI and content-rendering dependencies in the website subtree.

**Testing / Tooling:**
- `wasm-bindgen`, `wasm-bindgen-futures`, `web-sys`, `js-sys`, and `wasm-bindgen-test` - Browser-side and WASM test/runtime plumbing in the wallet and network crates.
- `rust_xlsxwriter` - Spreadsheet export in `crates/z00z_simulator`.
- `serde_json`, `serde_yaml`, and `serde_yml` - Structured data handling across the Rust workspace and the website content pipeline.

## Key Dependencies

**Critical:**
- `z00z_crypto` - Cryptographic facade and Tari vendor isolation layer consumed by `z00z_core`, `z00z_storage`, `z00z_wallets`, and `z00z_simulator`.
- `z00z_core` - Core protocol and asset model consumed by storage, simulator, and wallet crates.
- `z00z_storage` - Canonical asset storage, checkpoint, and snapshot crate.
- `z00z_wallets` - Wallet domain, persistence, and RPC adapter crate.
- `z00z_networks_rpc` - Shared RPC transport crate for native and WASM wallet flows.
- `z00z_utils` - Required abstraction layer for I/O, codecs, time, config, RNG, and metrics.

**Infrastructure:**
- `tari_crypto`, `tari_bulletproofs_plus`, and `tari_utilities` - Path dependencies that anchor the crypto backend in `crates/z00z_crypto/tari/`.
- `redb` - Native wallet persistence in `crates/z00z_wallets/src/db/redb_wallet_store.rs` and embedded storage in `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`.
- `jmt` - Merkle tree storage engine for canonical asset state in `crates/z00z_storage/src/assets/store.rs`.
- `rexie` - IndexedDB persistence for the browser wallet backend in `crates/z00z_wallets/src/wasm/indexeddb_backend.rs`.
- `jsonrpsee` - RPC method generation and client/server plumbing in `crates/z00z_wallets/src/adapters/rpc/`.
- `prometheus` - Optional metrics backend behind the `prometheus` feature in `crates/z00z_utils/Cargo.toml`.
- `next-auth` and `next-intl` - Website auth and localization layers in `website/website_2025-09-30/`.
- `@kazumatu981/markdown-it-kroki` - Markdown diagram rendering in `website/website_2025-09-30/src/app/(protected-pages)/(docs)/[...slug]/page.tsx`.

## Configuration

**Environment:**
- Workspace feature gates are significant: `test-fast`, `wallet_debug_dump`, `wallet_debug_tools`, `egui`, `wasm`, `os_hardening`, `claim-auth-sign`, `qr-codes`, and the `ownership_policy_*` family in `crates/z00z_wallets/Cargo.toml`.
- `z00z_core` uses feature-gated JSON, binary, and deterministic-rng support from `Cargo.toml`.
- Website runtime configuration is centralized in `website/website_2025-09-30/src/configs/app.config.ts`, `website/website_2025-09-30/next.config.mjs`, `website/website_2025-09-30/tsconfig.json`, `website/website_2025-09-30/postcss.config.mjs`, and `website/website_2025-09-30/tailwind.config.ts`.
- Browser automation configuration lives in `webdriver.json` and `crates/z00z_wallets/webdriver.json`.
- Build metadata is tracked in `versions.yaml` and consumed by `scripts/cargo_build.py`.

**Build:**
- `scripts/cargo_build.py` and `scripts/cargo_build.sh` provide repository-local crate build orchestration.
- `website/website_2025-09-30/Dockerfile` and `website/website_2025-09-30/docker-compose.yml` define the active website container flow.
- The root `docker/Dockerfile` still targets a legacy `zuz-node` layout and does not match the current workspace tree.
- `website/website_2025-09-30/next.config.mjs` traces `content/**/*` and `public/configs/**/*` into the server bundle.
- `website/website_2025-09-30/tailwind.config.ts` uses class-based dark mode and a CSS-variable driven theme.

## Platform Requirements

**Development:**
- Rust toolchain compatible with the workspace MSRV of 1.90.0.
- Python 3 with PyYAML available for `scripts/cargo_build.py`.
- Bash-compatible shell for repository scripts.
- Node.js 18 or newer for the active website container build.
- Firefox headless for WebDriver-driven browser tests.
- Docker for the website and Kroki local compose flow.

**Production / Execution Targets:**
- Native Rust binaries and libraries for core, crypto, storage, simulator, and wallet functionality.
- Browser/WASM execution for wallet persistence and RPC flows.
- Next.js website deployment for the documentation and UI subtree.
- Local filesystem persistence for genesis artifacts, checkpoints, snapshots, `.wlt` wallet files, and simulator outputs.

## Workspace Notes

- `crates/z00z_extensions/Cargo.toml` and `crates/z00z_telemetry/Cargo.toml` currently declare no external dependencies.
- `config/z00z_blockchain_config.yaml` exists but contains only whitespace.
- `website/website_2025-09-30/package-lock.json` is present, so the website dependency tree is locked.
- `Cargo.lock` is present at the workspace root, so Rust dependency resolution is pinned.

---

*Stack analysis: 2026-05-06*
