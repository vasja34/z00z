//! Genesis integration entry point.
//!
//! The canonical genesis test root is `crates/z00z_core/tests/`. This crate
//! keeps the root-level module map in `test_genesis_mod.rs` and leaves
//! `tests/fixtures/` as the only owned subdirectory for generated artifacts.
//!
//! ```bash
//! cargo test --release -p z00z_core --test genesis_tests
//! cargo test --release -p z00z_core --test genesis_tests test_genesis_reproducibility
//! cargo test --release -p z00z_core test_genesis_manifest_goldens -- --nocapture
//! ```

#[path = "test_genesis_mod.rs"]
mod genesis;
