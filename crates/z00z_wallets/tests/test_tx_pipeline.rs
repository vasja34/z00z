#![allow(deprecated)]

#[path = "test_inc/test_range_proof_env.inc"]
mod test_common;

use std::sync::Arc;

use test_common::RangeProofEnvGuard;
use z00z_core::{
    assets::{AssetClass, AssetDefinition, AssetPkgWire, AssetWire},
    Asset, Commitment,
};
use z00z_crypto::expert::{
    encoding::ByteArray,
    keys::{RistrettoPublicKey, RistrettoSecretKey},
    traits::PublicKeyTrait,
};
use z00z_crypto::{
    create_commitment, hash_to_scalar_domain, Hidden, Z00ZRistrettoPoint, Z00ZScalar,
};
use z00z_utils::{rng::SystemRngProvider, time::SystemTimeProvider};
use z00z_wallets::tx::fee_estimator::calculate_fee_for_wires;
use z00z_wallets::{
    receiver::ReceiverCard,
    stealth::{build_card_stealth_leaf, build_stealth_leaf},
    tx::{
        balance_blindings, build_tx_package_digest, format_payref_short, generate_mac_key,
        generate_payref, verify_blind_balance, verify_payref, AssetSelector, AssetSelectorImpl,
        FeeEstimator, FeeEstimatorImpl, Prover, ProverImpl, SelectionStrategy, Signer, SignerImpl,
        TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire, TxPackage, TxProofWire,
        TxVerifier, TxVerifierImpl, TxWire, Z00ZTxId,
    },
};

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";
fn scalar(seed: u64) -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Z00ZScalar::try_from_bytes(bytes).expect("valid scalar")
}

fn make_asset(id_byte: u8, amount: u64) -> Asset {
    let def = AssetDefinition {
        id: [id_byte; 32],
        class: AssetClass::Coin,
        name: "Test Asset".to_string(),
        symbol: "TST".to_string(),
        decimals: 0,
        serials: 1,
        nominal: 1,
        domain_name: "test.local".to_string(),
        version: 1,
        crypto_version: 1,
        policy_flags: 0,
        metadata: None,
    };

    Asset {
        definition: Arc::new(def),
        serial_id: 0,
        amount,
        commitment: Commitment::default(),
        range_proof: None,
        nonce: [0u8; 32],
        lock_height: None,
        owner_pub: None,
        owner_signature: None,
        is_frozen: false,
        is_slashed: false,
        is_burned: false,
        r_pub: None,
        owner_tag: None,
        enc_pack: None,
        secret: None,
        tag16: None,
        leaf_ad_id: None,
    }
}

fn package_json() -> Vec<u8> {
    let asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 1_000_000)
        .expect("asset");
    let fee_seed = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 9, 1)
        .expect("fee seed");
    let wire = AssetWire::from_asset(&asset);
    let fee_seed = AssetWire::from_asset(&fee_seed);
    let fee = calculate_fee_for_wires(1, &[wire.clone(), fee_seed]).expect("fee");
    let fee_asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 9, fee)
        .expect("fee asset");
    let fee_wire = AssetWire::from_asset(&fee_asset);
    let tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![TxInputWire {
            asset_id_hex: hex::encode([1u8; 32]),
            serial_id: 1,
        }],
        outputs: vec![
            TxOutputWire {
                role: TxOutRole::Recipient,
                asset_wire: AssetPkgWire::from_wire(&wire),
            },
            TxOutputWire {
                role: TxOutRole::Fee,
                asset_wire: AssetPkgWire::from_wire(&fee_wire),
            },
        ],
        fee,
        nonce: 0,
        context: TxContextWire::default(),
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
    };
    let payload = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: CHAIN_ID,
        chain_type: CHAIN_TYPE.to_string(),
        chain_name: CHAIN_NAME.to_string(),
        tx: tx.clone(),
        tx_digest_hex: build_tx_package_digest(
            "TxPackage",
            "regular_tx",
            1,
            CHAIN_ID,
            CHAIN_TYPE,
            CHAIN_NAME,
            &tx,
        )
        .expect("digest"),
        status: "prepared".to_string(),
    };
    serde_json::to_vec(&payload).expect("serialize")
}

#[test]
fn test_builder_live() {
    let _guard = RangeProofEnvGuard::new();
    let k_dh = [7u8; 32];
    let r_pub = [8u8; 32];
    let owner = [9u8; 32];
    let s_out = [10u8; 32];

    let leaf = build_stealth_leaf(&k_dh, &r_pub, &owner, 777, 42, s_out).expect("leaf");
    assert_ne!(leaf.c_amount, [0u8; 32]);
}

#[test]
fn test_balance_live() {
    let input = scalar(100);
    let out = scalar(17);
    let fee = scalar(9);
    let sum = &out + &fee;
    let in_two = &sum - &input;

    assert!(verify_blind_balance(&[input, in_two], &[out, fee]));

    let change = balance_blindings(&scalar(50), &[scalar(20)]);
    assert!((&scalar(20) + &change).ct_eq(&scalar(50)));
}

#[test]
fn test_prover_live() {
    let prover = ProverImpl::new().expect("prover");
    let blind = Hidden::hide(scalar(44));
    let proof = prover.create_proof(123, &blind).expect("proof");
    let commit = create_commitment(123, blind.reveal()).expect("commit");
    assert!(prover
        .verify_proof(&proof, commit.as_bytes())
        .expect("verify"));
}

#[test]
fn test_signer_live() {
    let signer = SignerImpl::new(SystemRngProvider);
    let sk = Hidden::hide(RistrettoSecretKey::from(55u64));
    let msg = b"phase14 signer";
    let sig = signer.sign_message(msg, &sk).expect("sig");

    let pk = RistrettoPublicKey::from_secret_key(sk.reveal());
    assert!(signer.verify(msg, &sig, pk.as_bytes()).expect("verify"));
}

#[test]
fn test_selector_live() {
    let selector = AssetSelectorImpl::new(SystemRngProvider);
    let assets = vec![make_asset(1, 40), make_asset(2, 30), make_asset(3, 50)];

    let out = selector
        .select(&assets, 60, 5, SelectionStrategy::MinInputs)
        .expect("select");
    assert!(out.total_amount >= 65);
}

#[test]
fn test_ecdh_out_live() {
    let _guard = RangeProofEnvGuard::new();
    let recv_secret = [0x22u8; 32];
    let view_sk = hash_to_scalar_domain(b"z00z.consensus.view_key.v1", &[&recv_secret]);
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);
    let owner =
        z00z_crypto::hash::poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&recv_secret]);
    let card = ReceiverCard {
        version: 1,
        owner_handle: owner,
        view_pk: view_pk.to_bytes(),
        identity_pk: [5u8; 32],
        card_id: None,
        metadata: None,
        signature: [0u8; 64],
    };

    let leaf = build_card_stealth_leaf(&card, 1000, 1).expect("leaf");
    assert_ne!(leaf.asset_id, [0u8; 32]);
}

#[test]
fn test_fee_est_live() {
    let est = FeeEstimatorImpl::new(SystemTimeProvider, 10, 2);
    let fee = est.estimate_by_size(100).expect("fee");
    assert!(fee.medium >= 10);
}

#[test]
fn test_verifier_live() {
    let verifier = TxVerifierImpl::new();
    let payload = package_json();
    let result = verifier.verify(&payload).expect("verify");
    assert!(result.valid);
}

#[test]
fn test_id_payref_live() {
    let block = [1u8; 32];
    let out = [2u8; 32];

    let payref = generate_payref(&block, &out);
    assert!(verify_payref(&payref, &block, &out));
    assert!(!format_payref_short(&payref).is_empty());

    let key = generate_mac_key().expect("key");
    let tx_id = Z00ZTxId::derive(&key, &out).expect("tx id");
    assert!(tx_id.verify(&key, &out));
}
