#![cfg(not(target_arch = "wasm32"))]

use z00z_core::{genesis::asset_std::asset_from_dev_cfg, Asset};
use z00z_crypto::{domains::AssetIdDomain, hash_zk::hash_zk};
use z00z_wallets::{
    build_tx_output_unchecked,
    key::receiver_keys::derive_rotated_view_secret_key,
    key::{derive_view_secret_key, ReceiverKeys, ReceiverSecret},
    receiver::{
        ReceiveStatus, ScanResult, StealthOutputScanner, WalletReveal, WalletStealthOutput,
    },
    tx::{derive_spend_nullifier, verify_spend_rules, SpendIn, SpendRuleErr, SpendStmt},
    SenderWallet,
};

fn mk_keys(seed: u8) -> ReceiverKeys {
    let mut bytes = [seed; 32];
    bytes[31] ^= 0x5A;
    let secret = ReceiverSecret::from_bytes(bytes).expect("receiver secret");
    ReceiverKeys::from_receiver_secret(secret).expect("receiver keys")
}

fn mk_secret(seed: u8) -> ReceiverSecret {
    let mut bytes = [seed; 32];
    bytes[31] ^= 0x5A;
    ReceiverSecret::from_bytes(bytes).expect("receiver secret")
}

fn mk_asset(amount: u64, out: &z00z_wallets::TxStealthOutput) -> Asset {
    let mut asset = asset_from_dev_cfg("z00z", 0, amount).expect("asset");
    asset.commitment = z00z_crypto::Commitment::from_bytes(&out.c_amount)
        .expect("commitment")
        .0;
    asset.owner_pub = None;
    asset.owner_signature = None;
    asset.r_pub = Some(out.r_pub);
    asset.owner_tag = Some(out.owner_tag);
    asset.enc_pack = Some(out.enc_pack.clone());
    asset.tag16 = out.tag16;
    asset.leaf_ad_id = Some(asset.definition.id);
    asset
}

fn mk_output(keys: &ReceiverKeys, seed: u8, amount: u64) -> z00z_wallets::TxStealthOutput {
    let card = keys.export_receiver_card().expect("card");
    let mut sender = SenderWallet::new([seed; 32]);
    let aid = asset_from_dev_cfg("z00z", 0, amount)
        .expect("asset")
        .definition
        .id;
    build_tx_output_unchecked(
        &card,
        None,
        &mut sender,
        &[seed ^ 0x3C; 32],
        0,
        amount,
        &aid,
    )
    .expect("stealth output")
}

fn mk_wallet_output(
    asset_secret: WalletReveal<[u8; 32]>,
    blinding: WalletReveal<[u8; 32]>,
) -> WalletStealthOutput {
    WalletStealthOutput {
        asset_id: [0xAA; 32],
        serial_id: 7,
        pack_version: z00z_core::assets::AssetPackVersion::Memo,
        amount: 99,
        asset_secret,
        blinding,
        memo: Some(b"SECRET_MEMO".to_vec()),
        r_pub: [0xBB; 32],
        owner_tag: [0xCC; 32],
        decrypted_at: 42,
    }
}

fn mk_spend_stmt(secret: ReceiverSecret, out: &z00z_wallets::TxStealthOutput) -> SpendStmt {
    let c_in = z00z_crypto::Commitment::from_bytes(&out.c_amount)
        .expect("commitment")
        .0;
    let s_in = [0xA5; 32];
    let asset_id_in = hash_zk::<AssetIdDomain>("", &[&s_in]);
    let nullifier_in = derive_spend_nullifier(3, &s_in);

    SpendStmt {
        receiver_secret: secret,
        spend_ins: vec![SpendIn {
            chain_id: 3,
            r_pub_in: out.r_pub,
            owner_tag_in: out.owner_tag,
            leaf_ad_id_in: asset_id_in,
            nullifier_in: Some(nullifier_in),
            s_in,
            c_in: c_in.clone(),
        }],
        c_outs: vec![c_in],
        range_ok: true,
    }
}

#[test]
fn test_live_view_path() {
    let secret_bytes = [0x11; 32];
    let secret = ReceiverSecret::from_bytes(secret_bytes).expect("receiver secret");
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(secret_bytes).expect("receiver secret"),
    )
    .expect("receiver keys");
    let live = derive_view_secret_key(&secret).expect("live view key");
    let rotated = derive_rotated_view_secret_key(&secret, 1).expect("rotated view key");

    assert_eq!(keys.reveal_view_sk().as_bytes(), live.as_bytes());
    assert_ne!(live.as_bytes(), rotated.as_bytes());
}

#[test]
fn test_live_scan_spend() {
    let keys = mk_keys(0x21);
    let secret = mk_secret(0x21);
    let out = mk_output(&keys, 0x33, 7_000);
    let asset = mk_asset(7_000, &out);

    let scan = StealthOutputScanner::from_keys(&keys).scan_leaf(&asset);
    assert!(matches!(scan, ScanResult::Mine { .. }));

    let stmt = mk_spend_stmt(secret, &out);
    verify_spend_rules(&stmt).expect("spend rules");
}

#[test]
fn test_rotation_is_explicit() {
    let mut rotated_keys = mk_keys(0x44);
    let live_keys = mk_keys(0x44);
    let secret = mk_secret(0x44);
    let rotated = derive_rotated_view_secret_key(&secret, 1).expect("rotated view key");

    rotated_keys.rotate_view().expect("rotate view");
    assert_eq!(rotated_keys.reveal_view_sk().as_bytes(), rotated.as_bytes());
    assert_eq!(
        live_keys.reveal_view_sk().as_bytes(),
        derive_view_secret_key(&secret).expect("live").as_bytes()
    );

    let out = mk_output(&rotated_keys, 0x66, 9_000);
    let asset = mk_asset(9_000, &out);

    let live_scan = StealthOutputScanner::from_keys(&live_keys).scan_leaf(&asset);
    assert!(matches!(live_scan, ScanResult::NotMine));

    let live_stmt = mk_spend_stmt(secret, &out);
    let live_spend = verify_spend_rules(&live_stmt);
    assert!(matches!(live_spend, Err(SpendRuleErr::BadOwnerTag { .. })));

    let rotated_scan = StealthOutputScanner::from_keys(&rotated_keys).scan_leaf(&asset);
    assert!(matches!(rotated_scan, ScanResult::Mine { .. }));
}

#[test]
fn test_hot_path_guard() {
    let output_src = include_str!("../src/stealth/output.rs");
    let spending_src = include_str!("../src/tx/spend_events.rs");

    assert!(!output_src.contains("derive_rotated_view_secret_key"));
    assert!(!spending_src.contains("derive_rotated_view_secret_key"));
}

// TASK-042 anchor: test_wallet_reveal_matrix_public_surfaces
#[test]
fn test_reveal_surface_matrix() {
    let cases = [
        (
            "present",
            WalletReveal::present([0x11; 32]),
            WalletReveal::present([0x22; 32]),
            "Present(<redacted>)",
        ),
        (
            "redacted",
            WalletReveal::redacted(),
            WalletReveal::redacted(),
            "Redacted",
        ),
        (
            "unavailable",
            WalletReveal::unavailable(),
            WalletReveal::unavailable(),
            "Unavailable",
        ),
    ];

    let asset_secret_hex = hex::encode([0x11; 32]);
    let blinding_hex = hex::encode([0x22; 32]);

    for (label, asset_secret, blinding, marker) in cases {
        let output = mk_wallet_output(asset_secret, blinding);
        let scan = ScanResult::Mine {
            wallet_output: Box::new(output.clone()),
        };

        assert_eq!(
            scan.recv_report().status,
            ReceiveStatus::Detected,
            "{label}"
        );
        let rendered = format!("{output:?}");
        assert!(rendered.contains(marker), "{label}");
        assert!(!rendered.contains("SECRET_MEMO"), "{label}");
        assert!(!rendered.contains(&asset_secret_hex), "{label}");
        assert!(!rendered.contains(&blinding_hex), "{label}");

        let scan_rendered = format!("{scan:?}");
        assert!(!scan_rendered.contains("SECRET_MEMO"), "{label}");
        assert!(!scan_rendered.contains(&asset_secret_hex), "{label}");
        assert!(!scan_rendered.contains(&blinding_hex), "{label}");
    }
}
