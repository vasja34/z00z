use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

use crate::genesis::genesis_config::{
    build_config, parse_assets_from_yaml, parse_core_sections, GenesisConfig, PolicyProfileConfig,
    WalletProfileConfig, MAX_CONFIG_FILE_SIZE,
};
use crate::genesis::validator::{
    validate_manifest_ref_keys, validate_manifest_ref_section_key,
    validate_manifest_top_level_keys, GenesisError,
};
use crate::policies::PolicyConfigEntryV1;
use crate::rights::parse_rights_from_yaml;
use crate::vouchers::VoucherBootstrapEntryV1;
use z00z_utils::{
    config::{from_yaml_value, YamlValue},
    io::load_yaml_bounded,
};

#[derive(Clone, Debug, serde::Deserialize)]
struct ManifestRoot {
    #[serde(default)]
    assets: Option<YamlValue>,
    #[serde(default)]
    rights: Option<YamlValue>,
    #[serde(default)]
    policies: Option<YamlValue>,
    #[serde(default)]
    vouchers: Option<YamlValue>,
    #[serde(default)]
    manifest_refs: Option<ManifestRefs>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestRefs {
    #[serde(default)]
    assets: Option<String>,
    #[serde(default)]
    rights: Option<String>,
    #[serde(default)]
    policies: Option<String>,
    #[serde(default)]
    vouchers: Option<String>,
    #[serde(default)]
    actions_config: Option<String>,
}

pub(crate) fn load_ref_config(path: &Path) -> Result<Option<GenesisConfig>, GenesisError> {
    let root: YamlValue = load_yaml_bounded(path, MAX_CONFIG_FILE_SIZE)
        .map_err(|err| GenesisError::ConfigLoadFailed(err.to_string()))?;
    validate_top_level_yaml_keys(&root, path)?;
    let doc: ManifestRoot = from_yaml_value(root.clone())
        .map_err(|err| GenesisError::ConfigParseFailed(err.to_string()))?;
    let Some(refs) = doc.manifest_refs else {
        return Ok(None);
    };

    if refs.actions_config.is_some() {
        validate_manifest_ref_section_key("actions_config")?;
    }

    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut seen = BTreeMap::<PathBuf, String>::new();
    let assets_yaml = section_yaml(
        &root,
        base,
        "assets",
        doc.assets.as_ref(),
        refs.assets.as_deref(),
        &mut seen,
    )?;
    let rights_yaml = section_yaml(
        &root,
        base,
        "rights",
        doc.rights.as_ref(),
        refs.rights.as_deref(),
        &mut seen,
    )?;
    let policies_yaml = section_yaml(
        &root,
        base,
        "policies",
        doc.policies.as_ref(),
        refs.policies.as_deref(),
        &mut seen,
    )?;
    let vouchers_yaml = section_yaml(
        &root,
        base,
        "vouchers",
        doc.vouchers.as_ref(),
        refs.vouchers.as_deref(),
        &mut seen,
    )?;

    let (chain, outputs, performance) = parse_core_sections(&root)?;
    let assets = parse_assets_from_yaml(&assets_yaml)?;
    let rights = parse_rights_from_yaml(&rights_yaml)
        .map_err(|err| GenesisError::ConfigParseFailed(err.to_string()))?;
    let policies = parse_vec::<PolicyConfigEntryV1>(&policies_yaml, "policies")?;
    let vouchers = parse_vec::<VoucherBootstrapEntryV1>(&vouchers_yaml, "vouchers")?;
    let wallet_profiles = parse_vec::<WalletProfileConfig>(&root, "wallet_profiles")?;
    let policy_profiles = parse_vec::<PolicyProfileConfig>(&root, "policy_profiles")?;

    Ok(Some(build_config(
        chain,
        assets,
        rights,
        policies,
        vouchers,
        wallet_profiles,
        policy_profiles,
        outputs,
        performance,
    )?))
}

fn section_yaml(
    root: &YamlValue,
    base: &Path,
    key: &str,
    inline: Option<&YamlValue>,
    rel: Option<&str>,
    seen: &mut BTreeMap<PathBuf, String>,
) -> Result<YamlValue, GenesisError> {
    match (inline, rel) {
        (Some(_), Some(_)) => Err(GenesisError::ConfigParseFailed(format!(
            "{key} section defined both inline and via manifest_refs"
        ))),
        (Some(_), None) | (None, None) => Ok(root.clone()),
        (None, Some(rel)) => load_ref_yaml(base, key, rel, seen),
    }
}

fn load_ref_yaml(
    base: &Path,
    key: &str,
    rel: &str,
    seen: &mut BTreeMap<PathBuf, String>,
) -> Result<YamlValue, GenesisError> {
    let full = resolve_ref(base, key, rel)?;
    if let Some(prev) = seen.insert(full.clone(), key.to_string()) {
        return Err(GenesisError::ConfigParseFailed(format!(
            "{key} ref reuses {} path already owned by {prev}",
            full.display()
        )));
    }

    let yaml: YamlValue = load_yaml_bounded(&full, MAX_CONFIG_FILE_SIZE)
        .map_err(|err| GenesisError::ConfigLoadFailed(err.to_string()))?;
    let Some(_mapping) = yaml.as_mapping() else {
        return Err(GenesisError::ConfigParseFailed(format!(
            "{key} ref file must be a mapping: {}",
            full.display()
        )));
    };
    let top_level_keys = collect_string_keys(&yaml, &full)?;
    validate_manifest_top_level_keys(&top_level_keys, &full)?;
    validate_manifest_ref_keys(key, &top_level_keys, &full)?;
    Ok(yaml)
}

fn resolve_ref(base: &Path, key: &str, rel: &str) -> Result<PathBuf, GenesisError> {
    if rel.trim().is_empty() {
        return Err(GenesisError::ConfigParseFailed(format!(
            "{key} ref path must not be empty"
        )));
    }

    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        return Err(GenesisError::ConfigParseFailed(format!(
            "{key} ref path must stay relative: {rel}"
        )));
    }

    let mut out = PathBuf::new();
    for part in rel_path.components() {
        match part {
            Component::CurDir => {}
            Component::Normal(seg) => out.push(seg),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(GenesisError::ConfigParseFailed(format!(
                    "{key} ref path must not escape the manifest root: {rel}"
                )));
            }
        }
    }

    Ok(base.join(out))
}

fn parse_vec<T>(yaml: &YamlValue, key: &str) -> Result<Vec<T>, GenesisError>
where
    T: serde::de::DeserializeOwned,
{
    yaml.get(key)
        .map(|value| {
            from_yaml_value(value.clone())
                .map_err(|err| GenesisError::ConfigParseFailed(err.to_string()))
        })
        .transpose()
        .map(|value| value.unwrap_or_default())
}

fn validate_top_level_yaml_keys(yaml: &YamlValue, source: &Path) -> Result<(), GenesisError> {
    let Some(_mapping) = yaml.as_mapping() else {
        return Err(GenesisError::ConfigParseFailed(format!(
            "genesis config {} must be a top-level mapping",
            source.display()
        )));
    };
    validate_manifest_top_level_keys(&collect_string_keys(yaml, source)?, source)
}

fn collect_string_keys(yaml: &YamlValue, source: &Path) -> Result<Vec<String>, GenesisError> {
    let Some(mapping) = yaml.as_mapping() else {
        return Err(GenesisError::ConfigParseFailed(format!(
            "genesis config {} must be a top-level mapping",
            source.display()
        )));
    };
    mapping
        .keys()
        .map(|key| {
            key.as_str().map(str::to_string).ok_or_else(|| {
                GenesisError::ConfigParseFailed(format!(
                    "genesis config {} has a non-string top-level key",
                    source.display()
                ))
            })
        })
        .collect()
}
