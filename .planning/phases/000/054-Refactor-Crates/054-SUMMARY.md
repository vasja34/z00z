---
phase: 054-Refactor-Crates
status: complete
completed_at: 2026-06-09
summary_artifact_for: .planning/phases/054-Refactor-Crates/
---

<!-- markdownlint-disable MD060 -->

# Phase 054 Summary

## Result

Phase 054 is complete. The repository now uses one canonical module-path story
across the phase-owned `z00z_storage`, `z00z_runtime/*`, and
`z00z_rollup_node` surfaces: backend and planner ownership are explicit,
storage bridge modules are gone, delayed rename fallout is closed, final docs
match the landed topology, root implementation modules are private behind one
crate-root facade, and no live alias or shim path remains in the phase-owned
Rust trees.

Final release-only revalidation also closed the last hidden suite regressions:
simulator wallet unlocks now resolve through the actor-owned canonical password
path, shared-state claim-registry tests are serialized, the wallet
deterministic-object assertion matches the real non-fast release contract, and
the broad workspace release suite now passes green.

On 2026-06-09 the bounded `054-08` continuation closed the last runtime
planner trust gap left after the original closeout: ingress now recomputes the
payload-bound digest for tx and claim packages, the public planner lane no
longer has a caller-controlled digest bypass, and the follow-up
attack-surface ledger is closed without reopening unrelated waves.

## Completed Plans

- `054-01` through `054-02`: closed the backend seam guardrails, extracted the
  backend planes, and rewired `SettlementStore` onto the storage-owned seam.
- `054-03` through `054-04`: moved runtime planner authority into
  `batch_planner.rs`, then isolated placement and shard-execution metadata to
  runtime-only operational boundaries.
- `054-05` through `054-07`: removed storage bridge wiring, landed canonical
  module roots and hidden test-support owners, completed the delayed rename
  wave, tightened the last duplicate public paths at the runtime or node or
  storage roots, synchronized final docs and migration tables, and recorded the
  honest closeout evidence.
- `054-08`: rebound runtime planner digests at ingress, removed the last
  caller-controlled routing path, added public-lane source guards, and closed
  `AS-20260609-001` in the existing phase packet.

## Removed Legacy Surface

- Removed bridge or shim modules from the live storage tree, including the old
  settlement-side `redb_backend` and `store_*` helper shells.
- Removed the last duplicate public-path seams in the phase-owned roots by
  keeping runtime or node implementation modules private and by retiring the
  public `z00z_storage::error::*` path in favor of the crate-root error facade.
- Removed live `#[path]`-driven hot spots from the phase-owned storage,
  runtime, and rollup-node crates.
- Removed plan-owned `empty_file` placeholders from the runtime and node
  surfaces, replacing directory-retention cases with canonical `.gitkeep`
  files only where still needed.
- Removed the old support `.inc` file layout from the active storage suite in
  favor of `src/test_support/*` plus `tests/snapshot_suite/*`.
- Removed the last caller-controlled digest-authority seam in
  `z00z_aggregators` by forcing route lookup, intake ids, and `plan_digest`
  onto one payload-verified runtime path.

## Final Review Evidence

The `054-07` closeout review loop reopened twice on real issues: first on the
wallet `ReceiverSecret::set_fail_usable(...)` test hook hidden outside
`cfg(test)`, then on a stricter post-closeout canonical-path audit that found
duplicate public root paths in runtime or node facades plus the public
`z00z_storage::error::*` seam. Both were fixed, and the final two review
passes were consecutive clean passes with no significant alias, shim,
duplicate-path, or docs-truth issues remaining.

The `054-08` continuation review loop reopened once more on a real gap: the
runtime code had already removed the direct public `WorkItem` construction
bypass, but no external-facing source guard yet proved that invariant as a
release-tested public API contract. A new `z00z_aggregators`
`test_live_guardrails.rs` source-guard suite closed that gap, after which the
remaining follow-up work was docs-truth synchronization only and the final two
review passes were consecutive clean passes.

## Final Validation Snapshot

Phase-closeout evidence is recorded in `054-01-SUMMARY.md` through
`054-08-SUMMARY.md`. The original closeout slices reran the mandatory
bootstrap gate, the stale broad feature command, alias or source-shape audits,
`cargo fmt --all --check`, `cargo clippy --all-targets --all-features`,
`cargo doc --no-deps`, `git diff --check`, and direct wallet regression tests
for the repaired compile gate and rename-guard truth surface.

The later release-only rerun then fixed the remaining hidden regressions and
revalidated the broad workspace gate with current code:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` —
  passed.
- `cargo test -p z00z_wallets --release -q` — passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools -q` —
  passed.
- `cargo test --all --release -q` — passed.
- `cargo test --release --features test-fast --features wallet_debug_dump` —
  still failed immediately because the selected packages do not expose those
  feature names in the live manifest.

The post-closeout `054-08` continuation then reran the live follow-up gate in
the order required by the continuation plan:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` —
  passed.
- `cargo test -p z00z_aggregators --release -q` — passed.
- `cargo test --release` — passed.
- `cargo doc --no-deps` — passed with pre-existing rustdoc warnings outside
  the `054-08` scope.
- `git diff --check` — passed.
