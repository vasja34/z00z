---
phase: 047-wallet-redesign
plan: 5
status: complete
completed_at: 2026-05-18
next_plan: 047-06
---

# Phase 047-05 Summary

## ✅ Completed Scope

`047-05` is complete for the receive and scan persistence coupling slice.
`recv_range(...)` now persists scan-hit owned assets plus `ScanStatePayload`
through one `.wlt` write transaction, replay stays idempotent for matching
owned-asset wire data, and compatibility receive helpers remain explicitly
non-canonical instead of becoming a second authority plane.

## 🔑 Files Changed

- `crates/z00z_wallets/src/db/redb_wallet_store/mutations/mod.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/mutations/upserts.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
- `crates/z00z_wallets/src/db/mod.rs`
- `crates/z00z_wallets/src/services/wallet_service_tests.rs`

## ✅ Landed Changes

- Added an in-transaction `upsert_scan_state_with_txn(...)` seam so receive
  flow can write scan progress inside the same RedB transaction as owned-asset
  inserts.
- Extended the `.wlt` owned-asset store with `persist_scan_batch(...)`, which
  deduplicates one scan batch by `asset_id`, records `OwnedAssetSource::Scan`,
  idempotently no-ops only for matching wire payloads, and fails closed on
  conflicting duplicates.
- Rewired `recv_range(...)` so scan-hit collection, owned-asset persistence,
  and cursor advancement happen on one wallet-native authority plane instead
  of the old Snapshot claimed-asset path.
- Kept `recv_route(..., ReceiveNext::PersistClaim)` and manual claim language
  compatibility-only, with comments and tests that stop them from becoming a
  second live persistence authority.
- Strengthened restart and replay coverage so re-scanning the same chunk after
  cursor reset does not create duplicate owned assets and keeps `ScanRef`
  metadata coherent.

## ⚠️ Boundary Kept Intact

- `047-05` does not claim the wider tx build, cancel, broadcast, reconcile,
  or asset-view cutover is complete. That remains `047-06` scope.
- `047-05` does not retire Snapshot compatibility artifacts everywhere. It
  closes the live receive path, not the whole bridge-retirement wave.
- `047-05` keeps wallet-side ownership detection unchanged; remote chunks and
  proofs still supply evidence only and never become ownership authority.

## 👁️ Review Passes

- Pass 1: Re-read the receive and replay assertions against the live seams and
  found one order-dependent restart assertion in the migrated test shape. The
  lookup was made deterministic before closeout.
- Pass 2: Rechecked the atomic owned-asset plus scan-cursor write path and the
  compatibility/manual-claim boundary after the deterministic test fix. No
  significant issues found.
- Pass 3: Rechecked the final closeout claims against the live seams
  `recv_range(...)`, `persist_scan_batch(...)`,
  `upsert_scan_state_with_txn(...)`, `OwnedAssetSource::Scan`, and the final
  broad release gate. No significant issues found.

Two consecutive clean passes were achieved on passes 2 and 3.

## ✅ Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  on the final tree.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed on the final tree.
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_recv_range_ -- --nocapture`
  passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_recv_route_gate -- --nocapture`
  passed.
