---
phase: 042-refactor-wallets
reviewed: 2025-07-16T00:00:00Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - crates/z00z_wallets/src/core/foundation/mod.rs
  - crates/z00z_wallets/src/core/foundation/domains.rs
  - crates/z00z_wallets/src/core/foundation/hashing.rs
  - crates/z00z_wallets/src/core/network/mod.rs
  - crates/z00z_wallets/src/core/network/kernel.rs
  - crates/z00z_wallets/src/core/key/manager/mod.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager.rs
  - crates/z00z_wallets/src/core/key/manager/key_state.rs
  - crates/z00z_wallets/src/core/key/manager/key_cache.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager_impl.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager_impl_cache.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager_impl_cache_validation.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager_impl_gap.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager_impl_state.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager_impl_system.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager_impl_trait.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager_redb.rs
  - crates/z00z_wallets/src/core/key/manager/key_manager_redb_wallet.rs
  - crates/z00z_wallets/src/core/key/receiver/mod.rs
  - crates/z00z_wallets/src/core/key/receiver/stealth_keys.rs
  - crates/z00z_wallets/src/core/key/receiver/stealth_keys_secret.rs
  - crates/z00z_wallets/src/core/key/receiver/stealth_keys_identity.rs
  - crates/z00z_wallets/src/core/key/receiver/stealth_keys_receiver.rs
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 042: Code Review Report — Pass 3

**Reviewed:** 2025-07-16  
**Depth:** standard  
**Files Reviewed:** 23  
**Status:** issues_found → fixed in-pass

## Summary

Pass 3 covers the new `core/` tree: `foundation/` (domains + hashing), `network/` (stub),
`key/manager/` (10 files), and `key/receiver/` (4 files).

The only finding was a WARNING in `hashing.rs` — a `#![allow(missing_docs)]` directive
that silently suppressed the crate-wide `#![warn(missing_docs)]` for the entire file,
leaving 14 public type aliases, 2 enums, and their methods with no documentation.
The fix was applied in-pass: the directive was removed and every previously undocumented
public item received a `///` doc comment. The crate compiles with 0 warnings and all
37 tests pass after the fix.

All other 22 files are clean. Notable positive observations:

- `domains.rs` is the single source of truth for all domain constants — no duplication.
- `hashing.rs` uses `subtle::ConstantTimeEq` in `verify_index_mac()` ✅
- `key_manager_impl_state.rs` clears cache on `init_from_seed()` re-entry ✅
- `key_manager_impl_gap.rs` uses `compare_exchange_weak` + retry loop with `checked_sub`/`checked_add` ✅
- `key_manager_impl_cache_validation.rs` wraps derived secret in `Zeroizing::new()` ✅ and calls `verify_key()` after derivation ✅
- `key_manager_impl_trait.rs`: `clear()` calls `bip44_manager.zeroize_seed()` ✅; `derive_secret_transient()` returns `Zeroizing<RistrettoSecretKey>` ✅
- `stealth_keys_secret.rs`: `ReceiverSecret([u8; 32])` derives `Zeroize + ZeroizeOnDrop` ✅; `from_encrypted` uses `Zeroizing::new()` for plaintext ✅
- `stealth_keys_identity.rs`: zero scalar and zero point rejection ✅
- `stealth_keys_receiver.rs`: secret fields wrapped in `Hidden<Z00ZScalar>` ✅; `kdf_params()` uses weak params only under `#[cfg(test)]` ✅
- `key_manager_redb.rs`: `map_auth_error()` maps `CryptoOperationFailed` → `AuthenticationFailed`, preventing crypto-error leakage to callers ✅

---

## Warnings

### WR-01: `hashing.rs` suppressed `missing_docs` for entire file

**File:** `crates/z00z_wallets/src/core/foundation/hashing.rs:2`  
**Issue:** `#![allow(missing_docs)]` was placed on line 2, between the module `//!` doc comment
and its continuation. This blanket suppression silenced `#![warn(missing_docs)]` for all 14 public
type aliases (`TxHashHasher`, `SchnorrChallengeHasher`, `PasswordBloomHasher`, `IndexMacHasher`,
`SnapshotChecksumHasher`, `WalletIntegrityHasher`, `WalletFileIdHasher`, `WalletSeedSaltHasher`,
`WalletFingerprintHasher`, `PayRefHasher`, `RedbWalletDataKeyHasher`, `RedbWalletIndexKeyHasher`,
`RedbWalletIntegrityKeyHasher`), the `ChallengeSize` and `ChallengeBytes` enums, and their methods.

**Fix applied:**

- Removed `#![allow(missing_docs)]`
- Added `///` doc comments to every previously undocumented public item
- `cargo check` result: `Finished 'release' profile [optimized] target(s) in 6.16s` — 0 warnings ✅
- `cargo test` result: `37 passed; 0 failed` ✅

---

_Reviewed: 2025-07-16_  
_Reviewer: gsd-code-reviewer (pass 3)_  
_Depth: standard_
