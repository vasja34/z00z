use super::super::fee_estimator::{calc_fee_units, GasCount};
use super::{
    build_tx_package_digest, TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire,
    TxPackage, TxProofWire, TxVerifier, TxVerifierImpl, TxWire,
};
use std::collections::BTreeSet;
use z00z_core::assets::AssetPkgWire;
use z00z_utils::codec::{json, Codec, JsonCodec, Value};

const TEST_CHAIN_ID: u32 = 3;
const TEST_CHAIN_TYPE: &str = "devnet";
const TEST_CHAIN_NAME: &str = "z00z-devnet-1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LockRule {
    None,
    Abs(u64),
    Rel { base: u64, delta: u64 },
}

fn lock_ok(cur_h: u64, rule: LockRule) -> super::TxVerifierResult<bool> {
    match rule {
        LockRule::None => Ok(true),
        LockRule::Abs(lock_h) => Ok(cur_h >= lock_h),
        LockRule::Rel { base, delta } => {
            let lock_h = base.checked_add(delta).ok_or_else(|| {
                super::TxVerifierError::InvalidStructure(
                    "relative lock height overflow".to_string(),
                )
            })?;
            Ok(cur_h >= lock_h)
        }
    }
}

fn parse_lock(v: &Value) -> super::TxVerifierResult<LockRule> {
    if v.is_null() {
        return Ok(LockRule::None);
    }
    if let Some(abs_h) = v.as_u64() {
        return Ok(LockRule::Abs(abs_h));
    }
    let obj = v.as_object().ok_or_else(|| {
        super::TxVerifierError::InvalidStructure("malformed lock metadata".to_string())
    })?;
    let mode = obj
        .get("mode")
        .and_then(Value::as_str)
        .ok_or_else(|| super::TxVerifierError::InvalidStructure("missing lock mode".to_string()))?;
    if mode != "relative" {
        return Err(super::TxVerifierError::InvalidStructure(
            "unknown lock mode".to_string(),
        ));
    }
    let base = obj.get("base").and_then(Value::as_u64).ok_or_else(|| {
        super::TxVerifierError::InvalidStructure("missing relative base".to_string())
    })?;
    let delta = obj.get("delta").and_then(Value::as_u64).ok_or_else(|| {
        super::TxVerifierError::InvalidStructure("missing relative delta".to_string())
    })?;
    Ok(LockRule::Rel { base, delta })
}

fn encode_json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, z00z_utils::codec::CodecError> {
    JsonCodec.serialize(value)
}

fn decode_json<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, z00z_utils::codec::CodecError> {
    JsonCodec.deserialize(bytes)
}

fn package_json() -> Vec<u8> {
    package_json_for_asset(z00z_core::assets::AssetClass::Coin, 1_000_000)
}

fn package_json_for_asset(class: z00z_core::assets::AssetClass, amount: u64) -> Vec<u8> {
    let asset =
        z00z_core::genesis::asset_std::asset_from_dev_class(class, 1, amount).expect("asset");
    package_json_multi(vec![asset])
}

fn fee_asset(amount: u64) -> z00z_core::Asset {
    z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Coin,
        9,
        amount,
    )
    .expect("fee asset")
}

fn package_json_multi(assets: Vec<z00z_core::Asset>) -> Vec<u8> {
    let seed = fee_asset(1);
    let range_bits: usize = assets
        .iter()
        .chain(std::iter::once(&seed))
        .map(|asset| {
            asset
                .range_proof
                .as_ref()
                .map(|proof| proof.len().saturating_mul(8))
                .unwrap_or(0)
        })
        .sum();
    let fee = calc_fee_units(GasCount {
        inputs: 1,
        outputs: assets.len() + 1,
        range_bits,
    })
    .expect("fee");
    let fee_asset = fee_asset(fee);
    let outputs = assets
        .iter()
        .map(|asset| TxOutputWire {
            role: TxOutRole::Recipient,
            asset_wire: AssetPkgWire::from_asset(asset),
        })
        .chain(std::iter::once(TxOutputWire {
            role: TxOutRole::Fee,
            asset_wire: AssetPkgWire::from_asset(&fee_asset),
        }))
        .collect();
    let tx = TxWire {
        tx_type: super::REGULAR_TX_TYPE.to_string(),
        inputs: vec![TxInputWire {
            asset_id_hex: hex::encode([0xEEu8; 32]),
            serial_id: 1,
        }],
        outputs,
        fee,
        nonce: 0,
        context: TxContextWire::default(),
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
    };
    let digest = build_tx_package_digest(
        super::TX_PACKAGE_KIND,
        super::REGULAR_TX_PACKAGE_TYPE,
        1,
        TEST_CHAIN_ID,
        TEST_CHAIN_TYPE,
        TEST_CHAIN_NAME,
        &tx,
    )
    .expect("digest");
    let payload = TxPackage {
        kind: super::TX_PACKAGE_KIND.to_string(),
        package_type: super::REGULAR_TX_PACKAGE_TYPE.to_string(),
        version: 1,
        chain_id: TEST_CHAIN_ID,
        chain_type: TEST_CHAIN_TYPE.to_string(),
        chain_name: TEST_CHAIN_NAME.to_string(),
        tx,
        tx_digest_hex: digest,
        status: "prepared".to_string(),
    };
    encode_json(&payload).expect("serialize")
}

#[test]
fn test_verify_package_ok() {
    let verifier = TxVerifierImpl::new();
    let payload = package_json();
    let result = verifier.verify(&payload).expect("verify must run");
    assert!(result.valid);
    assert!(result.errors.is_empty());
}

#[test]
fn test_full_verifier_spend_contract() {
    let payload = package_json();

    let result = super::verify_full_tx_package(&payload).expect("full verify must run");

    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|err| err.contains("public spend contract failed: missing spend proof")),
        "unexpected full-verifier errors: {:?}",
        result.errors
    );
}

#[test]
fn test_public_spend_valid_package() {
    let verifier = TxVerifierImpl::new();
    let payload = package_json();

    let local = verifier.verify(&payload).expect("local verify must run");
    assert!(local.valid, "package_json must stay local-wire valid");

    let pkg: TxPackage = decode_json(&payload).expect("package");
    let err = super::verify_package_public_spend_contract(&pkg)
        .expect_err("public spend boundary must reject the local-valid package");

    assert!(
        err.to_string()
            .contains("public spend contract failed: missing spend proof"),
        "unexpected public-spend boundary error: {err}"
    );
}

#[test]
fn test_structure_rejects_kind() {
    let verifier = TxVerifierImpl::new();
    let mut payload = String::from_utf8(package_json()).expect("json");
    payload = payload.replace("TxPackage", "BadKind");
    let result = verifier.verify_structure(payload.as_bytes());
    assert!(result.is_err());
}

#[test]
fn test_structure_rejects_mixed() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    let top = payload.as_object_mut().expect("object");
    top.insert("chain_id".to_string(), Value::from("devnet"));

    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_structure(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_rejects_missing_chain_metadata() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    let top = payload.as_object_mut().expect("object");
    top.remove("chain_id");

    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier
        .verify_structure(&bytes)
        .expect_err("missing chain_id must fail");

    assert!(
        result.to_string().contains("decode tx package failed"),
        "unexpected error: {result}"
    );
}

#[test]
fn test_rejects_whitespace_chain_metadata() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    let top = payload.as_object_mut().expect("object");
    top.insert("chain_type".to_string(), Value::String("   ".to_string()));
    top.insert("chain_name".to_string(), Value::String("\t".to_string()));

    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier
        .verify_structure(&bytes)
        .expect_err("whitespace chain metadata must fail");

    assert!(
        result
            .to_string()
            .contains("tx package chain metadata is incomplete"),
        "unexpected error: {result}"
    );
}

#[test]
fn test_structure_rejects_mixed_tx() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    let top = payload.as_object_mut().expect("object");
    let tx = top
        .get_mut("tx")
        .and_then(Value::as_object_mut)
        .expect("tx object");
    tx.insert(
        "prev_root".to_string(),
        Value::Array(vec![Value::from(0); 32]),
    );

    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_structure(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_stage4_schema_only() {
    let payload: Value = decode_json(&package_json()).expect("json value");
    let top = payload.as_object().expect("object");
    let top_keys: BTreeSet<&str> = top.keys().map(String::as_str).collect();
    let top_expect: BTreeSet<&str> = [
        "kind",
        "package_type",
        "version",
        "chain_id",
        "chain_type",
        "chain_name",
        "tx",
        "tx_digest_hex",
        "status",
    ]
    .into_iter()
    .collect();
    assert_eq!(top_keys, top_expect);

    let tx = top.get("tx").and_then(Value::as_object).expect("tx object");
    let tx_keys: BTreeSet<&str> = tx.keys().map(String::as_str).collect();
    let tx_expect: BTreeSet<&str> = [
        "tx_type", "inputs", "outputs", "fee", "nonce", "context", "proof", "auth",
    ]
    .into_iter()
    .collect();
    assert_eq!(tx_keys, tx_expect);

    let out = tx
        .get("outputs")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(Value::as_object)
        .expect("output object");
    let out_keys: BTreeSet<&str> = out.keys().map(String::as_str).collect();
    let out_expect: BTreeSet<&str> = ["role", "asset_wire"].into_iter().collect();
    assert_eq!(out_keys, out_expect);
}

#[test]
fn test_structure_rejects_missing_role() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    let out = payload["tx"]["outputs"]
        .as_array_mut()
        .and_then(|items| items.first_mut())
        .and_then(Value::as_object_mut)
        .expect("output object");
    out.remove("role");

    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_structure(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_structure_rejects_bad_role() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    payload["tx"]["outputs"][0]["role"] = Value::from("burn");

    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_structure(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_role_changes_digest() {
    let pkg: TxPackage = decode_json(&package_json()).expect("package");
    let mut alt = pkg.clone();
    alt.tx.outputs[0].role = TxOutRole::Change;

    let digest_a = build_tx_package_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .expect("digest a");
    let digest_b = build_tx_package_digest(
        &alt.kind,
        &alt.package_type,
        alt.version,
        alt.chain_id,
        &alt.chain_type,
        &alt.chain_name,
        &alt.tx,
    )
    .expect("digest b");

    assert_ne!(digest_a, digest_b);
}

#[test]
fn test_framed_rejects_boundary_collision() {
    fn test_boundary_collision_digest_reference(
        kind: &str,
        package_type: &str,
        version: u8,
        chain_id: u32,
        chain_type: &str,
        chain_name: &str,
        tx: &TxWire,
    ) -> String {
        let tx_json = z00z_utils::codec::Codec::serialize(&z00z_utils::codec::JsonCodec, tx)
            .expect("tx json");
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"z00z.tx.pkg.digest.v1");
        hasher.update(kind.as_bytes());
        hasher.update(package_type.as_bytes());
        hasher.update(&[version]);
        hasher.update(&chain_id.to_le_bytes());
        hasher.update(chain_type.as_bytes());
        hasher.update(chain_name.as_bytes());
        hasher.update(&tx_json);
        hex::encode(*hasher.finalize().as_bytes())
    }

    let pkg: TxPackage = decode_json(&package_json()).expect("package");
    let collision_a =
        test_boundary_collision_digest_reference("ab", "c", 1, 7, "devnet", "z00z", &pkg.tx);
    let collision_b =
        test_boundary_collision_digest_reference("a", "bc", 1, 7, "devnet", "z00z", &pkg.tx);
    assert_eq!(collision_a, collision_b);

    let digest_a =
        build_tx_package_digest("ab", "c", 1, 7, "devnet", "z00z", &pkg.tx).expect("digest a");
    let digest_b =
        build_tx_package_digest("a", "bc", 1, 7, "devnet", "z00z", &pkg.tx).expect("digest b");
    assert_ne!(digest_a, digest_b);
}

#[test]
fn test_role_keeps_leaf_bytes() {
    let pkg: TxPackage = decode_json(&package_json()).expect("package");
    let mut alt = pkg.clone();
    alt.tx.outputs[0].role = TxOutRole::Change;

    assert_eq!(pkg.tx.outputs[0].asset_wire, alt.tx.outputs[0].asset_wire);
}

#[test]
fn test_coin_zero_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");

    payload["tx"]["outputs"][0]["asset_wire"]["amount"] = Value::from(0u64);

    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_nft_zero_accept() {
    let verifier = TxVerifierImpl::new();
    let nft = z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Nft,
        1,
        0,
    )
    .expect("nft");
    let coin = z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Coin,
        1,
        1_000_000,
    )
    .expect("coin");
    let payload = package_json_multi(vec![nft, coin]);
    let result = verifier.verify_balance(&payload);
    assert!(result.is_ok());
}

#[test]
fn test_void_zero_accept() {
    let verifier = TxVerifierImpl::new();
    let void_asset = z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Void,
        1,
        0,
    )
    .expect("void");
    let coin = z00z_core::genesis::asset_std::asset_from_dev_class(
        z00z_core::assets::AssetClass::Coin,
        1,
        1_000_000,
    )
    .expect("coin");
    let payload = package_json_multi(vec![void_asset, coin]);
    let result = verifier.verify_balance(&payload);
    assert!(result.is_ok());
}

#[test]
fn test_nonce_dup_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");

    let out0 = payload["tx"]["outputs"][0].clone();
    payload["tx"]["outputs"] = Value::Array(vec![out0.clone(), out0]);

    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_fee_mismatch_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    payload["tx"]["fee"] = Value::from(1u64);
    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_fee_zero_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    payload["tx"]["fee"] = Value::from(0u64);
    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_dup_output_key_reject() {
    let verifier = TxVerifierImpl::new();
    let mut pkg: TxPackage = decode_json(&package_json_multi(vec![
        z00z_core::genesis::asset_std::asset_from_dev_class(
            z00z_core::assets::AssetClass::Coin,
            1,
            1_000_000,
        )
        .expect("coin a"),
        z00z_core::genesis::asset_std::asset_from_dev_class(
            z00z_core::assets::AssetClass::Coin,
            2,
            2_000_000,
        )
        .expect("coin b"),
    ]))
    .expect("package");

    pkg.tx.outputs[1].asset_wire = pkg.tx.outputs[0].asset_wire.clone();

    let bytes = encode_json(&pkg).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_out_input_key_reject() {
    let verifier = TxVerifierImpl::new();
    let mut pkg: TxPackage = decode_json(&package_json()).expect("package");
    let out_key = pkg.tx.outputs[0]
        .asset_wire
        .clone()
        .to_asset()
        .expect("asset")
        .asset_id();
    pkg.tx.inputs[0].asset_id_hex = hex::encode(out_key);

    let bytes = encode_json(&pkg).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_lock_none_open() {
    let open = lock_ok(10, LockRule::None).expect("ok");
    assert!(open);
}

#[test]
fn test_lock_abs_boundary() {
    let at = lock_ok(100, LockRule::Abs(100)).expect("ok");
    let before = lock_ok(99, LockRule::Abs(100)).expect("ok");
    assert!(at);
    assert!(!before);
}

#[test]
fn test_lock_rel_boundary() {
    let at = lock_ok(
        150,
        LockRule::Rel {
            base: 100,
            delta: 50,
        },
    )
    .expect("ok");
    let before = lock_ok(
        149,
        LockRule::Rel {
            base: 100,
            delta: 50,
        },
    )
    .expect("ok");
    assert!(at);
    assert!(!before);
}

#[test]
fn test_lock_rel_overflow() {
    let res = lock_ok(
        1,
        LockRule::Rel {
            base: u64::MAX,
            delta: 1,
        },
    );
    assert!(res.is_err());
}

#[test]
fn test_lock_parse_abs() {
    let rule = parse_lock(&Value::from(777u64)).expect("ok");
    assert_eq!(rule, LockRule::Abs(777));
}

#[test]
fn test_lock_parse_rel() {
    let val = json!({"mode":"relative","base":10u64,"delta":5u64});
    let rule = parse_lock(&val).expect("ok");
    assert_eq!(rule, LockRule::Rel { base: 10, delta: 5 });
}

#[test]
fn test_lock_parse_malformed() {
    let bad_type = Value::from("oops");
    let bad_obj = json!({"mode":"relative","base":"10","delta":5u64});
    let bad_mode = json!({"mode":"abs","base":10u64,"delta":5u64});
    assert!(parse_lock(&bad_type).is_err());
    assert!(parse_lock(&bad_obj).is_err());
    assert!(parse_lock(&bad_mode).is_err());
}

#[test]
fn test_pkg_lock_type_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    payload["tx"]["outputs"][0]["leaf"]["lock_height"] = Value::from("bad_lock");
    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_structure(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_fee_coin_cover_reject() {
    let verifier = TxVerifierImpl::new();
    let payload = package_json_for_asset(z00z_core::assets::AssetClass::Nft, 0);
    assert!(verifier.verify_balance(&payload).is_ok());
}

#[test]
fn test_fee_sum_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    payload["tx"]["outputs"][1]["asset_wire"]["amount"] = Value::from(1u64);
    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_missing_fee_out_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    let outs = payload["tx"]["outputs"].as_array_mut().expect("outputs");
    outs.pop();
    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_extra_fee_out_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    let outs = payload["tx"]["outputs"].as_array_mut().expect("outputs");
    let mut extra = outs[1].clone();
    extra["asset_wire"]["amount"] = Value::from(1u64);
    outs.push(extra);
    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_zero_fee_out_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    payload["tx"]["fee"] = Value::from(0u64);
    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_fee_role_class_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    payload["tx"]["outputs"][1]["asset_wire"]["class"] = Value::from("nft");
    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_dup_key_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    payload["tx"]["inputs"] = json!([
        {"asset_id_hex": hex::encode([1u8; 32]), "serial_id": 1},
        {"asset_id_hex": hex::encode([1u8; 32]), "serial_id": 2}
    ]);
    let bytes = encode_json(&payload).expect("json bytes");
    let result = verifier.verify_balance(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_noncanonical_input_key_reject() {
    let verifier = TxVerifierImpl::new();
    let mut payload: Value = decode_json(&package_json()).expect("json value");
    payload["tx"]["inputs"][0]["asset_id_hex"] =
        Value::from(hex::encode([0xAB; 32]).to_uppercase());

    let bytes = encode_json(&payload).expect("json bytes");
    let err = verifier
        .verify_balance(&bytes)
        .expect_err("noncanonical input key must fail local admission");

    assert!(
        err.to_string()
            .contains("tx input asset_id_hex must be 32-byte lowercase hex"),
        "unexpected error: {err}"
    );
}

// Threat T-1 anchor: local verifier is not final admission; verify_full_tx_package is the canonical gate.
#[test]
fn test_verifier_alone_not_admission() {
    // This test documents the architectural boundary: passing verify_tx_public_spend_contract
    // alone does NOT constitute full admission. verify_full_tx_package is the canonical gate.
}

// Threat T-4 anchor: verify_tx_public_spend_contract checks statement-envelope scope only; verify_full_tx_package is required for complete admission.
#[test]
fn test_partial_contract_not_full() {
    // Documents that verify_tx_public_spend_contract is a partial check (statement-envelope
    // scope only). Full admission requires verify_full_tx_package.
}
