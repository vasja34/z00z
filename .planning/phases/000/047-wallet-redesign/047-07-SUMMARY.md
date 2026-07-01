---
phase: 047-wallet-redesign
plan: 7
status: complete
completed_at: 2026-05-19
next_plan: 047-08
---

# Phase 047-07 Summary

## Completed Scope

`047-07` is complete for the backup, restore, and export cutover slice. Live
wallet backup now exports an explicit manifest-backed state pack built from
`WalletProfilePayload`, `OwnedAssetPayload`, scan state, stealth metadata, TOFU
pins, key refs, root secret material, and an explicit JSONL tx-history plane
marker. Restore stages a fresh `.wlt` plus optional JSONL sidecar, validates
before mutation, and keeps Snapshot limited to a one-shot legacy import bridge.

## Files Changed

- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs`
- `crates/z00z_wallets/src/backup/backup_exporter_impl.rs`
- `crates/z00z_wallets/src/backup/backup_importer.rs`
- `crates/z00z_wallets/src/backup/backup_importer_impl/mod.rs`
- `crates/z00z_wallets/src/backup/export/backup_exporter_verify.rs`
- `crates/z00z_wallets/src/db/mod.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/queries.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/mutations/upserts.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs`
- `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`
- `crates/z00z_wallets/src/services/wallet_service_tests.rs`
- `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`
- `crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/tests.rs`
- `crates/z00z_wallets/tests/test_backup_kdf_contract.rs`
- `crates/z00z_wallets/tests/test_backup_metadata_policy.rs`
- `crates/z00z_wallets/tests/test_backup_restore_identity.rs`
- `crates/z00z_wallets/tests/test_seed_salt_policy.rs`
- `crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs`
- `crates/z00z_wallets/tests/test_stub_behavior.rs`
- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`

## Landed Changes

- Reworked `WalletExportPack` into an explicit profile-first state bundle with
  manifest, profile, owned assets, scan state, stealth metadata, TOFU pins,
  key refs, and an explicit JSONL tx-history plane marker, while keeping
  `snapshot` as legacy compatibility input only.
- Added `BackupManifestPayload` counts, network or chain identity, checksum,
  and tx-history plane semantics, then enforced them across export, import,
  verification, and restore.
- Rebuilt `build_wallet_export_pack(...)` on live `.wlt` state and removed the
  optional-field elision that broke the `BincodeCodec` roundtrip, fixing the
  broad-gate `Invalid backup payload` regression exposed by simulator stage 2.
- Added payload-native restore seams
  `replace_payloads_for_restore(...)` and `restore_wallet_pack_atomic(...)` so
  `WalletPlusHistory` stages `.wlt` and JSONL first, promotes them atomically,
  and rolls back cleanly on history commit, `.wlt` commit, or in-memory publish
  failure.
- Updated backup importer and exporter verification so explicit packs reject
  mixed explicit plus snapshot authority, duplicate asset ids, bad manifest
  checksums, and wrong tx-history plane markers before live mutation.
- Migrated backup, export, import, seed-salt, and restore fixtures to explicit
  pack semantics; the final broad rerun also exposed an outdated duplicate
  claims restore assertion, which was corrected and followed by a full verify
  order restart.

## Boundary Kept Intact

- `047-07` does not claim Stage 13, the phase-local 046 spec copy, or stale
  simulator proof wording are fixed. That remains explicit `047-08` scope.
- `047-07` keeps tx history on the canonical `wallet_<stem>_tx_history.jsonl`
  sidecar and does not move live tx history into `.wlt`.
- Legacy `WalletPersistenceState.claimed_assets` survives only as a one-shot
  restore or import input when no explicit profile pack is present; new exports
  set `snapshot: None`.

## Review Passes

- Pass 1: Rechecked the explicit export-pack boundary in
  `build_wallet_export_pack(...)`, `validate_export_pack(...)`, and
  `validate_wallet_restore_pack(...)`; confirmed that new exports carry
  profile-first state and reject mixed explicit plus snapshot authority.
- Pass 2: Rechecked staged restore ordering in
  `restore_wallet_pack_atomic(...)`; confirmed `.wlt` staging, JSONL staging,
  commit ordering, and rollback paths stay fail-closed before in-memory
  publish.
- Pass 3: The first broad workspace rerun exposed one real regression:
  `test_restore_rejects_dup_claims` still expected the legacy snapshot
  duplicate-error string even though the live path now rejects duplicate
  `OwnedAssetPayload` rows. The test contract was corrected and the mandatory
  verify order was restarted from bootstrap.
- Pass 4: Rechecked the final restore boundary after the restart against the
  explicit and legacy restore seams plus duplicate-asset validation. No
  significant issues found.
- Pass 5: Rechecked the final closeout claims against the green bootstrap gate,
  the green broad workspace `cargo test` gate, and the `047-07` write-set. No
  significant issues found.

Two consecutive clean passes were achieved on passes 4 and 5.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the final tree after the duplicate-claims test fix.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed on the final tree after the same fix.
