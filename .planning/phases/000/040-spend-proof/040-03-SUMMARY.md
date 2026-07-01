# 040-03 Summary

## ✅ Scope

This summary records the completion state for `040-03-PLAN.md`, covering the
landed `040-05 Nullifier Semantics` and `040-06 Full Regular Package
Verification Entry Point` slice of Phase 040.

## ✅ Outcome

Plan 03 is closed on the already-landed nullifier/full-verifier slice.

Regular-spend nullifiers are deterministic and chain-scoped, remain distinct
from claim nullifiers, and are carried through the public spend statement while
post-acceptance replay rejection stays explicit on the current asset-id spent
boundary. At the same time, `verify_full_tx_package(...)` remains the one
canonical full regular-package verifier, and both wallet and simulator paths
prove that local wire validation alone is insufficient when the public spend
contract fails.

## ✅ Repository Changes

- `crates/z00z_wallets/src/core/tx/spend_rules.rs` now owns deterministic
  `derive_spend_nullifier(...)`, duplicate-nullifier rejection, and fail-closed
  nullifier validation inside the shared spend-rule contract.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs` carries the
  deterministic nullifier vector through the shared public spend statement so
  producer and verifier consume the same scoped nullifier surface.
- `crates/z00z_wallets/src/core/tx/state_update.rs` keeps replay closure on the
  existing checkpoint/state boundary, where repeated accepted inputs are
  rejected through the live asset-id spent model rather than through a new
  standalone nullifier store.
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs` keeps
  `verify_full_tx_package(...)` as the one canonical entry point that composes
  local package checks with the public spend contract.
- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs` locks the wallet
  side of the full-verifier contract, including proofless-package rejection and
  rejection of packages that pass local wire checks but fail the public spend
  boundary.
- `crates/z00z_wallets/tests/test_spend_nullifier_semantics.rs` locks same-scope
  determinism, scope drift, claim-nullifier separation, duplicate rejection,
  and bad-nullifier rejection on the current spend-rule surface.
- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` locks replay
  rejection and checkpoint acceptance truth on the current asset-id spent
  boundary.
- `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` locks the
  simulator-side regression proving that local wire success is insufficient when
  the public spend contract fails.

## ✅ Validation

- Focused nullifier validation passed:
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_nullifier_semantics -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture`
- Focused full-verifier validation passed:
  - `cargo test -p z00z_wallets --release --features test-fast --lib test_full_verifier_rejects_missing_public_spend_contract -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --lib test_public_spend_boundary_rejects_local_valid_package -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`
- Latest T3 rerun-backed evidence also passed on 2026-04-25 during the Phase
  040 test-execution review loop:
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_nullifier_semantics -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --lib core::tx::tx_verifier::tests -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`
- Bootstrap and broad release gates remained green from the same closeout cycle
  with no intervening code edits:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test --release --features test-fast --features wallet_debug_dump`

## ✅ Review Loop

The closeout stayed local, evidence driven, and tied to the explicit GSD review
gate.

1. `/GSD-Review-Tasks-Execution` was the mandatory closeout prompt for the
  landed `040-05`/`040-06` slice, so the nullifier/full-verifier artifacts had
  to survive direct task-scoped review instead of relying on earlier optimism.
2. The review sequence met the GSD floor of three YOLO passes, and focused
  wallet plus simulator reruns were used to resolve any review concern before
  the next pass.
3. Plan 03 was treated as closed only after the final review sequence produced
  at least two consecutive passes with no significant issues on the same
  landed slice.

## ✅ Current Boundary

This summary closes only the third Phase 040 execution plan. The next active
slice is `040-04-PLAN.md`, which must prove end-to-end Stage-4/6/11 continuity
for the proof-bearing package and keep any output-constructor cleanup strictly
bounded behind that roundtrip truth.
