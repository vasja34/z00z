//! Key RPC method definitions - HD key management and receiver operations
//!
//! This module defines the JSON-RPC 2.0 interface for key management operations
//! including HD derivation (BIP44), extended public key export, and receiver-card flows.
//!
//! # Architecture Compliance
//!
//! - ✅ Trait-based interface (KeyRpc trait)
//! - ✅ jsonrpsee integration (#[rpc(server, client)])
//! - ✅ Proper error handling (RpcResult)
//! - ✅ JSON-RPC 2.0 method naming (wallet.key.*)

#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::key::{
    RuntimeCreatePaymentRequestResponse, RuntimeDeriveReceiverResponse,
    RuntimeGetReceiverCardResponse, RuntimeLabelReceiverResponse, RuntimeListReceiversResponse,
    RuntimePaymentRequestMetaInput, RuntimePubMaterialExportResponse, RuntimeReceiverFilter,
    RuntimeRotateKeyResponse, RuntimeValidatePaymentRequestResponse,
    RuntimeValidateReceiverCardResponse,
};

#[cfg(not(target_arch = "wasm32"))]
use super::super::types::wallet::SessionToken;

/// Key Management RPC trait for HD wallet operations
///
/// Provides methods for BIP44 key derivation, receiver-card/payment-request flows,
/// extended public key management, and key rotation.
///
/// # JSON-RPC 2.0 Methods
///
/// Complete list of wallet.key.* methods:
/// - `wallet.key.derive_receiver` - Derive receiver anchor at BIP44 path
/// - `wallet.key.get_receiver_card` - Export signed stealth receiver card
/// - `wallet.key.create_payment_request` - Create signed payment request
/// - `wallet.key.validate_payment_request` - Validate signed payment request
/// - `wallet.key.export_public_material` - Export account pub material (XChaCha20-Poly1305)
/// - `wallet.key.rotate_master_key` - Rotate master key
/// - `wallet.key.list_receivers` - List receiver entries with pagination
/// - `wallet.key.validate_receiver_card` - Validate signed stealth receiver card
/// - `wallet.key.label_receiver` - Add/update receiver label
///
/// # Receiver Model (updated 2026-03-02)
///
/// Z00Z uses stealth ECDH one-time receiver material (`ReceiverCard` + `PaymentRequest`).
/// Active stealth methods:
/// - `wallet.key.derive_receiver`
/// - `wallet.key.get_receiver_card`
/// - `wallet.key.create_payment_request`
/// - `wallet.key.validate_payment_request`
/// - `wallet.key.list_receivers`
/// - `wallet.key.validate_receiver_card`
/// - `wallet.key.label_receiver`
///
/// # Priority: P0 CRITICAL
///
/// HD wallet functionality is CORE requirement, not optional feature.
/// Without key derivation, wallet cannot support:
/// - Multiple accounts (BIP44)
/// - Change outputs
/// - Hardware wallet integration
/// - Deterministic receiver generation
#[cfg(not(target_arch = "wasm32"))]
#[rpc(server, client)]
pub trait KeyRpc {
    /// Derive receiver anchor at BIP44 path
    ///
    /// Derives receiver-facing public material at the specified BIP44 path and
    /// returns the public key plus canonical path echo. Supports standard BIP44 paths:
    /// - m/44'/coin'/account'/change/index
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "wallet.key.derive_receiver",
    ///   "params": {
    ///     "wallet_id": "...",
    ///     "path": "m/44'/0'/0'/0/0"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    ///
    /// # Security
    /// - Requires active session token
    /// - Rate limited: 20 requests/minute
    /// - Audit logged: No (read-only operation)
    ///
    /// # Returns
    /// - `Ok(DeriveReceiverResponse)` - Public key and canonical path
    /// - `Err(RpcError::InvalidParams)` - Invalid BIP44 path
    /// - `Err(RpcError::WalletLocked)` - Wallet not unlocked
    /// - `Err(RpcError::NotFound)` - Wallet not found
    #[method(name = "wallet.key.derive_receiver")]
    async fn derive_receiver(
        &self,
        session: SessionToken,
        path: String,
    ) -> RpcResult<RuntimeDeriveReceiverResponse>;

    /// Get signed stealth receiver card for current wallet session.
    ///
    /// Returns owner handle and public stealth material required by payers
    /// to construct stealth outputs.
    #[method(name = "wallet.key.get_receiver_card")]
    async fn get_receiver_card(
        &self,
        session: SessionToken,
    ) -> RpcResult<RuntimeGetReceiverCardResponse>;

    /// Create signed payment request for current wallet session.
    ///
    /// Produces a compact signed payment request payload bound to current
    /// receiver identity and chain id.
    #[method(name = "wallet.key.create_payment_request")]
    async fn create_payment_request(
        &self,
        session: SessionToken,
        amount: Option<u64>,
        expiry_secs: u64,
        metadata: Option<RuntimePaymentRequestMetaInput>,
    ) -> RpcResult<RuntimeCreatePaymentRequestResponse>;

    /// Validate signed payment request payload.
    ///
    /// Verifies encoding bounds, request signature, expiry, chain binding,
    /// and TOFU identity status.
    #[method(name = "wallet.key.validate_payment_request")]
    async fn validate_payment_request(
        &self,
        session: SessionToken,
        request_compact: String,
    ) -> RpcResult<RuntimeValidatePaymentRequestResponse>;

    /// Export account-level pub material.
    ///
    /// Canonical live public-material export contract using XChaCha20-Poly1305 for the payload
    /// envelope.
    #[method(name = "wallet.key.export_public_material")]
    async fn export_public_material(
        &self,
        session: SessionToken,
        account: u32,
        password: String,
    ) -> RpcResult<RuntimePubMaterialExportResponse>;

    /// Run the current master-key rotation flow.
    ///
    /// The durable rotation contract performs the full authorization, audit,
    /// and rate-limit flow, rewrites persisted wallet encryption state, then
    /// reports the resulting key-material fingerprint and the persisted rewrite
    /// count.
    ///
    /// # JSON-RPC Method
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "wallet.key.rotate_master_key",
    ///   "params": {
    ///     "session": {"wallet_id": "...", "token": "..."},
    ///     "password": "***",
    ///     "confirmation": "ROTATE"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    ///
    /// # Security
    /// - Requires password + confirmation code
    /// - Rate limited: 1 successful confirmed rotation/hour per wallet; password
    ///   and confirmation rejections do not consume the slot
    /// - Audit logged: CRITICAL operation
    /// - Rewrites persisted wallet encryption state under one rotation contract
    ///
    /// # Returns
    /// - `Ok(RotateKeyResponse)` - Current key-material fingerprint after the
    ///   rotation flow plus the count of rewrapped persisted records
    /// - `Err(RpcError::AuthFailed)` - Password incorrect
    /// - `Err(RpcError::InvalidParams)` - Confirmation code wrong
    /// - `Err(RpcError::WalletLocked)` - Wallet not unlocked
    /// - `Err(RpcError::RequestFailed)` - Underlying wallet state was not found
    #[method(name = "wallet.key.rotate_master_key")]
    async fn rotate_master_key(
        &self,
        session: SessionToken,
        password: String,
        confirmation: String,
    ) -> RpcResult<RuntimeRotateKeyResponse>;

    /// List receiver entries with pagination.
    #[method(name = "wallet.key.list_receivers")]
    async fn list_receivers(
        &self,
        session: SessionToken,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeReceiverFilter>,
    ) -> RpcResult<RuntimeListReceiversResponse>;

    /// Validate receiver card checksum.
    ///
    /// Receiver-oriented validation of the compact receiver-card payload.
    #[method(name = "wallet.key.validate_receiver_card")]
    async fn validate_receiver_card(
        &self,
        card_compact: String,
    ) -> RpcResult<RuntimeValidateReceiverCardResponse>;

    /// Add or update a receiver label.
    #[method(name = "wallet.key.label_receiver")]
    async fn label_receiver(
        &self,
        session: SessionToken,
        receiver_id: String,
        label: String,
    ) -> RpcResult<RuntimeLabelReceiverResponse>;
}
