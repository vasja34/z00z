# Phase 034: Mix1 Fixes - Context

**Gathered:** 2026-04-09
**Status:** Planning complete; documentary truth synchronized through the Plan 09 closure package

## Phase Boundary

Phase 034 reserves the pre-existing `034-mix1-fixes` directory as the
canonical next phase surface for the first mixed follow-up fix bundle without
creating a duplicate phase folder.

For planning purposes, `034-TODO.md` is the canonical Phase 034 planning
inventory. Downstream planning must cover the numbered Phase 034 tasks in that
inventory sequentially, one canonical task after another, while preserving the
main semantic closure chain first and the post-closure sidecars after it.

This phase is about producing execution-ready planning for the canonical Phase
034 backlog, not about rewriting backlog semantics, renaming task titles,
dropping tasks, creating a parallel implementation layer, or introducing
concept drift relative to the existing codebase and phase-local audit sources.

## Current Cryptographic Truth

- Claim-source continuity is now storage-authoritative on the accepted package
  path. `AssetStore::claim_source_contract_for_item(...)`, simulator bundle
  verification, and wallet verification all consume the same carried
  storage-backed membership contract, and missing or drifted rows reject fail
  closed instead of being re-derived through a helper seam. This closure does
  not by itself claim a broader anchored authority lifecycle beyond the carried
  membership contract and signed claim tuple.
- The regular public spend contract is already real and fail closed, and the
  shipped boundary now authenticates one signed nullifier field. Deterministic
  `chain_id || s_in` derivation is enforced in the witness bridge and
  structural-rule layer rather than by the standalone public verifier. Phase
  034 wording must stay aligned to that narrower contract through one canonical
  derivation helper, one exact domain-separated symbol, and one fixed-width
  `chain_id` framing shared by witness, signed-proof construction, and
  structural-rule paths.
- Checkpoint acceptance now binds finalize, reload, and Scenario 1 promotion
  paths to one backend-defined package-coupled contract over `proof_sys`,
  statement shape, exec identity, the persisted snapshot or link tuple, and
  bound root or payload invariants. Package-coupled checkpoint integrity is
  real and validator-facing verification is no longer a placeholder story, but
  the shipped boundary still does not claim a generic standalone proof backend
  or final cryptographic closure.
- Public sender-construction authority now lives under `core::stealth`.
  `core::tx` keeps tx-level bridge utilities and deprecated test-only shims,
  but active code, docs, and future work must not treat `builder.rs` or
  `output_flow.rs` as canonical construction owners.

## Planning Interpretation Rules

- Interpret Q63, Q64, Q65, and Q47 from the audit body plus `Exact Fixes
  Required Summary`, not from title-only shorthand.
- Interpret Q64 and Q65 using the body-level semantic reading already frozen in
  `034-TODO.md`; do not follow the crossed title-only reading of the summary
  rows.
- Every numbered task inherits its `MANDATORY pre-read` block from
  `034-TODO.md`. A task is not planning-ready until that pre-read block is
  consumed and reflected in the downstream plan.
- If a new repository-backed design constraint is discovered, planning must
  fail closed: update the canonical source first, then `034-TODO.md`, then the
  affected tests or text guards, and only then resume dependent planning.
- No dependent planning output may quietly continue on stale truth after a new
  blocker or design-constraint discovery.

## Implementation Decisions

### Canonical planning inventory

- **D-01:** `.planning/phases/034-mix1-fixes/034-TODO.md` is the canonical
  planning inventory for Phase 034.
- **D-02:** The planner must cover every numbered canonical Phase 034 task in
  `034-TODO.md`.
- **D-03:** Planning must proceed sequentially, one canonical task after
  another, in the order already defined by `034-TODO.md`.

### Task identity and wording

- **D-04:** Task titles must remain exactly as written in `034-TODO.md`.
- **D-05:** Task wording must remain exactly as written in `034-TODO.md`.
- **D-06:** The planner may add traceability, dependencies, and execution
  details, but must not normalize, merge, reword, or reinterpret the task
  labels themselves. If a repository-backed design constraint forces backlog
  wording change, the canonical source must be updated first before any backlog
  text moves.

### Blocker and exclusion policy

- **D-07:** No numbered task may be excluded from planning under normal
  conditions.
- **D-08:** Only an extreme principle-level blocker that cannot be worked
  around may prevent a task from being planned to execution depth.
- **D-09:** Blocker handling is fail closed. If such a blocker appears,
  dependent planning must stop immediately, the blocker must be recorded with
  exact source evidence, and canonical-source-first update flow must be applied
  before planning resumes.

### Reuse and anti-drift rules

- **D-10:** Do not duplicate the existing codebase or its logic.
- **D-11:** Do not introduce a parallel layer for claim-source verification,
  spend proof handling, checkpoint verification, sender authority, or suffix
  cleanup when an existing truthful seam can be extended.
- **D-12:** Prevent codebase concept drift by keeping Phase 034 planning tied
  to the existing live seams and the phase-local canonical sources.

### Main chain and sidecar handling

- **D-13:** The main semantic closure chain remains the execution backbone:
  Q63 claim continuity, Q64 regular-spend nullifier semantics, Q65
  authoritative checkpoint backend, then documentation allowlist review.
- **D-14:** The optional post-closure sidecars already present in
  `034-TODO.md` must still appear in planning order after the main closure
  chain; their optional status means they are not semantic closure evidence,
  not that they may be omitted from the planning inventory.
- **D-15:** Suffix cleanup planning must preserve the source-backed rule from
  `034-suffixes-V1-Vn.md`, which remains the authoritative suffix inventory for
  this sidecar. Its live rows may be normalized into a stable English
  execution inventory for implementation convenience, but that normalization is
  subordinate to the source inventory rather than a replacement authority.
  Production-current suffixes may be collapsed to canonical unsuffixed
  Rust-facing names, but reserved-future surfaces may be retired only where
  the authoritative inventory proves they are not still needed by current
  production read, import, open-session, or migration paths.
- **D-16:** `034-08` harness and seam-reuse lock-in is a prerequisite gate for
  the main validation waves. `034-09` through `034-13` must inherit the single
  selected test homes and must not reintroduce duplicate helpers or ambiguous
  ownership.
- **D-17:** Documentation honesty remains blocked until Q63, Q64, and Q65 are
  implemented and re-verified. Documentation allowlist narrowing and wording
  reclassification may occur only after the closure waves turn green, must
  preserve append-only audit history, and must cover the live requirements,
  active phase context, live code wording surfaces, and the concrete planning
  docs named by `034-fix-spec-4.md` before Q47 is considered honestly closed.

### Interpretation and inheritance guards

- **D-18:** Q64 and Q65 semantics are determined by the audit body and the
  `Exact Fixes Required Summary`, not by the crossed summary-row titles.
- **D-19:** Every numbered task inherits the execution rules, validation
  matrix, explicit phase boundary, and completion-gate constraints from
  `034-TODO.md`; the context may compress them, but may not weaken them.
- **D-20:** Optional sidecars are execution-visible but closure-invisible:
  `034-15`, `034-16`, and `034-17` may be planned only after `034-14`, and
  none of them may be used as evidence that Phase 034 closed Q63, Q64, Q65, or
  Q47.
- **D-21:** Once `034-12` and `034-13` turn green, downstream planning and
  closeout must update this file's `Current Cryptographic Truth` section so the
  active phase-local truth no longer advertises already-closed blockers.

### the agent's Discretion

- The agent may choose the internal structure of per-task planning artifacts,
  dependency notation, and traceability formatting.
- The agent may identify verified versus proposed code targets while keeping
  task wording untouched.
- The agent may decide how much implementation detail is needed per task, as
  long as every numbered task is covered sequentially and the anti-drift rules
  remain intact.

## Specific Ideas

- `034-TODO.md` is the single canonical scheduling surface for Phase 034.
- Planning must be sequential and complete, not selective.
- No task title or task wording may be changed.
- No duplicate logic, no parallel layer, and no concept drift are allowed.
- If a task hits an extreme principle blocker, record that blocker explicitly
  instead of silently dropping or weakening the task.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase boundary and active state

- `.planning/ROADMAP.md` §Phase 034: Mix1 Fixes — defines the canonical phase
  surface and dependency on Phase 033.
- `.planning/STATE.md` — confirms Phase 034 is the active planning phase.
- If `.planning/ROADMAP.md` and `.planning/STATE.md` diverge on current
  Phase 034 planning or execution status, treat `.planning/STATE.md` as the
  live status authority until the roadmap row is synchronized.

### Canonical Phase 034 inventory

- `.planning/phases/034-mix1-fixes/034-TODO.md` — canonical numbered planning
  inventory, execution order, dependency chain, and completion gates.

### Phase-local semantic sources

- `.planning/phases/034-mix1-fixes/034-33AUDIT.md` — authoritative inherited
  closure-gap source from the Phase 033 audit baseline for the Phase 034 mixed
  follow-up bundle; use it for exact inherited issue wording and evidence
  boundaries, while `034-TODO.md` plus this context file remain the active
  Phase 034 execution overlay.
- `.planning/phases/034-mix1-fixes/034-fix-spec-4.md` — phase-local refinement
  for regular-spend and legacy-sender seams where consistent with the audit.
- `.planning/phases/034-mix1-fixes/034-deferred.md` — deferred-history context
  that remains non-normative for new requirements but still relevant as local
  evidence.
- `.planning/phases/034-mix1-fixes/034-suffixes-V1-Vn.md` — authoritative
  production-current vs reserved-future suffix inventory for the suffix sidecar.

### Repository policy constraints

- `.github/copilot-instructions.md` — repository-local anti-duplication,
  no-parallel-layer, and identifier-length rules that remain relevant to Phase
  034 planning.
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` — primary architectural
  authority for ONE SOURCE OF TRUTH, ownership boundaries, public API rules,
  and duplicate-abstraction avoidance.

### Codebase architecture evidence

- `.planning/codebase/ARCHITECTURE.md` — crate ownership, layer flow, and live
  integration boundaries relevant to Phase 034.
- `.planning/codebase/CONCERNS.md` — known seam traps, placeholder surfaces,
  reverse test coupling, and fragile areas that Phase 034 planning must not
  accidentally normalize.
- `.planning/codebase/STRUCTURE.md` — concrete crate and module placement for
  the files already named in `034-TODO.md`.

## Existing Code Insights

### Reusable Assets

- Existing live seams named in `034-TODO.md`: these are the only approved
  starting points for downstream planning; extend them instead of inventing new
  abstraction layers.
- The pre-existing `.planning/phases/034-mix1-fixes/` directory: this is the
  only canonical planning surface for this phase.

### Established Patterns

- Extend truthful existing seams instead of adding new facades or parallel
  implementations.
- Keep semantic closure work separate from post-closure hygiene sidecars.
- Distinguish verified existing code targets from proposed targets in per-task
  plans when live code verification is still pending.

### Architecture Watchpoints

- Wallet runtime surfaces are split between real persistence and residual
  placeholder or stub-adjacent service behavior; planning must choose the live
  owner seam, not the most convenient exported surface.
- Storage and wallet tests have reverse coupling in some areas; `034-08` must
  lock one canonical test home per seam before the main validation waves start.
- Simulator Stage 6 still contains demo-oriented checkpoint material; Phase 034
  planning must not treat Stage 6 wording as authoritative checkpoint truth.
- Debug-dump and forensic helper paths are not canonical semantic evidence for
  closure; prefer contract-level validation homes over decrypted dump-oriented
  helpers.

### Integration Points

- Claim-source continuity planning must connect to the existing storage, claim,
  simulator, and verifier seams already listed in `034-TODO.md`.
- Spend nullifier planning must connect to the existing public-spend contract,
  wire contract, witness bridge, and spend-rule seams already listed in
  `034-TODO.md`.
- Checkpoint authority planning must connect to the existing finalize, reload,
  and proof-backend seams already listed in `034-TODO.md`.

## Task Transfer Coverage

The table below confirms that every numbered task from `034-TODO.md` is
transferred into the phase context as an execution-visible planning unit.

| Task | Class | Source authority | Hard deps | Primary anchors | Context transfer rule |
| --- | --- | --- | --- | --- | --- |
| `034-01` | Main chain | `034-33AUDIT.md` Q63 | None | `store_query.rs`; `test_claim_source_proof.rs` | Move claim-source authority from helper-owned off-store reconstruction to persisted membership state. |
| `034-02` | Main chain | `034-33AUDIT.md` Q63 | `034-01` | `claim_tx_verifier_impl_proof.rs`; Stage 3 claim package seams | Migrate all live producer, consumer, verifier, and fixture-helper callers to the storage-backed claim seam and leave no output-leaf reconstruction path as an unmarked authority surface. |
| `034-03` | Main chain | `034-33AUDIT.md` Q64 semantic gap: regular-spend nullifier semantics; `034-fix-spec-4.md` Workstream A | None | `domains.rs`; `tx_wire_types.rs`; `spend_rules.rs` | Add one deterministic nullifier domain, one fixed-width `chain_id` framing, and one canonical derivation surface for both the wire and structural contracts, without creating a second spend system. |
| `034-04` | Main chain | `034-33AUDIT.md` Q64 semantic gap: regular-spend nullifier semantics; `034-fix-spec-4.md` Workstream A | `034-03` | `witness_gate.rs`; `spend_verification.rs`; `spend_rules.rs` | Bind nullifier semantics into verifier, witness bridge, and structural rules through the same canonical helper and fail-closed reject behavior. |
| `034-05` | Main chain | `034-fix-spec-4.md` Workstream B | `034-03`; `034-04` | `stealth/output.rs`; `tx/mod.rs`; `builder.rs`; simulator and wallet callers | Caller-by-caller transfer from legacy `core::tx` sender authority to `core::stealth`; grep, wording, and public-surface fail-closed guards are part of completion. |
| `034-06` | Main chain | `034-33AUDIT.md` Q65 semantic gap: authoritative checkpoint proof backend | None | `artifact_proof_draft.rs`; `artifact_final.rs`; `codec.rs` | Define one backend-owned checkpoint acceptance contract over proof-system typing, statement shape, exec identity, and payload invariants instead of compatibility-payload acceptance. |
| `034-07` | Main chain | `034-33AUDIT.md` Q65 semantic gap: authoritative checkpoint proof backend | `034-06` | `state_checkpoint.rs`; backend validate seams | Bind finalize or load acceptance to the backend-defined checkpoint acceptance contract instead of a renamed compatibility path. |
| `034-08` | Main gate | `034-33AUDIT.md` Scope; `034-fix-spec-4.md` Anti-Drift | `034-01` through `034-07` | `test_claim_source_proof.rs`; `test_spend_witness_gate.rs`; `test_checkpoint_finalization.rs`; `test_s5_sender_examples.rs`; `test_claim_acceptance.rs`; `test_scenario1_stage_surface.rs` | Lock one truthful primary test home per seam, including an explicit sender-migration seam home and source-text ownership guard home, before main closure-wave validation starts. |
| `034-09` | Validation wave | `034-33AUDIT.md` Q63 | `034-01`; `034-02`; `034-08` | claim continuity test homes | Prove persisted membership, not helper reconstruction, is the live claim-source truth. |
| `034-10` | Validation wave | `034-33AUDIT.md` Q64 semantic gap: regular-spend nullifier semantics | `034-03`; `034-04`; `034-05`; `034-08` | spend nullifier test homes | Prove deterministic nullifier semantics across public spend, witness bridge, and legacy-sender migration path. |
| `034-11` | Validation wave | `034-33AUDIT.md` Q65 semantic gap: authoritative checkpoint proof backend | `034-06`; `034-07`; `034-08` | checkpoint backend test homes | Prove finalize, reload, and acceptance all consume one backend-owned acceptance path and preserve legacy-decode classification only as non-authoritative compatibility. |
| `034-12` | Documentation gate | `034-33AUDIT.md` Q47 | `034-09`; `034-10`; `034-11` | `.planning/REQUIREMENTS.md`; active docs; live code wording surfaces; concrete planning docs named by `034-fix-spec-4.md` | Documentation allowlist and wording may narrow only after closure-wave proof turns green, and the concrete planning docs named by `034-fix-spec-4.md` remain in-scope update targets for this task. |
| `034-13` | Documentation validation wave | `034-33AUDIT.md` Failure Paths; doc update rules | `034-12` | Stage 12 and stage-surface tests | Validate the new wording across active docs, live code wording, and the concrete planning docs touched by `034-12` without overclaiming beyond implemented seams. |
| `034-14` | Closeout gate | Completion ledger in `034-TODO.md` | `034-09` through `034-13` | regression and phase-proof sweep homes | Final closeout must show Q63, Q64, Q65, and Q47 no longer remain open in the live closure ledger. |
| `034-15` | Optional sidecar | `034-deferred.md` tiny debt spec | `034-14` | `store_query.rs`; `test_search_api.rs` | Behavior-preserving `keep_path(...)` cleanup only; never semantic closure evidence. |
| `034-16` | Optional sidecar | `.github/copilot-instructions.md` naming rule | `034-14` | non-Tari signature-like identifiers; affected tests and text guards | Inventory-backed 5-word hygiene only; no semantic closure claim and no Tari edits. |
| `034-17` | Optional sidecar | `034-suffixes-V1-Vn.md` Fixed Table and Bottom Line | `034-14` | backup, seed, checkpoint, claim, and proof-width suffix surfaces | Use `034-suffixes-V1-Vn.md` as the authoritative suffix inventory, optionally normalizing its live rows into an execution list, then collapse only production-current Rust-facing suffixes; compatibility-live reserved surfaces must not be blindly retired. |

## Optional Sidecar Containment

| Sidecar | Activation point | Allowed source | Must preserve | Must not claim |
| --- | --- | --- | --- | --- |
| `034-15` | Only after `034-14` | `034-deferred.md` tiny debt note | Behavior-preserving `keep_path(...)` complexity cleanup | Any closure of Q63, Q64, Q65, or Q47 |
| `034-16` | Only after `034-14` | `.github/copilot-instructions.md` Local Rust Rules | Non-Tari, inventory-backed, behavior-preserving honest renames | Any closure of Q63, Q64, Q65, or Q47 |
| `034-17` | Only after `034-14` | `034-suffixes-V1-Vn.md` Fixed Table and Bottom Line, with optional normalized execution inventory subordinate to the source table | No on-wire, no on-disk, no discriminator mutation; no blind retirement of compatibility-live reserved surfaces | Any closure of Q63, Q64, Q65, or Q47 |

## Deferred Ideas

- None beyond the already-declared post-closure sidecars in `034-TODO.md`.
- Do not import new work from older ledgers or unrelated cleanup themes unless
  the canonical Phase 034 sources are updated first.

---

*Phase: 034-mix1-fixes*
*Context gathered: 2026-04-09*
