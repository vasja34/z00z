---
phase: 043-gaps-fixes
doc: 043-tests-tasks
status: evidence-synced
created: 2026-05-06
updated: 2026-05-08
owner: Z00Z Wallets and Storage
scope: execution checklist for Phase 043 test coverage, including additive spec-2 E2E coverage
---

# Phase 043 Test Tasks

## 🎯 Purpose

This file turns [043-TEST-SPEC.md](./043-TEST-SPEC.md) into a concrete implementation checklist. The original order follows `043-TODO.md`; the additive spec-2 test slice follows `043-fixes-spec-2.md`, `043-TODO-2.md`, and plans `043-11` through `043-18`. The file names follow the current test homes already present in the tree.

The task list does not create new behavior. It extends the existing test homes, keeps skip reservations explicit, and records any gap back into `043-coverage.md` rather than widening Phase 043.

## 🧭 Ordered Task List

| Step | Primary homes | What to implement or extend | Done when |
| --- | --- | --- | --- |
| 043-01 | `043-coverage.md` only | Freeze the phase ledger, keep every EV/PH43/D-043/AC-043 row mapped to one owner file, one test home, one evidence slot, and one status cell. Do not add runtime tests for planning-only artifacts. | The coverage ledger is complete and any out-of-scope families stay excluded. |
| 043-02 | [test_tx_balance.rs](../../../crates/z00z_wallets/tests/test_tx_balance.rs), [test_tx_tamper.rs](../../../crates/z00z_wallets/tests/test_tx_tamper.rs), [test_tx_fee.rs](../../../crates/z00z_wallets/tests/test_tx_fee.rs), [test_stealth_request.rs](../../../crates/z00z_wallets/tests/test_stealth_request.rs) | Extend the assembler and request contract cases so they prove resolved-input balance, fee-output inclusion, tamper rejection, request validation, and explicit ToFU/expiry/chain binding. | A reviewer can point to one valid positive case and one negative case for every assembler/request rule. |
| 043-03 | [test_claim_source_proof.rs](../../../crates/z00z_storage/tests/test_claim_source_proof.rs), [test_tx_pedersen.rs](../../../crates/z00z_wallets/tests/test_tx_pedersen.rs), [test_spend_proof_backend.rs](../../../crates/z00z_wallets/tests/test_spend_proof_backend.rs), [test_tx_wrong_root.rs](../../../crates/z00z_wallets/tests/test_tx_wrong_root.rs) | Extend the storage and conservation cases so they keep membership separate from conservation, preserve distinct tamper classes, and keep operator audit explicit. | Membership, wrong-root/path, conservation, and audit mismatch failures are all observable as distinct assertions. |
| 043-04 | [test_wallet_export_pack_boundary.rs](../../../crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs), [test_tx_store_integration.rs](../../../crates/z00z_wallets/tests/test_tx_store_integration.rs), [test_redb_wlt_open.rs](../../../crates/z00z_wallets/tests/test_redb_wlt_open.rs), [test_wallet_service_suite.rs](../../../crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs), backup exporter/importer/crypto suites | Extend archive tests so they prove encrypted export, restore isolation, metadata integrity, and non-mutating rejection of tampered or mismatched archive payloads. | The canonical wallet snapshot stays separate from the forensic envelope and failures leave restore state unchanged. |
| 043-05 | [test_stealth_scanner_flow.rs](../../../crates/z00z_wallets/tests/test_stealth_scanner_flow.rs), [test_import_error_taxonomy.rs](../../../crates/z00z_wallets/tests/test_import_error_taxonomy.rs), [test_runtime_validation_result.rs](../../../crates/z00z_wallets/tests/test_runtime_validation_result.rs) | Extend receive/report cases so they distinguish typed opening semantics, precise internal reject classes, and outward compatibility mapping. | `ReceiveStatus::InvalidProof` remains outward-only while internal reject classes stay precise. |
| 043-06 | [test_stealth_scanner_cache.rs](../../../crates/z00z_wallets/tests/test_stealth_scanner_cache.rs), [test_stealth_scanner_prefilter.rs](../../../crates/z00z_wallets/tests/test_stealth_scanner_prefilter.rs) | Extend tag-cache and prefilter cases so strict tag-only mode requires completeness, request liveness stays separate from completeness, and direct-scan fallback remains available. | `TagFilterOnly` cannot be justified by size or liveness alone, and fallback still works when completeness is absent. |
| 043-07 | [test_stealth_output.rs](../../../crates/z00z_wallets/tests/test_stealth_output.rs), [test_live_path_enforcement.rs](../../../crates/z00z_wallets/tests/test_live_path_enforcement.rs), [test_e2e_send_scan.rs](../../../crates/z00z_wallets/tests/test_e2e_send_scan.rs) | Extend sender/output tests so validated builders own approved flows, raw builders remain explicit seams, and live RPC plus simulator entrypoints cannot regress to raw-builder use. | Approved flows route through validated builders, and the live path fails if a raw-builder regression reappears. |
| 043-08 | `043-coverage.md` plus the tx/conservation homes | Re-run the narrow tx and conservation regressions in the exact TODO order and update the coverage ledger with the decisive anchors. | Every tx/conservation row points to one named regression home and the typed failure classes remain visible. |
| 043-09 | `043-coverage.md` plus the receive/tag/output homes | Re-run the narrow receive, tag, and output regressions in the exact TODO order and update the coverage ledger with the final anchors. | Every EV-008 through EV-014 row points to one final regression home and no approved-flow or strict-tag regression is left on prose alone. |
| 043-10 | `043-coverage.md`, `043-SUMMARY.md`, archive closeout homes, simulator gates | Close the archive envelope and the phase honestly, then capture final evidence, residual risks, and explicit deferrals. | The summary and coverage ledger agree with the decisive command outputs and the final simulator truth gates. |
| 043-11 | [test_tx_pedersen.rs](../../../crates/z00z_wallets/tests/test_tx_pedersen.rs), [test_spend_proof_backend.rs](../../../crates/z00z_wallets/tests/test_spend_proof_backend.rs), fallback [test_tx_wrong_root.rs](../../../crates/z00z_wallets/tests/test_tx_wrong_root.rs), fallback [test_tx_balance.rs](../../../crates/z00z_wallets/tests/test_tx_balance.rs) | Add typed audit outcome tests for `AssetClassAuditTarget`, `AssetClassAuditStatus`, `AssetClassAuditMismatchClass`, and `AssetClassAuditOutcome`; preserve public verifier honesty. | Pass and fail-closed audit paths expose target, status, mismatch class, and entry index where applicable. |
| 043-12 | [test_wallet_service_suite.rs](../../../crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs), [test_redb_wlt_open.rs](../../../crates/z00z_wallets/tests/test_redb_wlt_open.rs) | Add wallet-stem and canonical filename tests for snapshot, JSONL, and live tx-history dir; keep `.wlt` restore semantics wallet-state-first. | Canonical names share the same stem; legacy `tx_history_<stem>.jsonl` stays non-canonical. |
| 043-13 | [test_wallet_export_pack_boundary.rs](../../../crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs), [test_backup_importer_suite.rs](../../../crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs), [test_tx_store_integration.rs](../../../crates/z00z_wallets/tests/test_tx_store_integration.rs), [test_wallet_service_suite.rs](../../../crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs) | Add required canonical JSONL export, replay/import, tamper rejection, import-mode gating, and full-field `TxRecord` view tests. | JSONL is required, colocated, hash-bound, replayable, and fail-closed before mutation on malformed or tampered input. |
| 043-14 | [test_tx_store_integration.rs](../../../crates/z00z_wallets/tests/test_tx_store_integration.rs), [test_wallet_service_suite.rs](../../../crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs), [tx_rpc_storage.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs) | Add path-boundary tests proving archive path, snapshot, canonical JSONL, live tx-history directory, and `outputs/tx_exports` stay distinct. | No artifact path is reused for another semantic role. |
| 043-15 | `043-coverage.md`, `043-SUMMARY.md`, spec-2 test homes | Map every spec-2 regression to a named anchor or explicit spec-backed deferral; run redaction gates. | Coverage and summary carry all spec-2 test evidence without leaking plaintext secrets. |
| 043-16 | `043-coverage.md`, `043-SUMMARY.md`, `043-TODO-2.md` | Close the spec-2 slice against exact validation commands and residual-risk notes. | Spec, code, tests, coverage, and summary agree on the canonical JSONL and live tx-store/RPC boundaries. |
| 043-17 | [test_wallet_export_pack_boundary.rs](../../../crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs), [test_backup_importer_suite.rs](../../../crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs), [test_tx_store_integration.rs](../../../crates/z00z_wallets/tests/test_tx_store_integration.rs), [test_wallet_service_suite.rs](../../../crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs), [test_redb_wlt_open.rs](../../../crates/z00z_wallets/tests/test_redb_wlt_open.rs), [test_rpc_types_serialization.rs](../../../crates/z00z_wallets/tests/test_rpc_types_serialization.rs) | Implement the spec-2 E2E scenario set `SPEC2-E2E-01` through `SPEC2-E2E-09` from [043-TEST-SPEC.md](./043-TEST-SPEC.md), reusing existing homes unless a focused new file is justified. | Every spec-2 behavior has a positive and negative runtime assertion in an existing truthful home or an explicit deferral. |
| 043-18 | `043-coverage.md`, `043-SUMMARY.md`, [043-TEST-SPEC.md](./043-TEST-SPEC.md), [043-TESTS-TASKS.md](./043-TESTS-TASKS.md) | Run the spec-2 E2E evidence wave, update ledger and summary, satisfy `SPEC2-E2E-10`, prove no parallel closeout artifact exists, and re-run redaction/hygiene gates. | `043-18` is the final E2E evidence sync for the additive spec-2 test slice, with closeout evidence carried only by the existing ledger and summary. |

## 🔎 Validation Rules

| Rule | Requirement |
| --- | --- |
| Command order | Use the validation order already frozen in `043-TODO.md`, then apply the additive `043-TODO-2.md` and `043-17`/`043-18` gates without weakening the narrow gates. |
| Scope discipline | Keep edits inside the existing test homes unless the spec explicitly says a new home is allowed. |
| Skip reservations | Do not create [test_wallet_json_export.rs](../../../crates/z00z_wallets/tests/test_wallet_json_export.rs) unless archive serialization ownership moves. Do not create [test_tx_stealth_flow.rs](../../../crates/z00z_wallets/tests/test_tx_stealth_flow.rs) unless sender-flow ownership moves. Do not create `043-TEST-SPEC-2.md`, `043-TESTS-TASKS-2.md`, `043-coverage-2.md`, or `043-SUMMARY-2.md`. |
| Boundary discipline | Do not add any `crates/z00z_crypto/tari/**` test, helper, or fixture. |
| Evidence discipline | If a scenario cannot be proven by the current homes, record the gap in `043-coverage.md` and keep the phase boundary narrow. |

## 🧪 Implementation Notes

| Area | What the engineer should reuse | What must stay explicit |
| --- | --- | --- |
| Tx assembly | Existing balance, tamper, fee, and request fixtures | Resolved-input evidence, fee-output inclusion, and typed failure classes |
| Storage/conservation | Existing proof, claim-source, spend-proof, and root-tamper fixtures | Membership vs conservation separation, and operator-invoked audit |
| Archive | Existing backup exporter/importer/crypto suite, wallet service restore suite, and wallet open fixtures | Optional forensic envelope, non-mutating reject paths, and redacted/hash-bound evidence |
| Receive/tag/output | Existing scanner, import taxonomy, tag cache, and live-path fixtures | Compatibility mapping, completeness gating, and validated-builder routing |
| Spec-2 archive/JSONL | Existing backup export/import, wallet service, tx-store, RedB open, and RPC serialization fixtures | Required wallet-prefixed JSONL, full-field replay, explicit import mode, no persisted forensic toggle, and no artifact-path collapse |

## ✅ Exit Conditions

The task set is complete when:

| Condition | Pass signal |
| --- | --- |
| Coverage sync | `043-coverage.md` names one decisive anchor for every required row or an explicit spec-backed deferral. |
| Scenario coverage | Each journey in [043-TEST-SPEC.md](./043-TEST-SPEC.md) has at least one positive and one negative example in the relevant home. |
| Regression order | The 043-08, 043-09, and 043-10 waves stay in the exact TODO order. |
| Scope control | No new parallel layer, no vendor edit, and no surprise test home were introduced. |
| Spec-2 E2E sync | Plans `043-17` and `043-18` landed the additive E2E assertions and updated the existing coverage/summary artifacts only. |
