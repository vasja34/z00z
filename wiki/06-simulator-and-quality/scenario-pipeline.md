---
title: "Scenario Pipeline"
description: "How scenario_1 acts as the canonical integration harness and how it ties into the repository verification story."
---

`z00z_simulator` is intentionally an integration harness, not a second application framework. Its README and module root are explicit that scenario code must enter other crates through stable facades and that `scenario_1` remains the canonical executable home for the Phase 059 object model. `crates/z00z_simulator/README.md:6-30` `crates/z00z_simulator/README.md:62-92`

## 🎯 At A Glance

| Component | Responsibility | Key file | Source |
|---|---|---|---|
| Simulator root | Re-exports actor, config, context, design, result, and scenario facades. | `crates/z00z_simulator/src/lib.rs` | `crates/z00z_simulator/src/lib.rs:6-39` |
| `scenario_1` module | Declares stages, runner, and process guard for the canonical scenario pipeline. | `crates/z00z_simulator/src/scenario_1/mod.rs` | `crates/z00z_simulator/src/scenario_1/mod.rs:8-37` |
| CLI binary | Resolves `--config` and `--design` arguments, then delegates to the runner. | `crates/z00z_simulator/bin/scenario_1.rs` | `crates/z00z_simulator/bin/scenario_1.rs:11-73` |
| Release-style verify gate | Supplies the heavy repository verification path that the simulator complements. | `.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` | `.github/skills/z00z-full-verify-gate/scripts/full_verify.sh:64-103` |

## 🧭 Scenario Shape

```mermaid
graph TB
  Scenario1[scenario_1] --> Stage1[stage_1]
  Scenario1 --> Stage4[stage_4]
  Scenario1 --> Stage9[stage_9]
  Scenario1 --> Stage13[stage_13]
  Scenario1 --> Runner[runner and verification helpers]
  Scenario1 --> Support[support modules]

  style Scenario1 fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Stage1 fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Stage4 fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Stage9 fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Stage13 fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Runner fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Support fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
```
<!-- Sources: crates/z00z_simulator/src/scenario_1/mod.rs:8-27 -->

```mermaid
sequenceDiagram
  autonumber
  box rgb(255,243,224) Infrastructure / Runtime
    participant CLI as scenario_1 binary
  end
  box rgb(255,243,224) Infrastructure / Runtime
    participant Runner as scenario_1::runner
  end
  box rgb(255,243,224) Infrastructure / Runtime
    participant Harness as Simulator context
  end
  box rgb(232,245,233) External / Validation
    participant Artifacts as Public artifacts
  end
  CLI->>Runner: run default or path-based execution
  Runner->>Harness: build scenario context and stage flow
  Harness->>Artifacts: emit flow JSON and summary material
  Artifacts-->>CLI: scenario result and stage statuses
```
<!-- Sources: crates/z00z_simulator/bin/scenario_1.rs:11-67, crates/z00z_simulator/src/lib.rs:23-39, crates/z00z_simulator/README.md:75-92 -->

```mermaid
flowchart LR
  StableFacades[Stable crate facades] --> Scenario[Simulator stages]
  Scenario --> Evidence[asset_flow / voucher_flow / right_flow]
  Evidence --> Verify[full verify or release review]
  Verify --> Decision[accept or debug]

  style StableFacades fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Scenario fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Evidence fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Verify fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Decision fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
```
<!-- Sources: crates/z00z_simulator/README.md:24-30, crates/z00z_simulator/README.md:75-92, .github/skills/z00z-full-verify-gate/scripts/full_verify.sh:79-83 -->

## 📦 Why `scenario_1` Matters

| Aspect | Current contract | Source |
|---|---|---|
| Canonical executable | `scenario_1` is the live home of the Phase 059 object model and was extended in place. | `crates/z00z_simulator/README.md:62-66` |
| Object lanes | Covers asset transfer, voucher lifecycle, right lifecycle, and right-gated voucher actions. | `crates/z00z_simulator/README.md:67-73` |
| Artifact anchors | Publishes `asset_flow.json`, `voucher_flow.json`, `right_flow.json`, `wallet_scan.json`, `val_flow.json`, `watch_flow.json`, and `sim_summary.md`. | `crates/z00z_simulator/README.md:75-89` |
| Negative evidence | Reject and fix surfaces are mandatory in the Phase 059 packet. | `crates/z00z_simulator/README.md:91-93` |

## 🔑 Integration Boundary Rules

| Rule | Meaning in practice | Source |
|---|---|---|
| Use stable facades | If the harness needs a new entrypoint, add it to the owner crate instead of deep-importing internals. | `crates/z00z_simulator/README.md:24-30` |
| Harness-only code stays narrow | Deterministic fixture setup is fine; business-rule ownership is not. | `crates/z00z_simulator/README.md:12-22` |
| Secret artifact policy is strict | Plaintext wallet-secret artifacts are debug-only, gated, and outside the default contract. | `crates/z00z_simulator/README.md:32-45` |

## 📖 References

- `crates/z00z_simulator/README.md:6-30`
- `crates/z00z_simulator/README.md:62-92`
- `crates/z00z_simulator/src/lib.rs:6-39`
- `crates/z00z_simulator/src/scenario_1/mod.rs:8-37`
- `crates/z00z_simulator/bin/scenario_1.rs:11-73`

## Related Pages

| Page | Relationship |
|---|---|
| [Workspace Overview](../01-getting-started/workspace-overview.md) | Gives the top-level context for why the simulator exists as its own crate. |
| [Wallet Architecture](../04-wallet-and-rpc/wallet-architecture.md) | One of the main stable facades the simulator composes. |
| [Settlement Runtime And Rollup](../05-storage-runtime/settlement-runtime-and-rollup.md) | Covers the downstream layers the simulator exercises. |
