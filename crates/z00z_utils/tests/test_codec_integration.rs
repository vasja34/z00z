use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
/// Integration tests for codec module with complex data structures
use z00z_utils::prelude::{BincodeCodec, Codec, CodecError, JsonCodec, YamlCodec};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ComplexData {
    id: u64,
    name: String,
    tags: Vec<String>,
    metadata: BTreeMap<String, String>,
    nested: Option<NestedData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NestedData {
    value: i32,
    data: Vec<u8>,
}

#[test]
fn test_json_codec_complex_data() {
    let data = ComplexData {
        id: 42,
        name: "test_item".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        metadata: {
            let mut m = BTreeMap::new();
            m.insert("key1".to_string(), "value1".to_string());
            m
        },
        nested: Some(NestedData {
            value: 100,
            data: vec![1, 2, 3, 4, 5],
        }),
    };

    let codec = JsonCodec;
    let encoded = codec.serialize(&data).expect("encode failed");
    let decoded: ComplexData = codec.deserialize(&encoded).expect("decode failed");

    assert_eq!(data, decoded);
}

#[test]
fn test_yaml_codec_complex_data() {
    let data = ComplexData {
        id: 123,
        name: "yaml_test".to_string(),
        tags: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        metadata: {
            let mut m = BTreeMap::new();
            m.insert("key".to_string(), "value".to_string());
            m
        },
        nested: Some(NestedData {
            value: 200,
            data: vec![10, 20, 30],
        }),
    };

    let codec = YamlCodec;
    let encoded = codec.serialize(&data).expect("encode failed");
    let decoded: ComplexData = codec.deserialize(&encoded).expect("decode failed");

    assert_eq!(data, decoded);
}

#[test]
fn test_bincode_codec_complex_data() {
    let data = ComplexData {
        id: 999,
        name: "bincode_test".to_string(),
        tags: vec!["x".to_string(), "y".to_string()],
        metadata: {
            let mut m = BTreeMap::new();
            m.insert("test".to_string(), "data".to_string());
            m
        },
        nested: Some(NestedData {
            value: 300,
            data: vec![255, 128, 64],
        }),
    };

    let codec = BincodeCodec;
    let encoded = codec.serialize(&data).expect("encode failed");
    let decoded: ComplexData = codec.deserialize(&encoded).expect("decode failed");

    assert_eq!(data, decoded);
}

#[test]
fn test_codec_format_sizes() {
    let data = ComplexData {
        id: 1,
        name: "size_test".to_string(),
        tags: vec!["a".to_string(), "b".to_string()],
        metadata: {
            let mut m = BTreeMap::new();
            m.insert("k1".to_string(), "v1".to_string());
            m
        },
        nested: Some(NestedData {
            value: 42,
            data: vec![1, 2, 3],
        }),
    };

    let json = JsonCodec.serialize(&data).unwrap();
    let yaml = YamlCodec.serialize(&data).unwrap();
    let bincode = BincodeCodec.serialize(&data).unwrap();

    // Bincode should be compact (binary encoding)
    // JSON and YAML are human-readable so might be similar or larger
    // Just verify all encode successfully
    assert!(!json.is_empty());
    assert!(!yaml.is_empty());
    assert!(!bincode.is_empty());
}

#[test]
fn test_codec_with_enum_variants() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    enum Status {
        Active,
        Inactive,
        Pending(String),
        Error(i32),
    }

    let variants = vec![
        Status::Active,
        Status::Inactive,
        Status::Pending("processing".to_string()),
        Status::Error(404),
    ];

    let json_codec = JsonCodec;
    for status in &variants {
        let encoded = json_codec.serialize(status).expect("encode failed");
        let decoded: Status = json_codec.deserialize(&encoded).expect("decode failed");
        assert_eq!(*status, decoded);
    }
}

#[test]
fn test_codec_with_nested_collections() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct NestedCollections {
        matrix: Vec<Vec<i32>>,
        lookup: BTreeMap<String, Vec<String>>,
    }

    let data = NestedCollections {
        matrix: vec![vec![1, 2, 3], vec![4, 5, 6]],
        lookup: {
            let mut m = BTreeMap::new();
            m.insert("group1".to_string(), vec!["a".to_string(), "b".to_string()]);
            m.insert("group2".to_string(), vec!["x".to_string(), "y".to_string()]);
            m
        },
    };

    let bincode = BincodeCodec;
    let encoded = bincode.serialize(&data).expect("encode failed");
    let decoded: NestedCollections = bincode.deserialize(&encoded).expect("decode failed");

    assert_eq!(data, decoded);
}

#[test]
fn test_cross_format_compatibility_structure() {
    // Test that we can serialize with one codec and deserialize with another (when format allows)
    let data = ComplexData {
        id: 777,
        name: "cross_format".to_string(),
        tags: vec!["test".to_string()],
        metadata: {
            let mut m = BTreeMap::new();
            m.insert("k".to_string(), "v".to_string());
            m
        },
        nested: None,
    };

    // JSON -> JSON
    let json_codec = JsonCodec;
    let json_encoded = json_codec.serialize(&data).expect("json encode failed");
    let json_decoded: ComplexData = json_codec
        .deserialize(&json_encoded)
        .expect("json decode failed");
    assert_eq!(data, json_decoded);

    // YAML -> YAML
    let yaml_codec = YamlCodec;
    let yaml_encoded = yaml_codec.serialize(&data).expect("yaml encode failed");
    let yaml_decoded: ComplexData = yaml_codec
        .deserialize(&yaml_encoded)
        .expect("yaml decode failed");
    assert_eq!(data, yaml_decoded);

    // Bincode -> Bincode
    let bincode_codec = BincodeCodec;
    let bincode_encoded = bincode_codec
        .serialize(&data)
        .expect("bincode encode failed");
    let bincode_decoded: ComplexData = bincode_codec
        .deserialize(&bincode_encoded)
        .expect("bincode decode failed");
    assert_eq!(data, bincode_decoded);
}

#[test]
fn test_codec_error_handling() {
    let json_codec = JsonCodec;

    // Invalid JSON
    let invalid_json = b"{ invalid json }";
    let result: Result<ComplexData, CodecError> = json_codec.deserialize(invalid_json);
    assert!(result.is_err());

    match result {
        Err(CodecError::Json(_)) => {} // Expected
        other => panic!("Expected Json error, got: {:?}", other),
    }
}

#[test]
fn test_compat_surface_round_trips() {
    let payload: z00z_utils::codec::Value = z00z_utils::codec::json!({
        "event": "compat",
        "count": 2,
        "nested": {
            "ok": true,
            "tags": ["json", "value"]
        }
    });

    let encoded = JsonCodec
        .serialize(&payload)
        .expect("encode compat payload");
    let decoded: z00z_utils::codec::Value = JsonCodec
        .deserialize(&encoded)
        .expect("decode compat payload");

    assert_eq!(decoded, payload);
    assert_eq!(
        decoded.get("event").and_then(|v| v.as_str()),
        Some("compat")
    );
}

#[test]
fn test_codec_preserves_data_integrity() {
    let original = ComplexData {
        id: u64::MAX,
        name: "special_chars_!@#$%^&*()".to_string(),
        tags: vec!["λ".to_string(), "你好".to_string(), "мир".to_string()],
        metadata: {
            let mut m = BTreeMap::new();
            m.insert("emoji".to_string(), "🎉🎊🎈".to_string());
            m
        },
        nested: Some(NestedData {
            value: i32::MIN,
            data: (0..=255).collect(),
        }),
    };

    // Test JSON codec
    let json_codec = JsonCodec;
    let json_encoded = json_codec.serialize(&original).expect("json encode failed");
    let json_decoded: ComplexData = json_codec
        .deserialize(&json_encoded)
        .expect("json decode failed");
    assert_eq!(original, json_decoded, "JSON codec failed");

    // Test YAML codec
    let yaml_codec = YamlCodec;
    let yaml_encoded = yaml_codec.serialize(&original).expect("yaml encode failed");
    let yaml_decoded: ComplexData = yaml_codec
        .deserialize(&yaml_encoded)
        .expect("yaml decode failed");
    assert_eq!(original, yaml_decoded, "YAML codec failed");

    // Test Bincode codec
    let bincode_codec = BincodeCodec;
    let bincode_encoded = bincode_codec
        .serialize(&original)
        .expect("bincode encode failed");
    let bincode_decoded: ComplexData = bincode_codec
        .deserialize(&bincode_encoded)
        .expect("bincode decode failed");
    assert_eq!(original, bincode_decoded, "Bincode codec failed");
}

#[test]
fn test_codec_name_methods() {
    assert_eq!(JsonCodec.name(), "json");
    assert_eq!(YamlCodec.name(), "yaml");
    assert_eq!(BincodeCodec.name(), "bincode");
}

#[test]
fn test_codec_pretty_print() {
    let data = ComplexData {
        id: 1,
        name: "pretty".to_string(),
        tags: vec!["a".to_string()],
        metadata: Default::default(),
        nested: None,
    };

    let json_codec = JsonCodec;
    let _normal = json_codec.serialize(&data).expect("encode failed");
    let pretty = json_codec
        .serialize_pretty(&data)
        .expect("pretty encode failed");

    // Pretty print should contain newlines
    let pretty_str = String::from_utf8(pretty).unwrap();
    assert!(
        pretty_str.contains('\n'),
        "Pretty should be formatted with newlines"
    );
}
