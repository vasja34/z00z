//! Domain separation collision resistance tests.
//!
//! Verifies that domain separation prevents collision attacks.

use z00z_crypto::hash::derive_domain_hash;

#[test]
fn test_derive_domain_resist_collision() {
    // Different domain/data boundaries produce different hashes
    let hash1 = derive_domain_hash("ab", b"c");
    let hash2 = derive_domain_hash("a", b"bc");

    assert_ne!(hash1, hash2, "Collision: 'ab'+'c' == 'a'+'bc'");
}

#[test]
fn test_derive_domain_empty_domain() {
    // Empty domain should work
    let hash1 = derive_domain_hash("", b"data");
    let hash2 = derive_domain_hash("x", b"data");

    assert_ne!(hash1, hash2);
}

#[test]
fn test_derive_domain_empty_data() {
    // Empty data should work
    let hash1 = derive_domain_hash("domain", b"");
    let hash2 = derive_domain_hash("domain", b"x");

    assert_ne!(hash1, hash2);
}

#[test]
fn test_derive_domain_hash_empty() {
    // Both empty - should be deterministic
    let hash1 = derive_domain_hash("", b"");
    let hash2 = derive_domain_hash("", b"");

    assert_eq!(hash1, hash2);
}

#[test]
fn test_derive_domain_long_inputs() {
    // Very long inputs
    let long_domain = "a".repeat(1000);
    let long_data = vec![0x42u8; 10000];

    let hash1 = derive_domain_hash(&long_domain, &long_data);
    let hash2 = derive_domain_hash(&long_domain, &long_data);

    assert_eq!(hash1, hash2);
}

#[test]
fn test_derive_domain_hash_deterministic() {
    // Same inputs produce same output
    let hash1 = derive_domain_hash("context", b"data");
    let hash2 = derive_domain_hash("context", b"data");

    assert_eq!(hash1, hash2);
}

#[test]
fn test_derive_domain_domain_diff() {
    // Different domains produce different hashes
    let hash1 = derive_domain_hash("domain1", b"data");
    let hash2 = derive_domain_hash("domain2", b"data");

    assert_ne!(hash1, hash2);
}

#[test]
fn test_derive_domain_data_diff() {
    // Different data produces different hashes
    let hash1 = derive_domain_hash("domain", b"data1");
    let hash2 = derive_domain_hash("domain", b"data2");

    assert_ne!(hash1, hash2);
}

#[test]
fn test_prefix_collision_attack() {
    // Classic length-extension attack prevented
    // If not length-prefixed: "abc" || "def" == "ab" || "cdef"

    let hash1 = derive_domain_hash("abc", b"def");
    let hash2 = derive_domain_hash("ab", b"cdef");

    assert_ne!(
        hash1, hash2,
        "Length-prefixing failed! Vulnerable to collision attacks"
    );
}

#[test]
fn test_multi_boundary_test() {
    // Test multiple boundary variations
    let cases = [
        ("a", b"bc" as &[u8]),
        ("ab", b"c"),
        ("abc", b""),
        ("", b"abc"),
    ];

    let mut hashes = Vec::new();
    for (domain, data) in &cases {
        hashes.push(derive_domain_hash(domain, data));
    }

    // All must be unique
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(
                hashes[i], hashes[j],
                "Collision at cases {} and {}: {:?} vs {:?}",
                i, j, cases[i], cases[j]
            );
        }
    }
}
