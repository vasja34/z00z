---
title: "Settlement Recovery Failover"
description: "Recovery-state export, journal-lineage fencing, same-lineage standby takeover, and fail-closed restart rules across storage and runtime."
---

Recovery is intentionally modeled as a storage-exported semantic state plus a runtime-owned restart gate, not as a second source of truth. `SettlementStore::recovery_state()` exports the durable recovery record, runtime wraps that into `ShardRecoveryRecord`, and `RecoveryBoundary::resume(...)` only re-enters execution when journal lineage, routing generation, shard identity, state root, and backend generation metadata all still match. `crates/z00z_storage/src/settlement/README.md:193-210` `crates/z00z_storage/src/settlement/store.rs:659-700` `crates/z00z_runtime/aggregators/src/recovery.rs:63-212`

## 🎯 At A Glance

| Component | Responsibility | Key file | Source |
|---|---|---|---|
| Recovery contract README | Defines recovery metadata as the exported baseline and rejects shared WAL authority. | `crates/z00z_storage/src/settlement/README.md` | `crates/z00z_storage/src/settlement/README.md:193-210` |
| Recovery state type | Owns `version`, `state_root`, proof metadata, `journal_lineage`, and optional route context. | `crates/z00z_storage/src/settlement/store.rs` | `crates/z00z_storage/src/settlement/store.rs:241-298` |
| Runtime recovery gate | Enforces restart and standby takeover legality. | `crates/z00z_runtime/aggregators/src/recovery.rs` | `crates/z00z_runtime/aggregators/src/recovery.rs:63-212` |
| Distributed simulator | Pre-filters replicated state drift before calling `RecoveryBoundary`. | `crates/z00z_runtime/aggregators/src/dist_sim.rs` | `crates/z00z_runtime/aggregators/src/dist_sim.rs:272-356` |
| Recovery policy summary | Documents same-lineage takeover and fail-closed rejection classes. | `crates/z00z_runtime/aggregators/README.md` | `crates/z00z_runtime/aggregators/README.md:30-38` |
| Concrete scenarios | Proves same-lineage takeover and primary restart through tests. | `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs` | `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs:53-118` |

## 📦 Architecture

```mermaid
graph TB
  Store[SettlementStore] --> RS[SettlementRecoveryState]
  RS --> Rec[ShardRecoveryRecord]
  Rec --> Sim[DistSim sync_verdict]
  Sim --> Gate[RecoveryBoundary resume]
  Gate --> Ticket[ShardExecTicket RecoveryPending]

  style Store fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style RS fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style Rec fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Sim fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Gate fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Ticket fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
```
<!-- Sources: crates/z00z_storage/src/settlement/store.rs:659-700, crates/z00z_runtime/aggregators/src/recovery.rs:41-70, crates/z00z_runtime/aggregators/src/dist_sim.rs:272-356 -->

```mermaid
sequenceDiagram
  autonumber
  box rgb(255,224,178) Storage / DA layer
    participant Storage as SettlementStore
  end
  box rgb(255,243,224) Infrastructure / Runtime
    participant Runtime as RecoveryBoundary
  end
  box rgb(243,229,245) Domain logic
    participant Sim as DistSim
  end
  box rgb(243,229,245) Domain logic
    participant Exec as ShardExecTicket
  end
  Storage-->>Runtime: SettlementRecoveryState
  Runtime-->>Sim: ShardRecoveryRecord
  Sim->>Sim: sync_verdict on replicated state
  Sim->>Runtime: current replicated recovery state
  Runtime->>Runtime: lineage, route, root, and role checks
  Runtime-->>Exec: RecoveryPending ticket
```
<!-- Sources: crates/z00z_storage/src/settlement/store.rs:659-700, crates/z00z_runtime/aggregators/src/dist_sim.rs:272-356, crates/z00z_runtime/aggregators/src/recovery.rs:71-212 -->

```mermaid
stateDiagram-v2
  [*] --> Active
  Active --> Captured: capture()
  Captured --> RestartPrimary: requester is live primary
  Captured --> TakeoverStandby: requester is ready standby
  Captured --> Rejected: lineage or route drift
  Captured --> Rejected: stale root or metadata drift
  RestartPrimary --> RecoveryPending
  TakeoverStandby --> RecoveryPending

  classDef public fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  classDef domain fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  classDef storage fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  classDef danger fill:#FFE0E0,stroke:#D32F2F,stroke-width:1px,color:#B71C1C
  class Active public
  class Captured domain
  class RestartPrimary domain
  class TakeoverStandby storage
  class Rejected danger
  class RecoveryPending domain
```
<!-- Sources: crates/z00z_runtime/aggregators/src/recovery.rs:41-212, crates/z00z_runtime/aggregators/README.md:32-37 -->

## 🔑 Exported Recovery Payload

| Field | Meaning | Source |
|---|---|---|
| `version` | Durable HJMT version of the active state. | `crates/z00z_storage/src/settlement/store.rs:243-250` |
| `state_root` | Current semantic `SettlementStateRoot`. | `crates/z00z_storage/src/settlement/store.rs:243-250` |
| `root_generation` and `proof_version` | Storage proof-generation metadata that must remain stable across restart. | `crates/z00z_storage/src/settlement/store.rs:245-248` |
| `bucket_policy_generation` and `bucket_policy_id` | Active bucket policy identity carried into recovery checks. | `crates/z00z_storage/src/settlement/store.rs:248-249` |
| `journal_lineage` | Durable lineage digest used to reject wrong-lineage resumes. | `crates/z00z_storage/src/settlement/store.rs:250-280` |
| `route` | Optional `SettlementRouteCtx`; required for nonzero recovery versions in runtime resume. | `crates/z00z_storage/src/settlement/store.rs:251-282` `crates/z00z_runtime/aggregators/src/recovery.rs:104-145` |

## ⚙️ Resume Gate Conditions

| Check | Failure meaning | Source |
|---|---|---|
| Placement still owns the shard | Recovery route is no longer owned by the live placement table. | `crates/z00z_runtime/aggregators/src/recovery.rs:71-78` |
| Routing generation still matches | Placement drifted to a different route generation. | `crates/z00z_runtime/aggregators/src/recovery.rs:80-85` |
| Live primary still matches | Prevents split-brain primary drift. | `crates/z00z_runtime/aggregators/src/recovery.rs:87-92` |
| Expected journal lineage matches everywhere | Rejects wrong-lineage recovery. | `crates/z00z_runtime/aggregators/src/recovery.rs:94-102` |
| Durable route metadata still matches shard placement | Rejects wrong shard, wrong route digest, and stale batch replay. | `crates/z00z_runtime/aggregators/src/recovery.rs:104-145` |
| Durable version and state root still match | Rejects stale restart and stale local root. | `crates/z00z_runtime/aggregators/src/recovery.rs:148-170` |
| Requester role is lawful | Only the live primary may restart; only a ready standby may take over. | `crates/z00z_runtime/aggregators/src/recovery.rs:173-205` |

## 📌 Distributed Simulation Layer

`DistSim` is not an optional toy layer here. Before it delegates to `RecoveryBoundary`, it checks that the requesting node actually holds replicated state for the latest batch, latest `journal_lineage`, latest version, latest state root, latest backend generation metadata, and the same route context. That catches partial replay, wrong-lineage replication, stale local roots, and wrong-shard route drift before runtime even evaluates the role-based resume rules. `crates/z00z_runtime/aggregators/src/dist_sim.rs:272-356`

The runtime README summarizes the same contract in operational language: same-lineage standby takeover is lawful only when shard id, routing generation, journal lineage, and live local root metadata all match; wrong lineage, wrong generation, stale local root, stale restart, standby down, and split-brain states reject fail-closed. `crates/z00z_runtime/aggregators/README.md:30-38`

## Related Pages

| Page | Relationship |
|---|---|
| [Publication Route Authority](./publication-route-authority.md) | Explains the lawful runtime-to-storage handoff path that precedes recovery evidence. |
| [Runtime Aggregator Surface](./runtime-aggregator-surface.md) | Covers the broader planner, placement, dispatch, and consensus seams around this recovery gate. |
| [Settlement Path Proofs](./settlement-path-proofs.md) | Explains the proof-generation metadata reused in the recovery payload. |
