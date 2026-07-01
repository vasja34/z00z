---
title: "Crate Boundaries"
description: "How Z00Z distributes ownership across crates and which seams are intentionally reserved."
---

Z00Z’s most important architectural habit is that each crate says what it owns and what it must not absorb. The READMEs and crate roots are unusually explicit about these negative boundaries, so they are the right source for deciding whether a change belongs in an owner crate, a transport seam, or the simulator. `crates/z00z_utils/README.md:3-25` `crates/z00z_runtime/aggregators/README.md:18-29` `crates/z00z_simulator/README.md:12-22`

## 🎯 At A Glance

| Boundary owner | Owns | Refuses to own | Source |
|---|---|---|---|
| `z00z_utils` | Shared infrastructure primitives. | Product-domain behavior. | `crates/z00z_utils/README.md:3-25` |
| `z00z_networks_rpc` | Transport dispatch and adaptation. | Peer identity, auth, retry, lifecycle policy. | `crates/z00z_networks/rpc/src/lib.rs:4-19` |
| `z00z_storage` | Settlement roots, proofs, and stores. | Generic backup ownership and public physical-root authority. | `crates/z00z_storage/README.md:4-18` `crates/z00z_storage/src/settlement/README.md:104-121` |
| `z00z_simulator` | Cross-crate harness behavior and artifacts. | A second owner for wallet, storage, crypto, or network rules. | `crates/z00z_simulator/README.md:12-22` |

## 🧭 Ownership Decision Tree

```mermaid
flowchart TD
  Start[Need to add behavior] --> Scope{What changes?}
  Scope -->|Generic I/O or config| Utils[z00z_utils]
  Scope -->|Protocol object semantics| Core[z00z_core]
  Scope -->|Wallet UX, inventory, or tx wire| Wallet[z00z_wallets]
  Scope -->|Settlement root, proof, or path| Storage[z00z_storage]
  Scope -->|Planner or verdict path| Runtime[z00z_runtime/*]
  Scope -->|Only harness glue| Simulator[z00z_simulator]

  style Start fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Scope fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style Utils fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Core fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Wallet fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Storage fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style Runtime fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Simulator fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
```
<!-- Sources: crates/z00z_utils/README.md:11-25, crates/z00z_wallets/README.md:23-37, crates/z00z_storage/src/settlement/README.md:82-121, crates/z00z_simulator/README.md:12-22 -->

```mermaid
graph LR
  Utils[z00z_utils] -->|shared primitives only| Core[z00z_core]
  Rpc[z00z_networks_rpc] -->|transport only| Wallet[z00z_wallets]
  Onion[OnionNet] -->|overlay placeholder| Wallet
  Storage[z00z_storage] -->|settlement truth| Runtime[z00z_runtime]
  Runtime -->|planning and verdicts| Rollup[z00z_rollup_node]
  Simulator[z00z_simulator] -->|stable facades only| Wallet
  Simulator --> Storage

  style Utils fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Core fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Rpc fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Wallet fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Onion fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Storage fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style Runtime fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Rollup fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Simulator fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
```
<!-- Sources: crates/z00z_utils/README.md:53-77, crates/z00z_networks/rpc/README.md:5-18, crates/z00z_networks/onionnet/README.md:16-31, crates/z00z_simulator/README.md:24-30 -->

```mermaid
sequenceDiagram
  autonumber
  box rgb(227,242,253) Public API / User
    participant Dev
  end
  box rgb(236,239,241) Neutral / Support
    participant README as Owner README
  end
  box rgb(236,239,241) Neutral / Support
    participant Facade as Stable Facade
  end
  box rgb(236,239,241) Neutral / Support
    participant Impl as Internal Module
  end
  Dev->>README: confirm owner and exclusions
  README-->>Dev: preferred seam
  Dev->>Facade: make change at supported boundary
  Facade-->>Dev: stable public contract
  Dev->>Impl: descend only when owner is confirmed
```
<!-- Sources: crates/z00z_core/README.md:3-20, crates/z00z_wallets/README.md:171-183, crates/z00z_storage/src/settlement/README.md:144-157 -->

## 🔑 Reserved And Protected Seams

| Seam | Why it exists | Practical implication | Source |
|---|---|---|---|
| `z00z_networks/onionnet` placeholder | Reserves the namespace and module layout before live overlay code lands. | New privacy-overlay work should fill reserved modules in place instead of inventing a new crate. | `crates/z00z_networks/onionnet/README.md:3-31` |
| `z00z_telemetry` thin facade | Holds one stable observability entrypoint without yet claiming richer behavior. | Observability code should point here, but domain semantics still belong elsewhere. | `crates/z00z_telemetry/README.md:3-12` |
| `z00z_extensions` boundary | Preserves a space for repository-owned add-ons. | Extensions may compose owner APIs but must not fork ownership. | `crates/z00z_extensions/README.md:3-12` |

## 📖 References

- `crates/z00z_utils/README.md:3-25`
- `crates/z00z_networks/rpc/src/lib.rs:4-19`
- `crates/z00z_storage/src/settlement/README.md:104-121`
- `crates/z00z_runtime/aggregators/README.md:18-29`
- `crates/z00z_simulator/README.md:12-30`

## Related Pages

| Page | Relationship |
|---|---|
| [Workspace Map](../01-getting-started/workspace-map.md) | Lists the crates whose boundaries are described here. |
| [Wallet Architecture](../04-wallet-and-rpc/wallet-architecture.md) | Shows one of the most boundary-sensitive crates in detail. |
| [Networking And Telemetry](../07-networking-and-observability/networking-and-telemetry.md) | Expands the transport and reserved-overlay seams mentioned here. |
