---
phase: 054-Refactor-Crates
plan: 054-05
status: complete
completed_at: 2026-06-08
next_plan: 054-06
requirements-completed: [PH54-05]
---

# 054-05 Summary

## Outcome

Phase `054-05` is complete.

The storage canonical-module cleanup is now real in the live crate graph.
`z00z_storage` no longer relies on bridge modules for the active settlement
hot spots: `backend/common` owns the shared codec/query/roots/rows helpers,
`backend/redb` owns the durable backend path, the duplicate serialization
temp-tree helper is gone, and the old `tx_plan` bridge shell is no longer part
of the live module layout.

## Landed Changes

- Declared `crates/z00z_storage/src/backend/common/{query,rows}.rs` as the
  canonical live helper modules under `backend/common/mod.rs`.
- Moved the namespace hash-domain ownership into
  `crates/z00z_storage/src/backend/common/codec.rs`, so `ns_key` and the
  namespace domains no longer depend on `settlement/store.rs` re-export glue.
- Rebound `crates/z00z_storage/src/backend/common/query.rs` and
  `crates/z00z_storage/src/backend/common/rows.rs` onto direct canonical
  imports instead of being compiled through `#[path = ...]` aliases from
  `settlement/store.rs`.
- Removed the live `settlement/store.rs` bridge wiring for `store_query`,
  `store_rows`, `ns_key`, and the `redb_backend` alias surface.
- Deleted the obsolete bridge or shim files that no longer own live module
  identity:
  - `crates/z00z_storage/src/settlement/redb_backend.rs`
  - `crates/z00z_storage/src/settlement/store_codec.rs`
  - `crates/z00z_storage/src/settlement/store_mem.rs`
  - `crates/z00z_storage/src/settlement/store_query.rs`
  - `crates/z00z_storage/src/settlement/store_roots.rs`
  - `crates/z00z_storage/src/settlement/store_rows.rs`
  - `crates/z00z_storage/src/settlement/store_types.rs`
- Retargeted settlement internals to the canonical paths:
  - direct `crate::backend::redb::*` imports instead of `store::redb_backend`
  - direct `crate::backend::common::codec::ns_key`
  - crate-private visibility widened only where `backend/common` now
    legitimately owns the helper implementations.
- Removed the former `settlement/redb_backend*.rs` doc-path from
  `crates/z00z_storage/src/settlement/README.md` so repository docs now point
  at the canonical `backend/common/*` and `backend/redb/*` locations.

## Validation

Executed and passed on the current tree:

- `cargo fmt --all`
- `cargo test -p z00z_storage --release --features test-params-fast`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `git diff --check`

The plan-mandated broad workspace command is still stale against the live
manifest on this repository state:

- `cargo test --release --features test-fast --features wallet_debug_dump`

Observed failure:

- `error: none of the selected packages contains these features: test-fast, wallet_debug_dump`

The live-equivalent release evidence for this slice is therefore the green
targeted `z00z_storage` release gate plus the green bootstrap gate.

## Review Loop

- Review pass 1 found the remaining `store_query` and `store_rows` bridge
  ownership and converted them into live `backend/common` modules.
- Review pass 2 caught the visibility fallout from the ownership move
  (`require_hjmt_mode`, proof/journal helpers, `HjmtPlan`, and `ForestCache`)
  and tightened it to crate-private canonical access instead of reintroducing
  shims.
- Review pass 3 reran the focused release gate and alias audit; no significant
  live bridge or shim seams remained in `crates/z00z_storage/src`.
- Review pass 4 reran bootstrap, the stale broad workspace command, and
  `git diff --check`; no significant issues remained beyond the already
  documented stale feature-name blocker, giving the required consecutive clean
  closure.

## Closeout

The storage canonical-module cleanup is now summary-backed complete. Phase
`054-06` is the active next lane for the delayed rename wave.
