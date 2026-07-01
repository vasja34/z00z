/// Generates and signs payment request.
pub fn generate_request(
    keys: &ReceiverKeys,
    params: RequestParams,
    chain_id: u32,
) -> Result<PaymentRequest, PaymentRequestError> {
    let now = current_unix_timestamp()?;
    let mut request = PaymentRequest {
        version: REQ_VER_1,
        owner_handle: keys.owner_handle,
        view_pk: keys
            .view_pk
            .as_bytes()
            .try_into()
            .map_err(|_| PaymentRequestError::InvalidPublicKey)?,
        identity_pk: keys
            .identity_pk
            .as_bytes()
            .try_into()
            .map_err(|_| PaymentRequestError::InvalidPublicKey)?,
        req_id: generate_req_id()?,
        chain_id,
        amount: params.amount,
        expiry: now.saturating_add(params.expiry_seconds),
        metadata: Some(RequestMetadata {
            memo: params.memo,
            payment_id: params.payment_id,
            min_confirmations: None,
            return_receiver: None,
            created_at: now,
        }),
        signature: [0u8; 64],
    };

    request.sign(keys.reveal_identity_sk())?;
    Ok(request)
}

/// Generates random request identifier.
pub fn generate_req_id() -> Result<[u8; 32], PaymentRequestError> {
    let provider = SystemRngProvider;
    let mut rng = provider.rng();
    let mut req_id = [0u8; 32];
    rng.fill_bytes_ext(&mut req_id);

    if req_id == [0u8; 32] {
        return Err(PaymentRequestError::RngFailure);
    }

    Ok(req_id)
}

/// Encodes request as compact URL-safe base64.
pub fn encode_request_compact(request: &PaymentRequest) -> String {
    URL_SAFE_NO_PAD.encode(request.canonical_encoding())
}

/// Decodes request from compact URL-safe base64.
pub fn decode_request_compact(encoded: &str) -> Result<PaymentRequest, PaymentRequestError> {
    let raw = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| PaymentRequestError::InvalidCompact)?;
    PaymentRequest::from_untrusted_bytes(&raw)
}

/// Encodes payment request as NFC record.
pub fn to_nfc_ndef(request: &PaymentRequest) -> NdefRecord {
    let qr_data = request.to_qr_code_data();
    NdefRecord::new_uri(format!("z00z:pay?data={qr_data}"))
}

/// Creates merchant-focused signed payment request.
pub fn create_invoice_for_merchant(
    card: &ReceiverCard,
    identity_sk: &Z00ZScalar,
    chain_id: u32,
    amount: u64,
    memo: Option<String>,
) -> Result<PaymentRequest, PaymentRequestError> {
    let now = current_unix_timestamp()?;
    let mut request = PaymentRequest {
        version: REQ_VER_1,
        owner_handle: card.owner_handle,
        view_pk: card.view_pk,
        identity_pk: card.identity_pk,
        req_id: generate_req_id()?,
        chain_id,
        amount: Some(amount),
        expiry: now.saturating_add(24 * 3600),
        metadata: Some(RequestMetadata {
            memo,
            payment_id: None,
            min_confirmations: None,
            return_receiver: None,
            created_at: now,
        }),
        signature: [0u8; 64],
    };

    request.sign(identity_sk)?;
    Ok(request)
}

/// Handles payment request expiry in background.
pub async fn handle_payment_request_expiry(
    request: PaymentRequest,
    on_expiry: impl Fn(&PaymentRequest) + Send + 'static,
) {
    let remaining = request.remaining_seconds();
    if remaining > 0 {
        tokio::time::sleep(tokio::time::Duration::from_secs(remaining as u64)).await;
    }
    on_expiry(&request);
}

/// Generates QR code for request URI.
#[cfg(feature = "qr-codes")]
pub fn to_qr_code(request: &PaymentRequest) -> Result<qrcode::QrCode, PaymentRequestError> {
    let compact = encode_request_compact(request);
    let uri = format!("z00z:pay?data={compact}");
    qrcode::QrCode::new(uri.as_bytes()).map_err(|_| PaymentRequestError::Qr)
}