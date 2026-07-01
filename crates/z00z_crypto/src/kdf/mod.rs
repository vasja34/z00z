use hkdf::Hkdf;
use sha2::Sha256;
use thiserror::Error;

use crate::{
    domains::{
        AssetIdDomain, HashToScalarDomain, LeafAdDomain, OwnerTagDomain, ReceiverIdDomain,
        ViewKeyDomain,
    },
    error::CryptoError,
    hash::frame_bytes,
    hash_zk::{hash_to_scalar_zk, hash_zk},
    types::{Z00ZRistrettoPoint, Z00ZScalar},
};

mod argon2_kdf;
mod argon2_params;
mod hkdf_kdf;
pub mod kdf_domains;
mod secret_bytes;

#[derive(Debug, Error)]
pub enum KdfError {
    #[error("Argon2id parameter error")]
    Argon2Params,
    #[error("Argon2id execution error")]
    Argon2Execution,
    #[error("HKDF expansion error")]
    HkdfExpansion,
    #[error("HKDF info must be non-empty")]
    HkdfInfoEmpty,
    #[error("HKDF requires salt when IKM has low entropy (< 32 bytes)")]
    HkdfSaltRequired,
}

pub use argon2_kdf::derive_argon2id32_key;
pub use argon2_params::{
    Argon2Params, MAX_ARGON2_TOTAL_COST, MAX_KDF_TIME_MS, MAX_MEM_LIMIT_KIB, MAX_OPS_LIMIT,
    MAX_PARALLELISM,
};
pub use hkdf_kdf::hkdf_expand_32;
pub use secret_bytes::SecretBytes32;

const H2S_LABEL: &str = "H2S";
const KDF_CONSENSUS_SALT: &[u8] = b"z00z.consensus.kdf.v1";
const KDF_WALLET_SALT: &[u8] = b"z00z.wallet.kdf.v1";
const KDF_WALLET_VARIABLE_SALT: &[u8] = b"z00z.wallet.kdf.variable.v1";

fn encode_h2s_input(domain: &[u8], data: &[&[u8]]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&frame_bytes(domain));
    for chunk in data {
        encoded.extend_from_slice(&frame_bytes(chunk));
    }
    encoded
}

fn hkdf_expand_crypto(
    salt: &[u8],
    info: &[u8],
    ikm: &[u8],
    out: &mut [u8],
) -> Result<(), CryptoError> {
    if info.is_empty() {
        return Err(CryptoError::InvalidParameters { param: "kdf_info" });
    }
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    hk.expand(info, out)
        .map_err(|_| CryptoError::InvalidParameters {
            param: "kdf_expand",
        })
}

pub fn derive_view_sk(receiver_secret: &[u8; 32]) -> Result<Z00ZScalar, CryptoError> {
    hash_to_scalar_zk::<ViewKeyDomain>("", &[receiver_secret])
}

pub fn derive_owner_handle(receiver_secret: &[u8; 32]) -> [u8; 32] {
    hash_zk::<ReceiverIdDomain>("", &[receiver_secret])
}

pub fn compute_owner_tag(owner_handle: &[u8; 32], k_dh: &[u8; 32]) -> [u8; 32] {
    hash_zk::<OwnerTagDomain>("", &[owner_handle, k_dh])
}

pub fn derive_asset_id(s_out: &[u8; 32]) -> [u8; 32] {
    hash_zk::<AssetIdDomain>("", &[s_out])
}

pub fn derive_leaf_ad(
    asset_id: &[u8; 32],
    serial_id: u32,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
    c_amount: &[u8; 32],
) -> [u8; 32] {
    let serial_bytes = serial_id.to_le_bytes();
    hash_zk::<LeafAdDomain>("", &[asset_id, &serial_bytes, r_pub, owner_tag, c_amount])
}

pub fn hash_to_scalar_domain(domain: &[u8], data: &[&[u8]]) -> Z00ZScalar {
    try_hash_to_scalar_domain(domain, data)
        .expect("hash_to_scalar_domain fallback is forbidden on the stable surface")
}

pub fn try_hash_to_scalar_domain(domain: &[u8], data: &[&[u8]]) -> Result<Z00ZScalar, CryptoError> {
    let encoded = encode_h2s_input(domain, data);
    let scalar = hash_to_scalar_zk::<HashToScalarDomain>(H2S_LABEL, &[&encoded])?;
    if scalar.is_zero() {
        return Err(CryptoError::InvalidScalar);
    }
    Ok(scalar)
}

pub fn derive_view_pk(view_sk: &Z00ZScalar) -> Z00ZRistrettoPoint {
    Z00ZRistrettoPoint::from_secret_key(view_sk)
}

pub fn kdf_consensus(info: &[u8], ikm: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut okm = [0u8; 32];
    hkdf_expand_crypto(KDF_CONSENSUS_SALT, info, ikm, &mut okm)?;
    Ok(okm)
}

pub fn kdf_wallet(info: &[u8], ikm: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut okm = [0u8; 32];
    hkdf_expand_crypto(KDF_WALLET_SALT, info, ikm, &mut okm)?;
    Ok(okm)
}

pub fn kdf_wallet_variable(
    info: &[u8],
    ikm: &[u8],
    out_len: usize,
) -> Result<Vec<u8>, CryptoError> {
    if out_len == 0 {
        return Err(CryptoError::InvalidParameters { param: "out_len" });
    }
    let mut okm = vec![0u8; out_len];
    hkdf_expand_crypto(KDF_WALLET_VARIABLE_SALT, info, ikm, &mut okm)?;
    Ok(okm)
}

pub fn kdf_from_dh(dh: &Z00ZRistrettoPoint) -> Result<[u8; 32], CryptoError> {
    if dh.is_identity() {
        return Err(CryptoError::IdentityPoint);
    }
    let dh_bytes = dh.compress().to_bytes();
    kdf_consensus(b"ecdh/k_dh", &dh_bytes)
}

pub fn derive_pack_key(k_dh: &[u8; 32]) -> Result<[u8; 32], CryptoError> {
    kdf_consensus(b"asset_pack/key", k_dh)
}

pub fn derive_pack_nonce(k_dh: &[u8; 32], seq: u64) -> Result<[u8; 12], CryptoError> {
    let seq_bytes = seq.to_le_bytes();
    let derived = kdf_consensus(b"asset_pack/nonce", &[k_dh.as_slice(), &seq_bytes].concat())?;
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&derived[..12]);
    Ok(nonce)
}

pub fn derive_db_encryption_key(master: &[u8; 32]) -> Result<[u8; 32], CryptoError> {
    kdf_wallet(b"db/encryption", master)
}

pub fn derive_encrypt_and_mac_keys(master: &[u8; 32]) -> Result<([u8; 32], [u8; 32]), CryptoError> {
    let enc = kdf_wallet(b"encrypt", master)?;
    let mac = kdf_wallet(b"mac", master)?;
    Ok((enc, mac))
}

pub fn derive_symmetric_key_from_ecdh(dh: &Z00ZRistrettoPoint) -> Result<[u8; 32], CryptoError> {
    kdf_from_dh(dh)
}

pub fn generate_hedged_r(secret_ctx: &[u8], message: &[u8], add_entropy: &[u8; 32]) -> Z00ZScalar {
    hash_to_scalar_domain(b"hedged_r", &[secret_ctx, message, add_entropy])
}

#[cfg(test)]
mod test_kdf;
