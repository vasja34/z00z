---
phase: 059-Core-Upgrade
plan: 059-06
status: complete
completed: 2026-06-17
owner: Z00Z Planning
---

# 059-06 Summary: Runtime Admission, Validator Verdicts, And Watcher Alerts

## Scope Delivered

- Extended the existing runtime lane in place so typed object packages can move
  through aggregator work items, ordered batches, validator inspection, watcher
  alerts, and rollup status or RPC surfaces without creating a second semantic
  authority beside storage and validators.
- Added fail-closed validator object verdicts for unknown policy, missing or
  out-of-scope rights, fee-boundary violations, wrong-family or stale proof
  conditions, and other typed object action failures, while reusing storage
  contract checks instead of duplicating settlement logic.
- Added watcher alert and evidence-export coverage for object reject codes, and
  projected those reject codes through rollup-node status and RPC so runtime
  object failures stay visible without exposing wallet secrets.
- Fixed the canonical runtime binding gap by making `WorkItem` admission or
  plan identity bind the attached object package while preserving payload-based
  routing, so a `TxPackage` cannot be replayed under different object
  semantics on the same intake or plan digest.
- Added regression coverage for validator object verdicts, watcher object
  alerts, rollup projection, and aggregator digest binding behavior.

## Boundary Kept

- Aggregators still do not decide object semantics: they carry typed evidence,
  preserve payload routing, and leave semantic authority to validators and
  storage-backed contract checks.
- The fix did not create a second route key, object journal, or alternate plan
  path; routing remains payload-digest-based while admission or plan identity
  now binds object-package bytes explicitly.
- Wallet typed-object persistence, wallet scan or RPC object flows, simulator
  Alice/Bob/Charlie object transfer evidence, and final cross-crate closeout
  remain in `059-07` through `059-10`.

## Validation

- Mandatory bootstrap gate passed on the final code:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted runtime release validation passed:
  `cargo test -p z00z_aggregators --release -- --nocapture`
  `cargo test -p z00z_validators --release -- --nocapture`
  `cargo test -p z00z_watchers --release -- --nocapture`
  `cargo test -p z00z_rollup_node --release -- --nocapture`
- Broad workspace validation passed on the final code:
  `cargo test --release`
- `git diff --check` must stay clean on the touched runtime and planning files
  for this slice.
- Manual review against `.github/prompts/gsd-review-tasks-execution.prompt.md`
  was run in three passes:
  pass 1 found validator or watcher test dependency gaps and unreachable-match
  noise around the new object verdict surface; both were fixed.
  pass 2 found runtime binding drift where object packages changed semantics
  without changing `WorkItem` admission or plan identity; this was fixed by
  binding object-package bytes into the admission digest while preserving the
  payload route key.
  pass 3 found no significant code or planning sync issues after the final
  runtime gate, broad `cargo test --release`, and state or roadmap closeout.

## Next Plan

Execution moves to `059-07-PLAN.md` for wallet typed object inventory and
persistence.
