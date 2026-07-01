use super::Error;

/// Genesis generation and validation errors.
#[derive(Debug, Error)]
pub enum GenesisError {
    #[error("config load failed: {0}")]
    ConfigLoadFailed(String),

    #[error("config parse failed: {0}")]
    ConfigParseFailed(String),

    #[error("invalid config: {0}")]
    InvalidConfig(String),

    #[error("insecure genesis seed: {0}")]
    InsecureGenesisSeed(String),

    #[error("missing protected genesis anchor for {network}")]
    MissingGenesisAnchor { network: String },

    #[error("test seed not allowed in production")]
    TestSeedInProduction,

    #[error(
        "blinding derivation failed for definition {definition_id:?} serial {serial_id}: {error}"
    )]
    BlindingDerivationFailed {
        definition_id: [u8; 32],
        serial_id: u32,
        error: String,
    },

    #[error("nonce derivation failed for serial {serial_id}: {error}")]
    NonceDerivationFailed { serial_id: u32, error: String },

    #[error("asset creation failed for definition {definition_id:?} serial {serial_id}: {error}")]
    AssetCreationFailed {
        definition_id: [u8; 32],
        serial_id: u32,
        error: String,
    },

    #[error("right derivation failed for {right_id} index {right_index}: {error}")]
    RightDerivationFailed {
        right_id: String,
        right_index: u32,
        error: String,
    },

    #[error(
        "proof generation failed for definition {definition_id:?} serial {serial_id}: {error}"
    )]
    ProofGenerationFailed {
        definition_id: [u8; 32],
        serial_id: u32,
        error: String,
    },

    #[error("proof verification failed for asset {asset_id:?} serial {serial_id}: {error}")]
    ProofVerificationFailed {
        asset_id: [u8; 32],
        serial_id: u32,
        error: String,
    },

    #[error("serialization failed: {0}")]
    SerializationFailed(String),

    #[error("file write failed for {path}: {error}")]
    FileWriteFailed { path: String, error: String },

    #[error("genesis thread pool build failed: {0}")]
    ThreadPoolBuildFailed(String),

    #[error("registry insert failed: {0}")]
    RegistryInsertFailed(String),

    #[error("definition not found: {0:?}")]
    DefinitionNotFound([u8; 32]),

    #[error("genesis state mismatch for {network}: expected {expected:?}, computed {computed:?}")]
    GenesisStateMismatch {
        expected: [u8; 32],
        computed: [u8; 32],
        network: String,
    },

    #[error("terminal collision detected for {terminal_id:?}: {error}")]
    TerminalCollision {
        terminal_id: [u8; 32],
        error: String,
    },

    #[error("asset error: {0}")]
    AssetError(#[from] crate::AssetError),
}
