---
description: 'Workspace memory for canonical Rust module topology in the Z00Z repository.'
applyTo: '**/*.rs'
---

# Rust Module Topology Memory

## Canonical Module Wiring

- Prefer canonical Rust module wiring through `mod` declarations and direct root-level files.
- When a module family is flattened, remove duplicate shim directories and `include!` bridge files instead of keeping parallel layouts.
- Update tests and source anchors that reference removed shim files so the guardrails follow the live topology.

## Path Attribute Exception

- Use `#[path = "..."]` only when Rust module visibility or privacy rules make canonical flat wiring impossible without introducing a worse structure.
- Typical valid case in this repository: a helper file must stay a child module to access parent-private items, but the physical shim subdirectory must still be removed.
