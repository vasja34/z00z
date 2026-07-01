use super::{
    parse_asset_domain_name, parse_asset_policy, parse_policy_flags, AssetClass, AssetDefinition,
    AssetError, BTreeMap, DefinitionWire, Path, YamlValue, MAX_REGISTRY_CATALOG_FILE_SIZE,
};
use z00z_utils::io::load_yaml_bounded;

fn parse_metadata(asset_yaml: &YamlValue) -> Result<Option<BTreeMap<String, String>>, AssetError> {
    let Some(value) = asset_yaml.get("metadata") else {
        return Ok(None);
    };
    let mapping = value
        .as_mapping()
        .ok_or_else(|| AssetError::InvalidMetadata("metadata must be a mapping".into()))?;

    let mut metadata = BTreeMap::new();
    for (key, value) in mapping {
        let key = key
            .as_str()
            .ok_or_else(|| AssetError::InvalidMetadata("metadata keys must be strings".into()))?;
        let value = value
            .as_str()
            .ok_or_else(|| AssetError::InvalidMetadata("metadata values must be strings".into()))?;
        metadata.insert(key.to_string(), value.to_string());
    }

    Ok(Some(metadata))
}

/// Compute asset ID deterministically from config fields
///
/// Uses domain-separated hashing to generate unique 32-byte IDs.
///
/// # Arguments
///
/// * `symbol` - Asset symbol
/// * `class` - Asset class
/// * `domain_name` - Domain name for asset
/// * `serials` - Number of serials
/// * `nominal` - Nominal value
///
/// # Returns
///
/// 32-byte asset ID
pub(crate) fn compute_asset_id_from_catalog(
    name: &str,
    symbol: &str,
    class: AssetClass,
    decimals: u8,
    domain_name: &str,
    serials: u32,
    nominal: u64,
    version: u8,
    crypto_version: u8,
    policy_flags: u8,
    metadata: Option<&BTreeMap<String, String>>,
) -> [u8; 32] {
    AssetDefinition::derive_id(
        class,
        name,
        symbol,
        decimals,
        serials,
        nominal,
        domain_name,
        version,
        crypto_version,
        policy_flags,
        metadata,
    )
}

fn parse_asset_definitions(yaml: &YamlValue) -> Result<Vec<DefinitionWire>, AssetError> {
    let assets = yaml
        .get("assets")
        .and_then(|v| v.as_sequence())
        .ok_or_else(|| AssetError::InvalidAsset("Missing 'assets' array in config".into()))?;

    let mut wire_definitions = Vec::new();

    for asset_yaml in assets {
        let id_str = asset_yaml
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AssetError::InvalidAsset("Missing 'id' field".into()))?;

        let symbol = asset_yaml
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or(id_str)
            .to_string();

        let name = asset_yaml
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AssetError::InvalidAsset("Missing 'name' field".into()))?
            .to_string();

        let class_str = asset_yaml
            .get("class")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AssetError::InvalidAsset("Missing 'class' field".into()))?;

        let class = match class_str.to_lowercase().as_str() {
            "coin" => AssetClass::Coin,
            "token" => AssetClass::Token,
            "nft" => AssetClass::Nft,
            "void" => AssetClass::Void,
            _ => {
                return Err(AssetError::InvalidAsset(
                    format!("Unknown asset class: {}", class_str).into(),
                ));
            }
        };

        let policy = asset_yaml
            .get("policy")
            .ok_or_else(|| AssetError::InvalidAsset("Missing 'policy' field".into()))?;

        let (decimals, serials, nominal) = parse_asset_policy(policy)?;
        let domain_name = parse_asset_domain_name(asset_yaml)?;
        let flags = parse_policy_flags(policy, class)?;
        let metadata = parse_metadata(asset_yaml)?;
        let _mutable_flags_section = asset_yaml.get("mutable_flags").and_then(|v| v.as_mapping());

        let version = 1;
        let crypto_version = 1;
        let id = compute_asset_id_from_catalog(
            &name,
            &symbol,
            class,
            decimals,
            &domain_name,
            serials,
            nominal,
            version,
            crypto_version,
            flags,
            metadata.as_ref(),
        );

        wire_definitions.push(DefinitionWire {
            id,
            class,
            name,
            symbol,
            decimals,
            serials,
            nominal,
            domain_name,
            version,
            crypto_version,
            policy_flags: flags,
            metadata,
        });
    }

    Ok(wire_definitions)
}

/// Load secondary registry-catalog definitions from a YAML file
///
/// Parses YAML catalog data and converts it to wire-format definitions.
/// This is the main catalog parsing logic extracted from `registry.rs`.
///
/// Expected YAML format:
/// ```yaml
/// version: 1  # optional, defaults to 1
/// assets:
///   - id: "some_id"  # used as symbol fallback
///     symbol: "Z00Z"
///     name: "Z00Z Privacy Coin"
///     class: Coin
///     domain_name: "z00z.core.assets.coin.devnet.v1"
///     policy:
///       decimals: 8
///       serials: 50000
///       nominal: 100000000
///       burnable: true
///       flags:
///         gas: true
///         fungible: true
/// ```
///
/// # Arguments
///
/// * `path` - Path to the registry-catalog YAML file
///
/// # Returns
///
/// Tuple of (version_number, wire_definitions)
///
/// # Errors
///
/// - File not found or read error
/// - Invalid YAML syntax
/// - Missing required fields
/// - Invalid asset class
pub(crate) fn load_catalog_from_yaml(
    path: &Path,
) -> Result<(u64, Vec<DefinitionWire>), AssetError> {
    let yaml: YamlValue = load_yaml_bounded(path, MAX_REGISTRY_CATALOG_FILE_SIZE)?;
    let version_num = yaml.get("version").and_then(|v| v.as_u64()).unwrap_or(1);
    let wire_definitions = parse_asset_definitions(&yaml)?;

    Ok((version_num, wire_definitions))
}
