use std::path::PathBuf;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use test_common::managed_test_output_root;
use z00z_core::{
    assets::{AssetPkgWire, AssetWire},
    genesis::asset_std::asset_from_dev_class,
};
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::tx::fee_estimator::calculate_fee_for_wires;
use z00z_wallets::tx::{
    build_tx_package_digest, TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire,
    TxPackage, TxProofWire, TxVerifier, TxVerifierImpl, TxWire,
};

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";

fn out_dir() -> PathBuf {
    managed_test_output_root("e2e09")
}

fn mk_pkg() -> Vec<u8> {
    let asset =
        asset_from_dev_class(z00z_core::assets::AssetClass::Coin, 1, 2_000_000).expect("asset");
    let fee_seed =
        asset_from_dev_class(z00z_core::assets::AssetClass::Coin, 9, 1).expect("fee seed");
    let wire = AssetWire::from_asset(&asset);
    let fee_seed = AssetWire::from_asset(&fee_seed);
    let fee = calculate_fee_for_wires(1, &[wire.clone(), fee_seed]).expect("fee");
    let fee_asset =
        asset_from_dev_class(z00z_core::assets::AssetClass::Coin, 9, fee).expect("fee asset");
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
    let pkg = TxPackage {
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

    serde_json::to_vec(&pkg).expect("serialize")
}

#[test]
fn test_stage4_pass() {
    if cfg!(debug_assertions) {
        return;
    }

    let pkg = mk_pkg();
    let parsed: TxPackage = serde_json::from_slice(&pkg).expect("decode pkg");
    assert_eq!(parsed.tx.inputs.len(), 1, "must have one input");
    assert_eq!(
        parsed.tx.outputs.len(),
        2,
        "must have recipient and fee outputs"
    );

    let out_asset = parsed.tx.outputs[0]
        .asset_wire
        .clone()
        .to_asset()
        .expect("decode output asset");
    assert!(
        out_asset.owner_signature.is_some(),
        "owner signature must be present"
    );
    assert!(
        out_asset.range_proof.is_some(),
        "range proof must be present"
    );
    assert!(
        out_asset
            .commitment
            .as_bytes()
            .iter()
            .any(|byte| *byte != 0),
        "commitment must be non-zero"
    );
    assert_ne!(out_asset.nonce, [0u8; 32], "nonce must be non-zero");

    let ver = TxVerifierImpl::new();

    assert!(ver.verify_structure(&pkg).is_ok(), "structure must pass");
    assert!(ver.verify_signatures(&pkg).is_ok(), "signatures must pass");
    assert!(
        ver.verify_range_proofs(&pkg).is_ok(),
        "range proofs must pass"
    );
    assert!(ver.verify_balance(&pkg).is_ok(), "balance must pass");

    let got = ver.verify(&pkg).expect("verify runs");
    assert!(got.valid, "aggregated verify must be valid");
    assert!(got.errors.is_empty(), "aggregated errors must be empty");

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e09");
    write_file(out_dir().join("e2e09_pkg.json"), &pkg).expect("write pkg");
    write_file(out_dir().join("package_valid.json"), &pkg).expect("write package_valid");

    let mut out = String::from("E2E-09 verifier\n");
    out.push_str(&format!("valid={}\n", got.valid));
    out.push_str(&format!("errors_len={}\n", got.errors.len()));
    write_file(out_dir().join("e2e09_verify.txt"), out.as_bytes()).expect("write verify");
    write_file(out_dir().join("verify_out.txt"), out.as_bytes()).expect("write verify_out");
}
