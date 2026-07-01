//! Codec module tests

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct TestData {
    id: u32,
    name: String,
    tags: Vec<String>,
    active: bool,
    score: f64,
}

#[cfg(test)]
mod json_tests {
    use super::*;
    use crate::codec::{Codec, JsonCodec};

    #[test]
    fn test_json_all_types() {
        let codec = JsonCodec;
        let data = TestData {
            id: 42,
            name: "test".to_string(),
            tags: vec!["rust".to_string(), "codec".to_string()],
            active: true,
            score: 9.5,
        };

        let bytes = codec.serialize(&data).unwrap();
        let decoded: TestData = codec.deserialize(&bytes).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_json_pretty_vs_compact() {
        let codec = JsonCodec;
        let data = TestData {
            id: 1,
            name: "data".to_string(),
            tags: vec![],
            active: false,
            score: 1.0,
        };

        let compact = codec.serialize(&data).unwrap();
        let pretty = codec.serialize_pretty(&data).unwrap();

        // Pretty should be larger due to whitespace
        assert!(pretty.len() > compact.len());

        // Both should deserialize to same value
        let from_compact: TestData = codec.deserialize(&compact).unwrap();
        let from_pretty: TestData = codec.deserialize(&pretty).unwrap();
        assert_eq!(from_compact, from_pretty);
    }
}

#[cfg(test)]
mod bincode_tests {
    use super::*;
    use crate::codec::{BincodeCodec, Codec};

    #[test]
    fn test_bincode_all_types() {
        let codec = BincodeCodec;
        let data = TestData {
            id: 99,
            name: "bincode-test".to_string(),
            tags: vec!["binary".to_string(), "efficient".to_string()],
            active: false,
            score: 8.7,
        };

        let bytes = codec.serialize(&data).unwrap();
        let decoded: TestData = codec.deserialize(&bytes).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_bincode_compact() {
        let codec = BincodeCodec;
        let data = TestData {
            id: 12345,
            name: "very long name that should be compressed in bincode format".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
            active: true,
            score: std::f64::consts::PI,
        };

        let bytes = codec.serialize(&data).unwrap();
        // Should be reasonably compact
        assert!(bytes.len() < 200);
    }
}

#[cfg(test)]
mod yaml_tests {
    use super::*;
    use crate::codec::{Codec, YamlCodec};

    #[test]
    fn test_yaml_all_types() {
        let codec = YamlCodec;
        let data = TestData {
            id: 7,
            name: "yaml-config".to_string(),
            tags: vec!["config".to_string(), "human-readable".to_string()],
            active: true,
            score: 10.0,
        };

        let bytes = codec.serialize(&data).unwrap();
        let decoded: TestData = codec.deserialize(&bytes).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_yaml_readability() {
        let codec = YamlCodec;
        let data = TestData {
            id: 1,
            name: "readable".to_string(),
            tags: vec!["format".to_string()],
            active: true,
            score: 5.0,
        };

        let bytes = codec.serialize(&data).unwrap();
        let yaml_str = String::from_utf8(bytes).unwrap();

        // YAML should be human-readable
        assert!(yaml_str.contains("id:"));
        assert!(yaml_str.contains("name:"));
        assert!(yaml_str.contains("tags:"));
        assert!(yaml_str.contains("active:"));
        assert!(yaml_str.contains("score:"));
    }
}

#[cfg(test)]
mod cross_codec_tests {
    use super::*;
    use crate::codec::{BincodeCodec, Codec, JsonCodec, YamlCodec};

    #[test]
    fn test_all_codecs_compatible() {
        let data = TestData {
            id: 100,
            name: "cross-test".to_string(),
            tags: vec![
                "json".to_string(),
                "bincode".to_string(),
                "yaml".to_string(),
            ],
            active: true,
            score: 7.5,
        };

        // Each codec should round-trip correctly
        let json_codec = JsonCodec;
        let json_bytes = json_codec.serialize(&data).unwrap();
        let json_result: TestData = json_codec.deserialize(&json_bytes).unwrap();
        assert_eq!(data, json_result);

        let bincode_codec = BincodeCodec;
        let bincode_bytes = bincode_codec.serialize(&data).unwrap();
        let bincode_result: TestData = bincode_codec.deserialize(&bincode_bytes).unwrap();
        assert_eq!(data, bincode_result);

        let yaml_codec = YamlCodec;
        let yaml_bytes = yaml_codec.serialize(&data).unwrap();
        let yaml_result: TestData = yaml_codec.deserialize(&yaml_bytes).unwrap();
        assert_eq!(data, yaml_result);
    }

    #[test]
    fn test_codec_names() {
        let json = JsonCodec;
        let bincode = BincodeCodec;
        let yaml = YamlCodec;

        assert_eq!(json.name(), "json");
        assert_eq!(bincode.name(), "bincode");
        assert_eq!(yaml.name(), "yaml");
    }

    #[test]
    fn test_format_sizes() {
        let data = TestData {
            id: 12345,
            name: "test".repeat(10),
            tags: (0..20).map(|i| format!("tag{}", i)).collect(),
            active: true,
            score: std::f64::consts::PI,
        };

        let json_bytes = JsonCodec.serialize(&data).unwrap();
        let bincode_bytes = BincodeCodec.serialize(&data).unwrap();
        let yaml_bytes = YamlCodec.serialize(&data).unwrap();

        // All formats should serialize successfully
        assert!(!json_bytes.is_empty());
        assert!(!bincode_bytes.is_empty());
        assert!(!yaml_bytes.is_empty());

        println!("JSON size: {}", json_bytes.len());
        println!("Bincode size: {}", bincode_bytes.len());
        println!("YAML size: {}", yaml_bytes.len());
    }
}
