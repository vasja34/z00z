#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use z00z_core::{AssetClass, AssetWire};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_wallets::chain::{
    LocalNodeSim, RemoteScanEvidence, RemoteScanProofHint, RemoteScanRange, RemoteScanResumeHint,
    RemoteScanWorker, RemoteScanWorkerImpl,
};
use z00z_wallets::key::ReceiverKeys;
use z00z_wallets::receiver::{ReceiverCard, ScanChunk};
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::services::{AppService, WalletService};
use z00z_wallets::{bind_stealth_output_wire, build_output_bundle, WalletError};

const PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";
const SEED24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

async fn setup_wallet(name: &str) -> (Arc<WalletService>, tempfile::TempDir, PersistWalletId) {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let wallets = Arc::new(WalletService::with_output_dir(output_dir));
    let app = AppService::with_wallet_service(Arc::clone(&wallets));

    let created = app
        .create_wallet(
            name.to_string(),
            PASSWORD.to_string(),
            Some(SEED24.to_string()),
        )
        .await
        .expect("create wallet");
    wallets
        .unlock_wallet_in_memory(&created.wallet_id, &SafePassword::from(PASSWORD))
        .await
        .expect("unlock wallet");

    (wallets, temp, created.wallet_id)
}

fn make_recv_chunk(keys: &ReceiverKeys, height: u64, amount: u64, mark: u8) -> ScanChunk {
    let card = ReceiverCard {
        version: 1,
        owner_handle: keys.owner_handle,
        view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
        identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };
    let base_asset =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, amount).expect("asset");
    let output = build_output_bundle(
        format!("recv-{height}-{mark}"),
        z00z_wallets::tx::TxOutRole::Recipient,
        AssetClass::Coin,
        &card,
        amount,
        1,
    )
    .expect("output");
    let asset = bind_stealth_output_wire(AssetWire::from_asset(&base_asset), &output.leaf)
        .expect("bind output wire")
        .to_asset()
        .expect("scanned asset");

    ScanChunk {
        height,
        hash: vec![height as u8; 32],
        leaves: vec![asset],
    }
}

#[tokio::test]
async fn test_worker_authoritative_flow() {
    let (wallets, _temp, wallet_id) = setup_wallet("remote-worker-ok").await;
    let recv_keys = wallets
        .receiver_keys(&wallet_id)
        .await
        .expect("receiver keys");
    let range = RemoteScanRange {
        start_height: 7,
        end_height: 7,
    };
    let chunk = make_recv_chunk(&recv_keys, 7, 310, 17);

    let node = LocalNodeSim::default();
    node.set_remote_scan_evidence(
        range.clone(),
        RemoteScanEvidence {
            chunks: vec![chunk.clone()],
            proof_hints: vec![RemoteScanProofHint {
                checkpoint_height: 7,
                proof_bytes: vec![0xAB],
            }],
            resume_hint: None,
        },
    );

    let mut worker = RemoteScanWorkerImpl::with_local_sim(node);
    let out = wallets
        .recv_range_from_worker(&wallet_id, &mut worker, &range, &[], None)
        .await
        .expect("worker-assisted receive");

    assert_eq!(out.outputs.len(), 1);
    assert_eq!(out.stat.done_ckpt, 1);
    assert_eq!(out.stat.cursor.height(), 7);
    assert_eq!(
        wallets
            .list_claimed_assets(&wallet_id)
            .await
            .expect("claims")
            .len(),
        1
    );
    assert_eq!(worker.progress().fetched_ckpt, 1);
    assert!(!worker.is_fetching());
}

#[tokio::test]
async fn test_worker_transport_keeps_state() {
    let (wallets, _temp, wallet_id) = setup_wallet("remote-worker-transport").await;
    let node = LocalNodeSim::default();
    node.fail_next_remote_scan_transport("simulated network partition");

    let mut worker = RemoteScanWorkerImpl::with_local_sim(node);
    let range = RemoteScanRange {
        start_height: 7,
        end_height: 7,
    };
    let err = wallets
        .recv_range_from_worker(&wallet_id, &mut worker, &range, &[], None)
        .await
        .expect_err("transport failure must fail closed");

    assert!(matches!(err, WalletError::InvalidConfig(_)));
    assert!(err
        .to_string()
        .contains("remote scan worker transport error: simulated network partition"));
    assert!(wallets
        .list_claimed_assets(&wallet_id)
        .await
        .expect("claims")
        .is_empty());
}

#[tokio::test]
async fn test_worker_stale_hint_rejected() {
    let (wallets, _temp, wallet_id) = setup_wallet("remote-worker-stale").await;
    let recv_keys = wallets
        .receiver_keys(&wallet_id)
        .await
        .expect("receiver keys");

    let first_range = RemoteScanRange {
        start_height: 7,
        end_height: 7,
    };
    let first_chunk = make_recv_chunk(&recv_keys, 7, 310, 17);
    let node = LocalNodeSim::default();
    node.set_remote_scan_evidence(
        first_range.clone(),
        RemoteScanEvidence {
            chunks: vec![first_chunk],
            proof_hints: Vec::new(),
            resume_hint: None,
        },
    );

    let mut first_worker = RemoteScanWorkerImpl::with_local_sim(node.clone());
    wallets
        .recv_range_from_worker(&wallet_id, &mut first_worker, &first_range, &[], None)
        .await
        .expect("prime authoritative cursor");

    let stale_range = RemoteScanRange {
        start_height: 8,
        end_height: 8,
    };
    node.set_remote_scan_evidence(
        stale_range.clone(),
        RemoteScanEvidence {
            chunks: vec![make_recv_chunk(&recv_keys, 8, 420, 18)],
            proof_hints: Vec::new(),
            resume_hint: Some(RemoteScanResumeHint {
                next_height: 8,
                last_chunk_hash: vec![0xFF; 32],
            }),
        },
    );

    let mut stale_worker = RemoteScanWorkerImpl::with_local_sim(node);
    let err = wallets
        .recv_range_from_worker(&wallet_id, &mut stale_worker, &stale_range, &[], None)
        .await
        .expect_err("stale resume hint must fail closed");

    assert!(matches!(err, WalletError::InvalidConfig(_)));
    assert!(err
        .to_string()
        .contains("remote resume hint mismatches local cursor"));
    assert_eq!(
        wallets
            .list_claimed_assets(&wallet_id)
            .await
            .expect("claims")
            .len(),
        1
    );
}

#[tokio::test]
async fn test_worker_malicious_no_bypass() {
    let (wallets, _temp, wallet_id) = setup_wallet("remote-worker-malicious").await;
    let recv_keys = wallets
        .receiver_keys(&wallet_id)
        .await
        .expect("receiver keys");
    let range = RemoteScanRange {
        start_height: 7,
        end_height: 7,
    };

    let node = LocalNodeSim::default();
    node.set_remote_scan_evidence(
        range.clone(),
        RemoteScanEvidence {
            chunks: vec![make_recv_chunk(&recv_keys, 7, 310, 17)],
            proof_hints: vec![RemoteScanProofHint {
                checkpoint_height: 7,
                proof_bytes: Vec::new(),
            }],
            resume_hint: None,
        },
    );

    let mut worker = RemoteScanWorkerImpl::with_local_sim(node);
    let err = wallets
        .recv_range_from_worker(&wallet_id, &mut worker, &range, &[], None)
        .await
        .expect_err("malicious worker evidence must fail closed");

    assert!(matches!(err, WalletError::InvalidConfig(_)));
    assert!(err
        .to_string()
        .contains("remote proof hint bytes must not be empty"));
    assert!(wallets
        .list_claimed_assets(&wallet_id)
        .await
        .expect("claims")
        .is_empty());
}

#[tokio::test]
async fn test_worker_restart_reuses_node() {
    let node = LocalNodeSim::default();
    let range = RemoteScanRange {
        start_height: 11,
        end_height: 11,
    };
    node.set_remote_scan_evidence(
        range.clone(),
        RemoteScanEvidence {
            chunks: vec![ScanChunk {
                height: 11,
                hash: vec![11u8; 32],
                leaves: Vec::new(),
            }],
            proof_hints: Vec::new(),
            resume_hint: None,
        },
    );

    let mut first = RemoteScanWorkerImpl::with_local_sim(node.clone());
    let first_evidence = first
        .fetch_range_evidence(&range)
        .expect("first fetch survives");
    drop(first);

    let mut second = RemoteScanWorkerImpl::with_local_sim(node);
    let second_evidence = second
        .fetch_range_evidence(&range)
        .expect("second fetch survives");

    assert_eq!(first_evidence.chunks.len(), 1);
    assert_eq!(second_evidence.chunks.len(), 1);
    assert_eq!(second_evidence.chunks[0].height, 11);
}
