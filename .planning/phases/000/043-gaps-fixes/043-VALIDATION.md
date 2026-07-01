---
phase: 043
slug: gaps-fixes
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-07
updated: 2026-05-08
last_nyquist_audit: 2026-05-08
gap_count: 0
---

<!-- markdownlint-disable MD013 MD033 MD041 MD055 MD056 -->

# Phase 043 - Validation Strategy

> Reconstructed Nyquist validation contract for Phase 043 from the completed planning, coverage, summary, and verification artifacts in `.planning/phases/043-gaps-fixes/`. The numbered plan chain is summary-backed complete through `043-10-PLAN.md`, and the 2026-05-07 repair pass closed the previously reported archive/output gaps with targeted reruns on those slices. The repair pass did not repeat the full workspace-wide sweep.
>
> Additive note: `043-fixes-spec-2.md` and `043-TODO-2.md` reopened Phase 043 for plans `043-11` through `043-18`. Plans `043-17` and `043-18` now land the dedicated spec-2 E2E test implementation and evidence-sync slice, with their commands and closeout evidence recorded in `043-coverage.md` and `043-SUMMARY.md`.

## 2026-05-07 Repair Pass

- `cargo test -p z00z_wallets --lib wallet_plus_history -- --nocapture`
- `cargo test -p z00z_wallets --lib tx_history_only -- --nocapture`
- `cargo test -p z00z_simulator --test test_claim_acceptance test_claim_service_single_entrypoint -- --nocapture`
- `cargo test -p z00z_wallets --lib validated_serial_build_ok -- --nocapture`

## Test Infrastructure

| Property | Value |
| --- | --- |
| Framework | Rust `cargo test` across the workspace |
| Config file | `Cargo.toml` workspace root |
| Quick run command | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| Full suite command | `cargo test --release --features test-fast --features wallet_debug_dump` |
| Estimated runtime | workspace-dependent; release-style sweep |

## Per-Task Verification Map

- **043-01 | Plan 01 | Wave T1 | Coverage ledger freeze and failing-test lock-in**
  - **Requirement:** `T-043-01-01 / 02 / 03`
  - **Threat Ref:** `T-043-01-01 / 02 / 03`
  - **Secure Behavior:** The execution ledger maps every EV, PH43, D-043, and AC-043 row and keeps out-of-scope cleanup excluded.
  - **Test Type:** diagnostics + unit
  - **Automated Command:** `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && rg -n "EV-|PH43-|D-043-|AC-043-" .planning/phases/043-gaps-fixes/043-coverage.md && rg -n "not implemented in Phase 1|# TODO|placeholder|best-effort only|does not assert that a downstream proof verifier ran here" crates/z00z_wallets/src crates/z00z_storage/src`
  - **File Exists:** `043-coverage.md`
  - **Status:** green

- **043-02 | Plan 02 | Wave T2 | Transaction assembler closure**
  - **Requirement:** `PH43-TXASM`
  - **Threat Ref:** `T-043-02-01 / 02 / 03`
  - **Secure Behavior:** Assembler paths are fail-closed, resolved-input evidence is required, and public package checks stay honest.
  - **Test Type:** unit
  - **Automated Command:** `cargo test -p z00z_wallets --test test_tx_balance -- --nocapture && cargo test -p z00z_wallets --test test_tx_tamper -- --nocapture && cargo test -p z00z_wallets --test test_tx_fee -- --nocapture && cargo test -p z00z_wallets tx_verifier --lib -- --nocapture`
  - **File Exists:** tx suite files
  - **Status:** green

- **043-03 | Plan 03 | Wave T3 | Storage membership and conservation separation**
  - **Requirement:** `PH43-CONSERVE / PH43-ASSETAUDIT`
  - **Threat Ref:** `T-043-03-01 / 02 / 03`
  - **Secure Behavior:** Storage membership, conservation, and operator audit remain separate layers.
  - **Test Type:** unit
  - **Automated Command:** `cargo test -p z00z_storage --test test_assets_suite -- --nocapture && cargo test -p z00z_storage --test test_claim_source_proof -- --nocapture && cargo test -p z00z_wallets --test test_tx_pedersen -- --nocapture && cargo test -p z00z_wallets --test test_spend_proof_backend -- --nocapture && cargo test -p z00z_wallets --test test_tx_wrong_root -- --nocapture`
  - **File Exists:** storage + wallet proof suite files
  - **Status:** green

- **043-04 | Plan 04 | Wave T4 | Optional forensic archive envelope**
  - **Requirement:** `PH43-ARCHIVE`
  - **Threat Ref:** `T-043-04-01 / 02 / 03`
  - **Secure Behavior:** Canonical `.wlt` stays wallet-state-only, and the forensic archive remains optional and non-mutating on failure.
  - **Test Type:** e2e / scenario
  - **Automated Command:** `cargo test -p z00z_wallets --test test_wallet_export_pack_boundary -- --nocapture && cargo test -p z00z_wallets --test test_tx_store_integration -- --nocapture && cargo test -p z00z_wallets --test test_redb_wlt_open -- --nocapture && cargo test -p z00z_wallets backup_exporter --lib -- --nocapture && cargo test -p z00z_wallets backup_importer_impl --lib -- --nocapture && cargo test -p z00z_wallets wallet_backup --lib -- --nocapture && cargo test -p z00z_wallets --lib wallet_plus_history -- --nocapture && cargo test -p z00z_wallets --lib tx_history_only -- --nocapture`
  - **File Exists:** archive suite files
  - **Status:** green

- **043-05 | Plan 05 | Wave T5 | Receive DTO and status honesty**
  - **Requirement:** `PH43-RECEIVE`
  - **Threat Ref:** `T-043-05-01 / 02 / 03`
  - **Secure Behavior:** Receive DTO/status honesty and outward-only compatibility stay stable.
  - **Test Type:** unit + e2e
  - **Automated Command:** `cargo test -p z00z_wallets --test test_stealth_scanner_flow -- --nocapture && cargo test -p z00z_wallets --test test_import_error_taxonomy -- --nocapture && cargo test -p z00z_wallets --test test_runtime_validation_result -- --nocapture && cargo test -p z00z_wallets stealth_scanner --lib -- --nocapture && cargo test -p z00z_wallets stealth_scan_support --lib -- --nocapture`
  - **File Exists:** receive suite files
  - **Status:** green

- **043-06 | Plan 06 | Wave T6 | Tag16 completeness gate**
  - **Requirement:** `PH43-TAG`
  - **Threat Ref:** `T-043-06-01 / 02 / 03`
  - **Secure Behavior:** TagFilterOnly requires completeness; fallback remains available when completeness is absent.
  - **Test Type:** unit
  - **Automated Command:** `cargo test -p z00z_wallets --test test_stealth_scanner_cache -- --nocapture && cargo test -p z00z_wallets --test test_stealth_scanner_prefilter -- --nocapture && cargo test -p z00z_wallets stealth_scanner --lib -- --nocapture && cargo test -p z00z_wallets stealth_scan_support --lib -- --nocapture`
  - **File Exists:** tag suite files
  - **Status:** green

- **043-07 | Plan 07 | Wave T7 | Stealth output builder contract hardening**
  - **Requirement:** `PH43-OUTPUT`
  - **Threat Ref:** `T-043-07-01 / 02 / 03`
  - **Secure Behavior:** Approved sender flows use validated builders; raw builders stay explicit seams.
  - **Test Type:** unit + e2e
  - **Automated Command:** `cargo test -p z00z_wallets --test test_stealth_output -- --nocapture && cargo test -p z00z_wallets --test test_live_path_enforcement --features test-fast -- --nocapture && cargo test -p z00z_wallets --test test_e2e_send_scan -- --nocapture && cargo test -p z00z_simulator --test test_claim_acceptance test_claim_service_single_entrypoint -- --nocapture && cargo test -p z00z_wallets --lib validated_serial_build_ok -- --nocapture`
  - **File Exists:** output suite files
  - **Status:** green

- **043-08 | Plan 08 | Wave T8 | Tx and conservation regression wave**
  - **Requirement:** `PH43-TXASM / PH43-CONSERVE / PH43-ASSETAUDIT`
  - **Threat Ref:** `T-043-08-01 / 02 / 03`
  - **Secure Behavior:** Tx/conservation rows are backed by named narrow regressions and typed failure classes remain visible.
  - **Test Type:** unit + e2e
  - **Automated Command:** `cargo test -p z00z_wallets --test test_tx_balance -- --nocapture && cargo test -p z00z_wallets --test test_tx_tamper -- --nocapture && cargo test -p z00z_wallets --test test_tx_fee -- --nocapture && cargo test -p z00z_wallets --test test_tx_pedersen -- --nocapture && cargo test -p z00z_wallets --test test_spend_proof_backend -- --nocapture && cargo test -p z00z_wallets --test test_tx_wrong_root -- --nocapture && cargo test -p z00z_storage --test test_assets_suite -- --nocapture && cargo test -p z00z_storage --test test_claim_source_proof -- --nocapture`
  - **File Exists:** coverage ledger + proof suite files
  - **Status:** green

- **043-09 | Plan 09 | Wave T9 | Receive, tag, and output regression wave**
  - **Requirement:** `PH43-RECEIVE / PH43-TAG / PH43-OUTPUT`
  - **Threat Ref:** `T-043-09-01 / 02 / 03`
  - **Secure Behavior:** Receive/tag/output rows stay bound to decisive regression homes and raw-builder regressions stay blocked.
  - **Test Type:** unit + e2e
  - **Automated Command:** `cargo test -p z00z_wallets --test test_stealth_scanner_flow -- --nocapture && cargo test -p z00z_wallets --test test_import_error_taxonomy -- --nocapture && cargo test -p z00z_wallets --test test_runtime_validation_result -- --nocapture && cargo test -p z00z_wallets --test test_stealth_scanner_cache -- --nocapture && cargo test -p z00z_wallets --test test_stealth_scanner_prefilter -- --nocapture && cargo test -p z00z_wallets --test test_stealth_output -- --nocapture && cargo test -p z00z_wallets --test test_live_path_enforcement --features test-fast -- --nocapture && cargo test -p z00z_wallets --test test_e2e_send_scan -- --nocapture`
  - **File Exists:** receive/tag/output suite files
  - **Status:** green

- **043-10 | Plan 10 | Wave T10 | Archive closure and phase closeout**
  - **Requirement:** `PH43-TXASM / PH43-CONSERVE / PH43-ASSETAUDIT / PH43-ARCHIVE / PH43-RECEIVE / PH43-TAG / PH43-OUTPUT`
  - **Threat Ref:** `T-043-10-01 / 02 / 03`
  - **Secure Behavior:** Closeout evidence, summary, and simulator gates agree, and archive evidence stays redacted or hash-bound.
  - **Test Type:** e2e / scenario + diagnostics
  - **Automated Command:** `cargo test -p z00z_wallets --test test_wallet_export_pack_boundary -- --nocapture && cargo test -p z00z_wallets --test test_tx_store_integration -- --nocapture && cargo test -p z00z_wallets --test test_redb_wlt_open -- --nocapture && cargo test -p z00z_wallets backup_exporter --lib -- --nocapture && cargo test -p z00z_wallets backup_importer_impl --lib -- --nocapture && cargo test -p z00z_wallets wallet_backup --lib -- --nocapture && cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump && rg -n "seed_phrase|wallet_identity|tx_bytes|enc_pack|asset_secret|blinding" .planning/phases/043-gaps-fixes/043-SUMMARY.md .planning/phases/043-gaps-fixes/043-coverage.md`
  - **File Exists:** summary + archive suite files
  - **Status:** green

## Wave 0 Requirements

Existing infrastructure covers the current Phase 043 validation surface.

No new framework installation was needed beyond the repository bootstrap gate, the existing Rust test suites, and the phase-local regression and verification artifacts.

## Spec-2 E2E Final Evidence

| Plan | Required validation focus | Status |
| --- | --- | --- |
| `043-17` | Implement `SPEC2-E2E-01` through `SPEC2-E2E-09` from `043-TEST-SPEC.md` in existing Rust test homes. | landed |
| `043-18` | Re-run the spec-2 E2E command wave, update `043-coverage.md` and `043-SUMMARY.md`, run redaction and no-parallel-artifact gates. | landed |

Required command anchors for the additive spec-2 E2E slice:

- `cargo test -p z00z_wallets --test test_wallet_export_pack_boundary -- --nocapture`
- `cargo test -p z00z_wallets --lib backup_importer_impl -- --nocapture`
- `cargo test -p z00z_wallets --test test_tx_store_integration -- --nocapture`
- `cargo test -p z00z_wallets --lib wallet_service_suite -- --nocapture`
- `cargo test -p z00z_wallets --test test_redb_wlt_open -- --nocapture`
- `cargo test -p z00z_wallets --test test_rpc_types_serialization -- --nocapture`
- `cargo test -p z00z_wallets --test test_tx_pedersen -- --nocapture`
- `cargo test -p z00z_wallets --test test_spend_proof_backend -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- `rg -n '"seed_phrase"|"wallet_identity"|"tx_bytes"|"enc_pack"|"asset_secret"|"blinding"' .planning/phases/043-gaps-fixes/043-SUMMARY.md .planning/phases/043-gaps-fixes/043-coverage.md`
- `ls .planning/phases/043-gaps-fixes | rg '043-(SUMMARY|coverage|TEST-SPEC|TESTS-TASKS)-2|SUMMARY-2|coverage-2'`

## Manual-Only Verifications

All phase behaviors have automated verification.

## Validation Sign-Off

- [x] Input state detected as reconstructed Phase 043 validation audit
- [x] Test infrastructure detected
- [x] State B reconstruction completed against the existing planning, summary, coverage, and verification artifacts
- [x] Requirement-to-test map covers `043-01` through `043-10`
- [x] No generated test files were required for the completed `043-01` through `043-10` scope
- [x] Additive spec-2 E2E test implementation and evidence sync landed in `043-17` and `043-18`
- [x] Sampling continuity is preserved through the phase 043 closeout gates
- [x] `nyquist_compliant: true` set in frontmatter

Approval: passed 2026-05-07

## Nyquist Audit Trail

### 2026-05-07 State B Reconstruction

- Input state: State B (`043-VALIDATION.md` absent before this reconstruction, `043-SUMMARY.md` present).
- Gap classification: no MISSING or PARTIAL rows in the reconstructed requirement map.
- The closeout verification report `043-10-VERIFICATION.md` agrees with `043-SUMMARY.md` and `043-coverage.md`.
- No new test files were generated because the existing anchors already cover the current phase 043 scope.

### Reconstruction Sources

- `.planning/phases/043-gaps-fixes/043-fixes-spec.md`
- `.planning/phases/043-gaps-fixes/043-TODO.md`
- `.planning/phases/043-gaps-fixes/043-coverage.md`
- `.planning/phases/043-gaps-fixes/043-SUMMARY.md`
- `.planning/phases/043-gaps-fixes/043-TEST-SPEC.md`
- `.planning/phases/043-gaps-fixes/043-TESTS-TASKS.md`
- `.planning/phases/043-gaps-fixes/043-10-VERIFICATION.md`
- `.planning/phases/043-gaps-fixes/043-CONTEXT.md`
