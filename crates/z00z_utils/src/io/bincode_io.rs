use super::{
    atomic_write_with_context, load_with_context, load_with_context_bounded, DeserializeOwned,
    IoError, Path, Serialize,
};
use crate::codec::{BincodeCodec, Codec};

/// Save a value to compact bincode bytes.
pub fn save_bincode<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<(), IoError> {
    let path = path.as_ref();
    atomic_write_with_context(
        path,
        value,
        |v| BincodeCodec.serialize(v).map_err(|e| e.to_string()),
        "bincode",
    )
}

/// Load a value from bincode bytes.
pub fn load_bincode<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, IoError> {
    let path = path.as_ref();
    load_with_context(
        path,
        |bytes| BincodeCodec.deserialize(bytes).map_err(|e| e.to_string()),
        "bincode",
    )
}

/// Load a value from bincode bytes with an explicit file-size cap.
pub fn load_bincode_bounded<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    max_file_size: u64,
) -> Result<T, IoError> {
    let path = path.as_ref();
    load_with_context_bounded(
        path,
        max_file_size,
        |bytes| {
            BincodeCodec
                .deserialize_bounded(bytes, max_file_size)
                .map_err(|e| e.to_string())
        },
        "bincode",
    )
}
