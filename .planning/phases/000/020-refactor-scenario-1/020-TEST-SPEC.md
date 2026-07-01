---
phase: 020-refactor-scenario-1
artifact: test-spec
status: current
source: revised-plans-context-validation
updated: 2026-03-26
---

# Phase 020 Test Spec

## Purpose

This document defines the test surface for Phase 020 after the planning
revision that changed the phase from a structural-only helper refactor into an
explicit Scenario 1 stage split.

The current source of truth is the revised plan package:

- `020-CONTEXT.md`
- `020-VALIDATION.md`
- `020-01-PLAN.md`
- `020-02-PLAN.md`
- `020-03-PLAN.md`
- `020-04-PLAN.md`

This spec replaces the earlier fallback model and must be read as the current
test contract for Phase 020.

## Target Stage Map

Phase 020 targets one explicit 12-stage Scenario 1 map:

1. `genesis_init`
2. `wallet_create`
3. `claim_prepare`
4. `claim_publish`
5. `tx_plan`
6. `tx_prepare`
7. `transfer_receive`
8. `transfer_claim`
9. `bundle_build`
10. `bundle_publish`
11. `checkpoint_apply_storage`
12. `checkpoint_finalize`

The old coarse stages 3 through 6 are replaced by eight smaller homogeneous
stages.

## What The Tests Must Prove

Phase 020 tests must prove all of the following:

1. The runner and YAML files expose the explicit 12-stage map above.
2. The runner and YAML files expose one explicit stage-to-entrypoint map for
  final stages 3-12, so no implementation wave has to guess which current
  module lineage owns each new stage.
3. The macro flow still remains `claim -> prepare -> receive -> bundle -> apply -> finalize`.
4. The Stage 3 public helper and snapshot surface stays stable while the claim
   lane is split into `claim_prepare` and `claim_publish`.
5. The old Stage 4 body is split into `tx_plan` and `tx_prepare` without losing
   path remap, prep snapshot, or machine-readable continuity fields.
6. The old Stage 5 and Stage 6 lane is split into `transfer_receive`,
   `transfer_claim`, `bundle_build`, and `bundle_publish`.
7. Explicit Stage 6 reuse by later stages remains allowed when it reflects
   already materialized artifacts instead of recomputing prior-stage data.
8. Artifact names, JSON field names, wallet-visible meanings, proof binding,
   and storage-backed apply ownership remain stable.
9. Final checkpoint publication keeps its current two-outcome contract: stage
  12 remains `draft_only` when `proof_mode != opaque_test`, and emits final
  artifact or link or audit surfaces only when `proof_mode == opaque_test`.
10. The release-style Scenario 1 gate set remains green after the split.

## Scope Boundaries

### In Scope

- Source-shape tests that lock the 12-stage map and the new split surfaces.
- Integration tests that prove behavior preservation across the claim,
  tx-prepare, transfer, bundle, apply, and finalize lanes.
- Focused unit coverage for any new shared logging or path helpers only if they
  introduce branching or non-trivial remap logic.

### Out Of Scope

- Browser or UI automation.
- New wallet features.
- New artifact contracts.
- Forcing removal of Stage 6 reuse for architectural purity.
- Claiming closure of older backlog items not explicitly implemented by Phase
  020.

## Required Test Files

### 1. `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

This is the primary structural guardrail for the phase.

It must prove:

- the final stage count is exactly 12;
- the stage names match the target stage map in order;
- `runner.rs` and `scenario_design.yaml` agree on stage numbering and names;
- `runner.rs` and `scenario_design.yaml` agree on the exact final `rust_entry`
  mapping chosen for stages 3-12;
- the macro flow still groups into claim, prepare, receive, bundle, apply, and
  finalize lanes;
- the file no longer encodes the old fixed 8-stage assumption.

Recommended assertions:

- `runner_has_exact_12_stage_map`
- `design_yaml_matches_12_stage_map`
- `rust_entry_map_matches_final_stage_owners`
- `macro_flow_order_is_stable`

### 2. `crates/z00z_simulator/tests/test_claim_acceptance.rs`

Keep this as the anchor for claim-lane behavior preservation.

It must continue to prove:

- Stage 3 helpers still route through canonical APIs;
- claim publication outputs remain deterministic;
- the split into `claim_prepare` and `claim_publish` does not narrow the
  currently used Stage 3 helper or snapshot surface.
- the extracted helper seams in `stage_3_utils/claim_pkg.rs` and
  `stage_3_utils/audit.rs` remain implementation detail only, with the
  existing Stage 3 public helper surface preserved.

### 3-7. Additional Claim Anchors

- `crates/z00z_simulator/tests/test_claim_emit.rs`
- `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`
- `crates/z00z_simulator/tests/test_claim_snapshot.rs`
- `crates/z00z_simulator/tests/test_claim_persist.rs`
- `crates/z00z_simulator/tests/test_claim_integration.rs`

These remain the fine-grained guards around the split claim lane.

They must continue to pass after the lane becomes stages 3 and 4.

### 8. `crates/z00z_simulator/tests/test_stage4_source_shape.rs`

This file now guards the split of the old Stage 4 body into:

- stage 5 `tx_plan`
- stage 6 `tx_prepare`

It must prove:

- the runner and YAML no longer model tx-prepare as one coarse stage;
- `stage_4.rs` is a compact facade over explicit helper seams;
- Stage 4 remains simulator-local and does not reimplement canonical wallet or
  storage logic.

### 9-11. Stage 4 Continuity Anchors

- `crates/z00z_simulator/tests/test_stage4_cfg_paths.rs`
- `crates/z00z_simulator/tests/test_stage4_chain_path.rs`
- `crates/z00z_simulator/tests/test_pipeline_genesis_tx.rs`

These remain the continuity anchors for the split tx-prepare lane.

They must continue to prove:

- path remap stability;
- prep snapshot continuity;
- machine-readable `claim_root_hex`, `prep_root_hex`, `post_apply_root_hex`,
  and downstream continuity fields;
- pipeline continuity from claim into tx-prepare.

Current release closure set for `020-02`:

- `test_stage4_source_shape`
- `test_stage4_tamper`
- `test_stage4_cfg_paths`
- `test_stage4_chain_path`
- `test_pipeline_genesis_tx`

### 12. `crates/z00z_simulator/tests/test_stage5_source_shape.rs`

This file now guards the split of the old Stage 5 and Stage 6 lane into:

- stage 7 `transfer_receive`
- stage 8 `transfer_claim`
- stage 9 `bundle_build`
- stage 10 `bundle_publish`

It must prove:

- the transfer lane is no longer represented as two coarse stages;
- Stage 5 and Stage 6 internals are split behind explicit helper seams;
- any Stage 6 artifact or helper reuse consumed later is explicit rather than
  accidental.

Implementation note from `020-03` closure:

- The split was closed through `crates/z00z_simulator/src/scenario_1/stage_5_utils/mod.rs`
  and `crates/z00z_simulator/src/scenario_1/stage_6_utils/mod.rs`.
- The wider helper filename fan-out proposed in `020-03-PLAN.md` remained
  aspirational and was not materialized where it would only duplicate thin
  wrappers.

### 13-18. Transfer, Apply, And Finalization Anchors

- `crates/z00z_simulator/tests/test_stage5_receive_bridge.rs`
- `crates/z00z_simulator/tests/test_stage6_checkpoint_storage_bridge.rs`
- `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`
- `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`
- `crates/z00z_simulator/tests/test_stage8_proof_path.rs`
- `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`

These remain the behavior-preserving anchors for the transfer, bundle, apply,
and finalize lanes.

Filename continuity note:

- `test_stage7_jmt_wallet_scan.rs` and `test_stage8_proof_path.rs` are named
  after the current module lineage in the codebase.
- After the stage-map expansion lands, these files must either keep their
  current filenames as lineage anchors or be renamed in the same wave as the
  runner and YAML changes.
- In either case, their assertions must target final stage 11
  `checkpoint_apply_storage` and final stage 12 `checkpoint_finalize`, not the
  pre-refactor numeric ids 7 and 8.

They must continue to prove:

- canonical/runtime receive parity;
- report-first behavior;
- draft/checkpoint continuity;
- `checkpoint_apply_storage` ownership after renumbering to final stage 11;
- `checkpoint_finalize` proof-bound publication after renumbering to final
  stage 12, including both `draft_only` and finalized outputs;
- unified end-to-end acceptance.

Current release closure set for `020-03`:

- `test_scenario1_stage_surface`
- `test_stage5_source_shape`
- `test_stage5_receive_bridge`
- `test_stage6_checkpoint_storage_bridge`
- `test_stage6_checkpoint_final_gate`
- `test_stage7_jmt_wallet_scan`
- `test_stage8_proof_path`
- `test_scenario1_unified_gate`

Current release closure set for `020-04`:

- `test_scenario1_stage_surface`
- `test_stage4_cfg_paths`
- `test_stage4_chain_path`
- `test_claim_acceptance`
- `test_stage5_receive_bridge`
- `test_stage6_checkpoint_storage_bridge`
- `test_stage6_checkpoint_final_gate`
- `test_stage7_jmt_wallet_scan`
- `test_stage8_proof_path`
- `test_pipeline_genesis_tx`
- `test_scenario1_unified_gate`
- full `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
- release `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`

Closure note:

- Legacy stage4-oriented tests now treat tx-plan rejection as final stage `5`, not the pre-split stage `4` or `6` assumptions.
- `stage_4_snapshot.json` is an allowed upstream `claim_publish` artifact on later tx-plan failures and is no longer treated as a post-tx artifact that must disappear.
- The existing `test_stage7_jmt_wallet_scan.rs` and `test_stage8_proof_path.rs` lineage-anchor filenames remain valid, but their assertions now close on final stage `11` and `12` behavior only.

## Conditional New Unit Tests

Only add extra unit tests if new shared helpers contain real branching.

Possible files:

- `crates/z00z_simulator/tests/test_scenario_paths.rs`
- `crates/z00z_simulator/tests/test_scenario_logging.rs`

Create them only if `scenario_paths.rs` or `scenario_logging.rs` gain
non-trivial behavior such as path remap decisions, output selection, or log
row normalization. If those modules remain thin wrappers, integration coverage
is enough.

## Requirement To Test Mapping

| Requirement | What Must Hold | Tests |
| --- | --- | --- |
| `SCN1-03` | Storage-backed transfer execution remains canonical across the new stages 7-12 | `test_stage5_receive_bridge`, `test_stage6_checkpoint_storage_bridge`, `test_stage6_checkpoint_final_gate`, `test_scenario1_unified_gate` |
| `SCN1-04` | Canonical root/path continuity survives the new stages 3-10 | `test_scenario1_stage_surface`, `test_claim_acceptance`, `test_stage4_source_shape`, `test_stage4_cfg_paths`, `test_stage4_chain_path`, `test_pipeline_genesis_tx` |
| `SCN1-05` | Final checkpoint publication stays draft/final separated and proof-bound | `test_stage6_checkpoint_final_gate`, `test_stage8_proof_path`, `test_scenario1_unified_gate` |

## Wave 0 Implemented Artifacts

These tests now exist in the workspace and form the structural guardrail set
for the later Phase 020 waves:

- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/test_stage4_source_shape.rs`
- `crates/z00z_simulator/tests/test_stage5_source_shape.rs`

## Release Commands

Use the simulator commands already standardized in the workspace:

```bash
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_source_shape -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_tamper -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_cfg_paths -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_chain_path -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_pipeline_genesis_tx -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage5_receive_bridge -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_storage_bridge -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage7_jmt_wallet_scan -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage8_proof_path -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump
```

## Acceptance Summary

Phase 020 test coverage is complete only when:

1. the 12-stage map is explicit and locked by tests;
2. the split claim, tx-prepare, transfer, and bundle lanes preserve the same
   artifact and wallet-visible semantics as before;
3. explicit Stage 6 reuse is tested as allowed behavior when artifact-led;
4. the full release-style Scenario 1 gate set remains green.
