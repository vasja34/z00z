---
phase: 032-crypto-audit-scenario-1
plan: "02"
subsystem: crypto-audit
tags: [scenario1, claim-v2, root-binding, wallet-verifier, fail-closed]
requires:
  - phase: 032-crypto-audit-scenario-1
    plan: "01"
    provides: frozen Scenario 1 semantic and trust-language contract for claim binding work
provides:
  - expanded `ClaimStmtV2` bytes that bind the authoritative source root and claim scope hash
  - wallet claim-helper and verifier enforcement that fail closed on root, proof, asset-id, and source-commitment drift
affects: [032-03, 032-04, 032-07]
tech-stack:
  added: []
  patterns:
    - narrow contract delta instead of broad claim-pipeline rewrite
    - fail-closed wallet helper verification against canonical source leaf state
key-files:
  created: []
  modified:
    - crates/z00z_crypto/src/claim/v2.rs
    - crates/z00z_crypto/tests/test_claim_v2_contract.rs
    - crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs
    - crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs
    - crates/z00z_wallets/src/core/tx/test_claim_tx.rs
key-decisions:
  - "Phase 032 plan 02 extends the canonical claim-v2 statement minimally with `source_root` and `claim_scope_hash` instead of introducing a broader wire-version fork."
  - "Wallet verification must reject statement/proof/root drift and statement/source-commitment drift before any compatibility-only path can succeed."
  - "The broad workspace release-style test remains corroborating evidence only when captured output is clean but the full run times out."
patterns-established:
  - "Root-bound claim statements must be rebuilt from canonical source-leaf state, not from simulator-local or opaque proof assumptions."
  - "Manual review passes can close the review gate only when at least two consecutive passes are clean."
requirements-completed: [PH32-CLAIM-BIND]
duration: 24 min
completed: 2026-04-05
---

# Phase 032 Plan 02: Claim Tuple Binding Summary

**Expanded `ClaimStmtV2` so the authority signature binds the authoritative source root and claim scope hash, then tightened wallet verification to fail closed on proof, root, asset-id, and source-commitment drift**

## Performance

- **Duration:** 24 min
- **Started:** 2026-04-05T11:07:48Z
- **Completed:** 2026-04-05T11:31:14Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Extended `ClaimStmtV2` with authoritative `source_root` and `claim_scope_hash`, updated canonical serialization, and kept one canonical byte shape for the claim-v2 contract.
- Updated claim-v2 contract tests so roundtrip stability and mismatch coverage now include the expanded authenticated tuple.
- Refactored wallet claim-statement construction and verification so the signed statement is populated from real claim-package state and rejected when the statement drifts from the canonical proof root or canonical source commitment.

## Task Commits

No task commit was created in this execution pass.

The repository-required `/z00z-git-versioning` flow remains release-tag oriented. Because Phase 032 is still executing sequentially inside one active worktree, this plan was closed summary-first and the next explicit version-managed sync remains deferred to a deliberate checkpoint instead of generating a misleading mid-phase release tag.

## Files Created/Modified

- `crates/z00z_crypto/src/claim/v2.rs` - Expanded the canonical claim statement with `source_root`, `claim_scope_hash`, and explicit root mismatch rejection.
- `crates/z00z_crypto/tests/test_claim_v2_contract.rs` - Updated contract vectors and added root mismatch coverage for the new bound statement shape.
- `crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs` - Clarified that wallet helper construction now produces a root-bound claim-v2 statement.
- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` - Derived authoritative source roots from canonical source-leaf state and rejected statement/proof/source-commitment drift fail closed.
- `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` - Added regression coverage for source-commitment drift rejection.

## Decisions Made

- The canonical claim-v2 statement shape grows in place instead of introducing a parallel statement variant or compatibility decoder.
- Wallet verification keeps claim-proof ownership in the canonical `claim_v2` contract and only adds the missing authoritative tuple checks around it.
- Source-commitment drift is treated as a correctness bug in the signed statement boundary, not as a later storage-only concern.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Added fail-closed source-commitment binding to wallet proof verification**

- **Found during:** Task 2 review pass
- **Issue:** The updated verifier checked proof root and asset-id alignment, but it still allowed the signed `claim_source_commitment` to drift away from the canonical source leaf.
- **Fix:** Compared the canonical source leaf commitment against `stmt.claim_source_commitment` in `verify_source_meta()` and added a dedicated wallet regression test.
- **Files modified:** `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`
- **Verification:** `bootstrap_tests.sh`; targeted release claim-v2 contract test; targeted wallet unit tests for `test_report_ok`, `test_proof_blob_root_mix`, and `test_source_commitment_drift_rejected`
- **Committed in:** Not yet committed; pending next explicit version-managed sync

---

**Total deviations:** 1 auto-fixed (1 missing critical functionality)
**Impact on plan:** Tightened the intended fail-closed boundary and closed a real signature-binding gap discovered during review.

## Issues Encountered

- The named `cargo test ... claim_tx` invocation did not exercise the intended wallet unit tests, so validation was rerun with exact `--lib` test names.
- Parallel wallet test attempts hit Cargo build-directory lock contention, so wallet unit validation was rerun sequentially.
- The broad release-style workspace test produced clean captured output across many suites but did not finish before timeout, so it is recorded as corroborating evidence only rather than as a final green completion signal.

## User Setup Required

None.

## Next Phase Readiness

- `032-03` can now move claim production and consumption onto storage-owned authoritative roots and proof retrieval without relying on a partial signed tuple.
- Later spend and checkpoint plans can assume the claim authority signature no longer omits the authoritative root-bound tuple.

## Threat Flags

None.

## Self-Check: PASSED

- Verified `.planning/phases/032-crypto-audit-scenario-1/032-02-SUMMARY.md` exists.
- Verified `crates/z00z_crypto/src/claim/v2.rs` exists.
- Verified `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` exists.
- Verified `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` exists.

---
*Phase: 032-crypto-audit-scenario-1*
*Completed: 2026-04-05*
