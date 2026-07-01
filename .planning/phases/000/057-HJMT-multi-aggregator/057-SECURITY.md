---
phase: 057
slug: hjmt-multi-aggregator
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-14
---

# Phase 057 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## 🔒 Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Runtime publication -> storage settlement | Runtime owns route lineage and publication binding, while storage owns committed shard roots and proof truth. | `ShardRootLeafV1`, `CheckpointPublicationV1`, route generation metadata, `PublicationBinding` |
| Runtime/storage -> rollup node | The node may validate and compose publication handoff state, but it must not become a second route or proof authority. | checkpoint handoff rows, topology metadata, prior-root continuity |
| Canonical publication -> validators/watchers | Downstream consumers must accept the same digest and verdict story that runtime and storage produced. | publication digest, binding digest, checkpoint continuity, verdict kind |
| Storage scope birth -> public publication | First-seen `definition_id` and `serial_id` births remain storage-owned and may surface only through lawful shard-root and checkpoint changes. | touched shard roots, touched shard leaves, checkpoint digest deltas |
| Simulator evidence -> live runtime lineage | Simulator traces prove execution outcomes only and must resolve back to the inherited runtime/config/process/journal lineage. | `leaf_flow.json`, `proof_flow.json`, `pub_flow.json`, `val_flow.json`, `watch_flow.json`, config/process lineage |
| Bench and closeout packet -> phase claims | Bench homes and planning closeout files may support claims, but they must stay attached to the live owner seams and not fork a second authority lane. | bench lane names, fixture manifests, summaries, `057-TEST-SPEC.md`, `057-TESTS-TASKS.md` |

## 🛡️ Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-057-01 | Tampering | Root-generation boundary | mitigate | `crates/z00z_storage/tests/test_hjmt_root_generation.rs` keeps pre-shard and post-shard generation handling explicit through `RootGenerationTagV1`, `ShardRootLeafV1`, and fail-closed manifest checks. | closed |
| T-057-02 | Integrity | Checkpoint publication ordering | mitigate | `test_hjmt_root_generation.rs` and `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs` keep canonical ordered shard leaves, prior-root continuity, and one checkpoint digest story. | closed |
| T-057-03 | Integrity | First publication bridge | mitigate | The checked fixture manifests plus `publication_contract_binds_runtime_route_metadata()` keep the first public checkpoint tied to the live Phase 056 lineage instead of a synthetic bridge. | closed |
| T-057-04 | Integrity | Layered proof surface | mitigate | `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs` and `test_hjmt_live_proof_families.rs` keep shard-local proofs semantic and public inclusion additive rather than replacing proof truth. | closed |
| T-057-05 | Replay | Historical proof lineage | mitigate | `test_hjmt_historical_proofs.rs` stores and checks `public_root_v1()`, route, shard, and prior-root continuity so old proofs cannot replay under a different public root or generation. | closed |
| T-057-06 | Tampering | Cross-shard and wrong-lineage rejects | mitigate | `test_cross_shard_counterexample_rejects()` plus validator publication-contract tests keep forbidden counterexamples fail-closed. | closed |
| T-057-07 | Spoofing | Topology-generic publication engine | mitigate | `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs` and the YAML-driven `scenario_config.yaml` keep `SIM-5A7S-PUB` an acceptance fixture rather than a hard-coded second engine. | closed |
| T-057-08 | Tampering | Runtime publication lineage | mitigate | `bind_publication_contract(...)`, `publication_contract_binds_runtime_route_metadata()`, and `test_scenario_settlement.rs` keep publication binding and digest truth attached to the real Phase 056 runtime lineage. | closed |
| T-057-09 | Repudiation | Trace emission linkage | mitigate | `crates/z00z_simulator/tests/test_scenario_settlement.rs` and `test_scenario1_stage_surface.rs` keep trace files evidentiary only and reject detached or stale publication traces. | closed |
| T-057-10 | Elevation of privilege | Join and transfer authority | mitigate | `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`, `test_hjmt_migrate.rs`, and `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs` allow owner change only after lawful route-generation and activation checkpoints. | closed |
| T-057-11 | Integrity | Carry-forward leaf bytes | mitigate | `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs` and `test_hjmt_migrate.rs` keep `FOV-G-002` through `FOV-G-004` byte-identical for unchanged carried-forward leaves. | closed |
| T-057-12 | Integrity | Crash recovery visibility | mitigate | `crates/z00z_storage/src/settlement/test_live_recovery.rs`, `test_hjmt_migrate.rs`, and `test_hjmt_split_brain_fencing.rs` reject partial or ambiguous public roots during recovery. | closed |
| T-057-13 | Integrity | Downstream truth reuse | mitigate | `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs` and `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` keep validators and watchers on the same binding and verdict story. | closed |
| T-057-14 | Spoofing | Scope-birth continuity | mitigate | `crates/z00z_storage/tests/test_hjmt_scope_birth.rs` and `test_scenario_settlement.rs` prove first-seen scope birth only through storage-owned shard-root and checkpoint deltas. | closed |
| T-057-15 | Integrity | Scenario/config/design drift | mitigate | `test_hjmt_runtime_config.rs`, `test_scenario1_stage_surface.rs`, and `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` keep executable stage flow synchronized with YAML and trace expectations. | closed |
| T-057-16 | Repudiation | False closeout claims | mitigate | `057-06-SUMMARY.md`, `057-TEST-SPEC.md`, `057-TESTS-TASKS.md`, and `crates/z00z_storage/tests/test_bench_lanes.rs` keep closure claims attached to the live gate matrix and owner homes. | closed |
| T-057-17 | Repudiation | Stale evidence packet | mitigate | `test_scenario1_stage_surface.rs`, the checked manifests, and the numbered `057-0X-SUMMARY.md` closeout artifacts keep traces, fixtures, and outputs bound to the executed tree. | closed |
| T-057-18 | Tampering | Second harness drift | mitigate | `crates/z00z_storage/tests/test_bench_lanes.rs`, `crates/z00z_storage/benches/settlement_hjmt.rs`, and `settlement_shard.rs` keep benchmark work on the accepted storage homes only. | closed |
| T-057-19 | Repudiation | False supersession of live continuation | mitigate | `057-07-PLAN.md`, `057-07-SUMMARY.md`, `057-TEST-SPEC.md`, and `057-TESTS-TASKS.md` renormalize the live continuation instead of leaving it mislabeled as historical-only. | closed |
| T-057-20 | Tampering | Continuation trace drift | mitigate | `test_scenario_settlement.rs`, validator/watcher publication-contract suites, and live guardrails keep `val_flow.json` and `watch_flow.json` aligned with the shared publication-binding contract. | closed |
| T-057-21 | Elevation of privilege | Local rebinding or digest fork | mitigate | `publication_binding_has_one_runtime_entry_point()`, `test_downstream_publication_binding_stays_shared()`, and the runtime-owned `bind_publication_contract(...)` path block second binding constructors or local digest lanes. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

## ⚠️ Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|

No accepted risks.

## 🧾 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-14 | 21 | 21 | 0 | Codex `gsd-secure-phase` |

## 🧪 Verification Evidence

- Workspace-first review of `057-01-PLAN.md` through `057-07-PLAN.md` confirmed
  a plan-authored 21-item threat register for Phase 057.
- Workspace-first review of `057-01-SUMMARY.md` through `057-07-SUMMARY.md`
  confirmed every plan-time threat closed on the executed tree and that the
  long-running `test_scenario1_stage_surface` and broad release gates were
  already recorded green where the slice required them.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the live tree during this security pass.
- `cargo test -p z00z_aggregators --release --features test-params-fast`
  passed on the live tree during this security pass.
- `cargo test -p z00z_storage --release --features test-params-fast` passed on
  the live tree during this security pass.
- `cargo test -p z00z_validators --release` passed on the live tree during
  this security pass.
- `cargo test -p z00z_watchers --release` passed on the live tree during this
  security pass.
- `cargo test -p z00z_simulator --release --features test-params-fast
  --features wallet_debug_tools --test test_hjmt_runtime_config -- --nocapture`
  passed on the live tree during this security pass.
- `cargo test -p z00z_simulator --release --features test-params-fast
  --features wallet_debug_tools --test test_scenario_settlement
  -- --nocapture` passed on the live tree during this security pass.

## 🚫 Blocking Findings

No blocking findings remain on the current live tree.

## ✅ Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-14
