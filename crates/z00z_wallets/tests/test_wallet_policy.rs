#![cfg(not(target_arch = "wasm32"))]

use z00z_core::assets::AssetPkgWire;
use z00z_core::Asset;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::time::MockTimeProvider;
use z00z_wallets::{
    persistence::{TxRecord, TxStatus, TxStorage, TxStorageImpl},
    tx::{TxAuthWire, TxContextWire, TxOutRole, TxOutputWire, TxPackage, TxProofWire, TxWire},
    wallet::{Policy, PolicyError, PolicyImpl, PolicyRules, PolicySpendContext},
};

fn policy_asset(serial_id: u32, amount: u64) -> Asset {
    z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", serial_id, amount)
        .expect("valid std asset")
}

fn tx_bytes(asset: &Asset) -> Vec<u8> {
    let package = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: 3,
        chain_type: "devnet".to_string(),
        chain_name: "devnet".to_string(),
        tx: TxWire {
            tx_type: "regular_tx".to_string(),
            inputs: Vec::new(),
            outputs: vec![TxOutputWire {
                role: TxOutRole::Recipient,
                asset_wire: AssetPkgWire::from_asset(asset),
            }],
            fee: 0,
            nonce: 1,
            context: TxContextWire::default(),
            proof: TxProofWire::default(),
            auth: TxAuthWire::default(),
        },
        tx_digest_hex: hex::encode([7u8; 32]),
        status: "pending".to_string(),
    };

    JsonCodec.serialize(&package).expect("serialize tx package")
}

fn tx_record(tx_hash: &str, asset: &Asset) -> TxRecord {
    TxRecord {
        tx_hash: tx_hash.to_string(),
        tx_bytes: tx_bytes(asset),
        imported: false,
        status: TxStatus::Pending,
        timestamp_ms: 1_000,
        block_height: None,
        confirmation_evidence: None,
    }
}

fn context_for_day(
    store: &TxStorageImpl<MockTimeProvider>,
    asset_id: [u8; 32],
    day_start_ms: u64,
    day_end_ms: u64,
) -> PolicySpendContext {
    store
        .policy_spend_window(asset_id, day_start_ms, day_end_ms)
        .expect("policy spend window")
        .into()
}

#[test]
fn test_daily_limit_blocks_spend() {
    let dir = tempfile::tempdir().unwrap();
    let time = MockTimeProvider::from_unix_secs(10);
    let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
    let mut store = TxStorageImpl::new(&history_path, time.clone());

    let spent_asset = policy_asset(1, 6);
    store.put(tx_record("tx1", &spent_asset)).unwrap();

    let rules = PolicyRules {
        max_tx_amount: None,
        max_daily_amount: Some(10),
        allowed_assets: None,
        allowed_recipients: None,
        require_confirmation: false,
        time_restrictions: None,
    };
    let policy = PolicyImpl::new(rules, time);
    let asset = policy_asset(2, 5);
    let context = context_for_day(&store, spent_asset.definition.id, 0, 86_400_000);

    let err = policy.validate_spend_with_context(&asset, 5, "alice", &context);
    assert_eq!(
        err,
        Err(PolicyError::DailyLimitExceeded {
            spent: 6,
            amount: 5,
            limit: 10,
        })
    );
}

#[test]
fn test_confirmation_blocks_until_confirmed() {
    let dir = tempfile::tempdir().unwrap();
    let time = MockTimeProvider::from_unix_secs(20);
    let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
    let mut store = TxStorageImpl::new(&history_path, time.clone());

    let spent_asset = policy_asset(3, 2);
    store.put(tx_record("tx1", &spent_asset)).unwrap();

    let rules = PolicyRules {
        max_tx_amount: None,
        max_daily_amount: None,
        allowed_assets: None,
        allowed_recipients: None,
        require_confirmation: true,
        time_restrictions: None,
    };
    let policy = PolicyImpl::new(rules, time.clone());
    let asset = policy_asset(4, 1);

    let pending_context = context_for_day(&store, spent_asset.definition.id, 0, 86_400_000);
    let err = policy.validate_spend_with_context(&asset, 1, "alice", &pending_context);
    assert_eq!(err, Err(PolicyError::ConfirmationRequired { pending: 1 }));

    store.record_confirmed("tx1", 9).unwrap();
    let confirmed_context = context_for_day(&store, spent_asset.definition.id, 0, 86_400_000);
    policy
        .validate_spend_with_context(&asset, 1, "alice", &confirmed_context)
        .unwrap();
}

#[test]
fn test_restart_restores_send_window() {
    let dir = tempfile::tempdir().unwrap();
    let time = MockTimeProvider::from_unix_secs(30);
    let history_path = dir.path().join("wallet_abc_tx_history.jsonl");
    let mut store = TxStorageImpl::new(&history_path, time.clone());

    let first_asset = policy_asset(5, 3);
    let second_asset = policy_asset(6, 4);
    store.put(tx_record("tx1", &first_asset)).unwrap();
    store.put(tx_record("tx2", &second_asset)).unwrap();

    let reopened = TxStorageImpl::new(&history_path, time.clone());
    let context = context_for_day(&reopened, first_asset.definition.id, 0, 86_400_000);

    assert_eq!(context.spent_today, 7);
    assert_eq!(context.pending_confirmation_count, 2);

    let rules = PolicyRules {
        max_tx_amount: None,
        max_daily_amount: Some(10),
        allowed_assets: None,
        allowed_recipients: None,
        require_confirmation: false,
        time_restrictions: None,
    };
    let policy = PolicyImpl::new(rules, time);
    let asset = policy_asset(7, 4);
    let err = policy.validate_spend_with_context(&asset, 4, "alice", &context);

    assert_eq!(
        err,
        Err(PolicyError::DailyLimitExceeded {
            spent: 7,
            amount: 4,
            limit: 10,
        })
    );
}
