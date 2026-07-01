use super::*;
use crate::config_name::validate_domain_name;

pub const GENESIS_MANIFEST_REF_SECTIONS: &[&str] = &["assets", "rights", "policies", "vouchers"];
const GENESIS_MANIFEST_UNSUPPORTED_ACTION_KEYS: &[&str] = &["actions", "actions_config"];
const GENESIS_MANIFEST_DEPRECATED_KEYS: &[(&str, &str)] =
    &[("network", "chain"), ("refs", "manifest_refs")];

pub fn validate_manifest_top_level_keys(
    keys: &[String],
    source: &std::path::Path,
) -> Result<(), GenesisError> {
    if let Some((deprecated, canonical)) = GENESIS_MANIFEST_DEPRECATED_KEYS
        .iter()
        .find(|(deprecated, _)| keys.iter().any(|key| key == deprecated))
    {
        return Err(GenesisError::ConfigParseFailed(format!(
            "{} uses deprecated top-level key {}; use {}",
            source.display(),
            deprecated,
            canonical,
        )));
    }

    if let Some(key) = keys
        .iter()
        .find(|key| GENESIS_MANIFEST_UNSUPPORTED_ACTION_KEYS.contains(&key.as_str()))
    {
        return Err(GenesisError::ConfigParseFailed(format!(
            "{} uses unsupported top-level key {}; actions stay nested under policies and actions_config.yaml is intentionally absent",
            source.display(),
            key,
        )));
    }

    Ok(())
}

pub fn validate_manifest_ref_section_key(section_key: &str) -> Result<(), GenesisError> {
    if GENESIS_MANIFEST_UNSUPPORTED_ACTION_KEYS.contains(&section_key) {
        return Err(GenesisError::ConfigParseFailed(
            "actions_config.yaml is intentionally unsupported; actions stay nested under policies"
                .to_string(),
        ));
    }

    if !GENESIS_MANIFEST_REF_SECTIONS.contains(&section_key) {
        return Err(GenesisError::ConfigParseFailed(format!(
            "unsupported manifest_refs section {}; expected one of {:?}",
            section_key, GENESIS_MANIFEST_REF_SECTIONS
        )));
    }

    Ok(())
}

pub fn validate_manifest_ref_keys(
    expected_section: &str,
    keys: &[String],
    source: &std::path::Path,
) -> Result<(), GenesisError> {
    if keys.len() != 1 || keys.first().map(String::as_str) != Some(expected_section) {
        return Err(GenesisError::ConfigParseFailed(format!(
            "{} must contain exactly one top-level key named {}; found {:?}",
            source.display(),
            expected_section,
            keys,
        )));
    }

    Ok(())
}

/// Validate genesis configuration schema.
pub fn validate_config_schema(config: &GenesisConfig) -> Result<(), GenesisError> {
    validate_genesis_config_for(config, &GenesisGenerationPlan::full_bootstrap())
}

fn validate_common_config(
    config: &GenesisConfig,
    plan: &GenesisGenerationPlan,
) -> Result<(), GenesisError> {
    let seed = &config.chain.domains.genesis_seed;

    if seed == &[0u8; 32] {
        return Err(GenesisError::ConfigParseFailed(
            "genesis_seed cannot be all zeros (weak entropy)".to_string(),
        ));
    }

    let is_sequential = seed.windows(2).all(|w| w[1] == w[0].wrapping_add(1));
    if is_sequential {
        return Err(GenesisError::ConfigParseFailed(
            "genesis_seed is sequential pattern (weak entropy)".to_string(),
        ));
    }

    if seed.windows(2).all(|w| w[0] == w[1]) {
        return Err(GenesisError::ConfigParseFailed(
            "genesis_seed is repeating byte pattern (weak entropy)".to_string(),
        ));
    }

    if config.chain.id < 1 || config.chain.id > 3 {
        return Err(GenesisError::ConfigParseFailed(format!(
            "network_id must be 1-3, got {}",
            config.chain.id
        )));
    }

    if config.chain.magic_bytes.len() != 4 {
        return Err(GenesisError::ConfigParseFailed(format!(
            "magic_bytes must be exactly 4 bytes, got {}",
            config.chain.magic_bytes.len()
        )));
    }

    let valid_chain_type = config
        .chain
        .chain_type
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid_chain_type {
        return Err(GenesisError::ConfigParseFailed(format!(
            "chain_type contains invalid characters: {}",
            config.chain.chain_type
        )));
    }

    let allowed_chain_types = ["mainnet", "testnet", "devnet"];
    if !allowed_chain_types.contains(&config.chain.chain_type.as_str()) {
        return Err(GenesisError::ConfigParseFailed(format!(
            "chain_type must be one of {:?}, got: {}",
            allowed_chain_types, config.chain.chain_type
        )));
    }

    if config.outputs.assets_export_path.trim().is_empty() {
        return Err(GenesisError::ConfigParseFailed(
            "outputs.assets_export_path must not be empty".to_string(),
        ));
    }

    if config.outputs.snapshot_export_path.trim().is_empty() {
        return Err(GenesisError::ConfigParseFailed(
            "outputs.snapshot_export_path must not be empty".to_string(),
        ));
    }

    if matches!(
        config.performance.num_threads,
        crate::genesis::genesis_config::ThreadCountConfig::Fixed(0)
    ) {
        return Err(GenesisError::ConfigParseFailed(
            "performance.num_threads must be >= 1 or 'auto'".to_string(),
        ));
    }

    validate_wallet_profile_schema(&config.wallet_profiles)?;
    validate_policy_profile_schema(&config.policy_profiles)?;
    validate_profile_cross_references(config, plan)?;

    Ok(())
}

fn validate_profile_cross_references(
    config: &GenesisConfig,
    plan: &GenesisGenerationPlan,
) -> Result<(), GenesisError> {
    let asset_ids: BTreeSet<&str> = config
        .assets
        .iter()
        .map(|asset| asset.id.as_str())
        .collect();
    let right_classes: BTreeSet<&str> = config
        .rights
        .iter()
        .map(|right| right.right_class.as_str())
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

    let validate_asset_refs = plan.includes_lane(GenesisLane::Assets);
    let validate_right_refs = plan.includes_lane(GenesisLane::Rights);

    for profile in &config.wallet_profiles {
        if validate_asset_refs {
            if let Some(asset_id) = profile.locked_asset_id.as_deref() {
                if !asset_ids.contains(asset_id) {
                    return Err(GenesisError::ConfigParseFailed(format!(
                        "wallet profile {} references unknown locked_asset_id {}",
                        profile.id, asset_id
                    )));
                }
            }
        }

        if !validate_right_refs || profile.object_family != ObjectFamily::Right {
            continue;
        }

        let live_anchors = match &profile.live_anchor {
            crate::genesis::genesis_config::ProfileAnchor::One(anchor) => {
                std::slice::from_ref(anchor)
            }
            crate::genesis::genesis_config::ProfileAnchor::Many(anchors) => anchors.as_slice(),
        };
        for anchor in live_anchors {
            if !right_classes.contains(anchor.as_str()) {
                return Err(GenesisError::ConfigParseFailed(format!(
                    "wallet profile {} references unknown right live_anchor {}",
                    profile.id, anchor
                )));
            }
        }

        for (field_name, value) in [
            ("disclosure_policy", profile.disclosure_policy.as_deref()),
            ("retention_policy", profile.retention_policy.as_deref()),
        ] {
            if let Some(value) = value {
                if !right_policy_ids.contains(value) {
                    return Err(GenesisError::ConfigParseFailed(format!(
                        "wallet profile {} references unknown rights.{field_name}_id {}",
                        profile.id, value
                    )));
                }
            }
        }
    }

    for profile in &config.policy_profiles {
        if let Some(retention_profile) = profile.retention_profile.as_deref() {
            if !policy_profile_ids.contains(retention_profile) {
                return Err(GenesisError::ConfigParseFailed(format!(
                    "policy profile {} references unknown retention_profile {}",
                    profile.id, retention_profile
                )));
            }
        }

        for applies_to_profile in &profile.applies_to_profiles {
            if !policy_profile_ids.contains(applies_to_profile.as_str()) {
                return Err(GenesisError::ConfigParseFailed(format!(
                    "policy profile {} references unknown applies_to_profiles entry {}",
                    profile.id, applies_to_profile
                )));
            }
        }
    }

    Ok(())
}

pub fn validate_genesis_config_for(
    config: &GenesisConfig,
    plan: &GenesisGenerationPlan,
) -> Result<(), GenesisError> {
    plan.validate_shape()?;
    validate_common_config(config, plan)?;

    if plan.includes_lane(GenesisLane::Assets) {
        validate_assets_schema(&config.assets)?;
    }

    if plan.includes_lane(GenesisLane::Rights) {
        validate_rights_schema(&config.rights)?;
    }

    let policy_records = if plan.needs_policy_resolution(config) {
        Some(crate::genesis::generate_genesis_policies(
            &config.assets,
            &config.policies,
        )?)
    } else {
        None
    };

    if plan.includes_lane(GenesisLane::Vouchers) {
        let policy_records = policy_records.as_deref().ok_or_else(|| {
            GenesisError::InvalidConfig(
                "voucher generation plan must resolve policy records".to_string(),
            )
        })?;
        validate_voucher_schema(&config.vouchers, policy_records)?;
    }

    Ok(())
}

/// Validate assets configuration schema.
pub fn validate_assets_schema(assets: &[AssetConfigEntry]) -> Result<(), GenesisError> {
    let mut asset_ids = BTreeSet::new();

    for (idx, asset) in assets.iter().enumerate() {
        if !asset_ids.insert(&asset.id) {
            return Err(GenesisError::ConfigParseFailed(format!(
                "Duplicate asset ID: {}",
                asset.id
            )));
        }

        validate_domain_name("assets.domain_name", asset.domain_name.as_str())
            .map_err(|err| GenesisError::ConfigParseFailed(err.to_string()))?;

        if asset.policy.serials == 0 {
            return Err(GenesisError::ConfigParseFailed(format!(
                "Asset[{}] serials must be > 0",
                idx
            )));
        }

        if asset.policy.nominal == 0 && asset.class != AssetClass::Void {
            return Err(GenesisError::ConfigParseFailed(format!(
                "Asset[{}] nominal must be > 0 (except Void assets)",
                idx
            )));
        }

        if (asset.policy.serials as u64)
            .checked_mul(asset.policy.nominal)
            .is_none()
        {
            return Err(GenesisError::ConfigParseFailed(format!(
                "Asset[{}] total supply overflow: {} × {}",
                idx, asset.policy.serials, asset.policy.nominal
            )));
        }

        if asset.policy.decimals > 32 {
            return Err(GenesisError::ConfigParseFailed(format!(
                "Asset[{}] decimals {} exceeds maximum 32",
                idx, asset.policy.decimals
            )));
        }

        const MAX_SERIALS_PER_ASSET: u32 = 1_000_000_000;
        if asset.policy.serials > MAX_SERIALS_PER_ASSET {
            return Err(GenesisError::ConfigParseFailed(format!(
                "Asset[{}] serials {} exceeds maximum {} (DoS protection)",
                idx, asset.policy.serials, MAX_SERIALS_PER_ASSET
            )));
        }

        const MAX_NOMINAL_VALUE: u64 = 1_000_000_000_000;
        if asset.policy.nominal > MAX_NOMINAL_VALUE {
            return Err(GenesisError::ConfigParseFailed(format!(
                "Asset[{}] nominal {} exceeds maximum {} (overflow protection)",
                idx, asset.policy.nominal, MAX_NOMINAL_VALUE
            )));
        }
    }

    Ok(())
}

/// Validate rights configuration schema.
pub fn validate_rights_schema(
    rights: &[crate::rights::RightsConfigEntry],
) -> Result<(), GenesisError> {
    let mut right_ids = BTreeSet::new();

    if rights.is_empty() {
        return Err(GenesisError::ConfigParseFailed(
            "Missing or empty 'rights' array in config".to_string(),
        ));
    }

    for (idx, right) in rights.iter().enumerate() {
        if !right_ids.insert(&right.id) {
            return Err(GenesisError::ConfigParseFailed(format!(
                "Duplicate right ID: {}",
                right.id
            )));
        }

        right
            .validate()
            .map_err(|err| GenesisError::ConfigParseFailed(err.to_string()))?;

        if right.count == 0 {
            return Err(GenesisError::ConfigParseFailed(format!(
                "Right[{}] count must be > 0",
                idx
            )));
        }
    }

    Ok(())
}

pub fn validate_voucher_schema(
    vouchers: &[crate::vouchers::VoucherBootstrapEntryV1],
    policy_records: &[crate::genesis::GenesisPolicyRecord],
) -> Result<(), GenesisError> {
    let mut policy_lookup = BTreeMap::new();
    for policy in policy_records {
        policy_lookup.insert(policy.descriptor.label.as_str(), policy);
    }
    let mut seen_ids = BTreeSet::new();

    for voucher in vouchers {
        voucher
            .validate()
            .map_err(|err| GenesisError::ConfigParseFailed(err.to_string()))?;

        if !seen_ids.insert(voucher.id.as_str()) {
            return Err(GenesisError::ConfigParseFailed(format!(
                "duplicate voucher ID: {}",
                voucher.id
            )));
        }

        let policy = policy_lookup
            .get(voucher.policy_label.as_str())
            .ok_or_else(|| {
                GenesisError::ConfigParseFailed(format!(
                    "voucher {} references unknown policy {}",
                    voucher.id, voucher.policy_label
                ))
            })?;

        if policy.descriptor.primary_family != ObjectFamily::Voucher {
            return Err(GenesisError::ConfigParseFailed(format!(
                "voucher {} must reference a voucher policy, got {}",
                voucher.id,
                policy.descriptor.primary_family.as_str()
            )));
        }
    }

    Ok(())
}

fn validate_wallet_profile_schema(
    wallet_profiles: &[crate::genesis::genesis_config::WalletProfileConfig],
) -> Result<(), GenesisError> {
    let mut seen_ids = BTreeSet::new();

    for profile in wallet_profiles {
        profile
            .validate()
            .map_err(|err| GenesisError::ConfigParseFailed(err.to_string()))?;

        if !seen_ids.insert(profile.id.as_str()) {
            return Err(GenesisError::ConfigParseFailed(format!(
                "duplicate wallet profile ID: {}",
                profile.id
            )));
        }
    }

    Ok(())
}

fn validate_policy_profile_schema(
    policy_profiles: &[crate::genesis::genesis_config::PolicyProfileConfig],
) -> Result<(), GenesisError> {
    let mut seen_ids = BTreeSet::new();

    for profile in policy_profiles {
        profile
            .validate()
            .map_err(|err| GenesisError::ConfigParseFailed(err.to_string()))?;

        if !seen_ids.insert(profile.id.as_str()) {
            return Err(GenesisError::ConfigParseFailed(format!(
                "duplicate policy profile ID: {}",
                profile.id
            )));
        }
    }

    Ok(())
}

/// Validate version compatibility for genesis state export/import.
pub fn validate_version_compatibility(version: &str) -> Result<(), GenesisError> {
    const SUPPORTED_VERSIONS: &[&str] = &["3.0", "3.8", "3.8.0"];

    if !SUPPORTED_VERSIONS.contains(&version) {
        return Err(GenesisError::ConfigParseFailed(format!(
            "Unsupported version: {}. Supported: {:?}",
            version, SUPPORTED_VERSIONS
        )));
    }

    Ok(())
}

/// Validate genesis seed meets M1 security requirements.
pub fn validate_genesis_seed(seed: &[u8; 32], network_type: ChainType) -> Result<(), GenesisError> {
    if seed == &[0u8; 32] {
        return Err(GenesisError::InsecureGenesisSeed(
            "All-zero seed is forbidden".into(),
        ));
    }

    if seed == &[0xFF; 32] {
        return Err(GenesisError::InsecureGenesisSeed(
            "All-ones seed is forbidden".into(),
        ));
    }

    if is_sequential_pattern(seed) {
        return Err(GenesisError::InsecureGenesisSeed(
            "Sequential pattern seed is forbidden".into(),
        ));
    }

    if has_repeating_bytes(seed) {
        return Err(GenesisError::InsecureGenesisSeed(
            "Repeating byte pattern seed is forbidden".into(),
        ));
    }

    if is_protected_network(network_type) && is_known_test_seed(seed) {
        return Err(GenesisError::TestSeedInProduction);
    }

    Ok(())
}

fn is_sequential_pattern(seed: &[u8; 32]) -> bool {
    seed.windows(2).all(|w| w[1] == w[0].wrapping_add(1))
}

fn has_repeating_bytes(seed: &[u8; 32]) -> bool {
    seed.windows(2).all(|w| w[0] == w[1])
}

fn is_known_test_seed(seed: &[u8; 32]) -> bool {
    const KNOWN_TEST_SEEDS: &[[u8; 32]] = &[[42; 32]];
    KNOWN_TEST_SEEDS.contains(seed)
}

fn is_protected_network(network_type: ChainType) -> bool {
    matches!(network_type, ChainType::Mainnet | ChainType::Testnet)
}
