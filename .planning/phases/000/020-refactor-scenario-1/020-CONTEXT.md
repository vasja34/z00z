<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD047 -->
# Phase 020: 020-refactor-scenario-1 - Context

**Gathered:** 2026-03-25
**Status:** Revised after user clarification

<domain>
## Phase Boundary

Phase 020 is not limited to a structural-only helper split. The phase may
change the Scenario 1 stage map to break oversized stages into smaller
homogeneous execution stages, provided the canonical artifact lane and
wallet-visible behavior remain stable.

The split priority is explicit: first the current Stage 3 and Stage 4 lane,
then the current Stage 5 and Stage 6 lane. The goal is not micro-stages. The
goal is a compact sequence of stages where each stage owns one consistent kind
of logic.

### Target Stage Map After Refactor

Phase 020 now targets one explicit 12-stage Scenario 1 map:

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

The split replaces the old coarse stages 3 through 6 with two claim stages,
two tx-prepare stages, two transfer stages, and two bundle stages.

This phase includes deeper synchronized rewrites of
`scenario_config.yaml` and `scenario_design.yaml`, broader scenario_1 cleanup
for shared logging and path helpers, and structure-focused regression coverage
for the new stage surface.

This phase does not add new business capabilities, new wallet semantics, or a
second storage or checkpoint lane. Functional backlog items from earlier todo
ledgers remain out of scope unless a refactor wave proves that one is already
part of the touched surface.

### Gap Coverage Boundary

- **Closed directly by Phase 020:** splitting oversized stages into smaller
  homogeneous stages, aligning Stage 3 and Stage 4 first, then Stage 5 and
  Stage 6, deeper YAML synchronization, and broader scenario_1 cleanup for
  shared logging and path helpers.
- **Allowed in Phase 020:** preserving or formalizing reverse Stage 6 reuse by
  later stages when it is artifact-led and avoids rebuilding prior-stage data
  from scratch.
- **Not closed directly by Phase 020:** upstream nullifier semantics, broader
  receive taxonomy expansion, backup feature work, or unrelated wallet debt.

</domain>

<decisions>
## Implementation Decisions

### Stage restructuring
- **D-01:** Phase 020 may expand or reorder the Scenario 1 stage map if that is
  required to split oversized stages into smaller homogeneous execution stages.
- **D-01A:** The working target is a 12-stage map with stages 3-10 renamed and
  split as `claim_prepare`, `claim_publish`, `tx_plan`, `tx_prepare`,
  `transfer_receive`, `transfer_claim`, `bundle_build`, and
  `bundle_publish`.
- **D-02:** The execution order for the refactor is fixed as current Stage 3 and
  Stage 4 first, then current Stage 5 and Stage 6.
- **D-03:** The existing `stage_3_utils` and `stage_4_utils` pattern is the
  alignment baseline and should be normalized before Stage 5 and Stage 6 move.
- **D-04:** Stable public module paths for the stage entry functions must remain
  usable for the runner and tests even if the stage map expands.

### Stage 6 reuse policy
- **D-05:** Reverse Stage 6 reuse by Stage 7 or Stage 8 is acceptable when it
  consumes already materialized Stage 6 artifacts or helper logic instead of
  rebuilding the same data from zero.
- **D-06:** Phase 020 must not spend effort removing reverse Stage 6 reuse only
  for architectural purity.
- **D-07:** If a Stage 6 helper surface is kept for later stages, it must be
  made explicit and stable rather than remaining an accidental side effect of a
  large facade file.

### YAML synchronization
- **D-08:** `scenario_config.yaml` and `scenario_design.yaml` are first-class
  refactor targets and may be rewritten more deeply to match the new internal
  modular and stage structure.
- **D-09:** YAML changes must move in the same wave as the Rust stage split they
  describe so code and scenario metadata do not drift.

### Cleanup scope
- **D-10:** Broader cleanup around the direct split is allowed when it unifies
  logging or path helpers across `scenario_1` and reduces repetition.
- **D-11:** The cleanup may extend beyond the four monolithic stage files when
  the touched files are part of the same execution lane.

### Behavior and contract preservation
- **D-12:** Canonical artifact semantics and wallet-visible behavior remain
  locked even when stage numbering or stage boundaries change.
- **D-13:** The storage-backed `resolve -> verify -> apply` ownership boundary,
  checkpoint publication semantics, and proof-bearing artifact lane established
  by phases 017 through 019 remain binding.
- **D-14:** The refactor must not introduce a second storage model, alternate
  wallet proof path, or simulator-local replacement for canonical checkpoint or
  claim behavior.

### the agent's Discretion
- Exact stage count and stage numbering after the split, provided the result is
  monotonic, compact, and each new stage owns one homogeneous responsibility.
- Exact shared helper file names for broader `scenario_1` logging or path
  cleanup.
- Exact internal split of Stage 5 and Stage 6 helpers once Stage 3 and Stage 4
  alignment is in place.

</decisions>

<specifics>
## Specific Ideas

- The user wants one large stage to be split into several smaller but still
  meaningful stages, not into micro-stages.
- The user explicitly corrected the earlier assumption that Phase 020 must keep
  the old stage ordering unchanged.
- The user explicitly chose Stage 3 and Stage 4 first, then Stage 5 and Stage
  6.
- The user explicitly allowed reverse Stage 6 reuse by later stages where that
  reuse reflects previously generated artifacts.
- The user explicitly wants deeper YAML rewrites that follow the new internal
  split.
- The user explicitly widened cleanup scope to include broader scenario_1
  logging and path helper unification.

## Execution Spine

1. Audit the current runner surface, YAML stage definitions, and Stage 3 or 4
   internal boundaries.
2. Introduce the new claim-lane stage split in `runner.rs`,
   `scenario_design.yaml`, and `scenario_config.yaml`, then align Stage 3.
3. Extend the same stage-splitting discipline to the current Stage 4 tx-prepare
   lane and preserve machine-readable continuity artifacts.
4. After Stage 3 and Stage 4 are aligned, split the current Stage 5 and Stage 6
   lane and formalize any kept Stage 6 reuse surface instead of treating it as
   a defect.
5. In the same later wave, widen scenario_1 cleanup for shared logging and path
   helpers where it reduces duplication across the refactored lane.
6. Close the phase only after the expanded stage map, YAML, and release-style
   simulator gates all agree.

## Expected Outputs

- An expanded or reordered Scenario 1 stage map with smaller homogeneous
  execution stages.
- One explicit 12-stage Scenario 1 map replacing the old 8-stage layout.
- Thinner Stage 3, Stage 4, Stage 5, and Stage 6 facades aligned to the new
  stage map.
- Deeper synchronized updates to `scenario_config.yaml` and
  `scenario_design.yaml` that reflect the new stage structure.
- Broader scenario_1 logging or path helper cleanup where it simplifies the new
  stage lanes.
- Structure regression coverage that locks the new stage surface rather than the
  old eight-stage assumption.

## Validation Gates

- **G-01 Stage Surface Gate:** pass only if `runner.rs`, `mod.rs`, and the YAML
  files expose one explicit new stage map and the macro flow remains claim ->
  prepare -> receive -> bundle -> apply -> finalize.
- **G-01A Count Gate:** pass only if the final stage map exposes exactly 12
  stages with the stage names listed in `Target Stage Map After Refactor`.
- **G-02 Homogeneous Split Gate:** pass only if the old large Stage 3 through
  Stage 6 bodies are replaced by smaller responsibility-focused stages rather
  than one facade plus hidden monolith helpers.
- **G-03 YAML Sync Gate:** pass only if the rewritten YAML matches the final
  stage split, `rust_entry` wiring, and path contracts with no stale old layout.
- **G-04 Artifact Stability Gate:** pass only if artifact filenames, JSON field
  names, and wallet-visible report meanings remain stable.
- **G-05 Behavior Regression Gate:** pass only if the release-style Scenario 1
  simulator gates remain green after the stage-map expansion.
- **G-06 Cleanup Gate:** pass only if broader logging and path helper cleanup
  reduces duplication without creating a second execution or artifact lane.

## Parallelization Safety

- Stage 3 and Stage 4 work should complete before Stage 5 and Stage 6 begin.
- The stage-map rewrite and YAML rewrite must move in lockstep and therefore are
  not safe to split into unrelated parallel waves.
- Broader logging or path helper cleanup is safe only after the claim and
  prepare lane shape is stable.

## Blockers And Rollback

- If the proposed stage split creates micro-stages with no clear homogeneous
  responsibility, stop and collapse that split before proceeding.
- If YAML rewrites drift away from the implemented stage map, stop and fix the
  YAML in the same wave rather than carrying stale metadata forward.
- If broader cleanup starts to alter artifact meaning or apply ownership,
  rollback that cleanup and keep only the behavior-neutral refactor steps.

</specifics>

<canonical_refs>
## Canonical References

- `.planning/ROADMAP.md`
- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`
- `.planning/phases/020-refactor-scenario-1/020-RESEARCH.md`
- `.planning/phases/017-scenario-1/017-CONTEXT.md`
- `.planning/phases/018-a-b-c/018-CONTEXT.md`
- `.planning/phases/019-gaps-1/019-CONTEXT.md`
- `crates/z00z_simulator/src/scenario_1/mod.rs`
- `crates/z00z_simulator/src/scenario_1/runner.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4.rs`
- `crates/z00z_simulator/src/scenario_1/stage_5.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6.rs`
- `crates/z00z_simulator/src/scenario_1/stage_7.rs`
- `crates/z00z_simulator/src/scenario_1/stage_8.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`

</canonical_refs>

<code_context>
## Existing Code Insights

- `stage_3_utils` and `stage_4_utils` already show the desired direction for a
  smaller, responsibility-oriented split.
- `runner.rs` and the YAML files are the real stage-surface contract and must
  be updated together if the stage map changes.
- `stage_7.rs` and `stage_8.rs` already consume Stage 6 helpers; the phase
  should treat that reuse pragmatically rather than assuming it must disappear.
- Existing Scenario 1 tests already provide the release-style safety net for the
  canonical artifact lane.

## Test Spine

- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_chain_path -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage5_receive_bridge -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_storage_bridge -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage7_jmt_wallet_scan -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage8_proof_path -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture`

</code_context>

<deferred>
## Deferred Ideas

None.

</deferred>

---

*Phase: 020-refactor-scenario-1*
*Context revised: 2026-03-25*