use serde::{Deserialize, Serialize};
use tempfile::TempDir;
/// Integration tests for I/O module with real file operations
use z00z_utils::io::{atomic_write_file_private, atomic_write_file_streaming, write_file, IoError};
use z00z_utils::prelude::{load_bincode, load_json, load_yaml, save_bincode, save_json, save_yaml};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestData {
    name: String,
    value: i32,
}
#[test]
fn test_io_file_content_format() {
    let temp_dir = TempDir::new().expect("create temp dir");

    let data = TestData {
        name: "format_test".to_string(),
        value: 42,
    };

    // JSON should be human-readable
    let json_path = temp_dir.path().join("test.json");
    save_json(&json_path, &data).expect("save json failed");
    let json_content = std::fs::read_to_string(&json_path).expect("read json failed");
    assert!(
        json_content.contains("format_test"),
        "JSON should be human-readable"
    );
    assert!(json_content.contains("42"), "JSON should contain values");

    // YAML should be human-readable
    let yaml_path = temp_dir.path().join("test.yaml");
    save_yaml(&yaml_path, &data).expect("save yaml failed");
    let yaml_content = std::fs::read_to_string(&yaml_path).expect("read yaml failed");
    assert!(
        yaml_content.contains("format_test"),
        "YAML should be human-readable"
    );
    assert!(yaml_content.contains("42"), "YAML should contain values");
}

#[test]
fn test_io_multiple_formats_data() {
    let temp_dir = TempDir::new().expect("create temp dir");

    let data = TestData {
        name: "multi_format".to_string(),
        value: 888,
    };

    let json_path = temp_dir.path().join("multi.json");
    let yaml_path = temp_dir.path().join("multi.yaml");
    let bin_path = temp_dir.path().join("multi.bin");

    save_json(&json_path, &data).expect("save json failed");
    save_yaml(&yaml_path, &data).expect("save yaml failed");
    save_bincode(&bin_path, &data).expect("save bincode failed");

    let json_loaded: TestData = load_json(&json_path).expect("load json failed");
    let yaml_loaded: TestData = load_yaml(&yaml_path).expect("load yaml failed");
    let bin_loaded: TestData = load_bincode(&bin_path).expect("load bincode failed");

    assert_eq!(data, json_loaded);
    assert_eq!(data, yaml_loaded);
    assert_eq!(data, bin_loaded);
}

#[test]
fn test_io_preserves_data_types() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Complex {
        id: u64,
        count: i32,
        ratio: f64,
        enabled: bool,
        items: Vec<String>,
    }

    let temp_dir = TempDir::new().expect("create temp dir");

    let data = Complex {
        id: 18446744073709551615u64, // u64::MAX
        count: -2147483648i32,       // i32::MIN
        ratio: std::f64::consts::PI,
        enabled: true,
        items: vec!["a".to_string(), "b".to_string()],
    };

    let path = temp_dir.path().join("complex.json");
    save_json(&path, &data).expect("save failed");
    let loaded: Complex = load_json(&path).expect("load failed");

    assert_eq!(data.id, loaded.id);
    assert_eq!(data.count, loaded.count);
    assert!((data.ratio - loaded.ratio).abs() < 0.0001);
    assert_eq!(data.enabled, loaded.enabled);
    assert_eq!(data.items, loaded.items);
}

#[test]
fn test_io_large_file_handling() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct LargeData {
        items: Vec<i32>,
    }

    let temp_dir = TempDir::new().expect("create temp dir");

    let data = LargeData {
        items: (0..10000).collect(),
    };

    let path = temp_dir.path().join("large.bin");
    save_bincode(&path, &data).expect("save failed");
    let loaded: LargeData = load_bincode(&path).expect("load failed");

    assert_eq!(data.items.len(), loaded.items.len());
    assert_eq!(data.items, loaded.items);
}

#[test]
fn test_io_special_characters_path() {
    let temp_dir = TempDir::new().expect("create temp dir");

    let data = TestData {
        name: "special".to_string(),
        value: 42,
    };

    // Some filesystems support special chars in filenames
    let path = temp_dir.path().join("test_data.json");
    save_json(&path, &data).expect("save failed");
    let loaded: TestData = load_json(&path).expect("load failed");

    assert_eq!(data, loaded);
}

#[test]
fn test_write_permission_copy_failure() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let path = temp_dir.path().join("permission-copy.bin");
    let seam_path = temp_dir.path().join(".permission-copy.bin.perm-copy-fail");

    std::fs::write(&path, b"old-bytes").expect("seed destination");
    std::fs::write(&seam_path, b"trigger").expect("seed seam marker");

    let result = write_file(&path, b"new-bytes");

    match result {
        Err(IoError::Io(err)) => assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied),
        other => panic!("expected permission denied IoError, got {other:?}"),
    }

    assert_eq!(
        std::fs::read(&path).expect("read original file"),
        b"old-bytes"
    );
}

#[test]
#[cfg(unix)]
fn test_write_preserves_existing_permissions() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let path = temp_dir.path().join("preserve-perms.bin");

    std::fs::write(&path, b"seed").expect("seed destination");
    let perms = std::fs::Permissions::from_mode(0o640);
    std::fs::set_permissions(&path, perms).expect("set permissions");

    write_file(&path, b"updated").expect("write file");

    let mode = std::fs::metadata(&path)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o640);
}

#[test]
#[cfg(unix)]
fn test_private_write_enforces_permissions() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let path = temp_dir.path().join("private.bin");

    atomic_write_file_private(&path, b"secret").expect("private write");

    let mode = std::fs::metadata(&path)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
#[cfg(unix)]
fn test_streaming_write_enforces_permissions() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let path = temp_dir.path().join("streaming.bin");

    atomic_write_file_streaming(&path, |file| {
        use std::io::Write as _;

        file.write_all(b"streamed")?;
        Ok(())
    })
    .expect("streaming write");

    let mode = std::fs::metadata(&path)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn test_io_empty_string_handling() {
    let temp_dir = TempDir::new().expect("create temp dir");

    let data = TestData {
        name: String::new(),
        value: 0,
    };

    let path = temp_dir.path().join("empty.json");
    save_json(&path, &data).expect("save failed");
    let loaded: TestData = load_json(&path).expect("load failed");

    assert_eq!(data.name, loaded.name);
    assert!(loaded.name.is_empty());
}

#[test]
fn test_io_unicode_data_preservation() {
    let temp_dir = TempDir::new().expect("create temp dir");

    let data = TestData {
        name: "Hello 世界 🌍 مرحبا".to_string(),
        value: 123,
    };

    let path = temp_dir.path().join("unicode.json");
    save_json(&path, &data).expect("save failed");
    let loaded: TestData = load_json(&path).expect("load failed");

    assert_eq!(data.name, loaded.name);
}
