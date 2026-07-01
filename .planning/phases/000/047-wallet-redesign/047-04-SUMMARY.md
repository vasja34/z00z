---
phase: 047-wallet-redesign
plan: 4
status: complete
completed_at: 2026-05-17
next_plan: 047-05
---

# Phase 047-04 Summary

## ✅ Completed Scope

`047-04` is complete for the owned-asset authority cutover slice.
`OwnedAssetPayload` plus the internal `.wlt` `WalletAssetStore` now form the
live claimed-asset authority, public compatibility paths read through that
object-backed store, and the tx reservation or confirm bridge now keeps Stage
13 and portable reconcile behavior aligned with the same owned-asset plane.

## 🔑 Files Changed

- `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_registry.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/tests.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_tests.rs`
- `crates/z00z_wallets/src/services/wallet_service_tests.rs`

## ✅ Landed Changes

- Kept one owned asset equal to one encrypted `.wlt` object with one stable
  `object_id`, one internal authority port, and fail-closed duplicate handling
  unless the insert is explicitly idempotent.
- Rewired `put_claimed_asset(...)`, restore-only `set_claimed_assets(...)`, and
  live asset catalog reads to the owned-asset store, so the old
  `wallet_claimed_assets` map remains cache or fallback state instead of
  durable truth.
- Added `.wlt` reservation, release, and spend-confirm support for local owned
  assets, then hooked tx build, cancel, and reconcile through that same live
  authority with imported-vs-change output provenance.
- Replaced the old `serial + 1/+2/+3` tx-output serial policy with inherited
  output serial selection so the live tx lane no longer drifts outside the
  selected asset definition serial pool.
- Hardened the 047 regression harness so repeated asset-id query helpers no
  longer reseed one wallet twice, and profile-only normal saves can honestly
  prove that the snapshot-sidecar bridge is unchanged or absent.

## ⚠️ Boundary Kept Intact

- `047-04` does not claim receive or scan cursor coupling is atomic yet. That
  remains the explicit `047-05` scope.
- `047-04` does not claim the full tx lifecycle and asset-view cutover is done.
  Only the reservation or confirmation support needed to keep one owned-asset
  authority landed here; `047-06` still owns the wider tx surface.
- Snapshot compatibility artifacts can still exist where older bridges need
  them. This slice closes live authority, not full bridge retirement.

## 👁️ Review Passes

- Pass 1: The full release gate exposed a repeated asset-seeding regression, a
  stale snapshot-sidecar test assumption, and one backup restore failure that
  no longer reproduced after the earlier test pollution was removed.
- Pass 2: Rechecked the helper hardening and the profile-only snapshot-sidecar
  expectation against the current owned-asset authority semantics. No
  significant issues found.
- Pass 3: Rechecked the closeout claims against the live seams
  `put_claimed_asset(...)`, `confirm_claimed_asset_spend(...)`,
  `reserve_claimed_asset_inputs(...)`, `release_claimed_asset_reservation(...)`,
  `list_claimed_assets_live_or_cached(...)`, `OwnedAssetSource::Import`,
  `OwnedAssetSource::Change`, and `inherited_output_serial(...)`. No
  significant issues found.

Two consecutive clean passes were achieved on passes 2 and 3.

## ✅ Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the final tree.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed on the final tree.
- `cargo test --release --features test-fast --features wallet_debug_dump -p
  z00z_wallets id_query -- --nocapture` passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p
  z00z_wallets test_save_skips_snapshot_rewrite -- --nocapture`
  passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p
  z00z_wallets test_tx_import_reconcile_portable -- --nocapture` passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p
  z00z_simulator --test test_pipeline_genesis_tx test_s4_bob_pending_ok -- --nocapture`
  passed.
