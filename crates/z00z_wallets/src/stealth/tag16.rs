//! Wallet facade over the crypto-owned `tag16` and wallet `leaf_ad` formulas.
//!
//! The canonical formulas now live in `z00z_crypto::protocol::stealth_bind` and stay here
//! only as wallet-facing wrappers plus request-bound helpers.

//! leaf_ad binds 5 public leaf fields.
//! Changing any bound field invalidates `enc_pack` authentication in decrypt flow.
//! See §4.7.0.

#[cfg(test)]
use std::time::{Duration, Instant};

use z00z_crypto::{
    compute_leaf_ad as crypto_compute_leaf_ad, compute_tag16 as crypto_compute_tag16,
    hash_zk::hash_zk,
};
#[cfg(test)]
use z00z_utils::rng::{RngCoreExt, SystemRngProvider};

use crate::domains::WalletTag16HashProdDomain;

/// Canonical leaf_ad preimage size in bytes.
///
/// Layout:
/// - `[0..32]` asset_id
/// - `[32..36]` serial_id (LE)
/// - `[36..68]` r_pub
/// - `[68..100]` owner_tag
/// - `[100..132]` c_amount
#[cfg(test)]
pub const LEAF_PREIMAGE_SIZE: usize = 132;

/// Compute 16-bit prefilter tag from ECDH key and leaf binding bytes.
pub fn compute_tag16(k_dh: &[u8; 32], leaf_ad: &[u8; 32]) -> u16 {
    crypto_compute_tag16(k_dh, leaf_ad)
}

/// Compute 16-bit prefilter tag with request binding.
pub fn compute_tag16_with_req(k_dh: &[u8; 32], req_id: &[u8; 32]) -> u16 {
    let hash = hash_zk::<WalletTag16HashProdDomain>("z00z.consensus.tag16.v1", &[k_dh, req_id]);
    u16::from_le_bytes([hash[0], hash[1]])
}

/// Compute leaf associated binding hash.
pub fn compute_leaf_ad(
    asset_id: &[u8; 32],
    serial_id: u32,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
    c_amount: &[u8; 32],
) -> [u8; 32] {
    crypto_compute_leaf_ad(asset_id, serial_id, r_pub, owner_tag, c_amount)
}

/// Encode leaf_ad inputs into canonical 132-byte preimage.
///
/// Field order is consensus-critical and must not change.
#[cfg(test)]
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

/// Craft inputs that collide with target tag16.
#[cfg(test)]
pub fn craft_tag16_collision(target_tag: u16) -> ([u8; 32], [u8; 32]) {
    let provider = SystemRngProvider;
    let mut rng = provider.rng();
    let mut k_dh = [0u8; 32];
    let mut leaf_ad = [0u8; 32];

    for _ in 0..1_000_000 {
        rng.fill_bytes_ext(&mut k_dh);
        rng.fill_bytes_ext(&mut leaf_ad);
        if compute_tag16(&k_dh, &leaf_ad) == target_tag {
            return (k_dh, leaf_ad);
        }
    }

    panic!("failed to craft tag16 collision");
}

/// Benchmark tag16 computation throughput.
#[cfg(test)]
pub fn benchmark_tag16_compute(iterations: usize) -> Duration {
    let provider = SystemRngProvider;
    let mut rng = provider.rng();
    let mut k_dh = [0u8; 32];
    let mut leaf_ad = [0u8; 32];

    let start = Instant::now();
    for _ in 0..iterations {
        rng.fill_bytes_ext(&mut k_dh);
        rng.fill_bytes_ext(&mut leaf_ad);
        let _ = compute_tag16(&k_dh, &leaf_ad);
    }
    start.elapsed()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use z00z_crypto::hash_zk::hash_zk;

    use z00z_crypto::domains::StealthLeafAdProdDomain;

    use super::{
        benchmark_tag16_compute, compute_leaf_ad, compute_tag16, compute_tag16_with_req,
        craft_tag16_collision, encode_leaf_preimage,
    };

    #[test]
    fn test_tag16_deterministic() {
        let first = compute_tag16(&[3u8; 32], &[4u8; 32]);
        let second = compute_tag16(&[3u8; 32], &[4u8; 32]);
        assert_eq!(first, second);

        let req_a = compute_tag16_with_req(&[5u8; 32], &[6u8; 32]);
        let req_b = compute_tag16_with_req(&[5u8; 32], &[6u8; 32]);
        assert_eq!(req_a, req_b);
    }

    #[test]
    fn test_tag16_per_leaf_unique() {
        let k_dh = [7u8; 32];
        let mut seen = HashSet::new();

        for serial in 0..64u32 {
            let leaf_ad = compute_leaf_ad(&[8u8; 32], serial, &[9u8; 32], &[10u8; 32], &[11u8; 32]);
            seen.insert(compute_tag16(&k_dh, &leaf_ad));
        }

        assert!(seen.len() > 48);
    }

    #[test]
    fn test_tag16_collision_probability() {
        let k_dh = [12u8; 32];
        let mut seen = HashSet::new();

        for serial in 0..1024u32 {
            let mut asset_id = [13u8; 32];
            asset_id[..4].copy_from_slice(&serial.to_le_bytes());
            let leaf_ad = compute_leaf_ad(&asset_id, serial, &[14u8; 32], &[15u8; 32], &[16u8; 32]);
            seen.insert(compute_tag16(&k_dh, &leaf_ad));
        }

        let collisions = 1024usize.saturating_sub(seen.len());
        assert!(collisions < 64);
    }

    #[test]
    fn test_leaf_ad_binding() {
        let base = compute_leaf_ad(&[17u8; 32], 42, &[18u8; 32], &[19u8; 32], &[20u8; 32]);

        let changed_asset = compute_leaf_ad(&[21u8; 32], 42, &[18u8; 32], &[19u8; 32], &[20u8; 32]);
        assert_ne!(base, changed_asset);

        let changed_serial =
            compute_leaf_ad(&[17u8; 32], 43, &[18u8; 32], &[19u8; 32], &[20u8; 32]);
        assert_ne!(base, changed_serial);
    }

    #[test]
    fn test_bind_asset_id() {
        let base = compute_leaf_ad(&[1u8; 32], 42, &[3u8; 32], &[4u8; 32], &[5u8; 32]);
        let changed = compute_leaf_ad(&[2u8; 32], 42, &[3u8; 32], &[4u8; 32], &[5u8; 32]);
        assert_ne!(base, changed);
    }

    #[test]
    fn test_bind_serial_id() {
        let base = compute_leaf_ad(&[1u8; 32], 42, &[3u8; 32], &[4u8; 32], &[5u8; 32]);
        let changed = compute_leaf_ad(&[1u8; 32], 43, &[3u8; 32], &[4u8; 32], &[5u8; 32]);
        assert_ne!(base, changed);
    }

    #[test]
    fn test_bind_r_pub() {
        let base = compute_leaf_ad(&[1u8; 32], 42, &[3u8; 32], &[4u8; 32], &[5u8; 32]);
        let changed = compute_leaf_ad(&[1u8; 32], 42, &[6u8; 32], &[4u8; 32], &[5u8; 32]);
        assert_ne!(base, changed);
    }

    #[test]
    fn test_bind_owner_tag() {
        let base = compute_leaf_ad(&[1u8; 32], 42, &[3u8; 32], &[4u8; 32], &[5u8; 32]);
        let changed = compute_leaf_ad(&[1u8; 32], 42, &[3u8; 32], &[7u8; 32], &[5u8; 32]);
        assert_ne!(base, changed);
    }

    #[test]
    fn test_bind_c_amount() {
        let base = compute_leaf_ad(&[1u8; 32], 42, &[3u8; 32], &[4u8; 32], &[5u8; 32]);
        let changed = compute_leaf_ad(&[1u8; 32], 42, &[3u8; 32], &[4u8; 32], &[8u8; 32]);
        assert_ne!(base, changed);
    }

    #[test]
    fn test_leaf_ad_deterministic() {
        let first = compute_leaf_ad(&[9u8; 32], 17, &[10u8; 32], &[11u8; 32], &[12u8; 32]);
        let second = compute_leaf_ad(&[9u8; 32], 17, &[10u8; 32], &[11u8; 32], &[12u8; 32]);
        assert_eq!(first, second);
    }

    #[test]
    fn test_leaf_ad_unique() {
        let first = compute_leaf_ad(&[9u8; 32], 17, &[10u8; 32], &[11u8; 32], &[12u8; 32]);
        let second = compute_leaf_ad(&[9u8; 32], 17, &[13u8; 32], &[11u8; 32], &[12u8; 32]);
        assert_ne!(first, second);
    }

    #[test]
    fn test_out_len_32() {
        let out = compute_leaf_ad(&[1u8; 32], 2, &[3u8; 32], &[4u8; 32], &[5u8; 32]);
        assert_eq!(out.len(), 32);
    }

    #[test]
    fn test_order_sensitive() {
        let asset_id = [0x11u8; 32];
        let serial_id = 7u32;
        let r_pub = [0x22u8; 32];
        let owner_tag = [0x33u8; 32];
        let c_amount = [0x44u8; 32];

        let canonical = compute_leaf_ad(&asset_id, serial_id, &r_pub, &owner_tag, &c_amount);
        let swapped = compute_leaf_ad(&asset_id, serial_id, &owner_tag, &r_pub, &c_amount);
        assert_ne!(canonical, swapped);
    }

    #[test]
    fn test_all_fields_change() {
        let base = compute_leaf_ad(
            &[0x10u8; 32],
            19,
            &[0x20u8; 32],
            &[0x30u8; 32],
            &[0x40u8; 32],
        );

        let c1 = compute_leaf_ad(
            &[0x11u8; 32],
            19,
            &[0x20u8; 32],
            &[0x30u8; 32],
            &[0x40u8; 32],
        );
        let c2 = compute_leaf_ad(
            &[0x10u8; 32],
            20,
            &[0x20u8; 32],
            &[0x30u8; 32],
            &[0x40u8; 32],
        );
        let c3 = compute_leaf_ad(
            &[0x10u8; 32],
            19,
            &[0x21u8; 32],
            &[0x30u8; 32],
            &[0x40u8; 32],
        );
        let c4 = compute_leaf_ad(
            &[0x10u8; 32],
            19,
            &[0x20u8; 32],
            &[0x31u8; 32],
            &[0x40u8; 32],
        );
        let c5 = compute_leaf_ad(
            &[0x10u8; 32],
            19,
            &[0x20u8; 32],
            &[0x30u8; 32],
            &[0x41u8; 32],
        );

        assert_ne!(base, c1);
        assert_ne!(base, c2);
        assert_ne!(base, c3);
        assert_ne!(base, c4);
        assert_ne!(base, c5);
    }

    #[test]
    fn test_hash_frame_stable() {
        let asset_id = [0x51u8; 32];
        let serial = 0x0102_0304u32;
        let r_pub = [0x52u8; 32];
        let owner_tag = [0x53u8; 32];
        let c_amount = [0x54u8; 32];

        let got = compute_leaf_ad(&asset_id, serial, &r_pub, &owner_tag, &c_amount);
        let serial_le = serial.to_le_bytes();
        let expect = hash_zk::<StealthLeafAdProdDomain>(
            "z00z.consensus.leaf_ad.v1",
            &[&asset_id, &serial_le, &r_pub, &owner_tag, &c_amount],
        );
        assert_eq!(got, expect, "split-slice framing must stay canonical");

        let packed = encode_leaf_preimage(&asset_id, serial, &r_pub, &owner_tag, &c_amount);
        let packed_hash =
            hash_zk::<StealthLeafAdProdDomain>("z00z.consensus.leaf_ad.v1", &[&packed]);
        assert_ne!(
            got, packed_hash,
            "single-slice framing changes hash semantics"
        );
    }

    #[test]
    fn test_tag16_collision_craft() {
        let target = compute_tag16(&[1u8; 32], &[2u8; 32]);
        let (k_dh, leaf_ad) = craft_tag16_collision(target);
        assert_eq!(compute_tag16(&k_dh, &leaf_ad), target);
    }

    #[test]
    fn test_tag16_bench() {
        let elapsed = benchmark_tag16_compute(256);
        assert!(elapsed >= std::time::Duration::ZERO);
    }
}
