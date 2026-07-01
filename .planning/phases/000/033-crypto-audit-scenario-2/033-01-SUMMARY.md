---
phase: 033-crypto-audit-scenario-2
plan: 1
subsystem: claim-integrity
completed_at: 2026-04-06T21:20:06Z
tags:
  - phase-033
  - claim-trust
  - claim-verifier
  - simulator-consumer
dependency_graph:
  requires:
    - PH32-CLAIM-BIND
    - PH32-CLAIM-TRUST
    - PH32-HONEST
  provides:
    - narrowed-helper-owned-claim-trust-boundary
    - verified-tuple-drift-tests
    - verified-precise-reject-branches
  affects:
    - .planning/REQUIREMENTS.md
    - docs/code-review/032-scenario-1-crypto-status.md
    - .planning/phases/033-crypto-audit-scenario-2/033-CONTEXT.md
    - .planning/phases/033-crypto-audit-scenario-2/033-TODO.md
tech_stack:
  added: []
  patterns:
    - helper-owned canonical claim-source contract
    - fail-closed claim verifier rejection
    - release-style targeted validation
key_files:
  modified:
    - .planning/REQUIREMENTS.md
    - docs/code-review/032-scenario-1-crypto-status.md
    - .planning/phases/033-crypto-audit-scenario-2/033-CONTEXT.md
    - .planning/phases/033-crypto-audit-scenario-2/033-TODO.md
  validated:
    - crates/z00z_wallets/src/core/tx/test_claim_tx.rs
    - crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs
decisions:
  - Narrow PH32-CLAIM-TRUST to the current helper-owned canonical one-item claim-source contract instead of overclaiming persisted storage-backed continuity.
  - Treat Task 1 and Task 3 as satisfied by the existing repository tests only after exact release-style verification proved the seams green in the current tree.
metrics:
  task_commits: 1
  docs_commits: 0
  targeted_wallet_tests: 4
  targeted_simulator_tests: 8
---

# Phase 033 Plan 01: Claim Integrity First Slice Summary

Phase 033 plan 01 now closes the first claim-integrity slice by narrowing `PH32-CLAIM-TRUST` to the helper-owned canonical claim-source boundary the code actually enforces, while verifying that tuple-drift and precise reject-class seams are already green at the live wallet and simulator boundaries.

## Completed Tasks

### Task 1: Full Tuple Or Partial Story

- Verified the live wallet verifier already contains exact end-to-end tuple-drift tests for post-sign `claim_source_asset_id` and `chain_id` mutations in [crates/z00z_wallets/src/core/tx/test_claim_tx.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/test_claim_tx.rs).
- Confirmed both wallet verifier tests pass under release configuration:
  - `core::tx::claim_tx::claim_tx_tests::test_claim_source_asset_id_drift_rejected`
  - `core::tx::claim_tx::claim_tx_tests::test_chain_id_drift_rejected_before_proof`
- No code edit was required for Task 1 because the seam already existed in the current tree and the validation proved it green.

### Task 2: Authoritative Store Or Local Reconstruction

- Chose the honest requirement-narrowing branch explicitly allowed by [033-01-PLAN.md](/home/vadim/Projects/z00z/.planning/phases/033-crypto-audit-scenario-2/033-01-PLAN.md) because the accepted path still relies on helper-owned one-item reconstruction rather than persisted storage-backed membership continuity.
- Updated [REQUIREMENTS.md](/home/vadim/Projects/z00z/.planning/REQUIREMENTS.md) to close `PH32-CLAIM-TRUST` only under the narrowed helper-owned canonical boundary.
- Aligned [032-scenario-1-crypto-status.md](/home/vadim/Projects/z00z/docs/code-review/032-scenario-1-crypto-status.md), [033-CONTEXT.md](/home/vadim/Projects/z00z/.planning/phases/033-crypto-audit-scenario-2/033-CONTEXT.md), and [033-TODO.md](/home/vadim/Projects/z00z/.planning/phases/033-crypto-audit-scenario-2/033-TODO.md) so active phase truth no longer says `PH32-CLAIM-TRUST` remains open.
- Task commit: `02acf0d8` with message `docs(033-01): narrow claim trust boundary`.

### Task 3: Precise Reject Semantics

- Verified the live wallet verifier already contains dedicated version-drift tests in [crates/z00z_wallets/src/core/tx/test_claim_tx.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/test_claim_tx.rs):
  - `core::tx::claim_tx::claim_tx_tests::test_source_root_ver_rejected_with_precise_error`
  - `core::tx::claim_tx::claim_tx_tests::test_source_proof_ver_rejected_with_precise_error`
- Verified the simulator consumer already pins deterministic stale-proof and version-reject behavior in [crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs):
  - `test_claim_pkg_consumer_rejects_stale_storage_proof`
  - `test_claim_pkg_consumer_rejects_source_root_ver`
  - `test_claim_pkg_consumer_rejects_source_proof_ver`
- No code edit was required for Task 3 because the current tree already satisfied the named reject-branch coverage once validated directly.

## Validation

### Bootstrap Gate

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```

- Result: passed in-session before targeted release verification.

### Wallet Verifier Exact Tests

```bash
cargo test -p z00z_wallets --release --lib \
  core::tx::claim_tx::claim_tx_tests::test_claim_source_asset_id_drift_rejected \
  -- --exact --nocapture

cargo test -p z00z_wallets --release --lib \
  core::tx::claim_tx::claim_tx_tests::test_chain_id_drift_rejected_before_proof \
  -- --exact --nocapture

cargo test -p z00z_wallets --release --lib \
  core::tx::claim_tx::claim_tx_tests::test_source_root_ver_rejected_with_precise_error \
  -- --exact --nocapture

cargo test -p z00z_wallets --release --lib \
  core::tx::claim_tx::claim_tx_tests::test_source_proof_ver_rejected_with_precise_error \
  -- --exact --nocapture
```

- Result: all 4 exact wallet verifier tests passed.

### Simulator Consumer Release Tests

```bash
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump \
  --test test_claim_pkg_crypto_support -- --nocapture
```

- Result: 8 passed, 0 failed.
- Relevant covered cases include bundle-version rejection, raw-shape rejection, stale-proof rejection, source-root version rejection, source-proof version rejection, authority-anchor rejection, and wrong-signature rejection.

## Decisions Made

1. The plan closed `PH32-CLAIM-TRUST` by narrowing the requirement, not by pretending the helper seam is persisted authoritative continuity.
2. Existing verifier and consumer tests were accepted as plan-complete evidence only after exact release-style reruns proved the named seams green in the current repository state.

## Deviations from Plan

### Auto-applied Plan Branch

- Task 2 took the explicit requirement-narrowing branch already authorized by the plan instead of a larger persisted-storage refactor.
- This was not an architectural deviation outside plan scope; it was one of the plan's stated honest closure paths.

### Review Workflow Caveat

- The plan requires repeated `/GSD-Review-Tasks-Execution` YOLO review loops.
- This session validated the concrete code and documentation seams directly, but did not execute an external multi-pass review agent loop because that workflow is not exposed as an executable tool in the current environment.

## Known Stubs

None.

## Threat Flags

None.

## Self-Check

- SUMMARY file path is present at `.planning/phases/033-crypto-audit-scenario-2/033-01-SUMMARY.md`.
- Task commit `02acf0d8` exists and contains the Task 2 requirement/doc narrowing.
- Verified files referenced above exist in the repository.

## Self-Check: PASSED
