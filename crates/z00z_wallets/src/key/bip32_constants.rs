/// Highest valid non-hardened BIP-32 child index.
pub const MAX_BIP32_INDEX: u32 = 2_147_483_647; // 2^31 - 1

// ============================================================================
// BIP-44 PATH COMPONENTS - VIEW KEY SEPARATION
// ============================================================================

/// Account offset for view key derivation (standard BIP-44 compliant).
///
/// View keys use account range [100000, 199999] to separate from spend keys [0, 99999].
/// This maintains standard BIP-44 semantics:
/// - Spend keys: m/44'/1337'/account'/change/index (account < 100000)
/// - View keys:  m/44'/1337'/(account+100000)'/change/index
///
/// Benefits:
/// - Standard BIP-44 change field semantics preserved (0=external, 1=internal)
/// - Hardware wallet compatible (scans standard account ranges)
/// - Recovery tools work normally within each account range
/// - Smaller offset than 1M reduces recovery scan time
///
/// Example:
/// - Spend path: m/44'/1337'/0'/0/5
/// - View path:  m/44'/1337'/100000'/0/5 (same change, index)
pub const VIEW_KEY_ACCOUNT_OFFSET: u32 = 100_000;

// ============================================================================
// BIP-39 SEED TYPE
// ============================================================================

/// BIP-39 seed material (64 bytes)
///
/// This type represents the output of a BIP-39 mnemonic + passphrase.
/// It is the ONLY valid input for BIP-32/BIP-44 key derivation.
///
/// # Compile-time checks
///
/// `Bip39Seed64` must not implement `Clone`.
///
/// ```compile_fail
/// use z00z_wallets::key::Bip39Seed64;
///
/// fn require_clone<T: Clone>() {}
/// require_clone::<Bip39Seed64>();
/// ```
///
/// # Security
///
/// - Seed MUST be generated from cryptographically secure source
/// - Seed bytes are automatically zeroized on drop via `Zeroize` trait
/// - For defense in depth, call `zeroize_all()` explicitly before drop
///   to ensure no copies remain in memory after moves
/// - Seed is validated for weak entropy patterns
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct Bip39Seed64([u8; 64]);

impl fmt::Debug for Bip39Seed64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bip39Seed64")
            .field("seed", &"<redacted>")
            .finish()
    }
}

impl Bip39Seed64 {
    /// Create new BIP-39 seed from 64-byte array
    pub fn new(seed: [u8; 64]) -> Self {
        Bip39Seed64(seed)
    }

    /// Create new BIP-39 seed from slice
    ///
    /// # Returns
    ///
    /// `Result<Self, Bip44Error>` - Error if seed is not exactly 64 bytes
    pub fn from_slice(seed: &[u8]) -> Result<Self, Bip44Error> {
        if seed.len() != 64 {
            return Err(Bip44Error::InvalidSeed(seed.len()));
        }

        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(seed);
        Ok(Bip39Seed64(bytes))
    }

    /// Get reference to seed bytes
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Consume and return seed bytes
    pub fn into_bytes(self) -> [u8; 64] {
        self.0
    }

    /// Check if seed is all zeros (weak entropy)
    pub fn is_all_zeros(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    /// Check if all bytes are identical (weak entropy)
    pub fn is_all_identical(&self) -> bool {
        let first = self.0[0];
        self.0.iter().all(|&b| b == first)
    }

    /// Validate seed entropy
    ///
    /// # Returns
    ///
    /// `Result<(), Bip44Error>` - Error if seed has weak entropy
    pub fn validate_entropy(&self) -> Result<(), Bip44Error> {
        crate::key::validate_entropy(&self.0)
            .map_err(|e| Bip44Error::WeakEntropy(e.to_string()))
    }

    /// Explicitly zeroize seed bytes (defense in depth).
    pub fn zeroize_all(&mut self) {
        self.0.zeroize();
    }
}

impl From<[u8; 64]> for Bip39Seed64 {
    fn from(bytes: [u8; 64]) -> Self {
        Bip39Seed64(bytes)
    }
}

impl From<Bip39Seed64> for [u8; 64] {
    fn from(seed: Bip39Seed64) -> Self {
        seed.0
    }
}

impl AsRef<[u8; 64]> for Bip39Seed64 {
    fn as_ref(&self) -> &[u8; 64] {
        &self.0
    }
}

impl AsRef<[u8]> for Bip39Seed64 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// ============================================================================
// Z00Z BIP-44 DERIVATION POLICY CONSTANTS
// ============================================================================
// These constants define the authoritative Z00Z derivation policy.
// All modules MUST use these constants, not hardcoded values.

/// BIP-44 purpose constant (always 44')
pub const Z00Z_BIP44_PURPOSE: u32 = 44;

/// Z00Z coin type from SLIP-0044 registry
pub const Z00Z_BIP44_ASSET: u32 = 1337;
