use super::*;

#[test]
fn test_database_error_is_transient() {
    let error = WalletError::DatabaseError("Connection timeout".to_string());
    assert!(error.is_transient());
}

#[test]
fn test_insufficient_balance_non_transient() {
    let error = WalletError::InsufficientBalance {
        needed: 1000,
        available: 500,
    };
    assert!(!error.is_transient());
}

#[test]
fn test_locked_is_not_transient() {
    let error = WalletError::Locked;
    assert!(!error.is_transient());
}

#[test]
fn test_crypto_error_non_transient() {
    let error = WalletError::CryptoError("Signature verification failed".to_string());
    assert!(!error.is_transient());
}

#[test]
fn test_open_mismatches_unsupported_format() {
    let chain_err = WalletError::WalletChainMismatch {
        expected: "devnet".to_string(),
        actual: "mainnet".to_string(),
    };
    let network_err = WalletError::WalletNetworkMismatch {
        expected: "p2p".to_string(),
        actual: "tor".to_string(),
    };

    assert_eq!(
        chain_err.to_public_error(WalletErrorStage::UnlockOpen),
        WalletPublicError::UnsupportedFormat
    );
    assert_eq!(
        network_err.to_public_error(WalletErrorStage::UnlockOpen),
        WalletPublicError::UnsupportedFormat
    );
}