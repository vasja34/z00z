---
phase: 060-Gaps-Closing
plan: 060-02
status: complete
completed_at: 2026-06-20
next_plan: 060-03
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-02-PLAN.md
---

# 060-02 Summary: Canonical Bootstrap Authority Freeze

## Completed Scope

`060-02` is complete for the Phase 060 bootstrap-authority freeze slice.

The repository now tells one bootstrap story consistently across core and
settlement-facing docs: `z00z_core::genesis` is the only canonical bootstrap
authority, `GenesisConfig` is the canonical typed manifest, and
`src/assets/assets_config.yaml` is explicitly demoted to registry, example,
fixture, or compatibility data instead of an equal bootstrap source of truth.

Because Phase 060 treats future or target wording as live scope and requires
release validation when Rust or tests change, this slice also had to close the
stale dev-fixture test drift that broad validation exposed. The live dev
fixture caps coin serial ids at `< 20`, while several release tests and one
support example still encoded older high-serial assumptions. The final tree now
reads that cap through the canonical genesis helper instead of leaving hidden
`% 100` or `serial_id > 19` drift behind the new authority wording.

Broad release reruns also exposed a second consistency issue outside the first
wallet-focused drift pass: the shared Stage 13 test-support helper was
re-clearing and restabilizing the same root for every caller, which widened the
lock window and produced a release-only failure in the claimed-asset simulator
path. The final tree now keeps one canonical shared-root stabilization path in
test support, reuses the caller-local output home for `test_hjmt_e2e`, and
closes the last stale high-serial bridge fixture in `test_s5_spec6_bridge.rs`
through the same canonical genesis helper.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-02-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_core/README.md`
- `crates/z00z_core/src/assets/mod.rs`
- `crates/z00z_core/src/genesis/README.md`
- `crates/z00z_core/src/genesis/asset_std.rs`
- `crates/z00z_core/tests/assets/test_integration_assets_test4.rs`
- `crates/z00z_core/tests/assets/test_integration_assets_test5.rs`
- `crates/z00z_core/tests/assets/test_integration_assets_test25.rs`
- `crates/z00z_simulator/src/test_support/stage13_shared_cases.rs`
- `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`
- `crates/z00z_simulator/tests/test_hjmt_e2e.rs`
- `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs`
- `crates/z00z_storage/src/settlement/README.md`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/test_mod.rs`
- `crates/z00z_wallets/src/services/test_wallet_service.rs`
- `crates/z00z_wallets/tests/test_import_error_taxonomy.rs`
- `crates/z00z_wallets/tests/test_s5_sender_examples.rs`
- `crates/z00z_wallets/tests/test_s5_sender_examples_support.inc`
- `crates/z00z_wallets/tests/test_s5_spec6_bridge.rs`

## Boundary Kept

- No new config home, no second bootstrap authority, and no new runtime
  bootstrap path was introduced.
- Production bootstrap semantics did not move away from
  `z00z_core::genesis`; the added helper only exposes the already-live dev
  fixture serial cap so tests stop duplicating stale assumptions.
- The checked-in `assets_config.yaml` surface remains non-authoritative data;
  this slice did not promote it back into a bootstrap owner path.
- No parallel requirements layer or duplicate module or function authority was
  introduced.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 found one significant issue after the wording freeze: broad
  release-mode validation still contained stale dev-fixture serial assumptions
  above the live `< 20` cap, so the authority story was not yet self-consistent
  on the checked tree. The canonical `serials_from_dev_class(...)` helper plus
  targeted test fixes were added.
- Pass 2 reran the mandatory bootstrap gate and
  `cargo test --release -p z00z_wallets --lib`. That pass cleared the first
  wallet-facing high-serial drift, but the next broader simulator rerun exposed
  a second issue: shared Stage 13 restabilization under one lock widened the
  same-root contention window and broke `test_cross_stage_actors_claimed` in
  release mode.
- Pass 3 narrowed the shared Stage 13 helper so already-stable shared roots are
  reused without unconditional clear-and-restabilize, routed
  `test_hjmt_e2e` through a caller-local output home, reran the mandatory
  bootstrap gate, and reran
  `cargo test --release -p z00z_simulator --test test_pipeline_genesis_tx` plus
  `cargo test --release -p z00z_simulator --test test_hjmt_e2e`. Those gates
  passed, but the subsequent broad rerun still exposed one last stale
  high-serial bridge fixture in `test_s5_spec6_bridge.rs`.
- Pass 4 fixed that bridge fixture through `serials_from_dev_class(...)`,
  reran the mandatory bootstrap gate, reran
  `cargo test --release -p z00z_wallets --test test_s5_spec6_bridge`, and
  reran the full `cargo test --release` gate on the final tree. No significant
  issues remained.

Two consecutive clean review passes were achieved on passes 3 and 4.

## Validation

- Mandatory bootstrap gate passed before closeout:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `rg -n "single bootstrap authority|GenesisConfig|assets_config.yaml" crates/z00z_core/README.md crates/z00z_core/src/genesis/README.md crates/z00z_core/src/assets/mod.rs crates/z00z_storage/src/settlement/README.md`
  confirms the canonical bootstrap wording on the final tree.
- `cargo test --release -p z00z_wallets --lib` passed after the first
  validation-driven drift closure.
- `cargo test --release -p z00z_simulator --test test_pipeline_genesis_tx`
  passed after the shared Stage 13 stabilization-path fix.
- `cargo test --release -p z00z_simulator --test test_hjmt_e2e` passed after
  the caller-local output-home routing fix.
- `cargo test --release -p z00z_wallets --test test_s5_sender_examples` passed
  after the final example drift closure.
- `cargo test --release -p z00z_wallets --test test_s5_spec6_bridge` passed
  after the last bridge-fixture drift closure.
- `cargo test --release` passed on the final tree.
- `git diff --check` is clean for the files changed in this slice.

## Result

`060-02` is complete. Phase 060 advances to `060-03-PLAN.md` for the HJMT
process-model and YAML shard-mapping contract slice.
