//! Asset integration entry point.
//!
//! The canonical asset test root is `crates/z00z_core/tests/`. This crate keeps
//! the root-level module map in `test_assets_mod.rs` and reserves
//! `tests/fixtures/` for generated helper artifacts only.
//!
//! ```bash
//! cargo test --release -p z00z_core --test assets_tests
//! cargo test --release -p z00z_core --test assets_tests test_assets_nonce_uniqueness
//! cargo test --release -p z00z_core --test asset_signature_domain
//! ```

#[path = "test_assets_mod.rs"]
mod assets;
