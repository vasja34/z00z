use std::path::PathBuf;

use z00z_core::config_paths::{
    core_config_dir, devnet_genesis_path, DEVNET_ASSETS_CONFIG, DEVNET_POLICIES_CONFIG,
    DEVNET_RIGHTS_CONFIG, DEVNET_VOUCHERS_CONFIG,
};
use z00z_core::genesis::genesis_config::load_genesis_config;
use z00z_utils::config::YamlValue;
use z00z_utils::io::load_yaml;

const EXPECTED_REF_LAYOUT: &[(&str, &str)] = &[
    ("assets", DEVNET_ASSETS_CONFIG),
    ("rights", DEVNET_RIGHTS_CONFIG),
    ("policies", DEVNET_POLICIES_CONFIG),
    ("vouchers", DEVNET_VOUCHERS_CONFIG),
];

fn canonical_genesis_dir() -> PathBuf {
    core_config_dir()
}

fn canonical_root_manifest_path() -> PathBuf {
    devnet_genesis_path()
}

#[test]
fn test_live_canonical_root_manifest() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_genesis_config(canonical_root_manifest_path().to_str().ok_or("utf8 path")?)?;

    assert_eq!(config.chain.name, "z00z-devnet-1");
    assert_eq!(config.assets.len(), 4);
    assert_eq!(config.rights.len(), 5);
    assert_eq!(config.policies.len(), 4);
    assert_eq!(config.vouchers.len(), 3);
    assert_eq!(
        config.performance.num_threads,
        z00z_core::genesis::genesis_config::ThreadCountConfig::Named(
            z00z_core::genesis::genesis_config::ThreadCountMode::Auto,
        ),
    );

    Ok(())
}

#[test]
fn test_root_layout() -> Result<(), Box<dyn std::error::Error>> {
    let root: YamlValue = load_yaml(canonical_root_manifest_path())?;
    let refs = root
        .get("manifest_refs")
        .and_then(YamlValue::as_mapping)
        .ok_or("root manifest must expose manifest_refs")?;

    assert!(root.get("actions_config").is_none());
    assert!(refs
        .get(YamlValue::String("actions_config".to_string()))
        .is_none());

    for (section, file_name) in EXPECTED_REF_LAYOUT {
        assert_eq!(
            refs.get(YamlValue::String((*section).to_string()))
                .and_then(YamlValue::as_str),
            Some(*file_name),
            "manifest_refs.{section} drifted",
        );
    }

    Ok(())
}

#[test]
fn test_section_files_are_atomic() -> Result<(), Box<dyn std::error::Error>> {
    for (section, file_name) in EXPECTED_REF_LAYOUT {
        let yaml: YamlValue = load_yaml(canonical_genesis_dir().join(file_name))?;
        let mapping = yaml
            .as_mapping()
            .ok_or("section file must stay a mapping")?;
        assert_eq!(
            mapping.len(),
            1,
            "section file {file_name} must keep one canonical top-level key",
        );
        assert!(
            yaml.get(*section).is_some(),
            "section file {file_name} must contain top-level key {section}",
        );
    }

    Ok(())
}

#[test]
fn test_legacy_small_fixture_is_removed() {
    let legacy_name = ["genesis", "config", "devnet", "small.yaml"].join("_");
    let legacy_small =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("src/genesis/{legacy_name}"));
    assert!(
        !legacy_small.exists(),
        "legacy small-devnet root must not survive the configs migration",
    );
    assert!(
        canonical_root_manifest_path().exists(),
        "canonical configs root manifest must exist",
    );
}
