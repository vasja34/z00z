# System Patterns

## 🏛️ Architecture Shape

The repository is a Rust workspace with a layered design:

- `z00z_utils` provides cross-cutting abstractions and utility boundaries
- `z00z_crypto` wraps approved cryptographic primitives and vendored Tari code
- `z00z_core` contains protocol-facing domain logic
- Storage, wallet, simulator, networking, runtime, telemetry, and rollup crates
  build on the foundation layer

## 🔑 Key Patterns

### 🔒 One Source Of Truth

Business-logic crates should use `z00z_utils` abstractions instead of calling
raw low-level libraries directly for file I/O, config, codecs, time, logging,
metrics, or RNG.

### 🧩 Trait-Based Dependency Injection

External dependencies should be injected through traits or equivalent explicit
boundaries so that deterministic implementations can be used in tests.

### 🧪 Verification-Driven Changes

Changes are expected to flow through formatting, clippy, targeted tests, and
when needed the stronger repository verification scripts.

### 🧱 Vendor Isolation

Vendored Tari code exists under `crates/z00z_crypto/tari/` and is treated as a
protected subtree. Integration changes must happen through supported wrapper
surfaces, not by editing vendor sources.

## 🗂️ Workspace Pattern Notes

- `z00z_core`, `z00z_storage`, `z00z_wallets`, `z00z_simulator`, and RPC
  surfaces appear to be active implementation areas
- Runtime subcrates, telemetry, and extensions currently look thinner or more
  scaffold-like in the inspected manifests
- Configuration-driven behavior is important, especially around genesis and
  assets

## 📐 Planning Workflow Pattern

- The repository actively uses `.planning/` as a phase-driven execution and
  audit workspace for larger architecture and security work
- Phase artifacts capture context, inventories, plans, summaries, validation,
  and closeout evidence, so real project state may advance there before it is
  reflected in top-level docs
- Memory-bank updates should treat `.planning/` artifacts as high-signal
  evidence when summarizing active work, risks, and next steps

## 🧠 Dashboard-First Memory Pattern

- `activeContext.md` is the primary entry point for cross-session continuity and
  should be updated before other memory-bank files when current truth changes
- The dashboard must separate verified baseline, active delta, open gaps, and
  next actions so future sessions do not confuse completed work with prepared
  or speculative work
- Supporting files such as `progress.md`, `systemPatterns.md`, `techContext.md`,
  and `tasks/` provide depth, but they should reconcile to the dashboard rather
  than compete with it

## 🧭 Hardening Patterns Established In Phases 025-031

- Compatibility lanes are increasingly treated as explicit migration-only
  surfaces: legacy or experimental APIs stay behind named gates instead of
  sharing the default production facade
- Fail-closed behavior is now a repeated cross-crate pattern for nonce, time,
  root-mode, config, and proof-validation boundaries
- Canonical typed ownership is preferred over stringly or loosely typed state,
  especially around claim proofs, nullifiers, wallet metadata, and checkpoint
  identity
- Root facades and crate entrypoints are being narrowed toward curated exports
  and explicit boundary notes rather than wildcard re-exports or implicit
  ownership
- Phase closeout is treated as incomplete until planning truth, validation
  evidence, and summary artifacts agree on the same end state

## ⚠️ Design Risks To Watch

- Architectural drift away from `z00z_utils`
- Leakage of vendor-specific or dependency-specific types into public APIs
- Privacy-sensitive behavior changing without corresponding documentation and
  verification
- Large workspace changes that skip crate-by-crate impact review
