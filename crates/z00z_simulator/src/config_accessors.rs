use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io,
};

use super::{
    ScenarioCfg, ScenarioCfgErr, Stage1PathsCfg, Stage2PathsCfg, Stage3PathsCfg, Stage5PathsCfg,
    Stage6PathsCfg, Stage6ProofMode, Stage7PathsCfg, Stage8PathsCfg,
};

impl ScenarioCfg {
    /// Returns the live Phase-056 HJMT runtime reference when configured.
    pub fn hjmt_runtime_ref(&self) -> Option<&super::HjmtCfgRef> {
        self.hjmt_runtime.as_ref()
    }

    /// Returns the runtime-observability contract when configured.
    pub fn runtime_observability_ref(&self) -> Option<&super::RuntimeObservabilityCfg> {
        self.runtime_observability.as_ref()
    }

    /// Returns the Phase-057 publication observability contract when configured.
    pub fn publication_observability_ref(&self) -> Option<&super::PublicationObservabilityCfg> {
        self.runtime_observability_ref()
            .map(|observability| &observability.publication)
    }

    /// Returns the configured HJMT runtime config root path when present.
    pub fn hjmt_config_root(&self) -> Option<PathBuf> {
        self.hjmt_runtime_ref()
            .map(|runtime| PathBuf::from(&runtime.config_root))
            .filter(|path| !path.as_os_str().is_empty())
    }

    /// Returns stage 1 config path.
    /// This accessor layer stays a consolidation pass over live abstractions.
    pub fn stage1_genesis_config(&self) -> String {
        self.stage1_genesis
            .as_ref()
            .map(|stage| stage.genesis_config.clone())
            .filter(|path| !path.is_empty())
            .unwrap_or_else(|| z00z_core::config_paths::DEVNET_GENESIS_CONFIG.to_string())
    }

    /// Returns stage 1 paths.
    pub fn stage1_paths(&self) -> Stage1PathsCfg {
        self.stage1_genesis
            .as_ref()
            .map(|stage| stage.paths.clone())
            .unwrap_or_default()
    }

    /// Returns stage 2 paths.
    pub fn stage2_paths(&self) -> Stage2PathsCfg {
        self.stage2_wallet_create
            .as_ref()
            .map(|stage| stage.paths.clone())
            .unwrap_or_default()
    }

    /// Returns whether the private stage 2 debug secret artifact is enabled.
    /// The remaining fix set is retention policy and wrapping discipline for
    /// the debug-only lane, not reopening the hardened default lane.
    pub const fn stage2_secret_artifact_enabled(&self) -> bool {
        cfg!(feature = "wallet_debug_tools")
    }

    /// Returns the private stage 2 debug secret artifact path when enabled.
    /// The path stays private so retention policy can remain scoped to the
    /// debug-only lane.
    pub fn stage2_secret_artifact_path(&self, wallets_dir: &Path) -> Option<PathBuf> {
        self.stage2_secret_artifact_enabled()
            .then(|| wallets_dir.join("private").join("wlt_secrets_debug.md"))
    }

    /// Returns stage 3 paths.
    pub fn stage3_paths(&self) -> Stage3PathsCfg {
        self.stage3_claim
            .as_ref()
            .map(|stage| stage.paths.clone())
            .unwrap_or_default()
    }

    /// Returns stage 4 claim publication paths.
    pub fn stage4_claim_paths(&self) -> Stage3PathsCfg {
        self.stage4_claim_publish
            .as_ref()
            .map(|stage| stage.paths.clone())
            .unwrap_or_else(|| Stage3PathsCfg {
                genesis_dir: "genesis".to_string(),
                claim_dir: "claim_publish".to_string(),
                wallets_dir: "wallets_publish".to_string(),
                events_dir: "events_publish".to_string(),
                logs_dir: "logs_publish".to_string(),
                export_dir: "wallets_publish_export".to_string(),
                snapshot_file: "stage_4_snapshot.json".to_string(),
                claim_state_file: "claim_publish_state.json".to_string(),
                logger_file: "claim_publish_logger.json".to_string(),
                rpc_logger_file: "rpc_publish_logger.json".to_string(),
            })
    }

    /// Returns stage 5 paths.
    pub fn stage5_paths(&self) -> Stage5PathsCfg {
        self.stage5_transfer
            .as_ref()
            .map(|stage| stage.paths.clone())
            .unwrap_or_default()
    }

    /// Returns stage 6 paths.
    pub fn stage6_paths(&self) -> Stage6PathsCfg {
        self.stage6_bundle
            .as_ref()
            .map(|stage| stage.paths.clone())
            .unwrap_or_default()
    }

    /// Returns stage 6 proof mode.
    pub fn stage6_proof_mode(&self) -> Stage6ProofMode {
        self.stage6_bundle
            .as_ref()
            .map(|stage| stage.proof_mode)
            .unwrap_or_default()
    }

    /// Returns stage 7 paths.
    pub fn stage7_paths(&self) -> Stage7PathsCfg {
        self.stage7_apply
            .as_ref()
            .map(|stage| stage.paths.clone())
            .unwrap_or_default()
    }

    /// Returns stage 8 paths.
    pub fn stage8_paths(&self) -> Stage8PathsCfg {
        self.stage8_finalize
            .as_ref()
            .map(|stage| stage.paths.clone())
            .unwrap_or_default()
    }

    /// Loads scenario config from YAML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ScenarioCfgErr> {
        let bytes = io::read_file(path.as_ref())?;
        let codec = YamlCodec;
        codec.deserialize(&bytes).map_err(ScenarioCfgErr::Decode)
    }

    /// Returns the SHA-256 digest of one scenario-facing config file.
    pub fn config_digest(path: impl AsRef<Path>) -> Result<String, ScenarioCfgErr> {
        let bytes = io::read_file(path.as_ref())?;
        Ok(hex::encode(Sha256::digest(&bytes)))
    }
}
