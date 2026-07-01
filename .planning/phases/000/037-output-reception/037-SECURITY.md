---
phase: 037
slug: output-reception
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-23
---

# Phase 037 - Security

> Per-phase security contract for the output-reception plan chain. This file
> verifies only the threats declared in the Phase 037 plan artifacts and closes
> them against summary-backed implementation and validation evidence.

## 🔑 Trust Boundaries

| Boundary | Description | Data Crossing |
| -------- | ----------- | ------------- |
| canonical range receive -> compatibility receive lanes | `WalletService::recv_range(...)` remains the only canonical range authority while single-asset service and RPC receive stay compatibility-only | ownership classification, receive reports, exact-asset receive status |
| detector prefilters -> ownership classification | Request metadata, tag16 hints, and inbox-style ideas must not become standalone proof of ownership | `Tag16Context`, active-request metadata, candidate ordering |
| detection -> persistence | Detection results may report ownership, but only the explicit persistence gate may mutate claimed state | `ReceiveNext`, claimed assets, scrubbed owner fields |
| scanner docs -> live implementation | Proposed-only scanner seams and richer config vocabulary must not be read as implemented parity | scanner knobs, config names, stub seams |
| request cache -> request-bound scan selection | Active request ordering must stay deterministic, expiry-aware, and fallback-last | request candidates, expiry state, fallback path |
| receive results -> operator visibility | Alerting must remain bounded to actionable failures, with non-owned outputs staying non-alerting | reject classes, severity, progress DTOs |
| public RPC receive -> service boundary | The public exact-asset receive adapter must stay aligned to the service boundary and must not redefine canonical receive semantics | `receive_asset_impl(...)`, `scan_asset_report(...)`, `receiver_keys(...)` |
| duplicate helper and test files -> future reviews | Orphan duplicate files must remain explicitly non-canonical so they cannot be mistaken for authority | runtime helper wording, standalone test-surface wording |

## 🚨 Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
| --------- | -------- | --------- | ----------- | ---------- | ------ |
| T-037-01 | T | `037-ARCHITECTURE.md` | mitigate | Closed by the Plan 01 baseline freeze: `037-01-SUMMARY.md` records one live receive ledger anchored to `WalletService::recv_range(...)` and live file paths only. | closed |
| T-037-02 | I | `wallet_service_actions_receive.rs` docs | mitigate | Closed by Plan 01 and Plan 10: compatibility-only wording now bounds `scan_asset_report(...)`, `receive_asset(...)`, and outward `wallet.asset.receive_asset` without redefining the canonical lane. | closed |
| T-037-03 | E | `scan_engine_impl.rs` docs | mitigate | Closed by Plans 01, 03, and 04: `ScanEngineImpl` is explicitly documented as stub-only and proposed-only, with the stub contract validated in `037-03-SUMMARY.md` and `037-04-SUMMARY.md`. | closed |
| T-037-04 | S | Tag16 strict path | mitigate | Closed by Plan 02: strict tag-only ownership now requires concrete `Tag16Context` materialization, and `add_request(...)` remains liveness metadata only. | closed |
| T-037-05 | T | detector boundary | mitigate | Closed by Plan 02: receive detection and classification stay separate from downstream import, tx validation, and proof verification; detector-side `InvalidProof` remains frozen compatibility vocabulary only. | closed |
| T-037-06 | D | prefilter tests | mitigate | Closed by Plan 02 focused regressions in `test_stealth_scanner_prefilter`, `test_scenario1_semantics`, and the bridge reject-path check recorded in `037-02-SUMMARY.md`. | closed |
| T-037-07 | T | `recv_route` gating | mitigate | Closed by Plan 03: `recv_route(...)` now preserves an explicit `ReportOnly` versus `PersistClaim` split, backed by the focused `recv_route_gate` regression. | closed |
| T-037-08 | I | `037-ARCHITECTURE.md` | mitigate | Closed by Plan 03: the architecture ledger now maps the live module and receive flow only, removing stale trait-stack claims and documenting superseded scanner seams as proposed-only. | closed |
| T-037-09 | E | claimed persistence boundary | mitigate | Closed by Plan 03: `recv_claim_asset(...)` remains the canonical detector-to-claimed adaptation seam and preserves the compatibility scrub contract before persistence. | closed |
| T-037-10 | S | inbox-assisted receive | mitigate | Closed by Plan 04: inbox-assisted receive remains future-only until a concrete live hint source exists and deterministic ordered non-expired candidate selection is already closed. | closed |
| T-037-11 | T | scan-engine docs | mitigate | Closed by Plan 04: the repository selected the explicit de-scope branch, and `ScanEngineImpl` now returns one shared deferred-not-implemented contract instead of implying parity. | closed |
| T-037-12 | E | receive orchestration | mitigate | Closed by Plan 04: no speculative inbox module or second receive, detect, or store lane was added, and the architecture file remains aligned to the single live receive flow. | closed |
| T-037-13 | T | `OptimizedScanner` parity | mitigate | Closed by Plan 05: `OptimizedScanner` remains an optional batching wrapper subordinate to the canonical detector, with parity checks for Mine, MaybeMine, NotMine, request-bound scans, and fallback behavior. | closed |
| T-037-14 | D | test suite scope | mitigate | Closed by Plans 05 and the validation artifacts: Task 9 remains anchored to the approved phase test package, and `037-TEST-EXECUTION-SUMMARY.md` truthfully records only the landed T1 plus narrow T5 slice instead of overclaiming broader coverage. | closed |
| T-037-15 | I | test evidence | mitigate | Closed by Plan 05 and `037-VALIDATION.md`: the phase prefers existing suites and adds one dedicated phase-local source-shape guard only for the new duplicate-quarantine scenario. | closed |
| T-037-16 | I | receive crypto checks | mitigate | Closed by Plan 06 and earlier detector-boundary work: constant-time tag comparison and downstream proof verification remain the receive guardrails, with no relaxed ownership shortcut introduced. | closed |
| T-037-17 | T | persistence model | mitigate | Closed by Plan 06: wallet-native claimed persistence remains the current canonical receive target, and no third persistence layer is defined. | closed |
| T-037-18 | D | storage docs | mitigate | Closed by Plan 06: storage docs now classify implemented, future-unification, and stale buckets explicitly, quarantining old `SpendableAsset` and SQLite vocabulary. | closed |
| T-037-19 | T | reception API docs | mitigate | Closed by Plan 07: the reception API narrative is rebased to live service and RPC seams only, while proposed-only receive API names remain quarantined. | closed |
| T-037-20 | I | scanner-config docs | mitigate | Closed by Plan 07: only implemented scanner knobs are described as live behavior, while `ScanConfig` and `DoSMitigationConfig` stay future-only vocabulary. | closed |
| T-037-21 | D | receive orchestration | mitigate | Closed by Plan 07: tag16, inbox, and parallel ideas remain documented as strategy inputs over the same canonical detector, not as a second orchestration stack. | closed |
| T-037-22 | T | phase terminology | mitigate | Closed by Plan 08: stale historical names were translated into live implementation terms only, and the architecture ledger now describes the deterministic request-candidate policy directly. | closed |
| T-037-23 | D | request candidate iteration | mitigate | Closed by Plan 08 and the current Task 9 slice: active requests are stored in deterministic order, expired requests are skipped, and the requestless fallback candidate remains explicit and last. | closed |
| T-037-24 | R | receive docs | mitigate | Closed by Plan 08: documentation is rebased to the exact implemented ordering path, and the ordering policy is reinforced by targeted regressions plus the clean review chain. | closed |
| T-037-25 | D | observability surface | mitigate | Closed by Plan 09 and the Task 9 T5 slice: `ReceiveReject::NotMine` is now non-alerting while actionable failures remain warning-level, preventing both over-alerting and under-alerting. | closed |
| T-037-26 | T | progress contract | mitigate | Closed by Plan 09: progress semantics are anchored to `ScanStatePayload`, `ScanRangeOut`, and `ScanRangeStat` only, with no second progress surface implied. | closed |
| T-037-27 | R | operator guidance | mitigate | Closed by Plan 09: severity mapping is explicit through `ReceiveReject::is_alerting()` and aligned adapter logging, while review passes found no remaining guidance drift. | closed |
| T-037-28 | T | RPC receive boundary | mitigate | Closed by Plan 10: the public adapter remains aligned to `receive_asset_impl(...)`, `scan_asset_report(...)`, and exact-`asset_id` behavior, validated by `test_asset_impl_suite`. | closed |
| T-037-29 | R | duplicate file evidence | mitigate | Closed by Plan 10 and `test_phase037_output_reception.rs`: orphan runtime and standalone RPC test files now carry explicit non-canonical notes and source-shape guards. | closed |
| T-037-30 | I | final phase docs | mitigate | Closed by Plan 10 plus `037-VALIDATION.md`: final Phase 037 documentation points to the wired live seams only and truthfully bounds partial residual test work without reopening canonical receive authority. | closed |

## ✅ Accepted Risks Log

No accepted risks.

## 📌 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
| ---------- | ------------- | ------ | ---- | ------ |
| 2026-04-23 | 30 | 30 | 0 | the agent (gsd-security-auditor) |

## 👍 Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-23
