# TASK016 - Continue Phase 040 Internal Theorem Relation

**Status:** Completed  
**Added:** 2026-04-28  
**Updated:** 2026-04-29

## Original Request

Continue Phase 040 through `040-10-PLAN.md`, expand the scope to connect the
existing proof components into one internal theorem relation, pass membership
witnesses into the theorem witness, verify every input against `prev_root`
inside the canonical theorem path, close the statement-only bypass, add tamper
tests, and keep wording scoped to internal theorem-relation closure.

## Thought Process

The key correction is that the current repository work can close an internal
wallet and simulator theorem relation, but it cannot honestly claim public or
trustless theorem closure. The live verifier still checks a deterministic
canonical artifact and statement binding rather than a public cryptographic
proof of witness knowledge. The implementation therefore had to lift state
membership into `SpendProofWitness` and `CanonicalSpendProofBackend` while
keeping public proof-of-knowledge and checkpoint theorem finality open in
planning and validation language. The follow-up rollup guard closes the
output-proof-only admission blind spot by binding the wallet public spend
theorem contract to checkpoint artifact, link, and execution-input evidence,
but it is still not a public proof-of-knowledge backend.

## Implementation Plan

- Remove the temporary `tari_utilities_bridge` dependency path and rely on the
  direct vendored Tari utilities path through `z00z_crypto`
- Freeze the live spend suite on `regular_spend_theorem_bpplus` with
  `CanonicalSpendProofBackend`
- Add explicit membership witnesses to the spend witness and validate them
  against `prev_root` inside the backend relation
- Close statement-only, wrong-root, membership, nullifier, balance, and range
  relation bypasses with focused tests
- Wire Scenario 1 Stage 4 runtime proof generation through prep-derived
  membership witnesses
- Keep Phase 040 docs, UAT, validation, and stage-surface guards aligned to
  internal theorem-relation closure without public proof overclaim
- Add a focused rollup settlement guard that verifies public tx theorem
  contract evidence against checkpoint artifact, link, and exec-input
  inclusion

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

| ID | Description | Status | Updated | Notes |
| ---- | ------------- | -------- | --------- | ------- |
| 16.1 | Remove bridge dependency and freeze canonical suite | Complete | 2026-04-28 | `tari_utilities_bridge` is deleted in the dirty tree; live suite is `regular_spend_theorem_bpplus` |
| 16.2 | Lift membership into backend witness relation | Complete | 2026-04-28 | `SpendProofWitness` carries membership witnesses and backend validates against `prev_root` |
| 16.3 | Add focused tamper coverage | Complete | 2026-04-28 | Backend, public verifier, tx tamper, wrong-root, and witness-gate tests passed |
| 16.4 | Wire simulator runtime membership path | Complete | 2026-04-28 | Scenario 1 Stage 4 uses prep-derived membership witnesses and simulator release suite passed |
| 16.5 | Align planning and memory truth | Complete | 2026-04-28 | Memory bank, validation, state, and roadmap now record the green full workspace verify gate |
| 16.6 | Preserve broader public theorem follow-up | Complete | 2026-04-28 | Public/trustless proof-of-knowledge, checkpoint theorem finality, and rollup settlement remain explicit follow-up scope, not completed claims |
| 16.7 | Add focused rollup public-artifact binding guard | Complete | 2026-04-28 | `verify_settlement_theorem` binds wallet public spend theorem contract evidence to checkpoint artifact, link, exec ID, spend and checkpoint roots, and tx inclusion |
| 16.8 | Complete review loop | Complete | 2026-04-28 | Three independent review passes reported no significant issues after the current-authority legacy phrase gate |
| 16.9 | Harden direct backend verification | Complete | 2026-04-29 | `verify()` now checks public relation drift before deterministic artifact acceptance; output theorem leaves use the `leaf_ad_id` namespace |

## Progress Log

### 2026-04-28 Internal Relation

- Removed the temporary `tari_utilities_bridge` surface from the workspace dirty
  tree and kept the direct vendored Tari utilities dependency path.
- Updated the wallet proof path so `SpendProofWitness` carries explicit
  membership witnesses and `CanonicalSpendProofBackend` checks statement shape,
  per-input membership against `prev_root`, nullifier/order/balance rules, and
  output range relation before artifact generation.
- Added and validated tamper tests for statement-only construction, `prev_root`,
  membership proof, membership path, membership leaf, witness material,
  nullifier, balance, and range relation.
- Updated Scenario 1 Stage 4 runtime flow to derive membership witnesses from
  the prep file and use the membership-aware witness gate.
- Narrowed Phase 040 docs and stage-surface guard language to internal
  theorem-relation closure and kept public proof-of-knowledge, checkpoint
  theorem finality, and rollup settlement closure open.
- Validation passed for `cargo fmt`, focused wallet release tests, Scenario 1
  release execution, focused stage-surface guard, and
  `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools`.
- Fixed the full-workspace blocker in
  `core::tx::witness_gate::tests::test_gate_typed_root` by making the unit
  test build the matching membership root before calling the membership-aware
  witness gate.
- Validation then passed for the targeted wallet lib test, the focused
  `test_spend_witness_gate` integration file, and the canonical
  `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` gate with
  exit code 0.

### 2026-04-28 Rollup Guard

- Added `z00z_rollup_node::verify_settlement_theorem` as a focused settlement
  guard over public artifacts: tx package structure and digest, wallet public
  spend theorem contract, checkpoint proof payload, checkpoint link, execution
  input ID, spend-root-to-checkpoint-root alignment, and tx row inclusion.
- Added `crates/z00z_rollup_node/tests/test_settlement_theorem.rs` with a
  canonical acceptance case plus checkpoint replay, missing tx, root mismatch,
  bad link, and bad package rejection cases.
- Corrected the test fixture so output state keys do not overlap consumed
  input state keys and so the negative inclusion case really changes the
  checkpoint execution input.
- Validation passed for `cargo fmt`,
  `cargo test -p z00z_rollup_node --features test-params-fast --all-targets`,
  `cargo clippy -p z00z_rollup_node --features test-params-fast --all-targets -- -D warnings`,
  `cargo test -p z00z_wallets --features test-params-fast --test test_spend_proof_backend -- --nocapture`,
  and `cargo test -p z00z_wallets --features test-params-fast --test test_tx_proof_verifier -- --nocapture`.

### 2026-04-28 Review Loop

- Current-authority legacy phrase grep over the active Phase 040 authority files
  returned no live matches; historical 040-07/040-08/040-09 artifacts still
  retain archived old-boundary language by design.
- Three independent `/GSD-Review-Tasks-Execution`-style review passes returned
  PASS with no significant findings.
- The review loop confirmed that `verify_settlement_theorem` remains a focused
  public-artifact binding guard and does not claim public/trustless
  proof-of-knowledge, checkpoint theorem finality, or full rollup settlement
  closure.

### 2026-04-29 Direct Verifier Hardening

- Hardened `CanonicalSpendProofBackend::verify()` so forged deterministic
  artifacts cannot bypass public relation checks for output range proofs,
  duplicate public inputs, input/output theorem leaf overlap, or balance drift.
- Fixed the theorem statement projection so output `AssetLeaf.asset_id` inside
  `SpendProofStmt` uses the output `leaf_ad_id` namespace, matching input
  theorem leaf comparisons while keeping storage/package asset IDs bound by the
  canonical statement bytes.
- Added `test_backend_rejects_forged_overlap` and kept the forged range and
  forged balance rejection coverage green.
- Updated `040-UAT.md`, `040-10-PLAN.md`, and `040-TODO.md` to keep the active
  authority chain scoped to internal theorem-relation readiness and explicit
  open boundaries.
- Validation passed for focused wallet backend and public verifier tests, the
  simulator stage-surface guard, the bootstrap gate, and
  `cargo test --release --features test-params-fast --features wallet_debug_tools`.
