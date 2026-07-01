//! Shared simulator state across stage execution.

use std::{path::PathBuf, sync::Arc};

use z00z_core::{genesis::GenesisRightRecord, Asset, AssetDefinitionRegistry, ChainType};
use z00z_storage::settlement::TerminalLeaf;
use z00z_utils::logger::Logger;
use z00z_wallets::WalletService;

use crate::{ScenarioCfg, SimActor};

/// Shared mutable scenario context.
pub struct SimContext {
    /// Loaded scenario config.
    pub config: ScenarioCfg,
    /// Active chain type.
    pub chain_type: ChainType,
    /// Asset definition registry loaded from genesis.
    pub registry: AssetDefinitionRegistry,
    /// Minted assets for current run.
    pub assets: Vec<Asset>,
    /// Deterministic genesis rights carried into settlement storage.
    pub genesis_rights: Vec<GenesisRightRecord>,
    /// Scenario actors.
    pub actors: Vec<SimActor>,
    /// Produced terminal leaves.
    pub leaves: Vec<TerminalLeaf>,
    /// Current block height in simulation timeline.
    pub block_height: u64,
    /// Output base directory.
    pub outputs_dir: PathBuf,
    /// Injected logger for structured output.
    pub logger: Arc<dyn Logger>,
    /// Shared wallet service for cross-stage RPC reuse.
    pub wallet_service: Option<Arc<WalletService>>,
}
