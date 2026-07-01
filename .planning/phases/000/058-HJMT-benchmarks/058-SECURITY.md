---
phase: 058
slug: hjmt-benchmarks
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-15
register_authored_at_plan_time: true
---

# Phase 058 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## 🔑 Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Planning packet -> implementation | Phase plans, source audit, test spec, tests tasks, and evidence ledger must map to exact live homes instead of TODO-only names. | owner homes, commands, digests, verdict rows |
| Runtime config -> node startup | Checked YAML and runtime manifest drive live topology, planner, journal, and startup behavior. | ports, shard topology, route digests, journal lineage, backend generation |
| Import/export fixtures -> startup contract | Route, recovery, publication, and proof artifacts must roundtrip without creating a second trust path. | route tables, recovery state, checkpoint publications, proof blobs |
| Simulator release lane -> public packet | `scenario_1` emits the canonical release packet and must not depend on private-only or debug-only surfaces. | `run_meta.json`, `wallet_scan.json`, `hist_flow.json`, `occ_flow.json`, `sim_summary.md` |
| Publication lineage -> downstream consumers | Validators and watchers must consume one publication digest story and reject detached evidence. | publication digest, public root, settlement root, route generation |
| Benchmark harness -> repository verdict | Bench reports may influence readiness claims only through explicit archived evidence and supported classifications. | measurement reports, timing slices, proof bytes, score or unsupported rows |
| Historical replay -> wallet and proof boundary | Scope birth, wallet promotion, historical replay, and occupancy evidence must stay bound to imported lineage artifacts. | route generations, proof verdicts, disclosure guards, wallet states |
| Final docs -> project status consumers | `ROADMAP.md`, `STATE.md`, summaries, and validation docs must carry one consistent closeout story. | phase status, final verdict, open caps, archive-home claims |

---

## 🚨 Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Evidence | Status |
|-----------|----------|-----------|-------------|------------|----------|--------|
| T-058-01 | Tampering | Evidence ledger | mitigate | Freeze readiness claims to exact commands, owner homes, digests, and verdict rows instead of green-test shorthand. | `058-EVIDENCE-LEDGER.md`; `058-VALIDATION.md`; `058-SUMMARY.md` | closed |
| T-058-02 | Spoofing | Source audit and owner-path routing | mitigate | Keep exact-live, successor-live, and proposed paths explicit so TODO contract names cannot masquerade as repository facts. | `058-SOURCE-AUDIT.md`; `058-TEST-SPEC.md`; `058-CONTEXT.md` | closed |
| T-058-03 | Repudiation | Planning packet completeness | mitigate | Route simulator, failover, benchmark, and fixture evidence through numbered plans and final validation instead of prose-only claims. | `058-01-PLAN.md`; `058-02-PLAN.md`; `058-05-PLAN.md`; `058-VALIDATION.md` | closed |
| T-058-04 | Repudiation | Simulator public release lane | mitigate | Make the release packet canonical and validate it from the public `--release` lane rather than private-only or debug-only paths. | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_simulator/tests/test_scenario_settlement.rs`; `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | closed |
| T-058-05 | Tampering | Design/stage sync | mitigate | Fail closed when trace payloads drift from the canonical release packet or design expectations. | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`; `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs` | closed |
| T-058-06 | Tampering | Trace packet lineage | mitigate | Keep one packet inventory and one digest story, and reject detached public trace additions. | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_simulator/tests/test_hjmt_e2e.rs`; `crates/z00z_simulator/tests/test_scenario_settlement.rs` | closed |
| T-058-07 | Information Disclosure | Public simulator artifacts | mitigate | Mark emitted vs pending packet files explicitly and keep public packet redaction at `public_only`. | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_simulator/tests/test_scenario_settlement.rs`; `crates/z00z_simulator/tests/test_hjmt_e2e.rs` | closed |
| T-058-08 | Tampering | Runtime YAML and topology loading | mitigate | Prove config changes alter live behavior and reject fixture-bound fake configurability. | `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`; `crates/z00z_rollup_node/tests/test_hjmt_process.rs`; `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`; `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs` | closed |
| T-058-09 | Tampering | Import/export artifacts | mitigate | Roundtrip route and recovery artifacts through strict codecs and reject tampered imports. | `crates/z00z_storage/tests/test_hjmt_import_export.rs` | closed |
| T-058-10 | Tampering | Startup route and lineage contract | mitigate | Reject wrong route digest, wrong journal lineage, wrong root generation, and malformed proof inputs before live work begins. | `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`; `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs` | closed |
| T-058-11 | Tampering | Startup proof and backend contract | mitigate | Reject stale-route or wrong proof-family or unsupported backend-generation startup states. | `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`; `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`; `crates/z00z_storage/tests/test_hjmt_storage_boundary.rs` | closed |
| T-058-12 | Tampering | Final runtime/publication packet | mitigate | Keep runtime and publication evidence on one lineage and fail closed on same-route successor drift. | `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`; `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`; `058-SUMMARY.md` | closed |
| T-058-13 | Repudiation | Validator and watcher consumers | mitigate | Keep validator and watcher evidence on the same publication contract and reject detached publication truth paths. | `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`; `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` | closed |
| T-058-14 | Denial of Service | Topology coverage | mitigate | Keep a positive non-`5x7` topology live so the canonical fixture does not become a hidden system ceiling. | `crates/z00z_rollup_node/tests/test_hjmt_topology.rs` | closed |
| T-058-15 | Tampering | Multi-aggregator coverage | mitigate | Keep planner-mode and process-topology checks explicit so shared-memory shortcuts cannot fake release topology coverage. | `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs`; `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_simulator/tests/test_scenario_settlement.rs` | closed |
| T-058-16 | Repudiation | Benchmark corpus completeness | mitigate | Bind readiness claims to archived benchmark reports, timing slices, and explicit measurement lanes rather than broad hot-path numbers. | `crates/z00z_storage/benches/settlement_benches.md`; `crates/z00z_storage/tests/test_bench_lanes.rs` | closed |
| T-058-17 | Repudiation | Heavy-profile routing | mitigate | Keep `SIM-BATCH-1000` heavy-only and preserve correctness profiles separately. | `crates/z00z_storage/benches/settlement_benches.md`; `crates/z00z_storage/tests/test_bench_lanes.rs`; `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs` | closed |
| T-058-18 | Repudiation | Compression and score claims | mitigate | Mark unsupported compression wins and require versioned archived evidence before any score claim can stand. | `crates/z00z_storage/benches/settlement_benches.md`; `crates/z00z_storage/tests/test_bench_lanes.rs` | closed |
| T-058-19 | Repudiation | Shard-scaling and lookup fields | mitigate | Require explicit shard-scaling fields and reject score shortcuts such as worker-local throughput standing in for durable-root-published TPS. | `crates/z00z_storage/benches/settlement_benches.md`; `crates/z00z_storage/tests/test_bench_lanes.rs` | closed |
| T-058-20 | Tampering | Scope birth under replay and recovery | mitigate | Keep first-seen scope birth, restart, and failover evidence tied to the committed settlement seam. | `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`; `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` | closed |
| T-058-21 | Elevation of Privilege | Wallet promotion boundary | mitigate | Require proof-before-ownership and keep wallet promotion downstream of proof validation. | `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`; `crates/z00z_simulator/tests/test_hjmt_e2e.rs`; `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` | closed |
| T-058-22 | Tampering | Historical and occupancy replay | mitigate | Bind historical and occupancy verdicts to imported route and publication artifacts and reject reinterpretation drift. | `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`; `crates/z00z_storage/tests/test_hjmt_privacy_regression.rs`; `crates/z00z_simulator/tests/test_hjmt_e2e.rs` | closed |
| T-058-23 | Tampering | Cross-flow lineage agreement | mitigate | Require `scope_flow`, `plan_flow`, `journal_flow`, `leaf_flow`, `pub_flow`, and `wallet_scan` to tell one consistent first-live-object story. | `crates/z00z_simulator/tests/test_hjmt_e2e.rs`; `crates/z00z_simulator/tests/test_scenario_settlement.rs`; `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` | closed |
| T-058-24 | Repudiation | Fixture-family closeout | mitigate | Keep fixture closure tied to exact manifests, regeneration evidence, and fail-closed import/export readers. | `058-07-SUMMARY.md`; `058-SUMMARY.md`; `crates/z00z_storage/tests/test_hjmt_import_export.rs` | closed |
| T-058-25 | Repudiation | Final repository verdict | mitigate | Require the final repository verdict to match the closed evidence matrix exactly and keep stronger labels gated by explicit thresholds. | `058-SUMMARY.md`; `058-07-SUMMARY.md`; `docs/tech-papers/Z00Z-HJMT-Acceptance-Thresholds.md`; `crates/z00z_storage/benches/settlement_benches.md` | closed |
| T-058-26 | Repudiation | Planning-state synchronization | mitigate | Synchronize the phase packet, `ROADMAP.md`, `STATE.md`, and final validation docs on one closeout story. | `058-SUMMARY.md`; `058-VALIDATION.md`; `.planning/ROADMAP.md`; `.planning/STATE.md` | closed |
| T-058-27 | Information Disclosure | Final public artifact contract | mitigate | Keep disconnected or redaction-violating artifacts out of the final public packet and mark missing exact homes as pending instead of emitted. | `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_simulator/tests/test_scenario_settlement.rs`; `crates/z00z_simulator/tests/test_hjmt_e2e.rs` | closed |

*Status: open · closed*  
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## ✅ Accepted Risks Log

No accepted risks.

---

## 🧾 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-15 | 27 | 27 | 0 | Codex `/gsd-secure-phase 058` |

### Audit Notes

- `058-01-PLAN.md` through `058-07-PLAN.md` all contained parseable
  `<threat_model>` blocks, so the register was treated as authored at plan
  time rather than generated retroactively.
- No `## Threat Flags` sections were present in `058-*-SUMMARY.md`; the audit
  therefore used the plan threat models plus live implementation, validation,
  and phase-closeout artifacts as the security evidence base.
- The Phase 058 closeout verdict is now `integrated upgrade`; that verdict is
  evidence-gated by the standalone threshold packet and is not treated as a
  free-form marketing label.

---

## ✅ Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-15
