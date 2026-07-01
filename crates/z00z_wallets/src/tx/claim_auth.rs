use z00z_crypto::{claim::ClaimAuthoritySig, Z00ZRistrettoPoint};

#[cfg(any(test, doctest, feature = "claim-auth-sign"))]
use z00z_crypto::expert::keys::RistrettoSecretKey;
#[cfg(any(test, doctest, feature = "claim-auth-sign"))]
use z00z_crypto::expert::traits::SecretKeyTrait;
#[cfg(any(test, doctest, feature = "claim-auth-sign"))]
use z00z_utils::rng::SystemRngProvider;

use super::claim_errors::ClaimTxError;

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_even_hex(value: &str) -> bool {
    value.len().is_multiple_of(2) && is_lower_hex(value)
}

pub(crate) const OWNER_ATTEST_CTX: &[u8] = b"z00z.claim.owner_attest.v1";
pub const CLAIM_AUTH_SIM_CHAIN_ID: u32 = 3;
pub const CLAIM_AUTH_SIM_CHAIN_TYPE: &str = "devnet";
pub const CLAIM_AUTH_SIM_CHAIN_NAME: &str = "z00z-devnet-1";
const CLAIM_AUTH_PK_BYTES: [u8; 32] = [
    0x48, 0x05, 0x19, 0x98, 0xee, 0x9f, 0x83, 0x8e, 0xd0, 0x95, 0x5e, 0x93, 0x51, 0x52, 0x88, 0xf8,
    0x97, 0x00, 0xbd, 0xdd, 0xa2, 0xfd, 0xac, 0xff, 0xe2, 0x82, 0xb2, 0x9e, 0x01, 0xc1, 0x7f, 0x7a,
];
#[cfg(any(test, doctest, feature = "claim-auth-sign"))]
const CLAIM_AUTH_SEED: [u8; 64] = [0xA7u8; 64];

#[cfg(any(test, doctest, feature = "claim-auth-sign"))]
fn claim_auth_sk() -> RistrettoSecretKey {
    RistrettoSecretKey::from_uniform_bytes(&CLAIM_AUTH_SEED)
        .expect("claim authority seed must produce a valid secret key")
}

/// Return the trusted claim authority public key for live claim contract verification.
pub fn claim_auth_pk() -> Z00ZRistrettoPoint {
    Z00ZRistrettoPoint::try_from_bytes(CLAIM_AUTH_PK_BYTES)
        .expect("claim authority public key bytes must stay canonical")
}

/// Enforce the current immutable simulator-only claim authority anchor.
pub fn require_claim_auth_simulator_anchor(
    chain_id: u32,
    chain_type: &str,
    chain_name: &str,
) -> Result<(), String> {
    if chain_id == CLAIM_AUTH_SIM_CHAIN_ID
        && chain_type == CLAIM_AUTH_SIM_CHAIN_TYPE
        && chain_name == CLAIM_AUTH_SIM_CHAIN_NAME
    {
        return Ok(());
    }

    Err(format!(
        "claim authority anchor is simulator-only: expected chain_id={} chain_type={} chain_name={}, got chain_id={} chain_type={} chain_name={}",
        CLAIM_AUTH_SIM_CHAIN_ID,
        CLAIM_AUTH_SIM_CHAIN_TYPE,
        CLAIM_AUTH_SIM_CHAIN_NAME,
        chain_id,
        chain_type,
        chain_name,
    ))
}

/// Sign one canonical claim contract statement with the trusted claim authority key.
#[cfg(any(test, doctest, feature = "claim-auth-sign"))]
pub fn sign_claim_auth(stmt: &z00z_crypto::claim::ClaimStmt) -> Result<ClaimAuthoritySig, String> {
    let auth_sk = claim_auth_sk();
    let mut rng = SystemRngProvider.rng();
    ClaimAuthoritySig::sign(stmt, &auth_sk, &mut rng).map_err(|e| e.to_string())
}

pub(crate) fn decode_claim_auth(value: &str) -> Result<ClaimAuthoritySig, ClaimTxError> {
    if value.is_empty() || !is_even_hex(value) {
        return Err(ClaimTxError::AuthoritySigDecode(
            "claim_authority_sig_hex invalid".to_string(),
        ));
    }
    let sig_bytes = hex::decode(value).map_err(|_| {
        ClaimTxError::AuthoritySigDecode("claim_authority_sig_hex decode failed".to_string())
    })?;
    ClaimAuthoritySig::from_bytes(&sig_bytes)
        .map_err(|e| ClaimTxError::AuthoritySigDecode(e.to_string()))
}
