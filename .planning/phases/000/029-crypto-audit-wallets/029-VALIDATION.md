---
phase: 029
slug: crypto-audit-wallets
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-30
---

# Phase 029 — Validation Strategy

> 📌 Reconstructed from executed Phase 029 plans, summaries, test contract, verification evidence, and the wallet-owned Rust test seams because no prior validation file existed in this phase directory.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust `cargo test` plus release-style wallet crate gates and structural `rg` scans for doc-owned reconciliation invariants |
| **Config file** | `Cargo.toml` workspace manifest plus crate manifests under `crates/*/Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | release-mode multi-command wallet suite |

## Sampling Rate

- 📌 After every task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` plus the task-specific targeted wallet test or structural scan command.
- 📌 After every plan wave: run `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump`.
- 📌 Before `/gsd-verify-work`: keep the authoritative wallet crate release gate green and preserve the phase-local verification bundle recorded in `029-VERIFICATION.md`.
- 📌 Max feedback latency: one bootstrap cycle plus one targeted release test or one structural scan for the reconciliation wave.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | --------- | ----------------- | ----------- | ------ |
| 029-01-01 | 01 | 1 | PH29-RECON | doc-structure | `V01` | ✅ | ✅ green |
| 029-01-02 | 01 | 1 | PH29-RECON | doc-structure | `V02` | ✅ | ✅ green |
| 029-02-01 | 02 | 2 | PH29-VIEWKEY | integration | `V03` | ✅ | ✅ green |
| 029-02-02 | 02 | 2 | PH29-VIEWKEY | integration | `V04` | ✅ | ✅ green |
| 029-03-01 | 03 | 3 | PH29-KDF, PH29-BACKUP | integration | `V05` | ✅ | ✅ green |
| 029-03-02 | 03 | 3 | PH29-KDF, PH29-BACKUP | migration | `V06` | ✅ | ✅ green |
| 029-04-01 | 04 | 4 | PH29-PANIC | integration | `V07` | ✅ | ✅ green |
| 029-04-02 | 04 | 4 | PH29-SEEDSALT | integration | `V08` | ✅ | ✅ green |
| 029-05-01 | 05 | 4 | PH29-KEYMGR | integration | `V09` | ✅ | ✅ green |
| 029-05-02 | 05 | 4 | PH29-SECRET | integration | `V10` | ✅ | ✅ green |
| 029-06-01 | 06 | 5 | PH29-DIGEST | integration | `V11` | ✅ | ✅ green |
| 029-06-02 | 06 | 5 | PH29-VALIDATION, PH29-BACKUP, PH29-SECRET | integration | `V12` | ✅ | ✅ green |

📌 Status legend: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky.

## Command Catalog

- `V01`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test --release --features test-fast --features wallet_debug_dump && rg -n 'expect\(|unwrap\(|derive_view_secret_key|derive_view_key_versioned|DERIVED_KEY_TTL_SECONDS|saturating_sub\(|compute_seed_salt|WalletBackupCrypto|build_tx_package_digest|WalletExportPack|seed_phrase|RuntimeValidationResult|generate_identity_keypair|derive_s_out|SafePassword|feature' crates/z00z_wallets/src`
- `V02`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test --release --features test-fast --features wallet_debug_dump && rg -n 'wallet_service|chain_service|stealth_keys|redb_wallet_store|wallet_backup|tx_verifier|file_key_store|seed_phrase|snapshot|RuntimeValidationResult|generate_identity_keypair|derive_s_out|open decision|source_ambiguity|proposed new file' .planning/phases/029-crypto-audit-wallets/029-RECONCILIATION.md`
- `V03`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_view_key_contract -- --nocapture && ! rg -n 'derive_rotated_view_secret_key' crates/z00z_wallets/src/core/stealth/output.rs crates/z00z_wallets/src/core/tx/spending.rs && cargo test --release --features test-fast --features wallet_debug_dump`
- `V04`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_view_key_contract -- --nocapture && cargo test -p z00z_wallets --release --test test_e2e_send_scan -- --nocapture && cargo test -p z00z_wallets --release --test test_rpc_key_derive_e2e -- --nocapture --test-threads=1 && cargo test --release --features test-fast --features wallet_debug_dump`
- `V05`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_backup_kdf_contract -- --nocapture && cargo test -p z00z_wallets --release --test test_backup_restore_identity -- --nocapture && cargo test -p z00z_wallets --release --test test_wallet_persistence_backup_service -- --nocapture && cargo test -p z00z_wallets --release --test test_wlt_validator -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump`
- `V06`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_wallet_kdf_migration -- --nocapture && cargo test -p z00z_wallets --release --test test_wallet_persistence_backup_service -- --nocapture && cargo test -p z00z_wallets --release --test test_redb_wlt_open -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump`
- `V07`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_wallet_service_errors -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump`
- `V08`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_seed_salt_policy -- --nocapture && cargo test -p z00z_wallets --release --test test_show_seed_phrase_plaintext -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump`
- `V09`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_key_manager -- --nocapture && cargo test -p z00z_wallets --release --test test_receiver_secret_validation -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump`
- `V10`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_file_key_store -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump`
- `V11`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_tx_digest_framing -- --nocapture && cargo test -p z00z_wallets --release --test test_tx_tamper -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump`
- `V12`: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_wallets --release --test test_runtime_validation_result -- --nocapture && cargo test -p z00z_wallets --release --test test_backup_metadata_policy -- --nocapture && cargo test -p z00z_wallets --release --test test_wallet_export_pack_boundary -- --nocapture && cargo test -p z00z_wallets --release --test test_wlt_validator -- --nocapture && cargo test -p z00z_wallets --release --test test_show_seed_phrase_plaintext -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump`

## Requirement Coverage Summary

| Requirement | Status | Evidence |
| ----------- | ------ | -------- |
| PH29-RECON | COVERED | 📌 `029-RECONCILIATION.md`, plan `029-01` structural `rg` scans, and the green plan summary prove the scope-freeze artifact exists, is file-anchored, and carries the authoritative execution map. |
| PH29-VIEWKEY | COVERED | 📌 `test_view_key_contract.rs`, `test_e2e_send_scan.rs`, and `test_rpc_key_derive_e2e.rs` prove live-path derivation stays singular while historical recovery remains explicit. |
| PH29-KDF | COVERED | 📌 `test_backup_kdf_contract.rs`, `test_wallet_kdf_migration.rs`, `test_redb_wlt_open.rs`, and `test_wallet_persistence_backup_service.rs` prove explicit KDF metadata, bounded compatibility, persisted rewrite, and reopen under canonical V2 semantics. |
| PH29-BACKUP | COVERED | 📌 `test_backup_kdf_contract.rs`, `test_backup_metadata_policy.rs`, `test_wallet_export_pack_boundary.rs`, `test_backup_restore_identity.rs`, `test_wlt_validator.rs`, and the green wallet release gate prove explicit backup KDF, metadata policy, and encrypted restore-identity preservation. |
| PH29-PANIC | COVERED | 📌 `test_wallet_service_errors.rs` plus the release-style `z00z_wallets` gate prove operator-facing wallet failures are typed and fail closed instead of panicking. |
| PH29-SEEDSALT | COVERED | 📌 `test_seed_salt_policy.rs` and `test_show_seed_phrase_plaintext.rs` prove persisted wallet-owned salt reuse across new writes and reveal or export boundaries. |
| PH29-KEYMGR | COVERED | 📌 `test_key_manager.rs` and `test_receiver_secret_validation.rs` prove loud gap-limit invariant handling and early receiver-secret rejection. |
| PH29-SECRET | COVERED | 📌 `test_receiver_secret_validation.rs`, `test_file_key_store.rs`, and `test_wallet_export_pack_boundary.rs` prove zeroizing ownership boundaries for secret-bearing wallet paths. |
| PH29-DIGEST | COVERED | 📌 `test_tx_digest_framing.rs` and `test_tx_tamper.rs` prove framed digest semantics and canonical tamper rejection. |
| PH29-VALIDATION | COVERED | 📌 `test_runtime_validation_result.rs`, `test_wlt_validator.rs`, `test_backup_metadata_policy.rs`, and the green wallet release gate prove explicit warning-capable validation and aligned DTO policy. |

📌 Gap analysis result: all declared Phase 029 requirements are `COVERED`; no `PARTIAL` or `MISSING` automated requirement references were found.

## Wave 0 Requirements

📌 Existing infrastructure covers all phase requirements. No Wave 0 scaffolding is needed.

## Manual-Only Verifications

All phase behaviors have automated verification.

## Reconstruction Notes

- 📌 `029-01` through `029-06` each have summary-backed closure artifacts.
- 📌 `029-TEST-SPEC.md` is verification-backed and defines the canonical Phase 029 test seams and release-style commands.
- 📌 `029-VERIFICATION.md` records the executed green verification bundle for the whole phase.
- 📌 The strongest automated seams for this phase are:
  - `crates/z00z_wallets/tests/test_view_key_contract.rs`
  - `crates/z00z_wallets/tests/test_backup_kdf_contract.rs`
  - `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs`
  - `crates/z00z_wallets/tests/test_wallet_service_errors.rs`
  - `crates/z00z_wallets/tests/test_seed_salt_policy.rs`
  - `crates/z00z_wallets/tests/test_receiver_secret_validation.rs`
  - `crates/z00z_wallets/tests/test_file_key_store.rs`
  - `crates/z00z_wallets/tests/test_tx_digest_framing.rs`
  - `crates/z00z_wallets/tests/test_runtime_validation_result.rs`
  - `crates/z00z_wallets/tests/test_backup_metadata_policy.rs`
  - `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
  - `crates/z00z_wallets/tests/test_backup_restore_identity.rs`
  - `crates/z00z_wallets/tests/test_e2e_send_scan.rs`
  - `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`
  - `crates/z00z_wallets/tests/test_redb_wlt_open.rs`
  - `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs`
  - `crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs`
  - `crates/z00z_wallets/tests/test_tx_tamper.rs`
  - `crates/z00z_wallets/tests/test_wlt_validator.rs`

📌 The phase also depends on two automated structural guards that are not Rust test files but still close the requirement contract:

- the plan `029-01` reconciliation scans over wallet source and the final `029-RECONCILIATION.md` artifact;
- the plan `029-02` hot-path source guard that blocks `derive_rotated_view_secret_key(...)` from re-entering live scan and spend paths.

## Gap Audit

| Requirement | Coverage | Evidence |
| ----------- | -------- | -------- |
| PH29-RECON | COVERED | `029-01-SUMMARY.md` plus the reconciliation artifact and structural verification commands |
| PH29-VIEWKEY | COVERED | `029-VERIFICATION.md` plus `test_view_key_contract`, `test_e2e_send_scan`, and `test_rpc_key_derive_e2e` |
| PH29-KDF | COVERED | `029-VERIFICATION.md` plus `test_backup_kdf_contract`, `test_wallet_kdf_migration`, `test_redb_wlt_open`, and `test_wallet_persistence_backup_service` |
| PH29-BACKUP | COVERED | `029-VERIFICATION.md` plus `test_backup_kdf_contract`, `test_backup_metadata_policy`, `test_wallet_export_pack_boundary`, `test_backup_restore_identity`, `test_wlt_validator`, and the wallet release gate |
| PH29-PANIC | COVERED | `029-VERIFICATION.md` plus `test_wallet_service_errors` and the wallet release gate |
| PH29-SEEDSALT | COVERED | `029-VERIFICATION.md` plus `test_seed_salt_policy` and `test_show_seed_phrase_plaintext` |
| PH29-KEYMGR | COVERED | `029-VERIFICATION.md` plus `test_key_manager` and `test_receiver_secret_validation` |
| PH29-SECRET | COVERED | `029-VERIFICATION.md` plus `test_receiver_secret_validation`, `test_file_key_store`, and `test_wallet_export_pack_boundary` |
| PH29-DIGEST | COVERED | `029-VERIFICATION.md` plus `test_tx_digest_framing` and `test_tx_tamper` |
| PH29-VALIDATION | COVERED | `029-VERIFICATION.md` plus `test_runtime_validation_result`, `test_wlt_validator`, `test_backup_metadata_policy`, and the wallet release gate |

📌 No Nyquist gaps were found during reconstruction, so no additional test files were required.

## Validation Sign-Off

- [x] All tasks have `<automated>` verify coverage or existing infrastructure coverage.
- [x] Sampling continuity is preserved; no three consecutive tasks rely on missing automated verification.
- [x] Wave 0 is not required because existing infrastructure covers all phase requirements.
- [x] No watch-mode flags are part of the phase validation contract.
- [x] Feedback latency remains bounded to bootstrap plus targeted release validation or structural scans.
- [x] `nyquist_compliant: true` is set in frontmatter.

📌 Approval: approved 2026-03-30
