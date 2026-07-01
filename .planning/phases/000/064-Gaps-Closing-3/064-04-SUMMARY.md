---
phase: 064-Gaps-Closing-3
plan: 064-04
status: complete
completed_at: 2026-06-30
next_plan: 064-05
summary_artifact_for: .planning/phases/064-Gaps-Closing-3/064-04-PLAN.md
---

# 064-04 Summary: Storage Proof Boundaries And Runtime Adversarial Closure

## Outcome

`064-04` is complete. `PLAN-064-G04` now closes `REC-064-P2-02`,
`REC-064-P1-06`, `REC-064-P1-07`, `REC-064-P1-08`, `REC-064-P1-09`,
`REC-064-P1-10`, and `REC-064-P1-11` through one canonical
storage/runtime/rollup validation packet.

Checkpoint and snapshot authority now stay explicit on the current tree.
`test_checkpoint_store.rs` locks `seal_artifact()` in as the only canonical
statement-bound checkpoint path and keeps Stage 4 raw-save access on the
local simulator seam only, while `test_prep_snapshot.rs` covers the explicit
`PrepSnapshot` adversarial matrix with deterministic negative cases for
version gating, witness decode and family drift, path or serial or terminal
mixes, duplicate ids, leaf mismatches, and root mismatches.

Settlement and theorem boundary claims also stay honest. The new
`test_settlement_proof_boundaries.rs` proves semantic settlement roots stay
separate from backend proof state and rejects tampered backend or bound roots,
while `test_rollup_theorem_guard.rs` now rejects detached statements, wrong
checkpoint-proof payloads, mismatched ids, and broken link roots using the
live theorem verifier.

Local DA and runtime adversarial closure is now isolated on canonical
acceptance targets instead of mixed test files. `test_da_local_sim.rs` proves
the local adapter rejects forged source labels, forged publication digests,
payload drift, missing resolve results, and replayed inputs without claiming a
real DA network, while `test_recovery_failover.rs` and
`test_publication_binding.rs` close the deterministic failover and anti-fork
packet around split-brain fencing, replay safety, and one binding authority.

The supporting doc and source-guard corpus was truth-restored in the same
slice. Storage live-guardrail docs now point at the canonical planning
authority files instead of deleted phase-local duplicates, and the theorem or
recovery reference docs now name the live acceptance targets instead of stale
test paths. No production alias, shim, or second authority layer was added.

## Files Changed

- `crates/z00z_storage/tests/test_checkpoint_store.rs`
- `crates/z00z_storage/tests/test_prep_snapshot.rs`
- `crates/z00z_storage/tests/test_settlement_proof_boundaries.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`
- `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
- `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`
- `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
- `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`
- `wiki/05-storage-runtime/rollup-theorem-verifier.md`
- `docs/tech-papers/done/Z00Z-HJMT-Upgrade.md`
- `docs/tech-papers/refactor-recomendations.md`
- `.planning/phases/064-Gaps-Closing-3/064-04-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_storage --test test_checkpoint_store --test test_prep_snapshot --test test_settlement_proof_boundaries -- --nocapture`
- `cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard --test test_da_local_sim -- --nocapture`
- `cargo test --release -p z00z_aggregators --test test_recovery_failover --test test_publication_binding -- --nocapture`
- `cargo test --release -p z00z_storage -p z00z_rollup_node -p z00z_aggregators`
- `cargo test --release`
- `git diff --check -- crates/z00z_storage/tests/test_checkpoint_store.rs crates/z00z_storage/tests/test_prep_snapshot.rs crates/z00z_storage/tests/test_settlement_proof_boundaries.rs crates/z00z_storage/tests/test_live_guardrails.rs crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs crates/z00z_rollup_node/tests/test_da_local_sim.rs crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs crates/z00z_runtime/aggregators/tests/test_publication_binding.rs wiki/05-storage-runtime/rollup-theorem-verifier.md docs/tech-papers/done/Z00Z-HJMT-Upgrade.md docs/tech-papers/refactor-recomendations.md`
- `rg -n "test_settlement_theorem|test_checkpoint_store_api|test_hjmt_split_brain_fencing" wiki docs crates .planning/phases/064-Gaps-Closing-3`

- Result:
  - The mandatory bootstrap gate passed.
  - The targeted `z00z_storage`, `z00z_rollup_node`, and
    `z00z_aggregators` release-mode acceptance targets for `PLAN-064-G04`
    all passed.
  - The broad `cargo test --release -p z00z_storage -p z00z_rollup_node -p z00z_aggregators`
    rerun passed on the current tree.
  - The full workspace `cargo test --release` rerun still honestly reproduces
    current-tree `z00z_core` blockers outside the modified `064-04`
    storage/runtime/rollup slice:
    `genesis::genesis_manifest::test_genesis_plan_rights_only_requires_policy_resolution_when_needed`
    fails with
    `ConfigParseFailed("wallet profile validator_mandate_lock_v1 references unknown locked_asset_id z00z")`,
    and `genesis::genesis_rights::test_genesis_rights_deterministic`
    still reports current rights snapshot drift rooted in
    `crates/z00z_core/configs/devnet_genesis_config.yaml`.
  - `git diff --check` stayed clean for the touched `064-04` packet.
  - The stale deleted test names `test_settlement_theorem`,
    `test_checkpoint_store_api`, and `test_hjmt_split_brain_fencing` no
    longer appear in the checked corpus.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times
against this slice:

- Attempt 1
  - `gsd --print "/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md"`
  - Result: failed with `402 Prompt tokens limit exceeded: 66474 > 38936`
- Attempt 2
  - `gsd --print "/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md current_task=\"Expand local DA and runtime negative coverage without claiming a real DA network.\""`
  - Result: failed with `402 Prompt tokens limit exceeded: 66499 > 38936`
- Attempt 3
  - `gsd --print "/GSD-Review-Tasks-Execution current_spec=.planning/phases/064-Gaps-Closing-3/064-04-PLAN.md current_task=\"Add theorem-boundary negative tests for detached statement, wrong proof payload, wrong ids, and wrong link roots.\""`
  - Result: failed with `402 Prompt tokens limit exceeded: 66507 > 38936`

Equivalent workspace-first review passes were executed manually against the
same scope.

- Pass 1
  - `git diff -- crates/z00z_storage/tests/test_checkpoint_store.rs crates/z00z_storage/tests/test_prep_snapshot.rs crates/z00z_storage/tests/test_settlement_proof_boundaries.rs crates/z00z_storage/tests/test_live_guardrails.rs crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs crates/z00z_rollup_node/tests/test_da_local_sim.rs crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs crates/z00z_runtime/aggregators/tests/test_publication_binding.rs wiki/05-storage-runtime/rollup-theorem-verifier.md docs/tech-papers/done/Z00Z-HJMT-Upgrade.md docs/tech-papers/refactor-recomendations.md`
  - `rg -n "test_settlement_theorem|test_checkpoint_store_api|test_hjmt_split_brain_fencing" wiki docs crates .planning/phases/064-Gaps-Closing-3`
  - Result: canonical acceptance targets and docs align, and no stale deleted
    path names remain in the checked corpus
- Pass 2
  - `cargo test --release -p z00z_storage --test test_checkpoint_store --test test_prep_snapshot --test test_settlement_proof_boundaries -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard --test test_da_local_sim -- --nocapture`
  - `cargo test --release -p z00z_aggregators --test test_recovery_failover --test test_publication_binding -- --nocapture`
  - `git diff --check -- crates/z00z_storage/tests/test_checkpoint_store.rs crates/z00z_storage/tests/test_prep_snapshot.rs crates/z00z_storage/tests/test_settlement_proof_boundaries.rs crates/z00z_storage/tests/test_live_guardrails.rs crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs crates/z00z_rollup_node/tests/test_da_local_sim.rs crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs crates/z00z_runtime/aggregators/tests/test_publication_binding.rs wiki/05-storage-runtime/rollup-theorem-verifier.md docs/tech-papers/done/Z00Z-HJMT-Upgrade.md docs/tech-papers/refactor-recomendations.md`
  - Result: clean
- Pass 3
  - `cargo test --release -p z00z_storage -p z00z_rollup_node -p z00z_aggregators`
  - `cargo test --release`
  - Result: no significant issues remained in the modified `064-04` slice;
    only the current-tree `z00z_core` genesis/config blockers outside the
    changed scope were reproduced

Passes 2 and 3 were consecutive clean manual review passes for the modified
scope.

## Completion Notes

- `064-04-SUMMARY.md` closes `PLAN-064-G04` and advances the active execution
  lane to `064-05-PLAN.md`.
- The `064-04` packet stayed on one canonical path per behavior: no
  production alias, shim, or second authority layer was introduced.
- The remaining broad workspace blocker is still the pre-existing
  `z00z_core` genesis/config surface, not the modified `064-04`
  storage/runtime/rollup packet.
