#![deny(clippy::indexing_slicing)]
#![cfg_attr(not(test), warn(clippy::expect_used, clippy::unwrap_used))]

use crate::domains::{AssetIdHashDomain, ChecksumHashDomain, TestNonceDomain};
use crate::{DomainHasher, DomainHasher256};

pub type AssetIdHasher = DomainHasher<AssetIdHashDomain>;
pub type ChecksumHasher = DomainHasher256<ChecksumHashDomain>;
pub type TestNonceHasher = DomainHasher256<TestNonceDomain>;

pub fn try_take_32(bytes: impl AsRef<[u8]>) -> Option<[u8; 32]> {
    let bytes = bytes.as_ref();
    if bytes.len() < 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(bytes.get(..32)?);
    Some(out)
}

pub fn safe_take_32(bytes: impl AsRef<[u8]>) -> Result<[u8; 32], crate::CryptoError> {
    let bytes = bytes.as_ref();
    let prefix = bytes
        .get(..32)
        .ok_or(crate::CryptoError::InvalidParameters {
            param: "hash_truncated",
        })?;

    let mut out = [0u8; 32];
    out.copy_from_slice(prefix);
    Ok(out)
}

pub fn try_take_n<const N: usize>(bytes: impl AsRef<[u8]>) -> Option<[u8; N]> {
    let bytes = bytes.as_ref();
    let prefix = bytes.get(..N)?;

    let mut out = [0u8; N];
    out.copy_from_slice(prefix);
    Some(out)
}

pub fn take_32(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    try_take_32(bytes).unwrap_or([0u8; 32])
}
