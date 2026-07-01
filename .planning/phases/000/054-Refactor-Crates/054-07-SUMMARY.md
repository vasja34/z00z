---
phase: 054-Refactor-Crates
plan: 054-07
status: complete
completed_at: 2026-06-08
next_plan: none
requirements-completed: [PH54-07]
---

# 054-07 Summary

## Outcome

Plan `054-07` is complete.

The final Phase 054 closeout now matches the landed repository shape: touched
runtime, storage, and rollup-node module trees use one canonical source path,
the final docs and migration tables describe only the shipped topology, and the
phase packet records the remaining manifest-only blocker honestly instead of
hiding it behind target-state wording.

## Code And Planning Changes

- Synchronized the landed-topology docs to the real module graph:
  - `crates/z00z_runtime/aggregators/README.md`
  - `crates/z00z_runtime/validators/README.md`
  - `crates/z00z_runtime/watchers/README.md`
  - `crates/z00z_rollup_node/README.md`
  - `crates/z00z_storage/README.md`
  - `crates/z00z_storage/src/settlement/README.md`
  - `crates/z00z_storage/src/settlement/root_types.md`
- Closed the storage source-shape story in planning truth:
  - `.planning/phases/054-Refactor-Crates/054-TODO.md`
  - `.planning/phases/054-Refactor-Crates/054-CONTEXT.md`
  - `.planning/ROADMAP.md`
  - `.planning/STATE.md`
- Recorded the landed test/support topology instead of stale target examples:
  - hidden crate support under `crates/z00z_storage/src/test_support/*`
  - canonical snapshot integration suite under
    `crates/z00z_storage/tests/snapshot_suite/*`
- Finished the strict no-alias or no-shim public-path closeout for the
  phase-owned roots:
  - runtime and rollup-node implementation modules are private behind their
    crate-root `pub use` facades
  - `z00z_storage::error` is private, leaving the crate-root error exports as
    the only public path
  - storage integration tests now import `SerializationError` from
    `z00z_storage::SerializationError`
  - wallet rename guards now match the live canonical nested test layout and
    its explicit exceptions
- Fixed the last full-workspace compile blocker outside the phase-owned
  refactor packet by widening the wallet test-only `ReceiverSecret`
  fail-injection hook from `feature = "test-params-fast"` to
  `cfg(any(test, feature = "test-params-fast"))`:
  - `crates/z00z_wallets/src/key/receiver/stealth_keys.rs`
  - `crates/z00z_wallets/src/key/receiver/stealth_keys_secret.rs`

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` —
  passed again after the strict canonical-path cleanup.
- `cargo test --release --features test-fast --features wallet_debug_dump` —
  still fails on the live manifest because the workspace does not define
  `test-fast` or `wallet_debug_dump` for the selected packages.
- `cargo fmt --all --check` — passed.
- `cargo clippy --all-targets --all-features` — passed.
- `cargo doc --no-deps` — passed with pre-existing rustdoc warnings in
  untouched doc surfaces under `z00z_crypto`, `z00z_core`, `z00z_wallets`, and
  `z00z_simulator`.
- `cargo test -p z00z_wallets services::wallet_service::wallet_service_tests::tests::test_receiver_keys_retries_unusable -- --exact`
  — passed.
- `cargo test -p z00z_wallets --release --features test-params-fast --test test_rename_guards`
  — passed.
- `cargo test -p z00z_wallets --release -q` — passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools -q`
  — passed.
- `cargo test --all --release -q` — passed.
- `git diff --check` — passed.
- `rg -n "#\\[path|include!\\(" crates/z00z_storage crates/z00z_runtime crates/z00z_rollup_node -g '!**/*.md'`
  — clean; no live `#[path]` or module-body `include!` remains in the
  phase-owned crates.

## Review Loop

- Review pass 1 used the `GSD-Review-Tasks-Execution` checklist against the
  live source shape and rename residue. No live module-path alias or shim
  remained; only persisted RedB table labels and negative guardrail assertions
  referenced old strings.
- Review pass 2 reran the full closeout validation set and found one real
  blocker: `cargo test --all` still hit the wallet compile gate because
  `ReceiverSecret::set_fail_usable(...)` was hidden outside `cfg(test)`. That
  gating drift was fixed in scope.
- Review pass 3 reran bootstrap, clippy, rustdoc, the targeted wallet test,
  and the alias audits after the fix. No significant issues remained.
- Review pass 4 repeated the source-shape, docs-truth, and hygiene audits.
  No significant issues remained again, giving the required consecutive clean
  closure.
- Review pass 5 reopened once on the stricter user-requested canonical-path
  audit: runtime or node roots still exposed implementation modules publicly
  next to the stable crate-root re-exports, and `z00z_storage::error::*`
  remained a second public error path next to the crate-root facade. Those
  duplicate public paths were removed in scope.
- Review pass 6 found only fallout from that visibility tightening: two
  storage integration tests still imported the old
  `z00z_storage::error::SerializationError` path, and the wallet rename-guard
  suite still encoded stale live or removed path assumptions for the canonical
  nested test layout. Those issues were fixed immediately.
- Review pass 7 reran bootstrap, the stale broad feature command, the targeted
  wallet rename-guard suite, and the source-shape or alias audits. No
  significant issues remained.
- Review pass 8 repeated the same audits. No significant issues remained again,
  giving the required consecutive clean closure after the stricter audit
  reopened the packet.
- Review pass 9 rechecked the final release-only rerun evidence against the
  live workspace. Code surfaces were clean, but one material planning-truth
  issue remained: several closeout documents still claimed `cargo test --all`
  was blocked even though `cargo test --all --release -q` had already passed.
  That drift was fixed in scope.
- Review pass 10 reran the same planning-truth and command-evidence audit after
  the doc fixes. No significant issues remained.
- Review pass 11 repeated the same audit again. No significant issues remained
  again, giving the required consecutive clean closure for the final
  release-only refresh.

## Closeout

- `054-SUMMARY.md`, `.planning/ROADMAP.md`, and `.planning/STATE.md` now carry
  the final phase-complete truth.
- Phase 054 is complete; no active `054` execution lane remains.
