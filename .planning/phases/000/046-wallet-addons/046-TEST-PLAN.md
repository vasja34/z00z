---
phase: 046-wallet-addons
artifact: test-plan
status: compatibility-alias
source: 046-TEST-SPEC.md, 046-TESTS-TASKS.md
updated: 2026-05-14
---

# Phase 046 Test Plan

## 🎯 Purpose

This file is the compatibility handoff for workflows that explicitly request
`046-TEST-PLAN.md`.

The canonical planning artifacts remain:

- `046-TEST-SPEC.md`
- `046-TESTS-TASKS.md`

This document does not introduce a third divergent plan. It summarizes the
approved planning state, the classification outcome, the current workflow stop
gate, and the exact next execution boundary for future test implementation.

## ⚠️ Workflow Status

Phase 046 add-tests planning is complete at the specification level only.

- the add-tests workflow normally targets completed phases backed by
  `*-SUMMARY.md`, but Phase 046 is still planning-active under
  `.planning/phases/046-wallet-addons/`
- the compatibility adaptation for this phase is planning-first and does not
  claim RED-GREEN execution, passing test files, or implementation-backed
  coverage that has not landed yet
- `046-CONTEXT.md`, `046-wallet-addon-spec.md`, and `046-01-PLAN.md` through
  `046-06-PLAN.md` are the governing source packet
- `046-TEST-SPEC.md` now freezes the classification result, required
  invariants, journey matrix, regression inventory, and completion criteria
- `046-TESTS-TASKS.md` now freezes the ordered implementation checklist,
  validation rules, exact release-style commands, and exit conditions
- Phase 046 remains planning-derived, not verification-backed
- planned simulator homes under
  `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/` do not yet exist
  and must not be described as already-landed evidence

## 📦 Classification Outcome

### TDD And Integration Targets

- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs`
  Canonical payment-request and rotate-master-key RPC boundary.
- `crates/z00z_wallets/src/services/wallet_service_tests.rs`
  Canonical wallet-owned restore, restart, session, and post-rotate lifecycle
  boundary.
- `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`
  Canonical importer mode, checksum, version, and chain-preservation seam.
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
  Canonical live wallet.tx lifecycle seam for reserve, cancel, import, export,
  reconcile, and receipt assertions.
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs`
  History cursor and filter traversal seam.
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs`
  History sort and receipt projection seam.
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl/tests/mod.rs`
  Shared wallet.tx fixture and helper seam named directly by Phase 046 plans.
- `crates/z00z_wallets/src/adapters/rpc/logging/summarize/tests.rs`
  Redaction and secret-clean logging seam.

### E2E Targets

- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
  Forensic export, canonical JSONL requirement, and secret-clean archive
  boundary.
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`
  JSONL replay, artifact separation, imported/exported evidence, and canonical
  tx history storage seam.
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/tests.rs`
  Backup RPC roundtrip and wrong-password restore boundary.
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs` (planned)
  Canonical Stage 13 simulator lifecycle, storage-contract, restore, payment,
  session, rotate, and status-marker seam.
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tamper.rs` (planned)
  Canonical Stage 13 fail-closed tamper matrix seam.

### Skip Targets

- browser automation or UI E2E
  not part of the wallet-addons phase contract
- `crates/z00z_crypto/tari/**`
  read-only vendor code
- any second tx-history lane, second claimed-asset store, second scan
  authority, or receiver-only lifecycle fork
  forbidden by the Phase 046 architectural boundary
- wording-only runtime expansion for Plan 05
  the correct proof is source-shape and residue validation, not synthetic new
  execution behavior

## ✅ Existing Anchors To Reuse

- `test_create_payment_request_ok`
- `test_rejects_bad_payment_id`
- `test_validate_req_ok`
- `test_validate_req_bad_compact`
- `test_validate_req_big_payload`
- `test_rotate_master_key_stub`
- `test_master_rejects_bad_password`
- `test_rejects_non_literal_confirmation`
- `test_recv_range_restart`
- `test_stays_live_post_rotate`
- `test_unlock_attempt_precheck_rate`
- `test_unlock_attempt_precheck_enforces`
- `test_string_kdf_restore_fails`
- `test_restore_backup_with_wallet_plus_history_imports_tx_store`
- `test_restore_backup_with_tx_history_only_imports_without_wallet_restore`
- `test_restore_backup_with_wallet_plus_history_rejects_tampered_forensic_archive_without_wallet_mutation`
- `test_restore_backup_with_wallet_plus_history_fails_closed_without_forensic_archive`
- `test_tx_list_paginates_cursor`
- `test_tx_export_portable_json`
- `test_tx_import_reconcile_portable`
- `test_tx_reconcile_requires_confirmation_evidence`
- `test_tx_reconcile_rejects_mismatched_evidence`
- `test_tx_list_reflects_cancel`
- `test_rpc_uses_injected_tx`
- `jsonl_import_is_explicit`
- `jsonl_replay_preserves_record`
- `jsonl_replay_preserves_full_tx_package_bytes`
- `jsonl_replay_rejects_tamper`
- `artifact_paths_stay_distinct`
- `tx_history_appends_admission_sequence`
- `test_tx_get_paginates_cursor`
- `test_tx_get_sorts_timestamp`
- `test_tx_history_includes_receipt`
- `test_tx_get_includes_receipt`
- `test_backup_seed_encrypted`
- `test_forensic_export_requires_jsonl`
- `test_forensic_export_rejects_mismatched_live_jsonl_bytes`
- `test_backup_create_list_restore`
- `test_backup_restore_wrong_password`
- `test_secrets_redacted`
- `test_tx_data_redacted`
- `test_rotate_key_redaction`
- `test_rotate_key_top_level`
- `test_rotate_key_confirmation`

These anchors are baseline evidence only. Future implementation should extend
them where they already own the seam instead of creating duplicate coverage
files for the same contract.

## 🧪 Planned Scenario Groups

### Group 1: Stage 13 Contract And Dispatch

What it proves:

- Scenario 1 now owns a first-class Stage 13 wallet-addons execution slice
- exact `S13-1` through `S13-15` contract ids stay wired into config,
  dispatcher, and verifier surfaces
- the release-style Scenario 1 path proves Stage 13 as part of the canonical
  simulator lane rather than a simulator-only shortcut

Primary home:

- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs` (planned)

### Group 2: Canonical Wallet.tx Lifecycle And Root Binding

What it proves:

- pending build reserves claimed assets
- cancel releases the reservation for a later build
- import, export, receipt, reconcile, and details all stay on one canonical
  wallet.tx lifecycle lane
- `prev_root`, `state_root`, and `flat_root` remain semantically distinct

Primary homes:

- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs` (planned)
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tamper.rs` (planned)

### Group 3: Backup, Restore, JSONL Replay, And Scan Resume

What it proves:

- `.wlt` and canonical wallet-prefixed JSONL history remain separate
  persistence planes
- `WalletPlusHistory` restore is all-or-nothing
- wrong password, tamper, missing archive, decode drift, and replay failure
  reject before wallet mutation
- wallet-side `recv_range(...)` restart resumes from persisted scan state

Primary homes:

- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
- `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`
- `crates/z00z_wallets/src/services/wallet_service_tests.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/tests.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs` (planned)

### Group 4: Payment Requests, TOFU, Session Hardening, And Rotation

What it proves:

- payment request signatures, expiry, chain binding, and TOFU gates are
  enforced before tx construction
- stale, locked, or rate-limited sessions cannot reach sensitive wallet paths
- `rotate_master_key` remains an audited in-memory rederive flow
- logs stay redacted for secrets, tx data, and rotate confirmations

Primary homes:

- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_tests.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`
- `crates/z00z_wallets/src/services/wallet_service_tests.rs`
- `crates/z00z_wallets/src/adapters/rpc/logging/summarize/tests.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs` (planned)

### Group 5: Wording And Boundary Hygiene

What it proves:

- the Plan 05 wording cleanup does not reintroduce `stub`, `placeholder`,
  `Phase 1`, `residue`, `JWT`, false ledger-authority claims, or durable
  seed-rotation claims
- wording-only changes stay wording-only and do not grow a synthetic runtime
  surface

Primary validation mode:

- focused residue scans over the exact Plan 05 touched files

## 🔐 Critical Invariants To Preserve

- one canonical `wallet.tx.*` lifecycle lane
- one canonical `.wlt` claimed-asset persistence plane
- one canonical wallet-prefixed JSONL tx-history artifact
- one wallet-side receive and scan-resume authority
- one in-memory `rotate_master_key` boundary with no durable-seed rewrite claim
- fail-closed tamper rejection before post-state mutation
- distinct pending, cancelled, confirmed, imported, and exported evidence
- distinct `prev_root`, `state_root`, and `flat_root` semantics

## 🛠️ Commands For Future Implementation Phase

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
cargo test -p z00z_wallets --release --features test-fast -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump stage13
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump
cargo fmt
cargo clippy --all-targets --all-features
cargo test --all
```

Focused file-specific commands remain governed by `046-01-PLAN.md` through
`046-06-PLAN.md` and summarized in `046-TESTS-TASKS.md`.

## 🚫 Current Stop Gate

This compatibility handoff remains planning-first only.

The next allowed move is implementation of the named regressions and scenario
groups into the existing wallet homes plus the planned Stage 13 simulator
homes. Until those files land and their focused or release commands execute
green, Phase 046 must not be described as RED-GREEN verified.

The current phase-local deliverables are:

- `046-TEST-SPEC.md`
- `046-TESTS-TASKS.md`
- `046-TEST-PLAN.md`

## 🔗 Canonical References

- `046-TEST-SPEC.md`
- `046-TESTS-TASKS.md`
- `046-CONTEXT.md`
- `046-wallet-addon-spec.md`
- `046-01-PLAN.md`
- `046-02-PLAN.md`
- `046-03-PLAN.md`
- `046-04-PLAN.md`
- `046-05-PLAN.md`
- `046-06-PLAN.md`
