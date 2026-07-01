---
phase: 031-refactor-architecture
plan: "08"
requirements_completed:
  - PH31-STORAGE
status: completed
task_commits: []
review_surface_metrics:
  file_store_reload_replay_guards: 2
  redb_rehydrate_negative_cases: 2
  validation_anchor_markers: 5
---

# Phase 031 Plan 08: Storage Checkpoint Proof And Rehydrate Boundary Summary

Canonical checkpoint sealing now fail-closes on persisted replay evidence, file-store link reloads revalidate the replay bundle, and RedB rehydrate coverage proves that attested checkpoint metadata cannot drift across reload.

## Accomplishments

- Hardened [crates/z00z_storage/src/checkpoint/store.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/store.rs) so the canonical seal path only succeeds when the referenced snapshot row and exec-input row already exist, and so file-store `load_link()` now revalidates snapshot presence, exec presence, replay tuple consistency, and root consistency on reload.
- Clarified the compatibility-only role of `cp_proof` inside [crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs) and [crates/z00z_storage/src/checkpoint/artifact_final.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/artifact_final.rs), while keeping statement-derived checkpoint ids and typed reject surfaces intact.
- Expanded [crates/z00z_storage/tests/test_checkpoint_store_api.rs](/home/vadim/Projects/z00z/crates/z00z_storage/tests/test_checkpoint_store_api.rs) and [crates/z00z_storage/tests/test_redb_rehydrate.rs](/home/vadim/Projects/z00z/crates/z00z_storage/tests/test_redb_rehydrate.rs) with missing-row and attested-proof-drift regressions so durable reload and file-store reload enforce the same storage-owned replay semantics.
- Added explicit ownership markers in [crates/z00z_storage/src/assets/store_internal/store_rows.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/store_rows.rs), [crates/z00z_storage/src/assets/store_internal/store_types.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/store_types.rs), [crates/z00z_storage/tests/assets/test_store_api.rs](/home/vadim/Projects/z00z/crates/z00z_storage/tests/assets/test_store_api.rs), [crates/z00z_storage/tests/test_checkpoint_root_binding.rs](/home/vadim/Projects/z00z/crates/z00z_storage/tests/test_checkpoint_root_binding.rs), and [crates/z00z_storage/tests/snapshot/test_replay_bound.rs](/home/vadim/Projects/z00z/crates/z00z_storage/tests/snapshot/test_replay_bound.rs) so the storage validation map is anchored to concrete code and tests.
- Preserved `ClaimNullifier` as a bounded non-finding by documenting the chain-bound wallet contract in [crates/z00z_wallets/src/core/claim/nullifier.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/claim/nullifier.rs) and [crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs) instead of widening the storage replay key shape.

## Task Commits

| Task | Commit | Purpose |
| --- | --- | --- |
| Task 1-3 | _not created_ | The repository worktree already contained extensive unrelated modifications, and the required `/z00z-git-versioning` stage-all flow would have captured unrelated files. |

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture`
- `cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture`
- `cargo test -p z00z_storage --release --test test_redb_rehydrate rehydrate_rejects_attested_proof_drift -- --exact --nocapture`
- `cargo test -p z00z_storage --release --features test-fast -- --nocapture`
- `cargo test -p z00z_storage --release --test test_assets_suite -- --nocapture`
- `cargo test -p z00z_storage --release --test test_checkpoint_root_binding -- --nocapture`
- `cargo test -p z00z_storage --release --test test_snapshot_suite snapshot::replay_bound::test_replay_ok_no_exec -- --exact --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump`

## Deviations From Plan

### Auto-fixed Issues

1. [Rule 3 - Blocking issue] File-store `load_link()` only validated the artifact/link tuple and did not re-check persisted snapshot or exec replay evidence after reload.
   Fix: added snapshot load, exec load, replay-tuple validation, and root validation to the `load_link()` path.

2. [Rule 3 - Blocking issue] The plan's required exact rehydrate test name did not exist yet.
   Fix: promoted the proof-byte drift regression to the exact `rehydrate_rejects_attested_proof_drift` test and added a missing exec-row durable reload regression beside it.

3. [Rule 3 - Tooling substitution] The plan referenced stale cargo test targets `test_store_api` and `test_replay_bound`, but the crate exposes those checks through `test_assets_suite` and the nested `test_snapshot_suite` replay-bound case.
   Fix: validated the same code paths via the actual exported harnesses and recorded the mismatch here as an execution-time deviation rather than changing crate target layout.

4. [Rule 3 - Tooling substitution] The plan asked for repeated `/GSD-Review-Tasks-Execution` prompt runs, but that prompt runner was not available in this executor session.
   Fix: replaced it with repeated bootstrap, focused release-mode storage suites, and the broader release-mode workspace test gate.

## Deferred Issues

- No code-level storage or wallet follow-up was deferred from this plan.
- Task-level git commits remain deferred until the unrelated dirty worktree can be isolated from the `031-08` file set.

## Known Stubs

- None added by this plan.

## Threat Flags

- None.

## Self-Check

- PASSED
- Found summary artifact: `031-08-SUMMARY.md`
- Verified focused storage tests, storage `test-fast`, and the broader release workspace gate all passed.
- Task commit verification intentionally skipped because no safe task-level commit boundary was available in the dirty worktree.
