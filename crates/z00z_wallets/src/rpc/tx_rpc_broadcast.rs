use crate::chain::BroadcastError;

pub(crate) const TX_BROADCAST_MAX_RETRIES: u32 = 3;

pub(crate) fn try_parse_tx_bytes(tx_data: &str) -> Result<Vec<u8>, String> {
    let trimmed = tx_data.trim();
    let without_prefix = trimmed.strip_prefix("0x").unwrap_or(trimmed);

    let is_hex = !without_prefix.is_empty()
        && without_prefix.bytes().all(|byte| byte.is_ascii_hexdigit())
        && without_prefix.len().is_multiple_of(2);

    if is_hex {
        return z00z_crypto::expert::encoding::from_hex(without_prefix)
            .map_err(|error| format!("Invalid tx_data hex: {error}"));
    }

    Ok(trimmed.as_bytes().to_vec())
}

pub(crate) fn is_retryable_broadcast_error(err: &BroadcastError) -> bool {
    matches!(err, BroadcastError::Network(_) | BroadcastError::Timeout)
}

pub(crate) fn run_with_retry<F>(max_retries: u32, mut op: F) -> (u32, Result<(), BroadcastError>)
where
    F: FnMut() -> Result<(), BroadcastError>,
{
    let max_attempts = max_retries.saturating_add(1);
    let mut attempts: u32 = 0;
    loop {
        attempts = attempts.saturating_add(1);
        match op() {
            Ok(()) => return (attempts, Ok(())),
            Err(err) => {
                if attempts >= max_attempts || !is_retryable_broadcast_error(&err) {
                    return (attempts, Err(err));
                }
            }
        }
    }
}
