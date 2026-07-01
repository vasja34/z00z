use crate::{
    domains::{PackFlowDomain, PackKeyDomain, PackMacDomain, PackNonceDomain, XofBlockDomain},
    error::CryptoError,
    hash_zk::hash_zk,
};
use subtle::ConstantTimeEq;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pack {
    pub s_out: [u8; 32],
    pub v: u64,
    pub r_out: [u8; 32],
    pub asset_class: u8,
}

impl Pack {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(73);
        bytes.extend_from_slice(&self.s_out);
        bytes.extend_from_slice(&self.v.to_le_bytes());
        bytes.extend_from_slice(&self.r_out);
        bytes.push(self.asset_class);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 73 {
            return Err(CryptoError::InvalidLength);
        }

        let mut s_out = [0u8; 32];
        s_out.copy_from_slice(&bytes[0..32]);

        let mut v_bytes = [0u8; 8];
        v_bytes.copy_from_slice(&bytes[32..40]);
        let v = u64::from_le_bytes(v_bytes);

        let mut r_out = [0u8; 32];
        r_out.copy_from_slice(&bytes[40..72]);
        let asset_class = bytes[72];

        Ok(Self {
            s_out,
            v,
            r_out,
            asset_class,
        })
    }
}

fn derive_pack_key(k_dh: &[u8; 32], asset_id: &[u8; 32], serial_id: u32) -> [u8; 32] {
    let serial_bytes = serial_id.to_le_bytes();
    hash_zk::<PackKeyDomain>("", &[k_dh, asset_id, &serial_bytes])
}

fn derive_pack_nonce(
    leaf_ad: &[u8; 32],
    r_pub: &[u8; 32],
    asset_id: &[u8; 32],
    serial_id: u32,
) -> [u8; 32] {
    let serial_bytes = serial_id.to_le_bytes();
    hash_zk::<PackNonceDomain>("", &[leaf_ad, r_pub, asset_id, &serial_bytes])
}

fn gen_keystream(pack_key: &[u8; 32], nonce: &[u8; 32], length: usize) -> Vec<u8> {
    let blocks = length.div_ceil(32);
    let mut keystream = Vec::with_capacity(blocks * 32);

    let stream_domain = hash_zk::<PackFlowDomain>("", &[pack_key, nonce]);

    for index in 0..blocks {
        let counter = (index as u32).to_le_bytes();
        let block = hash_zk::<XofBlockDomain>("", &[&stream_domain, pack_key, nonce, &counter]);
        keystream.extend_from_slice(&block);
    }

    keystream.truncate(length);
    keystream
}

fn compute_tag(
    pack_key: &[u8; 32],
    nonce: &[u8; 32],
    leaf_ad: &[u8; 32],
    ciphertext: &[u8],
) -> [u8; 32] {
    let len_bytes = (ciphertext.len() as u64).to_le_bytes();
    hash_zk::<PackMacDomain>("", &[pack_key, nonce, leaf_ad, ciphertext, &len_bytes])
}

pub fn seal_zkpack(
    plaintext: &Pack,
    k_dh: &[u8; 32],
    leaf_ad: &[u8; 32],
    r_pub: &[u8; 32],
    asset_id: &[u8; 32],
    serial_id: u32,
) -> Result<Vec<u8>, CryptoError> {
    let pt_bytes = plaintext.to_bytes();
    let pack_key = derive_pack_key(k_dh, asset_id, serial_id);
    let nonce = derive_pack_nonce(leaf_ad, r_pub, asset_id, serial_id);
    let keystream = gen_keystream(&pack_key, &nonce, pt_bytes.len());

    let ciphertext: Vec<u8> = pt_bytes
        .iter()
        .zip(keystream.iter())
        .map(|(plain, key_byte)| plain ^ key_byte)
        .collect();

    let tag = compute_tag(&pack_key, &nonce, leaf_ad, &ciphertext);

    let mut output = Vec::with_capacity(ciphertext.len() + 32);
    output.extend_from_slice(&ciphertext);
    output.extend_from_slice(&tag);
    Ok(output)
}

pub fn open_zkpack(
    ciphertext: &[u8],
    k_dh: &[u8; 32],
    leaf_ad: &[u8; 32],
    r_pub: &[u8; 32],
    asset_id: &[u8; 32],
    serial_id: u32,
) -> Result<Pack, CryptoError> {
    if ciphertext.len() != 105 {
        return Err(CryptoError::InvalidLength);
    }

    let (ct_data, tag_received) = ciphertext.split_at(73);
    let mut tag_array = [0u8; 32];
    tag_array.copy_from_slice(tag_received);

    let pack_key = derive_pack_key(k_dh, asset_id, serial_id);
    let nonce = derive_pack_nonce(leaf_ad, r_pub, asset_id, serial_id);
    let tag_expected = compute_tag(&pack_key, &nonce, leaf_ad, ct_data);

    if !bool::from(tag_expected.ct_eq(&tag_array)) {
        return Err(CryptoError::AuthenticationFailed);
    }

    let keystream = gen_keystream(&pack_key, &nonce, ct_data.len());
    let plaintext: Vec<u8> = ct_data
        .iter()
        .zip(keystream.iter())
        .map(|(cipher_byte, key_byte)| cipher_byte ^ key_byte)
        .collect();

    Pack::from_bytes(&plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_open_roundtrip() {
        let plaintext = Pack {
            s_out: [0x42; 32],
            v: 1000,
            r_out: [0x01; 32],
            asset_class: 1,
        };

        let k_dh = [0xAA; 32];
        let leaf_ad = [0xBB; 32];
        let r_pub = [0xCC; 32];
        let asset_id = [0xDD; 32];
        let serial_id = 1u32;

        let ciphertext =
            seal_zkpack(&plaintext, &k_dh, &leaf_ad, &r_pub, &asset_id, serial_id).unwrap();

        let recovered =
            open_zkpack(&ciphertext, &k_dh, &leaf_ad, &r_pub, &asset_id, serial_id).unwrap();

        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn test_modified_ciphertext_fails() {
        let plaintext = Pack {
            s_out: [0x42; 32],
            v: 1000,
            r_out: [0x01; 32],
            asset_class: 1,
        };

        let k_dh = [0xAA; 32];
        let leaf_ad = [0xBB; 32];
        let r_pub = [0xCC; 32];
        let asset_id = [0xDD; 32];
        let serial_id = 1u32;

        let mut ciphertext =
            seal_zkpack(&plaintext, &k_dh, &leaf_ad, &r_pub, &asset_id, serial_id).unwrap();
        ciphertext[0] ^= 0x01;

        let result = open_zkpack(&ciphertext, &k_dh, &leaf_ad, &r_pub, &asset_id, serial_id);
        assert!(result.is_err());
    }
}
