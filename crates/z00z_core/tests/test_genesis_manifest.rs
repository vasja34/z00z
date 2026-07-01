use crate::genesis::helpers::create_test_config;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use tempfile::{tempdir, TempDir};
use z00z_core::config_paths::devnet_genesis_path;
use z00z_core::genesis::genesis_config::{load_genesis_config, GenesisConfig};
use z00z_core::genesis::validator::{compute_genesis_state_hash, GenesisError};
use z00z_core::genesis::{
    compute_genesis_manifest_hash, compute_genesis_policies_digest, compute_genesis_rights_digest,
    compute_genesis_vouchers_digest, create_asset_definition, export_genesis_settlement_artifacts,
    generate_genesis_lanes, generate_genesis_policies, generate_genesis_settlement_corpus,
    resolve_genesis_context, run_genesis, run_genesis_with_plan, validate_genesis_config_for,
    ChainType, GenesisExportKind, GenesisGenerationPlan, GenesisLane, GenesisPolicyRecord,
    GenesisRightRecord, GenesisSeed, GenesisSettlementCorpus, GenesisSettlementManifest,
    GenesisVoucherRecord, GENESIS_GENERATION_RECEIPT_FILE, GENESIS_POLICIES_FILE,
    GENESIS_POLICIES_REPLAY_DIGEST_LABEL, GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL,
    GENESIS_RIGHTS_FILE, GENESIS_RIGHTS_REPLAY_DIGEST_LABEL, GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL,
    GENESIS_ROOT_GENERATION, GENESIS_SETTLEMENT_MANIFEST_FILE, GENESIS_VOUCHERS_FILE,
    GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL, GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL,
};
use z00z_utils::io::{load_json, write_file};
use z00z_utils::prelude::{NoopLogger, NoopMetrics};

fn canonical_genesis_path() -> PathBuf {
    devnet_genesis_path()
}

fn canonical_config() -> Result<GenesisConfig, Box<dyn std::error::Error>> {
    load_genesis_config(canonical_genesis_path().to_str().ok_or("utf8 path")?).map_err(Into::into)
}

fn retarget_outputs(config: &mut GenesisConfig, root: &std::path::Path) {
    config.outputs.assets_export_path = root.join("artifacts").display().to_string();
    config.outputs.snapshot_export_path = root.join("snapshots").display().to_string();
    config.outputs.logging_path = root.join("logs").display().to_string();
}

fn write_temp_config(
    temp: &TempDir,
    config: &GenesisConfig,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = temp.path().join("genesis_plan.yaml");
    let yaml = serde_yaml::to_string(config)?;
    write_file(&path, yaml.as_bytes())?;
    Ok(path)
}

fn generated_output_dir(base: &std::path::Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut dirs = std::fs::read_dir(base)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    dirs.sort();
    dirs.into_iter()
        .next()
        .ok_or_else(|| "expected one generated output dir".into())
}

type CorpusBuildOut = (
    Vec<z00z_core::assets::AssetDefinition>,
    Vec<GenesisPolicyRecord>,
    GenesisSettlementCorpus,
    ChainType,
    GenesisSeed,
);

fn build_corpus() -> Result<CorpusBuildOut, Box<dyn std::error::Error>> {
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
    Ok((definitions, policies, corpus, network, genesis_seed))
}

#[test]
fn test_manifest_exports_rights() -> Result<(), Box<dyn std::error::Error>> {
    let (definitions, policies, corpus, network, genesis_seed) = build_corpus()?;
    let state_hash = compute_genesis_state_hash(&corpus);
    let temp = tempdir()?;
    let (rights_path, manifest_path) = export_genesis_settlement_artifacts(
        temp.path(),
        &definitions,
        &policies,
        &corpus,
        network,
        GENESIS_ROOT_GENERATION,
        &state_hash,
        genesis_seed.as_bytes(),
    )?;
    let manifest: GenesisSettlementManifest = load_json(&manifest_path)?;
    let policies: Vec<GenesisPolicyRecord> = load_json(temp.path().join(GENESIS_POLICIES_FILE))?;
    let rights: Vec<GenesisRightRecord> = load_json(&rights_path)?;
    let vouchers: Vec<GenesisVoucherRecord> = load_json(temp.path().join(GENESIS_VOUCHERS_FILE))?;

    assert_eq!(
        rights_path.file_name().and_then(|value| value.to_str()),
        Some(GENESIS_RIGHTS_FILE),
    );
    assert_eq!(
        manifest_path.file_name().and_then(|value| value.to_str()),
        Some(GENESIS_SETTLEMENT_MANIFEST_FILE),
    );
    assert_eq!(manifest.policy_count, policies.len());
    assert_eq!(manifest.right_count, corpus.total_right_count());
    assert_eq!(manifest.voucher_count, vouchers.len());
    assert_eq!(manifest.policies_artifact, GENESIS_POLICIES_FILE);
    assert_eq!(manifest.rights_artifact, GENESIS_RIGHTS_FILE);
    assert_eq!(manifest.vouchers_artifact, GENESIS_VOUCHERS_FILE);
    assert_eq!(manifest.root_generation, GENESIS_ROOT_GENERATION);
    assert_eq!(
        manifest.policies_replay_digest,
        hex::encode(compute_genesis_policies_digest(
            &policies,
            GENESIS_POLICIES_REPLAY_DIGEST_LABEL,
        )),
    );
    assert_eq!(
        manifest.policies_output_roundtrip_digest,
        hex::encode(compute_genesis_policies_digest(
            &policies,
            GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL,
        )),
    );
    assert_eq!(
        manifest.deterministic_replay_digest,
        hex::encode(compute_genesis_rights_digest(
            &corpus.rights,
            GENESIS_RIGHTS_REPLAY_DIGEST_LABEL,
        )),
    );
    assert_eq!(
        manifest.rights_output_roundtrip_digest,
        hex::encode(compute_genesis_rights_digest(
            &rights,
            GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL,
        )),
    );
    assert_eq!(
        manifest.vouchers_replay_digest,
        hex::encode(compute_genesis_vouchers_digest(
            &vouchers,
            GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL,
        )),
    );
    assert_eq!(
        manifest.vouchers_output_roundtrip_digest,
        hex::encode(compute_genesis_vouchers_digest(
            &vouchers,
            GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL,
        )),
    );
    assert_eq!(
        manifest.manifest_hash,
        hex::encode(compute_genesis_manifest_hash(&manifest)),
    );

    Ok(())
}

#[test]
fn test_manifest_hash_drift() -> Result<(), Box<dyn std::error::Error>> {
    let (definitions, policies, original, network, genesis_seed) = build_corpus()?;
    let mut changed = build_corpus()?.2;
    changed.rights[0].leaf.payload_commitment[0] ^= 0xA5;

    let temp = tempdir()?;
    let original_manifest_path = export_genesis_settlement_artifacts(
        temp.path().join("original").as_path(),
        &definitions,
        &policies,
        &original,
        network,
        GENESIS_ROOT_GENERATION,
        &compute_genesis_state_hash(&original),
        genesis_seed.as_bytes(),
    )?
    .1;
    let changed_manifest_path = export_genesis_settlement_artifacts(
        temp.path().join("changed").as_path(),
        &definitions,
        &policies,
        &changed,
        network,
        GENESIS_ROOT_GENERATION,
        &compute_genesis_state_hash(&changed),
        genesis_seed.as_bytes(),
    )?
    .1;
    let original_manifest: GenesisSettlementManifest = load_json(&original_manifest_path)?;
    let changed_manifest: GenesisSettlementManifest = load_json(&changed_manifest_path)?;

    assert_ne!(
        original_manifest.manifest_hash,
        changed_manifest.manifest_hash
    );
    assert_ne!(
        original_manifest.deterministic_replay_digest,
        changed_manifest.deterministic_replay_digest,
    );
    assert_ne!(original_manifest.state_hash, changed_manifest.state_hash);

    Ok(())
}

#[test]
fn test_genesis_plan_full_bootstrap_matches_legacy_run() -> Result<(), Box<dyn std::error::Error>> {
    let config = canonical_config()?;
    let plan = GenesisGenerationPlan::full_bootstrap();
    let legacy_temp = tempdir()?;
    let plan_temp = tempdir()?;

    let mut legacy_config = config.clone();
    retarget_outputs(&mut legacy_config, legacy_temp.path());
    let legacy_path = write_temp_config(&legacy_temp, &legacy_config)?;
    run_genesis(legacy_path.to_str().ok_or("utf8 path")?, None)?;
    let legacy_manifest: GenesisSettlementManifest = load_json(
        generated_output_dir(&legacy_temp.path().join("artifacts"))?
            .join(GENESIS_SETTLEMENT_MANIFEST_FILE),
    )?;

    let mut plan_config = config;
    retarget_outputs(&mut plan_config, plan_temp.path());
    let plan_path = write_temp_config(&plan_temp, &plan_config)?;
    let receipt = run_genesis_with_plan(plan_path.to_str().ok_or("utf8 path")?, &plan, None)?;
    let plan_manifest: GenesisSettlementManifest = load_json(
        generated_output_dir(&plan_temp.path().join("artifacts"))?
            .join(GENESIS_SETTLEMENT_MANIFEST_FILE),
    )?;

    assert_eq!(
        serde_json::to_value(&plan_manifest)?,
        serde_json::to_value(&legacy_manifest)?
    );
    assert_eq!(
        receipt.full_manifest_file.as_deref(),
        Some(GENESIS_SETTLEMENT_MANIFEST_FILE),
    );

    Ok(())
}

#[test]
fn test_genesis_plan_assets_only_skips_rights_validation() -> Result<(), Box<dyn std::error::Error>>
{
    let mut config = create_test_config();
    config.rights.clear();
    let plan = GenesisGenerationPlan::assets_only();

    validate_genesis_config_for(&config, &plan)?;
    let ctx = resolve_genesis_context(config, &plan)?;
    let outputs = generate_genesis_lanes(&ctx, &plan, Arc::new(NoopLogger), Arc::new(NoopMetrics))?;

    assert!(outputs.rights.is_none());
    assert_eq!(
        outputs
            .assets
            .as_ref()
            .ok_or("missing asset lane")?
            .total_count(),
        10,
    );

    Ok(())
}

#[test]
fn test_genesis_plan_rights_only_requires_policy_resolution_when_needed(
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = GenesisGenerationPlan::rights_only();

    let mut config = create_test_config();
    config.assets.clear();
    config.policies.clear();
    validate_genesis_config_for(&config, &plan)?;
    let ctx = resolve_genesis_context(config, &plan)?;
    assert!(
        ctx.policies.is_empty(),
        "rights-only without policy entries should not force policy resolution",
    );
    let outputs = generate_genesis_lanes(&ctx, &plan, Arc::new(NoopLogger), Arc::new(NoopMetrics))?;
    assert_eq!(
        outputs.rights.as_ref().ok_or("missing rights lane")?.len(),
        2,
    );

    let mut canonical = canonical_config()?;
    canonical.assets.clear();
    let ctx_with_policies = resolve_genesis_context(canonical, &plan)?;
    assert!(
        !ctx_with_policies.policies.is_empty(),
        "rights-only should still resolve policies when canonical policy records exist",
    );

    Ok(())
}

#[test]
fn test_genesis_plan_vouchers_only_rejects_non_voucher_policy(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = canonical_config()?;
    config.vouchers[0].policy_label = "right_delegate_policy_v1".to_string();

    let err = resolve_genesis_context(config, &GenesisGenerationPlan::vouchers_only()).unwrap_err();
    assert!(
        err.to_string().contains("voucher policy"),
        "unexpected error: {err}",
    );

    Ok(())
}

#[test]
fn test_genesis_partial_run_does_not_emit_full_settlement_manifest(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut config = canonical_config()?;
    retarget_outputs(&mut config, temp.path());
    let config_path = write_temp_config(&temp, &config)?;

    let receipt = run_genesis_with_plan(
        config_path.to_str().ok_or("utf8 path")?,
        &GenesisGenerationPlan::assets_only(),
        None,
    )?;
    let output_dir = generated_output_dir(&temp.path().join("artifacts"))?;

    assert!(output_dir.join(GENESIS_GENERATION_RECEIPT_FILE).exists());
    assert!(!output_dir.join(GENESIS_SETTLEMENT_MANIFEST_FILE).exists());
    assert_eq!(receipt.output_kind, GenesisExportKind::PartialLaneSet);
    assert!(receipt.full_manifest_file.is_none());

    Ok(())
}

#[test]
fn test_genesis_selected_lanes_preserve_terminal_collision_checks(
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = GenesisGenerationPlan::selected([GenesisLane::Rights, GenesisLane::Vouchers]);
    let ctx = resolve_genesis_context(canonical_config()?, &plan)?;
    let mut outputs =
        generate_genesis_lanes(&ctx, &plan, Arc::new(NoopLogger), Arc::new(NoopMetrics))?;

    let collision_id = outputs.rights.as_ref().ok_or("missing rights lane")?[0]
        .leaf
        .terminal_id;
    outputs.vouchers.as_mut().ok_or("missing vouchers lane")?[0].terminal_id = collision_id;

    let err =
        z00z_core::genesis::ensure_terminal_collision_free(&outputs.combined_corpus()).unwrap_err();
    assert!(
        matches!(err, GenesisError::TerminalCollision { .. }),
        "unexpected error: {err}",
    );

    Ok(())
}
