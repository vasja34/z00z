---
phase: 054-Refactor-Crates
plan: 054-06
status: complete
completed_at: 2026-06-08
next_plan: 054-07
requirements-completed: [PH54-06]
---

# 054-06 Summary

## Outcome

Phase `054-06` is complete.

The delayed rename wave is now landed across the runtime, storage, and
rollup-node slices covered by this plan: canonical file and module names are
live, placeholder tails are cleaned up, and the stable public re-export surface
now also resolves through one canonical public path at the phase-owned crate
roots.

## Landed Changes

- Renamed the runtime aggregator modules to their canonical file identities:
  `agg_iface` -> `service`, `agg_ingress` -> `ingress`, `agg_ordering` ->
  `ordering`, `agg_recovery` -> `recovery`, `agg_scheduler` -> `scheduler`,
  and `agg_types` -> `types`.
- Renamed the runtime validator modules to the live canonical set:
  `artifact_decode` -> `artifact`, `checkpoint_flow` -> `checkpoint`,
  `claim_nulls` -> `nullifier`, `claim_pkg_verify` -> `claim_verify`,
  `reconcile_rules` -> `reconcile`, `spend_rules` -> `spend`,
  `tx_pkg_verify` -> `tx_verify`, `val_engine` -> `engine`, and `verdicts` ->
  `verdict`.
- Renamed the watcher modules to the canonical live names:
  `censorship_watch` -> `censorship`, `provider_compare` -> `provider`,
  `publication_watch` -> `publication`, `status_view` -> `status`, and
  `watcher_engine` -> `engine`.
- Renamed the settlement support files to the canonical live names:
  `types_identity` -> `identity`, `types_query` -> `query`,
  `types_record` -> `record`, `README.MD` -> `README.md`, and
  `root-types.md` -> `root_types.md`.
- Renamed the rollup-node runtime files from `da_adapter` -> `da` and
  `lifecycle` -> `runtime`, while keeping `NodeRuntime`, `DaAdapter`, and
  `DaError` stable at the crate facade.
- Replaced the plan-owned runtime and node `empty_file` placeholders with
  `.gitkeep` where directory retention is still required, and removed the
  root-level `crates/z00z_rollup_node/src/empty_file` placeholder entirely.
- Fixed the last rename fallout in `z00z_storage` by rebinding
  `backend/common/{query,rows}.rs` to the canonical `SettlementStore.nullifier`
  field instead of the dead `claim_nulls` name.
- Tightened the root public API story after the rename wave: the implementation
  modules in `z00z_runtime/{aggregators,validators,watchers}/src/lib.rs` and
  `crates/z00z_rollup_node/src/lib.rs` are now private `mod` entries, so the
  crate-root `pub use` facade is the only public path for the renamed runtime
  or node exports.
- Removed the last storage-side duplicate public error path by making
  `z00z_storage::error` private and keeping `SerializationError`,
  `SerializationResult`, `CheckpointError`, and `CheckpointResult` available
  only from the crate root.

## Validation

Executed and passed on the current tree:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo fmt --all --check`
- `cargo test --release -p z00z_aggregators -p z00z_validators -p z00z_watchers`
- `cargo test --release -p z00z_storage -p z00z_rollup_node --features test-params-fast`
- `cargo test -p z00z_storage --release --features test-params-fast`
- `cargo test -p z00z_wallets --release --features test-params-fast --test test_rename_guards`
- `git diff --check`

The plan-mandated broad workspace command is still stale against the live
manifest on this repository state:

- `cargo test --release --features test-fast --features wallet_debug_dump`

Observed failure:

- `error: none of the selected packages contains these features: test-fast, wallet_debug_dump`

The live-equivalent release evidence for this slice is therefore the green
targeted release gates above plus the green bootstrap gate.

## Review Loop

- Review pass 1 ran the rename-wave old-name audit across the touched crates;
  no live `agg_*`, `val_*`, watcher, settlement-type, rollup-node, or
  `empty_file` module references remained after the rename pass.
- Review pass 2 found one real in-scope regression: `backend/common` still
  referenced the dead `claim_nulls` field name after the canonical
  `nullifier` rename. The field drift was fixed before any closeout claim.
- Review pass 3 reran the targeted release gates after the fix; runtime,
  storage, and rollup-node validation all passed with no significant rename
  fallout.
- Review pass 4 re-audited the reachable module surface. Stable public
  re-exports remain intentionally preserved, but no live bridge module, legacy
  alias name, or placeholder shim remains on the touched runtime, storage, or
  node paths. This gave the required consecutive clean closure.
- Review pass 5 reopened once on a stricter public-path audit: the runtime or
  node roots still exposed implementation modules publicly alongside stable
  crate-root re-exports, and `z00z_storage::error::*` remained reachable next
  to the crate-root error facade. Those duplicate public paths were removed.
- Review pass 6 caught the small fallout from that visibility tightening:
  storage integration tests still imported `z00z_storage::error::SerializationError`,
  and `z00z_wallets/tests/test_rename_guards.rs` still encoded stale live or
  removed path expectations for the canonical nested test layout. Both were
  fixed in scope.
- Review pass 7 reran bootstrap, the targeted storage and wallet guards, and
  the source-shape audits with no significant issues remaining.
- Review pass 8 repeated the same audits with no significant issues again,
  giving the required consecutive clean closure after the stricter canonical
  public-path pass.

## Closeout

The delayed rename wave is now summary-backed complete. Phase `054-07` is the
active next lane for docs, migration tables, and final closeout gates.
