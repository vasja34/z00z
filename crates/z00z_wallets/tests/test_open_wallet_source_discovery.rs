#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::DomainHasher;
use z00z_utils::codec::{BincodeCodec, Codec};
use z00z_utils::io;
use z00z_utils::rng::SystemRngProvider;
use z00z_wallets::db::{
    create_wallet_store, open_wallet_store, write_wallet_profile, WalletIdentity,
};
use z00z_wallets::domains::WalletPasswordVerifierDomain;
use z00z_wallets::rpc::types::common::PersistWalletId;
use z00z_wallets::rpc::types::wallet::{PersistWalletSettings, WalletSource};
use z00z_wallets::services::WalletService;
use z00z_wallets::wallet::persistence::{PasswordVerifierState, ReceiverDeriverState};
use z00z_wallets::wallet::WalletState;
use z00z_wallets::WalletError;

#[path = "test_inc/test_wallet_env.inc"]
mod test_common;

fn default_identity() -> WalletIdentity {
    WalletIdentity {
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
    }
}

fn password_verifier_state(password: &SafePassword) -> PasswordVerifierState {
    let salt = [7u8; 32];
    let hash = DomainHasher::<WalletPasswordVerifierDomain>::new_with_label("wallet_password")
        .chain(salt)
        .chain(password.reveal().as_slice())
        .finalize();

    let mut verifier = [0u8; 32];
    verifier.copy_from_slice(&hash.as_ref()[..32]);

    PasswordVerifierState { salt, verifier }
}

fn create_test_wallet_with_identity(path: &Path, wallet_id: &str, identity: WalletIdentity) {
    let rng = SystemRngProvider;
    let wallet_id = PersistWalletId(wallet_id.to_string());
    let password = SafePassword::from("pw1");
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    create_wallet_store(path, &wallet_id, &password, seed_phrase, &identity, rng).unwrap();
    let profile = z00z_wallets::db::WalletProfilePayload::new_with_checksum(
        wallet_id.clone(),
        "wallet_test".to_string(),
        0,
        0,
        password_verifier_state(&password),
        ReceiverDeriverState {
            next_payment_index: 0,
            next_change_index: 0,
        },
        PersistWalletSettings {
            auto_lock_timeout: 0,
            default_fee: "0".to_string(),
            currency_display: "Z00Z".to_string(),
            policy_rules: None,
            created_at: 0,
            updated_at: 0,
        },
        [3u8; 16],
        WalletState::Locked,
    );
    let profile_bytes = BincodeCodec.serialize(&profile).unwrap();
    let session = open_wallet_store(path, &wallet_id, &password, &identity).unwrap();
    write_wallet_profile(&session, profile_bytes, SystemRngProvider).unwrap();
}

fn create_test_wallet(path: &Path) {
    create_test_wallet_with_identity(path, "wallet_test", default_identity());
}

#[tokio::test]
async fn test_open_wallet_discovers_unlocks() {
    let src_dir = tempfile::tempdir().unwrap();
    let src_path = src_dir.path().join("wallet_selected.wlt");
    create_test_wallet(&src_path);

    let out_dir = tempfile::tempdir().unwrap();
    let service = WalletService::with_output_dir(out_dir.path().to_path_buf());
    let _env = test_common::WalletEnvGuard::new("tor", "mainnet");

    let discovery = service
        .open_wallet_source(WalletSource::Path {
            path: src_path.to_string_lossy().to_string(),
        })
        .await
        .unwrap();

    assert_eq!(discovery.wallet_id.0, "wallet_test");
    assert_eq!(discovery.chain, "devnet");

    // Prove the file was imported into the managed output directory:
    // unlock-by-id should succeed after open_wallet_source.
    let password = SafePassword::from("pw1");
    let _token = service
        .unlock_wallet_in_memory(&discovery.wallet_id, &password)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_open_wallet_bytes_unlocks() {
    let src_dir = tempfile::tempdir().unwrap();
    let src_path = src_dir.path().join("wallet_selected.wlt");
    create_test_wallet(&src_path);

    let bytes = io::read_file(&src_path).unwrap();

    let out_dir = tempfile::tempdir().unwrap();
    let service = WalletService::with_output_dir(out_dir.path().to_path_buf());
    let _env = test_common::WalletEnvGuard::new("tor", "mainnet");

    let discovery = service
        .open_wallet_source(WalletSource::Bytes { bytes })
        .await
        .unwrap();

    assert_eq!(discovery.wallet_id.0, "wallet_test");
    assert_eq!(discovery.chain, "devnet");

    let password = SafePassword::from("pw1");
    let _token = service
        .unlock_wallet_in_memory(&discovery.wallet_id, &password)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_open_wallet_identity_mismatch() {
    let src_dir = tempfile::tempdir().unwrap();
    let canonical_path = src_dir.path().join("wallet_selected.wlt");
    create_test_wallet(&canonical_path);

    let conflicting_path = src_dir.path().join("wallet_selected_conflict.wlt");
    create_test_wallet_with_identity(
        &conflicting_path,
        "wallet_test",
        WalletIdentity {
            network: "tor".to_string(),
            chain: "mainnet".to_string(),
        },
    );

    let canonical_bytes = io::read_file(&canonical_path).unwrap();
    let conflicting_bytes = io::read_file(&conflicting_path).unwrap();

    let out_dir = tempfile::tempdir().unwrap();
    let service = WalletService::with_output_dir(out_dir.path().to_path_buf());

    service
        .open_wallet_source(WalletSource::Bytes {
            bytes: canonical_bytes,
        })
        .await
        .unwrap();

    let error = service
        .open_wallet_source(WalletSource::Bytes {
            bytes: conflicting_bytes,
        })
        .await
        .unwrap_err();

    assert!(
        matches!(error, WalletError::WalletNetworkMismatch { .. })
            || matches!(error, WalletError::WalletChainMismatch { .. })
    );
}
