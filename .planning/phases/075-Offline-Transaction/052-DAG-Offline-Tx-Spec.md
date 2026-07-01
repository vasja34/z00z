# 052 DAG Offline Tx Spec

Status: future DAG synthesis

## 🎯 Purpose

This document describes only the future DAG layer that could sit on top of the live Phase 050 offline transaction flow. It does not redefine the current package model, receiver routing, verifier, or import gate.

For current behavior, use:

- [050-Offline-Tx-Spec](../050-offline-tx/050-Offline-Tx-Spec.md)
- [050-TODO](../050-offline-tx/050-TODO.md)

## 🔒 Scope Boundary

This document may propose future orchestration around the live Phase 050 surface, but it must not:

- introduce a second transaction family,
- introduce a second verifier,
- introduce a second import pipeline,
- redefine the current receiver-routing contract,
- restate Phase 050 backlog items as if they were DAG requirements.

## 📦 Future DAG Ideas Worth Preserving

### 1. `TxPackage` As The DAG Node

The reusable idea is to treat each `TxPackage` as a graph node. A child package can depend on a parent package when it spends an output created by that parent package.

The future DAG layer should reuse `TxPackage` rather than inventing a separate transaction payload family.

### 2. Minimal Ancestor Closure

If an input is not anchored in the current base state, the publisher must provide the ancestor package that created it. That rule may recurse until every unresolved input is either state-anchored or ancestor-provided.

This remains the cleanest future publication rule for any offline bundle design.

### 3. Topological Apply In A Working Window

A future aggregator can resolve dependencies, order packages topologically, and then apply the ordered set in a bounded working window.

That idea fits the current checkpoint and bundle-lane substrate, but it should stay a wrapper around the existing package flow instead of becoming a separate runtime.

### 4. Explicit Conflict And Cycle Rejection

A DAG layer must reject cycles, duplicate ancestor claims, conflicting spends, and inconsistent package membership deterministically.

Conflict handling should be explicit and local. It should not leak into the live package digest or verifier contract.

### 5. Transport Metadata Stays Optional

Any future bundle metadata should be transport-only unless the live digest or verifier contract explicitly adopts it.

That keeps presentation and publication concerns separate from the canonical package contract.

## 🚫 Non-Goals

The future DAG layer is not allowed to become a hidden second transaction system. In particular, do not add:

- `BundlePackage_v1`
- `TxPackage_v1`
- `receiver_view`
- `offline_chain.rs`
- `dag_storage.rs`
- `offline_service.rs`
- `dag_validator.rs`
- a standalone offline import pipeline
- a standalone publish-proof backend

## 🧭 Recommended Implementation Direction

1. Reuse `TxPackage` as the only portable node format.
2. Reuse the current wallet verification flow before any ancestor logic.
3. Resolve ancestors as a wrapper around existing packages, not as a new transaction family.
4. Feed ordered operations into the current checkpoint builder and replay substrate.
5. Keep verification, reporting, and import gating separate.
6. Keep transport helpers outside digest semantics unless the live package contract changes first.

## 🧱 Future Backlog Shape

A later implementation would most likely break down into these themes:

- node identity and digest deduplication,
- ancestor resolution,
- conflict policy,
- working-window apply,
- checkpoint handoff,
- simulator coverage,
- RPC and report exposure.

## ✅ Success Criteria For A Future DAG Wave

A future DAG wave should be considered valid only if it:

- reuses the live Phase 050 package contract,
- keeps the current receiver-routing contract intact,
- avoids a second verifier or import path,
- preserves deterministic package ordering,
- keeps the current checkpoint handoff contract working.

If any of those conditions fail, the design has drifted away from a wrapper model and should be revised before implementation starts.
