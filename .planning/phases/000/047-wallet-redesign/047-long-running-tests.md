# Phase 047 Long-Running Release Tests

**Date:** 2026-05-21  
**Scope:** Release-mode profiling and remediation of the `wallet_service_tests` harness  
**Primary command requested:** `cargo test --release --features test-fast --features wallet_debug_dump`

## ✅ Verification Summary

- The release-only investigation reproduced the real issue inside the shared
  `crates/z00z_wallets/src/services/wallet_service_tests.rs` harness instead of
  finding a single intrinsically slow test.
- Root cause 1: `test_yaml_auto_lock` self-deadlocked by acquiring the
  wallet-config env lock and then calling a helper that tried to acquire the
  same process-global lock again.
- Root cause 2: the sync and async wallet-config env locks were not actually
  shared, so `test_open_source_backfills_yaml` could race under the full
  parallel harness and silently fall back to embedded defaults.
- Remediation:
  - unified sync/async config-path tests onto one semaphore-backed global lock;
  - split the wallet-service test helper into locked and raw variants so
    env-mutating tests do not recurse into the same lock;
  - tightened the concurrent derive watchdog from `90s` to `10s` so future
    regressions fail fast instead of stretching release runs.
- Result after remediation:
  - `bootstrap_tests.sh` passed;
  - `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets --lib wallet_service_tests::tests::` passed;
  - the shared wallet-service harness finished in `0.83s` after the final fix.

## 📋 Confirmed Tests Over 15 Seconds

| # | test_name | test_path | recommendation | comments |
| --- | --- | --- | --- | --- |
| 1 | none confirmed | n/a | Keep the unified env lock and fast-fail timeout in place; no additional test splitting is required for Phase 047 closeout. | After the fix, the full `wallet_service_tests` release harness completed in under 1 second and no isolated wallet-service test exceeded `0.25s`. |

## 🔬 Profiling Snapshot

Release-mode timing against the compiled `z00z_wallets` libtest binary:

- whole harness with `--test-threads 1`: `3.30s`
- whole harness with `--test-threads 2`: `1.76s`
- whole harness with `--test-threads 4`: `1.10s`
- whole harness with `--test-threads 8`: `0.91s`
- final cargo-driven harness run after the shared-lock fix:
  `101 passed`, `finished in 0.83s`, command wall time `29.90s`

Slowest isolated wallet-service tests in release mode:

- `test_unlock_path_requires_password`: `0.25s`
- `test_backup_waits_history_lock`: `0.18s`
- `test_ex4_restart_resume`: `0.11s`
- `test_recv_range_restart`: `0.09s`
- `test_derivation_deterministic_restart`: `0.09s`

## 🔍 Interpretation

- Current evidence does **not** support a claim that any
  `wallet_service_tests` release case is inherently long-running.
- Current evidence **does** support that the old “60+ seconds” story came from
  harness-level coordination bugs:
  - a self-deadlock in `test_yaml_auto_lock`;
  - a split sync/async env-lock model that let YAML-backed tests race under the
    shared parallel libtest harness.
- Once those were fixed, the release harness runtime dropped back to normal
  sub-second execution.
- The broad workspace release gate is still expected to be long because of
  acceptance-style simulator and stage tests outside the wallet-service harness.
  Current late-tail examples from a clean release run include:
  - `test_stage4_chain_path`: `55.02s`
  - `test_stage4_output_crypto`: `45.69s`
  - `test_claim_persist_restart`: `44.75s`
  - `test_stage2_secret_artifacts`: `38.83s`
  - `test_claim_publish_stage3_paths`: `32.82s`
  Those runtimes belong to cross-stage acceptance coverage, not to the fixed
  `wallet_service_tests` harness.

## ✅ Recommended Next Step

No extra decomposition work is required for Phase 047 closeout unless a future
change reintroduces either:

- nested wallet-config env lock acquisition inside test helpers; or
- non-shared sync/async env mutation guards around `Z00Z_WALLET_CONFIG_PATH`.
