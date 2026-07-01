# Unnamed CodeViz Diagram

```mermaid
graph TD

    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"]
    base.cv::z00z_crypto["**Z00Z Crypto**<br>/home/vadim/Projects/z00z/crates/z00z_crypto"]
    base.cv::z00z_extensions["**Z00Z Extensions**<br>/home/vadim/Projects/z00z/crates/z00z_extensions"]
    base.cv::z00z_networks["**Z00Z Networks**<br>/home/vadim/Projects/z00z/crates/z00z_networks"]
    base.cv::z00z_rollup_node["**Z00Z Rollup Node**<br>/home/vadim/Projects/z00z/crates/z00z_rollup_node"]
    base.cv::z00z_runtime["**Z00Z Runtime**<br>/home/vadim/Projects/z00z/crates/z00z_runtime"]
    base.cv::z00z_simulator["**Z00Z Simulator**<br>/home/vadim/Projects/z00z/crates/z00z_simulator"]
    base.cv::z00z_storage["**Z00Z Storage**<br>/home/vadim/Projects/z00z/crates/z00z_storage"]
    base.cv::z00z_telemetry["**Z00Z Telemetry**<br>/home/vadim/Projects/z00z/crates/z00z_telemetry"]
    base.cv::z00z_utils["**Z00Z Utils**<br>/home/vadim/Projects/z00z/crates/z00z_utils"]
    base.cv::anyhow["**Anyhow**<br>/home/vadim/Projects/z00z/Cargo.toml `anyhow = { version = "1.0" }`"]
    base.cv::serde_json["**Serde JSON**<br>/home/vadim/Projects/z00z/Cargo.toml `serde_json = "1"`"]
    base.cv::serde["**Serde**<br>/home/vadim/Projects/z00z/Cargo.toml `serde = "1"`"]
    base.cv::rand["**Rand**<br>/home/vadim/Projects/z00z/Cargo.toml `rand = { version = "0.8" }`"]
    base.cv::thiserror["**ThisError**<br>/home/vadim/Projects/z00z/Cargo.toml `thiserror = { version = "2" }`"]
    base.cv::borsh["**Borsh**<br>/home/vadim/Projects/z00z/Cargo.toml `borsh = { version = "1.5.7" }`"]
    base.cv::tokio["**Tokio**<br>/home/vadim/Projects/z00z/Cargo.toml `tokio = { version = "1.47" }`"]
    base.cv::log["**Log**<br>/home/vadim/Projects/z00z/Cargo.toml `log = { version = "0.4" }`"]
    base.cv::log4rs["**Log4rs**<br>/home/vadim/Projects/z00z/Cargo.toml `log4rs = { version = "1.4" }`"]
    base.cv::user["**Z00Z User**<br>[External]"]
    subgraph base.cv::z00z_wallets["**Z00Z Wallets**<br>/home/vadim/Projects/z00z/crates/z00z_wallets"]
        base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"]
        base.cv::z00z_wallet_egui["**Z00Z Wallet GUI**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/bin/z00z_wallet_egui.rs `fn main()`, /home/vadim/Projects/z00z/crates/z00z_wallets/scripts/z00z_wallet_egui.sh `cargo run`"]
        base.cv::z00z_wallet_wasm["**Z00Z Wallet WASM**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/wasm ``, /home/vadim/Projects/z00z/crates/z00z_wallets/scripts/z00z_wallet_wasm.sh `wasm-pack build`"]
        base.cv::z00z_wallet_cli_tools["**Z00Z Wallet CLI Tools**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/bin/gen_password_bloom.rs `fn main()`, /home/vadim/Projects/z00z/crates/z00z_wallets/bin/z00z-wallet-addr-convert.rs `fn main()`, /home/vadim/Projects/z00z/crates/z00z_wallets/bin/z00z-wallet-validate.rs `fn main()`"]
        base.cv::z00z_wallet_rpc["**Z00Z Wallet RPC Server**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/docs/rpc-user-guide.md ``, /home/vadim/Projects/z00z/crates/z00z_wallets/examples/example_2_rpc_complete.rs ``"]
        base.cv::z00z_wallets_db["**Z00Z Wallet Local Storage**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/db ``, /home/vadim/Projects/z00z/crates/z00z_wallets/docs/db-compression.md ``"]
        %% Edges at this level (grouped by source)
        base.cv::z00z_wallet_egui["**Z00Z Wallet GUI**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/bin/z00z_wallet_egui.rs `fn main()`, /home/vadim/Projects/z00z/crates/z00z_wallets/scripts/z00z_wallet_egui.sh `cargo run`"] -->|"Uses"| base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"]
        base.cv::z00z_wallet_wasm["**Z00Z Wallet WASM**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/wasm ``, /home/vadim/Projects/z00z/crates/z00z_wallets/scripts/z00z_wallet_wasm.sh `wasm-pack build`"] -->|"Uses"| base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"]
        base.cv::z00z_wallet_cli_tools["**Z00Z Wallet CLI Tools**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/bin/gen_password_bloom.rs `fn main()`, /home/vadim/Projects/z00z/crates/z00z_wallets/bin/z00z-wallet-addr-convert.rs `fn main()`, /home/vadim/Projects/z00z/crates/z00z_wallets/bin/z00z-wallet-validate.rs `fn main()`"] -->|"Uses"| base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"]
        base.cv::z00z_wallet_rpc["**Z00Z Wallet RPC Server**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/docs/rpc-user-guide.md ``, /home/vadim/Projects/z00z/crates/z00z_wallets/examples/example_2_rpc_complete.rs ``"] -->|"Uses"| base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"]
        base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"] -->|"Reads from and writes to"| base.cv::z00z_wallets_db["**Z00Z Wallet Local Storage**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/db ``, /home/vadim/Projects/z00z/crates/z00z_wallets/docs/db-compression.md ``"]
    end
    %% Edges at this level (grouped by source)
    base.cv::user["**Z00Z User**<br>[External]"] -->|"Manages funds using GUI"| base.cv::z00z_wallet_egui["**Z00Z Wallet GUI**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/bin/z00z_wallet_egui.rs `fn main()`, /home/vadim/Projects/z00z/crates/z00z_wallets/scripts/z00z_wallet_egui.sh `cargo run`"]
    base.cv::user["**Z00Z User**<br>[External]"] -->|"Manages funds using CLI"| base.cv::z00z_wallet_cli_tools["**Z00Z Wallet CLI Tools**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/bin/gen_password_bloom.rs `fn main()`, /home/vadim/Projects/z00z/crates/z00z_wallets/bin/z00z-wallet-addr-convert.rs `fn main()`, /home/vadim/Projects/z00z/crates/z00z_wallets/bin/z00z-wallet-validate.rs `fn main()`"]
    base.cv::user["**Z00Z User**<br>[External]"] -->|"Manages funds using Web UI (via browser)"| base.cv::z00z_wallet_wasm["**Z00Z Wallet WASM**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/wasm ``, /home/vadim/Projects/z00z/crates/z00z_wallets/scripts/z00z_wallet_wasm.sh `wasm-pack build`"]
    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"] -->|"Uses for error handling"| base.cv::anyhow["**Anyhow**<br>/home/vadim/Projects/z00z/Cargo.toml `anyhow = { version = "1.0" }`"]
    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"] -->|"Uses for JSON operations"| base.cv::serde_json["**Serde JSON**<br>/home/vadim/Projects/z00z/Cargo.toml `serde_json = "1"`"]
    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"] -->|"Uses for serialization"| base.cv::serde["**Serde**<br>/home/vadim/Projects/z00z/Cargo.toml `serde = "1"`"]
    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"] -->|"Uses for randomness"| base.cv::rand["**Rand**<br>/home/vadim/Projects/z00z/Cargo.toml `rand = { version = "0.8" }`"]
    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"] -->|"Uses for error definitions"| base.cv::thiserror["**ThisError**<br>/home/vadim/Projects/z00z/Cargo.toml `thiserror = { version = "2" }`"]
    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"] -->|"Uses for binary serialization"| base.cv::borsh["**Borsh**<br>/home/vadim/Projects/z00z/Cargo.toml `borsh = { version = "1.5.7" }`"]
    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"] -->|"Uses for async runtime"| base.cv::tokio["**Tokio**<br>/home/vadim/Projects/z00z/Cargo.toml `tokio = { version = "1.47" }`"]
    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"] -->|"Uses for logging"| base.cv::log["**Log**<br>/home/vadim/Projects/z00z/Cargo.toml `log = { version = "0.4" }`"]
    base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"] -->|"Uses for logging configuration"| base.cv::log4rs["**Log4rs**<br>/home/vadim/Projects/z00z/Cargo.toml `log4rs = { version = "1.4" }`"]
    base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"] -->|"Uses for cryptographic operations"| base.cv::z00z_crypto["**Z00Z Crypto**<br>/home/vadim/Projects/z00z/crates/z00z_crypto"]
    base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"] -->|"Uses for error handling"| base.cv::anyhow["**Anyhow**<br>/home/vadim/Projects/z00z/Cargo.toml `anyhow = { version = "1.0" }`"]
    base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"] -->|"Uses for serialization"| base.cv::serde["**Serde**<br>/home/vadim/Projects/z00z/Cargo.toml `serde = "1"`"]
    base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"] -->|"Uses for async operations"| base.cv::tokio["**Tokio**<br>/home/vadim/Projects/z00z/Cargo.toml `tokio = { version = "1.47" }`"]
    base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"] -->|"Uses for logging"| base.cv::log["**Log**<br>/home/vadim/Projects/z00z/Cargo.toml `log = { version = "0.4" }`"]
    base.cv::z00z_wallets_core_logic["**Z00Z Wallet Core Logic**<br>/home/vadim/Projects/z00z/crates/z00z_wallets/src/core ``, /home/vadim/Projects/z00z/crates/z00z_wallets/src/services ``"] -->|"Submits transactions to"| base.cv::z00z_rollup_node["**Z00Z Rollup Node**<br>/home/vadim/Projects/z00z/crates/z00z_rollup_node"]
    base.cv::z00z_rollup_node["**Z00Z Rollup Node**<br>/home/vadim/Projects/z00z/crates/z00z_rollup_node"] -->|"Relies on"| base.cv::z00z_core["**Z00Z Core**<br>/home/vadim/Projects/z00z/crates/z00z_core"]

```
---
*Generated by [CodeViz.ai](https://codeviz.ai) on 05/02/2026, 15:41:36*
