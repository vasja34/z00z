---
phase: 046-wallet-addons
artifact: tests-tasks
status: planned
source: 046-TEST-SPEC.md
updated: 2026-05-14
phase_state: planning-active
---

# Phase 046 Test Tasks

## 🎯 Purpose

This file turns `046-TEST-SPEC.md` into a concrete implementation checklist.
The phase is still planning-active, so the list below preserves the split
between existing test homes that must be extended and planned Stage 13 homes
that must be created exactly where the numbered plans expect them.

The task list does not create new behavior. It tells a future engineer where
to land the proofs for the already-approved Phase 046 boundaries, what not to
duplicate, and which exact commands must close the phase honestly.

## Scope Inputs

- `046-TEST-SPEC.md`
- `046-wallet-addon-spec.md`
- `046-CONTEXT.md`
- `046-01-PLAN.md`
- `046-02-PLAN.md`
- `046-03-PLAN.md`
- `046-04-PLAN.md`
- `046-05-PLAN.md`
- `046-06-PLAN.md`

## Execution Strategy

- Freeze home ownership before writing new assertions so existing wallet seams
  stay extended rather than duplicated.
- Land Stage 13 contract scaffolding before Stage 13 full-behavior scenarios so
  simulator-owned proofs have a stable home.
- Extend existing wallet RPC, tx-store, backup, restore, and security homes
  before widening to Stage 13 end-to-end proofs.
- Keep Plan 05 as wording-only hygiene with residue scans, not runtime feature
  invention.
- Use focused commands first and exact release-style simulator commands last.

## Wave Overview

| Step | Primary homes | What to implement or extend | Done when |
| --- | --- | --- | --- |
| 046-01 | `046-TEST-SPEC.md`, `046-01-PLAN.md` through `046-06-PLAN.md` | Freeze the truthful home map before writing tests. Mark each home as existing or planned, preserve the planning-active state, and do not claim landed Stage 13 coverage before the Stage 13 files exist. | A reviewer can tell which assertions extend live tests today and which assertions will land in the planned Stage 13 files. |
| 046-02 | `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs` (planned), Stage 13 release smoke commands | Add Stage 13 contract and dispatch assertions proving the 13-stage contract, exact `S13-1` through `S13-15` ids, and canonical Scenario 1 routing. | Stage 13 is recognized as first-class by the runner, verifier, and release-style scenario path. |
| 046-03 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs` | Extend wallet.tx pending, cancel, reconcile, details, receipt, cursor, filter, and sort tests so the live RPC contract proves the Phase 046 lifecycle. | Wallet RPC tests cover reserve, cancel release, portable import, reconcile evidence, history traversal, and receipt projection without simulator shortcuts. |
| 046-04 | `crates/z00z_wallets/tests/test_tx_store_integration.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl/tests/mod.rs`, `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs` (planned), `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tamper.rs` (planned) | Add the root-binding, imported/exported marker, artifact-boundary, and tamper fail-closed assertions. Keep claimed-asset mutation and root vocabulary honest on both wallet and simulator paths. | Imported/exported evidence stays canonical, root fields stay distinct, and tamper never mutates claimed assets or emits fake post-commit artifacts. |
| 046-05 | `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`, `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`, `crates/z00z_wallets/src/services/wallet_service_tests.rs` | Extend backup, restore, JSONL replay, and restart coverage so `.wlt`, canonical JSONL, `WalletPlusHistory`, fail-closed restore, and `recv_range(...)` restart are proven on the live wallet boundary. | Restore and restart tests prove both persistence planes, all-or-nothing commit semantics, and wallet-side scan resume. |
| 046-06 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_tests.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`, `crates/z00z_wallets/src/services/wallet_service_tests.rs`, `crates/z00z_wallets/src/adapters/rpc/logging/summarize/tests.rs` | Extend payment request, TOFU, session hardening, rotate-master-key, and log-redaction coverage so the security boundary is proven before Stage 13 integrates it. | The live wallet and RPC tests prove request validation, TOFU gating, session limits, rotation auth, audit behavior, and secret-clean logging. |
| 046-07 | Focused residue scans over the Plan 05 touched files | Run wording-only hygiene validation. Remove stale `stub`, `placeholder`, `Phase 1`, `residue`, `JWT`, false ledger-authority, and durable seed-rotation wording without adding fake runtime logic. | The forbidden wording no longer appears on the touched wallet, RPC, backup, or scanner files and the replacements match the Phase 046 boundary text exactly. |
| 046-08 | `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs` (planned), `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tamper.rs` (planned) | Land the named Stage 13 regression inventory from `046-06-PLAN.md`: lifecycle, storage-contract, receiver continuation, tamper, backup or restore, payment request negatives, session negatives, rotation negatives, and full history-marker coverage. | The planned Stage 13 test files own the end-to-end simulator proof of the complete Phase 046 story. |
| 046-09 | Existing wallet homes plus the exact release simulator commands | Run the focused wallet and simulator validation waves in the same order as the numbered plans, then run the exact release simulator commands. | The future engineer has recorded green focused outputs and green exact release outputs before calling the phase complete. |
| 046-10 | `046-06-SUMMARY.md` (future closeout artifact) plus final Rust gates | Close the test work honestly: summarize which existing homes were extended, which planned homes were created, which hygiene scans passed, and which exact commands were green. Then run `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test --all`, and `cargo doc --no-deps` if public Rust docs changed. | Closeout evidence, final command outputs, and phase wording all agree on the same implemented scope with no parallel artifact chain. |

## Exact Per-File Landing Execution Plan

This section turns the top-level waves into an exact file-first landing order.
The preferred test names are the same planned landing names defined in
`046-TEST-SPEC.md`. Existing anchors may absorb the behavior under a nearby
existing function name only when that avoids duplicate coverage and keeps the
behavior mapping one-to-one.

| Wave | File | Extend or Create | Preferred future test functions | Why this file owns them |
| --- | --- | --- | --- | --- |
| T2 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs` | Extend | `test_tx_build_reserves_then_cancel_releases`; `test_tx_reconcile_updates_claimed_assets`; `test_tx_reconcile_rejects_bad_evidence_without_asset_mutation`; `test_tx_import_rejects_wrong_chain_without_history_mutation`; `test_tx_history_filters_pending_cancelled_confirmed`; `test_tx_export_import_detects_receiver_owned_outputs` | This is the canonical live wallet.tx pending, cancel, import, export, and reconcile seam. |
| T2 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs` | Extend | `test_tx_history_cursor_roundtrip_preserves_status_markers`; `test_tx_history_filter_pending_cancelled_confirmed` | Cursor and filter traversal must stay on the wallet RPC history seam. |
| T2 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs` | Extend | `test_tx_history_sort_roundtrip_keeps_receipt_projection`; `test_tx_history_receipt_projection_survives_sort` | Receipt projection and sort semantics must stay on the wallet RPC history seam. |
| T2 to T4 | `crates/z00z_wallets/tests/test_tx_store_integration.rs` | Extend | `test_tx_history_jsonl_replay_preserves_import_export_markers`; `test_tx_history_artifact_paths_stay_distinct_after_reconcile` | Canonical JSONL replay and artifact-boundary proof are integration-owned here. |
| T3 | `crates/z00z_wallets/src/services/wallet_service_tests.rs` | Extend | `test_claimed_assets_restore_from_wlt_snapshot`; `test_wallet_plus_history_restore_keeps_tx_jsonl`; `test_restore_wrong_password_rejects_without_wallet_mutation_service_boundary`; `test_recv_range_restart_reuses_persisted_scan_state_after_restore` | Restore, restart, and wallet-owned receive authority stay on the service boundary. |
| T3 | `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs` | Extend | `test_claimed_assets_checksum_tamper_rejected`; `test_wallet_plus_history_import_rejects_corrupt_history_before_restore` | Importer-only checksum, decode, and mode failures belong to the importer seam. |
| T3 | `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/tests.rs` | Extend | `test_restore_wrong_password_rejects_without_wallet_mutation`; `test_backup_rpc_restore_preserves_wallet_plus_history_boundary` | The live RPC restore surface owns wrong-password and WalletPlusHistory RPC coverage. |
| T4 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs` | Extend | `test_payment_request_negative_paths_reject_before_build`; `test_rotate_master_key_rpc_boundary_requires_auth_and_rate_limit` | Payment request validation and rotate-master-key auth are live key RPC boundaries. |
| T4 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_tests.rs` | Extend | `test_send_tofu_confirmation_required_for_relabelled_receiver_card` | TOFU drift on the send path already lives on the asset implementation seam. |
| T4 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs` | Extend | `test_send_tofu_confirmation_required_for_changed_view_or_identity_material` | Companion TOFU and identity-material drift coverage stays on the existing asset RPC seam. |
| T4 | `crates/z00z_wallets/src/adapters/rpc/logging/summarize/tests.rs` | Extend | `test_rotate_master_key_logs_remain_redacted`; `test_wallet_tx_session_failures_log_without_secrets` | Redaction assertions belong to the summarizer seam, not to simulator glue. |
| T1 then T5 | `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs` | Create | `test_stage13_contract_declares_exact_s13_step_ids`; `test_stage13_dispatch_is_registered_in_runner_and_verifier`; `test_stage13_runs_wallet_tx_rpc_lifecycle`; `test_stage13_storage_contract_matches_prev_root_and_post_store`; `test_stage13_receiver_import_continuation_uses_receiver_path`; `test_stage13_backup_restore_compares_claimed_assets_and_history`; `test_stage13_payment_request_negative_paths`; `test_stage13_session_hardening_negative_paths`; `test_stage13_rotate_master_key_auth_rate_limit_and_redaction`; `test_stage13_history_covers_statuses_and_import_export_markers` | This is the canonical planned Stage 13 positive and end-to-end scenario home. |
| T5 | `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tamper.rs` | Create | `test_stage13_rejects_tampered_tx_id_without_claimed_mutation`; `test_stage13_rejects_tampered_tx_hash_without_claimed_mutation`; `test_stage13_rejects_wrong_chain_id_without_claimed_mutation`; `test_stage13_rejects_zero_verified_block_height_without_claimed_mutation`; `test_stage13_rejects_bad_checkpoint_or_confirmed_root_without_mutation`; `test_stage13_rejects_mismatched_spent_or_created_asset_ids_without_mutation` | Stage 13 tamper branches need a dedicated fail-closed home so positive scenario coverage stays readable. |

### Landing Order Constraints

- land T2 wallet RPC tests before T5 Stage 13 positive flow so the simulator can
  reuse already-proven wallet behavior instead of embedding duplicate fixtures
- land T3 restore and replay tests before the Stage 13 backup or restore
  scenario so the simulator only mirrors the live wallet restore contract
- land T4 key, asset, session, and redaction tests before the Stage 13
  negative-path matrix so the simulator inherits live error names and not a
  simulator-only alias set
- keep the T5 `tamper.rs` file branch-local: one fail-closed cause per test
  function, one unchanged claimed-asset assertion family, one unchanged root or
  artifact assertion family

## Task Waves

### Wave T0: Harness And Truth Lock-In

- files to inspect:
  - `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs`
  - `crates/z00z_wallets/src/services/wallet_service_tests.rs`
  - `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
  - `crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
- deliverables:
  - explicit extend-versus-create decision for every `046-Sxx` scenario
  - explicit note that Stage 13 homes are planned, not landed
- completion gate:
  - no scenario still has an ambiguous home
  - no planned Stage 13 seam is described as already executable

### Wave T1: Stage 13 Contract Scaffold

- why now:
  - Stage 13 must have a stable simulator-owned home before full end-to-end
    behavior is added
- files to create:
  - `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs`
- implementation tasks:
  - add Stage 13 contract and dispatch assertions
  - prove exact `S13-1` through `S13-15` ids are wired through config and
    verifier surfaces
- completion gate:
  - Scenario 1 recognizes Stage 13 as first-class and the exact release command
    can target the new stage layout
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

### Wave T2: Wallet.tx Lifecycle And History

- files to extend:
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs`
  - `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- implementation tasks:
  - prove reserve then cancel release
  - prove reconcile mutates claimed assets only on valid evidence
  - prove imported or exported markers and history receipt traversal stay
    canonical
- completion gate:
  - pending, cancelled, confirmed, imported, and exported evidence are all
    distinct and no second tx lane appears
- command gate:
  - focused `cargo test -p z00z_wallets ...` for the touched wallet homes

### Wave T3: Backup, Restore, JSONL Replay, And Receive Restart

- files to extend:
  - `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
  - `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`
  - `crates/z00z_wallets/src/services/wallet_service_tests.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/tests.rs`
- implementation tasks:
  - prove `.wlt` and canonical JSONL remain separate persistence planes
  - prove `WalletPlusHistory` restore is all-or-nothing
  - prove restart resumes from persisted scan state on the wallet-owned path
- completion gate:
  - wrong password, tamper, missing archive, decode drift, and replay failure
    all reject before wallet mutation
- command gate:
  - focused wallet-service and backup command bundle from the numbered plans

### Wave T4: Payment Requests, TOFU, Session Hardening, And Rotation

- files to extend:
  - `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_tests.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`
  - `crates/z00z_wallets/src/services/wallet_service_tests.rs`
  - `crates/z00z_wallets/src/adapters/rpc/logging/summarize/tests.rs`
- implementation tasks:
  - prove signature, expiry, chain binding, and TOFU gates reject before tx
    construction
  - prove stale or locked sessions reject sensitive wallet paths
  - prove rotate auth, rate limit, audit outcome, and redaction remain live and
    in-memory only
- completion gate:
  - no payment, session, or rotation scenario leaks secrets or claims durable
    seed rotation
- command gate:
  - focused key, asset, wallet-service, and logging test commands

### Wave T5: Stage 13 Full Scenario And Tamper Matrix

- files to extend or create:
  - `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs`
  - `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tamper.rs`
- implementation tasks:
  - prove full wallet.tx lifecycle, receiver continuation, storage-contract
    root binding, backup or restore comparison, payment negatives, session
    negatives, rotation negatives, and full status-marker coverage
  - prove tamper rejects without claimed-asset mutation or fake post-commit
    artifacts
- completion gate:
  - Stage 13 owns the full end-to-end simulator proof of Phase 046 and keeps
    `prev_root`, `state_root`, and `flat_root` distinct
- command gate:
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump stage13`

### Wave T6: Hygiene And Release Closeout

- files or surfaces:
  - focused residue scans over the exact Plan 05 touched files
  - future `046-06-SUMMARY.md` closeout evidence
- implementation tasks:
  - remove stale wording without inventing runtime logic
  - run exact release-style simulator commands and final Rust gates
- completion gate:
  - focused residue scans are clean
  - exact simulator release commands are green
  - final closeout evidence agrees with implemented scope
- command gate:
  - `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features`
  - `cargo test --all`

## 🔎 Validation Rules

| Rule | Requirement |
| --- | --- |
| Planning truth | Keep existing and planned homes labeled honestly until the Stage 13 files land. |
| Bootstrap-first order | Start each implementation wave with `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`; if it fails, repair the local slice before any broader run. |
| Focused command order | Use the focused wallet or simulator commands already frozen in `046-01-PLAN.md` through `046-06-PLAN.md` before the final release smoke commands. |
| Exact simulator release commands | Use `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump` and `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` as the canonical release-style Scenario 1 gates. |
| Service-home discipline | Keep wallet service restore and restart coverage in `crates/z00z_wallets/src/services/wallet_service_tests.rs`; keep importer-only coverage in `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`; do not rewrite them into `crates/z00z_wallets/tests/` integration paths. |
| No parallel lanes | Do not add a second tx-history writer, a second claimed-asset store, a second scanner authority, a receiver-only tx lane, or a second root engine. |
| Wording-only discipline | Plan 05 is not a runtime feature wave. Validate it with focused residue scans and source-shape honesty checks, not with invented runtime DTOs or fake state transitions. |
| Evidence discipline | Future closeout evidence belongs in the existing numbered Phase 046 summary or validation chain. Do not create a second test spec, second tasks file, or parallel coverage ledger unless a later spec explicitly requires one. |

## 🧪 Implementation Notes

| Area | What the engineer should reuse | What must stay explicit |
| --- | --- | --- |
| Stage 13 simulator surface | Existing Scenario 1 stage patterns, the logged transport builder, and the current Stage 4 or Stage 6 structure as style guidance only | Stage 13 must prove the live `wallet.tx.*` lifecycle, not restyle the old simulator-only tx lane |
| Wallet tx lifecycle | `test_tx_pending_body.rs`, `test_tx_history_cursor_filters.rs`, `test_tx_history_receipt_sort.rs`, and `tx_impl/tests/mod.rs` | Pending reservation, cancellation release, reconcile evidence, imported/exported markers, and history traversal must remain on the live wallet RPC contract |
| Backup and restore | `test_wallet_export_pack_boundary.rs`, `backup_importer_impl/tests.rs`, `wallet_service_tests.rs`, `backup_impl/tests.rs` | `.wlt` claimed assets and canonical JSONL history are separate persistence planes and restore must stay all-or-nothing |
| Receive resume | `test_recv_range_restart` in `wallet_service_tests.rs` | Wallet-side `recv_range(...)`, `ScanStatePayload`, and `PersistClaim` remain the only receive-resume authority |
| Payment and security | `key_impl/tests.rs`, `asset_impl/asset_impl_tests.rs`, `asset_impl_tests.rs`, `wallet_service_tests.rs`, `logging/summarize/tests.rs` | Signature, expiry, chain binding, TOFU, session limits, rotation audit, and log redaction must remain live wallet boundaries |
| Hygiene | Focused no-match scans over the exact Plan 05 touched files | Wording fixes must not create or imply new runtime behavior |

## 📋 Focused Validation Checklist

These commands are the execution anchors already frozen by the numbered plans.
Run the narrow command for the current slice first; only then widen to the
release-style and full-workspace gates.

| Slice | Commands |
| --- | --- |
| Stage 13 contract and dispatch | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` then `cargo test --release --features test-fast --features wallet_debug_dump` |
| Wallet tx lifecycle and history | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` then focused `cargo test -p z00z_wallets ...` for the touched wallet homes |
| Simulator Stage 13 tamper and history | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` then `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump stage13` |
| Backup or restore and receive restart | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` then focused wallet-service and backup tests |
| Final release smoke | `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump` then `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` |
| Final closeout gate | `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test --all`, and `cargo doc --no-deps` if public Rust docs changed |

## ✅ Exit Conditions

The Phase 046 test task set is complete when all of the following are true:

| Condition | Pass signal |
| --- | --- |
| Truthful home ownership | Every scenario from `046-TEST-SPEC.md` maps to an existing extended home, a newly created planned Stage 13 home, or an explicit hygiene command. |
| Wallet lifecycle proof | Wallet RPC tests prove reserve, cancel, reconcile, imported/exported evidence, and history traversal on the live contract. |
| Restore and restart proof | Backup, restore, JSONL replay, and scan restart tests prove both persistence planes and all-or-nothing behavior. |
| Security proof | Payment request, TOFU, session hardening, rotation, and redaction tests prove the live wallet boundary without simulator-only aliases. |
| Stage 13 proof | The planned Stage 13 test files prove contract wiring, full lifecycle, receiver continuation, root binding, tamper fail-closed behavior, and status-marker completeness. |
| Hygiene proof | Focused residue scans over the Plan 05 files are clean and no wording-only slice was widened into fake runtime behavior. |
| Release proof | The exact simulator release commands and the final Rust gates are green on the landed implementation. |

If a required regression cannot be supported by the current homes, record the
gap in the future Phase 046 closeout artifacts rather than widening the phase
with a parallel seam or a second documentation chain.
