---
phase: 042-refactor-wallets
reviewed: 2026-05-03T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - crates/z00z_wallets/src/core/key/bip/bip32.rs
  - crates/z00z_wallets/src/core/key/bip/bip32_key_deriver.rs
  - crates/z00z_wallets/src/core/key/bip/bip32_path_validator.rs
  - crates/z00z_wallets/src/core/key/bip/bip32_ristretto_bridge.rs
  - crates/z00z_wallets/src/core/key/seed/seed_mnemonic.rs
  - crates/z00z_wallets/src/core/mod.rs
  - crates/z00z_wallets/src/core/key/mod.rs
  - crates/z00z_wallets/src/core/stealth/crypto/ecdh.rs
  - crates/z00z_wallets/src/core/stealth/crypto/ephemeral.rs
  - crates/z00z_wallets/src/core/persistence/assets/test_asset_storage_impl_suite.rs
findings:
  critical: 2
  warning: 0
  info: 0
  total: 2
status: issues_found
---

# Phase 042-01 Review Report

## Summary

One YOLO review pass was executed for the requested scope only.
Two BLOCKER-level issues were found and fixed in-place.
Post-fix validation passed for compile and targeted tests.

## Critical Issues

### CR-01: Behavioral/API drift in seed mnemonic relocation (BLOCKER)

File: crates/z00z_wallets/src/core/key/seed/seed_mnemonic.rs

Issue:
The relocation included broad behavior changes (error taxonomy, entropy handling, and removal of convenience APIs) instead of a move-only compatibility-preserving change.
This violated the phase threat model expectation to avoid semantic edits during structural migration.

Fix applied:
Restored the pre-move implementation from HEAD into the relocated target path:

- source restored from: crates/z00z_wallets/src/core/key/seed_mnemonic.rs (HEAD)
- target: crates/z00z_wallets/src/core/key/seed/seed_mnemonic.rs

### CR-02: Regression in key bridge/deriver semantics during bip move (BLOCKER)

Files:

- crates/z00z_wallets/src/core/key/bip/bip32_key_deriver.rs
- crates/z00z_wallets/src/core/key/bip/bip32_path_validator.rs
- crates/z00z_wallets/src/core/key/bip/bip32_ristretto_bridge.rs

Issue:
The move introduced non-trivial semantic changes (validation/error mapping/derivation behavior and sensitive-buffer handling), plus a compile regression in bridge include scope (`Zeroizing` unresolved).

Fix applied:

- Restored pre-move versions from HEAD into relocated paths for:

  - bip32_key_deriver.rs
  - bip32_path_validator.rs
  - bip32_ristretto_bridge.rs

- Added missing import in include host:

  - crates/z00z_wallets/src/core/key/bip/bip32.rs: `use zeroize::{Zeroize, Zeroizing};`

## Validation Executed

- cargo check -p z00z_wallets --release --features test-fast --features wallet_debug_dump
- cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_bip44
- cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_ecdh
- cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --lib asset_storage
- cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --lib core::key::seed::tests::test_mixed_script_mnemonic_rejected

All passed after fixes.
