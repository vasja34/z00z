use std::{sync::Arc, thread};

use rand::RngCore;
use z00z_core::genesis::ChainType;
use z00z_crypto::expert::{encoding::ByteArray, keys::RistrettoPublicKey, traits::PublicKeyTrait};
use z00z_utils::rng::MockRngProvider;
use z00z_wallets::key::KeyManagerError;
use z00z_wallets::key::{
    compute_schnorr_challenge, AeadId, Argon2idParams, Bip44Path, ChallengeSize,
    CipherSeedContainer, KdfId, KeyManager, KeyManagerImpl, KeyManagerMetadata, KeyManagerState,
    MAX_DERIVED_PUBKEY_CACHE,
};

fn test_seed_bytes(seed: u64) -> [u8; 64] {
    let rng = MockRngProvider::with_u64_seed(seed);
    let mut inner = rng.rng();

    let mut bytes = [0u8; 64];
    inner.fill_bytes(&mut bytes);
    bytes
}

#[test]
fn test_concurrent_derive_key() {
    let mut km = KeyManagerImpl::new();
    let seed_bytes = test_seed_bytes(2_345_678);
    km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

    let km = Arc::new(km);
    let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
    let mut handles = Vec::new();

    for _ in 0..16 {
        let km = Arc::clone(&km);
        let handle = thread::spawn(move || km.derive_key(&path).unwrap());
        handles.push(handle);
    }

    let mut keys = Vec::new();
    for handle in handles {
        keys.push(handle.join().expect("thread must not panic"));
    }

    let first = keys
        .first()
        .cloned()
        .expect("must have at least one derived key");
    for key in keys {
        assert_eq!(key, first);
    }

    assert_eq!(km.get_public_key(&path).expect("key must be cached"), first);
    km.validate_cache().unwrap();
    assert_eq!(km.derivation_count(), 1);
}

#[test]
fn test_concurrent_derivation() {
    let mut km = KeyManagerImpl::new();
    let seed_bytes = test_seed_bytes(2_345_678);
    km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

    let km = Arc::new(km);
    let path = Bip44Path::new_z00z(0, 0, 0).unwrap();

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let km = Arc::clone(&km);
            thread::spawn(move || km.derive_key(&path).unwrap())
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("thread must not panic"))
        .collect();

    assert!(results.windows(2).all(|w| w[0] == w[1]));

    assert_eq!(
        km.get_public_key(&path).expect("key must be cached"),
        results[0]
    );
    km.validate_cache().unwrap();
    assert_eq!(km.derivation_count(), 1);
}

#[test]
fn test_cache_evicts_old_entries() {
    let mut km = KeyManagerImpl::new();
    let seed_bytes = test_seed_bytes(2_345_678);
    km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

    for i in 0..(MAX_DERIVED_PUBKEY_CACHE + 100) {
        let path = Bip44Path::new_z00z(0, 0, i as u32).unwrap();
        km.derive_key(&path).unwrap();
    }

    let first_path = Bip44Path::new_z00z(0, 0, 0).unwrap();
    let last_path = Bip44Path::new_z00z(0, 0, (MAX_DERIVED_PUBKEY_CACHE + 99) as u32).unwrap();

    assert!(km.get_public_key(&first_path).is_none());
    assert!(km.get_public_key(&last_path).is_some());
}

#[test]
fn test_cache_cleared_key_rotation() {
    let mut km = KeyManagerImpl::new();
    let seed_bytes = test_seed_bytes(2_345_678);
    km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

    for i in 0..100 {
        let path = Bip44Path::new_z00z(0, 0, i).unwrap();
        km.derive_key(&path).unwrap();
    }

    let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
    assert!(km.get_public_key(&path).is_some());

    let seed_bytes2 = test_seed_bytes(9_876_543);
    km.init_from_seed(&seed_bytes2, ChainType::Devnet).unwrap();

    assert!(km.get_public_key(&path).is_none());
}

#[test]
fn test_next_external_rejects_corruption() {
    let mut km = KeyManagerImpl::new();
    let seed_bytes = test_seed_bytes(2_345_678);
    km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

    km.set_gap_ext(5, 10);

    let err = km.next_external().unwrap_err();
    assert!(matches!(err, KeyManagerError::StateCorrupted));
}

#[test]
fn test_state_chain_roundtrip() {
    let encrypted_seed = CipherSeedContainer {
        version: CipherSeedContainer::VERSION,
        birthday: 12345,
        kdf: KdfId::Argon2id,
        kdf_params: Argon2idParams::DEFAULT,
        aead: AeadId::XChaCha20Poly1305,
        salt: [0u8; 32],
        nonce: [0u8; 24],
        ciphertext: vec![0u8; 64],
    };

    let metadata = KeyManagerMetadata {
        wallet_id: "wallet-001".to_string(),
        created_at: 1,
        version: 1,
        label: None,
    };

    let state =
        KeyManagerImpl::state_from_encrypted_seed(encrypted_seed, metadata, ChainType::Mainnet)
            .unwrap();

    let encoded = serde_json::to_vec(&state).unwrap();
    let decoded: KeyManagerState = serde_json::from_slice(&encoded).unwrap();

    assert_eq!(decoded.chain, ChainType::Mainnet);
}

#[test]
fn test_sign_and_verify_roundtrip() {
    let mut km = KeyManagerImpl::new();

    let seed_bytes = test_seed_bytes(2_345_678);
    km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

    let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
    let msg = b"hello";

    let sig = km.sign(&path, msg).unwrap();
    let pk = km.derive_key(&path).unwrap();

    let public_nonce_bytes: &[u8; 32] = sig
        .get_public_nonce()
        .as_bytes()
        .try_into()
        .expect("nonce must be 32 bytes");
    let public_key_bytes: &[u8; 32] = pk.as_bytes().try_into().expect("pk must be 32 bytes");

    let challenge = compute_schnorr_challenge(
        public_nonce_bytes,
        public_key_bytes,
        msg,
        ChallengeSize::B512,
    )
    .into_b512()
    .expect("must be B512");
    assert!(sig.verify_raw_uniform(&pk, &challenge));
}

#[test]
fn test_sign_nonce_is_deterministic() {
    let mut km = KeyManagerImpl::new();

    let seed_bytes = test_seed_bytes(2_345_678);
    km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

    let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
    let msg = b"same-message";

    let sig1 = km.sign(&path, msg).unwrap();
    let sig2 = km.sign(&path, msg).unwrap();

    assert_eq!(sig1.get_public_nonce(), sig2.get_public_nonce());

    let sig3 = km.sign(&path, b"different-message").unwrap();
    assert_ne!(sig1.get_public_nonce(), sig3.get_public_nonce());
}

#[test]
fn test_sign_fails_wrong_pubkey() {
    let mut km = KeyManagerImpl::new();

    let seed_bytes = test_seed_bytes(2_345_678);
    km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

    let path_a = Bip44Path::new_z00z(0, 0, 0).unwrap();
    let path_b = Bip44Path::new_z00z(0, 0, 1).unwrap();
    let msg = b"msg";

    let sig = km.sign(&path_a, msg).unwrap();
    let pk_wrong = km.derive_key(&path_b).unwrap();

    let public_nonce_bytes: &[u8; 32] = sig
        .get_public_nonce()
        .as_bytes()
        .try_into()
        .expect("nonce must be 32 bytes");
    let public_key_bytes: &[u8; 32] = pk_wrong.as_bytes().try_into().expect("pk must be 32 bytes");

    let challenge = compute_schnorr_challenge(
        public_nonce_bytes,
        public_key_bytes,
        msg,
        ChallengeSize::B512,
    )
    .into_b512()
    .expect("must be B512");
    assert!(!sig.verify_raw_uniform(&pk_wrong, &challenge));
}

#[test]
fn test_key_manager_split_contract() {
    let source = include_str!("../src/key/manager_core.rs");

    for part in ["manager_cache.rs", "manager_state.rs", "manager_impl.rs"] {
        let needle = format!("include!(\"{part}\");");
        assert!(
            source.contains(&needle),
            "manager_core.rs must keep facade include for {part}"
        );
    }
}

#[test]
fn test_secret_transient_matches_public() {
    let mut km = KeyManagerImpl::new();

    let seed_bytes = test_seed_bytes(2_345_678);
    km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

    let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
    let sk = km.derive_secret_transient(&path).unwrap();
    let pk_from_sk = RistrettoPublicKey::from_secret_key(&*sk);
    let pk = km.derive_key(&path).unwrap();
    assert_eq!(pk, pk_from_sk);
}

mod key_manager_state_tests {
    use z00z_core::genesis::ChainType;
    use z00z_crypto::expert::{
        keys::{RistrettoPublicKey, RistrettoSecretKey},
        traits::{PublicKeyTrait, SecretKeyTrait},
    };
    use z00z_wallets::key::{
        AeadId, Argon2idParams, Bip44Path, CipherSeedContainer, KdfId, KeyManager, KeyManagerError,
        KeyManagerImpl,
    };

    #[test]
    fn test_lock_poisoned_errors() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = super::test_seed_bytes(2_345_678);
        km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();

        km.poison_cache();

        let err = km.derive_key(&path).unwrap_err();
        assert!(matches!(
            err,
            KeyManagerError::LockPoisoned {
                lock: "derived_public_keys"
            }
        ));
    }

    #[test]
    fn test_corrupted_state_combinations() {
        let mut km = KeyManagerImpl::new();

        assert!(matches!(
            km.validate_state(),
            Err(KeyManagerError::NotInitialized)
        ));

        let encrypted_seed = CipherSeedContainer {
            version: CipherSeedContainer::VERSION,
            birthday: 12345,
            kdf: KdfId::Argon2id,
            kdf_params: Argon2idParams::DEFAULT,
            aead: AeadId::XChaCha20Poly1305,
            salt: [0u8; 32],
            nonce: [0u8; 24],
            ciphertext: vec![0u8; 64],
        };
        km.set_encrypted_seed_only(encrypted_seed);

        assert!(matches!(
            km.validate_state(),
            Err(KeyManagerError::StateCorrupted)
        ));
    }

    #[test]
    fn test_cache_without_master_detected() {
        let km = KeyManagerImpl::new();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let sk = RistrettoSecretKey::from_uniform_bytes(&[1u8; 64]).unwrap();
        let pk = RistrettoPublicKey::from_secret_key(&sk);
        km.insert_cache(path, pk).unwrap();

        assert!(matches!(
            km.validate_state(),
            Err(KeyManagerError::StateCorrupted)
        ));
    }

    #[test]
    fn test_cached_key_tampering_detected() {
        let mut km = KeyManagerImpl::new();
        let seed_bytes = super::test_seed_bytes(2_345_678);
        km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();

        let path = Bip44Path::new_z00z(0, 0, 0).unwrap();
        let _ = km.derive_key(&path).unwrap();

        let sk = RistrettoSecretKey::from_uniform_bytes(&[9u8; 64]).unwrap();
        let pk = RistrettoPublicKey::from_secret_key(&sk);
        km.insert_cache(path, pk).unwrap();

        assert!(matches!(
            km.validate_cache(),
            Err(KeyManagerError::CacheCorrupted)
        ));
    }

    #[test]
    fn test_valid_state_combinations() {
        let km = KeyManagerImpl::new();
        assert!(matches!(
            km.validate_state(),
            Err(KeyManagerError::NotInitialized)
        ));

        let mut km = KeyManagerImpl::new();
        let seed_bytes = super::test_seed_bytes(2_345_678);
        km.init_from_seed(&seed_bytes, ChainType::Devnet).unwrap();
        assert!(km.validate_state().is_ok());
    }
}
