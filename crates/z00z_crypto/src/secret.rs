//! Secret bytes wrapper with memory wiping
//!
//! Provides a secure container for sensitive data that automatically
//! wipes memory on drop. Follows security best practices for handling
//! cryptographic secrets.

use std::ops::{Deref, DerefMut};
use zeroize::Zeroize;

/// Byte buffer that is wiped on drop
///
/// # Security Properties
///
/// - **Automatic wiping**: Memory is zeroed when dropped
/// - **Deref support**: Can be used like `&[u8]` or `&mut [u8]`
/// - **No copying**: Wraps Vec without extra allocations
///
/// # Example
///
/// ```
/// use z00z_crypto::secret::SecretBytes;
///
/// let secret = SecretBytes::new(vec![1, 2, 3, 4]);
/// assert_eq!(secret.len(), 4);
/// assert_eq!(*secret, [1, 2, 3, 4]);
///
/// // Memory is wiped when dropped
/// drop(secret);
/// ```
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SecretBytes {
    bytes: Vec<u8>,
}

impl SecretBytes {
    /// Create new secret from owned bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Create from slice (copies data)
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self::new(bytes.to_vec())
    }

    /// Borrow as slice
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    /// Borrow as mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Explicit wipe (also happens automatically on drop)
    pub fn wipe(&mut self) {
        // Zeroize the bytes in-place without changing length
        // This ensures memory is wiped but structure is preserved
        if !self.bytes.is_empty() {
            // Get mutable slice and zeroize it
            let slice = &mut self.bytes[..];
            slice.zeroize();
        }
    }

    /// DANGEROUS: Creates a copy of secret data in memory.
    ///
    /// # Security Warning
    ///
    /// Only use when absolutely necessary (e.g., key derivation).
    /// Prefer borrowing via `as_slice()` when possible.
    /// The copy will be independently zeroized on drop.
    ///
    /// # Example
    ///
    /// ```
    /// use z00z_crypto::secret::SecretBytes;
    ///
    /// let secret = SecretBytes::new(vec![1, 2, 3, 4]);
    /// // Only clone when needed for key derivation
    /// let derived = secret.dangerous_clone();
    /// ```
    #[must_use]
    pub fn dangerous_clone(&self) -> Self {
        Self::new(self.bytes.clone())
    }
}

impl Deref for SecretBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl DerefMut for SecretBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl AsRef<[u8]> for SecretBytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsMut<[u8]> for SecretBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_bytes_basic() {
        let secret = SecretBytes::new(vec![1, 2, 3, 4]);
        assert_eq!(secret.len(), 4);
        assert_eq!(*secret, [1, 2, 3, 4]);
    }

    #[test]
    fn test_secret_bytes_from_slice() {
        let data = [1, 2, 3, 4];
        let secret = SecretBytes::from_slice(&data);
        assert_eq!(*secret, data);
    }

    #[test]
    fn test_secret_bytes_mut() {
        let mut secret = SecretBytes::new(vec![1, 2, 3, 4]);
        secret[0] = 10;
        assert_eq!(*secret, [10, 2, 3, 4]);
    }

    #[test]
    fn test_secret_bytes_wipe() {
        let mut secret = SecretBytes::new(vec![1, 2, 3, 4]);
        secret.wipe();
        assert_eq!(*secret, [0, 0, 0, 0]);
    }

    #[test]
    fn test_secret_bytes_dangerous_clone() {
        let secret1 = SecretBytes::new(vec![1, 2, 3, 4]);
        // SAFETY: Clone needed for test verification
        let secret2 = secret1.dangerous_clone();
        assert_eq!(*secret1, *secret2);
    }

    #[test]
    fn test_secret_bytes_as_ref() {
        let secret = SecretBytes::new(vec![1, 2, 3, 4]);
        let slice: &[u8] = secret.as_ref();
        assert_eq!(slice, [1, 2, 3, 4]);
    }
}
