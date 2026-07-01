---
phase: 062-Gaps-Closing-2
plan: 062-15
status: complete
completed_at: 2026-06-26
next_plan: 062-16
summary_artifact_for: .planning/phases/062-Gaps-Closing-2/062-15-PLAN.md
---

# 062-15 Summary: Shared Object Vocabulary And Error Ownership

## Outcome

`062-15` is complete. The repository now exposes one canonical root facade for
shared object vocabulary and generic cross-object validation errors:
`z00z_core::ObjectFamily`, `z00z_core::ObjectRoleV1`, and
`z00z_core::AssetError` are the live public authority paths, while the
`z00z_core::assets::*` paths remain compatibility facades only.

The non-asset owners that consume those shared contracts now import them from
the root facade instead of from `assets`, so actions, policies, rights,
vouchers, genesis, storage, validators, watchers, and aggregator test surfaces
all follow one canonical import path. The shared-vocabulary docs now describe
that root-owned contract explicitly, and storage-side settlement docs state
that storage persists concrete owner-family leaves without introducing a second
asset-owned authority path.

The misplaced owner tests were also corrected without semantic rewrite.
`test_policy_descriptor` now lives under `crate::policies`, and
`test_voucher_config` now lives under `crate::vauchers`, removing the old
asset-owned `#[path = "../assets/..."]` inclusion pattern while keeping the
same owner-module semantics.

With those ownership and path corrections in place, the mandatory bootstrap
gate is green, the focused release reruns for the touched owner surfaces are
green, and the final broad `cargo test --release` rerun is green. The active
execution lane advances to `062-16`.

## Files Changed

- `crates/z00z_core/src/lib.rs`
- `crates/z00z_core/src/assets/asset_error.rs`
- `crates/z00z_core/src/assets/object_family.rs`
- `crates/z00z_core/src/assets/mod.rs`
- `crates/z00z_core/src/actions/action_descriptor.rs`
- `crates/z00z_core/src/actions/action_pool.rs`
- `crates/z00z_core/src/policies/condition_descriptor.rs`
- `crates/z00z_core/src/policies/policy_descriptor.rs`
- `crates/z00z_core/src/policies/policy_template.rs`
- `crates/z00z_core/src/policies/mod.rs`
- `crates/z00z_core/src/policies/test_policy_descriptor.rs`
- `crates/z00z_core/src/rights/config.rs`
- `crates/z00z_core/src/rights/right_policy.rs`
- `crates/z00z_core/src/rights/test_rights_config.rs`
- `crates/z00z_core/src/vauchers/mod.rs`
- `crates/z00z_core/src/vauchers/test_voucher_config.rs`
- `crates/z00z_core/src/vauchers/voucher_bootstrap.rs`
- `crates/z00z_core/src/vauchers/voucher_config.rs`
- `crates/z00z_core/src/vauchers/voucher_lifecycle.rs`
- `crates/z00z_core/src/vauchers/voucher_policy.rs`
- `crates/z00z_core/src/genesis/genesis_config.rs`
- `crates/z00z_core/src/genesis/genesis_error.rs`
- `crates/z00z_storage/src/settlement/record.rs`
- `crates/z00z_storage/src/settlement/object_package_contract.rs`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs`
- `crates/z00z_runtime/aggregators/src/types.rs`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`
- `crates/z00z_runtime/watchers/tests/test_object_alerts.rs`
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `.planning/phases/062-Gaps-Closing-2/062-15-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_core test_policy_descriptor -- --nocapture`
- `cargo test --release -p z00z_core test_voucher_config -- --nocapture`
- `cargo test --release -p z00z_core test_rights_config -- --nocapture`
- `cargo test --release -p z00z_core test_registry_suite -- --nocapture`
- `cargo test --release -p z00z_validators --test test_object_policy_verdicts -- --nocapture`
- `cargo test --release -p z00z_aggregators object_package_rebinds_intake_digest_without_rerouting -- --nocapture`
- `cargo test --release -p z00z_watchers --test test_object_alerts -- --nocapture`
- `cargo test --release`
- `git diff --check -- crates/z00z_core/src/lib.rs crates/z00z_core/src/assets/asset_error.rs crates/z00z_core/src/assets/object_family.rs crates/z00z_core/src/assets/mod.rs crates/z00z_core/src/actions/action_descriptor.rs crates/z00z_core/src/actions/action_pool.rs crates/z00z_core/src/policies/condition_descriptor.rs crates/z00z_core/src/policies/policy_descriptor.rs crates/z00z_core/src/policies/policy_template.rs crates/z00z_core/src/rights/right_policy.rs crates/z00z_core/src/rights/config.rs crates/z00z_core/src/rights/test_rights_config.rs crates/z00z_core/src/vauchers/voucher_bootstrap.rs crates/z00z_core/src/vauchers/voucher_config.rs crates/z00z_core/src/vauchers/voucher_lifecycle.rs crates/z00z_core/src/vauchers/voucher_policy.rs crates/z00z_core/src/genesis/genesis_config.rs crates/z00z_core/src/genesis/genesis_error.rs crates/z00z_core/src/policies/mod.rs crates/z00z_core/src/vauchers/mod.rs crates/z00z_core/src/policies/test_policy_descriptor.rs crates/z00z_core/src/vauchers/test_voucher_config.rs crates/z00z_storage/src/settlement/record.rs crates/z00z_storage/src/settlement/object_package_contract.rs crates/z00z_runtime/watchers/tests/test_object_alerts.rs crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs crates/z00z_runtime/aggregators/src/types.rs crates/z00z_runtime/aggregators/src/batch_planner.rs .planning/phases/Z00Z-IMPL-PHASES.md`
- `rg -n "assets::ObjectFamily|assets::AssetError|assets::ObjectRoleV1|crate::assets::ObjectFamily|crate::assets::AssetError|crate::assets::ObjectRoleV1" crates/z00z_core/src crates/z00z_runtime crates/z00z_storage -S`

Result:

- `bootstrap_tests.sh` completed green before broader validation.
- The owner-module release tests for policies, vouchers, rights, validators,
  aggregators, and watchers completed green on the root-facade path.
- `test_registry_suite` completed green under the requested release command but
  matched `0` tests on the current tree; the moved owner-test surfaces were
  instead proven by the focused `test_policy_descriptor` and
  `test_voucher_config` release reruns.
- `git diff --check` stayed clean on the touched scope.
- The remaining `assets::*` path hits are limited to internal compatibility or
  self-references inside `crates/z00z_core/src/assets/*`.
- The final broad `cargo test --release` rerun completed green on the current
  tree.

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
required review loop was executed manually against the same scope.

- Pass 1
  - Reviewed `062-15-PLAN.md`, `062-TODO.md`, `asset-only.md`, and the live
    code owners before edits.
  - Result: found the remaining asset-owned authority drift for shared
    vocabulary and generic errors, plus the misplaced policy/voucher owner
    tests still included from `../assets/*`.
- Pass 2
  - Re-read the edited root facade, shared-vocabulary docs, owner-module test
    modules, and canonical import sites across core/storage/runtime.
  - Result: clean.
- Pass 3
  - Re-ran the scoped canonical-path grep, `git diff --check`, and the focused
    release reruns on the touched owner surfaces.
  - Result: clean.
- Pass 4
  - Reran the mandatory broad `cargo test --release` gate on the current tree
    after the ownership cleanup.
  - Result: clean.
- Pass 5
  - Re-read `062-15-SUMMARY.md`, `STATE.md`, and `ROADMAP.md` after closeout
    updates and reran `git diff --check` on the new planning artifacts.
  - Result: clean.

Passes 4 and 5 were consecutive clean review runs for the final `062-15`
closeout state.

## Task Status

- `TASK-079`
  - Implemented by promoting `ObjectFamily` and `ObjectRoleV1` to the root
    `z00z_core` facade, documenting that path as canonical, and migrating
    non-asset owners to the root import path.
- `TASK-080`
  - Implemented by promoting `AssetError` to the root `z00z_core` facade,
    documenting `z00z_core::AssetError` as the canonical path, and removing
    asset-owned import wording from the consuming owner surfaces.
- `TASK-081`
  - Implemented by relocating the misplaced policy and voucher owner tests into
    their owner modules and removing the old asset-owned `#[path]` inclusions.
