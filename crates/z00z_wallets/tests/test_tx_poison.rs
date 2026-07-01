use std::path::PathBuf;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use serde_json::Value;
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
    managed_test_output_root("e2e10")
}

fn mk_pkg() -> Vec<u8> {
    let asset =
        asset_from_dev_class(z00z_core::assets::AssetClass::Coin, 1, 2_000_000).expect("asset");
    let wire = AssetWire::from_asset(&asset);
    let fee = calculate_fee_for_wires(1, std::slice::from_ref(&wire)).expect("fee");

    let tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![TxInputWire {
            asset_id_hex: hex::encode([1u8; 32]),
            serial_id: 1,
        }],
        outputs: vec![TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: AssetPkgWire::from_wire(&wire),
        }],
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

fn add_prev_root(v: &mut Value) {
    let top = v.as_object_mut().expect("top object");
    let tx = top
        .get_mut("tx")
        .and_then(Value::as_object_mut)
        .expect("tx object");
    tx.insert(
        "prev_root".to_string(),
        Value::Array((0..32).map(Value::from).collect()),
    );
}

fn add_spent_meta(v: &mut Value) {
    let top = v.as_object_mut().expect("top object");
    let tx = top
        .get_mut("tx")
        .and_then(Value::as_object_mut)
        .expect("tx object");
    tx.insert(
        "spent_delta".to_string(),
        Value::Array(vec![Value::String("asset:01".to_string())]),
    );
}

fn add_ckpt_meta(v: &mut Value) {
    let top = v.as_object_mut().expect("top object");
    let tx = top
        .get_mut("tx")
        .and_then(Value::as_object_mut)
        .expect("tx object");
    tx.insert(
        "checkpoint_meta".to_string(),
        serde_json::json!({
            "height": 7,
            "root": [1, 2, 3, 4]
        }),
    );
}

fn set_chain_meta(v: &mut Value, chain_type: &str, chain_name: &str) {
    let top = v.as_object_mut().expect("top object");
    top.insert(
        "chain_type".to_string(),
        Value::String(chain_type.to_string()),
    );
    top.insert(
        "chain_name".to_string(),
        Value::String(chain_name.to_string()),
    );
}

#[test]
fn test_stage4_poison() {
    if cfg!(debug_assertions) {
        return;
    }

    let ver = TxVerifierImpl::new();
    let base = mk_pkg();
    assert!(
        ver.verify_structure(&base).is_ok(),
        "baseline package must pass verify_structure"
    );

    let mut variants: Vec<(String, Value)> = Vec::new();

    let mut first_payload: Value = serde_json::from_slice(&base).expect("json baseline");
    add_prev_root(&mut first_payload);
    variants.push(("prev_root_in_tx".to_string(), first_payload));

    let mut second_payload: Value = serde_json::from_slice(&base).expect("json baseline");
    add_spent_meta(&mut second_payload);
    variants.push(("spent_meta_in_tx".to_string(), second_payload));

    let mut third_payload: Value = serde_json::from_slice(&base).expect("json baseline");
    add_ckpt_meta(&mut third_payload);
    variants.push(("checkpoint_meta_top".to_string(), third_payload));

    let mut reasons = String::from("E2E-10 reject reasons\n");
    let mut corpus = Vec::new();
    let mut fail_count = 0usize;

    for (id, val) in &variants {
        let bytes = serde_json::to_vec(val).expect("variant json");
        let got = ver.verify_structure(&bytes);
        match got {
            Ok(_) => panic!("poison variant must fail: {id}"),
            Err(err) => {
                fail_count += 1;
                let msg = err.to_string();
                assert!(
                    msg.contains("invalid structure"),
                    "poison must fail as structure boundary: id={id}, msg={msg}"
                );
                reasons.push_str(&format!("id={id} reject={msg}\n"));
                corpus.push(serde_json::json!({
                    "id": id,
                    "payload": val,
                    "reject": msg
                }));
            }
        }
    }

    assert_eq!(
        fail_count,
        variants.len(),
        "all poison variants must be rejected"
    );
    assert!(
        ver.verify_structure(&base).is_ok(),
        "baseline package must still pass after poison checks"
    );

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e10");
    write_file(out_dir().join("baseline_valid.json"), &base).expect("write baseline");
    let corpus_obj = serde_json::json!({
        "test": "E2E-10",
        "variants": corpus
    });
    let corpus_bytes = serde_json::to_vec_pretty(&corpus_obj).expect("corpus bytes");
    write_file(out_dir().join("poison_corpus.json"), &corpus_bytes).expect("write corpus");
    write_file(out_dir().join("reject_reasons.txt"), reasons.as_bytes()).expect("write reasons");
}

#[test]
fn test_stage4_whitespace_chain_meta() {
    if cfg!(debug_assertions) {
        return;
    }

    let ver = TxVerifierImpl::new();
    let base = mk_pkg();
    let mut payload: Value = serde_json::from_slice(&base).expect("json baseline");

    set_chain_meta(&mut payload, "   ", "\t");

    let bytes = serde_json::to_vec(&payload).expect("variant json");
    let got = ver.verify_structure(&bytes);

    assert!(got.is_err(), "whitespace-only chain metadata must fail");
    let msg = got
        .expect_err("whitespace-only chain metadata must fail")
        .to_string();
    assert!(msg.contains("invalid structure"), "msg={msg}");
}
