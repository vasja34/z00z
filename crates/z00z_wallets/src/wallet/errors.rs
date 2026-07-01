//! Wallet error types
//!
//! Structured error types using `thiserror` as per Z00Z Design Foundation.

use crate::key::Bip44Error;
use crate::security::encryption::WalletEncryptionError;
use thiserror::Error;

include!("errors_types.rs");
include!("errors_impls.rs");

#[cfg(test)]
mod tests {
    include!("test_errors_suite.rs");
}
