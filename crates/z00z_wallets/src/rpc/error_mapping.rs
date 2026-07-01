//! RPC error mapping
//!
//! Maps WalletError to JSON-RPC ErrorObjectOwned.

use crate::{
    rpc::types::tx::{RuntimeTxErrorCode, RuntimeTxLifecycle},
    WalletError,
};
use jsonrpsee::types::ErrorObjectOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RuntimeTxRpcErrorData {
    pub error_codes: Vec<RuntimeTxErrorCode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<RuntimeTxLifecycle>,
}

fn push_runtime_tx_error_code(
    error_codes: &mut Vec<RuntimeTxErrorCode>,
    error_code: RuntimeTxErrorCode,
) {
    if !error_codes.contains(&error_code) {
        error_codes.push(error_code);
    }
}

pub(crate) fn map_wallet_error_code(error: &WalletError) -> RuntimeTxErrorCode {
    match error {
        WalletError::UnsupportedVersion(_) | WalletError::UnsupportedKdf(_) => {
            RuntimeTxErrorCode::UnsupportedPackageVersion
        }
        WalletError::WalletChainMismatch { .. } | WalletError::WalletNetworkMismatch { .. } => {
            RuntimeTxErrorCode::WrongChain
        }
        WalletError::InvalidAssetPack(_) => RuntimeTxErrorCode::UnsupportedReceiveVersion,
        WalletError::CommitmentMismatch => RuntimeTxErrorCode::InvalidPublicSpendProof,
        WalletError::InvalidConfig(message)
        | WalletError::InvalidTransaction(message)
        | WalletError::DatabaseError(message)
        | WalletError::SerializationError(message)
        | WalletError::CryptoError(message)
        | WalletError::ContextError(message)
        | WalletError::CommandFailed(message)
        | WalletError::Io(message) => map_message_error_code(message),
        _ => RuntimeTxErrorCode::InternalError,
    }
}

pub(crate) fn map_message_error_code(message: &str) -> RuntimeTxErrorCode {
    let lower = message.to_ascii_lowercase();

    if lower.contains("unsupported portable transaction package version")
        || lower.contains("tx package version must be non-zero")
    {
        return RuntimeTxErrorCode::UnsupportedPackageVersion;
    }

    if lower.contains("unsupported request version")
        || lower.contains("unsupported card version")
        || lower.contains("unsupported receiver-card record version")
        || lower.contains("unsupported asset pack")
        || lower.contains("unsupported receive")
    {
        return RuntimeTxErrorCode::UnsupportedReceiveVersion;
    }

    if lower.contains("metadata hash")
        || lower.contains("tx_digest_hex does not match payload")
        || lower.contains("tx digest")
    {
        return RuntimeTxErrorCode::InvalidDigest;
    }

    if lower.contains("chain id")
        || lower.contains("chain mismatch")
        || lower.contains("wrong chain")
    {
        return RuntimeTxErrorCode::WrongChain;
    }

    if lower.contains("missing spend proof")
        || lower.contains("public spend contract failed")
        || lower.contains("invalid public spend proof")
        || lower.contains("invalid range proof")
        || lower.contains("invalid signature")
    {
        return RuntimeTxErrorCode::InvalidPublicSpendProof;
    }

    if lower.contains("no wallet-owned outputs") || lower.contains("no owned outputs") {
        return RuntimeTxErrorCode::NoOwnedOutputs;
    }

    if lower.contains("not import-ready") {
        return RuntimeTxErrorCode::NotImportReady;
    }

    if lower.contains("duplicate owned asset id conflicts")
        || lower.contains("claim conflict")
        || lower.contains("duplicate conflict")
        || lower.contains("tx payload conflicts")
    {
        return RuntimeTxErrorCode::DuplicateConflict;
    }

    if lower.contains("already spent") || lower.contains("spend confirmation status mismatch") {
        return RuntimeTxErrorCode::AlreadySpent;
    }

    if lower.contains("scan state changed during receive persistence") {
        return RuntimeTxErrorCode::CursorConflict;
    }

    if lower.contains("worker evidence") {
        return RuntimeTxErrorCode::WorkerEvidenceRejected;
    }

    if lower.contains("invalid tx_data")
        || lower.contains("invalid portable transaction package")
        || lower.contains("valid utf-8")
        || lower.contains("invalid tx output wire")
    {
        return RuntimeTxErrorCode::InvalidEncoding;
    }

    if lower.contains("invalid tx package")
        || lower.contains("invalid portable tx package")
        || lower.contains("asset decode failed")
        || lower.contains("unsupported tx package kind")
        || lower.contains("unsupported tx package subtype")
        || lower.contains("invalid reconcile output asset")
        || lower.contains("tx package")
    {
        return RuntimeTxErrorCode::InvalidPackage;
    }

    RuntimeTxErrorCode::InternalError
}

pub(crate) fn map_verify_error_codes(errors: &[String]) -> Vec<RuntimeTxErrorCode> {
    let mut error_codes = Vec::new();
    for error in errors {
        push_runtime_tx_error_code(&mut error_codes, map_message_error_code(error));
    }
    error_codes
}

pub(crate) fn runtime_tx_error_response(
    rpc_code: i32,
    message: impl Into<String>,
    error_codes: Vec<RuntimeTxErrorCode>,
    lifecycle: Option<RuntimeTxLifecycle>,
) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        rpc_code,
        message.into(),
        Some(RuntimeTxRpcErrorData {
            error_codes,
            lifecycle,
        }),
    )
}

/// Map WalletError to JSON-RPC ErrorObjectOwned
///
/// Converts domain-specific `WalletError` variants into standardized JSON-RPC 2.0 error responses.
///
/// # Error Code Mapping
///
/// | Code    | Error Variant          | Description                      |
/// |---------|------------------------|----------------------------------|
/// | -32001  | NotOwned               | Asset not owned by this wallet  |
/// | -32002  | NotFound               | Asset not found                 |
/// | -32003  | Locked                 | Wallet is locked                |
/// | -32004  | InsufficientBalance    | Not enough funds                |
/// | -32005  | InvalidAmount          | Invalid amount specified        |
/// | -32006  | InvalidFee             | Invalid fee                     |
/// | -32007  | InvalidPassword        | Invalid password (reserved)     |
/// | -32008  | InvalidTransaction     | Transaction validation failed   |
/// | -32009  | TooManyInputs          | Privacy constraint (max 4)      |
/// | -32010  | TooManyOutputs         | Privacy constraint (max 2)      |
/// | -32011  | InsufficientInputs     | No inputs provided              |
/// | -32012  | BalanceProofFailed     | Pedersen balance proof failed   |
/// | -32013  | DatabaseError          | Database operation failed       |
/// | -32014  | SerializationError     | Encoding/decoding failed        |
/// | -32015  | CryptoError            | Cryptographic operation failed  |
/// | -32016  | ContextError           | Context error                   |
/// | -32017  | CommandNotFound        | Command not found               |
/// | -32018  | CommandFailed          | Command execution failed        |
/// | -32019  | InvalidConfig          | Invalid configuration           |
/// | -32020  | KeyDerivation          | Key derivation failed           |
/// | -32025  | WalletInUse            | Wallet is already open          |
/// | -32026  | WalletNetworkMismatch  | Wrong network for this wallet   |
/// | -32027  | WalletChainMismatch    | Wrong chain for this wallet     |
///
/// # Examples
///
/// ```rust
/// use z00z_wallets::WalletError;
/// use z00z_wallets::rpc::error_mapping::map_wallet_error_to_rpc;
///
/// let error = WalletError::InsufficientBalance {
///     needed: 1000,
///     available: 500,
/// };
///
/// let rpc_error = map_wallet_error_to_rpc(error);
/// assert_eq!(rpc_error.code(), -32004);
/// assert!(rpc_error.message().contains("1000"));
/// ```
///
/// # Usage in RPC Methods
///
/// ```rust,ignore
/// async fn create_wallet(&self, name: String, password: String) -> RpcResult<WalletId> {
///     self.app_service
///         .create_wallet(name, password, None)
///         .await
///         .map_err(map_wallet_error_to_rpc)  // ← Convert error here
/// }
/// ```
pub fn map_wallet_error_to_rpc(e: WalletError) -> ErrorObjectOwned {
    match e {
        WalletError::InvalidParams(message) => {
            let message = if message == "Password required" {
                "Password required"
            } else {
                "Invalid params"
            };
            ErrorObjectOwned::owned(-32602, message, None::<()>)
        }
        WalletError::InvalidAssetPack(_) => {
            ErrorObjectOwned::owned(-32028, "Invalid asset pack", None::<()>)
        }
        WalletError::CommitmentMismatch => {
            ErrorObjectOwned::owned(-32029, "Commitment mismatch", None::<()>)
        }
        WalletError::NotOwned => {
            ErrorObjectOwned::owned(-32001, "Asset not owned by this wallet", None::<()>)
        }
        WalletError::Locked => ErrorObjectOwned::owned(
            -32003,
            "Wallet locked".to_string(),
            Some(
                "The wallet is locked. Unlock it first using wallet.session.unlock_wallet method.",
            ),
        ),
        WalletError::SessionExpired => ErrorObjectOwned::owned(
            crate::rpc::types::security::SecurityErrorCode::SessionExpired.code(),
            crate::rpc::types::security::SecurityErrorCode::SessionExpired
                .message()
                .to_string(),
            None::<()>,
        ),
        WalletError::SessionInvalid => ErrorObjectOwned::owned(
            crate::rpc::types::security::SecurityErrorCode::SessionInvalid.code(),
            crate::rpc::types::security::SecurityErrorCode::SessionInvalid
                .message()
                .to_string(),
            None::<()>,
        ),
        WalletError::InvalidPassword => {
            ErrorObjectOwned::owned(-32007, "Invalid password", None::<()>)
        }
        WalletError::RateLimited {
            retry_after_seconds,
        } => ErrorObjectOwned::owned(
            -32022,
            format!("Rate limited: retry after {} seconds", retry_after_seconds),
            Some(format!(
                "Too many requests. Please wait {} seconds before retrying.",
                retry_after_seconds
            )),
        ),
        WalletError::JobNotFound(_) => ErrorObjectOwned::owned(-32021, "Job not found", None::<()>),
        WalletError::KeyDerivation(_) => {
            ErrorObjectOwned::owned(-32020, "Key derivation failed", None::<()>)
        }
        WalletError::InsufficientBalance { needed, available } => ErrorObjectOwned::owned(
            -32004,
            format!("Insufficient balance: need {}, have {}", needed, available),
            None::<()>,
        ),
        WalletError::TooManyInputs => {
            ErrorObjectOwned::owned(-32009, "Too many inputs (max 4 for privacy)", None::<()>)
        }
        WalletError::TooManyOutputs => {
            ErrorObjectOwned::owned(-32010, "Too many outputs (max 2 for privacy)", None::<()>)
        }
        WalletError::InvalidAmount(_) => {
            ErrorObjectOwned::owned(-32005, "Invalid amount", None::<()>)
        }
        WalletError::InvalidFee(_) => ErrorObjectOwned::owned(-32006, "Invalid fee", None::<()>),
        WalletError::InvalidTransaction(_) => {
            ErrorObjectOwned::owned(-32008, "Invalid transaction", None::<()>)
        }
        WalletError::InsufficientInputs => {
            ErrorObjectOwned::owned(-32011, "No inputs provided", None::<()>)
        }
        WalletError::BalanceProofFailed => {
            ErrorObjectOwned::owned(-32012, "Balance proof failed", None::<()>)
        }
        WalletError::DatabaseError(_) => {
            ErrorObjectOwned::owned(-32013, "Database error", None::<()>)
        }
        WalletError::SerializationError(_) => {
            ErrorObjectOwned::owned(-32014, "Serialization error", None::<()>)
        }
        WalletError::NotFound(serial_id) => ErrorObjectOwned::owned(
            -32002,
            format!("Asset not found: {}", serial_id),
            None::<()>,
        ),
        WalletError::CryptoError(_) => ErrorObjectOwned::owned(-32015, "Crypto error", None::<()>),
        WalletError::IdentityPointNotAllowed => {
            ErrorObjectOwned::owned(-32015, "Identity point not allowed", None::<()>)
        }
        WalletError::DuplicateEphemeralR => {
            ErrorObjectOwned::owned(-32015, "Duplicate ephemeral R", None::<()>)
        }
        WalletError::ContextError(_) => {
            ErrorObjectOwned::owned(-32016, "Context error", None::<()>)
        }
        WalletError::CommandNotFound(_) => {
            ErrorObjectOwned::owned(-32017, "Command not found", None::<()>)
        }
        WalletError::CommandFailed(_) => {
            ErrorObjectOwned::owned(-32018, "Command failed", None::<()>)
        }
        WalletError::InvalidConfig(_) => {
            ErrorObjectOwned::owned(-32019, "Invalid config", None::<()>)
        }
        WalletError::ChecksumMismatch { .. } => ErrorObjectOwned::owned(
            -32020,
            "Wallet file corrupted: checksum mismatch",
            None::<()>,
        ),
        WalletError::InvalidPermissions(_) => {
            ErrorObjectOwned::owned(-32021, "Invalid file permissions", None::<()>)
        }
        WalletError::Io(_) => ErrorObjectOwned::owned(-32022, "I/O error", None::<()>),
        WalletError::WalletInUse => ErrorObjectOwned::owned(-32025, "Wallet in use", None::<()>),
        WalletError::WalletNetworkMismatch { .. } => {
            ErrorObjectOwned::owned(-32026, "Wallet network mismatch", None::<()>)
        }
        WalletError::WalletChainMismatch { .. } => {
            ErrorObjectOwned::owned(-32027, "Wallet chain mismatch", None::<()>)
        }
        WalletError::WalletAlreadyExists => {
            ErrorObjectOwned::owned(-32024, "Wallet already exists", None::<()>)
        }
        WalletError::UnsupportedVersion(version) => ErrorObjectOwned::owned(
            -32023,
            format!("Unsupported version: {}", version),
            None::<()>,
        ),
        WalletError::UnsupportedKdf(_) => {
            ErrorObjectOwned::owned(-32023, "Unsupported format", None::<()>)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_not_owned() {
        let err = WalletError::NotOwned;
        let rpc_err = map_wallet_error_to_rpc(err);
        assert_eq!(rpc_err.code(), -32001);
        assert_eq!(rpc_err.message(), "Asset not owned by this wallet");
    }

    #[test]
    fn test_map_insufficient_balance() {
        let err = WalletError::InsufficientBalance {
            needed: 1000,
            available: 500,
        };
        let rpc_err = map_wallet_error_to_rpc(err);
        assert_eq!(rpc_err.code(), -32004);
        assert!(rpc_err.message().contains("1000"));
        assert!(rpc_err.message().contains("500"));
    }

    #[test]
    fn test_map_not_found() {
        let err = WalletError::NotFound(42);
        let rpc_err = map_wallet_error_to_rpc(err);
        assert_eq!(rpc_err.code(), -32002);
        assert!(rpc_err.message().contains("42"));
    }

    #[test]
    fn test_map_too_many_inputs() {
        let err = WalletError::TooManyInputs;
        let rpc_err = map_wallet_error_to_rpc(err);
        assert_eq!(rpc_err.code(), -32009);
    }

    #[test]
    fn test_map_crypto_error() {
        let err = WalletError::CryptoError("signature failed".to_string());
        let rpc_err = map_wallet_error_to_rpc(err);
        assert_eq!(rpc_err.code(), -32015);
        assert_eq!(rpc_err.message(), "Crypto error");
    }

    #[test]
    fn test_map_locked() {
        let err = WalletError::Locked;
        let rpc_err = map_wallet_error_to_rpc(err);
        assert_eq!(rpc_err.code(), -32003);
        assert_eq!(rpc_err.message(), "Wallet locked");
        assert!(rpc_err
            .data()
            .unwrap()
            .to_string()
            .contains("wallet.session.unlock_wallet"));
    }

    #[test]
    fn test_maps_pack_commitment_mismatch() {
        let asset_pack = map_wallet_error_to_rpc(WalletError::InvalidAssetPack("bad package"));
        assert_eq!(asset_pack.code(), -32028);
        assert_eq!(asset_pack.message(), "Invalid asset pack");

        let commitment = map_wallet_error_to_rpc(WalletError::CommitmentMismatch);
        assert_eq!(commitment.code(), -32029);
        assert_eq!(commitment.message(), "Commitment mismatch");
    }

    #[test]
    fn test_map_session_security_variants() {
        let expired = map_wallet_error_to_rpc(WalletError::SessionExpired);
        assert_eq!(
            expired.code(),
            crate::rpc::types::security::SecurityErrorCode::SessionExpired.code()
        );
        assert_eq!(
            expired.message(),
            crate::rpc::types::security::SecurityErrorCode::SessionExpired.message()
        );

        let invalid = map_wallet_error_to_rpc(WalletError::SessionInvalid);
        assert_eq!(
            invalid.code(),
            crate::rpc::types::security::SecurityErrorCode::SessionInvalid.code()
        );
        assert_eq!(
            invalid.message(),
            crate::rpc::types::security::SecurityErrorCode::SessionInvalid.message()
        );
    }

    #[test]
    fn test_maps_retry_hint() {
        let rpc_err = map_wallet_error_to_rpc(WalletError::RateLimited {
            retry_after_seconds: 42,
        });
        assert_eq!(rpc_err.code(), -32022);
        assert!(rpc_err.message().contains("42"));
        assert!(rpc_err
            .data()
            .expect("rate-limited data")
            .to_string()
            .contains("42"));
    }

    #[test]
    fn test_error_messages_bounded_echo() {
        let password = "StrongPassw0rd!";
        let seed_phrase = "abandon abandon abandon abandon abandon abandon";
        let private_key = "privkey_very_sensitive";
        let tx_blob = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

        let candidates = vec![
            WalletError::InvalidParams(password.to_string()),
            WalletError::KeyDerivation(seed_phrase.to_string()),
            WalletError::DatabaseError(private_key.to_string()),
            WalletError::SerializationError(tx_blob.to_string()),
            WalletError::CryptoError(password.to_string()),
            WalletError::ContextError(seed_phrase.to_string()),
            WalletError::CommandNotFound(private_key.to_string()),
            WalletError::CommandFailed(tx_blob.to_string()),
            WalletError::InvalidConfig(password.to_string()),
            WalletError::InvalidPermissions(seed_phrase.to_string()),
            WalletError::Io(private_key.to_string()),
            WalletError::InvalidAmount(tx_blob.to_string()),
            WalletError::InvalidFee(password.to_string()),
            WalletError::InvalidTransaction(seed_phrase.to_string()),
            WalletError::JobNotFound(tx_blob.to_string()),
        ];

        for err in candidates {
            let rpc_err = map_wallet_error_to_rpc(err);
            let msg = rpc_err.message().to_string();
            let data = rpc_err.data().map(|d| d.to_string()).unwrap_or_default();

            assert!(!msg.contains(password));
            assert!(!msg.contains(seed_phrase));
            assert!(!msg.contains(private_key));
            assert!(!msg.contains(tx_blob));

            assert!(!data.contains(password));
            assert!(!data.contains(seed_phrase));
            assert!(!data.contains(private_key));
            assert!(!data.contains(tx_blob));
        }
    }

    #[test]
    fn test_each_wallet_error_variant() {
        fn test_assert_code(err: WalletError, expected: i32) {
            let rpc_err = map_wallet_error_to_rpc(err);
            assert_eq!(rpc_err.code(), expected);
        }

        test_assert_code(WalletError::NotOwned, -32001);
        test_assert_code(WalletError::NotFound(1), -32002);
        test_assert_code(WalletError::Locked, -32003);
        test_assert_code(
            WalletError::InsufficientBalance {
                needed: 1,
                available: 0,
            },
            -32004,
        );
        test_assert_code(WalletError::InvalidAmount("x".to_string()), -32005);
        test_assert_code(WalletError::InvalidFee("x".to_string()), -32006);
        test_assert_code(WalletError::InvalidPassword, -32007);
        test_assert_code(WalletError::InvalidTransaction("x".to_string()), -32008);
        test_assert_code(WalletError::TooManyInputs, -32009);
        test_assert_code(WalletError::TooManyOutputs, -32010);
        test_assert_code(WalletError::InsufficientInputs, -32011);
        test_assert_code(WalletError::BalanceProofFailed, -32012);
        test_assert_code(WalletError::DatabaseError("x".to_string()), -32013);
        test_assert_code(WalletError::SerializationError("x".to_string()), -32014);
        test_assert_code(WalletError::CryptoError("x".to_string()), -32015);
        test_assert_code(WalletError::ContextError("x".to_string()), -32016);
        test_assert_code(WalletError::CommandNotFound("x".to_string()), -32017);
        test_assert_code(WalletError::CommandFailed("x".to_string()), -32018);
        test_assert_code(WalletError::InvalidConfig("x".to_string()), -32019);
        test_assert_code(WalletError::KeyDerivation("x".to_string()), -32020);
        test_assert_code(
            WalletError::ChecksumMismatch {
                expected: "a".to_string(),
                actual: "b".to_string(),
            },
            -32020,
        );
        test_assert_code(WalletError::JobNotFound("job".to_string()), -32021);
        test_assert_code(WalletError::InvalidPermissions("x".to_string()), -32021);
        test_assert_code(
            WalletError::RateLimited {
                retry_after_seconds: 1,
            },
            -32022,
        );
        test_assert_code(WalletError::Io("x".to_string()), -32022);
        test_assert_code(WalletError::WalletAlreadyExists, -32024);
        test_assert_code(WalletError::WalletInUse, -32025);
        test_assert_code(WalletError::UnsupportedVersion(1), -32023);
        test_assert_code(WalletError::InvalidParams("x".to_string()), -32602);
    }
}
