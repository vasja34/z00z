//! Shared scenario config types and loader.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_utils::codec::CodecError;

mod config_accessors;
mod config_defaults;

/// Scenario metadata block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioMeta {
    /// Scenario numeric id.
    pub id: u32,
    /// Scenario display name.
    pub name: String,
    /// Scenario description.
    pub description: String,
}

/// Simulation runtime config block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimCfg {
    /// Stage-2 simulator reproducibility toggle.
    ///
    /// It does not establish one repo-wide randomness selector across every
    /// simulator stage. The remaining work is a consolidation pass over live abstractions rather than a brand-new design.
    pub use_mock_rng: bool,
    /// Stage-2 mock-lane seed.
    ///
    /// When `use_mock_rng` is false, the stage-2 transport stays on the normal
    /// system-random path.
    /// When `use_mock_rng` is true, `Some(seed)` selects deterministic stage-2
    /// reproducibility and `None` keeps the deterministic zero-seed fallback
    /// instead of claiming secure randomness.
    #[serde(default)]
    pub mock_rng_seed: Option<u64>,
    /// Abort run on first fail.
    pub abort_on_fail: bool,
}

/// Genesis asset config item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetCfg {
    /// Name.
    pub name: String,
    /// Symbol.
    pub symbol: String,
    /// Asset class.
    pub class: String,
    /// Dev config key used by `def_from_dev_cfg`.
    pub dev_cfg_key: String,
    /// Max serial count for this asset.
    pub serials: u64,
    /// Nominal units.
    pub nominal: u64,
    /// Asset index in scenario list.
    pub asset_index: usize,
}

/// Output file config block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputCfg {
    /// Base output directory.
    pub dir: String,
}

/// Phase-local HJMT runtime config root reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HjmtCfgRef {
    /// Canonical runtime profile name.
    pub profile: String,
    /// Canonical config-root path for the runtime plane.
    pub config_root: String,
}

/// Runtime observability profile definition for the runtime trace packet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfileCfg {
    /// Stable profile id.
    pub id: String,
    /// Whether repeated runs on the same config must stay bit-stable.
    pub deterministic: bool,
    /// Short human explanation.
    pub purpose: String,
}

/// Runtime trace-pack file layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeTraceCfg {
    pub cfg_flow_file: String,
    pub tx_flow_file: String,
    pub route_flow_file: String,
    pub plan_flow_file: String,
    pub journal_flow_file: String,
    pub scope_flow_file: String,
    pub proc_flow_file: String,
    pub recovery_flow_file: String,
    pub leaf_flow_file: String,
    pub proof_flow_file: String,
    pub pub_flow_file: String,
    pub val_flow_file: String,
    pub watch_flow_file: String,
}

/// Release-packet files that stay on the canonical public simulator lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimePacketCfg {
    pub run_meta_file: String,
    pub wallet_scan_file: String,
    pub sim_summary_file: String,
    #[serde(default)]
    pub emitted_public_files: Vec<String>,
}

/// Config-driven positive topology example that the publication lane must
/// accept without code edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationTopologyStageCfg {
    pub stage_id: String,
    pub topology: String,
    pub aggregator_count: usize,
    pub shard_count: usize,
    pub route_generation: u64,
    pub owner_aggregator_id: u16,
    pub standby_aggregator_ids: Vec<u16>,
}

/// Config-driven positive topology example that the publication lane must
/// accept without code edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationTopologyCfg {
    pub fixture_id: String,
    pub shard_id: u16,
    pub old_topology: String,
    pub new_topology: String,
    pub old_aggregator_count: usize,
    pub old_shard_count: usize,
    pub new_aggregator_count: usize,
    pub new_shard_count: usize,
    pub route_generation_from: u64,
    pub route_generation_to: u64,
    pub owner_aggregator_id: u16,
    pub standby_aggregator_ids: Vec<u16>,
    pub join_mode: String,
    pub transfer_target: String,
    pub activation_checkpoint: u64,
    #[serde(default)]
    pub transition_stages: Vec<PublicationTopologyStageCfg>,
    #[serde(default)]
    pub removed_aggregator_ids: Vec<u16>,
    #[serde(default)]
    pub removed_aggregator_absent_from_owner_tables: bool,
    #[serde(default)]
    pub removed_aggregator_absent_from_standby_tables: bool,
    #[serde(default)]
    pub all_shards_owned_across_stages: bool,
    #[serde(default)]
    pub prior_lineage_preserved: bool,
    #[serde(default)]
    pub publication_continuity_preserved: bool,
}

/// Publication observability contract for the Phase 057 publication packet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationObservabilityCfg {
    pub acceptance_profile: String,
    pub inherited_runtime_profile: String,
    pub topology_status: String,
    pub public_leaf_count: usize,
    pub publication_activation_checkpoint: u64,
    pub positive_topology_examples: Vec<PublicationTopologyCfg>,
}

/// Executable runtime-observability contract for `scenario_1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeObservabilityCfg {
    /// Active execution profile id for the current run.
    pub active_profile: String,
    /// Profiles that are valid correctness or integration lanes in this phase.
    pub supported_profiles: Vec<RuntimeProfileCfg>,
    /// Profiles that are live but benchmark-heavy and must stay out of the
    /// default deterministic correctness lane.
    #[serde(default)]
    pub heavy_only_profiles: Vec<String>,
    /// Publication acceptance contract for Phase 057.
    pub publication: PublicationObservabilityCfg,
    /// Required trace-pack paths.
    pub traces: RuntimeTraceCfg,
    /// Additional public release-packet files.
    pub packet: RuntimePacketCfg,
}

/// Stage 3 claim/distribution config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage3ClaimCfg {
    /// Active distribution mode: class_split | coin_sets | uniform_all.
    pub active: Option<String>,
    /// Deterministic RNG seed used for distribution.
    pub rng_seed: Option<u64>,
    /// When true, stage 3 consumes genesis_*.bin inputs after claim.
    pub consume_bins: Option<bool>,
    /// Optional snapshot fault mode for runtime tests: count_mismatch | mid_abort.
    pub snapshot_fault: Option<String>,
    /// Optional resume fault mode for runtime tests: half_abort | reject_first | replay_first.
    pub resume_fault: Option<String>,
    /// Stage 3 local paths.
    #[serde(default)]
    pub paths: Stage3PathsCfg,
}

/// Stage 1 genesis config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Stage1GenesisCfg {
    /// Stage 1 local genesis config path.
    #[serde(default)]
    pub genesis_config: String,
    /// Stage 1 local paths.
    #[serde(default)]
    pub paths: Stage1PathsCfg,
}

/// Stage 2 actor config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2ActorCfg {
    pub name: String,
    pub password: String,
    pub mock_rng_seed: u64,
}

/// Stage 2 wallet creation config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2WalletCfg {
    pub wallet_chain: String,
    pub actors: Vec<Stage2ActorCfg>,
    #[serde(default)]
    pub paths: Stage2PathsCfg,
}

/// Stage 1 paths and file names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage1PathsCfg {
    pub genesis_dir: String,
    pub logs_dir: String,
    pub snapshot_file: String,
    pub state_hash_file: String,
    pub logger_file: String,
    pub fallback_genesis_dir: String,
    pub cli_target_dir: String,
}

/// Stage 2 paths and file names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage2PathsCfg {
    pub wallets_dir: String,
    pub keys_dir: String,
    pub logs_dir: String,
    pub snapshot_file: String,
    pub logger_file: String,
    pub rpc_logger_file: String,
}

/// Stage 3 paths and file names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage3PathsCfg {
    pub genesis_dir: String,
    pub claim_dir: String,
    pub wallets_dir: String,
    pub events_dir: String,
    pub logs_dir: String,
    pub export_dir: String,
    pub snapshot_file: String,
    pub claim_state_file: String,
    pub logger_file: String,
    pub rpc_logger_file: String,
}

/// Stage 5 paths and file names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage5PathsCfg {
    pub logs_dir: String,
    pub transactions_dir: String,
    pub tx_file: String,
    pub snapshot_file: String,
    pub logger_file: String,
}

/// Stage 6 paths and file names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage6PathsCfg {
    pub logs_dir: String,
    pub transactions_dir: String,
    pub frag1_file: String,
    pub frag2_file: String,
    pub checkpoint_file: String,
    pub report_file: String,
    pub logger_file: String,
}

/// Stage 7 paths and file names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage7PathsCfg {
    pub logs_dir: String,
    pub transactions_dir: String,
    pub checkpoint_file: String,
    pub logger_file: String,
}

/// Stage 8 paths and file names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage8PathsCfg {
    pub logs_dir: String,
    pub transactions_dir: String,
    pub checkpoint_file: String,
    pub logger_file: String,
}

/// Stage 6 checkpoint proof mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Stage6ProofMode {
    DraftOnly,
    #[default]
    OpaqueTest,
}

/// Private-only stage-12 summary class for draft checkpoint runs.
pub const STAGE12_PRIVATE_DRAFT_EVIDENCE_CLASS: &str = "draft_private_checkpoint_summary_v1";

/// Public stage-12 summary class for finalized checkpoint runs.
pub const STAGE12_FINAL_PUBLIC_EVIDENCE_CLASS: &str = "final_public_checkpoint_summary_v1";

impl Stage6ProofMode {
    /// Returns the stage-12 checkpoint summary contract for the current proof
    /// lane.
    pub const fn stage12_evidence_class(self) -> &'static str {
        match self {
            Self::DraftOnly => STAGE12_PRIVATE_DRAFT_EVIDENCE_CLASS,
            Self::OpaqueTest => STAGE12_FINAL_PUBLIC_EVIDENCE_CLASS,
        }
    }

    /// Returns whether this lane is allowed to feed public checkpoint and
    /// publication evidence.
    pub const fn allows_public_checkpoint_evidence(self) -> bool {
        matches!(self, Self::OpaqueTest)
    }
}

/// Stage 4 path configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage4PathsCfg {
    pub outputs_dir: String,
    pub logs_dir: String,
    pub transactions_dir: String,
    pub wallets_dir: String,
    pub tx_pkg_file: String,
    pub snapshot_file: String,
    pub logger_file: String,
    pub rpc_logger_file: String,
    pub alice_keys_file: String,
    pub bob_keys_file: String,
    #[serde(default)]
    pub wallets_state_before_file: Option<String>,
    #[serde(default)]
    pub wallets_state_after_file: Option<String>,
    #[serde(default)]
    pub wallets_state_diff_file: Option<String>,
    #[serde(default)]
    pub wallets_state_report_md_file: Option<String>,
    #[serde(default)]
    pub wallets_state_report_xlsx_file: Option<String>,
}

/// Stage 4 input selection options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage4SelectionCfg {
    pub distinct_serial_ids_min: u32,
    pub distinct_serial_ids_target: u32,
    pub distinct_serial_ids_max: u32,
}

/// Stage 4 output planning options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage4OutputsCfg {
    #[serde(default = "config_defaults::default_bob_outputs_count")]
    pub bob_outputs_count: u32,
}

/// Stage 4 fee recipient configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage4FeeSinkCfg {
    #[serde(default = "config_defaults::default_fee_wallet_id")]
    pub wallet_id: String,
    #[serde(default)]
    pub receiver_card_hex: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub rng_seed: Option<u64>,
}

/// Stage 4 transaction options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stage4TransactionCfg {
    pub class: String,
    pub symbol: String,
    pub mode: String,
    #[serde(default)]
    pub input_assets_selection: Stage4SelectionCfg,
    #[serde(default)]
    pub outputs: Stage4OutputsCfg,
    #[serde(default)]
    pub fee_sink: Stage4FeeSinkCfg,
    #[serde(default)]
    pub amount: Option<u64>,
    #[serde(default)]
    pub fraction: Option<f64>,
}

/// Stage 4 list-assets filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage4ListFilterCfg {
    pub asset_class: Option<String>,
    pub min_balance: Option<u64>,
}

/// Stage 4 RPC options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage4RpcCfg {
    pub transport: String,
    pub unlock_method: String,
    pub lock_method: String,
    pub list_assets_method: String,
    pub import_asset_method: String,
    #[serde(default = "config_defaults::default_build_transaction_method")]
    pub build_transaction_method: String,
    pub list_limit: usize,
    pub list_filter: Stage4ListFilterCfg,
}

/// Stage 4 root configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stage4TxPrepareCfg {
    pub enabled: bool,
    pub sender_actor: String,
    pub receiver_actor: String,
    pub paths: Stage4PathsCfg,
    pub transaction: Stage4TransactionCfg,
    pub rpc: Stage4RpcCfg,
}

/// Stage 5 transfer config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Stage5TransferCfg {
    #[serde(default = "config_defaults::stage5_recipient_output_index")]
    pub recipient_output_index: usize,
    #[serde(default)]
    pub paths: Stage5PathsCfg,
}

/// Stage 6 bundle config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Stage6BundleCfg {
    #[serde(default)]
    pub paths: Stage6PathsCfg,
    #[serde(default)]
    pub proof_mode: Stage6ProofMode,
}

/// Stage 7 apply config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Stage7ApplyCfg {
    #[serde(default)]
    pub paths: Stage7PathsCfg,
}

/// Stage 8 finalize config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Stage8FinalizeCfg {
    #[serde(default)]
    pub paths: Stage8PathsCfg,
}

/// Stage 13 HJMT settlement examples config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage13HjmtCfg {
    pub enabled: bool,
    pub backend_modes: Vec<String>,
    pub rights_manifest_file: String,
    pub output_dir: String,
    pub examples_file: String,
    pub tamper_report_file: String,
    pub proof_size_report_file: String,
    pub cache_scheduler_metrics_file: String,
    pub replay_roots_file: String,
    pub expected_right_classes: Vec<String>,
}

/// Scenario 1 object-flow contract row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ObjectFlowCaseCfg {
    pub id: String,
    pub family: String,
    pub action: String,
    pub policy_label: String,
    pub lane: String,
    pub actors: Vec<String>,
    #[serde(default)]
    pub required_rights: Vec<String>,
    pub expected_verdict: String,
    pub evidence_files: Vec<String>,
}

/// Scenario 1 explicit object-flow matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ObjectFlowMatrixCfg {
    #[serde(default)]
    pub positive: Vec<ObjectFlowCaseCfg>,
    #[serde(default)]
    pub negative: Vec<ObjectFlowCaseCfg>,
}

/// Root scenario config model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioCfg {
    /// Scenario metadata.
    pub scenario: ScenarioMeta,
    /// Chain name: devnet/testnet/mainnet.
    pub chain: String,
    /// Phase-local HJMT runtime config root.
    #[serde(default)]
    pub hjmt_runtime: Option<HjmtCfgRef>,
    /// Executable runtime-observability surface for Phase 056.
    #[serde(default)]
    pub runtime_observability: Option<RuntimeObservabilityCfg>,
    /// Stage 1 genesis config.
    #[serde(default)]
    pub stage1_genesis: Option<Stage1GenesisCfg>,
    /// Runtime simulation config.
    pub simulation: SimCfg,
    /// Genesis assets carried by the current scenario config.
    #[serde(default)]
    pub genesis_assets: Vec<AssetCfg>,
    /// Stage 3 claim/distribution config.
    pub stage3_claim: Option<Stage3ClaimCfg>,
    /// Stage 4 claim publication config.
    #[serde(default)]
    pub stage4_claim_publish: Option<Stage3ClaimCfg>,
    /// Stage 2 wallet creation config.
    #[serde(default)]
    pub stage2_wallet_create: Option<Stage2WalletCfg>,
    /// Stage 4 tx prepare config.
    #[serde(default)]
    pub stage4_tx_prepare: Option<Stage4TxPrepareCfg>,
    /// Stage 5 transfer config.
    #[serde(default)]
    pub stage5_transfer: Option<Stage5TransferCfg>,
    /// Stage 6 bundle config.
    #[serde(default)]
    pub stage6_bundle: Option<Stage6BundleCfg>,
    /// Stage 7 apply config.
    #[serde(default)]
    pub stage7_apply: Option<Stage7ApplyCfg>,
    /// Stage 8 finalize config.
    #[serde(default)]
    pub stage8_finalize: Option<Stage8FinalizeCfg>,
    /// Stage 13 HJMT settlement examples config.
    #[serde(default)]
    pub stage13_hjmt_settlement_examples: Option<Stage13HjmtCfg>,
    /// Explicit object-family flow coverage contract for Phase 059.
    #[serde(default)]
    pub object_flow_matrix: Option<ObjectFlowMatrixCfg>,
    /// Output settings.
    pub outputs: OutputCfg,
}

/// Config loading errors.
#[derive(Debug, Error)]
pub enum ScenarioCfgErr {
    /// I/O read failed.
    #[error("failed to read scenario config: {0}")]
    Io(#[from] z00z_utils::io::IoError),
    /// YAML decode failed.
    #[error("failed to decode scenario config yaml: {0}")]
    Decode(#[from] CodecError),
}
