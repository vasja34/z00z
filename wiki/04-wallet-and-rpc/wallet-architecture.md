---
title: "Wallet Architecture"
description: "Stable wallet facades, typed object inventory, services, and JSON-RPC registration."
---

The wallet crate is not an asset-only balance engine anymore. Its documented public model is a typed object inventory in which assets remain spendable cash, vouchers are conditional claims, and rights are authority inventory with zero spendable value. `crates/z00z_wallets/README.md:11-37`

## 🎯 At A Glance

| Component | Responsibility | Key file | Source |
|---|---|---|---|
| Wallet root facade | Re-exports stealth, wallet errors, services, receiver, and tx surfaces. | `crates/z00z_wallets/src/lib.rs` | `crates/z00z_wallets/src/lib.rs:97-156` |
| Services facade | Publishes orchestration boundaries like `WalletService` and `AppService`. | `crates/z00z_wallets/src/services/mod.rs` | `crates/z00z_wallets/src/services/mod.rs:1-24` |
| RPC surface | Groups method modules and registration helpers for wallet/app dispatchers. | `crates/z00z_wallets/src/rpc/mod.rs` | `crates/z00z_wallets/src/rpc/mod.rs:24-91` |
| Transport seam | Supplies transport-only dispatch and local in-process testing helpers. | `crates/z00z_networks/rpc/src/lib.rs` | `crates/z00z_networks/rpc/src/lib.rs:4-19` |

## 🧭 Wallet Surface

```mermaid
graph TB
  Wallet[z00z_wallets root] --> Stealth[stealth output surface]
  Wallet --> Receiver[receiver facade]
  Wallet --> Tx[tx facade]
  Wallet --> Services[services facade]
  Wallet --> Rpc[rpc module]
  Wallet --> Db[native db facade]

  style Wallet fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Stealth fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Receiver fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style Tx fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style Services fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Rpc fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style Db fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
```
<!-- Sources: crates/z00z_wallets/src/lib.rs:97-156, crates/z00z_wallets/README.md:171-183 -->

```mermaid
sequenceDiagram
  autonumber
  box rgb(227,242,253) Public API / User
    participant Caller
  end
  box rgb(243,229,245) Domain logic
    participant Rpc as wallet RPC
  end
  box rgb(255,243,224) Infrastructure / Runtime
    participant Svc as WalletService
  end
  box rgb(255,224,178) Storage / DA layer
    participant Store as object inventory or asset store
  end
  Caller->>Rpc: wallet.object.* or wallet.asset.*
  Rpc->>Svc: typed request
  Svc->>Store: load or mutate owned objects
  Store-->>Svc: typed result
  Svc-->>Rpc: wallet result
  Rpc-->>Caller: JSON-RPC response
```
<!-- Sources: crates/z00z_wallets/README.md:23-37, crates/z00z_wallets/src/rpc/mod.rs:43-91, crates/z00z_wallets/src/services/mod.rs:16-24 -->

```mermaid
flowchart LR
  Asset[Asset id] --> Spendable[Spendable value lane]
  Voucher[Voucher id] --> ObjectRpc[wallet.object.*]
  Right[Right id] --> ObjectRpc
  Voucher -. not accepted .-> AssetRpc[wallet.asset.*]
  Right -. not accepted .-> AssetRpc

  style Asset fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Spendable fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Voucher fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style ObjectRpc fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Right fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style AssetRpc fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
```
<!-- Sources: crates/z00z_wallets/README.md:16-25, crates/z00z_wallets/README.md:38-44 -->

## 📦 Typed Object Model In The Wallet

| Object family | Wallet meaning | Allowed public lane | Source |
|---|---|---|---|
| Assets | Spendable value. | `wallet.asset.*` and `wallet.object.*` projections. | `crates/z00z_wallets/README.md:16-24` |
| Vouchers | Conditional claims with explicit lifecycle and redemption paths. | `wallet.object.*` only. | `crates/z00z_wallets/README.md:16-24` |
| Rights | Authority inventory contributing zero to spendable balance. | `wallet.object.*` only. | `crates/z00z_wallets/README.md:18-25` |
| Unknown-policy objects | Durable quarantine until policy descriptors are accepted. | Quarantine, not spendable balance. | `crates/z00z_wallets/README.md:20-21` `crates/z00z_wallets/README.md:40-44` |

## 🔑 Stable Facades And Internal Detail

| Preferred entrypoint | Why it is preferred | Source |
|---|---|---|
| `z00z_wallets::db::{WltSession, ScanStatePayload}` | Stable wallet-store boundary types. | `crates/z00z_wallets/README.md:171-177` |
| `z00z_wallets::services::{RateLimitPrecheck, WalletService}` | Canonical orchestration layer. | `crates/z00z_wallets/README.md:173-175` `crates/z00z_wallets/src/services/mod.rs:16-24` |
| `z00z_wallets::receiver::*` | Receiver-card and payment-request flows. | `crates/z00z_wallets/README.md:175-177` |
| `z00z_wallets::tx::{ClaimTxVerifier, TxVerifier}` | Transaction verification entrypoints. | `crates/z00z_wallets/README.md:177-181` |

## 📖 References

- `crates/z00z_wallets/README.md:11-44`
- `crates/z00z_wallets/src/lib.rs:97-156`
- `crates/z00z_wallets/src/rpc/mod.rs:24-91`
- `crates/z00z_wallets/src/services/mod.rs:1-24`
- `crates/z00z_networks/rpc/src/lib.rs:4-19`

## Related Pages

| Page | Relationship |
|---|---|
| [Object Model And Genesis](../03-core-protocol/object-model-and-genesis.md) | Defines the object families the wallet projects. |
| [Settlement Runtime And Rollup](../05-storage-runtime/settlement-runtime-and-rollup.md) | Shows where wallet outputs land once committed. |
| [Networking And Telemetry](../07-networking-and-observability/networking-and-telemetry.md) | Expands the transport seam the wallet composes. |
