use z00z_utils::{prelude::TimeProvider, time::TimeError};

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
///         if output.nonce == [0u8; 32] {
///             return Err(TxError::ZeroNonce);
///         }
///
///         let mut seen = HashSet::new();
///         if !seen.insert(output.nonce) {
///             return Err(TxError::DuplicateNonce);
///         }
///
///         if spent_nonces.contains(&output.nonce) {
///             return Err(TxError::NonceAlreadySpent);
///         }
///     }
///     Ok(())
/// }
/// ```
pub type Nonce = [u8; 32];

pub fn get_timestamp_micros(time_provider: &dyn TimeProvider) -> Result<u64, TimeError> {
    try_get_timestamp_micros(time_provider)
}

/// 🎲 Generate timestamp using TimeProvider with explicit error handling.
///
/// Use this function in security-critical code paths where silent epoch fallback
/// is unacceptable.
pub fn try_get_timestamp_micros(time_provider: &dyn TimeProvider) -> Result<u64, TimeError> {
    time_provider.try_unix_timestamp_micros()
}
