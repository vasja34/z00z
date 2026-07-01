---
phase: 054
slug: refactor-crates
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-08
updated: 2026-06-09
---

# Phase 054 - Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## 🔑 Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Public storage facade -> backend seam | Callers use semantic settlement APIs while raw row and journal traits stay below `SettlementTreeBackend`. | `SettlementPath`, `SettlementStateRoot`, proof blobs, raw backend rows |
| Storage semantics -> runtime planner | Runtime admission must rebind tx and claim payloads onto one payload-verified digest before routing, and still must not become proof or settlement-root authority. | Verified route keys, `BatchRoute`, `PlanDigest`, ordered work items |
| Runtime placement or execution -> validator or watcher | Validators and watchers read operational placement or execution projections only. | `ShardPlacementView`, `ShardExecTicket`, verdict metadata |
| Runtime crates -> rollup node root | `z00z_rollup_node` remains the orchestration root over aggregator, validator, watcher, and DA adapters. | Service bindings, publication state, placement snapshot, execution ticket |
| Docs and guardrails -> downstream users | README and source-shape checks must describe only the landed topology and the crate-root facades. | Module paths, crate-root exports, exception records |

---

## 🚨 Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-054-01 | Spoofing | backend seam | mitigate | Keep `SettlementTreeBackend` and settlement proofs authoritative over low-level backend traits. | closed |
| T-054-02 | Tampering | proof or public surface | mitigate | Guard source shape so backend-root or layout authority cannot widen during structural refactor. | closed |
| T-054-03 | Repudiation | backend error boundary | mitigate | Keep `StoreBackendError` and seam contract names stable across the move boundary. | closed |
| T-054-04 | Spoofing | concrete adapter ownership | mitigate | Keep `SettlementStore` and `SettlementTreeBackend` as the semantic settlement owners. | closed |
| T-054-05 | Tampering | helper extraction | mitigate | Preserve proof, replay, and query semantics through parity gates and live acceptance suites. | closed |
| T-054-06 | Elevation of privilege | fake backend scope | mitigate | Keep the backend wave limited to the live `common`, `memory`, and `redb` surfaces only. | closed |
| T-054-07 | Spoofing | runtime planner metadata | mitigate | Keep semantic and replay helpers in storage and limit runtime ownership to batch admission and ordering. | closed |
| T-054-08 | Tampering | `tx_plan` split | mitigate | Use an explicit semantic split so dry-run, rollback, and touched-path behavior stay intact. | closed |
| T-054-09 | Elevation of privilege | aggregator authority | mitigate | Keep runtime planner outputs metadata-only and block proof or settlement-root ownership drift. | closed |
| T-054-10 | Spoofing | validator or watcher boundary | mitigate | Bind validators and watchers to downstream operational projections only. | closed |
| T-054-11 | Tampering | node orchestration root | mitigate | Keep `z00z_rollup_node` as the top-level orchestration owner. | closed |
| T-054-12 | Elevation of privilege | placement authority | mitigate | Tie placement views to route generation and reject unowned shard routes. | closed |
| T-054-13 | Tampering | canonical cleanup | mitigate | Restrict mechanical cleanup to post-seam canonicalization without changing root, proof, or replay semantics. | closed |
| T-054-14 | Repudiation | serialization helper ownership | mitigate | Remove the duplicate temp-tree helper and keep one canonical serialization helper path. | closed |
| T-054-15 | Elevation of privilege | cleanup scope | mitigate | Restrict cleanup scope to active storage hot spots instead of unrelated crate families. | closed |
| T-054-16 | Tampering | rename wave | mitigate | Keep the rename wave behavior-neutral and back it with targeted release and guard suites. | closed |
| T-054-17 | Repudiation | public export continuity | mitigate | Preserve stable crate-root re-exports and remove duplicate public paths after renames. | closed |
| T-054-18 | Elevation of privilege | layout placeholders | mitigate | Keep placeholder cleanup cosmetic and enforce the canonical test or support layout explicitly. | closed |
| T-054-19 | Repudiation | closeout docs | mitigate | Require landed-topology wording and explicit exception recording in final docs. | closed |
| T-054-20 | Tampering | docs authority boundaries | mitigate | Keep the semantic-truth vs operational-metadata distinction explicit in closeout docs and status projections. | closed |
| T-054-21 | Elevation of privilege | final cleanup wave | mitigate | Scope closeout to docs and evidence sync and reopen the packet on any real regression. | closed |
| T-054-22 | Spoofing | runtime planner digest ingress | mitigate | Recompute tx and claim digests at ingress, route only from verified runtime metadata, and keep the public planner lane on one canonical `WorkItem` path. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## 🧾 Threat Evidence

| Threat ID | Evidence |
|-----------|----------|
| T-054-01 | `crates/z00z_storage/src/backend/mod.rs:1-5,32-47`; `crates/z00z_storage/src/settlement/store/mod.rs:95-147`; `crates/z00z_storage/tests/test_live_guardrails.rs:155-194` |
| T-054-02 | `crates/z00z_storage/tests/test_live_guardrails.rs:155-194,289-345,347-392`; `crates/z00z_storage/tests/test_downstream_guardrails.rs:130-191` |
| T-054-03 | `crates/z00z_storage/tests/test_live_guardrails.rs:179-193`; `.planning/phases/054-Refactor-Crates/054-01-SUMMARY.md:24-45` |
| T-054-04 | `.planning/phases/054-Refactor-Crates/054-02-SUMMARY.md:16-21,40-53`; `crates/z00z_storage/src/settlement/store/mod.rs:95-147` |
| T-054-05 | `crates/z00z_storage/tests/test_readme_examples.rs:35-108,111-162`; `crates/z00z_storage/tests/test_bench_lanes.rs:185-238`; `.planning/phases/054-Refactor-Crates/054-02-SUMMARY.md:54-79` |
| T-054-06 | `crates/z00z_storage/src/backend/mod.rs:7-10`; `.planning/phases/054-Refactor-Crates/054-02-SUMMARY.md:24-39` |
| T-054-07 | `.planning/phases/054-Refactor-Crates/054-03-SUMMARY.md:16-21,45-53`; `crates/z00z_runtime/aggregators/src/batch_planner.rs:99-197`; `crates/z00z_storage/src/settlement/tx_plan_help.rs:73-93` |
| T-054-08 | `crates/z00z_storage/src/settlement/tx_plan_help.rs:39-93`; `crates/z00z_storage/src/settlement/tx_plan_types.rs:15-97`; `crates/z00z_storage/src/settlement/store/hjmt_plan.rs:54-122` |
| T-054-09 | `.planning/phases/054-Refactor-Crates/054-03-SUMMARY.md:80-90`; `crates/z00z_runtime/aggregators/src/batch_planner.rs:176-197` |
| T-054-10 | `crates/z00z_runtime/validators/src/engine.rs:14-23`; `crates/z00z_runtime/watchers/src/engine.rs:34-65`; `.planning/phases/054-Refactor-Crates/054-04-SUMMARY.md:16-21,72-80` |
| T-054-11 | `crates/z00z_rollup_node/src/runtime.rs:17-92`; `.planning/phases/054-Refactor-Crates/054-04-SUMMARY.md:18-21,40-45` |
| T-054-12 | `crates/z00z_runtime/aggregators/src/placement.rs:85-111,145-163`; `crates/z00z_runtime/aggregators/src/shard_exec.rs:51-66,123-135` |
| T-054-13 | `.planning/phases/054-Refactor-Crates/054-05-SUMMARY.md:16-21,25-52,77-88`; `crates/z00z_storage/src/settlement/store/mod.rs:6-16,40-60` |
| T-054-14 | `crates/z00z_storage/src/serialization/mod.rs:7-13,26-28`; `crates/z00z_storage/src/serialization/build.rs:10-27` |
| T-054-15 | `.planning/phases/054-Refactor-Crates/054-05-SUMMARY.md:36-52,63-73` |
| T-054-16 | `.planning/phases/054-Refactor-Crates/054-06-SUMMARY.md:16-20,63-71,87-95` |
| T-054-17 | `crates/z00z_runtime/aggregators/src/lib.rs:3-27`; `crates/z00z_runtime/validators/src/lib.rs:3-21`; `crates/z00z_runtime/watchers/src/lib.rs:3-19`; `crates/z00z_rollup_node/src/lib.rs:8-38`; `crates/z00z_storage/src/lib.rs:4-13`; `.planning/phases/054-Refactor-Crates/054-06-SUMMARY.md:51-59,96-113` |
| T-054-18 | `crates/z00z_wallets/tests/test_rename_guards.rs:52-58,88-128,130-181,185-215`; `.planning/phases/054-Refactor-Crates/054-06-SUMMARY.md:45-47,104-113` |
| T-054-19 | `.planning/phases/054-Refactor-Crates/054-07-SUMMARY.md:16-20,24-40,83-113`; `.planning/phases/054-Refactor-Crates/054-SUMMARY.md:14-20,52-58` |
| T-054-20 | `crates/z00z_runtime/watchers/src/engine.rs:34-65`; `crates/z00z_rollup_node/src/runtime.rs:51-92`; `.planning/phases/054-Refactor-Crates/054-07-SUMMARY.md:41-50` |
| T-054-21 | `.planning/phases/054-Refactor-Crates/054-07-SUMMARY.md:51-81,89-113`; `.planning/phases/054-Refactor-Crates/054-SUMMARY.md:52-76` |
| T-054-22 | `crates/z00z_runtime/aggregators/src/ingress.rs:12-66`; `crates/z00z_runtime/aggregators/src/types.rs:27-106,244-260`; `crates/z00z_runtime/aggregators/src/batch_planner.rs:99-191`; `crates/z00z_runtime/aggregators/src/service.rs:8-14`; `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs:1-43`; `.planning/phases/054-Refactor-Crates/054-08-SUMMARY.md:1-77` |

Unregistered flags: none.

Current audit note: no `store_codec`, `store_roots`, or `store_mem` module-path
aliases remain in the live `crates/z00z_storage/src/settlement/store/*`
implementation, and no alternate digest-authority or planner-ready bypass path
remains in the live `crates/z00z_runtime/aggregators/*` public lane.

---

## ⚠️ Accepted Risks Log

No accepted risks.

---

## ✅ Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-08 | 21 | 21 | 0 | Codex |
| 2026-06-09 | 22 | 22 | 0 | Codex |

---

## 👍 Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-09
