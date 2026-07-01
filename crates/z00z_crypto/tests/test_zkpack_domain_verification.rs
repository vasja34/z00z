//! Verification test: Domain strings match SPEC §2.2.2.1
//!
//! This test ensures ZkPack AEAD uses correct domain separation strings.

use tari_crypto::hashing::DomainSeparation;
use z00z_crypto::domains::{PackKeyDomain, PackMacDomain, PackNonceDomain};

#[test]
fn test_verify_zkpack_domain_strings() {
    // SPEC §2.2.2.1 requires these exact base domain strings
    assert_eq!(
        PackKeyDomain::domain(),
        "z00z.consensus.pack_key.v1",
        "PackKeyDomain MUST match SPEC §2.2.2.1"
    );
    assert_eq!(
        PackNonceDomain::domain(),
        "z00z.consensus.pack_nonce.v1",
        "PackNonceDomain MUST match SPEC §2.2.2.1"
    );
    assert_eq!(
        PackMacDomain::domain(),
        "z00z.consensus.pack_mac.v1",
        "PackMacDomain MUST match current prior-sponge domain mapping"
    );

    // Verify versions (all should be v1)
    assert_eq!(PackKeyDomain::version(), 1);
    assert_eq!(PackNonceDomain::version(), 1);
    assert_eq!(PackMacDomain::version(), 1);

    // Tari appends `.v{version}` on top of the declared base domain.
    assert_eq!(
        PackKeyDomain::domain_separation_tag(""),
        "z00z.consensus.pack_key.v1.v1"
    );
    assert_eq!(
        PackNonceDomain::domain_separation_tag(""),
        "z00z.consensus.pack_nonce.v1.v1"
    );
    assert_eq!(
        PackMacDomain::domain_separation_tag(""),
        "z00z.consensus.pack_mac.v1.v1"
    );

    println!("✅ All ZkPack domain strings verified against SPEC §2.2.2.1");
}

#[test]
fn test_verify_domain_uniqueness() {
    // Ensure no domain string collisions
    let domains = [
        PackKeyDomain::domain(),
        PackNonceDomain::domain(),
        PackMacDomain::domain(),
    ];

    // Check all domains are unique
    for i in 0..domains.len() {
        for j in (i + 1)..domains.len() {
            assert_ne!(
                domains[i], domains[j],
                "Domain collision detected: {} == {}",
                domains[i], domains[j]
            );
        }
    }

    println!("✅ All domain strings are unique");
}
