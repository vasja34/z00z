//! Public wallet ECDH helpers for stealth flows.
//!
//! Phase 1 keeps this surface as the documented wallet/app entrypoint while
//! formula ownership remains split between crypto point ECDH and wallet runtime ECDH.

pub use super::crypto::{
    ecdh::{compute_dh_receiver, compute_dh_sender},
    encoding::{decode_public_key, decode_r_pub, encode_public_key, encode_r_pub},
    ephemeral::{compute_r_pub, derive_r_hedged, generate_r_hedged, recover_r},
};

use z00z_crypto::{
    protocol::ecdh::{
        compute_stealth_dh_sender, generate_ephemeral_keypair, recover_stealth_dh_receiver,
    },
    Z00ZRistrettoPoint, Z00ZScalar,
};

use crate::wallet::WalletError;

/// Sender-side ECDH derivation output.
pub struct SenderDhResult {
    /// Ephemeral scalar used for this output.
    pub r: Z00ZScalar,
    /// Ephemeral public key `R_pub = r * G`.
    pub r_pub: Z00ZRistrettoPoint,
    /// Shared point `dh = r * view_pk`.
    pub dh: Z00ZRistrettoPoint,
}

/// Derive sender ECDH tuple `(r, R_pub, dh)` with deterministic scalar input.
pub fn sender_derive_dh_with_r(
    view_pk: &Z00ZRistrettoPoint,
    r: &Z00ZScalar,
) -> Result<SenderDhResult, WalletError> {
    if view_pk.is_identity() {
        return Err(WalletError::IdentityPointNotAllowed);
    }
    if r.is_zero() {
        return Err(WalletError::CryptoError("zero scalar rejected".to_string()));
    }

    let r_pub = generate_ephemeral_keypair(r).map_err(map_stealth_crypto_err)?;
    let dh = compute_stealth_dh_sender(r, view_pk).map_err(map_stealth_crypto_err)?;

    Ok(SenderDhResult {
        r: r.dangerous_clone(),
        r_pub,
        dh,
    })
}

/// Derive receiver shared point `dh = view_sk * R_pub`.
pub fn receiver_derive_dh(
    view_sk: &Z00ZScalar,
    r_pub: &Z00ZRistrettoPoint,
) -> Result<Z00ZRistrettoPoint, WalletError> {
    if r_pub.is_identity() {
        return Err(WalletError::IdentityPointNotAllowed);
    }

    recover_stealth_dh_receiver(view_sk, r_pub).map_err(map_stealth_crypto_err)
}

fn map_stealth_crypto_err(err: z00z_crypto::CryptoError) -> WalletError {
    match err {
        z00z_crypto::CryptoError::IdentityPoint => WalletError::IdentityPointNotAllowed,
        other => WalletError::CryptoError(other.to_string()),
    }
}
