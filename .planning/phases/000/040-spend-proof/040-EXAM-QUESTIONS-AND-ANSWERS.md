# Phase Final Exam

**Phase:** `040-spend-proof`
**Generated:** `2026-04-29`
**Scope Sources:** `.planning/phases/040-spend-proof` artifacts, live wallet spend-proof implementation, simulator runtime evidence, rollup public-artifact binding evidence, and Phase 040 audit ledger.

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

This exam pressure-tests whether Phase 040 really closes the canonical internal
regular-spend theorem relation it claims, while keeping stronger public proof,
checkpoint finality, and rollup settlement claims bounded to repository truth.

## ⛔ Constraints

- Treat every question as an implementation-truth audit, not a design-intent
  recall prompt.
- Do not accept an answer that relies only on planning prose when executable
  code, tests, or produced artifacts should exist.
- Do not convert deterministic public-artifact checks into a stronger public
  proof claim unless the repository evidence actually closes that proof.
- Keep the answer scope tied to Phase 040, including the explicit boundaries it
  refuses to close.

## Scope Note

This exam is meant to verify the live Phase 040 spend-proof boundary: canonical
carrier and statement discipline, internal theorem relation over public facts
and private witness data, fail-closed verifier behavior, simulator continuity,
rollup public-artifact binding, and honest closeout language.

## 🔍 Answering Standard

- Answers must discover their own evidence path through the repository.
- Questions should be solvable only from repository study, but should not name
  the exact helper, file, or test that resolves them unless that hint is itself
  part of the intended challenge.

## Theme 1: Theorem Boundary And Claim Truth

### 1. Closure Claim Classification

🔴 **Quest:** What exactly is the strongest proof claim Phase 040 is allowed to close, and what repository evidence separates that claim from a stronger public proof-of-knowledge claim?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Phase 040 may close only the canonical wallet and simulator internal spend theorem-relation claim. The live regular-spend producer validates `T(S, W)` before emitting one deterministic canonical artifact for `regular_spend_theorem_bpplus`, and the public verifier fail-closes on canonical statement, public relation, proof-byte, suite, and authorization drift. It may not claim a public or trustless proof-of-knowledge backend, because verifier-side acceptance recomputes deterministic artifact bytes from public statement data and checks public relations; it does not verify a cryptographic proof that the package author knows `ReceiverSecret`, ordered `s_in` values, or membership witnesses.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` freezes the live `SPEND_PROOF_SUITE` as `regular_spend_theorem_bpplus`.
2. `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs` separates proving from public verification: `prove(...)` calls `validate_witness(...)`, while `verify(...)` has no witness parameter and checks statement shape, public relations, decoded artifact fields, and deterministic `theorem_bytes` equality.
3. `crates/z00z_wallets/src/core/tx/spend_verification.rs` recomposes the public spend statement, checks public input and output relations, calls `backend.verify(...)`, and verifies spend authorization, while its boundary comment explicitly says the live contract is narrower than a finished full-ZK spend theorem.
4. `crates/z00z_wallets/tests/test_spend_proof_backend.rs` includes forged deterministic artifact tests for range, balance, and input/output overlap, showing that public verification must rely on public relation checks rather than witness knowledge.
5. `.planning/phases/040-spend-proof/040-VALIDATION.md`, `040-UAT.md`, and `040-CLOSEOUT-GATES.md` all keep public/trustless proof-of-knowledge, checkpoint theorem finality, and full rollup settlement proof closure as open boundaries.

**Reasoning:**

- The internal claim is closed because the producer side has access to `SpendProofWitness { receiver_secret, input_s_in, membership }` and rejects witness, membership, range, nullifier, and balance drift before `encode_artifact(...)` emits the canonical artifact.
- The public verifier is meaningful but narrower: it recomputes the canonical statement and validates public relations, then compares decoded artifact fields against deterministic hashes derived from `stmt.statement_hash()` and `stmt.public_hash()`.
- A stronger proof-of-knowledge reading fails because the verifier never receives or verifies a zero-knowledge proof over the private witness. The `forged_artifact(...)` test helper can construct syntactically matching deterministic artifact bytes from public statement material, and rejection is provided by public relation checks such as forged range, forged balance, and forged overlap tests.
- The historical `040-08-SUMMARY.md` and `040-09-SUMMARY.md` are useful only as contrast: they document retraction and the older statement-bound baseline, while current authority is the `040-10` internal theorem-relation boundary.

**Gap Or Blocker:** None for the internal theorem-relation classification. The stronger public/trustless proof-of-knowledge backend remains explicitly out of scope and would require a real verifier-side cryptographic proof system plus focused proof-of-knowledge tests.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None for the bounded internal claim; the answer intentionally does not upgrade it into public proof-of-knowledge closure.

### 2. Internal Relation Scope

🔴 **Quest:** Which public facts and private witness families must participate in the completed internal spend relation, and what would demonstrate that the implementation is accepting a weaker relation than the phase promised?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The completed internal spend relation is `T(S, W)` over a canonical public statement `S` and a producer-side private witness `W`. `S` includes the package digest, typed `prev_root`, chain scope and transaction version, ordered `SpendInputRef` values, ordered `SpendInputLeaf` public facts, ordered output theorem leaves, output range and balance public facts, and the regular-spend nullifier vector. `W` includes `ReceiverSecret`, ordered `s_in[i]` for each consumed input, and explicit membership witnesses against `prev_root`. These private witness and membership requirements are closed on the internal producer/prove path, not as a public proof-of-knowledge verifier.

**Evidence Trail:**

1. `.planning/phases/040-spend-proof/040-INTEGRITY-GATES.md` lists the public input inventory and private witness inventory for the `040-10` theorem relation.
2. `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs` implements the typed public statement in `SpendProofStmt` and the private witness in `SpendProofWitness`.
3. `CanonicalSpendProofBackend::prove(...)` calls `validate_witness(...)`; that path validates statement shape, membership witnesses, output ranges, and `verify_spend_rules(...)` before encoding the artifact.
4. `CanonicalSpendProofBackend::verify(...)` has no witness parameter; it checks public statement shape, public relations, decoded artifact fields, and deterministic theorem bytes.
5. `crates/z00z_wallets/tests/test_spend_proof_backend.rs` and `crates/z00z_wallets/tests/test_tx_proof_verifier.rs` cover statement-only, witness, membership, nullifier, range, balance, overlap, suite, input-pairing, and statement-drift rejections.

**Reasoning:**

- The public side of `S` is not just a raw transcript. `build_spend_proof_stmt(...)` constructs typed fields from transaction inputs, proof inputs, outputs, package digest, chain scope, and previous root; `SpendProofStmt::from_parts(...)` rejects mismatched input-ref, input-leaf, and nullifier lengths.
- The private side of `W` is intentionally producer-side: `ReceiverSecret` and ordered `s_in[i]` drive owner-tag, `leaf_ad_id`, and nullifier checks through `verify_spend_rules(...)`, while `SpendMembershipWitness` binds each carried input to `prev_root` by root, path, leaf, proof bytes, and proof item.
- A weaker producer relation would be visible if the backend accepted statement-only shapes, empty or mismatched witness vectors, membership witnesses whose root/path/leaf/proof item drifted from the carried public inputs, or witness values that failed owner-tag, `leaf_ad_id`, or nullifier derivation.
- A weaker public-verifier relation would be visible if forged deterministic artifacts could bypass public relation checks for output range proofs, balance, duplicate inputs or nullifiers, input/output theorem leaf overlap, input pairing, or canonical statement drift.
- The tempting but false interpretation is that public verification proves witness knowledge. It does not: public verification is a public-artifact and relation check, while witness knowledge is enforced before artifact production.

**Gap Or Blocker:** None for the internal relation scope. The gap remains exactly the stronger public/trustless proof-of-knowledge path, which would need verifier-side cryptographic witness-proof validation rather than deterministic artifact comparison.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: balance and range are public statement/output relation checks enforced during prove and verify; `ReceiverSecret + s_in` drive owner-tag, `leaf_ad_id`, and nullifier witness checks.

### 3. Canonical Suite Exclusivity

🔴 **Quest:** How can a solver prove that the live regular-spend path has converged on one semantic proof family rather than preserving a compatibility branch or semantic alias that could change verifier meaning?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The live executable regular-spend path has converged on one semantic proof family because the positive path uses only `SPEND_PROOF_SUITE = "regular_spend_theorem_bpplus"` with `CanonicalSpendProofBackend`, and every alternative suite is rejected rather than decoded through a fallback. A solver must separate live source and runtime surfaces from historical or negative-check documentation: older names may still appear in archived summaries or plan text as retraction, banned-term, or negative-search context, but the live wallet source, tests, simulator runtime artifact, and verifier seams do not preserve an executable compatibility branch.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` defines the only live suite constant as `regular_spend_theorem_bpplus`.
2. `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs` returns `CanonicalSpendProofBackend` from `default_spend_proof_backend()`, reports `SPEND_PROOF_SUITE` from `suite_id()`, embeds that suite during artifact encoding, and rejects any decoded artifact whose suite differs.
3. `crates/z00z_wallets/src/core/tx/spend_verification.rs` writes `SPEND_PROOF_SUITE` in `build_public_spend_contract(...)` and rejects `proof.proof_suite != SPEND_PROOF_SUITE` with `BadProofSuite` during public verification.
4. `crates/z00z_simulator/src/scenario_1/outputs/transactions/tx_alice_to_bob_pkg.json` carries `"proof_suite": "regular_spend_theorem_bpplus"` in the persisted simulator transaction package.
5. Live-surface searches over `crates/z00z_wallets/**` and `crates/z00z_simulator/**` found no executable `regular_spend_statement_bound_v1`, `StatementBoundSpendProofBackend`, `regular_spend_theorem_bpplus_v1`, or `theorem_v2` path.
6. Backend, decoder, statement, and public-verifier tests reject noncanonical or legacy suite drift.

**Reasoning:**

- Positive convergence is shown by one constant, one default backend, one suite ID in artifact bytes, and one suite string in runtime package output.
- Negative convergence is shown by absence of executable legacy branches plus explicit rejection of any artifact or proof wire that tries to carry a different semantic suite.
- The archived `040-08` and `040-09` documents do not defeat this claim because they are historical retraction and baseline evidence, not live executable authority.
- A false optimistic reading would treat any old phrase in planning files as an active compatibility path. That fails because live Rust source and runtime artifacts route through the canonical suite, and tests exercise noncanonical suite rejection rather than fallback acceptance.

**Gap Or Blocker:** None for live-path suite exclusivity. The only caveat is documentary: old suite names remain valid when they are explicitly historical, retracted, or banned-term context.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: No tests were rerun during doublecheck; the verification was repository read and grep backed.

### 4. Historical Baseline Discipline

🔴 **Quest:** How should an answer distinguish archived implementation checkpoints from current Phase 040 authority when deciding whether the phase is complete or only historically documented?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** An answer must treat archived implementation checkpoints as historical evidence, not current completion authority. `040-08-SUMMARY.md` is an audit-retraction artifact, and `040-09-SUMMARY.md` is the last completed historical baseline; neither defines the active closeout target after Phase 040 continued on `040-10-PLAN.md`. Current authority is live code and tests plus the active `040-10` ledgers: `040-Spend-Proof-Spec.md`, `040-INTEGRITY-GATES.md`, `040-CLOSEOUT-GATES.md`, `040-VALIDATION.md`, `040-UAT.md`, `040-CONTEXT.md`, and `040-TODO.md`. A phase-complete claim is valid only where those current sources and executable evidence agree.

**Evidence Trail:**

1. `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md` states that `040-09-SUMMARY.md` remains the last historical implementation baseline and that `040-10-PLAN.md` changes normative authority for current work.
2. `.planning/phases/040-spend-proof/040-CLOSEOUT-GATES.md` tracks only internal `040-10` closeout gates and explicitly says `040-09-SUMMARY.md` is archive-only baseline evidence, not the active closeout target.
3. `.planning/phases/040-spend-proof/040-VALIDATION.md` says active validation authority follows `040-10-PLAN.md` and keeps public proof-of-knowledge, checkpoint theorem finality, and rollup settlement proof closure open.
4. `.planning/phases/040-spend-proof/040-UAT.md` tracks the active `040-10` internal theorem-relation closure sweep rather than a public/checkpoint/rollup closeout.
5. `.planning/phases/040-spend-proof/040-CONTEXT.md` says `040-09-SUMMARY.md` remains a historical checkpoint while current context serves active `040-10` closure.
6. `.planning/phases/040-spend-proof/040-08-SUMMARY.md` marks older theorem-closeout language as historical overclaim, not current repository truth.
7. `.planning/STATE.md` and `.planning/ROADMAP.md` keep Phase 040 active on plan 10 with internal relation evidence green and broader public, checkpoint, and rollup proof boundaries open.

**Reasoning:**

- Historical checkpoints can prove what was implemented or retracted at their own point in time, but they cannot close a reopened phase against newer authority.
- If archived summaries and current ledgers disagree, the answer must follow live code, active validation, closeout gates, and top-level planning state, then name the disagreement as truth drift or historical contrast.
- A claim is only complete when current artifacts and executable evidence close it. If a claim appears only in archived summaries, or current validation lists it as open, it is historically documented rather than completed.
- This discipline prevents the `040-08` overclaim and the `040-09` statement-bound baseline from being accidentally promoted into the active `040-10` internal theorem-relation closeout.

**Gap Or Blocker:** None for authority classification. Any future upgrade from historical documentation to completed proof work must land in live code/tests and the current validation and closeout ledgers.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: `040-09-SUMMARY.md` was an authority artifact for its own historical moment, but current `STATE.md`, `ROADMAP.md`, and `040-10` ledgers supersede it for active Phase 040 completion decisions.

### 5. Open Boundary Integrity

🔴 **Quest:** Which stronger checkpoint, public proof, or settlement assertions would become overclaims if stated as completed, and what kind of repository evidence would be required before each assertion could be upgraded?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Three stronger assertions would become overclaims if stated as completed by Phase 040: public or trustless proof-of-knowledge for regular spend, checkpoint theorem finality or standalone trustless publish theorem, and full rollup settlement proof closure. Current Phase 040 closes the internal wallet/simulator theorem relation and a rollup public-artifact binding guard; it does not close those stronger proof systems.

**Evidence Trail:**

1. `.planning/phases/040-spend-proof/040-VALIDATION.md` lists `public proof-of-knowledge`, `checkpoint theorem finality`, and `rollup settlement proof closure` as manual open boundaries.
2. The validation map marks the simulator/checkpoint and rollup rows green only for package-coupled continuity and public-artifact binding, while keeping public proof, checkpoint theorem, and full settlement closure open.
3. The manual-only table in `040-VALIDATION.md` says the current verifier checks deterministic canonical artifacts and public binding, not cryptographic witness knowledge.
4. `.planning/phases/040-spend-proof/040-CLOSEOUT-GATES.md` bounds closeout to one canonical internal theorem-relation path and requires public/trustless proof-of-knowledge, checkpoint theorem finality, and rollup proof closure to stay out of completed language.
5. `crates/z00z_wallets/src/core/tx/spend_verification.rs` says the live public spend contract is real but still narrower than a finished full-ZK spend theorem.
6. `crates/z00z_simulator/src/scenario_1/stage_11.rs` and `stage_12.rs` say package-coupled checkpoint integrity exists, but publish is not yet fully trustless and authoritative publish-proof closure does not exist.
7. `crates/z00z_rollup_node/src/lib.rs` says settlement verification intentionally accepts only public artifacts, never rebuilds private witnesses, and never treats output range proofs as settlement closure.
8. `crates/z00z_rollup_node/tests/test_settlement_theorem.rs` covers the current rollup binding matrix: canonical bundle acceptance, checkpoint replay rejection, missing transaction rejection, root mismatch rejection, bad link rejection, and bad package rejection.

**Reasoning:**

- A public/trustless proof-of-knowledge claim would overstate the live verifier because verifier-side acceptance does not prove knowledge of `ReceiverSecret`, ordered `s_in`, or membership witnesses. To upgrade it, the repository would need a real public verifier-side cryptographic proof path over the private witness plus focused tests showing forged deterministic artifacts cannot pass without witness knowledge.
- A checkpoint theorem finality claim would overstate Stage 11 and Stage 12 because they prove package-coupled checkpoint continuity and fail-closed mutation boundaries, not standalone trustless checkpoint or publish-proof finality. To upgrade it, Stage 11 or the checkpoint apply path would need to consume and verify a public theorem artifact or state-transition proof, with negative tests for replay, root, link, proof, and mutation drift before checkpoint mutation.
- A full rollup settlement proof claim would overstate `verify_settlement_theorem(...)` because it verifies public-artifact coupling and transaction inclusion, not a final rollup proof system. To upgrade it, settlement admission would need to verify the stronger proof path end to end, with tests that fail on proof-system drift as well as bad package, missing transaction, checkpoint replay, root mismatch, and link drift.
- The current green validation rows are therefore meaningful but bounded. Treating them as full public proof, checkpoint finality, or settlement finality would convert explicit open boundaries into false closeout language.

**Gap Or Blocker:** The three stronger claims remain open by design. Their closure requires new live implementation, targeted negative tests, and updated current validation and closeout ledgers.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: current rollup tests cover public-artifact binding negatives; proof-system drift belongs to the future evidence required for full settlement proof closure, not to the already-closed Phase 040 claim.

## Theme 2: Carrier, Statement, And Digest Discipline

### 6. Non-Empty Carrier Contract

🔴 **Quest:** What proves that accepted regular-spend packages must carry explicit proof and authorization material, and which malformed or placeholder shapes must fail before admission is considered valid?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Accepted regular-spend packages must carry explicit spend-proof and spend-authorization material on the public carrier path: `tx.proof.spend` and `tx.auth.spend` must exist and satisfy canonical field, version, suite, and non-empty payload checks before admission succeeds. This is enforced by wire schema, public contract verification, full-package admission routing, and dedicated negative tests for placeholder and malformed forms.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` defines `TxProofWire { spend: Option<SpendProofWire> }` and `TxAuthWire { spend: Option<SpendAuthWire> }`; `SpendProofWire` and `SpendAuthWire` use `deny_unknown_fields` with explicit fields (`ver`, `proof_suite`, `prev_root_hex`, `statement_hex`, `proof_hex`, `inputs`, `receiver_card_compact`, `spend_sig_hex`).
2. `crates/z00z_wallets/src/core/tx/spend_verification.rs` fail-closes on carrier absence and malformed carrier surfaces: `MissingProof`, `MissingAuth`, bad versions, bad suite, bad previous root, missing statement blob, missing proof blob, and input pairing/count drift.
3. `crates/z00z_wallets/src/core/tx/tx_verifier.rs` shows full admission is composed: local verifier success is not enough; `verify_full_tx_package(...)` additionally requires `verify_package_public_spend_contract(...)`.
4. `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs` proves a local-valid package is rejected by full verification when public spend proof is missing.
5. `crates/z00z_wallets/tests/test_spend_proof_wire.rs` proves non-empty carrier discipline and placeholder rejection: empty legacy carrier, missing required proof fields, empty placeholder statement or proof blobs, bad proof or auth versions, noncanonical statement/proof hex, and opaque statement-unbound proof bytes are all rejected.
6. `crates/z00z_wallets/tests/test_direct_tx_receive.rs` proves proofless packages are not import-ready and do not produce owned outputs.
7. `.planning/phases/040-spend-proof/040-CONTEXT.md` and `040-VALIDATION.md` record `040-01` as landed non-empty versioned carrier with fail-closed placeholder rejection.

**Reasoning:**

- The carrier contract is explicit at three layers: typed wire schema, boundary verifier checks, and full-admission routing.
- Schema alone is insufficient, so runtime checks enforce presence and canonical field constraints before deeper theorem/binding checks.
- Local structural validity is intentionally weaker than full admission. The full verifier must reject proofless or placeholder packages even when local wire checks pass.
- The reject matrix covers both absent carriers and deceptive malformed carriers: wrong versions, wrong suite, missing fields, empty blobs, opaque blobs, and canonicalization drift.
- This proves admission discipline, not stronger public proof-of-knowledge closure.

**Gap Or Blocker:** None for the non-empty carrier admission boundary.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: this closure is about accepted regular-spend package admission and fail-closed carrier integrity, not about public/trustless proof-of-knowledge closure.

### 7. Statement Recomposition

🔴 **Quest:** How can a solver verify that the producer and verifier are bound to the same canonical public statement, rather than accepting a detached or ad hoc transcript supplied by the package author?
🔵 **Ans:**

### 8. Digest Root Authority

🔴 **Quest:** What establishes the authoritative public binding root for the spend package, and how would repository evidence expose an implementation that mistakenly promotes a narrower helper digest into public authority?
🔵 **Ans:**

### 9. Input Pairing Semantics

🔴 **Quest:** What evidence proves that carried input proof facts are positionally paired with consumed transaction input references, including both state key and leaf-match data, and what mutation should reject if that pairing is real?
🔵 **Ans:**

### 10. Authorization Binding

🔴 **Quest:** How should a solver prove that spend authorization is bound to the canonical statement and proof family rather than merely attached to a structurally plausible package envelope?
🔵 **Ans:**

## Theme 3: Witness, Membership, And Rule Invariants

### 11. Witness Privacy Boundary

🔴 **Quest:** Which witness values are allowed to remain private during proof generation, which public facts are allowed to be persisted, and what repository evidence would reveal an accidental promotion of witness material into the public carrier?
🔵 **Ans:**

### 12. Membership Against Previous Root

🔴 **Quest:** What proves that membership is checked against the same previous root used by the public spend statement, and which tamper cases should fail if the root, path, leaf, or proof item drifts?
🔵 **Ans:**

### 13. Spend Rule Preservation

🔴 **Quest:** How can a solver show that ownership derivation, input key derivation, owner tag, leaf identifier, nullifier, balance, and range conditions remain one ordered relation after proof integration?
🔵 **Ans:**

### 14. Nullifier Separation

🔴 **Quest:** What repository evidence distinguishes regular-spend replay semantics from claim replay semantics, and how can the answer avoid overstating the current replay model as a standalone persisted nullifier registry?
🔵 **Ans:**

### 15. Output Range And Balance Relations

🔴 **Quest:** What proves that forged public output relations, missing or invalid range proofs, duplicate identifiers, input-output overlap, and balance drift fail closed at the current theorem boundary?
🔵 **Ans:**

## Theme 4: Runtime Continuity And Admission Gates

### 16. Producer Runtime Reality

🔴 **Quest:** What evidence shows that the runtime producer constructs the spend contract from real selected inputs, output data, previous-root membership, and receiver-side witness material rather than from static fixtures?
🔵 **Ans:**

### 17. Local Wire Versus Full Admission

🔴 **Quest:** How can a solver demonstrate that structural transaction-package validation is insufficient on its own, and that public spend-contract verification is part of the composed admission path?
🔵 **Ans:**

### 18. Persisted Package Continuity

🔴 **Quest:** What proves that the proof-bearing package survives persistence and reload without changing the statement, proof bytes, input references, or output facts that later consumers rely on?
🔵 **Ans:**

### 19. Checkpoint Apply Rejection Boundary

🔴 **Quest:** Which repository evidence demonstrates that checkpoint mutation remains blocked when execution input, previous root, proof bytes, input references, or bridge outputs diverge from the accepted package?
🔵 **Ans:**

### 20. Package-Coupled Checkpoint Authority

🔴 **Quest:** How should an answer prove the current checkpoint path is package-coupled and fail-closed while avoiding the conclusion that a standalone checkpoint theorem backend has been completed?
🔵 **Ans:**

## Theme 5: Rollup Binding, Coverage, And Closeout Honesty

### 21. Public Artifact Settlement Boundary

🔴 **Quest:** What exactly is bound by the rollup-facing settlement check, and what would show that the implementation is verifying public artifact continuity rather than rebuilding or proving private spend witnesses?
🔵 **Ans:**

### 22. Inclusion And Root Coupling

🔴 **Quest:** How can a solver verify that settlement admission ties the transaction package to checkpoint artifact data, execution input identity, previous root, link data, input references, outputs, and inclusion in the executed transaction rows?
🔵 **Ans:**

### 23. Rollup Negative Matrix

🔴 **Quest:** Which adverse settlement cases must be rejected to make the public-artifact binding claim credible, and how should the answer distinguish those cases from proof of full settlement finality?
🔵 **Ans:**

### 24. Validation Coverage Adequacy

🔴 **Quest:** Does the phase validation map cover each promised family of carrier, statement, producer, verifier, nullifier, full-admission, runtime, checkpoint, rollup, and closeout behavior with executable evidence, and where would a coverage gap have to be recorded?
🔵 **Ans:**

### 25. Closeout Truth Drift

🔴 **Quest:** What final repository-backed argument would convince a reviewer that Phase 040 closeout language matches live implementation truth, including both completed internal guarantees and the boundaries that require future proof work?
🔵 **Ans:**

## Summary Table

| Q | Title | Proof Status | Verification | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- |
| 1 | Closure Claim Classification | Full Evidence | VERIFIED | None for the bounded internal claim | None; stronger public proof-of-knowledge requires a real verifier-side proof system and focused tests |
| 2 | Internal Relation Scope | Full Evidence | VERIFIED | None for internal `T(S, W)` scope | Public proof-of-knowledge still requires verifier-side witness-proof validation |
| 3 | Canonical Suite Exclusivity | Full Evidence | VERIFIED | None for live executable surfaces | Keep archived old-suite mentions clearly historical or negative-check only |
| 4 | Historical Baseline Discipline | Full Evidence | VERIFIED | None for authority classification | Upgrade historical claims only through live code/tests plus current validation and closeout ledgers |
| 5 | Open Boundary Integrity | Full Evidence | VERIFIED | Public PoK, checkpoint finality, and full settlement proof remain open by design | Land stronger proof paths, focused negative tests, and updated validation/closeout ledgers before upgrading claims |
| 6 | Non-Empty Carrier Contract | Full Evidence | VERIFIED | None for admission boundary | Keep full-admission routing fail-closed; do not weaken public spend contract gate into local-wire-only acceptance |
| 7 | Statement Recomposition | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q6 |
| 8 | Digest Root Authority | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q7 |
| 9 | Input Pairing Semantics | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q8 |
| 10 | Authorization Binding | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q9 |
| 11 | Witness Privacy Boundary | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q10 |
| 12 | Membership Against Previous Root | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q11 |
| 13 | Spend Rule Preservation | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q12 |
| 14 | Nullifier Separation | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q13 |
| 15 | Output Range And Balance Relations | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q14 |
| 16 | Producer Runtime Reality | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q15 |
| 17 | Local Wire Versus Full Admission | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q16 |
| 18 | Persisted Package Continuity | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q17 |
| 19 | Checkpoint Apply Rejection Boundary | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q18 |
| 20 | Package-Coupled Checkpoint Authority | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q19 |
| 21 | Public Artifact Settlement Boundary | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q20 |
| 22 | Inclusion And Root Coupling | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q21 |
| 23 | Rollup Negative Matrix | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q22 |
| 24 | Validation Coverage Adequacy | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q23 |
| 25 | Closeout Truth Drift | Unanswered | UNVERIFIED | Not solved yet | Solve sequentially after Q24 |
