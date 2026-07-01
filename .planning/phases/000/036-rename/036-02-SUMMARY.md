# 036-02 Summary

## Scope

This summary records the completion state for `036-02-PLAN.md`, covering task
`036-04 Legacy Backup Payload Import-Export Retirement`, task
`036-05 Seed-Container Contract Migration Hold`, and task
`036-06 Receiver-Card Publication Migration Hold`.

## Outcome

Plan 02 is fully closed.

Phase 036 now keeps one truthful live backup payload contract, with legacy
backup payload compatibility lanes removed in raw-row order, while the
persisted seed-container contract and the published receiver-card trust
contract remain explicit hold-only surfaces with no rename or delete in this
wave.

## Repository Changes

- `crates/z00z_wallets/src/core/backup/backup_wire.rs` now exposes one
  canonical `BackupPayload` type after retiring the legacy `V1` and `V3`
  payload rows.
- `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` now validates
  backup metadata as V4-only, keeps one canonical AAD builder, and removes the
  legacy payload decode compatibility lanes.
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs` now emits only
  the canonical backup payload and no longer carries legacy AAD compatibility
  builders.
- `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs` now verifies
  only the canonical export-pack path and no longer carries legacy verify
  compatibility lanes.
- `crates/z00z_wallets/src/services/wallet_service_actions_backup.rs` now
  requires a real imported export pack and no longer keeps a stale dead legacy
  restore branch.
- `crates/z00z_wallets/src/core/backup/backup_importer.rs` comments now match
  the current chain-bound backup contract instead of describing retired
  compatibility semantics.
- `crates/z00z_wallets/src/core/key/seed_cipher_container.rs` remains an
  explicit hold-only persisted contract with `VERSION_V1` and
  `AAD_VERSION_V1` unchanged.
- `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` remains an
  explicit hold-only published trust contract with `ReceiverCardRecordV1`
  unchanged.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test -p z00z_wallets --release --features test-fast --lib services::wallet_service::wallet_service_tests::tests::legacy_v1_restore_fails -- --exact --nocapture`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`: passed with exit code `0`
- `mcp_codacy_mcp_se_codacy_cli_analyze`: clean on the Plan 02 code and planning files touched in this wave
- repository grep checks after the final cleanup found no live `BackupPayloadV1`, `BackupPayloadV3`, `build_aad_bytes_v1`, `build_aad_bytes_v2`, `decode_export_pack_v2`, `decode_export_pack_v3`, `decode_legacy_payload_v1`, `verify_export_pack_v2`, `verify_export_pack_v3`, or `verify_legacy_payload_v1` symbols in active wallet backup code

## Review Loop

The review loop closed truthfully with the following sequence:

1. the first review pass found stale tests still asserting the retired legacy
   backup-KDF error path instead of the current version-mismatch rejection;
   those expectations were fixed in the backup importer, contract, and wallet
   service tests
2. the intended later `/GSD-Review-Tasks-Execution` subagent passes were blocked
   by rate limiting, so they were not claimed as clean automated passes
3. a manual follow-up review found and removed a stale dead legacy restore
   branch in `wallet_service_actions_backup.rs`
4. two consecutive manual review passes after that cleanup found no remaining
   material issues in the active Plan 02 slice

## Current Boundary

This summary records only Plan 02 closure. It does not claim any work from
`036-03-PLAN.md`, including the hold-only claim-v2 protocol rows, the
future-reserved address-v2 rows, or the test-only review track.
