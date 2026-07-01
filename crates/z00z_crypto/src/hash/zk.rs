use tari_crypto::hashing::DomainSeparation;

use super::{frame_u64_le, policy::poseidon2_hash, typed::ConsensusHash32};
use crate::error::CryptoError;
use crate::types::Z00ZScalar;

pub fn hash_zk<D: DomainSeparation>(context: &'static str, data: &[&[u8]]) -> [u8; 32] {
    let domain = D::domain();
    let mut chunks = Vec::with_capacity(data.len() + 1);
    chunks.push(context.as_bytes());
    chunks.extend_from_slice(data);
    poseidon2_hash(domain.as_bytes(), &chunks)
}

pub fn hash_consensus<D: DomainSeparation>(
    context: &'static str,
    data: &[&[u8]],
) -> ConsensusHash32 {
    ConsensusHash32::from_bytes(hash_zk::<D>(context, data))
}

pub fn hash_to_scalar_zk<D: DomainSeparation>(
    context: &'static str,
    data: &[&[u8]],
) -> Result<Z00ZScalar, CryptoError> {
    let domain = D::domain();
    let mut base = Vec::with_capacity(data.len() + 1);
    base.push(context.as_bytes());
    base.extend_from_slice(data);

    let ctr0 = frame_u64_le(0);
    let ctr1 = frame_u64_le(1);
    let mut in0 = Vec::with_capacity(base.len() + 1);
    in0.extend_from_slice(&base);
    in0.push(&ctr0);

    let mut in1 = Vec::with_capacity(base.len() + 1);
    in1.extend_from_slice(&base);
    in1.push(&ctr1);

    let h0 = poseidon2_hash(domain.as_bytes(), &in0);
    let h1 = poseidon2_hash(domain.as_bytes(), &in1);

    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(&h0);
    buf[32..].copy_from_slice(&h1);

    Z00ZScalar::from_uniform_bytes(&buf).map_err(|_| CryptoError::InvalidScalar)
}
