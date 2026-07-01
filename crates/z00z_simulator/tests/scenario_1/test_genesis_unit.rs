use std::{collections::HashSet, str::FromStr, sync::Arc};

use z00z_core::{
    assets::{Asset, AssetClass, AssetDefinition, BlindingFactor},
    genesis::{
        create_asset_definition, generate_all_genesis_assets,
        genesis_config::{load_genesis_config, GenesisConfig},
        validator::compute_genesis_state_hash,
        ChainType, GenesisSeed,
    },
};
use z00z_simulator::scenario_1::stage_1;
use z00z_utils::{
    io::{load_bincode, save_bincode},
    logger::NoopLogger,
    metrics::NoopMetrics,
    rng::DeterministicRngProvider,
};

fn load_devnet_fixture() -> (GenesisConfig, GenesisSeed, ChainType) {
    let cfg_path = z00z_core::config_paths::devnet_genesis_path();
    let cfg = load_genesis_config(&cfg_path.to_string_lossy()).expect("load devnet config");
    let seed = GenesisSeed::from_config(&cfg).expect("derive genesis seed");
    let net = ChainType::from_str(&cfg.chain.chain_type).expect("parse chain type");
    (cfg, seed, net)
}

fn build_defs(cfg: &GenesisConfig, seed: &GenesisSeed, net: ChainType) -> Vec<AssetDefinition> {
    cfg.assets
        .iter()
        .map(|entry| create_asset_definition(entry, seed.as_bytes(), net).expect("build asset def"))
        .collect()
}

fn make_test_asset(serial_id: u32, amount: u64) -> Asset {
    let definition = Arc::new(
        AssetDefinition::new(
            [7u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "T1".to_string(),
            8,
            serial_id + 1,
            100_000_000,
            "test.io".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("create test definition"),
    );
    let mut rng = DeterministicRngProvider::from_seed([serial_id as u8; 32]).rng();
    let blinding = BlindingFactor::random(&mut rng);
    let nonce = [serial_id as u8; 32];
    Asset::new(definition, serial_id, amount, &blinding, nonce, &mut rng).expect("create asset")
}

#[test]
fn test_stage1_state_is_deterministic() {
    let (cfg, seed, net) = load_devnet_fixture();
    let defs = build_defs(&cfg, &seed, net);

    let acc1 = generate_all_genesis_assets(
        &defs,
        seed.as_bytes(),
        net,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
    )
    .expect("genesis accumulator #1");

    let acc2 = generate_all_genesis_assets(
        &defs,
        seed.as_bytes(),
        net,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
    )
    .expect("genesis accumulator #2");

    assert_eq!(
        compute_genesis_state_hash(&acc1),
        compute_genesis_state_hash(&acc2),
        "state hash must be deterministic for same config + seed",
    );
}

#[test]
fn test_stage1_def_ids_unique() {
    let (cfg, seed, net) = load_devnet_fixture();
    let mut ids: HashSet<[u8; 32]> = HashSet::new();

    for item in &cfg.assets {
        let def = create_asset_definition(item, seed.as_bytes(), net).expect("build asset def");
        assert!(
            ids.insert(def.id),
            "duplicate definition id for {}",
            def.symbol
        );
    }

    assert_eq!(ids.len(), 4, "must have exactly 4 unique definition ids");
}

#[test]
fn test_stage1_bincode_roundtrip() {
    let assets = vec![make_test_asset(42, 1_000)];
    let tmp = tempfile::NamedTempFile::new().expect("temp file");

    save_bincode(tmp.path(), &assets).expect("save bincode");
    let loaded: Vec<Asset> = load_bincode(tmp.path()).expect("load bincode");

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].serial_id, 42);
    assert_eq!(loaded[0].amount, 1_000);
    assert_eq!(
        loaded[0].commitment.as_bytes(),
        assets[0].commitment.as_bytes()
    );
}

#[test]
fn test_stage1_rejects_non_devnet() {
    let (mut cfg, _, _) = load_devnet_fixture();
    cfg.chain.chain_type = "testnet".to_string();

    let result = stage_1::run_core_with_config(&cfg);
    assert!(result.is_err(), "run_core must reject non-devnet config");
    assert!(
        result
            .expect_err("must fail for non-devnet")
            .contains("expected devnet"),
        "error message must mention 'expected devnet'"
    );
}
