use crate::assets::registry::AssetDefinitionRegistry;

use super::*;

pub const GENESIS_GENERATION_RECEIPT_FILE: &str = "genesis_generation_receipt.json";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum GenesisLane {
    Policies,
    Assets,
    Rights,
    Vouchers,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenesisSelection {
    FullBootstrap,
    PoliciesOnly,
    AssetsOnly,
    RightsOnly,
    VouchersOnly,
    Selected(BTreeSet<GenesisLane>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenesisExportKind {
    FullBootstrap,
    PartialLaneSet,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenesisGenerationPlan {
    pub selection: GenesisSelection,
    pub export_kind: GenesisExportKind,
}

impl Default for GenesisGenerationPlan {
    fn default() -> Self {
        Self::full_bootstrap()
    }
}

impl GenesisGenerationPlan {
    #[must_use]
    pub fn full_bootstrap() -> Self {
        Self {
            selection: GenesisSelection::FullBootstrap,
            export_kind: GenesisExportKind::FullBootstrap,
        }
    }

    #[must_use]
    pub fn policies_only() -> Self {
        Self {
            selection: GenesisSelection::PoliciesOnly,
            export_kind: GenesisExportKind::PartialLaneSet,
        }
    }

    #[must_use]
    pub fn assets_only() -> Self {
        Self {
            selection: GenesisSelection::AssetsOnly,
            export_kind: GenesisExportKind::PartialLaneSet,
        }
    }

    #[must_use]
    pub fn rights_only() -> Self {
        Self {
            selection: GenesisSelection::RightsOnly,
            export_kind: GenesisExportKind::PartialLaneSet,
        }
    }

    #[must_use]
    pub fn vouchers_only() -> Self {
        Self {
            selection: GenesisSelection::VouchersOnly,
            export_kind: GenesisExportKind::PartialLaneSet,
        }
    }

    #[must_use]
    pub fn selected<I>(lanes: I) -> Self
    where
        I: IntoIterator<Item = GenesisLane>,
    {
        Self {
            selection: GenesisSelection::Selected(lanes.into_iter().collect()),
            export_kind: GenesisExportKind::PartialLaneSet,
        }
    }

    #[must_use]
    pub(crate) fn is_full_bootstrap(&self) -> bool {
        matches!(self.selection, GenesisSelection::FullBootstrap)
    }

    #[must_use]
    pub(crate) fn includes_lane(&self, lane: GenesisLane) -> bool {
        match &self.selection {
            GenesisSelection::FullBootstrap => true,
            GenesisSelection::PoliciesOnly => lane == GenesisLane::Policies,
            GenesisSelection::AssetsOnly => lane == GenesisLane::Assets,
            GenesisSelection::RightsOnly => lane == GenesisLane::Rights,
            GenesisSelection::VouchersOnly => lane == GenesisLane::Vouchers,
            GenesisSelection::Selected(lanes) => lanes.contains(&lane),
        }
    }

    #[must_use]
    pub(crate) fn requested_lanes(&self) -> BTreeSet<GenesisLane> {
        match &self.selection {
            GenesisSelection::FullBootstrap => BTreeSet::from([
                GenesisLane::Policies,
                GenesisLane::Assets,
                GenesisLane::Rights,
                GenesisLane::Vouchers,
            ]),
            GenesisSelection::PoliciesOnly => BTreeSet::from([GenesisLane::Policies]),
            GenesisSelection::AssetsOnly => BTreeSet::from([GenesisLane::Assets]),
            GenesisSelection::RightsOnly => BTreeSet::from([GenesisLane::Rights]),
            GenesisSelection::VouchersOnly => BTreeSet::from([GenesisLane::Vouchers]),
            GenesisSelection::Selected(lanes) => lanes.clone(),
        }
    }

    #[must_use]
    pub(crate) fn needs_policy_resolution(&self, config: &GenesisConfig) -> bool {
        self.includes_lane(GenesisLane::Policies)
            || self.includes_lane(GenesisLane::Vouchers)
            || (self.includes_lane(GenesisLane::Rights) && !config.policies.is_empty())
    }

    #[must_use]
    pub(crate) fn needs_cross_lane_checks(&self) -> bool {
        self.is_full_bootstrap() || self.requested_lanes().len() > 1
    }

    pub(crate) fn validate_shape(&self) -> Result<(), GenesisError> {
        match (&self.selection, self.export_kind) {
            (GenesisSelection::FullBootstrap, GenesisExportKind::FullBootstrap) => Ok(()),
            (GenesisSelection::FullBootstrap, GenesisExportKind::PartialLaneSet) => {
                Err(GenesisError::InvalidConfig(
                    "full_bootstrap plan must keep full_bootstrap export kind".to_string(),
                ))
            }
            (_, GenesisExportKind::FullBootstrap) => Err(GenesisError::InvalidConfig(
                "partial lane plans must not claim full_bootstrap export kind".to_string(),
            )),
            (GenesisSelection::Selected(lanes), _) if lanes.is_empty() => {
                Err(GenesisError::InvalidConfig(
                    "selected genesis plan must request at least one lane".to_string(),
                ))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GenesisResolvedContext {
    pub config: GenesisConfig,
    pub seed: GenesisSeed,
    pub chain_id: u32,
    pub network_type: ChainType,
    pub policies: Vec<GenesisPolicyRecord>,
    pub policy_lookup: BTreeMap<String, GenesisPolicyRecord>,
}

#[derive(Clone, Debug, Default)]
pub struct GenesisLaneOutputs {
    pub asset_definitions: Option<Vec<AssetDefinition>>,
    pub assets: Option<GenesisAssetAccumulator>,
    pub rights: Option<Vec<GenesisRightRecord>>,
    pub vouchers: Option<Vec<GenesisVoucherRecord>>,
    pub policies: Option<Vec<GenesisPolicyRecord>>,
}

impl GenesisLaneOutputs {
    #[must_use]
    pub fn combined_corpus(&self) -> GenesisSettlementCorpus {
        let mut corpus = self.assets.clone().unwrap_or_default();
        if let Some(rights) = &self.rights {
            corpus.rights = rights.clone();
        }
        if let Some(vouchers) = &self.vouchers {
            corpus.vouchers = vouchers.clone();
        }
        corpus
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenesisGenerationReceipt {
    pub selection: GenesisSelection,
    pub output_kind: GenesisExportKind,
    pub network: String,
    pub emitted_files: Vec<String>,
    pub asset_count: usize,
    pub policy_count: usize,
    pub right_count: usize,
    pub voucher_count: usize,
    pub full_manifest_file: Option<String>,
}

pub(super) fn build_genesis_thread_pool(
    thread_config: &crate::genesis::genesis_config::ThreadCountConfig,
) -> Result<rayon::ThreadPool, GenesisError> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_config.resolved_threads())
        .thread_name(|idx| format!("z00z-genesis-worker-{idx}"))
        .build()
        .map_err(|err| GenesisError::ThreadPoolBuildFailed(err.to_string()))
}

pub fn load_genesis_context(
    config_path: &str,
    plan: &GenesisGenerationPlan,
) -> Result<GenesisResolvedContext, GenesisError> {
    let config = load_genesis_config(config_path)?;
    resolve_genesis_context(config, plan)
}

pub fn resolve_genesis_context(
    config: GenesisConfig,
    plan: &GenesisGenerationPlan,
) -> Result<GenesisResolvedContext, GenesisError> {
    plan.validate_shape()?;
    crate::genesis::validator::validate_genesis_config_for(&config, plan)?;

    let seed = GenesisSeed::from_config(&config)?;
    let network_type = ChainType::from_str(&config.chain.chain_type)?;
    let policies = if plan.needs_policy_resolution(&config) {
        generate_genesis_policies(&config.assets, &config.policies)?
    } else {
        Vec::new()
    };
    let policy_lookup = policy_lookup(&policies)?;

    Ok(GenesisResolvedContext {
        chain_id: config.chain.id,
        config,
        seed,
        network_type,
        policies,
        policy_lookup,
    })
}

pub fn generate_genesis_lanes(
    ctx: &GenesisResolvedContext,
    plan: &GenesisGenerationPlan,
    logger: Arc<dyn Logger>,
    metrics: Arc<dyn MetricsSink>,
) -> Result<GenesisLaneOutputs, GenesisError> {
    plan.validate_shape()?;

    let mut outputs = GenesisLaneOutputs::default();

    if plan.is_full_bootstrap() {
        let definitions = ctx
            .config
            .assets
            .iter()
            .map(|asset| {
                create_prechecked_asset_definition(asset, ctx.seed.as_bytes(), ctx.network_type)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let corpus = generate_genesis_settlement_corpus_prechecked(
            &definitions,
            &ctx.config.rights,
            &ctx.config.vouchers,
            &ctx.policies,
            ctx.seed.as_bytes(),
            ctx.chain_id,
            ctx.network_type,
            logger,
            metrics,
        )?;
        outputs.asset_definitions = Some(definitions);
        outputs.assets = Some(GenesisSettlementCorpus {
            coins: corpus.coins.clone(),
            tokens: corpus.tokens.clone(),
            nfts: corpus.nfts.clone(),
            voids: corpus.voids.clone(),
            rights: Vec::new(),
            vouchers: Vec::new(),
        });
        outputs.rights = Some(corpus.rights);
        outputs.vouchers = Some(corpus.vouchers);
        outputs.policies = Some(ctx.policies.clone());
        return Ok(outputs);
    }

    if plan.includes_lane(GenesisLane::Assets) {
        let definitions = ctx
            .config
            .assets
            .iter()
            .map(|asset| {
                create_prechecked_asset_definition(asset, ctx.seed.as_bytes(), ctx.network_type)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let assets = generate_all_genesis_assets(
            &definitions,
            ctx.seed.as_bytes(),
            ctx.network_type,
            logger,
            metrics,
        )?;
        outputs.asset_definitions = Some(definitions);
        outputs.assets = Some(assets);
    }

    if plan.includes_lane(GenesisLane::Policies) {
        outputs.policies = Some(ctx.policies.clone());
    }

    if plan.includes_lane(GenesisLane::Rights) {
        outputs.rights = Some(generate_genesis_rights_with_policies(
            &ctx.config.rights,
            &ctx.policy_lookup,
            ctx.seed.as_bytes(),
            ctx.chain_id,
            ctx.network_type,
            GENESIS_ROOT_GENERATION,
        )?);
    }

    if plan.includes_lane(GenesisLane::Vouchers) {
        outputs.vouchers = Some(generate_genesis_vouchers(
            &ctx.config.vouchers,
            &ctx.policy_lookup,
            ctx.seed.as_bytes(),
            ctx.chain_id,
            ctx.network_type,
            GENESIS_ROOT_GENERATION,
        )?);
    }

    if plan.needs_cross_lane_checks() {
        ensure_terminal_collision_free(&outputs.combined_corpus())?;
    }

    Ok(outputs)
}

fn build_owner_registry(
    definitions: &[AssetDefinition],
) -> Result<AssetDefinitionRegistry, GenesisError> {
    AssetDefinitionRegistry::from_definitions(definitions)
        .map_err(|err| GenesisError::RegistryInsertFailed(err.to_string()))
}

fn export_selected_assets(
    output_dir: &Path,
    config: &GenesisConfig,
    definitions: &[AssetDefinition],
    accumulator: &GenesisAssetAccumulator,
    emitted_files: &mut Vec<String>,
    logger: &dyn Logger,
) -> Result<(), GenesisError> {
    let mut export_config = config.outputs.clone();
    export_config.assets_export_path = output_dir.to_string_lossy().to_string();

    for (asset_cfg, definition) in config.assets.iter().zip(definitions.iter()) {
        let assets = match definition.class {
            AssetClass::Coin => &accumulator.coins,
            AssetClass::Token => &accumulator.tokens,
            AssetClass::Nft => &accumulator.nfts,
            AssetClass::Void => &accumulator.voids,
        };

        if assets.is_empty() {
            continue;
        }

        export_genesis_assets(assets, &asset_cfg.symbol, &export_config)?;
        emitted_files.push(format!("genesis_{}.json", asset_cfg.symbol));
        emitted_files.push(format!("genesis_{}.bin", asset_cfg.symbol));
        logger.info(&format!(
            "Exported {} - {} assets",
            asset_cfg.symbol,
            assets.len()
        ));
    }

    Ok(())
}

fn save_lane_json<T: serde::Serialize>(
    output_dir: &Path,
    file_name: &str,
    payload: &T,
) -> Result<(), GenesisError> {
    let path = output_dir.join(file_name);
    z00z_utils::io::save_json(&path, payload).map_err(|err| GenesisError::FileWriteFailed {
        path: path.display().to_string(),
        error: err.to_string(),
    })
}

fn export_partial_receipt(
    output_dir: &Path,
    ctx: &GenesisResolvedContext,
    plan: &GenesisGenerationPlan,
    outputs: &GenesisLaneOutputs,
    logger: &dyn Logger,
) -> Result<GenesisGenerationReceipt, GenesisError> {
    let mut emitted_files = Vec::new();

    if let (Some(definitions), Some(accumulator)) = (&outputs.asset_definitions, &outputs.assets) {
        export_selected_assets(
            output_dir,
            &ctx.config,
            definitions,
            accumulator,
            &mut emitted_files,
            logger,
        )?;
    }

    if let Some(policies) = &outputs.policies {
        save_lane_json(output_dir, GENESIS_POLICIES_FILE, policies)?;
        emitted_files.push(GENESIS_POLICIES_FILE.to_string());
        logger.info(&format!("Exported {} genesis policies", policies.len()));
    }

    if let Some(rights) = &outputs.rights {
        save_lane_json(output_dir, GENESIS_RIGHTS_FILE, rights)?;
        emitted_files.push(GENESIS_RIGHTS_FILE.to_string());
        logger.info(&format!("Exported {} rights", rights.len()));
    }

    if let Some(vouchers) = &outputs.vouchers {
        save_lane_json(output_dir, GENESIS_VOUCHERS_FILE, vouchers)?;
        emitted_files.push(GENESIS_VOUCHERS_FILE.to_string());
        logger.info(&format!("Exported {} vouchers", vouchers.len()));
    }

    emitted_files.push(GENESIS_GENERATION_RECEIPT_FILE.to_string());
    let receipt = GenesisGenerationReceipt {
        selection: plan.selection.clone(),
        output_kind: plan.export_kind,
        network: ctx.network_type.as_str().to_string(),
        emitted_files,
        asset_count: outputs
            .assets
            .as_ref()
            .map_or(0, GenesisSettlementCorpus::total_count),
        policy_count: outputs.policies.as_ref().map_or(0, Vec::len),
        right_count: outputs.rights.as_ref().map_or(0, Vec::len),
        voucher_count: outputs.vouchers.as_ref().map_or(0, Vec::len),
        full_manifest_file: None,
    };
    save_lane_json(output_dir, GENESIS_GENERATION_RECEIPT_FILE, &receipt)?;
    logger.info(&format!(
        "Exported generation receipt to {}",
        output_dir.join(GENESIS_GENERATION_RECEIPT_FILE).display()
    ));
    Ok(receipt)
}

/// Main genesis orchestration function.
pub fn run_genesis(config_path: &str, cli_command: Option<&str>) -> Result<(), GenesisError> {
    let plan = GenesisGenerationPlan::full_bootstrap();
    run_genesis_with_plan(config_path, &plan, cli_command).map(|_| ())
}

pub fn run_genesis_with_plan(
    config_path: &str,
    plan: &GenesisGenerationPlan,
    cli_command: Option<&str>,
) -> Result<GenesisGenerationReceipt, GenesisError> {
    let time_provider = SystemTimeProvider;
    let total_start_ms = time_provider.compat_unix_timestamp_millis();
    let stdout_logger: Arc<dyn Logger> = Arc::new(StdoutLogger);

    let ctx = load_genesis_context(config_path, plan)?;
    prepare_genesis_logging_dir(&ctx.config.outputs.logging_path)?;
    if plan.is_full_bootstrap()
        && ctx.config.outputs.snapshot_export_path != ctx.config.outputs.assets_export_path
    {
        prepare_genesis_snapshot_root(&ctx.config.outputs.snapshot_export_path)?;
    }
    let output_dir =
        create_timestamped_output_dir(&ctx.config.outputs.assets_export_path, ctx.network_type)?;
    let host_parallelism = std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1);
    let genesis_pool = build_genesis_thread_pool(&ctx.config.performance.num_threads)?;
    let genesis_pool_threads = genesis_pool.install(rayon::current_num_threads);

    stdout_logger.info("Generating genesis assets (nested parallel)...");
    stdout_logger.info(&format!("Output: {}", output_dir.display()));
    stdout_logger.info(&format!("Network: {}", ctx.network_type.as_str()));
    stdout_logger.info(&format!("Asset types: {}", ctx.config.assets.len()));
    stdout_logger.info(&format!("Right templates: {}", ctx.config.rights.len()));
    stdout_logger.info(&format!("Policy templates: {}", ctx.config.policies.len()));
    stdout_logger.info(&format!("Voucher templates: {}", ctx.config.vouchers.len()));
    stdout_logger.info(&format!("Requested lanes: {:?}", plan.requested_lanes()));
    stdout_logger.info(&format!("Host parallelism: {}", host_parallelism));
    stdout_logger.info(&format!(
        "Genesis thread config: {}",
        ctx.config.performance.num_threads.configured_label()
    ));
    stdout_logger.info(&format!(
        "Genesis thread pool threads: {}",
        genesis_pool_threads
    ));
    let gen_start_ms = time_provider.compat_unix_timestamp_millis();

    use z00z_utils::prelude::{FileLogger, NoopMetrics};
    use z00z_utils::time::format_unix_timestamp_milliseconds_compact;

    let log_filename = format!(
        "genesis_generation_{}.log",
        format_unix_timestamp_milliseconds_compact(time_provider.compat_unix_timestamp_millis())
    );
    let log_path = Path::new(&ctx.config.outputs.logging_path).join(log_filename);
    let logger: Arc<dyn Logger> =
        Arc::new(
            FileLogger::new(&log_path).map_err(|e| GenesisError::FileWriteFailed {
                path: log_path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?,
        );
    let metrics: Arc<dyn MetricsSink> = Arc::new(NoopMetrics);

    logger.info(&format!("Logging to: {}", log_path.display()));
    logger.info(&format!(
        "Genesis generation started - Network: {}",
        ctx.network_type.as_str()
    ));
    logger.info(&format!("Asset types: {}", ctx.config.assets.len()));
    logger.info(&format!("Right templates: {}", ctx.config.rights.len()));
    logger.info(&format!("Policy templates: {}", ctx.config.policies.len()));
    logger.info(&format!("Voucher templates: {}", ctx.config.vouchers.len()));
    logger.info(&format!("Requested lanes: {:?}", plan.requested_lanes()));
    logger.info(&format!("Host parallelism: {}", host_parallelism));
    logger.info(&format!(
        "Genesis thread config: {}",
        ctx.config.performance.num_threads.configured_label()
    ));
    logger.info(&format!(
        "Genesis thread pool threads: {}",
        genesis_pool_threads
    ));

    let outputs = genesis_pool
        .install(|| generate_genesis_lanes(&ctx, plan, logger.clone(), metrics.clone()))?;
    let gen_duration_ms = time_provider
        .compat_unix_timestamp_millis()
        .saturating_sub(gen_start_ms)
        .max(1);

    let accumulator = outputs.combined_corpus();
    logger.info(&format!(
        "Generated {} total assets and {} rights in {:.2}s",
        accumulator.total_count(),
        accumulator.total_right_count(),
        (gen_duration_ms as f64) / 1000.0,
    ));
    logger.info(&format!(
        "Asset breakdown: Coins={}, Tokens={}, NFTs={}, Voids={}",
        accumulator.coins.len(),
        accumulator.tokens.len(),
        accumulator.nfts.len(),
        accumulator.voids.len()
    ));
    logger.info(&format!("Rights: {}", accumulator.total_right_count()));
    logger.info(&format!("Vouchers: {}", accumulator.total_voucher_count()));
    logger.info(&format!(
        "Settlement leaves: {}",
        accumulator.total_leaf_count()
    ));

    logger.info("Phase 3: Verifying cryptographic proofs...");
    let verify_start_ms = time_provider.compat_unix_timestamp_millis();
    let all_assets_flat = outputs
        .assets
        .as_ref()
        .map(GenesisSettlementCorpus::flatten)
        .unwrap_or_default();
    if !all_assets_flat.is_empty() {
        logger.info("Starting cryptographic proof verification...");
        verify_genesis_assets(&all_assets_flat)?;
    }
    let verify_duration_ms = time_provider
        .compat_unix_timestamp_millis()
        .saturating_sub(verify_start_ms)
        .max(1);
    if !all_assets_flat.is_empty() {
        logger.info(&format!(
            "All {} proofs verified successfully in {:.2}s",
            all_assets_flat.len(),
            (verify_duration_ms as f64) / 1000.0
        ));
    }

    logger.info("Phase 4: Exporting to disk...");
    logger.info(&format!("Exporting assets to: {}", output_dir.display()));

    let mut full_state_hash = None;
    let receipt = if plan.is_full_bootstrap() {
        let definitions = outputs.asset_definitions.as_ref().ok_or_else(|| {
            GenesisError::InvalidConfig(
                "full_bootstrap plan must emit asset definitions".to_string(),
            )
        })?;
        let owner_registry = build_owner_registry(definitions)?;
        logger.info(&format!(
            "Built explicit asset registry owner with {} definitions",
            definitions.len()
        ));
        owner_registry
            .sync_global_fallback()
            .map_err(|err| GenesisError::RegistryInsertFailed(err.to_string()))?;
        logger.info("Synced explicit registry owner into GLOBAL_ASSET_REGISTRY fallback");

        logger.info("Phase 3.5: Computing genesis state hash...");
        let state_hash = compute_genesis_state_hash(&accumulator);
        full_state_hash = Some(state_hash);
        logger.info(&format!("Genesis state hash: {}", hex::encode(state_hash)));
        logger.warn("IMPORTANT: Hardcode this hash in consensus parameters for mainnet/testnet!");
        verify_genesis_consensus(ctx.network_type, &state_hash)?;

        let mut emitted_files = Vec::new();
        export_selected_assets(
            &output_dir,
            &ctx.config,
            definitions,
            outputs.assets.as_ref().ok_or_else(|| {
                GenesisError::InvalidConfig(
                    "full_bootstrap plan must emit genesis assets".to_string(),
                )
            })?,
            &mut emitted_files,
            logger.as_ref(),
        )?;
        emitted_files.push(GENESIS_POLICIES_FILE.to_string());
        emitted_files.push(GENESIS_RIGHTS_FILE.to_string());
        emitted_files.push(GENESIS_VOUCHERS_FILE.to_string());
        emitted_files.push(GENESIS_SETTLEMENT_MANIFEST_FILE.to_string());

        let (rights_path, manifest_path) = export_genesis_settlement_artifacts(
            &output_dir,
            definitions,
            &ctx.policies,
            &accumulator,
            ctx.network_type,
            GENESIS_ROOT_GENERATION,
            &state_hash,
            ctx.seed.as_bytes(),
        )?;
        logger.info(&format!(
            "Exported {} rights to {}",
            accumulator.total_right_count(),
            rights_path.display()
        ));
        logger.info(&format!(
            "Exported settlement manifest to {}",
            manifest_path.display()
        ));

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(cmd) = cli_command {
            create_genesis_snapshot_zip(
                Path::new(&ctx.config.outputs.snapshot_export_path),
                &output_dir,
                ctx.network_type,
                config_path,
                cmd,
            )?;
        }

        GenesisGenerationReceipt {
            selection: plan.selection.clone(),
            output_kind: plan.export_kind,
            network: ctx.network_type.as_str().to_string(),
            emitted_files,
            asset_count: accumulator.total_count(),
            policy_count: ctx.policies.len(),
            right_count: accumulator.total_right_count(),
            voucher_count: accumulator.total_voucher_count(),
            full_manifest_file: Some(GENESIS_SETTLEMENT_MANIFEST_FILE.to_string()),
        }
    } else {
        export_partial_receipt(&output_dir, &ctx, plan, &outputs, logger.as_ref())?
    };

    let total_duration_ms = time_provider
        .compat_unix_timestamp_millis()
        .saturating_sub(total_start_ms)
        .max(1);
    let total_duration_secs = (total_duration_ms / 1000).max(1);

    logger.info(&format!(
        "Genesis generation complete! Total time: {:.2}s",
        (total_duration_ms as f64) / 1000.0
    ));
    logger.info(&format!("Output directory: {}", output_dir.display()));
    logger.info("Genesis generation complete!");
    logger.info(&format!(
        "Total time: {:.2}s",
        (total_duration_ms as f64) / 1000.0
    ));
    logger.info(&format!("Output: {}", output_dir.display()));
    logger.info(&format!("Log file: {}", log_path.display()));

    if plan.is_full_bootstrap() {
        let definitions = outputs.asset_definitions.as_ref().ok_or_else(|| {
            GenesisError::InvalidConfig(
                "full_bootstrap plan must emit asset definitions".to_string(),
            )
        })?;
        let state_hash = full_state_hash.ok_or_else(|| {
            GenesisError::InvalidConfig(
                "full_bootstrap plan must compute genesis state hash".to_string(),
            )
        })?;
        write_genesis_report(
            &output_dir,
            definitions,
            &accumulator,
            GenesisReportArgs {
                network_type: ctx.network_type,
                gen_duration_secs: (gen_duration_ms / 1000).max(1),
                verify_duration_secs: (verify_duration_ms / 1000).max(1),
                total_duration_secs,
                state_hash: &state_hash,
                cli_command,
            },
        )?;
        let report_timestamp = generate_timestamp();
        logger.info(&format!(
            "Report: {}/genesis_report_{}.txt",
            output_dir.display(),
            report_timestamp
        ));
    }

    Ok(receipt)
}
