---
phase: 060-Gaps-Closing
plan: 060-13
status: complete
completed_at: 2026-06-22
next_plan: 060-14
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-13-PLAN.md
---

# 060-13 Summary: Prepared Tx Balance, Voucher Conservation, And FeeEnvelope Coverage

## Completed Scope

`060-13` is complete for the supplemental wallet/object reject-path reopen.

This slice closes the three remaining wallet/object-side rejection seams on the
live tree without introducing a second validator or a parallel accounting
plane. The regular prepared transaction path now has explicit negative coverage
for plaintext and commitment imbalance on the existing `TxAssembler` gate, the
typed voucher package path now has explicit conservation-mismatch rejection on
the existing object-package contract and wallet RPC mapping, and malformed
`FeeEnvelope` structures now reject on the typed object path while preserving
the shipped split between regular native fee outputs and `FeeEnvelope`.

The release evidence is sufficient on the current tree. The mandatory
bootstrap rerun for the current closeout wave is already green, the plan-owned
targeted `060-13` release filters are green, and the current-tree
`cargo test --release` rerun was already green before this summary with the
same live code surfaces, including the full `scenario_1` tail.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-13-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Boundary Kept

- No second balance validator, no second object-package reject layer, and no
  second accounting plane were introduced.
- Regular `wallet.tx.*` stays cash-only; voucher ids and right ids are still
  rejected on that path.
- Rights remain zero-value control objects and were not widened into
  asset-like balance semantics.
- The native fee-output lane stays separate from `FeeEnvelope`; this slice only
  proves the reject contracts around the shipped split.

## Review Loop

Manual workspace-first review was used instead of repeated slash-prompt
execution because `/GSD-Review-Tasks-Execution` is not a callable tool in this
environment.

- Pass 1 reran the regular prepared-tx reject filters and confirmed that the
  live `TxAssembler` path rejects both plaintext and commitment imbalance
  without any shadow verifier.
- Pass 2 reran the voucher conservation and `FeeEnvelope` reject filters across
  storage, wallet RPC, validator, and the cryptographic claim-flow anchor.
- Pass 3 rechecked the current-tree broad release evidence and confirmed the
  targeted packet is covered by the already-green `cargo test --release` rerun
  because no code changed after that broad gate.
- Pass 4 synchronized `STATE` and `ROADMAP` so the canonical active lane now
  moves to `060-14-PLAN.md` with no status drift.

Two consecutive clean review passes were achieved on passes 3 and 4 after all
targeted reject filters completed green.

## Validation

- Mandatory bootstrap gate was already green on the current closeout wave:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Regular prepared-tx reject filters passed:
  `cargo test -p z00z_wallets --release --lib test_assemble_rejects_ -- --nocapture`
  `cargo test -p z00z_wallets --release --lib test_tx_build_raw_tx -- --nocapture`
  `cargo test -p z00z_wallets --release --lib test_tx_build_rejects_voucher -- --nocapture`
  `cargo test -p z00z_wallets --release --lib test_tx_send_rejects_right -- --nocapture`
- Voucher conservation reject filters passed:
  `cargo test -p z00z_storage --release --lib test_delta_rejects_redeem_mismatch -- --nocapture`
  `cargo test -p z00z_wallets --release test_build_rejects_value_mismatch -- --nocapture`
  `cargo test -p z00z_core --release --features deterministic-rng --test genesis_tests claim_flow::test_claim_cryptographic_balance_validation -- --nocapture`
- `FeeEnvelope` reject filters passed:
  `cargo test -p z00z_storage --release --lib test_delta_rejects_bad_fee -- --nocapture`
  `cargo test -p z00z_storage --release --test test_fee_envelope test_envelope_rejects_pre_mutation -- --nocapture`
  `cargo test -p z00z_storage --release --test test_fee_envelope test_wrong_transition_binding_rejects -- --nocapture`
  `cargo test -p z00z_storage --release --test test_fee_envelope test_support_keeps_blob_surface -- --nocapture`
  `cargo test -p z00z_wallets --release test_build_rejects_bad_fee -- --nocapture`
  `cargo test -p z00z_validators --release validator_rejects_malformed_fee_envelope_contract -- --nocapture`
- Broad workspace release evidence was already green on the same live code tree:
  `cargo test --release`

## Result

`060-13` is complete. Phase 060 now advances to `060-14-PLAN.md` for the
refund/source and incomplete-runtime reopen packet, while future full
`z00z-verification-orchestrator` reruns remain operator-owned manual work.
