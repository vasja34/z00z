---
phase: 067
plan: 067-03
status: complete
completed_at: 2026-07-04
next_plan: 067-04
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-03-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-03 Summary: Secondary Replay Verifier

## Outcome

`067-03` is complete.

`PHASE-2` now closes on a runtime-owned secondary replay path instead of
fixture-byte comparison or primary-owned precomputation. The live runtime now
rebuilds ingress items, replans the batch, rechecks ready-secondary recovery,
recomputes membership, publication, theorem, and optional DA bindings, and only
accepts a claimed primary subject when the locally replayed `CommitSubject`
matches exactly.

The closeout also records the honest validation unblocker that surfaced during
the final broad reruns: the checkpoint contract config remains storage-owned at
`crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`. The storage
loader constant now matches that canonical path again, and no root-level
parallel config lane remains.

## Files Changed

- `.planning/phases/067-Sharded-Concensus/067-03-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-03-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_runtime/aggregators/src/secondary_replay.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/tests/test_secondary_replay_verifier.rs`
- `crates/z00z_runtime/validators/Cargo.toml`
- `crates/z00z_runtime/validators/src/verdict.rs`
- `crates/z00z_storage/src/checkpoint/contract_config.rs`

## Landed Changes

- Runtime-owned replay verifier
  - Added `SecondaryReplayVerifier`, `SecondaryReplayRequest`,
    `SecondaryReplayVerdict`, `SecondaryReplayAccept`,
    `SecondaryReplayReject`, and `SecondaryReplayRejectCode`.
  - `replay_subject()` now reuses `IngressBoundary`, `BatchPlanner`,
    `ShardPlacementTable`, `RecoveryBoundary::resume(...TakeoverSecondary)`,
    `membership_digest_for_voters`, `CommitSubject::from_runtime`, and live
    publication or theorem or DA inputs rather than trusting fixture bytes.
- Stable failure evidence
  - Replay mismatches now fail closed on stable named codes including
    `ShapeInvalid`, `WrongRoute`, `WrongPlanDigest`, `WrongRoot`,
    `WrongLineage`, `WrongProofVersion`, `WrongPolicyGeneration`,
    `WrongPublicationBinding`, `WrongTheoremDigest`,
    `WrongDaAvailability`, `MembershipDrift`, `StaleSecondaryState`, and
    `WrongTerm`.
  - The replay test matrix now asserts the exact accept path and the named
    reject path for route, plan, root, lineage, proof, policy, publication,
    theorem, membership, DA, term, and stale-secondary failures.
- Existing primitives reused only
  - No new external crates were added for `067-03`.
  - The only dependency movement in this slice was wiring the existing
    workspace crate `z00z_crypto` into `z00z_validators` so the validator
    surface could expose theorem-digest helpers instead of creating a parallel
    hashing lane.
  - `SettlementTheoremBundle::theorem_digest()` and
    `ResolvedBatch::theorem_digest()` now expose the digest through the
    existing validator-owned surface.
- Honest storage-path unblocker
  - Restored `CHECKPOINT_CONTRACT_CONFIG_PATH` to
    `crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml`, matching the
    live storage module, phase docs, and recursive-proof docs.
  - The final bootstrap and broad release reruns therefore exercised the same
    single storage-owned config gate that the repository docs declare as
    canonical.

## Validation

Commands green during the final `067-03` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_storage --lib checkpoint::contract_config::tests::test_repo_contract_loads -- --nocapture`
- `cargo test --release -p z00z_storage checkpoint::contract_config -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_secondary_replay_verifier -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_dist_journal -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
- `cargo test --release`
- `bash scripts/audit/audit_release_feature_guards.sh`

The broad release rerun explicitly passed the restored storage config gate,
the new replay-verifier suite, the long `scenario_1` integration tail, wallet
and storage and rollup and validator and watcher suites, and the default
doc-test tail that `cargo test --release` executes for the workspace.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --bare --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-03-PLAN.md current_task="067-03-T1" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83737 > 38936`
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-03-PLAN.md current_task="067-03-T1"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-03-PLAN.md current_task="Secondary Replay Verifier" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66676 > 38936`

Equivalent workspace-first manual review was executed against the same scope.

- Pass 1
  - Re-read `067-03-PLAN.md`, `067-TODO.md` `14.3`, `067-CONTEXT.md`,
    `secondary_replay.rs`, the updated theorem-digest helpers in
    `verdict.rs`, and the new replay tests.
  - Result: found one real closeout gap in the public reject-code evidence:
    the replay suite did not yet assert stable named rejects for
    `MembershipDrift`, `WrongDaAvailability`, and `WrongTerm`. Extended
    `test_named_drift_axes_reject_with_stable_codes` to cover all three.
- Pass 2
  - Re-read `contract_config.rs`,
    `docs/tech-papers/Recursive-Proof-Spec.md`, and
    `docs/tech-papers/Recursive-Ready-Checkpoint-Contract.md`, then reran
    `bootstrap_tests.sh`, the storage subset, and the targeted replay tests.
  - Result: clean. The checkpoint contract path is storage-owned and crate
    local, no root-level parallel config path remains, and the focused storage
    and replay gates all passed after the path restore.
- Pass 3
  - Re-ran `cargo test --release` and
    `bash scripts/audit/audit_release_feature_guards.sh`, then re-read
    `067-COVERAGE.md`, `067-03-PLAN.md`, and the touched runtime files.
  - Result: clean. The broad release rerun, the default doc-test tail, and the
    release feature-guard audit all passed, and the replay verifier remains the
    single runtime-owned vote-eligibility path with no parallel crypto or HJMT
    layer.

Passes 2 and 3 were consecutive clean manual review runs after the last
in-scope fix.

## Closeout

`067-03` closes `PHASE-2` by making every secondary vote mean an independent
deterministic replay over the live runtime inputs that actually matter:
ingress, planning, placement, recovery, publication, theorem, and optional DA
state. The repository now has one canonical replay-verifier path, one
canonical storage-owned checkpoint contract path, one stable reject-code story
for scenario evidence, and one honest release validation result for this slice.

`067-04` is now the next canonical execution lane.
