use z00z_core::{
    genesis::asset_std::{asset_from_dev_class, serials_from_dev_class},
    Asset, AssetClass, AssetPkgWire, AssetWire,
};
use z00z_crypto::claim::{ClaimAuthoritySig, ClaimStmt};
use z00z_crypto::expert::{keys::RistrettoSecretKey, traits::SecretKeyTrait};
use z00z_storage::settlement::{
    DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem, TerminalId,
    TerminalLeaf as StoreLeaf,
};
use z00z_utils::rng::SystemRngProvider;
use z00z_wallets::{
    key::ReceiverKeys,
    stealth::bind_stealth_output_wire,
    tx::{
        asset_wire_to_leaf, build_claim_stmt, build_claim_tx_digest, compute_claim_scope_hash,
        derive_output_nonce, sign_owner_attest, ClaimAuthWire, ClaimContextWire, ClaimInputWire,
        ClaimOutputWire, ClaimProofWire, ClaimScopeKey, ClaimTxPackage, ClaimTxWire, CLAIM_PKG,
    },
};

const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";
const CLAIM_AUTH_TEST_SEED: [u8; 64] = [0xA7u8; 64];
pub(crate) const ZERO_ROOT: [u8; 32] = [0u8; 32];

pub fn make_pkg(chain_id: u32, wire: AssetWire, keys: &ReceiverKeys) -> ClaimTxPackage {
    let leaf = asset_wire_to_leaf(&wire).expect("leaf");
    let claim_id = derive_output_nonce(&wire.definition.id, wire.serial_id);
    let owner = keys.owner_handle;
    let nullifier = z00z_wallets::claim::derive_nullifier(&claim_id, &owner, chain_id);
    let asset_id_hex = hex::encode(leaf.asset_id);

    let mut tx = ClaimTxWire {
        tx_type: "claim_tx".to_string(),
        inputs: vec![ClaimInputWire {
            claim_id_hex: hex::encode(claim_id),
            claim_source_asset_id_hex: asset_id_hex.clone(),
            claim_source_commitment_hex: hex::encode(wire.commitment.as_bytes()),
        }],
        outputs: vec![ClaimOutputWire {
            asset_id_hex,
            amount: wire.amount,
            asset_class: "coin".to_string(),
            owner_binding_hex: hex::encode(owner),
            nonce_hex: hex::encode(derive_output_nonce(&claim_id, 0)),
            asset_wire: Some(AssetPkgWire::from_wire(&wire)),
            owner_attest_hex: None,
        }],
        fee: 0,
        nonce: 0,
        context: ClaimContextWire {
            recipient_wallet_id: "alice".to_string(),
            recipient_owner_hex: hex::encode(owner),
            claim_scope_hash_hex: hex::encode(compute_claim_scope_hash(&ClaimScopeKey {
                chain_id,
                scenario_tag: "scenario_1_genesis_claim".to_string(),
                ruleset_version: 1,
            })),
            recipient_card_hex: Some(hex::encode(
                keys.export_receiver_card()
                    .expect("card")
                    .canonical_encoding(),
            )),
            nullifier_hex: nullifier.to_hex(),
        },
        proof: ClaimProofWire {
            proof_type: "genesis_claim".to_string(),
            proof_hex: String::new(),
        },
        auth: ClaimAuthWire {
            claim_authority_sig_hex: String::new(),
        },
    };

    let attest = sign_owner_attest(
        keys,
        chain_id,
        &tx,
        0,
        tx.outputs[0].asset_wire.as_ref().expect("asset wire"),
    )
    .expect("owner attest");
    tx.outputs[0].owner_attest_hex = Some(attest);

    let mut pkg = ClaimTxPackage {
        kind: "TxPackage".to_string(),
        package_type: "claim_tx".to_string(),
        version: CLAIM_PKG,
        chain_id,
        chain_type: CHAIN_TYPE.to_string(),
        chain_name: CHAIN_NAME.to_string(),
        tx,
        tx_digest_hex: String::new(),
        status: "prepared".to_string(),
    };

    fill_claim(&mut pkg);
    sync_pkg(&mut pkg);
    pkg
}

pub fn sync_pkg(pkg: &mut ClaimTxPackage) {
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
}

pub fn pair_leaf(leaf: &StoreLeaf, amount: u64) -> (StoreLeaf, Asset) {
    let canon = leaf.clone();
    let serials = serials_from_dev_class(AssetClass::Coin).expect("dev coin serials");
    assert!(
        leaf.serial_id < serials,
        "test leaf serial_id {} exceeds live dev fixture limit {}",
        leaf.serial_id,
        serials
    );
    let runtime = bind_stealth_output_wire(
        AssetWire::from_asset(
            &asset_from_dev_class(AssetClass::Coin, leaf.serial_id, amount).expect("asset"),
        ),
        leaf,
    )
    .expect("bind wire")
    .to_asset()
    .expect("runtime asset");
    (canon, runtime)
}

fn fill_claim(pkg: &mut ClaimTxPackage) {
    let proof = make_claim_source_proof(pkg.tx.outputs[0].asset_wire.as_ref().expect("leaf"))
        .expect("claim proof");

    pkg.tx.proof.proof_type = "claim_source".to_string();
    pkg.tx.proof.proof_hex = hex::encode(proof.to_bytes().expect("proof bytes"));
    let stmt = build_claim_stmt(pkg).expect("claim stmt");
    let sig = sign_claim_auth_test(&stmt).expect("claim auth");
    pkg.tx.auth.claim_authority_sig_hex = hex::encode(sig.to_bytes().expect("sig bytes"));
}

fn sign_claim_auth_test(stmt: &ClaimStmt) -> Result<ClaimAuthoritySig, String> {
    let auth_sk = RistrettoSecretKey::from_uniform_bytes(&CLAIM_AUTH_TEST_SEED)
        .expect("claim auth test seed must stay canonical");
    let mut rng = SystemRngProvider.rng();
    ClaimAuthoritySig::sign(stmt, &auth_sk, &mut rng).map_err(|e| e.to_string())
}

fn make_claim_source_proof(
    leaf_pkg: &AssetPkgWire,
) -> Result<z00z_crypto::ClaimSourceProof, String> {
    let wire = leaf_pkg
        .clone()
        .to_wire()
        .map_err(|e| format!("claim source to_wire failed: {e}"))?;
    let leaf = asset_wire_to_leaf(&wire).map_err(|e| format!("claim source leaf failed: {e}"))?;
    let path = SettlementPath::new(
        DefinitionId::new(wire.definition.id),
        SerialId::new(leaf.serial_id),
        TerminalId::new(leaf.asset_id),
    );
    let item = StoreItem::new(path, leaf).map_err(|e| format!("claim source item failed: {e}"))?;
    let mut store = SettlementStore::new();
    store
        .put_settlement_item(item.clone())
        .map_err(|e| format!("claim source store insert failed: {e}"))?;
    let (_, proof) = store
        .claim_source_contract_for_item(&item)
        .map_err(|e| format!("claim source proof failed: {e}"))?;
    Ok(proof)
}
