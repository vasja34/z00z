---
phase: 034-mix1-fixes
plan: 06
subsystem: semantic-validation-waves
tags: [spend-nullifier, checkpoint-backend, simulator, storage, review-loop, fail-closed]
requires:
  - phase: 034-05
    provides: sender-authority migration baseline consumed by the spend validation wave
  - phase: 034-04
    provides: backend-owned checkpoint contract chain consumed by the checkpoint validation wave
provides:
  - Exact spend-nullifier evidence for missing, malformed, duplicate, and signed-field drift rejection
  - Exact checkpoint-backend evidence for compatibility-only rejection, proof-system mismatch, and reload-drift rejection
  - Correct plan-owned verify commands using named integration-test binaries instead of false green test filters
affects: [034-06, PH34-SPEND-NULLIFIER, PH34-CHECKPOINT-BACKEND]
tech-stack:
  added: []
  patterns: [exact integration-test binaries, fail-closed semantic guards, review-driven validation tightening]
key-files:
  created:
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-06-SUMMARY.md
  modified:
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-06-PLAN.md
    - /home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_spend_witness_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_scenario1_spend_gate.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/tests/test_checkpoint_finalization.rs
key-decisions:
  - "Treat named integration-test binaries (`--test ...`) as mandatory for plan-owned evidence so cargo filter-only runs cannot produce false green 0-test results."
  - "Close the spend wave only after explicit missing-nullifier rejection exists on both the wallet and Scenario 1 public seams."
  - "Close the checkpoint wave only after unsupported proof-system bytes fail closed on the load surface in addition to the already-green finalize, store, rehydrate, and promotion guards."
patterns-established:
  - "Phase-owned semantic validation now requires exact seam-home regressions, broader simulator release coverage, and repeated clean review passes before summary-backed closure."
requirements-completed: [034-06]
completed: 2026-04-10
reviewed: 2026-04-10T00:00:00Z
---

# Phase 034 Plan 06 Summary

## Outcome

Plan 06 is complete. The Phase 034 spend-nullifier and checkpoint-backend
closures are now backed by exact task-owned validation waves instead of
narrowed wording alone, and the plan-owned verify commands have been corrected
so they execute the intended integration-test binaries rather than false green
0-test filters.

## Accomplishments

- Added explicit wallet and Scenario 1 regressions proving that an empty
  `nullifier_hex` fails closed on the live public spend seam.
- Added an exact checkpoint regression proving that unsupported proof-system
  bytes reject on the live load surface with `CheckpointError::ProofSysMix`.
- Revalidated the existing spend and checkpoint seam-home suites after the new
  regressions landed.
- Corrected `034-06-PLAN.md` verify commands to use `--test` named
  integration-test binaries for all plan-owned task waves.
- Completed the repeated YOLO review loop and ended on two consecutive clean
  review passes after the in-scope fixes landed.

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed.
- `cargo test -p z00z_wallets --release --test test_spend_witness_gate -- --nocapture` passed with 16 tests.
- `cargo test -p z00z_wallets --release --test test_scenario1_semantics -- --nocapture` passed with 9 tests.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture` passed; the new single-test guard `scenario1_public_spend_gate_rejects_missing_nullifier_value` was re-run directly and passed.
- `cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture` passed with 7 tests.
- `cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture` passed with 11 tests.
- `cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture` passed with 15 tests.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture` passed with 6 tests.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` passed on the current tree.
- `cargo test --release --features test-fast --features wallet_debug_dump` remained outside the Plan 06 seam-owned closure scope and was tracked separately as a broader workspace gate in `.planning/ROADMAP.md`; later Phase 034 closure artifacts superseded that broader-gate note with the fresh rerun transcript.

## Issues Encountered

- The original plan-owned cargo commands used test-name filters without `--test`,
  which produced false green runs with `0 tests` on several integration-test
  binaries. That plan drift was fixed before closure.
- The first two review passes found two real validation gaps: missing-nullifier
  rejection was not explicit on both public spend seams, and unsupported
  proof-system bytes were not exercised on the checkpoint load surface.
- The broader workspace release gate remained outside Plan 06 scope and did not
  change the seam-owned semantic closure recorded by this summary.

## Next Phase Readiness

- Plan 07 can now reclassify live documentation and wording surfaces without
  relying on narrowed claims for Q64 or Q65.
- Stage-surface and requirement wording updates can treat the spend and
  checkpoint closures as implemented, tested seams while still preserving the
  historical audit trail as append-only evidence.

## Threat Flags

None for the Plan 06 scope after the final review-driven fixes.

## Self-Check

PASSED with the broader workspace gate kept outside the Plan 06 closure claim
and later superseded by the fresh Phase 034 rerun evidence.

---
*Phase: 034-mix1-fixes*
*Completed: 2026-04-10*
