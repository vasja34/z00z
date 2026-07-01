# 040-05 Summary

## ⚠️ Scope

This summary records the then-current status of `040-05-PLAN.md`, covering the
`040-09 Proof-Theorem Preservation` slice and the adjacent public-input and
digest-root integrity evidence used to keep the Phase 040 spend-proof
contract honest before the later theorem-backend closure work in `040-07` and
`040-08`.

## ⚠️ Outcome

The spend-proof path now binds each carried proof input to the exact
`tx.inputs` state key, rejects mismatched proof-bundle pairing before
signing or verification, and makes the versioned `input_asset_id_hex`
contract explicit.

`040-09` is still open, however. The carried proof wire is versioned and
non-empty, but the current `SpendProofBackend` still authenticates only a
statement-bound envelope derived from the canonical statement bytes plus a
public hash, with an empty witness surface, instead of a real
theorem-carrying spend proof. That means the validator-facing verifier
remains narrower than the full `verify_spend_rules(...)` theorem.

## ✅ Repository Changes

- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
  - added `SpendInputProofWire::input_asset_id_hex`
  - bumped `SPEND_PROOF_WIRE_VER` to `2`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
  - canonicalizes and verifies the new input-asset binding
  - rejects mismatched `tx.inputs` / `proof.inputs` pairings with `InputRefMismatch`
  - keeps malformed input asset keys on the verifier side classified as `InvalidHex`
- `crates/z00z_wallets/tests/test_spend_prover_contract.rs`
  - added regression coverage for forged receiver-bound proof inputs and mismatched input-asset binding
- `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
  - added regression coverage for mismatched input-asset binding on the public verifier
- `crates/z00z_wallets/tests/test_spend_statement.rs`
  - kept statement drift coverage aligned with the canonical proof carrier after the new binding change
- `.planning/phases/040-spend-proof/040-INTEGRITY-GATES.md`
  - updated the theorem, public-input, and digest-root ledger entries to match the current code state without overstating theorem closure

## ✅ Validation

Focused validation passed after the fix and the wire-version bump:

- `cargo test -p z00z_wallets --release --features test-fast --test test_spend_statement -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast --test test_spend_prover_contract -- --nocapture`
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_proof_verifier -- --nocapture`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

A broader release gate was also run with the requested flags:

- `cargo test --release --features test-fast --features wallet_debug_dump`

That broader gate passed for the workspace state captured at the time this
summary was written.

## ⚠️ Review Loop

The slice review found one real issue before the fix: the public builder
accepted caller-supplied proof inputs without an explicit binding to the
transaction input state key. That was fixed by adding `input_asset_id_hex`
to the proof wire and checking it on both producer and verifier paths.

The latest review loop then reopened `040-09`: although the pairing gap is
closed, the accepted-path verifier still authenticates a statement-bound
backend artifact with no witness-proof relation instead of a real
theorem-carrying spend proof.

## ⚠️ Current Boundary

At the time of this summary, `040-10` and `040-11` validated green: the public-input surface is
explicit and fail closed, and the package digest remains the only
authoritative public root. At that same point, `040-09` stayed open until the
spend-proof backend stopped accepting the current statement-bound hash artifact
as sufficient proof or the plan was explicitly re-scoped to keep that narrower
boundary as the intended end state.

Later Phase 040 execution attempted to close `040-09` with a theorem-branded
backend, but the 2026-04-27 repository-backed audit reopened that claim. Use
the current Phase 040 ledgers, not this historical summary alone, for live repository truth.
