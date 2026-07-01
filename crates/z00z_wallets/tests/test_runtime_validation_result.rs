use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::key::validate_entropy_result;
use z00z_wallets::rpc::types::common::RuntimeValidationResult;

#[test]
fn test_runtime_warnings_json() {
    let valid =
        RuntimeValidationResult::valid_with_warnings(vec!["entropy heuristic warning".to_string()]);
    let valid_json = String::from_utf8(JsonCodec.serialize(&valid).unwrap()).unwrap();
    assert!(valid_json.contains("warnings"));
    assert!(!valid_json.contains("error"));

    let invalid = RuntimeValidationResult::invalid("invalid receiver");
    let invalid_json = String::from_utf8(JsonCodec.serialize(&invalid).unwrap()).unwrap();
    assert!(invalid_json.contains("error"));
    assert!(!invalid_json.contains("warnings"));
}

#[test]
fn test_entropy_keeps_warnings() {
    let seed = [
        0xffu8, 0xfeu8, 0xfdu8, 0xfbu8, 0xf7u8, 0xefu8, 0xdfu8, 0xbfu8, 0xfeu8, 0xfdu8, 0xfbu8,
        0xf7u8, 0xefu8, 0xdfu8, 0xbfu8, 0xffu8,
    ];

    let result = validate_entropy_result(&seed);
    assert!(result.valid);
    assert!(result.error.is_none());
    assert!(!result.warnings.is_empty());
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.contains("bit count")));
}

#[test]
fn test_entropy_rejects_patterns() {
    let mut seed = [0u8; 64];
    for (index, byte) in seed.iter_mut().enumerate() {
        *byte = index as u8;
    }

    let result = validate_entropy_result(&seed);
    assert!(!result.valid);
    assert!(result.warnings.is_empty());
    assert!(result
        .error
        .as_deref()
        .is_some_and(|error| error.contains("sequential")));
}
