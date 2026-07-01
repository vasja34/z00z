# Z00Z

<!-- markdownlint-disable MD060 -->

## What This Is

Z00Z is a privacy-focused blockchain workspace centered on confidential assets,
wallet flows, storage boundaries, and simulator-driven validation. The current
planning scope uses GSD artifacts to structure incremental work across the Rust
crates without blurring ownership between protocol, storage, wallet, and
simulation layers.

## Core Value

Confidential asset and wallet flows must remain correct, explicit, and storage-safe.

## Requirements

### Validated

- ✓ Canonical asset storage, checkpoint, and snapshot boundaries already exist in the live codebase.
- ✓ Deterministic JMT serialization and visualization are implemented inside `z00z_storage`.
- ✓ RedB-backed live-state durability and deterministic storage-owned search are implemented without changing canonical root semantics.
- ✓ Crypto-surface remediation in `z00z_crypto` is complete through Phase 025.
- ✓ Core crypto-audit remediation in `z00z_core` is complete through Phase 026.

### Active

- [ ] Define the next milestone from the completed v0.15 baseline.

### Out of Scope

- Moving storage ownership into `z00z_wallets` or `z00z_core` — violates current crate boundaries.
- Replacing live checkpoint or snapshot semantics during phase 015 bootstrap — not required to start planning.

## Context

- The repository already has `.planning/codebase/` analysis artifacts, but it did not yet have canonical root planning files for roadmap-driven GSD flow.
- `crates/z00z_storage` already owns JMT-backed asset state and exposes snapshot and checkpoint modules.
- Phase 015 completed deterministic serialization and human-readable visualization for JMT-backed storage state.
- Phase 016 completed RedB-backed durable reload and deterministic storage-owned asset search.
- Phase 025 completed the crypto-surface audit remediation in `z00z_crypto`.
- Phase 026 completed the core crypto-audit remediation in `z00z_core`.

## Constraints

- **Tech stack**: Keep implementation in Rust workspace crates and follow existing crate ownership boundaries.
- **Architecture**: Use `z00z_utils` abstractions where persistence or codec helpers are needed.
- **Security**: Do not expose raw `jmt` internals as the public downstream contract.
- **Process**: Phase 015 must be routable through standard GSD planning after bootstrap.

## Key Decisions

| Decision | Rationale | Outcome |
| -------- | --------- | ------- |
| Bootstrap `.planning` around phase 015 now | Existing repo had codebase docs but no roadmap or state files | ✓ Good |
| Keep storage work inside `z00z_storage` | Matches live crate ownership and preserved canonical boundaries across phases 015 and 016 | ✓ Good |
| Persist RedB artifacts with canonical codecs and ids | Preserved root-safe durability without inventing a second artifact schema | ✓ Good |
| Derive search ordering from canonical path semantics | Kept convenience indexes subordinate to root ownership after durable reload | ✓ Good |

---
Last updated: 2026-03-28 after phase 026 completion
