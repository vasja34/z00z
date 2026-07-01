use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use tempfile::tempdir;
use z00z_core::genesis::genesis_config::load_genesis_config;
use z00z_core::genesis::{
    create_asset_definition, generate_genesis_policies, generate_genesis_settlement_corpus,
    ChainType, GenesisRightRecord, GenesisSeed,
};
use z00z_core::rights::RightClassConfig;
use z00z_storage::settlement::{
    DefinitionId, RightClass, RightLeaf, SerialId, SettlementLeafFamily, SettlementListReq,
    SettlementLookup, SettlementPath, SettlementStore, StoreItem, TerminalId,
};
use z00z_utils::prelude::{NoopLogger, NoopMetrics};

fn canonical_genesis_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../z00z_core/configs/devnet_genesis_config.yaml")
}

fn right_class(class: RightClassConfig) -> RightClass {
    match class {
        RightClassConfig::MachineCapability => RightClass::MachineCapability,
        RightClassConfig::DataAccess => RightClass::DataAccess,
        RightClassConfig::ServiceEntitlement => RightClass::ServiceEntitlement,
        RightClassConfig::ValidatorMandate => RightClass::ValidatorMandate,
        RightClassConfig::OneTimeUse => RightClass::OneTimeUse,
    }
}

fn right_item(record: &GenesisRightRecord) -> Result<StoreItem, Box<dyn std::error::Error>> {
    let path = SettlementPath::new(
        DefinitionId::new(record.definition_id),
        SerialId::new(record.serial_id),
        TerminalId::new(record.leaf.terminal_id),
    );
    let leaf = RightLeaf {
        version: record.leaf.version,
        terminal_id: TerminalId::new(record.leaf.terminal_id),
        right_class: right_class(record.leaf.right_class),
        issuer_scope: record.leaf.issuer_scope,
        provider_scope: record.leaf.provider_scope,
        holder_commitment: record.leaf.holder_commitment,
        control_commitment: record.leaf.control_commitment,
        beneficiary_commitment: record.leaf.beneficiary_commitment,
        payload_commitment: record.leaf.payload_commitment,
        valid_from: record.leaf.valid_from,
        valid_until: record.leaf.valid_until,
        challenge_from: record.leaf.challenge_from,
        challenge_until: record.leaf.challenge_until,
        use_nonce: record.leaf.use_nonce,
        revocation_policy_id: record.leaf.revocation_policy_id,
        transition_policy_id: record.leaf.transition_policy_id,
        challenge_policy_id: record.leaf.challenge_policy_id,
        disclosure_policy_id: record.leaf.disclosure_policy_id,
        retention_policy_id: record.leaf.retention_policy_id,
    };
    Ok(StoreItem::new(path, leaf)?)
}

#[test]
fn test_ingestion_creates_rights() -> Result<(), Box<dyn std::error::Error>> {
    let path = canonical_genesis_path();
    let config = load_genesis_config(path.to_str().expect("utf8 path"))?;
    let genesis_seed = GenesisSeed::from_config(&config)?;
    let network = ChainType::from_str(&config.chain.chain_type)?;
    let definitions = config
        .assets
        .iter()
        .map(|asset| create_asset_definition(asset, genesis_seed.as_bytes(), network))
        .collect::<Result<Vec<_>, _>>()?;
    let policies = generate_genesis_policies(&config.assets, &config.policies)?;
    let corpus = generate_genesis_settlement_corpus(
        &definitions,
        &config.rights,
        &config.vouchers,
        &policies,
        genesis_seed.as_bytes(),
        config.chain.id,
        network,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
    )?;
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;

    for record in &corpus.rights {
        let item = right_item(record)?;
        let _ = store.put_settlement_item(item)?;
    }
    let page = store.list_settlement(SettlementListReq::all(corpus.total_right_count().max(1)))?;
    assert_eq!(page.items().len(), corpus.total_right_count());

    for record in &corpus.rights {
        let path = SettlementPath::new(
            DefinitionId::new(record.definition_id),
            SerialId::new(record.serial_id),
            TerminalId::new(record.leaf.terminal_id),
        );
        let item = store
            .lookup_settlement(SettlementLookup::Terminal(TerminalId::new(
                record.leaf.terminal_id,
            )))?
            .expect("generated right item");
        let expected_leaf = *right_item(record)?.right_leaf()?;
        assert_eq!(item.path(), path);
        assert_eq!(item.right_leaf()?, &expected_leaf);

        let proof = store.settlement_proof_blob(&path)?;
        assert_eq!(proof.hjmt_leaf_family(), Some(SettlementLeafFamily::Right));
        store.validate_settlement_proof_blob(&proof)?;
    }

    let root = store.settlement_root()?;
    drop(store);

    let reopened = SettlementStore::load(temp.path())?;
    assert_eq!(reopened.settlement_root()?, root);

    Ok(())
}
