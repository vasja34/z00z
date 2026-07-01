---
phase: 031-refactor-architecture
plan: "05"
requirements_completed:
  - PH31-WLT-SEAMS
status: completed
task_commits:
  - 8357c31c
  - dd9b36e6
  - a4f6cbeb
  - 06a4da15
review_surface_metrics:
  before_lines: 14
  after_lines: 109
  before_top_level_includes: 5
  after_top_level_includes: 0
---

# Phase 031 Plan 05: Wallet Service Seam Split Summary

Wallet service callers still enter through one stable shallow facade, but the canonical root no longer presents the service layer as a flat `include!`-assembled surface.

## Accomplishments

- Reworked [crates/z00z_wallets/src/services/wallet_service.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_service.rs) into an explicit seam map where the stable type lane stays in the root and the behavior-heavy action, session, store, and test lanes are wired through named internal modules.
- Promoted only crate-local helper seams needed by the split so the new module boundaries preserve the old behavior contract without widening the public `z00z_wallets::services` API.
- Updated [crates/z00z_wallets/tests/test_phase30_split.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_phase30_split.rs) to guard the new root seam map and to assert that [crates/z00z_wallets/src/services/mod.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/mod.rs) keeps reachability-only exports visibly demoted.
- Updated [crates/z00z_wallets/tests/test_wallet_service_errors.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_wallet_service_errors.rs) so the shallow `RateLimitPrecheck` facade remains covered alongside the existing error-surface test.

## Task Commits

| Task | Commit | Purpose |
| --- | --- | --- |
| Task 1 RED | `8357c31c` | Added the failing wallet service seam guard that rejects the legacy flat root assembly. |
| Task 1 GREEN | `dd9b36e6` | Split the wallet service root into explicit internal seams and preserved behavior through crate-local helper visibility. |
| Task 2 RED | `a4f6cbeb` | Added failing shallow-facade and reachability demotion guards. |
| Task 2 GREEN | `06a4da15` | Aligned `services/mod.rs` and the tests to the new shallow facade contract. |

## Review Surface And Compile-Time Notes

- Review clarity improved materially: the wallet service root changed from 14 lines with 5 top-level `include!` seams to 109 lines with an explicit seam map and 0 top-level `include!` matches under the plan guard.
- The split keeps the stable caller contract obvious: `WalletService` and `RateLimitPrecheck` remain the shallow facade, while `AddressUsedOracle` and `Sleeper` are now explicitly documented as reachability-only integration seams.
- No measurable compile-time regression was observed beyond normal warm-cache variance. The post-split targeted release guard compiles stabilized in the same general band as the surrounding reruns, with the source-shape guard and shallow-facade guard both completing in roughly 16-21 seconds on repeated release-mode invocations.
- The root split should remain the canonical pattern for future wallet service growth because it makes internal ownership visible without forcing downstream callers onto deep module imports.

## Verification

- `cargo test -p z00z_wallets --release --test test_phase30_split -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_wallet_service_errors -- --nocapture`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `rg -n '^include!\(' crates/z00z_wallets/src/services/wallet_service.rs`

## Deviations From Plan

### Auto-fixed Issues

1. [Rule 3 - Blocking issue] The explicit module split surfaced crate-local privacy edges that the old text `include!` assembly had been hiding.
   Fix: promoted only the required helper methods, constants, and internal types to `pub(crate)` across the wallet service seams.

2. [Rule 3 - Tooling substitution] The plan asked for repeated `/GSD-Review-Tasks-Execution` prompt runs, but that review prompt was not available in this executor session.
   Fix: replaced it with repeated RED or GREEN verification cycles using the source-shape guard, shallow-facade guard, targeted error-surface tests, and the bootstrap sanity suite.

## Deferred Issues

- [deferred-items.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/deferred-items.md) records the unrelated wallet integration blocker in `crates/z00z_wallets/tests/test_tx_assetpack.rs:5`, where the test still imports the stale `z00z_core::leaf::PackErr` path. This remained outside the wallet service seam split scope.

## Known Stubs

- None added by this plan.

## Threat Flags

- None.

## Self-Check

- PASSED
- Found summary artifact: `031-05-SUMMARY.md`
- Found task commits: `8357c31c`, `dd9b36e6`, `a4f6cbeb`, `06a4da15`
