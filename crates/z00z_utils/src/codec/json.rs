//! JSON codec implementation

use super::traits::{Codec, CodecError};
use serde::{de::DeserializeOwned, Serialize};

/// JSON codec using serde_json
///
/// Serializes/deserializes data to/from JSON format.
/// Useful for human-readable configuration files and API responses.
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::codec::{Codec, JsonCodec};
///
/// let codec = JsonCodec;
/// let data = vec![1, 2, 3];
/// let bytes = codec.serialize(&data)?;
/// assert!(bytes.starts_with(b"["));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    type Error = CodecError;

    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(value).map_err(|e| CodecError::Json(e.to_string()))
    }

    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T, Self::Error> {
        let mut stream = serde_json::Deserializer::from_slice(bytes).into_iter::<T>();
        let value = stream
            .next()
            .ok_or_else(|| CodecError::Json("empty JSON input".to_string()))?
            .map_err(|e| CodecError::Json(e.to_string()))?;

        // Reject additional values or trailing non-whitespace.
        if stream.next().is_some() {
            return Err(CodecError::TrailingBytes {
                consumed: stream.byte_offset(),
                total: bytes.len(),
            });
        }

        Ok(value)
    }

    fn serialize_pretty<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec_pretty(value).map_err(|e| CodecError::Json(e.to_string()))
    }

    fn name(&self) -> &'static str {
        "json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    #[test]
    fn test_json_codec_serialize() {
        let codec = JsonCodec;
        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        let bytes = codec.serialize(&data).unwrap();
        assert!(!bytes.is_empty());
        let json_str = String::from_utf8(bytes).unwrap();
        assert!(json_str.contains("\"name\""));
        assert!(json_str.contains("\"test\""));
    }

    #[test]
    fn test_json_codec_deserialize() {
        let codec = JsonCodec;
        let json = r#"{"name":"hello","value":99}"#;

        let result: TestStruct = codec.deserialize(json.as_bytes()).unwrap();
        assert_eq!(result.name, "hello");
        assert_eq!(result.value, 99);
    }

    #[test]
    fn test_json_trailing_data_rejected() {
        let codec = JsonCodec;
        let json = r#"{"name":"hello","value":99} {"name":"x","value":1}"#;

        let result: Result<TestStruct, _> = codec.deserialize(json.as_bytes());
        assert!(matches!(result, Err(CodecError::TrailingBytes { .. })));
    }

    #[test]
    fn test_json_trailing_garbage_rejected() {
        let codec = JsonCodec;
        let json = r#"{"name":"hello","value":99}GARBAGE"#;

        let result: Result<TestStruct, _> = codec.deserialize(json.as_bytes());
        assert!(matches!(result, Err(CodecError::TrailingBytes { .. })));
    }

    #[test]
    fn test_json_codec_round_trip() {
        let codec = JsonCodec;
        let original = TestStruct {
            name: "round-trip".to_string(),
            value: 123,
        };

        let bytes = codec.serialize(&original).unwrap();
        let deserialized: TestStruct = codec.deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_json_codec_pretty() {
        let codec = JsonCodec;
        let data = TestStruct {
            name: "pretty".to_string(),
            value: 42,
        };

        let bytes = codec.serialize_pretty(&data).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();
        // Pretty print should have newlines
        assert!(json_str.contains('\n'));
    }

    #[test]
    fn test_json_codec_name() {
        let codec = JsonCodec;
        assert_eq!(codec.name(), "json");
    }

    #[test]
    fn test_json_codec_error_handling() {
        let codec = JsonCodec;
        let invalid_json = b"not valid json {";

        let result: Result<TestStruct, _> = codec.deserialize(invalid_json);
        assert!(result.is_err());
        match result.unwrap_err() {
            CodecError::Json(_) => {} // Expected
            _ => panic!("Expected Json error"),
        }
    }

    #[test]
    fn test_json_codec_with_primitives() {
        let codec = JsonCodec;

        // String
        let s = "hello world".to_string();
        let bytes = codec.serialize(&s).unwrap();
        let decoded: String = codec.deserialize(&bytes).unwrap();
        assert_eq!(s, decoded);

        // i32
        let num = 42i32;
        let bytes = codec.serialize(&num).unwrap();
        let decoded: i32 = codec.deserialize(&bytes).unwrap();
        assert_eq!(num, decoded);

        // Vec
        let vec = vec![1, 2, 3, 4, 5];
        let bytes = codec.serialize(&vec).unwrap();
        let decoded: Vec<i32> = codec.deserialize(&bytes).unwrap();
        assert_eq!(vec, decoded);
    }
}
