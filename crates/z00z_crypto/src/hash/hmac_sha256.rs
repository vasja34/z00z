use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::dst;
use crate::CryptoError;

type HmacSha256 = Hmac<Sha256>;

pub fn hmac_sha256(key: &[u8], domain: &str, label: &str, msg: &[u8]) -> [u8; 32] {
    try_hmac_sha256(key, domain, label, msg)
        .expect("HMAC-SHA256 initialization unexpectedly failed")
}

pub fn try_hmac_sha256(
    key: &[u8],
    domain: &str,
    label: &str,
    msg: &[u8],
) -> Result<[u8; 32], CryptoError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| CryptoError::InvalidParameters { param: "hmac_key" })?;

    let dst_bytes = dst(domain, label);
    let dst_len = (dst_bytes.len() as u64).to_le_bytes();
    mac.update(&dst_len);
    mac.update(&dst_bytes);

    let msg_len = (msg.len() as u64).to_le_bytes();
    mac.update(&msg_len);
    mac.update(msg);

    Ok(mac.finalize().into_bytes().into())
}

pub fn hmac_sha256_raw(key: &[u8], msg: &[u8]) -> [u8; 32] {
    try_hmac_sha256_raw(key, msg).expect("HMAC-SHA256 initialization unexpectedly failed")
}

pub fn try_hmac_sha256_raw(key: &[u8], msg: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| CryptoError::InvalidParameters { param: "hmac_key" })?;
    mac.update(msg);
    Ok(mac.finalize().into_bytes().into())
}

pub fn verify_hmac(key: &[u8], domain: &str, label: &str, msg: &[u8], expected: &[u8; 32]) -> bool {
    let mut mac = match HmacSha256::new_from_slice(key) {
        Ok(mac) => mac,
        Err(_) => return false,
    };

    let dst_bytes = dst(domain, label);
    let dst_len = (dst_bytes.len() as u64).to_le_bytes();
    mac.update(&dst_len);
    mac.update(&dst_bytes);

    let msg_len = (msg.len() as u64).to_le_bytes();
    mac.update(&msg_len);
    mac.update(msg);

    mac.verify_slice(expected).is_ok()
}
