#[path = "test_inc/test_spend_public_support.inc"]
mod test_spend_public_support;

use z00z_core::assets::AssetLeaf;
use z00z_crypto::{
    create_commitment, domains::AssetIdDomain, hash_zk::hash_zk, Z00ZRistrettoPoint, Z00ZScalar,
};
use z00z_storage::settlement::CheckRoot;
use z00z_wallets::{
    key::{derive_owner_handle, derive_view_secret_key, ReceiverSecret},
    stealth::ecdh::compute_dh_sender,
    stealth::kdf::{compute_owner_tag, derive_k_dh},
    tx::{
        build_public_spend_contract, build_spend_assets, verify_tx_public_spend_contract,
        SpendBuildErr, SpendInputLeaf, SpendInputRef, SpendPlan, SpendProofApi, SpendProofErr,
        SpendPublicErr, SpendWitness, TxAuthWire, TxProofWire,
    },
};

#[derive(Default)]
struct SpyCs {
    bind_root_calls: usize,
    prove_input_calls: usize,
    check_balance_calls: usize,
}

impl SpendProofApi for SpyCs {
    fn bind_root(&mut self, _prev_root: CheckRoot) -> Result<(), SpendProofErr> {
        self.bind_root_calls += 1;
        Ok(())
    }

    fn prove_input(
        &mut self,
        _idx: usize,
        _inp: &SpendInputRef,
        _leaf: &SpendInputLeaf,
        _s_in: [u8; 32],
        _recv_sec: [u8; 32],
    ) -> Result<(), SpendProofErr> {
        self.prove_input_calls += 1;
        Ok(())
    }

    fn check_balance(
        &mut self,
        _c_ins: &[[u8; 32]],
        _c_outs: &[[u8; 32]],
    ) -> Result<(), SpendProofErr> {
        self.check_balance_calls += 1;
        Ok(())
    }
}

fn assert_counts(cs: &SpyCs, bind: usize, prove: usize, balance: usize) {
    assert_eq!(cs.bind_root_calls, bind);
    assert_eq!(cs.prove_input_calls, prove);
    assert_eq!(cs.check_balance_calls, balance);
}

fn test_scalar(seed: u64) -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Z00ZScalar::try_from_bytes(bytes).expect("scalar")
}

fn canonical_spend_plan() -> SpendPlan {
    let recv_secret = ReceiverSecret::from_bytes([2u8; 32]).expect("receiver secret");
    let view_sk = derive_view_secret_key(&recv_secret).expect("view key");
    let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

    let r = test_scalar(77);
    let r_pub = Z00ZRistrettoPoint::from_secret_key(&r).to_bytes();
    let dh = compute_dh_sender(&r, &view_pk).expect("dh");
    let k_in = derive_k_dh(&dh);
    let owner_handle = derive_owner_handle(&recv_secret);
    let owner_tag = compute_owner_tag(&owner_handle, &k_in);

    let s_in = [7u8; 32];
    let asset_id = hash_zk::<AssetIdDomain>("", &[&s_in]);

    let blind = test_scalar(41);
    let c_in = create_commitment(12, &blind).expect("c_in");
    let c_in_bytes: [u8; 32] = c_in.as_bytes().try_into().expect("commitment bytes");

    let out = AssetLeaf {
        c_amount: c_in_bytes,
        ..AssetLeaf::default()
    };
    SpendPlan {
        chain_id: 3,
        prev_root: [1u8; 32].into(),
        inputs: vec![SpendInputRef {
            asset_id,
            serial_id: 1,
        }],
        leaf_sums: vec![SpendInputLeaf {
            asset_id,
            serial_id: 1,
            leaf_ad_id: asset_id,
            r_pub,
            owner_tag,
            c_amt: c_in_bytes,
        }],
        outputs: vec![out.into()],
    }
}

fn canonical_spend_witness() -> SpendWitness {
    SpendWitness {
        recv_sec: [2u8; 32],
        s_in_vec: vec![[7u8; 32]],
    }
}

#[test]
fn test_producer_contract_verifies() {
    let (tx, _) = test_spend_public_support::canonical_public_contract_tx();

    assert!(tx.proof.spend.is_some());
    assert!(tx.auth.spend.is_some());
    verify_tx_public_spend_contract(
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &tx,
    )
    .expect("canonical producer output must verify through the public contract");
}

#[test]
fn test_builder_accepts_plan() {
    let mut cs = SpyCs::default();
    let plan = canonical_spend_plan();
    let witness = canonical_spend_witness();

    build_spend_assets(&mut cs, &plan, &witness).expect("canonical spend plan must build");

    assert_counts(&cs, 1, 1, 1);
}

#[test]
fn test_builder_rejects_empty_inputs() {
    let mut cs = SpyCs::default();
    let mut plan = canonical_spend_plan();
    plan.inputs.clear();

    let err = build_spend_assets(&mut cs, &plan, &canonical_spend_witness())
        .expect_err("empty inputs must reject");

    assert_eq!(err, SpendBuildErr::EmptyInputs);
    assert_counts(&cs, 0, 0, 0);
}

#[test]
fn test_builder_rejects_outputs() {
    let mut cs = SpyCs::default();
    let mut plan = canonical_spend_plan();
    plan.outputs.clear();

    let err = build_spend_assets(&mut cs, &plan, &canonical_spend_witness())
        .expect_err("empty outputs must reject");

    assert_eq!(err, SpendBuildErr::EmptyOutputs);
    assert_counts(&cs, 0, 0, 0);
}

#[test]
fn test_builder_rejects_input_mismatch() {
    let mut cs = SpyCs::default();
    let mut plan = canonical_spend_plan();
    plan.leaf_sums.clear();

    let err = build_spend_assets(&mut cs, &plan, &canonical_spend_witness())
        .expect_err("input length mismatch must reject");

    assert_eq!(err, SpendBuildErr::InputMismatch);
    assert_counts(&cs, 0, 0, 0);
}

#[test]
fn test_builder_duplicate_input() {
    let mut cs = SpyCs::default();
    let mut plan = canonical_spend_plan();
    plan.inputs.push(SpendInputRef {
        asset_id: plan.inputs[0].asset_id,
        serial_id: plan.inputs[0].serial_id.saturating_add(1),
    });
    plan.leaf_sums.push(plan.leaf_sums[0].clone());

    let mut witness = canonical_spend_witness();
    witness.s_in_vec.push([8u8; 32]);

    let err =
        build_spend_assets(&mut cs, &plan, &witness).expect_err("duplicate input must reject");

    assert_eq!(err, SpendBuildErr::DupInput);
    assert_counts(&cs, 0, 0, 0);
}

#[test]
fn test_builder_rejects_bad_witness() {
    let mut cs = SpyCs::default();
    let plan = canonical_spend_plan();
    let mut witness = canonical_spend_witness();
    witness.recv_sec = [0u8; 32];

    let err = build_spend_assets(&mut cs, &plan, &witness).expect_err("bad witness must reject");

    assert_eq!(err, SpendBuildErr::BadWitness { idx: 0 });
    assert_counts(&cs, 0, 0, 0);
}

#[test]
fn test_builder_bad_leaf() {
    let mut cs = SpyCs::default();
    let mut plan = canonical_spend_plan();
    plan.leaf_sums[0].owner_tag = [0u8; 32];

    let err = build_spend_assets(&mut cs, &plan, &canonical_spend_witness())
        .expect_err("bad leaf must reject");

    assert_eq!(err, SpendBuildErr::BadLeaf { idx: 0 });
    assert_counts(&cs, 1, 0, 0);
}

#[test]
fn test_builder_rejects_bad_rules() {
    let mut cs = SpyCs::default();
    let mut plan = canonical_spend_plan();
    plan.leaf_sums[0].r_pub = [1u8; 32];

    let err = build_spend_assets(&mut cs, &plan, &canonical_spend_witness())
        .expect_err("bad rules must reject");

    assert_eq!(err, SpendBuildErr::BadRules);
    assert_counts(&cs, 1, 1, 0);
}

#[test]
fn test_producer_forged_receiver_input() {
    let (tx, prev_root) = test_spend_public_support::canonical_public_contract_tx();
    let mut proof_inputs = tx.proof.spend.as_ref().expect("spend proof").inputs.clone();
    proof_inputs[0].owner_tag_hex = hex::encode([0xAA; 32]);
    let bare_tx = z00z_wallets::tx::TxWire {
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
        ..tx
    };

    let err = build_public_spend_contract(
        &test_spend_public_support::receiver_keys(),
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &bare_tx,
        prev_root,
        proof_inputs,
        test_spend_public_support::canonical_proof_witness(),
    )
    .expect_err("forged receiver-bound proof inputs must reject before signing");

    assert_eq!(err, SpendPublicErr::ReceiverInputMismatch { idx: 0 });
}

#[test]
fn test_producer_input_asset_binding() {
    let (tx, prev_root) = test_spend_public_support::canonical_public_contract_tx();
    let mut proof_inputs = tx.proof.spend.as_ref().expect("spend proof").inputs.clone();
    proof_inputs[0].input_asset_id_hex = hex::encode([0xBB; 32]);
    let bare_tx = z00z_wallets::tx::TxWire {
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
        ..tx
    };

    let err = build_public_spend_contract(
        &test_spend_public_support::receiver_keys(),
        test_spend_public_support::CHAIN_ID,
        test_spend_public_support::TX_VERSION,
        test_spend_public_support::CHAIN_TYPE,
        test_spend_public_support::CHAIN_NAME,
        &bare_tx,
        prev_root,
        proof_inputs,
        test_spend_public_support::canonical_proof_witness(),
    )
    .expect_err("mismatched tx input binding must reject before signing");

    assert_eq!(err, SpendPublicErr::InputRefMismatch { idx: 0 });
}
