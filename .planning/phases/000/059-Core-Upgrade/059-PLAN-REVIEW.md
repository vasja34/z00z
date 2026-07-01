# Phase 059 Plan Review

**Date:** 2026-06-16
**Scope:** `059-TODO.md`, referenced corpus docs, `059-CONTEXT.md`, `059-SOURCE-AUDIT.md`, `059-TEST-SPEC.md`, `059-TESTS-TASKS.md`, and `059-01-PLAN.md` through `059-10-PLAN.md`.

## Verdict

The Phase 059 plan packet is execution-ready after this review pass.

Issues found and fixed:

| ID | Severity | Finding | Fix |
|---|---|---|---|
| F-01 | High | Referenced corpus docs were listed in context but not explicitly imported as per-source planning constraints. | Added a referenced corpus coverage map to `059-CONTEXT.md` and corpus constraints to `059-SOURCE-AUDIT.md`. |
| F-02 | Medium | `059-TESTS-TASKS.md` used fallback names for runtime validator/watcher packages. | Replaced fallback names with actual package names: `z00z_aggregators`, `z00z_validators`, `z00z_watchers`, and `z00z_rollup_node`. |
| F-03 | Medium | Final closeout plan did not explicitly require evidence-ledger mapping for corpus constraints. | Updated `059-10-PLAN.md` so final evidence maps D-IDs, TODO micro rows, and corpus constraints. |
| F-04 | Low | Source-audit plan did not explicitly require reading all corpus docs cited by `059-TODO.md`. | Updated `059-01-PLAN.md` read-first/context and acceptance criteria. |
| F-05 | Medium | The context guaranteed section-level TODO coverage, but the review goal requires stricter list-bullet and table-row traceability. | Added a TODO bullet/table coverage ledger to `059-CONTEXT.md` and required `059-01`/`059-10` to preserve it in source audit and final evidence. |

## TODO Coverage Result

`059-CONTEXT.md` contains a micro coverage map for every `059-TODO.md` section:
key terms, Sections 1 through 12, Appendix A, and Appendix B. It also contains a
TODO bullet/table coverage ledger for the extracted 190 Markdown list bullets
and 112 Markdown table rows in `059-TODO.md`. The numbered plans cover those
rows through source audit, core schema/policy, genesis, storage, typed deltas,
runtime, wallets, simulator, and final evidence closure.

No `059-TODO.md` section is intentionally dropped. Deferrals are explicit:
universal VM-like policy execution, subjective/oracle-heavy policy conditions,
cross-chain bridge semantics, external issuer solvency systems, market pricing
of vouchers, and UI polish outside semantic wallet/RPC boundaries.

## Corpus Coverage Result

| Corpus source | Coverage status |
|---|---|
| `docs/Z00Z-Main-Whitepaper.md` | Covered through wallet-local possession, checkpoint finality, soft-confirmation limits, package/settlement separation, and simulator publication paths. |
| `docs/Z00Z-Smart-Cash-Whitepaper.md` | Covered through bounded smart-cash scope, no universal private VM, typed settlement objects, right/fee separation, and future-proof deferrals. |
| `docs/Z00Z-UseCases-Whitepaper.md` | Covered through budget, grant, allowance, aid/community voucher, service, and agent-right scenario requirements in genesis, wallet, simulator, and UAT plans. |
| `docs/Z00Z-Uniqueness-Whitepaper.md` | Covered through private wallet-local rights, no public-account model, no full-wallet delegation, minimal settlement evidence, and explicit external trust boundaries. |
| `docs/tech-papers/done/Z00Z-HJMT-Design.md` | Covered through one `SettlementStateRoot`, one `SettlementPath`, in-place `SettlementLeaf` family extension, no live `AssetStateRoot`/`AssetPath` revival, proof-family checks, and `RightLeaf`/`FeeEnvelope` separation. |

## Crypto-Architect Gate

Input type: planning documents for a cryptographic/blockchain protocol upgrade.

Material cryptographic requirements incorporated:

- Policy/action descriptors must use canonical bytes and stable hashes.
- Genesis derivation must be domain-separated by network, chain id, object class, object id, index, root generation, and descriptor hash.
- Voucher derivation labels must not reuse asset or right labels.
- Proof and package bindings must include object family, selected action, descriptor hash, prior root, intended deletes/creates/updates, required rights, fee support, and checkpoint context.
- Validators must fail closed for wrong-family proofs, stale roots, unknown policy descriptors, unbacked vouchers, replay, double redeem, right-as-value, and fee-boundary confusion.
- Wallets must quarantine unknown policies and must not convert voucher/right objects into spendable cash.

Crypto verdict: no S0/S1 plan blocker remains. Remaining risks are explicit implementation risks and are routed to tests, simulator negative artifacts, and final review gates.

## Security-Audit Gate

Scope: planning docs only; no production source code was modified.

Security constraints verified:

- No plan introduces a parallel storage authority, wallet-only policy truth, or duplicate runtime semantic authority.
- Unknown policy, malformed descriptors, wrong-family proofs, invalid backing, forced voucher acceptance, and value-bearing rights are all represented as rejection paths.
- Secrets and private openings remain wallet-local; storage validates committed leaves, roots, proofs, and typed deltas only.
- `FeeEnvelope` remains processing support and cannot become voucher backing or right authority.
- Existing wallet asset payload compatibility and durable quarantine are required before typed object RPC surfaces.

Security verdict: no unresolved planning blocker remains. Implementation must still validate with bootstrap tests, targeted release tests, full `cargo test --release`, and repeated `/GSD-Review-Tasks-Execution`.

## Doublecheck Pass 1

Workspace-first claims checked:

| Claim | Status | Evidence |
|---|---|---|
| `059-CONTEXT.md` contains 72 D-ID decisions and a micro coverage map. | Verified | `059-CONTEXT.md` decisions D-01 through D-72 and `Micro Coverage Map For 059-TODO.md`. |
| All 72 context decisions are covered by numbered plans. | Verified | `check.decision-coverage-plan` returned `passed: true`, `covered: 72`, `uncovered: []`. |
| Runtime package names are `z00z_aggregators`, `z00z_validators`, and `z00z_watchers`. | Verified | `crates/z00z_runtime/*/Cargo.toml`. |
| `059-TODO.md` cites the main, smart-cash, use-case, uniqueness, and HJMT design docs. | Verified | `059-TODO.md` Section 2.1 and Appendix B. |

No disputed or fabrication-risk claims found in this pass.

## Doublecheck Pass 2 Against `059-TODO.md`

Adversarial check target: every issue and suggestion in `059-TODO.md` must appear in context and plans.

Result:

- Asset clean-cash semantics are covered by D-05, D-08, D-50 and plans `059-02`, `059-05`, `059-08`, `059-10`.
- Voucher conditional-value semantics, full backing, lifecycle, partial redeem, refund, expiry, and receiver safety are covered by D-06, D-14, D-27, D-44 through D-46, D-57, D-67, D-70 through D-72 and plans `059-02` through `059-05`, `059-08`, `059-09`, `059-10`.
- Right authority-only semantics, delegation, consumption, revocation, expiry, challenge, and zero-value checks are covered by D-07, D-13, D-27, D-28, D-40, D-47, D-58, D-67, D-70 through D-72 and plans `059-02`, `059-03`, `059-05`, `059-08`, `059-09`, `059-10`.
- Policy/action descriptor semantics and trust tiers are covered by D-15, D-49 through D-53 and plans `059-02`, `059-03`, `059-06`, `059-10`.
- Storage and settlement requirements are covered by D-16 through D-20, D-44 through D-48, D-59 through D-63 and plans `059-04`, `059-05`, `059-06`, `059-10`.
- Wallet persistence, scan, quarantine, package builder, RPC, backup, and asset-only balance are covered by D-21 through D-25, D-43, D-64 through D-68 and plans `059-07`, `059-08`, `059-10`.
- Simulator Alice/Bob/Charlie paths and negative evidence are covered by D-26 through D-29, D-37, D-58, D-69 through D-72 and plans `059-09`, `059-10`.
- Validator, aggregator, watcher, and rollup boundaries are covered by D-30 through D-32, D-49 through D-53 and plans `059-06`, `059-09`, `059-10`.
- Tests and validation gates are covered by D-33 through D-37, `059-TEST-SPEC.md`, `059-TESTS-TASKS.md`, and every plan `<verify>` section.

No missing TODO issue remains after the corpus traceability fixes in this review.

## Repeat Review Pass

Date: 2026-06-16.

Workspace-first repeat checks:

- `check.decision-coverage-plan` returned `passed: true`, `covered: 72`, `uncovered: []`.
- All `@crates`, `@docs`, and `@.planning` references in numbered plans resolve to existing workspace paths.
- The referenced corpus sources are present in `059-CONTEXT.md`, `059-SOURCE-AUDIT.md`, `059-01-PLAN.md`, `059-10-PLAN.md`, and this review artifact.
- All numbered plan `<verify>` sections include the mandatory bootstrap fail-fast gate, release test gate, repeated `/GSD-Review-Tasks-Execution`, and `/z00z-git-versioning` commit rule.
- No stale runtime validator/watcher fallback package names remain.
- No duplicate or parallel storage/runtime authority layer was found in the reviewed plan packet.

Repeat verdict: no new issue was found. No additional plan correction was
required in this pass.

## Repeat Review Pass 2

Date: 2026-06-16.

Workspace-first repeat checks:

- Extracted `059-TODO.md` list bullets: 190.
- Extracted `059-TODO.md` Markdown table rows: 112.
- Added `059-CONTEXT.md` TODO bullet/table coverage ledger so the stricter
  "every bullet" review goal is explicit instead of implied by section names.
- Updated `059-01-PLAN.md` to preserve the bullet/table ledger during source
  audit and closeout evidence.
- Updated `059-10-PLAN.md` so `059-EVIDENCE-LEDGER.md` must map D-IDs, micro
  rows, bullet/table groups, corpus constraints, tests, docs, simulator
  artifacts, and explicit deferrals.

Repeat verdict: the only new issue was traceability wording, and it is fixed.

## Demand-Specific Doublecheck Pass 3

Date: 2026-06-16.

Review target: the explicit user requirement that every `059-TODO.md`
position, condition, object, interaction, and implementation warning is visible
in `059-CONTEXT.md` and in the numbered plans, with every plan task telling
the implementer what `059-TODO.md` context must be reread and preserved.

Fix applied in this pass:

- Added `<todo_trace>` to every `<task type="auto">` in `059-01-PLAN.md`
  through `059-10-PLAN.md`.
- Each `<todo_trace>` now names the relevant `059-TODO.md` sections and states
  the concrete obligations the implementer must preserve before editing code.
- `059-10-PLAN.md` now explicitly says Phase 059 cannot close on broad
  section-level coverage alone; it must map every TODO heading, list bullet,
  table row, figure obligation, referenced corpus constraint, and explicit
  deferral into `059-EVIDENCE-LEDGER.md`.

User requirement coverage:

| Requirement | Explicit plan coverage |
|---|---|
| New core types affect `z00z_storage`, `z00z_wallets`, and `z00z_simulator`, not only `z00z_core`. | `059-02` creates core vocabulary, `059-03` genesis publication, `059-04` storage leaves/proofs, `059-05` typed deltas, `059-07`/`059-08` wallet storage/services/RPC/backup, `059-09` simulator, `059-10` closure. |
| Expand tests and simulations to all object types and interactions. | `059-TEST-SPEC.md`, `059-TESTS-TASKS.md`, `059-09`, and `059-10` require positive and negative evidence for Assets, Vouchers, Rights, policies, FeeEnvelope, roots, wallet projections, verdicts, alerts, and simulator artifacts. |
| Process `059-TODO.md` at micro level so no paragraph, bullet, table row, idea, or requirement is lost. | `059-CONTEXT.md` has the micro coverage map and bullet/table ledger; `059-01` preserves it during source audit; `059-10` must close it in `059-EVIDENCE-LEDGER.md`; all plan tasks now include `<todo_trace>`. |
| Wallet structures must adapt to all new object classes and interactions. | `059-07` owns durable typed object inventory and projections; `059-08` owns scan, package builder, RPC, backup, lifecycle, quarantine, and cash-only asset APIs. |
| Simulator must adapt existing simulations to all classes and Alice/Bob/Charlie interactions. | `059-09` expands existing `scenario_1` rather than replacing it; stages 1, 4, 5, 6, 11, and 13 cover all object classes, cross-object actions, positive paths, negative paths, persisted evidence, and proposed fixes. |
| Core genesis must support all object classes, and not every class has limited supply like native assets. | `059-03` explicitly preserves asset finite-supply semantics while defining right authority-instance genesis and voucher conditional-claim/bootstrap semantics with backing/reserve evidence. |
| New policies, vouchers, and rights must be generated deterministically. | `059-02` defines policy/action descriptors; `059-03` adds deterministic policy generation, bootstrap voucher generation, right compatibility, domain-separated derivation, and manifest export. |
| Decide whether genesis stays unified or each object class gets its own birthplace. | `059-03` chooses one shared `z00z_core::genesis` orchestration boundary with per-object typed generators and a single settlement manifest. |
| Prevent duplicated codebase logic and concept drift. | `059-01` source audit, `059-04` one settlement root/path, `059-07` one wallet inventory facade, `059-09` existing simulator stage adaptation, and `059-10` final evidence closure all prohibit parallel object layers. |

## Core Module Root Doublecheck Pass 4

Date: 2026-06-16.

Review target: verify the explicit user request that Phase 059 plans adequately
cover these core paths: `crates/z00z_core/src/actions`,
`crates/z00z_core/src/assets`, `crates/z00z_core/src/genesis`,
`crates/z00z_core/src/policies`, `crates/z00z_core/src/rights`, and
`crates/z00z_core/src/vauchers`.

Workspace-first finding:

- `assets` and `genesis` already had live source files and explicit plan
  ownership.
- `actions`, `policies`, `rights`, and `vauchers` existed as directories but had
  no files in the inspected workspace.
- `059-02` previously planned policy/action/voucher types only under
  `z00z_core::assets`, which risked leaving the empty module roots unresolved or
  creating a later parallel vocabulary.

Fix applied in this pass:

- Updated `059-02-PLAN.md` so it explicitly owns population or intentional
  re-exporting of `z00z_core::actions`, `z00z_core::policies`,
  `z00z_core::rights`, and `z00z_core::vauchers`, with one canonical
  implementation per descriptor/config/lifecycle concept.
- Updated `059-03-PLAN.md` so genesis must consume canonical action, policy,
  right, and voucher types from those module roots instead of defining private
  genesis-local duplicates.
- Updated `059-CONTEXT.md` crate-impact and planning acceptance criteria so the
  module-root requirement is visible before implementation starts.

Pass 4 verdict: fixed. Phase 059 plans now require all six listed core paths to
be populated or intentionally re-exported without codebase concept drift.

Section-by-section coverage check:

| `059-TODO.md` section | Primary plan evidence |
|---|---|
| Key Terms Used In This Paper | `059-01`, `059-02`, `059-10` |
| 1. Why This Paper Exists | `059-01`, `059-02`, `059-10` |
| 1.1 Design Problem | `059-01`, `059-02`, `059-04`, `059-07`, `059-09`, `059-10` |
| 1.2 Design Thesis | `059-02` through `059-10` |
| 1.3 Reader Outcome | `059-01`, `059-10` |
| 2. Position In The Z00Z Corpus | `059-01`, `059-10` |
| 2.1 Corpus Role | `059-01`, `059-10` |
| 2.2 Current Maturity Versus Target Architecture | `059-01`, `059-10` |
| 3. Core Thesis: Asset, Voucher, And Right | `059-02`, `059-05`, `059-08`, `059-09`, `059-10` |
| 3.1 The Minimal Triad | `059-02`, `059-10` |
| 3.2 Why This Split Is Minimal | `059-02`, `059-05`, `059-10` |
| 3.3 Why Voucher Is Not Redundant With Right | `059-02`, `059-05`, `059-09`, `059-10` |
| 3.4 Cross-Object Binding Rules | `059-03`, `059-05`, `059-06`, `059-08`, `059-09`, `059-10` |
| 4. Asset: Final Value And Cash Boundary | `059-02`, `059-08`, `059-09`, `059-10` |
| 4.1 What Asset Means | `059-02`, `059-07`, `059-08`, `059-09` |
| 4.2 Why Asset Must Stay Clean | `059-02`, `059-08`, `059-10` |
| 4.3 Cash-Grade Invariants | `059-02`, `059-03`, `059-05`, `059-09`, `059-10` |
| 4.4 What This Paper Does Not Claim About Assets | `059-02`, `059-10` |
| 5. Voucher: Conditional Value, Not Dirty Cash | `059-02`, `059-03`, `059-04`, `059-05`, `059-08`, `059-09`, `059-10` |
| 5.1 Economic Meaning | `059-02`, `059-08`, `059-09`, `059-10` |
| 5.2 Fully Backed Vouchers | `059-02`, `059-03`, `059-05`, `059-06`, `059-09` |
| 5.3 Voucher Is Not Final Cash | `059-02`, `059-07`, `059-08`, `059-09` |
| 5.4 Voucher Lifecycle | `059-02`, `059-05`, `059-08`, `059-09` |
| 5.5 Partial Redeem | `059-02`, `059-03`, `059-05`, `059-09` |
| 5.6 Why Vouchers Are Better Than Encumbered Cash | `059-02`, `059-05`, `059-10` |
| 6. Right: Authority Without Value | `059-02`, `059-03`, `059-05`, `059-08`, `059-09`, `059-10` |
| 6.1 What Right Means | `059-02`, `059-03`, `059-07`, `059-08` |
| 6.2 Stateless And Stateful Rights | `059-03`, `059-05`, `059-08`, `059-09` |
| 6.3 Rights And Delegation | `059-02`, `059-05`, `059-08`, `059-09` |
| 6.4 Why Right Does Not Duplicate Voucher | `059-02`, `059-07`, `059-10` |
| 7. Policy, ActionPool, And Condition Model | `059-02`, `059-03`, `059-05`, `059-06`, `059-08`, `059-09`, `059-10` |
| 7.1 Fixed CashPolicy For Native Asset | `059-02`, `059-03`, `059-08` |
| 7.2 VoucherPolicy And ActionPool | `059-02`, `059-03`, `059-06`, `059-09` |
| 7.3 Core-Safe Condition Classes | `059-02`, `059-06`, `059-10` |
| 7.4 Validator And Wallet Responsibilities | `059-06`, `059-08`, `059-09` |
| 7.5 Minimum Policy Contract Surface | `059-03`, `059-06`, `059-10` |
| 7.6 Minimum Action Semantics | `059-05`, `059-06`, `059-09` |
| 7.7 Package And Witness Boundary | `059-05`, `059-06`, `059-08`, `059-09` |
| 7.8 Separate Fee-Support Boundary | `059-05`, `059-09`, `059-10` |
| 8. Payment, Acceptance, And Receiver Safety | `059-07`, `059-08`, `059-09`, `059-10` |
| 8.1 Clean Payment Versus Voucher Transfer | `059-08`, `059-09`, `059-10` |
| 8.2 One-Sided Cash Stays | `059-08`, `059-09`, `059-10` |
| 8.3 Refund Is Not Arbitrary Clawback | `059-08`, `059-09`, `059-10` |
| 8.4 Unknown Policy And Wallet Quarantine | `059-07`, `059-08`, `059-09`, `059-10` |
| 9. Storage And Settlement Architecture | `059-04`, `059-05`, `059-06`, `059-09`, `059-10` |
| 9.1 One Settlement-Root Contract And Semantic Object View | `059-04`, `059-09`, `059-10` |
| 9.2 Live HJMT Leaves And The Voucher Target | `059-04`, `059-09`, `059-10` |
| 9.3 What Belongs In Canonical State | `059-04`, `059-05`, `059-10` |
| 9.3.1 Per-Object Storage Split | `059-04`, `059-07`, `059-10` |
| 9.4 Why Policies And ActionPool Live Mostly Outside The Committed State | `059-03`, `059-04`, `059-06`, `059-10` |
| 9.5 Conservation And Supply | `059-03`, `059-04`, `059-05`, `059-10` |
| 9.6 Why Not Nested Rights Or Nested Vouchers | `059-05`, `059-10` |
| 9.7 Where Objects Live And Who Uses Them | `059-04`, `059-07`, `059-08`, `059-10` |
| 9.8 End-To-End Role Path | `059-06`, `059-08`, `059-09`, `059-10` |
| 9.9 Admission, Verdict, And Alert Surfaces | `059-06`, `059-09`, `059-10` |
| 10. Security Boundary And Non-Goals | `059-02`, `059-03`, `059-05`, `059-06`, `059-09`, `059-10` |
| 10.1 What Validators Must Verify | `059-05`, `059-06`, `059-09`, `059-10` |
| 10.2 What Core Z00Z Should Refuse | `059-02`, `059-03`, `059-05`, `059-06`, `059-08`, `059-10` |
| 10.3 Residual Risks | `059-06`, `059-10` |
| 10.4 Non-Goals | `059-10` |
| 11. MVP Recommendation | `059-03`, `059-08`, `059-09`, `059-10` |
| 11.1 MVP Object Set | `059-03`, `059-09`, `059-10` |
| 11.2 MVP Use-Case Priority | `059-03`, `059-08`, `059-09`, `059-10` |
| 11.3 Future Expansion | `059-10` |
| 11.4 From Whitepaper To Full Spec | `059-03`, `059-07`, `059-08`, `059-10` |
| 12. Conclusion | `059-10` |
| Appendix A. Core Claims And Non-Claims | `059-01`, `059-02`, `059-10` |
| Appendix B. Reading Map | `059-01`, `059-10` |

Paragraph/list/table-row doublecheck result:

- `059-CONTEXT.md` retains the paragraph-level coverage rule: broad section
  references are insufficient for closeout.
- `059-CONTEXT.md` tracks the extracted 190 Markdown list bullets and 112
  Markdown table rows from `059-TODO.md` as coverage groups.
- `059-01-PLAN.md` requires preserving that ledger during source audit.
- `059-10-PLAN.md` requires final evidence mapping for every TODO heading,
  micro row, bullet/table group, corpus constraint, test, doc, simulator
  artifact, explicit non-goal, and deferral.

Pass 3 verdict: after adding `<todo_trace>` blocks, every numbered plan task
has explicit local instructions for which `059-TODO.md` context to reread and
which obligations to preserve. No missing section-level plan coverage remains.

## Residual Risk

This is a planning review, not implementation proof. The actual guarantee must
be re-established after code changes by running the mandated bootstrap gate,
targeted release tests, full release test suite, simulator evidence, final
evidence ledger, and repeated GSD execution review.
