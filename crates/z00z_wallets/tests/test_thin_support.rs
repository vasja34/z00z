#![allow(dead_code)]

use std::sync::Arc;

use jsonrpsee::types::ErrorObjectOwned;
use tempfile::TempDir;
use tokio::sync::OnceCell;
use z00z_core::{assets::AssetClass, genesis::asset_std::asset_from_dev_class};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    time::{MockTimeProvider, TimeProvider},
};
use z00z_wallets::{
    chain::ReceiverCardRecord,
    rpc::{
        methods::{
            AppRpcImpl, AppRpcServer, TxRpcImpl, TxRpcServer, WalletRpcImpl, WalletRpcServer,
        },
        types::{common::PersistWalletId, wallet::SessionToken},
    },
    stealth::{build_seeded_output_bundle, SenderWallet},
    tx::{verify_full_tx_package, ThinIndexEntry, ThinSnapshotContext, TxOutRole, TxPackage},
    WalletService,
};

static FIXTURE_ENTRY: OnceCell<ThinIndexEntry> = OnceCell::const_new();

pub async fn fixture_entry() -> ThinIndexEntry {
    FIXTURE_ENTRY.get_or_init(build_fixture_entry).await.clone()
}

pub fn context_for_entry(
    entry: &ThinIndexEntry,
    compatibility_generation: u64,
    issued_at_ms: u64,
    expires_at_ms: u64,
) -> ThinSnapshotContext {
    ThinSnapshotContext {
        chain_id: entry.chain_id.clone(),
        compatibility_generation,
        prev_root_hex: entry.prev_root_hex.clone(),
        checkpoint_id_hex: None,
        issued_at_ms,
        expires_at_ms,
    }
}

pub fn tx_json(entry: &ThinIndexEntry) -> String {
    String::from_utf8(entry.tx_bytes.clone()).expect("utf8 tx json")
}

pub fn expected_package(entry: &ThinIndexEntry) -> TxPackage {
    JsonCodec
        .deserialize(&entry.tx_bytes)
        .expect("decode expected package")
}

pub fn assert_runtime_tx_error_codes(error: &ErrorObjectOwned, expected_codes: &[&str]) {
    let data = error.data().expect("typed tx error payload");
    let payload: serde_json::Value =
        serde_json::from_str(data.get()).expect("typed tx error payload json");
    let error_codes = payload
        .get("error_codes")
        .and_then(|value| value.as_array())
        .expect("error_codes array");
    let actual = error_codes
        .iter()
        .map(|value| value.as_str().expect("error code string"))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected_codes);
}

pub struct ThinRpcEnv {
    _temp: TempDir,
    pub time: Arc<MockTimeProvider>,
    pub service: Arc<WalletService>,
    pub session: SessionToken,
    pub wallet_id: PersistWalletId,
    pub rpc: TxRpcImpl,
}

impl ThinRpcEnv {
    pub async fn new(name: &str, unix_secs: u64) -> Self {
        let time = Arc::new(MockTimeProvider::from_unix_secs(unix_secs));
        let time_provider: Arc<dyn TimeProvider> = time.clone();
        let temp = tempfile::tempdir().expect("tempdir");
        let output_dir = temp.path().join("wallets");
        let service = Arc::new(WalletService::with_output_dir_and_time(
            output_dir,
            Arc::clone(&time_provider),
        ));
        let app_service = Arc::new(z00z_wallets::services::AppService::with_wallet_service(
            Arc::clone(&service),
        ));
        let app_rpc = AppRpcImpl::new(app_service);
        let wallet_rpc = WalletRpcImpl::new(Arc::clone(&service));
        let create = app_rpc
            .create_wallet(name.to_string(), "StrongPassw0rd!".to_string(), None)
            .await
            .expect("create wallet");
        let wallet_id = create.wallet_id.clone();
        let session = wallet_rpc
            .unlock_wallet(wallet_id.clone(), "StrongPassw0rd!".to_string())
            .await
            .expect("unlock wallet");
        let rpc = TxRpcImpl::with_dependencies(Arc::clone(&service), time_provider);

        Self {
            _temp: temp,
            time,
            service,
            session,
            wallet_id,
            rpc,
        }
    }
}

async fn build_fixture_entry() -> ThinIndexEntry {
    ThinIndexEntry::from_tx_bytes(build_fixture_tx_bytes().await).expect("fixture entry")
}

async fn build_fixture_tx_bytes() -> Vec<u8> {
    let env = ThinRpcEnv::new("thin-fixture", 100).await;
    seed_spendable_coin(&env.service, &env.wallet_id, 50_000, 7).await;
    let recipient = compact_receiver(&env.service, &env.wallet_id).await;
    let built = env
        .rpc
        .build_transaction(env.session.clone(), recipient, 123, None)
        .await
        .expect("build fixture transaction");
    let tx_bytes = built.raw_tx.as_bytes().to_vec();
    let verify = verify_full_tx_package(&tx_bytes).expect("verify fixture transaction");
    assert!(
        verify.valid,
        "fixture verification errors: {:?}",
        verify.errors
    );
    tx_bytes
}

async fn compact_receiver(service: &WalletService, wallet_id: &PersistWalletId) -> String {
    let recv_keys = service
        .receiver_keys(wallet_id)
        .await
        .expect("receiver keys");
    let card = recv_keys.export_receiver_card().expect("receiver card");
    ReceiverCardRecord::new(&card, card.canonical_encoding(), 0)
        .expect("receiver record")
        .to_compact()
        .expect("receiver compact")
}

async fn seed_spendable_coin(
    service: &WalletService,
    wallet_id: &PersistWalletId,
    amount: u64,
    serial_id: u32,
) {
    for offset in 0..3u32 {
        let current_serial = serial_id.saturating_add(offset);
        let mut asset = asset_from_dev_class(AssetClass::Coin, current_serial, amount)
            .expect("valid std asset");
        let recv_keys = service
            .receiver_keys(wallet_id)
            .await
            .expect("receiver keys");
        let card = recv_keys.export_receiver_card().expect("receiver card");
        let tx_seed = [41u8; 32];
        let tx_digest = [current_serial as u8; 32];
        let mut sender_wallet = SenderWallet::new(tx_seed);
        let output = build_seeded_output_bundle(
            wallet_id.0.clone(),
            TxOutRole::Recipient,
            AssetClass::Coin,
            &card,
            None,
            &mut sender_wallet,
            &tx_digest,
            asset.serial_id,
            [current_serial as u8; 32],
            asset.amount,
            &asset.definition.id,
            asset.serial_id,
        )
        .expect("stealth output");

        asset.commitment = z00z_crypto::Commitment::from_bytes(&output.leaf.c_amount)
            .expect("commitment")
            .0;
        asset.owner_pub = None;
        asset.owner_signature = None;
        asset.r_pub = Some(output.leaf.r_pub);
        asset.owner_tag = Some(output.leaf.owner_tag);
        asset.enc_pack = Some(output.leaf.enc_pack);
        asset.tag16 = Some(output.leaf.tag16);
        asset.leaf_ad_id = Some(output.leaf.asset_id);
        asset.range_proof = Some(output.leaf.range_proof);

        service
            .put_claimed_asset(wallet_id, asset)
            .await
            .expect("seed put_claimed_asset must succeed");
    }
}
