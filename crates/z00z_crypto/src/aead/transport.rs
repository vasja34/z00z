use super::{
    build_aad, open, seal, MIN_ENVELOPE_SIZE, POLY1305_TAG_SIZE, XCHACHA20_POLY1305_ID,
    XCHACHA_NONCE_SIZE,
};
use crate::CryptoError;

const ASSET_PACK_DOMAIN: &str = "z00z.asset.pack.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetPackCt {
    pub nonce: [u8; XCHACHA_NONCE_SIZE],
    pub ciphertext: Vec<u8>,
}

fn build_pack_aad(metadata: &[u8]) -> Vec<u8> {
    build_aad(ASSET_PACK_DOMAIN, metadata)
}

fn parse_envelope(envelope: &[u8]) -> Result<AssetPackCt, CryptoError> {
    if envelope.len() < MIN_ENVELOPE_SIZE {
        return Err(CryptoError::InvalidParameters { param: "envelope" });
    }

    if envelope[0] != XCHACHA20_POLY1305_ID {
        return Err(CryptoError::InvalidParameters { param: "algorithm" });
    }

    let mut nonce = [0u8; XCHACHA_NONCE_SIZE];
    nonce.copy_from_slice(&envelope[1..1 + XCHACHA_NONCE_SIZE]);
    let ciphertext = envelope[1 + XCHACHA_NONCE_SIZE..].to_vec();

    if ciphertext.len() < POLY1305_TAG_SIZE {
        return Err(CryptoError::InvalidParameters { param: "envelope" });
    }

    Ok(AssetPackCt { nonce, ciphertext })
}

fn compose_envelope(ct: &AssetPackCt) -> Vec<u8> {
    let mut envelope = Vec::with_capacity(1 + XCHACHA_NONCE_SIZE + ct.ciphertext.len());
    envelope.push(XCHACHA20_POLY1305_ID);
    envelope.extend_from_slice(&ct.nonce);
    envelope.extend_from_slice(&ct.ciphertext);
    envelope
}

pub fn encrypt_asset_package_transport(
    key: &[u8; 32],
    metadata: &[u8],
    payload: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let aad = build_pack_aad(metadata);
    seal(key, &aad, payload)
}

pub fn decrypt_asset_package_transport(
    key: &[u8; 32],
    metadata: &[u8],
    envelope: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let aad = build_pack_aad(metadata);
    open(key, &aad, envelope)
}

pub fn encrypt_asset_pack(
    key: &[u8; 32],
    metadata: &[u8],
    payload: &[u8],
) -> Result<AssetPackCt, CryptoError> {
    let envelope = encrypt_asset_package_transport(key, metadata, payload)?;
    parse_envelope(&envelope)
}

pub fn decrypt_asset_pack(
    key: &[u8; 32],
    metadata: &[u8],
    ct: &AssetPackCt,
) -> Result<Vec<u8>, CryptoError> {
    let envelope = compose_envelope(ct);
    decrypt_asset_package_transport(key, metadata, &envelope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let aad = b"additional_data";
        let plaintext = b"secret_message";

        let ciphertext = encrypt_asset_pack(&key, aad, plaintext).unwrap();
        let recovered = decrypt_asset_pack(&key, aad, &ciphertext).unwrap();

        assert_eq!(recovered.as_slice(), plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = [0x42u8; 32];
        let key2 = [0x43u8; 32];
        let aad = b"aad";
        let pt = b"payload";

        let ct = encrypt_asset_pack(&key1, aad, pt).unwrap();
        let result = decrypt_asset_pack(&key2, aad, &ct);

        assert!(result.is_err(), "wrong key must fail");
    }

    #[test]
    fn test_wrong_aad_fails() {
        let key = [0x42u8; 32];
        let aad1 = b"aad_1";
        let aad2 = b"aad_2";
        let pt = b"payload";

        let ct = encrypt_asset_pack(&key, aad1, pt).unwrap();
        let result = decrypt_asset_pack(&key, aad2, &ct);

        assert!(result.is_err(), "wrong AAD must fail");
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key = [0x42u8; 32];
        let aad = b"aad";
        let pt = b"payload";

        let ct1 = encrypt_asset_pack(&key, aad, pt).unwrap();
        let ct2 = encrypt_asset_pack(&key, aad, pt).unwrap();

        assert_ne!(ct1.nonce, ct2.nonce, "nonces must be unique");
    }

    #[test]
    fn test_tamper_detection() {
        let key = [0x42u8; 32];
        let aad = b"aad";
        let pt = b"payload";

        let mut ct = encrypt_asset_pack(&key, aad, pt).unwrap();
        ct.ciphertext[0] ^= 0x01;

        let result = decrypt_asset_pack(&key, aad, &ct);
        assert!(result.is_err(), "tampered ciphertext must be rejected");
    }
}
