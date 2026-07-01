//! Core security utilities.

/// Password policy and validation logic.
pub mod password;

#[cfg(not(target_arch = "wasm32"))]
pub mod encryption;

/// Secret and key vault helpers.
pub mod vault;

// Re-export all public types
#[cfg(not(target_arch = "wasm32"))]
pub use encryption::{EncryptedWalletContainer, WalletEncryption, WalletEncryptionError};
pub use vault::{
    EncryptionScheme, FileKeyStore, FileKeyStoreError, SecretStore, SecretStoreError,
    SecretStoreImpl, SecureKeyStore,
};
pub use z00z_crypto::secret::SecretBytes;
