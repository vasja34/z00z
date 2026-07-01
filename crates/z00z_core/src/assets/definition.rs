// crates/z00z_core/src/assets/definition.rs
//
// AssetDefinition - Immutable policy and metadata for asset classes
//
// This module defines the AssetDefinition struct that separates asset policy
// (definition) from individual asset instances (Asset). This enables:
// - Memory efficiency via Arc<AssetDefinition> sharing
// - Clear separation of concerns (policy vs state)
// - Versioning of cryptographic parameters
//
// Security: AssetDefinition is immutable after creation and can be safely
// shared across threads via Arc.

// ============================================================================
// Imports — Organized by convention
// ============================================================================

// External crates

// Standard library
use std::borrow::Cow;
use std::collections::BTreeMap;

// Internal modules
use super::assets::{AssetClass, AssetError};
use super::policy_flags::{BURNABLE, FUNGIBLE, GAS, MINTABLE};

#[path = "definition_id.rs"]
mod definition_id;
#[path = "definition_validate.rs"]
mod definition_validate;

#[path = "definition_format.rs"]
mod definition_format;
/// 📋 Immutable asset definition containing policy and metadata
///
/// AssetDefinition represents the "type" or "class" of an asset, containing
/// all the immutable properties that define what the asset is and how it
/// can be used. Individual assets (Asset instances) reference this definition
/// via `Arc<AssetDefinition>` for memory efficiency.
///
/// # Architecture
///
/// ```text
/// AssetDefinition (1)  ←─── Asset (N instances)
///     │                         │
///     ├─ id                     ├─ definition: Arc<AssetDefinition>
///     ├─ class                  ├─ serial_id: u32
///     ├─ name                   ├─ amount: u64
///     ├─ symbol                 ├─ commitment
///     ├─ decimals               ├─ range_proof
///     ├─ serials                └─ nonce
///     ├─ nominal
///     └─ ...
/// ```
///
/// # Memory Efficiency
///
/// For N assets with M unique definitions:
/// - Old architecture: N × sizeof(AssetDefinition)
/// - New architecture: M × sizeof(AssetDefinition) + N × sizeof(Arc) ≈ M × ~200 + N × 8 bytes
///
/// Example: 10,000 assets with 10 unique definitions:
/// - Old: 10,000 × 200 = 2,000,000 bytes
/// - New: 10 × 200 + 10,000 × 8 = 82,000 bytes (~96% reduction)
///
/// # Security Considerations
///
/// - `id` MUST be derived deterministically from domain + name + symbol
/// - `crypto_version` enables migration to new cryptographic primitives
/// - Definitions SHOULD be loaded from trusted configuration only
/// - Modifications MUST create new definitions (immutability)
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::definition::AssetDefinition;
/// use z00z_core::assets::AssetClass;
/// use z00z_core::assets::policy_flags::BURNABLE;
/// use std::sync::Arc;
///
/// // Create a coin definition
/// let def = AssetDefinition::new(
///     [0u8; 32],           // id
///     AssetClass::Coin,
///     "Z00Z Coin".to_string(),
///     "Z00Z".to_string(),
///     8,                   // decimals
///     50_000,              // total series
///     100_000_000,         // nominal per series
///     "z00z.io".to_string(),
///     1,                   // version
///     1,                   // crypto_version
///     BURNABLE,            // flags
///     None,                // metadata
/// ).expect("valid definition");
///
/// // Share across multiple Assets
/// let shared_def = Arc::new(def);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AssetDefinition {
    /// 🔑 Deterministic 32-byte asset ID
    ///
    /// Derived from: hash(domain || name || symbol || version)
    /// MUST be globally unique per asset type
    pub id: [u8; 32],

    /// 🏷️ Asset class (Coin, Token, NFT, Void)
    pub class: AssetClass,

    /// 📝 Human-readable asset name
    ///
    /// Example: "Z00Z Privacy Coin"
    /// Max length: 64 characters (enforced during validation)
    pub name: String,

    /// 🔤 Trading symbol
    ///
    /// Example: "Z00Z"
    /// Max length: 16 characters (enforced during validation)
    pub symbol: String,

    /// 🔢 Decimal places for display
    ///
    /// - Coins/Tokens: typically 8 (Satoshi-like)
    /// - NFTs/Void: MUST be 0 (indivisible)
    pub decimals: u8,

    /// 🔢 Total number of genesis series
    ///
    /// For coins: number of independent series (e.g., 50,000)
    /// For tokens: 1 (single series)
    /// For NFTs: total supply count
    /// MUST be > 0
    pub serials: u32,

    /// 💰 Nominal value per series (smallest unit)
    ///
    /// For coins: value per genesis coin (e.g., 100,000,000 = 1 coin with 8 decimals)
    /// For tokens: total supply / serials
    /// For NFTs: 1 (each is unique)
    /// MUST be > 0
    pub nominal: u64,

    /// 🌐 Domain name for issuer verification
    ///
    /// Example: "z00z.io"
    /// Used for: DNS-based issuer verification, metadata resolution
    /// Max length: 253 characters (DNS limit)
    pub domain_name: String,

    /// 📌 Protocol version
    ///
    /// Increments on breaking changes to asset structure
    /// Current: 1
    pub version: u8,

    /// 🔐 Cryptographic parameters version
    ///
    /// Separate from protocol version to enable crypto upgrades without
    /// changing asset structure. Validators MUST support multiple versions
    /// in parallel during migration periods.
    ///
    /// Versions:
    /// - 1: Bulletproof+ 64-bit range proofs (current)
    /// - 2: Bulletproof+ 128-bit range proofs (future)
    /// - 3+: Reserved for future cryptographic primitives
    pub crypto_version: u8,

    /// 🚩 IMMUTABLE feature flags (POLICY - AssetDefinition level)
    ///
    /// These flags define the POLICY of the asset class and CANNOT change per
    /// asset instance. They are definition/catalog metadata; the `mintable`
    /// bit does not by itself expose a generic public runtime mint selector on
    /// the current tree.
    /// Bit 0: gas - Asset can be used to pay transaction fees
    /// Bit 1: fungible - Units are interchangeable (true for Coin/Token, false for NFT/Void)
    /// Bit 2: mintable - Definition permits explicit issuance semantics when a
    /// dedicated runtime contract uses them
    /// Bit 4: burnable - Can be sent to burn addresses
    /// Bit 3, 5-7: Reserved (mutable state flags live in Asset struct, not here)
    pub policy_flags: u8,

    /// 📦 Extended metadata (optional)
    ///
    /// Key-value store for additional asset properties:
    /// - "icon_url": URL to asset icon
    /// - "description": Long-form description
    /// - "whitepaper": URL to whitepaper
    /// - "contract_address": Smart contract address (if applicable)
    ///
    /// MUST NOT exceed 1KB total size (enforced during validation)
    pub metadata: Option<BTreeMap<String, String>>,
}

impl AssetDefinition {
    /// 🏗️ Create a new AssetDefinition with validation
    ///
    /// # Arguments
    ///
    /// All parameters are validated according to asset class constraints.
    /// See struct field documentation for details.
    ///
    /// # Returns
    ///
    /// * `Ok(AssetDefinition)` if all validations pass
    /// * `Err(AssetError)` if any validation fails
    ///
    /// # Validation Rules
    ///
    /// - `serials` > 0
    /// - `nominal` > 0
    /// - `decimals` = 0 for NFT/Void classes
    /// - `name` length ≤ 64
    /// - `symbol` length ≤ 16
    /// - `domain_name` length ≤ 253
    /// - `metadata` total size ≤ 1KB
    ///
    /// # Examples
    ///
    /// ```rust
    /// use z00z_core::assets::definition::AssetDefinition;
    /// use z00z_core::assets::AssetClass;
    /// use z00z_core::assets::policy_flags::BURNABLE;
    ///
    /// let def = AssetDefinition::new(
    ///     [1u8; 32],
    ///     AssetClass::Coin,
    ///     "Test Coin".to_string(),
    ///     "TST".to_string(),
    ///     8,
    ///     1000,
    ///     100_000_000,
    ///     "test.io".to_string(),
    ///     1,
    ///     1,
    ///     BURNABLE,  // policy_flags: burnable
    ///     None,
    /// ).expect("valid definition");
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        _id: [u8; 32],
        class: AssetClass,
        name: String,
        symbol: String,
        decimals: u8,
        serials: u32,
        nominal: u64,
        domain_name: String,
        version: u8,
        crypto_version: u8,
        policy_flags: u8,
        metadata: Option<BTreeMap<String, String>>,
    ) -> Result<Self, AssetError> {
        let id = Self::derive_id(
            class,
            &name,
            &symbol,
            decimals,
            serials,
            nominal,
            &domain_name,
            version,
            crypto_version,
            policy_flags,
            metadata.as_ref(),
        );

        let def = Self {
            id,
            class,
            name,
            symbol,
            decimals,
            serials,
            nominal,
            domain_name,
            version,
            crypto_version,
            policy_flags,
            metadata,
        };

        def.validate()?;

        Ok(def)
    }

    /// ✅ Validate this AssetDefinition
    ///
    /// Checks all constraints and returns detailed error on failure.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all validations pass
    /// * `Err(AssetError)` with detailed message if validation fails
    pub fn validate(&self) -> Result<(), AssetError> {
        self.validate_fields()?;
        self.validate_id()
    }
}

#[cfg(test)]
#[path = "test_definition.rs"]
mod tests;
