<!-- markdownlint-disable MD001 MD022 MD032 MD033 -->

# Phase 032: Crypto Audit Scenario 1 - Context

**Gathered:** 2026-04-05
**Status:** Partially remediated; Wave 3 is implemented, Wave 4 and Wave 5 remain blocked for closeout
**Source:** `032-TODO.md`, `prompt.txt`, and current-tree Scenario 1 reconciliation
**Execution Status:** Wave 3 (`032-04`) is already implemented as a current-stack
accepted public spend-contract remediation. Wave 4 (`032-05`) and Wave 5
(`032-06`) remain blocked until authoritative checkpoint truthfulness and
simulator secret-hygiene gates close. No further note in this phase may
describe Wave 3 as DLEQ-locked or as a commitment to any specific future proof
backend unless that backend actually lands in code.

<domain>
## 🎯 Phase Boundary

- This phase turns the Scenario 1 crypto audit into one execution-ready
  remediation phase focused on truthful claim authenticity, authoritative source
  root binding, real validator or checkpoint trust boundaries, and secret or
  debug hygiene across the current `Alice -> claim package -> publish -> Bob
  scan -> spend -> validator` flow.
- The phase is not allowed to blindly implement every statement from
  `032-TODO.md` as if it were still live. Planning must first preserve the
  current-tree truth: Scenario 1 has already moved onto `claim_source`
  packages with real `ClaimAuthoritySigV2`, but the resulting flow is still not
  authoritative end to end because the signed statement does not bind the source
  root.
- The phase covers three linked tracks inside one umbrella phase:
  claim-path soundness, checkpoint or validator trust-boundary truthfulness, and
  simulator secret-lifecycle hardening.
- The phase is a blocked remediation phase, not a general feature phase.
  Downstream planning must treat unresolved claim authenticity, unresolved
  public spend verification, unresolved checkpoint truthfulness, and unresolved
  secret-artifact handling as no-go gates for implementation claims and for any
  documentation that implies cryptographic trustworthiness.
- This phase does not authorize speculative product redesign, a brand-new proof
  backend by default, or a documentation rewrite that overstates trustlessness.
  The first responsibility is to make current guarantees honest and fail-closed.

</domain>

<security_model>
## 🚨 Security Model

### 🎯 Security Goals

- Preserve truthful claim authenticity: an accepted claim package must be bound
  to an authoritative source root and an authority-authenticated statement.
- Preserve truthful spend authorization: accepted spend paths must not be
  described as publicly proven unless a real public verifier enforces them.
- Preserve truthful checkpoint integrity: checkpoint or validator artifacts must
  not carry success semantics if proof or spent-set validation is placeholder,
  synthetic, or bypassable.
- Preserve wallet-secret confidentiality: Scenario 1 must not normalize default
  plaintext export of passwords, seed phrases, receiver secrets, or equivalent
  wallet recovery material.
- Preserve restart and replay safety: claim, spend, and checkpoint artifacts
  must fail closed on replay, stale roots, or cross-run contamination.

### 🔑 Assets At Risk

- Wallet passwords, seed phrases, receiver secrets, and any debug-exported
  recovery material.
- Claim source commitments, source roots, authority statements, proof blobs, and
  claim-package publish artifacts.
- Nullifiers, spent-set state, checkpoint proofs, checkpoint drafts, and any
  accepted post-claim or post-spend state transitions.
- Receiver identity and stealth-binding material including `leaf_ad_id`,
  `owner_tag`, `tag16`, request-bound tags, and `s_out` semantics where they
  affect ownership claims.

### ⚙️ Adversaries And Failure Cases

- Malicious claimant forging claim statements, proof roots, or authority-bound
  semantics.
- Malicious simulator operator or local filesystem reader extracting secrets
  from artifacts or debug dumps.
- Replay or rollback attacker reusing stale claim, spend, or checkpoint
  artifacts.
- Concurrent execution or environment cross-contamination attacker exploiting
  process-global configuration.
- Accidental internal overclaim: developers or docs treating simulator-success
  flows as proof of cryptographic correctness when placeholders remain.

### 📌 Trust Boundaries

- Storage-owned source-root and source-proof boundary.
- Wallet claim-verifier boundary and statement-construction boundary.
- Public spend-verifier boundary versus wallet-local structural witness checks.
- Checkpoint builder and spent-set enforcement boundary.
- Simulator artifact, resume-state, and debug-secret boundary.

</security_model>

<decisions>
## 🔑 Implementation Decisions

### Current-Tree Reconciliation Policy

- **D-01:** `032-TODO.md` is the audit input, not an automatically trusted live
  finding list. Every blocker must be reconciled against current code before it
  becomes an implementation task.
- **D-02:** Planning must explicitly separate stale findings from live defects.
  The stale version of the claim-path finding is "Scenario 1 still uses only the
  old placeholder claim proof path." The live version is narrower and more
  important: Scenario 1 emits `claim_source`, but the signed statement omits
  `source_root`, so the authority signature does not anchor the proof root.
- **D-03:** Any final phase summary must stop claiming that the repository
  already provides trustless validator verification, authoritative checkpoint
  proof verification, or a live STARK or FRI stack unless the implementation in
  this phase actually delivers that boundary.

### Claim Authenticity And Source-Root Binding

- **D-04:** Treat the current `claim_v2` flow as the canonical live surface;
  do not regress back toward the older placeholder `claim::proof` path.
- **D-05:** `ClaimStmtV2` or its canonical signed statement successor must bind
  the authoritative source root directly. A self-consistent proof blob root is
  not enough if the authority signature does not cover that root.
- **D-05a:** The canonical claim statement must bind the full authenticated
  claim tuple: `asset_id`, source commitment, `chain_id`, scenario or ruleset
  version, and the authenticated source or genesis root. Root binding alone is
  not sufficient if the rest of the claim tuple can drift.
- **D-06:** `ClaimStmtV2::chk_source(...)` must evolve from version-compatibility
  checking into a contract that participates in authoritative root binding, not
  a minimal format check only.
- **D-07:** Simulator claim-package builders must stop manufacturing a proof root
  from a temporary singleton `AssetStore` if that root is not the same
  repository-authoritative published source root that downstream verification is
  expected to trust.
- **D-08:** Claim verification must stay fail-closed on wrong root, wrong proof,
  wrong asset path, forged authority signature, or version mismatch.

### Storage-Owned Proof Ownership

- **D-09:** Keep claim-source proof ownership storage-owned, consistent with
  Phases 025 and 028. Scenario 1 should consume authoritative storage proofs or
  an explicitly equivalent authenticated inclusion contract, not invent a
  simulator-local trust root.
- **D-10:** The authoritative source root must come from one canonical storage
  or snapshot boundary. Planning must make that ownership explicit instead of
  letting the simulator mint a locally valid but globally untrusted root.
- **D-11:** Production-looking claim packages must not carry proof semantics
  that are only locally true inside the simulator's temporary construction path.

### Checkpoint And Validator Trust Boundary

- **D-12:** The phase must treat spend-verification truthfulness and checkpoint
  truthfulness as linked but distinct boundaries.
- **D-13:** Structural witness checks, non-empty proof-byte acceptance, or
  synthetic proof placeholders must not be described as real public
  cryptographic verification.
- **D-14:** If a full public verifier is not delivered in this phase, the code
  and docs must say so explicitly and must reject any artifact class that claims
  stronger verified semantics than the implementation actually enforces.
- **D-15:** Planning must include negative coverage for forged spend packages,
  forged claim packages, invalid `cp_proof`, replay attempts, and spent-set or
  checkpoint bypass attempts.

### Ownership And Stealth Semantics

- **D-16:** The phase must preserve the truthful wallet-side rule that spend
  ownership depends on receiver-secret-gated verification, while avoiding the
  stronger unsupported claim that the current public proof boundary already
  proves this end to end.
- **D-16a:** If this phase delivers a real public spend verifier, the proved
  statement must bind the concrete spend tuple. At minimum planning must account
  for previous root, input references, output leaves, owner-tag relation,
  asset-id relation, balance equation, range proofs, nullifier semantics,
  `chain_id`, versioning, and explicit transcript or public-input binding.
- **D-16b:** `032-04` must be described as delivering a current-stack public
  spend-contract verifier at the accepted boundary, not as locking the design to
  DLEQ, STARK/FRI, or any other backend-specific proof primitive. Future proof
  backends may replace the internal proof object only if they preserve the same
  canonical spend statement and fail-closed acceptance semantics.
- **D-17:** Any claim that "Alice cannot steal because she does not know
  `s_out`" remains disallowed. The current code and design notes both permit
  sender knowledge of sender-generated output secret material.
- **D-18:** The canonical anti-theft statement must instead be framed around the
  additional receiver-secret requirement at spend authorization, and planning
  must verify exactly where that requirement is locally enforced versus publicly
  proven.
- **D-19:** `leaf_ad_id`, `s_out`, and receiver-binding semantics stay in scope
  where they materially affect truthful Scenario 1 security claims, but this
  phase should not broaden into a generic protocol redesign unless one of those
  semantics blocks claim-path soundness directly.

### Simulator Secret And Hygiene Hardening

- **D-20:** Plaintext secret dumps, weak deterministic RNG, fixed simulator
  credentials, and unsafe environment mutation remain in-scope hardening items
  when they affect Scenario 1 security posture or make audit claims misleading.
- **D-21:** Debug-only secret visibility must be explicit, gated, and never part
  of the default Scenario 1 contract.
- **D-22:** Secret-handling fixes in this phase must stay aligned with Phase 031
  simulator guardrails and must not reintroduce default plaintext artifacts.

### Dependency And Execution Policy

- **D-23:** Reuse existing `claim_v2`, storage proof, wallet ownership, and
  simulator surfaces where possible. The default solution is not to add a new
  proof system first.
- **D-24:** Execution order must be wave-based with explicit go or no-go gates:
  blocked-status freeze, semantic freeze, claim-root authority binding,
  spend-verifier truthfulness, checkpoint truthfulness, then simulator hygiene
  and documentation closeout.
- **D-25:** Downstream planning must preserve explicit compatibility with the
  post-Phase-031 facade and trust-boundary rules; fixes must not tunnel through
  internal seams that Phase 031 just tried to harden.

### the agent's Discretion

- Exact type names and module placement for the signed source-root binding.
- Whether authoritative source-root binding lands by extending `ClaimStmtV2` or
  by introducing a new canonical statement wrapper, provided the final authority
  signature covers the root.
- Exact batching of simulator hygiene fixes after the claim-path contract is
  frozen.

</decisions>

<semantic_freeze>
## ⛔ Canonical Semantics Freeze

- `leaf_ad_id` must be treated as a canonical public-input boundary, not a
  loosely interpreted compatibility field. Planning must freeze exactly which
  bytes define it and where those bytes are validated.
- `s_out` semantics must be frozen explicitly. Planning must answer whether
  sender-side knowledge is permitted, where receiver-secret requirements begin,
  and which boundaries are wallet-local versus publicly proven.
- If sender ignorance of the final spend secret is elevated from an honesty
  question into a hard protocol requirement, planning must spin out a separate
  claim or rekey protocol instead of pretending the current flow already has
  that property.
- Receiver identity binding must be frozen explicitly. Planning must state what
  a receiver card, payment request, or pinned card proves, and where that proof
  is only local policy versus an externally verifiable contract. This freeze is
  not complete until planning carries forward TOFU or pinning rules, signed
  directory-binding rules where applicable, mandatory request or card
  validation, and rotation or mismatch handling.
- Request-bound `tag16` and related tag-derivation paths must be frozen
  explicitly. Planning must decide which path is canonical privacy behavior and
  which compatibility paths remain allowed only for legacy or test reasons.
  This freeze is not complete until planning states whether payment-request mode
  is the preferred privacy path, whether `req_id` also binds optional `k_dh`
  derivation, and which divergence tests prove the chosen behavior.

</semantic_freeze>

<waves>
## 🚩 Execution Waves And Gates

### 🚨 Wave 0: Blocked-Status Freeze

- Restate the blocked verdict and convert it into planning gates.
- Normalize stale-versus-live findings from `032-TODO.md` so legacy placeholder
  findings are tracked without misdescribing the current-tree `claim_v2` path.
- Produce a planning contract that forbids any downstream overclaim about
  trustless claim, spend, or checkpoint verification.

**Gate:** no implementation planning may proceed until the security model,
semantic freeze set, and transfer matrix are complete.

### 🔑 Wave 1: Canonical Semantics Freeze

- Freeze `leaf_ad_id`, `s_out`, receiver identity binding, and request-bound
  `tag16` semantics.
- Identify exactly which live files own these semantics and which tests prove
  them.

**Gate:** no claim-path, spend, or checkpoint remediation is allowed on top of
unstable semantic definitions.

### ⚙️ Wave 2: Claim Authenticity And Authoritative Source-Root Binding

- Bind the signed claim statement to the authoritative source root.
- Bind the signed claim statement to the full authenticated claim tuple:
  `asset_id`, source commitment, `chain_id`, scenario or ruleset version, and
  the authoritative source root.
- Remove or isolate any simulator-local proof-root construction that can mint a
  locally valid but globally untrusted source root.
- Preserve fail-closed behavior for wrong root, wrong asset path, wrong proof,
  forged authority statement, and version mismatch.

**Gate:** a claim package must not be describable as authentic unless the root
it proves is the same root the authority signature binds.

### ⛔ Wave 3: Spend-Verifier Truthfulness

- Final remediation target: replace the current structural spend gate with a
  real public verifier. Explicit fencing is allowed only as an interim honesty
  measure during planning and cannot count as full remediation completion.
- If a real public verifier is delivered, planning must specify the exact
  proved statement and public-input binding set: previous root, input refs,
  output leaves, owner-tag relation, asset-id relation, balance equation, range
  proofs, nullifier semantics, `chain_id`, version, and transcript binding.
- Separate wallet-local ownership rules from any public-proof claims.
- Make unsupported anti-theft or sender-cannot-spend language impossible in code
  comments, docs, or acceptance criteria.

**Gate:** no accepted flow may claim public spend proof unless a real public
verifier enforces it.

### 📌 Wave 4: Checkpoint Truthfulness And Spent-Set Enforcement

- Final remediation target: make checkpoint proof verification and spent-set
  enforcement cryptographically authoritative. Placeholder removal or fencing is
  required immediately, but honesty fencing alone does not count as full
  remediation completion.
- Make checkpoint proof and spent-set ownership explicit at the storage or
  validator boundary.
- Prove that synthetic success placeholders are unreachable in accepted flows.

**Gate:** checkpoint artifacts cannot claim integrity stronger than the live
proof and spent-set enforcement actually provided.

### ✅ Wave 5: Simulator Secret Hygiene And Honest Closeout

- Remove default plaintext secret exports and define the `wallet_debug_dump`
  operational policy.
- If any debug-secret export survives at all, it must be isolated behind a
  separate feature and separate test-only executable with runtime confirmation,
  cleanup expectations, and explicit non-production labeling.
- Eliminate process-global config mutation where it contaminates trust
  boundaries.
- Remove weak deterministic RNG constructions and replace fixed actor
  credentials with seeded, domain-separated derivation or equivalent test-safe
  credential generation.
- Close the phase with honest docs and test evidence, not narrative confidence
  alone.

**Gate:** no default run may emit long-lived plaintext secret artifacts or imply
that simulator-only placeholder semantics are production-like.

</waves>

<specifics>
## ✅ Specific Ideas

- The highest-value correction from discuss-phase is this: Scenario 1 is no
  longer best described as "still on placeholder claim proof only." The more
  accurate finding is "already on `claim_source`, but still missing
  authority-bound source-root authenticity."
- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` already
  performs useful fail-closed checks on proof structure, blob or path
  consistency, asset-path matching, and authority-signature validity. The gap is
  not total absence of verification; it is missing authoritative root binding in
  the signed statement.
- `crates/z00z_storage/src/assets/store_internal/store_query.rs` already owns a
  canonical `claim_source_proof(...)` constructor. That is the natural root of
  trust to preserve instead of letting Scenario 1 create simulator-local proof
  roots.
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` remains the
  most important producer-side integration point because it currently bridges
  wallet claim logic, storage proof generation, and serialized claim packages.
- `prompt.txt` is directionally correct that the phase can be one umbrella phase
  with separate tracks, not one undifferentiated "fix everything at once"
  implementation burst.
- `032-TODO.md` contains both older placeholder findings and later execution
  guidance. The context must transfer both by labeling them as live defect,
  legacy reconciliation task, or deferred/non-blocking item instead of dropping
  them.

</specifics>

<canonical_refs>
## 📚 Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Inputs

- `.planning/phases/032-crypto-audit-scenario-1/032-TODO.md` — fused Scenario 1
  audit input, including broad end-to-end claims and claimed blockers that must
  be reconciled against the live tree.
- `.planning/phases/032-crypto-audit-scenario-1/prompt.txt` — user direction on
  how to merge the broad audit into one practical umbrella phase with explicit
  deliverable tracks.
- `.planning/ROADMAP.md` — Phase 032 registration and dependency position after
  Phase 031.

### Prior Phase Constraints

- `.planning/phases/000/025-crypto-audit-crypto/025-CONTEXT.md` — `claim_v2`
  adoption, storage-owned source-proof ownership, zero-root rejection, and
  fail-closed crypto-boundary expectations.
- `.planning/phases/000/028-crypto-audit-storage/028-CONTEXT.md` — truthful
  checkpoint semantics, explicit trust-boundary rules, and storage-owned proof
  authenticity requirements.
- `.planning/phases/000/029-crypto-audit-wallets/029-CONTEXT.md` — truthful
  receiver-secret-gated ownership language and wallet-side spend-rule boundary.
- `.planning/phases/000/031-refactor-architecture/031-CONTEXT.md` — facade-only
  simulator expectations, debug-secret gating, and boundary-hardening rules that
  Phase 032 must preserve.

### Claim And Proof Surfaces

- `crates/z00z_crypto/src/claim/v2.rs` — `ClaimStmtV2`,
  `ClaimSourceProof`, `ClaimAuthoritySigV2`, and the current version-only
  `chk_source(...)` contract.
- `crates/z00z_crypto/src/claim/prover.rs` — placeholder or legacy claim-proof
  generation surface referenced by the TODO and requiring explicit isolation or
  removal from accepted Scenario 1 paths.
- `crates/z00z_crypto/src/claim/verifier.rs` — placeholder or legacy
  verification surface that must not be treated as a production-truth boundary.
- `crates/z00z_crypto/src/claim/proof.rs` — older placeholder claim-proof API
  that remains important as a stale or misleading fallback surface.
- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` — live
  statement construction and proof or authority verification path.
- `crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs` — shared helpers around
  claim-statement construction.
- `crates/z00z_wallets/src/core/tx/claim_auth.rs` — live claim-authority
  trust-anchor seam that must either bind to explicit config or chain-state
  policy or be labeled immutable simulator-only policy.
- `crates/z00z_wallets/src/core/tx/claim_tx.rs` — higher-level claim tx flow and
  integration into wallet verification.

### Simulator And Storage Integration

- `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` — current
  Scenario 1 claim-package builder and the main producer-side integration point.
- `crates/z00z_simulator/src/scenario_1/stage_3_runtime.rs` — runtime call path
  that emits Scenario 1 claim packages.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` — verified claim-package
  loading and publish path used before storage insertion.
- `crates/z00z_storage/src/assets/store_internal/store_query.rs` —
  authoritative `claim_source_root()` and `claim_source_proof(...)` storage
  contracts.
- `crates/z00z_storage/src/checkpoint/build.rs` — checkpoint proof and spent-set
  trust boundary that still constrains truthful validator claims.
- `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs` — draft proof
  artifact constructor that must not preserve placeholder semantics into stored
  checkpoint artifacts.
- `crates/z00z_storage/src/checkpoint/artifact_final.rs` — finalized checkpoint
  artifact path that must reject non-authoritative proof representations.
- `crates/z00z_storage/src/checkpoint/codec.rs` — checkpoint proof decoding and
  canonical artifact rejection seam.
- `crates/z00z_storage/src/checkpoint/store_fs.rs` — persisted finalize/load
  boundary where invalid or synthetic checkpoint artifacts must fail closed.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` —
  placeholder checkpoint aggregation and synthetic-proof risk surface.

### Ownership, Stealth, And Request Binding

- `crates/z00z_wallets/src/core/tx/witness_gate.rs` — current structural spend
  gate that must not be mistaken for a real public spend verifier.
- `crates/z00z_wallets/src/core/stealth/output.rs` — live sender-side stealth
  output construction and `s_out`-related semantics.
- `crates/z00z_wallets/src/core/stealth/output_build.rs` — accepted send-path
  constructor seam for request or card validation and request-bound tag
  behavior.
- `crates/z00z_wallets/src/core/stealth/output_validator.rs` — live validator
  seam that re-derives `s_out` and must share the same canonical freeze.
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` — canonical
  receiver-side scan helpers and `leaf_ad_id`-dependent validation seam.
- `crates/z00z_wallets/src/core/tx/spending.rs` — unified spending module and
  event/state semantics relevant to truthful spend-boundary language.
- `crates/z00z_wallets/src/core/address/stealth_card.rs` — receiver-card
  identity surface that must be explicitly classified as public proof,
  wallet-local policy, or simulator policy.
- `crates/z00z_wallets/src/core/address/stealth_request.rs` — request-binding
  surface for payment-request and tag-derivation semantics.
- `crates/z00z_wallets/src/core/address/stealth_trust.rs` — trust policy seam
  for request and card validation language.

### Secret Hygiene And Artifact Policy

- `crates/z00z_simulator/src/scenario_1/stage_2.rs` — stage entry point whose
  defaults must not normalize plaintext secret handling.
- `crates/z00z_simulator/src/config.rs` — canonical simulator config seam for
  feature gating, secret-artifact defaults, and deterministic test knobs.
- `crates/z00z_simulator/src/config_accessors.rs` — helper seam for reading
  simulator config without process-global ambiguity.
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` — secret
  artifact export surface, including plaintext or long-lived debug material.
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` — runtime
  transport and configuration seam including environment mutation concerns.
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs` —
  post-claim artifact and secret-handling seam.

</canonical_refs>

<todo_coverage>
## ✅ TODO Coverage Map

- `032-TODO.md` items `1`, `1.5`, `5`, and `6` map to Wave 1 and must be
  executed through `032-01-PLAN.md`, including semantic freeze,
  receiver-card/request validation, TOFU or pinning rules, rotation or mismatch
  handling, and request-bound tag behavior.
- `032-TODO.md` item `2` maps to Wave 3 and must be executed through
  `032-04-PLAN.md` as a current-stack public spend-contract remediation, not an
  honesty-only downgrade. The reconciled architecture note for this work is
  backend-agnostic: it records the accepted contract and its public-input
  binding set without committing the phase to DLEQ or any other future proof
  backend.
- `032-TODO.md` item `3` maps to Wave 6 and must be executed through
  `032-07-PLAN.md` as an explicit honest closeout that drops unsupported
  STARK/FRI claims unless a real stack lands.
- `032-TODO.md` item `4` and Ideas-Document claims `D2` through `D4` map to
  Wave 4 and must be executed through `032-05-PLAN.md`, including storage
  finalize/load artifact rejection at the canonical checkpoint boundary.
- `032-TODO.md` claim-path gaps around authority-bound source-root authenticity
  and trust-anchor integrity map to Waves 2 and 3 through `032-02-PLAN.md` and
  `032-03-PLAN.md`.
- `032-TODO.md` honest-summary bullets, including passive-observer caveats and
  withheld-data caveats, map to Wave 6 through `032-07-PLAN.md`.

</todo_coverage>

<code_context>
## 🔍 Existing Code Insights

### Reusable Assets

- `ClaimAuthoritySigV2` already gives the phase a real signature primitive; the
  missing work is what exact statement bytes it signs.
- Planning must also decide and record the long-term authority trust anchor:
  who owns the real claim-authority keypair and where its public key is bound in
  configuration or chain state.
- `AssetStore::claim_source_proof(...)` already provides a storage-owned proof
  export path and should be treated as the preferred authoritative source-proof
  constructor.
- Wallet claim verification already contains fail-closed subchecks for path,
  leaf, asset id, and signature consistency that can be retained while the root
  binding is strengthened.

### Established Patterns

- Prior crypto and storage phases already established the rule that proof
  ownership stays storage-owned and that production semantics must not overclaim
  verification strength.
- Phase 031 already established that the simulator is an integration harness,
  not an excuse to reintroduce default plaintext secret surfaces.
- The repository prefers typed, fail-closed security contracts over implicit or
  stringly typed semantics.

### Integration Points

- Producer-side source-root authenticity converges in
  `stage_3_utils/claim_pkg.rs`.
- Consumer-side package trust converges in `claim_pkg_consumer.rs`.
- Claim-statement semantics converge in `claim/v2.rs` and wallet claim-verifier
  helpers.
- Validator or checkpoint truthfulness converges in storage checkpoint build or
  simulator bundle-lane proof handling.
- `leaf_ad_id`, `tag16`, and receiver-binding truthfulness converge in wallet
  stealth construction, scan-support, request, and card-validation seams and
  must be frozen before downstream remediation changes.

</code_context>

<evidence>
## 💯 Validation Gates And Required Evidence

- Claim-path evidence must prove that the authority-signed statement binds the
  same authoritative source root that downstream verification checks.
- Claim-path evidence must also prove that the authority-signed statement binds
  `asset_id`, source commitment, `chain_id`, and scenario or ruleset version as
  part of one canonical authenticated tuple.
- Storage-boundary evidence must prove that accepted Scenario 1 claim packages
  consume storage-owned source roots or an explicitly equivalent authenticated
  inclusion contract.
- Spend-boundary evidence must prove either that a real public verifier now
  exists or that all public-proof claims were removed and fenced to wallet-local
  semantics only.
- If a real public verifier is delivered, spend-boundary evidence must show that
  the exact spend tuple and transcript or public-input binding set were
  enforced, not merely that a verifier function exists.
- Spend-boundary evidence must also distinguish the accepted current-stack
  public contract from any deferred future proof backend. Docs and summaries
  must not imply DLEQ, STARK/FRI, or equivalent backend commitments unless the
  code in this phase actually adds them.
- Checkpoint evidence must prove that placeholder success states such as
  `PassProof` and `NoSpent` are unreachable in accepted flows, or that accepted
  flows reject when real proof and spent-set enforcement are unavailable.
- Replay and negative evidence must cover forged claim packages, forged
  authority statements, wrong roots, duplicate spent inputs, replayed
  checkpoint fragments, and checkpoint or spent-set bypass attempts.
- Secret-hygiene evidence must prove that default Scenario 1 runs do not emit
  plaintext wallet secrets and that any debug-export path is explicitly gated,
  labeled, and operationally constrained.
- Honest-language evidence must prove that code comments, docs, and summaries do
  not claim trustless verification, STARK/FRI enforcement, or receiver-only
  `s_out` secrecy unless the code delivered in this phase actually enforces it.

</evidence>

<guidance>
## ⚙️ Implementation Guidance And Test Obligations

### 🔑 Implementation Guidance

- Treat `scenario_1` as a harness around real verifier components, never as the
  place where proof semantics are weakened for accepted flows.
- Introduce or bind a real claim-authority trust anchor and make placeholder
  authority mode impossible outside isolated test-only targets.
- Keep proof generation and proof verification in `z00z_crypto` and
  `z00z_wallets`; simulator code may supply authenticated inputs, fixtures, and
  deterministic test configuration only.
- Keep the source of truth for nullifier uniqueness and spent-set checks in
  storage-owned code, not in Scenario 1 glue.
- Keep domain separation explicit and versioned for every hash, transcript,
  KDF label, authorization message, and statement digest touched by this phase.
- Use canonical serialization before every signed or hashed statement and make
  the verifier reject ambiguous encoding.
- Keep filesystem and serialization plumbing behind `z00z_utils` abstractions;
  do not create simulator-local bypasses for artifact handling.
- Any placeholder proof or authorization mode must fail to compile or fail to
  start outside explicitly isolated test-only targets.
- Any remaining debug-secret path must use zeroizing or hidden secret wrappers;
  raw long-lived `String` handling is not an acceptable remediation shape.
- Scenario-facing docs must carry an explicit cryptography-status disclaimer
  whenever any placeholder or simulator-only boundary remains.

### ✅ Test Obligations

- Positive tests must cover: valid genesis claim with authenticated root and
  authority binding; deterministic restart after stage-3 or stage-6 resume;
  and receiver-only decryption success after proof verification.
- Negative tests must cover: forged source commitment, forged authority
  signature, wrong chain id, wrong ruleset version, wrong authenticated root,
  duplicate spent input, replayed nullifier, malformed proof bytes, and
  checkpoint fragment replay or bypass attempts.
- Property and adversarial tests must cover: canonical serialization stability,
  statement-hash stability, checkpoint artifact loaders, claim-package decoders,
  resume-state fuzzing, and concurrent Scenario 1 execution without config
  bleed-through.
- Dependency and primitive checks must cover: `cargo audit`, `cargo deny`,
  negative tests for range-proof parameter mismatches, and standard-vector
  signature coverage where new wrappers are introduced.
- Artifact-hygiene tests must prove that default runs do not emit plaintext
  secret files and that any debug-export path is explicitly gated.

</guidance>

<preserve>
## 👍 Preserve During Remediation

- Preserve explicit BLAKE3 domain separation for claim output leaf hashes and
  owner-binding hashes.
- Preserve chain-bound nullifier derivation instead of regressing toward a bare
  claim identifier.
- Preserve range-proof generation tied to the derived blinding value rather than
  dummy scalar placeholders.
- Preserve real stealth receive behavior for intended receivers.
- Preserve foreign-receiver rejection and avoid false ownership positives.
- Preserve Bulletproofs+ or equivalent live range-proof verification where the
  current tree already performs it.
- Preserve nullifier-replay resistance and duplicate-spend rejection semantics
  already verified in the current tree.
- Preserve the sequencing where proof validation precedes ownership detection in
  post-transaction scanning.
- Preserve disciplined checkpointing and report-artifact sequencing while
  replacing placeholder crypto and overclaiming language.

</preserve>

<open_questions>
## ❓ Open Ambiguities And Planning Preconditions

- Planning must explicitly assign owners for: claim-authority trust anchor,
  claim-membership witness format, checkpoint proof enforcement, and simulator
  secret-policy enforcement. The TODO owner map is not assumed to remain
  correct; it is superseded by explicit reassignment during planning.
- Planning must answer whether `scenario_1` remains permanently simulator-only
  or is a migration path toward a production-like claim flow.
- Planning must define the storage-owned spent-set invariant and its exact
  source of truth.
- Planning must define whether stage-3 range-proof amount or asset-class limits
  are consensus-bound anywhere outside the simulator.
- Planning must define the approved `wallet_debug_dump` artifact policy,
  including retention, cleanup, CI publication, and accidental commit
  prevention.
- Planning must preserve the honest caveat that an aggregator or operator may
  censor or withhold data without directly stealing funds; data availability and
  censorship resistance must not be overstated as a side effect of claim or
  spend remediation.

</open_questions>

<transfer_matrix>
## 📋 TODO Transfer Matrix

| TODO Provision | Transfer Into This Context | Scope Status |
| --- | --- | --- |
| Final blocked verdict and no-ship posture | Captured in `Status`, `Execution Status`, `Phase Boundary`, and `Wave 0` blocked gate | Active blocker |
| Security model must be explicit | Captured in `Security Model` with goals, assets, adversaries, and trust boundaries | Active blocker |
| Reconcile stale placeholder findings against live tree | Captured in `D-01` to `D-03`, `Wave 0`, and specifics on stale-versus-live transfer | Active reconciliation |
| Placeholder claim-proof and authority-signature surfaces must not remain accepted truth boundaries | Captured in canonical refs to legacy claim surfaces plus `D-04`, `D-05`, and `Wave 2` | Active reconciliation |
| Canonical claim statement must bind `asset_id`, source commitment, `chain_id`, scenario/ruleset version, and authenticated root together | Captured in `D-05a`, `Wave 2`, and validation gates | Active blocker |
| Signed statement must bind authoritative `source_root` | Captured in `D-05`, `D-06`, `D-09`, `D-10`, `Wave 2`, and validation gates | Active blocker |
| Simulator must not mint simulator-local proof roots as authoritative | Captured in `D-07`, `D-10`, `D-11`, and `Wave 2` | Active blocker |
| Negative claim verification coverage for wrong root, wrong proof, wrong asset path, forged authority, version mismatch | Captured in `D-08`, `D-15`, and validation gates | Active blocker |
| `leaf_ad_id` canonicalization must be frozen | Captured in `D-19`, `Canonical Semantics Freeze`, canonical refs, and `Wave 1` | Active blocker |
| `s_out` semantics must be frozen honestly | Captured in `D-17` to `D-19`, `Canonical Semantics Freeze`, and `Wave 1` | Active blocker |
| Real public spend verifier must bind the full spend statement and transcript/public-input set | Captured in `D-16a`, `Wave 3`, and validation gates | Active blocker |
| Current structural spend gate must be replaced or fenced from public-proof claims | Captured in `D-12` to `D-18`, canonical refs to `witness_gate.rs`, and `Wave 3`; full remediation target remains a real public verifier and fencing alone is only an interim honesty measure | Active blocker |
| Checkpoint proof and spent-set truthfulness must be authoritative | Captured in `D-12` to `D-15`, canonical refs to checkpoint surfaces, `Wave 4`, and validation gates; full remediation target remains authoritative proof and spent-set enforcement rather than honesty fencing alone | Active blocker |
| Placeholder `PassProof` and `NoSpent` success paths must be unreachable in accepted flows | Captured in `Wave 4` and validation gates | Active blocker |
| Receiver-card identity binding must be hardened and classified honestly | Captured in `Canonical Semantics Freeze`, ownership/request refs, and `Wave 1`, including required carry-over for TOFU or pinning, signed-directory rules, mandatory validation, and rotation or mismatch handling | Active blocker |
| Request-bound `tag16` path must be made canonical or explicitly bounded | Captured in `Canonical Semantics Freeze`, ownership/request refs, and `Wave 1`, including required carry-over for preferred payment-request mode, optional `req_id`-bound `k_dh`, and divergence tests | Active blocker |
| Simulator secret handling and `wallet_debug_dump` policy must be explicit | Captured in `D-20` to `D-22`, secret-hygiene refs, `Wave 5`, validation gates, and explicit separate-feature or separate-executable quarantine rules for any surviving debug export | Active blocker |
| Honest trust-language must replace unsupported claims about trustlessness, receiver-only secrecy, or live STARK/FRI verification | Captured in `D-03`, `D-14`, `D-17`, validation gates, and deferred section | Active blocker |
| Real claim-authority trust anchor and placeholder-authority isolation must be explicit | Captured in `Implementation Guidance And Test Obligations`, `Existing Code Insights`, and `Open Ambiguities And Planning Preconditions` | Active blocker |
| Canonical serialization, ambiguous-encoding rejection, and `z00z_utils` filesystem/serialization boundary must be explicit | Captured in `Implementation Guidance And Test Obligations` | Active blocker |
| Dependency audits, fuzzing, property tests, malformed-proof negatives, and non-test placeholder-mode kill switch must be explicit | Captured in `Implementation Guidance And Test Obligations` plus validation gates | Active blocker |
| Preserve positive verified properties while replacing placeholder crypto | Captured explicitly in `Preserve During Remediation`, including real stealth receive, foreign-receiver rejection, live range-proof verification, nullifier replay resistance, domain separation, and proof-before-ownership sequencing | Preserve during remediation |
| TODO owner map must be carried or explicitly superseded | Captured in `Open Ambiguities And Planning Preconditions` as pending reassignment before planning execution | Planning prerequisite |
| Scenario cryptography-status disclaimer must remain explicit while placeholders exist | Captured in `Implementation Guidance And Test Obligations` | Active blocker |
| Honest censorship/data-availability caveat must survive remediation planning | Captured in `Open Ambiguities And Planning Preconditions` as a non-theft but still important trust-boundary caveat | Active blocker |
| Do not default to a brand-new proof backend or speculative redesign | Captured in phase boundary, `D-23`, and deferred section | Explicitly deferred unless proven necessary |

</transfer_matrix>

<deferred>
## ⏸️ Deferred Ideas

- A brand-new STARK or FRI stack is deferred unless planning proves that the
  current boundary cannot be made truthful without it.
- Any TODO language that assumes live STARK or FRI enforcement is transferred as
  an honesty requirement: planning must either explicitly defer that capability
  or implement it before any summary claims it exists.
- A full formal anonymity proof for all metadata channels is deferred. This
  phase only needs the subset of stealth semantics required to make Scenario 1
  security claims truthful.
- Broader protocol redesign around directory trust, request-bound tags, or card
  rotation is deferred unless one of those items becomes a hard blocker for the
  claim-path or spend-boundary fixes above. Receiver-card and request-binding
  hardening are not dropped; only wider product redesign beyond the truthful
  boundary is deferred.

</deferred>

---

*Phase: 032-crypto-audit-scenario-1*
*Context gathered: 2026-04-05 via current-tree Scenario 1 reconciliation*
