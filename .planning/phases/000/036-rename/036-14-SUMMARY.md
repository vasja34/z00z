---
phase: 036-rename
plan: 14
status: completed
updated: 2026-04-18
---

# 036-14 Summary

## Scope

This file records the Wave 3 delete-first residue cleanup for
`036-14-PLAN.md` on the active `036-a2-legacy-removing-spec.md` ->
`036-TODO-3.md` authority chain.

`036-14` closed the remaining helper-only Rust `legacy|Legacy` residue that
was still present after the earlier truth-restoration passes.

## Outcome

The earlier closure logic for this slice was incomplete because it relied on a
bounded scan that missed underscore-linked identifiers such as
`legacy_wallet`. The authoritative closure scan for this phase is the
substring-based command:

`rg -n "legacy|Legacy" crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'`

Using that scan, the remaining Wave 3 Rust residue was identified and deleted
without introducing rename-only replacement shims such as `compat_v1`,
`raw_v1`, or `backup_v1`.

The final cleanup removed the remaining `legacy|Legacy` substrings from these
Rust files:

- `crates/z00z_crypto/tests/test_hash_policy.rs`
- `crates/z00z_crypto/tests/test_fail_closed.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3_runtime.rs`
- `crates/z00z_simulator/tests/test_stage4_source_shape.rs`
- `crates/z00z_utils/src/compression/test_compression.rs`
- `crates/z00z_wallets/tests/test_redb_wlt_open.rs`
- `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs`
- `crates/z00z_wallets/tests/test_phase30_split.rs`
- `crates/z00z_wallets/tests/test_receiver_card_record.rs`

After that patch, the authoritative Rust substring scan returned no matches
outside the protected Tari subtree.

## Repository Changes

- The remaining helper-only and test-only residue from the wallet, crypto,
  simulator, and utility surfaces was deleted instead of being hidden behind
  compatibility-neutral renames.
- No Tari vendor files were touched.
- No new compatibility placeholders were introduced.

## Validation

- `rg -n "legacy|Legacy" crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'`:
  passed with zero matches after the patch
- `get_errors` across the edited Rust files: no diagnostics

`036-14` owns the code cleanup. The broader deterministic validation reruns and
the final closure proof are owned by `036-15` and recorded in
`036-15-SUMMARY.md`.

## Review Loop

The repo-local review contract was re-read from
`.github/prompts/gsd-review-tasks-execution.prompt.md` before closeout. The
direct `/GSD-Review-Tasks-Execution` runner was not exposed as a local CLI in
this environment, so review evidence for this wave used the best-effort path:
authoritative prompt reread, focused diff inspection, and then the shared
deterministic validation reruns captured by `036-15`.

## Current Boundary

`036-14` is now summary-backed complete. The live execution pointer advances to
`036-15-PLAN.md`, which owns the validation closure and canonical artifact sync
for the Rust zero-substring outcome.
