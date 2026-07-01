---
phase: 034-mix1-fixes
plan: 02
subsystem: spend-nullifier
tags: [spend-nullifier, chain-id, public-verifier, stage4, simulator, fail-closed]
requires:
  - phase: 034-01
    provides: storage-backed claim continuity baseline for downstream Scenario 1 flows
  - phase: 034-03
    provides: sender-authority retirement over the now-stable spend-nullifier contract
provides:
  - Dedicated `SpendNullifierDomain` with one canonical `derive_spend_nullifier(chain_id, s_in)` helper
  - Explicit `nullifier_hex` spend wire field and structural `SpendIn` nullifier plus `chain_id` contract
  - Signed public-verifier nullifier enforcement plus structural deterministic mismatch enforcement
  - Stage 4, Stage 5, Stage 6, and simulator interop package loaders bound to verified package truth
affects: [034-03, 034-06, PH34-SPEND-NULLIFIER, scenario-1-spend-flow]
tech-stack:
  added: []
  patterns: [canonical nullifier helper, chain-bound spend semantics, verified-package-before-deserialize, fail-closed consumer reuse]
key-files:
  created:
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-02-SUMMARY.md
  modified:
    - /home/vadim/Projects/z00z/crates/z00z_crypto/src/domains.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/tx_wire_types.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_rules.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/witness_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_verification.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/tx_verifier.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/tx_verifier_tests.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/claim_wire_types.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/mod.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_spend_witness_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_scenario1_semantics.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_s5_closure_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_support.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_runtime_support.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_impl.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/examples/simulator_interop/support.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_scenario1_spend_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_scenario1_stage_surface.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_stage5_receive_bridge.rs
    - /home/vadim/Projects/z00z/.planning/REQUIREMENTS.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-CONTEXT.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-TODO.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-TEST-SPEC.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-TESTS-TASKS.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-02-PLAN.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-06-PLAN.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-fix-spec-4.md
key-decisions:
  - "Keep the shipped boundary honest: the standalone public verifier authenticates one signed nullifier field, while witness and structural layers enforce deterministic `chain_id || s_in` recomputation."
  - "Require explicit regular `TxPackage` chain metadata and propagate `pkg.chain_id` through Stage 4, Stage 6, and example validators instead of relying on a devnet constant."
  - "Force Stage 4, Stage 5, Stage 6, and example package consumers to verify raw package bytes before deserialize so tampered chain metadata or signed-field drift cannot bypass the canonical verifier path."
patterns-established:
  - "Downstream regular-tx consumers must reuse verified package truth; raw JSON deserialize is not an authority surface for Stage 4 packages."
requirements-completed: [PH34-SPEND-NULLIFIER]
completed: 2026-04-10
revision: nullifier-closure-clean2
reviewed: 2026-04-10T00:00:00Z
---

# Phase 034 Plan 02 Summary

## Outcome

Plan 02 is complete. The regular spend-nullifier contract now has one canonical domain, one canonical derivation helper, one signed public field, and one fail-closed structural recomputation path, with downstream stage and example consumers reusing the same verified package truth.

## Accomplishments

- Added `SpendNullifierDomain` and exported one canonical `derive_spend_nullifier(chain_id, s_in)` helper instead of cloning derivation logic across wallet and simulator seams.
- Extended the regular spend wire and structural rule surfaces with explicit nullifier data and explicit `chain_id` framing so deterministic `chain_id || s_in` semantics can be checked consistently.
- Bound the public spend contract to `nullifier_hex`, rejecting malformed hex, duplicate nullifiers, and post-signature drift, while the structural rule layer rejects deterministic mismatch and duplicate values.
- Removed implicit regular-tx chain metadata defaults and pushed the package `chain_id` through Stage 4 build, Stage 4 verification, Stage 5 receive-bridge load, Stage 6 package load, and simulator interop example load.
- Tightened active requirements and wording guards so they describe the shipped boundary honestly instead of overstating the standalone public verifier’s role.

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed as the mandatory fail-fast gate.
- `cargo test -p z00z_wallets --release test_s1_spend_boundary -- --nocapture` passed.
- `cargo test -p z00z_wallets --release test_s1_nullifier_closure -- --nocapture` passed.
- `cargo test -p z00z_wallets --release delivered_closure_stays_tied_to_open_claim_and_spend_gaps -- --nocapture` passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump test_phase033_task64_keeps_nullifier_scope -- --nocapture` passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump stage5_rejects_tampered_stage4_package_chain_metadata -- --nocapture` passed.
- `cargo test -p z00z_simulator --release --example simulator_interop package_loader_rejects_tampered_stage4_chain_metadata -- --nocapture` passed.
- `cargo test --release --features test-fast --features wallet_debug_dump` passed on the current tree after the stage and example loader fixes.
- The scoped Phase 034 review loop converged with two consecutive `CLEAN` passes for task `034-04 Regular-Spend Verifier And Rule Integration`.

## Files Created Or Modified

- `/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_rules.rs` now owns the canonical nullifier derivation helper and structural nullifier enforcement.
- `/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/spend_verification.rs` now authenticates `nullifier_hex` on the signed public seam and rejects malformed, duplicate, and drifted nullifier values.
- `/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/witness_gate.rs` now derives spend nullifiers from the caller’s `chain_id` instead of an implicit constant.
- `/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs` now verifies both package structure and the embedded public spend contract before accepting Stage 4 package bytes.
- `/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_support.rs` and `/home/vadim/Projects/z00z/crates/z00z_simulator/examples/simulator_interop/support.rs` now fail closed by verifying Stage 4 package bytes before deserialize.

## Issues Encountered

- Active requirements and wording guards initially overstated the standalone public verifier as if it independently recomputed deterministic nullifiers; this was narrowed to the shipped public-field plus witness-and-structural split before closure.
- A Stage 5 receive-bridge seam still raw-deserialized Stage 4 packages and could bypass the stricter package truth; the loader was changed to verify bytes first.
- A downstream simulator interop example still had the same raw-deserialize bypass; that example loader was fixed and regression-tested separately.

## Next Phase Readiness

- `034-03` can now start from a stable spend-nullifier baseline: regular-tx chain metadata is explicit, the canonical nullifier helper is frozen, and downstream package consumers no longer bypass the verified package path.
- Documentation reclassification remains blocked until the checkpoint-backend closure and later validation waves complete under Plans 04 through 08.

## Known Stubs

None for the Plan 02 spend-nullifier scope.

## Threat Flags

None. The active spend seam is now fail closed on malformed nullifier hex, duplicate nullifiers, signed-field drift, structural deterministic mismatch, and tampered Stage 4 package chain metadata across the validated stage and example consumers.

## Self-Check

PASSED.

---
*Phase: 034-mix1-fixes*
*Completed: 2026-04-10*
