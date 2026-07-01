---
phase: 036-rename
plan: 13
status: truth-restored
updated: 2026-04-18
---

# 036-13 Summary

## Scope

This file records the execution and validation state for `036-13-PLAN.md`,
covering the first Wave 2 production delete subset on the active
`036-a2-legacy-removing-spec.md` -> `036-TODO-3.md` authority chain.
The storage subset landed, the validation blocker was resolved truthfully, and
the mandatory broad release gate is now green.

## Outcome

Plan 13 no longer carries a false-closeout claim.

The landed `036-13` diff stayed narrow, but it did not perform delete-first
removal. It only renamed still-live storage compatibility helpers and locals to
compatibility-neutral names while preserving the same persisted bytes, decode
fallbacks, proof-system values, link upgrade behavior, and metadata contracts.
On 2026-04-18 that false-closeout state was repaired by restoring explicit
legacy naming for the still-live storage compatibility lanes, so the plan now
records those owners truthfully as blocker-carry-forward residue rather than as
completed deletion.

During validation, the mandatory broad release gate first exposed stale wallet
stealth KDF fixture expectations in
`crates/z00z_wallets/tests/fixtures/stealth_kdf_vectors.yaml`. That blocker sat
outside the authorized `036-13` production subset and did not require any
change to the landed storage code. After the fixture was synchronized to the
already-live wallet derivation outputs, the targeted wallet test, the required
bootstrap gate, and the full release-style workspace gate all passed.

## Repository Changes

- `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`,
  `crates/z00z_storage/src/checkpoint/codec.rs`,
  `crates/z00z_storage/src/checkpoint/link.rs`, and
  `crates/z00z_storage/src/checkpoint/ids.rs` now carry explicit legacy-compat
  naming for the still-live storage compatibility lanes so the repo no longer
  treats rename-only masking as completed deletion.
- `crates/z00z_wallets/tests/fixtures/stealth_kdf_vectors.yaml` was refreshed
  during validation so the broad release gate matches the already-live wallet
  stealth derivation outputs.
- `.planning/phases/036-rename/036-a2-legacy-removing-spec.md` and
  `.planning/phases/036-rename/036-TODO-3.md` now record that this exact
  storage-only subset landed in `036-13`, while the artifact proof-system rows
  and every wallet or core compatibility row remain blocked.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_stealth_kdf_vectors`:
  passed
- `cargo test -p z00z_storage --release --features test-fast --test test_checkpoint_store_api --test test_checkpoint_finalization --test test_checkpoint_link_injective --test test_redb_rehydrate`:
  passed
- `rg -n "legacy_snap_id|check_artifact_legacy_compat|decode_link_legacy_compat_bin|encode_link_legacy_compat_bin" crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs crates/z00z_storage/src/checkpoint/codec.rs crates/z00z_storage/src/checkpoint/link.rs crates/z00z_storage/src/checkpoint/ids.rs`:
  passed with the expected explicit legacy-compat matches only

## Review Loop

The review loop completed in three passes and cleared the false-closeout state:

1. review pass 1 rechecked the Wave 1 authorization boundary and confirmed
  that only the three storage files were in scope for `036-13`
2. the landed storage diff was checked against the actual code paths to ensure
  that only private helper names changed and that no encoded value, persisted
  marker, proof-system constant, or transport literal moved
3. the final pass restored explicit legacy-compat naming in the still-live
  storage compatibility helpers, reran the required bootstrap gate, reran the
  targeted storage suite, and reran the mandatory broad release gate green
  while the narrow `036-13` file set stayed behavior-identical

As with the earlier continuation plans, the exact runtime entrypoint for
`/GSD-Review-Tasks-Execution` was not exposed as a direct CLI in this
environment, so the review evidence used the repo-local best-effort path:
planning-authority rereads, narrow-scope diff review, deterministic test gates,
and repeated in-scope review passes in YOLO mode.

## Current Boundary

`036-13` is no longer the live false-closeout blocker. The production scope
stays limited to the storage subset, but its current state is now a truthful
blocker-carry-forward record under explicit legacy naming. The live execution
pointer moves to `036-14-PLAN.md`, while `036-15-PLAN.md` remains blocked until
the missing Wave 3 summary continuity is repaired.
