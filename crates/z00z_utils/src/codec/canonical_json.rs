use super::{CodecError, Value};
use serde::Serialize;

/// Serialize a value into stable canonical JSON bytes.
pub fn to_canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CodecError> {
    let value = serde_json::to_value(value).map_err(|err| CodecError::Json(err.to_string()))?;
    let mut output = Vec::new();
    write_value(&value, &mut output)?;
    Ok(output)
}

fn write_value(value: &Value, output: &mut Vec<u8>) -> Result<(), CodecError> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(flag) => {
            if *flag {
                output.extend_from_slice(b"true");
            } else {
                output.extend_from_slice(b"false");
            }
        }
        Value::Number(number) => output.extend_from_slice(number.to_string().as_bytes()),
        Value::String(string) => {
            let encoded =
                serde_json::to_string(string).map_err(|err| CodecError::Json(err.to_string()))?;
            output.extend_from_slice(encoded.as_bytes());
        }
        Value::Array(items) => {
            output.push(b'[');
            for (index, item) in items.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_value(item, output)?;
            }
            output.push(b']');
        }
        Value::Object(entries) => {
            output.push(b'{');
            let mut keys = entries.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            for (index, key) in keys.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                let encoded =
                    serde_json::to_string(key).map_err(|err| CodecError::Json(err.to_string()))?;
                output.extend_from_slice(encoded.as_bytes());
                output.push(b':');
                let entry = entries.get(*key).ok_or_else(|| {
                    CodecError::Json(format!("canonical JSON key missing: {key}"))
                })?;
                write_value(entry, output)?;
            }
            output.push(b'}');
        }
    }

    Ok(())
}
