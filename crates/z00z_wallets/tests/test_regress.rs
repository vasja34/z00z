#![allow(deprecated)]

#[path = "test_inc/test_range_proof_env.inc"]
mod test_common;

use test_common::RangeProofEnvGuard;
use z00z_crypto::{hash_to_scalar_domain, Z00ZRistrettoPoint};
use z00z_wallets::{receiver::ReceiverCard, stealth::build_card_stealth_leaf};

const REG_15_1: &str = "REG-15-1";
const REG_15_2: &str = "REG-15-2";
const REG_15_3: &str = "REG-15-3";

fn make_card() -> ReceiverCard {
    let recv_secret = [0x22u8; 32];
    let view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&recv_secret]);
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);
    let owner =
        z00z_crypto::hash::poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&recv_secret]);
    ReceiverCard {
        version: 1,
        owner_handle: owner,
        view_pk: view_pk.to_bytes(),
        identity_pk: [4u8; 32],
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    }
}

#[test]
fn test_no_trunc() {
    let src = include_str!("../src/tx/test_output_builder.rs");
    assert!(
        src.contains("debug_assert_eq!(bytes.len(), 32"),
        "{REG_15_1}"
    );
    assert!(src.contains(".try_into()"), "{REG_15_1}");
    assert!(!src.contains("min(bytes.len())"), "{REG_15_1}");
}

#[test]
fn test_name_proof() {
    let src = include_str!("../src/tx/prover.rs");
    assert!(src.contains("fn verify_proof("), "{REG_15_2}");
    assert!(src.contains("proof: &RangeProof"), "{REG_15_2}");
    assert!(src.contains("commitment: &[u8]"), "{REG_15_2}");
    assert!(
        !src.contains("fn verify_proof(&self, _proof:"),
        "{REG_15_2}"
    );
}

#[test]
fn test_ser_pass() {
    let _guard = RangeProofEnvGuard::new();
    let card = make_card();
    let leaf = build_card_stealth_leaf(&card, 1000, 7).expect("leaf");
    assert_eq!(leaf.serial_id, 7, "{REG_15_3}");
    assert_ne!(leaf.serial_id, 0, "{REG_15_3}");
}

#[test]
fn test_ser_diff() {
    let _guard = RangeProofEnvGuard::new();
    let card = make_card();
    let first = build_card_stealth_leaf(&card, 1000, 1).expect("first");
    let second = build_card_stealth_leaf(&card, 1000, 2).expect("second");
    assert_ne!(first.serial_id, second.serial_id, "{REG_15_3}");
}
