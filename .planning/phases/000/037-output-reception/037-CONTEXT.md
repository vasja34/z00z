# Phase 037: Output Reception - Context

**Gathered:** 2026-04-22
**Status:** Review-locked for planning

## Phase Boundary

Phase 037 schedules and executes the existing output-reception backlog in
`037-TODO.md`. The phase covers request-aware receive orchestration,
ownership-detection truth, persistence gating, public receive boundary cleanup,
and the adjacent documentation or validation work already enumerated in that
file.

This phase does not create a new receive architecture, duplicate existing
wallet logic, or introduce a parallel scan or persistence layer. Planning must
translate the canonical backlog into sequential execution, one canonical task
after another, without renaming, rewriting, or excluding any task title.

`037-ARCHITECTURE.md` now exists as the executed output of `037-01-PLAN.md`
and is the canonical receive ledger for the frozen Task 0 baseline. Later
numbered plans may depend on that ledger as current phase evidence, while
Phase 037 itself still remains queued and review-locked in
`.planning/ROADMAP.md` and `.planning/STATE.md`. That means the numbered plans
still remain preparation artifacts rather than an already-cleared execution
grant for the full phase.

## Implementation Decisions

### Canonical planning inventory

- **D-01:** `037-TODO.md` is the canonical planning inventory for Phase 037.
- **D-02:** The planner must cover every canonical task already listed in
  `037-TODO.md`.
- **D-03:** Task titles and wording from `037-TODO.md` are locked and must not
  be changed during planning.

### Sequencing and execution shape

- **D-04:** Planning must proceed sequentially, scheduling one canonical Phase
  037 task after another rather than parallelizing independent task groups.
- **D-05:** No task may be excluded from the plan set. If a principle blocker
  makes a task impossible to execute honestly, the blocker must be recorded
  explicitly instead of silently skipping the task.

### Architecture and concept-drift guardrails

- **D-06:** Phase 037 must reuse the live receive codebase and its existing
  ownership boundaries rather than duplicating code or reproducing logic in a
  new layer.
- **D-07:** Planning must prevent concept drift: the canonical receive,
  request-binding, and persistence seams remain the existing wallet service,
  scanner, and RPC boundaries already named in `037-TODO.md`.
- **D-08:** Any planning decomposition must stay subordinate to the current
  codebase truth and must not reframe already-implemented receive surfaces as
  greenfield work.

### the agent's Discretion

The planner may choose substeps, validation anchors, and file-order inside each
canonical task, but it has no discretion to rename tasks, merge them into a new
conceptual layer, skip mandatory tasks, or invent extra capabilities outside
the existing Phase 037 backlog.

## Specific Ideas

- Keep the context simple: it exists to lock the planning authority chain, not
  to redesign Phase 037.
- The downstream planner must treat `037-TODO.md` as the source of task truth
  and convert it into numbered plans in the same task order, except where this
  context explicitly delays dependent work such as Task 9 until the later
  decision-gated tasks are closed.
- Prevent duplicate or shadow architectures during planning. Reuse the live
  receive codebase exactly where the backlog points.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase authority

- `.planning/ROADMAP.md` — Phase 037 registration, dependency on Phase 036, and
  queued-phase status.
- `.planning/STATE.md` — current milestone truth; Phase 036 remains active while
  Phase 037 is queued for planning.
- `.planning/phases/037-output-reception/037-TODO.md` — canonical Phase 037
  backlog; every task is mandatory and must be preserved verbatim.

### Receive boundary truth

- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` — live
  `recv_range(...)`, `scan_asset_report(...)`, and compatibility-only
  single-asset receive surfaces.
- `crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs` —
  `recv_route(...)` as the receive-to-persist gate.
- `crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs` —
  canonical `receiver_keys(...)` / `live_receiver_keys(...)` derivation
  boundary.
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs` — canonical
  ownership-detection scanner and request registration surface.
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` — low-level
  request/tag scan helpers, including `scan_dh(...)` ordering.
- `crates/z00z_wallets/src/core/address/stealth_scanner/types_tag_cache.rs` —
  active request tracking and `Tag16Cache` behavior.
- `crates/z00z_wallets/src/core/chain/scan_engine_impl.rs` — current
  not-implemented scan-engine seam that must not be overclaimed.
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
  — public RPC `receive_asset_impl(...)` boundary.

## Review-Locked Corrections

- `WalletService::recv_range(...)` remains the canonical implemented
  request-aware range-receive authority.
- `WalletService::receive_asset(...)` and outward
  `wallet.asset.receive_asset` remain compatibility-only single-asset lanes and
  must not be described as equivalent privacy-aware range receive.
- `StealthOutputScanner::add_request(...)` remains request-liveness metadata
  only. Strict tag-only receive paths require explicit `add_tag_context(...)`
  registration before `scan_leaf_tag_only(...)` is allowed to claim ownership.
- The current request-candidate path is not yet deterministic or expiry-aware:
  `Tag16Cache` tracks active request ids in a `HashSet`, `scan_dh(...)` tries
  the `req_id = None` base candidate before request-bound candidates, and the
  reviewed receive path does not perform scanner-local request expiry pruning.
- Receive detection remains ownership detection plus decrypt, parse, and
  commitment-opening checks. The outward `InvalidProof` vocabulary is a stable
  public reject code, not proof that inline range-proof verification ran inside
  the canonical detector.
- Wallet-native claimed-asset persistence remains the live canonical baseline,
  and the existing compatibility scrub at the persistence boundary must remain
  preserved unless an explicit tested replacement is introduced.
- `ScanEngineImpl` remains stub-only and must stay documented as proposed or
  delegated future work unless parity is actually implemented.
- `OptimizedScanner` remains a batching wrapper over the same canonical
  detector, not a second ownership authority.
- The public single-asset receive adapter remains exact-`asset_id` only.
  Definition-id queries stay rejected unless a new public contract is
  introduced explicitly.
- Orphan duplicate surfaces remain non-canonical evidence traps:
  `wallet_service_actions_runtime.rs` is not wired through
  `wallet_service_actions.rs`, and `asset_impl_tests.rs` is not the bound test
  suite while `asset_impl.rs` binds tests through `test_asset_impl_suite.rs`.

## Sequential Review Gates

1. Tasks 0 through 5 are the baseline-freeze and documentation-rebase gate and
   must complete first.
2. Tasks 6, 7, 8, 11, 12, 13, and 17 are decision-gated tasks. Each must keep
   its chosen outcome explicit: `de-scope`, `thin delegate`, `optional`,
   `future-only`, or `implemented`.
3. Task 9 is a dependent coverage task and must only add tests for gaps that
   remain unresolved after the decision-gated tasks are resolved.
4. Tasks 10, 14, 15, and 16 are cross-cutting integrity gates and must not be
   skipped as "documentation only" work.
5. Tasks 18 and 19 are public-surface and orphan-surface closeout gates and
   must execute against wired live seams only.

## Task Transfer Matrix

Coverage result: 20 canonical tasks transferred, 0 excluded, 0 renamed.

| Task | Exact canonical task | Transfer state | Live-code status | Context carry-over requirement |
| --- | --- | --- | --- | --- |
| 0 | Freeze the implemented Phase 037 baseline before extending it | Transferred | Supported baseline | Preserve `recv_range(...)`, `StealthOutputScanner`, `recv_route(...)`, wallet-native claimed persistence, `ScanStatePayload`, request-aware helpers, and `receiver_keys(...)` as implemented baseline rather than greenfield scope. |
| 1 | Keep `WalletService::recv_range(...)` as the canonical receive path | Transferred | Supported baseline | Keep `recv_range(...)` canonical, rebase docs onto the live service path, keep `WalletService::receive_asset(...)` and public RPC receive compatibility-only, and keep `ScanEngineImpl` explicitly non-parity until implemented. |
| 2 | Materialize request-bound `Tag16Context` explicitly | Transferred | Partially implemented | Preserve `add_request(...)` as metadata only, require explicit `Tag16Context` materialization before strict tag-only scan, reject request-bound strict tag-only flows without contexts, and document that tag16 prefiltering becomes request-aware only after context registration. |
| 3 | Keep proof verification downstream of ownership detection | Transferred | Supported baseline | Keep receive detection focused on ownership classification only, do not add inline range-proof verification, document `Detected` as ownership detection instead of final import validation, and route proof verification to downstream validation boundaries. |
| 4 | Preserve explicit `ReceiveNext::PersistClaim` gating | Transferred | Supported baseline | Keep `recv_route(...)` as the receive-to-persist gate, forbid auto-persist from detection-only results, route all new persistence through the same gate, preserve `ReportOnly` versus `PersistClaim`, and preserve compatibility scrub semantics at persistence. |
| 5 | Re-baseline Phase 037 architecture documentation to live code | Transferred | Supported baseline | Replace stale trait-stack and module-tree claims with live ownership naming around `WalletService::recv_range(...)`, `leaf_scan`, and `StealthOutputScanner`, and mark trait-based scanner variants as superseded or future-only unless code is added. |
| 6 | Implement inbox-assisted receive only at the service or adapter boundary | Transferred | Conditional / open | If inbox-assisted receive stays in scope, keep it as candidate-selection metadata only, place plumbing in a sibling service or adapter, route through existing `recv_range(...)`, keep inbox notify-only, keep helper routing request-bound rather than wallet-global, and do not land assisted receive before Task 15 closes deterministic ordered non-expired candidate selection. |
| 7 | Resolve `ScanEngineImpl` by either de-scoping it or making it a thin delegate | Transferred | Conditional / open | Keep the decision binary and explicit: de-scope `ScanEngineImpl` from Phase 037 docs or implement it only as a thin delegate over the canonical receive path, without duplicating detection, rejection mapping, cursor persistence, or claimed persistence logic. |
| 8 | Decide whether `OptimizedScanner` stays optional or becomes canonical | Transferred | Conditional / open | Keep `OptimizedScanner` either optional or explicitly integrated behind the canonical flow; if promoted, preserve identical output, cursor, rejection, stats, DoS, and persistence semantics and never introduce a separate ownership or persistence implementation. |
| 9 | Add only the missing tests that cover unresolved Phase 037 gaps | Transferred | Conditional / dependent | Reuse existing restart, persistence, route-gate, exact-`asset_id`, status-mapping, and adapter/service parity coverage; prefer extending existing wallet-service tests for service-boundary behavior; place new focused integration tests under `crates/z00z_wallets/tests/` when a new entry point is introduced; add new tests only for newly introduced inbox, scan-engine, facade, callback, or promoted-parallel paths; and do not clone already covered scenarios. |
| 10 | Enforce crypto and security guardrails on every new receive path | Transferred | Supported baseline | Preserve constant-time tag comparisons, keep tag16 and inbox hits as prefilters only, keep proof verification downstream, preserve explicit persistence gating, and route every new receive flow through the same canonical detection core. |
| 11 | Resolve the Section 6.4 storage-model drift without creating a third persistence layer | Transferred | Conditional / open | Reclassify storage text into implemented, future-unification, and stale parts; choose one canonical persistence target; if unified, adapt existing `AssetStorage` boundaries instead of adding a new tree; do not present the old `SpendableAsset` or SQLite design as mandatory current work; preserve compatibility scrub unless explicitly replaced. |
| 12 | Resolve the Section 6.5 reception-API drift with a thin facade or documentation rebase | Transferred | Conditional / open | Keep `Receiver`, `ReceptionConfig`, `ReceptionResult`, and callback API proposed-only unless implemented; any kept facade must be thin over `recv_range(...)`, `scan_asset_report(...)`, outward receive mapping, and claimed queries; delegate progress and persistence to existing service boundaries; keep `WalletService::receive_asset(...)` compatibility-only; do not build a second orchestration stack. |
| 13 | Resolve the scanner-config and DoS-policy drift against the live scanner surface | Transferred | Conditional / open | Either rebase docs onto live knobs such as `max_ckpt`, `DoSMitigation`, explicit request registration, and `background_scan_strategy()`, or keep any facade thin over `StealthOutputScanner` and `OptimizedScanner`; richer `ScanConfig` and `DoSMitigationConfig` must stay future-only unless code exists; tag16, inbox, and parallel knobs remain strategy inputs over the same detection core. |
| 14 | Translate surviving ECC ideas into live surfaces and quarantine stale names | Transferred | Open documentation gate | Retain useful ECC ideas only after translating them into current checkpoint, DA, inbox-hint, and request-bound routing terms, and quarantine stale names such as `receiver::scanner`, `receiver::storage`, `Receiver`, `ReceptionResult`, `OutputScanner`, `FullScanner`, `HybridScanner`, `ScanConfig`, and `DoSMitigationConfig` unless real code is added. |
| 15 | Make request-candidate ordering deterministic and expiry-aware | Transferred | Open live gap | Replace or wrap `HashSet`-driven active-request iteration with a stable candidate order, filter expired requests before iteration, keep `req_id = None` fallback explicit and last, terminate iteration on the first successful candidate, and rebase docs onto the enforced ordering policy. |
| 16 | Add receive-path observability only for actionable rejections | Transferred | Partially implemented | Current RPC receive still emits an operator-facing warning on `ReceiveStatus::NotMine`; Phase 037 must remove or downgrade that signal before claiming the invariant is closed, must map stable actionable failures onto explicit logs or counters, must keep docs anchored to current `ReceiveReject` and `ReceiveReport` vocabulary unless a richer tree is implemented, and must add operator severity guidance for `InvalidInput`, `InvalidProof`, and `RuntimeFail`. |
| 17 | Finish the canonical progress, partial-success, and callback contract | Transferred | Conditional / open | Keep resume persistence anchored to `ScanStatePayload`, decide whether current `ScanRangeOut` and `ScanRangeStat` remain the only implemented progress surface or whether a thin facade is added, route callbacks through canonical cursor/stat progression only, and rebase partial-success language onto live `ScanRangeOut`, `ReceiveReport`, and `WalletResult` contracts. |
| 18 | Rebase the public RPC receive surface onto the refactored service boundary | Transferred | Strong live support | Keep outward `wallet.asset.receive_asset` as the public single-asset API only, map it through `scan_asset_report(...)`, `receiver_keys(...)`, and stable outward status mapping, preserve exact-`asset_id` lookup semantics, reject definition-id queries, and reuse or extend `test_asset_impl_suite.rs`, `asset_impl_server_transfer.rs`, `wallet_service_actions_receive.rs`, and `wallet_service_actions_reachability.rs` as the canonical evidence anchors. |
| 19 | Quarantine orphaned duplicate receive surfaces that are no longer wired | Transferred | Strong live support | Keep `wallet_service_actions_runtime.rs` and `asset_impl_tests.rs` explicitly non-canonical until a keep/remove decision is made, map all docs and future tasks to wired seams only, and require any intentionally retained duplicate file to carry a repository-local non-canonical note. |

## Open Conditional Tasks

- Tasks 6, 7, 8, 11, 12, 13, and 17 remain explicit decision gates and must
  stay labeled conditional or proposed until code and tests close them.
- Task 9 remains dependent work and must only add coverage for real unresolved
  gaps introduced by other in-scope changes.
- Tasks 14, 15, and 16 remain active review gates because stale naming,
  request-order nondeterminism, and actionable observability are not fully
  closed by the current baseline.

## Validation Anchors

- `test_wallet_service_suite.rs` remains the canonical service-boundary suite
  for restart, route-gate, persistence, and `scan_asset_report(...)` coverage.
- `test_asset_impl_suite.rs` remains the canonical adapter suite for public
  receive status mapping, exact-`asset_id` semantics, definition-id rejection,
  and adapter/service parity.
- `test_stealth_scanner_prefilter.rs` remains the proof anchor that
  `add_request(...)` alone is insufficient for strict tag-only scanning and
  that explicit `add_tag_context(...)` is already a live requirement.
- Any future Phase 037 implementation slice must validate against wired live
  seams rather than orphan duplicate files or stub-only placeholders.

## Existing Code Insights

### Reusable Assets

- `WalletService::recv_range(...)`: the current canonical request-aware
  range-receive authority that planning must preserve.
- `WalletService::scan_asset_report(...)`: the live single-asset report and
  reject-classification seam reused by RPC receive flows.
- `recv_route(...)`: the existing explicit `ReportOnly` vs `PersistClaim`
  decision gate for claimed-asset mutation.
- `WalletService::receiver_keys(...)` / `live_receiver_keys(...)`: the canonical
  receiver-key derivation boundary already shared by service and RPC call paths.
- `StealthOutputScanner`: the live ownership detector; `OptimizedScanner`
  remains a wrapper, not a second canonical detector.

### Established Patterns

- Request-aware receive is implemented on the wallet-service path, not through a
  separate scan engine.
- Detection and persistence are intentionally separated: receive detection does
  not imply automatic claimed-asset mutation.
- The outward single-asset RPC lane exists, but it is compatibility-shaped and
  must not be treated as equivalent to the canonical privacy-aware range lane.
- Duplicate or orphaned receive-adjacent files are evidence traps and must not
  be turned into a second implementation authority during planning.

### Integration Points

- Numbered Phase 037 plans should map directly onto the files and seams already
  referenced by `037-TODO.md`.
- Validation should anchor to the live wallet and RPC suites that already
  exercise `recv_range(...)`, `recv_route(...)`, `receiver_keys(...)`, and
  `scan_asset_report(...)` behavior.

## Deferred Ideas

None — this context intentionally fixes planning authority and anti-drift rules
without widening Phase 037 beyond the existing canonical backlog.

---

*Phase: 037-output-reception*
*Context gathered: 2026-04-22*
