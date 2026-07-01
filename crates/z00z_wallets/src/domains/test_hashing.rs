use super::*;

#[test]
fn test_canonicalize_string() {
    let input = "  Hello World  ";
    let canonical = canonicalize_string(input);
    assert_eq!(canonical, b"hello world");

    let input2 = "Hello\tWorld\n";
    let canonical2 = canonicalize_string(input2);
    assert_eq!(canonical2, b"hello world");
}

#[test]
fn test_prefix_ambiguity_is_prevented() {
    let hash1 = WalletMasterKeyHasher::new_with_label("prefix")
        .chain(b"ab")
        .chain(b"c")
        .finalize();

    let hash2 = WalletMasterKeyHasher::new_with_label("prefix")
        .chain(b"a")
        .chain(b"bc")
        .finalize();

    assert_ne!(hash1.as_ref(), hash2.as_ref());
}

#[test]
fn test_derivation_consistency() {
    let seed = [42u8; 32];
    let key1 = derive_wallet_master_key(&seed);
    let key2 = derive_wallet_master_key(&seed);
    assert_eq!(key1, key2);
}

#[test]
fn test_different_seeds_different_keys() {
    let seed1 = [42u8; 32];
    let seed2 = [43u8; 32];
    let key1 = derive_wallet_master_key(&seed1);
    let key2 = derive_wallet_master_key(&seed2);
    assert_ne!(key1, key2);
}

#[test]
fn test_schnorr_challenge_deterministic() {
    let nonce = [1u8; 32];
    let pubkey = [2u8; 32];
    let msg = b"test message";

    let challenge1 = compute_schnorr_challenge(&nonce, &pubkey, msg, ChallengeSize::B256)
        .into_b256()
        .expect("must be B256");
    let challenge2 = compute_schnorr_challenge(&nonce, &pubkey, msg, ChallengeSize::B256)
        .into_b256()
        .expect("must be B256");

    assert_eq!(challenge1, challenge2);
    assert_eq!(challenge1.len(), 32);
}

#[test]
fn test_schnorr_challenge_different_nonce() {
    let nonce1 = [1u8; 32];
    let nonce2 = [2u8; 32];
    let pubkey = [3u8; 32];
    let msg = b"test message";

    let challenge1 = compute_schnorr_challenge(&nonce1, &pubkey, msg, ChallengeSize::B256)
        .into_b256()
        .expect("must be B256");
    let challenge2 = compute_schnorr_challenge(&nonce2, &pubkey, msg, ChallengeSize::B256)
        .into_b256()
        .expect("must be B256");

    assert_ne!(challenge1, challenge2);
}

#[test]
fn test_schnorr_challenge_different_pubkey() {
    let nonce = [1u8; 32];
    let pubkey1 = [2u8; 32];
    let pubkey2 = [3u8; 32];
    let msg = b"test message";

    let challenge1 = compute_schnorr_challenge(&nonce, &pubkey1, msg, ChallengeSize::B256)
        .into_b256()
        .expect("must be B256");
    let challenge2 = compute_schnorr_challenge(&nonce, &pubkey2, msg, ChallengeSize::B256)
        .into_b256()
        .expect("must be B256");

    assert_ne!(challenge1, challenge2);
}

#[test]
fn test_schnorr_challenge_different_message() {
    let nonce = [1u8; 32];
    let pubkey = [2u8; 32];
    let msg1 = b"test message 1";
    let msg2 = b"test message 2";

    let challenge1 = compute_schnorr_challenge(&nonce, &pubkey, msg1, ChallengeSize::B256)
        .into_b256()
        .expect("must be B256");
    let challenge2 = compute_schnorr_challenge(&nonce, &pubkey, msg2, ChallengeSize::B256)
        .into_b256()
        .expect("must be B256");

    assert_ne!(challenge1, challenge2);
}

#[test]
fn test_index_mac_is_deterministic() {
    let key = [7u8; 32];
    let msg = b"index-key-msg";

    let mac1 = compute_index_mac(&key, msg);
    let mac2 = compute_index_mac(&key, msg);
    assert!(verify_index_mac(&mac1, &mac2));
}

#[test]
fn test_index_mac_key_separation() {
    let key1 = [0u8; 32];
    let mut key2 = [0u8; 32];
    key2[0] = 1;

    let msg = b"same-msg";
    let mac1 = compute_index_mac(&key1, msg);
    let mac2 = compute_index_mac(&key2, msg);
    assert!(!verify_index_mac(&mac1, &mac2));
}

#[test]
fn test_index_rejects_wrong_message() {
    let key = [9u8; 32];
    let msg1 = b"msg-1";
    let msg2 = b"msg-2";

    let mac1 = compute_index_mac(&key, msg1);
    let mac2 = compute_index_mac(&key, msg2);
    assert!(!verify_index_mac(&mac1, &mac2));
}

#[test]
fn test_wallet_seed_hash_binding() {
    let hash_a = compute_wallet_seed_hash(b"seed_a");
    let hash_b = compute_wallet_seed_hash(b"seed_b");
    assert_ne!(hash_a, hash_b);
}
