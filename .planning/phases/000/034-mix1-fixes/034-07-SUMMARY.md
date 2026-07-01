---
phase: 034-mix1-fixes
plan: 07
subsystem: documentation-allowlist-and-wording-guards
tags: [doc-allowlist, wording-guards, simulator, storage, review-loop, honest-closure]
requires:
  - phase: 034-06
    provides: green spend and checkpoint semantic validation waves required before live wording reclassification
provides:
  - Honest active-documentation reclassification for the closed Phase 034 seams
  - Stage-surface and storage wording guards aligned to the implemented live wording
  - Review-backed evidence that the Plan 07 wording-guard deltas are clean after fixup
affects: [034-07, PH34-DOC-ALLOWLIST, Q47]
tech-stack:
  added: []
  patterns: [active-doc reclassification after green closure, wrap-tolerant wording guards, scoped clean review loop]
key-files:
  created:
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-07-SUMMARY.md
  modified:
    - /home/vadim/Projects/z00z/.planning/REQUIREMENTS.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-CONTEXT.md
    - /home/vadim/Projects/z00z/.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md
    - /home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/stage_12.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/tx/state_checkpoint.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs
    - /home/vadim/Projects/z00z/crates/z00z_simulator/tests/test_scenario1_stage_surface.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_scenario1_semantics.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/tests/test_redb_rehydrate.rs
key-decisions:
  - "Active requirements and planning docs may describe the implemented storage-backed claim continuity, deterministic spend-nullifier closure, core::stealth sender authority, and backend-defined package-coupled checkpoint acceptance only after the exact semantic waves are already green."
  - "Wording guards should be resilient to line wrapping and benign reflow, but must still preserve polarity and avoid self-referential tautologies."
  - "Plan-owned review closure is satisfied only after two consecutive clean scoped review passes on the modified regions."
patterns-established:
  - "When live wording changes, broaden verification beyond the initially targeted binaries because cross-crate wording guards can fail later in simulator or storage suites."
requirements-completed: [034-07]
completed: 2026-04-10
reviewed: 2026-04-10T10:51:54+00:00
---

# Phase 034 Plan 07 Summary

## Outcome

Plan 07 is complete. The active documentation allowlist is now honestly
reclassified to the implemented Phase 034 truth, and the simulator plus storage
wording-guard suites were tightened so they stay specific while tolerating live
line wrapping and wording layout changes.

## Accomplishments

- Updated active requirements and phase context so the closed Phase 034 seams
  now describe the implemented truth instead of the earlier pre-closure
  blocker wording.
- Updated active forward-looking planning references so they no longer point
  future sender-construction work at `builder.rs` or `output_flow.rs` as
  canonical public authority.
- Updated live code wording on the checkpoint-facing seams so the package-coupled
  acceptance boundary is described honestly without overclaiming standalone
  backend or final trustless closure.
- Fixed simulator wording guards after the active wording changes invalidated
  brittle exact-phrase assertions.
- Fixed storage wording guards after the broader workspace rerun exposed the
  same class of brittle source-text assertions on checkpoint wording.
- Completed the required review loop with two consecutive clean scoped review
  passes on the modified Plan 07 regions.

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_runtime -- --nocapture` passed with 14 tests.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture` passed after the wording-guard fixes with 28 tests.
- `cargo test -p z00z_storage --release --features test-fast --test test_redb_rehydrate -- --nocapture` passed after the storage wording-guard fixes with 15 tests.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` passed on the current tree.
- `cargo test --release --features test-fast --features wallet_debug_dump` reran cleanly on the current tree during the Plan 07 verification sweep.
- Two consecutive scoped review passes over the modified Plan 07 regions returned `no findings`.

## Issues Encountered

- The initial simulator wording-guard suite still depended on exact long phrases
  that no longer matched the honest reclassified wording after line wrapping and
  phrasing updates.
- The broader workspace release rerun surfaced the same issue in the storage
  wording-guard suite, including one self-referential test-file tautology.
- One storage proof-tamper assertion had to be aligned to the actual generic
  load-surface mismatch message rather than the deeper backend detail string.

## Next Phase Readiness

- Plan 08 can now perform the closure-proof sweep against active artifacts that
  already reflect the implemented Phase 034 truth.
- Historical append-only audit artifacts remain untouched, so the next sweep can
  distinguish active truth from preserved historical evidence without reopening
  Plan 07 wording work.

## Threat Flags

None for the Plan 07 scope after the final review-backed wording-guard fixes.

## Self-Check

PASSED on bootstrap, exact Plan 07 binaries, simulator release gate, broader
workspace release rerun, and the required two-pass scoped review loop.

---
*Phase: 034-mix1-fixes*
*Completed: 2026-04-10*
