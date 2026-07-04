//! ConfigSource tests

use crate::config::{ConfigSource, EnvConfig, LayeredConfig, YamlConfig};
use std::env;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use tempfile::NamedTempFile;

fn cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn test_env_config_get_existing() {
    let _guard = env_lock().lock().unwrap();
    env::set_var("TEST_KEY_UTILS_1", "test_value");
    let config = EnvConfig;
    let result = config.get("TEST_KEY_UTILS_1").unwrap();
    assert_eq!(result, Some("test_value".to_string()));
    env::remove_var("TEST_KEY_UTILS_1");
}

#[test]
fn test_env_config_get_missing() {
    let config = EnvConfig;
    let result = config.get("NONEXISTENT_KEY_XYZ_123").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_env_config_get_typed() {
    let _guard = env_lock().lock().unwrap();
    env::set_var("TEST_PORT_UTILS", "8080");
    let config = EnvConfig;
    let result = config.get_typed::<u16>("TEST_PORT_UTILS").unwrap();
    assert_eq!(result, Some(8080));
    env::remove_var("TEST_PORT_UTILS");
}

#[test]
fn test_yaml_config_from_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "server:\n  port: 8080\n  host: localhost").unwrap();

    let config = YamlConfig::from_file(file.path()).unwrap();
    let port = config.get("server.port").unwrap();
    assert_eq!(port, Some("8080".to_string()));

    let host = config.get("server.host").unwrap();
    assert_eq!(host, Some("localhost".to_string()));
}

#[test]
fn test_yaml_config_nested_navigation() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "assets:\n  default:\n    decimals: 8").unwrap();

    let config = YamlConfig::from_file(file.path()).unwrap();
    let decimals = config.get("assets.default.decimals").unwrap();
    assert_eq!(decimals, Some("8".to_string()));
}

#[test]
fn test_yaml_config_missing_key() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "server:\n  port: 8080").unwrap();

    let config = YamlConfig::from_file(file.path()).unwrap();
    let missing = config.get("nonexistent.key").unwrap();
    assert_eq!(missing, None);
}

#[test]
fn test_config_leaf_is_error() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "server:\n  host: null").unwrap();

    let config = YamlConfig::from_file(file.path()).unwrap();
    let result = config.get("server.host");

    match result {
        Err(crate::config::ConfigError::Parse { key, value, error }) => {
            assert_eq!(key, "server.host");
            assert_eq!(value, "<null-yaml>");
            assert!(error.contains("null"));
        }
        Ok(_) => panic!("expected null-leaf error, got success"),
        Err(err) => panic!("expected parse error, got {err}"),
    }
}

/// CRITICAL TEST: LayeredConfig priority order
#[test]
fn test_layered_config_priority_env() {
    let _guard = env_lock().lock().unwrap();
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "port: 8080").unwrap();

    env::set_var("PORT", "9090");
    let config = LayeredConfig::with_yaml(file.path()).unwrap();

    let port = config.get("PORT").unwrap();
    assert_eq!(port, Some("9090".to_string()), "ENV should win over YAML");

    env::remove_var("PORT");
}

/// CRITICAL TEST: LayeredConfig falls back to YAML when ENV not set
#[test]
fn test_layered_config_priority_yaml() {
    let _guard = env_lock().lock().unwrap();
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "port: 8080").unwrap();

    env::remove_var("PORT_YAML_FALLBACK_TEST");
    let config = LayeredConfig::with_yaml(file.path()).unwrap();

    let port = config.get("port").unwrap();
    assert_eq!(
        port,
        Some("8080".to_string()),
        "YAML should be used when ENV not set"
    );
}

/// CRITICAL TEST: LayeredConfig returns None when key not found anywhere
#[test]
fn test_layered_config_priority_none() {
    let _guard = env_lock().lock().unwrap();
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "port: 8080").unwrap();

    env::remove_var("MISSING_KEY_XYZ");
    let config = LayeredConfig::with_yaml(file.path()).unwrap();

    let missing = config.get("MISSING_KEY_XYZ").unwrap();
    assert_eq!(missing, None, "Should return None when key not found");
}

#[test]
fn test_layered_config_get_typed() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "port: 8080").unwrap();

    let config = LayeredConfig::with_yaml(file.path()).unwrap();
    let port: u16 = config.get_typed("port").unwrap().unwrap_or(3000);
    assert_eq!(port, 8080);
}

#[test]
fn test_layered_config_env_only() {
    let _guard = env_lock().lock().unwrap();
    env::set_var("TEST_VAR", "test_value");
    let config = LayeredConfig::env_only();
    let value = config.get("TEST_VAR").unwrap();
    assert_eq!(value, Some("test_value".to_string()));
    env::remove_var("TEST_VAR");
}

#[test]
fn test_closed_missing_default_yaml() {
    let _guard = cwd_lock().lock().unwrap();
    let original = env::current_dir().unwrap();
    let temp = tempfile::tempdir().unwrap();

    env::set_current_dir(temp.path()).unwrap();
    let result = LayeredConfig::new();
    env::set_current_dir(original).unwrap();

    match result {
        Err(crate::config::ConfigError::Io(err)) => {
            assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        }
        Ok(_) => panic!("expected not-found error, got success"),
        Err(err) => panic!("expected not-found error, got {err}"),
    }
}

#[test]
fn test_optional_yaml_downgrades_missing() {
    let config = LayeredConfig::with_optional_yaml("/tmp/does-not-exist-z00z-utils.yaml").unwrap();
    let value = config.get("MISSING_KEY").unwrap();
    assert_eq!(value, None);
}

#[cfg(unix)]
#[test]
fn test_config_non_utf8_error() {
    let _guard = env_lock().lock().unwrap();
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let key = "TEST_NON_UTF8_ENV_CONFIG";
    env::set_var(key, OsString::from_vec(vec![0x66, 0x6f, 0x80, 0x6f]));

    let result = EnvConfig.get(key);
    env::remove_var(key);

    match result {
        Err(crate::config::ConfigError::Parse { key, error, .. }) => {
            assert_eq!(key, "TEST_NON_UTF8_ENV_CONFIG");
            assert!(error.contains("valid UTF-8"));
        }
        Ok(_) => panic!("expected non-utf8 env error, got success"),
        Err(err) => panic!("expected parse error, got {err}"),
    }
}

#[test]
fn test_typed_redacts_parse_value() {
    let _guard = env_lock().lock().unwrap();
    env::set_var("TEST_PARSE_SECRET", "secret-token-123");

    let result = EnvConfig.get_typed::<u16>("TEST_PARSE_SECRET");

    env::remove_var("TEST_PARSE_SECRET");

    match result {
        Err(crate::config::ConfigError::Parse { key, value, error }) => {
            assert_eq!(key, "TEST_PARSE_SECRET");
            assert_eq!(value, "<redacted len=16>");
            assert!(error.contains("invalid digit"));
        }
        Ok(_) => panic!("expected parse error, got success"),
        Err(err) => panic!("expected parse error, got {err}"),
    }
}
