# Phase 031 Verification Matrix

## Scope

This document is the Wave 4 verification artifact for Phase 031.
It ties final closeout claims to the Wave 0 inventory, the Wave 1 through Wave 3 summary-backed gates, and the final release-style validation runs required before Phase 031 may close.

## Wave Coverage

| Wave | Plans | Evidence | Result |
| --- | --- | --- | --- |
| W0 | `031-01` | [031-INVENTORY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-INVENTORY.md), [031-IMPORT-GRAPH.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-IMPORT-GRAPH.md), [031-01-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-01-SUMMARY.md) | PASS |
| W1 | `031-02`, `031-03`, `031-04` | [031-02-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-02-SUMMARY.md), [031-03-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-03-SUMMARY.md), [031-04-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-04-SUMMARY.md) | PASS |
| W2 | `031-05`, `031-06`, `031-07` | [031-05-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-05-SUMMARY.md), [031-06-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-06-SUMMARY.md), [031-07-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-07-SUMMARY.md) | PASS |
| W3 | `031-08`, `031-09` | [031-08-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-08-SUMMARY.md), [031-09-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-09-SUMMARY.md) | PASS |
| W4 | `031-10` | this file, [031-RETIREMENT.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-RETIREMENT.md), [031-10-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-10-SUMMARY.md), and the phase-local validation logs below | PASS |

## Gate Matrix

| Gate | Anchor | Evidence | Status |
| --- | --- | --- | --- |
| G-00 | Wave 0 import and caller proof | [031-INVENTORY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-INVENTORY.md), [031-IMPORT-GRAPH.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-IMPORT-GRAPH.md) | PASS |
| G-01 | Root export inventory | [031-02-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-02-SUMMARY.md) | PASS |
| G-02 | Tari leakage guard | [031-03-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-03-SUMMARY.md) | PASS |
| G-03 | Wallet service split | [031-05-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-05-SUMMARY.md) | PASS |
| G-04 | Wallet drift coverage | [031-06-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-06-SUMMARY.md) | PASS |
| G-05 | Wallet lock denial and edge ownership | [031-07-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-07-SUMMARY.md) | PASS |
| G-06 | Storage proof binding | [031-08-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-08-SUMMARY.md) | PASS |
| G-07 | Simulator deep-import guard | [031-09-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-09-SUMMARY.md) | PASS |
| G-08 | Simulator secret-output gate | [031-09-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-09-SUMMARY.md) | PASS |
| G-09 | Simulator reset sandbox | [031-09-SUMMARY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-09-SUMMARY.md) | PASS |
| G-10 | Planning closeout | `z00z_utils` README boundary note, this verification file, retirement evidence, synchronized planning truth, and final release-style validation | PASS |

## Final Validation Evidence

The required Phase 031 closeout gates were rerun after the Wave 3 closeout so the final planning sync is evidence-backed instead of inferred.

| Gate | Command | Evidence | Result | Notes |
| --- | --- | --- | --- | --- |
| V-01 | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` | [031-10-bootstrap.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-bootstrap.log) | PASS | Phase-local rerun ends with `=== BOOTSTRAP COMPLETE ===`. |
| V-02 | `cargo test -p z00z_utils --release -- --nocapture` | [031-10-z00z-utils-release.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-z00z-utils-release.log) | PASS | Unit, integration, and doc tests all completed green. |
| V-03 | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump -- --nocapture` | [031-10-wallet-release.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-wallet-release.log) | PASS | Targeted wallet release suite and wallet doctests completed green. |
| V-04 | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | [031-10-simulator-release.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-simulator-release.log) | PASS | Targeted simulator release suite completed green. |
| V-05 | `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run` | [031-10-full-verify-max-safe.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-full-verify-max-safe.log) | PASS | Canonical max-safe sweep ended with `[summary] planned=314 skipped=21 failed=0`. |
| V-06 | `phase planning grep` | [031-10-phase-grep.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-phase-grep.log) | PASS | The planning corpus still points at the Phase 031 retirement anchors and the final requirement rows that were pending before this closeout. |
| V-07 | `public facade grep` | [031-10-pub-surface-grep.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-pub-surface-grep.log) | PASS | No uncovered default-public `compat`, `legacy`, or `shim` re-export surface was found by the stricter public-facade grep. |
| V-08 | `compatibility name grep` | [031-10-compat-grep.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-compat-grep.log) | PASS WITH EXPLICIT EXCEPTIONS | Remaining hits are compatibility helpers, protocol version constants, or bounded live contracts reviewed in [031-RETIREMENT.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-RETIREMENT.md). |

## Additional Requested Evidence

The broad workspace release command was requested during closeout reconciliation even though `031-10-PLAN.md` does not list it as a formal gating command.

| Command | Evidence | Result | Notes |
| --- | --- | --- | --- |
| `cargo test --release --features test-fast --features wallet_debug_dump` | [031-10-workspace-release.log](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/logs/031-10-workspace-release.log) | PASS | Fresh rerun completed green, so the earlier broader-workspace concern is no longer a conflicting evidence source. |

Exact grep commands executed for V-06 through V-08:

- `rg -n "shim|suffix|retire|rollback|G-0|W0|W1|W2|W3|W4|PH31-" .planning/phases/031-refactor-architecture .planning/ROADMAP.md .planning/REQUIREMENTS.md -g '*.md'`
- `rg -n "pub use .*compat|pub use .*legacy|pub use .*shim|pub .*\b(V[0-9]+|v[0-9]+)\b" crates -g '*.rs'`
- `rg -n "\bcompat_|\blegacy_|\bshim_|\b(V[0-9]+|v[0-9]+)\b" crates -g '*.rs'`

## Review Gate

The plan requires `/.github/prompts/gsd-review-tasks-execution.prompt.md` in YOLO mode at least 3 times and only allows closure after 2 consecutive clean runs.
That prompt runner was unavailable in this executor environment, so Wave 4 closeout uses the exact release-style validation matrix above, the explicit grep guards, and the additional requested broad-workspace rerun as the recorded substitution.
