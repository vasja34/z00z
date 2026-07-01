use super::*;

#[test]
fn test_blake2b256_simple() {
    let data = b"test data";
    let hash1 = blake2b_256_simple(data);
    let hash2 = blake2b_256_simple(data);
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 32);
}

#[test]
fn test_blake2b512_simple() {
    let data = b"test data";
    let hash1 = blake2b_512_simple(data);
    let hash2 = blake2b_512_simple(data);
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64);
}

#[test]
fn test_sha256_256_simple() {
    let data = b"test data";
    let hash1 = sha256_256_simple(data);
    let hash2 = sha256_256_simple(data);
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 32);
}

#[test]
fn test_derive_key_from_seed() {
    let seed = b"test-seed";
    let key1 = derive_key_from_seed::<AssetIdHashDomain>(seed, "context1");
    let key2 = derive_key_from_seed::<AssetIdHashDomain>(seed, "context1");
    let key3 = derive_key_from_seed::<AssetIdHashDomain>(seed, "context2");

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
    assert_eq!(key1.len(), 32);
}

#[test]
fn test_protocol_domains_compile() {
    let _ = DomainHasher::<AssetIdHashDomain>::new_with_label("test");
    let _ = DomainHasher::<ChecksumHashDomain>::new_with_label("test");
    let _ = DomainHasher::<TestNonceDomain>::new_with_label("test");
}

#[test]
fn test_domain_separation_consistency() {
    let data = b"test";

    let hash1 = DomainHasher::<AssetIdHashDomain>::new_with_label("test")
        .chain(data)
        .finalize();
    let hash2 = DomainHasher::<AssetIdHashDomain>::new_with_label("test")
        .chain(data)
        .finalize();

    assert_eq!(hash1.as_ref(), hash2.as_ref());
}
