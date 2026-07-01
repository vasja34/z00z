---
phase: 047-wallet-redesign
plan: 3
status: complete
completed_at: 2026-05-17
next_plan: 047-04
---

# Phase 047-03 Summary

## ✅ Completed Scope

`047-03` is complete. New wallets now create, save, close, and reopen from
`WalletProfilePayload`; normal non-asset saves no longer rewrite the Snapshot
compatibility sidecar; and live wallet defaults for create/open/recover/backup
consume YAML-backed configuration instead of hardcoded literals.

## 🔑 Files Changed

- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock_open.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`
- `crates/z00z_wallets/src/services/wallet_paths.rs`
- `crates/z00z_wallets/src/wallet_config.yaml`
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_paths_suite.rs`
- `crates/z00z_wallets/src/services/app/test_app_service_suite.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/tests.rs`
- `crates/z00z_wallets/src/services/wallet_service_tests.rs`
- `crates/z00z_wallets/tests/test_phase2_production_hardening.rs`

## ✅ Landed Changes

- Added profile-only `.wlt` persistence for the normal create/save/open-session
  paths while keeping the dual profile-plus-snapshot writer only for explicit
  compatibility seams that still need the legacy claimed-asset sidecar.
- Rewired the normal save path to serialize `WalletProfilePayload` only, so
  non-asset saves do not mutate `Snapshot.claimed_assets`.
- Kept compatibility load behavior honest: profile-only wallets reopen cleanly,
  snapshot-only legacy wallets backfill a profile on load, and claimed assets
  can still be rehydrated from the snapshot sidecar until the owned-asset
  authority wave lands.
- Expanded `wallet_config.yaml` and wired live defaults so wallet settings,
  auto-lock, backup location, and recovery gap limit come from YAML-backed
  config or explicit overrides instead of hardcoded literals.
- Fixed the older phase-2 tamper regression test so it now attacks the live
  encrypted wallet object pointer instead of assuming every normal save must
  carry a snapshot sidecar.

## ⚠️ Boundary Kept Intact

- `Snapshot` remains compatibility-only on this slice. `047-03` does not claim
  owned assets are object-backed yet.
- Claimed assets still reload from the snapshot sidecar until `047-04` moves the
  live authority to `OwnedAssetPayload`.

## 👁️ Review Passes

- Pass 1: Found a validation regression in
  `test_encrypted_load_fails_tampering`, which still assumed a mandatory
  snapshot sidecar after the profile-only cutover. Fixed by targeting the live
  profile object pointer first and falling back to the legacy snapshot pointer.
- Pass 2: Rechecked the profile-only create/save/recovery boundary and the
  YAML-backed defaults cutover against the plan and current code. No significant
  issues found.
- Pass 3: Rechecked the compatibility boundary, validation evidence, and summary
  claims against the final tree. No significant issues found.

Two consecutive clean passes were achieved on passes 2 and 3.

## ✅ Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the final tree.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed on the final tree.
- `cargo test --release --features test-fast --features wallet_debug_dump -p
  z00z_wallets phase047_ -- --nocapture` passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p
  z00z_wallets wallet_yaml -- --nocapture` passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p
  z00z_wallets create_and_save_keep_profile_only_wlt -- --nocapture` passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p
  z00z_wallets normal_save_does_not_rewrite_snapshot_claimed_assets -- --nocapture`
  passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p
  z00z_wallets --test test_phase2_production_hardening
  test_encrypted_load_fails_tampering -- --nocapture` passed.
