use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonrpsee::types::ErrorObjectOwned;

use crate::rpc::types::{
    common::{PersistWalletId, RuntimeValidationResult},
    key::{
        RuntimeCreatePaymentRequestResponse, RuntimePaymentRequestMetaInput,
        RuntimePubMaterialExportResponse, RuntimeReceiverFilter, RuntimeRotateKeyResponse,
        RuntimeValidatePaymentRequestResponse,
    },
    security::{AuditResult, PersistAuditLogEntry, RiskLevel, SessionToken},
};
use crate::services::WalletService;
use crate::{
    domains::WalletKeyFingerprintDomain,
    key::Bip44Path,
    receiver::{PaymentRequest, PaymentRequestError, RequestParams, ValidationOutcome},
    security::encryption::WalletEncryption,
    wallet::WalletError,
    ChainType,
};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::{aead, DomainHasher};
use z00z_utils::rng::{RngCoreExt, SystemRngProvider};

pub(super) fn fingerprint_4(input: &[u8]) -> [u8; 4] {
    let hash = DomainHasher::<WalletKeyFingerprintDomain>::new_with_label("key_fingerprint")
        .chain(input)
        .finalize();

    let mut out = [0u8; 4];
    out.copy_from_slice(&hash.as_ref()[..4]);
    out
}

pub(super) fn invalid_params(message: impl Into<String>) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32602, message.into(), None::<()>)
}

pub(super) fn not_found(message: impl Into<String>) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32004, message.into(), None::<()>)
}

pub(super) async fn audit_event(
    service: &WalletService,
    wallet_id: Option<PersistWalletId>,
    method: &str,
    result: AuditResult,
    context: Option<String>,
) {
    let entry = PersistAuditLogEntry {
        timestamp: service.now_ms(),
        wallet_id,
        method: method.to_string(),
        client_ip: None,
        user_agent: None,
        result,
        risk_level: RiskLevel::Critical,
        context,
    };

    service.push_key_audit_entry(entry).await;
}

pub(super) fn decode_cursor(cursor: &str) -> Result<usize, ErrorObjectOwned> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| invalid_params("Invalid cursor"))?;

    if bytes.len() != 4 {
        return Err(invalid_params("Invalid cursor"));
    }

    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes);
    Ok(u32::from_be_bytes(raw) as usize)
}

pub(super) fn encode_cursor(offset: usize) -> String {
    URL_SAFE_NO_PAD.encode(u32::try_from(offset).unwrap_or(u32::MAX).to_be_bytes())
}

pub(super) fn runtime_chain_id(chain: ChainType) -> u32 {
    match chain {
        ChainType::Mainnet => 1,
        ChainType::Testnet => 2,
        ChainType::Devnet => 3,
    }
}

pub(super) async fn wallet_chain_id(
    service: &WalletService,
    wallet_id: &PersistWalletId,
) -> Result<u32, ErrorObjectOwned> {
    let chain = service
        .resolve_persisted_wallet_chain_type(wallet_id)
        .await
        .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;
    Ok(runtime_chain_id(chain))
}

pub(super) fn parse_payment_id(
    meta: &Option<RuntimePaymentRequestMetaInput>,
) -> Result<Option<[u8; 16]>, ErrorObjectOwned> {
    match meta.as_ref().and_then(|item| item.payment_id.as_ref()) {
        Some(raw_hex) => {
            let bytes = hex::decode(raw_hex)
                .map_err(|_| invalid_params("metadata.payment_id must be valid hex"))?;
            let arr: [u8; 16] = bytes
                .try_into()
                .map_err(|_| invalid_params("metadata.payment_id must be 16 bytes"))?;
            Ok(Some(arr))
        }
        None => Ok(None),
    }
}

pub(super) fn make_req_params(
    amount: Option<u64>,
    expiry_secs: u64,
    meta: Option<RuntimePaymentRequestMetaInput>,
    payment_id: Option<[u8; 16]>,
) -> RequestParams {
    RequestParams {
        amount,
        expiry_seconds: expiry_secs,
        memo: meta.and_then(|item| item.memo),
        payment_id,
    }
}

pub(super) fn req_response(request: &PaymentRequest) -> RuntimeCreatePaymentRequestResponse {
    RuntimeCreatePaymentRequestResponse {
        owner_handle: hex::encode(request.owner_handle),
        view_key: hex::encode(request.view_pk),
        identity_key: hex::encode(request.identity_pk),
        req_id: hex::encode(request.req_id),
        chain_id: request.chain_id,
        amount: request.amount,
        expiry: request.expiry,
        signature: hex::encode(request.signature),
        request_compact: crate::receiver::request::encode_request_compact(request),
    }
}

pub(super) fn map_req_decode_err(err: PaymentRequestError) -> ErrorObjectOwned {
    match err {
        PaymentRequestError::InvalidRequestSize => {
            invalid_params("REQUEST_PAYLOAD_OVERSIZED_OR_EMPTY")
        }
        PaymentRequestError::InvalidCompact
        | PaymentRequestError::InvalidRequestBytes
        | PaymentRequestError::InvalidRequestFlag
        | PaymentRequestError::InvalidRequestString
        | PaymentRequestError::InvalidPublicKey
        | PaymentRequestError::IdentityPoint => invalid_params("REQUEST_PAYLOAD_MALFORMED"),
        _ => invalid_params("REQUEST_PAYLOAD_MALFORMED"),
    }
}

pub(super) fn map_req_validate_err(err: PaymentRequestError) -> ErrorObjectOwned {
    match err {
        PaymentRequestError::WrongChainId => invalid_params("REQUEST_CHAIN_MISMATCH"),
        PaymentRequestError::RequestExpired => invalid_params("REQUEST_EXPIRED"),
        PaymentRequestError::VerifyFailed | PaymentRequestError::InvalidSignature => {
            invalid_params("REQUEST_INVALID_SIGNATURE")
        }
        PaymentRequestError::PinRevoked => invalid_params("REQUEST_ID_REVOKED"),
        _ => invalid_params("REQUEST_VALIDATION_FAILED"),
    }
}

pub(super) fn outcome_text(outcome: ValidationOutcome) -> String {
    match outcome {
        ValidationOutcome::Approved => "approved",
        ValidationOutcome::RequiresUserConfirmation => "confirm",
        ValidationOutcome::IdentityMismatch => "id_mismatch",
    }
    .to_string()
}

pub(super) fn validate_limit(limit: Option<usize>) -> Result<usize, ErrorObjectOwned> {
    let limit = limit.unwrap_or(50);
    if limit == 0 || limit > 100 {
        return Err(invalid_params("Invalid limit (must be 1..=100)"));
    }
    Ok(limit)
}

pub(super) fn apply_receiver_filter(
    entries: &mut Vec<(Bip44Path, [u8; 32])>,
    filter: Option<&RuntimeReceiverFilter>,
) {
    if let Some(filter) = filter {
        entries.retain(|(path, _)| match filter.change {
            Some(change) => change == (path.change().index() == 1),
            None => true,
        });
    }
}

pub(crate) async fn build_pub_export(
    service: &WalletService,
    wallet_id: &PersistWalletId,
    account: u32,
    password: String,
) -> Result<RuntimePubMaterialExportResponse, ErrorObjectOwned> {
    let safe_pw = SafePassword::from(password);
    let pub_material = service
        .derive_account_pub_material(wallet_id, account)
        .await
        .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;

    let pub_plain = format!(
        "z00z-pub-material-v1:account={}:pubkey={}",
        account,
        hex::encode(pub_material)
    );

    const AAD_DOMAIN: &str = "z00z.wallet.key.export_public_material";
    let mut aad_ctx = Vec::with_capacity(wallet_id.0.len() + 1 + 4);
    aad_ctx.extend_from_slice(wallet_id.0.as_bytes());
    aad_ctx.push(0);
    aad_ctx.extend_from_slice(&account.to_le_bytes());
    let aad = aead::build_aad_multipart(AAD_DOMAIN, &[aad_ctx.as_slice()])
        .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;

    let rng_provider = SystemRngProvider;
    let mut rng = rng_provider.rng();
    let mut salt = [0u8; 16];
    rng.fill_bytes_ext(&mut salt);

    let mut key = WalletEncryption::derive_key(&safe_pw, &salt)
        .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;
    let envelope = aead::seal(&key, &aad, pub_plain.as_bytes())
        .map_err(|e| ErrorObjectOwned::owned(-32603, e.to_string(), None::<()>))?;
    key.fill(0);

    let mut packed = Vec::with_capacity(16 + envelope.len());
    packed.extend_from_slice(&salt);
    packed.extend_from_slice(&envelope);

    Ok(RuntimePubMaterialExportResponse {
        schema_version: 2,
        encrypted_pub_material: base64::engine::general_purpose::STANDARD.encode(packed),
        algorithm: "xchacha20poly1305".to_string(),
        account,
        fingerprint: hex::encode(fingerprint_4(&pub_material)),
    })
}

pub(crate) async fn check_rotate_confirm(
    service: &WalletService,
    wallet_id: &PersistWalletId,
    confirmation: &str,
) -> Result<(), ErrorObjectOwned> {
    if confirmation == "ROTATE" {
        return Ok(());
    }

    audit_event(
        service,
        Some(wallet_id.clone()),
        "wallet.key.rotate_master_key",
        AuditResult::Denied,
        Some("secondary_confirmation_mismatch".to_string()),
    )
    .await;

    Err(invalid_params(
        "Invalid confirmation (expected literal 'ROTATE')",
    ))
}

pub(crate) async fn check_rotate_password(
    service: &WalletService,
    wallet_id: &PersistWalletId,
    password: &SafePassword,
) -> Result<(), ErrorObjectOwned> {
    match service
        .confirm_wallet_password_with_backoff(wallet_id, password)
        .await
    {
        Ok(()) => Ok(()),
        Err(error @ WalletError::InvalidPassword) => {
            audit_event(
                service,
                Some(wallet_id.clone()),
                "wallet.key.rotate_master_key",
                AuditResult::Denied,
                Some("password_mismatch".to_string()),
            )
            .await;
            Err(crate::rpc::error_mapping::map_wallet_error_to_rpc(error))
        }
        Err(error) => Err(crate::rpc::error_mapping::map_wallet_error_to_rpc(error)),
    }
}

pub(crate) async fn finish_rotate(
    service: &WalletService,
    session: &SessionToken,
    password: &SafePassword,
) -> Result<RuntimeRotateKeyResponse, ErrorObjectOwned> {
    let outcome = service
        .rotate_master_key_persisted(session, password)
        .await
        .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;

    Ok(RuntimeRotateKeyResponse {
        new_fingerprint: hex::encode(outcome.new_fingerprint),
        rotated_at: service.now_ms(),
        records_rewrapped: outcome.records_rewrapped,
    })
}

pub(crate) fn validate_req_response(
    request: &PaymentRequest,
    outcome: ValidationOutcome,
) -> RuntimeValidatePaymentRequestResponse {
    RuntimeValidatePaymentRequestResponse {
        result: RuntimeValidationResult::valid(),
        outcome: Some(outcome_text(outcome)),
        req_id: Some(hex::encode(request.req_id)),
        owner_handle: Some(hex::encode(request.owner_handle)),
        expiry: Some(request.expiry),
    }
}
