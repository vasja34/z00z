---
phase: 043-03
reviewed: 2026-05-06T21:56:38Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - crates/z00z_storage/src/assets/proof.rs
  - crates/z00z_storage/src/assets/store_internal/store_query.rs
  - crates/z00z_storage/src/assets/store_internal/proof_help.rs
  - crates/z00z_wallets/src/tx/commit_audit.rs
  - crates/z00z_wallets/src/tx/mod.rs
  - crates/z00z_wallets/src/tx/spend/spend_verification.rs
  - crates/z00z_wallets/src/tx/verify/tx_verifier.rs
  - crates/z00z_storage/tests/test_assets_suite.rs
  - crates/z00z_storage/tests/test_claim_source_proof.rs
  - crates/z00z_wallets/tests/test_tx_pedersen.rs
  - crates/z00z_wallets/tests/test_spend_proof_backend.rs
  - crates/z00z_wallets/tests/test_tx_wrong_root.rs
  - crates/z00z_wallets/tests/test_tx_verifier_suite.rs
  - crates/z00z_wallets/tests/test_spend_statement.rs
  - crates/z00z_wallets/tests/test_spend_witness_gate.rs
  - crates/z00z_storage/tests/assets/test_store_api.rs
  - crates/z00z_storage/tests/test_checkpoint_root_binding.rs
  - crates/z00z_wallets/tests/test_tx_proof_verifier.rs
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
status: issues_found
---

# Phase 043: Code Review Report

**Reviewed:** 2026-05-06T21:56:38Z
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

I reviewed the storage proof seam, the wallet-side conservation audit, the public spend verifier, and the related regression tests for Phase 043 Plan 03. The main issues are a loss of structured proof-failure taxonomy at the storage wrapper boundary and missing negative coverage for the new asset-class audit branches.

## Warnings

### WR-01: Storage proof wrappers erase the structured failure taxonomy

**File:** [crates/z00z_storage/src/assets/store_internal/store_query.rs](file:///home/vadim/Projects/z00z_storage/src/assets/store_internal/store_query.rs#L48)

**Issue:** `proof_scan()` and `claim_source_proof()` both convert `chk_blob(...)` / proof generation failures into `AssetStoreError::Backend(String)` at [line 60](file:///home/vadim/Projects/z00z_storage/src/assets/store_internal/store_query.rs#L60) and [line 235](file:///home/vadim/Projects/z00z_storage/src/assets/store_internal/store_query.rs#L235). That flattens the distinct `ProofChkErr` classes defined in [proof.rs](file:///home/vadim/Projects/z00z_storage/src/assets/proof.rs#L20), so callers of the new storage scan seam can no longer distinguish root, path, leaf, or backend-proof mismatches even though Phase 043 requires distinct failure classes.

**Fix:** Carry `ProofChkErr` through a structured `AssetStoreError` variant, or return `Result<ProofScanOut, ProofChkErr>` from the scan boundary and let higher-level callers map it at their own boundary.

### WR-02: The new asset-class audit branches lack negative regression coverage

**File:** [crates/z00z_wallets/src/tx/commit_audit.rs](file:///home/vadim/Projects/z00z_wallets/src/tx/commit_audit.rs#L36)

**Issue:** The new audit helper defines `RootBind`, `Leaf`, `SpendProof`, and `AssetClass` failure branches in [commit_audit.rs](file:///home/vadim/Projects/z00z_wallets/src/tx/commit_audit.rs#L36) and implements them in [validate_entry](file:///home/vadim/Projects/z00z_wallets/src/tx/commit_audit.rs#L147), but the current coverage in [test_spend_proof_backend.rs](file:///home/vadim/Projects/z00z_wallets/tests/test_spend_proof_backend.rs#L95) only exercises the happy path, wrong root, missing spend proof, missing total, and total mismatch. There is no regression test that tampers root-bind, leaf fields, spend-proof output leaves, or asset-class selection, so the new separation logic can regress silently.

**Fix:** Add a table-driven negative test in [test_spend_proof_backend.rs](file:///home/vadim/Projects/z00z_wallets/tests/test_spend_proof_backend.rs#L95) that mutates each of those inputs and asserts the matching `AssetClassAuditErr` variant.

---

_Reviewed: 2026-05-06T21:56:38Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
