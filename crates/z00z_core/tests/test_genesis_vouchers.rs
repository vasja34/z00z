use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use z00z_core::config_paths::devnet_genesis_path;
use z00z_core::genesis::genesis_config::load_genesis_config;
use z00z_core::genesis::{
    create_asset_definition, generate_genesis_policies, generate_genesis_settlement_corpus,
    ChainType, GenesisSeed,
};
use z00z_core::vouchers::VoucherLifecycleV1;
use z00z_utils::prelude::{NoopLogger, NoopMetrics};

fn canonical_genesis_path() -> PathBuf {
    devnet_genesis_path()
}

#[test]
fn test_covers_positive_negative() -> Result<(), Box<dyn std::error::Error>> {
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

    assert_eq!(corpus.vouchers.len(), 3);
    assert!(
        corpus
            .vouchers
            .iter()
            .any(|record| record.config.lifecycle == VoucherLifecycleV1::Active),
        "transferable active voucher missing",
    );
    assert!(
        corpus
            .vouchers
            .iter()
            .any(|record| record.config.lifecycle == VoucherLifecycleV1::PendingAcceptance),
        "non-transferable pending voucher missing",
    );
    assert!(
        corpus
            .vouchers
            .iter()
            .any(|record| record.config.lifecycle == VoucherLifecycleV1::Expired),
        "expired negative voucher missing",
    );
    assert!(
        corpus
            .vouchers
            .iter()
            .all(|record| record.config.audit_commitment.is_some()),
        "genesis vouchers must keep audit commitments",
    );
    assert!(
        corpus
            .vouchers
            .iter()
            .all(|record| record.terminal_id != record.config.replay_nonce),
        "voucher replay nonces must be domain-derived, not raw config bytes",
    );

    Ok(())
}

#[test]
fn test_reject_missing_policy_binding() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_genesis_config(canonical_genesis_path().to_str().expect("utf8 path"))?;
    config.vouchers[0].policy_label = "missing_policy".to_string();
    let genesis_seed = GenesisSeed::from_config(&config)?;
    let network = ChainType::from_str(&config.chain.chain_type)?;
    let definitions = config
        .assets
        .iter()
        .map(|asset| create_asset_definition(asset, genesis_seed.as_bytes(), network))
        .collect::<Result<Vec<_>, _>>()?;
    let policies = generate_genesis_policies(&config.assets, &config.policies)?;

    let err = generate_genesis_settlement_corpus(
        &definitions,
        &config.rights,
        &config.vouchers,
        &policies,
        genesis_seed.as_bytes(),
        config.chain.id,
        network,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
    )
    .unwrap_err();

    assert!(
        err.to_string().contains("references unknown policy"),
        "unexpected error: {err}",
    );

    Ok(())
}
