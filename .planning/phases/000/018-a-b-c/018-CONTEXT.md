<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD047 -->
# Phase 018: 018-A-B-C - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

🎯 Phase 018 closes the remaining proof and artifact gaps in `scenario_1` after Phase 017 so the simulator can prove one canonical path from claim publication through regular transfer apply into final checkpoint publication.

🎯 This phase must deliver a proof-complete path for `Alice -> Bob -> Charlie` evidence, wallet-side JMT inclusion verification, one canonical ledger continuity artifact, and one finalized Stage 8 acceptance path with sealed checkpoint artifact, link, and audit surfaces.

🎯 This phase does not add a new ledger model, new storage backend, or a second wallet proof system. It finishes adoption of the existing `z00z_storage`, `z00z_wallets`, and `scenario_1` surfaces.

</domain>

<decisions>
## Implementation Decisions

### Canonical continuity
- **D-01:** Stage 4 prep continuity must be derived from the full claim-backed live store, not from a selected-input subset store.
- **D-02:** Phase 018 must emit one machine-readable ledger continuity artifact that ties together `claim root -> prep root -> post-apply root -> draft id -> exec input id -> checkpoint id`.
- **D-03:** `claim_post`, `pre_tx`, and `post_tx` remain observational exports; they are not sufficient as the canonical chain path unless continuity is written explicitly.

### Wallet-side JMT proof path
- **D-04:** Wallet-side discovery for this phase must use a proof-validated JMT scan path: enumerate committed rows from the persisted `post_tx` store, build `ProofBlob`, verify via `chk_blob(...)`, then run stealth ownership detection.
- **D-05:** Detached leaf-only scanning is insufficient for Phase 018 acceptance because it proves ownership of a leaf in hand, not inclusion of that leaf in canonical committed ledger state.
- **D-06:** The new JMT wallet scan helper should be implemented once in `z00z_simulator` and reused for evidence generation rather than creating actor-specific one-off logic.

### Charlie runtime and full path
- **D-07:** Phase 018 must perform a real post-apply Charlie wallet runtime update after Stage 7 canonical apply, not only emit bridge fragments or evidence-only JSON.
- **D-08:** The acceptance story for `Alice -> Bob -> Charlie` must end with refreshed wallet evidence showing Charlie state change in the standard wallet report surface.
- **D-09:** Stage 6 remains a bridge and must not mutate Charlie wallet runtime.

### Tx proof verification depth
- **D-10:** Phase 018 should not expand cryptographic scope by replacing the existing checkpoint proof model unless a failing requirement forces it.
- **D-11:** Phase 018 must strengthen proof coverage by adding explicit propagation and binding checks that the same proof-bearing transaction path is preserved across Stage 6 exec input, Stage 7 draft apply, and Stage 8 final artifact publication.
- **D-12:** `PassProof` is not sufficient as the only acceptance story; tests and artifacts must demonstrate proof flow, not just non-empty bytes.

### Final checkpoint publication
- **D-13:** `draft_only` must remain available as a negative or bridge gate, but Phase 018 acceptance must also include at least one finalized `OpaqueTest` Stage 8 path.
- **D-14:** Stage 8 summary output must expose the final sealed checkpoint artifact path, link path, audit path, and `checkpoint_id_hex` when finalization succeeds.
- **D-15:** Draft and final checkpoint surfaces must remain separate contracts and separate artifacts.

### Lifecycle proof and wallet evidence
- **D-16:** `pending -> confirmed` lifecycle transitions are only acceptable for this phase if the committed JMT scan path is shown to drive the resulting wallet evidence.
- **D-17:** Phase 018 must refresh the standard wallet evidence artifacts after the committed post-apply scan so Charlie, Bob, and any other affected actors are visible through the existing before/after/diff/report outputs.
- **D-18:** The explanation of `leaf scan` versus `JMT scan` is part of the phase deliverable and should be reflected in machine-readable or human-readable acceptance output.
- **D-19:** Phase 018 must add one explicit wallet-balance invariant gate over the refreshed before/after/diff or confirmed evidence so the scenario proves wallet totals coherently after the committed JMT-driven update, not only transaction-local balance correctness.

### Scope guardrails
- **D-20:** Do not add a new simulator-local ledger engine, a new proof format, or direct parsing of exported JMT artifact files as the wallet source of truth.
- **D-21:** Do not bypass `z00z_storage` ownership of canonical roots, proofs, checkpoint drafts, checkpoint artifacts, or persistence semantics.

### Execution order and stop gates
- **D-22:** The required execution spine is fixed: Stage 3 claim-backed store publish -> Stage 4 full-store continuity repair -> Stage 6 bridge and exec input only -> Stage 7 canonical apply -> Stage 7 proof-validated JMT wallet scan and wallet evidence refresh -> Stage 8 finalized checkpoint publication.
- **D-23:** Stage 7 wallet refresh must not run from fragment JSON, detached leaf scan results, or pre-apply exports; it must run only after canonical apply has produced committed `post_tx` state.
- **D-24:** Stage 8 finalization must be blocked if any earlier gate fails: claim-to-prep continuity mismatch, committed-store proof validation failure, Charlie wallet update failure, or wallet-balance invariant failure.
- **D-25:** If the post-apply JMT wallet scan or wallet-balance invariant gate fails, the phase may keep bridge or draft artifacts for debugging, but it must not treat refreshed wallet evidence or finalized checkpoint output as accepted results.
- **D-26:** Parallel work is safe only across artifact formatting and report writing after canonical state has already been validated; continuity repair, canonical apply, JMT proof verification, wallet mutation, and finalization remain sequential gates.

### the agent's Discretion
- Exact module name for the new post-apply JMT scan helper.
- Exact artifact file names for Charlie scan evidence and ledger continuity, as long as they remain deterministic and machine-readable.
- Exact internal report structure used to connect proof verification results to wallet evidence rows.

</decisions>

<specifics>
## Specific Ideas

📌 The user wants the phase plan to close every gap listed in `.planning/phases/018-a-b-c/todo.md` and to treat `.planning/phases/018-a-b-c/018-RESEARCH.md` as an input summary rather than a separate speculative branch.

📌 The user explicitly wants a provable full path for `Alice -> Bob -> Charlie` on artifacts, not just an inferred design-level argument.

📌 The user explicitly asked for clarification of the difference between leaf scanning and JMT scanning. The locked explanation is:

- Leaf scan means the wallet already has a specific leaf or output and only checks ownership hints such as stealth fields or `tag16`.
- JMT scan means the flow first enumerates committed rows from canonical storage, proves inclusion of each row against the committed root, and only then checks whether the proven row belongs to the wallet.
- Therefore stealth fields and `tag16` are ownership hints, not a ledger traversal mechanism by themselves.

📌 The user wants the minimal code gaps called out directly. The minimum implementation seams are:

- Rebase Stage 4 continuity on the full claim-backed store.
- Add one post-apply proof-validated JMT wallet scan helper.
- Wire Charlie runtime update and refreshed wallet evidence after Stage 7.
- Add one global wallet-balance invariant gate over refreshed wallet evidence.
- Add one canonical ledger continuity artifact.
- Enable one finalized Stage 8 acceptance path with artifact/link/audit outputs.

📌 The operational execution order is locked for this phase:

1. Publish or load the claim-backed canonical store.
2. Repair Stage 4 continuity from that full store.
3. Keep Stage 6 bridge-only.
4. Apply canonical state in Stage 7.
5. Run the proof-validated JMT wallet scan against committed `post_tx` state.
6. Refresh wallet evidence and enforce the wallet-balance invariant gate.
7. Finalize Stage 8 only after the earlier gates pass.

📌 The user wants planning to stay focused on implementing the listed gaps, not on broadening the simulator into new unrelated capabilities.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase inputs
- `.planning/ROADMAP.md` — current registration of Phase 18 and milestone placement.
- `.planning/PROJECT.md` — project-level constraints and crate-boundary rules.
- `.planning/REQUIREMENTS.md` — active milestone requirements and storage-safe boundary expectations.
- `.planning/STATE.md` — current decisions, blockers, and prior phase outcomes.
- `.planning/phases/018-a-b-c/todo.md` — complete gap list that this phase must close.
- `.planning/phases/018-a-b-c/018-RESEARCH.md` — repository-backed research synthesis and recommended implementation spine.

### Prior phase decisions
- `.planning/phases/017-scenario-1/017-CONTEXT.md` — locked Phase 017 decisions for storage-backed apply, proof boundary, and Stage 7/8 decomposition.
- `.planning/phases/016-jmt-search-and-redb/016-CONTEXT.md` — durable storage and deterministic search constraints that Phase 018 must preserve.
- `.planning/phases/015-jmt-serialization-visualization/015-CONTEXT.md` — storage-owned JMT artifact and inspection boundary constraints.

### Scenario 1 implementation surface
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` — executable stage design and acceptance wiring.
- `crates/z00z_simulator/src/scenario_1/stage_4.rs` — current prep continuity and pre-state artifact construction.
- `crates/z00z_simulator/src/scenario_1/stage_5.rs` — current leaf-oriented wallet receive checks.
- `crates/z00z_simulator/src/scenario_1/stage_6.rs` — bridge path and current proof placeholder behavior.
- `crates/z00z_simulator/src/scenario_1/stage_7.rs` — canonical draft apply and current post-apply export seam.
- `crates/z00z_simulator/src/scenario_1/stage_8.rs` — draft-only versus finalized checkpoint publication path.
- `crates/z00z_simulator/src/scenario_1/storage_view.rs` — observational `claim_post`, `pre_tx`, and `post_tx` export surfaces.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` — canonical claim publication into storage.

### Storage and wallet contracts
- `crates/z00z_storage/src/assets/store.rs` — canonical row listing, proof generation, and root-owned storage behavior.
- `crates/z00z_storage/src/assets/proof.rs` — `ProofBlob` validation semantics.
- `crates/z00z_storage/src/assets/store_internal/proof_help.rs` — inclusion proof helper path used by storage.
- `crates/z00z_storage/src/checkpoint/store.rs` — draft, artifact, link, and audit persistence.
- `crates/z00z_storage/src/snapshot/store.rs` — snapshot ownership and replay boundary.
- `crates/z00z_wallets/src/core/tx/state_update.rs` — wallet-side typed witness and state-update boundary.

### Supporting specs
- `specs/014-z00z-storage/checkpoint-storage-spec.md` — checkpoint artifact requirements.
- `specs/014-z00z-storage/jmt-gaps-workflow.md` — migration sequence for JMT gaps.
- `specs/014-z00z-storage/snapshot-storage-spec.md` — storage snapshot requirements.
- `specs/014-z00z-storage/публикации assets в JMT.md` — current scenario-backed observations about publication into JMT.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `publish_claims_store(...)` in `claim_pkg_consumer.rs` is already the correct claim-to-storage handoff.
- `AssetStore::list(...)`, `proof_blob(...)`, and `chk_blob(...)` already provide the proof-validated committed-row path needed for wallet-side JMT scanning.
- `build_cp_draft(...)` in Stage 7 and `seal_artifact(...)` in Stage 8 already provide the canonical draft and final checkpoint seams.
- Existing wallet evidence artifacts from Stage 4 should be extended rather than replaced, and the Stage 4 balance and diff helpers should be reused for one explicit post-apply wallet invariant gate.

### Established Patterns
- Canonical state and proof ownership live in `z00z_storage`, not in simulator-local JSON artifacts.
- Scenario exports under `storage_view.rs` are inspection aids and should not become the source of truth.
- Wallet ownership detection already exists, but Phase 018 must move it behind proof-validated committed-state enumeration.
- Stage decomposition from Phase 017 remains valid: Stage 6 bridge, Stage 7 apply, Stage 8 finalize.

### Integration Points
- Stage 4 is the continuity repair point.
- Stage 7 is the correct insertion point for the post-apply JMT wallet scan and Charlie runtime update.
- Stage 8 is the correct insertion point for final sealed checkpoint path reporting.
- `scenario_design.yaml`, scenario stage modules, and acceptance tests must move together.

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within the listed Phase 018 gaps.

</deferred>

---

*Phase: 018-a-b-c*
*Context gathered: 2026-03-24*