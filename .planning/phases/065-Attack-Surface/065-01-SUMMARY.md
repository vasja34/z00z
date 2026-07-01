---
phase: 065-Attack-Surface
plan: 065-01
status: complete
completed_at: 2026-07-01
next_plan: 065-02
summary_artifact_for: .planning/phases/065-Attack-Surface/065-01-PLAN.md
---

# 065-01 Summary: Theorem-Verified Validator Acceptance

## Outcome

`065-01` is complete.

The live tree now uses one validator-owned theorem bundle for accepted-path
settlement truth. `z00z_validators` owns the canonical
`verify_settlement_theorem(...)` implementation and the typed
`SettlementTheoremBundle`, while `z00z_rollup_node` only re-exports that
canonical verifier and no longer carries a parallel rollup-owned theorem
implementation.

`ResolvedBatch` no longer carries an artifact-only accepted lane. The
accepted-path contract now requires a full `SettlementTheoremBundle`
containing the `TxPackage`, `CheckpointArtifact`, `CheckpointExecInput`, and
`CheckpointLink`, and `ResolvedBatch::new(...)` requires that bundle up
front. This removes optional theorem state from the accepted path and makes
the theorem/publication/link story one typed validator-owned input packet.

The local DA adapter now validates theorem coherence on both publish and
resolve. It builds checkpoint artifacts from the real execution input,
includes `tx_package + exec_input + link` in the request payload digest, and
constructs `ResolvedBatch` only from a theorem-validated bundle. Validator
checkpoint flow also rechecks that the published checkpoint id matches both
the artifact and the canonical link.

## Files Changed

- `.planning/phases/065-Attack-Surface/065-01-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `crates/z00z_rollup_node/README.md`
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_rollup_node/src/lib.rs`
- `crates/z00z_rollup_node/tests/support/test_theorem_fixture.rs`
- `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/validators/Cargo.toml`
- `crates/z00z_runtime/validators/src/checkpoint.rs`
- `crates/z00z_runtime/validators/src/lib.rs`
- `crates/z00z_runtime/validators/src/verdict.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`
- `crates/z00z_runtime/validators/tests/test_theorem_support.rs`
- `crates/z00z_wallets/tests/test_sensitive_rpc_session.rs`

## Landed Changes

- `crates/z00z_runtime/validators/src/verdict.rs`
  - added `SettlementTheoremBundle` and moved the canonical
    `verify_settlement_theorem(...)` implementation into `z00z_validators`
  - made `ResolvedBatch` theorem-owned through `pub theorem:
    SettlementTheoremBundle`
  - removed the accepted-path artifact-only construction contract by requiring
    `ResolvedBatch::new(...)` to take a theorem bundle
- `crates/z00z_rollup_node/src/lib.rs`
  - replaced the rollup-owned verifier implementation with a re-export of the
    validator-owned canonical verifier surface
- `crates/z00z_runtime/aggregators/src/types.rs`
  - extended `PublicationRequest` with `tx_package`, `exec_input`, and `link`
    so the publish/resolve path carries the full theorem input set
- `crates/z00z_rollup_node/src/da.rs`
  - made publish fail closed unless a real theorem bundle validates
  - made resolve construct `ResolvedBatch` only from a verified theorem bundle
  - bound the payload digest to `pub_in`, `tx_package`, `exec_input`, `link`,
    and `nullifiers`
- `crates/z00z_runtime/validators/src/checkpoint.rs`
  - added explicit link checkpoint-id coherence on top of artifact-derived
    checkpoint coherence
- `crates/z00z_rollup_node/README.md`
  - removed architectural drift and documented the verifier as a canonical
    validator-owned re-export instead of a rollup-owned path
- validator and rollup regression fixtures/tests
  - added theorem-support fixtures
  - updated DA/local publication tests to assert `link()` and `exec_input()`
    survive resolve intact
  - added negative theorem-link rejection coverage
- broad validation path repair
  - fixed the stale include-path in
    `crates/z00z_wallets/tests/test_sensitive_rpc_session.rs` from the removed
    `.planning/phases/064-Gaps-Closing-3/064-TODO.md` root to the live
    canonical `.planning/phases/000/064-Gaps-Closing-3/064-TODO.md` path so
    the workspace release suite uses one real Phase 064 authority location

## Validation

Commands already green on the current tree:

- `cargo fmt --all`
- `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- `cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard -- --nocapture`
- `cargo test --release -p z00z_validators --test test_object_policy_verdicts -- --nocapture`
- `cargo test --release -p z00z_rollup_node --test test_da_local_sim -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release --quiet`
- `git diff --check -- .planning/STATE.md .planning/ROADMAP.md crates/z00z_rollup_node/README.md crates/z00z_rollup_node/src/da.rs crates/z00z_rollup_node/src/lib.rs crates/z00z_rollup_node/tests/test_da_local_sim.rs crates/z00z_runtime/aggregators/src/types.rs crates/z00z_runtime/validators/Cargo.toml crates/z00z_runtime/validators/src/checkpoint.rs crates/z00z_runtime/validators/src/lib.rs crates/z00z_runtime/validators/src/verdict.rs crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs crates/z00z_rollup_node/tests/support/test_theorem_fixture.rs crates/z00z_runtime/validators/tests/test_theorem_support.rs`
- `rg -n "pub fn verify_settlement_theorem|fn verify_settlement_theorem" crates`
- `rg -n "Option<.*SettlementTheorem|Option<.*theorem|Option<.*CheckpointExecInput|Option<.*CheckpointLink|Option<.*TxPackage" crates/z00z_rollup_node crates/z00z_runtime/validators`

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted multiple times
in YOLO mode, but the available runtime does not currently provide a reliable
callable review path for this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-01-PLAN.md current_task="Theorem-Verified Validator Acceptance"'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 30s gsd --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-01-PLAN.md current_task="Theorem-Verified Validator Acceptance" --yolo'`
  - Result: failed with `402 Prompt tokens limit exceeded: 66676 > 38936`
- Attempt 3
  - `timeout 30s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-01-PLAN.md current_task="Theorem-Verified Validator Acceptance" --yolo'`
  - Result: failed with `402 Prompt tokens limit exceeded: 66676 > 38936`
- Attempt 4
  - `timeout 30s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/065-Attack-Surface/065-01-PLAN.md current_task="Theorem-Verified Validator Acceptance" --yolo'`
  - Result: failed with `402 Prompt tokens limit exceeded: 66676 > 38936`

Equivalent workspace-first review was executed manually against the same
scope.

- Pass 1
  - Re-read `065-01-PLAN.md`, `065-TODO.md`, and the touched rollup/validator
    code paths.
  - Result: found one real architectural drift issue in
    `crates/z00z_rollup_node/README.md`; the README incorrectly described the
    verifier as rollup-owned and was corrected to the canonical validator-owned
    re-export wording.
- Pass 2
  - Re-checked the theorem-bundle construction path, duplicate-verifier
    surface, and anti-placeholder contract through the touched code and grep
    evidence.
  - Result: clean. One canonical theorem verifier remains in
    `crates/z00z_runtime/validators/src/verdict.rs`, `ResolvedBatch` has no
    optional theorem lane, and `PublicationRequest` carries the full theorem
    input set.
- Pass 3
  - Re-checked negative test coverage and scoped diff hygiene on the touched
    files.
  - Result: clean. The current tree has explicit rejection coverage for bad
    theorem links, route-digest drift, publication-binding drift, exec-input
    drift, proof tamper, and checkpoint coherence drift on the touched
    validator/rollup/simulator paths.
- Pass 4
  - Reproduced the first broad `cargo test --release` blocker and traced it to
    a stale include-path in
    `crates/z00z_wallets/tests/test_sensitive_rpc_session.rs`.
  - Result: fixed. The test now points at the live canonical
    `.planning/phases/000/064-Gaps-Closing-3/064-TODO.md` path instead of the
    removed non-`000` root.
- Pass 5
  - Re-ran the final broad release gate after the theorem helper fixtures were
    renamed to canonical `test_*` file names to satisfy the workspace rename
    guard.
  - Result: green. `cargo test --release --quiet` completed on the current
    tree, so `065-01` is closed and `065-02` is unblocked.

Passes 2 and 3 were consecutive clean manual review runs for the theorem-owned
`065-01` slice before the separate broad-release path fix in pass 4.

## Closeout

The final bootstrap rerun and the final broad workspace release rerun are both
green on the current tree. The only additional closeout delta after the
theorem-bundle landing was the canonical `test_*` rename for the two helper
fixture files required by the workspace rename guard.
