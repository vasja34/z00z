use z00z_crypto::expert::encoding::to_hex;
use z00z_crypto::{hash_to_scalar_domain, kdf_wallet};

#[test]
fn test_h2scalar_golden_vectors() {
    struct TestVector {
        domain: &'static [u8],
        data: &'static [u8],
        expected_hex: &'static str,
    }

    let vectors = [
        TestVector {
            domain: b"test/domain/1",
            data: b"input_data_1",
            expected_hex: "4311bc73070544a98a3cdf1c9433e1dff987232c668733ec6a2fc3169cd13d03",
        },
        TestVector {
            domain: b"test/domain/2",
            data: b"input_data_2",
            expected_hex: "f053bb06050ad3d6684d71b40487ed4c9c1979e31526d0c292de0c457afbfc0a",
        },
    ];

    for vec in &vectors {
        let scalar = hash_to_scalar_domain(vec.domain, &[vec.data]);
        let hex = to_hex(&scalar.to_bytes());
        assert_eq!(hex, vec.expected_hex, "golden vector mismatch");
    }
}

#[test]
fn test_kdf_golden_vectors() {
    struct TestVector {
        info: &'static [u8],
        ikm: &'static [u8],
        expected_hex: &'static str,
    }

    let vectors = [
        TestVector {
            info: b"test/info",
            ikm: b"test/ikm",
            expected_hex: "349dd6cc409514be9a7ac594e8f8c5da86b78b657e51597c92464a3f299d7057",
        },
        TestVector {
            info: b"wallet/info/2",
            ikm: b"wallet/ikm/2",
            expected_hex: "61e727eeb9e42f1bce1341658b975b4b3104ed9002983b4d4567977f06882d24",
        },
    ];

    for vec in &vectors {
        let result = kdf_wallet(vec.info, vec.ikm).expect("kdf");
        let hex = to_hex(&result);
        assert_eq!(hex, vec.expected_hex, "golden vector mismatch");
    }
}
