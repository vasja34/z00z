# Phase 056: Source Audit

**Date:** 2026-06-11
**Status:** review-hardened planning coverage map

## 🎯 Purpose

This document proves that the full `056-TODO.md` planning authority is routed
into the Phase 056 planning packet without creating a duplicate design layer.
It is evidence for review and doublecheck, not a replacement for
`056-CONTEXT.md` or the numbered plans.

## 🔑 Coverage Summary

| Source authority | Packet destination | Status |
| --- | --- | --- |
| `056-TODO.md` boundary language | `056-CONTEXT.md` domain + decisions | Covered |
| `056-TODO.md` workstreams | `056-01` through `056-06` | Covered |
| `056-TODO.md` gates `056-G1`..`056-G10` | `056-CONTEXT.md` gate map + numbered plans | Covered |
| `056-TODO.md` tests, benches, execution profiles, artifacts, fixtures, exit criteria | `056-07-PLAN.md` plus dependent slices | Covered |
| Upgrade, fixture, and design doc corpus named by `056-TODO.md` | `056-CONTEXT.md` cross-read contracts + plan coverage contracts | Covered |
| No-duplicate-layer and no-concept-drift rule | `056-CONTEXT.md` decisions D-02, D-03, D-07, D-09, D-18 | Covered |

## 🧭 Live Path Corrections Used By This Packet

| Contract or concern in TODO | Verified live anchor | Planning rule |
| --- | --- | --- |
| Runtime planner truth | `crates/z00z_runtime/aggregators/src/batch_planner.rs` | Do not reimplement in storage or simulator. |
| Placement and standby metadata | `crates/z00z_runtime/aggregators/src/placement.rs` | Keep operational only; do not promote to protocol truth. |
| Recovery boundary | `crates/z00z_runtime/aggregators/src/recovery.rs` | Extend in place; do not build a second recovery authority. |
| Semantic settlement truth | `crates/z00z_storage/src/settlement/*` | Runtime hands off semantic work only. |
| Durable seam | `crates/z00z_storage/src/backend/mod.rs` | Keep `StorageBackend` and `JournalBackend` authoritative. |
| Node composition root | `crates/z00z_rollup_node/src/runtime.rs` | Do not add a new super-orchestrator layer. |
| Live scenario config | `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` | Remains the executable simulator config anchor. |
| Live design sync doc | `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` | Must update in the same slice as runtime stage drift. |
| New runtime YAML surfaces | proposed `config/hjmt_runtime/` | Proposed only until `056-01` verifies the final layout. |

## ✅ TODO Section Transfer Matrix

| TODO section | Reflected in context | Reflected in plans |
| --- | --- | --- |
| `Mission` | Yes | `056-01`..`056-07` |
| `This phase owns` | Yes | `056-01`..`056-07` |
| `This phase does not own` | Yes | `056-01`, `056-06`, `056-07` |
| `Phase handoff` | Yes | `056-06`, `056-07` |
| `Global upgrade rules active in every HJMT phase` | Yes | all numbered plans |
| `Inputs inherited from Phase 055 and consumed here` | Yes | `056-02`..`056-07` |
| `Primary upgrade sections owned by Phase 056` | Yes | `056-01`..`056-07` |
| `Required cross-read sections` | Yes | all numbered plans |
| `Fixture ownership for this phase` | Yes | `056-02`, `056-04`, `056-07` |
| `Embedded audit contract` | Yes | `056-02`, `056-04`, `056-05`, `056-06` |
| `Canonical SIM-5A7S runtime profile` | Yes | `056-01`, `056-04`, `056-06`, `056-07` |
| `Process and config contract` | Yes | `056-01`, `056-05`, `056-06` |
| `Journal and WAL decision captured by this phase` | Yes | `056-04` |
| `Startup self-test gate owned by this phase` | Yes | `056-05` |
| `Release blockers owned or co-owned by Phase 056` | Yes | `056-01`..`056-07` |
| `Phase-owned acceptance subset` | Yes | `056-01`..`056-07` |
| `Mandatory implementation gates` | Yes | gate-owned plans |
| `Workstream 1` | Yes | `056-01` |
| `Workstream 2` | Yes | `056-02` |
| `Workstream 3` | Yes | `056-03` |
| `Workstream 4` | Yes | `056-04` |
| `Workstream 5` | Yes | `056-05` |
| `Workstream 6` | Yes | `056-06` |
| `Required test coverage` | Yes | `056-07` |
| `Required benchmark slices` | Yes | `056-07` |
| `Required execution profiles` | Yes | `056-06`, `056-07` |
| `Required scenario coverage` | Yes | `056-02`..`056-07` |
| `Required artifacts` | Yes | `056-01`, `056-03`, `056-04`, `056-05`, `056-06`, `056-07` |
| `Fixture ownership` | Yes | `056-02`, `056-04`, `056-07` |
| `Exit criteria` | Yes | `056-07` |

## 🔒 Crypto-Architect Findings Integrated

| Finding | Packet response |
| --- | --- |
| Route-table bytes, generation, and digest are security-critical composition inputs. | `056-02` owns canonical route-table contract, digest binding, and planner-mode equivalence. |
| Silent reroute is a security failure, not an availability shortcut. | `056-04` owns same-lineage failover acceptance and wrong-lineage/split-brain reject matrix. |
| Canonical serialization and fail-closed parser behavior must precede trust. | `056-02` and `056-05` require route-table and startup preflight codec validation before live work. |
| Protocol truth and operational placement must remain distinct. | `056-CONTEXT.md` decisions D-03, D-07, and D-12 freeze that split. |
| Proof and root-generation tags remain security-critical even when Phase 056 is not a proof phase. | `056-05` startup preflight includes proof/root tag expectation checks; `056-06` keeps trace evidence tied to compiled expectations. |

## 🛡️ Security-Audit Findings Integrated

| Finding | Packet response |
| --- | --- |
| New YAML surfaces must use existing config abstractions and reject partial or inconsistent state. | `056-05` routes config loading through repository config abstractions and fail-closed preflight. |
| Trace artifacts can become misleading truth sources if detached from runtime lineage. | `056-CONTEXT.md` decisions D-14 and D-15 plus `056-06` require config digest, route digest, journal lineage, and process topology linkage. |
| New process topology work can accidentally smuggle shared-memory shortcuts. | `056-01` makes separate OS process behavior a hard acceptance contract. |
| Import/export and restart behavior can create replay or stale-state acceptance if lineage checks are weak. | `056-04` schedules lineage, generation, and stale-root reject coverage before closeout. |

## 🧪 Doublecheck Coverage Claims To Re-Verify After Plan Creation

1. Every major `056-TODO.md` section is mapped to `056-CONTEXT.md` and at
   least one numbered plan.
2. No numbered plan claims an already-existing path for
   `aggregator-config.yaml`, `planner-config.yaml`, or `storage-config.yaml`
   before `056-01` verifies it.
3. No numbered plan duplicates runtime planner truth, storage semantic truth,
   or simulator evidence ownership.
4. Gates `056-G1` through `056-G10` each have one primary owner plan and one
   explicit closeout path.

## 📋 Literal Bullet Ledger

This ledger exists because section-level coverage was not enough for a strict
Phase 056 review. Each bullet class below must stay explicit in both context
and the numbered packet.

| TODO bullet class | Context anchor | Plan anchor |
| --- | --- | --- |
| `AggregatorId(0)..AggregatorId(4)` and `ShardId(0)..ShardId(6)` canonical fixture membership | `056-CONTEXT.md` literal bullet preservation map | `056-01` |
| Placement spread and standby expectation in `SIM-5A7S` | `056-CONTEXT.md` literal bullet preservation map | `056-01`, `056-04` |
| Batch profiles `broad` / `hot-shard` / `hot-serial` / `delete-heavy` / `search-heavy` / `proof-heavy` / `mixed present or absent` / cross-shard reject | `056-CONTEXT.md` literal bullet preservation map | `056-02`, `056-03`, `056-07` |
| Failure profiles `primary down` / `standby down` / `stale restart` / `wrong lineage` / `split-brain` / `route migration during crash` / publication handoff | `056-CONTEXT.md` literal bullet preservation map | `056-04`, `056-07` |
| Route fixtures `SRT-G-001`..`SRT-G-004` and tamper fixtures `SRT-T-001`..`SRT-T-008` | `056-CONTEXT.md` literal bullet preservation map | `056-02`, `056-07` |
| Failover fixtures `FOV-001`, `FOV-T-001`, `FOV-T-002` | `056-CONTEXT.md` literal bullet preservation map | `056-04`, `056-07` |
| Upgrade `12.1` owned fixtures `Route migration fixture` and `Failover fixture` | `056-CONTEXT.md` literal bullet preservation map | `056-02`, `056-04`, `056-07` |
| Required tests list | `056-CONTEXT.md` literal bullet preservation map | `056-01`..`056-07`, closed by `056-07` |
| Execution profiles `SIM-SMALL`, `SIM-MEDIUM`, `SIM-CACHE-EDGE`, and reserved `SIM-BATCH-1000` rule | `056-CONTEXT.md` literal bullet preservation map | `056-06`, `056-07` |
| Trace artifacts `cfg_flow.json` through `recovery_flow.json` | `056-CONTEXT.md` literal bullet preservation map | `056-03`, `056-04`, `056-06`, `056-07` |
| Config change proof bullets for ports, placement, planner mode, journal path, backend selection, and simulator scenario selection | `056-CONTEXT.md` literal bullet preservation map | `056-05`, `056-06`, `056-07` |
