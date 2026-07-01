# 040-09 Report

## 🎯 Scope

This report audits the material factual claims in `040-09-PLAN.md` against the
current repository-backed implementation and its owning tests.

Scope rules used here:

- Code and executable tests take precedence over planning language.
- Phase ledgers are treated as secondary repository evidence, not as a
  substitute for code or tests.
- Pure execution instructions are omitted unless they assert a present-tense
  fact.
- Absence claims are marked conservatively when the audited surface is too
  narrow to prove them strongly.
- Crypto-surface claims in this report are anchored directly to
  `crates/z00z_crypto/src/*`.

## 🔍 Method

The audit cross-checked the plan against these implementation and test owners:

- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
- `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`
- `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
- `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
- `crates/z00z_wallets/tests/support/test_phase040_spend_proof_support.rs`
- `crates/z00z_simulator/src/scenario_1/stage_11.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_crypto/src/backend/backend_tari.rs`
- `crates/z00z_crypto/src/backend/backend_range_proofs.rs`
- `crates/z00z_crypto/src/lib.rs`
- `.planning/phases/040-spend-proof/040-TEST-SPEC.md`
- `.planning/phases/040-spend-proof/040-VALIDATION.md`

Fresh reruns were also executed during the audit, but the table below uses
repository-backed anchors as the primary evidence surface.

For the crypto-capability rows, the approved surface was read directly from the
`z00z_crypto` source tree above.

## ✅ Executive Verdict

`040-09-PLAN.md` is materially honest only on the Path C statement-bound
boundary.

The repository backs all of the following:

- the live suite is still `regular_spend_statement_bound_v1`,
- the default backend is still `StatementBoundSpendProofBackend`,
- theorem-v2 artifacts are rejected on the shipped backend and public-verifier
  seams,
- N11/N12/N13/N14 are covered or honestly reclassified on the live verifier
  seam,
- Stage 11 remains package-coupled and is not a standalone checkpoint theorem
  backend.

The repository does not back any claim that the approved Tari-backed surface
already provides a live spend-theorem backend on the current seam.

## 📏 Verdict Legend

- `SUPPORTED`: backed by the audited code/test surface.
- `SUPPORTED WITH QUALIFICATION`: directionally correct, but the evidence is
  narrower than the wording.
- `LEDGER-BACKED ONLY`: backed by phase ledgers, not by a stronger live-code
  fact.
- `NOT REPO-BACKED AS A LIVE FACT`: the reviewed repository surface does not
  justify the claim as written.
- `NEEDS BROADER SCAN`: the in-scope audit surface is too narrow to prove the
  claim strongly.

## 📋 Claim-By-Claim Table

| Claim | Source anchor | Test anchor | Verdict |
| --- | --- | --- | --- |
| Repository-Proven Starting Point #1: the live suite is still `regular_spend_statement_bound_v1`. | `crates/z00z_wallets/src/core/tx/tx_wire_types.rs:16` | `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:137`; `crates/z00z_wallets/tests/test_spend_proof_backend.rs:91` | `SUPPORTED` |
| Repository-Proven Starting Point #2: `default_spend_proof_backend()` still returns `StatementBoundSpendProofBackend`. | `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs:169-172` | `crates/z00z_wallets/tests/test_spend_proof_backend.rs:91` | `SUPPORTED` |
| Repository-Proven Starting Point #3: backend and public-verifier tests accept statement-bound artifacts on the live seam, while theorem-v2 stays unsupported. | `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs:231-232`; `crates/z00z_wallets/src/core/tx/spend_verification.rs:188`; `crates/z00z_wallets/src/core/tx/spend_verification.rs:727` | `crates/z00z_wallets/tests/test_spend_proof_backend.rs:91`; `crates/z00z_wallets/tests/test_spend_proof_backend.rs:105`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:137`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:159` | `SUPPORTED` |
| Repository-Proven Starting Point #4: the approved Tari-backed Bulletproofs+ facade is repository-backed for range proofs, not for a general spend-theorem backend. | `crates/z00z_crypto/src/backend/backend_tari.rs:119`; `crates/z00z_crypto/src/backend/backend_tari.rs:155`; `crates/z00z_crypto/src/backend/backend_tari.rs:200`; `crates/z00z_crypto/src/backend/backend_range_proofs.rs:40`; `crates/z00z_crypto/src/backend/backend_range_proofs.rs:83`; `crates/z00z_crypto/src/lib.rs:245-246` | `crates/z00z_wallets/tests/test_spend_proof_backend.rs:105`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:159` | `SUPPORTED WITH QUALIFICATION` |
| Repository-Proven Starting Point #5: Stage 11 remains the existing package-coupled acceptance seam and must not be described as standalone backend authority. | `crates/z00z_simulator/src/scenario_1/stage_11.rs:8-11` | `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:744`; `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:768`; `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:827`; `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:836-851` | `SUPPORTED` |
| Repository-Proven Starting Point #6: the ledgers now describe the honest statement-bound boundary and record N11/N12/N13/N14 as closed or explicitly reclassified on the live seam. | `.planning/phases/040-spend-proof/040-VALIDATION.md:39-43`; `.planning/phases/040-spend-proof/040-VALIDATION.md:62-64` | `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:200`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:223`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:246`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:280`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:322` | `LEDGER-BACKED ONLY` |
| Capability Freeze Outcome reason #1 and #2: the approved surface does not expose a repository-backed general spend-theorem verifier, and the live backend still rejects theorem-v2 as unsupported. | `crates/z00z_crypto/src/backend/backend_tari.rs:119`; `crates/z00z_crypto/src/backend/backend_tari.rs:155`; `crates/z00z_crypto/src/backend/backend_tari.rs:200`; `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs:231-232` | `crates/z00z_wallets/tests/test_spend_proof_backend.rs:105`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:159` | `SUPPORTED` |
| Capability Freeze Outcome reason #3: reference-only input membership still stays on the `prev_root` checkpoint/pre-state seam. | `crates/z00z_wallets/src/core/tx/tx_wire_types.rs:66`; `crates/z00z_wallets/src/core/tx/tx_wire_types.rs:97` | No dedicated direct test anchor was identified in the audited seam. | `SUPPORTED WITH QUALIFICATION` |
| Capability Freeze Outcome frozen result: keep `regular_spend_theorem_bpplus_v1` as a future target identifier rather than a live suite fact. | `crates/z00z_wallets/src/core/tx/tx_wire_types.rs:16`; `crates/z00z_wallets/tests/support/test_phase040_spend_proof_support.rs:224`; `crates/z00z_wallets/tests/support/test_phase040_spend_proof_support.rs:232`; `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs:231-232` | `crates/z00z_wallets/tests/test_spend_proof_backend.rs:105`; `crates/z00z_wallets/tests/test_spend_proof_backend.rs:118`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:159` | `SUPPORTED` |
| Live Suite Outcome: `040-09-02` closes as an honest reopen rather than a theorem-backend upgrade. | `.planning/phases/040-spend-proof/040-VALIDATION.md:39`; `.planning/phases/040-spend-proof/040-VALIDATION.md:48`; `crates/z00z_wallets/src/core/tx/tx_wire_types.rs:16`; `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs:169-172` | `crates/z00z_wallets/tests/test_spend_proof_backend.rs:91`; `crates/z00z_wallets/tests/test_spend_proof_backend.rs:105`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:137`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:159` | `SUPPORTED WITH QUALIFICATION` |
| Live Suite Outcome evidence #3 and #4: the mandatory broad gate is green and the last broad-gate-only regression was unrelated to the theorem boundary. | No stable implementation anchor in the audited source slice. The repository stores this mainly as validation language, not as a durable code fact. | No durable test-file anchor proves the historical regression narrative by itself. | `NOT REPO-BACKED AS A LIVE FACT` |
| Public-Verifier Matrix Outcome / N11: missing output `leaf_ad_id`, `r_pub`, and `owner_tag` reject through `SpendPublicErr::MissingOutputField`. | `crates/z00z_wallets/src/core/tx/spend_verification.rs:373-381`; `crates/z00z_wallets/src/core/tx/spend_verification.rs:808-824` | `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:179-194`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:200`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:223-240` | `SUPPORTED` |
| Public-Verifier Matrix Outcome / N12: output `leaf_ad_id` drift is honestly reclassified to `SpendPublicErr::StatementMismatch` on the current statement-bound seam, not to a standalone `BadOutputLeafAd` branch. | `crates/z00z_wallets/src/core/tx/spend_verification.rs:894`; `.planning/phases/040-spend-proof/040-TEST-SPEC.md:233` | `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:246` | `SUPPORTED` |
| Public-Verifier Matrix Outcome / N13: invalid output range-proof bytes reject fail closed as `BadRangeProof`. | `crates/z00z_wallets/src/core/tx/spend_verification.rs:839-859` | `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:280` | `SUPPORTED` |
| Public-Verifier Matrix Outcome / N14: balance mismatch rejects as `BadBalance`. | `crates/z00z_wallets/src/core/tx/spend_verification.rs:874`; `crates/z00z_wallets/src/core/tx/spend_verification.rs:886` | `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:322` | `SUPPORTED` |
| Remaining Gap Inventory / Gap E: checkpoint/state-transition theorem work remains residual future work; Phase 040 only reuses the existing checkpoint pipeline. | `crates/z00z_simulator/src/scenario_1/stage_11.rs:8-11` | `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:827`; `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:836-851` | `SUPPORTED` |
| Remaining Gap Inventory / Gap F: Phase 040 does not close rollup settlement or full transition proving. | No positive implementation anchor was identified in the audited spend-proof seam. | No in-scope test anchor proves rollup settlement closure. | `NEEDS BROADER SCAN` |
| Decision Rule and Final Verdict Rule: there is no honest theorem-closure path unless repository-backed proving capability exists on the approved seam. | `crates/z00z_crypto/src/backend/backend_tari.rs:119`; `crates/z00z_crypto/src/backend/backend_tari.rs:155`; `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs:231-232` | `crates/z00z_wallets/tests/test_spend_proof_backend.rs:105`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:159` | `SUPPORTED` |
| Required End State #2: the truthful naming contract is “live statement-bound suite” plus “versioned theorem target identifier,” not a mixed live theorem claim. | `crates/z00z_wallets/src/core/tx/tx_wire_types.rs:16`; `crates/z00z_wallets/tests/support/test_phase040_spend_proof_support.rs:232` | `crates/z00z_wallets/tests/test_spend_proof_backend.rs:105`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:159` | `SUPPORTED` |
| Path C Honest Partial and `040-CG` closeout wording: the repository-backed bounded closeout is “honest statement-bound boundary plus explicit residual handoff,” not “live theorem backend closed.” | `.planning/phases/040-spend-proof/040-VALIDATION.md:43`; `.planning/phases/040-spend-proof/040-VALIDATION.md:62-64` | `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:200`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:246`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:280`; `crates/z00z_wallets/tests/test_tx_proof_verifier.rs:322`; `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:744`; `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:827` | `LEDGER-BACKED ONLY` |

## ⚠️ Findings That Matter Most

1. The plan is reliable when it says the live seam is still statement-bound.
2. The plan is reliable when it says theorem-v2 is still future-target /
   unsupported on the shipped seam.
3. The N11/N12/N13/N14 closure is repository-backed on the live verifier seam.
4. Stage 11 is still bounded to package-coupled continuity and must not be
   described as standalone checkpoint theorem authority.
5. Claims about a broad green gate or the exact root cause of a transient
   regression are not strong repository facts unless they are re-backed by a
   durable validation artifact or a fresh rerun log.
6. The audited spend-proof seam does not itself prove or disprove system-wide
   rollup settlement proving; that requires a broader architectural scan.

## 🧾 Bottom Line

The strongest repository-backed conclusion is simple:

- Phase 040 `040-09` is honest only as a bounded Path C statement-bound
  continuation.
- The current repository does not back a live theorem backend on the approved
  seam.
- Any wording stronger than that should be treated as unsupported until new
  code and tests land.
