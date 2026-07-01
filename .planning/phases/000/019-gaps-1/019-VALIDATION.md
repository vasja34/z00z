---
phase: 019
slug: gaps-1
status: blocked
nyquist_compliant: false
wave_0_complete: true
created: 2026-03-24
updated: 2026-03-25
---

# Phase 019 - Validation Strategy

📌 Per-phase validation contract for wallet replay protection, receive
taxonomy hardening, and backup convergence.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test -p z00z_simulator --test test_stage3_nullifier_store -- --nocapture && cargo test -p z00z_wallets --test test_e2e_runtime_parity -- --nocapture && cargo test -p z00z_wallets --test test_stealth_scanner_prefilter -- --nocapture && cargo test -p z00z_wallets backup_impl::tests::test_backup_create_list_restore -- --nocapture` |
| **Release quick run command** | `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_stage3_nullifier_store -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets --test test_e2e_runtime_parity -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets --test test_stealth_scanner_prefilter -- --nocapture && cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets backup_impl::tests::test_backup_create_list_restore -- --nocapture` |
| **Full suite command** | `cargo test -p z00z_wallets && cargo test -p z00z_simulator` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task-local `<automated>` command from
  the owning PLAN file.
- **After every plan wave:** Run the relevant wallet or simulator suite for
  the active contract, plus the shared quick-run gate and the release quick-run
  gate before handoff when the touched targets support `test-fast` and
  `wallet_debug_dump`.
- **Before `/gsd-verify-work`:** `cargo test -p z00z_wallets && cargo test -p
  z00z_simulator` must be green.
- **Max feedback latency:** ~120 seconds.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | --------- | ----------------- | ----------- | ------ |
| 019-01-01 | 01 | 1 | PH19-NULL | integration | `cargo test -p z00z_simulator --test test_stage3_nullifier_store -- --nocapture` | ✅ | ✅ green |
| 019-01-02 | 01 | 1 | PH19-NULL | integration | `cargo test -p z00z_simulator -- --nocapture` | ✅ | ✅ green |
| 019-02-01 | 02 | 2 | PH19-SCAN | integration | `cargo test -p z00z_wallets --test test_e2e_runtime_parity -- --nocapture` + release quick run counterpart | ✅ | ✅ green |
| 019-02-02 | 02 | 2 | PH19-SCAN | integration | `cargo test -p z00z_wallets --test test_stealth_scanner_prefilter -- --nocapture` + release quick run counterpart | ✅ | ✅ green |
| 019-03-01 | 03 | 3 | PH19-BACKUP | unit | `cargo test -p z00z_wallets test_export_import_wallet_payload -- --nocapture` + release quick run counterpart | ✅ | ✅ green |
| 019-03-02 | 03 | 3 | PH19-BACKUP | integration | `cargo test -p z00z_wallets backup_impl::tests::test_backup_create_list_restore -- --nocapture` + `cargo test -p z00z_wallets legacy_v1_restore_fails -- --nocapture` + release quick run counterparts | ✅ | ✅ green |
| 019-03-03 | 03 | 3 | PH19-BACKUP | phase gate | `cargo test -p z00z_wallets && cargo test -p z00z_simulator` | ✅ | ⚠️ blocked outside phase scope |

📌 Status legend: `⬜ pending` · `✅ green` · `❌ red` · `⚠️ flaky`

---

## Wave 0 Requirements

✅ Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| -------- | ----------- | ---------- | ----------------- |
| Confirm that public backup messaging and operator-facing semantics distinguish V1 legacy readability from V2 full restore behavior during rollout. | PH19-BACKUP | The tests can prove behavior, but final operator-facing wording may still live in RPC messages or docs reviewed during execution. | Inspect the public backup and restore surfaces after Plan 03 and confirm they do not promise full restore on V1-only payloads. |

## Validation Audit 2026-03-25

| Metric | Count |
| ------ | ----- |
| Gaps found | 2 |
| Resolved | 1 |
| Escalated | 1 |

Audit result:

- `PH19-NULL` is covered by explicit simulator regression tests in debug and release profiles.
- `PH19-SCAN` is covered by parity and prefilter suites in debug and release profiles.
- `PH19-BACKUP` has targeted green coverage for payload roundtrip, backup create or restore, runtime restore response, dispatcher roundtrip, legacy V1 explicit rejection, and the previously stale `test_verify_backup_wrong_wallet` backup exporter expectation.
- The earlier backup-specific full-suite blocker is resolved: `cargo test -p z00z_wallets core::backup::backup_exporter_impl::tests::test_verify_backup_wrong_wallet -- --nocapture` is green on the current branch state.
- The earlier unrelated wallet compile blocker is resolved on the current branch state: `cargo test -p z00z_wallets --no-run` is green after removing the integration-test dependency on the feature-gated `sign_claim_auth` re-export.
- No missing automated tests were found for phase 019 itself in this validation pass.

---

## Validation Sign-Off

- [x] All planned tasks have `<automated>` verify or existing test
  infrastructure.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all MISSING references.
- [x] No watch-mode flags.
- [x] Feedback latency remains phase-local and bounded.
- [ ] All phase-level automated gates are green.
- [ ] `nyquist_compliant: true` can be retained in frontmatter.

**Approval:** blocked 2026-03-25
