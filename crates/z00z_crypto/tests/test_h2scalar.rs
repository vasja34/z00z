//! Phase 3 Hash-to-Scalar tests
//!
//! SPEC: specs/007-z00z-ecc-spec-2/E2E-TEST-EXAMPLES.md §3.1–§3.4

use hex_literal::hex;
use rand::RngCore;
use z00z_crypto::{
    domains::{EphemeralScalarDomain, HashToScalarDomain, ViewKeyDomain},
    hash::hash_to_scalar_zk,
    types::Z00ZScalar,
};

#[test]
fn test_h2scalar_nonzero_canonical() {
    let mut rng = z00z_utils::rng::SystemRngProvider.rng();

    for _ in 0..10_000 {
        let mut data = [0u8; 32];
        rng.fill_bytes(&mut data);

        let s = hash_to_scalar_zk::<ViewKeyDomain>("", &[&data]).expect("hash_to_scalar_zk");
        assert!(!s.is_zero(), "h2scalar returned zero scalar");

        let bytes = s.to_bytes();
        assert!(
            Z00ZScalar::try_from_bytes(bytes).is_ok(),
            "h2scalar not canonical"
        );
    }
}

#[test]
fn test_h2scalar_domain_separation() {
    let cases = [[0x42u8; 32], [0x00u8; 32], [0xFFu8; 32]];

    for secret in cases {
        let view_sk =
            hash_to_scalar_zk::<ViewKeyDomain>("", &[&secret]).expect("hash_to_scalar_zk");
        let ident_sk = hash_to_scalar_zk::<HashToScalarDomain>("Z00Z/IDENTITY", &[&secret])
            .expect("hash_to_scalar_zk");
        let r_scalar =
            hash_to_scalar_zk::<EphemeralScalarDomain>("", &[&secret]).expect("hash_to_scalar_zk");

        assert_ne!(
            view_sk.to_bytes(),
            ident_sk.to_bytes(),
            "VIEW and IDENTITY scalars must differ"
        );
        assert_ne!(
            view_sk.to_bytes(),
            r_scalar.to_bytes(),
            "VIEW and R scalars must differ"
        );
        assert_ne!(
            ident_sk.to_bytes(),
            r_scalar.to_bytes(),
            "IDENTITY and R scalars must differ"
        );
    }
}

#[test]
fn test_h2scalar_golden_vectors() {
    const ZERO_SECRET: [u8; 32] = [0x00; 32];
    const ALICE_SECRET: [u8; 32] = [0x11; 32];
    const BOB_SECRET: [u8; 32] = [0x22; 32];
    const MAX_SECRET: [u8; 32] = [0xFF; 32];

    const ZERO_VIEW_SK: [u8; 32] =
        hex!("8b84ed02824ea7bf988070e81c99dbc81d5b0f0f9f61730fcb77a35a43f0b305");
    const ALICE_VIEW_SK: [u8; 32] =
        hex!("14ef70529859ec3146a2d101d2f1b511b586717a5f835348e72260e8e31d9b06");
    const BOB_VIEW_SK: [u8; 32] =
        hex!("7ffc87d637958b3d1a594b4fda4c838917a04e0a7bdaf4322e19179ab0f1dc0f");
    const MAX_VIEW_SK: [u8; 32] =
        hex!("15c32de57518f7ab07a2b0c12983500fa6018a258deb0ef5fc9945d8b4a60c04");

    assert_eq!(
        hash_to_scalar_zk::<ViewKeyDomain>("", &[&ZERO_SECRET])
            .expect("hash_to_scalar_zk")
            .to_bytes(),
        ZERO_VIEW_SK
    );
    assert_eq!(
        hash_to_scalar_zk::<ViewKeyDomain>("", &[&ALICE_SECRET])
            .expect("hash_to_scalar_zk")
            .to_bytes(),
        ALICE_VIEW_SK
    );
    assert_eq!(
        hash_to_scalar_zk::<ViewKeyDomain>("", &[&BOB_SECRET])
            .expect("hash_to_scalar_zk")
            .to_bytes(),
        BOB_VIEW_SK
    );
    assert_eq!(
        hash_to_scalar_zk::<ViewKeyDomain>("", &[&MAX_SECRET])
            .expect("hash_to_scalar_zk")
            .to_bytes(),
        MAX_VIEW_SK
    );
}

#[test]
#[ignore]
fn test_h2scalar_distribution() {
    let mut hist = [0u32; 256];

    for i in 0u64..10_000 {
        let bytes = i.to_le_bytes();
        let s = hash_to_scalar_zk::<HashToScalarDomain>("Z00Z/TEST", &[&bytes])
            .expect("hash_to_scalar_zk");
        hist[s.to_bytes()[0] as usize] += 1;
    }

    let expected = 10_000.0 / 256.0;
    let min_count = hist.iter().copied().min().expect("non-empty histogram");
    let max_count = hist.iter().copied().max().expect("non-empty histogram");
    let chi_square: f64 = hist
        .iter()
        .map(|&count| {
            let delta = count as f64 - expected;
            (delta * delta) / expected
        })
        .sum();

    assert!(
        hist.iter().all(|&count| count > 0),
        "distribution left an empty bucket: min={}, max={}, chi_square={chi_square:.3}",
        min_count,
        max_count,
    );
    assert!(
        chi_square < 420.0,
        "distribution chi-square too large: min={}, max={}, chi_square={chi_square:.3}",
        min_count,
        max_count,
    );
}
