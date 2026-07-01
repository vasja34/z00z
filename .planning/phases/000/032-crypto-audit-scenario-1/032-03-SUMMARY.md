---
phase: 032-crypto-audit-scenario-1
plan: "03"
subsystem: crypto-audit
tags: [scenario1, claim-trust, storage-boundary, simulator-anchor, fail-closed]
requires:
  - phase: 032-crypto-audit-scenario-1
    plan: "02"
    provides: root-bound claim statement and wallet verifier contract used by producer and consumer trust checks
provides:
  - storage-owned helper for authoritative claim root and proof derivation from one canonical store item
  - Stage 3 claim package production and consumption that re-check the same storage-owned root and proof contract
  - simulator-only authority-anchor enforcement plus negative regression coverage for forged or stale packages
affects: [032-04, 032-05, 032-07]
tech-stack:
  added: []
  patterns:
    - shared canonical claim source contract reused by producer and consumer, with persisted-storage continuity still pending
    - simulator-only claim authority anchor fenced at consume time
    - fail-closed package acceptance on root, proof blob, signature, and anchor drift
key-files:
  created:
    - .planning/phases/032-crypto-audit-scenario-1/deferred-items.md
  modified:
    - crates/z00z_storage/src/assets/store_internal/store_query.rs
    - crates/z00z_storage/tests/test_claim_source_proof.rs
    - crates/z00z_wallets/src/core/tx/claim_auth.rs
    - crates/z00z_wallets/src/core/tx/claim_tx.rs
    - crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs
    - crates/z00z_simulator/src/claim_pkg_consumer.rs
    - crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs
key-decisions:
  - "Stage 3 no longer mints claim proof semantics inline; it requests the shared claim root/proof contract through one storage-owned helper, but that helper still re-derives a synthetic one-item store contract rather than persisted-store continuity."
  - "Accepted Scenario 1 claim package consumption is fenced to the immutable simulator anchor `devnet / z00z-devnet-1` until a later phase introduces a live authority binding."
  - "Producer and consumer both rebuild the same canonical `StoreItem` contract before trusting any claim package root or proof bytes."
patterns-established:
  - "Authority signature validation remains the generic wallet verifier's job, while simulator consumption adds explicit authoritative-root and anchor checks on top."
  - "Targeted integration tests must use exact `--test ...` targets; name filters that run zero tests are not valid closeout evidence."
requirements-completed: []
duration: 22 min
completed: 2026-04-05
---

# Phase 032 Plan 03: Claim Trust Summary

Scenario 1 claim packages now share one canonical claim-source contract across production and consumption, and simulator acceptance fails closed on stale proofs, wrong signatures, and wrong authority anchors. A follow-up audit found that this contract is still re-derived from a synthetic one-item store helper, so the original persisted storage-owned continuity wording remains open.

## Performance

- **Duration:** 22 min
- **Started:** 2026-04-05T11:31:14Z
- **Completed:** 2026-04-05T11:53:30Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added `AssetStore::claim_source_contract_for_item(...)` so claim proof semantics are no longer minted inline in Stage 3 and producer/consumer now share one helper-owned canonical contract.
- Rewired Stage 3 claim package production to request that storage-owned contract and fail closed if the signed statement root drifts from the returned authoritative proof root.
- Rewired claim package consumption to reject packages whose authority anchor, source root, proof version, or proof blob diverge from the same storage-owned authoritative contract.
- Added storage and simulator regression tests that cover the green path plus wrong authority signature, wrong authority anchor, and stale storage-proof rejection.

## Post-Closeout Correction

Follow-up repo audit on 2026-04-05 found that the implementation is materially stronger than the old simulator-local minting path, but still narrower than the original requirement wording.

- `claim_source_contract_for_item(...)` lives in storage code, but it currently constructs a fresh off-backend one-item store and derives root/proof from that synthetic contract.
- Producer and consumer therefore share one canonical helper contract, but they do not yet prove continuity against persisted storage-backed membership state that survives outside the package payload.
- The original `PH32-CLAIM-TRUST` wording remains open until the helper derives proofs from real persisted store state or the requirement is formally narrowed to the current canonical-helper boundary.

## Task Commits

No task commit was created in this execution pass.

The repository-required `/z00z-git-versioning` flow remains release-tag oriented. Because Phase 032 is still executing sequentially inside one active worktree, this plan was closed summary-first and the next explicit version-managed sync remains deferred to a deliberate checkpoint instead of generating a misleading mid-phase release tag.

## Files Created/Modified

- [crates/z00z_storage/src/assets/store_internal/store_query.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/store_query.rs) - Added the helper that derives one canonical claim root and membership proof from a canonical `StoreItem`.
- [crates/z00z_storage/tests/test_claim_source_proof.rs](/home/vadim/Projects/z00z/crates/z00z_storage/tests/test_claim_source_proof.rs) - Added a roundtrip regression that proves the helper matches the normal storage-owned root/proof path exactly.
- [crates/z00z_wallets/src/core/tx/claim_auth.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/claim_auth.rs) - Added explicit simulator-only anchor constants and anchor enforcement.
- [crates/z00z_wallets/src/core/tx/claim_tx.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/claim_tx.rs) - Re-exported the simulator anchor enforcement seam.
- [crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs) - Removed inline local root minting and switched Stage 3 to the storage-owned contract helper.
- [crates/z00z_simulator/src/claim_pkg_consumer.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/claim_pkg_consumer.rs) - Added authoritative root/proof and simulator-anchor rechecks before accepted consumption.
- [crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs) - Added negative trust-boundary coverage and aligned helper use with the storage-owned contract.
- [.planning/phases/032-crypto-audit-scenario-1/deferred-items.md](/home/vadim/Projects/z00z/.planning/phases/032-crypto-audit-scenario-1/deferred-items.md) - Logged the unrelated pre-existing storage complexity warning as an out-of-scope deferred item.

## Decisions Made

- The canonical claim-source boundary stays storage-owned at the API seam even though the current helper still re-derives a one-item synthetic contract; simulator code may request the contract, but not invent a parallel proof-root meaning.
- The current claim authority anchor is explicitly simulator-only and must reject production-looking chain identity until a later phase binds it to live configuration or chain state.
- Consumer-side trust validation layers on top of the generic wallet verifier instead of replacing it, so signature correctness and authoritative-root correctness remain separate explicit checks.

## Review Passes

- **Pass 1:** Spec-drift review against `032-03-PLAN.md` confirmed that producer and consumer now share the same canonical helper contract and no longer trust simulator-local root minting.
- **Pass 2:** Crypto and fail-closed review found no unresolved code issue after exact integration-target reruns proved signature, anchor, and stale-proof rejection paths. Clean.
- **Pass 3:** Validation and regression review stayed clean after `cargo fmt --all`, the bootstrap rerun evidence, and exact release integration tests. Clean.

The last two review passes were consecutive clean runs.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Corrected targeted release validation commands so the exact integration tests actually ran**

- **Found during:** Task verification
- **Issue:** The initial `cargo test -p ... test_claim_source_proof` and `cargo test -p ... test_claim_pkg_crypto_support` forms only filtered by name and could complete with zero executed tests.
- **Fix:** Reran validation with exact integration-test targets: `--test test_claim_source_proof` and `--test test_claim_pkg_crypto_support`.
- **Files modified:** None
- **Verification:** Exact storage and simulator release integration tests both executed and passed.
- **Committed in:** Not yet committed; pending next explicit version-managed sync

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Validation evidence is now exact and trustworthy. No production code scope creep was introduced.

## Issues Encountered

- The mandatory bootstrap log was too large for inline tool output, so it had to be verified from the saved resource tail instead of the direct terminal payload.
- Workspace diagnostics still report one pre-existing cyclomatic-complexity warning on `keep_path` in storage query code. It predates this plan's new helper and was logged to the phase deferred-items file rather than widened into an unrelated refactor.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support -- --nocapture`
- `cargo fmt --all`

## User Setup Required

None.

## Next Phase Readiness

- `032-04` can now define the canonical public spend-verifier contract on top of an honest claim package trust boundary instead of a simulator-local authoritative assumption.
- A later follow-up phase must either anchor `claim_source_contract_for_item(...)` in persisted storage-backed membership state or formally narrow `PH32-CLAIM-TRUST` to the current canonical-helper boundary.
- Later adversarial verification work can treat wrong-anchor and stale-proof rejection as already established Phase 032 behavior.

## Threat Flags

None.

## Self-Check: PASSED

- Verified [.planning/phases/032-crypto-audit-scenario-1/032-03-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/032-crypto-audit-scenario-1/032-03-SUMMARY.md) exists.
- Verified [crates/z00z_storage/src/assets/store_internal/store_query.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/store_query.rs) exists.
- Verified [crates/z00z_simulator/src/claim_pkg_consumer.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/claim_pkg_consumer.rs) exists.
- Verified [crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs) exists.

---
*Phase: 032-crypto-audit-scenario-1*
*Completed: 2026-04-05*
