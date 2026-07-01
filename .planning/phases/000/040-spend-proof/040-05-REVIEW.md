---
phase: 040-spend-proof
reviewed: 2026-04-25T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - crates/z00z_wallets/src/core/tx/spend_rules.rs
  - crates/z00z_wallets/src/core/tx/spend_verification.rs
  - crates/z00z_wallets/src/core/tx/tx_verifier.rs
  - crates/z00z_wallets/tests/test_spend_statement.rs
  - crates/z00z_wallets/tests/test_spend_prover_contract.rs
  - crates/z00z_wallets/tests/test_tx_proof_verifier.rs
  - .planning/STATE.md
  - .planning/phases/040-spend-proof/040-INTEGRITY-GATES.md
  - .planning/phases/040-spend-proof/040-05-SUMMARY.md
  - .planning/phases/040-spend-proof/040-CLOSEOUT-GATES.md
findings:
  critical: 1
  warning: 0
  info: 0
  total: 1
status: issues_found
---

# Phase 040-05 Code Review Report

**Reviewed:** 2026-04-25T00:00:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Planning truth drift is now aligned with code: the active Phase 040 Plan 05 artifacts accurately record the reopened theorem gap and do not overclaim 040-10 or 040-11 closure. One material technical issue remains in the reviewed surface.

The local/full verifier split is intentionally not tracked as a live warning in
this review. The active spec keeps `TxVerifierImpl` as the local package
verifier and names `verify_full_tx_package(...)` as the canonical composed
admission wrapper, so that split is a documented boundary until a separate
design decision changes it.

## Critical Issues

### CR-01: Public spend verifier still accepts a statement-bound hash backend instead of a theorem-carrying proof

**File:** `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`, `crates/z00z_wallets/src/core/tx/spend_verification.rs`

**Issue:** `proof_hex` is now built and verified through `SpendProofBackend`, but the shipped backend still derives its accepted artifact only from canonical statement bytes and a public hash while `SpendProofWitness` stays empty. The validator-facing path therefore authenticates statement integrity only, not the full `verify_spend_rules(...)` theorem. This leaves 040-09 materially open: the carried artifact proves that the statement was bound consistently, but not that a theorem backend re-established the hidden spend-rule relations from an actual proof system.

**Fix:** Replace the current statement-bound hash backend with a backend-produced proof artifact whose prover consumes non-empty witness material and whose verifier replays the full theorem relation, or explicitly narrow the shipped contract so this path is no longer described as proof-preserving.

---

_Reviewed: 2026-04-25T00:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
