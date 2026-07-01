use super::tx_lane_runtime_suite_support::{
    input_asset_id_hex, input_from_out, mk_balanced_out_from_input, mk_card, mk_cfg, mk_out,
    mk_out_with_serial, mk_pick_wire, mk_wire, pick_cfg,
};
use super::{
    build_canon_snapshot, build_prep_file, decode_output_pack, pick_sender_rows, prep_leaf,
    prep_root, prep_store, split_amount_cfg, to_tx_output_wires, verify_commitment_balance_gate,
    verify_spend_witness_gate, verify_tx_package,
};
use crate::scenario_1::stage_6::shared_cases;
use std::{path::PathBuf, sync::OnceLock};
use z00z_core::AssetClass;
use z00z_crypto::{create_commitment, Z00ZScalar};
use z00z_utils::{
    codec::{Codec, JsonCodec, Value},
    io::{path_exists, read_file, read_to_string},
};
use z00z_wallets::{
    backup::{decode_tx_history_rows, WalletTxHistoryEntryKind},
    persistence::TxStatus,
    stealth::build_output_bundle as core_build_output_bundle,
    tx::{
        verify_fee_opening_eq as core_fee_opening_eq,
        verify_self_decrypt as core_verify_self_decrypt, TxInputWire, TxPackage,
    },
};

fn actor_history_path(out: &std::path::Path, actor_name: &str) -> std::path::PathBuf {
    let map_path = out.join("wallets/wlt_map.md");
    let map = read_to_string(&map_path).expect("read wallet map");

    for line in map.lines() {
        let cells = line.split('|').map(str::trim).collect::<Vec<_>>();
        if cells.len() != 3 || !cells[0].eq_ignore_ascii_case(actor_name) {
            continue;
        }

        let wlt_path = std::path::PathBuf::from(cells[2]);
        let file_name = wlt_path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("wallet file name");
        let wallet_stem = file_name
            .strip_prefix("wallet_")
            .and_then(|name| name.strip_suffix(".wlt"))
            .expect("canonical wallet file name");

        return out
            .join("wallets")
            .join(format!("wallet_{wallet_stem}_tx_history.jsonl"));
    }

    panic!("missing actor in wallet map: {actor_name}");
}

fn assert_actor_tx_history(
    out: &std::path::Path,
    actor_name: &str,
    pkg: &TxPackage,
    min_physical_rows: usize,
) {
    let history_path = actor_history_path(out, actor_name);
    let noncanonical_dir = history_path.with_extension("");

    assert!(
        path_exists(&history_path).expect("stat tx history"),
        "missing tx history for {actor_name}: {}",
        history_path.display()
    );
    assert!(
        !path_exists(&noncanonical_dir).expect("stat noncanonical history dir"),
        "noncanonical tx history directory exists for {actor_name}: {}",
        noncanonical_dir.display()
    );

    let rows = decode_tx_history_rows(&read_file(&history_path).expect("read tx history"))
        .expect("decode tx history rows");
    assert!(
        rows.len() >= min_physical_rows,
        "{actor_name} tx history row count {} below expected minimum {min_physical_rows}",
        rows.len()
    );
    assert_eq!(rows[0].sequence, 1);
    assert_eq!(rows[0].previous_entry_hash, None);
    for pair in rows.windows(2) {
        assert_eq!(pair[1].sequence, pair[0].sequence + 1);
        assert_eq!(pair[1].previous_entry_hash, Some(pair[0].entry_hash));
    }

    let target_rows = rows
        .iter()
        .filter(|row| row.record.tx_hash == pkg.tx_digest_hex)
        .collect::<Vec<_>>();

    assert_eq!(
        target_rows.len(),
        2,
        "{actor_name} must have one Created+Confirmed pair for the Alice-to-Bob tx"
    );
    assert_eq!(target_rows[0].entry_kind, WalletTxHistoryEntryKind::Created);
    assert_eq!(
        target_rows[1].entry_kind,
        WalletTxHistoryEntryKind::Confirmed
    );

    assert_eq!(target_rows[0].record.status, TxStatus::Pending);
    assert_eq!(target_rows[1].record.status, TxStatus::Confirmed);

    for row in target_rows {
        let decoded: TxPackage = JsonCodec
            .deserialize(&row.record.tx_bytes)
            .expect("decode tx history package bytes");
        assert_eq!(&decoded, pkg);
    }
}

fn stage6_shared_out() -> &'static PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(shared_cases::default_stage6_out)
}

#[test]
fn test_split_fraction_ok() {
    let cfg = mk_cfg("fraction", Some(1.0), None);
    let (send, change) = split_amount_cfg(777, &cfg).expect("fraction=1.0 must pass");
    assert_eq!(send, 777);
    assert_eq!(change, 0);

    let cfg = mk_cfg("fraction", Some(0.1), None);
    let (send, change) = split_amount_cfg(1000, &cfg).expect("fraction=0.1 must pass");
    assert_eq!(send, 100);
    assert_eq!(change, 900);
}

#[test]
fn test_split_fraction_fail() {
    let cfg = mk_cfg("fraction", None, None);
    let err = split_amount_cfg(100, &cfg).expect_err("missing fraction must fail");
    assert_eq!(
        err,
        "stage4: transaction.fraction must be set when mode=fraction"
    );

    let cfg = mk_cfg("fraction", Some(0.0), None);
    let err = split_amount_cfg(100, &cfg).expect_err("zero fraction must fail");
    assert_eq!(err, "stage4: transaction.fraction must be in range (0, 1]");

    let cfg = mk_cfg("fraction", Some(-0.1), None);
    let err = split_amount_cfg(100, &cfg).expect_err("negative fraction must fail");
    assert_eq!(err, "stage4: transaction.fraction must be in range (0, 1]");

    let cfg = mk_cfg("fraction", Some(1.01), None);
    let err = split_amount_cfg(100, &cfg).expect_err("oversized fraction must fail");
    assert_eq!(err, "stage4: transaction.fraction must be in range (0, 1]");
}

#[test]
fn test_split_amount_guard() {
    let cfg = mk_cfg("amount", None, Some(250));
    let (send, change) = split_amount_cfg(1000, &cfg).expect("amount mode must pass");
    assert_eq!(send, 250);
    assert_eq!(change, 750);

    let cfg = mk_cfg("amount", None, None);
    let err = split_amount_cfg(1000, &cfg).expect_err("missing amount must fail");
    assert_eq!(
        err,
        "stage4: transaction.amount must be set when mode=amount"
    );

    let cfg = mk_cfg("amount", None, Some(0));
    let err = split_amount_cfg(1000, &cfg).expect_err("zero amount must fail");
    assert_eq!(err, "stage4: transaction.amount must be > 0");

    let cfg = mk_cfg("amount", None, Some(1001));
    let err = split_amount_cfg(1000, &cfg).expect_err("oversized amount must fail");
    assert_eq!(
        err,
        "stage4: transaction.amount=1001 exceeds input_amount=1000"
    );
}

#[test]
fn test_split_mode_fail() {
    let cfg = mk_cfg("bad", None, None);
    let err = split_amount_cfg(1000, &cfg).expect_err("bad mode must fail");
    assert_eq!(
        err,
        "stage4: transaction.mode must be 'fraction' or 'amount', got 'bad'"
    );
}

#[test]
fn test_stage4_pick_rows() {
    let cfg = pick_cfg();
    let rows = vec![
        mk_pick_wire(7, 50, "Z00Z"),
        mk_pick_wire(8, 40, "Z00Z"),
        mk_pick_wire(9, 30, "Z00Z"),
        mk_pick_wire(10, 99, "OTHER"),
    ];

    let picked = pick_sender_rows(rows, [7u8; 32], &cfg).expect("pick rows");

    assert_eq!(picked.len(), 3);
    let serials: std::collections::BTreeSet<u32> = picked.iter().map(|row| row.serial_id).collect();
    assert_eq!(serials.len(), 3);
    assert!(picked
        .iter()
        .all(|row| row.definition.class == AssetClass::Coin));
    assert!(picked.iter().all(|row| row.definition.symbol == "Z00Z"));
}

#[test]
fn test_stage4_pick_fail() {
    let cfg = pick_cfg();
    let rows = vec![
        mk_pick_wire(7, 50, "Z00Z"),
        mk_pick_wire(7, 40, "Z00Z"),
        mk_pick_wire(7, 30, "Z00Z"),
    ];

    let err = pick_sender_rows(rows, [7u8; 32], &cfg).expect_err("must fail on low serial pool");
    assert!(
        err.contains("distinct serial_id requirement not satisfied"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_stage4_pick_skips_plain() {
    let cfg = pick_cfg();
    let rows = vec![
        mk_wire(7, 50, "Z00Z"),
        mk_wire(8, 40, "Z00Z"),
        mk_wire(9, 30, "Z00Z"),
        mk_pick_wire(10, 25, "Z00Z"),
        mk_pick_wire(11, 20, "Z00Z"),
        mk_pick_wire(12, 15, "Z00Z"),
    ];

    let picked = pick_sender_rows(rows, [7u8; 32], &cfg).expect("pick rows");

    assert_eq!(picked.len(), 3);
    assert!(picked.iter().all(|row| row.r_pub.is_some()));
    assert!(picked.iter().all(|row| row.owner_tag.is_some()));
    assert!(picked.iter().all(|row| row.enc_pack.is_some()));
    assert!(picked.iter().all(|row| row.tag16.is_some()));
}

#[test]
fn test_fee_gate_rejects_mismatch() {
    let mut blind_bytes = [0u8; 32];
    blind_bytes[0] = 7;
    let fee_blind = Z00ZScalar::try_from_bytes(blind_bytes).expect("scalar");

    let fee_commit = create_commitment(17, &fee_blind).expect("fee commitment");

    assert!(core_fee_opening_eq(&fee_commit, 17, &fee_blind).is_ok());
    assert!(core_fee_opening_eq(&fee_commit, 18, &fee_blind).is_err());
}

#[test]
fn test_output_self_decrypt_ok() {
    let out = mk_out();
    assert!(core_verify_self_decrypt(&out).is_ok());
}

#[test]
fn test_tx_out_match_party() {
    let bob = core_build_output_bundle(
        "bob".to_string(),
        z00z_wallets::tx::TxOutRole::Recipient,
        AssetClass::Coin,
        &mk_card(7),
        55,
        3,
    )
    .expect("bob output");
    let alice = core_build_output_bundle(
        "alice".to_string(),
        z00z_wallets::tx::TxOutRole::Change,
        AssetClass::Coin,
        &mk_card(8),
        21,
        4,
    )
    .expect("alice output");

    let tx_outputs = to_tx_output_wires(&[bob, alice]).expect("tx outputs");

    assert_eq!(tx_outputs[0].role, z00z_wallets::tx::TxOutRole::Recipient);
    assert_eq!(tx_outputs[1].role, z00z_wallets::tx::TxOutRole::Change);
}

#[test]
fn test_output_tag_fail() {
    let mut out = mk_out();
    out.leaf.tag16 ^= 1;

    let err = decode_output_pack(&out).expect_err("tag tamper must fail");
    assert!(err.contains("tag16 mismatch"), "unexpected error: {err}");
}

#[test]
fn test_output_value_fail() {
    let mut out = mk_out();
    out.value += 1;

    let err = decode_output_pack(&out).expect_err("value tamper must fail");
    assert!(err.contains("value mismatch"), "unexpected error: {err}");
}

#[test]
fn test_output_commit_fail() {
    let mut out = mk_out();
    let pack = decode_output_pack(&out).expect("decode pack");
    let mut other_blind = [0u8; 32];
    other_blind[0] = 9;
    let blind = Z00ZScalar::try_from_bytes(other_blind).expect("blinding scalar");
    let commit = create_commitment(pack.value, &blind).expect("other commitment");
    out.leaf.c_amount = commit.as_bytes().try_into().expect("commit bytes");
    let leaf_ad = super::compute_leaf_ad(
        &out.leaf.asset_id,
        out.leaf.serial_id,
        &out.leaf.r_pub,
        &out.leaf.owner_tag,
        &out.leaf.c_amount,
    );
    out.leaf.tag16 = super::compute_tag16(&out.k_dh, &leaf_ad);
    out.leaf.enc_pack = super::ZkPack::encrypt(
        &out.k_dh,
        &leaf_ad,
        &out.leaf.r_pub,
        &out.leaf.asset_id,
        out.leaf.serial_id,
        &pack.to_bytes(),
    );

    let err = core_verify_self_decrypt(&out).expect_err("commit tamper must fail");
    assert!(
        err.contains("commitment mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_output_range_fail() {
    let mut out = mk_out();
    assert!(!out.leaf.range_proof.is_empty(), "range proof must exist");
    out.leaf.range_proof[0] ^= 1;

    let err = core_verify_self_decrypt(&out).expect_err("range tamper must fail");
    assert!(
        err.contains("range proof verify failed"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_balance_gate_ok() {
    let recv_sec = [7u8; 32];
    let outputs = vec![mk_out()];
    let inputs = vec![input_from_out(&outputs[0])];

    assert!(verify_commitment_balance_gate(recv_sec, &inputs, &outputs, 0).is_ok());
}

#[test]
fn test_balance_gate_fail() {
    let recv_sec = [7u8; 32];
    let outputs = vec![mk_out()];
    let mut inputs = vec![input_from_out(&outputs[0])];
    inputs[0].commitment =
        create_commitment(1, &Z00ZScalar::try_from_bytes([3u8; 32]).expect("scalar"))
            .expect("commit");

    let err = verify_commitment_balance_gate(recv_sec, &inputs, &outputs, 0)
        .expect_err("bad fee must fail");
    assert!(err.contains("balance"), "unexpected error: {err}");
}

#[test]
fn test_witness_gate_ok() {
    let recv_sec = [7u8; 32];
    let input_out = mk_out_with_serial(3);
    let output_out = mk_balanced_out_from_input(&input_out, 4);
    let outputs = vec![output_out];
    let inputs = vec![input_from_out(&input_out)];
    let row = prep_leaf(
        &inputs[0],
        &TxInputWire {
            asset_id_hex: input_asset_id_hex(&inputs[0]),
            serial_id: inputs[0].serial_id,
        },
    )
    .expect("prep row");
    let prev_root = prep_root(&[row]).expect("typed root");

    let result = verify_spend_witness_gate(3, recv_sec, &inputs, &outputs, prev_root);
    assert!(result.is_ok(), "unexpected error: {:?}", result.err());
}

#[test]
fn test_witness_gate_fail() {
    let recv_sec = [7u8; 32];
    let outputs = vec![mk_out()];
    let inputs = vec![input_from_out(&outputs[0])];

    let err = verify_spend_witness_gate(
        3,
        recv_sec,
        &inputs,
        &outputs,
        z00z_storage::settlement::CheckRoot::new([0u8; 32]),
    )
    .expect_err("zero root must fail");
    assert!(
        err.contains("membership root mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_tx_validation_nullifier_drift() {
    let out = stage6_shared_out();
    let pkg_path = out.join("transactions/tx_alice_to_bob_pkg.json");
    let pkg_text = read_to_string(&pkg_path).expect("read tx package");
    let mut pkg: TxPackage = JsonCodec
        .deserialize(pkg_text.as_bytes())
        .expect("decode tx package");

    assert_actor_tx_history(out, "alice", &pkg, 2);
    assert_actor_tx_history(out, "bob", &pkg, 2);

    pkg.tx.proof.spend.as_mut().expect("spend proof").inputs[0].nullifier_hex =
        hex::encode([0xAB; 32]);
    pkg.tx_digest_hex = z00z_wallets::tx::build_tx_package_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .expect("rebuild tx digest");

    let tx_bytes = JsonCodec.serialize(&pkg).expect("serialize tx package");
    let err = verify_tx_package(&tx_bytes)
        .expect_err("signed nullifier drift must fail serialized tx package validation");

    assert!(
        err.contains(
            "public spend contract failed: carried spend statement mismatches recomputed statement"
        ),
        "unexpected error: {err}"
    );
}

#[test]
fn test_tx_validation_chain_id() {
    let out = stage6_shared_out();
    let pkg_path = out.join("transactions/tx_alice_to_bob_pkg.json");
    let pkg_text = read_to_string(&pkg_path).expect("read tx package");
    let mut pkg_value: Value = JsonCodec
        .deserialize(pkg_text.as_bytes())
        .expect("decode tx package json");
    pkg_value
        .as_object_mut()
        .expect("tx package object")
        .remove("chain_id");
    let tx_bytes = JsonCodec
        .serialize(&pkg_value)
        .expect("serialize tx package json");

    let err = verify_tx_package(&tx_bytes)
        .expect_err("missing chain_id must fail serialized tx package validation");

    assert!(
        err.contains("decode tx package failed"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_canonical_snapshot_store_order() {
    let out_a = mk_out();
    let mut out_b = mk_out();
    out_b.leaf.serial_id = 19;
    out_b.leaf.asset_id[0] ^= 1;

    let wire_a = input_from_out(&out_a);
    let wire_b = input_from_out(&out_b);
    let input_a = TxInputWire {
        asset_id_hex: input_asset_id_hex(&wire_a),
        serial_id: wire_a.serial_id,
    };
    let input_b = TxInputWire {
        asset_id_hex: input_asset_id_hex(&wire_b),
        serial_id: wire_b.serial_id,
    };

    let claim_store = prep_store(&[
        prep_leaf(&wire_b, &input_b).expect("prep row b"),
        prep_leaf(&wire_a, &input_a).expect("prep row a"),
    ])
    .expect("claim store");

    let prep = build_prep_file(
        &claim_store,
        &[wire_b.clone(), wire_a.clone()],
        &[input_b.clone(), input_a.clone()],
    )
    .expect("prep file");
    let (snapshot, _) = build_canon_snapshot(&prep, &claim_store).expect("canonical snapshot");

    assert_eq!(prep.rows.len(), 2);
    assert_eq!(prep.rows[0].serial_id, input_b.serial_id);
    assert_eq!(prep.rows[1].serial_id, input_a.serial_id);

    assert_eq!(snapshot.entries.len(), 2);
    let snapshot_serials = snapshot
        .entries
        .iter()
        .map(|entry| entry.path().serial_id.get())
        .collect::<Vec<_>>();
    assert_eq!(snapshot_serials, vec![input_a.serial_id, input_b.serial_id]);
}
