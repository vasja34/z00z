#[path = "test_inc/test_range_proof_env.inc"]
mod test_common;

use test_common::RangeProofEnvGuard;
use z00z_core::{
    assets::AssetPkgWire,
    genesis::asset_std::{asset_from_dev_class, serials_from_dev_class},
    AssetClass, AssetWire,
};
use z00z_storage::settlement::TerminalLeaf;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{
        receiver_scan_leaf, receiver_scan_report, ReceiveNext, ReceiveReject, ReceiveReport,
        ReceiveStatus, ReceiverCard, ScanResult, StealthOutputScanner,
    },
    stealth::{
        bind_stealth_output_wire, build_card_stealth_leaf,
        ecdh::compute_dh_receiver,
        kdf::{compute_leaf_ad, derive_k_dh},
        zkpack::ZkPack,
    },
    tx::{asset_wire_to_leaf, wire_decrypt_leaf},
    WalletError,
};

struct GoodCase {
    keys: ReceiverKeys,
    leaf: TerminalLeaf,
    amount: u64,
}

fn make_keys() -> ReceiverKeys {
    let sec = ReceiverSecret::generate().expect("secret");
    ReceiverKeys::from_receiver_secret(sec).expect("keys")
}

fn make_card(keys: &ReceiverKeys) -> ReceiverCard {
    let card = keys.export_receiver_card().expect("card");
    card.verify().expect("verify");
    card
}

fn make_case() -> GoodCase {
    let _guard = RangeProofEnvGuard::new();
    let keys = make_keys();
    let card = make_card(&keys);
    let amount = 777u64;
    let serial_id = serials_from_dev_class(AssetClass::Coin).expect("dev coin serials") - 1;
    let leaf = build_card_stealth_leaf(&card, amount, serial_id).expect("leaf");
    GoodCase { keys, leaf, amount }
}

fn make_dto(leaf: &TerminalLeaf, amount: u64) -> AssetPkgWire {
    let asset = asset_from_dev_class(AssetClass::Coin, leaf.serial_id, amount).expect("std asset");
    let mut wire =
        bind_stealth_output_wire(AssetWire::from_asset(&asset), leaf).expect("bind wire");
    wire.amount = amount;
    AssetPkgWire::from_wire(&wire)
}

fn hydrate_pair(
    leaf: &TerminalLeaf,
    amount: u64,
) -> (TerminalLeaf, TerminalLeaf, z00z_core::Asset) {
    let dto = make_dto(leaf, amount);
    let wire = dto.clone().to_wire().expect("wire");
    let canon = asset_wire_to_leaf(&wire).expect("canon");
    let runtime_leaf = wire_decrypt_leaf(&wire).expect("runtime leaf");
    let runtime = dto.to_asset().expect("runtime");
    assert_eq!(runtime_leaf, leaf.clone());
    (canon, runtime_leaf, runtime)
}

fn leaf_key(keys: &ReceiverKeys, leaf: &TerminalLeaf) -> [u8; 32] {
    let r_pub = z00z_wallets::stealth::ecdh::decode_r_pub(&leaf.r_pub).expect("r_pub");
    let dh = compute_dh_receiver(keys.reveal_view_sk(), &r_pub).expect("dh");
    derive_k_dh(&dh)
}

fn alt_commit(keys: &ReceiverKeys, leaf: &TerminalLeaf) -> [u8; 32] {
    let pack = receiver_scan_leaf(keys, leaf)
        .expect("scan")
        .expect("owned pack");

    for bit in 0u8..=u8::MAX {
        let mut bytes = pack.blinding;
        bytes[31] ^= bit;
        if bytes == pack.blinding {
            continue;
        }

        let Ok(blind) = z00z_crypto::Z00ZScalar::try_from_bytes(bytes) else {
            continue;
        };

        let out: [u8; 32] = z00z_crypto::create_commitment(pack.value, &blind)
            .expect("commitment")
            .as_bytes()
            .try_into()
            .expect("commitment bytes");
        if out != leaf.c_amount {
            return out;
        }
    }

    panic!("alt commitment");
}

fn bad_enc_leaf(leaf: &TerminalLeaf) -> TerminalLeaf {
    let mut bad = leaf.clone();
    bad.enc_pack.ciphertext[0] ^= 1;
    bad
}

fn bad_pack_leaf(keys: &ReceiverKeys, leaf: &TerminalLeaf) -> TerminalLeaf {
    let mut bad = leaf.clone();
    let mut pack = receiver_scan_leaf(keys, leaf)
        .expect("scan")
        .expect("owned pack");
    let k_dh = leaf_key(keys, leaf);
    let leaf_ad = compute_leaf_ad(
        &leaf.asset_id,
        leaf.serial_id,
        &leaf.r_pub,
        &leaf.owner_tag,
        &leaf.c_amount,
    );

    pack.value = pack.value.saturating_add(1);
    let bytes = pack.to_bytes().expect("pack bytes");
    bad.enc_pack = ZkPack::encrypt(
        &k_dh,
        &leaf_ad,
        &leaf.r_pub,
        &leaf.asset_id,
        leaf.serial_id,
        &bytes,
    );
    bad
}

fn bad_wrap(leaf: &TerminalLeaf, amount: u64) -> z00z_core::Asset {
    let mut runtime = make_dto(leaf, amount).to_asset().expect("runtime");
    let definition = z00z_core::AssetDefinition::new(
        [0u8; 32],
        runtime.definition.class,
        format!("{}-wrap", runtime.definition.name),
        runtime.definition.symbol.clone(),
        runtime.definition.decimals,
        runtime.definition.serials,
        runtime.definition.nominal,
        runtime.definition.domain_name.clone(),
        runtime.definition.version,
        runtime.definition.crypto_version,
        runtime.definition.policy_flags,
        runtime.definition.metadata.clone(),
    )
    .expect("canonical wrapped definition");
    runtime.definition = std::sync::Arc::new(definition);
    runtime
}

fn bad_report(status: ReceiveStatus, reject: ReceiveReject) -> ReceiveReport {
    ReceiveReport {
        status,
        reject: Some(reject),
        next: ReceiveNext::ReportOnly,
    }
}

fn check_bad_leaf(
    scanner: &StealthOutputScanner,
    keys: &ReceiverKeys,
    leaf: &TerminalLeaf,
    amount: u64,
    want: ReceiveReport,
) {
    let pack = receiver_scan_leaf(keys, leaf).expect("scan");
    assert!(pack.is_none());

    let report = receiver_scan_report(keys, leaf).expect("report");
    assert_eq!(report, want);

    let dto = make_dto(leaf, amount);
    let wire = dto.clone().to_wire().expect("wire");
    let canon = asset_wire_to_leaf(&wire).expect("canon");
    let runtime_leaf = wire_decrypt_leaf(&wire).expect("runtime leaf");

    assert_eq!(runtime_leaf, leaf.clone());
    assert_eq!(canon.owner_tag, leaf.owner_tag);

    match dto.to_asset() {
        Ok(runtime) => {
            let scan = scanner.scan_leaf(&runtime);
            assert!(!matches!(scan, ScanResult::Mine { .. }));
            assert_eq!(scan.recv_report(), want);
        }
        Err(err) => {
            assert_eq!(want.status, ReceiveStatus::InvalidProof);
            assert!(
                err.to_string().contains("Range proof verification failed"),
                "unexpected runtime hydration error: {err}"
            );
        }
    }
}

#[test]
fn test_s5_spec6_bridge_ok() {
    let case = make_case();
    let (canon, runtime_leaf, runtime) = hydrate_pair(&case.leaf, case.amount);
    let scanner = StealthOutputScanner::from_keys(&case.keys);

    let pack = receiver_scan_leaf(&case.keys, &runtime_leaf)
        .expect("scan")
        .expect("owned pack");
    let report = receiver_scan_report(&case.keys, &runtime_leaf).expect("report");
    let scan = scanner.scan_leaf(&runtime);
    let runtime_report = scan.recv_report();

    assert_eq!(report.status, ReceiveStatus::Detected);
    assert_eq!(report.next, ReceiveNext::ReportOnly);
    assert_eq!(report, runtime_report);
    assert_eq!(pack.value, case.amount);
    assert_eq!(runtime_leaf.asset_id, case.leaf.asset_id);
    assert_ne!(canon.asset_id, runtime_leaf.asset_id);
    assert_eq!(canon.owner_tag, case.leaf.owner_tag);

    let ScanResult::Mine { wallet_output } = scan else {
        panic!("expected Mine, got {scan:?}");
    };

    assert_eq!(wallet_output.amount, case.amount);
    assert_eq!(wallet_output.asset_id, runtime.asset_id());
    assert_eq!(wallet_output.asset_id, canon.asset_id);
    assert_ne!(wallet_output.asset_id, case.leaf.asset_id);
    assert_eq!(wallet_output.serial_id, case.leaf.serial_id);
    assert_eq!(wallet_output.asset_secret, Some(pack.s_out));
    assert_eq!(wallet_output.blinding, Some(pack.blinding));
    assert_eq!(wallet_output.r_pub, case.leaf.r_pub);
    assert_eq!(wallet_output.owner_tag, case.leaf.owner_tag);
}

#[test]
fn test_s5_spec6_bridge_rejects() {
    let case = make_case();
    let scanner = StealthOutputScanner::from_keys(&case.keys);
    let good = receiver_scan_report(&case.keys, &case.leaf).expect("good report");
    assert_eq!(good.status, ReceiveStatus::Detected);

    let mut bad_owner = case.leaf.clone();
    bad_owner.owner_tag[0] ^= 1;
    check_bad_leaf(
        &scanner,
        &case.keys,
        &bad_owner,
        case.amount,
        bad_report(ReceiveStatus::NotMine, ReceiveReject::NotMine),
    );

    let mut bad_tag = case.leaf.clone();
    bad_tag.tag16 ^= 1;
    check_bad_leaf(
        &scanner,
        &case.keys,
        &bad_tag,
        case.amount,
        bad_report(ReceiveStatus::InvalidProof, ReceiveReject::InvalidProof),
    );

    let bad_enc = bad_enc_leaf(&case.leaf);
    check_bad_leaf(
        &scanner,
        &case.keys,
        &bad_enc,
        case.amount,
        bad_report(ReceiveStatus::InvalidProof, ReceiveReject::InvalidProof),
    );

    let mut bad_amt = case.leaf.clone();
    bad_amt.c_amount = alt_commit(&case.keys, &case.leaf);
    check_bad_leaf(
        &scanner,
        &case.keys,
        &bad_amt,
        case.amount,
        bad_report(ReceiveStatus::InvalidProof, ReceiveReject::InvalidProof),
    );

    let bad_pack = bad_pack_leaf(&case.keys, &case.leaf);
    assert!(matches!(
        receiver_scan_leaf(&case.keys, &bad_pack),
        Err(WalletError::CommitmentMismatch)
    ));
    assert!(matches!(
        receiver_scan_report(&case.keys, &bad_pack),
        Err(WalletError::CommitmentMismatch)
    ));
    let (_, _, bad_runtime) = hydrate_pair(&bad_pack, case.amount);
    let pack_scan = scanner.scan_leaf(&bad_runtime);
    assert!(!matches!(pack_scan, ScanResult::Mine { .. }));
    assert_eq!(
        pack_scan.recv_report(),
        bad_report(ReceiveStatus::InvalidProof, ReceiveReject::InvalidProof)
    );

    let wrap_runtime = bad_wrap(&case.leaf, case.amount);
    let wrap_scan = scanner.scan_leaf(&wrap_runtime);
    let ScanResult::Mine { wallet_output } = wrap_scan else {
        panic!("expected Mine after definition-id tamper, got {wrap_scan:?}");
    };
    assert_eq!(wallet_output.amount, case.amount);
    assert_eq!(wallet_output.asset_id, wrap_runtime.asset_id());
    assert_ne!(wallet_output.asset_id, case.leaf.asset_id);
}
