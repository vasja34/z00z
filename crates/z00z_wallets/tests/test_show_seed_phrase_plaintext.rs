#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use base64::Engine as _;
use redb::ReadableDatabase;
use z00z_crypto::aead;
use z00z_crypto::expert::{encoding::SafePassword, traits::DomainSeparation};
use z00z_utils::codec::{BincodeCodec, Codec};
use z00z_utils::compression::zstd_decompress_bounded;
use z00z_utils::io;
use z00z_wallets::services::{AppService, WalletService};

#[tokio::test]
async fn test_wallet_show_phrase_plaintext() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");

    let wallets = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&wallets));

    let password = "Aa1!bB2@cC3#dD4$eE5%";

    let created = app
        .create_wallet("wallet-seed-plain".to_string(), password.to_string(), None)
        .await
        .expect("create wallet");

    // Clear in-memory state to ensure show_seed_phrase hits disk.
    wallets
        .unregister_wallet(&created.wallet_id)
        .await
        .expect("unregister");

    let safe_password = SafePassword::from(password);
    let session = wallets
        .unlock_wallet_in_memory(&created.wallet_id, &safe_password)
        .await
        .expect("unlock must succeed");

    let shown = wallets
        .show_seed_phrase(
            &session,
            SafePassword::from(password),
            "I understand".to_string(),
        )
        .await
        .expect("show_seed_phrase must succeed");

    let exported = wallets
        .export_wallet_payload(&created.wallet_id, &SafePassword::from(password))
        .await
        .expect("export wallet");

    // Encrypted-only response must decrypt back to the original seed phrase.
    let nonce_hex = shown
        .encrypted_payload
        .metadata
        .nonce
        .strip_prefix("0x")
        .unwrap_or(&shown.encrypted_payload.metadata.nonce);
    let nonce_bytes = hex::decode(nonce_hex).expect("nonce hex");
    assert_eq!(
        nonce_bytes.len(),
        z00z_crypto::aead::XCHACHA_NONCE_SIZE,
        "nonce must be 24 bytes"
    );
    let mut nonce = [0u8; z00z_crypto::aead::XCHACHA_NONCE_SIZE];
    nonce.copy_from_slice(&nonce_bytes);

    let payload = base64::engine::general_purpose::STANDARD
        .decode(exported.ciphertext.as_bytes())
        .expect("payload base64");
    let magic = b"z00z-wexp\0";
    assert!(payload.starts_with(magic), "export payload must be framed");
    let container: z00z_wallets::security::encryption::EncryptedWalletContainer = BincodeCodec
        .deserialize(&payload[magic.len() + 4..])
        .expect("container decode");
    let context = [z00z_wallets::key::Z00ZKeyBranch::WalletBackup.as_aad_byte()];
    let export_aad = aead::build_aad_multipart(
        z00z_wallets::domains::AeadEnvelopeDomain::domain(),
        &[&context[..]],
    )
    .expect("export aad");
    let plaintext = z00z_wallets::security::encryption::WalletEncryption::decrypt_wallet(
        &SafePassword::from(password),
        &export_aad,
        &container,
    )
    .expect("decrypt export");
    let export_pack: z00z_wallets::wallet::persistence::WalletExportPack = BincodeCodec
        .deserialize(plaintext.as_ref())
        .expect("export pack decode");
    let salt = export_pack
        .wallet_profile
        .as_ref()
        .and_then(|profile| profile.seed_salt)
        .expect("seed salt present");
    let aad = z00z_crypto::aead::build_aad_multipart(
        "wallet.seed_phrase_response",
        &[created.wallet_id.0.as_bytes()],
    )
    .expect("aad");
    let mut key = z00z_wallets::security::encryption::WalletEncryption::derive_key(
        &SafePassword::from(password),
        &salt,
    )
    .expect("derive key");

    let mut envelope = Vec::new();
    envelope.push(z00z_crypto::aead::XCHACHA20_POLY1305_ID);
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(
        &hex::decode(&shown.encrypted_payload.ciphertext).expect("ciphertext hex"),
    );
    let recovered = z00z_crypto::aead::open(&key, &aad, &envelope).expect("decrypt");
    key.fill(0);

    let recovered_phrase = String::from_utf8(recovered).expect("utf8");
    assert_eq!(recovered_phrase, created.seed_phrase);

    // Encrypted payload must not include plaintext seed phrase.
    assert!(
        !shown
            .encrypted_payload
            .ciphertext
            .contains(&created.seed_phrase),
        "ciphertext must not contain plaintext seed phrase"
    );
    assert!(
        !format!("{:?}", shown.encrypted_payload.metadata).contains(&created.seed_phrase),
        "metadata must not contain plaintext seed phrase"
    );

    // Verify the persistent "revealed_at" marker is written to the encrypted secrets store.
    let wlt_path = {
        use z00z_wallets::domains::hashing::compute_wallet_file_id;
        let hash = compute_wallet_file_id(&created.wallet_id.0);
        let wallet_id_hex = hex::encode(&hash[..8]);
        output_dir.join(format!("wallet_{wallet_id_hex}.wlt"))
    };
    {
        const MAX_DECOMPRESSED_WLT_BYTES: usize = 128 * 1024 * 1024;

        let zstd = io::read_file(&wlt_path).expect("read wlt");
        let db_bytes =
            zstd_decompress_bounded(&zstd, MAX_DECOMPRESSED_WLT_BYTES).expect("decompress wlt");

        let work_path = tempfile::Builder::new()
            .prefix("z00z_wallet_seed_check_")
            .suffix(".wlt.work")
            .tempfile_in("/dev/shm")
            .expect("tempfile_in /dev/shm")
            .into_temp_path();
        let work_path_buf = work_path.to_path_buf();
        io::atomic_write_file_private(&work_path_buf, &db_bytes).expect("write work wlt");

        let db = redb::Database::open(&work_path_buf).expect("open redb");
        let read_txn = db.begin_read().expect("read txn");
        let secrets = read_txn
            .open_table(redb::TableDefinition::<&str, &[u8]>::new("secrets"))
            .expect("open secrets");

        assert!(
            secrets
                .get("seed_main.revealed_at")
                .expect("read marker")
                .is_some(),
            "expected seed_main.revealed_at marker to exist"
        );
    }

    // Second call must succeed (policy no longer enforces show-once).
    let safe_password = SafePassword::from(password);
    let session = wallets
        .unlock_wallet_in_memory(&created.wallet_id, &safe_password)
        .await
        .expect("unlock must succeed");

    let shown2 = wallets
        .show_seed_phrase(
            &session,
            SafePassword::from(password),
            "I understand".to_string(),
        )
        .await
        .expect("second show_seed_phrase must succeed");

    assert!(shown2.encrypted_payload.is_encrypted());

    let _ = wlt_path;
}
