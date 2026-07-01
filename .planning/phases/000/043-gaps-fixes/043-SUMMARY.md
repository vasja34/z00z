---
phase: 043-gaps-fixes
plan: 18
subsystem: wallets
tags: [storage, backup, receive, tag, output, simulator, regression, e2e]
provides:
  - Phase 043 closeout evidence for storage, backup, receive, tag, output, and simulator gates
  - Landed coverage rows for every EV, PH43, D-043, and AC-043 requirement
  - Final additive spec-2 E2E closeout evidence through 043-18
affects: [Phase 044 and later planning]
tech-stack:
  added:
    - none
  patterns:
    - proof scan summaries
    - optional forensic archive envelopes
    - canonical JSONL tx-history replay
    - explicit live-store versus RPC export boundaries
    - validated send-path routing
key-files:
  created:
    - .planning/phases/043-gaps-fixes/043-SUMMARY.md
    - .planning/phases/043-gaps-fixes/043-TODO-2.md
  modified:
    - .planning/phases/043-gaps-fixes/043-coverage.md
    - crates/z00z_storage/src/assets/proof.rs
    - crates/z00z_storage/src/assets/store_internal/store_query.rs
    - crates/z00z_wallets/src/backup/crypto/backup_wire.rs
    - crates/z00z_wallets/src/backup/export/backup_exporter_impl.rs
    - crates/z00z_wallets/src/backup/export/backup_exporter_verify.rs
    - crates/z00z_wallets/src/backup/import/backup_importer.rs
    - crates/z00z_wallets/src/backup/import/backup_importer_impl.rs
    - crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs
    - crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs
    - crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs
    - crates/z00z_wallets/src/tx/tx_assembler.rs
    - crates/z00z_wallets/src/tx/verify/tx_verifier.rs
    - crates/z00z_wallets/src/tx/commit_audit.rs
    - crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs
    - crates/z00z_wallets/tests/test_spend_proof_backend.rs
    - crates/z00z_wallets/tests/test_redb_wlt_open.rs
    - crates/z00z_wallets/tests/test_tx_store_integration.rs
    - crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs
    - crates/z00z_wallets/tests/test_tx_poison.rs
    - crates/z00z_wallets/tests/test_live_path_enforcement.rs
    - crates/z00z_storage/tests/test_claim_source_proof.rs
    - crates/z00z_storage/tests/assets/test_store_api.rs
key-decisions:
  - "Keep storage membership and Pedersen conservation separate."
  - "Keep the optional forensic archive bound to the existing encrypted backup transport."
  - "Keep validated builders on the approved send path."
duration: phase-level
completed: 2026-05-06
---

# Phase 043: Gaps-Fixes Summary

The original closeout had no known implementation gaps after the 2026-05-07 repair pass closed the last live-audit issues in the archive import seam and the routed simulator output path. This summary keeps that original closeout record and now also records final additive spec-2 evidence through 043-18.

## Performance

- **Duration:** phase-level
- **Tasks:** 7 original closeout gates plus additive spec-2 slices through 043-18
- **Files modified:** 16+ key files, plus the coverage ledger

## Accomplishments

- Storage proof handling now preserves structured proof failures and exposes a sanitized scan summary instead of flattening JMT witness errors.
- Backup import/export now supports an optional forensic archive while keeping the canonical wallet snapshot flow separate and explicitly validated.
- Tx assembly, request-contract validation, conservation checks, wrong-root rejection, receive-path taxonomy, and live send-path routing all have matching regression coverage and release-style validation.
- The additive 043-11 pass keeps manual asset-class auditing on the existing tx seam while adding typed target, status, outcome, and mismatch diagnostics.
- The additive 043-12 pass keeps wallet snapshot, canonical JSONL history, and live tx-history directory naming on one shared wallet-stem helper without widening `.wlt` restore semantics.
- The additive 043-13 pass adds the canonical JSONL tx-history entry contract and explicit replay/import path while keeping `WalletForensicPack` inside the existing encrypted backup payload seam.
- The additive 043-14 pass locks live tx-history storage, canonical JSONL history, and `outputs/tx_exports` as distinct artifact paths without adding a new storage contract or plaintext replay carrier.
- The additive 043-15 pass maps the spec-2 regression matrix into `043-coverage.md`, reruns the audit/archive/JSONL/live-store regression wave, and keeps summary/coverage evidence redacted.
- The additive 043-16 pass finalizes the existing coverage ledger, summary, and TODO-2 closeout state with exact closeout validation, acceptance-group confirmation, no new dependency declaration, and no parallel closeout artifact.
- The additive 043-17 pass lands runtime spec-2 E2E assertions in the existing wallet/export/import/tx-store/service/RPC/audit test homes and records the scenario map in `043-coverage.md`.
- The additive 043-18 pass syncs final E2E evidence into the existing test spec, tests task list, coverage ledger, and summary without creating a parallel closeout surface.
- Final spec-2 closeout truth: `wallet_<wallet_stem_hex>.wlt`, `wallet_<wallet_stem_hex>_tx_history.jsonl`, the live tx-history directory, the explicit encrypted archive path, and `outputs/tx_exports` are distinct artifacts; the legacy `tx_history_<wallet_stem_hex>.jsonl` order is non-canonical only.
- Canonical JSONL replay/import remains fail-closed before live-store writes and preserves the full `TxRecord` view for inspection after replay; deeper encrypted payload interpretation stays role- and key-dependent.
- Any future strengthening of public conservation claims must go through a protocol-change spec before implementation or closeout wording changes.

## Validation

Targeted repair pass rerun on 2026-05-07:

- `cargo test -p z00z_wallets --lib wallet_plus_history -- --nocapture`
- `cargo test -p z00z_wallets --lib tx_history_only -- --nocapture`
- `cargo test -p z00z_wallets test_restore_backup_with_wallet_plus_history_rejects_tampered_forensic_archive_without_wallet_mutation --lib -- --nocapture`
- `cargo test -p z00z_simulator --test test_claim_acceptance test_claim_service_single_entrypoint -- --nocapture`
- `cargo test -p z00z_wallets --lib validated_serial_build_ok -- --nocapture`

Historical closeout commands retained from the original phase record:

- `cargo test -p z00z_wallets --test test_wallet_export_pack_boundary -- --nocapture`
- `cargo test -p z00z_wallets --test test_tx_store_integration -- --nocapture`
- `cargo test -p z00z_wallets --test test_redb_wlt_open -- --nocapture`
- `cargo test -p z00z_wallets --test test_stealth_request -- --nocapture`
- `cargo test -p z00z_wallets --test test_tx_wrong_root -- --nocapture`
- `cargo test -p z00z_wallets backup_exporter --lib -- --nocapture` returned 0 tests, which matches the current exporter helper shape.
- `cargo test -p z00z_wallets backup_importer_impl --lib -- --nocapture`
- `cargo test -p z00z_wallets wallet_backup --lib -- --nocapture`
- `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`

Exact additive 043-16 closeout commands run on 2026-05-08:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo fmt --all`
- `cargo test -p z00z_wallets --test test_wallet_export_pack_boundary -- --nocapture`
- `cargo test -p z00z_wallets --test test_tx_store_integration -- --nocapture`
- `cargo test -p z00z_wallets --test test_redb_wlt_open -- --nocapture`
- `cargo test -p z00z_wallets --lib backup_exporter -- --nocapture`
- `cargo test -p z00z_wallets --lib backup_importer_impl -- --nocapture`
- `cargo test -p z00z_wallets --lib wallet_backup -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- redaction `rg` exactly as listed in `043-TODO-2.md`, with the forbidden quoted-field pattern intentionally not duplicated into this scanned summary file
- `git diff -- Cargo.toml crates/z00z_wallets/Cargo.toml crates/z00z_storage/Cargo.toml crates/z00z_utils/Cargo.toml`
- `git diff --check`

Acceptance group closeout for 043-16:

- Public conservation witness boundary: closed as an honesty boundary only; future strengthening requires a protocol-change spec.
- Manual asset-class audit outcome behavior: closed on the typed target, status, outcome, and mismatch classes on the existing audit seam.
- Forensic archive/import behavior: closed on the encrypted backup payload seam, explicit import modes, required canonical JSONL history, and fail-closed replay validation.
- Naming and placement behavior: closed on stem-synced `.wlt`, canonical JSONL history, live tx-history directory, explicit archive path, and `outputs/tx_exports` as separate artifacts.

Exact outputs referenced by the closeout gate:

- 2026-05-08 additive 043-11 pass: `bootstrap_tests.sh` ended with `=== BOOTSTRAP COMPLETE ===`
- 2026-05-08 additive 043-11 pass: `test_tx_pedersen` returned `2 passed; 0 failed`
- 2026-05-08 additive 043-11 pass: `test_spend_proof_backend` returned `36 passed; 0 failed`
- 2026-05-08 additive 043-11 pass: `test_tx_wrong_root` returned `2 passed; 0 failed`
- 2026-05-08 additive 043-11 pass: `test_tx_balance` returned `1 passed; 0 failed`
- 2026-05-08 additive 043-11 pass: broad release gate `cargo test --release --features test-fast --features wallet_debug_dump` returned `exit code 0`
- 2026-05-08 additive 043-12 pass: `bootstrap_tests.sh` ended with `=== BOOTSTRAP COMPLETE ===`
- 2026-05-08 additive 043-12 pass: source-shape `rg` confirmed `wallet_stem`, `wallet_snapshot_name`, `wallet_history_jsonl_name`, `wallet_history_jsonl_path`, and `wallet_tx_history_dir` live on the existing wallet persistence seam and tests.
- 2026-05-08 additive 043-12 pass: `wallet_stem_names_sync` returned `1 passed; 0 failed`
- 2026-05-08 additive 043-12 pass: `wlt_open_without_jsonl` returned `1 passed; 0 failed`
- 2026-05-08 additive 043-12 pass: broad release gate `cargo test --release --features test-fast --features wallet_debug_dump` returned `exit code 0`
- 2026-05-08 additive 043-13 pass: `bootstrap_tests.sh` ended with `=== BOOTSTRAP COMPLETE ===`
- 2026-05-08 additive 043-13 pass: source-shape `rg` confirmed the canonical JSONL entry/helpers, explicit forensic constructor and import-mode seams, shared wallet JSONL naming, and no persisted forensic toggle.
- 2026-05-08 additive 043-13 pass: `test_forensic_export_requires_jsonl` returned `1 passed; 0 failed`
- 2026-05-08 additive 043-13 pass: `backup_importer_impl` returned `18 passed; 0 failed`
- 2026-05-08 additive 043-13 pass: `test_tx_store_integration` returned `2 passed; 0 failed`
- 2026-05-08 additive 043-13 pass: `jsonl_import_keeps_view` returned `1 passed; 0 failed`
- 2026-05-08 additive 043-13 pass: broad release gate `cargo test --release --features test-fast --features wallet_debug_dump` returned `exit code 0`
- 2026-05-08 additive 043-13 pass: `/GSD-Review-Tasks-Execution` was run in YOLO mode 3 times; the final 2 passes found no significant issues.
- 2026-05-08 additive 043-14 pass: `bootstrap_tests.sh` ended with `=== BOOTSTRAP COMPLETE ===`
- 2026-05-08 additive 043-14 pass: source-shape `rg` confirmed one-json-per-tx live storage wording, wallet-prefixed JSONL history paths, `tx_export_dir()`, and `outputs/tx_exports` references.
- 2026-05-08 additive 043-14 pass: `test_tx_store_integration` returned `3 passed; 0 failed`
- 2026-05-08 additive 043-14 pass: `wallet_stem_names_sync` returned `1 passed; 0 failed`
- 2026-05-08 additive 043-14 pass: `test_redb_wlt_open` returned `13 passed; 0 failed`
- 2026-05-08 additive 043-14 pass: broad release gate `cargo test --release --features test-fast --features wallet_debug_dump` returned `exit code 0`
- 2026-05-08 additive 043-14 pass: `/GSD-Review-Tasks-Execution` was run in YOLO mode 3 times; the final 2 passes found no significant issues.
- 2026-05-08 additive 043-15 pass: `bootstrap_tests.sh` ended with `=== BOOTSTRAP COMPLETE ===`
- 2026-05-08 additive 043-15 pass: source-shape `rg` confirmed wallet-prefixed JSONL naming, legacy non-canonical naming references, and typed audit outcome anchors.
- 2026-05-08 additive 043-15 pass: redaction `rg` over summary and coverage returned no matches for quoted secret-bearing evidence field names.
- 2026-05-08 additive 043-15 pass: matrix-anchor `rg` confirmed all six spec-2 regression matrix rows in `043-coverage.md`.
- 2026-05-08 additive 043-15 pass: `test_tx_pedersen` returned `2 passed; 0 failed`
- 2026-05-08 additive 043-15 pass: `test_spend_proof_backend` returned `36 passed; 0 failed`
- 2026-05-08 additive 043-15 pass: `test_wallet_export_pack_boundary` returned `2 passed; 0 failed`
- 2026-05-08 additive 043-15 pass: `backup_importer_impl` returned `18 passed; 0 failed`
- 2026-05-08 additive 043-15 pass: `test_tx_store_integration` returned `3 passed; 0 failed`
- 2026-05-08 additive 043-15 pass: `test_redb_wlt_open` returned `13 passed; 0 failed`
- 2026-05-08 additive 043-15 pass: broad release gate `cargo test --release --features test-fast --features wallet_debug_dump` returned `exit code 0`
- 2026-05-08 additive 043-15 pass: `/GSD-Review-Tasks-Execution` was run in YOLO mode 3 times; the final 2 passes found no significant issues.
- 2026-05-08 additive 043-16 pass: `bootstrap_tests.sh` ended with `=== BOOTSTRAP COMPLETE ===`
- 2026-05-08 additive 043-16 pass: `cargo fmt --all` returned `exit code 0`; rustfmt repeated pre-existing stable-toolchain warnings for unstable config keys.
- 2026-05-08 additive 043-16 pass: `test_wallet_export_pack_boundary` returned `2 passed; 0 failed`
- 2026-05-08 additive 043-16 pass: `test_tx_store_integration` returned `3 passed; 0 failed`
- 2026-05-08 additive 043-16 pass: `test_redb_wlt_open` returned `13 passed; 0 failed`
- 2026-05-08 additive 043-16 pass: `backup_exporter` module filter returned `0 tests`; exporter behavior remains covered by `test_wallet_export_pack_boundary`.
- 2026-05-08 additive 043-16 pass: `backup_importer_impl` returned `18 passed; 0 failed`
- 2026-05-08 additive 043-16 pass: `wallet_backup` returned `6 passed; 0 failed`
- 2026-05-08 additive 043-16 pass: broad release gate `cargo test --release --features test-fast --features wallet_debug_dump` returned `exit code 0`
- 2026-05-08 additive 043-16 pass: redaction `rg` over summary and coverage returned no matches for quoted secret-bearing evidence field names.
- 2026-05-08 additive 043-16 pass: Cargo manifest diff check over workspace, wallet, storage, and utils manifests returned no diff.
- 2026-05-08 additive 043-16 pass: `git diff --check` returned `exit code 0`
- 2026-05-08 additive 043-17 pass: `bootstrap_tests.sh` ended with `=== BOOTSTRAP COMPLETE ===`
- 2026-05-08 additive 043-17 pass: `test_wallet_export_pack_boundary` returned `4 passed; 0 failed`
- 2026-05-08 additive 043-17 pass: `backup_importer_impl` returned `18 passed; 0 failed`
- 2026-05-08 additive 043-17 pass: `test_tx_store_integration` returned `5 passed; 0 failed`
- 2026-05-08 additive 043-17 pass: `wallet_service_suite` module filter returned `0 tests`; direct service anchors `wallet_stem_names_sync`, `legacy_history_name_rejected`, and `jsonl_import_keeps_view` each returned `1 passed; 0 failed`
- 2026-05-08 additive 043-17 pass: `test_redb_wlt_open` returned `13 passed; 0 failed`
- 2026-05-08 additive 043-17 pass: `test_rpc_types_serialization` returned `7 passed; 0 failed`
- 2026-05-08 additive 043-17 pass: `test_tx_pedersen` returned `2 passed; 0 failed`
- 2026-05-08 additive 043-17 pass: `test_spend_proof_backend` returned `36 passed; 0 failed`
- 2026-05-08 additive 043-17 pass: broad release gate `cargo test --release --features test-fast --features wallet_debug_dump` returned `exit code 0`
- 2026-05-08 additive 043-18 pass: `bootstrap_tests.sh` ended with `=== BOOTSTRAP COMPLETE ===`
- 2026-05-08 additive 043-18 pass: source-shape `rg` confirmed `SPEC2-E2E-*`, wallet-prefixed JSONL naming, typed audit outcomes, canonical JSONL entry anchors, explicit forensic import modes, and `outputs/tx_exports`.
- 2026-05-08 additive 043-18 pass: `test_wallet_export_pack_boundary` returned `4 passed; 0 failed`
- 2026-05-08 additive 043-18 pass: `backup_importer_impl` returned `18 passed; 0 failed`
- 2026-05-08 additive 043-18 pass: `test_tx_store_integration` returned `5 passed; 0 failed`
- 2026-05-08 additive 043-18 pass: `wallet_service_suite` module filter returned `0 tests`; direct service anchors `wallet_stem_names_sync`, `legacy_history_name_rejected`, and `jsonl_import_keeps_view` remained the effective service checks.
- 2026-05-08 additive 043-18 pass: `test_redb_wlt_open` returned `13 passed; 0 failed`
- 2026-05-08 additive 043-18 pass: `test_rpc_types_serialization` returned `7 passed; 0 failed`
- 2026-05-08 additive 043-18 pass: `test_tx_pedersen` returned `2 passed; 0 failed`
- 2026-05-08 additive 043-18 pass: `test_spend_proof_backend` returned `36 passed; 0 failed`
- 2026-05-08 additive 043-18 pass: broad release gate `cargo test --release --features test-fast --features wallet_debug_dump` returned `exit code 0`
- 2026-05-08 additive 043-18 pass: redaction `rg` over summary and coverage returned no matches for the forbidden quoted secret-bearing evidence fields.
- 2026-05-08 additive 043-18 pass: no-parallel-artifact `rg` over the phase directory returned no matches for secondary closeout, coverage, summary, test-spec, or tests-tasks artifacts.
- 2026-05-08 additive 043-18 pass: `git diff --check` returned `exit code 0`
- 2026-05-08 additive 043-18 pass: `/GSD-Review-Tasks-Execution` was run in YOLO mode 3 times; the final 2 passes found no significant issues.

- 2026-05-07 repair pass: `3 passed; 0 failed` for `wallet_plus_history`
- 2026-05-07 repair pass: `3 passed; 0 failed` for `tx_history_only`
- 2026-05-07 repair pass: `1 passed; 0 failed` for `test_restore_backup_with_wallet_plus_history_rejects_tampered_forensic_archive_without_wallet_mutation`
- 2026-05-07 repair pass: `1 passed; 0 failed` for `test_claim_service_single_entrypoint`
- 2026-05-07 repair pass: `1 passed; 0 failed` for `validated_serial_build_ok`

- archive closeout boundary: `1 passed; 0 failed`
- tx store integration: `1 passed; 0 failed`
- wallet open / redb gates: `12 passed; 0 failed`
- broad release gate: `exit code 0`
- scenario 1 release run: `exit code 0`
- simulator release test-fast: `exit code 0`

## Task Commits

- No commit was created for this closeout; the work remains in the current working tree.

## Files Created/Modified

- `.planning/phases/043-gaps-fixes/043-SUMMARY.md` - phase closeout summary and validation record
- `.planning/phases/043-gaps-fixes/043-coverage.md` - landed coverage map for every phase row
- `.planning/phases/043-gaps-fixes/043-TEST-SPEC.md` - final spec-2 E2E scenario ownership status
- `.planning/phases/043-gaps-fixes/043-TESTS-TASKS.md` - final spec-2 E2E task status
- `.planning/phases/043-gaps-fixes/043-TODO-2.md` - additive spec-2 backlog closeout checklist
- `crates/z00z_storage/src/assets/proof.rs` - sanitized proof scan summary and root-bind checks
- `crates/z00z_wallets/src/backup/import/backup_importer_impl.rs` - mode-aware forensic archive import
- `crates/z00z_wallets/src/backup/crypto/backup_wire.rs` - canonical JSONL tx-history row plus encode/decode validation helpers
- `crates/z00z_wallets/src/backup/export/backup_exporter_impl.rs` - explicit forensic file export path requiring a canonical JSONL history artifact
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs` - canonical JSONL replay entrypoint that validates before live tx-store writes
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs` - service-level fail-closed archive restore proof
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs` - shared wallet stem, canonical snapshot name, and canonical JSONL history name helpers
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs` - live tx-history store boundary wording for one JSON file per transaction
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs` - summary-oriented RPC export tree boundary wording
- `crates/z00z_wallets/tests/test_tx_store_integration.rs` - live tx-history directory, canonical JSONL file, and RPC export tree path separation coverage
- `crates/z00z_wallets/tests/test_rpc_types_serialization.rs` - persisted backup settings serialization guard against forensic/history toggles
- `crates/z00z_wallets/src/tx/tx_assembler.rs` - canonical transaction assembly and verification
- `crates/z00z_wallets/src/receiver/scan/types_receive.rs` - honest receive status and taxonomy
- `crates/z00z_wallets/src/receiver/scan/types_tag_cache.rs` - advisory tag cache wording and strict prefilter boundary
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs` - validated live send routing

## Decisions & Deviations

- The 2026-05-07 repair pass reran only the decisive archive/output slices that were previously open; it did not repeat the entire workspace-wide phase sweep.
- The exporter library filter produced zero tests, so the actual exporter coverage came from the export-boundary and importer/library gates.
- The 043-12 naming pass intentionally kept `test_redb_wlt_open.rs` limited to snapshot restore/load truth; archive/export omission remains owned by the archive/export seam instead of being hidden in `.wlt` restore.
- The 043-13 forensic JSONL pass intentionally left `test_wallet_json_export.rs` unchanged because `test_wallet_export_pack_boundary.rs` and `backup_importer_impl` own the archive serialization and import contract more directly.
- The 043-14 boundary pass intentionally made the live-store/RPC-export distinction explicit with comments and path tests only; it did not expand canonical tx-history storage semantics.
- The 043-15 regression pass records `cargo test -p z00z_wallets --lib wallet_service_suite -- --nocapture` as a 0-test module-filter run; the effective service-level anchors are the direct `wallet_stem_names_sync` and `jsonl_import_keeps_view` filters plus the full release gate.
- The 043-16 closeout records `cargo test -p z00z_wallets --lib backup_exporter -- --nocapture` as a 0-test module-filter run; the effective exporter anchor remains the export-boundary integration test.
- The 043-18 evidence sync records `cargo test -p z00z_wallets --lib wallet_service_suite -- --nocapture` as a 0-test module-filter run; direct service anchors remain the effective service checks.
- No new external dependency declaration was introduced in the additive spec-2 slice; no new `z00z_utils` bypass was needed for the planning-only closeout update.

## Next Phase Readiness

The additive 043-11 through 043-18 slices have no known residual implementation gaps. Future phases should rely on the landed coverage ledger plus the targeted 2026-05-08 evidence for the manual asset-class audit, wallet-stem naming, canonical forensic JSONL, live-store/RPC-export boundary, regression matrix, closeout contracts, and runtime E2E scenario coverage.
