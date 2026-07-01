use super::{
    atomic_write_with_context, load_with_context, load_with_context_bounded, DeserializeOwned,
    IoError, Path, Serialize,
};
use crate::codec::{Codec, YamlCodec};

/// Save a value to YAML.
pub fn save_yaml<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<(), IoError> {
    let path = path.as_ref();
    atomic_write_with_context(
        path,
        value,
        |v| YamlCodec.serialize(v).map_err(|e| e.to_string()),
        "yaml",
    )
}

/// Load a value from YAML.
pub fn load_yaml<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, IoError> {
    let path = path.as_ref();
    load_with_context(
        path,
        |bytes| YamlCodec.deserialize(bytes).map_err(|e| e.to_string()),
        "yaml",
    )
}

/// Load a value from YAML with an explicit file-size cap.
pub fn load_yaml_bounded<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    max_file_size: u64,
) -> Result<T, IoError> {
    let path = path.as_ref();
    load_with_context_bounded(
        path,
        max_file_size,
        |bytes| YamlCodec.deserialize(bytes).map_err(|e| e.to_string()),
        "yaml",
    )
}
