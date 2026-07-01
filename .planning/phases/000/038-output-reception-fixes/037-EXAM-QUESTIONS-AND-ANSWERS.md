# Phase Final Exam

**Phase:** `037-output-reception`
**Generated:** `2026-04-23`
**Scope Sources:** `037-CONTEXT.md`, `037-TODO.md`, `037-STORY.md`, `037-ARCHITECTURE.md`, `037-TEST-SPEC.md`, `037-VALIDATION.md`, `037-SECURITY.md`, `037-UAT.md`, `037-REVIEW.md`, `037-TEST-EXECUTION-SUMMARY.md`, `037-FULL-AUDIT.md`, `037-CONCEPT-DRIFT-REPORT.md`, and live receive/test evidence in `z00z_wallets`

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

Pressure-test whether Phase 037 really keeps output reception singular, bounded,
and honest on the live tree. The solver must distinguish delivered receive
truth from compatibility-only surfaces, decision-gated future work, partial
test closure, and documentation that would become overclaim if read more
strongly than the code and evidence allow.

## ⛔ Constraints

- Every answer must be proved from repository evidence rather than lifted from
  summary prose alone.
- The question bank must distinguish canonical receive authority from
  compatibility, wrapper, stub, or duplicate surfaces.
- A green narrow rerun or a closed threat row is not enough by itself; answers
  must also account for partial validation state, pending UAT, residual Task 9
  waves, and wording honesty.
- The question wording must not spoon-feed the exact file, helper, test,
  requirement row, or symbol that resolves the answer.

## Scope Note

This exam verifies the live Phase 037 story for canonical range receive,
explicit persistence gating, deterministic request selection, public
single-asset compatibility behavior, observability severity boundaries,
duplicate-surface quarantine, and honest partial-closeout language. It is also
designed to expose whether any future-only or conditional branch is being read
as delivered functionality without corresponding live proof.

## 🔍 Answering Standard

- Answers must discover their own evidence path through the repository.
- Questions are intentionally phrased at the level of guarantees, boundaries,
  scenario closure, replay and continuity risk, and documentation honesty
  rather than file-by-file breadcrumbs.
- A correct answer may conclude that a claim is only partially true, remains
  future-only, or is overstated, provided that conclusion is proved from the
  live repository state.

## 🎯 Theme 1: Closure And Scope Honesty

### 1. Delivered Closure Versus Residual Partiality

🔴 **Quest:** What exactly does Phase 037 close on the live tree today, and which stronger interpretations would be false even though the phase has a frozen architecture ledger, a verified security contract, and several green focused validation slices?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Phase 037 closes the frozen `z00z_wallets` receive baseline on the live tree, not the whole output-reception backlog as a universally complete phase. Closed today are the implemented receive guardrails: `WalletService::recv_range(...)` remains the only canonical range receive authority; persistence stays behind `recv_route(..., ReceiveNext::PersistClaim)`; the outward single-asset receive path stays compatibility-only; request selection is deterministic, expiry-aware, and fallback-last; `ReceiveReject::NotMine` stays non-alerting while actionable rejects remain operator-visible; and orphan duplicate receive surfaces remain explicitly non-canonical.

**Reasoning:** The repository closes a bounded receive slice, not blanket Phase 037 finality. `037-ARCHITECTURE.md` freezes the implemented ledger around the canonical `recv_range(...)` lane, the explicit persistence gate, the compatibility-only single-asset path, deterministic request ordering, and duplicate-surface quarantine. Live code in `wallet_service_actions_receive.rs`, `wallet_service_actions_reachability.rs`, and `asset_impl_server_transfer.rs` matches that ledger. But the stronger reading that "Phase 037 is fully closed" is false. `037-VALIDATION.md` still says `status: partial` and `nyquist_compliant: false`; `037-TEST-EXECUTION-SUMMARY.md` says only Task 9 Wave T1 plus the narrow current T5 slice landed; and `037-UAT.md` shows all five user-facing proof items still pending. So it is also false that inbox-assisted receive, `ScanEngineImpl` parity, or full Task 9/UAT/end-to-end closure are delivered merely because the architecture ledger is frozen, the threat register is verified, and several focused validation slices are green.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 2. Security Closure Versus Phase Closure

🔴 **Quest:** How can the phase legitimately report a fully closed threat register while still remaining partial overall, and what repository evidence forces those two status layers to stay separate?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Phase 037 can legitimately report a closed threat register because the security artifact closes only the threats declared in the Phase 037 plan artifacts. That does not upgrade the phase to full closure, because the repository keeps a separate closeout layer for validation, residual coverage, and user-facing proof.

**Reasoning:** `037-SECURITY.md` is scope-bounded: it says it verifies only the threats declared in the Phase 037 plan artifacts and records `status: verified` with `threats_open: 0`. But `037-VALIDATION.md` separately remains `status: partial` with `nyquist_compliant: false`, and its requirement map keeps Task 9 only partially closed. `037-TEST-EXECUTION-SUMMARY.md` says only Wave T1 plus the narrow current T5 slice landed, while later backlog waves remain open. `037-UAT.md` still has five pending user-facing proof items. `037-REVIEW.md` is also clean only for that same narrow landed slice, and the latest `037-FULL-AUDIT.md` still reports overall status `partial`. So the repository forces two honest status layers: declared threats are closed for the implemented receive slice, but phase closure remains partial because validation, residual test coverage, and UAT are not finished.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 3. Evidence Tiers For Honest Closeout

🔴 **Quest:** What are the distinct evidence tiers that must be kept apart before someone can claim full Phase 037 closure, and what wrong conclusion follows if plan summaries, focused reruns, full-workspace sweeps, and pending UAT are treated as interchangeable?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Full Phase 037 closure requires keeping four evidence tiers separate: the frozen implemented receive baseline, the summary-backed numbered-plan and phase-local focused proof slice, the broader repo-wide release rerun, and the still-open UAT closure gate. If those tiers are treated as interchangeable, a solver can falsely conclude that a summary-backed and narrowly green slice already proves full phase closure.

**Reasoning:** The frozen implementation baseline is the canonical `recv_range(...)` lane with explicit `PersistClaim` gating and compatibility-only single-asset receive, as recorded in `037-ARCHITECTURE.md` and reflected in `wallet_service_actions_receive.rs`. The next tier is phase-local proof: `037-VALIDATION.md`, `037-REVIEW.md`, and `037-TEST-EXECUTION-SUMMARY.md` explicitly limit the landed evidence to the numbered-plan chain plus the Task 9 T1 and narrow T5 slice. A separate tier is the broader repo-wide release rerun, which is currently green, but the validation and full-audit artifacts explicitly say that this still must not be read as full Phase 037 closure. The last tier is UAT, and `037-UAT.md` still records five pending proof obligations. Collapsing these tiers would overclaim that green focused reruns or a green workspace sweep already equal full closure, even though residual Task 9 waves and all UAT obligations remain open.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 4. Reopen Conditions

🔴 **Quest:** What concrete contradiction, regression, or newly discovered overclaim would be sufficient to reopen Phase 037 honestly, even if the current phase package still looks clean at a glance?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Phase 037 should be reopened as soon as repository evidence shows either a material overclaim in the phase package or a contradiction between live code and the frozen receive ledger. One such contradiction is enough if it breaks the single canonical `recv_range(...)` lane, the explicit `recv_route(..., ReceiveNext::PersistClaim)` persistence gate, the compatibility-only status of the public single-asset receive path, the non-canonical status of duplicate surfaces unless explicitly rewired, or the deterministic request contract that prunes expired requests and keeps the fallback last.

**Reasoning:** `037-FULL-AUDIT.md` already proves that documentary overstatement is itself material: the audit found and corrected a validation overclaim about broader proof state. The same reopen standard applies to live-behavior drift. `037-ARCHITECTURE.md`, `037-VALIDATION.md`, `037-TEST-EXECUTION-SUMMARY.md`, and `037-UAT.md` define the current honest boundary: one canonical receive lane, explicit persistence gating, compatibility-only outward receive, deterministic request selection, duplicate-surface quarantine, partial Task 9 closure, and five still-pending UAT items. So reopen would also be required if a second canonical receive authority appeared, if detection began mutating claimed state without `recv_route(...)`, if the public single-asset path were promoted to parity without new proof, if duplicate surfaces became authoritative without rewiring, or if pending Task 9/UAT work were described as closed without new evidence. The package stays honest only while live code, the architecture ledger, and the partial-closeout artifacts continue to agree.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 5. Future-Only Branches Versus Shipped Behavior

🔴 **Quest:** Which major receive-adjacent ideas are intentionally left conditional, deferred, or future-only in this phase, and what would count as documentary dishonesty if any of them were described as already delivered?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Phase 037 intentionally leaves multiple receive-adjacent ideas outside current delivered canonical closure: inbox-assisted receive is explicitly deferred and future-only until a real live hint source exists; `ScanEngineImpl` remains stub-only, proposed-only, and non-parity; `AssetStorage` remains a future-unification seam rather than the current claimed-persistence target; the documented high-level reception facade vocabulary (`Receiver`, `ReceptionConfig`, `ReceptionResult`, callback/event APIs) remains proposed-only unless real code is added; the richer scanner/config vocabulary (`ScanConfig`, `DoSMitigationConfig`, `receiver::scanner`, `receiver::storage`) remains future-only or superseded unless implemented; historical names such as `OutputScanner`, `FullScanner`, and `HybridScanner` remain superseded vocabulary rather than live receive surfaces; and `OptimizedScanner` remains only an optional batching wrapper over the canonical detector, not a second or canonical receive lane.

**Reasoning:** `037-TODO.md` explicitly selects the defer branch for inbox-assisted receive and the de-scope branch for `ScanEngineImpl`, requires the reception-facade vocabulary to stay proposed-only until code exists, and keeps richer scanner/config names future-only unless implemented. `037-ARCHITECTURE.md` freezes the same boundaries by marking inbox hints future-only, `ScanEngineImpl` stub-only/non-parity, `AssetStorage` future-unification only, `OptimizedScanner` optional rather than canonical, and the historical reception/scanner names as superseded or planning-only vocabulary. `037-VALIDATION.md` reinforces those boundaries as validated branch-control decisions. Documentary dishonesty would therefore be any Phase 037 text that presents any of those seams as implemented, parity-complete, canonical, or current-live Phase 037 behavior without corresponding code, tests, and explicit promotion evidence. The same dishonesty would occur if compatibility-only or optional surfaces were silently promoted in prose into the canonical privacy receive path.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

## 🔑 Theme 2: Canonical Receive Authority

### 6. One Canonical Range Authority Or More Than One

🔴 **Quest:** What repository evidence proves that there is still exactly one canonical range-receive authority, and what observation would show that a second authority had quietly appeared?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The repository still shows exactly one canonical range-receive authority because the architecture ledger names `WalletService::recv_range(...)` as the canonical Phase 037 lane, the UAT requires that the live flow keep exactly one canonical range lane and that no second range authority appear, and the live service code matches that contract by loading `ScanStatePayload`, scanning with `StealthOutputScanner`, replaying hits, and persisting only through `recv_route(..., ReceiveNext::PersistClaim)`.

**Reasoning:** `037-ARCHITECTURE.md` freezes `recv_range(...)` as the canonical receive lane and records the full flow through receiver-key derivation, scanner use, replay, `recv_claim_asset(...)`, `recv_route(..., ReceiveNext::PersistClaim)`, and scan-state persistence. `wallet_service_actions_receive.rs` mirrors that by documenting `recv_range(...)` as the canonical request-aware lane and by implementing the actual scan and persistence path there. Neighboring seams are explicitly kept subordinate: `scan_asset_report(...)`, `receive_asset(...)`, and outward `wallet.asset.receive_asset` stay compatibility-only; `ScanEngineImpl` stays stub-only and non-parity; and the RPC single-asset adapter routes through `scan_asset_report(...)` instead of replacing the range lane. A second authority would therefore be visible if another live seam were documented or wired as canonical/parity, claimed to replace `recv_range(...)`, or started doing range scanning plus claimed persistence outside the `recv_range(...)` to `recv_route(..., PersistClaim)` path.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 7. Ownership Detection Versus Final Validation

🔴 **Quest:** Where does the live tree draw the line between ownership detection, outward receive classification, and later proof or import validation, and why would collapsing those boundaries create an overclaim about what receive actually proves?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The live tree keeps a strict boundary between canonical ownership detection, outward receive classification, and downstream validation. Receive proves ownership detection plus outward receive classification at this seam; it does not prove final import success or downstream proof validity.

**Reasoning:** `leaf_scan.rs` shows the boundary directly. `receiver_scan_leaf(...)` decides ownership by returning an owned pack or no pack, while `receiver_scan_report(...)` converts that detector outcome into a `ReceiveReport` with `ReportOnly` next-step semantics, so this seam stops at detection and classification rather than persistence or downstream proof verification. In `wallet_service_actions_receive.rs`, `scan_asset_report(...)` is explicitly a compatibility-only single-asset classification lane, while `recv_range(...)` is the canonical request-aware receive lane. In `asset_impl_server_transfer.rs`, the RPC path consumes `scan_asset_report(...)` for outward receive behavior and, on `Detected`, reconstructs the owned output only to build the public response; it does not promote receive into a full import or proof-validation verifier. `037-ARCHITECTURE.md` states the same rule explicitly: receive detection and classification stay separate from downstream import, tx validation, and proof-verification boundaries. Collapsing those layers would therefore overclaim that a receive result proves final import readiness or proof correctness, when the live seam proves only ownership detection plus outward receive classification.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 8. Detection Versus Persistence Mutation

🔴 **Quest:** What evidence proves that detection alone cannot mutate claimed state, and what mutation pattern would demonstrate that the explicit persistence gate has stopped being authoritative?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** In the live canonical receive flow, detection alone cannot mutate claimed state. Detection/report paths stop at `ReceiveNext::ReportOnly`, and claimed persistence is reached only when the flow explicitly crosses `recv_route(..., ReceiveNext::PersistClaim)`.

**Reasoning:** `leaf_scan.rs` maps full-leaf detection/report outcomes to `ReceiveReport` values whose `next` is `ReceiveNext::ReportOnly`. `037-ARCHITECTURE.md` freezes the live ledger as replay -> `recv_claim_asset(...)` -> `recv_route(..., ReceiveNext::PersistClaim)` and separately states that claim persistence stays behind `recv_route(...)`. `wallet_service_actions_receive.rs` matches that contract: `claim_scan_hits(...)` checks `scanner.scan_leaf(...)`, derives a claim candidate with `recv_claim_asset(...)`, and only then calls `self.recv_route(wallet_id, asset, ReceiveNext::PersistClaim).await?`. `wallet_service_actions_reachability.rs` makes the gate explicit: `ReportOnly` returns `Ok(false)`, while `PersistClaim` alone calls `put_claimed_asset(...)`, the actual wallet-native claimed-state mutator. A mutation pattern that would break this guarantee would be any live detection/report path that writes claimed assets directly after `scan_leaf(...)` or report classification, calls `put_claimed_asset(...)` without going through `recv_route(..., PersistClaim)`, writes `wallet_claimed_assets` directly, or changes `ReportOnly` so it persists state.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 9. Shared Receiver Boundary Without Semantic Drift

🔴 **Quest:** How does the repository let multiple receive-adjacent surfaces depend on the same receiver-side key boundary without allowing any one of those surfaces to redefine the canonical receive story?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The repository allows multiple receive-adjacent surfaces to share one receiver-side key boundary by centralizing receiver key derivation in `receiver_keys(...) -> live_receiver_keys(...)`, while keeping canonical receive authority attached to `recv_range(...)` rather than to key access itself.

**Reasoning:** In `wallet_service_actions_receiver.rs`, `receiver_keys(...)` is the authoritative wallet-native receiver material for both `scan_asset_report(...)` and `recv_range(...)`. But `wallet_service_actions_receive.rs` keeps the semantic split explicit: `scan_asset_report(...)` is a compatibility-only single-asset lane, while `recv_range(...)` is the preferred canonical request-aware receive lane and the only lane that owns request registration, replay gating, claimed persistence through `recv_route(..., ReceiveNext::PersistClaim)`, and scan-state cursor handling. `037-ARCHITECTURE.md` freezes the same rule at the architecture level: compatibility surfaces may reuse live receiver keys or detector helpers, but they do not define the canonical receive architecture. `asset_impl_server_transfer.rs` matches that contract in code, because the public RPC path reuses `scan_asset_report(...)` and `receiver_keys(...)` for exact-asset outward status/output reconstruction without becoming the canonical range receive lane. `037-VALIDATION.md` then closes the split as validated repository truth by separately marking the canonical `recv_range(...)` lane, the compatibility-only `wallet.asset.receive_asset` path, and the duplicate-surface non-canonical quarantine as covered.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 10. Compatibility Receive Without Promotion

🔴 **Quest:** What keeps the public single-asset receive lane compatibility-only instead of allowing it to become a silent replacement for the canonical privacy-aware receive path?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The public single-asset receive lane stays compatibility-only because Phase 037 freezes it as an exact-asset outward adapter over shared receiver logic, while the canonical privacy-aware receive path remains the request-aware `recv_range(...)` lane with replay, explicit `recv_route(..., ReceiveNext::PersistClaim)` gating, and `ScanStatePayload` cursor persistence. The compatibility path may reuse `scan_asset_report(...)`, `receiver_keys(...)`, and the shared detector, but it remains explicitly non-canonical.

**Reasoning:** `037-ARCHITECTURE.md` explicitly lists `scan_asset_report(...)`, `WalletService::receive_asset(...)`, and outward `wallet.asset.receive_asset` under compatibility-only surfaces and says they do not define the canonical receive architecture. `wallet_service_actions_receive.rs` repeats the same split by documenting `scan_asset_report(...)` as a compatibility-only lane and `receive_asset(...)` as a compatibility-only reachability surface, not the canonical Phase 037 execution path. In code, `asset_impl_server_transfer.rs` handles exactly one located asset: it calls `lookup_receive_asset(...)`, runs `scan_asset_report(...)`, fetches `receiver_keys(...)`, and returns outward status/output metadata for that single asset. It does not become the request-aware range lane, does not own scan-state resume logic, and does not replace the `recv_range(...) -> recv_claim_asset(...) -> recv_route(..., PersistClaim)` flow. `037-UAT.md` and `037-VALIDATION.md` keep this split separately named and separately checked by treating the canonical range lane and the compatibility-only public single-asset path as different proof surfaces. Silent promotion would therefore require new code and validation that explicitly reclassify that public single-asset path as part of the canonical receive architecture.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

## ♻️ Theme 3: Candidate Selection, Continuity, And Replay Pressure

### 11. Deterministic Candidate Contract

🔴 **Quest:** What is the current contract for request-aware candidate selection, and which parts of that contract would have to drift before the repository should reject the phase's ordering claims as no longer true?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** In the current live request-aware receive lane, only non-expired `PaymentRequest`s enter the active request iteration set, active request ids are iterated in stable `BTreeSet` order, request-bound candidates are evaluated before the generic `req_id = None` fallback, and that fallback remains explicit and last.

**Reasoning:** `Tag16Cache` stores active request ids in a `BTreeSet`, `add_request(...)` returns early for expired requests, and `test_active_requests_are_sorted_and_skip_expired` proves sorted iteration with expired requests excluded. The live scanner feeds `self.tag16_cache.active_requests().copied()` into `scan_owned(...)`, which materializes candidates through `ordered_request_candidates(...)`; that helper appends every `derive_k_dh_with_req(...)` candidate first and the plain `derive_k_dh(...)` fallback last, with an explicit anti-shadowing comment, and `ordered_request_candidates_puts_fallback_last` locks that order in tests. `wallet_service_actions_receive.rs` also filters expired requests before `scanner.add_request(...)`, and `037-ARCHITECTURE.md` freezes the same policy as deterministic, expiry-aware, and fallback-last. The phase ordering claims should therefore be rejected if expired requests can re-enter the live active iteration set, if the live path stops iterating the ordered active-request set, or if the generic fallback can run before live request-bound candidates.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: The first sentence is scoped to the live `PaymentRequest` receive lane, not every auxiliary cache mutation API.

### 12. Expiry Pruning And Fallback Discipline

🔴 **Quest:** How does the live tree prove that expired requests cannot still influence ownership outcomes and that the generic fallback candidate cannot shadow active request-bound matches?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** In the live receive lane, expired requests do not influence request-aware ownership detection because they are pruned before entering the active request-id set, and the generic fallback cannot shadow an active request-bound match because request-bound candidates are evaluated first and scanning returns on the first successful match.

**Reasoning:** `wallet_service_actions_receive.rs` filters expired requests before registration, and `Tag16Cache::add_request(...)` independently drops expired requests. `test_active_requests_are_sorted_and_skip_expired` proves that expired request ids are absent from the active set. `stealth_scanner.rs` then feeds that active set into request-aware scanning through `self.tag16_cache.active_requests().copied()`. On the fallback side, `ordered_request_candidates(...)` appends all `derive_k_dh_with_req(...)` candidates before the plain `derive_k_dh(...)` fallback, `ordered_request_candidates_puts_fallback_last` locks that sequence, and `scan_cached_keys(...)` returns immediately on the first `DetectState::Mine`, so a later generic fallback cannot preempt or overwrite an earlier active request-bound success. `037-ARCHITECTURE.md` freezes the same expiry-aware, fallback-last policy as live receive truth.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 13. First-Win Semantics And Ownership Stability

🔴 **Quest:** Why is first-win short-circuiting part of the receive truth rather than a local optimization detail, and what evidence would show that a later candidate can still rewrite an already-successful ownership decision?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** First-win short-circuiting is part of receive truth, not a local optimization, because the live ownership contract is “evaluate deterministic candidates in order and stop at the first `Mine`.” Once an earlier candidate succeeds, the scanner must not continue into later candidates that could determine the returned ownership result.

**Reasoning:** `stealth_scan_support.rs` puts request-bound candidates before the request-less fallback in `ordered_request_candidates(...)`, and `scan_cached_keys(...)` returns immediately on the first `DetectState::Mine`. That makes candidate order semantically real instead of advisory only. `scan_cached_keys_first_win` proves the contract by using a panic-after-first iterator and demonstrating that scanning must not read past the first winner. `037-ARCHITECTURE.md` freezes the same deterministic request-ordering and fallback-last policy as live receive behavior. Evidence of drift would therefore be any code or regression test showing that `scan_cached_keys(...)` continues after a `Mine`, that a later candidate is still consumed after the first winner, or that a later candidate path can determine the returned ownership result after an earlier success.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 14. Resume Continuity Versus Replay Confusion

🔴 **Quest:** How do persisted progress state and claimed-state mutation interact in the canonical receive flow, and what stale-artifact or replay misunderstanding would expose a real continuity gap?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** In the canonical receive flow, persisted progress state and claimed-state mutation are coordinated but not the same artifact. `ScanStatePayload` records where `recv_range(...)` should resume, while claimed-state changes occur only when replayed in-range hits cross `recv_route(..., ReceiveNext::PersistClaim)` into wallet-native claimed storage.

**Reasoning:** `recv_range(...)` loads the saved cursor, computes the replay start, scans the requested range, replays only the in-scope leaves through `claim_scan_hits(...)`, and saves the updated cursor only after that replay completes. The replay path turns detected leaves into claim candidates with `recv_claim_asset(...)`, and claimed-state mutation remains explicit because `recv_route(...)` mutates state only on `PersistClaim`; `ReportOnly` does not. The architecture and validation artifacts keep these as separate proof surfaces: cursor/progress remain in `ScanStatePayload` and `ScanRangeStat`, while claimed persistence stays behind the explicit persistence gate. The test contract names the same continuity rule as monotonic cursor plus one-time claim persistence, and `test_recv_range_restart` proves restart continuity by resuming from the saved checkpoint while preserving a unique claimed set across both runs.

**Gap Or Blocker:** None.

**Verification:** verified against live code, restart tests, and phase artifacts. Residual caveat: None.

### 15. Metadata Hints Versus Ownership Proof

🔴 **Quest:** Under what conditions could request metadata, tag metadata, or future hint-style inputs become dangerous pseudo-ownership signals, and what evidence proves that the current phase still prevents that escalation?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Request metadata, tag metadata, or future hint-style inputs become dangerous pseudo-ownership signals if they can by themselves establish ownership, emit `Mine` classification, or drive claim persistence instead of remaining subordinate inputs to the canonical detector and the explicit persistence gate. The current phase still prevents that escalation.

**Reasoning:** In live code, `add_request(...)` is documented as liveness metadata only and does not materialize the concrete `Tag16Context` required for a strict tag-only ownership claim. The strict tag-only path also states that callers must provide `add_tag_context(...)`, never falls back to direct scan, and `add_request(...)` alone does not authorize strict tag-only ownership. The phase architecture freezes the same boundary: request metadata may influence candidate selection but does not create a second ownership authority, and inbox hints remain future-only until a real live source exists. Validation records this detector-boundary closure and separately records that inbox-assisted receive stays deferred without a live hint source. Security closes the matching threat rows by stating that request metadata, tag16 hints, and inbox-style ideas must not become standalone proof of ownership. The test plan and tests-tasks package then define the future acceptance gate: if hints are ever wired, they must remain notify-only or candidate-narrowing only, must never persist by themselves, and must converge to the same canonical claimed result as the non-hinted path.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: `037-TEST-PLAN.md` and `037-TESTS-TASKS.md` are future guardrails, not proof that a live hinted path already exists.

## 🚨 Theme 4: Public Boundary, Observability, And Non-Canonical Surfaces

### 16. Exact Identity Matching Under Collision Pressure

🔴 **Quest:** What keeps the public receive contract tied to exact asset identity even when a more permissive definition-level interpretation would look superficially plausible?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The public receive contract stays tied to exact asset identity because the query predicate accepts only `asset.asset_id() == asset_id`, `lookup_receive_asset(...)` rejects `definition.id` queries as `RECEIVE_INVALID_INPUT` instead of widening to a same-definition family, and the outward response preserves the canonical `leaf.asset_id()` of the matched leaf.

**Reasoning:** `asset_rpc_balance.rs` defines `asset_matches_query_id(...)` as exact `asset.asset_id()` equality. `asset_impl_support_assets.rs` uses that exact-match predicate both against cache and persisted wallet assets, then adds an explicit reject branch: if the submitted id matches only `definition.id`, the public receive path returns `ReceiveReject::InvalidInput` rather than silently accepting a looser definition-level interpretation. After that exact lookup, `asset_impl_server_transfer.rs` scans the matched leaf and, on `Detected`, returns `canonical_id = leaf.asset_id()` in the outward response. The tests close both sides of the contract: `asset_receive_exact_asset_id_survives_definition_collision` proves that two assets may share one definition while `receive_asset(...)` still returns the requested exact id, and the definition-id rejection tests prove that the public lane rejects a definition-level query both with and without cache. `037-UAT.md` and `037-FULL-AUDIT.md` corroborate the same rule by describing the public lane as compatibility-only and exact-`asset_id` scoped.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 17. Actionable Failure Versus Ordinary Foreign Output

🔴 **Quest:** What distinguishes actionable receive failures from ordinary foreign-output classification in the current phase, and what evidence proves that this severity split is implementation truth rather than documentation aspiration?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** In the current phase, ordinary foreign-output classification is the non-alerting `NotMine` path, while actionable receive failures are `InvalidInput`, `InvalidProof`, and `RuntimeFail`. This severity split is implementation truth, not documentation aspiration.

**Reasoning:** `types_receive.rs` preserves the stable outward status vocabulary and makes only `NotMine` non-alerting through `ReceiveReject::is_alerting()`. The same file keeps detector-side failures under the existing outward compatibility labels without implying that downstream proof verification happened here. `asset_impl_server_transfer.rs` turns that rule into live behavior in `log_receive_reject(...)`: alerting rejects are logged as warnings, while `NotMine` is logged at debug severity. The adapter also keeps `InvalidProof` and `NotMine` on distinct match arms in `receive_asset_impl(...)`, preserving the split between actionable receive failure and ordinary foreign-output classification. `037-TEST-SPEC.md` and `037-EVAL-REVIEW.md` align with that same rule by documenting that observability severity stays constrained to actionable receive failures only. This claim would be falsified if `NotMine` started warning operators, or if invalid input/proof/runtime failures were downgraded into ordinary foreign-output noise.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: The phase test spec corroborates the severity split but does not itself prove that a broader generic observability-hook surface is already landed.

### 18. Outward Status Mapping Versus Service Truth

🔴 **Quest:** How can a solver verify that the outward public status surface still reflects the underlying receive truth instead of silently inventing its own compatibility semantics?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** A solver can verify this by tracing one frozen receive code family from detector/report translation into the service seam and then into the RPC adapter, and by checking parity tests on the public API for the same cases. The outward surface is still faithful because it reuses the receive truth instead of inventing a second compatibility vocabulary.

**Reasoning:** `types_receive.rs` freezes the outward receive vocabulary: `ScanResult::recv_report()` produces `ReceiveReport`, `ReceiveStatus::rpc_code()` defines the stable public status labels, and `ReceiveReject::{recv_status,rpc_code}` keeps reject outcomes inside that same receive code family without claiming downstream proof verification. `wallet_service_actions_receive.rs` exposes `scan_asset_report(...)` as the service receive-report seam, and `wallet_service_actions_reachability.rs` makes `WalletService::recv_code(...)` a direct alias of `status.rpc_code()`. In `asset_impl_server_transfer.rs`, `receive_asset_impl(...)` starts from `scan_asset_report(...)`, returns detected success with `WalletService::recv_code(status)`, and sends rejects through `recv_err(reject)`; `asset_impl.rs` shows that `recv_err(...)` serializes `reason.rpc_code()` rather than inventing RPC-local labels. `asset_receive_api_sync` then checks parity for `RECEIVE_DETECTED`, `RECEIVE_NOT_MINE`, `RECEIVE_INVALID_PROOF`, and malformed `RECEIVE_INVALID_INPUT`. The surface would drift only if RPC stopped routing through `recv_code(...)` and `recv_err(...)`, introduced new outward labels, or translated the same service result into different public meanings.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: The cited parity test covers detected, not-mine, invalid-proof, and invalid-input cases; it is not a dedicated runtime-fail parity test.

### 19. Duplicate Surface Quarantine

🔴 **Quest:** What evidence proves that orphan runtime or standalone test surfaces remain explicitly non-canonical, and what observation would show that one of those duplicates has become authoritative again?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The orphan runtime helper remains non-canonical because the live `WalletService` include stack keeps the canonical receive seam bound to `wallet_service_actions_receive.rs` and does not wire in `wallet_service_actions_runtime.rs`. The standalone RPC test file remains non-canonical because `asset_impl.rs` binds the canonical receive test module from `test_asset_impl_suite.rs`, not from the standalone `asset_impl_tests.rs` file.

**Reasoning:** In `wallet_service_actions.rs`, the live include stack explicitly keeps canonical receive wiring in `wallet_service_actions_receive.rs` and leaves `wallet_service_actions_runtime.rs` unwired; the runtime file itself also carries a KEEP/REMOVE note stating that it remains an in-tree non-canonical duplicate and must not be used as receive ownership authority. On the RPC side, `asset_impl.rs` states that canonical RPC receive tests are bound from `test_asset_impl_suite.rs` and that the standalone `asset_impl_tests.rs` file remains a non-canonical duplicate; the standalone file repeats that quarantine note locally. `test_phase037_output_reception.rs` turns both facts into executable source-shape guards by asserting that the runtime duplicate is not included in the live service stack, that the canonical receive include note remains present, that `asset_impl.rs` still binds `#[path = "test_asset_impl_suite.rs"] mod asset_impl_tests;`, and that both duplicate files still contain explicit non-canonical wording. `037-ARCHITECTURE.md` and `037-VALIDATION.md` freeze the same quarantine as phase truth and mark it covered.

**Observation That Would Reopen The Issue:** A duplicate would become authoritative again if `wallet_service_actions.rs` started including `wallet_service_actions_runtime.rs`, or if `asset_impl.rs` stopped binding `asset_impl_tests` from `test_asset_impl_suite.rs` so that the module resolved back to the standalone `asset_impl_tests.rs` file. Removal of the explicit quarantine notes would be an additional drift signal, but the authoritative change is the live wiring or module binding change.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 20. Wrapper And Stub Promotion Threshold

🔴 **Quest:** Where does the repository explicitly bound optional batching or stub-style receive seams to non-parity status, and what evidence would have to exist before promoting either seam without overclaim?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The repository explicitly bounds both seams below canonical parity, but with different contracts. `ScanEngineImpl` is a proposed-only stub seam that returns not-implemented errors and must remain non-parity until a future thin delegate to `WalletService::recv_range(...)` lands. `OptimizedScanner` is only an optional batching wrapper over the canonical `StealthOutputScanner`; its landed tests prove detector-level parity for `Mine`, `MaybeMine`, `NotMine`, and request-bound leaves, but not promotion to a live receive authority.

**Reasoning:** `scan_engine_impl.rs` and `037-ARCHITECTURE.md` explicitly mark `ScanEngineImpl` as stub-only, non-parity, and outside the implemented `recv_range(...)` lane. `037-VALIDATION.md` says the seam stays stub-only without implying parity and requires `recv_range(...)`, cursor-resume, and claimed-persistence parity tests if a thin delegate lands later. `optimized_scanner.rs` says `OptimizedScanner` owns batching only and not crypto validation, claimed persistence, or a second receive pipeline; `037-ARCHITECTURE.md` freezes the same boundary. Its unit tests prove canonical-equivalent detector classification, while `037-TEST-SPEC.md` keeps wrapper parity bounded to thinness unless later code makes it a live seam. Promotion without overclaim therefore requires more than wording drift: `ScanEngineImpl` would need a real delegate into the canonical receive lane plus parity proof over the full receive semantics, and `OptimizedScanner` would need later code-and-validation-backed promotion proving that any live routing through it preserves canonical receive semantics end-to-end rather than only detector-level parity.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

## 📌 Theme 5: Tests, Documentation, And Residual Gaps

### 21. Covered Scenario Families Versus Deferred Waves

🔴 **Quest:** Which scenario families are genuinely covered by landed code and executed tests today, and which families remain intentionally deferred even though the phase already carries a detailed test contract?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The covered families split into two layers. Across the current phase evidence package, the repository already covers the canonical `recv_range(...)` lane with explicit persistence gating, the compatibility-only exact-`asset_id` public receive slice, and duplicate-surface quarantine. Inside the currently landed Task 9 execution slice, it additionally proves deterministic request ordering with expiry pruning, fallback-last, and first-win behavior, plus the narrow current severity split where `NotMine` stays non-alerting and actionable rejects remain operator-visible. The intentionally deferred families are T2 assisted receive and hinted-path convergence, T3 optional wrapper parity, T4 residual public-RPC reinforcement beyond the already-green exact-id slice, broader T5 callback or progress-hook expansion, T6 final residual sweep, `ScanEngineImpl` delegate parity, and all five pending UAT checks.

**Reasoning:** `037-VALIDATION.md` marks the canonical `recv_range(...)` lane, compatibility-only RPC lane, and duplicate-surface quarantine as covered phase behavior. `037-TEST-EXECUTION-SUMMARY.md` then narrows the currently landed Task 9 execution slice to T1 deterministic request ordering plus the current T5 severity contract and explicitly says the remaining assisted-receive, wrapper-parity, residual RPC expansion, and final backlog-sweep waves remain open. `037-TEST-SPEC.md` and `037-TESTS-TASKS.md` preserve those open families as named backlog waves rather than letting the detailed contract be mistaken for executed proof, and `037-UAT.md` still shows five pending user-facing checks. The artifact set is therefore honest about the distinction between phase-wide covered behavior and the narrower residual test backlog that remains deferred.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 22. Cross-Checking Planning Against Execution

🔴 **Quest:** How do the planning artifacts, validation package, review package, and test-execution summary police one another against overclaim, and where would a solver expect to find tension if the stories stopped matching?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The artifact set prevents overclaim by separating intent, landed execution, review cleanliness, and closure truth into different files that must agree with one another. `037-TEST-SPEC.md` and `037-TESTS-TASKS.md` define what should eventually be covered, `037-TEST-EXECUTION-SUMMARY.md` records only what actually landed, `037-REVIEW.md` says whether that landed slice is clean, `037-VALIDATION.md` aggregates the current evidence without upgrading still-open waves, and `037-FULL-AUDIT.md` appends truth-fix reruns whenever one layer drifts.

**Reasoning:** `037-VALIDATION.md` explicitly identifies `037-ARCHITECTURE.md`, `037-REVIEW.md`, and `037-TEST-EXECUTION-SUMMARY.md` as evidence-bearing inputs and still preserves partial phase status while Task 9 residual waves and UAT remain open. `037-REVIEW.md` scopes itself to the narrow landed T1 plus current T5 slice. `037-TEST-EXECUTION-SUMMARY.md` says directly that it does not claim full backlog closure. `037-UAT.md` still records five pending user-facing obligations. `037-FULL-AUDIT.md` shows how disagreements are handled: it keeps an append-only record of truth-fix runs instead of silently rewriting the story. A solver should therefore expect tension anywhere one layer silently outruns another, such as validation claiming full closure while execution remains partial, review claiming cleanliness without matching green commands, UAT staying pending while a closeout artifact implies completion, or a later audit rerun having to correct stale wording.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 23. Strongest Missing Evidence For Full Closure

🔴 **Quest:** What is the strongest missing repository evidence that still prevents full Phase 037 closure today, and why is that missing proof not replaceable by narrower green slices or a clean code review?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The strongest missing evidence is closure-grade proof for the still-open Task 9 residual waves and the five pending UAT items, because those are the remaining explicit phase obligations that no current artifact can honestly mark complete.

**Reasoning:** `037-VALIDATION.md` remains partial and marks Task 9 as only T1 plus the narrow current T5 slice landed. `037-TEST-EXECUTION-SUMMARY.md` says the assisted-receive, wrapper-parity, residual RPC expansion, and final backlog-sweep waves remain open. `037-UAT.md` still records five pending user-facing checks with zero passes. The later reruns in `037-FULL-AUDIT.md` then say the phase is still partial only because UAT and the Task 9 residual waves remain open. Narrow green slices cannot replace that missing proof because they validate only already-landed seams, and a clean `037-REVIEW.md` cannot replace it because review can show the landed slice is bug-free without proving the unexecuted scenario families or user-facing obligations are complete.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 24. Remediation Direction For Drift Discovery

🔴 **Quest:** If answering this exam reveals a real contradiction between live receive behavior, test evidence, and phase wording, what remediation direction does the current Phase 037 artifact set require before the contradiction can be considered honestly resolved?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** If the exam uncovers a real contradiction, the current Phase 037 artifact set requires a truth-fix workflow, not a wording-only patch. The contradiction must be repaired at the authoritative layer first, then propagated through execution, validation, review, and audit artifacts with fresh command-backed evidence before the phase wording can be treated as honestly resolved.

**Reasoning:** `037-FULL-AUDIT.md` already models this remediation direction. When it found truth drift, it appended the finding, applied a direct artifact fix, reran the relevant checks, and then rechecked that `037-VALIDATION.md`, `037-REVIEW.md`, and `037-TEST-EXECUTION-SUMMARY.md` agreed again. `037-TEST-SPEC.md` and `037-TESTS-TASKS.md` also require planning artifacts to stay synchronized with what was actually implemented, and `037-VALIDATION.md` warns that no reader should infer full closeout while pending UAT or residual waves remain open. The honest repair path is therefore: identify whether code or wording is wrong against the frozen phase authority, fix the code or narrow the docs, rerun the strongest focused tests plus any required release gate, update the phase truth artifacts, and leave the phase partial until the open obligations are actually re-proved.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.

### 25. Adjacent Workspace Truth Versus Phase-Local Proof

🔴 **Quest:** Which adjacent workspace facts may legitimately inform a Phase 037 answer but may not be used as standalone closure proof for the phase itself, and what does that restriction reveal about the repository's honesty model?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Adjacent workspace facts may inform a Phase 037 answer only as context or drift detectors, not as standalone phase closure proof. Legitimate examples are the repository-wide release-gate disposition, out-of-scope workspace failures or fixes that can falsify stale wording, and append-only full-audit reruns that explain how truth changed over time. Standalone closure proof still has to come from the phase-local closure-truth package for the frozen `z00z_wallets` scope.

**Reasoning:** `037-FULL-AUDIT.md` repeatedly distinguishes frozen phase scope from broader workspace facts. It uses repository-wide gate status to keep the phase wording honest, but it also says that green workspace status does not upgrade pending UAT and Task 9 residual waves into closure. Earlier append-only runs preserved the historical simulator-red context, while later reruns recorded that the green release gate was current truth; in both cases the audit still insisted that Phase 037 remained partial because phase-local obligations were open. The same file excludes out-of-scope crates and backup artifacts from phase authority, and the current phase-local closure-truth artifacts still show `UAT = 5 pending` and Task 9 residual waves open. This reveals the repository's honesty model: broader workspace facts may falsify stale claims or explain why a truth artifact must change, but they cannot override scoped phase-local proof obligations or inflate a partial phase into a closed one.

**Gap Or Blocker:** None.

**Verification:** `doublecheck` status: VERIFIED. Residual caveat: None.
