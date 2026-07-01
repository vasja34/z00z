#![forbid(unsafe_code)]
#![doc = include_str!("nonce_docs.md")]

//! let counter_value = counter.increment_unsafe(&time_provider)?;
//! tx.save_counter(&counter)?; // Persist BEFORE using
//! tx.commit()?;
//! let nonce = derive_nonce(&seed, counter_value, timestamp, &prev_output);
//! ```
//!
//! **Nonce reuse = TOTAL PRIVACY LOSS!**
//!
//! ## 📚 Additional Documentation
//!
//! - **Validation:** See [`Nonce`] type documentation for validation requirements
//! - **Counter Management:** See [`NonceCounter`] for persistence guidelines
//! - **Security Model:** See `docs/genesis/genesis_spec_crypto_review.md`

// ============================================================================
// Imports — Organized by convention
// ============================================================================

#[path = "nonce_counter.rs"]
mod nonce_counter;
#[path = "nonce_derivation.rs"]
mod nonce_derivation;
#[path = "nonce_type.rs"]
mod nonce_type;

pub use nonce_counter::NonceCounter;
pub use nonce_derivation::{
    derive_genesis_nonce, derive_nonce, derive_nonce_minimal, derive_nonce_simple,
    try_derive_nonce_minimal, try_derive_nonce_simple,
};
pub use nonce_type::{get_timestamp_micros, try_get_timestamp_micros, Nonce};

/// 🔑 32-byte cryptographic nonce for Asset privacy
///
/// Nonces MUST be globally unique to prevent privacy leakage.
/// This type alias ensures consistent usage across the codebase.
///
/// # Nonce Uniqueness Validation
///
/// **Responsibility Boundary:**
/// - `Asset::new()` - Creates assets with provided nonce (NO validation)
/// - Transaction Validator - MUST enforce nonce uniqueness and non-zero check
///
/// This design avoids duplicate validation overhead:
/// 1. Asset creation in tests/benches doesn't need validation
/// 2. Transaction processing validates once for all outputs
/// 3. Clear separation: construction vs. protocol rules
///
/// **Required Validation in Transaction Layer:**
/// ```rust,ignore
/// pub fn validate_transaction_nonces(
///     outputs: &[Asset],
///     spent_nonces: &HashSet<[u8; 32]>,
/// ) -> Result<(), TxError> {
///     for output in outputs {
///         // Check 1: No zero nonces (privacy leak)
///         if output.nonce == [0u8; 32] {
///             return Err(TxError::ZeroNonce);
///         }
///         
///         // Check 2: No duplicate nonces within transaction
///         let mut seen = HashSet::new();
///         if !seen.insert(output.nonce) {
///             return Err(TxError::DuplicateNonce);
///         }
///         
///         // Check 3: No nonce reuse from blockchain history
///         if spent_nonces.contains(&output.nonce) {
///             return Err(TxError::NonceAlreadySpent);
///         }
///     }
///     Ok(())
/// }
/// ```
/// 🔐 Derive deterministic nonce from wallet seed and context
///
/// Uses BLAKE2b with domain separation to generate cryptographically secure
/// nonces that are:
/// - Deterministic (same inputs → same output)
/// - Unique (different counter → different nonce)
/// - Unpredictable (requires secret wallet_seed)
/// - Recoverable (wallet can regenerate from seed + counter)
///
/// # Algorithm
///
/// ```text
/// nonce = BLAKE2b_256(
///     domain_separator ||
///     wallet_seed ||
///     counter ||
///     timestamp ||
///     prev_output_hash
/// )
/// ```
///
/// # Arguments
///
/// * `wallet_seed` - 32-byte master seed (MUST be kept secret)
/// * `counter` - Monotonically increasing counter from NonceCounter
/// * `timestamp` - Current Unix timestamp in microseconds (for uniqueness)
/// * `prev_output_hash` - Hash of previous output in transaction chain
///
/// # Returns
///
/// 32-byte nonce suitable for Asset construction
///
/// # Security
///
/// - Domain separation prevents cross-protocol attacks
/// - Counter guarantees no reuse even with same timestamp
/// - prev_output_hash adds transaction-level entropy
/// - wallet_seed provides secret entropy (must never be exposed)
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::nonce::derive_nonce;
///
/// let wallet_seed = [42u8; 32];
/// let counter = 1;
/// let timestamp = 1234567890;
/// let prev_hash = [0u8; 32];
///
/// let nonce = derive_nonce(&wallet_seed, counter, timestamp, &prev_hash);
/// assert_eq!(nonce.len(), 32);
///
/// // Deterministic: same inputs produce same output
/// let nonce2 = derive_nonce(&wallet_seed, counter, timestamp, &prev_hash);
/// assert_eq!(nonce, nonce2);
///
/// // Unique: different counter produces different output
/// let nonce3 = derive_nonce(&wallet_seed, counter + 1, timestamp, &prev_hash);
/// assert_ne!(nonce, nonce3);
/// ```
/// 🎲 Generate timestamp using TimeProvider (always required)
///
/// Helper function to get high-resolution timestamp for nonce derivation.
/// Uses TimeProvider trait for testability and consistency.
///
/// # Arguments
///
/// * `time_provider` - Implementation of TimeProvider trait
///
/// # Returns
///
/// Unix timestamp in microseconds since epoch
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::nonce::get_timestamp_micros;
/// use z00z_utils::prelude::SystemTimeProvider;
///
/// let time = SystemTimeProvider;
/// let timestamp = get_timestamp_micros(&time).expect("timestamp");
/// assert!(timestamp > 0);
/// ```
/// 🔐 Simple nonce derivation - seed and counter required
///
/// Level 2 Convenience: Requires only wallet_seed and counter.
/// Automatically uses current timestamp and zero previous hash.
/// BEST FOR: Typical production usage with seed and counter from wallet.
///
/// # Arguments
///
/// * `wallet_seed` - Secret wallet seed (never expose)
/// * `counter` - Monotonically increasing counter from NonceCounter
/// * `time_provider` - Time provider for timestamp
///
/// # Returns
///
/// 32-byte nonce suitable for Asset construction
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::nonce::derive_nonce_simple;
/// use z00z_utils::prelude::SystemTimeProvider;
///
/// let wallet_seed = [42u8; 32];
/// let counter = 1;
/// let time = SystemTimeProvider;
///
/// let nonce = derive_nonce_simple(&wallet_seed, counter, &time).expect("simple nonce");
/// assert_eq!(nonce.len(), 32);
/// ```
/// 🔐 Minimal nonce derivation - Requires CSPRNG for security
///
/// Level 3 Emergency Function: Generates random nonce using provided RNG.
/// Counter and seed are both generated randomly.
/// BEST FOR: Testing, emergency scenarios, or development with proper RNG.
///
/// # ⚠️ SECURITY NOTICE
///
/// This function generates nonces for testing/emergency use:
/// - Random seed means NO wallet recovery possible
/// - Random counter means NO deterministic nonce reproduction
/// - REQUIRES cryptographically secure RNG (e.g., rand::rngs::OsRng)
/// - Wallet cannot regenerate same nonces after restart
///
/// **DO NOT USE IN PRODUCTION WITHOUT UNDERSTANDING IMPLICATIONS**
///
/// For production, prefer:
/// - `derive_nonce_simple(seed, counter, time_provider)` - for typical usage
/// - `derive_nonce(seed, counter, timestamp, prev_hash)` - for full control
///
/// # Arguments
///
/// * `rng` - Cryptographically secure random number generator (must implement `rand::RngCore` + `rand::CryptoRng`)
/// * `time_provider` - Time provider for timestamp
///
/// # Returns
///
/// 32-byte nonce (cryptographically random)
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::nonce::derive_nonce_minimal;
/// use z00z_utils::prelude::SystemTimeProvider;
/// use rand::rngs::OsRng;
///
/// let time = SystemTimeProvider;
/// // Emergency-only random nonce generation with secure RNG
/// let nonce = derive_nonce_minimal(&mut OsRng, &time).expect("minimal nonce");
/// assert_eq!(nonce.len(), 32);
/// ```
/// 🏗️ Derive deterministic genesis nonce
///
/// Generates nonces for genesis asset generation with:
/// - **Deterministic**: Same inputs → same nonce (reproducible genesis)
/// - **Network-aware**: Domain separation by network type
/// - **Unique per asset**: Uses definition_id + serial_id
///
/// # Arguments
///
/// * `genesis_seed` - Genesis master seed (32 bytes)
/// * `definition_id` - Asset definition ID (32 bytes)
/// * `serial_id` - Serial number within definition
///
/// # Returns
///
/// 32-byte deterministic nonce
///
/// # Security Model
///
/// **Use ONLY for genesis generation:**
/// - Genesis is deterministic (all nodes must agree)
/// - Genesis outputs are semi-public by design
/// - NOT suitable for user transactions (no privacy)
///
/// **Domain Separation:**
/// - Domain: "z00z.core.assets.nonce.genesis.v1"
/// - Label: "genesis_nonce"
/// - Network-specific (mainnet vs testnet)
///
/// **Uniqueness Guarantees:**
/// - Different definition_id → different nonce
/// - Different serial_id → different nonce
/// - Different genesis_seed → different nonce
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::nonce::derive_genesis_nonce;
///
/// let genesis_seed = [0xABu8; 32];
/// let definition_id = [0x01u8; 32];
/// let serial_id = 100;
///
/// let nonce = derive_genesis_nonce(&genesis_seed, &definition_id, serial_id);
/// assert_eq!(nonce.len(), 32);
///
/// // Deterministic: same inputs → same nonce
/// let nonce2 = derive_genesis_nonce(&genesis_seed, &definition_id, serial_id);
/// assert_eq!(nonce, nonce2);
///
/// // Unique: different serial → different nonce
/// let nonce3 = derive_genesis_nonce(&genesis_seed, &definition_id, 101);
/// assert_ne!(nonce, nonce3);
/// ```
///
/// # Implementation Notes
///
/// Uses the same pattern as `derive_genesis_blinding()` from genesis module:
/// 1. Domain-separated hashing (GenesisDomain)
/// 2. Blake2b-512 for wide reduction
/// 3. Network-aware (embedded in domain)
/// 4. Truncation to 32 bytes (first 32 bytes of hash)
///
/// See: `crates/z00z_core/docs/genesis/genesis_spec_crypto_review.md`
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "test_nonce.rs"]
mod tests;
