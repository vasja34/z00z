//! Root-level genesis module map for `tests/test_genesis.rs`.
//!
//! Every owned genesis test file lives directly under `crates/z00z_core/tests/`
//! with a `test_genesis_<behavior>.rs` name. `tests/fixtures/` remains the only
//! allowed owned subdirectory for generated helper artifacts.

#[path = "test_genesis_batch_verification.rs"]
pub mod batch_verification;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_claim_flow.rs"]
pub mod claim_flow;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_commitment_sum.rs"]
pub mod commitment_sum;
#[path = "test_genesis_config.rs"]
pub mod config;
#[path = "test_genesis_config_schema_catalog.rs"]
pub mod config_schema_catalog;
#[path = "test_genesis_cross_network_isolation.rs"]
pub mod cross_network_isolation;
#[path = "test_genesis_crypto_security.rs"]
pub mod crypto_security;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_determinism.rs"]
pub mod determinism;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_manifest.rs"]
pub mod genesis_manifest;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_policies.rs"]
pub mod genesis_policies;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_rights.rs"]
pub mod genesis_rights;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_state_verification.rs"]
pub mod genesis_state_verification;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_vouchers.rs"]
pub mod genesis_vouchers;
#[path = "test_genesis_helpers.rs"]
pub mod helpers;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_integration.rs"]
pub mod integration;
#[path = "test_genesis_multi_asset.rs"]
pub mod multi_asset;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_range_proofs.rs"]
pub mod range_proofs;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_reproducibility.rs"]
pub mod reproducibility;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_security_validation.rs"]
pub mod security_validation;
#[cfg(feature = "deterministic-rng")]
#[path = "test_genesis_settlement_corpus.rs"]
pub mod settlement_corpus;
#[path = "test_genesis_validation.rs"]
pub mod validation;
