#![allow(dead_code)]

use sha2::{Digest, Sha256};
use z00z_aggregators::{
    BatchId, BatchPlanned, BatchPlanner, IngressBoundary, RejectClass, RejectRecord,
    RouteRangeRule, ShardId, ShardRouteTable, WorkItem, WorkPayload,
};
use z00z_storage::checkpoint::CheckpointDraftId;
use z00z_wallets::tx::{
    build_claim_tx_digest, build_tx_package_digest, ClaimAuthWire, ClaimContextWire,
    ClaimProofWire, ClaimTxPackage, ClaimTxWire, TxAuthWire, TxContextWire, TxPackage, TxProofWire,
    TxWire,
};

pub const HASH_MIN: [u8; 32] = [0u8; 32];
pub const HASH_MAX: [u8; 32] = [0xffu8; 32];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannerOut {
    Accept(BatchPlanned),
    Reject { class: RejectClass, detail: String },
}

pub fn batch_id(label: &str) -> BatchId {
    let digest: [u8; 32] = Sha256::digest(label.as_bytes()).into();
    BatchId::new(CheckpointDraftId::new(digest))
}

pub fn bridge_table() -> ShardRouteTable {
    ShardRouteTable {
        routing_generation: 0,
        shard_set: vec![ShardId::new(0)],
        rules: vec![RouteRangeRule::new(HASH_MIN, HASH_MAX, ShardId::new(0))],
        previous_generation_digest: None,
        activation_checkpoint: 11,
    }
}

pub fn split_table() -> ShardRouteTable {
    let bridge = bridge_table();
    let split_a = [0x55; 32];
    let split_b = [0xaa; 32];

    ShardRouteTable {
        routing_generation: 1,
        shard_set: vec![ShardId::new(0), ShardId::new(1), ShardId::new(2)],
        rules: vec![
            RouteRangeRule::new(HASH_MIN, split_a, ShardId::new(0)),
            RouteRangeRule::new(
                next_hash(split_a).expect("split a + 1"),
                split_b,
                ShardId::new(1),
            ),
            RouteRangeRule::new(
                next_hash(split_b).expect("split b + 1"),
                HASH_MAX,
                ShardId::new(2),
            ),
        ],
        previous_generation_digest: Some(bridge.digest()),
        activation_checkpoint: 42,
    }
}

pub fn span_table(items: &[WorkItem], target_shard: ShardId) -> ShardRouteTable {
    assert!(!items.is_empty(), "span_table needs at least one item");

    let mut keys = items.iter().map(route_key).collect::<Vec<_>>();
    keys.sort();
    let min_key = keys[0];
    let max_key = *keys.last().expect("max key");

    let mut shard_set = Vec::new();
    let mut rules = Vec::new();

    if min_key != HASH_MIN {
        shard_set.push(ShardId::new(0));
        rules.push(RouteRangeRule::new(
            HASH_MIN,
            prev_hash(min_key).expect("min key > zero"),
            ShardId::new(0),
        ));
    }

    shard_set.push(target_shard);
    rules.push(RouteRangeRule::new(min_key, max_key, target_shard));

    if max_key != HASH_MAX {
        shard_set.push(ShardId::new(2));
        rules.push(RouteRangeRule::new(
            next_hash(max_key).expect("max key < ff"),
            HASH_MAX,
            ShardId::new(2),
        ));
    }

    let bridge = bridge_table();
    ShardRouteTable {
        routing_generation: 1,
        shard_set,
        rules,
        previous_generation_digest: Some(bridge.digest()),
        activation_checkpoint: 64,
    }
}

pub fn split_items(left: &WorkItem, right: &WorkItem) -> ShardRouteTable {
    let left_key = route_key(left);
    let right_key = route_key(right);
    let low_key = left_key.min(right_key);

    let bridge = bridge_table();
    ShardRouteTable {
        routing_generation: 1,
        shard_set: vec![ShardId::new(0), ShardId::new(1)],
        rules: vec![
            RouteRangeRule::new(HASH_MIN, low_key, ShardId::new(0)),
            RouteRangeRule::new(
                next_hash(low_key).expect("split + 1"),
                HASH_MAX,
                ShardId::new(1),
            ),
        ],
        previous_generation_digest: Some(bridge.digest()),
        activation_checkpoint: 71,
    }
}

pub fn verdict(result: Result<BatchPlanned, RejectRecord>) -> PlannerOut {
    match result {
        Ok(planned) => PlannerOut::Accept(planned),
        Err(err) => PlannerOut::Reject {
            class: err.class,
            detail: err.detail,
        },
    }
}

pub fn tx_item(seed: &str) -> WorkItem {
    let mut pkg = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: 3,
        chain_type: "devnet".to_string(),
        chain_name: format!("z00z-{seed}"),
        tx: TxWire {
            tx_type: "regular_tx".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            fee: 0,
            nonce: 0,
            context: TxContextWire::default(),
            proof: TxProofWire::default(),
            auth: TxAuthWire::default(),
        },
        tx_digest_hex: String::new(),
        status: "received".to_string(),
    };
    pkg.tx_digest_hex = build_tx_package_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .expect("tx digest");
    IngressBoundary
        .normalize(WorkPayload::Tx(Box::new(pkg)))
        .expect("normalized tx")
}

pub fn claim_item(seed: &str) -> WorkItem {
    let mut pkg = ClaimTxPackage {
        kind: "ClaimTxPackage".to_string(),
        package_type: "claim_tx".to_string(),
        version: 1,
        chain_id: 3,
        chain_type: "devnet".to_string(),
        chain_name: format!("z00z-{seed}"),
        tx: ClaimTxWire {
            tx_type: "claim_tx".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            fee: 0,
            nonce: 0,
            context: ClaimContextWire {
                recipient_wallet_id: "wallet".to_string(),
                recipient_owner_hex: "00".repeat(32),
                claim_scope_hash_hex: "11".repeat(32),
                recipient_card_hex: None,
                nullifier_hex: "22".repeat(32),
            },
            proof: ClaimProofWire {
                proof_type: "genesis_claim".to_string(),
                proof_hex: "33".repeat(32),
            },
            auth: ClaimAuthWire {
                claim_authority_sig_hex: "44".repeat(64),
            },
        },
        tx_digest_hex: String::new(),
        status: "received".to_string(),
    };
    pkg.tx_digest_hex = build_claim_tx_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .expect("claim digest");
    IngressBoundary
        .normalize(WorkPayload::Claim(Box::new(pkg)))
        .expect("normalized claim")
}

pub fn planner_copies(table: &ShardRouteTable, count: usize) -> Vec<BatchPlanner> {
    (0..count)
        .map(|_| BatchPlanner::new(table.clone()))
        .collect()
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(nibble(byte >> 4));
        out.push(nibble(byte & 0x0f));
    }
    out
}

pub fn hex_decode(raw: &str) -> Vec<u8> {
    assert_eq!(raw.len() % 2, 0, "hex must have even length");
    raw.as_bytes()
        .chunks_exact(2)
        .map(|pair| (decode_nibble(pair[0]) << 4) | decode_nibble(pair[1]))
        .collect()
}

pub fn next_hash(mut hash: [u8; 32]) -> Option<[u8; 32]> {
    for index in (0..hash.len()).rev() {
        if hash[index] != u8::MAX {
            hash[index] += 1;
            for tail in &mut hash[index + 1..] {
                *tail = 0;
            }
            return Some(hash);
        }
    }
    None
}

pub fn prev_hash(mut hash: [u8; 32]) -> Option<[u8; 32]> {
    for index in (0..hash.len()).rev() {
        if hash[index] != 0 {
            hash[index] -= 1;
            for tail in &mut hash[index + 1..] {
                *tail = u8::MAX;
            }
            return Some(hash);
        }
    }
    None
}

fn route_key(item: &WorkItem) -> [u8; 32] {
    let bytes = hex_decode(item.digest_hex());
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

fn nibble(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => panic!("nibble out of range"),
    }
}

fn decode_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => panic!("invalid hex digit"),
    }
}
