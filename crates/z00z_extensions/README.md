# z00z_extensions

This crate reserves the extension boundary for repository-owned add-ons that do
not belong inside the core wallet, storage, runtime, or network crates.

## Boundary

- Extensions may compose canonical crate APIs, but they must not redefine or
  fork domain ownership.
- Cross-cutting infrastructure still belongs in `z00z_utils`.
- Extension code must stay explicit about which owning crate remains the source
  of truth for persistence, transport, crypto, and runtime semantics.
