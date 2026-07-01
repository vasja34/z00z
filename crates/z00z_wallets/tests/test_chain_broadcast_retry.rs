#![cfg(not(target_arch = "wasm32"))]

use z00z_utils::io::read_file;
use z00z_utils::time::MockTimeProvider;
use z00z_wallets::backup::{decode_tx_history_rows, WalletTxHistoryEntryKind};
use z00z_wallets::chain::{
    Broadcast, BroadcastError, BroadcastImpl, ChainClientImpl, LocalNodeSim,
};
use z00z_wallets::persistence::tx::{TxStatus, TxStorage, TxStorageImpl};

fn history_kinds(path: &std::path::Path) -> Vec<WalletTxHistoryEntryKind> {
    decode_tx_history_rows(&read_file(path).expect("read tx history"))
        .expect("decode tx history rows")
        .into_iter()
        .map(|row| row.entry_kind)
        .collect()
}

#[test]
fn dup_store_idempotent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let history_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let time = MockTimeProvider::from_unix_secs(100);
    let node = LocalNodeSim::default();
    let client = ChainClientImpl::with_local_sim(node);
    let store = TxStorageImpl::new(&history_path, time.clone());
    let broadcast = BroadcastImpl::new(client, store, time.clone());

    let first = broadcast.broadcast(b"same-tx").expect("first broadcast");
    let second = broadcast
        .broadcast(b"same-tx")
        .expect("duplicate broadcast");

    assert_eq!(first.tx_hash, second.tx_hash);

    let store = TxStorageImpl::new(&history_path, time);
    let record = store
        .get(&format!("tx_{}", first.tx_hash))
        .expect("stored tx record");
    assert_eq!(record.status, TxStatus::Pending);
    assert_eq!(
        history_kinds(&history_path),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
        ]
    );
}

#[test]
fn retry_transient_confirms() {
    let temp = tempfile::tempdir().expect("temp dir");
    let history_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let time = MockTimeProvider::from_unix_secs(101);
    let node = LocalNodeSim::default();
    node.fail_next_submit_network("transient submit outage");
    let client = ChainClientImpl::with_local_sim(node.clone());
    let store = TxStorageImpl::new(&history_path, time.clone());
    let broadcast = BroadcastImpl::new(client, store, time.clone());

    let result = broadcast
        .broadcast_with_retry(b"retry-me", 3)
        .expect("retry broadcast");
    node.confirm_transaction(&result.tx_hash, 7)
        .expect("confirm tx");

    let confirmed_height = broadcast
        .wait_for_confirmation(&result.tx_hash, 1_000)
        .expect("confirmed height");
    assert_eq!(confirmed_height, 7);

    let store = TxStorageImpl::new(&history_path, time);
    let record = store
        .get(&format!("tx_{}", result.tx_hash))
        .expect("stored tx record");
    assert_eq!(record.status, TxStatus::Confirmed);
    assert_eq!(record.block_height, Some(7));
    assert_eq!(
        history_kinds(&history_path),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Confirmed,
        ]
    );
}

#[test]
fn confirm_reject_fails() {
    let temp = tempfile::tempdir().expect("temp dir");
    let history_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let time = MockTimeProvider::from_unix_secs(102);
    let node = LocalNodeSim::default();
    let client = ChainClientImpl::with_local_sim(node.clone());
    let store = TxStorageImpl::new(&history_path, time.clone());
    let broadcast = BroadcastImpl::new(client, store, time.clone());

    let result = broadcast.broadcast(b"reject-me").expect("broadcast");
    node.fail_transaction(&result.tx_hash).expect("fail tx");

    let err = broadcast
        .wait_for_confirmation(&result.tx_hash, 1_000)
        .expect_err("rejected tx must fail");
    assert!(matches!(err, BroadcastError::Rejected(_)));

    let store = TxStorageImpl::new(&history_path, time);
    let record = store
        .get(&format!("tx_{}", result.tx_hash))
        .expect("stored tx record");
    assert_eq!(record.status, TxStatus::Failed);
    assert_eq!(record.block_height, None);
    assert!(record.confirmation_evidence.is_none());
    assert_eq!(
        history_kinds(&history_path),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Failed,
        ]
    );
}

#[test]
fn confirm_timeout_pending() {
    let temp = tempfile::tempdir().expect("temp dir");
    let history_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let time = MockTimeProvider::from_unix_secs(103);
    let node = LocalNodeSim::default();
    let client = ChainClientImpl::with_local_sim(node);
    let store = TxStorageImpl::new(&history_path, time.clone());
    let broadcast = BroadcastImpl::with_poll_interval_ms(client, store, time.clone(), 100);

    let result = broadcast.broadcast(b"timeout-me").expect("broadcast");
    let err = broadcast
        .wait_for_confirmation(&result.tx_hash, 250)
        .expect_err("pending tx must time out");
    assert!(matches!(err, BroadcastError::Timeout));

    let store = TxStorageImpl::new(&history_path, time);
    let record = store
        .get(&format!("tx_{}", result.tx_hash))
        .expect("stored tx record");
    assert_eq!(record.status, TxStatus::Pending);
    assert_eq!(
        history_kinds(&history_path),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
        ]
    );
}

#[test]
fn confirm_rbf_fails() {
    let temp = tempfile::tempdir().expect("temp dir");
    let history_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let time = MockTimeProvider::from_unix_secs(104);
    let node = LocalNodeSim::default();
    let client = ChainClientImpl::with_local_sim(node.clone());
    let store = TxStorageImpl::new(&history_path, time.clone());
    let broadcast = BroadcastImpl::new(client, store, time.clone());

    let result = broadcast.broadcast(b"replace-me").expect("broadcast");
    node.replace_transaction(&result.tx_hash)
        .expect("replace tx");

    let err = broadcast
        .wait_for_confirmation(&result.tx_hash, 1_000)
        .expect_err("replaced tx must fail");
    assert!(matches!(err, BroadcastError::Replaced(_)));

    let store = TxStorageImpl::new(&history_path, time);
    let record = store
        .get(&format!("tx_{}", result.tx_hash))
        .expect("stored tx record");
    assert_eq!(record.status, TxStatus::Failed);
    assert_eq!(
        history_kinds(&history_path),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Failed,
        ]
    );
}

#[test]
fn confirm_reorg_fails() {
    let temp = tempfile::tempdir().expect("temp dir");
    let history_path = temp.path().join("wallet_abc_tx_history.jsonl");
    let time = MockTimeProvider::from_unix_secs(105);
    let node = LocalNodeSim::default();
    let client = ChainClientImpl::with_local_sim(node.clone());
    let store = TxStorageImpl::new(&history_path, time.clone());
    let broadcast = BroadcastImpl::new(client, store, time.clone());

    let result = broadcast.broadcast(b"reorg-me").expect("broadcast");
    node.reorg_transaction(&result.tx_hash).expect("reorg tx");

    let err = broadcast
        .wait_for_confirmation(&result.tx_hash, 1_000)
        .expect_err("reorged tx must fail");
    assert!(matches!(err, BroadcastError::Reorg(_)));

    let store = TxStorageImpl::new(&history_path, time);
    let record = store
        .get(&format!("tx_{}", result.tx_hash))
        .expect("stored tx record");
    assert_eq!(record.status, TxStatus::Failed);
    assert_eq!(
        history_kinds(&history_path),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Failed,
        ]
    );
}
