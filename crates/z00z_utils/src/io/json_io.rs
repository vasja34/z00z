use super::{
    atomic_write_with_context, load_with_context, load_with_context_bounded, DeserializeOwned,
    IoError, Path, Serialize,
};
use crate::codec::{Codec, JsonCodec};

/// Save a value to pretty-printed JSON.
pub fn save_json<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<(), IoError> {
    let path = path.as_ref();
    atomic_write_with_context(
        path,
        value,
        |v| JsonCodec.serialize_pretty(v).map_err(|e| e.to_string()),
        "json",
    )
}

/// Load a value from JSON.
pub fn load_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, IoError> {
    let path = path.as_ref();
    load_with_context(
        path,
        |bytes| JsonCodec.deserialize(bytes).map_err(|e| e.to_string()),
        "json",
    )
}

/// Load a value from JSON with an explicit file-size cap.
pub fn load_json_bounded<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    max_file_size: u64,
) -> Result<T, IoError> {
    let path = path.as_ref();
    load_with_context_bounded(
        path,
        max_file_size,
        |bytes| JsonCodec.deserialize(bytes).map_err(|e| e.to_string()),
        "json",
    )
}
