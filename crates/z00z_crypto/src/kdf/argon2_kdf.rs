use argon2::{Algorithm, Argon2, Params, Version};

use super::{argon2_params::Argon2Params, secret_bytes::SecretBytes32, KdfError};

pub fn derive_argon2id32_key(
    password: &[u8],
    salt: &[u8; 32],
    params: &Argon2Params,
) -> Result<SecretBytes32, KdfError> {
    params.validate_untrusted()?;

    let params = Params::new(
        params.memory,
        params.iterations,
        params.parallelism,
        Some(32),
    )
    .map_err(|_| KdfError::Argon2Params)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; 32];
    argon2
        .hash_password_into(password, salt, &mut output)
        .map_err(|_| KdfError::Argon2Execution)?;

    Ok(SecretBytes32::new(output))
}
