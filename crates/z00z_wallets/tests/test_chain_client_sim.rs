use std::time::Duration;

use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::time::MockTimeProvider;
use z00z_wallets::chain::{
    ChainClient, ChainClientError, ChainClientImpl, ChainNetworkInfo, ChainTxStatus, LocalNodeSim,
};
use z00z_wallets::tx::fee_estimator::FeeRateSource;
use z00z_wallets::tx::{
    build_tx_package_digest, FeeEstimator, FeeEstimatorImpl, FeeEstimatorResult, TxPackage, TxWire,
};

#[derive(Debug, Clone)]
struct NodeFeeRateSource {
    node: LocalNodeSim,
}

impl FeeRateSource for NodeFeeRateSource {
    fn get_fee_per_weight(&self) -> FeeEstimatorResult<u64> {
        self.node
            .get_fee_per_weight()
            .map_err(|err| z00z_wallets::tx::FeeEstimatorError::Network(err.to_string()))
    }
}

#[test]
fn sim_round_trip() {
    let node = LocalNodeSim::new(
        ChainNetworkInfo {
            chain_id: "devnet".to_string(),
            version: "sim-v1".to_string(),
            peer_count: 3,
        },
        5,
    );
    node.insert_block(12, b"block-12".to_vec(), b"header-12".to_vec());

    let client = ChainClientImpl::with_local_sim(node.clone());
    assert_eq!(client.get_tip_height().expect("tip"), 12);
    assert_eq!(client.get_block(12).expect("block"), b"block-12".to_vec());
    assert_eq!(
        client.get_header(12).expect("header"),
        b"header-12".to_vec()
    );

    let info = client.get_network_info().expect("network info");
    assert_eq!(info.chain_id, "devnet");
    assert_eq!(info.version, "sim-v1");
    assert_eq!(info.peer_count, 3);

    let tx_hash = client
        .submit_transaction(b"portable tx bytes")
        .expect("submit");
    assert_eq!(
        client.get_transaction_status(&tx_hash).expect("pending"),
        ChainTxStatus::Pending
    );

    node.confirm_transaction(&tx_hash, 13).expect("confirm");
    assert_eq!(client.get_tip_height().expect("tip"), 13);
    assert_eq!(
        client.get_transaction_status(&tx_hash).expect("confirmed"),
        ChainTxStatus::Confirmed
    );
}

#[test]
fn sim_dup_submit() {
    let node = LocalNodeSim::default();
    let client = ChainClientImpl::with_local_sim(node);

    let first = client.submit_transaction(b"same tx").expect("first");
    let second = client.submit_transaction(b"same tx").expect("second");

    assert_eq!(first, second);
    assert_eq!(
        client.get_transaction_status(&first).expect("status"),
        ChainTxStatus::Pending
    );
}

#[test]
fn sim_uses_tx_digest_for_tx_packages() {
    let node = LocalNodeSim::default();
    let client = ChainClientImpl::with_local_sim(node);
    let tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        fee: 0,
        nonce: 7,
        context: Default::default(),
        proof: Default::default(),
        auth: Default::default(),
    };
    let tx_digest_hex =
        build_tx_package_digest("TxPackage", "regular_tx", 1, 1, "devnet", "devnet", &tx)
            .expect("digest");
    let package = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: 1,
        chain_type: "devnet".to_string(),
        chain_name: "devnet".to_string(),
        tx,
        tx_digest_hex: tx_digest_hex.clone(),
        status: "pending".to_string(),
    };
    let tx_bytes = JsonCodec.serialize(&package).expect("serialize");

    let tx_hash = client.submit_transaction(&tx_bytes).expect("submit");

    assert_eq!(tx_hash, tx_digest_hex);
    assert_eq!(
        client.get_transaction_status(&tx_hash).expect("status"),
        ChainTxStatus::Pending
    );
}

#[test]
fn sim_typed_errors() {
    let node = LocalNodeSim::default();
    let client = ChainClientImpl::with_local_sim(node.clone());

    assert!(matches!(
        client.get_block(99).unwrap_err(),
        ChainClientError::BlockNotFound(99)
    ));
    assert!(matches!(
        client.get_transaction_status("missing").unwrap_err(),
        ChainClientError::TxNotFound(tx_hash) if tx_hash == "missing"
    ));

    node.fail_next_network_info("simulated network outage");
    assert!(matches!(
        client.get_network_info().unwrap_err(),
        ChainClientError::Network(message) if message == "simulated network outage"
    ));
}

#[test]
fn sim_fee_refresh() {
    let time = MockTimeProvider::from_unix_secs(100);
    let node = LocalNodeSim::default();
    node.set_fee_per_weight(7);

    let source = NodeFeeRateSource { node: node.clone() };
    let mut estimator = FeeEstimatorImpl::with_network_rate(time.clone(), 1, 3, source, 5);

    estimator.update_rates().expect("initial refresh");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 7);

    node.set_fee_per_weight(9);
    time.advance_by(Duration::from_secs(6));
    estimator.update_rates().expect("refreshed rate");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 9);

    let fee = estimator.estimate_by_size(10).expect("fee");
    assert_eq!(fee.medium, 90);
}
