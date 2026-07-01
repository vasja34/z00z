#![cfg(not(target_arch = "wasm32"))]

use z00z_crypto::expert::encoding::{ByteArray, SafePassword};
use z00z_utils::rng::SystemRngProvider;
use z00z_wallets::{
    db::{create_wallet_store, WalletIdentity},
    key::{Bip44Path, KeyManager, KeyManagerError, KeyManagerImpl},
    rpc::types::common::PersistWalletId,
};

#[test]
fn test_unlock_from_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("wallet_test.wlt");

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("correct horse battery staple");
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    let identity = WalletIdentity {
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
    };

    create_wallet_store(
        &path,
        &wallet_id,
        &password,
        seed_phrase,
        &identity,
        SystemRngProvider,
    )
    .expect("create_wallet_store must succeed");

    let km1 = KeyManagerImpl::unlock_from_storage(&path, &wallet_id, &password, &identity)
        .expect("unlock_from_storage must succeed");

    let km2 = KeyManagerImpl::unlock_from_storage(&path, &wallet_id, &password, &identity)
        .expect("unlock_from_storage must succeed");

    let path0 = Bip44Path::new_z00z(0, 0, 0).expect("valid path");

    let k1 = km1.derive_key(&path0).expect("derive_key");
    let k2 = km2.derive_key(&path0).expect("derive_key");

    assert_eq!(k1.to_vec(), k2.to_vec(), "keys must be deterministic");
}

#[test]
fn test_unlock_wrong_password() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("wallet_test.wlt");

    let wallet_id = PersistWalletId("wallet_test".to_string());
    let password = SafePassword::from("correct horse battery staple");
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    let identity = WalletIdentity {
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
    };

    create_wallet_store(
        &path,
        &wallet_id,
        &password,
        seed_phrase,
        &identity,
        SystemRngProvider,
    )
    .expect("create_wallet_store must succeed");

    let wrong = SafePassword::from("wrong password");
    let err = KeyManagerImpl::unlock_from_storage(&path, &wallet_id, &wrong, &identity)
        .expect_err("expected auth failure");

    assert!(
        matches!(err, KeyManagerError::AuthenticationFailed),
        "expected AuthenticationFailed"
    );
}
