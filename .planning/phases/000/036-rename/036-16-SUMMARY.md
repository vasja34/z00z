---
phase: 036-rename
plan: 16
status: completed
updated: 2026-04-18
---

# 036-16 Summary

## Scope

This file records the Wave 6 cleanup and closure for `036-16-PLAN.md` on the
active `036-a2-legacy-removing-spec.md` -> `036-TODO-3.md` authority chain.

`036-16` closes the non-Tari `#[allow(dead_code)]` sweep truthfully: the frozen
inventory was revalidated before edits, the remaining warnings were eliminated
by honest deletion or boundary narrowing, and the final wallets compile now
finishes without dead-code warnings outside Tari.

## Outcome

The frozen inventory from `036-TODO-3.md` was consumed without drift. The last
live warning surfaces were resolved by:

- removing the dead `WalletIo::atomic_write_file_private` trait method and its
  test-only implementer copies,
- keeping `WalletKvTable::store_name` available only where the backend boundary
  actually needs it,
- deleting the dead `ZERO_ROOT` placeholder constants from
  `crates/z00z_simulator/src/scenario_1/stage_3.rs` and
  `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`, while keeping
  the remaining wallet support constant live through
  `crates/z00z_wallets/tests/test_s5_sender_examples.rs`.

This closeout is intentionally scoped to the dead_code-attribute sweep. It does
not claim broader legacy deletion or version-rename work. It claims only what
the code and validation now prove:

- the frozen non-Tari inventory was handled without touching Tari vendor code;
- the remaining dead-code warnings were resolved honestly rather than
  suppressed;
- `cargo test --release --features test-fast --features wallet_debug_dump --no-run`
  reran green with no warnings after the final fix.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
  earlier in the sweep
- `cargo test --release --features test-fast --features wallet_debug_dump --no-run`:
  passed with zero warnings after the final fix
- `get_errors` on the edited wallets files: no diagnostics
- `rg -n "#\\[allow\\(dead_code\\)\\]" crates --glob '*.rs' --glob
  '!crates/z00z_crypto/tari/**'`: passed with no matches outside Tari

## Review Loop

The review discipline from `.github/prompts/gsd-review-tasks-execution.prompt.md`
was kept in view, but no direct local runner for
`/GSD-Review-Tasks-Execution` was exposed in this environment. Closure therefore
relied on the deterministic gates, live compile check, and final zero-grep
proof.

## Canonical Artifact Sync

The following artifacts were updated to match the live repository state:

- `.planning/phases/036-rename/036-TODO-3.md`
- `.planning/STATE.md`

## Current Boundary

`036-16` is now summary-backed complete. The live execution pointer advances to
`036-17-PLAN.md` for the next non-Tari rename slice.
