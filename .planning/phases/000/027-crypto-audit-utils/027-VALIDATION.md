---
phase: 027
slug: crypto-audit-utils
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-29
---

# Phase 027 — Validation Strategy

> Per-phase validation contract reconstructed from executed Phase 027 artifacts, existing automated test seams, and the green verification bundle.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust `cargo test` plus release-style workspace commands |
| **Config file** | none - existing Cargo workspace conventions and per-crate tests cover the phase |
| **Quick run command** | `cargo test -p z00z_utils --release` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | release-style multi-command Rust suite |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p z00z_utils --release`
- **After every plan wave:** Run `cargo test --release --features test-fast --features wallet_debug_dump`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** one task-level quick run or one wave-level full gate

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | --------- | ----------------- | ----------- | ------ |
| 027-01-01 | 01 | 1 | PH27-MEMLOCK | integration | `cargo test -p z00z_utils --release --test test_os_hardening_integration` | ✅ | ✅ green |
| 027-01-02 | 01 | 1 | PH27-MEMLOCK | miri | `cargo +nightly miri test -p z00z_utils --test test_os_hardening_integration` | ✅ | ✅ green |
| 027-02-01 | 02 | 2 | PH27-CONFIG | integration | `cargo test -p z00z_utils --release --test test_config_integration` | ✅ | ✅ green |
| 027-02-02 | 02 | 2 | PH27-CONFIG | example-check | `cargo check -p z00z_utils --release --example config_demo` | ✅ | ✅ green |
| 027-03-01 | 03 | 3 | PH27-TIME | policy-scan | `cargo test -p z00z_utils --release --test test_time_policy_micros` | ✅ | ✅ green |
| 027-03-03 | 03 | 3 | PH27-TIME | example-check | `cargo check -p z00z_utils --release --example time_provider_demo` | ✅ | ✅ green |
| 027-03-02 | 03 | 3 | PH27-TIME | downstream-release | `see canonical time-policy scan below` | ✅ | ✅ green |
| 027-04-01 | 04 | 4 | PH27-RNG | genesis-release | `cargo test -p z00z_core --test genesis_tests -- --nocapture` | ✅ | ✅ green |
| 027-04-02 | 04 | 4 | PH27-RNG | simulator-release | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 027-05-01 | 05 | 5 | PH27-LOGGER | integration | `cargo test -p z00z_utils --release --test test_logger_integration` | ✅ | ✅ green |
| 027-05-02 | 05 | 5 | PH27-IO | integration | `cargo test -p z00z_utils --release --test test_io_integration` | ✅ | ✅ green |
| 027-06-01 | 06 | 6 | PH27-JSON | integration | `cargo test -p z00z_utils --release --test test_codec_integration` | ✅ | ✅ green |
| 027-06-02 | 06 | 6 | PH27-JSON | workspace-release | `cargo test --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |

Status legend: ⬜ pending, ✅ green, ❌ red, ⚠️ flaky.

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Reconstruction Notes

- `027-01` through `027-06` each have summary-backed closure artifacts.
- `027-TEST-SPEC.md` is verification-backed and defines the canonical test seams and commands for the phase.
- `027-VERIFICATION.md` records the executed green verification bundle for the full phase.
- Existing test seams already cover every declared Phase 027 requirement:
  - `crates/z00z_utils/tests/test_os_hardening_integration.rs`
  - `crates/z00z_utils/examples/time_provider_demo.rs`
  - `crates/z00z_utils/tests/test_config_integration.rs`
  - `crates/z00z_utils/tests/test_time_policy_micros.rs`
  - `crates/z00z_utils/tests/test_logger_integration.rs`
  - `crates/z00z_utils/tests/test_io_integration.rs`
  - `crates/z00z_utils/tests/test_codec_integration.rs`
  - `crates/z00z_wallets/tests/test_addr_rate_limit_integration.rs`
  - `crates/z00z_wallets/tests/test_key_manager.rs`
  - `crates/z00z_wallets/tests/test_stealth_request.rs`
  - `crates/z00z_core/tests/genesis/test_reproducibility.rs`
  - `crates/z00z_core/tests/genesis/test_genesis.rs`
  - `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`

Canonical time-policy scan command used for `027-03-02`:

```bash
rg -n '\.unix_timestamp(_millis|_micros)?\(|duration_since\(SystemTime::UNIX_EPOCH\).*as_micros\(' crates --glob 'crates/**/src/**/*.rs' --glob 'crates/**/bin/**/*.rs' --glob '!crates/z00z_crypto/tari/**' --glob '!crates/z00z_utils/src/time/**' --glob '!**/tests/**' --glob '!**/examples/**' --glob '!**/benches/**' --glob '!**/fuzz/**'
```

---

## Gap Audit

| Requirement | Coverage | Evidence |
| ----------- | -------- | -------- |
| PH27-MEMLOCK | COVERED | `test_os_hardening_integration` + memlock ordering unit seam in `os_hardening.rs` + Miri command recorded in `027-VERIFICATION.md` |
| PH27-CONFIG | COVERED | `test_config_integration` + `config_demo` example check |
| PH27-TIME | COVERED | `test_time_policy_micros` + `time_provider_demo` example check + clean production-perimeter scan + downstream wallet/core/simulator anchors |
| PH27-RNG | COVERED | `genesis_tests` + simulator release gates + allowlist checks recorded in `027-04-SUMMARY.md` |
| PH27-LOGGER | COVERED | `test_logger_integration` + green full workspace release gate |
| PH27-IO | COVERED | `test_io_integration` + green full workspace release gate |
| PH27-JSON | COVERED | `test_codec_integration` + `test_logger_integration` + green full workspace release gate |

📌 No Nyquist gaps were found during reconstruction, so no additional test files were required.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or existing infrastructure coverage
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency kept at task or wave granularity through existing Cargo commands
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-29
