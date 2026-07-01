---
phase: 059-Core-Upgrade
plan: 059-09
status: complete
completed: 2026-06-18
owner: Z00Z Planning
---

# 059-09 Summary: Simulator Object Lanes And Alice Bob Charlie Evidence

## Scope Delivered

- Expanded the existing `scenario_1` executable home in place with one
  `object_flow_matrix` that covers Asset, Voucher, Right, fee-support, and
  cross-object interactions instead of introducing a second simulator lane.
- Added explicit Alice/Bob/Charlie evidence for 17 positive and 13 negative
  object-flow cases, including right-gated voucher actions, voucher lifecycle
  branches, cross-actor transfer continuity, and reject-code coverage.
- Synced `voucher_flow.json` into the release packet and contract tests so the
  pending public artifact inventory now stays aligned across runtime config,
  stage-surface expectations, and settlement verification.
- Wired a Phase 059 Stage 1 genesis fixture plus policy/voucher artifact
  invariants through the existing simulator packet and verification path.
- Localized the repeated Stage 13 recopy flow so mixed-fixture reruns reuse the
  same simulator path without reopening the RedB `Database already open`
  failure.
- Fixed the wallet-side `RedeemPartial` lifecycle mapping used by object RPC
  verdict checks and closed the release rename-guard blocker by normalizing the
  `generated_kani_*` fixture naming surface.

## Boundary Kept

- `scenario_1` remains the only executable simulator home for Phase 059.
- No parallel object packet, no second genesis authority, and no wallet-local
  verdict dialect were introduced.
- The simulator continues to reuse the existing Stage 1 through Stage 13 packet
  and the existing release/report artifacts instead of inventing a sidecar
  output path.

## Validation

- Mandatory bootstrap gate passed on the final code:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted simulator release validation passed:
  `cargo test -p z00z_simulator --release test_scenario1_object_flows -- --nocapture`
  `cargo test -p z00z_simulator --release test_scenario1_object_flows_reject_codes -- --nocapture`
  `cargo test -p z00z_simulator --release --test test_scenario_settlement -- --nocapture`
- Targeted wallet rename-guard validation passed after the generated fixture
  rename normalization:
  `cargo test -p z00z_wallets --release --test test_rename_guards`
- Broad workspace validation passed on a fresh rerun:
  `cargo test --release`
- Manual review against `.github/prompts/gsd-review-tasks-execution.prompt.md`
  closed the real issues found in this slice: packet inventory drift, direct
  filesystem or serde bypasses in the new test, repeated Stage 13 local-root
  lock collisions, the wallet `RedeemPartial` compile blocker, and the
  generated-fixture rename-guard blocker.
- Workspace-first doublecheck of the closeout claims confirmed the
  `object_flow_matrix` counts, Alice/Bob/Charlie coverage, `voucher_flow.json`
  packet sync, Stage 1 policy/voucher artifact assertions, localized Stage 13
  recopy reset, wallet `RedeemPartial` mapping, and final green release-mode
  gates against repository files and command outputs.

## Next Plan

Execution moves to `059-10-PLAN.md` for final cross-crate test closure, docs,
evidence-ledger sync, and Phase 059 closeout verification.
