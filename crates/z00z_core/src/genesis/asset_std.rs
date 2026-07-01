#[cfg(feature = "deterministic-rng")]
use z00z_utils::rng::DeterministicRngProvider;

#[cfg(feature = "deterministic-rng")]
use crate::{
    assets::{nonce::derive_genesis_nonce, Asset, AssetClass, AssetDefinition},
    config_paths::devnet_genesis_path,
    genesis::{
        derive_deterministic_rng_seed, derive_genesis_blinding,
        genesis_config::load_genesis_config, validator::GenesisError, ChainType,
    },
};

#[cfg(not(feature = "deterministic-rng"))]
use crate::{
    assets::{Asset, AssetClass, AssetDefinition},
    config_paths::devnet_genesis_path,
    genesis::{genesis_config::load_genesis_config, validator::GenesisError},
};

fn create_canonical_asset_definition(
    cfg: &crate::genesis::genesis_config::AssetConfigEntry,
) -> Result<AssetDefinition, GenesisError> {
    AssetDefinition::new(
        [0u8; 32],
        cfg.class,
        cfg.name.clone(),
        cfg.symbol.clone(),
        cfg.policy.decimals,
        cfg.policy.serials,
        cfg.policy.nominal,
        cfg.domain_name.clone(),
        1,
        1,
        cfg.policy.asset_flags(cfg.class),
        cfg.metadata.clone(),
    )
    .map_err(|err| GenesisError::AssetCreationFailed {
        definition_id: [0u8; 32],
        serial_id: 0,
        error: err.to_string(),
    })
}

fn dev_cfg_path() -> String {
    devnet_genesis_path().display().to_string()
}

fn code_for_class(class: AssetClass) -> &'static str {
    match class {
        AssetClass::Coin => "z00z",
        AssetClass::Token => "zUSD",
        AssetClass::Nft => "zNFT",
        AssetClass::Void => "zBurnSink",
    }
}

/// Build one standard asset definition from the canonical dev fixture.
pub fn def_from_dev_cfg(asset_code: &str) -> Result<AssetDefinition, GenesisError> {
    let config = load_genesis_config(&dev_cfg_path())?;

    let entry = config
        .assets
        .iter()
        .find(|item| item.id == asset_code)
        .ok_or_else(|| {
            GenesisError::InvalidConfig(format!("asset id not found in dev config: {asset_code}"))
        })?;

    create_canonical_asset_definition(entry)
}

/// Return the canonical serial capacity for the standard dev fixture class.
pub fn serials_from_dev_class(class: AssetClass) -> Result<u32, GenesisError> {
    Ok(def_from_dev_cfg(code_for_class(class))?.serials)
}

/// Build one standard test asset from the canonical dev fixture.
///
/// This keeps the explicit precomputed genesis-id compatibility seam local to runtime genesis
/// generation instead of exposing it through a public helper.
#[cfg(feature = "deterministic-rng")]
pub fn asset_from_dev_cfg(
    asset_code: &str,
    serial_id: u32,
    amount: u64,
) -> Result<Asset, GenesisError> {
    let config = load_genesis_config(&dev_cfg_path())?;
    let chain = config.chain.chain_type.parse::<ChainType>()?;
    let seed = config.chain.domains.genesis_seed;

    let entry = config
        .assets
        .iter()
        .find(|item| item.id == asset_code)
        .ok_or_else(|| {
            GenesisError::InvalidConfig(format!("asset id not found in dev config: {asset_code}"))
        })?;

    let definition = create_canonical_asset_definition(entry)?;
    let definition_id = definition.id;
    let blinding = derive_genesis_blinding(&seed, &definition_id, serial_id, chain)?;
    let nonce = derive_genesis_nonce(&seed, &definition_id, serial_id);
    let rng_seed = derive_deterministic_rng_seed(&seed, &definition_id, serial_id, chain);

    #[cfg(feature = "deterministic-rng")]
    let mut rng = DeterministicRngProvider::from_seed(rng_seed).rng();

    #[cfg(feature = "deterministic-rng")]
    return Asset::new(
        std::sync::Arc::new(definition),
        serial_id,
        amount,
        &blinding,
        nonce,
        &mut rng,
    )
    .map_err(|err| GenesisError::AssetCreationFailed {
        definition_id,
        serial_id,
        error: err.to_string(),
    });
}

#[cfg(not(feature = "deterministic-rng"))]
pub fn asset_from_dev_cfg(
    asset_code: &str,
    serial_id: u32,
    amount: u64,
) -> Result<Asset, GenesisError> {
    let _ = (asset_code, serial_id, amount);
    Err(GenesisError::InvalidConfig(
        "z00z_core deterministic-rng feature is required for asset_from_dev_cfg".to_string(),
    ))
}

/// Build one standard test asset for a concrete class using the canonical dev fixture.
pub fn asset_from_dev_class(
    class: AssetClass,
    serial_id: u32,
    amount: u64,
) -> Result<Asset, GenesisError> {
    asset_from_dev_cfg(code_for_class(class), serial_id, amount)
}
