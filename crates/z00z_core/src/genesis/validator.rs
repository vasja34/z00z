//! Genesis Validation Module
//!
//! Comprehensive validation for genesis assets and state.
//! Includes batch verification, parallel validation support, and config validation.

use crate::assets::{Asset, AssetClass, ObjectFamily};
use crate::domains::GenesisStateHashDomain;
use crate::genesis::generation::{GenesisGenerationPlan, GenesisLane};
use crate::genesis::genesis_config::{AssetConfigEntry, GenesisConfig};
use crate::genesis::ChainType;
use crate::genesis::GenesisAssetAccumulator;
use crate::vouchers::VoucherBackingReferenceV1;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use z00z_crypto::DomainHasher;

#[path = "genesis_config_validate.rs"]
mod genesis_config_validate;
#[path = "genesis_error.rs"]
mod genesis_error;
#[path = "genesis_verification.rs"]
mod genesis_verification;

pub use self::genesis_config_validate::{
    validate_assets_schema, validate_config_schema, validate_genesis_config_for,
    validate_genesis_seed, validate_manifest_ref_keys, validate_manifest_ref_section_key,
    validate_manifest_top_level_keys, validate_rights_schema, validate_version_compatibility,
    validate_voucher_schema, GENESIS_MANIFEST_REF_SECTIONS,
};
pub use self::genesis_error::GenesisError;
pub use self::genesis_verification::{
    compute_genesis_state_hash, detect_chain_type, validate_genesis_commitments_batch,
    verify_genesis_assets, verify_genesis_consensus, ValidationReport,
};

#[cfg(test)]
#[path = "test_validator.rs"]
mod test_validator;
