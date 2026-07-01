use std::path::PathBuf;

use tempfile::TempDir;
use z00z_core::config_paths::{
    core_config_dir, devnet_genesis_path, DEVNET_ASSETS_CONFIG, DEVNET_POLICIES_CONFIG,
    DEVNET_RIGHTS_CONFIG, DEVNET_VOUCHERS_CONFIG,
};
use z00z_core::genesis::genesis_config::load_genesis_config;
use z00z_utils::config::YamlValue;
use z00z_utils::io::{load_yaml, read_to_string, write_file};
use z00z_utils::prelude::{Codec, YamlCodec};

const ROOT_CORE_BLOCK: &str = r#"
chain:
  id: 3
  type: devnet
  name: "z00z-devnet-1"
  magic_bytes: [0x5A, 0x30, 0x30, 0x44]
  domains:
    genesis_seed: [0x14, 0x92, 0x51, 0x77, 0x44, 0xa1, 0x09, 0x33, 0x71, 0xf8, 0x42, 0x10, 0x8e, 0x6d, 0x5b, 0x9a, 0x20, 0x3f, 0x18, 0x61, 0xd4, 0x72, 0xc1, 0x99, 0x0e, 0x2d, 0x7c, 0x4a, 0xb5, 0x88, 0x13, 0xef]

outputs:
  assets_export_path: "crates/z00z_core/outputs/genesis/"
  snapshot_export_path: "crates/z00z_core/outputs/genesis/"
  logging_path: "crates/z00z_core/outputs/log/"

performance:
  num_threads: auto
"#;

const SECTION_FILES: &[(&str, &str)] = &[
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

fn copy_canonical_section_files(temp: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    for (_, file_name) in SECTION_FILES {
        let content = read_to_string(canonical_genesis_dir().join(file_name))?;
        write_file(temp.path().join(file_name), content.as_bytes())?;
    }

    Ok(())
}

fn build_split_inline_fixture() -> Result<String, Box<dyn std::error::Error>> {
    let mut root: serde_yaml::Mapping = load_yaml::<YamlValue>(canonical_root_manifest_path())?
        .as_mapping()
        .cloned()
        .ok_or("canonical root manifest must be a mapping")?;
    let refs = root
        .remove(YamlValue::String("manifest_refs".to_string()))
        .ok_or("canonical root manifest must define manifest_refs")?;
    let refs = refs
        .as_mapping()
        .ok_or("manifest_refs must be a mapping in canonical root manifest")?;

    for (section_key, _) in SECTION_FILES {
        let key = YamlValue::String((*section_key).to_string());
        let ref_path = refs
            .get(&key)
            .and_then(YamlValue::as_str)
            .ok_or("canonical root manifest must define every section ref")?;
        let section_yaml: YamlValue = load_yaml(canonical_genesis_dir().join(ref_path))?;
        let section_value = section_yaml
            .get(*section_key)
            .cloned()
            .ok_or("section file must contain the expected top-level key")?;
        root.insert(key, section_value);
    }

    Ok(String::from_utf8(
        YamlCodec.serialize(&YamlValue::Mapping(root))?,
    )?)
}

#[test]
fn test_live_root_manifest_loads() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_genesis_config(canonical_root_manifest_path().to_str().ok_or("utf8 path")?)?;

    assert_eq!(config.chain.id, 3);
    assert_eq!(config.chain.chain_type, "devnet");
    assert_eq!(config.assets.len(), 4);
    assert_eq!(config.rights.len(), 5);
    assert_eq!(config.policies.len(), 4);
    assert_eq!(config.vouchers.len(), 3);
    assert_eq!(config.wallet_profiles.len(), 6);
    assert_eq!(config.policy_profiles.len(), 3);
    assert_eq!(config.assets[0].id, "z00z");
    assert_eq!(
        config.assets[0].description.as_deref(),
        Some("Native confidential coin"),
    );
    assert_eq!(config.wallet_profiles[0].id, "fee_credit_v1");
    assert_eq!(config.policy_profiles[0].id, "corporate_eu_transfer_v1");

    Ok(())
}

#[test]
fn test_keeps_genesis_shape() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let inline_path = temp.path().join("inline.yaml");
    write_file(&inline_path, build_split_inline_fixture()?.as_bytes())?;

    let from_root =
        load_genesis_config(canonical_root_manifest_path().to_str().ok_or("utf8 path")?)?;
    let from_inline = load_genesis_config(inline_path.to_str().ok_or("utf8 path")?)?;

    assert_eq!(
        serde_json::to_value(&from_root)?,
        serde_json::to_value(&from_inline)?,
    );

    Ok(())
}

#[test]
fn test_reject_duplicate_ref_sources() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    copy_canonical_section_files(&temp)?;

    let root = format!(
        "version: 1\n\nassets:\n  - id: duplicate-inline\nmanifest_refs:\n  assets: \"{DEVNET_ASSETS_CONFIG}\"\n  rights: \"{DEVNET_RIGHTS_CONFIG}\"\n  policies: \"{DEVNET_POLICIES_CONFIG}\"\n  vouchers: \"{DEVNET_VOUCHERS_CONFIG}\"\n{ROOT_CORE_BLOCK}"
    );
    let root_path = temp.path().join("root.yaml");
    write_file(&root_path, root.as_bytes())?;

    let err = load_genesis_config(root_path.to_str().ok_or("utf8 path")?).unwrap_err();
    assert!(
        err.to_string()
            .contains("assets section defined both inline and via manifest_refs"),
        "unexpected error: {err}",
    );

    Ok(())
}

#[test]
fn test_reject_reused_ref_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    copy_canonical_section_files(&temp)?;

    let root = format!(
        "version: 1\n\nmanifest_refs:\n  assets: \"{DEVNET_ASSETS_CONFIG}\"\n  rights: \"{DEVNET_ASSETS_CONFIG}\"\n  policies: \"{DEVNET_POLICIES_CONFIG}\"\n  vouchers: \"{DEVNET_VOUCHERS_CONFIG}\"\n{ROOT_CORE_BLOCK}"
    );
    let root_path = temp.path().join("root.yaml");
    write_file(&root_path, root.as_bytes())?;

    let err = load_genesis_config(root_path.to_str().ok_or("utf8 path")?).unwrap_err();
    assert!(
        err.to_string().contains("ref reuses"),
        "unexpected error: {err}",
    );

    Ok(())
}

#[test]
fn test_reject_parent_traversal() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    copy_canonical_section_files(&temp)?;

    let root = format!(
        "version: 1\n\nmanifest_refs:\n  assets: \"../escape.yaml\"\n  rights: \"{DEVNET_RIGHTS_CONFIG}\"\n  policies: \"{DEVNET_POLICIES_CONFIG}\"\n  vouchers: \"{DEVNET_VOUCHERS_CONFIG}\"\n{ROOT_CORE_BLOCK}"
    );
    let root_path = temp.path().join("root.yaml");
    write_file(&root_path, root.as_bytes())?;

    let err = load_genesis_config(root_path.to_str().ok_or("utf8 path")?).unwrap_err();
    assert!(
        err.to_string()
            .contains("must not escape the manifest root"),
        "unexpected error: {err}",
    );

    Ok(())
}

#[test]
fn test_reject_actions_config() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    copy_canonical_section_files(&temp)?;

    let root = format!(
        "version: 1\n\nmanifest_refs:\n  assets: \"{DEVNET_ASSETS_CONFIG}\"\n  rights: \"{DEVNET_RIGHTS_CONFIG}\"\n  policies: \"{DEVNET_POLICIES_CONFIG}\"\n  vouchers: \"{DEVNET_VOUCHERS_CONFIG}\"\n  actions_config: \"actions_config.yaml\"\n{ROOT_CORE_BLOCK}"
    );
    let root_path = temp.path().join("root.yaml");
    write_file(&root_path, root.as_bytes())?;

    let err = load_genesis_config(root_path.to_str().ok_or("utf8 path")?).unwrap_err();
    assert!(
        err.to_string()
            .contains("actions_config.yaml is intentionally unsupported"),
        "unexpected error: {err}",
    );

    Ok(())
}

#[test]
fn test_reject_bad_section_shape() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    copy_canonical_section_files(&temp)?;
    write_file(
        temp.path().join("bad_assets.yaml"),
        b"wrong:\n  - id: drift\n",
    )?;

    let root = format!(
        "version: 1\n\nmanifest_refs:\n  assets: \"bad_assets.yaml\"\n  rights: \"{DEVNET_RIGHTS_CONFIG}\"\n  policies: \"{DEVNET_POLICIES_CONFIG}\"\n  vouchers: \"{DEVNET_VOUCHERS_CONFIG}\"\n{ROOT_CORE_BLOCK}"
    );
    let root_path = temp.path().join("root.yaml");
    write_file(&root_path, root.as_bytes())?;

    let err = load_genesis_config(root_path.to_str().ok_or("utf8 path")?).unwrap_err();
    assert!(
        err.to_string()
            .contains("must contain exactly one top-level key named assets"),
        "unexpected error: {err}",
    );

    Ok(())
}

#[test]
fn test_reject_deprecated_network_alias() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    copy_canonical_section_files(&temp)?;

    let root = format!(
        "version: 1\n\nmanifest_refs:\n  assets: \"{DEVNET_ASSETS_CONFIG}\"\n  rights: \"{DEVNET_RIGHTS_CONFIG}\"\n  policies: \"{DEVNET_POLICIES_CONFIG}\"\n  vouchers: \"{DEVNET_VOUCHERS_CONFIG}\"\n{}",
        ROOT_CORE_BLOCK.replacen("chain:", "network:", 1)
    );
    let root_path = temp.path().join("root.yaml");
    write_file(&root_path, root.as_bytes())?;

    let err = load_genesis_config(root_path.to_str().ok_or("utf8 path")?).unwrap_err();
    assert!(
        err.to_string().contains("deprecated top-level key network"),
        "unexpected error: {err}",
    );

    Ok(())
}

#[test]
fn test_reject_deprecated_refs_alias() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    copy_canonical_section_files(&temp)?;

    let root = format!(
        "version: 1\n\nrefs:\n  assets: \"{DEVNET_ASSETS_CONFIG}\"\n  rights: \"{DEVNET_RIGHTS_CONFIG}\"\n  policies: \"{DEVNET_POLICIES_CONFIG}\"\n  vouchers: \"{DEVNET_VOUCHERS_CONFIG}\"\n{ROOT_CORE_BLOCK}"
    );
    let root_path = temp.path().join("root.yaml");
    write_file(&root_path, root.as_bytes())?;

    let err = load_genesis_config(root_path.to_str().ok_or("utf8 path")?).unwrap_err();
    assert!(
        err.to_string().contains("deprecated top-level key refs"),
        "unexpected error: {err}",
    );

    Ok(())
}
