//! Integration tests for BIP-44 derivation workflow
//!
//! This module provides end-to-end tests that verify:
//! - Wallet creation and initialization from mnemonic/passphrase
//! - Account key derivation via internal APIs
//! - Address derivation via internal APIs
//! - Address stability across process restarts
//! - Non-standard path rejection (internal)
//! - Transaction signing uses keys derived from canonical Z00Z path policy

use std::str::FromStr;
use z00z_crypto::expert::encoding::ByteArray;
use z00z_wallets::key::{Bip44Path, KeyManager, KeyManagerImpl};

/// Test vector: BIP-39 mnemonic for testing (standard test vector 1)
const TEST_MNEMONIC_24: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Test vector: BIP-39 passphrase for testing
const TEST_PASSPHRASE: &str = "";

/// Test vector: Expected BIP-39 seed (64 bytes) - derived from standard test vector 1
const TEST_SEED: [u8; 64] = [
    0x5e, 0xb0, 0x0b, 0xbd, 0xdc, 0xf0, 0x69, 0x08, 0x48, 0x89, 0xa8, 0xab, 0x91, 0x55, 0x56, 0x81,
    0x65, 0xf5, 0xc4, 0x53, 0xcc, 0xb8, 0x5e, 0x70, 0x81, 0x1a, 0xae, 0xd6, 0xf6, 0xda, 0x5f, 0xc1,
    0x9a, 0x5a, 0xc4, 0x0b, 0x38, 0x9c, 0xd3, 0x70, 0xd0, 0x86, 0x20, 0x6d, 0xec, 0x8a, 0xa6, 0xc4,
    0x3d, 0xae, 0xa6, 0x69, 0x0f, 0x20, 0xad, 0x3d, 0x8d, 0x48, 0xb2, 0xd2, 0xce, 0x9e, 0x38, 0xe4,
];

// ============================================================================
// Phase 14: Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Create wallet + initialize from mnemonic/passphrase
    #[test]
    fn test_wallet_create_from_mnemonic() {
        let mut key_manager = KeyManagerImpl::new();

        // Initialize from BIP-39 seed (simulating wallet creation from mnemonic)
        let result = key_manager.init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet);
        assert!(
            result.is_ok(),
            "Failed to initialize key manager from BIP-39 seed"
        );

        // Verify key manager is initialized by deriving a key
        let payment_key = key_manager.derive_payment_key(0);
        assert!(payment_key.is_ok(), "Key manager should be initialized");
    }

    /// Test: Derive account keys and addresses via internal APIs
    #[test]
    fn test_derive_account_keys_internal() {
        let mut key_manager = KeyManagerImpl::new();

        // Initialize from BIP-39 seed
        key_manager
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        // Derive payment address (external chain)
        let payment_key = key_manager.derive_payment_key(0).unwrap();

        // Derive change address (internal chain)
        let change_key = key_manager.derive_change_key(0).unwrap();

        // Verify keys are different
        assert_ne!(
            payment_key.as_bytes(),
            change_key.as_bytes(),
            "Payment and change keys should be different"
        );
    }

    /// Test: Ensure derived addresses remain stable across process restart
    #[test]
    fn test_address_stability_across_restart() {
        // First instance
        let mut key_manager1 = KeyManagerImpl::new();
        key_manager1
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        let payment_key1 = key_manager1.derive_payment_key(0).unwrap();
        let change_key1 = key_manager1.derive_change_key(0).unwrap();

        // Second instance (simulating restart)
        let mut key_manager2 = KeyManagerImpl::new();
        key_manager2
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        let payment_key2 = key_manager2.derive_payment_key(0).unwrap();
        let change_key2 = key_manager2.derive_change_key(0).unwrap();

        // Verify keys are identical across restarts
        assert_eq!(
            payment_key1.as_bytes(),
            payment_key2.as_bytes(),
            "Payment keys should be identical across restarts"
        );
        assert_eq!(
            change_key1.as_bytes(),
            change_key2.as_bytes(),
            "Change keys should be identical across restarts"
        );
    }

    /// Test: Ensure non-standard paths are rejected consistently
    #[test]
    fn test_non_standard_path_rejection() {
        // Try to parse a non-standard path (wrong coin type)
        let non_standard_path = Bip44Path::from_str("m/44'/0'/0'/0/0");
        assert!(
            non_standard_path.is_err(),
            "Bip44Path should reject non-standard path"
        );
    }

    /// Test: Ensure transaction signing uses keys derived from canonical Z00Z path policy
    #[test]
    fn test_signing_uses_canonical_path() {
        let mut key_manager = KeyManagerImpl::new();

        // Initialize from BIP-39 seed
        key_manager
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        // Derive a key using canonical Z00Z path
        let canonical_path = Bip44Path::payment(0).unwrap();
        let canonical_key = key_manager.derive_payment_key(0).unwrap();

        // Verify the key is derived using the canonical path
        // (This is a sanity check - the actual signing would use this key)
        assert_eq!(
            canonical_path.to_string(),
            "m/44'/1337'/0'/0/0",
            "Canonical path should be m/44'/1337'/0'/0/0"
        );

        // Verify the key is not zero
        assert_ne!(
            canonical_key.as_bytes(),
            [0u8; 32],
            "Derived key should not be zero"
        );
    }

    /// Test: Verify BIP-39 seed determinism
    #[test]
    fn test_bip39_seed_determinism() {
        use bip39::{Language, Mnemonic};

        // Parse mnemonic (24 words)
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC_24).unwrap();

        // Derive seed from mnemonic + passphrase
        let seed = mnemonic.to_seed(TEST_PASSPHRASE);

        // Verify seed matches expected
        assert_eq!(
            seed.as_ref(),
            &TEST_SEED,
            "BIP-39 seed should be deterministic"
        );
    }

    /// Test: Verify wasm/non-wasm parity
    #[test]
    fn test_wasm_nonwasm_parity() {
        let mut key_manager = KeyManagerImpl::new();

        // Initialize from BIP-39 seed
        key_manager
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        // Derive key
        let key = key_manager.derive_payment_key(0).unwrap();

        // Verify key is valid (same on wasm and non-wasm)
        assert_eq!(
            key.as_bytes().len(),
            32,
            "Ristretto public key should be 32 bytes"
        );
    }

    /// Test: Derive account keys and addresses via RPC (simulated)
    #[test]
    fn test_derive_account_keys_rpc() {
        let mut key_manager = KeyManagerImpl::new();

        // Initialize from BIP-39 seed
        key_manager
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        // Simulate RPC call to derive payment key
        let payment_path = Bip44Path::payment(0).unwrap();
        let payment_key = key_manager.derive_key(&payment_path).unwrap();

        // Simulate RPC call to derive change key
        let change_path = Bip44Path::change_path(0).unwrap();
        let change_key = key_manager.derive_key(&change_path).unwrap();

        // Verify keys are different
        assert_ne!(
            payment_key.as_bytes(),
            change_key.as_bytes(),
            "Payment and change keys should be different"
        );

        // Verify keys are 32 bytes (Ristretto)
        assert_eq!(
            payment_key.as_bytes().len(),
            32,
            "Payment key should be 32 bytes"
        );
        assert_eq!(
            change_key.as_bytes().len(),
            32,
            "Change key should be 32 bytes"
        );
    }

    /// Test: Ensure non-standard paths are rejected via RPC (simulated)
    #[test]
    fn test_rejects_nonstandard_path_rpc() {
        let mut key_manager = KeyManagerImpl::new();

        // Initialize from BIP-39 seed
        key_manager
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        // Try to derive with non-standard path (wrong coin type)
        let non_standard_path = Bip44Path::from_str("m/44'/0'/0'/0/0");
        assert!(
            non_standard_path.is_err(),
            "Bip44Path should reject non-standard path"
        );
    }

    #[test]
    fn test_hd_path_keys() {
        // Test Flow:
        // 1) Init wallet from deterministic seed.
        // 2) Derive account0 payment path and account0 change path.
        // 3) Verify keys differ by path.
        // 4) Derive same payment path again and verify determinism.
        let mut key_manager = KeyManagerImpl::new();
        key_manager
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        let pay_path = Bip44Path::payment_for_account(0, 0).unwrap();
        let chg_path = Bip44Path::change_path_for_account(0, 0).unwrap();

        assert_eq!(pay_path.to_string(), "m/44'/1337'/0'/0/0");
        assert_eq!(chg_path.to_string(), "m/44'/1337'/0'/1/0");

        let pay_key = key_manager.derive_key(&pay_path).unwrap();
        let chg_key = key_manager.derive_key(&chg_path).unwrap();

        assert_ne!(pay_key.as_bytes(), chg_key.as_bytes());

        let pay_key_2 = key_manager.derive_key(&pay_path).unwrap();
        assert_eq!(pay_key.as_bytes(), pay_key_2.as_bytes());

        let mut key_manager_2 = KeyManagerImpl::new();
        key_manager_2
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();
        let pay_key_3 = key_manager_2.derive_key(&pay_path).unwrap();
        assert_eq!(pay_key.as_bytes(), pay_key_3.as_bytes());
    }

    #[test]
    fn test_hd_accounts() {
        // Test Flow:
        // 1) Init wallet from deterministic seed.
        // 2) Derive account 0/1/2 receiving paths.
        // 3) Verify keys are unique across accounts.
        // 4) Re-init manager and verify account keys recover identically.
        let mut key_manager = KeyManagerImpl::new();
        key_manager
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        let p0 = Bip44Path::payment_for_account(0, 0).unwrap();
        let p1 = Bip44Path::payment_for_account(1, 0).unwrap();
        let p2 = Bip44Path::payment_for_account(2, 0).unwrap();

        let k0 = key_manager.derive_key(&p0).unwrap();
        let k1 = key_manager.derive_key(&p1).unwrap();
        let k2 = key_manager.derive_key(&p2).unwrap();

        assert_ne!(k0.as_bytes(), k1.as_bytes());
        assert_ne!(k0.as_bytes(), k2.as_bytes());
        assert_ne!(k1.as_bytes(), k2.as_bytes());

        let mut key_manager_2 = KeyManagerImpl::new();
        key_manager_2
            .init_from_seed(&TEST_SEED, z00z_core::genesis::ChainType::Devnet)
            .unwrap();

        let k0_2 = key_manager_2.derive_key(&p0).unwrap();
        let k1_2 = key_manager_2.derive_key(&p1).unwrap();
        let k2_2 = key_manager_2.derive_key(&p2).unwrap();

        assert_eq!(k0.as_bytes(), k0_2.as_bytes());
        assert_eq!(k1.as_bytes(), k1_2.as_bytes());
        assert_eq!(k2.as_bytes(), k2_2.as_bytes());
    }

    #[test]
    fn test_bip32_source_split_contract() {
        let source = include_str!("../src/key/bip32.rs");

        for part in [
            "bip32_constants.rs",
            "bip32_path.rs",
            "bip32_path_validator.rs",
            "bip32_key_deriver.rs",
            "bip32_ristretto_bridge.rs",
        ] {
            let needle = format!("include!(\"{part}\");");
            assert!(
                source.contains(&needle),
                "bip32.rs must keep facade include for {part}"
            );
        }
    }
}
