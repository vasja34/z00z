//! Secret store trait and error types.
//!
//! Defines the interface for encrypted secret storage with session management.

use crate::wallet::session::SessionHandle;
use thiserror::Error;

/// Secret store errors.
#[derive(Debug, Error)]
pub enum SecretStoreError {
    /// Secret store already initialized
    #[error("secret store already initialized")]
    AlreadyInitialized,

    /// Invalid password
    #[error("invalid password")]
    InvalidPassword,

    /// Secret store not initialized
    #[error("secret store not initialized")]
    NotInitialized,

    /// Already unlocked
    #[error("already unlocked")]
    AlreadyUnlocked,

    /// Not unlocked
    #[error("not unlocked")]
    NotUnlocked,

    /// Session expired
    #[error("session expired")]
    SessionExpired,

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] z00z_utils::io::IoError),

    /// Encryption error
    #[error("encryption error: {0}")]
    Encryption(String),

    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Result type for secret store operations.
pub type Result<T> = std::result::Result<T, SecretStoreError>;

/// Secret store trait.
///
/// Provides encrypted storage for wallet secrets with session-based access.
///
/// # Security Model
///
/// - Secrets are encrypted at rest using password-derived key
/// - Unlock operation creates time-limited session
/// - Session expires after inactivity timeout
/// - All sensitive data is zeroized on drop
///
/// # Examples
///
/// ```ignore
/// use z00z_wallets::security::vault::SecretStore;
///
/// let mut store = SecretStoreImpl::new(storage_path, time_provider, rng_provider);
///
/// // Initialize with password
/// store.init_new("mypassword")?;
///
/// // Unlock
/// let session = store.unlock("mypassword")?;
///
/// // Use secrets...
///
/// // Lock
/// store.lock()?;
/// ```
pub trait SecretStore {
    /// Initialize new secret store.
    ///
    /// Creates encrypted storage with given password.
    ///
    /// # Arguments
    ///
    /// * `password` - Password for encryption
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Store already initialized
    /// - I/O error during creation
    /// - Encryption fails
    fn init_new(&mut self, password: &str) -> Result<()>;

    /// Unlock secret store.
    ///
    /// Decrypts secrets and creates active session.
    ///
    /// # Arguments
    ///
    /// * `password` - Password for decryption
    ///
    /// # Returns
    ///
    /// Returns session handle for unlocked secrets.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Store not initialized
    /// - Invalid password
    /// - Already unlocked
    /// - Decryption fails
    fn unlock(&mut self, password: &str) -> Result<SessionHandle>;

    /// Lock secret store.
    ///
    /// Clears secrets from memory and invalidates session.
    ///
    /// # Errors
    ///
    /// Returns error if not unlocked.
    fn lock(&mut self) -> Result<()>;

    /// Check if store is unlocked.
    ///
    /// # Returns
    ///
    /// Returns `true` if store has active session.
    fn is_unlocked(&self) -> bool;

    /// Get active session.
    ///
    /// # Returns
    ///
    /// Returns session handle if unlocked, `None` otherwise.
    fn session(&self) -> Option<&SessionHandle>;

    /// Check if session is expired.
    ///
    /// # Arguments
    ///
    /// * `now_ms` - Current timestamp in Unix milliseconds
    /// * `timeout_ms` - Inactivity timeout in milliseconds
    ///
    /// # Returns
    ///
    /// Returns `true` if session is expired or no active session.
    fn is_session_expired(&self, now_ms: u64, timeout_ms: u64) -> bool;

    /// Update session activity.
    ///
    /// Resets inactivity timer.
    ///
    /// # Arguments
    ///
    /// * `now_ms` - Current timestamp in Unix milliseconds
    ///
    /// # Errors
    ///
    /// Returns error if not unlocked.
    fn update_activity(&mut self, now_ms: u64) -> Result<()>;
}
