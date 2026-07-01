use z00z_crypto::{
    domains::{PackKeyDomain, PackNonceDomain},
    hash_zk::hash_zk,
};

pub use z00z_crypto::kdf::compute_owner_tag;

pub use super::{
    crypto::ecdh::{derive_k_dh, derive_k_dh_with_req},
    tag::{compute_leaf_ad, compute_tag16, compute_tag16_with_req},
};

/// Derive asset pack key from DH key and asset binding.
pub fn derive_pack_key(k_dh: &[u8; 32], asset_id: &[u8; 32], serial_id: u32) -> [u8; 32] {
    // LE encoding MUST match OWF circuit — always 4 bytes
    let serial = serial_id.to_le_bytes();
    hash_zk::<PackKeyDomain>("", &[k_dh, asset_id, &serial])
}

/// Derive asset pack nonce from leaf and output binding.
pub fn derive_pack_nonce(
    leaf_ad: &[u8; 32],
    r_pub_b: &[u8; 32],
    asset_id: &[u8; 32],
    serial_id: u32,
) -> [u8; 32] {
    // LE encoding MUST match OWF circuit — always 4 bytes
    let serial = serial_id.to_le_bytes();
    hash_zk::<PackNonceDomain>("", &[leaf_ad, r_pub_b, asset_id, &serial])
}

/// Derive 12-byte AEAD nonce from full 32-byte pack nonce.
pub fn derive_nonce(
    leaf_ad: &[u8; 32],
    r_pub_b: &[u8; 32],
    asset_id: &[u8; 32],
    serial_id: u32,
) -> [u8; 12] {
    let full = derive_pack_nonce(leaf_ad, r_pub_b, asset_id, serial_id);
    // ChaCha20Poly1305 uses a 96-bit nonce; keep first 12 bytes of the 32-byte derivation output.
    let mut nonce12 = [0u8; 12];
    nonce12.copy_from_slice(&full[..12]);
    nonce12
}

/// Derive stealth output secret `s_out` from DH key, ephemeral public key, and serial.
///
/// Single canonical implementation delegated through the wallet KDF surface.
pub use super::crypto::ecdh::derive_s_out;

#[cfg(test)]
mod tests {
    use super::{derive_nonce, derive_pack_key};

    fn to_hex_lower(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(bytes.len() * 2);
        for &byte in bytes {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
        out
    }

    #[test]
    fn test_key_deterministic() {
        let k_dh = [0x11u8; 32];
        let asset_id = [0x22u8; 32];
        let key0 = derive_pack_key(&k_dh, &asset_id, 42);
        let key1 = derive_pack_key(&k_dh, &asset_id, 42);
        assert_eq!(key0, key1);
    }

    #[test]
    fn test_key_diff_asset() {
        let k_dh = [0x11u8; 32];
        let asset0 = [0x22u8; 32];
        let asset1 = [0x23u8; 32];
        let key0 = derive_pack_key(&k_dh, &asset0, 7);
        let key1 = derive_pack_key(&k_dh, &asset1, 7);
        assert_ne!(key0, key1);
    }

    #[test]
    fn test_key_diff_serial() {
        let k_dh = [0x11u8; 32];
        let asset_id = [0x22u8; 32];
        let key0 = derive_pack_key(&k_dh, &asset_id, 0);
        let key1 = derive_pack_key(&k_dh, &asset_id, 1);
        assert_ne!(key0, key1);
    }

    #[test]
    fn test_key_diff_dh() {
        let k0 = [0x11u8; 32];
        let k1 = [0x12u8; 32];
        let asset_id = [0x22u8; 32];
        let key0 = derive_pack_key(&k0, &asset_id, 9);
        let key1 = derive_pack_key(&k1, &asset_id, 9);
        assert_ne!(key0, key1);
    }

    #[test]
    fn test_key_golden() {
        let k_dh = [0x01u8; 32];
        let asset_id = [0x02u8; 32];
        let key = derive_pack_key(&k_dh, &asset_id, 0x0403_0201);
        let key_hex = to_hex_lower(&key);
        assert_eq!(key_hex.len(), 64);
        assert_eq!(
            key_hex,
            "7a71ab2255ccf0e5af2854a31c230d5b45923732f449cb95cb583fa6a91d6035"
        );
    }

    #[test]
    fn test_nonce_deterministic() {
        let leaf_ad = [0x31u8; 32];
        let r_pub = [0x32u8; 32];
        let asset_id = [0x33u8; 32];
        let nonce0 = derive_nonce(&leaf_ad, &r_pub, &asset_id, 42);
        let nonce1 = derive_nonce(&leaf_ad, &r_pub, &asset_id, 42);
        assert_eq!(nonce0, nonce1);
    }

    #[test]
    fn test_nonce_diff_leaf() {
        let leaf0 = [0x31u8; 32];
        let leaf1 = [0x41u8; 32];
        let r_pub = [0x32u8; 32];
        let asset_id = [0x33u8; 32];
        let nonce0 = derive_nonce(&leaf0, &r_pub, &asset_id, 42);
        let nonce1 = derive_nonce(&leaf1, &r_pub, &asset_id, 42);
        assert_ne!(nonce0, nonce1);
    }

    #[test]
    fn test_nonce_diff_rpub() {
        let leaf = [0x31u8; 32];
        let r0 = [0x32u8; 32];
        let r1 = [0x42u8; 32];
        let asset_id = [0x33u8; 32];
        let nonce0 = derive_nonce(&leaf, &r0, &asset_id, 42);
        let nonce1 = derive_nonce(&leaf, &r1, &asset_id, 42);
        assert_ne!(nonce0, nonce1);
    }

    #[test]
    fn test_nonce_len_12() {
        let leaf_ad = [0x31u8; 32];
        let r_pub = [0x32u8; 32];
        let asset_id = [0x33u8; 32];
        let nonce = derive_nonce(&leaf_ad, &r_pub, &asset_id, 42);
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_nonce_golden() {
        let leaf_ad = [0xA1u8; 32];
        let r_pub = [0xB2u8; 32];
        let asset_id = [0xC3u8; 32];
        let nonce = derive_nonce(&leaf_ad, &r_pub, &asset_id, 0x0807_0605);
        let nonce_hex = to_hex_lower(&nonce);
        assert_eq!(nonce_hex.len(), 24);
        assert_eq!(nonce_hex, "63498f86521097ba2fa3539d");
    }
}
