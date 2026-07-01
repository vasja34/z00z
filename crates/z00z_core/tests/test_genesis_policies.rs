use std::path::PathBuf;

use z00z_core::assets::ObjectFamily;
use z00z_core::config_paths::devnet_genesis_path;
use z00z_core::genesis::genesis_config::load_genesis_config;
use z00z_core::genesis::{generate_genesis_policies, GENESIS_POLICIES_REPLAY_DIGEST_LABEL};

fn canonical_genesis_path() -> PathBuf {
    devnet_genesis_path()
}

#[test]
fn test_genesis_policies_are_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_genesis_config(canonical_genesis_path().to_str().expect("utf8 path"))?;
    let first = generate_genesis_policies(&config.assets, &config.policies)?;
    let second = generate_genesis_policies(&config.assets, &config.policies)?;

    assert_eq!(first, second);
    assert!(
        first
            .iter()
            .any(|record| record.descriptor.label == "cash_policy_v1"),
        "native cash policy must stay built in",
    );
    assert!(
        first.iter().any(|record| {
            record.descriptor.label == "voucher_transferable_policy_v1"
                && record.descriptor.primary_family == ObjectFamily::Voucher
        }),
        "voucher policy should be exported",
    );
    assert!(
        first.iter().any(|record| {
            record.descriptor.label == "right_delegate_policy_v1"
                && record.descriptor.primary_family == ObjectFamily::Right
        }),
        "right policy should be exported",
    );
    assert!(
        z00z_core::genesis::compute_genesis_policies_digest(
            &first,
            GENESIS_POLICIES_REPLAY_DIGEST_LABEL,
        ) != [0u8; 32]
    );

    Ok(())
}

#[test]
fn test_reject_custom_asset_entries() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_genesis_config(canonical_genesis_path().to_str().expect("utf8 path"))?;
    let mut bad = config.policies[0].clone();
    bad.template.primary_family = ObjectFamily::Asset;
    config.policies.push(bad);

    let err = generate_genesis_policies(&config.assets, &config.policies).unwrap_err();
    assert!(
        err.to_string()
            .contains("asset-side custom genesis policy entries are forbidden"),
        "unexpected error: {err}",
    );

    Ok(())
}
