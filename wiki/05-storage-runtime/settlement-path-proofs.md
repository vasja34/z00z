---
title: "Settlement Path Proofs"
description: "Canonical settlement path shape, semantic versus physical roots, and the storage-owned proof contracts around ProofBlob, chk_item_settlement, chk_blob_settlement, and BatchProofBlobV1."
---

The settlement surface is intentionally typed around one canonical path shape: `definition_id -> serial_id -> terminal_id`. Everything else in the proof model hangs off that path discipline. The storage README is explicit that flat aliases, raw backend proof types, and root conflation are outside the public contract, and `mod.rs` re-exports only storage-owned roots, path types, proof envelopes, and validators. `crates/z00z_storage/src/settlement/README.md:3-16` `crates/z00z_storage/src/settlement/mod.rs:32-93`

## 🎯 At A Glance

| Component | Responsibility | Key file | Source |
|---|---|---|---|
| Public contract README | Defines canonical path order, root roles, and proof ownership. | `crates/z00z_storage/src/settlement/README.md` | `crates/z00z_storage/src/settlement/README.md:8-16` `crates/z00z_storage/src/settlement/README.md:104-120` |
| Public facade | Re-exports roots, path types, proof types, batch proof types, and validators. | `crates/z00z_storage/src/settlement/mod.rs` | `crates/z00z_storage/src/settlement/mod.rs:32-93` |
| Single-path proof envelope | Defines `ProofBlob`, `backend_root`, root binding, and blob validators. | `crates/z00z_storage/src/settlement/proof.rs` | `crates/z00z_storage/src/settlement/proof.rs:457-601` `crates/z00z_storage/src/settlement/proof.rs:1132-1218` |
| Batch proof contract | Verifies the shared batch envelope fail-closed, including root binding and canonical path ordering. | `crates/z00z_storage/src/settlement/proof_batch_verify.rs` | `crates/z00z_storage/src/settlement/proof_batch_verify.rs:65-73` `crates/z00z_storage/src/settlement/proof_batch_verify.rs:189-220` |
| Storage facade | Keeps semantic settlement APIs and proof emission behind `SettlementTreeBackend`. | `crates/z00z_storage/src/settlement/store.rs` | `crates/z00z_storage/src/settlement/store.rs:309-470` |

## 📦 Architecture

```mermaid
graph TB
  Path[SettlementPath] --> Store[SettlementStore]
  Store --> Root[SettlementStateRoot]
  Root --> Check[CheckRoot]
  Store --> Item[ProofItem]
  Store --> Blob[ProofBlob]
  Blob --> Batch[BatchProofBlobV1]
  Item --> ItemChk[chk_item_settlement]
  Blob --> BlobChk[chk_blob_settlement]
  Batch --> BatchChk[check_batch_contract_v1]

  style Path fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style Store fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style Root fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style Check fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style Item fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style Blob fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style Batch fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
  style ItemChk fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style BlobChk fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style BatchChk fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
```
<!-- Sources: crates/z00z_storage/src/settlement/README.md:126-149, crates/z00z_storage/src/settlement/mod.rs:38-87, crates/z00z_storage/src/settlement/store.rs:313-387, crates/z00z_storage/src/settlement/proof.rs:1132-1232, crates/z00z_storage/src/settlement/proof_batch_verify.rs:65-73 -->

```mermaid
sequenceDiagram
  autonumber
  box rgb(227,242,253) Public API / User
    participant Caller
  end
  box rgb(255,224,178) Storage / DA layer
    participant Store as SettlementStore
  end
  box rgb(237,231,246) Crypto / Proof
    participant Blob as ProofBlob
  end
  box rgb(255,224,178) Storage / DA layer
    participant Verify as chk_blob_settlement
  end
  Caller->>Store: settlement_proof_blob(path)
  Store-->>Caller: encoded proof blob
  Caller->>Verify: bytes plus expected root/path/leaves
  Verify->>Verify: decode blob and check semantic binding
  Verify->>Verify: verify definition, serial, terminal proofs against backend_root
  Verify-->>Caller: validated ProofBlob
```
<!-- Sources: crates/z00z_storage/src/settlement/README.md:216-254, crates/z00z_storage/src/settlement/proof.rs:1161-1218 -->

```mermaid
flowchart LR
  D[DefinitionId] --> S[SerialId]
  S --> T[TerminalId]
  T --> L[SettlementLeaf]
  L --> SR[SettlementStateRoot]
  SR --> CR[CheckRoot]
  T --> BR[backend_root only for proof verification]

  style D fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style S fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style T fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style L fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style SR fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style CR fill:#FFE0B2,stroke:#F57C00,stroke-width:1px,color:#E65100
  style BR fill:#EDE7F6,stroke:#5E35B1,stroke-width:1px,color:#311B92
```
<!-- Sources: crates/z00z_storage/src/settlement/README.md:84-120, crates/z00z_storage/src/settlement/proof.rs:477-507, crates/z00z_storage/src/settlement/proof.rs:581-589 -->

## 🔑 Path And Root Model

| Contract | Meaning | Source |
|---|---|---|
| `DefinitionId` | Namespace-level identity for one definition family. | `crates/z00z_storage/src/settlement/README.md:84-91` |
| `SerialId` | Serial bucket within one definition namespace. | `crates/z00z_storage/src/settlement/README.md:84-91` |
| `TerminalId` | Terminal settlement leaf identity. | `crates/z00z_storage/src/settlement/README.md:84-91` |
| `SettlementStateRoot` | Public semantic commitment for the canonical hierarchy. | `crates/z00z_storage/src/settlement/README.md:94-100` |
| `CheckRoot` | Checkpoint-facing type derived from `SettlementStateRoot`. | `crates/z00z_storage/src/settlement/README.md:96-100` |
| `backend_root` | Proof-local physical root bytes, never a public state root. | `crates/z00z_storage/src/settlement/README.md:108-120` `crates/z00z_storage/src/settlement/proof.rs:581-589` |

## 📁 Proof Surface

| API | What it proves | Important boundary | Source |
|---|---|---|---|
| `ProofItem` | Semantic tuple of root, path, definition leaf, serial leaf, and leaf payload. | No backend branch proofs. | `crates/z00z_storage/src/settlement/README.md:233-240` |
| `ProofBlob` | One storage-owned witness blob with semantic item plus backend proof bytes. | Carries `backend_root` but binds it to `SettlementStateRoot` via `root_bind`. | `crates/z00z_storage/src/settlement/proof.rs:457-507` `crates/z00z_storage/src/settlement/proof.rs:1069-1076` |
| `ProofScanOut` | Sanitized view after verification. | Strips raw branch proofs so callers do not treat membership evidence as broader theorem evidence. | `crates/z00z_storage/src/settlement/proof.rs:937-1066` |
| `BatchProofBlobV1` | Shared batch proof envelope. | Must verify fail-closed through `check_batch_contract_v1(...)`. | `crates/z00z_storage/src/settlement/README.md:249-254` `crates/z00z_storage/src/settlement/proof_batch_verify.rs:65-73` |

## ⚙️ Verification Steps

| Verifier | What it checks | What it intentionally does not do | Source |
|---|---|---|---|
| `chk_item_settlement(...)` | Root equality, path equality, definition-leaf match, serial-leaf match, terminal-leaf match. | No backend proof decoding. | `crates/z00z_storage/src/settlement/proof.rs:1132-1159` |
| `chk_blob_settlement(...)` | Decodes the blob, reuses `chk_item_settlement(...)`, checks terminal leaf hash, checks `root_bind`, verifies definition/serial/terminal proofs against `backend_root`. | Does not reinterpret `backend_root` as semantic state authority. | `crates/z00z_storage/src/settlement/proof.rs:1161-1218` |
| `check_batch_contract_v1(...)` | Checks header, canonical path ordering, exact usage, openings, transcript binding, and atomic roots. | Does not permit partial acceptance on contract drift. | `crates/z00z_storage/src/settlement/proof_batch_verify.rs:65-73` `crates/z00z_storage/src/settlement/proof_batch_verify.rs:189-220` |

## 📌 Boundary Rules

Downstream code is supposed to consume `SettlementStore`, `ProofItem`, `ProofBlob`, `chk_item_settlement`, and `chk_blob_settlement` rather than reconstruct witness semantics by hand. The README calls out three forbidden compressions explicitly: do not treat `backend_root` as the public state root, do not collapse the three-level path into one flat identity, and do not expose raw JMT proof or node types from higher layers. `crates/z00z_storage/src/settlement/README.md:526-538`

## Related Pages

| Page | Relationship |
|---|---|
| [Settlement Runtime And Rollup](./settlement-runtime-and-rollup.md) | Higher-level overview of how storage composes with runtime and rollup. |
| [Object Package Rejects](./object-package-rejects.md) | Admission surface built on top of the same typed object and proof model. |
| [Rollup Theorem Verifier](./rollup-theorem-verifier.md) | Explains how rollup reuses only public checkpoint and spend artifacts rather than raw settlement proofs. |
