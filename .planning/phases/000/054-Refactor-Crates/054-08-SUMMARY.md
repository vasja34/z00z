---
phase: 054-Refactor-Crates
plan: 054-08
status: complete
completed_at: 2026-06-09
next_plan: none
requirements-completed: [PH54-08]
---

# 054-08 Summary

## Outcome

Plan `054-08` is complete.

The runtime planner boundary now fails closed on one canonical payload-bound
digest path. `IngressBoundary::normalize(...)` recomputes tx and claim digests
from payload bytes, rejects forged `tx_digest_hex` metadata before planning,
and emits the only planner-ready `WorkItem` type. Route lookup, ordered intake
ids, and `plan_digest` now use verified runtime metadata only, and the
remaining public-path bypass risk is guarded by release-tested source-shape
checks.

## Code And Planning Changes

- Bound runtime ingress to the live wallet digest helpers instead of caller
  metadata:
  - `crates/z00z_runtime/aggregators/src/ingress.rs`
  - `crates/z00z_runtime/aggregators/src/types.rs`
- Removed the planner's last raw-digest authority path:
  - `crates/z00z_runtime/aggregators/src/batch_planner.rs`
  - `crates/z00z_runtime/aggregators/src/service.rs`
- Added regression and source-shape coverage for the canonical public lane:
  - `crates/z00z_runtime/aggregators/src/ingress.rs`
  - `crates/z00z_runtime/aggregators/src/batch_planner.rs`
  - `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`
- Closed the follow-up security and planning record without reopening older
  Phase 054 waves:
  - `crates/z00z_runtime/aggregators/README.md`
  - `.planning/phases/054-Refactor-Crates/054-attack-surface-report.md`
  - `.planning/phases/054-Refactor-Crates/054-attack-surface-db.jsonl`
  - `.planning/phases/054-Refactor-Crates/054-SECURITY.md`
  - `.planning/phases/054-Refactor-Crates/054-VALIDATION.md`
  - `.planning/phases/054-Refactor-Crates/054-SUMMARY.md`
  - `.planning/ROADMAP.md`
  - `.planning/STATE.md`

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` —
  passed.
- `cargo test -p z00z_aggregators --release -q` — passed.
- `cargo test --release` — passed.
- `cargo doc --no-deps` — passed with pre-existing rustdoc warnings outside
  the `054-08` scope.
- `git diff --check` — passed.

## Review Loop

- Review pass 1 found one material coverage gap: the code closed direct
  `WorkItem` bypasses, but no external-facing source guard yet proved that the
  public API still forced planner-ready items through ingress. Added
  `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs` in scope.
- Review pass 2 reran the task review against code plus phase ledgers and found
  only planning-truth drift: the attack-surface, security, validation,
  roadmap, and state artifacts still implied `AS-20260609-001` was not yet
  closed. Fixed those records in scope.
- Review pass 3 reran the planner-boundary, canonical-path, and docs-truth
  audit after the fixes. No significant issues remained.
- Review pass 4 repeated the same audit. No significant issues remained again,
  giving the required consecutive clean closure.

## Closeout

- `AS-20260609-001` is closed with repository-backed code, tests, and docs.
- Phase 054 is fully complete through `054-08`; no active `054` execution lane
  remains.
