---
phase: 044-wallet-assets
artifact: test-spec
status: evidence-synced
source: context-todo-plans-and-live-test-anchors
updated: 2026-05-09
owner: Z00Z Wallets and Storage
scope: unit, integration, and end-to-end coverage for Phase 044 wallet assets
---

# Phase 044 Test Spec

## Purpose

This document defines the phase-local unit, integration, and end-to-end test
contract for Phase 044.

It is directly usable by another engineer or agent without guessing scenario
boundaries, invariants, failure paths, or pass oracles.

For this repository, E2E does not mean browser automation. It means realistic
Rust coverage across wallet service, RPC, filesystem, backup/restore,
portable-package, and simulator boundaries.

The spec is evidence-synced because the phase now has execution artifacts.
The planning chain is complete enough to define the test contract, and
`044-coverage.md`, `044-05-SUMMARY.md`, and `044-SUMMARY.md` now record the
closeout evidence.

## Workflow Status

- Mode: `verification-backed`.
- Source artifacts used:
  - `044-CONTEXT.md`
  - `044-TODO.md`
  - `044-01-PLAN.md` through `044-05-PLAN.md`
  - live test anchors already present in `crates/z00z_wallets`, `crates/z00z_core`,
    `crates/z00z_storage`, and `crates/z00z_simulator`
- Completion artifacts present:
  - none

## Classification

### TDD And Integration Targets

| Seam | Class | Why it matters |
| --- | --- | --- |
| `crates/z00z_wallets/src/persistence/assets/asset_storage.rs` and `asset_storage_impl.rs` | TDD / integration | Own wallet asset lifecycle rows, atomic reserve/release, and fail-closed state transitions. |
| `crates/z00z_wallets/src/tx/selection/asset_selector.rs` and `test_asset_selector_suite.rs` | TDD / integration | Prove only `Available` inputs are selectable and double reservation cannot succeed. |
| `crates/z00z_wallets/src/tx/verify/*`, `tx_assembler.rs`, `tx/proof/spend_proof_backend.rs` | TDD / integration | Own canonical package assembly, proof attachment, and verifier truth. |
| `crates/z00z_wallets/src/persistence/tx/tx_storage.rs` and `tx_storage_impl.rs` | TDD / integration | Own canonical JSONL history, deterministic reads and rewrites, and exact package-byte retention. |
| `crates/z00z_wallets/src/backup/*` and `wallet_service_store_persistence_pack.rs` | TDD / integration | Own backup/restore/migration byte preservation and path-contract truth. |
| `crates/z00z_wallets/src/chain/client/*`, `chain/broadcast/*`, `persistence/scans/*`, `tx/state/*` | TDD / integration | Own explicit admission, stored receipts, and storage-backed reconciliation. |
| `crates/z00z_wallets/src/adapters/rpc/methods/*` and `types/*` | integration | Own the user-facing history, pending, balance, send, broadcast, and receive surfaces. |
| `crates/z00z_wallets/src/receiver/*`, `services/wallet/actions/wallet_service_actions_receive.rs` | TDD / integration | Own report-only receive, pending receive, and canonical persist-claim finalization. |

### E2E Targets

| Home | Class | Why it matters |
| --- | --- | --- |
| `crates/z00z_wallets/tests/test_tx_store_integration.rs` | E2E / scenario | Proves JSONL history, folded reads, migration, and exact-byte retention through the real storage seam. |
| `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs` | E2E / scenario | Proves wallet-stem naming, backup/restore, report-only receive, and path-contract truth. |
| `crates/z00z_wallets/tests/test_tx_balance.rs` | E2E / scenario | Proves public balance truth across build, export, import, submit, confirm, cancel, fail, and reconcile. |
| `crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs` | E2E / scenario | Proves forensic backup/restore byte preservation and tamper rejection. |
| `crates/z00z_wallets/tests/test_tx_parity.rs`, `test_tx_roundtrip.rs`, `test_tx_tamper.rs`, `test_tx_wrong_root.rs` | E2E / scenario | Proves role-neutral submission, exact-byte roundtrip, and reject-on-tamper / wrong-root behavior. |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`, `test_tx_broadcast_body.rs`, `asset_impl_tests.rs` | E2E / scenario | Proves sender build/send, explicit admission evidence, and report-only versus persist-claim behavior. |
| `crates/z00z_wallets/tests/test_direct_tx_receive.rs`, `test_e2e_req_flow.rs`, `test_e2e_send_scan.rs`, `test_e2e_runtime_parity.rs` | E2E / scenario | Proves receiver preview, receive flow, scan parity, and end-to-end wallet journey truth. |
| `crates/z00z_wallets/tests/test_wallet_service_errors.rs`, `test_wallet_impl_suite.rs`, `test_rpc_dispatcher_roundtrip.rs` | E2E / scenario | Proves service and RPC failure modes, path handling, and public roundtrip consistency. |
| `crates/z00z_simulator/tests/test_wallet_integration.rs`, `test_claim_persist.rs`, `test_claim_post.rs`, `test_claim_snapshot.rs`, `test_e2e_phase4.rs` | E2E / scenario | Proves simulator-facing wallet behavior, persistence boundaries, and snapshot/report truth. |

### Skip Targets

| Item | Why it is skipped |
| --- | --- |
| `crates/z00z_crypto/tari/**` | Vendor code is read-only in this repository. |
| `044-coverage.md`, `044-SUMMARY.md`, `044-TEST-SPEC.md`, `044-TESTS-TASKS.md` | Planning and closeout artifacts are not runtime test seams. |
| Any proposed parallel tx-history database or second verifier / assembler / receive path | Phase 044 forbids duplicate authority layers. |

## Existing Test Anchors To Reuse

| Wave | Anchors | What they already prove |
| --- | --- | --- |
| W1 | `test_asset_storage_impl_suite.rs`, `test_asset_selector_suite.rs`, `test_tx_send_body.rs`, `test_stealth_output.rs`, `test_wallet_service_suite.rs` | Reservation, sender build/send, and approved output construction. |
| W2 | `test_tx_history_body.rs`, `test_tx_pending_body.rs`, `test_tx_history_cursor_filters.rs`, `test_tx_history_receipt_sort.rs`, `test_tx_store_integration.rs`, `test_wallet_paths_suite.rs`, `test_wallet_impl_suite.rs` | Journal-backed history, pending lists, cursor/filter behavior, and wallet-stem path contracts. |
| W3 | `test_backup_impl_suite.rs`, `test_backup_exporter_suite.rs`, `test_backup_importer_suite.rs`, `test_wallet_backup_suite.rs`, `test_tx_parity.rs`, `test_tx_roundtrip.rs`, `test_tx_tamper.rs`, `test_tx_wrong_root.rs` | Exact-byte backup/restore, tamper rejection, portable-package parity, and wrong-root failures. |
| W4 | `test_tx_broadcast_body.rs`, `test_state_update_suite.rs`, `asset_impl_tests.rs`, `test_direct_tx_receive.rs` | Admission evidence, checkpoint-backed reconciliation, and report-only versus persist-claim receive. |
| W5 | `test_tx_balance.rs`, `test_spec_terms_guard.rs`, `test_tx_drift.rs`, `test_wallet_service_errors.rs`, `test_tx_store_integration.rs` | Balance truth, terminology drift guards, storage drift guards, and service error behavior. |

## Existing Test Impact Matrix Appendix

The following TODO matrix homes are intentionally carried into the planning
artifacts so another engineer can see the full audit surface without chasing
the source TODO again. This appendix is documentation-only; `044-05-PLAN.md`
and `044-TESTS-TASKS.md` still control the update-or-no-change decision in the
final regression wave.

```text
crates/z00z_wallets/src/tx/selection/test_asset_selector_multi_suite.rs
crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs
crates/z00z_wallets/src/stealth/output/test_output.rs
crates/z00z_wallets/src/stealth/output/test_output_extra.rs
crates/z00z_wallets/src/stealth/output/test_facade_zkpack_suite.rs
crates/z00z_wallets/src/backup/crypto/wallet_backup.rs
crates/z00z_wallets/tests/test_rpc_types_serialization.rs
crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs
crates/z00z_wallets/tests/test_backup_restore_identity.rs
crates/z00z_wallets/tests/test_backup_metadata_policy.rs
crates/z00z_wallets/tests/test_backup_kdf_contract.rs
crates/z00z_wallets/tests/test_redb_wlt_open.rs
crates/z00z_wallets/src/db/redb/tests/redb_wallet_store.rs
crates/z00z_wallets/src/db/backends/wallet_store.rs
crates/z00z_wallets/src/db/redb/tests/test_redb_wallet_crypto_suite.rs
crates/z00z_wallets/src/db/redb/tests/test_storage_backend_suite.rs
crates/z00z_wallets/src/db/redb/tests/test_index_codecs_suite.rs
crates/z00z_wallets/tests/test_wlt_validator.rs
crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs
crates/z00z_wallets/tests/test_key_manager_storage_unlock.rs
crates/z00z_wallets/src/key/manager/test_key_manager_password_suite.rs
crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs
crates/z00z_wallets/src/services/app/test_app_service_suite.rs
crates/z00z_wallets/tests/test_app_service_create_wallet.rs
crates/z00z_wallets/tests/test_create_wallet_crypto_e2e.rs
crates/z00z_wallets/tests/test_deterministic_derivation_across_restarts.rs
crates/z00z_wallets/tests/test_phase2_production_hardening.rs
crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs
crates/z00z_wallets/src/wallet/snapshot/test_snapshot_suite.rs
crates/z00z_wallets/src/adapters/rpc/methods/test_storage_impl_suite.rs
crates/z00z_wallets/tests/test_tx_fee.rs
crates/z00z_wallets/src/tx/fees/test_fee_estimator_suite.rs
crates/z00z_wallets/src/tx/fees/fee_estimator.rs
crates/z00z_wallets/tests/test_tx_serial.rs
crates/z00z_wallets/tests/test_tx_digest_framing.rs
crates/z00z_wallets/src/tx/ids/tx_id.rs
crates/z00z_wallets/src/tx/ids/pay_ref.rs
crates/z00z_wallets/src/tx/multi_io.rs
crates/z00z_wallets/tests/test_tx_stealth_flow.rs
crates/z00z_wallets/src/tx/spend/spend_rules.rs
crates/z00z_wallets/src/tx/spend/spending.rs
crates/z00z_wallets/src/tx/spend/witness_gate.rs
crates/z00z_wallets/src/tx/claim/test_claim_tx.rs
crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl_proof.rs
crates/z00z_wallets/src/claim/test_nullifier_store.rs
crates/z00z_wallets/src/receiver/scan/test_stealth_scanner.rs
crates/z00z_wallets/src/receiver/scan/stealth_scanner/test_stealth_scanner.rs
crates/z00z_wallets/src/receiver/scan/test_stealth_scan_support_suite.rs
crates/z00z_wallets/src/receiver/manager/test_receiver_manager_suite.rs
crates/z00z_wallets/src/receiver/manager/test_canonical_snapshot_suite.rs
crates/z00z_wallets/src/receiver/request/test_stealth_request.rs
crates/z00z_wallets/src/receiver/card/test_stealth_card.rs
crates/z00z_wallets/src/receiver/card/test_stealth_trust_suite.rs
crates/z00z_wallets/src/key/receiver/test_stealth_keys_suite.rs
crates/z00z_wallets/tests/test_phase040_spend_proof_support.rs
crates/z00z_wallets/tests/test_phase14_pipeline.rs
crates/z00z_wallets/tests/test_phase24_gate.rs
crates/z00z_wallets/tests/test_s5_sender_examples.rs
crates/z00z_wallets/tests/test_s5_spec6_bridge.rs
crates/z00z_wallets/tests/test_s6_recv_examples.rs
crates/z00z_wallets/tests/test_spend_statement.rs
crates/z00z_wallets/tests/test_spend_witness_gate.rs
crates/z00z_wallets/tests/test_tx_pass.rs
crates/z00z_wallets/tests/test_tx_poison.rs
crates/z00z_wallets/tests/test_tx_spent_gate.rs
crates/z00z_core/src/assets/test_asset_suite.rs
crates/z00z_core/src/assets/test_registry_suite.rs
crates/z00z_core/tests/assets/test_integration_assets_test13.rs
crates/z00z_storage/tests/test_serialization_restore.rs
crates/z00z_simulator/tests/test_claim_persist.rs
crates/z00z_simulator/tests/test_claim_post.rs
crates/z00z_simulator/tests/test_claim_snapshot.rs
crates/z00z_simulator/tests/test_e2e_phase4.rs
crates/z00z_simulator/tests/test_scenario1_stage_surface.rs
crates/z00z_simulator/tests/test_stage4_output_crypto.rs
```

## Proposed New Test Files

None required for the base Phase 044 plan.

If a future implementation introduces a new seam that cannot truthfully live
inside the existing anchors above, prefer extending one of those homes before
creating a new test file.

## Test File Placement

| Scenario ID | Test File Path | Extend Or Create | Why This Is The Correct Home |
| --- | --- | --- | --- |
| `044-SC-01` | `044-coverage.md` plus source-shape guard commands in `044-05-PLAN.md` | Extend | Coverage completeness and drift-bar enforcement are closeout diagnostics, not a new runtime seam. |
| `044-SC-02` | `crates/z00z_wallets/src/persistence/assets/test_asset_storage_impl_suite.rs`, `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`, `crates/z00z_wallets/src/tx/selection/test_asset_selector_suite.rs` | Extend | Asset lifecycle, atomic reserve/release, and selectability all live on the asset ledger seam. |
| `044-SC-03` | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`, `crates/z00z_wallets/tests/test_stealth_output.rs`, `crates/z00z_wallets/src/tx/verify/test_tx_verifier_suite.rs` | Extend | Sender build/send correctness belongs where selected inputs, outputs, proofs, and verification meet. |
| `044-SC-04` | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_body.rs`, `test_tx_pending_body.rs`, `test_tx_history_cursor_filters.rs`, `test_tx_history_receipt_sort.rs`, `test_tx_impl_body.rs`, `test_tx_impl_suite.rs` | Extend | History, pending, details, cursor, and receipt visibility are all RPC-facing journal projections. |
| `044-SC-05` | `crates/z00z_wallets/src/services/wallet/tests/test_wallet_paths_suite.rs`, `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_wallet_impl_suite.rs`, `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs` | Extend | Wallet-stem naming and `.wlt` versus sidecar isolation belong to wallet service and wallet RPC homes. |
| `044-SC-06` | `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | Extend | JSONL rewrite/fold behavior belongs to the storage implementation and its integration seam. |
| `044-SC-07` | `crates/z00z_wallets/src/adapters/rpc/methods/test_backup_impl_suite.rs`, `crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs`, `crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs`, `crates/z00z_wallets/src/backup/crypto/test_wallet_backup_suite.rs`, `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs` | Extend | Forensic backup/restore byte preservation and tamper rejection belong to the backup stack and service roundtrip. |
| `044-SC-08` | `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`, `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | Extend | Legacy directory migration and source-shape guards belong where history storage and wallet path derivation meet. |
| `044-SC-09` | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs`, `tx_impl_server_lifecycle.rs`, `asset_impl_server_transfer.rs`, `crates/z00z_wallets/tests/test_tx_parity.rs`, `test_tx_roundtrip.rs`, `test_tx_tamper.rs`, `test_tx_wrong_root.rs` | Extend | Portable export/import and role-neutral submission belong to the same canonical tx bytes and tx hash. |
| `044-SC-10` | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs`, `crates/z00z_wallets/src/tx/state/test_state_update_suite.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | Extend | Admission receipts and confirmation evidence belong to broadcast and checkpoint-backed reconciliation. |
| `044-SC-11` | `crates/z00z_wallets/src/tx/state/test_state_update_suite.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs`, `crates/z00z_wallets/tests/test_tx_balance.rs` | Extend | Storage-backed reconciliation and its balance effects belong to the state-update and storage seams. |
| `044-SC-12` | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`, `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`, `crates/z00z_wallets/tests/test_direct_tx_receive.rs`, `crates/z00z_wallets/tests/test_e2e_req_flow.rs`, `crates/z00z_wallets/tests/test_tx_balance.rs` | Extend | Report-only preview, pending receive, and canonical persist-claim finalization belong to the receive and balance seams. |
| `044-SC-13` | `crates/z00z_wallets/tests/test_tx_balance.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`, `tx_impl_server_history.rs`, `tx_impl_server_lifecycle.rs` | Extend | Public `available` / `pending` balance truth is derived from lifecycle rows and surfaced by catalog/history RPCs. |
| `044-SC-14` | `crates/z00z_wallets/tests/test_spec_terms_guard.rs`, `test_tx_drift.rs`, `test_wallet_service_errors.rs`, `test_tx_store_integration.rs` | Extend | Source-shape guards and matrix completeness are the right home for stub, fake-success, pending-zero, and live-store regressions. |

## Required End-To-End Behaviors

| Behavior | Requirement | Primary Path | Pass Signal | Fail Signal |
| --- | --- | --- | --- | --- |
| Sender build is truthful | Selected inputs must be reserved before tx bytes are exposed, and real recipient/change/fee outputs must exist when required. | `AssetSelectorImpl` -> reservation -> `TxAssemblerImpl` -> `verify_full_tx_package(...)` | A built tx has real selected inputs, role-tagged outputs, and canonical bytes. | Stub bytes, empty detail rows, or failed verification leave reservations unprotected. |
| History is journal-backed | Details, pending lists, and history must read from tx journal rows and package bytes, not fabricated metadata. | `TxRecord` -> journal rows -> RPC history/details/pending | Built, submitted, failed, cancelled, and confirmed txs show truthful inputs, outputs, receipts, and status. | Empty inputs/outputs or fake pending zeros appear. |
| Live tx history has one canonical path | The live store is `wallet_<stem>_tx_history.jsonl`; `.wlt` remains wallet-state-only. | wallet stem -> sidecar JSONL -> storage reads | The sibling JSONL file is created and read as the live authority. | New writes target `wallet_<stem>_tx_history/` or put tx packages inside `.wlt`. |
| JSONL storage is deterministic | `put`, `update_status`, `delete`, `get`, `list`, and `list_by_status` must read and rewrite the canonical JSONL file deterministically and preserve exact package bytes. | canonical JSONL file -> read/modify/write -> history/details/pending views | Latest visible record matches the latest file state; malformed rows or hash mismatches fail closed. | Corrupt tails, hash mismatches, or partial rows are accepted. |
| Backup and restore preserve forensic history | Backups must store exact live JSONL bytes plus manifest, and restores must write JSONL bytes back without inventing rows. | live JSONL -> backup payload -> restore path | Restored JSONL bytes match exactly or are validated byte-for-byte. | Backup rebuilds from per-tx JSON, or restore invents rows. |
| Legacy per-tx JSON is regression-only | Existing `wallet_<stem>_tx_history/` directories are not live authority and must not be revived by runtime paths. | legacy dir shape -> source-shape guard -> JSONL authority | Legacy live-store strings are rejected and the simulator still proves the JSONL path. | New runtime writes return to per-tx JSON or the legacy dir becomes authoritative. |
| Portable export/import is role-neutral | Sender and receiver must submit the same canonical tx bytes through the same admission path. | portable package -> same `tx_hash` -> same admission trait | Role metadata stays audit-only; the bytes and hash stay stable. | Sender-only priority or a second tx schema appears. |
| Admission and confirmation are evidence-backed | RPC broadcast can only advance after admission trait evidence and stored receipts exist. | `WalletTxAdmitter` -> receipt -> checkpoint evidence | Admission and confirmation receipts are present and reconciled. | RPC-local fake success survives or no receipt is persisted. |
| Reconciliation is storage-authoritative | Pending and admitted rows move to final state only when typed checkpoint evidence matches journal expectations. | storage evidence -> `SpentEnt` / `CreatedEnt` -> final state | Matching evidence finalizes exactly once and is resumable. | Missing or wrong-root evidence advances state or double-applies. |
| Receive preview stays non-persistent | Report-only receive detects ownership but does not mutate claimed assets or balances. | `receive_asset_impl(...)` -> report-only response | Preview returns detection without storage mutation; `PersistClaim` finalizes once. | Report-only branches create claims or balances. |
| Balance derives from lifecycle truth | `available` and `pending` must be computed from lifecycle rows, not compatibility defaults. | lifecycle rows -> balance RPC | Pending rows produce non-zero pending and typed diagnostics when joins fail. | `pending = 0` masks unresolved lifecycle rows. |
| Regression guards are explicit | Stub, fake-success, empty-detail, and per-tx JSON drift must be rejected by source-shape guards and dedicated tests. | `rg` guards + drift tests | Forbidden patterns are absent or documented as legacy/test-only. | A forbidden pattern returns to the live path. |

## Critical Integration Paths

1. `AssetSelectorImpl` -> input reservation -> `TxAssemblerImpl` -> `verify_full_tx_package(...)` -> tx journal row.
2. `TxRecord` -> `wallet_<stem>_tx_history.jsonl` -> `TxStorageImpl` by-hash reads -> history/details/pending RPCs.
3. live JSONL bytes -> forensic backup payload -> restore path -> exact-byte-preserving sidecar.
4. legacy `wallet_<stem>_tx_history/` shape -> source-shape guard -> JSONL authority -> new writes only to JSONL.
5. portable package export/import -> same canonical tx bytes -> same admission trait -> role-neutral submit.
6. `WalletTxAdmitter` -> admission receipt -> checkpoint evidence -> storage-backed reconciliation.
7. `receive_asset_impl(...)` report-only branch -> no persistence; `recv_route(..., ReceiveNext::PersistClaim)` -> final claim.
8. lifecycle rows -> balance view -> typed diagnostics when row joins fail or pending rows remain unresolved.

## Input Fixtures And Preconditions

| Scenario ID | Inputs | Preconditions | Fixture Source |
| --- | --- | --- | --- |
| `044-SC-02`, `044-SC-03` | unlocked wallet, deterministic selected inputs, fee, recipient/change/output fixtures | wallet has at least one `Available` asset and no pre-existing reservation conflict | asset-storage, selector, send-body, and stealth-output test fixtures |
| `044-SC-04`, `044-SC-13` | journal rows, tx bytes, transaction statuses, timestamps, pagination filters | tx journal contains built, submitted, failed, cancelled, pending, and confirmed rows | history/body and store integration fixtures |
| `044-SC-05` | wallet stem, wallet id, output directory, simulated service open/create/import/export state | same stem must drive `.wlt` and JSONL sidecar names | wallet-path and wallet-service test fixtures |
| `044-SC-06` | JSONL rows with record hash, entry hash, previous hash, `tx_bytes`, and status transitions | storage root points at a single JSONL file and write lock is available | `test_tx_store_integration.rs` fixtures and inline `TxStorageImpl` tests |
| `044-SC-07`, `044-SC-08` | live JSONL bytes, backup manifest, legacy per-tx JSON directory, wallet identity, chain id | restore mode is explicit and migration is read-only unless writing JSONL | backup exporter/importer fixtures and wallet-service suite fixtures |
| `044-SC-09`, `044-SC-10`, `044-SC-11` | canonical tx package bytes, tx hash, admission receipt, checkpoint evidence, `SpentEnt`, `CreatedEnt` | sender/receiver submitters are configured and storage adapters are live | tx parity/roundtrip/tamper/wrong-root and state-update fixtures |
| `044-SC-12`, `044-SC-13` | report-only receive data, pending receive rows, lifecycle rows, wallet balances | receive route, claim persistence, and balance lookup seams are wired | asset RPC and wallet-service fixtures |
| `044-SC-14` | source tree, coverage ledger, TODO IDs, plan IDs, forbidden strings | test harness can execute `rg` inventory commands and the broad wallet release gate | TODO matrix, plan artifacts, and source-shape guard commands |

## Expected Outputs And Produced Artifacts

| Scenario ID | Expected Output | Persisted Artifact | Observable Signal |
| --- | --- | --- | --- |
| `044-SC-02` | one and only one successful reservation per input | asset lifecycle rows | one build succeeds, the conflicting one fails closed |
| `044-SC-03` | real selected inputs, role-tagged outputs, canonical package bytes | tx journal row plus send metadata | tx bytes appear only after reservation and verification |
| `044-SC-04` | truthful details, pending lists, and history | journal-backed RPC responses | no fabricated empty arrays, no fake pending zero |
| `044-SC-05` | canonical wallet stem naming | `wallet_<stem>.wlt` and `wallet_<stem>_tx_history.jsonl` | sidecar name matches stem, and `.wlt` stays wallet-state-only |
| `044-SC-06` | deterministic JSONL read/modify/write storage behavior | JSONL file and folded reads | latest visible row matches the latest valid file state |
| `044-SC-07` | exact JSONL backup/restore preservation | encrypted backup payload plus manifest | restored bytes match or tamper fails closed |
| `044-SC-08` | legacy live-store drift rejection | JSONL path and legacy-dir evidence | JSONL is authoritative and legacy live-store strings stay rejected |
| `044-SC-09` | portable, role-neutral tx bytes and tx hash | portable package and tx journal rows | sender and receiver submit the same bytes through the same path |
| `044-SC-10` | admission receipts and confirmation evidence | tx journal receipts and checkpoint evidence | RPC cannot report success without evidence |
| `044-SC-11` | final wallet state only when evidence matches | reconciled asset and tx rows | matching evidence finalizes once, missing evidence stays pending |
| `044-SC-12` | report-only preview and canonical persist-claim finalization | pending receive rows or no mutation on preview | preview stays non-persistent and finalization happens once |
| `044-SC-13` | accurate `available`, `pending`, and diagnostics | balance response and lifecycle views | pending rows remain visible until resolved |
| `044-SC-14` | forbidden shapes are rejected | source-shape guard notes and drift-test results | built stubs, fake success, empty details, and per-tx JSON drift are absent |

## Cryptographic And Security Invariants To Observe

| Invariant | Why it matters | Assertion shape |
| --- | --- | --- |
| Canonical tx bytes stay exact | The same bytes must survive build, journal, export, import, backup, and restore. | `tx_bytes` roundtrip compares byte-for-byte, not by semantic re-encoding. |
| `tx_hash`, `record_hash`, `entry_hash`, and `previous_entry_hash` stay bound | JSONL history must be tamper-evident. | Corrupted hash fields are rejected before fold or read. |
| `verify_full_tx_package(...)` is the package gate | Sender build and portable import must reject tampered or wrong-chain packages. | A tampered or wrong-root package fails before wallet state changes. |
| `WalletTxAdmitter` owns admission evidence | RPC broadcast may not fake acceptance locally. | Broadcast succeeds only when receipt evidence exists and is stored. |
| `SpentEnt` and `CreatedEnt` are required for finalization | Reconciliation must not invent confirmation. | Missing evidence leaves rows pending or quarantined. |
| `ReceiveNext::PersistClaim` is the only final receive route | Report-only receive must stay non-persistent. | Preview branches produce no claims or balance mutation. |
| `Available` is the only final spendable asset state | Compatibility labels must not become new authorities. | Tests reject `Validated` as spendable-state drift. |
| Legacy per-tx JSON is migration input only | The old directory shape must never regain live authority. | New writes never target `wallet_<stem>_tx_history/`. |
| Secrets never enter the live JSONL authority | JSONL must preserve forensic tx history, not wallet secrets. | Seed phrases, private blindings, and plaintext secret material do not appear in JSONL or summary artifacts. |

## Mermaid Flow

```mermaid
flowchart TD
  A[Available wallet assets] --> B[Reserve selected inputs]
  B --> C[Build canonical tx package]
  C --> D[verify_full_tx_package()]
  D --> E[Write tx journal row]
  E --> F[Append live JSONL history]
  F --> G[Backup / restore / migration / portable export]
  E --> H[WalletTxAdmitter]
  H --> I[Admission receipt]
  I --> J[Storage-backed reconciliation]
  J --> K[Receiver persist-claim or sender finalization]
  K --> L[Balance views from lifecycle rows]
  C --> M[Failure path]
  M --> N[Release reservations + typed failure]
```

## Clarifying Code Snippets

```rust
assert_eq!(wallet_snapshot_name(stem), format!("wallet_{stem}.wlt"));
assert_eq!(
    wallet_tx_history_name(stem),
    format!("wallet_{stem}_tx_history.jsonl")
);
```

```rust
assert_eq!(row.tx_bytes_hash, hash(&row.tx_bytes));
assert_eq!(
    row.entry_hash,
    hash(&(row.previous_entry_hash, row.record_hash))
);
```

## Scenario Matrix

| Scenario ID | Type | Goal | Positive Example | Negative Example | Main Assertions |
| --- | --- | --- | --- | --- | --- |
| `044-SC-01` | diagnostic | Coverage ledger and drift bars remain complete | every `EV-044-*`, `D-044-*`, `PH44-*`, `AC-044-*`, `T-044-*`, and `PT-044-*` row is mapped once | a missing ID, duplicate authority path, or stray vendor edit | inventory commands find all required IDs and no forbidden path appears |
| `044-SC-02` | integration | Asset reservation is atomic and safe | two build attempts on the same input result in one success and one conflict | both attempts reserve the same input or reservation masquerades as spend | only one reservation succeeds; release occurs on failure |
| `044-SC-03` | integration | Sender build/send is canonical | valid inputs produce real recipient/change/fee outputs and verifier-approved tx bytes | `BuiltTxStub`, fake outputs, or verification failure leaves tx bytes exposed | selected inputs, role-tagged outputs, and canonical package bytes are visible only after success |
| `044-SC-04` | integration | History, pending, and details are journal-backed | built/submitted/failed/cancelled/confirmed txs show truthful rows | empty arrays or fake zero pending appear for a live tx | inputs, outputs, receipts, and status match journal rows |
| `044-SC-05` | integration | Wallet-stem path contract stays canonical | `wallet_<stem>.wlt` and `wallet_<stem>_tx_history.jsonl` are emitted together | live writes target `wallet_<stem>_tx_history/` or tx packages are stored in `.wlt` | one stem yields exactly one snapshot path and one JSONL sidecar path |
| `044-SC-06` | unit / integration | JSONL storage rewrites deterministically | `put`, `update_status`, and `delete` preserve exact package bytes and folded reads return the latest view | malformed hashes, corrupt tails, or partial rows are accepted | exact `tx_bytes` survive and the latest file state stays authoritative |
| `044-SC-07` | E2E / scenario | Backup and restore preserve exact live JSONL bytes | archive validation and restore reproduce the same JSONL sidecar bytes | backup rebuilds from per-tx JSON or restore invents rows | byte-for-byte JSONL preservation and manifest verification hold |
| `044-SC-08` | E2E / scenario | Legacy per-tx JSON live-store drift is rejected | legacy per-tx JSON shapes are not treated as live authority | JSONL and legacy dir are merged automatically or legacy dir becomes authoritative | JSONL is authoritative when present and legacy remains a rejected compatibility shape |
| `044-SC-09` | E2E / scenario | Portable package export/import is role-neutral | sender and receiver submit the same canonical tx bytes through the same path | wrong-chain, tampered, or duplicate-conflict packages mutate balance | canonical bytes, tx hash, and idempotency stay stable across roles |
| `044-SC-10` | E2E / scenario | Admission and confirmation require evidence | broadcast stores admission receipts and confirmation matches checkpoint evidence | RPC-local fake success or missing receipts advance state | admission and confirmation are explicit, typed, and stored |
| `044-SC-11` | E2E / scenario | Reconciliation is storage-authoritative | matching `SpentEnt` and `CreatedEnt` evidence finalizes exactly once | missing or wrong-root evidence still finalizes or double-applies | pending rows stay pending until evidence matches journal expectations |
| `044-SC-12` | E2E / scenario | Receive preview stays non-persistent | report-only detects ownership without mutating claims, then `PersistClaim` finalizes once | preview writes claims or duplicate finalization succeeds | report-only and persist-claim remain behaviorally distinct |
| `044-SC-13` | integration | Public balance reflects lifecycle truth | `available` and `pending` match lifecycle rows | malformed pending rows are coerced to zero or hidden | diagnostics stay typed and pending rows never vanish silently |
| `044-SC-14` | diagnostic / regression | Source-shape guards block regressions | forbidden shapes stay absent or are explicitly labeled legacy/test-only | stub success, fake pending, empty details, or per-tx JSON live-store strings return to live paths | grep guards and drift tests catch the regression before completion |

## Canonical Commands

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_wallets --features test-fast --test test_asset_storage_impl_suite -- --nocapture`
- `cargo test -p z00z_wallets --features test-fast --test test_tx_send_body -- --nocapture`
- `cargo test -p z00z_wallets --features test-fast --test test_tx_history_body -- --nocapture`
- `cargo test -p z00z_wallets --features test-fast --test test_tx_store_integration -- --nocapture`
- `cargo test -p z00z_wallets --features test-fast --test test_backup_importer_suite -- --nocapture`
- `cargo test -p z00z_wallets --features test-fast --test test_tx_parity -- --nocapture`
- `cargo test -p z00z_wallets --features test-fast --test test_tx_balance -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- `rg -n "BuiltTxStub|pending = 0|inputs: vec!\\[\\]|outputs: vec!\\[\\]|wallet_tx_history_dir|collect_tx_history_records|format!\\(\\\"\\{tx_hash\\}\\.json\\\"\\)" crates/z00z_wallets/src crates/z00z_wallets/tests .planning/phases/044-wallet-assets`
- `rg -n "EV-044-|D-044-|PH44-|AC-044-|T-044-|PT-044-" .planning/phases/044-wallet-assets/044-coverage.md`

## Open Gaps

- The phase now has execution evidence, so the spec is no longer fallback-only.
- If a future scenario cannot be proved in the existing anchors listed above,
  record the gap in `044-coverage.md` rather than widening the phase into a
  parallel layer.
- `crates/z00z_crypto/tari/**` remains forbidden as a test target.
