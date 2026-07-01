#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use tempfile::TempDir;
use z00z_core::assets::AssetClass;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::MockTimeProvider;
use z00z_wallets::backup::{decode_tx_history_rows, WalletTxHistoryEntryKind};
use z00z_wallets::rpc::methods::{AssetRpcImpl, AssetRpcServer};
use z00z_wallets::rpc::types::asset::RuntimeAssetListFilter;
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::rpc::types::wallet::SessionToken;
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::tx::TxPackage;

const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const OPS_SRC: &str = include_str!("../src/rpc/asset_rpc_server_ops.rs");
const CATALOG_SRC: &str = include_str!("../src/rpc/asset_rpc_server_catalog.rs");
const STATE_SRC: &str = include_str!("../src/rpc/asset_rpc_support_state.rs");
const KEY_RPC_SRC: &str = include_str!("../src/rpc/key_rpc.rs");
const KEY_TYPES_SRC: &str = include_str!("../src/rpc/key_types.rs");
const KEY_SUPPORT_SRC: &str = include_str!("../src/rpc/key_rpc_support.rs");

fn next_wallet_name() -> String {
    static TEST_WALLET_SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = TEST_WALLET_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("asset-rpc-mutation-{seq}")
}

fn mk_rpc_with_disk() -> (
    AssetRpcImpl,
    Arc<WalletService>,
    TempDir,
    Arc<MockTimeProvider>,
) {
    let dir = tempfile::tempdir().expect("tempdir");
    let time = Arc::new(MockTimeProvider::default());
    let service = Arc::new(WalletService::create_service_custom_output_directory(
        dir.path().to_path_buf(),
        time.clone(),
        SystemRngProvider,
    ));
    let rpc =
        AssetRpcImpl::with_dependencies_and_wallet_service(time.clone(), Arc::clone(&service));
    (rpc, service, dir, time)
}

async fn create_unlocked_wallet(
    service: Arc<WalletService>,
    time: Arc<MockTimeProvider>,
) -> (PersistWalletId, SessionToken) {
    let password_text = "Test_Pass_Z00Z_42!".to_string();
    let password = SafePassword::from(password_text.clone());
    let app = AppService::with_dependencies(time, Arc::clone(&service));
    let wallet_id = app
        .create_wallet(
            next_wallet_name(),
            password_text,
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .expect("create_wallet")
        .wallet_id;
    let session = service
        .unlock_wallet_in_memory(&wallet_id, &password)
        .await
        .expect("unlock_wallet_in_memory");
    (wallet_id, session)
}

async fn seed_assets(service: &WalletService, wallet_id: &PersistWalletId) {
    for asset in [
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("coin fixture"),
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Token, 2, 100)
            .expect("token fixture"),
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Nft, 3, 1)
            .expect("nft fixture"),
        z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 4, 15)
            .expect("coin fixture"),
    ] {
        service
            .put_claimed_asset(wallet_id, asset)
            .await
            .expect("put_claimed_asset");
    }
}

async fn asset_id_by_class(
    rpc: &AssetRpcImpl,
    wallet_id: &PersistWalletId,
    class: AssetClass,
) -> [u8; 32] {
    asset_ids_by_class(rpc, wallet_id, class)
        .await
        .into_iter()
        .next()
        .expect("asset for class")
}

async fn asset_ids_by_class(
    rpc: &AssetRpcImpl,
    wallet_id: &PersistWalletId,
    class: AssetClass,
) -> Vec<[u8; 32]> {
    let response = rpc
        .list_assets(
            wallet_id.clone(),
            Some(50),
            None,
            Some(RuntimeAssetListFilter {
                asset_class: Some(class),
                min_balance: None,
            }),
        )
        .await
        .expect("list_assets");
    response
        .items
        .into_iter()
        .map(|asset| asset.to_asset().expect("asset wire").asset_id())
        .collect()
}

fn history_path(output_dir: &Path) -> PathBuf {
    let mut matches = std::fs::read_dir(output_dir)
        .expect("read_dir")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("_tx_history.jsonl"))
        })
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "expected one tx-history file");
    matches.pop().expect("history path")
}

fn decode_packages(path: &Path) -> Vec<(WalletTxHistoryEntryKind, TxPackage)> {
    decode_tx_history_rows(&std::fs::read(path).expect("read history bytes"))
        .expect("decode tx history rows")
        .into_iter()
        .map(|row| {
            let package = JsonCodec
                .deserialize::<TxPackage>(&row.record.tx_bytes)
                .expect("history tx_bytes must decode as TxPackage");
            (row.entry_kind, package)
        })
        .collect()
}

#[test]
fn local_mutations_use_exec() {
    assert!(STATE_SRC.contains("struct LocalMutationExec"));
    assert_eq!(OPS_SRC.matches(".local_mutation_exec(").count(), 4);
    assert_eq!(CATALOG_SRC.matches(".local_mutation_exec(").count(), 1);
    assert!(STATE_SRC.contains("build_tx_package_digest("));
    assert!(!OPS_SRC.contains("submit_local_asset_mutation("));
    assert!(!CATALOG_SRC.contains("submit_local_asset_mutation("));
    assert!(!STATE_SRC.contains("mutation_counter"));
    assert!(!STATE_SRC.contains("next_local_mutation_nonce"));
    assert!(!STATE_SRC.contains("tx_digest_hex: hex::encode("));
}

#[test]
fn rotate_docs_match_contract() {
    assert!(KEY_RPC_SRC
        .contains("Rewrites persisted wallet encryption state under one rotation contract"));
    assert!(KEY_TYPES_SRC.contains("persisted records rewrapped"));
    assert!(KEY_SUPPORT_SRC.contains("rotate_master_key_persisted("));
}

#[tokio::test]
async fn merge_split_swap_use_live_tx_packages() {
    let (rpc, service, dir, time) = mk_rpc_with_disk();
    let (wallet_id, session) =
        create_unlocked_wallet(Arc::clone(&service), Arc::clone(&time)).await;
    seed_assets(&service, &wallet_id).await;

    let coin_ids = asset_ids_by_class(&rpc, &wallet_id, AssetClass::Coin).await;
    let token_id = asset_id_by_class(&rpc, &wallet_id, AssetClass::Token).await;
    assert!(coin_ids.len() >= 2, "expected two coin fixtures");
    let coin_id = coin_ids[0];

    let merge = rpc
        .merge_assets(session.clone(), coin_ids[..2].to_vec())
        .await
        .expect("merge_assets");
    assert!(merge
        .tx_id
        .as_ref()
        .is_some_and(|tx_id| tx_id.0.starts_with("tx_") && !tx_id.0.contains("stub_tx_")));

    let split = rpc
        .split_asset(session.clone(), token_id, vec![40, 60])
        .await
        .expect("split_asset");
    assert!(split
        .tx_id
        .as_ref()
        .is_some_and(|tx_id| tx_id.0.starts_with("tx_") && !tx_id.0.contains("stub_tx_")));

    let swap = rpc
        .swap_assets(session, token_id, coin_id, 1)
        .await
        .expect("swap_assets");
    assert!(swap.tx_id.0.starts_with("tx_"));
    assert!(!swap.tx_id.0.contains("stub_tx_"));

    let packages = decode_packages(&history_path(dir.path()));
    assert_eq!(
        packages.len(),
        6,
        "three mutations must append created+submitted rows"
    );
    assert_eq!(
        packages.iter().map(|(kind, _)| *kind).collect::<Vec<_>>(),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
        ]
    );

    let created = packages
        .into_iter()
        .filter_map(|(kind, package)| {
            (kind == WalletTxHistoryEntryKind::Created).then_some(package)
        })
        .collect::<Vec<_>>();
    assert_eq!(created.len(), 3);

    let merge_pkg = &created[0];
    assert_eq!(merge_pkg.kind, "TxPackage");
    assert_eq!(merge_pkg.tx.inputs.len(), 2);
    assert_eq!(merge_pkg.tx.outputs.len(), 1);

    let split_pkg = &created[1];
    assert_eq!(split_pkg.tx.inputs.len(), 1);
    assert_eq!(split_pkg.tx.outputs.len(), 2);

    let swap_pkg = &created[2];
    assert_eq!(swap_pkg.tx.inputs.len(), 1);
    assert_eq!(swap_pkg.tx.outputs.len(), 1);
    assert_eq!(swap_pkg.tx.inputs[0].asset_id_hex, hex::encode(token_id));

    let merge_output = merge_pkg.tx.outputs[0]
        .asset_wire
        .clone()
        .to_asset()
        .expect("merge output asset");
    assert_eq!(merge.asset.asset_id, merge_output.asset_id());
    assert_eq!(merge.asset.serial_id, merge_output.serial_id);

    let split_outputs = split_pkg
        .tx
        .outputs
        .iter()
        .map(|output| {
            output
                .asset_wire
                .clone()
                .to_asset()
                .expect("split output asset")
        })
        .collect::<Vec<_>>();
    assert!(split_outputs
        .iter()
        .all(|asset| asset.serial_id == split.splits[0].asset.serial_id));
    assert_eq!(
        split
            .splits
            .iter()
            .map(|item| item.asset.asset_id)
            .collect::<Vec<_>>(),
        split_outputs
            .iter()
            .map(|asset| asset.asset_id())
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn merge_keeps_tx_id() {
    let (rpc, service, dir, time) = mk_rpc_with_disk();
    let (wallet_id, session) =
        create_unlocked_wallet(Arc::clone(&service), Arc::clone(&time)).await;
    seed_assets(&service, &wallet_id).await;

    let coin_ids = asset_ids_by_class(&rpc, &wallet_id, AssetClass::Coin).await;
    assert!(coin_ids.len() >= 2, "expected two coin fixtures");

    let first = rpc
        .merge_assets(session.clone(), coin_ids[..2].to_vec())
        .await
        .expect("first merge");
    let second = rpc
        .merge_assets(session, coin_ids[..2].to_vec())
        .await
        .expect("second merge");

    assert_eq!(first.tx_id, second.tx_id);
    assert_eq!(
        decode_packages(&history_path(dir.path()))
            .into_iter()
            .map(|(kind, _)| kind)
            .collect::<Vec<_>>(),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
        ]
    );
}

#[tokio::test]
async fn stake_unstake_append_live_tx_history() {
    let (rpc, service, dir, time) = mk_rpc_with_disk();
    let (wallet_id, session) =
        create_unlocked_wallet(Arc::clone(&service), Arc::clone(&time)).await;
    seed_assets(&service, &wallet_id).await;

    let token_id = asset_id_by_class(&rpc, &wallet_id, AssetClass::Token).await;

    let stake = rpc
        .stake_assets(session.clone(), token_id, 10)
        .await
        .expect("stake_assets");
    assert!(stake.stake_id.starts_with("stake_"));
    assert_eq!(stake.asset.asset_id, token_id);
    assert_eq!(stake.amount, 10);

    let unstake = rpc
        .unstake_assets(session, stake.stake_id.clone())
        .await
        .expect("unstake_assets");
    assert_eq!(unstake.stake_id, stake.stake_id);
    assert_eq!(unstake.asset.asset_id, token_id);
    assert_eq!(unstake.amount, 10);

    let packages = decode_packages(&history_path(dir.path()));
    assert_eq!(
        packages.len(),
        4,
        "stake+unstake must append created+submitted rows"
    );
    let created = packages
        .into_iter()
        .filter_map(|(kind, package)| {
            (kind == WalletTxHistoryEntryKind::Created).then_some(package)
        })
        .collect::<Vec<_>>();
    assert_eq!(created.len(), 2);
    assert!(created.iter().all(|package| package.kind == "TxPackage"));
    assert!(created.iter().all(|package| package.tx.inputs.len() == 1));
    assert!(created
        .iter()
        .all(|package| package.tx.inputs[0].asset_id_hex == hex::encode(token_id)));
    assert!(created.iter().all(|package| package.tx.outputs.len() == 1));
    assert!(created.iter().all(|package| {
        package.tx.outputs[0]
            .asset_wire
            .clone()
            .to_asset()
            .is_ok_and(|asset| asset.serial_id == stake.asset.serial_id)
    }));
}
