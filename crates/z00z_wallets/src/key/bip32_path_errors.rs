// ============================================================================
// Z00Z BIP-44 ERROR CONTRACT
// ============================================================================
// Machine-readable error reasons for non-standard BIP-44 path violations.
// These are stable and must not change to maintain RPC compatibility.

/// Machine-readable error reasons for BIP-44 path validation failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bip44ViolationReason {
    /// Purpose must be hardened (44')
    PurposeNotHardened,
    /// Purpose value must be 44
    PurposeValueMismatch,
    /// Asset type must be hardened
    AssetTypeNotHardened,
    /// Asset type value must match Z00Z coin type
    AssetTypeValueMismatch,
    /// Account must be hardened
    AccountNotHardened,
    /// Change must be non-hardened
    ChangeIsHardened,
    /// Change value must be 0 or 1
    InvalidChangeValue,
    /// Address index must be non-hardened
    AddressIndexIsHardened,
    /// Path component out of valid range
    IndexOutOfRange,
}

impl Display for Bip44ViolationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bip44ViolationReason::PurposeNotHardened => write!(f, "purpose_not_hardened"),
            Bip44ViolationReason::PurposeValueMismatch => write!(f, "purpose_value_mismatch"),
            Bip44ViolationReason::AssetTypeNotHardened => write!(f, "asset_type_not_hardened"),
            Bip44ViolationReason::AssetTypeValueMismatch => write!(f, "asset_type_value_mismatch"),
            Bip44ViolationReason::AccountNotHardened => write!(f, "account_not_hardened"),
            Bip44ViolationReason::ChangeIsHardened => write!(f, "change_is_hardened"),
            Bip44ViolationReason::InvalidChangeValue => write!(f, "invalid_change_value"),
            Bip44ViolationReason::AddressIndexIsHardened => write!(f, "address_index_is_hardened"),
            Bip44ViolationReason::IndexOutOfRange => write!(f, "index_out_of_range"),
        }
    }
}

/// BIP-44 derivation error types
#[derive(Debug, Clone, Error)]
pub enum Bip44Error {
    /// Purpose must be hardened (44')
    #[error("purpose must be hardened (44'), got {0}")]
    PurposeNotHardened(u32),
    /// Asset type must be hardened
    #[error("asset_type must be hardened, got {0}")]
    AssetTypeNotHardened(u32),
    /// Account must be hardened
    #[error("account must be hardened, got {0}")]
    AccountNotHardened(u32),
    /// Change must be non-hardened (0 or 1)
    #[error("change must be non-hardened, got {0}")]
    ChangeIsHardened(u32),
    /// Address index must be non-hardened
    #[error("address_index must be non-hardened, got {0}")]
    AddressIndexIsHardened(u32),
    /// Change must be 0 or 1
    #[error("change must be 0 or 1, got {0}")]
    InvalidChangeValue(u32),
    /// Invalid derivation path format
    #[error("invalid derivation path: {0}")]
    InvalidPath(String),
    /// BIP-32 operation failed
    #[error("bip32 error: {0}")]
    Bip32(#[from] bip32::Error),
    /// Seed length must be 32-64 bytes
    #[error("invalid seed length: {0}")]
    InvalidSeed(usize),
    /// Seed has weak entropy
    #[error("weak entropy: {0}")]
    WeakEntropy(String),
    /// Non-standard BIP-44 path violation
    #[error("non-standard BIP-44 path: {reason}")]
    NonStandardPath {
        /// The specific violation reason
        reason: Bip44ViolationReason,
        /// The component that failed validation
        component: String,
    },
    /// Index exceeds BIP-32 non-hardened limit (2^31 - 1)
    #[error("Index {field}={value} exceeds BIP-32 limit (max {max})")]
    IndexOutOfRange {
        /// Field name that exceeded the limit
        field: &'static str,
        /// Actual value that was out of range
        value: u32,
        /// Maximum allowed value
        max: u32,
    },
}
