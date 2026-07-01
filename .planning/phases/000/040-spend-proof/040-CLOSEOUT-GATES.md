# 040 Closeout Gates

## Internal Relation Closeout Reset

`040-09-SUMMARY.md` remains the historical implementation baseline, but this
ledger now tracks only the internal `040-10` closeout gates. Current-authority
rows below must describe the required internal theorem-relation end state and
must not claim public/trustless proof closure.

## Purpose

This ledger records the Phase 040 closeout boundary for one canonical internal
theorem-relation path. It validates wallet and simulator proof-generation
composition and forbids any second regular-tx proof lane, while checkpoint
theorem finality and rollup settlement proof closure remain open.

## 040-12 Checkpoint-Pipeline Reuse

| Claim | Evidence | Verdict |
| --- | --- | --- |
| `build_cp_draft(...)` still reuses the typed `prev_root` checkpoint pipeline together with `TxProofVerifier` and `SpentIndex`. | `state_update.rs`, `tx_verifier.rs` | Closed |
| `verify_full_tx_package(...)` remains the package-admission seam before checkpoint draft construction. | `tx_verifier.rs`, `bundle_lane_impl.rs` | Closed |
| `CheckpointPackageProofVerifier` remains an adapter around the package-coupled flow and does not become a standalone checkpoint backend. | `bundle_lane_impl.rs` | Closed |
| The accepted Stage-4 spend proof `prev_root_hex` is now bound to the Stage-11 exec root before checkpoint apply. | `stage_11_apply.rs`, `bundle_lane_impl.rs`, `test_checkpoint_acceptance.rs` | Closed |
| Checkpoint apply remains the second explicit verification seam and stays covered by checkpoint acceptance regressions. | `test_checkpoint_acceptance.rs` | Closed |
| Package admission can succeed while checkpoint apply still rejects a tampered exec row, so the checkpoint seam is not collapsed into the package seam. | `test_checkpoint_acceptance.rs` | Closed |

## 040-13 Missing-Code Closure Matrix

| Missing-code item | Outcome | Evidence |
| --- | --- | --- |
| Non-empty `TxProofWire` | Closed by `040-01`; the proof wire is versioned and explicit. | `040-TODO.md`, `040-INTEGRITY-GATES.md` |
| Concrete regular-tx prover | Closed for the internal canonical theorem carrier and suite with membership witnesses passed into the backend proof-generation path. | `tx_lane_runtime_flow.rs`, `witness_gate.rs`, `spend_proof_backend.rs`, `040-INTEGRITY-GATES.md` |
| Concrete regular-tx verifier | Still public-artifact bounded: it checks canonical deterministic artifact shape and statement binding, not a trustless proof of witness knowledge. | `spend_verification.rs`, `tx_verifier.rs`, `040-INTEGRITY-GATES.md` |
| Separate regular-tx proof layer outside the checkpoint artifact path | Explicitly out of scope; proof consumption closes through the existing checkpoint hooks in `040-04` and `040-06`. | `state_update.rs`, `bundle_lane_impl.rs`, `040-TODO.md` |
| Unified output constructor | Bounded follow-up only in `040-08`; not promoted into this closeout. | `040-TODO.md`, `040-06-PLAN.md` |

The active closeout gate remains bounded to the internal canonical theorem package path.

## 040-14 Prohibited Shortcut Checklist

| Shortcut | Re-check result | Evidence |
| --- | --- | --- |
| STARK proof support | Still prohibited: no second theorem backend, STARK lane, or parallel checkpoint proof system may be introduced. | `040-CONTEXT.md`, `040-INTEGRITY-GATES.md`, `tx_wire_types.rs`, `spend_proof_backend.rs`, `test_scenario1_stage_surface.rs` |
| `receiver_cards` in the regular persisted package | Not introduced; the live tx package carries compact receiver authorization material instead of a `receiver_cards` surface. | `040-CONTEXT.md`, `tx_wire_types.rs`, `spend_verification.rs`, `test_scenario1_stage_surface.rs` |
| Separate `C_fee` contract | Not introduced; fee-as-output semantics remain the live contract on both the verifier path and the persisted tx package surface. | `040-CONTEXT.md`, `040-Spend-Proof-Spec.md`, `tx_verifier.rs`, `test_scenario1_stage_surface.rs` |
| Mixed `compute_leaf_ad()` / `derive_leaf_ad()` runtime path | Not introduced; the wallet-local and crypto-boundary formulas stay separate and migration drift remains explicitly test-covered. | `040-CONTEXT.md`, `tag.rs`, `test_phase11_derivation.rs` |

## Phase 040 Completion Gate Re-check

| Gate clause | Result | Evidence |
| --- | --- | --- |
| `040-01` through `040-07` complete | Pass | `040-TODO.md`, `040-CONTEXT.md` |
| `040-10` retires alternate suite names and lands the canonical internal theorem relation across wallet and simulator seams | Pass for internal relation | `040-Spend-Proof-Spec.md`, `040-INTEGRITY-GATES.md`, `spend_proof_backend.rs`, `test_spend_proof_backend.rs`, `test_tx_proof_verifier.rs`, `test_spend_witness_gate.rs` |
| N11, N12, N13, and N14 are closed or explicitly reclassified on the live public-verifier seam | Pass | `040-TEST-SPEC.md`, `040-VALIDATION.md`, `test_tx_proof_verifier.rs` |
| All mandatory tests listed above exist and are green | Pass | `040-TODO.md`, `040-CONTEXT.md`, `test_tx_proof_verifier.rs`, `test_checkpoint_acceptance.rs`, `test_scenario1_tx_proof_roundtrip.rs`, `test_scenario1_stage_surface.rs` |
| `040-Spend-Proof-Spec.md`, `040-TODO.md`, and context still aligned | Pass | `040-Spend-Proof-Spec.md`, `040-TODO.md`, `040-CONTEXT.md`, `040-CLOSEOUT-GATES.md` |
| No remaining implementation step depends on the retired superseded draft | Pass | `040-TODO.md`, `040-CONTEXT.md` |
| Shortcut checklist re-run before marking `040-07` complete | Pass | `040-TODO.md`, `test_scenario1_stage_surface.rs`, `040-CLOSEOUT-GATES.md` |
| `040-09-SUMMARY.md` remains archive-only baseline evidence and does not define the active closeout target | Pass | `040-09-SUMMARY.md`, `040-10-PLAN.md`, `040-CLOSEOUT-GATES.md` |
| Public/trustless proof-of-knowledge, checkpoint theorem finality, and rollup proof closure are kept out of completed language | Required open boundary | `040-10-PLAN.md`, `040-Spend-Proof-Spec.md`, `040-UAT.md` |

## Review Notes

- `040-CONTEXT.md` already carries the prohibited shortcut mirror.
- `test_scenario1_stage_surface.rs` keeps the stage wording bounded to the
  package-coupled checkpoint path and rejects overclaiming about standalone
  checkpoint authority.
- `040-13` closes the legacy missing-code checklist according to the landed
  Phase 040 task owners, while active closeout authority now stays anchored on
  the final canonical theorem requirements recorded in
  `040-INTEGRITY-GATES.md`.
- The internal closeout boundary is one canonical theorem-relation path only.
- Membership against `prev_root` is closed inside the backend witness path;
  public/trustless proof-of-knowledge and rollup settlement verification remain
  open before any broader ledger can be marked complete.
