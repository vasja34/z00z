use blake2::{
    digest::consts::{U32, U64},
    Blake2b, Blake2bVar, Digest as _,
};
use tari_crypto::hashing::{DomainSeparatedHasher, DomainSeparation};

use super::{chain_len_prefixed, dst};
use crate::domains::{AssetIdHashDomain, GenericDeriveDomain};

pub fn blake2b_256(domain: &str, label: &str, parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Blake2b::<U32>::new();
    let dst = dst(domain, label);
    chain_len_prefixed(&mut hasher, &dst);
    for part in parts {
        chain_len_prefixed(&mut hasher, part);
    }
    hasher.finalize().into()
}

pub fn blake2b_512(domain: &str, label: &str, parts: &[&[u8]]) -> [u8; 64] {
    let mut hasher = Blake2b::<U64>::new();
    let dst = dst(domain, label);
    chain_len_prefixed(&mut hasher, &dst);
    for part in parts {
        chain_len_prefixed(&mut hasher, part);
    }
    hasher.finalize().into()
}

pub fn blake2b_256_simple(data: &[u8]) -> [u8; 32] {
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn blake2b_512_simple(data: &[u8]) -> [u8; 64] {
    let mut hasher = Blake2b::<U64>::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn derive_key_from_seed<D: DomainSeparation>(seed: &[u8], context: &'static str) -> [u8; 32] {
    let hash = DomainSeparatedHasher::<Blake2b<U64>, D>::new_with_label(context)
        .chain(seed)
        .finalize();

    let bytes = hash.as_ref();
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes[..32]);
    result
}

pub fn derive_domain_hash(domain: &str, data: &[u8]) -> [u8; 32] {
    let mut hasher = DomainHasher::<GenericDeriveDomain>::new_with_label("derive");

    let domain_len = (domain.len() as u64).to_le_bytes();
    hasher = hasher.chain(domain_len);
    hasher = hasher.chain(domain.as_bytes());

    let data_len = (data.len() as u64).to_le_bytes();
    hasher = hasher.chain(data_len);
    hasher = hasher.chain(data);

    finalize_to_32(hasher)
}

pub fn hash_asset_id(asset_data: &[u8]) -> [u8; 32] {
    let mut hasher = DomainHasher::<AssetIdHashDomain>::new_with_label("asset_id");
    hasher = hasher.chain(asset_data);
    finalize_to_32(hasher)
}

fn finalize_to_32<D: DomainSeparation>(hasher: DomainHasher<D>) -> [u8; 32] {
    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

pub type Blake2bHasher = Blake2b<U64>;
pub type Blake2bHasher256 = Blake2b<U32>;
pub type Blake2bVarHasher = Blake2bVar;
pub type DomainHasher<D> = DomainSeparatedHasher<Blake2bHasher, D>;
pub type DomainHasher256<D> = DomainSeparatedHasher<Blake2bHasher256, D>;
