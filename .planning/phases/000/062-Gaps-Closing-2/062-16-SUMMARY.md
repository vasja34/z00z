---
phase: 062-Gaps-Closing-2
plan: 062-16
status: complete
completed_at: 2026-06-26
next_plan: 062-17
summary_artifact_for: .planning/phases/062-Gaps-Closing-2/062-16-PLAN.md
---

# 062-16 Summary: Local HJMT Boundary And Route Evidence

## Outcome

`062-16` is complete. The live local HJMT route/proof/publication logic was
already present on the current tree; this closeout normalized the remaining
authority wording and fixture ownership so the plan now closes on one
canonical boundary path.

`ShardRouteTable` remains the runtime-owned route-table canonicalization lane,
while `PublicationRouteSnapshotV1` and `check_publication_route_v1(...)`
remain the storage-owned public proof contract. Wallet-facing code is now
explicitly bounded to public proofs/API only, storage-created scopes remain
storage-owned semantic truth, and `config/hjmt_runtime` is explicitly
documented and tested as runtime-owned fixture home rather than storage
semantic authority.

The SIM-5A7S manifest now carries those ownership strings directly and
`test_hjmt_topology` enforces them, so the live fixture contract and the docs
point at the same single path. With that wording closure in place, the
mandatory bootstrap rerun is green, focused
validator/watcher/rollup-node/simulator release validation is green, the final
broad `cargo test --release` rerun is green, and the active execution lane
advances to `062-17`.

## Files Changed

- `crates/z00z_runtime/aggregators/src/batch_planner.rs`
- `crates/z00z_storage/src/settlement/proof_batch.rs`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `config/hjmt_runtime/sim_5a7s/manifest.json`
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `.planning/phases/062-Gaps-Closing-2/062-16-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_validators --test test_hjmt_publication_contract`
- `cargo test --release -p z00z_watchers --test test_hjmt_publication_contract`
- `cargo test --release -p z00z_rollup_node --test test_hjmt_topology`
- `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_e2e:: -- --nocapture`
- `cargo test --release`
- `git diff --check -- crates/z00z_runtime/aggregators/src/batch_planner.rs crates/z00z_storage/src/settlement/proof_batch.rs crates/z00z_storage/src/settlement/proof_batch_verify.rs crates/z00z_storage/src/settlement/hjmt_journal.rs crates/z00z_storage/src/settlement/hjmt_commit.rs crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs crates/z00z_rollup_node/src/config.rs crates/z00z_rollup_node/tests/test_hjmt_topology.rs config/hjmt_runtime/sim_5a7s/manifest.json .planning/phases/Z00Z-IMPL-PHASES.md .planning/phases/062-Gaps-Closing-2/062-16-SUMMARY.md .planning/STATE.md .planning/ROADMAP.md`
- `rg -n "config/hjmt_runtime|wallet sees public proofs|storage-created scopes|route-table" crates config .planning/phases/Z00Z-IMPL-PHASES.md`

Result:

- `bootstrap_tests.sh` completed green before broader validation.
- The focused validator, watcher, topology, and simulator release reruns
  completed green on the live route/publication boundary.
- The SIM-5A7S manifest ownership fields are live and topology-tested.
- `git diff --check` stayed clean on the touched scope.
- The canonical boundary grep now finds the exact `wallet sees public
  proofs/API only`, `storage-created scopes`, `config/hjmt_runtime`, and
  `route-table` strings on the intended live code/doc/config paths.
- The final broad `cargo test --release` rerun completed green on the current
  tree.

## Manual Review Passes

Because `./.github/prompts/gsd-review-tasks-execution.prompt.md` is a local
prompt file rather than a callable tool in this session, the required YOLO
review loop was executed manually against that prompt and the live `062-16`
scope.

- Pass 1
  - Read `062-16-PLAN.md`, `062-TODO.md`, the HJMT source corpus, and the
    local planner/proof/journal/store/config paths before edits.
  - Result: found that the live proof logic already existed, but the exact
    canonical boundary strings and runtime-fixture ownership wording were not
    yet all visible on one searched path.
- Pass 2
  - Re-read the edited planner/proof/journal/store/config and manifest paths
    against the prompt's trust-boundary, spec-drift, and checklist criteria.
  - Result: clean.
- Pass 3
  - Re-ran the canonical boundary grep, reviewed the diff, and checked
    `git diff --check` on the touched scope.
  - Result: clean.
- Pass 4
  - Re-reviewed the focused release reruns for validators, watchers,
    rollup-node topology, and `scenario_1` HJMT evidence after the wording and
    fixture-ownership closeout.
  - Result: clean.
- Pass 5
  - Re-reviewed `062-16-SUMMARY.md`, `STATE.md`, and `ROADMAP.md` after
    closeout updates and reran the final scoped `git diff --check`.
  - Result: clean.

Passes 4 and 5 were consecutive clean review runs for the final `062-16`
closeout state.

## Task Status

- `TASK-085`
  - Closed by the live local HJMT proof/root/route evidence on the planner,
    proof, journal, commit, and runtime-fixture paths.
- `TASK-086`
  - Closed by the deterministic route-table bytes/digest contract and the
    explicit canonical route-table ownership wording on the live planner path.
- `TASK-087`
  - Closed by the existing single-shard and cross-shard guardrails staying on
    one planner acceptance path.
- `TASK-088`
  - Closed by keeping historical route/policy/epoch/root continuity bound to
    the existing proof contract without introducing a second history
    authority.
- `TASK-089`
  - Closed by the storage-owned publication route binding, checkpoint
    continuity, and shard-leaf checks on the public proof surface.
- `TASK-102`
  - Closed by the green validator/watcher route-binding release reruns,
    including the advisory-local-metadata negative coverage already present on
    the current tree.
- `TASK-104`
  - Closed by making the wallet/public-proof boundary explicit on canonical
    code, config, and doc paths while keeping raw backend/journal internals out
    of the wallet surface.
- `TASK-120`
  - Closed by documenting and testing that storage-created scopes stay
    storage-owned semantic truth while `config/hjmt_runtime` stays the
    runtime-owned fixture home.
