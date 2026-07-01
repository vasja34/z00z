//! # Assets Module
//!
//! Confidential multi-asset primitives plus the asset-definition registry used
//! by the live Z00Z protocol.
//!
//! ## Authority Boundary
//!
//! `z00z_core::genesis` is the only canonical bootstrap authority for assets,
//! rights, policies, and vouchers. Asset registry YAML is secondary data for
//! registry loading, examples, fixtures, and compatibility flows only.
//!
//! ## Public Owner Paths
//!
//! - [`registry`] - `AssetDefinitionRegistry`, global fallback registry, and snapshot sync
//! - [`snapshot`] - immutable registry snapshot types
//! - [`wire`] - DTOs for JSON/package serialization
//! - [`definition`] - immutable asset policy definitions
//! - [`assets`] - runtime asset instances
//! - [`gas`] - fee and gas schedule types
//! - [`policy_flags`] - asset policy bits
//! - [`secret`], [`nonce`], [`serial_id`], [`version`] - asset support primitives
//!
//! Cross-family owner paths stay outside this module:
//! - `crate::actions` for action pools
//! - `crate::policies` for policy descriptors
//! - `crate::rights` for rights config and semantics
//! - `crate::vouchers` for voucher config and semantics
//!
//! The removed compatibility shims under
//! `assets::{action_pool,policy_descriptor,right_config,voucher_config}` must
//! not return.
//!
//! ## Registry Catalog Loading
//!
//! ```ignore
//! use std::path::Path;
//! use std::sync::Arc;
//! use z00z_core::assets::registry::AssetDefinitionRegistry;
//! use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
//!     Path::new("configs/devnet_assets_config.yaml"),
//!     Arc::new(NoopLogger),
//!     Arc::new(NoopMetrics),
//!     Arc::new(SystemTimeProvider),
//! )?;
//! assert!(registry.len()? > 0);
//! # Ok(())
//! # }
//! ```
//!
//! The loaded YAML is a secondary registry catalog and must not be treated as
//! a second bootstrap manifest.
//!
//! ## Snapshot Sync
//!
//! ```ignore
//! use std::sync::Arc;
//! use z00z_core::assets::registry::AssetDefinitionRegistry;
//! use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = AssetDefinitionRegistry::new(
//!     Arc::new(NoopLogger),
//!     Arc::new(NoopMetrics),
//!     Arc::new(SystemTimeProvider),
//! );
//! let snapshot = registry.create_snapshot()?;
//!
//! let restored = AssetDefinitionRegistry::new(
//!     Arc::new(NoopLogger),
//!     Arc::new(NoopMetrics),
//!     Arc::new(SystemTimeProvider),
//! );
//! restored.update_from_snapshot(snapshot)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Direct Access
//!
//! ```ignore
//! use std::sync::Arc;
//! use z00z_core::assets::{AssetClass, AssetDefinition};
//! use z00z_core::assets::registry::AssetDefinitionRegistry;
//! use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = AssetDefinitionRegistry::new(
//!     Arc::new(NoopLogger),
//!     Arc::new(NoopMetrics),
//!     Arc::new(SystemTimeProvider),
//! );
//! let definition = AssetDefinition::new(
//!     [7u8; 32],
//!     AssetClass::Coin,
//!     "Example".into(),
//!     "EXM".into(),
//!     8,
//!     10,
//!     1_000,
//!     "example.z00z".into(),
//!     1,
//!     1,
//!     0,
//!     None,
//! )?;
//! registry.insert(definition)?;
//!
//! let asset_id = [7u8; 32];
//! assert!(registry.contains(&asset_id)?);
//! assert!(registry.get(&asset_id)?.is_some());
//! # Ok(())
//! # }
//! ```
//!
//! ## Config Semantics
//!
//! Registry policy parsing uses live `decimals`, `serials`, `nominal`, and `domain_name`
//! fields. The obsolete supply-cap field is not part of the registry schema,
//! and `z00z_utils` abstractions are always live instead of hiding behind a
//! removable feature gate.
//!
//! Integration tests live under `crates/z00z_core/tests/`, and generated
//! shared fixtures live under `crates/z00z_core/tests/fixtures/`.

pub mod amount;
mod asset_class;
mod asset_error;
mod asset_metadata;
mod asset_ownership;
mod asset_validation;
#[allow(clippy::module_inception)]
pub mod assets;
pub mod blinding;
pub mod commitment;
// NOTE: crypto.rs DELETED in v1.134.0 - use z00z_crypto public API instead
pub mod definition;
pub mod gas;
pub mod leaf;
pub mod nonce;
pub mod object_family;
pub mod policy_flags;
pub mod registry;
pub mod secret;
pub mod serial_id;
pub mod snapshot;
pub mod version;
pub mod wire;

// 🆕 New modules (refactoring v1.152.0)
mod registry_catalog; // Secondary registry-catalog parsing helpers (pub(crate) only)

use crate::domains::{NativeCoinDomainDevnet, NativeCoinDomainMainnet, NativeCoinDomainTestnet};
use z00z_crypto::expert::traits::DomainSeparation;

const NATIVE_FEE_NAME: &str = "Z00Z Native Coin";
const NATIVE_FEE_SYMBOL: &str = "Z00Z";
const NATIVE_FEE_VER: u8 = 1;
const NATIVE_FEE_CRYPTO_VER: u8 = 1;

pub fn native_fee_def(domain_name: &str) -> Option<AssetDefinition> {
    let allowed_domains = [
        NativeCoinDomainDevnet::domain(),
        NativeCoinDomainTestnet::domain(),
        NativeCoinDomainMainnet::domain(),
    ];

    if !allowed_domains.contains(&domain_name) {
        return None;
    }

    AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        NATIVE_FEE_NAME.to_string(),
        NATIVE_FEE_SYMBOL.to_string(),
        8,
        100,
        20_000,
        domain_name.to_string(),
        NATIVE_FEE_VER,
        NATIVE_FEE_CRYPTO_VER,
        NATIVE_CASH_POLICY_FLAGS,
        None,
    )
    .ok()
}

pub fn is_native_fee_def(def: &AssetDefinition) -> bool {
    native_fee_def(def.domain_name.as_str())
        .as_ref()
        .is_some_and(|expected| expected == def)
}

// Re-export registry-catalog parsing helpers for use in other z00z_core modules.
pub(crate) use registry_catalog::has_policy_flag_overrides;
pub use registry_catalog::parse_asset_domain_name;
pub use registry_catalog::parse_asset_policy;
pub(crate) use registry_catalog::parse_policy_flags;

// Re-exports for convenience
pub use blinding::BlindingFactorGenerator;
pub use commitment::{commit_amount, verify_commitment_opening};
pub use version::{validate_serial_id_version, AssetPackVersion};
pub use wire::{
    decode_asset_pkg_json, encode_asset_pkg_json, payload_has_secret_field, AssetPkgWire,
    AssetWire, DefinitionWire, ASSET_PKG_JSON_MAX_BYTES,
};

// NOTE: crypto.rs DELETED in v1.134.0 - use z00z_crypto public API instead
// Hash domains (AssetIdHashDomain, ChecksumHashDomain) moved to z00z_crypto internal

// Re-export protocol types from z00z_crypto
pub use z00z_crypto::{
    batch_verify_range_proofs, create_commitment, create_range_proof, verify_range_proof,
    KernelSignature, RangeProof, Z00ZCommitment as Commitment, Z00ZRangeProof,
    Z00ZRistrettoPoint as PublicKey, Z00ZScalar as BlindingFactor, Z00ZScalar, CHECKSUM_BYTES,
    LENGTH_BYTES, VERSION, VERSION_BYTES,
};

pub use amount::{Amount, MAX_AMOUNT};

// Re-export from assets module
pub use assets::{Asset, AssetClass, AssetError, AssetMetadata};
pub use definition::AssetDefinition;
pub use gas::{
    calculate_fee, GasAsset, GasMetered, GasPrice, GasSchedule, GasUnit, GasUsage,
    GAS_SCHEDULE_PLACEHOLDER,
};
pub use leaf::{
    decode_asset_pack, AssetLeaf, AssetPackPlain, AssetPackPlainMemo, DecodedAssetPack, PackErr,
};
pub use nonce::{
    derive_nonce, derive_nonce_minimal, derive_nonce_simple, get_timestamp_micros,
    try_derive_nonce_minimal, try_derive_nonce_simple, try_get_timestamp_micros, Nonce,
    NonceCounter,
};
pub use object_family::{ObjectFamily, ObjectRoleV1};
pub use policy_flags::{
    combine_flags, has_flag, native_cash_uses_action_pools, validate_flags, BURNABLE, FUNGIBLE,
    GAS, MINTABLE, NATIVE_CASH_POLICY_FLAGS, NONE,
};
pub use registry::{AssetDefinitionRegistry, AssetId, GLOBAL_ASSET_REGISTRY};
pub use secret::{generate_asset_secret_checked, AssetSecretError};
pub use serial_id::{
    deserialize_serial_id, serialize_serial_id, validate_serial_bounds, SerialIdError,
    SERIAL_ID_BYTE_LEN,
};
