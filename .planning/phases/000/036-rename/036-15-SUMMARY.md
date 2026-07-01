---
phase: 036-rename
plan: 15
status: completed
updated: 2026-04-18
---

# 036-15 Summary

## Scope

This file records the Wave 4 validation closure for `036-15-PLAN.md` on the
active `036-a2-legacy-removing-spec.md` -> `036-TODO-3.md` authority chain.

`036-15` closes the Rust-literal legacy-removal continuation truthfully: the
required deterministic gates reran after the final Wave 3 patch, the
authoritative substring-based residual scan is zero outside Tari, and the
canonical phase artifacts were updated to match that result.

## Outcome

The continuation slice is now closed on the repository-backed claim that there
are no remaining Rust `legacy|Legacy` substrings under `crates/**` outside the
protected Tari subtree.

This closeout is intentionally scoped to the Rust-literal removal target of the
`036-a2` continuation. It does not claim that every broader historical
compatibility concept across the repository is retired. It claims only what the
code and validation now prove:

- the authoritative Rust substring scan is zero outside Tari;
- the required deterministic gates reran green after the final cleanup patch;
- `036-14` and `036-15` are now summary-backed instead of being left on a
  false blocked narrative.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`:
  passed
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`:
  passed
- `rg -n "legacy|Legacy" crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'`:
  passed with zero matches

The earlier bounded closure pattern `rg -n "\\blegacy\\b|Legacy" ...` is
retired for this phase because it misses underscore-linked identifiers and can
produce a false zero-result.

## Review Loop

The review discipline from `.github/prompts/gsd-review-tasks-execution.prompt.md`
was applied in best-effort form because no direct local runner for
`/GSD-Review-Tasks-Execution` was exposed in this environment.

The closeout used three review passes:

1. rerun the deterministic gates after the final Wave 3 patch
2. rerun the authoritative substring scan and confirm zero residual Rust hits
3. re-read the canonical phase artifacts and repair every stale blocked-state
   claim that no longer matched the repository

The final two passes were clean: no new material findings were introduced after
the green reruns and zero-hit residual scan.

## Canonical Artifact Sync

The following artifacts were updated to match the live repository state:

- `.planning/phases/036-rename/036-a2-legacy-removing-spec.md`
- `.planning/phases/036-rename/036-TODO-3.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Current Boundary

`036-15` is now summary-backed complete. The Rust-literal legacy-removal slice
is closed through Wave 4, and the live execution pointer advances to
`036-16-PLAN.md` for the separate non-Tari `#[allow(dead_code)]` sweep.

Historical continuity note: `036-11-SUMMARY.md` and `036-12-SUMMARY.md` are
present in the same phase directory, but they no longer block the live pointer
for the now-closed `036-14` and `036-15` waves.
