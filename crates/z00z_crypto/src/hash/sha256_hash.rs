use sha2::{Digest as _, Sha256};

use super::{chain_len_prefixed, dst};

pub fn sha256_256(domain: &str, label: &str, parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    let dst = dst(domain, label);
    chain_len_prefixed(&mut hasher, &dst);
    for part in parts {
        chain_len_prefixed(&mut hasher, part);
    }
    hasher.finalize().into()
}

pub fn sha256_256_simple(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}
