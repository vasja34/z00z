use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;
use z00z_utils::config::{ConfigError, LayeredConfig, YamlConfig, YAML_CONFIG_MAX_BYTES};
/// Integration tests for config module with YAML files and ENV
use z00z_utils::prelude::ConfigSource;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
#[test]
fn test_config_yaml_type_conversion() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
port: 8080
pool_size: 100
timeout: 30.5
enabled: true
"#;

    std::fs::write(&config_path, yaml_content).expect("write yaml failed");
    let config = YamlConfig::from_file(&config_path).expect("load yaml failed");

    let port: u16 = config
        .get_typed("port")
        .expect("get port failed")
        .expect("port not found");
    assert_eq!(port, 8080);

    let pool: u32 = config
        .get_typed("pool_size")
        .expect("get pool_size failed")
        .expect("pool_size not found");
    assert_eq!(pool, 100);

    let enabled: bool = config
        .get_typed("enabled")
        .expect("get enabled failed")
        .expect("enabled not found");
    assert!(enabled);
}
#[test]
fn test_config_yaml_not_found() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("nonexistent.yaml");

    let result = YamlConfig::from_file(&config_path);
    match result {
        Err(ConfigError::Io(err)) => assert_eq!(err.kind(), std::io::ErrorKind::NotFound),
        Ok(_) => panic!("expected not-found error, got success"),
        Err(err) => panic!("expected not-found error, got {err}"),
    }
}

#[test]
fn test_config_malformed_yaml_error() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    std::fs::write(&config_path, "server: [unterminated").expect("write yaml failed");

    let result = YamlConfig::from_file(&config_path);
    assert!(matches!(result, Err(ConfigError::Yaml(_))));
}

#[cfg(unix)]
#[test]
fn test_config_permission_denied_error() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "server:\n  port: 8080\n").expect("write yaml failed");

    let mut perms = std::fs::metadata(&config_path)
        .expect("metadata failed")
        .permissions();
    perms.set_mode(0o000);
    std::fs::set_permissions(&config_path, perms).expect("chmod failed");

    let result = YamlConfig::from_file(&config_path);

    let mut restore = std::fs::metadata(&config_path)
        .expect("metadata failed")
        .permissions();
    restore.set_mode(0o600);
    std::fs::set_permissions(&config_path, restore).expect("restore chmod failed");

    match result {
        Err(ConfigError::Io(err)) => {
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        }
        Ok(_) => panic!("expected permission-denied error, got success"),
        Err(err) => panic!("expected permission-denied error, got {err}"),
    }
}

#[cfg(unix)]
#[test]
fn test_optional_hide_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "server:\n  port: 8080\n").expect("write yaml failed");

    let mut perms = std::fs::metadata(&config_path)
        .expect("metadata failed")
        .permissions();
    perms.set_mode(0o000);
    std::fs::set_permissions(&config_path, perms).expect("chmod failed");

    let result = LayeredConfig::with_optional_yaml(&config_path);

    let mut restore = std::fs::metadata(&config_path)
        .expect("metadata failed")
        .permissions();
    restore.set_mode(0o600);
    std::fs::set_permissions(&config_path, restore).expect("restore chmod failed");

    match result {
        Err(ConfigError::Io(err)) => {
            assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        }
        Ok(_) => panic!("expected permission-denied error, got success"),
        Err(err) => panic!("expected permission-denied error, got {err}"),
    }
}

#[test]
fn test_config_yaml_oversized_error() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    let oversized = format!(
        "payload: \"{}\"\n",
        "a".repeat((YAML_CONFIG_MAX_BYTES as usize) + 1)
    );
    std::fs::write(&config_path, oversized).expect("write yaml failed");

    let result = YamlConfig::from_file(&config_path);
    match result {
        Err(ConfigError::FileTooLarge { size, max }) => {
            assert!(size > max);
            assert_eq!(max, YAML_CONFIG_MAX_BYTES);
        }
        Ok(_) => panic!("expected oversized error, got success"),
        Err(err) => panic!("expected oversized error, got {err}"),
    }
}
#[test]
fn test_scalar_leaf_is_error() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    let yaml_content = "features:\n  - auth\n  - logging\n";
    std::fs::write(&config_path, yaml_content).expect("write yaml failed");

    let config = YamlConfig::from_file(&config_path).expect("load yaml failed");
    let result = config.get("features");

    match result {
        Err(ConfigError::Parse { key, error, .. }) => {
            assert_eq!(key, "features");
            assert!(error.contains("not a scalar"));
        }
        Ok(_) => panic!("expected non-scalar error, got success"),
        Err(err) => panic!("expected parse error, got {err}"),
    }
}
#[test]
fn test_requires_exact_key_match() {
    let _guard = env_lock().lock().unwrap();
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
server:
  port: 8080
"#;

    std::fs::write(&config_path, yaml_content).expect("write yaml failed");

    std::env::set_var("SERVER_PORT", "9000");

    let layered = LayeredConfig::with_yaml(&config_path).expect("create layered failed");
    let port = layered.get("server.port").expect("get port failed");
    assert_eq!(port, Some("8080".to_string()));

    std::env::remove_var("SERVER_PORT");
}
#[test]
fn test_config_multiple_formats() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
app:
  name: test_app
  version: 1.0.0
  debug: true
server:
  port: 3000
  workers: 4
features:
  - auth
  - logging
  - metrics
"#;

    std::fs::write(&config_path, yaml_content).expect("write yaml failed");
    let config = YamlConfig::from_file(&config_path).expect("load yaml failed");

    // Test string
    let app_name = config.get("app.name").expect("get app.name failed");
    assert_eq!(app_name, Some("test_app".to_string()));

    // Test number
    let port: u16 = config
        .get_typed("server.port")
        .expect("get port failed")
        .expect("port not found");
    assert_eq!(port, 3000);

    // Test boolean
    let debug: bool = config
        .get_typed("app.debug")
        .expect("get debug failed")
        .expect("debug not found");
    assert!(debug);
}

#[test]
fn test_config_special_characters() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    let yaml_content = concat!(
        "special:\n",
        "  chars: \"!@#$%^&*()\"\n",
        "  unicode: \"Hello world – café\"\n",
        "  escaped: \"quoted \\\"value\\\"\"\n",
    );

    std::fs::write(&config_path, yaml_content).expect("write yaml failed");
    let config = YamlConfig::from_file(&config_path).expect("load yaml failed");

    let chars = config.get("special.chars").expect("get chars failed");
    assert_eq!(chars, Some("!@#$%^&*()".to_string()));

    let unicode = config.get("special.unicode").expect("get unicode failed");
    assert_eq!(unicode, Some("Hello world – café".to_string()));
}
#[test]
fn test_optional_hide_malformed_yaml() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");
    std::fs::write(&config_path, "broken: [yaml").expect("write yaml failed");

    let result = LayeredConfig::with_optional_yaml(&config_path);
    assert!(matches!(result, Err(ConfigError::Yaml(_))));
}
#[test]
fn test_optional_hide_oversized_yaml() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    let oversized = format!(
        "payload: \"{}\"\n",
        "a".repeat((YAML_CONFIG_MAX_BYTES as usize) + 1)
    );
    std::fs::write(&config_path, oversized).expect("write yaml failed");

    let result = LayeredConfig::with_optional_yaml(&config_path);
    match result {
        Err(ConfigError::FileTooLarge { size, max }) => {
            assert!(size > max);
            assert_eq!(max, YAML_CONFIG_MAX_BYTES);
        }
        Ok(_) => panic!("expected oversized error, got success"),
        Err(err) => panic!("expected oversized error, got {err}"),
    }
}

#[test]
fn test_config_type_conversion_failure() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
not_a_number: "abc"
"#;

    std::fs::write(&config_path, yaml_content).expect("write yaml failed");
    let config = YamlConfig::from_file(&config_path).expect("load yaml failed");

    let result: Result<Option<u32>, ConfigError> = config.get_typed("not_a_number");
    assert!(result.is_err());
}
#[test]
fn test_config_large_nested_structure() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config_path = temp_dir.path().join("config.yaml");

    let yaml_content = r#"
level1:
  level2:
    level3:
      level4:
        level5:
          value: deep
"#;

    std::fs::write(&config_path, yaml_content).expect("write yaml failed");
    let config = YamlConfig::from_file(&config_path).expect("load yaml failed");

    let value = config
        .get("level1.level2.level3.level4.level5.value")
        .expect("get deep value failed");
    assert_eq!(value, Some("deep".to_string()));
}
