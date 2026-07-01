# Phase 033: Crypto Audit Scenario 2 - Context

**Gathered:** 2026-04-06  
**Status:** Guarded planning only

## Phase Boundary

Phase 033 does not redefine the Scenario 1 audit findings. Its job is to turn
the existing Phase 033 audit inventory into an execution-ready planning
surface.

The planning phase must consume the concrete task inventory already recorded in
`033-TODO.md` and produce a sequential execution plan for those tasks one by
one. This phase is about planning execution of the existing audit backlog, not
about renaming, merging, rewording, or rescoping that backlog.

No concrete crate, file, module, API, or trait target is approved by this
context alone. Each downstream task plan must verify live codebase targets
before naming them and must label any unverified target as proposed rather than
as an existing fact.

## Review Findings Integrated

- The canonical inventory is not safe to consume by raw `TASKS` display number
  alone because `033-TODO.md` contains duplicate display values and one
  carry-forward caution row.
- The summary inventory is authoritative as an index, but several rows require
  source-precedence rules before planning, especially the high-severity rows
  64 and 65.
- Open semantic gaps must remain explicit during planning: persisted
  claim-source continuity, spend-side nullifier semantics, and checkpoint proof
  authority are separate workstreams and must not be collapsed into one generic
  “crypto closure” bucket.
- The corrected active artifact set is authoritative over older optimistic wording.
- Planning truth must defer to implementation truth whenever older artifacts drift.
- Ownership, privacy-route, validator-boundary, secret-lifecycle, and
  documentation-honesty statements must remain narrow until the underlying gap
  is actually closed.

## Implementation Decisions

### Task Inventory Authority

- **D-01:** `033-TODO.md` is the canonical planning inventory for Phase 033.
- **D-02:** The planner must cover all 65 canonical Phase 033 tasks.
- **D-03:** Planning must proceed sequentially, one canonical task after
  another.

### Task Naming And Wording

- **D-04:** No task title may be renamed.
- **D-05:** No task wording may be reformulated.
- **D-06:** If filenames, H1 headings, historical labels, or numbering shapes
  are inconsistent, the planner must preserve the task text exactly as written
  in the Phase 033 artifacts instead of normalizing names.

### Canonical Task Identity

- **D-07:** Phase 033 planning operates on 65 canonical task slots, not on the
  raw `TASKS` display column alone.
- **D-08:** Canonical task 31 is the row titled `Distinct Claim Reject Paths`.
  Its source summary row repeats display `TASKS = 30`; planning must preserve
  the title text exactly while using canonical slot 31 for traceability.
- **D-09:** The caution row titled `Real theft-resistance boundary` reuses
  display `TASKS = 9`. It is a mandatory carry-forward guardrail attached to
  canonical task 9, not a 66th canonical task.
- **D-10:** Context-local slot keys and canonical task numbers may be used for
  traceability, but they must never replace or rewrite the source task title.

### Detail Source Mapping

- **D-11:** The summary tables in `033-TODO.md` are the canonical task index.
- **D-12:** Detailed planning context for each indexed task must be taken from
  the phase-local audit documents referenced by the `033-TODO.md` sections and
  the corresponding phase-local files.
- **D-13:** The planning phase must treat the phase-local audit documents as the
  source of execution intent, blocker wording, and gap-closure expectations.
- **D-14:** If the summary table, detailed source body, and upstream
  requirements differ, detailed source body plus upstream requirement truth win
  over title-only or index-only inference.
- **D-15:** Tasks 64 and 65 must always be interpreted from the full source row
  body in `033-32FULL-AUDIT.md`, not from the summary title alone, because the
  summary title/body pairing is crossed in `033-TODO.md`.

### Blocker Policy

- **D-16:** No canonical task may be omitted from planning.
- **D-17:** A task may remain unplanned only if a principled blocker truly
  cannot be worked around.
- **D-18:** Any such blocker must be recorded explicitly in the final planning
  report together with the exact reason the task could not be planned around.
- **D-19:** A blocked prerequisite also blocks every dependent planning output
  that relies on the blocked semantic gap being resolved.

### Planning Output Contract

- **D-20:** Every per-task plan must record: canonical task number, exact task
  title, source file, source section or row identity, relevant upstream
  requirement IDs, current delivered boundary, missing invariant, forbidden
  overclaim, blocker text, gap-closure path, required tests or proof surfaces,
  and `blocked_by` dependencies when present.
- **D-20a:** In Phase 033 execute plans, exact task title plus explicit source
  notes satisfy row identity only when the source numbering or pairing is
  anomalous, including canonical task 31, task 47's source question number,
  the task-9 carry-forward caution row, and high-severity tasks 64 and 65.
- **D-21:** Every per-task plan must label its target code locations as either
  existing verified targets or proposed targets.
- **D-22:** No downstream plan may widen the documented security claim beyond
  the narrow wording already allowed by the source audit artifacts.

## Security Semantics That Planning Must Not Strengthen

- `PH32-CLAIM-TRUST` is now narrowed to the current helper-owned canonical one-item claim-source contract shared by storage acceptance and wallet verification; this helper-owned continuity boundary is the remaining claim-trust blocker and persisted continuity remains deferred follow-up work rather than current requirement truth. The broader original persisted storage-backed continuity wording stays open until live code implements it or the requirement is formally narrowed and re-approved.
- `PH32-SPEND` remains open until spend-side nullifier semantics are bound into the regular public spend statement, or the requirement is formally narrowed. The broader original spend-contract wording stays open until nullifier semantics land in the regular public contract or the requirement is formally narrowed and re-approved.

- Checkpoint acceptance must stay described as package-coupled continuity and
  package-coupled anti-substitution unless a standalone authoritative checkpoint
  proof backend or standalone backend authority is explicitly delivered.
- Replay/stale closure remains inside the helper-owned claim, current-stack spend, and package-coupled checkpoint boundaries.
- Ownership and anti-theft wording must stay wallet-local and
  receiver-secret-gated. Planning must not restate this as sender ignorance of
  `s_out` or as already-proved public trustless exclusivity.
- Request-bound flow remains the accepted privacy route when available and
  approved. Card-bound behavior remains a compatibility lane unless equivalence
  is separately proved.
- Documentation, summaries, closeout notes, and planning outputs must preserve
  the delivered / partial / not-proved distinction from the audit inventory.
- Verification discipline must stay described as a bootstrap-first,
  targeted-closeout contract that names the reruns actually performed.
- Verification artifacts must reject unsupported broad-suite PASS language
  unless new evidence is produced.
- Logs and manifests stay evidentiary limiters.
- They are not semantic sources of truth.
- The honest whole-chain story is layered.
- Delivered bucket: live cryptography exists at the claim, stealth, scan, and
  current-stack spend seams.
- Partial bucket: ownership continuity, checkpoint handoff, and replay/spent
  gating still rely in part on wallet-local or structural boundaries.
- Not-proved bucket: full validator trustlessness, end-to-end trustless
  closure, and other final-proof claims remain open or out of scope.
- Review rule: do not flatten the validator leg into fully live crypto or
  claim full end-to-end trustless closure.

## Documentation Allowlist

- Documentation allowlist for active Phase 033 artifacts remains blocked by
  task 25, task 27, task 63, task 64, and task 65.
- Active docs may keep only repository-backed narrow claims already frozen by
  plans 1-15 plus explicit caution carry-forward rows.
- Stronger closure language stays cautioned or out of scope until those
  blockers are closed or honestly narrowed.

## Remaining Caution Answers

- Task 51: The missing piece is final cryptographic closure, not total absence of validator-facing verification.
- Task 52: Publish is not yet strong enough to be called fully trustless.
- Task 53: The live contract is real, but it is still narrower than a finished full-ZK spend theorem.
- Task 54: Source-root continuity is stronger than placeholder roots, but not yet authoritative persisted genesis membership.
- Task 55: The right finding is unfinished boundary, not unchanged old placeholder runtime.
- Task 56: Finish the policy layer that makes identity binding uniformly mandatory.
- Task 57: Promote request-bound mode from available option to normal privacy path.
- Task 58: Replace simulator-fixed authority roots with live anchored authority lifecycle.
- Task 59: Preserve current statement binding and replace helper continuity with authoritative membership proofs.
- Task 60: Finish authoritative proof and spent backends instead of describing the runtime as fully placeholder-based.
- Task 61: Keep default lane hardened and finish debug-lane confinement, wrapping, and retention policy.
- Task 62: This is a consolidation pass over live abstractions, not a brand-new design.
- Task 63: Keep this isolated high-severity row pinned to persisted storage-backed membership continuity or formal narrowing.
- Task 64: Keep this crossed high-severity row pinned to: the regular public spend contract verifies real proof/auth data but still carries no nullifier semantics. Extend the spend statement, persisted wire contract, and verifier to bind nullifier semantics, or narrow the requirement honestly.
- Task 65: Keep this crossed high-severity row pinned to: finalized checkpoint acceptance still relies on externally supplied verifier trust and compatibility payload bytes instead of a standalone proof backend. Implement an authoritative checkpoint proof backend and bind finalize/load acceptance to that backend, or narrow the requirement honestly.

## Risk Clusters

The clusters below are dominant risk families, not an exhaustive partition of
all 65 canonical tasks. The per-task coverage matrix remains authoritative for
complete task coverage, and some canonical tasks intentionally span more than
one risk family.

### Claim Continuity And Authority

- Covers canonical tasks 1-4, 17, 25, 28-32, 54, 58-60, and 63.
- Planning must keep persisted authority, helper-owned reconstruction, reject
  semantics, and membership continuity as separate subproblems.

### Ownership, Anti-Theft, And Privacy Routing

- Covers canonical tasks 5, 7-9, 11, 19, 33-37, 48-50, and the caution
  carry-forward row `Real theft-resistance boundary`.
- Planning must preserve wallet-local ownership language, `leaf_ad_id`
  identity freeze, and request-bound privacy wording.

### Spend Boundary And Nullifier Semantics

- Covers canonical tasks 10, 13, 25, 38-40, 51-53, and 65.
- Planning must keep the current public spend boundary distinct from the still
  missing nullifier semantics.

### Checkpoint Authority And Operator Boundary

- Covers canonical tasks 12, 15-18, 41-43, 55, 60, and 64.
- Planning must not call checkpoint acceptance authoritative backend closure
  unless the standalone proof backend actually lands.

### Secret Lifecycle, RNG, And Documentation Honesty

- Covers canonical tasks 20-24, 44-47, 61-62.
- Planning must keep secret-export closure, deterministic randomness,
  verification discipline, and documentation honesty as first-class work rather
  than editorial cleanup.

## Canonical Task Identity And Source Precedence

| Source anomaly | Planning interpretation |
| --- | --- |
| Display `TASKS = 30` appears twice in `033-TODO.md` | `Self-Consistency Versus Authority` remains canonical task 30; `Distinct Claim Reject Paths` is canonical task 31 while preserving the exact source title and display number in evidence. |
| The caution table contains `TASKS = 9` for `Real theft-resistance boundary` | This is a mandatory carry-forward caution row attached to canonical task 9, not a separate canonical task. It must still be preserved in plan evidence and guardrail language. |
| High-severity rows 64 and 65 have crossed summary title/body pairing | Use the full detailed source row body from `033-32FULL-AUDIT.md` and preserve the summary title as written; do not infer remediation from title alone. |

## Dependency And Blocker Propagation

- Sequential planning order remains mandatory, but semantic prerequisites must be
  recorded explicitly inside each downstream task plan.
- Canonical tasks 25, 63, 64, and 65 are prerequisite closure gates for any
  downstream work that attempts honest reclassification, documentation cleanup,
  or “what may stay” conclusions.
- Honest reclassification remains blocked until the broader original PH32-CLAIM-TRUST and PH32-SPEND gaps are implemented and re-verified, or formally narrowed and re-approved.
- Canonical task 47 is a documentation-governance gate. It must not be closed
  while the upstream semantic gaps it summarizes remain open.
- Caution tasks 48-62 do not authorize stronger claims. They are guardrail
  tasks that constrain how remediation and documentation plans may be written.
- If a task depends on a blocked claim-authority, spend-nullifier, or checkpoint
  authority gap, that dependency must be listed in `blocked_by` and the task
  must remain not implementation-ready.

## Task Coverage Matrix

| Canonical Task | Source `TASKS` | Exact Title Or Topic | Source File | Required Planning Treatment | Guardrail |
| --- | --- | --- | --- | --- | --- |
| 1 | 1 | Full Tuple Or Partial Story | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Preserve exact tuple language and verifier-boundary scope. |
| 2 | 2 | Authoritative Store Or Local Reconstruction | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Treat persisted authority as distinct from local reconstruction. |
| 3 | 3 | Precise Reject Semantics | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Preserve fail-closed reject-class language. |
| 4 | 4 | Publish-Bound Claim Continuity | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Keep publish continuity separate from standalone backend authority. |
| 5 | 5 | Sender Knowledge Versus Anti-Theft | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Do not widen into sender-ignorance theorem. |
| 6 | 6 | Canonical Output-Secret Semantics | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Preserve deterministic `s_out` wording and remove competing legacy models only through evidence-backed plans. |
| 7 | 7 | Associated-Data Identity Freeze | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Preserve `leaf_ad_id` identity freeze wording. |
| 8 | 8 | Request Privacy Versus Card Fallback | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Keep request-bound and card-bound paths distinct. |
| 9 | 9 | End-To-End Ownership Through The Chain | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Also preserve caution carry-forward row `Real theft-resistance boundary` from `033-EXAM-QUESTIONS-AND-ANSWERS-3.md`. |
| 10 | 10 | What The Current Public Boundary Actually Proves | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep delivered boundary narrower than full theorem. |
| 11 | 11 | Theft Windows Before And After Publication | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Keep withholding and anti-theft scope separated. |
| 12 | 12 | Proof Continuity Across Handoff | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Preserve package-coupled continuity wording. |
| 13 | 13 | The Requirement That Remains Open | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep nullifier semantics as the exact missing spend-side element. |
| 14 | 14 | Full-Chain Crypto Closure Versus Partial Security | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Preserve delivered / partial / not-proved buckets. |
| 15 | 15 | Placeholder Success Paths Truly Closed | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep scope on accepted current-stack path only. |
| 16 | 16 | Draft Versus Final Truth | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Preserve draft-vs-final distinction without hiding compatibility artifacts. |
| 17 | 17 | Injective Persistence Contract | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Keep raw artifact weakness separate from canonical persisted path. |
| 18 | 18 | Replay And Stale-Artifact Resistance | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Preserve replay/stale closure boundaries exactly. |
| 19 | 19 | Post-Scan And Post-Spend Theft Resistance | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Keep compositional defense wording; do not collapse into universal theorem. |
| 20 | 20 | Default Secret Silence | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Keep closure narrow to default plaintext secret artifact removal. |
| 21 | 21 | Explicit Debug Lane Only | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep debug-export surface explicitly feature-gated and non-default. |
| 22 | 22 | Seeded RNG Stays Bounded | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Preserve simulator-scoped wording; do not restate impossible-by-construction guarantees. |
| 23 | 23 | Verification Discipline Versus Overclaim | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Treat verification discipline as a real planning artifact, not style cleanup. |
| 24 | 24 | Is The Whole Scheme Really Secure | 033-EXAM-QUESTIONS-AND-ANSWERS-1.md | Implement or narrow missing invariant, then verify closure | Preserve the honest three-bucket answer. |
| 25 | 25 | Delivered Closure Versus Open Closure | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | This is a prerequisite summary gate for unresolved semantic gaps. |
| 26 | 26 | Planning Truth Versus Implementation Truth | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Preserve corrected artifacts over older optimistic wording. |
| 27 | 27 | Conditions For Honest Reclassification | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Reclassification now depends on keeping narrowed helper-owned `PH32-CLAIM-TRUST` language explicit while recording `PH32-SPEND` as the remaining still-open semantic gap until it is closed or formally narrowed. |
| 28 | 28 | Full Authenticated Claim Tuple | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Preserve full tuple signature scope. |
| 29 | 29 | Tuple Drift Under Plausible Package Shape | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Keep drift tests and recipient-binding nuance separate. |
| 30 | 30 | Self-Consistency Versus Authority | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Keep helper self-consistency distinct from authoritative continuity. |
| 31 | 30 | Distinct Claim Reject Paths | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Canonical task 31; source display `TASKS` repeats 30 in `033-TODO.md`. |
| 32 | 32 | The Seam That Keeps Claim Trust Partial | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep the exact partial seam explicit. |
| 33 | 33 | Sender Knowledge And The Narrower Anti-Theft Rule | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Preserve the narrower honest anti-theft rule. |
| 34 | 34 | Receiver-Held Secret As The Ownership Gate | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep receiver-secret ownership gate wallet-local unless proven otherwise. |
| 35 | 35 | Canonical Decrypt-Associated Asset Binding | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Preserve decrypt-associated binding wording exactly. |
| 36 | 36 | Request-Bound Route Versus Card-Bound Route | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep request-bound as preferred privacy route. |
| 37 | 37 | Exclusivity After Scan | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Do not widen wallet-local exclusivity into public theorem. |
| 38 | 38 | What The Spend Boundary Actually Proves | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Keep spend boundary distinct from missing nullifier semantics. |
| 39 | 39 | Structural Plausibility Versus Semantic Acceptance | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Preserve semantic-acceptance boundary. |
| 40 | 40 | The Missing Spend-Statement Element | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep the missing element phrased exactly and narrowly. |
| 41 | 41 | Checkpoint Continuity Or Compatibility-Looking Proof Bytes | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Preserve package-coupled continuity wording. |
| 42 | 42 | Real Protection Against The Operator Boundary | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep operator-boundary protection distinct from authoritative backend closure. |
| 43 | 43 | Replay And Stale-Artifact Closure | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Preserve concrete replay/stale classes and helper-owned limits. |
| 44 | 44 | Default Secret-Export Discipline | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Verify delivered boundary and preserve narrow wording; do not widen scope | Keep default-safe result scoped to plaintext debug-export lane. |
| 45 | 45 | Deterministic Randomness Boundaries | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Preserve stage-scoped randomness wording. |
| 46 | 46 | Honest Status Language Across Artifacts | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Plan closure or honest narrowing; verify exact blocker and gap-closure path | Documentation honesty is a first-class task, not editorial polish. |
| 47 | 47 | What May Stay In Documentation | 033-EXAM-QUESTIONS-AND-ANSWERS-2.md | Documentation gate; do not close until upstream dependency is resolved | This task must stay blocked until the task-25/task-27 governance gates and the canonical 63/64/65 semantic gaps are closed or honestly narrowed; plans must record those `blocked_by` dependencies explicitly. |
| 48 | 48 | Real stealth and range proofs | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Do not convert this caution row into a stronger theorem. |
| 49 | 49 | Alice and the asset secret | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Preserve terminology caution around “asset secret.” |
| 50 | 50 | Authoritative publish proofs | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Do not restate compatibility-shaped backend as authoritative proof closure. |
| 51 | 51 | Incomplete validator trust model | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Keep validator-facing verification real but narrower than full authority. |
| 52 | 52 | JMT publish trustlessness | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Preserve “not yet fully trustless” language. |
| 53 | 53 | Full ZK spend claim | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Do not widen the public spend boundary into full ZK closure. |
| 54 | 54 | Genesis membership continuity | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Preserve stronger-than-placeholder but not authoritative language. |
| 55 | 55 | Checkpoint placeholder boundary | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Keep “unfinished boundary” wording. |
| 56 | 56 | Receiver identity binding fix set | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Treat this as policy-layer completion work, not brand-new architecture. |
| 57 | 57 | Request-bound tag fix set | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Promote request-bound mode only through evidence-backed planning. |
| 58 | 58 | Real claim authority fix set | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Replace synthetic authority roots through live anchored lifecycle only. |
| 59 | 59 | Genesis membership fix set | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Preserve current statement binding while targeting authoritative membership proofs. |
| 60 | 60 | Checkpoint integrity fix set | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Finish authoritative backend work instead of overstating current runtime. |
| 61 | 61 | Secret lifecycle fix set | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Keep default lane hardened and debug lane constrained. |
| 62 | 62 | RNG, credential, and config fix set | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Guardrail review; preserve safe final reading and forbid stronger claim language | Treat this as consolidation over live abstractions. |
| 63 | 63 | Claim Source Continuity Remains Synthetic | 033-32FULL-AUDIT.md | High-severity remediation gate; implement closure before dependent reclassification or docs | Keep this row pinned to persisted storage-backed membership continuity or formal narrowing. |
| 64 | 64 | Checkpoint Proof Acceptance Is Compatibility-Payload Only | 033-32FULL-AUDIT.md | Keep the title verbatim while preserving the crossed high-severity row body: the regular public spend contract verifies real proof/auth data but still carries no nullifier semantics. Extend the spend statement, persisted wire contract, and verifier to bind nullifier semantics, or narrow the requirement honestly. | Use the full source row body, not title-only inference, because the summary row body and title are crossed in `033-TODO.md`. |
| 65 | 65 | Regular Spend Contract Still Lacks Nullifier Semantics | 033-32FULL-AUDIT.md | High-severity remediation gate; keep the crossed source row pinned to finalized checkpoint acceptance still relying on externally supplied verifier trust and compatibility payload bytes instead of a standalone proof backend, then implement authoritative checkpoint proof backend closure before dependent reclassification or docs | Use the full source row body, not title-only inference, because the summary row body and title are crossed in `033-TODO.md`. |

### Mandatory Carry-Forward Source Row

| Source `TASKS` | Exact Title Or Topic | Source File | Planning interpretation |
| --- | --- | --- | --- |
| 9 | Real theft-resistance boundary | 033-EXAM-QUESTIONS-AND-ANSWERS-3.md | Preserve this exact caution row as a mandatory guardrail attached to canonical task 9. Do not promote it to an extra canonical task and do not drop it from plan evidence. |

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 033 Planning Surface

- `.planning/phases/033-crypto-audit-scenario-2/033-TODO.md` — Canonical planning inventory.
- `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md` — Semantic boundary carried into the Phase 033 planning surface.

### Phase 033 Task Detail Sources

- `.planning/phases/033-crypto-audit-scenario-2/033-EXAM-QUESTIONS-AND-ANSWERS-1.md` — Detail source for the first task group.
- `.planning/phases/033-crypto-audit-scenario-2/033-EXAM-QUESTIONS-AND-ANSWERS-2.md` — Detail source for the second task group.
- `.planning/phases/033-crypto-audit-scenario-2/033-EXAM-QUESTIONS-AND-ANSWERS-3.md` — Detail source for the caution and safe-reading task group.
- `.planning/phases/033-crypto-audit-scenario-2/033-32FULL-AUDIT.md` — Detail source for the final high-severity task group.

### Upstream Phase Truth

- `.planning/ROADMAP.md` — Canonical roadmap registration for Phase 033.
- `.planning/STATE.md` — Canonical active-phase state for Phase 033.
- `.planning/REQUIREMENTS.md` — Upstream requirement truth, including narrowed helper-owned `PH32-CLAIM-TRUST` closure and the still-open `PH32-SPEND` requirement that motivates the remaining follow-up phase work.
- `.planning/PROJECT.md` — Project-level non-negotiable: confidential asset and wallet flows must remain correct, explicit, and storage-safe.

## Existing Code Insights

### Reusable Assets

- `033-TODO.md` consolidates the planning inventory into summary tables, but it
  must be consumed under the canonical identity rules above.
- The phase-local exam and audit documents contain the gap descriptions,
  blocker wording, and gap-closure paths that downstream plans must preserve.

### Established Patterns

- GSD phases in this repository use phase-local `CONTEXT.md` to lock decisions
  before planning.
- The active roadmap and state already point to the pre-existing
  `033-crypto-audit-scenario-2` directory, so planning must extend that
  surface rather than create a parallel structure.

### Integration Points

- `/gsd-plan-phase 033` is the next workflow step.
- The planning output must stay inside
  `.planning/phases/033-crypto-audit-scenario-2/`.
- The eventual plan set must preserve one-to-one traceability back to the
  canonical 65-task matrix above and the mandatory carry-forward caution row.

## Deferred Ideas

None. The user explicitly constrained this phase to planning the existing task
inventory without adding or renaming scope.

## Context Footer

Phase: `033-crypto-audit-scenario-2`  
Context gathered: `2026-04-06`
