//! ZkPack live path: deterministic ChaCha20Poly1305 AEAD over the fixed
//! `ZkPack_v1` wire profile.
//! The current wallet path closes on this AEAD facade plus the memo-capable
//! pack lane carried by the same encrypted envelope. Unsupported versions fail
//! closed; no alternate field-native or Poseidon2 wire is live on this path.
// NON-GOAL: Post-quantum security. ECDH remains quantum-vulnerable.
// NON-GOAL: Metadata hiding. The fixed wire profile is 89 bytes (1+72+16).
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use z00z_crypto::protocol::zkpack::{ZKPACK_TAG_LEN, ZKPACK_VER};
pub use z00z_crypto::ZkPackEncrypted;

use super::kdf::{derive_nonce, derive_pack_key};

/// Deterministic ZkPack v1 encrypt/decrypt facade.
pub struct ZkPack;

impl ZkPack {
    /// Encrypt plaintext with context-bound deterministic AEAD.
    pub fn encrypt(
        k_dh: &[u8; 32],
        leaf_ad: &[u8; 32],
        r_pub: &[u8; 32],
        asset_id: &[u8; 32],
        serial_id: u32,
        plaintext: &[u8],
    ) -> ZkPackEncrypted {
        let pack_key = derive_pack_key(k_dh, asset_id, serial_id);
        // SECURITY: r_pub MUST be freshly generated per output to prevent (key, nonce) reuse.
        let nonce12 = derive_nonce(leaf_ad, r_pub, asset_id, serial_id);
        let aad = make_aad(leaf_ad, r_pub, asset_id, serial_id);

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&pack_key));
        let nonce = Nonce::from_slice(&nonce12);
        let payload = Payload {
            msg: plaintext,
            aad: &aad,
        };

        let encrypted = cipher
            .encrypt(nonce, payload)
            .expect("ChaCha20Poly1305 encrypt failed with valid key/nonce");
        assert!(
            encrypted.len() >= ZKPACK_TAG_LEN,
            "AEAD output shorter than tag"
        );

        let mut ciphertext = encrypted;
        let tag_bytes = ciphertext.split_off(ciphertext.len() - ZKPACK_TAG_LEN);
        let mut tag = [0u8; ZKPACK_TAG_LEN];
        tag.copy_from_slice(&tag_bytes);

        ZkPackEncrypted {
            version: ZKPACK_VER,
            ciphertext,
            tag,
        }
    }

    /// Decrypt payload and return plaintext if authentication succeeds.
    pub fn decrypt(
        k_dh: &[u8; 32],
        leaf_ad: &[u8; 32],
        r_pub: &[u8; 32],
        asset_id: &[u8; 32],
        serial_id: u32,
        enc: &ZkPackEncrypted,
    ) -> Option<Vec<u8>> {
        if enc.version != ZKPACK_VER {
            return None;
        }

        let pack_key = derive_pack_key(k_dh, asset_id, serial_id);
        let nonce12 = derive_nonce(leaf_ad, r_pub, asset_id, serial_id);
        let aad = make_aad(leaf_ad, r_pub, asset_id, serial_id);

        let mut envelope = Vec::with_capacity(enc.ciphertext.len() + enc.tag.len());
        envelope.extend_from_slice(&enc.ciphertext);
        envelope.extend_from_slice(&enc.tag);

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&pack_key));
        let nonce = Nonce::from_slice(&nonce12);
        let payload = Payload {
            msg: &envelope,
            aad: &aad,
        };

        // SECURITY: Authentication and decryption are atomic in AEAD.
        // Any tamper/AAD mismatch returns error and no plaintext is returned.
        cipher.decrypt(nonce, payload).ok()
    }
}

fn make_aad(leaf_ad: &[u8; 32], r_pub: &[u8; 32], asset_id: &[u8; 32], serial_id: u32) -> Vec<u8> {
    let serial = serial_id.to_le_bytes();
    let mut aad = Vec::with_capacity(1 + 32 + 32 + 32 + 4);
    aad.push(ZKPACK_VER);
    aad.extend_from_slice(leaf_ad);
    aad.extend_from_slice(r_pub);
    aad.extend_from_slice(asset_id);
    aad.extend_from_slice(&serial);
    aad
}

#[cfg(test)]
#[path = "test_zkpack.rs"]
mod tests;
