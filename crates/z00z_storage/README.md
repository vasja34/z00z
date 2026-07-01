
# Z00Z Storage

This crate exposes the live settlement-first contract for storage-backed proofs,
roots, and path-indexed state transitions.

- `settlement` is the semantic authority: `SettlementPath`, `SettlementStateRoot`,
  `SettlementStore`, `chk_blob_settlement`, and `chk_item_settlement` are the
  live public surface.
- Runtime bind/publish stays runtime-owned, but storage remains the only
  authority for settlement roots, proof contracts, and recovery exports. Rollup
  consumes only the final public theorem bundle built from those authorities;
  there is no second authority path.
- `checkpoint`, `snapshot`, and `serialization` remain distinct crate surfaces;
  this crate does not collapse them into one generic backup layer.
- `backend` owns the durable and in-memory seams, while settlement-facing
  callers should use asset-path adapters that project wallet/domain data into
  `SettlementPath` and `SettlementStateRoot`.
- Deterministic serialization and inspection helpers are available as a runtime
  or canonical test surface.
- Hidden `fixture_support` helpers plus flat `tests/test_snapshot_*.rs` files are
  the canonical non-production harness layout after the Phase 054 source-shape cleanup.
