use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::kdf::{derive_argon2id32_key, hkdf_expand_32, Argon2Params};
use z00z_crypto::{CryptoError, Hidden};

use crate::domains::hashing::{
    redb_wallet_hkdf_info_data, redb_wallet_hkdf_info_index, redb_wallet_hkdf_info_integrity,
};

use super::{KdfParams, RedbKey32, WalletDerivedKeys};

pub(super) fn pad_salt32_zero(salt: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let len = salt.len().min(32);
    out[..len].copy_from_slice(&salt[..len]);
    out
}

// Keep this current zero-padding helper explicit at the live metadata gate so
// persisted KDF validation stays fail-closed.
pub(super) fn derive_key_zero_padding(
    password: &SafePassword,
    params: &KdfParams,
    argon2_params: &Argon2Params,
) -> Result<[u8; 32], CryptoError> {
    let salt32 = pad_salt32_zero(params.salt.as_slice());
    derive_argon2id32_key(password.reveal(), &salt32, argon2_params)
        .map(|secret| secret.into_inner())
        .map_err(|_| CryptoError::CryptoOperationFailed)
}

pub(super) fn derive_wallet_keys(master_key: &RedbKey32) -> Result<WalletDerivedKeys, CryptoError> {
    let data_key = hkdf_expand_32(master_key, &[], &redb_wallet_hkdf_info_data())
        .map(|secret| secret.into_inner())
        .map_err(|_| CryptoError::CryptoOperationFailed)?;

    let index_key = hkdf_expand_32(master_key, &[], &redb_wallet_hkdf_info_index())
        .map(|secret| secret.into_inner())
        .map_err(|_| CryptoError::CryptoOperationFailed)?;

    let integrity_key = hkdf_expand_32(master_key, &[], &redb_wallet_hkdf_info_integrity())
        .map(|secret| secret.into_inner())
        .map_err(|_| CryptoError::CryptoOperationFailed)?;

    Ok(WalletDerivedKeys {
        data_key: Hidden::hide(data_key),
        index_key: Hidden::hide(index_key),
        integrity_key: Hidden::hide(integrity_key),
    })
}
