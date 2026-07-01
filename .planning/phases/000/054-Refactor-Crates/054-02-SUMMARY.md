---
phase: 054-Refactor-Crates
plan: 054-02
status: complete
completed_at: 2026-06-08
next_plan: 054-03
requirements-completed: [PH54-02]
---

# 054-02 Summary

## Outcome

Phase `054-02` is complete.

The RedB adapter, backend-neutral helpers, and memory helper surface now live
under `crates/z00z_storage/src/backend/`, while `SettlementStore` routes
through a seam-owned storage plane instead of holding the old concrete backend
shape directly. The semantic settlement facade, proof behavior, and replay
ownership remain storage-owned, and no parallel semantic layer was introduced.

## Landed Changes

- Extracted RedB-specific durable code under
  `crates/z00z_storage/src/backend/redb/`:
  - `mod.rs`
  - `helpers.rs`
  - `hjmt.rs`
  - `state.rs`
  - `validate.rs`
- Extracted backend-neutral helpers under
  `crates/z00z_storage/src/backend/common/`:
  - `codec.rs`
  - `query.rs`
  - `roots.rs`
  - `rows.rs`
  - `types.rs`
- Moved the memory helper into
  `crates/z00z_storage/src/backend/memory.rs`.
- Reworked `crates/z00z_storage/src/backend/redb/mod.rs` so the concrete
  `RedbBackend` stays private and `StoragePlane` becomes the thin backend-plane
  wrapper used by the settlement facade.
- Reduced
  `crates/z00z_storage/src/settlement/redb_backend.rs` to a bridge module that
  re-exports `StoragePlane`, `HjmtPersistWork`, and the RedB backend state
  owner from the backend namespace.
- Rewired `crates/z00z_storage/src/settlement/store.rs` so
  `SettlementStore.backend` uses `StoragePlane` and the helper modules resolve
  through the new backend/common and backend/memory locations.
- Widened only the minimum visibility needed in
  `hjmt_config.rs`, `hjmt_journal.rs`, and `settlement/mod.rs` so the extracted
  backend modules can access store-owned semantic internals without turning
  them into new public API.
- Updated source-shape guardrails in
  `test_downstream_guardrails.rs` and `test_settlement_root.rs` to point at the
  live extracted file topology instead of deleted legacy helper paths.

## Validation

Executed and passed on the current tree:

- `cargo fmt --all`
- `cargo test -p z00z_storage --release --no-run`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_storage --release --features test-params-fast`
- `cargo bench -p z00z_storage --bench assets_proofs --no-run`

The plan-mandated broad workspace command is stale against the live manifest on
this repository state:

- `cargo test --release --features test-fast --features wallet_debug_dump`

Observed failure:

- `error: none of the selected packages contains these features: test-fast, wallet_debug_dump`

The live equivalent storage release gate on this tree is
`cargo test -p z00z_storage --release --features test-params-fast`, and it
passed after the seam extraction and `SettlementStore` rewire.

## Review Loop

- Review pass 1 checked the extracted storage diff for stale flat helper-path
  references and seam leakage; no significant issues remained after the
  rewiring fixes.
- Review pass 2 rechecked downstream usage, source-shape anchors, and diff
  hygiene; no significant issues remained.
- Review pass 3 repeated absence checks for deleted helper paths and for
  `StoragePlane` leakage into downstream runtime or wallet crates; no
  significant issues remained, giving the required consecutive clean passes.

## Closeout

The post-seam stabilization evidence required by hard step 7 is now recorded.
`054-03` can start the runtime planner split from a stabilized backend seam
instead of from speculative storage topology.
