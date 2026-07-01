//! Dispatcher wiring for app-owned RPC methods.
//!
//! This includes `app.*` lifecycle methods, plus app-level `chain.*`,
//! `network.*`, and wallet-local scan operations (even though their method
//! prefixes are not `app.*`).

#![cfg(not(target_arch = "wasm32"))]

use crate::rpc::dispatcher_handlers::{
    json_handler, json_typed_handler_jsonrpsee_err, map_error_object_owned, serialize_result,
    typed_handler_jsonrpsee_err, typed_handler_ok, WalletIdParams, WalletIdPasswordParams,
};
use crate::rpc::methods::chain_rpc::ChainScanRpc as _;
use crate::rpc::methods::{
    AppRpcImpl, AppRpcServer, ChainRpcImpl, ChainRpcServer, ChainScanRpcImpl, NetworkRpcImpl,
    NetworkRpcServer,
};
use crate::rpc::types::chain::RuntimeStartScanParams;
use crate::rpc::types::wallet::WalletSource;
use serde::Deserialize;
use std::sync::Arc;
use z00z_networks_rpc::RpcDispatcher;
use z00z_utils::codec::Value;

#[derive(Debug, Deserialize)]
struct WalletCreateParams {
    name: String,
    password: String,
    seed_phrase: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WalletImportParams {
    data: String,
    password: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct WalletRecoverFromSeedParams {
    name: String,
    password: String,
    mnemonic_a: String,
    mnemonic_b: String,
    network: String,
    chain: String,
}

#[derive(Debug, Deserialize)]
struct NetworkTorParams {
    enable: bool,
}

/// Registers the app-owned dispatcher-wired RPC methods into a generic [`RpcDispatcher`].
pub fn register_app_methods(dispatcher: &RpcDispatcher, rpc: Arc<AppRpcImpl>) {
    dispatcher.register_method(
        "app.wallet.list_wallets",
        json_handler(Arc::clone(&rpc), |rpc, _params| async move {
            let result = rpc.list_wallets().await.map_err(map_error_object_owned)?;
            serialize_result(result)
        }),
    );

    dispatcher.register_method(
        "app.wallet.create_wallet",
        json_typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: WalletCreateParams| async move {
                rpc.create_wallet(p.name, p.password, p.seed_phrase).await
            },
        ),
    );

    dispatcher.register_method(
        "app.wallet.delete_wallet",
        json_typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: WalletIdPasswordParams| async move {
                rpc.delete_wallet(p.wallet_id, p.password).await
            },
        ),
    );

    dispatcher.register_method(
        "app.wallet.export_wallet",
        json_typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: WalletIdPasswordParams| async move {
                rpc.export_wallet(p.wallet_id, p.password).await
            },
        ),
    );

    dispatcher.register_method(
        "app.wallet.import_wallet",
        json_typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: WalletImportParams| async move {
                rpc.import_wallet(p.data, p.password, p.name).await
            },
        ),
    );

    dispatcher.register_method(
        "app.wallet.open_wallet_source",
        json_typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, source: WalletSource| async move { rpc.open_wallet_source(source).await },
        ),
    );

    dispatcher.register_method(
        "app.wallet.recover_from_seed",
        json_typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: WalletRecoverFromSeedParams| async move {
                rpc.recover_from_seed(
                    p.name,
                    p.password,
                    p.mnemonic_a,
                    p.mnemonic_b,
                    p.network,
                    p.chain,
                )
                .await
            },
        ),
    );
}

/// Register wallet-local chain-scan RPC methods.
pub fn register_scan_methods(dispatcher: &RpcDispatcher, rpc: Arc<ChainScanRpcImpl>) {
    dispatcher.register_typed(
        "app.chain.start_local_scan",
        typed_handler_ok(
            Arc::clone(&rpc),
            |rpc, p: RuntimeStartScanParams| async move { rpc.start_local_scan(p).await },
        ),
    );

    dispatcher.register_typed(
        "app.chain.stop_local_scan",
        typed_handler_ok(Arc::clone(&rpc), |rpc, p: WalletIdParams| async move {
            rpc.stop_local_scan(p.wallet_id).await;
            Value::Null
        }),
    );

    dispatcher.register_typed(
        "app.chain.get_local_scan_status",
        typed_handler_ok(Arc::clone(&rpc), |rpc, p: WalletIdParams| async move {
            rpc.get_local_scan_status(p.wallet_id).await
        }),
    );

    dispatcher.register_typed(
        "app.chain.get_local_scan_tip",
        typed_handler_ok(Arc::clone(&rpc), |rpc, _p: Value| async move {
            rpc.get_local_scan_tip().await
        }),
    );
}

pub fn register_network_methods(dispatcher: &RpcDispatcher, rpc: Arc<NetworkRpcImpl>) {
    dispatcher.register_typed(
        "app.network.switch_to_onionet",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, _p: Value| async move {
            rpc.switch_to_onionet().await
        }),
    );

    dispatcher.register_typed(
        "app.network.switch_to_tor",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: NetworkTorParams| async move {
            rpc.switch_to_tor(p.enable).await
        }),
    );
}

pub fn register_chain_methods(dispatcher: &RpcDispatcher, rpc: Arc<ChainRpcImpl>) {
    dispatcher.register_typed(
        "app.chain.switch_to_mainnet",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, _p: Value| async move {
            rpc.switch_to_mainnet().await
        }),
    );

    dispatcher.register_typed(
        "app.chain.switch_to_testnet",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, _p: Value| async move {
            rpc.switch_to_testnet().await
        }),
    );

    dispatcher.register_typed(
        "app.chain.switch_to_devnet",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, _p: Value| async move {
            rpc.switch_to_devnet().await
        }),
    );
}

/// Registers all app-owned dispatcher-wired RPC methods into a generic [`RpcDispatcher`].
///
/// This includes:
/// - `app.wallet.*`
/// - `app.chain.*`
/// - `app.network.*`
/// - `app.chain.*` wallet-local scan methods
pub fn register_all_app_rpc_methods(
    dispatcher: &RpcDispatcher,
    app_rpc: Arc<AppRpcImpl>,
    chain_rpc: Arc<ChainRpcImpl>,
    network_rpc: Arc<NetworkRpcImpl>,
    scan_rpc: Arc<ChainScanRpcImpl>,
) {
    register_app_methods(dispatcher, app_rpc);
    register_chain_methods(dispatcher, chain_rpc);
    register_network_methods(dispatcher, network_rpc);
    register_scan_methods(dispatcher, scan_rpc);
}
