---
phase: 059-Core-Upgrade
plan: 059-05
status: complete
completed: 2026-06-17
owner: Z00Z Planning
---

# 059-05 Summary: Typed Object Deltas, Conservation, Lifecycle, And Fee Boundary

## Scope Delivered

- Extended the existing storage execution lane in place with typed mixed-object
  deltas, object-role-aware conservation checks, and explicit voucher
  lifecycle handling across `record.rs`, `tx_plan_types.rs`, `tx_plan_help.rs`,
  `store.rs`, `hjmt_plan.rs`, and `hjmt_commit.rs` instead of introducing a
  parallel object-settlement layer.
- Added storage query, backend, and recovery support so typed object deltas for
  assets, vouchers, rights, and fee support survive commit, reload, version
  lookup, and root-bound history traversal on the same canonical HJMT path.
- Kept the value split honest: rights remain zero-value authority objects,
  `FeeEnvelope` remains support-only, reserve-backed or genesis-backed voucher
  issue is treated as an issuance action instead of a fake local cash output,
  and storage continues to validate committed object state rather than wallet
  secrets or private openings.
- Added regression coverage for voucher lifecycle transitions, right zero-value
  rejection, fee-boundary rejection, reserve-backed issue accounting, typed
  delta store APIs, durable delta-history reload, and scheduler-aware async
  commit staging.

## Boundary Kept

- This slice did not introduce validator or watcher policy authority, wallet
  typed-object persistence, wallet package building, Alice/Bob/Charlie
  simulator transfer coverage, or final cross-crate closeout; those remain in
  `059-06` through `059-10`.
- Storage did not become a second wallet authority: descriptor hashes, roots,
  proofs, and typed deltas are persisted, while wallet-local secrets remain
  outside the storage record plane.
- The existing settlement root and scheduler path stayed canonical; typed
  object delta support was added on the current commit or replay flow instead
  of via a new object journal or alternate execution route.

## Validation

- Mandatory bootstrap gate passed on the final code:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted storage release validation passed:
  `cargo test -p z00z_storage --release test_model -- --nocapture`
  `cargo test -p z00z_storage --release test_store_api -- --nocapture`
  `cargo test -p z00z_storage --release --test test_hjmt_transition_proofs -- --nocapture`
  `cargo test -p z00z_storage --release --test test_fee_envelope -- --nocapture`
  `cargo test -p z00z_storage --release --features test-params-fast test_hjmt_reload_preserves_object_delta_history -- --nocapture`
  `cargo test -p z00z_storage --release --features test-params-fast --test test_async_scheduler -- --nocapture`
  `cargo test -p z00z_storage --release --features test-params-fast`
- Broad workspace validation passed on the final code:
  `cargo test --release`
- `git diff --check` must stay clean on the touched storage and planning files
  for this slice.
- Manual review against `.github/prompts/gsd-review-tasks-execution.prompt.md`
  was run in three passes:
  pass 1 found a missing negative fee-boundary proof for voucher backing plus a
  conservation bug where reserve-backed voucher issue was treated like a local
  value output; both were fixed.
  pass 2 found a typed-delta recovery gap and a scheduler-stage regression
  where delta-aware commit paths bypassed `sched_plan_ops(...)`; both were
  fixed and rerun under release tests.
  pass 3 found no significant code or planning sync issues after the final
  storage gate, broad `cargo test --release`, and state or roadmap closeout.

## Next Plan

Execution moves to `059-06-PLAN.md` for runtime admission, validator verdicts,
and watcher alerts over typed object packages.
