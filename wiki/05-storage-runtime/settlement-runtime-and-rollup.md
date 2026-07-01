---
title: "Settlement Runtime And Rollup"
description: "How storage-owned settlement truth composes with runtime planning, validator/watcher surfaces, and rollup theorem verification."
---

The canonical seam is explicit and single-path: runtime owns bind and publish, storage owns settlement roots plus proof and recovery truth, and rollup owns only the final public theorem verification over those published artifacts. Validators, watchers, and simulator traces consume that same seam; they must not create competing roots, proof contracts, publication bindings, or theorem paths. `crates/z00z_storage/README.md:4-18` `crates/z00z_runtime/aggregators/README.md:3-29` `crates/z00z_rollup_node/README.md:3-15`

## 🎯 At A Glance

| Component | Responsibility | Key file | Source |
|---|---|---|---|
| Settlement module | Exposes `SettlementPath`, `SettlementStateRoot`, stores, proof checks, and object package contracts. | `crates/z00z_storage/src/settlement/mod.rs` | `crates/z00z_storage/src/settlement/mod.rs:32-93` |
| Aggregator runtime | Owns planning, placement, publication, and recovery boundaries. | `crates/z00z_runtime/aggregators/src/lib.rs` | `crates/z00z_runtime/aggregators/src/lib.rs:18-44` |
| Validator runtime | Owns checkpoint flow, tx/claim verification, and verdict emission. | `crates/z00z_runtime/validators/src/lib.rs` | `crates/z00z_runtime/validators/src/lib.rs:14-28` |
| Watcher runtime | Owns observation and alert surfaces over published state. | `crates/z00z_runtime/watchers/src/lib.rs` | `crates/z00z_runtime/watchers/src/lib.rs:13-20` |
| Rollup verifier | Verifies the canonical theorem bundle without reconstructing private witnesses. | `crates/z00z_rollup_node/src/lib.rs` | `crates/z00z_rollup_node/src/lib.rs:85-165` |

## 🧭 Ownership Flow

```mermaid
graph TB
  Wallet[Wallet or object producer] --> Settlement[SettlementStore]
  Settlement --> Aggregator[AggregatorService]
  Aggregator --> Validator[ValidatorService]
  Aggregator --> Watcher[WatcherService]
  Settlement --> Rollup[verify_settlement_theorem]

  style Wallet fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Settlement fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style Aggregator fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Validator fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Watcher fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Rollup fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
```
<!-- Sources: crates/z00z_storage/src/settlement/README.md:144-157, crates/z00z_runtime/aggregators/README.md:14-16, crates/z00z_runtime/validators/README.md:13-18, crates/z00z_runtime/watchers/README.md:11-16, crates/z00z_rollup_node/README.md:8-15 -->

```mermaid
sequenceDiagram
  autonumber
  box rgb(255,224,178) Storage / DA layer
    participant Store as SettlementStore
  end
  box rgb(255,243,224) Infrastructure / Runtime
    participant Agg as AggregatorService
  end
  box rgb(255,243,224) Infrastructure / Runtime
    participant Val as ValidatorService
  end
  box rgb(255,243,224) Infrastructure / Runtime
    participant Watch as WatcherService
  end
  box rgb(237,231,246) Crypto / Proof
    participant Rollup as verify_settlement_theorem
  end
  Store-->>Agg: semantic StoreOp and route context
  Agg-->>Val: resolved batch
  Agg-->>Watch: publication state
  Store-->>Rollup: checkpoint artifact and exec input
  Rollup-->>Rollup: verify tx package, link, root, inclusion
```
<!-- Sources: crates/z00z_storage/src/settlement/README.md:172-183, crates/z00z_runtime/aggregators/src/lib.rs:34-44, crates/z00z_rollup_node/src/lib.rs:97-165 -->

```mermaid
flowchart LR
  ProofBlob[ProofBlob and batch proof] --> StorageTruth[storage-owned verification]
  StorageTruth --> RuntimeMeta[runtime-owned planner metadata]
  RuntimeMeta --> Verdicts[validator and watcher outputs]
  Verdicts -. not semantic truth .-> StorageTruth

  style ProofBlob fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style StorageTruth fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style RuntimeMeta fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Verdicts fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
```
<!-- Sources: crates/z00z_storage/src/settlement/README.md:214-255, crates/z00z_runtime/watchers/README.md:13-16, crates/z00z_runtime/validators/README.md:15-18 -->

## 📦 Why Storage Is The Semantic Authority

| Storage export | Meaning | Why downstream consumers must not replace it | Source |
|---|---|---|---|
| `SettlementPath` | Canonical typed address `definition_id -> serial_id -> terminal_id`. | Downstream code should consume typed paths, not invent flat aliases. | `crates/z00z_storage/src/settlement/README.md:8-16` `crates/z00z_storage/src/settlement/README.md:82-103` |
| `SettlementStateRoot` and `CheckRoot` | Semantic settlement commitment and checkpoint-facing root type. | `backend_root` is intentionally private proof-local data. | `crates/z00z_storage/src/settlement/README.md:104-121` |
| `ObjectPolicyRegistryV1` and `RuntimeObjectPackageV1` | Storage-owned object package contract used by validators. | Validator and runtime layers reuse the contract rather than forking a second object-policy surface. | `crates/z00z_storage/src/settlement/mod.rs:45-49` `crates/z00z_runtime/validators/src/lib.rs:26-28` |

## 🔑 Downstream Boundaries

| Crate | What it may consume | What it must not claim | Source |
|---|---|---|---|
| `z00z_runtime/aggregators` | Route metadata, placement, publication binding, semantic `StoreOp` handoff. | Settlement semantics or proof truth. | `crates/z00z_runtime/aggregators/README.md:20-29` |
| `z00z_runtime/validators` | Resolved batches, storage-owned proof contracts, object packages. | Planner admission or watcher projection. | `crates/z00z_runtime/validators/README.md:13-18` |
| `z00z_runtime/watchers` | Published state and placement metadata for observation. | Semantic truth beyond evidence and alerting. | `crates/z00z_runtime/watchers/README.md:11-16` |
| `z00z_rollup_node` | `TxPackage`, `CheckpointArtifact`, `CheckpointExecInput`, `CheckpointLink`. | A second protocol owner or private-witness rebuilder. | `crates/z00z_rollup_node/src/lib.rs:85-165` |

## 📖 References

- `crates/z00z_storage/src/settlement/mod.rs:32-93`
- `crates/z00z_storage/src/settlement/README.md:82-121`
- `crates/z00z_runtime/aggregators/src/lib.rs:18-44`
- `crates/z00z_runtime/validators/src/lib.rs:14-28`
- `crates/z00z_rollup_node/src/lib.rs:97-165`

## Related Pages

| Page | Relationship |
|---|---|
| [Object Model And Genesis](../03-core-protocol/object-model-and-genesis.md) | Explains the objects that eventually become settlement leaves. |
| [Wallet Architecture](../04-wallet-and-rpc/wallet-architecture.md) | Shows the user-facing producer of many settlement objects. |
| [Scenario Pipeline](../06-simulator-and-quality/scenario-pipeline.md) | Demonstrates how these layers are exercised end to end. |
