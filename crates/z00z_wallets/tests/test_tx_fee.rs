use std::path::PathBuf;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use test_common::managed_test_output_root;
use z00z_core::{
    assets::{AssetPkgWire, AssetWire},
    genesis::asset_std::asset_from_dev_class,
};
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::tx::fee_estimator::{
    calculate_fee_for_wires, BASE_TX_COST, FEE_WEIGHT_TAG, PER_INPUT_COST, PER_OUTPUT_COST,
    PER_RANGE_BIT_COST,
};
use z00z_wallets::tx::TxAssemblerImpl;
use z00z_wallets::tx::{
    build_tx_package_digest, TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire,
    TxPackage, TxProofWire, TxVerifier, TxVerifierImpl, TxWire,
};

const CHAIN_ID: u32 = 3;
const CHAIN_TYPE: &str = "devnet";
const CHAIN_NAME: &str = "z00z-devnet-1";

fn mk_wire(class: z00z_core::assets::AssetClass, amount: u64) -> AssetWire {
    let mut asset = asset_from_dev_class(class, 1, amount).expect("asset");
    asset.owner_pub = None;
    asset.owner_signature = None;
    AssetWire::from_asset(&asset)
}

fn mk_pkg(outputs: Vec<(TxOutRole, AssetWire)>, fee: u64, inputs: usize) -> Vec<u8> {
    let tx_wire = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: (0..inputs)
            .map(|idx| TxInputWire {
                asset_id_hex: hex::encode([idx as u8 + 1; 32]),
                serial_id: idx as u32 + 1,
            })
            .collect(),
        outputs: outputs
            .into_iter()
            .map(|(role, asset_wire)| TxOutputWire {
                role,
                asset_wire: AssetPkgWire::from_wire(&asset_wire),
            })
            .collect(),
        fee,
        nonce: 0,
        context: TxContextWire::default(),
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
    };
    let tx = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: CHAIN_ID,
        chain_type: CHAIN_TYPE.to_string(),
        chain_name: CHAIN_NAME.to_string(),
        tx: tx_wire.clone(),
        tx_digest_hex: build_tx_package_digest(
            "TxPackage",
            "regular_tx",
            1,
            CHAIN_ID,
            CHAIN_TYPE,
            CHAIN_NAME,
            &tx_wire,
        )
        .expect("digest"),
        status: "prepared".to_string(),
    };

    serde_json::to_vec(&tx).expect("json")
}

fn out_dir() -> PathBuf {
    managed_test_output_root("e2e08")
}

fn run_vec(
    vec_id: &str,
    inputs: usize,
    coin_n: usize,
    nft_n: usize,
    asm: &TxAssemblerImpl,
    ver: &TxVerifierImpl,
    log: &mut String,
) -> usize {
    let mut outputs = Vec::with_capacity(coin_n + nft_n);
    for idx in 0..coin_n {
        outputs.push(mk_wire(
            z00z_core::assets::AssetClass::Coin,
            5_000_000_000 + idx as u64 * 1_000_000,
        ));
    }
    for _ in 0..nft_n {
        outputs.push(mk_wire(z00z_core::assets::AssetClass::Nft, 0));
    }

    for (idx, out) in outputs.iter_mut().enumerate() {
        let mut nonce = [0u8; 32];
        nonce[0] = idx as u8 + 1;
        nonce[1] = vec_id.as_bytes()[0];
        out.nonce = nonce;
    }

    let rp_bits: usize = outputs
        .iter()
        .map(|out| {
            out.range_proof
                .as_ref()
                .map(|proof| proof.len() * 8)
                .unwrap_or(0)
        })
        .sum();

    let fee_seed = mk_wire(z00z_core::assets::AssetClass::Coin, 1);
    let mut fee_outs = outputs.clone();
    fee_outs.push(fee_seed);
    let fee_decl = asm
        .calculate_fee_for_wires(inputs, &fee_outs)
        .expect("builder fee");
    let fee_can = calculate_fee_for_wires(inputs, &fee_outs).expect("canonical fee");
    let fee_wire = mk_wire(z00z_core::assets::AssetClass::Coin, fee_decl);
    let mut tx_outs: Vec<(TxOutRole, AssetWire)> = outputs
        .clone()
        .into_iter()
        .map(|wire| (TxOutRole::Recipient, wire))
        .collect();
    tx_outs.push((TxOutRole::Fee, fee_wire));

    assert_eq!(fee_decl, fee_can, "fee mismatch on {vec_id}");

    let ok_pkg = mk_pkg(tx_outs.clone(), fee_decl, inputs);
    let ok_res = ver.verify_balance(&ok_pkg);
    assert!(ok_res.is_ok(), "valid fee must pass {vec_id}: {ok_res:?}");

    let mut bad_outs = tx_outs;
    bad_outs.pop();
    bad_outs.push((
        TxOutRole::Fee,
        mk_wire(z00z_core::assets::AssetClass::Coin, fee_decl + 1),
    ));
    let bad_pkg = mk_pkg(bad_outs, fee_decl + 1, inputs);
    assert!(
        ver.verify_balance(&bad_pkg).is_err(),
        "wrong fee must reject {vec_id}"
    );

    log.push_str(&format!(
        "vec={} inputs={} coin_n={} nft_n={} rp_bits={} fee_decl={} fee_can={} ok=1 bad=1\n",
        vec_id, inputs, coin_n, nft_n, rp_bits, fee_decl, fee_can
    ));

    rp_bits
}

fn run_bad(asm: &TxAssemblerImpl, ver: &TxVerifierImpl, log: &mut String) {
    let outputs = vec![
        mk_wire(z00z_core::assets::AssetClass::Coin, 8_000_000_000),
        mk_wire(z00z_core::assets::AssetClass::Coin, 8_100_000_000),
    ];

    let fee_seed = mk_wire(z00z_core::assets::AssetClass::Coin, 1);
    let mut fee_outs = outputs.clone();
    fee_outs.push(fee_seed);
    let fee_ok = asm
        .calculate_fee_for_wires(2, &fee_outs)
        .expect("builder fee bad-case");
    let fee_bad = fee_ok + 7;
    let mut tx_outs: Vec<(TxOutRole, AssetWire)> = outputs
        .into_iter()
        .map(|wire| (TxOutRole::Recipient, wire))
        .collect();
    tx_outs.push((
        TxOutRole::Fee,
        mk_wire(z00z_core::assets::AssetClass::Coin, fee_bad),
    ));
    let pkg = mk_pkg(tx_outs, fee_bad, 2);

    let got = ver.verify_balance(&pkg);
    assert!(got.is_err(), "explicit wrong-fee vector must reject");

    log.push_str(&format!(
        "wrong_fee vec=bad-1 inputs=2 outs=2 fee_ok={} fee_bad={} reject=1\n",
        fee_ok, fee_bad
    ));
}

#[test]
fn test_fee_units_ignore_value() {
    let asm = TxAssemblerImpl::new();
    let one = mk_wire(z00z_core::assets::AssetClass::Coin, 1);
    let big = mk_wire(z00z_core::assets::AssetClass::Coin, 9_999_999_999);

    let fee_one = asm
        .calculate_fee_for_wires(2, &[one.clone(), one.clone()])
        .expect("fee one");
    let fee_big = asm
        .calculate_fee_for_wires(2, &[big.clone(), big])
        .expect("fee big");

    assert_eq!(fee_one, fee_big);
}

#[test]
fn test_fee_units_track_structure() {
    let asm = TxAssemblerImpl::new();
    let rec_a = mk_wire(z00z_core::assets::AssetClass::Coin, 10);
    let rec_b = mk_wire(z00z_core::assets::AssetClass::Coin, 20);
    let fee_small = mk_wire(z00z_core::assets::AssetClass::Coin, 1);
    let fee_big = mk_wire(z00z_core::assets::AssetClass::Coin, 999_999);

    let no_fee = asm
        .calculate_fee_for_wires(2, &[rec_a.clone(), rec_b.clone()])
        .expect("fee no extra output");
    let with_small = asm
        .calculate_fee_for_wires(2, &[rec_a.clone(), rec_b.clone(), fee_small])
        .expect("fee with small extra output");
    let with_big = asm
        .calculate_fee_for_wires(2, &[rec_a, rec_b, fee_big])
        .expect("fee with big extra output");

    assert!(with_small > no_fee);
    assert_eq!(with_small, with_big);
}

#[test]
fn test_stage4_fee() {
    if cfg!(debug_assertions) {
        return;
    }

    assert_eq!(FEE_WEIGHT_TAG, "fee-weight-v1");
    assert_eq!(BASE_TX_COST, 64);
    assert_eq!(PER_INPUT_COST, 96);
    assert_eq!(PER_OUTPUT_COST, 900);
    assert_eq!(PER_RANGE_BIT_COST, 1);

    let asm = TxAssemblerImpl::new();
    let ver = TxVerifierImpl::new();

    let mut log = String::from("E2E-08 fee table\n");
    log.push_str("constants=fee-weight-v1,64,96,900,1\n");

    let low_1 = run_vec("low-1", 1, 1, 0, &asm, &ver, &mut log);
    let low_2 = run_vec("low-2", 2, 1, 0, &asm, &ver, &mut log);
    let mid_1 = run_vec("mid-1", 1, 2, 0, &asm, &ver, &mut log);
    let mid_2 = run_vec("mid-2", 2, 3, 0, &asm, &ver, &mut log);
    let high_1 = run_vec("high-1", 1, 4, 0, &asm, &ver, &mut log);
    let high_2 = run_vec("high-2", 2, 5, 0, &asm, &ver, &mut log);

    let low_max = low_1.max(low_2);
    let mid_min = mid_1.min(mid_2);
    let mid_max = mid_1.max(mid_2);
    let high_min = high_1.min(high_2);

    assert!(
        low_max < mid_min,
        "low rp_bits must be lower than mid rp_bits"
    );
    assert!(
        mid_max < high_min,
        "mid rp_bits must be lower than high rp_bits"
    );

    log.push_str(&format!(
        "bands low_max={} mid_min={} mid_max={} high_min={}\n",
        low_max, mid_min, mid_max, high_min
    ));
    run_bad(&asm, &ver, &mut log);

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e08");
    write_file(out_dir().join("e2e08_fee_table.txt"), log.as_bytes()).expect("write fee table");
}
