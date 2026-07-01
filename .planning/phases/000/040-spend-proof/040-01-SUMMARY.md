# 040-01 Summary

## ✅ Scope

This summary records the completion state for `040-01-PLAN.md`, covering the
landed `040-01 Proof Carrier Contract` and `040-02 Canonical Spend Statement`
slice of Phase 040.

## ✅ Outcome

Plan 01 is closed for the spend-proof carrier and canonical-statement
foundation.

The regular spend lane now carries one explicit versioned proof/auth carrier in
`TxProofWire` and `TxAuthWire`, while the canonical statement contract remains
centralized on the live `encode_spend_statement(...)` /
`build_public_spend_contract(...)` /
`verify_tx_public_spend_contract(...)` seam. The package digest remains the
only public proof-binding root, and bare wire-digest binding is rejected.

## ✅ Repository Changes

- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` now enforces the stronger
  spend-proof carrier contract, including canonical input-key handling for the
  regular spend lane.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs` now owns the shared
  canonical regular-spend statement builder used by both producer and verifier
  paths.
- `crates/z00z_wallets/src/core/tx/tx_digest.rs` now clears auth-only and
  excluded spend blobs before canonical digest recomputation so malformed or
  non-canonical excluded fields cannot perturb the package digest.
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs` now rejects non-canonical
  input asset-id keys through the same shared decoding discipline used by the
  spend statement and package digest.
- `crates/z00z_wallets/tests/test_spend_proof_wire.rs` now locks the carrier
  roundtrip, unknown-version rejection, and stronger-mode empty-placeholder
  rejection rules.
- `crates/z00z_wallets/tests/test_spend_statement.rs` now locks same-package
  stability, digest drift sensitivity, output drift sensitivity, canonical
  input binding, and rejection of bare wire digest as the only public root.
- `crates/z00z_wallets/tests/test_tx_proof_verifier.rs` and
  `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs` now cover the
  tightened canonical input and public-contract boundaries that protect the
  landed Plan 01 slice.
- `.planning/phases/040-spend-proof/040-TODO.md`,
  `.planning/phases/040-spend-proof/040-CONTEXT.md`, and
  `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md` were synchronized
  to the landed Plan 01 truth so downstream plans inherit the correct carrier,
  statement, and digest-root contract.

## ✅ Validation

- Focused wallet validation passed:
  - `cargo test -p z00z_wallets --release --features test-fast test_spend_proof_wire -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast test_spend_statement -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast test_tx_verifier_suite -- --nocapture`
- Mandatory bootstrap gate passed:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Full release validation passed:
  - `cargo test --release --features test-fast --features wallet_debug_dump`

## ✅ Review Loop

The review loop followed the GSD plan-closeout gate and stayed narrow until the
closeout evidence was clean.

1. `/GSD-Review-Tasks-Execution` was treated as the mandatory closeout review
  prompt for the landed `040-01`/`040-02` slice, run in YOLO mode across the
  plan evidence rather than as an optional postscript.
2. The review sequence was kept at or above the GSD floor of three YOLO review
  passes, and any material issue found on an early pass was fixed directly in
  the carrier, statement, digest, or verifier seam before another pass.
3. Plan 01 was treated as closed only after the final review sequence produced
  at least two consecutive passes with no significant issues on the same
  landed slice.

## ✅ Current Boundary

This summary closes only the first Phase 040 execution plan. The next active
slice is `040-02-PLAN.md`, which must build on this landed foundation to close
the concrete producer path (`040-03`) and the concrete verifier path
(`040-04`) without introducing a parallel proof architecture.

<!-- End of 040-01 Summary -->