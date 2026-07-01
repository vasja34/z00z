---
phase: 056
slug: hjmt-storage-aggregator
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-12
---

# Phase 056 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Runtime topology home -> rollup node | Disk-backed HJMT runtime home is loaded into live topology and preflight state. | YAML config, route digest, process paths, lineage expectations |
| Runtime planner -> storage settlement | Runtime emits semantic handoff only; storage remains subtree and proof owner. | `SettlementExecHandoff`, `route_table_digest`, shard and generation metadata |
| Runtime recovery -> durable storage state | Runtime recovery decisions are bound to live durable lineage and root metadata. | `SettlementRecoveryState`, journal lineage, root generation |
| Simulator evidence -> runtime truth | Simulator trace pack must prove the same runtime/config/lineage view as the live owner seams. | Config digests, route digest, process topology digest, trace files |
| Storage bench homes -> closeout claims | Bench claims must stay anchored to the existing storage bench homes. | Bench lane names, fixture manifests, runtime manifest references |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-056-01 | Spoofing | Runtime topology | mitigate | `test_hjmt_topology.rs` and `test_hjmt_process.rs` keep the canonical path OS-process based and reject malformed topology. | closed |
| T-056-02 | Tampering | Config home ownership | mitigate | `NodeConfig::from_hjmt_home(...)` plus `test_hjmt_preflight.rs` reject route-digest drift, missing startup blocks, bad backend generation, and config-reference drift. | closed |
| T-056-03 | Repudiation | `SIM-5A7S` acceptance fixture | mitigate | `manifest.json`, `test_hjmt_topology.rs`, and `test_hjmt_node_lifecycle.rs` freeze the topology contract and process inventory. | closed |
| T-056-04 | Tampering | Route-table authority | mitigate | `test_hjmt_shard_routing.rs` proves canonical bytes and fail-closed tamper vectors; `test_live_guardrails.rs` forbids caller-supplied digest authority. | closed |
| T-056-05 | Elevation of privilege | Planner ownership | mitigate | `test_live_guardrails.rs` keeps planner authority ingress-only; storage and simulator do not become a second route planner. | closed |
| T-056-06 | Integrity | Planner equivalence | mitigate | `test_hjmt_planner.rs` keeps central and per-aggregator planners aligned on accepted digest and reject semantics. | closed |
| T-056-07 | Elevation of privilege | Runtime-to-storage seam | mitigate | `test_hjmt_scope_birth.rs` and `test_live_guardrails.rs` keep the handoff semantic-only and prevent runtime ownership of subtree or proof truth. | closed |
| T-056-08 | Integrity | First-scope birth | mitigate | `test_hjmt_scope_birth.rs` and `settlement::store::test_live_recovery::*` keep first-seen scope birth restart-safe and durable-root bound. | closed |
| T-056-09 | Information disclosure | `scope_flow.json` | mitigate | `test_hjmt_scope_birth.rs` and `test_scenario1_stage_surface.rs` require `private_tree_id_exposed == false` and reject storage-private identifier leakage. | closed |
| T-056-10 | Availability | Failover routing | mitigate | `RecoveryBoundary::resume(...)` plus `test_hjmt_failover_same_lineage.rs` and `test_hjmt_split_brain_fencing.rs` reject silent reroute and keep takeover same-lineage only. | closed |
| T-056-11 | Replay | Recovery lineage | mitigate | `recovery.rs`, `test_hjmt_split_brain_fencing.rs`, and `settlement::store::test_live_recovery::*` reject wrong lineage, wrong generation, stale root, and stale restart. | closed |
| T-056-12 | Tampering | Journal baseline | mitigate | `test_live_guardrails.rs` keeps `JournalBackend` local and singular and documents that a shared cross-aggregator WAL is not protocol truth. | closed |
| T-056-13 | Misconfiguration | Runtime shape vs evidence | mitigate | `test_hjmt_topology.rs` and `test_hjmt_preflight.rs` reject inconsistent aggregator/shard ranges, duplicate owners, and config/home drift. | closed |
| T-056-14 | Integrity | Startup preflight | mitigate | `test_hjmt_preflight.rs` validates config digests, route digest, proof bytes, lineage, and handoff ordering before live work starts. | closed |
| T-056-15 | Tampering | YAML contract honesty | mitigate | `test_hjmt_preflight.rs` proves planner-mode, listen-address, standby-set, and journal-path changes alter config digests instead of bypassing the YAML contract. | closed |
| T-056-16 | Integrity | Runtime observability trace linkage | mitigate | `runner::validate_runtime_observability_artifacts(...)`, `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`, and the stabilized shared Stage 13 cache keep the full trace pack linked to one config-digest set, one route-table digest, one journal-lineage view, and one process-topology view on the live tree. | closed |
| T-056-17 | Tampering | Design/runtime concept drift | mitigate | `test_scenario1_stage_surface.rs` rejects invalid design YAML and narrowed or drifted stage contracts. | closed |
| T-056-18 | Spoofing | Accepted runtime path in simulator | mitigate | `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`, `056-06-SUMMARY.md`, and the green current-tree reruns prove the accepted simulator path is the live runtime plane rather than an in-process shortcut. | closed |
| T-056-19 | Repudiation | Phase closeout honesty | mitigate | `056-06-SUMMARY.md`, `056-07-SUMMARY.md`, `test_bench_lanes.rs`, and the current green workspace reruns keep phase closeout claims anchored to live evidence instead of stale partial results. | closed |
| T-056-20 | Tampering | Final evidence freshness | mitigate | `stage13_shared_cases.rs` rewrites promoted trace anchors onto the final cache root, `fixture_cache.rs` refreshes the content fingerprint after stabilization, and the current green simulator plus workspace reruns re-verify fresh evidence on the live tree. | closed |
| T-056-21 | Integrity | Bench-home ownership | mitigate | `test_bench_lanes.rs` keeps shard and cache-edge lanes in the existing storage bench homes and rejects a second bench authority surface. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-12 | 21 | 17 | 4 | Codex `gsd-secure-phase` |
| 2026-06-12 | 21 | 21 | 0 | Codex `gsd-secure-phase` |

---

## Verification Evidence

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on the live tree before the focused security pass.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_planner --test test_hjmt_shard_routing --test test_hjmt_failover_same_lineage --test test_hjmt_split_brain_fencing --test test_live_guardrails -- --nocapture` passed.
- `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture` passed.
- `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_topology --test test_hjmt_process --test test_hjmt_node_lifecycle -- --nocapture` passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_scope_birth --test test_live_guardrails --test test_bench_lanes -- --nocapture` passed.
- `cargo test -p z00z_storage --release --features test-params-fast test_live_recovery -- --nocapture` passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_pipeline_genesis_tx -- --nocapture` passed after the shared Stage 13 cache stabilization fix.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement -- --nocapture` passed on the live tree.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture` passed on the live tree.
- `cargo bench -p z00z_storage --bench settlement_shard --no-run` passed.
- `cargo bench -p z00z_storage --bench settlement_hjmt --no-run` passed.
- `cargo test --release` passed on the live tree after the Stage 13 cache stabilization fix; the prior simulator-linked failures are no longer reproducible on the current tree.

---

## Blocking Findings

No blocking findings remain on the current live tree.

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-12
