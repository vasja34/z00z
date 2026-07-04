---
phase: 067
plan: 067-02
status: complete
completed_at: 2026-07-03
next_plan: 067-03
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-02-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-02 Summary: Commit Subject And Certificate Types

## Outcome

`067-02` is complete.

`PHASE-1` now closes on concrete first-class quorum artifacts instead of a
`JournalCandidate` plus voter-id list shortcut. The runtime owns a canonical
`CommitSubject`, `ShardVote`, and `ShardQuorumCertificate` path with
domain-separated digests, stable binary encoding, one-shard membership
semantics, fail-closed vote or certificate validation, and a
`ConsensusCommit` that carries the subject and quorum certificate directly.

The closeout also records the honest broad-validation unblocker that surfaced
after the protocol work landed: the intentional `crates/z00z_extensions/`
reorganization had to stop being treated as a root workspace crate. The
workspace membership and related inventories now follow the namespace-only
layout without introducing a parallel runtime or crypto layer.

## Files Changed

- `.planning/phases/067-Sharded-Concensus/067-02-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-02-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `Cargo.toml`
- `scripts/audit/audit_extensions_boundary.sh`
- `scripts/cargo_build_config.yaml`
- `scripts/profile_samply.sh`
- `versions.yaml`
- `crates/z00z_crypto/src/domains.rs`
- `crates/z00z_runtime/aggregators/Cargo.toml`
- `crates/z00z_runtime/aggregators/src/commit_subject.rs`
- `crates/z00z_runtime/aggregators/src/consensus_adapter.rs`
- `crates/z00z_runtime/aggregators/src/dist_sim.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/src/shard_quorum_certificate.rs`
- `crates/z00z_runtime/aggregators/src/shard_vote.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/aggregators/tests/test_commit_subject.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_consensus.rs`
- `crates/z00z_runtime/aggregators/tests/test_shard_quorum_certificate.rs`

## Landed Changes

- First-class quorum artifacts
  - Added `CommitSubject` with stable field order, explicit versioning, live
    route or generation or root or lineage or publication or theorem bindings,
    and domain-separated digesting through existing `z00z_crypto` primitives.
  - Added `ShardVote` with voter role binding and deterministic local-signature
    seam for simulation-only quorum assembly.
  - Added `ShardQuorumCertificate` with canonical vote sorting, duplicate or
    inactive voter rejection, membership-digest binding, same-term checks, and
    below-quorum rejection.
- Canonical commit integration
  - `ConsensusCommit` now carries `subject` plus `certificate`.
  - `ConsensusAdapter` now binds placement generation, membership digest, and
    same-term freeze logic through the new artifact layer instead of wrapping
    ad hoc voter lists.
- Existing primitives reused only
  - No new external crates were added for `067-02`.
  - The only dependency movement was promoting the existing workspace crate
    `z00z_crypto` into the runtime dependency surface for canonical domain
    hashing.
  - Existing `z00z_core`, `z00z_crypto`, `z00z_storage`, and `z00z_utils`
    seams remain the only authority path; no parallel HJMT, crypto, or utility
    layer was introduced.
- Honest workspace unblocker
  - Removed the stale root-crate membership for `crates/z00z_extensions` from
    the workspace and related inventories because the directory is now a
    namespace container with nested placeholders, not a live root crate.
  - Hardened `scripts/audit/audit_extensions_boundary.sh` so it supports both
    the historical root-crate mode and the current namespace-directory mode
    without silently activating nested manifests.
- Test-surface cleanup
  - Shortened the new quorum test names to respect the repository identifier
    length rule while keeping their semantics intact.

## Validation

Commands green during the final `067-02` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_commit_subject -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_shard_quorum_certificate -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast test_quorum_freezes_term_roots -- --nocapture`
- `cargo test --release`
- `cargo test --release --doc`
- `cargo metadata --format-version 1 --no-deps`
- `bash scripts/audit/audit_extensions_boundary.sh`
- `rg -n 'crates/z00z_extensions|\\bz00z_extensions\\b' --glob '!**/*.md' --glob '!**/*.txt' --glob '!**/*.json' --glob '!**/*.yaml' --glob '!**/*.yml' --glob '!**/*.lock' --glob '!**/*.toml' .`

The broad release rerun explicitly passed the old workspace blocker
`tests/scenario_1/test_workspace_target_dir.rs` and the new quorum-targeted
tests `test_commit_subject`, `test_shard_quorum_certificate`, and
`test_hjmt_consensus`.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --bare --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-02-PLAN.md current_task="067-02-T1" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83737 > 38936`
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-02-PLAN.md current_task="067-02-T1"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-02-PLAN.md current_task="Commit Subject And Certificate Types" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66678 > 38936`

Equivalent workspace-first manual review was executed against the same scope.

- Pass 1
  - Re-read `067-02-PLAN.md`, `067-TODO.md` `14.2`, `067-CONTEXT.md`, the new
    quorum artifact modules, the updated `consensus_adapter.rs`, and the new
    tests.
  - Result: found one real closeout issue in the new test surface: several
    added identifiers exceeded the repository word-count rule. Renamed the new
    tests to `test_commit_subject_drifts`, `test_shard_qc_sorting`,
    `test_shard_qc_member_rejects`, and `test_shard_qc_field_rejects`.
- Pass 2
  - Re-read the workspace and inventory diff that was required to rerun the
    broad gate honestly, then ran `cargo metadata`, the namespace-aware
    `audit_extensions_boundary.sh`, and the repo-wide `rg` check for live code
    references to `z00z_extensions` as a crate.
  - Result: clean. The workspace no longer points at a phantom root crate, the
    namespace boundary stays fail-closed, and no live code imports a
    non-existent `z00z_extensions` crate path.
- Pass 3
  - Re-ran `bootstrap_tests.sh`, re-ran the full `cargo test --release`, then
    rechecked the doc-test tail explicitly with `cargo test --release --doc`.
  - Result: clean. Targeted quorum tests, the old `test_workspace_target_dir`
    blocker, the long `scenario_1` integration suite, wallet or storage or
    validator or watcher tails, and the workspace doc-tests all passed.

Passes 2 and 3 were consecutive clean manual review runs after the last
in-scope fix.

## Closeout

`067-02` closes `PHASE-1` by making shard-local quorum claims concrete,
deterministic, and fail closed on the live runtime path. The codebase now has
one canonical subject or vote or quorum-certificate story, one `ConsensusCommit`
path that carries those artifacts forward, one honest dependency story with no
new external crate adoption for this slice, and one workspace layout that
treats `z00z_extensions` as the namespace directory the repository now intends
it to be.

`067-03` is now the next canonical execution lane.
