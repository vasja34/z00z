//! Genesis Serialization/Deserialization Module
//!
//! Handles import/export of genesis assets in various formats (JSON, Bincode).
//! Includes version compatibility checking and format validation.
//! Uses z00z_utils codec abstraction for consistent serialization.
//! Includes atomic file I/O for safe genesis asset export.

use crate::assets::{Asset, AssetWire};
use crate::genesis::genesis_config::OutputsConfig;
use crate::genesis::validator::{validate_version_compatibility, GenesisError};
use crate::genesis::GenesisAssetAccumulator;
use std::path::Path;

use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};
use z00z_utils::io::{create_dir_all, read_to_string, remove_file, rename_file, write_file};

/// Container for genesis state export
///
/// Contains only generated assets + metadata, NOT config.
/// Config is input-only, not part of genesis output.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenesisStateExport {
    pub version: String,
    pub assets: GenesisAssetAccumulator,
    pub timestamp: String, // ISO 8601 format
    pub total_count: usize,
}

impl GenesisStateExport {
    /// Export to JSON bytes using z00z_utils codec
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, String> {
        let codec = JsonCodec;
        codec.serialize(self).map_err(|e| e.to_string())
    }

    /// Import from JSON bytes using z00z_utils codec
    pub fn from_json_bytes(data: &[u8]) -> Result<Self, String> {
        let codec = JsonCodec;
        codec.deserialize(data).map_err(|e| e.to_string())
    }

    /// Export to Bincode bytes using z00z_utils codec
    pub fn to_bincode_bytes(&self) -> Result<Vec<u8>, String> {
        let codec = BincodeCodec;
        codec.serialize(self).map_err(|e| e.to_string())
    }

    /// Import from Bincode bytes using z00z_utils codec
    pub fn from_bincode_bytes(data: &[u8]) -> Result<Self, String> {
        let codec = BincodeCodec;
        codec.deserialize(data).map_err(|e| e.to_string())
    }
}

// ============================================================================
// File I/O Functions (Atomic Writes)
// ============================================================================

/// Export genesis assets to JSON and Bincode files
///
/// Writes two files atomically:
/// - genesis_{symbol}.json
/// - genesis_{symbol}.bin
///
/// Prevents partial writes from corrupting genesis state.
///
/// # Serialization
///
/// This function uses `z00z_utils::codec` to serialize and then performs atomic file writes.
/// The atomic write pattern requires passing raw bytes to ensure POSIX rename atomicity.
pub fn export_genesis_assets(
    assets: &[Asset],
    symbol: &str,
    config: &OutputsConfig,
) -> Result<(), GenesisError> {
    let base_path = Path::new(&config.assets_export_path);

    let export_entries: Vec<AssetWire> = assets.iter().map(AssetWire::from_asset).collect();

    // JSON export using JsonCodec for consistency (pretty formatted)
    let codec = JsonCodec;
    let json_bytes = codec
        .serialize_pretty(&export_entries)
        .map_err(|e| GenesisError::SerializationFailed(e.to_string()))?;
    let json_path = base_path.join(format!("genesis_{}.json", symbol));
    atomic_write(&json_path, &json_bytes)?;

    // Bincode export using z00z_utils codec abstraction
    let codec = BincodeCodec;
    let bytes = codec
        .serialize(&export_entries)
        .map_err(|e| GenesisError::SerializationFailed(e.to_string()))?;
    let bin_path = base_path.join(format!("genesis_{}.bin", symbol));
    atomic_write(&bin_path, &bytes)?;

    Ok(())
}

/// Atomic file write (tmp → rename) with automatic cleanup
///
/// Prevents partial writes from corrupting genesis state.
/// Uses CleanupGuard to ensure temp files are removed on failure.
///
/// # Why Direct std::fs Instead of z00z_utils::io?
///
/// This function requires POSIX atomic rename semantics (z00z_utils::io::rename_file preserves this) which are not
/// available in z00z_utils::io abstraction. The two-phase commit pattern (write tmp,
/// then atomic rename) is critical for genesis integrity and needs direct fs control.
/// This is a low-level operation that cannot be abstracted without losing atomicity guarantees.
pub(crate) fn atomic_write(path: &Path, data: &[u8]) -> Result<(), GenesisError> {
    let tmp_path = path.with_extension("tmp");

    // RAII cleanup guard - automatically removes temp file on failure
    struct CleanupGuard<'a> {
        path: &'a Path,
        armed: bool,
    }

    impl<'a> Drop for CleanupGuard<'a> {
        fn drop(&mut self) {
            if self.armed {
                // Silently remove temp file on failure
                let _ = remove_file(self.path);
            }
        }
    }

    let mut guard = CleanupGuard {
        path: &tmp_path,
        armed: true,
    };

    // Write to temp file
    write_file(&tmp_path, data).map_err(|e| GenesisError::FileWriteFailed {
        path: tmp_path.display().to_string(),
        error: e.to_string(),
    })?;

    // Atomic rename (POSIX guarantees atomicity)
    rename_file(&tmp_path, path).map_err(|e| GenesisError::FileWriteFailed {
        path: path.display().to_string(),
        error: e.to_string(),
    })?;

    // Success: disarm cleanup guard
    guard.armed = false;

    Ok(())
}

// ============================================================================
// Import/Export Functions (Task 5.2, 5.3)
// ============================================================================

/// Import genesis Assets from JSON
///
/// **Spec Reference**: Lines 3272-3282 (genesis_spec_release_3.md)
///
/// Validates version compatibility before deserializing.
pub fn import_genesis_json(json: &str) -> Result<GenesisAssetAccumulator, GenesisError> {
    let codec = JsonCodec;
    let state: GenesisStateExport = codec
        .deserialize(json.as_bytes())
        .map_err(|e| GenesisError::SerializationFailed(e.to_string()))?;

    validate_version_compatibility(&state.version)?;

    Ok(state.assets)
}

/// Import genesis Assets from Bincode
///
/// **Spec Reference**: Lines 3284-3295 (genesis_spec_release_3.md)
pub fn import_genesis_binary(data: &[u8]) -> Result<GenesisAssetAccumulator, GenesisError> {
    let codec = BincodeCodec;
    let state: GenesisStateExport = codec
        .deserialize(data)
        .map_err(|e| GenesisError::SerializationFailed(e.to_string()))?;

    validate_version_compatibility(&state.version)?;

    Ok(state.assets)
}

/// Extract genesis Assets to directory (one file per class)
///
/// **Spec Reference**: Lines 3302-3329 (genesis_spec_release_3.md)
///
/// Creates:
/// - genesis_coins.json
/// - genesis_tokens.json
/// - genesis_nfts.json
/// - genesis_voids.json
///
/// Uses z00z_utils::io::create_dir_all for directory creation.
/// Calls atomic_write for safe file operations.
pub fn extract_genesis_assets(
    accumulator: &GenesisAssetAccumulator,
    output_dir: &Path,
) -> Result<Vec<std::path::PathBuf>, GenesisError> {
    // Create output directory using z00z_utils::io
    create_dir_all(output_dir).map_err(|e| GenesisError::FileWriteFailed {
        path: output_dir.display().to_string(),
        error: e.to_string(),
    })?;

    let mut paths = Vec::new();

    for (filename, assets) in [
        ("genesis_coins.json", &accumulator.coins),
        ("genesis_tokens.json", &accumulator.tokens),
        ("genesis_nfts.json", &accumulator.nfts),
        ("genesis_voids.json", &accumulator.voids),
    ] {
        let path = output_dir.join(filename);
        let codec = JsonCodec;
        let json_bytes = codec
            .serialize(&assets)
            .map_err(|e| GenesisError::SerializationFailed(e.to_string()))?;
        atomic_write(&path, &json_bytes)?;
        paths.push(path);
    }

    Ok(paths)
}

/// Load genesis Assets from extracted directory
///
/// **Spec Reference**: Lines 3331-3356 (genesis_spec_release_3.md)
pub fn load_genesis_assets(input_dir: &Path) -> Result<GenesisAssetAccumulator, GenesisError> {
    let mut accumulator = GenesisAssetAccumulator::new();

    accumulator.coins = load_asset_class_file(input_dir.join("genesis_coins.json"))?;
    accumulator.tokens = load_asset_class_file(input_dir.join("genesis_tokens.json"))?;
    accumulator.nfts = load_asset_class_file(input_dir.join("genesis_nfts.json"))?;
    accumulator.voids = load_asset_class_file(input_dir.join("genesis_voids.json"))?;

    Ok(accumulator)
}

/// Load single asset class file
///
/// **Spec Reference**: Lines 3358-3372 (genesis_spec_release_3.md)
fn load_asset_class_file(path: std::path::PathBuf) -> Result<Vec<Asset>, GenesisError> {
    let json = read_to_string(&path).map_err(|e| GenesisError::FileWriteFailed {
        path: path.display().to_string(),
        error: e.to_string(),
    })?;

    let codec = JsonCodec;
    codec
        .deserialize(json.as_bytes())
        .map_err(|e| GenesisError::SerializationFailed(e.to_string()))
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_state_export_creation() {
        let export = GenesisStateExport {
            version: "3.8.0".to_string(),
            assets: GenesisAssetAccumulator::new(),
            timestamp: "2024-12-04T00:00:00Z".to_string(),
            total_count: 0,
        };

        assert_eq!(export.version, "3.8.0");
        assert_eq!(export.total_count, 0);
        assert_eq!(export.assets.total_count(), 0);
    }

    #[test]
    fn test_genesis_state_export_serialization() {
        let export = GenesisStateExport {
            version: "3.8.0".to_string(),
            assets: GenesisAssetAccumulator::new(),
            timestamp: "2024-12-04T00:00:00Z".to_string(),
            total_count: 0,
        };

        // Test JSON serialization using JsonCodec
        let codec = JsonCodec;
        let json_bytes = codec.serialize(&export).unwrap();
        let json = String::from_utf8(json_bytes.clone()).unwrap();
        assert!(json.contains("3.8.0"));
        assert!(json.contains("2024-12-04T00:00:00Z"));

        // Test deserialization
        let deserialized: GenesisStateExport = codec.deserialize(&json_bytes).unwrap();
        assert_eq!(deserialized.version, export.version);
        assert_eq!(deserialized.total_count, export.total_count);
    }

    #[test]
    fn test_import_genesis_json() {
        let export = GenesisStateExport {
            version: "3.8.0".to_string(),
            assets: GenesisAssetAccumulator::new(),
            timestamp: "2024-12-04T00:00:00Z".to_string(),
            total_count: 0,
        };

        let codec = JsonCodec;
        let json_bytes = codec.serialize(&export).unwrap();
        let json = String::from_utf8(json_bytes).unwrap();
        let accumulator = import_genesis_json(&json).unwrap();

        assert_eq!(accumulator.total_count(), 0);
        assert_eq!(accumulator.coins.len(), 0);
    }

    #[test]
    fn test_import_genesis_binary() {
        let export = GenesisStateExport {
            version: "3.8.0".to_string(),
            assets: GenesisAssetAccumulator::new(),
            timestamp: "2024-12-04T00:00:00Z".to_string(),
            total_count: 0,
        };

        let codec = BincodeCodec;
        let bytes = codec.serialize(&export).unwrap();
        let accumulator = import_genesis_binary(&bytes).unwrap();

        assert_eq!(accumulator.total_count(), 0);
    }

    #[test]
    fn test_validate_version_compatibility() {
        // Supported versions
        assert!(validate_version_compatibility("3.0").is_ok());
        assert!(validate_version_compatibility("3.8").is_ok());
        assert!(validate_version_compatibility("3.8.0").is_ok());

        // Unsupported version
        assert!(validate_version_compatibility("2.0").is_err());
        assert!(validate_version_compatibility("4.0").is_err());
    }

    #[test]
    fn test_extract_load_genesis_assets() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let accumulator = GenesisAssetAccumulator::new();

        // Extract to temp directory
        let paths = extract_genesis_assets(&accumulator, temp_dir.path()).unwrap();
        assert_eq!(paths.len(), 4);

        // Verify files exist
        assert!(temp_dir.path().join("genesis_coins.json").exists());
        assert!(temp_dir.path().join("genesis_tokens.json").exists());
        assert!(temp_dir.path().join("genesis_nfts.json").exists());
        assert!(temp_dir.path().join("genesis_voids.json").exists());

        // Load back
        let loaded = load_genesis_assets(temp_dir.path()).unwrap();
        assert_eq!(loaded.total_count(), accumulator.total_count());
    }
}
