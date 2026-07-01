# Phase Entrance Exam

**Phase:** `032-crypto-audit-scenario-1`
**Generated:** `2026-04-06`
**Scope Sources:** `032-CONTEXT.md`, `032-TODO.md`, `032-TEST-SPEC.md`, `032-VALIDATION.md`, `032-VERIFICATION.md`, `032-HONEST-CLOSEOUT.md`, `032-FULL-AUDIT.md`, `.planning/REQUIREMENTS.md`, and live code or test evidence across `z00z_crypto`, `z00z_storage`, `z00z_wallets`, and `z00z_simulator`

## MUST

1. Every final answer in this document MUST be independently re-checked through
   the `doublecheck` skill before it is accepted as final.
2. Every answer MUST be a repository-backed proof system, using factual,
   mathematical, cryptographic, and logical proof where applicable.
3. If a proof cannot be closed, the answer MUST state exactly what evidence,
   artifact, mathematical argument, cryptographic assumption, or repository
   behavior is missing.
4. Every answer MUST stay tied to the live codebase, tests, logs, manifests,
   and phase artifacts for this repository.
5. Every answer in this document MUST function as a verification exam of the
   correct implementation of this phase, not as freeform commentary.
6. If answering a question reveals a real bug, gap, or overclaim, the answer
   MUST name it explicitly and state the remediation path.
7. This file is generated as a question sheet. The `Ans:` sections MUST remain
   blank until a later agent or model fills them.

## 🎯 Challenge

Pressure-test whether Scenario 1 is now described with cryptographic honesty.
The solver must separate what the current stack really proves from what it only
enforces locally, what it only verifies through compatibility-shaped handoffs,
and what still remains open despite targeted green evidence.

## ⛔ Constraints

- Every question must be answered from repository evidence, not from summary
  prose alone.
- The exam must distinguish delivered current-stack truth from broader
  trustless or future-backend claims.
- The solver must surface contradictions between code, tests, manifests, and
  closeout language instead of normalizing them.
- A green targeted rerun is not enough by itself; answers must also account for
  boundary scope, replay continuity, and documentation honesty.

## Scope Note

This exam verifies the accepted `Alice -> claim package -> publish -> Bob scan
-> spend -> checkpoint` flow as implemented today. It is designed to expose
whether the phase really closed claim authenticity, spend/checkpoint truth,
semantic freeze, replay safety, and secret hygiene at the boundaries it now
claims to defend, while also forcing the solver to identify the seams that are
still only partial.

## 🔍 Answering Standard

- Answers must discover their own evidence path through the repository.
- Questions are intentionally phrased at the level of guarantees, boundaries,
  drift, replay, and overclaim rather than file-by-file breadcrumbs.
- A correct answer may conclude that a claim is only partially true, remains
  open, or is overstated, provided that conclusion is proved from the
  repository.

## 🎯 Theme 1: Closure And Scope Honesty

### 1. Delivered Closure Versus Open Closure

🔴 **Quest:** Which original Scenario 1 security promises are genuinely closed by the live implementation and executed evidence, and which ones remain intentionally outside delivered closure even though several targeted release-style tests are green?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository closes a narrower but real set of Phase 032 Scenario 1 promises. Runtime and security closure is supported for authority binding of the full claim tuple (`PH32-CLAIM-BIND`), checkpoint rejection of placeholder success lanes (`PH32-CHECKPOINT`), and default secret-export hardening (`PH32-SECRET`). Supporting semantic and documentation-honesty closure is also supported for the frozen Scenario 1 meaning of `leaf_ad_id`, `s_out`, request-versus-card trust language (`PH32-SEM`) and for explicit anti-overclaim status language (`PH32-HONEST`). It does not close the broader original claim-trust or spend-verifier promises: `PH32-CLAIM-TRUST` remains open because claim packages still rely on a synthetic one-item helper contract instead of persisted storage-backed continuity, and `PH32-SPEND` remains open because the regular public spend statement still does not carry the original nullifier-semantics portion of the requirement. The green targeted release-style tests prove that the delivered current-stack boundaries work; they do not upgrade those two broader requirements into full closure.

**Evidence Trail:**

1. `.planning/REQUIREMENTS.md` marks `PH32-SEM`, `PH32-CLAIM-BIND`, `PH32-CHECKPOINT`, `PH32-SECRET`, and `PH32-HONEST` complete, while `PH32-CLAIM-TRUST` and `PH32-SPEND` remain open.
2. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` states what Scenario 1 now proves and what it does not prove, including the still-open broader `PH32-SPEND` and `PH32-CLAIM-TRUST` wording.
3. `.planning/phases/032-crypto-audit-scenario-1/032-VALIDATION.md` and `032-VERIFICATION.md` say the targeted release-style matrix is review-backed, but the broader original requirements stay open and the broad workspace release suite is not clean closeout evidence.
4. `crates/z00z_crypto/src/claim/v2.rs` shows the live claim statement includes the bound tuple fields, including `claim_source_asset_id`, `claim_source_commitment`, `source_root`, `chain_id`, `claim_scope_hash`, `recipient_binding`, and `nullifier`.
5. `crates/z00z_storage/src/assets/store_internal/store_query.rs` shows `AssetStore::claim_source_contract_for_item(...)` still builds a fresh off-store helper store, which is why persisted storage-backed continuity is not closed.
6. `crates/z00z_wallets/src/core/tx/spend_verification.rs` implements a real current-stack public spend contract, while the phase validation and closeout artifacts still record that nullifier semantics are absent from the regular public spend statement.
7. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` rechecks the package via `verify_tx_public_spend_contract(...)`, confirming a current-stack checkpoint gate rather than a final proof-backend claim.
8. `crates/z00z_simulator/tests/test_stage2_secret_artifacts.rs` and `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` support the closed `PH32-SECRET` lane and the simulator-only bounded mock-RNG caveat inside the current honest scope.
9. `.planning/phases/032-crypto-audit-scenario-1/.logs/032-test-spec-rerun-20260405T182809Z/manifest.current.txt` records `RESULT[17]=PASS` for the targeted matrix and `RESULT[18]=FAIL` for the broad workspace suite, so targeted green evidence cannot be read as total phase closure.

**Reasoning:**

- The authoritative status source for original promises is the Phase 032 requirements table: five requirements are marked complete and two remain open. That means the correct closure statement must separate delivered current-stack lanes from the original broader security program.
- The live code matches that split. `ClaimStmtV2` now binds the full authenticated claim tuple, the current-stack spend verifier enforces a real signed proof/auth contract, the checkpoint package lane rechecks that contract before later stages accept it, and the stage-2 secret path keeps debug export behind the explicit `wallet_debug_dump` gate.
- The same repository also documents and implements why full closure is still unavailable. Claim trust still depends on `claim_source_contract_for_item(...)`, which recreates a synthetic off-store helper contract instead of proving persisted membership continuity. The broader spend requirement still stays open because the regular public spend statement does not yet carry the original nullifier-semantics portion of `PH32-SPEND`.
- The tempting but false reading is: targeted release-style green reruns mean the whole original Scenario 1 security story is closed. The manifests and verification notes explicitly defeat that reading: the targeted matrix passed through `RESULT[17]`, but the broader workspace suite still recorded `RESULT[18]=FAIL`, and the verification artifact says those targeted passes are not honest evidence for full closure of `PH32-SPEND` or `PH32-CLAIM-TRUST`.

**Gap Or Blocker:** Full closure still needs two missing pieces: persisted storage-backed claim membership continuity instead of the synthetic one-item helper seam, and regular public spend-statement nullifier semantics. Those gaps can be closed only by implementing those contracts in code or by formally narrowing the original requirement wording and re-approving the narrower boundary.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The answer is intentionally partial because the repository itself keeps the broader original `PH32-CLAIM-TRUST` and `PH32-SPEND` promises open.

### 2. Current-Stack Truth Versus Future-Proof Ambition

🔴 **Quest:** Where does the repository force a reviewer to distinguish a stronger current-stack verification boundary from future-proof-system ambition, instead of allowing all proof-related language to collapse into one over-optimistic claim?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The repository forces that distinction in three places at once: architecture and planning artifacts, closeout and user-facing status artifacts, and live code comments or runtime error strings on the accepted spend and checkpoint seams. Those sources consistently require reviewers to describe a current-stack verifier boundary today while treating DLEQ, STARK/FRI, recursive checkpoint proofs, and stronger proof-backend claims as future or out-of-scope work unless code actually lands them.

**Evidence Trail:**

1. `.planning/phases/032-crypto-audit-scenario-1/032-04-ARCHITECTURE-NOTE.md` says Wave 3 delivers a `current-stack public spend-contract verifier at the accepted boundary`, then separately says it must not be described as DLEQ lock-in, a landed STARK/FRI stack, or a final proof-backend commitment.
2. The same architecture note gives the durable backend-agnostic rule: a future backend may replace the current proof object only if it proves the same canonical spend statement and keeps fail-closed acceptance semantics.
3. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md`, `docs/code-review/032-scenario-1-crypto-status.md`, and `032-07-SUMMARY.md` all repeat the same `proves / does not prove / out of scope` split for human-facing status language.
4. `crates/z00z_wallets/src/core/stealth/output.rs` says the accepted-flow wallet checks `do not upgrade those wallet checks into a public trustless verifier claim`.
5. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs` uses runtime wording `current-stack tx public spend proof build failed` and `current-stack tx public spend verifier failed`, which keeps the accepted boundary narrower than a generic final-proof claim.
6. `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` says the persisted checkpoint proof bytes are a `compatibility payload` and `not a standalone checkpoint-proof backend`.
7. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` says `CheckpointPackageProofVerifier` is a `current-stack package-coupled verifier` and explicitly not proof that a stronger standalone checkpoint proof backend was validated.

**Reasoning:**

- The architecture note is the canonical forcing function. It tells reviewers what Wave 3 may claim today, what it may not claim, and what rule any future backend must satisfy before it can replace the current proof object.
- The closeout and status notes propagate the same distinction into human-facing reporting, so current enforcement cannot be summarized as if STARK/FRI, recursive checkpoint proofs, or whole-chain trustless verification had already landed.
- The live code repeats the same honesty fence at the exact seams where overclaim would otherwise happen: wallet-local accepted-flow checks are labeled non-trustless, stage-4 errors are labeled `current-stack`, and stage-6 checkpoint proof plumbing is labeled compatibility or package-coupled rather than authoritative backend proof.
- The tempting but false reading is that once the repo has a persisted proof/auth contract, it can casually talk as if the final proof-backend question is settled. The repository defeats that reading by repeating the narrower boundary language in planning docs, closeout docs, status notes, comments, and runtime error strings.

**Gap Or Blocker:** None. The question asks whether the repository forces the distinction, and the combined planning, status, and code surfaces do that explicitly.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: Minor wording caveat only: the repo forbids whole-chain overclaim through the combined artifact set, not through one single sentence in the architecture note.

### 3. Blocked Remediation Discipline

🔴 **Quest:** What evidence proves that this phase remained a blocked remediation program with no-go gates around unresolved trust boundaries, rather than quietly turning into a feature-delivery phase that tolerated unfinished guarantees?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The final corrected Phase 032 artifact set preserves Phase 032 as a blocked remediation program rather than an ordinary feature-delivery phase. The repository does this by keeping explicit no-go language around unresolved trust boundaries, by requiring wave-based fail-closed execution rules, and by refusing to let green targeted tests or landed slices erase the still-open `PH32-SPEND` and `PH32-CLAIM-TRUST` gaps. The important nuance is that this discipline had to be reasserted after planning drift had marked those requirements complete too broadly; the final artifacts correct that drift instead of silently accepting it.

**Evidence Trail:**

1. `032-CONTEXT.md` says Wave 4 and Wave 5 remain blocked for closeout, calls Phase 032 `a blocked remediation phase, not a general feature phase`, and says unresolved claim authenticity, public spend verification, checkpoint truthfulness, and secret-artifact handling are `no-go gates for implementation claims`.
2. The same context file sets execution order `with explicit go or no-go gates` in decision `D-24`.
3. `032-04-PLAN.md` says that if the proof stack cannot support the required verifier contract, the task `must fail closed as a blocked remediation item rather than close on an honesty downgrade`.
4. `032-TEST-SPEC.md` says the broader phase remains reopened on `PH32-SPEND` and `PH32-CLAIM-TRUST`, says the verification artifact is intentionally narrower than full phase closeout, and says the test spec must not be read as phase-completion evidence for the broader reopened requirements.
5. `032-VALIDATION.md` marks the phase `status: partial`, `nyquist_compliant: false`, and records that `PH32-CLAIM-TRUST` and `PH32-SPEND` remain escalated implementation-level gaps rather than test-only gaps.
6. `032-VERIFICATION.md` states it must not be used to claim closure of the broader original `PH32-SPEND` or `PH32-CLAIM-TRUST` requirements.
7. `032-HONEST-CLOSEOUT.md` says the phase cannot yet be treated as fully closed against the original `PH32-SPEND` or `PH32-CLAIM-TRUST` wording.
8. `032-07-SUMMARY.md` records a post-closeout correction: planning had marked `PH32-SPEND` and `PH32-CLAIM-TRUST` complete more broadly than the code proved, and the artifacts were corrected to keep those broader requirements open.

**Reasoning:**

- A feature-delivery phase would allow green subtests or landed slices to blur into a generalized completion claim. Phase 032 does the opposite: it keeps the open trust boundaries explicit even after several remediation waves land.
- The no-go discipline is not just narrative. It appears in the phase boundary, in the per-wave plan contract, in the test-spec fallback rules, in the validation sign-off, and in the final closeout correction. Closure is therefore gated by unresolved trust-boundary proof, not by whether enough code shipped.
- The strongest contrary reading would be: Wave 3, Wave 5, and the green targeted reruns effectively converted Phase 032 into a mostly complete feature phase. The repository defeats that reading by retaining `partial` and `open` markers, by refusing Nyquist sign-off, and by explicitly correcting earlier over-broad completion language.

**Gap Or Blocker:** None. The question asks whether the blocked-remediation discipline is evidenced, and the repository documents that discipline directly and repeatedly.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The blocked-remediation discipline is proven by the final corrected artifact set, not by an uninterrupted absence of planning drift during the whole phase.

### 4. Planning Truth Versus Implementation Truth

🔴 **Quest:** If the planning artifacts, verification artifacts, and live code are compared side by side, where do they converge on the same boundary description and where does any residual wording still invite a more optimistic interpretation than the implementation can prove?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The corrected late-phase truth layer does converge across planning-closeout artifacts, verification artifacts, and live code on one narrower boundary description: Scenario 1 currently proves accepted current-stack claim, spend, checkpoint, and secret-hygiene enforcement at the boundaries the code actually delivers, while the broader original `PH32-CLAIM-TRUST` and `PH32-SPEND` wording remains open. Verification and closeout artifacts are explicit about that. Live code comments and accepted-path error strings repeat the same honesty fence through `current-stack`, `compatibility payload`, and `package-coupled` wording. But the planning corpus is not fully uniform: some earlier per-wave summaries, readiness notes, and one architecture note still use wording that can sound broader than the corrected implementation truth.

**Evidence Trail:**

1. `032-07-SUMMARY.md` says the closeout language was honest about current-stack boundaries, but later audit found planning had already marked `PH32-SPEND` and `PH32-CLAIM-TRUST` complete more broadly than the code proves.
2. `032-VALIDATION.md` marks `032-03-01` and `032-04-01` as `partial`, keeps overall `status: partial`, and says the helper-backed claim path and current-stack spend path are real but still narrower than the original broader requirement wording.
3. `032-VERIFICATION.md` says `PH32-SPEND` remains open and that the current tree proves only a narrower review-backed current-stack boundary.
4. `032-HONEST-CLOSEOUT.md` and `docs/code-review/032-scenario-1-crypto-status.md` both keep explicit delivered-versus-not-claimed boundaries and leave the broader original `PH32-SPEND` and `PH32-CLAIM-TRUST` wording open.
5. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` says `CheckpointPackageProofVerifier` is a current-stack package-coupled verifier, not proof that a stronger standalone checkpoint proof backend was validated.
6. `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` says checkpoint proof bytes are a compatibility payload and not a standalone checkpoint-proof backend.
7. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs` uses `current-stack tx public spend proof build failed` and `current-stack tx public spend verifier failed` wording on the accepted spend seam.
8. Residual optimistic wording remains in older planning artifacts. `032-02-SUMMARY.md` readiness text says later phases can move onto `storage-owned authoritative roots and proof retrieval`; `032-03-SUMMARY.md` accomplishment wording calls the helper-owned claim contract storage-owned or authoritative before its own correction section narrows that reading; `032-04-ARCHITECTURE-NOTE.md` still includes nullifier semantics inside the accepted-boundary description even though later corrected artifacts keep that broader `PH32-SPEND` wording open.
9. A smaller wording gap remains in live code: `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs` still says `stage4: tx verifier failed` without the narrower `current-stack` qualifier used in the accepted-path runtime flow.

**Reasoning:**

- The strongest convergence is in the corrected truth layer, not in every planning artifact equally. Late closeout, validation, verification, and user-facing status notes all agree that the delivered boundary is current-stack and narrower than the original broader requirement wording.
- Live code supports that same narrower story. Accepted spend and checkpoint seams are labeled as current-stack, compatibility-payload, or package-coupled rather than as a final proof backend.
- The answer cannot honestly claim full convergence across all planning artifacts because some older summaries and notes still sound stronger than the corrected closeout. The most important remaining wording conflict is that the architecture note still lists nullifier semantics inside the accepted boundary, while the corrected validation, verification, and honest-closeout artifacts say that portion of the original requirement remains open.

**Gap Or Blocker:** The comparison question is answerable now, but the repository still contains wording drift. A reviewer who reads older plan summaries or the older accepted-boundary phrasing without the later correction sections could still overread the delivered boundary.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The convergence claim is limited to the final corrected artifact set plus live-code honesty fences; the planning corpus as a whole still contains narrower-versus-broader wording tension.

### 5. Conditions For Honest Reclassification

🔴 **Quest:** What concrete conditions would have to become true before the phase could be honestly reclassified from partial closure to full closure, and which of those conditions are still absent in the current tree?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository makes the two primary full-closure blockers explicit. Phase 032 can be honestly reclassified from partial closure to full closure only when the still-open original `PH32-CLAIM-TRUST` and `PH32-SPEND` requirements stop being open in a repository-backed way. For `PH32-CLAIM-TRUST`, that means either implementing persisted storage-backed claim membership continuity and re-verifying it, or formally narrowing and re-approving the requirement. For `PH32-SPEND`, that means either carrying and validating nullifier semantics inside the regular-spend public contract and re-verifying it, or formally narrowing and re-approving the requirement. Those conditions are still absent in the current tree. Broad-suite release evidence is an additional honesty caveat for stronger release-confidence claims, but it is not presented by the repository as a co-equal requirement-level closure gate.

**Evidence Trail:**

1. `.planning/REQUIREMENTS.md` still marks `PH32-CLAIM-TRUST` and `PH32-SPEND` as `Open`.
2. `032-HONEST-CLOSEOUT.md` says the phase cannot yet be treated as fully closed against the original `PH32-SPEND` wording and also cannot yet be treated as fully closed against the original `PH32-CLAIM-TRUST` wording.
3. `032-VALIDATION.md` says not to mark Phase 032 Nyquist-compliant until the claim-source helper derives root/proof data from persisted store-backed membership state or the requirement is formally narrowed and re-approved.
4. The same validation table says not to mark Phase 032 Nyquist-compliant until the regular-spend verifier carries and validates nullifier semantics or the requirement is formally narrowed and re-approved.
5. `032-07-SUMMARY.md` repeats both open-until conditions: `PH32-SPEND` remains open until nullifier semantics are implemented in the regular-spend public contract or the requirement is formally narrowed, and `PH32-CLAIM-TRUST` remains open until claim-source proofs are anchored in persisted storage-backed membership state or the requirement is formally narrowed.
6. `032-FULL-AUDIT.md` says Phase 032 must remain recorded as partially complete until the blocked semantic gaps are implemented and re-verified.
7. Supporting implementation evidence matches those gaps: `crates/z00z_storage/src/assets/store_internal/store_query.rs` still derives claim proof material from a synthetic off-store helper seam, and `crates/z00z_wallets/src/core/tx/spend_verification.rs` still does not make nullifier semantics part of the regular public-spend verifier contract.
8. `032-07-SUMMARY.md` also records that historical broad-suite manifests still contain `RESULT[18]=FAIL` and that no new authoritative broad-suite PASS artifact was produced during the review session, but this is framed as a release-evidence caveat rather than as the primary semantic closure gate.

**Reasoning:**

- This answer is mostly direct repository restatement, not inference. Validation, honest-closeout, post-closeout correction, requirements, and full-audit artifacts all identify the same two remaining semantic blockers.
- The repository also gives two ways to remove each blocker: implement the missing semantics and re-verify them, or formally narrow and re-approve the original broader requirement wording.
- The absence of a new broad-suite PASS artifact matters if someone wants to make a stronger release-wide evidence claim, but the repository does not present that absence as the same type of full-closure blocker as the two still-open semantic requirements.

**Gap Or Blocker:** The current tree still lacks both semantic closures in implementation form, and the canonical requirements source still leaves both requirements open. Broad-suite evidence also remains unsuitable for stronger release-wide overclaims, but that is a supporting caveat rather than the main closure gate.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The two requirement-level semantic gaps are the primary full-closure blockers; broad-suite evidence is secondary support for broader release-confidence claims.

## 🔐 Theme 2: Claim Authenticity And Source Truth

### 6. Full Authenticated Claim Tuple

🔴 **Quest:** What is the complete authenticated claim statement in the accepted flow, and which components of that statement must drift before authority authentication is supposed to fail?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The complete authenticated claim statement in the accepted flow is the canonical `ClaimStmtV2` byte frame. It binds these fields together under one authority-signature contract: `chain_id`, `root_ver`, `proof_ver`, `tx_ver`, `range_ctx_hash`, `claim_id`, `claim_source_asset_id`, `claim_source_commitment`, `source_root`, `claim_scope_hash`, `recipient_binding`, `nullifier`, `owner_bind_digest`, and `output_leaf_hashes`. Because `ClaimAuthoritySigV2` signs and verifies the canonical `ClaimStmtV2` bytes, drift in any of those fields changes the signed message and invalidates statement-level authority verification. In the accepted package-verifier flow, however, some drift cases fail closed earlier on scope, proof, source metadata, or nullifier checks before the explicit authority-signature step runs, so the repository proves the whole signed tuple strongly but does not provide one separate direct signature-mutation test for every field.

**Evidence Trail:**

1. `crates/z00z_crypto/src/claim/v2.rs` defines `ClaimStmtV2` with exactly these fields: `chain_id`, `root_ver`, `proof_ver`, `tx_ver`, `range_ctx_hash`, `claim_id`, `claim_source_asset_id`, `claim_source_commitment`, `source_root`, `claim_scope_hash`, `recipient_binding`, `nullifier`, `owner_bind_digest`, and `output_leaf_hashes`.
2. `ClaimStmtV2::to_bytes()` serializes those fields into one canonical `CLM2` byte frame in fixed order.
3. `crates/z00z_crypto/tests/test_claim_v2_contract.rs` frame-vector test constructs the whole canonical frame and round-trips it through `ClaimStmtV2::from_bytes()`, proving the exact statement shape.
4. The same test file signs a `ClaimStmtV2` and shows that mutating `claim_scope_hash` causes `ClaimAuthoritySigV2::verify(...)` to fail with `SigInvalid`.
5. The same contract-test file also proves fail-closed mismatch handling for `root_ver`, `proof_ver`, and `source_root` against the authoritative source proof.
6. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` reconstructs the accepted-flow `ClaimStmtV2` from package state, including `claim_source_asset_id`, `claim_source_commitment`, `source_root`, `claim_scope_hash`, `recipient_binding`, `nullifier`, `owner_bind_digest`, and output leaf hashes.
7. The same verifier rejects source-root mismatch, proof-blob root mismatch, asset-path drift, and source-commitment drift against that statement/proof contract.
8. `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` proves practical fail-closed rejection for bad scope hash, proof-blob root mix, source-commitment drift, and bad authority signature in accepted-flow packages.
9. `032-CONTEXT.md` decisions `D-05` and `D-05a` say the canonical claim statement must bind the authoritative source root and the full authenticated claim tuple, and that root binding alone is insufficient if the rest of the tuple can drift.

**Reasoning:**

- The full signed statement is explicit in code, not inferred from prose. The canonical byte-frame contract is the `ClaimStmtV2` field set serialized by `to_bytes()`.
- Since the authority signature is computed over that canonical byte frame, drift in any statement component changes the signed message and therefore breaks statement-level signature validity.
- The accepted package verifier is stricter than just the signature step: some fields are checked earlier by structure, proof, source metadata, scope, or nullifier validation. So the safest wording is not that every drift is directly observed at the authority-check line, but that the full tuple is signature-covered while the accepted flow may reject some drifts even earlier.

**Gap Or Blocker:** The repository proves the complete statement shape and the full signature-coverage rule, but it does not contain one separate direct signature-mutation test for every individual field.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: Individual negative tests are sampled rather than exhaustive per field, and some accepted-flow drift cases fail before the explicit authority-signature step.

### 7. Tuple Drift Under Plausible Package Shape

🔴 **Quest:** Can a claim remain acceptable if the authenticated root, source commitment, scope, or recipient-binding semantics are changed while the surrounding package still looks structurally plausible, or does the repository now fail closed against that class of drift?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository now fails closed against that class of drift. A claim package can still look structurally plausible at the JSON or field-shape level, but it does not remain acceptable if the authoritative root, source commitment, scope, or recipient-binding semantics drift away from the canonical accepted-flow contract. Instead, the verifier rejects those cases before any later acceptance step can succeed. The proof is strongest for source-root, proof-root, source-commitment, scope-hash, and output owner-binding drift. Recipient-binding semantics are also fail-closed in code, but the repository’s direct negative test matrix there is narrower than for the root/proof/source-commitment paths.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/claim_tx.rs` runs the verifier in fail-closed order: structure, scope, tx fields, nullifier, recipient card, portable leaf, reconstructed `ClaimStmtV2`, claim proof, claim authority, then later owner-attest and digest checks. There is no later compatibility lane that can rescue a semantically drifted package after one of those checks fails.
2. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` reconstructs the canonical `ClaimStmtV2` from package state and checks source-root and source-metadata consistency against the accepted source proof and decoded proof blob.
3. The same proof verifier rejects source-root mismatch, proof-blob root mismatch, source-commitment drift, and asset-path inconsistency against the canonical source contract.
4. `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` proves that a structurally plausible package with a wrong `claim_scope_hash` is rejected as `claim_structure_invalid`.
5. The same wallet test file proves that structurally plausible proof payloads with bad root semantics are rejected as `claim_proof_invalid`, including proof-blob root mix and zero-root cases.
6. The same wallet test file proves that source-commitment drift is rejected as `claim_proof_invalid`.
7. The same wallet test file proves that output owner-binding drift against `recipient_owner_hex` is rejected as `claim_output_invalid`.
8. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl.rs` also rejects `recipient card owner_handle != recipient_owner_hex`, so recipient-binding semantics are fail-closed even when the card still parses and verifies.
9. `crates/z00z_crypto/tests/test_claim_v2_contract.rs` shows that mutating a signed claim-statement field such as `claim_scope_hash` invalidates authority verification itself, confirming that semantic drift inside the authenticated tuple cannot survive the signed-statement contract.
10. `032-CONTEXT.md` Wave 2 gate says a claim package must not be describable as authentic unless the root it proves is the same root the authority signature binds, and it preserves fail-closed behavior for wrong root, wrong asset path, wrong proof, forged authority statement, and version mismatch.
11. `032-02-SUMMARY.md` states that wallet verification was tightened to fail closed on proof, root, asset-id, and source-commitment drift.

**Reasoning:**

- The important distinction is between structural plausibility and semantic acceptance. The package can still decode and look well-formed, but once scope, proof-root, source commitment, or recipient-binding semantics stop matching the canonical statement and source contract, the verifier exits before acceptance.
- Scope drift is rejected before proof verification. Proof-root and source-commitment drift are rejected inside the proof/source contract. Recipient-binding drift is rejected on output binding and card-owner alignment. So semantic plausibility is not enough to reach any accepted lane.
- The remaining caveat is evidentiary rather than architectural: recipient-binding fail-closed behavior is clearly implemented, but the direct negative test matrix is less exhaustive there than for root/proof/source-commitment drift.

**Gap Or Blocker:** The repository does not contain a full direct mutation matrix for every recipient-binding variant or every asset-path variant, but the implemented verifier paths are fail-closed and no acceptance lane is left open for the drift classes that are evidenced.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: Recipient-binding semantics are supported by code-path evidence plus some direct negative coverage, but not by an exhaustive mutation matrix comparable to the proof-root and source-commitment coverage.

### 8. Self-Consistency Versus Authority

🔴 **Quest:** Where is the boundary between a claim that is merely internally self-consistent and a claim that is actually anchored to authoritative source truth, and what repository evidence proves that this distinction matters in practice?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository places that boundary at re-derivation against the canonical claim-source contract, not at package self-consistency alone. A claim is merely internally self-consistent if its own statement, proof bytes, and authority signature line up with one another inside the package payload. It becomes anchored to authoritative source truth only when producer and consumer both use the same canonical helper-derived claim-source root or proof contract for the item and the package survives comparison against that external contract. The distinction matters in practice because the consumer rejects stale proof reuse, wrong authority anchors, and wrong authority signatures even when the package is still structurally plausible. The important limitation is that the current canonical contract is still helper-derived from a synthetic one-item off-store build, so the current tree proves a stronger authoritative-helper boundary, not full persisted storage-backed continuity.

**Evidence Trail:**

1. `crates/z00z_storage/src/assets/store_internal/store_query.rs` defines `AssetStore::claim_source_contract_for_item(...)`, which rebuilds the claim-source root and proof from a fresh off-backend one-item store and returns that canonical helper contract.
2. `032-03-SUMMARY.md` says producer and consumer now share one canonical claim-source contract across production and consumption and that acceptance fails closed on stale proofs, wrong signatures, and wrong authority anchors.
3. The same summary's post-closeout correction says this contract is still re-derived from a synthetic one-item store helper, so the original persisted storage-owned continuity wording remains open.
4. `032-VALIDATION.md` marks `PH32-CLAIM-TRUST` as `partial` and says producer and consumer share one canonical helper-owned root or proof contract, but persisted storage-backed continuity remains open.
5. `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` proves that emitted proof bytes and `source_root` must match the canonical contract returned by `AssetStore::claim_source_contract_for_item(...)`.
6. The same simulator test file proves that stale proof reuse is rejected, so package-local plausibility is not enough.
7. The same simulator test file proves that wrong authority anchors and wrong authority signatures are also rejected even when the package is still structurally plausible.
8. `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` uses the helper contract during production and fails closed if the signed statement root drifts from the returned proof root.
9. `crates/z00z_simulator/src/claim_pkg_consumer.rs` re-derives the expected root and proof and rejects package drift against that helper-derived contract.
10. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` adds local proof, root, path, and commitment cross-checks under synthetic reconstruction, which strengthens package rejection but does not by itself upgrade the boundary into persisted storage-backed authority.

**Reasoning:**

- If package self-consistency alone were enough, then any package that carried coherent internal fields could pass as long as its own proof and signature looked plausible. The current repository no longer allows that.
- Instead, the accepted producer and consumer flow uses one shared helper-derived contract outside the package payload and rejects the package if it diverges from that contract. That is the practical line between mere coherence and accepted authority in the current tree.
- But that authority line is still narrower than the original broader claim-trust requirement. The re-derived contract is canonical for the current helper boundary, yet it is still synthesized from an off-store one-item helper rather than persisted continuity that survives outside helper reconstruction.

**Gap Or Blocker:** The repository proves that package self-consistency is insufficient and that a helper-derived canonical contract matters in practice, but it does not yet prove persisted storage-backed membership continuity. That surviving seam is exactly why `PH32-CLAIM-TRUST` remains partial.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: `authoritative source truth` in the current tree means authoritative relative to the shared helper-derived contract, not yet authoritative relative to persisted storage-backed continuity.

### 9. Distinct Claim Reject Paths

🔴 **Quest:** What evidence shows that wrong authenticated root, wrong proof bytes, wrong authority signature, wrong anchor context, and other claim-path manipulations are rejected as meaningfully distinct failures rather than flattened into one generic mismatch?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The repository does prove that claim-path failures are not flattened into one generic mismatch. At the wallet-verifier layer, claim verification is fail-closed and category-distinct: structure and scope faults, proof-path faults, authority-signature faults, fee faults, nullifier faults, and output or leaf-binding faults land in different `reject_class` buckets. Within that structure, several proof-path drifts are intentionally grouped under the shared `claim_proof_invalid` class rather than each getting a unique top-level label. The simulator consumer then adds a second distinction layer for authoritative-seam failures, using explicit messages for wrong authority anchor, storage-authoritative root mismatch, proof-version mismatch, and proof-blob mismatch. So the current tree separates these failures at reject-category plus seam-message granularity instead of collapsing them into a single generic digest or malformed-package error.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/claim_errors.rs` defines distinct failure variants such as `StructureMalformed`, `DigestMismatch`, `SourceProofMismatch`, `AuthoritySigInvalid`, `NullifierMismatch`, `OutputOwnerBindingMismatch`, `RecipientCardMismatch`, and `FeeNonZero`.
2. `crates/z00z_wallets/src/core/tx/claim_tx.rs` maps those variants into distinct `reject_class` values: `claim_structure_invalid`, `claim_proof_invalid`, `claim_authority_invalid`, `claim_fee_invalid`, `claim_nullifier_invalid`, and `claim_output_invalid`.
3. The same verifier runs in fixed order: structure, scope, tx fields, nullifier, recipient card, leaf, statement, proof, authority, owner attestation, and only then digest. That ordering means an earlier semantic failure wins before any late digest fallback can flatten it.
4. `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` proves structure and scope are distinct from proof or authority failures: `test_bad_scope_hash` returns `claim_structure_invalid`, while `test_bad_proof_type` returns `claim_proof_invalid` and `test_odd_sig_hex` returns `claim_authority_invalid`.
5. The same test file proves nullifier faults stay distinct: `test_bad_nullifier_hex` and `test_nullifier_mismatch` both return `claim_nullifier_invalid` instead of a proof or digest class.
6. The same file proves output and ownership-path faults stay distinct: `test_owner_binding_mismatch`, `test_nonce_mismatch`, `test_zero_nonce_out`, and `test_empty_outputs` all return `claim_output_invalid`.
7. The same file proves proof-path manipulations are grouped but still distinct from generic mismatch: `test_bad_proof_stmt`, `test_proof_blob_root_mix`, `test_source_commitment_drift_rejected`, `test_legacy_proof_stub`, and `test_zero_root_rejected` all return `claim_proof_invalid`.
8. `test_proof_beats_digest_mismatch` shows a mutated proof plus a bad `tx_digest_hex` still returns `claim_proof_invalid`, and its report shows `digest_checked == false`, proving the verifier does not flatten the result into digest mismatch when a more specific earlier proof failure exists.
9. `test_fee_beats_digest_mismatch` does the same for fees: a nonzero fee plus bad digest still returns `claim_fee_invalid`, with `digest_checked == false`.
10. `test_report_stops_auth` shows a malformed authority signature returns `claim_authority_invalid` after proof already passed, and the report records that authority and later stages did not run, proving distinct fail-stop behavior instead of generic late mismatch.
11. `crates/z00z_simulator/src/claim_pkg_consumer.rs` adds separate authoritative-seam error messages: `claim root version mismatch against storage authoritative root`, `claim source root mismatch against storage authoritative root`, `claim proof version mismatch against storage authoritative proof`, and `claim proof blob mismatch against storage authoritative proof`.
12. The same consumer preserves wallet-verifier `reject_class` and `errors` in package-load failures, so simulator consumers still see structured verifier failures instead of one undifferentiated load error.
13. `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` proves anchor context and authority signature stay distinct at the consumer seam: `test_claim_pkg_consumer_rejects_wrong_authority_anchor` requires `claim authority anchor is simulator-only`, while `test_claim_pkg_consumer_rejects_wrong_authority_signature` requires `claim_authority_invalid`.

**Reasoning:**

- The repository does not merely expose many error strings; it enforces category separation in the verifier by mapping concrete `ClaimTxError` variants into distinct reject classes.
- The fixed verifier order matters. Because digest verification is last, proof, fee, nullifier, authority, and output failures can be observed as their own categories instead of being overwritten by a generic digest mismatch.
- The strongest contrary reading would be: different claim manipulations ultimately just land in one catch-all package failure. The repository defeats that reading in two layers. First, wallet tests assert different `reject_class` values for different fault families. Second, the simulator consumer adds explicit authoritative-seam mismatch strings for root, proof, and anchor-context problems.
- The honest nuance is that distinctness is not one-unique-label-per-mutation. Several proof-path drifts intentionally share `claim_proof_invalid`. That still proves meaningful separation because proof-path failures are kept apart from authority, fee, nullifier, output, and generic structure failures, and the consumer seam further distinguishes root-versus-proof authoritative mismatches by message.

**Gap Or Blocker:** No blocker for the question itself. The main nuance is only granularity: not every individual manipulation gets its own top-level `reject_class`, and one consumer stale-proof test allows multiple acceptable failure substrings rather than pinning a single exact message.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: Distinctness is proven at reject-category plus seam-message granularity, not as a unique enum or reject class for every single individual mutation.

### 10. The Seam That Keeps Claim Trust Partial

🔴 **Quest:** Which surviving claim-path seam still depends on synthetic reconstruction rather than persisted continuity, and why does that single seam prevent the repository from claiming full claim-trust closure?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The surviving seam is the canonical claim-source helper seam implemented by `AssetStore::claim_source_contract_for_item(...)`. That helper does not derive the claim root and proof from persisted storage-backed membership continuity. Instead, it rebuilds a fresh one-item off-store contract from the supplied item and treats that synthetic reconstruction as the canonical root/proof pair. Because both claim production and claim consumption converge on that same helper, the repository proves canonical helper consistency and mismatch rejection at that seam, but it still does not prove persisted continuity. That single seam blocks full `PH32-CLAIM-TRUST` closure because the broader original requirement is about storage-owned authoritative continuity, not merely helper-consistent regeneration.

**Evidence Trail:**

1. `crates/z00z_storage/src/assets/store_internal/store_query.rs` shows `AssetStore::claim_source_contract_for_item(...)` building a fresh off-store backend via `Self::build(super::RedbBackend::off())`, inserting only the provided item, then deriving `claim_root` and `claim_proof` from that synthetic one-item store.
2. `crates/z00z_simulator/src/claim_pkg_consumer.rs` re-derives `expected_root` and `expected_proof` through that same helper and compares package values against the helper-derived contract, proving producer and consumer converge on one canonical helper seam.
3. The same consumer code reports explicit helper-seam mismatches such as `claim source root mismatch against storage authoritative root` and `claim proof blob mismatch against storage authoritative proof`, so the seam is enforced in practice.
4. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` states Scenario 1 does not yet prove persisted storage-backed claim membership continuity because the current helper re-derives the source root and proof from a synthetic one-item store contract.
5. The same closeout file says the phase proves a shared canonical claim-source helper boundary, not persisted storage-backed claim membership continuity.
6. `.planning/phases/032-crypto-audit-scenario-1/032-VALIDATION.md` marks `PH32-CLAIM-TRUST` partial because claim production and consumption share one canonical helper-owned root or proof contract, but persisted storage-backed continuity for the original requirement remains open.
7. The same validation artifact says not to mark Phase 032 Nyquist-compliant until the helper derives root and proof data from persisted store-backed membership state, or the requirement is formally narrowed and re-approved.
8. `.planning/REQUIREMENTS.md` still keeps `PH32-CLAIM-TRUST` open, which confirms that the helper seam is not accepted as full storage-owned authoritative closure.

**Reasoning:**

- The repository does not leave the surviving seam vague. The seam is concrete in code: a helper reconstructs the root and proof by inserting one item into a fresh off-store backend.
- That is enough to prove canonical reconstruction consistency. Producer and consumer agree on the same derived contract, so stale, forged, or mismatched packages can be rejected relative to that helper-defined source truth.
- But helper consistency is not the same thing as persisted continuity. The broader original claim-trust requirement was about an authoritative claim path anchored in storage-owned membership continuity, and the helper bypasses exactly that question by rebuilding the contract from the item itself.
- The strongest contrary reading would be: if both sides use the same helper and reject drift, claim trust is fully closed. The repository rejects that reading explicitly in closeout, validation, and requirements status language. It calls the seam canonical and useful, but still partial because the persisted-storage continuity question remains unanswered.

**Gap Or Blocker:** The blocker is singular and specific: the claim-source helper still reconstructs a synthetic one-item contract instead of proving continuity against persisted storage-backed membership state. Full closure requires either implementing that persisted continuity or formally narrowing and re-approving the broader original requirement.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The repository proves strong mismatch rejection at the helper seam, but only relative to the helper-derived canonical contract, not yet relative to persisted storage-backed continuity.

## 🧩 Theme 3: Ownership Semantics And Stealth Freeze

### 11. Sender Knowledge And The Narrower Anti-Theft Rule

🔴 **Quest:** What exactly disproves the idea that the sender is ignorant of output-secret material, and what narrower anti-theft rule remains defensible once that myth is removed?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The sender-ignorance myth is disproved by the accepted Scenario 1 output-build contract itself. The repository explicitly freezes `s_out` as output secret material that is derived during sender-side construction from sender-available build material, so Phase 032 forbids claiming sender ignorance of `s_out`. Once that overclaim is removed, the remaining defensible anti-theft rule is narrower: wallet-local spend ownership still requires receiver-secret-gated verification in addition to the output secret material, including the correct `s_out`.

**Evidence Trail:**

1. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` says `s_out` is part of output secret material, says current Scenario 1 output construction derives `s_out` from sender-side available material during output build, and explicitly says Phase 032 must not claim sender ignorance of `s_out`.
2. The same semantic-freeze file states the truthful anti-theft rule is narrower: wallet-local spend ownership still requires receiver-secret-gated verification in addition to the output secret material.
3. `crates/z00z_wallets/src/core/stealth/output_build.rs` documents that current Scenario 1 semantics keep sender-side derivation of `k_dh` and `s_out` explicit during output construction.
4. The same build path actually computes `s_out = derive_s_out(&build_mat.k_dh, &build_mat.r_pub, serial_id)`, and `build_mat.k_dh` is derived inside the sender-side construction path.
5. `crates/z00z_wallets/src/core/stealth/output.rs` defines `verify_owner_two_factor(...)` as the accepted wallet-local spend ownership rule and explicitly warns that this must not be read as proof that the current public verifier path already proves the same property end to end.
6. The same function requires a `receiver_secret` and the correct `s_out`, then derives the expected owner tag and expected `s_out` from receiver-held material before returning success.
7. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves the rule in practice: correct receiver secret plus correct `s_out` succeeds, substituting sender material fails, and tampering `s_out` fails.

**Reasoning:**

- The repository does not treat sender ignorance as an open philosophical question. It freezes the answer explicitly and then implements it in the sender-side build seam.
- Because the sender-side build path derives `s_out`, any statement that security depends on the sender not knowing `s_out` is already contradicted by live code.
- The honest remaining rule is therefore not ignorance-based. It is receiver-gate-based: accepted wallet-local ownership still requires receiver-secret-gated verification in addition to the output secret material that the sender may help construct.

**Gap Or Blocker:** No blocker for this question. The only important limitation is scope: the narrower anti-theft rule is proven as a wallet-local accepted-flow rule, not as a universally public or trustless exclusion theorem.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The strongest supported claim is that sender ignorance of `s_out` is false and that wallet-local ownership still requires the receiver-secret gate; it is not a proof that every public verifier path already enforces the same rule.

### 12. Receiver-Held Secret As The Ownership Gate

🔴 **Quest:** Which repository-backed chain of evidence shows that spend authorization depends on a receiver-held secret in addition to sender-visible material, and where does that dependence remain a wallet-local rule rather than a publicly proven exclusion property?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The repository-backed chain is direct and internally consistent. The semantic freeze defines the honest rule, the accepted output-build seam still exposes sender-side material including derived `s_out`, and `verify_owner_two_factor(...)` then makes accepted wallet-local spend ownership depend on both a receiver-held secret and the correct `s_out`. The semantic regression suite proves both factors are necessary by rejecting sender-derived substitute material and rejecting tampered `s_out`. That dependence remains wallet-local rather than publicly proven end to end because both code comments and Phase 032 status artifacts explicitly deny that the current public verifier path proves every wallet-local ownership invariant.

**Evidence Trail:**

1. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` states that wallet-local ownership checks may use receiver-secret-gated verification and decrypted `s_out` semantics, and that public trustless verification is not implied by those wallet-local checks.
2. The same file states the honest anti-theft rule: wallet-local spend ownership still requires receiver-secret-gated verification in addition to output secret material.
3. `crates/z00z_wallets/src/core/stealth/output_build.rs` keeps sender-side derivation of `k_dh` and `s_out` explicit during output construction, proving sender-visible build material exists but is not the final ownership gate.
4. `crates/z00z_wallets/src/core/stealth/output.rs` implements `verify_owner_two_factor(receiver_secret, r_pub, owner_tag, s_out, serial_id)` as the accepted wallet-local spend ownership rule.
5. The same function derives `owner_handle` and `view_sk` from `receiver_secret`, recomputes the expected owner tag, then recomputes expected `s_out` and checks that it matches the provided `s_out`.
6. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves the rule with three outcomes: receiver secret plus correct `s_out` passes; substituting sender material fails; and receiver secret plus tampered `s_out` fails.
7. `crates/z00z_wallets/src/core/stealth/output.rs` also states that the validated accepted-flow constructor enforces explicit request approval and route matching but does not upgrade those wallet checks into a public trustless verifier claim.
8. `docs/code-review/032-scenario-1-crypto-status.md` says the repository does not claim a universal trustless public spend verifier for every wallet-local ownership invariant beyond the current accepted boundary delivered in code.

**Reasoning:**

- Sender-visible material alone is insufficient because the accepted ownership rule still requires receiver-held secret material at verification time.
- The key proof is not just prose. It is the combination of the implementation of `verify_owner_two_factor(...)` and the regression test that rejects both fake receiver material and tampered `s_out`.
- The wallet-local versus public boundary is explicit. The repository repeatedly says that these checks are real and accepted-flow-relevant, but not yet a universally proven public-verifier property.

**Gap Or Blocker:** No blocker for the question itself. The remaining limitation is the same honest scope boundary: the dependence on receiver-held secret is repository-proven as an accepted wallet-local ownership rule, not as a completed end-to-end public exclusion property.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The strongest supported wording is `accepted wallet-local spend ownership rule`, not a fully public theorem about all spend authorization paths.

### 13. Canonical Decrypt-Associated Asset Binding

🔴 **Quest:** Has the decrypt-associated asset binding been frozen into one canonical meaning across output build, scan, spend, and runtime parity, or can two nearby asset-identification concepts still drift apart without immediate rejection?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The decrypt-associated asset binding has been frozen into one canonical meaning in the accepted wallet and runtime paths. `leaf_ad_id` is the canonical decrypt-associated-data boundary, and drift between that boundary and nearby asset-identification concepts is treated as semantic failure, not compatibility. The repository does preserve an explicit split between canonical state `asset_id` and decrypt-boundary `leaf_ad_id` at the spend-witness bridge, but that split is intentional and guarded. In the shipped scan, report, and spend-witness bridge paths, drift is rejected instead of being silently tolerated.

**Evidence Trail:**

1. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` says `leaf_ad_id` is the canonical decrypt-associated-data asset identifier and that any drift between the stored leaf asset identifier and the decrypt-associated identifier is a semantic failure, not a compatibility detail.
2. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` repeats that the decrypt-associated-data boundary is canonical `leaf_ad_id`, not a caller-chosen compatibility alias.
3. `crates/z00z_wallets/src/core/stealth/output_build.rs` derives `leaf_ad` from the build-time `asset_id`, serial id, `r_pub`, `owner_tag`, and commitment bytes, freezing one constructor-side meaning.
4. `crates/z00z_wallets/src/core/address/leaf_scan.rs` and `stealth_scan_support.rs` consume `leaf_ad_id` as the decrypt boundary during scan.
5. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves the canonical rule directly: an output scans as owned before drift, then fails once `asset.leaf_ad_id` is mutated.
6. `crates/z00z_wallets/src/core/address/leaf_scan.rs` includes `leaf_scan_ad_drift`, which corrupts the decrypt-boundary asset identifier and then observes both `receiver_scan_leaf()` failure and `RECEIVE_INVALID_PROOF` status in the report path.
7. `crates/z00z_wallets/src/core/tx/witness_gate.rs` explicitly preserves the intentional split between canonical state `asset_id` and decrypt-boundary `leaf_ad_id`: `wire_decrypt_leaf()` rewrites the leaf to use `leaf_ad_id` for the decrypt contract, and `resolve_input_pack()` requires `leaf_ad_id` when reconstructing spend-witness input state.

**Reasoning:**

- The repository is not claiming one universal `asset_id` with hidden dual meaning. It freezes one canonical decrypt boundary and then makes the build, scan, and witness bridge respect that choice.
- Drift is observable and rejected in live accepted paths. Both the semantic regression suite and the leaf-scan unit tests show that mutating the decrypt-boundary identifier makes the output fail ownership detection or report as invalid proof.
- The strongest contrary reading would be that two nearby asset-identification concepts can still slide past each other as long as local helpers agree. The bridge code defeats that reading by preserving the split intentionally and by requiring the decrypt path to use `leaf_ad_id` rather than the canonical state key.

**Gap Or Blocker:** No blocker for the question. The only caution is scope: the repository proves this freeze for accepted wallet, scan, report, and spend-witness bridge paths, not as a statement about every hypothetical helper or future verifier backend.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The freeze is strongest in shipped wallet and runtime paths; the phase note is supporting evidence, but the decisive proof comes from code and tests.

### 14. Request-Bound Route Versus Card-Bound Route

🔴 **Quest:** What keeps request-bound routing distinct from card-bound routing in the accepted privacy path, and what failure or privacy risk would reappear if those two modes were silently treated as interchangeable?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Request-bound routing and card-bound routing remain intentionally distinct in the accepted privacy path because they derive `tag16` differently, verify ownership through different contexts, and are admitted only after explicit request approval plus route-matching checks. They are not interchangeable. If they were silently treated as interchangeable, accepted-flow verification would break and the path would re-open cross-owner route substitution risk by letting a foreign request or mismatched route masquerade as an approved receiver path.

**Evidence Trail:**

1. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` says card-bound mode derives `tag16` from `k_dh` and canonical `leaf_ad`, while request-bound mode derives `tag16` from `k_dh` and `req_id`, and says those modes are intentionally distinct and must stay test-covered as distinct semantics.
2. `crates/z00z_wallets/src/core/stealth/output_validator.rs` encodes this distinction directly in `TagMode::{CardBound, RequestBound { req_id }}` and recomputes expected `tag16` differently for each mode.
3. `crates/z00z_wallets/src/core/stealth/output_build.rs` enforces `validate_request_bind(...)` before request-bound behavior is allowed, requiring the request and receiver card to describe the same route.
4. The same build path requires explicit request approval through `approve_req(...)`; request-bound mode is not available merely because a request object exists.
5. `crates/z00z_wallets/src/core/address/stealth_request.rs` implements `validate_all(...)` as the accepted wallet policy gate for request use and explicitly says this remains a wallet-local approval boundary rather than a public verifier claim.
6. `crates/z00z_wallets/src/core/address/stealth_trust.rs` implements explicit TOFU, identity-change, and rotation behavior, so request/card acceptance is policy-driven rather than silent substitution.
7. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves the semantic divergence directly: `scenario1_request_bound_tag16_diverges_from_card_bound_mode` shows request-bound and card-bound `tag16` differ, that request-bound output fails card-bound verification, and that it succeeds only under `verify_owner_tag_with_req(...)` with the correct `req_id`.
8. The same test file proves silent interchangeability is rejected: `scenario1_foreign_request_card_mismatch_fails_accepted_flow` builds a foreign request/card combination and gets `StealthError::InvalidStealthInput`.
9. `scenario1_tofu_and_rotation_are_explicit` proves first-seen routing and later rotation require explicit TOFU or confirmation rather than silent acceptance.

**Reasoning:**

- The distinction is cryptographic and policy-level at the same time. The tag derivation formulas differ, and the accepted-flow policy refuses to let the request path bypass explicit route approval.
- The repository therefore proves more than stylistic separation. Wrong-mode verification fails, and foreign request/card substitution is rejected before the path can proceed as accepted-flow privacy routing.
- The main honest nuance is that the privacy-risk framing is partly interpretive. The repository proves route-binding and semantic separation directly; the privacy consequence follows from that binding rather than from a separately named privacy theorem.

**Gap Or Blocker:** No blocker for the question. The only caution is wording: the strongest repository-backed statement is that silent interchange would break accepted-flow verification and route binding; privacy-risk language should be kept as a consequence, not overstated as a separately proved theorem.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The semantic separation and rejection behavior are directly proven; the privacy-risk consequence is strongly implied but should be phrased carefully.

### 15. Exclusivity After Scan

🔴 **Quest:** After a receiver successfully scans and stores an incoming output, does the live protocol require any re-encoding step to make that output safely exclusive, or should exclusivity already follow from the ownership rule enforced by the current stack?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** I found no repository evidence that the live accepted wallet and witness paths require a post-scan re-encoding step to make the output exclusive. The stronger supported reading is the opposite: exclusivity already follows from the current wallet-local ownership rule, which requires receiver-held secret material plus the correct `s_out`. The scanner and spend-witness gate both consume directly decrypted pack data rather than a second ownership-encoding transform. But this remains only a wallet-local exclusivity rule, not a publicly proven trustless exclusivity guarantee.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/stealth/output.rs` defines the accepted wallet-local ownership rule through `verify_owner_two_factor(...)`, which requires receiver-held secret material and the correct `s_out`.
2. The same function rejects wrong receiver material and rejects tampered `s_out`, showing exclusivity is enforced at the ownership-check layer rather than by a later re-encoding pass.
3. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves this directly in `scenario1_wallet_local_two_factor_ownership_needs_receiver_secret_and_s_out`: correct receiver secret plus correct `s_out` passes; sender-substitute material fails; and tampered `s_out` fails.
4. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` materializes the decrypted `pack.s_out` directly into `WalletStealthOutput.asset_secret` through `make_wallet_output(...)` instead of first re-encoding ownership into a new artifact.
5. `crates/z00z_wallets/src/core/tx/witness_gate.rs` consumes directly decrypted pack data through `resolve_input_pack(...)` and `resolve_input_secret(...)`, again without an additional post-scan ownership re-encoding step.
6. `crates/z00z_wallets/src/core/stealth/output_validator.rs` performs a sender-side consistency check over the already built output and decrypted pack fields, but it does not define a separate exclusivity re-encoding stage.
7. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` states that public trustless verification is not implied by the current wallet-local checks.

**Reasoning:**

- The live code paths that matter after scan and during spend-witness reconstruction work from the decrypted pack directly. I found no code path that requires re-encoding ownership into a new post-scan exclusivity format.
- The exclusivity mechanism the repository actually proves is the receiver-secret-plus-`s_out` rule. That rule is sufficient for accepted wallet-local exclusivity, so a separate post-scan re-encoding step is not evidenced as necessary.
- The answer stays partial because the question uses the phrase `safely exclusive`, and the repository is explicit that wallet-local ownership enforcement is not yet a universally public or trustless exclusivity theorem.

**Gap Or Blocker:** The evidence does not show a required re-encoding step, but it also does not elevate wallet-local exclusivity into a fully public proof boundary. That public or trustless gap is why the answer must remain partial.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The repository proves no required post-scan ownership re-encoding in accepted wallet and witness paths, but exclusivity remains a wallet-local enforced property rather than a fully public trustless guarantee.

## ⚙️ Theme 4: Spend And Checkpoint Verifier Boundaries

### 16. What The Spend Boundary Actually Proves

🔴 **Quest:** What does the accepted spend boundary actually prove today to an untrusted verifier, and what remains outside that proof even though the path is materially stronger and more truthful than the pre-phase state?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** Today the strongest honest spend claim is a narrower current-stack public spend contract over the persisted tx package, not full spend trust. To an untrusted verifier, the accepted public-verifier path proves that `TxWire.proof.spend` and `TxWire.auth.spend` are present with the expected versions; that the canonical spend statement is signed by the receiver identity key from the compact receiver card; and that the persisted transaction stays coherent with the claimed `prev_root`, positional input proof rows, outputs, recomputed leaf-ad relations, required range proofs, input/output disjointness, and commitment balance. This is materially stronger and more truthful than the pre-phase structural-only acceptance story. What still remains outside that proof is authoritative membership continuity of the consumed inputs under `prev_root`, nullifier semantics in the regular public spend statement, the broader original `PH32-SPEND` claim, and any stronger standalone checkpoint-proof backend.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/spend_verification.rs` defines the canonical statement encoder and verifier. The statement binds `chain_id`, `tx_version`, `tx_type`, `fee`, `nonce`, receiver handle and identity key, `prev_root`, serialized tx inputs, serialized proof rows, serialized outputs, and recomputed output leaf hashes.
2. The same verifier fail-closes on missing proof/auth, bad proof/auth version, zero `prev_root`, input-count mismatch, serial mismatch, duplicate input state refs, input `leaf_ad_hash` drift, duplicate output `leaf_ad_id`, input/output overlap, missing or bad range proofs, bad balance, and bad authorization.
3. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` explicitly narrows the surface: the regular spend proof carries local transaction-proof material only, while membership of reference-only inputs stays in the checkpoint or pre-state path where validators resolve leaves and check witnesses against `prev_root`.
4. The same wire types show that persisted `SpendProofWire` carries `ver`, `prev_root_hex`, and positional input proof rows, while persisted `SpendAuthWire` carries the receiver card and spend signature. There is no nullifier field in this regular public spend statement surface.
5. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs` builds persisted spend proof/auth and runs `verify_tx_public_spend_contract(...)` before package finalization, proving that the stage-4 package now includes a real wallet-level public verifier instead of only a structural witness story.
6. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` proves direct fail-closed rejection for missing auth, replay-style `prev_root` tamper, and `leaf_ad_hash` tamper, while also documenting the honest nuance that duplicate input `leaf_ad_id` values may remain valid when the true input refs are distinct.
7. `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` confirms that the persisted Scenario 1 tx package carries spend proof/auth and that simulator acceptance matches the wallet verifier on both canonical and fail-closed cases.
8. `.planning/phases/032-crypto-audit-scenario-1/032-04-SUMMARY.md` explicitly says this plan hardened the spend gate honestly but did not close `PH32-SPEND`, because the live regular-spend wire and persisted spend proof do not carry a nullifier field.
9. `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md` repeats that the current tree proves a narrower review-backed current-stack boundary and must not be used to claim closure of the broader original `PH32-SPEND` requirement.
10. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` explicitly narrows the later checkpoint seam: it proves that the checkpoint draft is bound to the persisted tx package contract, not that a stronger standalone checkpoint-proof backend was validated.
11. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` then shows that later checkpoint paths reject tampered exec rows, tampered package proof bytes, and tampered package digests, which strengthens the end-to-end story but does not upgrade the stage-4 spend verifier itself into a standalone authoritative backend.

**Reasoning:**

- The important honesty boundary is between tx-local public-contract coherence and broader spend-trust claims. The current verifier proves the former strongly: persisted proof/auth, signer binding, previous-root framing, proof-row pairing, output consistency, range-proof checks, disjointness, and balance are all enforced in one fail-closed contract.
- That is a real improvement over the pre-phase state, because stage-4 no longer stops at a structural witness narrative. A persisted tx package must now survive the wallet-level public verifier before it is finalized.
- But the repository also states its own limit. Input membership continuity lives in the checkpoint or pre-state path, not in the spend-proof wire itself, and nullifier semantics are still absent from the regular public spend statement. So the strongest honest answer is narrower than full spend trust.
- The later checkpoint layer strengthens package-coupled continuity and replay rejection, but even that code comments narrow the claim to the persisted tx package contract rather than a stronger independent checkpoint-proof backend.

**Gap Or Blocker:** The current tree still lacks nullifier semantics in the regular public spend statement and does not let the stage-4 public verifier alone prove authoritative membership continuity of consumed inputs under `prev_root`. Those are the remaining blockers that keep the broader spend-trust claim open.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: duplicate input `leaf_ad_id` values are intentionally allowed when true input refs remain distinct, so the honest invariant is unique input state refs plus output uniqueness and input/output disjointness, not universal `leaf_ad_id` uniqueness.

### 17. Structural Plausibility Versus Semantic Acceptance

🔴 **Quest:** Can structurally plausible but semantically incomplete spend artifacts still pass any acceptance lane, or do proof, authorization, previous-root, and output-relation checks now fail closed before any state mutation is allowed?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** In the current tree there is no proved accepted lane through which a structurally plausible but semantically incomplete spend artifact reaches an accepted tx package or checkpoint apply. Stage 4 now builds a real persisted public spend contract and runs the wallet-level public verifier before tx-package finalization, while stage 11 later fail-closes on package-coupled handoff drift before checkpoint draft creation and before `checkpoint_s7.json` emission. The wording still has to stay narrow, however: this is not proof that no early file mutation happens at all, because stage 4 writes the canonical prep snapshot and pre-tx view before the later witness-gate step, and not every negative branch in the public verifier has its own dedicated regression test in the current evidence set.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/spend_verification.rs` defines fail-closed public-verifier branches for missing proof/auth, bad versions, bad `prev_root`, input-proof drift, duplicate input refs, duplicate output `leaf_ad_id`, input/output overlap, bad balance, and authorization failure.
2. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` directly proves several semantic failures survive structural plausibility but still reject: missing spend auth returns `MissingAuth`, replay-style `prev_root` drift returns `AuthorizationFailed`, and tampered `leaf_ad_hash` returns `InputLeafAdHashMismatch`.
3. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs` shows the accepted stage-4 order: build spend proof/auth, run `verify_tx_public_spend_contract(...)`, then only later write the tx package. This means structural plausibility alone cannot reach an accepted package if the public contract fails.
4. The same stage-4 flow still has a later `spend_witness_gate` step, but that step is downstream of the real persisted public verifier and therefore is not a bypass around the public semantic contract.
5. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` runs `verify_stage7_handoff(...)` before `build_cp_draft(...)` and `save_draft(...)`, so checkpoint apply rejects package or handoff drift before accepted checkpoint persistence.
6. `verify_stage7_handoff(...)` first reuses `CheckpointPackageProofVerifier::verify_pkg_contract(&load.pkg)?`, which itself calls the current-stack tx public spend verifier, and then separately checks proof bytes, canonical input refs, and canonical outputs against the accepted stage-4 and stage-6 handoff.
7. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves that tampered exec-input rows, tampered tx-package proof bytes, and tampered tx-package digest all fail stage 11 instead of being accepted.
8. The same test file proves the strongest persistence-side consequence for exec-input drift: after the failure there is no emitted `checkpoint_s7.json`, and there is no post-tx checkpoint draft directory.
9. `.planning/phases/032-crypto-audit-scenario-1/032-05-SUMMARY.md` states the accepted boundary honestly: checkpoint acceptance is now tied to the persisted stage-4 package contract and stage-6 canonical bridge outputs, not to placeholder proof bytes or spent-state success semantics.

**Reasoning:**

- The critical distinction is between artifacts that still look well-formed and artifacts that are semantically acceptable to the accepted current-stack flow. The repository no longer treats those as equivalent.
- At stage 4, semantic checks happen before tx-package finalization. So a structurally plausible artifact with missing auth, broken signed-root relation, or broken input leaf transcript does not survive into an accepted package.
- At stage 11, semantic checks happen again at the package-coupled checkpoint handoff. A package or exec-input drift that still looks structurally plausible is rejected before checkpoint apply can emit accepted publication artifacts.
- The honest caveat is that the phrase “before any state mutation is allowed” is too broad for the current code. Some earlier stage-4 files, such as the canonical prep snapshot and pre-tx view, are written before the later witness-gate step. The narrower proven claim is that semantically incomplete artifacts do not survive into accepted tx-package finalization, sender-state persistence, or accepted checkpoint persistence lanes.
- A second caveat is evidentiary scope. Some fail-closed branches are directly regression-tested, while others are currently proven by code-path evidence rather than by their own standalone tests.

**Gap Or Blocker:** There is no blocker for the narrow claim that accepted tx-package and checkpoint-acceptance lanes fail closed on semantic spend drift. The remaining gap is only proof granularity: some negative branches such as duplicate-input-ref, duplicate-output-`leaf_ad_id`, input/output-overlap, and bad-balance rejection are explicit in code but are not each separately regression-tested in the evidence set used here.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: the strongest proven statement is about accepted tx-package and checkpoint-persistence lanes, not about the total absence of all earlier file writes in stage 4.

### 18. The Missing Spend-Statement Element

🔴 **Quest:** What exact public-input element is still missing from the regular spend statement and verifier contract, and why does that omission keep the broader spend-trust claim open?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The strongest exact wording the repository supports is not that one already-named spend-side field was forgotten, but that the regular public spend contract is still missing a nullifier field, more generally the nullifier-semantics surface. The current regular spend wire and statement bind previous root, input refs, proof rows, outputs, signer/card material, range-proof and balance relations, and chain/version framing, but they do not carry or verify nullifier semantics. That omission keeps the broader spend-trust claim open because the original `PH32-SPEND` requirement explicitly expected the public verifier contract to bind nullifier semantics, while the delivered contract proves only a narrower current-stack spend boundary.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` defines `SpendInputProofWire` with `serial_id`, `r_pub_hex`, `owner_tag_hex`, `commitment_hex`, `leaf_ad_id_hex`, and `leaf_ad_hash_hex`, but no nullifier field.
2. The same file defines `SpendProofWire` with only `ver`, `prev_root_hex`, and positional input proof rows, again with no nullifier field.
3. The same file defines `SpendAuthWire` with only `receiver_card_compact` and `spend_sig_hex`, so the persisted auth surface also carries no nullifier semantics.
4. `TxInputWire` in the same file explicitly says `serial_id` is not checkpoint nullifier material, which rules out the tempting but false reading that nullifier semantics are implicitly smuggled through the existing input-ref wire.
5. `crates/z00z_wallets/src/core/tx/spend_verification.rs` encodes the canonical regular spend statement from `chain_id`, `tx_version`, `tx_type`, `fee`, `nonce`, receiver handle and identity key, `prev_root`, serialized tx inputs, serialized proof rows, serialized outputs, and recomputed output leaf hashes.
6. The same verifier enforces proof/auth presence, root framing, input pairing, leaf-ad hashes, range proofs, balance, and authorization, but there is no nullifier input and no nullifier check in the regular public spend contract.
7. `.planning/REQUIREMENTS.md` keeps `PH32-SPEND` open and describes the broader spend-verifier expectation as including nullifier semantics in the public verifier contract.
8. `.planning/phases/032-crypto-audit-scenario-1/032-04-SUMMARY.md` explicitly says the live regular-spend wire and persisted spend proof do not carry a nullifier field, so that plan cannot honestly claim nullifier semantics in the delivered public spend statement.
9. `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md` and the later Phase 032 closeout artifacts repeat that `PH32-SPEND` remains open because the current tree still does not prove the original nullifier-semantics portion of the requirement.
10. `.planning/phases/032-crypto-audit-scenario-1/032-CONTEXT.md` and `.planning/phases/032-crypto-audit-scenario-1/032-TODO.md` show the intended spend-proof binding set included nullifier semantics, confirming that this is not a documentation accident but a real missing element relative to the original plan.

**Reasoning:**

- The repository makes the gap explicit in both code and audit artifacts. The regular spend contract is not missing some hidden internal nuance; it is missing the nullifier-binding surface itself.
- The code proves this at the schema level first: the persisted spend proof and auth wires have no nullifier field. The statement encoder then proves it again at the transcript level: nullifier semantics are not framed into the signed public spend statement.
- That matters because the original requirement was broader than “current-stack spend proof/auth exists.” It expected the public verifier contract to bind nullifier semantics too. Since the delivered contract does not do that, the stronger spend-trust claim cannot be honestly closed.
- The false optimistic reading would be: perhaps `serial_id`, `asset_id_hex`, or some other existing field already acts as nullifier material. The wire-type comments explicitly reject that reading by stating that `serial_id` is not checkpoint nullifier material, and the statement/verifier code contains no alternative nullifier-binding rule.

**Gap Or Blocker:** There is no blocker to the conclusion itself. The only wording caveat is precision: the safest exact phrase is “missing nullifier field / missing nullifier semantics surface,” not the name of some already-existing spend-side field identifier, because no such nullifier field currently exists in the regular spend wire schema.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: the answer is exact at the contract-surface level, but it intentionally does not invent a literal field name that the current spend schema does not yet have.

### 19. Checkpoint Continuity Or Compatibility-Looking Proof Bytes

🔴 **Quest:** Does the checkpoint handoff enforce authoritative continuity of the spend package it receives, or can later stages still treat compatibility-looking proof bytes as if they were an authoritative checkpoint-proof backend?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The current checkpoint handoff enforces real continuity of the received stage-4 spend package and canonical stage-6 bridge contract, but only in the narrower current-stack, package-coupled sense. Later stages no longer accept merely compatibility-looking proof bytes as enough for an accepted handoff. Instead, stage 11 revalidates the stage-4 package, compares exec proof bytes to the persisted package proof, compares canonical input refs to package inputs, and compares canonical outputs to stage-6 bridge outputs before draft creation. What the repository still does not support is the stronger claim that this is an authoritative standalone checkpoint-proof backend; the code and phase artifacts explicitly narrow it to a package-coupled continuity verifier.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` runs `verify_stage7_handoff(...)` before `build_cp_draft(...)` and before any checkpoint draft save, so handoff drift is checked prior to accepted checkpoint persistence.
2. The same `verify_stage7_handoff(...)` first calls `CheckpointPackageProofVerifier::verify_pkg_contract(&load.pkg)?`, which reuses the current-stack tx public spend verifier on the received stage-4 package.
3. The same handoff function then rejects `exec tx proof mismatch with stage4 package`, `exec input refs mismatch with stage4 package`, and `exec outputs mismatch with stage6 bridge outputs`, which proves the handoff is bound to the upstream package/bridge contract rather than to mere non-empty proof bytes.
4. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` explicitly comments that this is a current-stack package-coupled verifier and not validation of a stronger standalone checkpoint-proof backend.
5. The same stage-6 utilities also define `verify_pkg_contract(...)` as a current-stack tx public spend verification step, reinforcing that the checkpoint lane reuses the persisted tx-package contract rather than inventing a new backend proof authority.
6. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves that tampered exec-input rows, tampered stage-4 package proof bytes, and tampered tx-package digest all fail stage 11 instead of being accepted.
7. The same test file proves the strongest artifact consequence for exec-input drift: after failure there is no emitted `checkpoint_s7.json`, and no post-tx checkpoint draft directory.
8. The same checkpoint test suite also proves that replayed exec rows in post-tx storage are rejected on link reload, so the current continuity contract keeps replay-style compatibility drift from silently becoming accepted state.
9. `.planning/phases/032-crypto-audit-scenario-1/032-05-SUMMARY.md` states the same honest boundary in prose: accepted checkpoint apply is now tied to the persisted stage-4 package contract and stage-6 canonical bridge outputs, not to placeholder success or compatibility-looking proof bytes.
10. `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md` keeps the same caveat: the current tree proves a narrower review-backed current-stack boundary and must not be upgraded into a broader backend-proof closure claim.

**Reasoning:**

- The repository defeats the optimistic but false reading that later stages still treat “proof-looking bytes” as authoritative enough on their own. The accepted handoff path now compares those bytes against the persisted stage-4 package and canonical stage-6 bridge artifacts before any accepted checkpoint draft is created.
- That is real continuity of the received spend package, because the handoff is not free-floating. It is checked against the specific package proof, the specific input refs, and the specific canonical bridge outputs that upstream stages persisted.
- But the boundary is intentionally narrower than a standalone checkpoint-proof backend. Both code comments and phase summaries say so directly. The verifier proves consistency of the received package-coupled contract; it does not turn the checkpoint layer into an independent authoritative proof backend.
- The strongest honest answer therefore has two halves: later stages no longer accept compatibility-looking proof bytes as sufficient, but they still rely on a narrower package-coupled continuity contract rather than a stronger backend-proof system.

**Gap Or Blocker:** There is no blocker for this narrow conclusion. The blocker exists only against stronger wording: the repository itself documents that the checkpoint proof payload remains compatibility-oriented and that the accepted draft depends on caller-supplied verifier/trust hooks, so it cannot be described honestly as a standalone authoritative checkpoint-proof backend.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: the continuity proven here is continuity of the received package/bridge contract, not independent checkpoint-backend authority.

### 20. Real Protection Against The Operator Boundary

🔴 **Quest:** At checkpoint time, what is the real protection against an operator or aggregator in the current implementation: a fully authoritative proof backend, or a narrower package-coupled contract that must not be described as stronger than it is?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** At checkpoint time the real protection is not a fully authoritative standalone proof backend. It is a narrower package-coupled anti-substitution contract tied to the accepted stage-4 package and stage-6 bridge material. That protection is still meaningful: an operator or aggregator in the accepted current flow can no longer silently substitute exec proof bytes, canonical input refs, canonical bridge outputs, or crudely tampered persisted stage-4 package proof/digest and still reach draft creation or accepted checkpoint publication. But the repository is explicit that this must not be described more strongly than it is, because backend authority remains external and verifier-bound rather than self-sufficient inside the checkpoint artifact layer.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` verifies the stage-7 handoff before `build_cp_draft(...)`, so checkpoint draft creation is gated by upstream contract revalidation.
2. The same handoff logic explicitly rejects proof-byte drift, input-ref drift, and bridge-output drift against the received stage-4 package and stage-6 bridge contract.
3. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` reuses the current-stack tx public spend verifier on the stage-4 package, so later checkpoint logic is not trusting unchecked package bytes.
4. The same file explicitly comments that this is a current-stack package-coupled verifier and not validation of a stronger standalone checkpoint-proof backend.
5. `crates/z00z_storage/src/checkpoint/build.rs` documents the proof-verifier boundary as caller-supplied and states that `build_cp_draft(...)` does not by itself upgrade the draft into a self-sufficient proof artifact.
6. `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs` and `artifact_final.rs` describe `cp_proof` as verifier-bound compatibility payload bytes rather than standalone checkpoint identity or authority.
7. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves that tampered exec rows, tampered package proof bytes, and tampered package digest all fail closed instead of reaching accepted checkpoint emission.
8. The same test suite proves the strongest artifact-side consequence for exec-row tamper: no emitted `checkpoint_s7.json` and no post-tx checkpoint draft directory.
9. The same checkpoint test suite also proves replay-style post-tx exec-row tamper is rejected on link reload as `ReplayMix`.
10. `.planning/phases/032-crypto-audit-scenario-1/032-05-SUMMARY.md` and `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md` already preserve the same honest language: the accepted protection is real, but narrower than a broader backend-proof claim.

**Reasoning:**

- The current implementation does protect against a meaningful operator/aggregator class of abuse: silent substitution of package-coupled execution material in the accepted handoff path.
- It does that by forcing continuity checks between the accepted upstream package contract and the downstream checkpoint execution material before draft creation. So “compatibility-looking” proof bytes alone are no longer enough.
- But the protection remains narrower than backend authority. The storage layer itself says the verifier is external and caller-supplied, and the proof bytes inside checkpoint artifacts remain compatibility payloads rather than self-sufficient authority objects.
- The false optimistic reading would be: because stage 11 now rejects tampered handoff material, the checkpoint layer has become an authoritative proof backend. The repository defeats that reading directly in code comments and artifact contracts.

**Gap Or Blocker:** There is no blocker for the narrow conclusion. The blocker exists only against stronger wording: the repository does not prove actor-specific impossibility beyond the accepted package-coupled flow, and it explicitly denies that checkpoint artifacts alone constitute a standalone authoritative proof backend.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: the protection proven here is anti-substitution inside the accepted current flow, not a general backend-authority proof against every operator-controlled environment.

## 🔔 Theme 5: Replay Safety, Secret Hygiene, And Documentation Honesty

### 21. Replay And Stale-Artifact Closure

🔴 **Quest:** Which replay or stale-artifact scenarios are demonstrably closed across claim, spend, and checkpoint flows, and which ones are still bounded only by narrower helper-owned or compatibility-shaped contracts rather than by a final authoritative proof system?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The current tree does close several concrete replay or stale-artifact classes in accepted Scenario 1 flows, but it closes them only at the narrower boundaries the repository honestly documents. In the claim path, Stage 3 write and publish reject nullifier replay, and consumer load rejects stale or mismatched source-root or proof artifacts, wrong authority anchors, and wrong authority signatures. In the spend path, the current public spend contract rejects replay-style `prev_root` drift. In the checkpoint path, Stage 11 and post-tx reload reject tampered exec rows, tampered package proof or digest continuity, and replay-style exec-row reload drift. What the repository does not prove is a final authoritative proof system for all three surfaces: claim stale-proof rejection is still helper-owned rather than persisted-continuity-backed, spend replay closure is still only the current-stack contract and not the broader original nullifier-semantics closure, and checkpoint replay closure is still package-coupled compatibility handling rather than a standalone proof backend.

**Evidence Trail:**

1. `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs` proves Stage 3 write and publish reject repeated claim packages with `claim nullifier replay rejected`, and `crates/z00z_simulator/src/claim_pkg_store.rs` plus `crates/z00z_simulator/src/claim_pkg_consumer.rs` enforce the same nullifier-replay failure at reservation and publish time.
2. `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` proves consumer-side rejection of stale storage proof reuse, wrong authority anchor, and wrong authority signature, while `claim_pkg_consumer.rs` rejects source-root and proof-blob drift against the canonical claim-source contract.
3. `crates/z00z_storage/src/assets/store_internal/store_query.rs` shows that the current claim-source contract still comes from `AssetStore::claim_source_contract_for_item(...)`, which rebuilds a synthetic one-item off-store contract instead of proving persisted storage-backed continuity.
4. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` proves the current public spend verifier rejects replayed `prev_root`, and `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` proves accepted Scenario 1 stage-4 packages are checked by that same wallet verifier.
5. `032-HONEST-CLOSEOUT.md`, `032-VALIDATION.md`, and `032-VERIFICATION.md` keep `PH32-SPEND` partially open because the regular public spend statement still lacks the original nullifier-semantics portion of the broader requirement.
6. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves Stage 11 rejects tampered exec-input rows, tampered package proof bytes, tampered package digest continuity, and replayed exec-input rows on post-tx link reload as `ReplayMix`.
7. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` shows `CheckpointReplaySpentIndex` and `CheckpointPackageProofVerifier` fail closed on replay-style spent-state or package drift, while the same file explicitly says this is a current-stack package-coupled verifier rather than a stronger standalone checkpoint proof backend.

**Reasoning:**

- The question is not whether Phase 032 fully closes all original trust claims. It asks which replay or stale-artifact scenarios are actually closed now. The repository answers that directly with concrete negative tests and fail-closed code paths in all three surfaces.
- Claim replay closure is real for the accepted current flow: repeated nullifiers are rejected before Stage 3 rewrite or publish can succeed, and stale or mismatched claim proof artifacts are rejected on consumer load. But the external truth source for those stale-proof checks is still the helper-reconstructed one-item contract, not persisted continuity.
- Spend replay closure is also real but narrow. The current public spend verifier rejects replay-style `prev_root` drift, which is enough to close that specific stale-artifact class at the accepted current-stack boundary. The repository itself defeats the false stronger reading by leaving the broader nullifier-semantics portion of `PH32-SPEND` open.
- Checkpoint replay closure is real for accepted handoff and reload drift. Stage 11 no longer accepts stale or tampered package-coupled execution material, and post-tx reload rejects replay-mixed exec rows. But the code and phase artifacts explicitly say this is package-coupled compatibility verification, not a final authoritative checkpoint-proof system.
- The tempting but false reading would be: because replay-like failures now reject in claim, spend, and checkpoint flows, the repository has fully authoritative anti-replay proof closure everywhere. The live code and closeout artifacts reject that upgrade. They preserve helper-owned, current-stack, and package-coupled caveats as part of the current truth.

**Gap Or Blocker:** None. The repository is strong enough to classify the replay and stale-artifact closures honestly; the narrower helper-owned and compatibility-shaped limits are part of the proved answer rather than blockers to answering it.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: the closed replay classes are proven only inside the current helper-owned claim contract, current-stack spend contract, and package-coupled checkpoint contract, not as a final authoritative proof system.

### 22. Default Secret-Export Discipline

🔴 **Quest:** What prevents default Scenario 1 execution from publishing plaintext wallet recovery material, and where is the remaining secret-export path kept explicitly outside the normal output lane?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The repository fully proves the narrower claim that default Scenario 1 execution does not publish the plaintext stage-2 wallet-recovery markdown artifact. That behavior is deny-by-default because `z00z_simulator` has no default features, `ScenarioCfg::stage2_secret_artifact_enabled()` resolves to `cfg!(feature = "wallet_debug_dump")`, `ScenarioCfg::stage2_secret_artifact_path(...)` returns a path only when that feature is enabled, and `stage_2.rs` writes the plaintext secret table only when that gated private path exists. When the feature is absent, stage 2 explicitly logs that no plaintext wallet secret artifact was emitted on the default lane. When the feature is enabled, the surviving plaintext debug artifact is kept outside the normal wallet output lane at `wallets/private/wlt_secrets_debug.md`, written through the private-write path and regression-tested to stay off the old public path `wallets/wlt_secrets_debug.md` with mode `0600`.

**Evidence Trail:**

1. `crates/z00z_simulator/Cargo.toml` sets `default = []` and marks `wallet_debug_dump` as a debug-only feature that must never be enabled in production builds.
2. `crates/z00z_simulator/src/config_accessors.rs` makes the gate explicit: `stage2_secret_artifact_enabled()` returns `cfg!(feature = "wallet_debug_dump")` and `stage2_secret_artifact_path(...)` returns `wallets/private/wlt_secrets_debug.md` only when that gate is on.
3. `crates/z00z_simulator/src/scenario_1/stage_2.rs` obtains `secrets_table` through that accessor and calls `debug_write_wallet_secrets_md(...)` only when the `Option<PathBuf>` is present.
4. The same `stage_2.rs` branch logs `wallet_debug_dump disabled; default lane emitted no plaintext wallet secret artifact` when the feature gate is absent, which is the runtime contract for the safe default path.
5. `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` shows exactly what the gated artifact contains: passwords, seed phrases, receiver secret hex, and other recovery material written into `# Wallet Secrets (Stage 2) [DEBUG]`.
6. The same writer uses `atomic_write_file_private(...)`, and `crates/z00z_utils/src/io/atomic_write.rs` enforces Unix mode `0600` for that sensitive write path.
7. `crates/z00z_simulator/tests/test_stage2_secret_artifacts.rs` proves the old public path `wallets/wlt_secrets_debug.md` must never exist.
8. The same test file proves that when `wallet_debug_dump` is enabled, the artifact must exist only at `wallets/private/wlt_secrets_debug.md`, must carry the explicit debug banner, and must remain mode `0600`.
9. The same test file also proves that without `wallet_debug_dump`, even the private debug artifact must stay absent.
10. `crates/z00z_simulator/tests/test_wallet_integration.rs` repeats the same runtime guarantees: no public plaintext artifact on the default lane, private-lane-only artifact when `wallet_debug_dump` is enabled, and no leakage of passwords or seed phrases into `rpc_logger.json`.
11. `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` and the scenario design contract encode this as part of the public stage contract: `Keep wallet-secret debug output behind an explicit debug-only private lane` and `Default stage-2 outputs contain no plaintext wallet secret artifact`.
12. `crates/z00z_simulator/README.md`, `032-06-SUMMARY.md`, `032-HONEST-CLOSEOUT.md`, and `docs/code-review/032-scenario-1-crypto-status.md` all repeat the same narrowed policy: default public Scenario 1 output excludes plaintext wallet-secret artifacts, and the retained debug artifact is private-lane only.
13. `crates/z00z_simulator/src/scenario_1/stage_2_utils/checks.rs` still persists `export_wallet_encrypted_payload.json`, and `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs` still configures encrypted backups, which is why the honest claim must stay scoped to the plaintext debug artifact lane rather than all possible operational secret-bearing persistence surfaces.

**Reasoning:**

- The repository does not rely on convention alone. The default-safe behavior is implemented as a compile-time feature gate, a config accessor that returns no secret-artifact path by default, and a runtime branch that never writes the plaintext markdown unless that gated path exists.
- The surviving plaintext export path is intentionally segregated from the normal output lane. It lives under `wallets/private/`, uses a private atomic writer, and is guarded by tests that reject both the old public path and the wrong file mode.
- That is enough to prove the question's narrower claim: default Scenario 1 execution no longer publishes plaintext wallet recovery material on the normal public lane, and the remaining plaintext debug artifact is explicit, private, and non-default.
- It is not enough to claim that every other operational persistence surface disappeared. The same stage still writes encrypted export and backup artifacts, so the honest closure is about the plaintext debug markdown lane, not about abolishing all secret-bearing persistence in every form.

**Gap Or Blocker:** None for the question as asked. The only caveat is scope: this answer proves the retained plaintext debug secret-artifact lane is explicit, private, and non-default; it does not prove that all other operational secret-bearing persistence lanes are absent.

**Verification:**

- `doublecheck` status: VERIFIED after narrowing the claim to the plaintext debug artifact lane
- Residual caveat: encrypted export/import payloads and encrypted backup flows still exist, so the answer must stay scoped to the absence of default plaintext wallet-recovery publication on the normal lane

### 23. Deterministic Randomness Boundaries

🔴 **Quest:** How does the repository bound deterministic randomness so that reproducibility fixtures remain simulator-only and do not become accidental evidence of production-grade entropy?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The strongest repository-backed conclusion is narrower than a repo-wide entropy theorem. Phase 032 does prove that the seeded `SeqSecureRngProvider` path used for Scenario 1 stage-2 wallet-service construction is confined to simulator-only helpers, is entered only when `use_mock_rng` is true in that stage-2 transport path, and is explicitly documented as a reproducibility fixture rather than a production entropy guarantee. In that audited boundary, the false branch uses `WalletService::with_output_dir(...)`, which resolves to `SystemTimeProvider` plus default entropy from `SystemRngProvider`, while the true branch injects `MockTimeProvider` plus `SeqSecureRngProvider` from the simulator seed. The repository also repeatedly refuses to market that seeded path as production entropy. But the answer must remain partial because the repository does not establish one single global selector rule for every simulator stage: stage 3 has a separate seed-selection seam, and inside the stage-2 mock branch `mock_rng_seed = None` still falls back to seed `0` rather than secure randomness. So the honest closure is that the audited stage-2 reproducibility fixture is simulator-only and non-authoritative for production entropy claims, not that the whole repository has one perfectly unified deterministic-randomness boundary.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` defines `SeqSecureRngProvider` locally inside the simulator and labels the transport helpers `SIMULATOR-ONLY: DO NOT MOVE TO CORE`.
2. The same file enters the seeded stage-2 path only when `ctx.config.simulation.use_mock_rng` is true; otherwise it uses `WalletService::with_output_dir(...)`.
3. Inside that seeded branch, stage 2 constructs `MockTimeProvider` and `SeqSecureRngProvider` from `ctx.config.simulation.mock_rng_seed.unwrap_or(0)`, which is the exact deterministic fixture seam audited in this phase.
4. `crates/z00z_wallets/src/services/wallet_service_session_build.rs` shows that `WalletService::with_output_dir(...)` uses `SystemTimeProvider` on the non-mock path.
5. `crates/z00z_wallets/src/services/wallet_service_actions_hardening.rs` shows that the default wallet entropy path comes from `WalletEntropyFromRngProvider::new(SystemRngProvider)`.
6. `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs` injects deterministic seed phrases only when `use_mock_rng` is true and otherwise leaves the wallet creation request without a deterministic seed phrase.
7. The same `flow.rs` file uses `mock_rng_seed.unwrap_or(0)` for stage-2 mock timestamps and `SystemTimeProvider.compat_unix_timestamp_millis()` when `use_mock_rng` is false, so both time and entropy fixture behavior are bounded to the mock path.
8. `crates/z00z_simulator/tests/test_transport_rng_boundaries.rs` proves that the same stage-2 mock seed reproduces wallet ids, different seeds change wallet ids, and `mock_rng_seed = None` inside the mock path matches explicit zero seed.
9. `crates/z00z_simulator/src/config.rs` documents `use_mock_rng` as deterministic RNG for CI, which is the public config surface for the simulator fixture path.
10. `crates/z00z_simulator/src/scenario_1/stage_3.rs` proves the repository does not use one single randomness gate everywhere: stage 3 has its own selector `stage3_claim.rng_seed > simulation.mock_rng_seed > System`.
11. `crates/z00z_utils/src/rng/mock.rs` adds supporting evidence for the repository's general posture by compile-failing `MockRngProvider` in production-like builds outside test, debug, or explicit test features.
12. `032-06-PLAN.md`, `032-06-SUMMARY.md`, `032-VALIDATION.md`, `032-HONEST-CLOSEOUT.md`, and `docs/code-review/032-scenario-1-crypto-status.md` all describe seeded RNG as simulator-only fixture behavior or a reproducibility fixture and explicitly refuse to treat it as a production entropy guarantee.

**Reasoning:**

- The audited stage-2 boundary is real. Seeded transport reproducibility is not ambient randomness leaking into normal execution; it is an explicitly simulator-owned path entered through `use_mock_rng`, with simulator-only comments, reproducibility tests, and closeout language that refuses stronger security claims.
- The non-mock stage-2 path is also real. It falls back to the wallet service's normal system time and system entropy path rather than silently inheriting the deterministic fixture provider.
- The boundary is not globally uniform enough to support a stronger theorem. Stage 3 selects randomness through a separate seam, and the stage-2 mock branch treats absent seed as zero seed, not as secure randomness. That mismatch is exactly why the answer must stop at partial evidence.
- The honest takeaway is therefore limited but meaningful: the repository has bounded the audited stage-2 deterministic fixture tightly enough that it should not be cited as evidence of production-grade entropy, yet it still has enough selector variation across stages that broader repo-wide closure would overclaim.

**Gap Or Blocker:** The remaining blocker is unification and documentation precision. The repository still lacks one single randomness-boundary contract that all simulator stages follow, and some generic config comments are broader than the exact stage-2 runtime behavior where `use_mock_rng = true` plus `mock_rng_seed = None` still means deterministic zero-seed fallback.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: the verified claim is about the audited stage-2 simulator fixture boundary, not a universal repo-wide guarantee that `None` always means secure randomness or that `use_mock_rng` is the sole selector everywhere

### 24. Honest Status Language Across Artifacts

🔴 **Quest:** If logs, verification manifests, and closeout notes are audited together, where do they explicitly refuse to claim stronger verifier, proof-backend, censorship-resistance, or checkpoint-authority properties than the live code can prove?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository does explicitly refuse stronger claims across multiple artifact layers, but the refusals do not all live in the same kind of artifact. The clearest anti-overclaim language lives in `032-HONEST-CLOSEOUT.md`, `032-VERIFICATION.md`, `032-VALIDATION.md`, and `docs/code-review/032-scenario-1-crypto-status.md`. Together, those files explicitly reject live STARK/FRI claims, broader trustless-verifier claims, stronger checkpoint-authority language, censorship-resistance or withheld-data guarantees, and full closure of the broader `PH32-SPEND` and `PH32-CLAIM-TRUST` requirements. The logs and manifest files then reinforce that boundary indirectly by showing narrower execution evidence, mixed or blocked broad-suite results, and review notes that say the verification artifact is narrower than full phase closure. So the repository does not merely omit stronger claims; it actively instructs reviewers not to make them. The reason the answer remains partial is that the semantic refusal layer sits mostly in closeout, verification, validation, and status-note artifacts, while manifests and raw logs function mainly as evidentiary limiters rather than as standalone disclaimer documents.

**Evidence Trail:**

1. `032-HONEST-CLOSEOUT.md` contains a dedicated `What Scenario 1 Does Not Prove` section that explicitly rejects live STARK/FRI, broader trustless public verifier claims, general-purpose or on-chain verifier claims, censorship-resistance and withheld-data guarantees, broader `PH32-SPEND` closure, broader `PH32-CLAIM-TRUST` closure, and broader production-entropy claims.
2. The same closeout file has a separate `Explicit Out Of Scope Items` section naming final proof-backend selection, recursive or succinct checkpoint verification, and trustless publication or withheld-data recovery guarantees as out of scope.
3. The same closeout file's final status section repeats that the phase cannot yet be treated as fully closed against the original `PH32-SPEND` and `PH32-CLAIM-TRUST` wording and explicitly says it does not prove a final trustless ZK architecture, live STARK/FRI support, or withheld-data trustlessness.
4. The closeout ends with an explicit preservation rule: future summaries and docs must keep these `does not prove` and `out of scope` statements unless later code and tests invalidate them.
5. `032-VERIFICATION.md` says at the top that it is intentionally narrower than a full phase closeout and must not be used to claim closure of the missing nullifier-semantics portion of `PH32-SPEND` or the persisted storage-backed continuity portion of `PH32-CLAIM-TRUST`.
6. The same verification artifact later says that the verification artifact is narrower than full phase closure and does not close `PH32-SPEND`, which keeps the review loop from overpromoting test evidence into broader semantic closure.
7. `032-VALIDATION.md` turns this into a formal requirement by marking `PH32-HONEST` green only if status docs and closeout language do not overclaim STARK/FRI, trustless verification, stronger checkpoint authority, or closed spend and claim-trust semantics.
8. The same validation file keeps `PH32-CLAIM-TRUST` and `PH32-SPEND` marked partial or open, which blocks honest sign-off if later summaries pretend those gaps are gone.
9. `docs/code-review/032-scenario-1-crypto-status.md` has a dedicated `What This Status Does Not Claim` section rejecting live STARK/FRI, recursive checkpoint-proof, universal trustless public spend verification, whole-chain/on-chain verifier deployment, censorship resistance, full `PH32-SPEND` closure, and full `PH32-CLAIM-TRUST` closure.
10. The same status note also has an `Out Of Scope` section that rejects final proof-backend selection, trustless publication or data-availability guarantees, and production-grade entropy claims derived from simulator-only seeded RNG fixtures.
11. `032-VERIFICATION.md` records that broader workspace reruns remained mixed, stale, or blocked, including historical `RESULT[18]=FAIL` and a later host-disk-exhaustion blocker, so manifest-backed evidence itself is narrower than any claim of comprehensive clean closeout.
12. `032-HONEST-CLOSEOUT.md` explicitly says supporting long-running-suite evidence is supporting evidence only and does not replace the required honest closeout order, which prevents raw logs or manifests from being misused as proof of stronger closure than the narrative artifacts allow.

**Reasoning:**

- The repository has an explicit semantic disclaimer layer. That layer lives in closeout, verification, validation, and the public-facing status note, where the files directly name what Scenario 1 does not prove and what remains out of scope.
- The repository also has an evidentiary-limiter layer. Verification manifests and logs do not usually contain the anti-overclaim prose themselves, but they keep the review honest by showing that the executed evidence is narrower, mixed, or blocked in places where an overeager summary might otherwise pretend a broader clean closeout.
- Audited together, those layers work as a combined honesty system: narrative artifacts forbid stronger claims, and execution artifacts keep the repository from quietly upgrading partial evidence into final semantic closure.
- The answer remains partial only because the explicit refusals are concentrated in the narrative/control artifacts rather than every log or manifest file individually. The overall anti-overclaim system is real, but it is not evenly expressed in every artifact class.

**Gap Or Blocker:** The main limitation is distribution, not absence. Explicit disclaimer language is strong in the closeout, verification, validation, and status-note artifacts, but raw manifests and logs mostly act as narrower evidence boundaries rather than carrying standalone semantic disclaimer text.

**Verification:**

- `doublecheck` status: VERIFIED after narrowing the claim so that manifests and logs are treated as evidentiary limiters rather than standalone disclaimer sources
- Residual caveat: the repository clearly preserves anti-overclaim language, but that language is strongest in the narrative/control artifacts and only indirectly reinforced by raw execution logs and manifests

### 25. What May Stay In Documentation

🔴 **Quest:** After a full repository-backed review of this phase, which surviving statements about Scenario 1 deserve to stay in project-facing documentation as present truth, and which ones must still be framed as open gaps, narrow current-stack truth, or future work?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository supports a three-bucket documentation policy, but only if the middle bucket is phrased as qualified current-stack truth rather than closure. First, some statements may stay in project-facing documentation as present truth because the closeout, status note, and validation map all treat them as delivered behavior. Second, some statements may stay only as narrow current-stack truth with explicit caveats, because the repository proves a stronger boundary than the pre-032 placeholder state but still keeps the broader original requirements open. Third, several claims must remain framed as open gaps or future work and must not be promoted into present truth. The controlling rule is explicit: future summaries, docs, and review notes must preserve the repository's `does not prove` and `out of scope` statements unless later code and tests invalidate them.

**Evidence Trail:**

1. `032-HONEST-CLOSEOUT.md` has a `What Scenario 1 Now Proves` section that defines the delivered present-truth set for current accepted Scenario 1 flows.
2. The same closeout file has `What Scenario 1 Does Not Prove`, `Explicit Out Of Scope Items`, and `Final Honest Status` sections that define the claims that must remain qualified or excluded from present truth.
3. The closeout ends with an explicit preservation rule: future summaries, docs, or review notes must keep these `does not prove` and `out of scope` statements unless later code and tests invalidate them.
4. `docs/code-review/032-scenario-1-crypto-status.md` mirrors the same split through `Delivered In The Current Tree`, `What This Status Does Not Claim`, and `Out Of Scope`, which makes the policy project-facing rather than private to the planning directory.
5. `032-VALIDATION.md` formalizes the same split in the per-task verification map: `PH32-CHECKPOINT`, `PH32-SECRET`, and `PH32-HONEST` are green, while `PH32-CLAIM-TRUST` and `PH32-SPEND` remain partial or open.
6. The same validation artifact explicitly says `PH32-CLAIM-TRUST` remains open until persisted storage-backed continuity exists and `PH32-SPEND` remains open until nullifier semantics are carried and validated or the requirement is formally narrowed.
7. `032-VERIFICATION.md` says it must not be used to claim closure of the missing nullifier-semantics portion of `PH32-SPEND` or the persisted storage-backed continuity portion of `PH32-CLAIM-TRUST`, which blocks later documentation from turning narrower test evidence into broader semantic closure.
8. The already-solved exam questions Q1-Q24 line up with the same split: some claims are fully proven for the current boundary, some are proven only as helper-owned or current-stack truth, and some remain explicit open gaps or future-facing work.

**Reasoning:**

- The documentation-safe present-truth bucket is the set of statements the repository itself already publishes as delivered without contradiction.
- The qualified-current-stack bucket is the set of statements where the current tree genuinely proves something stronger than the old placeholder state, but the closeout and validation artifacts still refuse to call the broader original requirement closed.
- The future-work or open-gap bucket is anything the repository names under `does not prove`, `out of scope`, or still-partial requirement language. Promoting those items into present truth would directly violate the repository's own honest-status contract.

**Documentation Buckets:**

**Present truth that may stay as present truth:**

- Claim packages bind the accepted claim statement to the canonical source-root contract consumed by downstream verification.
- Accepted stage-4 spend flow verifies a persisted current-stack public spend contract instead of relying on structural witness success alone.
- Accepted checkpoint apply rejects placeholder proof and placeholder spent-state success lanes before checkpoint artifact emission.
- Default stage-2 runs do not emit a public plaintext wallet-secret artifact.

**Narrow current-stack truth that may stay only if explicitly qualified:**

- The current tree proves helper-owned canonical claim-source consistency and forged or stale rejection, but not persisted storage-backed continuity for the broader `PH32-CLAIM-TRUST` wording.
- The current tree proves the current-stack public spend boundary and persisted spend proof/auth contract, but not the broader nullifier-semantics portion of `PH32-SPEND`.
- Checkpoint protection may be described as package-coupled anti-substitution and anti-placeholder acceptance, not as a standalone backend or final trustless proof system.
- Receiver-secret and wallet-local ownership claims must stay scoped to the current accepted wallet-local boundary, not written as universal trustless theorems.
- The remaining seeded stage-2 transport RNG path may stay in documentation only as a simulator-only reproducibility fixture, not as production-grade entropy.

**Open gaps or future work that must not be left as present truth:**

- Live STARK support, live FRI support, recursive checkpoint proof systems, or equivalent final proof-backend commitments.
- End-to-end trustless public verifier claims beyond the current accepted boundary.
- General-purpose or whole-chain on-chain verifier deployment claims.
- Censorship resistance, withheld-data recovery, or trustless publication guarantees.
- Full closure of the broader original `PH32-SPEND` wording.
- Full closure of the broader original `PH32-CLAIM-TRUST` wording.
- Production-grade entropy guarantees derived from simulator-only seeded RNG fixtures.

**Gap Or Blocker:** The blocker is not missing language but required discipline. Bucket B must stay explicitly qualified, and Bucket C must remain excluded from present truth until later code and tests close the corresponding gaps or the original requirements are formally narrowed and re-approved.

**Verification:**

- `doublecheck` status: VERIFIED after correcting Bucket B so that claim-trust and spend statements remain qualified current-stack truth instead of being described as closed
- Residual caveat: the three-bucket model is repository-backed, but only if future docs preserve the closeout rule that `does not prove` and `out of scope` statements remain mandatory until superseded by new code and tests

## Summary Table

| Q | Title | Proof Status | Verification | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- |
| 1 | Delivered Closure Versus Open Closure | Partial Evidence | VERIFIED | Broader `PH32-CLAIM-TRUST` and `PH32-SPEND` remain open | Implement persisted claim continuity and spend nullifier semantics, or formally narrow the original requirements |
| 2 | Current-Stack Truth Versus Future-Proof Ambition | Full Evidence | VERIFIED | None | None |
| 3 | Blocked Remediation Discipline | Full Evidence | VERIFIED | None | None |
| 4 | Planning Truth Versus Implementation Truth | Partial Evidence | VERIFIED | Wording drift remains in older planning artifacts | Final corrected artifact set is authoritative over earlier optimistic phrasing |
| 5 | Conditions For Honest Reclassification | Partial Evidence | VERIFIED | Two requirement-level semantic gaps remain open | Implement and re-verify them, or formally narrow and re-approve the original wording |
| 6 | Full Authenticated Claim Tuple | Partial Evidence | VERIFIED | Not every field has a dedicated direct signature-mutation test | Canonical byte-frame signature contract still covers the full tuple |
| 7 | Tuple Drift Under Plausible Package Shape | Partial Evidence | VERIFIED | Recipient-binding drift coverage is narrower than root and proof drift coverage | Add broader direct mutation tests for recipient-binding and asset-path variants |
| 8 | Self-Consistency Versus Authority | Partial Evidence | VERIFIED | Persisted storage-backed continuity is still not proven | Replace the helper-derived seam with persisted continuity or formally narrow the requirement |
| 9 | Distinct Claim Reject Paths | Full Evidence | VERIFIED | Distinctness is category-level and seam-message-level, not one unique top-level label per mutation | Keep the verifier fail-closed ordering and, if needed, add stricter exact-message tests for stale-proof variants |
| 10 | The Seam That Keeps Claim Trust Partial | Full Evidence | VERIFIED | Persisted storage-backed claim continuity is still absent behind the helper seam | Replace the helper-owned seam with persisted continuity, or formally narrow and re-approve the broader claim-trust requirement |
| 11 | Sender Knowledge And The Narrower Anti-Theft Rule | Full Evidence | VERIFIED | The anti-theft rule is wallet-local, not a public theorem | Keep `sender ignorance` language forbidden and describe the receiver-secret gate as the narrower honest rule |
| 12 | Receiver-Held Secret As The Ownership Gate | Full Evidence | VERIFIED | The ownership gate is proven only for accepted wallet-local paths | Preserve `wallet-local ownership rule` wording until a public verifier proves the same invariant end to end |
| 13 | Canonical Decrypt-Associated Asset Binding | Full Evidence | VERIFIED | The strongest proof is limited to shipped wallet, scan, report, and spend-witness bridge paths | Keep `leaf_ad_id` as the canonical decrypt boundary and reject drift across those accepted paths |
| 14 | Request-Bound Route Versus Card-Bound Route | Full Evidence | VERIFIED | The privacy consequence is implied from route-binding failure, not separately proved as its own theorem | Keep request-bound and card-bound tag contexts and approval rules distinct |
| 15 | Exclusivity After Scan | Partial Evidence | VERIFIED | Exclusivity is wallet-local and not yet a public trustless guarantee | Keep post-scan handling direct and preserve the receiver-secret plus `s_out` ownership gate without overclaiming public exclusivity |
| 16 | What The Spend Boundary Actually Proves | Partial Evidence | VERIFIED | Current persisted public spend contract proved; authoritative input membership continuity, nullifier semantics, and stronger standalone backend remain outside this boundary | Keep Q17 scoped to fail-closed semantic acceptance before state mutation |
| 17 | Structural Plausibility Versus Semantic Acceptance | Partial Evidence | VERIFIED | No accepted tx-package or checkpoint-persistence lane remains for semantically incomplete spend artifacts; some earlier stage4 file writes still happen and some negative branches are code-proven more strongly than test-proven | Keep Q18 focused on the exact missing public-input element in the regular spend statement |
| 18 | The Missing Spend-Statement Element | Full Evidence | VERIFIED | The regular spend contract is missing a nullifier field / nullifier semantics surface, so the delivered public verifier remains narrower than original `PH32-SPEND` | Move Q19 to checkpoint continuity versus compatibility-looking proof bytes |
| 19 | Checkpoint Continuity Or Compatibility-Looking Proof Bytes | Full Evidence | VERIFIED | Stage11 enforces package-coupled continuity of stage4 proof, input refs, and stage6 bridge outputs; compatibility-looking proof bytes alone are not accepted, but this is still not a standalone backend | Use Q20 to state the exact operator-boundary protection without overstating backend authority |
| 20 | Real Protection Against The Operator Boundary | Full Evidence | VERIFIED | Real protection is package-coupled anti-substitution at checkpoint time, not standalone backend authority | Theme 5 can now move to replay, secret-export, RNG, and documentation-honesty boundaries |
| 21 | Replay And Stale-Artifact Closure | Full Evidence | VERIFIED | Concrete replay/stale classes are closed, but only inside helper-owned claim, current-stack spend, and package-coupled checkpoint boundaries | Keep documentation narrow unless persisted claim continuity, spend nullifier semantics, and standalone checkpoint backend land |
| 22 | Default Secret-Export Discipline | Full Evidence | VERIFIED | The default-safe result is proven for the plaintext debug artifact lane; encrypted operational export and backup surfaces still exist outside this narrower claim | Keep documentation scoped to non-default plaintext export closure unless broader secret-bearing persistence policy is separately audited |
| 23 | Deterministic Randomness Boundaries | Partial Evidence | VERIFIED | Stage 2 fixture boundary is well-bounded, but the repository still lacks one unified randomness-selector rule across every simulator stage | Keep docs scoped to the audited stage-2 simulator fixture unless stage-3 and generic config semantics are unified |
| 24 | Honest Status Language Across Artifacts | Partial Evidence | VERIFIED | Anti-overclaim language is explicit and strong, but it is concentrated in closeout, verification, validation, and status-note artifacts rather than every log or manifest | Keep manifests/logs framed as evidentiary limiters and keep the semantic disclaimer burden on the narrative/control artifacts |
| 25 | What May Stay In Documentation | Partial Evidence | VERIFIED | The repository supports a stable three-bucket documentation policy, but the middle bucket must stay explicitly qualified because broader claim-trust and spend requirements remain open | Preserve the closeout preservation rule and forbid Bucket B from drifting into closure language until code/tests or formal narrowing changes the requirement state |
