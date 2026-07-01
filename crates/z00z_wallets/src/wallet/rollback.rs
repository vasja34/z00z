//! Rollback strategy abstraction for error recovery.
//!
//! Provides traits for implementing rollback operations when transactions or operations
//! fail partially. This is a domain abstraction - concrete implementations live in
//! the services layer.
//!
//! # Design Principles
//!
//! - **Core Domain Abstraction:** Defines the contract for rollback operations
//! - **Implementation Agnostic:** Concrete rollback strategies (e.g., TransactionRollback)
//!   are implemented in the services layer
//! - **Async-First:** All operations are async to support database/network rollback
//!
//! # Examples
//!
//! ```rust,ignore
//! use z00z_wallets::wallet::rollback::{RollbackStrategy, RollbackError};
//! use async_trait::async_trait;
//!
//! struct TransactionRollback {
//!     spent_asset_ids: Vec<AssetId>,
//!     storage: Arc<dyn AssetStorage>,
//! }
//!
//! #[async_trait]
//! impl RollbackStrategy for TransactionRollback {
//!     async fn rollback(&self) -> Result<(), RollbackError> {
//!         for asset_id in &self.spent_asset_ids {
//!             self.storage
//!                 .mark_unspent(asset_id)
//!                 .await
//!                 .map_err(|e| RollbackError::Failed(e.to_string()))?;
//!         }
//!         Ok(())
//!     }
//! }
//! ```

use async_trait::async_trait;
use thiserror::Error;

/// Errors that can occur during rollback operations.
#[derive(Debug, Error)]
pub enum RollbackError {
    /// Rollback operation failed with a specific error message.
    #[error("Rollback failed: {0}")]
    Failed(String),

    /// Storage was unavailable during rollback attempt.
    #[error("Storage unavailable during rollback")]
    StorageUnavailable,

    /// Partial rollback - some operations succeeded, others failed.
    #[error("Partial rollback: {successful} succeeded, {failed} failed")]
    PartialRollback {
        /// Number of successfully rolled back operations.
        successful: usize,
        /// Number of failed rollback operations.
        failed: usize,
    },
}

/// Strategy for rolling back partially completed operations.
///
/// This trait defines the contract for implementing rollback logic when an operation
/// fails after making partial changes. Implementations should be idempotent - calling
/// rollback multiple times should be safe.
///
/// # Design Notes
///
/// - **Async:** All rollback operations are async to support I/O operations
/// - **Idempotent:** Implementations should be safe to call multiple times
/// - **Best Effort:** If rollback fails, log the error but don't panic
/// - **Send + Sync:** Required for use in async/multi-threaded contexts
///
/// # Examples
///
/// See module-level documentation for usage examples.
#[async_trait]
pub trait RollbackStrategy: Send + Sync {
    /// Attempt to rollback the operation.
    ///
    /// Returns `Ok(())` if rollback succeeded, or `Err(RollbackError)` if rollback failed.
    /// Implementations should be idempotent - calling this multiple times should be safe.
    ///
    /// # Errors
    ///
    /// Returns `RollbackError` if:
    /// - Storage is unavailable (`RollbackError::StorageUnavailable`)
    /// - Rollback operation fails (`RollbackError::Failed`)
    /// - Only partial rollback succeeded (`RollbackError::PartialRollback`)
    async fn rollback(&self) -> Result<(), RollbackError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockRollback {
        should_succeed: bool,
    }

    #[async_trait]
    impl RollbackStrategy for MockRollback {
        async fn rollback(&self) -> Result<(), RollbackError> {
            if self.should_succeed {
                Ok(())
            } else {
                Err(RollbackError::Failed("Mock rollback failed".to_string()))
            }
        }
    }

    #[tokio::test]
    async fn test_rollback_success() {
        let rollback = MockRollback {
            should_succeed: true,
        };
        assert!(rollback.rollback().await.is_ok());
    }

    #[tokio::test]
    async fn test_rollback_failure() {
        let rollback = MockRollback {
            should_succeed: false,
        };
        let result = rollback.rollback().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RollbackError::Failed(_)));
    }

    #[test]
    fn test_rollback_error_display() {
        let error = RollbackError::Failed("test error".to_string());
        assert_eq!(error.to_string(), "Rollback failed: test error");

        let error = RollbackError::StorageUnavailable;
        assert_eq!(error.to_string(), "Storage unavailable during rollback");

        let error = RollbackError::PartialRollback {
            successful: 3,
            failed: 2,
        };
        assert_eq!(error.to_string(), "Partial rollback: 3 succeeded, 2 failed");
    }
}
