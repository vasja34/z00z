use z00z_crypto::{hash_zk::hash_to_scalar_zk, hash_zk::hash_zk, Z00ZRistrettoPoint, Z00ZScalar};
use z00z_utils::rng::{RngCoreExt, SystemRngProvider};

use crate::domains::{
    RecoverRDomain, RetryDigestDomain, SenderSaltDomain, WalletEphemeralRHashProdDomain,
};
use crate::key::ReceiverSecret;

use crate::stealth::StealthError;

/// Generate hedged ephemeral scalar.
pub fn generate_r_hedged(
    sender_secret_salt: &[u8; 32],
    tx_digest: &[u8; 32],
    out_index: u32,
) -> Result<Z00ZScalar, StealthError> {
    // SECURITY: r is generated fresh per output (IND-CPA). NEVER reuse r.
    let provider = SystemRngProvider;
    let mut rng = provider.rng();
    let mut rng_bytes = [0u8; 32];
    rng.fill_bytes_ext(&mut rng_bytes);
    derive_r_hedged(&rng_bytes, sender_secret_salt, tx_digest, out_index)
}

/// Derive deterministic sender salt from receiver secret.
pub fn derive_sender_salt(receiver_secret: &ReceiverSecret) -> [u8; 32] {
    hash_zk::<SenderSaltDomain>("SENDER_SALT", &[receiver_secret.as_bytes()])
}

/// Generate independent random sender salt.
pub fn generate_sender_salt() -> [u8; 32] {
    let provider = SystemRngProvider;
    let mut rng = provider.rng();
    let mut salt = [0u8; 32];

    loop {
        rng.fill_bytes_ext(&mut salt);
        if salt != [0u8; 32] {
            return salt;
        }
    }
}

/// Get cryptographically secure random bytes.
pub fn get_rng_bytes() -> [u8; 32] {
    let provider = SystemRngProvider;
    let mut rng = provider.rng();
    let mut bytes = [0u8; 32];

    loop {
        rng.fill_bytes_ext(&mut bytes);
        if bytes != [0u8; 32] {
            return bytes;
        }
    }
}

/// Recover deterministic ephemeral scalar from wallet secret inputs.
pub fn recover_r(
    wallet_secret: &[u8; 32],
    ephemeral_seed: &[u8; 32],
    output_index: u32,
) -> Result<Z00ZScalar, StealthError> {
    let out = output_index.to_le_bytes();
    let r =
        hash_to_scalar_zk::<RecoverRDomain>("RECOVER_R", &[wallet_secret, ephemeral_seed, &out])
            .map_err(|_| StealthError::InvalidEphemeralScalar)?;

    if r.as_bytes() == [0u8; 32] {
        return Err(StealthError::ZeroScalarRejected);
    }

    Ok(r)
}

/// Compute ephemeral public point `R_pub = r * G`.
pub fn compute_r_pub(r: &Z00ZScalar) -> Result<Z00ZRistrettoPoint, StealthError> {
    let r_pub = Z00ZRistrettoPoint::from_secret_key(r);
    if r_pub.as_bytes() == [0u8; 32] {
        return Err(StealthError::IdentityPointRejected);
    }
    Ok(r_pub)
}

/// Derive a hedged ephemeral scalar from RNG bytes and sender-bound transaction context.
///
/// Returns an error if scalar derivation fails or yields the zero scalar.
pub fn derive_r_hedged(
    rng_bytes: &[u8; 32],
    sender_secret_salt: &[u8; 32],
    tx_digest: &[u8; 32],
    out_index: u32,
) -> Result<Z00ZScalar, StealthError> {
    let out_bytes = out_index.to_le_bytes();
    let scalar = hash_to_scalar_zk::<WalletEphemeralRHashProdDomain>(
        "R",
        &[rng_bytes, sender_secret_salt, tx_digest, &out_bytes],
    )
    .map_err(|_| StealthError::InvalidEphemeralScalar)?;

    if scalar.as_bytes() == [0u8; 32] {
        return Err(StealthError::ZeroScalarRejected);
    }

    Ok(scalar)
}

/// Generate retry scalar with a retry-bound digest.
pub fn generate_r_retry(
    rng_bytes: &[u8; 32],
    sender_secret_salt: &[u8; 32],
    tx_digest: &[u8; 32],
    out_index: u32,
    retry_index: u32,
) -> Result<Z00ZScalar, StealthError> {
    let retry_bytes = retry_index.to_le_bytes();
    let retry_digest = hash_zk::<RetryDigestDomain>("R_RETRY", &[tx_digest, &retry_bytes]);
    derive_r_hedged(rng_bytes, sender_secret_salt, &retry_digest, out_index)
}

#[cfg(test)]
mod tests {
    use super::{
        compute_r_pub, derive_r_hedged, derive_sender_salt, generate_r_hedged, generate_r_retry,
        generate_sender_salt, get_rng_bytes, recover_r,
    };
    use crate::key::ReceiverSecret;
    use crate::stealth::crypto::encoding;

    #[test]
    fn test_r_uniqueness_per_output() {
        let salt = [1u8; 32];
        let digest = [2u8; 32];
        let a = generate_r_hedged(&salt, &digest, 0).expect("r0");
        let b = generate_r_hedged(&salt, &digest, 1).expect("r1");
        assert_ne!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn test_r_deterministic_component() {
        let rng_bytes = [7u8; 32];
        let salt = [3u8; 32];
        let digest = [9u8; 32];

        let a = derive_r_hedged(&rng_bytes, &salt, &digest, 5).expect("a");
        let b = derive_r_hedged(&rng_bytes, &salt, &digest, 5).expect("b");
        assert_eq!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn test_r_rng_failure_protection() {
        let rng_bytes = [0u8; 32];
        let salt = [4u8; 32];
        let digest = [8u8; 32];

        let a = derive_r_hedged(&rng_bytes, &salt, &digest, 0).expect("a");
        let b = derive_r_hedged(&rng_bytes, &salt, &digest, 1).expect("b");
        assert_ne!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn test_r_never_zero() {
        let salt = [5u8; 32];
        let digest = [6u8; 32];

        for idx in 0..64 {
            let r = generate_r_hedged(&salt, &digest, idx).expect("r");
            assert_ne!(r.as_bytes(), &[0u8; 32]);
        }
    }

    #[test]
    fn test_r_pub_encoding_roundtrip() {
        let salt = [11u8; 32];
        let digest = [12u8; 32];
        let r = generate_r_hedged(&salt, &digest, 2).expect("r");
        let r_pub = compute_r_pub(&r).expect("r_pub");

        let encoded = encoding::encode_r_pub(&r_pub);
        let decoded = encoding::decode_r_pub(&encoded).expect("decode");
        assert_eq!(decoded.as_bytes(), r_pub.as_bytes());
    }

    #[test]
    fn test_r_pub_not_identity() {
        let salt = [13u8; 32];
        let digest = [14u8; 32];
        let r = generate_r_hedged(&salt, &digest, 3).expect("r");
        let r_pub = compute_r_pub(&r).expect("r_pub");
        assert_ne!(r_pub.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_r_pub_canonical_encoding() {
        let salt = [15u8; 32];
        let digest = [16u8; 32];
        let r = generate_r_hedged(&salt, &digest, 4).expect("r");
        let r_pub = compute_r_pub(&r).expect("r_pub");

        let encoded_a = encoding::encode_r_pub(&r_pub);
        let encoded_b = encoding::encode_r_pub(&r_pub);
        assert_eq!(encoded_a, encoded_b);
    }

    #[test]
    fn test_sender_salt_deterministic() {
        let secret = ReceiverSecret::generate().expect("secret");
        let first = derive_sender_salt(&secret);
        let second = derive_sender_salt(&secret);
        assert_eq!(first, second);
    }

    #[test]
    fn test_sender_salt_independent() {
        let first = generate_sender_salt();
        let second = generate_sender_salt();
        assert_ne!(first, [0u8; 32]);
        assert_ne!(second, [0u8; 32]);
        assert_ne!(first, second);
    }

    #[test]
    fn test_rng_bytes_entropy() {
        let first = get_rng_bytes();
        let second = get_rng_bytes();
        assert_ne!(first, [0u8; 32]);
        assert_ne!(second, [0u8; 32]);
        assert_ne!(first, second);
    }

    #[test]
    fn test_recover_r_deterministic() {
        let wallet_secret = [1u8; 32];
        let eph_seed = [2u8; 32];

        let first = recover_r(&wallet_secret, &eph_seed, 7).expect("first");
        let second = recover_r(&wallet_secret, &eph_seed, 7).expect("second");
        assert_eq!(first.as_bytes(), second.as_bytes());
    }

    #[test]
    fn test_r_retry_changes_digest() {
        let rng_bytes = [19u8; 32];
        let salt = [17u8; 32];
        let digest = [18u8; 32];

        let first = generate_r_retry(&rng_bytes, &salt, &digest, 1, 1).expect("first");
        let second = generate_r_retry(&rng_bytes, &salt, &digest, 1, 2).expect("second");
        assert_ne!(first.as_bytes(), second.as_bytes());

        let first_again = generate_r_retry(&rng_bytes, &salt, &digest, 1, 1).expect("first again");
        assert_eq!(first.as_bytes(), first_again.as_bytes());
    }
}
