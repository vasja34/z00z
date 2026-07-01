use std::path::{Path, PathBuf};
use std::sync::Arc;

use tempfile::TempDir;
use z00z_core::assets::AssetDefinitionRegistry;
use z00z_core::config_paths::{devnet_assets_path, devnet_rights_path};
use z00z_core::rights::load_rights_from_yaml;
use z00z_utils::config::YamlValue;
use z00z_utils::io::{load_yaml, read_to_string, write_file};
use z00z_utils::prelude::{Codec, NoopLogger, NoopMetrics, SystemTimeProvider, YamlCodec};

#[derive(Debug, serde::Deserialize)]
struct AssetsFixture {
    assets: Vec<YamlValue>,
}

fn assets_fixture_path() -> PathBuf {
    devnet_assets_path()
}

fn rights_fixture_path() -> PathBuf {
    devnet_rights_path()
}

fn load_assets_fixture() -> Result<AssetsFixture, Box<dyn std::error::Error>> {
    let path = assets_fixture_path();
    Ok(load_yaml(&path)?)
}

fn load_registry(path: &Path) -> Result<AssetDefinitionRegistry, Box<dyn std::error::Error>> {
    Ok(AssetDefinitionRegistry::load_catalog_from_yaml(
        path,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )?)
}

fn inject_forbidden_right_key(
    yaml: &str,
    key: &str,
    value: YamlValue,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut root: YamlValue = YamlCodec.deserialize(yaml.as_bytes())?;
    let rights = root
        .as_mapping_mut()
        .and_then(|mapping| mapping.get_mut(YamlValue::String("rights".to_string())))
        .and_then(YamlValue::as_sequence_mut)
        .ok_or("canonical rights fixture must contain a rights array")?;
    let first = rights
        .first_mut()
        .and_then(YamlValue::as_mapping_mut)
        .ok_or("canonical rights fixture must contain at least one right mapping")?;
    first.insert(YamlValue::String(key.to_string()), value);
    Ok(String::from_utf8(YamlCodec.serialize(&root)?)?)
}

#[test]
fn test_rights_config_loads() -> Result<(), Box<dyn std::error::Error>> {
    let assets = load_assets_fixture()?;
    let rights = load_rights_from_yaml(&rights_fixture_path())?;
    let registry = load_registry(&assets_fixture_path())?;

    assert!(
        !assets.assets.is_empty(),
        "canonical assets fixture must keep assets"
    );
    assert!(
        !rights.is_empty(),
        "canonical rights fixture must keep rights"
    );
    assert_eq!(registry.len()?, assets.assets.len());

    for right in &rights {
        right.validate()?;
        assert!(
            right
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("purpose"))
                .is_some(),
            "every canonical-fixture right must keep metadata.purpose",
        );
    }

    Ok(())
}

#[test]
fn test_rights_rejects_fee() -> Result<(), Box<dyn std::error::Error>> {
    let canonical = read_to_string(rights_fixture_path())?;
    let injected =
        inject_forbidden_right_key(&canonical, "budget_units", YamlValue::Number(5u64.into()))?;
    let temp = TempDir::new()?;
    let path = temp.path().join("assets_config_bad_fee.yaml");
    write_file(&path, injected.as_bytes())?;

    let err = match load_rights_from_yaml(&path) {
        Ok(_) => return Err("fee fields must be rejected in rights config".into()),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("rights.budget_units is forbidden"),
        "unexpected error: {err}",
    );

    Ok(())
}

#[test]
fn test_reject_value_like_keys() -> Result<(), Box<dyn std::error::Error>> {
    let canonical = read_to_string(rights_fixture_path())?;

    for key in [
        "support", "reserve", "amount", "nominal", "backing", "value", "payer", "sponsor", "fee",
    ] {
        let injected = inject_forbidden_right_key(
            &canonical,
            key,
            YamlValue::String("forbidden".to_string()),
        )?;
        let temp = TempDir::new()?;
        let path = temp
            .path()
            .join(format!("assets_config_forbidden_{key}.yaml"));
        write_file(&path, injected.as_bytes())?;

        let err = match load_rights_from_yaml(&path) {
            Ok(_) => return Err(format!("{key} must be rejected in rights config").into()),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains(&format!("rights.{key} is forbidden")),
            "unexpected error for {key}: {err}",
        );
    }

    Ok(())
}
