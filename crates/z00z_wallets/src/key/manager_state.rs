/// Named key branches used for wallet-specific contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Z00ZKeyBranch {
    /// Key branch for wallet backup encryption.
    WalletBackup = 1,
}

impl Z00ZKeyBranch {
    /// Numeric identifier for on-wire serialization.
    pub fn id(self) -> u8 {
        self as u8
    }

    /// Stable label used in domain separation.
    pub fn label(self) -> &'static str {
        match self {
            Self::WalletBackup => "z00z.wallet.backup",
        }
    }

    /// Single-byte AAD marker.
    pub fn as_aad_byte(self) -> u8 {
        self.id()
    }
}

// ============================================================================
// STATE TYPES FOR PERSISTENCE
// ============================================================================

/// Serializable state for key manager persistence.
///
/// This struct contains only non-sensitive data that can be safely serialized:
/// - Encrypted seed container (no raw seed bytes)
/// - Metadata for wallet recovery
///
/// # Security Guarantees
/// - Does NOT contain raw seed bytes
/// - Does NOT contain secret keys
/// - Does NOT contain RNG provider
/// - Uses encrypted seed container for secure storage
/// - Does NOT persist derivation cache (re-derived on demand)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyManagerState {
    /// Encrypted seed container bytes (binary format)
    pub encrypted_seed_bytes: Vec<u8>,
    /// Wallet metadata (e.g., creation time, label)
    pub metadata: KeyManagerMetadata,
    /// Chain identifier for Ristretto key separation (e.g., "devnet")
    pub chain: ChainType,
}

impl ConstantTimeEq for KeyManagerState {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.encrypted_seed_bytes
            .as_slice()
            .ct_eq(other.encrypted_seed_bytes.as_slice())
            & self.metadata.ct_eq(&other.metadata)
            & self
                .chain
                .as_str()
                .as_bytes()
                .ct_eq(other.chain.as_str().as_bytes())
    }
}

/// Metadata for wallet recovery and identification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyManagerMetadata {
    /// Wallet identifier
    pub wallet_id: String,
    /// Creation timestamp (Unix seconds)
    pub created_at: u64,
    /// Wallet version
    pub version: u32,
    /// Optional user label
    pub label: Option<String>,
}

impl ConstantTimeEq for KeyManagerMetadata {
    fn ct_eq(&self, other: &Self) -> Choice {
        let self_label = self.label.as_deref().unwrap_or("");
        let other_label = other.label.as_deref().unwrap_or("");

        self.wallet_id.as_bytes().ct_eq(other.wallet_id.as_bytes())
            & self.created_at.ct_eq(&other.created_at)
            & self.version.ct_eq(&other.version)
            & u8::from(self.label.is_some()).ct_eq(&u8::from(other.label.is_some()))
            & self_label.as_bytes().ct_eq(other_label.as_bytes())
    }
}
