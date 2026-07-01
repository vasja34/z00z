---
phase: 059-Core-Upgrade
plan: 059-02
status: complete
completed: 2026-06-16
owner: Z00Z Planning
---

# 059-02 Summary: Core Object Vocabulary, Policy Descriptors, And Action Pools

## Scope Delivered

- Landed canonical Phase 059 module roots in `z00z_core` for `actions`,
  `policies`, `rights`, and `vauchers` instead of leaving the new vocabulary
  stranded under `assets/*`.
- Added deterministic content-addressed core types for `ObjectFamily`,
  `ObjectRoleV1`, `ActionId`, `ActionPoolId`, `PolicyId`,
  `ActionDescriptorV1`, `ActionPoolDescriptorV1`, `ConditionDescriptorV1`,
  `PolicyDescriptorV1`, `RightPolicyV1`, `RightRequirementV1`,
  `VoucherPolicyV1`, `VoucherLifecycleV1`, and `VoucherConfigEntry`.
- Kept `assets/{action_pool,policy_descriptor,voucher_config}` as thin
  compatibility facades only, with one canonical implementation path for each
  concept and tests proving canonical bytes or hashes stay stable.
- Expanded right-config validation so rights stay zero-value authority and
  reject value-like keys including `support`, `reserve`, `amount`, `nominal`,
  `backing`, and `value` in addition to the already forbidden fee or payer or
  sponsor family.
- Synced the native cash fixed-profile story across descriptors and live asset
  definitions by routing `native_fee_def()` and gas tests through
  `NATIVE_CASH_POLICY_FLAGS`, preserving one canonical fixed-cash profile and
  no arbitrary action-pool path for native assets.
- Tightened the workspace rename guard to accept the already-live
  `generated_kani_*` test-file class explicitly, avoiding false failures
  without renaming repository fixtures or inventing a second naming rule.

## Boundary Kept

- This slice stayed inside `z00z_core` vocabulary and validation seams; it did
  not yet add genesis issuance, storage leaf families, wallet owned-object
  persistence, runtime verdict logic, or simulator Alice/Bob/Charlie transfer
  coverage for vouchers and rights.
- Native cash was not widened into programmable per-instance policy/action
  semantics.
- `assets/*` was not promoted into a second canonical owner; the new roots own
  the vocabulary and `assets/*` only re-export it where compatibility is still
  needed.
- No duplicate Phase 059 directory or parallel semantic layer was introduced.

## Validation

- Mandatory bootstrap gate passed on the final code:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted Phase 059 tests passed:
  `cargo test -p z00z_core --release test_policy_descriptor -- --nocapture`
  `cargo test -p z00z_core --release test_voucher_config -- --nocapture`
  `cargo test -p z00z_core --release test_rights_config -- --nocapture`
- Workspace guard regression passed:
  `cargo test -p z00z_wallets --release --test test_rename_guards test_test_file_prefix_guard -- --nocapture`
- Broad workspace validation passed on the final code:
  `cargo test --release`
- `git diff --check` on the touched core and planning files was clean.
- Manual review against `.github/prompts/gsd-review-tasks-execution.prompt.md`
  was run in three passes:
  pass 1 found two significant issues and both were fixed:
  the native-cash fixed-profile drift between `NATIVE_CASH_POLICY_FLAGS` and
  `native_fee_def()`, and the false-negative rename guard for live
  `generated_kani_*` tests.
  pass 2 found no further significant code issues after the fixes.
  pass 3 found no significant planning/state/roadmap sync issues after
  closeout.
- During broad validation, a stale bootstrap-owned `cargo-nextest` orphan tree
  was discovered holding the shared Stage 13 fixture lock; the orphan process
  tree was terminated so the fresh `cargo test --release` run could complete
  on the current code instead of hanging behind stale lock ownership.

## Next Plan

Execution moves to `059-03-PLAN.md` for core genesis policies, vouchers, and
publication.
