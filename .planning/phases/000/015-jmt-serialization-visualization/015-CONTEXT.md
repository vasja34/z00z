<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD047 -->
# Phase 015: JMT Serialization And Visualization - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning
**Source:** Manual bootstrap from existing phase 015 spec work

<domain>
## Phase Boundary

This phase covers deterministic serialization and human-readable visualization
for JMT-backed storage state inside `crates/z00z_storage/src/serialization`.
The phase stays storage-owned, preserves current `assets`, `checkpoint`, and
`snapshot` invariants, and avoids unrelated refactors.

</domain>

<decisions>
## Implementation Decisions

### Storage ownership
- Keep implementation inside `z00z_storage`; do not move ownership into
  `z00z_wallets` or `z00z_core`.

### Artifact format
- Prefer deterministic machine-readable serialization using the existing
  bincode-style storage codec pattern.

### Visualization scope
- Provide a human-readable export for debugging and inspection of node layout,
  links, hashes, namespaces, and canonical paths.

### Safety boundary
- Do not expose raw `jmt` node, batch, or proof types as the downstream public
  contract.

### the agent's Discretion
- Exact internal module split, helper names, and test fixture structure may be
  chosen during planning if they stay within current crate conventions.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase spec
- `specs/015-z00z-storage-addons/storage-serialization-plan.md` — concrete phase plan already prepared outside `.planning`
- `specs/015-z00z-storage-addons/doublecheck-report.md` — verified constraints around JMT serialization and visualization support

### Storage boundaries
- `crates/z00z_storage/src/assets/README.MD` — storage-owned JMT boundary rules
- `crates/z00z_storage/src/snapshot/codec.rs` — deterministic codec pattern to follow
- `crates/z00z_storage/src/snapshot/store.rs` — storage facade pattern to follow
- `crates/z00z_storage/src/assets/store.rs` — active JMT-backed asset store surface
- `crates/z00z_storage/src/assets/store_internal/tree_store.rs` — current internal tree access path

</canonical_refs>

<specifics>
## Specific Ideas

- Plan for both machine-readable serialization and human-readable visualization.
- Support reconstruction or restore of tree-relevant serialized state where useful for inspection.
- Keep public API small and explicit.
- Preserve current checkpoint and snapshot paths unless they explicitly opt into the new module.

</specifics>

<deferred>
## Deferred Ideas

- New persistence backends for serialization artifacts.
- Broader search or index features unrelated to serialization or visualization.
- Cross-crate ownership changes.

</deferred>

---

*Phase: 015-jmt-serialization-visualization*
*Context gathered: 2026-03-23 via manual bootstrap*
