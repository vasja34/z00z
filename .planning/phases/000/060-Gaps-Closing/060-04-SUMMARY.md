---
phase: 060-Gaps-Closing
plan: 060-04
status: complete
completed_at: 2026-06-20
next_plan: 060-05
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-04-PLAN.md
---

# 060-04 Summary: Rights Owner Move, Shim Demotion, And Dual-Authority YAML Closure

## Completed Scope

`060-04` is complete for the Phase 060 rights-owner and authority-closure
slice.

The repository now tells one owner-path story for rights config. Canonical
rights-config ownership moved under `crates/z00z_core/src/rights/config.rs`,
where `RightClassConfig`, `RightsConfigEntry`, forbidden-key validation,
`parse_rights_from_yaml(...)`, and `load_rights_from_yaml(...)` now live
together. `crates/z00z_core/src/rights/mod.rs` exports that owner path
directly, while `crates/z00z_core/src/assets/right_config.rs` was reduced to a
documented compatibility shim for legacy imports instead of remaining the
semantic owner.

This slice also closed the lingering dual-authority story around
`assets_config.yaml`. `crates/z00z_core/src/assets/assets_config_load.rs` now
delegates rights loading to the canonical `crate::rights` owner module, and the
mixed `src/assets/assets_config.yaml` file plus the core/genesis READMEs now
state explicitly that the file is registry/example/compatibility fixture data
rather than equal bootstrap authority with `GenesisConfig`. The runtime
bootstrap story stays singular: `z00z_core::genesis` remains the only canonical
live bootstrap authority.

Finally, the shim-demotion work was carried through internal imports and tests.
The `assets::{action_pool,policy_descriptor,voucher_config}` facades are now
documented as compatibility-only surfaces, internal repository users that still
sourced rights types through `assets` were moved onto `z00z_core::rights`, and
a regression guard now asserts that `rights/mod.rs` keeps local config
ownership while `assets/right_config.rs` stays only a compatibility shim.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-04-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_core/README.md`
- `crates/z00z_core/src/assets/action_pool.rs`
- `crates/z00z_core/src/assets/assets_config.yaml`
- `crates/z00z_core/src/assets/assets_config_load.rs`
- `crates/z00z_core/src/assets/mod.rs`
- `crates/z00z_core/src/assets/policy_descriptor.rs`
- `crates/z00z_core/src/assets/right_config.rs`
- `crates/z00z_core/src/assets/voucher_config.rs`
- `crates/z00z_core/src/genesis/README.md`
- `crates/z00z_core/src/genesis/genesis_config.rs`
- `crates/z00z_core/src/genesis/genesis_config_validate.rs`
- `crates/z00z_core/src/genesis/genesis_rights.rs`
- `crates/z00z_core/src/genesis/test_genesis_suite.rs`
- `crates/z00z_core/src/genesis/test_validator_suite.rs`
- `crates/z00z_core/src/rights/config.rs`
- `crates/z00z_core/src/rights/mod.rs`
- `crates/z00z_core/src/rights/test_rights_config.rs`
- `crates/z00z_core/tests/assets/test_rights_config.rs`
- `crates/z00z_core/tests/genesis/test_helpers.rs`
- `crates/z00z_core/tests/genesis/test_settlement_corpus.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/storage.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/storage_view.rs`
- `crates/z00z_storage/tests/test_genesis_ingestion.rs`

## Boundary Kept

- No symmetry-only `actions_config.yaml`, `policies_config.yaml`,
  `vouchers_config.yaml`, or `rights_config.yaml` file was introduced.
- `crates/z00z_utils/src/codec/canonical_json.rs`,
  `crates/z00z_core/src/genesis/genesis_policies.rs`, and
  `crates/z00z_storage/src/settlement/leaf.rs` were left untouched, so this
  slice did not reopen the canonical-json, policy-home, or storage-owner
  boundary debates.
- The `assets/*` facades remain available only as compatibility shims; the
  slice did not create a second live owner module or a second bootstrap path.
- `assets_config.yaml` survived only as mixed registry/fixture data with honest
  wording. It no longer claims equal bootstrap authority with `GenesisConfig`.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 found residual internal owner drift: several repository files still
  imported `RightClassConfig` through `assets`, and the rights-fixture test
  still called `assets_config.yaml` canonical authority. Those imports and
  messages were moved to the `rights` owner path and compatibility-fixture
  wording.
- Pass 2 found a leftover internal tail during the mandatory bootstrap rerun:
  `assets/right_config.rs` still reexported
  `parse_rights_from_yaml(...)` privately, which had become dead after the
  owner move. That reexport was removed, and the bootstrap gate was rerun from
  the start.
- Pass 3 reran the targeted release anchors and grep-backed owner-path audit.
  Surviving `assets::right_config` hits are now only the compatibility shim
  itself plus the regression guard that explicitly forbids owner-path drift.
- Pass 4 reran the full `cargo test --release` workspace gate, rechecked the
  no-symmetry-file and untouched-boundary constraints, and reran
  `git diff --check` on the changed slice files. No significant issues
  remained.

Two consecutive clean review passes were achieved on passes 3 and 4.

## Validation

- Mandatory bootstrap gate passed on the final tree:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_core --release --features deterministic-rng test_genesis_manifest_phase059_fixture -- --nocapture`
  passed.
- `cargo test -p z00z_core --release test_policy_descriptor -- --nocapture`
  passed.
- `cargo test -p z00z_core --release test_voucher_config -- --nocapture`
  passed.
- `cargo test -p z00z_core --release test_rights_config -- --nocapture`
  passed.
- `rg -n "crate::assets::right_config|assets::right_config|compatibility" crates/z00z_core`
  confirms that surviving `assets::right_config` hits are compatibility-only.
- `rg -n "assets::\\{[^\\n]*RightClassConfig|assets::RightsConfigEntry|assets::RightClassConfig|assets::load_rights_from_yaml|crate::assets::right_config|canonical assets config" .`
  confirms there are no remaining internal semantic-owner imports through
  `assets` and no stale `canonical assets config` wording.
- `rg -n "actions_config.yaml|policies_config.yaml|vouchers_config.yaml|rights_config.yaml" crates .planning/phases/060-Gaps-Closing`
  shows only planning/docs mentions and no new repository authority files.
- `git diff --name-only -- crates/z00z_utils/src/codec/canonical_json.rs crates/z00z_core/src/genesis/genesis_policies.rs crates/z00z_storage/src/settlement/leaf.rs`
  returned no output, confirming those guarded files were untouched.
- `cargo test --release` passed on the final tree.
- `git diff --check` is clean for the files changed in this slice.

## Result

`060-04` is complete. Phase 060 advances to `060-05-PLAN.md` for the HJMT
decommission coverage and `3A7S -> 2A7S -> 5A7S` scenario slice.
