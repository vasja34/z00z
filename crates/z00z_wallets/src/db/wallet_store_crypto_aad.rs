use z00z_crypto::DomainHasher;

use crate::domains::Z00ZRedbWalletAadIdDomain;

use super::{AAD_MASTER_KEY_LABEL, AAD_SECRET_PREFIX};

/// Canonical AAD for wrapping `MASTER_KEY` under `PW_KEY`.
pub fn aad_master_key(wallet_id: &[u8]) -> Vec<u8> {
    let wallet_id_aad = wallet_aad_id(wallet_id);
    let mut out = Vec::with_capacity(16 + AAD_MASTER_KEY_LABEL.len());
    out.extend_from_slice(&wallet_id_aad);
    out.extend_from_slice(AAD_MASTER_KEY_LABEL);
    out
}

/// Canonical AAD for encrypting a secrets record.
pub fn aad_secret(wallet_id: &[u8], secret_name: &str) -> Vec<u8> {
    let wallet_id_aad = wallet_aad_id(wallet_id);
    let name_bytes = secret_name.as_bytes();
    let name_len = (name_bytes.len() as u32).to_le_bytes();

    let mut out = Vec::with_capacity(16 + AAD_SECRET_PREFIX.len() + 4 + name_bytes.len());
    out.extend_from_slice(&wallet_id_aad);
    out.extend_from_slice(AAD_SECRET_PREFIX);
    out.extend_from_slice(&name_len);
    out.extend_from_slice(name_bytes);
    out
}

/// Canonical AAD for encrypting an object record.
pub fn aad_object(wallet_id: &[u8], object_id: u128, payload_version: u16) -> [u8; 34] {
    let wallet_id_aad = wallet_aad_id(wallet_id);
    let mut out = [0u8; 34];
    out[..16].copy_from_slice(&wallet_id_aad);
    out[16..32].copy_from_slice(&object_id.to_be_bytes());
    out[32..34].copy_from_slice(&payload_version.to_be_bytes());
    out
}

/// Derive a fixed-size AEAD binding value from `wallet_id`.
pub fn wallet_aad_id(wallet_id: &[u8]) -> [u8; 16] {
    let hash = DomainHasher::<Z00ZRedbWalletAadIdDomain>::new_with_label("wallet_aad_id")
        .chain(wallet_id)
        .finalize();

    let mut out = [0u8; 16];
    out.copy_from_slice(&hash.as_ref()[..16]);
    out
}
