use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use jsonschema::validator_for;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use z00z_core::config_paths::{
    assets_schema_path, devnet_assets_path, devnet_genesis_path, devnet_policies_path,
    devnet_rights_path, devnet_vouchers_path, genesis_schema_path, policies_schema_path,
    rights_schema_path, vouchers_schema_path,
};
use z00z_core::genesis::genesis_config::load_genesis_config;

fn load_yaml(path: &Path) -> Result<YamlValue, Box<dyn std::error::Error>> {
    Ok(serde_yaml::from_str(&fs::read_to_string(path)?)?)
}

fn extract_ref(schema: &YamlValue) -> &str {
    schema["allOf"][0]["$ref"]
        .as_str()
        .expect("schema wrapper must expose allOf[0].$ref")
}

fn schema_root_name(schema: &YamlValue) -> &str {
    extract_ref(schema)
        .rsplit('/')
        .next()
        .expect("schema wrapper ref must target a definition name")
}

fn schema_definition_names(
    schema: &YamlValue,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let definitions = schema["definitions"]
        .as_mapping()
        .ok_or("schema definitions must be a mapping")?;

    let mut names = BTreeSet::new();
    for key in definitions.keys() {
        let name = key
            .as_str()
            .ok_or("schema definition keys must be strings")?;
        names.insert(name.to_string());
    }

    Ok(names)
}

fn collect_internal_definition_refs(node: &YamlValue, refs: &mut BTreeSet<String>) {
    match node {
        YamlValue::Mapping(mapping) => {
            for (key, value) in mapping {
                if key.as_str() == Some("$ref") {
                    if let Some(reference) = value.as_str() {
                        if let Some(definition_name) = reference.strip_prefix("#/definitions/") {
                            refs.insert(definition_name.to_string());
                        }
                    }
                    continue;
                }

                collect_internal_definition_refs(value, refs);
            }
        }
        YamlValue::Sequence(sequence) => {
            for value in sequence {
                collect_internal_definition_refs(value, refs);
            }
        }
        _ => {}
    }
}

fn reachable_definition_names(
    schema: &YamlValue,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let definitions = schema["definitions"]
        .as_mapping()
        .ok_or("schema definitions must be a mapping")?;
    let root_name = schema_root_name(schema).to_string();
    let mut reachable = BTreeSet::from([root_name.clone()]);
    let mut queue = vec![root_name];

    while let Some(current) = queue.pop() {
        let definition = definitions
            .get(YamlValue::String(current.clone()))
            .ok_or("reachable definition missing from schema")?;
        let mut refs = BTreeSet::new();
        collect_internal_definition_refs(definition, &mut refs);
        for reference in refs {
            if reachable.insert(reference.clone()) {
                queue.push(reference);
            }
        }
    }

    Ok(reachable)
}

fn normalized_schema_body(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut schema = load_yaml(path)?;
    let mapping = schema.as_mapping_mut().ok_or("schema must be a mapping")?;
    mapping.remove(YamlValue::String("title".to_string()));
    mapping.remove(YamlValue::String("description".to_string()));
    mapping.remove(YamlValue::String("allOf".to_string()));
    Ok(serde_json::to_string(&serde_json::to_value(schema)?)?)
}

fn config_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("configs")
}

fn wrapper_cases() -> [(PathBuf, &'static str, PathBuf); 5] {
    [
        (
            genesis_schema_path(),
            "#/definitions/genesis_manifest",
            devnet_genesis_path(),
        ),
        (
            assets_schema_path(),
            "#/definitions/assets_catalog",
            devnet_assets_path(),
        ),
        (
            rights_schema_path(),
            "#/definitions/rights_catalog",
            devnet_rights_path(),
        ),
        (
            policies_schema_path(),
            "#/definitions/policies_catalog",
            devnet_policies_path(),
        ),
        (
            vouchers_schema_path(),
            "#/definitions/vouchers_catalog",
            devnet_vouchers_path(),
        ),
    ]
}

fn live_config_paths() -> [PathBuf; 5] {
    [
        devnet_genesis_path(),
        devnet_assets_path(),
        devnet_rights_path(),
        devnet_policies_path(),
        devnet_vouchers_path(),
    ]
}

fn load_json(path: &Path) -> Result<JsonValue, Box<dyn std::error::Error>> {
    Ok(serde_json::to_value(load_yaml(path)?)?)
}

#[test]
fn test_all_schema_files_parse() -> Result<(), Box<dyn std::error::Error>> {
    let dir = config_dir();
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("yaml")
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("schema_") && name.ends_with("_config.yaml"))
        {
            let _ = load_yaml(&path)?;
        }
    }

    Ok(())
}

#[test]
fn test_specific_schema_wrappers_cover_all_live_yaml_kinds(
) -> Result<(), Box<dyn std::error::Error>> {
    for (schema_path, expected_ref, sample_path) in wrapper_cases() {
        let schema = load_yaml(&schema_path)?;
        assert_eq!(extract_ref(&schema), expected_ref);
        assert!(
            sample_path.is_file(),
            "missing sample YAML: {}",
            sample_path.display()
        );

        let fragment = expected_ref
            .split("/definitions/")
            .nth(1)
            .expect("wrapper ref must target a definition");
        let definitions = schema["definitions"]
            .as_mapping()
            .expect("standalone schema definitions must be a mapping");
        assert!(
            definitions.contains_key(YamlValue::String(fragment.to_string())),
            "standalone schema missing definition {fragment}"
        );
    }

    Ok(())
}

#[test]
fn test_all_live_yaml_files_parse() -> Result<(), Box<dyn std::error::Error>> {
    for path in live_config_paths() {
        let _ = load_yaml(&path)?;
    }

    Ok(())
}

#[test]
fn test_live_yaml_samples_validate_against_schema_wrappers(
) -> Result<(), Box<dyn std::error::Error>> {
    for (schema_path, _, sample_path) in wrapper_cases() {
        let schema_json = load_json(&schema_path)?;
        let validator = validator_for(&schema_json)?;
        let sample_json = load_json(&sample_path)?;
        let errors: Vec<String> = validator
            .iter_errors(&sample_json)
            .map(|err| err.to_string())
            .collect();
        assert!(
            errors.is_empty(),
            "{} must validate against {}: {}",
            sample_path.display(),
            schema_path.display(),
            errors.join(" | ")
        );
    }

    Ok(())
}

#[test]
fn test_live_yaml_schema_mapping_is_unambiguous() -> Result<(), Box<dyn std::error::Error>> {
    let compiled: Vec<(PathBuf, PathBuf, jsonschema::Validator)> = wrapper_cases()
        .into_iter()
        .map(|(schema_path, _, sample_path)| {
            let schema_json = load_json(&schema_path)?;
            let validator = validator_for(&schema_json)?;
            Ok::<_, Box<dyn std::error::Error>>((schema_path, sample_path, validator))
        })
        .collect::<Result<_, _>>()?;

    for (expected_schema, sample_path, _) in &compiled {
        let sample_json = load_json(sample_path)?;
        let matching_schemas: Vec<String> = compiled
            .iter()
            .filter_map(|(schema_path, _, validator)| {
                let has_errors = validator.iter_errors(&sample_json).next().is_some();
                (!has_errors).then(|| schema_path.display().to_string())
            })
            .collect();

        assert_eq!(
            matching_schemas,
            vec![expected_schema.display().to_string()],
            "{} must validate against exactly one schema",
            sample_path.display()
        );
    }

    Ok(())
}

#[test]
fn test_schema_files_contain_only_their_canonical_root_definition(
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_root_names = [
        "genesis_manifest",
        "assets_catalog",
        "rights_catalog",
        "policies_catalog",
        "vouchers_catalog",
    ];

    for (schema_path, expected_ref, _) in wrapper_cases() {
        let schema = load_yaml(&schema_path)?;
        let definition_names = schema_definition_names(&schema)?;
        let expected_root_name = expected_ref
            .rsplit('/')
            .next()
            .ok_or("expected wrapper ref must end with definition name")?;

        let embedded_other_roots: Vec<&str> = canonical_root_names
            .into_iter()
            .filter(|root_name| *root_name != expected_root_name)
            .filter(|root_name| definition_names.contains(*root_name))
            .collect();

        assert!(
            embedded_other_roots.is_empty(),
            "{} must not embed sibling root definitions: {:?}",
            schema_path.display(),
            embedded_other_roots
        );
    }

    Ok(())
}

#[test]
fn test_schema_files_contain_only_reachable_definitions() -> Result<(), Box<dyn std::error::Error>>
{
    for (schema_path, _, _) in wrapper_cases() {
        let schema = load_yaml(&schema_path)?;
        let actual_definition_names = schema_definition_names(&schema)?;
        let reachable_definition_names = reachable_definition_names(&schema)?;

        assert_eq!(
            actual_definition_names,
            reachable_definition_names,
            "{} must keep only transitive definitions reachable from {}",
            schema_path.display(),
            schema_root_name(&schema)
        );
    }

    Ok(())
}

#[test]
fn test_schema_files_are_not_wrapper_only_clones() -> Result<(), Box<dyn std::error::Error>> {
    let mut normalized_bodies = std::collections::BTreeMap::new();

    for (schema_path, _, _) in wrapper_cases() {
        let body = normalized_schema_body(&schema_path)?;
        if let Some(existing_path) =
            normalized_bodies.insert(body, schema_path.display().to_string())
        {
            panic!(
                "{} and {} normalize to the same schema body after removing wrapper metadata",
                existing_path,
                schema_path.display()
            );
        }
    }

    Ok(())
}

#[test]
fn test_live_genesis_manifest_uses_canonical_chain_name_field(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = load_yaml(&devnet_genesis_path())?;
    let chain = manifest["chain"]
        .as_mapping()
        .ok_or("devnet genesis manifest must define chain mapping")?;

    assert!(
        chain.contains_key(YamlValue::String("name".to_string())),
        "canonical manifest must use chain.name",
    );
    assert!(
        !chain.contains_key(YamlValue::String("chain".to_string())),
        "canonical manifest must not use deprecated chain.chain",
    );

    Ok(())
}

#[test]
fn test_live_genesis_profiles_match_live_config_references(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_genesis_config(devnet_genesis_path().to_str().ok_or("utf8 path")?)?;
    let asset_ids: BTreeSet<&str> = config
        .assets
        .iter()
        .map(|asset| asset.id.as_str())
        .collect();
    let right_policy_ids: BTreeSet<&str> = config
        .rights
        .iter()
        .flat_map(|right| {
            [
                right.revocation_policy_id.as_str(),
                right.transition_policy_id.as_str(),
                right.challenge_policy_id.as_str(),
                right.disclosure_policy_id.as_str(),
                right.retention_policy_id.as_str(),
            ]
        })
        .collect();
    let policy_profile_ids: BTreeSet<&str> = config
        .policy_profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect();

    for profile in &config.wallet_profiles {
        if let Some(asset_id) = profile.locked_asset_id.as_deref() {
            assert!(
                asset_ids.contains(asset_id),
                "wallet profile {} must reference a live asset id",
                profile.id
            );
        }

        for reference in [
            profile.disclosure_policy.as_deref(),
            profile.retention_policy.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            assert!(
                right_policy_ids.contains(reference),
                "wallet profile {} must reference a live rights policy id",
                profile.id
            );
        }
    }

    for profile in &config.policy_profiles {
        if let Some(retention_profile) = profile.retention_profile.as_deref() {
            assert!(
                policy_profile_ids.contains(retention_profile),
                "policy profile {} must reference a live retention profile",
                profile.id
            );
        }

        for applies_to_profile in &profile.applies_to_profiles {
            assert!(
                policy_profile_ids.contains(applies_to_profile.as_str()),
                "policy profile {} must reference a live applies_to profile",
                profile.id
            );
        }
    }

    Ok(())
}

#[test]
fn test_genesis_schema_rejects_deprecated_chain_key() -> Result<(), Box<dyn std::error::Error>> {
    let schema_json = load_json(&genesis_schema_path())?;
    let validator = validator_for(&schema_json)?;
    let mut manifest = load_yaml(&devnet_genesis_path())?;
    let root = manifest
        .as_mapping_mut()
        .ok_or("devnet genesis manifest must be a mapping")?;
    let chain_section_key = YamlValue::String("chain".to_string());
    let chain_name_key = YamlValue::String("name".to_string());
    let deprecated_chain_key = YamlValue::String("chain".to_string());
    let chain = root
        .get_mut(&chain_section_key)
        .and_then(YamlValue::as_mapping_mut)
        .ok_or("devnet genesis manifest must define chain mapping")?;
    let name_value = chain
        .remove(&chain_name_key)
        .ok_or("devnet genesis manifest must define chain.name")?;
    chain.insert(deprecated_chain_key, name_value);

    let sample_json = serde_json::to_value(manifest)?;
    let errors: Vec<String> = validator
        .iter_errors(&sample_json)
        .map(|err| err.to_string())
        .collect();
    assert!(
        !errors.is_empty(),
        "genesis schema must reject deprecated chain.chain key"
    );

    Ok(())
}

#[test]
fn test_no_schema_uses_external_ref() -> Result<(), Box<dyn std::error::Error>> {
    let dir = config_dir();
    let mut schema_files = BTreeSet::new();

    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("schema_") && name.ends_with("_config.yaml"))
        {
            schema_files.insert(path);
        }
    }

    for path in schema_files {
        let schema = load_yaml(&path)?;
        let ref_value = extract_ref(&schema);
        assert!(
            ref_value.starts_with("#/definitions/"),
            "{} must keep refs internal",
            path.display()
        );
    }

    Ok(())
}
