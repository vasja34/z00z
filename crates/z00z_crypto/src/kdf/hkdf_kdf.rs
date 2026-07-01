use hkdf::Hkdf;
use sha2::Sha256;

use super::{secret_bytes::SecretBytes32, KdfError};

pub fn hkdf_expand_32(ikm: &[u8], salt: &[u8], info: &[u8]) -> Result<SecretBytes32, KdfError> {
    if info.is_empty() {
        return Err(KdfError::HkdfInfoEmpty);
    }

    if salt.is_empty() && ikm.len() < 32 {
        return Err(KdfError::HkdfSaltRequired);
    }

    let salt_opt = if salt.is_empty() { None } else { Some(salt) };
    let hkdf = Hkdf::<Sha256>::new(salt_opt, ikm);
    let mut output = [0u8; 32];
    hkdf.expand(info, &mut output)
        .map_err(|_| KdfError::HkdfExpansion)?;
    Ok(SecretBytes32::new(output))
}
