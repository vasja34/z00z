---
phase: 032-crypto-audit-scenario-1
plan: "04"
subsystem: crypto-audit
tags: [scenario1, spend-verifier, public-contract, fail-closed, wallet, simulator]
requires:
  - phase: 032-crypto-audit-scenario-1
    plan: "03"
    provides: authoritative claim-root contract and simulator claim publication path used as the previous-root input for spend verification
provides:
  - real public spend-verifier boundary rooted in wallet tx seams instead of structural witness-only acceptance
  - Scenario 1 stage-4 tx packages that persist spend proof/auth and verify them before package finalization
  - wallet and simulator regression coverage that rejects placeholder or overclaimed public spend success paths
affects: [032-05, 032-06, 032-07, scenario1, spend-acceptance]
tech-stack:
  added: []
  patterns:
    - canonical spend statement signed by receiver identity key and verified from persisted tx proof/auth
    - witness preparation separated from accepted public-verifier enforcement
    - simulator logs and helper names aligned to truthful public-contract semantics
key-files:
  created:
    - .planning/phases/032-crypto-audit-scenario-1/032-04-SUMMARY.md
    - crates/z00z_simulator/tests/test_scenario1_spend_gate.rs
  modified:
    - crates/z00z_wallets/src/core/tx/spend_verification.rs
    - crates/z00z_wallets/src/core/tx/witness_gate.rs
    - crates/z00z_wallets/src/core/tx/prover.rs
    - crates/z00z_wallets/src/core/tx/tx_wire_types.rs
    - crates/z00z_wallets/src/core/tx/mod.rs
    - crates/z00z_wallets/tests/test_spend_witness_gate.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs
    - crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs
    - crates/z00z_simulator/tests/test_stage4_gates.rs
    - crates/z00z_simulator/tests/test_stage4_digest.rs
key-decisions:
  - "Accepted spend flows now require persisted `TxWire.proof.spend` and `TxWire.auth.spend`, signed over a canonical public statement framed with chain/version and previous-root inputs."
  - "Duplicate input `leaf_ad_id` values remain allowed because Scenario 1 claim-origin inputs may legitimately reuse `asset.definition.id`; uniqueness stays enforced on input state refs, output `leaf_ad_id`, and input/output overlap."
  - "Simulator stage-4 narrative was upgraded from witness-gate language to public-contract language so logs and integration tests do not overclaim what is being verified."
patterns-established:
  - "Public spend verification binds the previous root, tx input refs, tx outputs, per-input proof rows, receiver card, and spend authorization signature into one canonical statement."
  - "Witness helpers may prepare inputs, but accepted paths must terminate in `verify_tx_public_spend_contract(...)` or an equivalent wrapper that proves the same contract."
  - "Targeted phase closeout evidence must use exact `--test ...` integration binaries when plain name filters can silently run zero tests."
requirements-completed: []
duration: continuation session
completed: 2026-04-05
---

# Phase 032 Plan 04: Honest Spend Public Verifier Summary

Scenario 1 stage-4 spend acceptance now persists and verifies a real public spend contract signed by the receiver identity key, and both wallet and simulator tests fail when structural-only or placeholder witness data tries to masquerade as public verification.

## Performance

- **Duration:** continuation session
- **Started:** carried over from prior Phase 032 execution state
- **Completed:** 2026-04-05T23:59:00Z
- **Tasks:** 2
- **Files modified:** 16

## Accomplishments

- Added a real public spend-verifier boundary in [crates/z00z_wallets/src/core/tx/spend_verification.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_verification.rs) that decodes persisted spend proof/auth, recomputes leaf-ad relations, checks range proofs and balance, and verifies the receiver-signed canonical statement.
- Rewired Scenario 1 stage-4 package construction in [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs) so `TxWire` carries real `proof.spend` and `auth.spend` before digest/package finalization and logs success as `spend_public_contract`.
- Expanded wallet coverage in [crates/z00z_wallets/tests/test_spend_witness_gate.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_spend_witness_gate.rs) to reject missing auth, replayed `prev_root`, and tampered `leaf_ad_hash`, and added [crates/z00z_simulator/tests/test_scenario1_spend_gate.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_scenario1_spend_gate.rs) so simulator acceptance agrees with the wallet verifier on both green and fail-closed paths.
- Updated stage-4 regression tests in [crates/z00z_simulator/tests/test_stage4_gates.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage4_gates.rs) and [crates/z00z_simulator/tests/test_stage4_digest.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage4_digest.rs) so the narrative and persisted package invariants now reflect real public-contract verification.

## Post-Closeout Correction

Follow-up audit on 2026-04-05 confirmed that this plan did harden the spend gate honestly, but it did not close `PH32-SPEND` as originally written.

- The landed verifier binds previous root, canonical input refs, outputs, range-proof commitments, `chain_id`, version, and framed statement bytes.
- The live regular-spend wire and persisted spend proof do not carry a nullifier field, so this plan cannot honestly claim that nullifier semantics are part of the delivered public spend statement.
- This summary therefore remains valid as evidence for current-stack spend-gate hardening, but it must not be used as proof that `PH32-SPEND` is complete.

## Task Commits

No task commit was created in this execution pass.

The repository-required `/z00z-git-versioning` flow remains release-tag oriented. Because Phase 032 is still executing sequentially inside one active worktree, this plan was closed summary-first and the next explicit version-managed sync remains deferred to a deliberate checkpoint instead of generating a misleading mid-phase release tag.

## Files Created/Modified

- [crates/z00z_wallets/src/core/tx/spend_verification.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_verification.rs) - Added the canonical public spend statement encoder, persisted proof/auth builders, and fail-closed verifier.
- [crates/z00z_wallets/src/core/tx/witness_gate.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/witness_gate.rs) - Demoted the witness gate to witness/public-input preparation plus wrapper verification over the real public contract.
- [crates/z00z_wallets/src/core/tx/prover.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/prover.rs) - Added spend authorization signing and verification helpers with an explicit domain-separation label.
- [crates/z00z_wallets/src/core/tx/tx_wire_types.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/tx_wire_types.rs) - Added persisted spend proof/auth wire objects on `TxProofWire` and `TxAuthWire`.
- [crates/z00z_wallets/src/core/tx/mod.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/mod.rs) - Re-exported the new public spend helpers and spend wire types for wallet/simulator use.
- [crates/z00z_wallets/src/core/tx/spend_rules.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_rules.rs) - Aligned public rule naming from `asset_id_in` to `leaf_ad_id_in` so the verifier contract matches the wire semantics.
- [crates/z00z_wallets/tests/test_view_key_contract.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_view_key_contract.rs) - Updated spend statement fixture naming to the canonical `leaf_ad_id_in` field.
- [crates/z00z_wallets/tests/test_spend_witness_gate.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_spend_witness_gate.rs) - Added direct public-contract acceptance and rejection coverage.
- [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs) - Builds persisted spend proof/auth, verifies them, and writes truthful stage-4 logs.
- [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs) - Renamed the simulator wrapper to `verify_spend_public_contract(...)` so the internal API matches the honest boundary.
- [crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs) - Re-exported the renamed simulator helper.
- [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs) - Imported the new wallet verifier/build helpers and the renamed simulator helper.
- [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs) - Updated direct helper tests to call `verify_spend_public_contract(...)`.
- [crates/z00z_simulator/tests/test_stage4_gates.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage4_gates.rs) - Updated event expectations to `spend_public_contract` / `public spend contract passed`.
- [crates/z00z_simulator/tests/test_stage4_digest.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage4_digest.rs) - Added assertions that persisted stage-4 packages actually contain spend proof/auth.
- [crates/z00z_simulator/tests/test_scenario1_spend_gate.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_scenario1_spend_gate.rs) - Added Scenario 1 integration coverage that reuses the wallet verifier against the persisted tx package.

## Decisions Made

- The spend acceptance boundary now lives in the persisted public verifier, not in structural witness preparation helpers.
- The canonical statement is signed with the receiver identity key exported through the compact receiver card, which keeps simulator and wallet verification on one public contract instead of a simulator-only signing path.
- Duplicate input `leaf_ad_id` rejection was removed on the input side because claim-origin inputs may legitimately share one asset-definition id; overlap detection still fences input/output replay risk.

## Review Passes

- **Pass 1:** Spec and threat-model review against [032-04-PLAN.md](/home/vadim/Projects/z00z/.planning/phases/032-crypto-audit-scenario-1/032-04-PLAN.md) confirmed the plan truths are now reflected in wallet verifier seams, simulator stage-4 logs, and explicit negative regression coverage.
- **Pass 2:** Crypto/security review of the persisted verifier path found no unresolved acceptance bypass after confirming signer binding, previous-root replay fencing, `leaf_ad_hash` recomputation, input/output disjointness, range-proof checks, and fail-closed handling for missing auth/proof. Clean.
- **Pass 3:** Validation review stayed clean after re-running the exact wallet and simulator integration binaries and the simulator rename-neutrality rerun. Clean.

The last two review passes were consecutive clean runs.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Relaxed false duplicate rejection on input `leaf_ad_id` values**

- **Found during:** Task 2 validation
- **Issue:** Scenario 1 stage-5 rejected an honest stage-4 package with `public spend contract verify failed: duplicate leaf_ad id` because the verifier assumed input `leaf_ad_id` uniqueness that claim-origin inputs do not guarantee.
- **Fix:** Kept unique input state refs, unique output `leaf_ad_id`, and input/output disjointness, but stopped rejecting duplicate input `leaf_ad_id` values.
- **Files modified:** [crates/z00z_wallets/src/core/tx/spend_verification.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_verification.rs)
- **Verification:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump stage4_public_contract_ok -- --nocapture`
- **Committed in:** Not yet committed; pending next explicit version-managed sync

**2. [Rule 1 - Bug] Fixed wallet public-contract fixture so negative tests target authorization semantics instead of synthetic balance failure**

- **Found during:** Task 1 validation
- **Issue:** The first direct wallet public-contract fixture created a fresh output with a different commitment blinding, so honest tests failed early with `BadBalance` before reaching replay/auth rejection logic.
- **Fix:** Reused the input commitment and changed only `leaf_ad_id` in the unit fixture, preserving balance while keeping input/output overlap avoided.
- **Files modified:** [crates/z00z_wallets/tests/test_spend_witness_gate.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_spend_witness_gate.rs)
- **Verification:** `cargo test -p z00z_wallets --release --features test-fast --test test_spend_witness_gate -- --nocapture`
- **Committed in:** Not yet committed; pending next explicit version-managed sync

**3. [Rule 2 - Missing Critical] Removed remaining simulator internal overclaim drift**

- **Found during:** Closeout review
- **Issue:** Stage-4 user-visible logs were already honest, but the simulator helper API still exported `verify_spend_witness_gate(...)`, which preserved the old boundary narrative in internal code.
- **Fix:** Renamed the simulator wrapper/API usage to `verify_spend_public_contract(...)` and updated the direct runtime tests accordingly.
- **Files modified:** [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs), [crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs), [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs), [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs), [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs)
- **Verification:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`
- **Committed in:** Not yet committed; pending next explicit version-managed sync

**4. [Rule 3 - Blocking] Switched final closeout evidence to exact integration-test binaries**

- **Found during:** Task verification
- **Issue:** The plan's name-filter form can compile successfully while running zero relevant tests, which is not acceptable closeout evidence.
- **Fix:** Used exact integration-test targets for the final green evidence.
- **Files modified:** None
- **Verification:** `cargo test -p z00z_wallets --release --features test-fast --test test_spend_witness_gate -- --nocapture`; `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`
- **Committed in:** Not yet committed; pending next explicit version-managed sync

---

**Total deviations:** 4 auto-fixed (2 bugs, 1 missing critical, 1 blocking)
**Impact on plan:** All deviations were required for correctness, truthful boundary naming, or trustworthy validation evidence. No out-of-scope feature work was introduced.

## Issues Encountered

- Honest Scenario 1 claim-origin inputs can share one `leaf_ad_id`, so a naive input-side uniqueness rule in the public verifier created a false reject and had to be aligned with the existing claim semantics.
- Direct wallet public-contract unit coverage initially tripped the verifier's real balance equation before it reached the intended auth/replay assertions, which required a balance-preserving fixture design.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_wallets --no-run`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --no-run`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump stage4_public_contract_ok -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump stage4_digest_stable_with_seed -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast --test test_spend_witness_gate -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`

## User Setup Required

None.

## Next Phase Readiness

- Scenario 1 spend acceptance now has a persisted, test-backed public verifier that future phases can reuse instead of arguing from witness structure or simulator-local overclaims.
- Later Phase 032 work can build adversarial or checkpoint-coupled coverage on top of this contract without reopening the honesty gap closed here.

## Threat Flags

None.

## Self-Check: PASSED

- Verified [.planning/phases/032-crypto-audit-scenario-1/032-04-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/032-crypto-audit-scenario-1/032-04-SUMMARY.md) exists.
- Verified [crates/z00z_wallets/src/core/tx/spend_verification.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_verification.rs) exists.
- Verified [crates/z00z_wallets/tests/test_spend_witness_gate.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_spend_witness_gate.rs) exists.
- Verified [crates/z00z_simulator/tests/test_scenario1_spend_gate.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_scenario1_spend_gate.rs) exists.
- Verified [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs) exists.

---
*Phase: 032-crypto-audit-scenario-1*
*Completed: 2026-04-05*
