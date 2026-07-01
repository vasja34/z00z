use crate::{
    domains::{RangeCtxDomain, StealthLeafAdProdDomain, StealthTag16ProdDomain},
    frame_bytes, frame_u32_le,
    hash_zk::hash_zk,
};

pub const LEAF_PREIMAGE_SIZE: usize = 132;

pub fn compute_tag16(k_dh: &[u8; 32], leaf_ad: &[u8; 32]) -> u16 {
    let hash = hash_zk::<StealthTag16ProdDomain>("z00z.consensus.tag16.v1", &[k_dh, leaf_ad]);
    u16::from_le_bytes([hash[0], hash[1]])
}

pub fn compute_leaf_ad(
    asset_id: &[u8; 32],
    serial_id: u32,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
    c_amount: &[u8; 32],
) -> [u8; 32] {
    let serial = serial_id.to_le_bytes();
    hash_zk::<StealthLeafAdProdDomain>(
        "z00z.consensus.leaf_ad.v1",
        &[asset_id, &serial, r_pub, owner_tag, c_amount],
    )
}

pub fn encode_leaf_preimage(
    asset_id: &[u8; 32],
    serial_id: u32,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
    c_amount: &[u8; 32],
) -> [u8; LEAF_PREIMAGE_SIZE] {
    let mut out = [0u8; LEAF_PREIMAGE_SIZE];
    out[0..32].copy_from_slice(asset_id);
    out[32..36].copy_from_slice(&serial_id.to_le_bytes());
    out[36..68].copy_from_slice(r_pub);
    out[68..100].copy_from_slice(owner_tag);
    out[100..132].copy_from_slice(c_amount);
    out
}

pub fn range_ctx_hash(
    asset_id: &[u8; 32],
    chain_id: u32,
    root_ver: u8,
    proof_ver: u8,
    policy_ver: u32,
    commitment: &[u8; 32],
    proof_bytes: &[u8],
) -> [u8; 32] {
    let chain_id = frame_u32_le(chain_id);
    let policy_ver = frame_u32_le(policy_ver);
    let root_ver = [root_ver];
    let proof_ver = [proof_ver];
    let proof_bytes = frame_bytes(proof_bytes);

    hash_zk::<RangeCtxDomain>(
        "",
        &[
            asset_id,
            &chain_id,
            &root_ver,
            &proof_ver,
            &policy_ver,
            commitment,
            proof_bytes.as_slice(),
        ],
    )
}
