use std::sync::Arc;

use z00z_networks_rpc::{LocalRpcTransport, RpcDispatcher, RpcError, RpcTransport};
use z00z_utils::codec::json;
use z00z_wallets::{
    rpc::{
        methods::{
            AppRpcImpl, AssetRpcImpl, BackupRpcImpl, ChainRpcImpl, ChainScanRpcImpl, KeyRpcImpl,
            NetworkRpcImpl, StorageRpcImpl, TxRpcImpl, WalletRpcImpl,
        },
        register_all_wallet_rpc_methods,
        types::{common::PersistWalletId, wallet::SessionToken},
    },
    services::{AppService, WalletService},
};

pub struct TestEnv {
    pub transport: LocalRpcTransport,
    pub wallet_service: Arc<WalletService>,
}

fn reg_rpc(
    app_service: Arc<AppService>,
    wallet_service: Arc<WalletService>,
    asset_rpc: Arc<AssetRpcImpl>,
) -> Arc<RpcDispatcher> {
    let dispatcher = Arc::new(RpcDispatcher::new());
    register_all_wallet_rpc_methods(
        &dispatcher,
        Arc::new(AppRpcImpl::new(Arc::clone(&app_service))),
        Arc::new(WalletRpcImpl::new(Arc::clone(&wallet_service))),
        asset_rpc,
        Arc::new(TxRpcImpl::new(Arc::clone(&wallet_service))),
        Arc::new(BackupRpcImpl::new(Arc::clone(&wallet_service))),
        Arc::new(KeyRpcImpl::new(Arc::clone(&wallet_service))),
        Arc::new(ChainRpcImpl::new(Arc::clone(&app_service))),
        Arc::new(NetworkRpcImpl::with_app_service(Arc::clone(&app_service))),
        Arc::new(ChainScanRpcImpl::new(app_service)),
        Arc::new(StorageRpcImpl::new(wallet_service)),
    )
    .expect("wallet RPC registration should succeed");
    dispatcher
}

pub fn setup_env(output_dir: std::path::PathBuf) -> TestEnv {
    let wallet_service = Arc::new(WalletService::with_output_dir(output_dir));
    let app_service = Arc::new(AppService::with_wallet_service(Arc::clone(&wallet_service)));
    let asset_rpc = Arc::new(AssetRpcImpl::with_wallet_service(Arc::clone(
        &wallet_service,
    )));
    let dispatcher = reg_rpc(
        app_service,
        Arc::clone(&wallet_service),
        Arc::clone(&asset_rpc),
    );

    TestEnv {
        transport: LocalRpcTransport::new(dispatcher),
        wallet_service,
    }
}

pub async fn unlock(env: &TestEnv, wallet_id: &PersistWalletId, password: &str) -> SessionToken {
    let value = env
        .transport
        .call(
            "wallet.session.unlock_wallet",
            json!({"wallet_id": wallet_id, "password": password}),
        )
        .await
        .expect("unlock_wallet must succeed");

    serde_json::from_value(value).expect("SessionToken must deserialize")
}

fn assert_dto_json(asset_data: &str) {
    let value: serde_json::Value = serde_json::from_str(asset_data).expect("dto json");
    assert!(value.get("secret").is_none());
    assert!(value.get("is_frozen").is_none());
    assert!(value.get("is_slashed").is_none());
    assert!(z00z_core::assets::decode_asset_pkg_json(asset_data.as_bytes()).is_ok());
}

fn as_json(wire: &z00z_core::AssetWire) -> String {
    let dto = z00z_core::assets::AssetPkgWire::from_wire(wire);
    let bytes = z00z_core::assets::encode_asset_pkg_json(&dto).expect("dto json bytes");
    let text = String::from_utf8(bytes).expect("dto utf8");
    assert_dto_json(&text);
    text
}

pub async fn import_wire(
    env: &TestEnv,
    session: &SessionToken,
    wire: &z00z_core::AssetWire,
) -> Result<serde_json::Value, RpcError> {
    env.transport
        .call(
            "wallet.asset.import_asset",
            json!({"session": session, "asset_data": as_json(wire)}),
        )
        .await
}
