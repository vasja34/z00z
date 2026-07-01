//! 📌 BIP-32/BIP-44 hierarchical deterministic key derivation.
//!
//! 🎯 This module implements **real** BIP-32 CKD (Child Key Derivation) with chain codes,
//! following the BIP-44 path structure.
//!
//! 🔑 # BIP-44 Path Structure
//!
//! ```text
//! m / purpose' / asset_type' / account' / change / address_index
//! ```
//!
//! 📌 - `purpose`: Always `44'` (hardened)
//! 📌 - `asset_type`: SLIP-0044 asset type (hardened)
//! 📌 - `account`: Account index (hardened)
//! 📌 - `change`: 0=external, 1=internal (non-hardened)
//! 📌 - `address_index`: Address index (non-hardened)
//!
//! 🔑 # Security Model
//!
//! 📌 Z00Z uses BIP-44 with a custom coin type (1337) and a **hardened account** boundary.
//! The account boundary is the primary security separator for spend vs view keys.
//!
//! 🔐 # Security (Operational)
//!
//! 📌 - Seed MUST be generated from a cryptographically secure source.
//! 📌 - Master key MUST be protected with `Hidden<T>` wrapper.
//! 📌 - Chain code is sensitive data - treat as secret.
//! 📌 - Ristretto bridge is NOT compatible with Bitcoin/Ethereum addresses.
//! 📌 - Domain separation prevents key reuse across contexts.
//! 📌 - All temporary buffers containing key material MUST be zeroed after use.

#![forbid(unsafe_code)]

use bip32::{ChildNumber, DerivationPath, XPrv};
use std::fmt::{self, Display};
use std::str::FromStr;
use subtle::{Choice, ConstantTimeEq};
use thiserror::Error;
use z00z_core::genesis::ChainType;
use z00z_crypto::expert::keys::RistrettoSecretKey;
use z00z_crypto::expert::traits::SecretKeyTrait;
use z00z_crypto::{DomainHasher, Hidden};
use zeroize::{Zeroize, Zeroizing};

#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(test)]
static SEED_ZEROIZED: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
pub(crate) fn reset_seed_zeroized() {
    SEED_ZEROIZED.store(false, Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn seed_zeroized() -> bool {
    SEED_ZEROIZED.load(Ordering::SeqCst)
}

use crate::domains::RistrettoBridgeDomain;

include!("bip32_constants.rs");
include!("bip32_path.rs");
include!("bip32_path_validator.rs");
include!("bip32_key_deriver.rs");
include!("bip32_ristretto_bridge.rs");
include!("bip44_manager.rs");
