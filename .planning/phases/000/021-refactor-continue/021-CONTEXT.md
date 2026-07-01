<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD047 -->
# Phase 021: 021-refactor-continue - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 021 continues the structural refactor of Scenario 1 that Phase 020 left
incomplete. Phase 020 achieved the 12-stage *runtime* mapping in `runner.rs`
and the matching YAML surface, but it did **not** produce 12 independent stage
source files. Instead stages 3–4 are packed inside `stage_3.rs`, stages 5–6
inside `stage_4.rs`, stages 7–8 inside `stage_5.rs`, and stages 9–10 inside
`stage_6.rs`. The file names conflict with the logical stage numbers, the files
are oversized (up to 2935 lines), and the stage boundaries inside each file
are implicit rather than explicit.

Phase 021 finishes the job: create explicit, homogeneous, compact stage files
that match the 12-stage map, and restore `scenario_design.yaml` to a clean
pre-phase-020 baseline renamed as `design_scenario_orig.yaml` so the old
pre-split specification is preserved for reference. The final Scenario 1 root
surface is strict: keep only `stage_1.rs` through `stage_12.rs`,
`scenario_config.yaml`, `scenario_design.yaml`, `design_scenario_orig.yaml`,
`mod.rs`, `runner.rs`, `Readme.md`, and `storage_view.rs` at the root. Any
lane-specific helpers or legacy support modules must live under `stage_*_utils/`
rather than as extra root files.

### What Phase 021 Delivers

1. Explicit per-stage Rust files for stages 3 through 12, each owning exactly
  one homogeneous responsibility, with canonical root names aligned to the
  runner stage number (`stage_3.rs` through `stage_12.rs`). No suffixed or
  alias stage files remain in the Scenario 1 root.
2. An explicit handling decision for `run_claim_genesis`, which currently lives
  in `stage_3.rs` but is not part of the 12-stage runner dispatch. The split
  must keep its location and re-export behavior deliberate rather than leaving
  it as accidental residue in the claim lane.
3. Thin stage facade files aligned to the 12-stage map so the runner dispatch
   and `mod.rs` re-exports match stage numbers, not arbitrary internal
   container files.
4. `design_scenario_orig.yaml` — a pre-phase-020 canonical user description
   preserved as reference (already created from git commit `96957d5f`).
5. Updated `scenario_design.yaml` returned to a descriptive design-document
  format rather than an executable-plan surrogate, while keeping it in sync
  with the runner dispatch.
6. A post-refactor `scenario_design.yaml` structure that mirrors the style and
  descriptive discipline of `design_scenario_orig.yaml`, but reflects the
  actual post-refactor stage boundaries and code-owned flow.
7. Stage-surface regression tests that assert one file per logical stage rather
   than the current multi-stage container layout.

### What Phase 021 Does NOT Deliver

- No new business capabilities, protocol behavior, or wallet semantics.
- No second storage or checkpoint path.
- No changes to `z00z_crypto/tari/` (READ-ONLY).
- No migration of simulator-only adapters into `z00z_core` or `z00z_storage`.

### Historical Baseline (Gap Phase 020 Left)

| Runner stage | Stage name | Current file | Problem |
| --- | --- | --- | --- |
| 3 | `claim_prepare` | `stage_3.rs` | Also hosts stage-4 logic (996 ln) |
| 4 | `claim_publish` | `stage_3.rs` | Same file as stage 3 |
| 5 | `tx_plan` | `stage_4.rs` | Also hosts stage-6 logic (2935 ln) |
| 6 | `tx_prepare` | `stage_4.rs` | Same file as stage 5 |
| 7 | `transfer_receive` | `stage_5.rs` | Also hosts stage-8 logic (833 ln) |
| 8 | `transfer_claim` | `stage_5.rs` | Same file as stage 7 |
| 9 | `bundle_build` | `stage_6.rs` | Also hosts stage-10 logic (1275 ln) |
| 10 | `bundle_publish` | `stage_6.rs` | Same file as stage 9 |
| 11 | `checkpoint_apply_storage` | `stage_7.rs` | Single-responsibility logic existed, but root file name did not match stage number |
| 12 | `checkpoint_finalize` | `stage_8.rs` | Single-responsibility logic existed, but root file name did not match stage number |

Target: stages 3–12 each resolve through canonical root files whose names match
their logical stage numbers, with any extracted helpers relocated under
`stage_*_utils/`.

</domain>

<decisions>
## Implementation Decisions

### Naming convention for new stage files

- **D-01:** Use the existing `stage_N.rs` naming convention aligned to the
  logical stage number (e.g. `stage_3.rs` = `claim_prepare` only,
  `stage_4.rs` = `claim_publish` only, etc.).
- **D-01A:** The current `stage_3.rs` through `stage_6.rs` files are
  containers for two logical stages each. Split them by carving out the second
  stage's code into the logically correct file name, then slim the original
  file to its single responsibility.
- **D-01B:** The Scenario 1 root does not permit suffix-named stage files.
  Intermediate names such as `stage_4_claim_publish.rs` or
  `stage_5_tx_plan.rs` are migration artifacts only and must be removed or
  moved under `stage_*_utils/` before the phase closes.

### Stage file split priority

- **D-02:** Split order follows Phase 020 precedent: claim lane first (stages
  3–4), then tx-prepare lane (stages 5–6), then transfer lane (stages 7–8),
  then bundle lane (stages 9–10).
- **D-03:** Stages 11 and 12 also follow the canonical root numbering contract.
  Their single-responsibility logic may stay intact, but the public files in the
  Scenario 1 root must be `stage_11.rs` and `stage_12.rs`, not `stage_7.rs`
  and `stage_8.rs`.

### Stage utils directories

- **D-04:** Existing `stage_3_utils/`, `stage_4_utils/`, `stage_5_utils/`, and
  `stage_6_utils/` stay in place. Their content is already split by
  responsibility and should not be renamed just because the parent stage file
  gains a new split. A utils directory that serves only the claim_publish stage
  may be renamed to match if that is trivially done in the same wave.

### Behavior and contract preservation

- **D-05:** Canonical artifact semantics, wallet-visible behavior, and
  runner dispatch (function signatures in `build_stage_map()`) remain locked.
- **D-06:** The storage-backed `resolve -> verify -> apply` ownership boundary,
  checkpoint publication semantics, and proof-bearing artifact lane from phases
  017–019 remain binding.
- **D-07:** All release-style simulator gates (`cargo run --release -p
  z00z_simulator --bin scenario_1 --features wallet_debug_dump` and the full
  test suite) must remain green after every split wave.
- **D-07A:** `scenario_design.yaml` remains descriptive, not executable. It
  documents the code path; it does not define or drive the code path.

### YAML synchronization

- **D-08:** `scenario_design.yaml` is updated in the same wave as the final
  stage split so the YAML `rust_entry` fields continue to match the runner
  dispatch functions.
- **D-08A:** The rewritten `scenario_design.yaml` must return to the
  descriptive structure exemplified by `design_scenario_orig.yaml`, while using
  the post-refactor stage boundaries and names.
- **D-08B:** The rewritten `scenario_design.yaml` must describe the real order
  of runtime operations in code. Inputs, outputs, calls, actions, and
  post-conditions must follow the actual execution sequence in the stage
  implementations, not an aspirational or legacy flow.
- **D-09:** `design_scenario_orig.yaml` (already created at commit `96957d5f`)
  is kept read-only as a reference artefact. It must not be modified by this
  phase.

### Agent's Discretion

- Exact new file names for the split stages, provided the result is
  monotonic, compact, and one file per logical stage.
- Exact internal boundary between the two stages currently packed in each
  container file (the public entry function signatures are fixed; only internal
  split is discretionary).
- Exact approach for migrating `stage_5_utils/` and `stage_6_utils/` content
  if a utils directory becomes clearly identified with only one of the two
  previously-co-hosted stages.

</decisions>

<specifics>
## Specific Requirements From User

- Phase 020 was done poorly: there are no explicit files `stage_9.rs`,
  `stage_10.rs`, `stage_11.rs`, `stage_12.rs`.
- `scenario_design.yaml` is just a user-facing description, NOT an operational
  run plan — all stages run sequentially from `runner.rs`. The approach used by
  phase 020 to treat it as the primary stage contract was correct; the gap is
  only the missing per-stage files and the drift away from a descriptive format
  toward an executable-looking document.
- `design_scenario_orig.yaml` was created from git commit `96957d5f` as the
  preserved pre-phase-020 reference baseline. It is 1219 lines and represents
  the pre-split 8-stage user description.
- After refactoring, `scenario_design.yaml` must structurally resemble
  `design_scenario_orig.yaml`: it should again read as a descriptive scenario
  document, not as an executable control surface.
- The new `scenario_design.yaml` must reflect the real code path exactly: the
  order of inputs, outputs, calls, and actions written in YAML must match the
  actual runtime behavior implemented in `runner.rs` and the stage files.
- `stage_3.rs` currently exposes `run_claim_genesis` in addition to
  `run_claim_prepare` and `run_claim_publish`. `run_claim_genesis` is re-
  exported from `mod.rs` but is not part of the 12-stage runner dispatch, so
  the split plan must account for it explicitly instead of treating
  `stage_3.rs` as a pure 2-function file.
- The new plan must bring the simulator to a structure with explicit stage split
  for stages 3–12, making them structurally correct, homogeneous, and more
  compact.

## Audit: What Phase 020 Planned vs. What Was Delivered

| Phase 020 Goal | Delivered | Gap |
| --- | --- | --- |
| 12-stage runtime map in `runner.rs` | ✅ Yes | — |
| 12-stage fallback design in `runner.rs` | ✅ Yes | — |
| `scenario_design.yaml` updated for 12 stages | ✅ Yes | — |
| Stage tests (`test_scenario1_stage_surface`) | ✅ Yes | — |
| One file per logical stage (stages 3–10) | ❌ No | Files `stage_3.rs`–`stage_6.rs` still host 2 runtime stages each |
| Explicit `stage_9.rs`, `stage_10.rs` | ❌ No | Missing entirely |
| `stage_3.rs` slim to claim_prepare only | ❌ No | Has both claim_prepare and claim_publish (996 lines) |
| `stage_4.rs` slim to tx_plan only | ❌ No | Has both tx_plan and tx_prepare (2935 lines) |
| `stage_5.rs` slim to transfer_receive only | ❌ No | Has both transfer_receive and transfer_claim (833 lines) |
| `stage_6.rs` slim to bundle_build only | ❌ No | Has both bundle_build and bundle_publish (1275 lines) |

## Execution Spine

1. Create `design_scenario_orig.yaml` from git (already done at `96957d5f`).
2. Split claim lane: carve `claim_publish` code out of `stage_3.rs` into a new
  file aligned to runner stage 4. Slim `stage_3.rs` to the remaining
  claim-lane responsibility, and make an explicit keep-or-extract decision for
  `run_claim_genesis` so `mod.rs` re-exports stay intentional.
3. Split tx lane: carve `tx_prepare` code out of `stage_4.rs` into a new file
   aligned to runner stage 6. Slim `stage_4.rs` to `tx_plan` only.
4. Split transfer lane: carve `transfer_claim` code out of `stage_5.rs` into a
   new file aligned to runner stage 8. Slim `stage_5.rs` to `transfer_receive`.
5. Split bundle lane: carve `bundle_publish` code out of `stage_6.rs` into a
   new file aligned to runner stage 10. Slim `stage_6.rs` to `bundle_build`.
6. Update `mod.rs` re-exports to reflect new file locations.
7. Update `runner.rs` imports to reflect new file locations (runner
   dispatch function bodies do not change).
8. Rewrite `scenario_design.yaml` back into a descriptive format, using
  `design_scenario_orig.yaml` as the structural reference while keeping its
  `rust_entry`, stage narrative, and step ordering aligned with the new file
  layout and actual code execution order.
9. Run release-style gates; close the phase only when all gates pass.

## Validation Gates

- **G-01 Stage File Count Gate:** each of the 12 logical stages (1–12) must
  resolve to exactly one primary Rust source file.
- **G-02 Homogeneous Split Gate:** no stage file hosts more than one logical
  runtime stage. Each file's public entry functions must cover exactly the
  stage(s) declared in that file's name.
- **G-03 YAML Sync Gate:** `scenario_design.yaml` `rust_entry` fields match
  the actual public function names exported after the split.
- **G-03A YAML Format Gate:** `scenario_design.yaml` is a descriptive design
  document again and no longer reads like an executable control script.
- **G-03B YAML Fidelity Gate:** the documented order of inputs, outputs, calls,
  actions, and checks matches the real stage code execution flow.
- **G-04 Artifact Stability Gate:** artifact filenames, JSON field names, and
  wallet-visible report meanings remain stable.
- **G-05 Behavior Regression Gate:** release-style Scenario 1 gates remain
  green after the full split.
- **G-06 Test Surface Gate:** `test_scenario1_stage_surface` asserts the
  full 12-stage surface and correctly identifies source file layout.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase context
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/020-refactor-scenario-1/020-CONTEXT.md` — original phase goal and decisions
- `.planning/phases/020-refactor-scenario-1/020-RESEARCH.md` — architectural patterns and stack

### Simulator source
- `crates/z00z_simulator/src/scenario_1/mod.rs`
- `crates/z00z_simulator/src/scenario_1/runner.rs` — canonical stage map and dispatch
- `crates/z00z_simulator/src/scenario_1/stage_3.rs` — claim_prepare + claim_publish (2-in-1)
- `crates/z00z_simulator/src/scenario_1/stage_4.rs` — tx_plan + tx_prepare (2-in-1)
- `crates/z00z_simulator/src/scenario_1/stage_5.rs` — transfer_receive + transfer_claim (2-in-1)
- `crates/z00z_simulator/src/scenario_1/stage_6.rs` — bundle_build + bundle_publish (2-in-1)
- `crates/z00z_simulator/src/scenario_1/stage_7.rs` — checkpoint_apply_storage (single ✅)
- `crates/z00z_simulator/src/scenario_1/stage_8.rs` — checkpoint_finalize (single ✅)
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/`
- `crates/z00z_simulator/src/scenario_1/stage_5_utils/`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` — current 12-stage YAML
- `crates/z00z_simulator/src/scenario_1/design_scenario_orig.yaml` — pre-phase-020 reference (read-only)

### Prior phase decisions
- `.planning/phases/017-scenario-1/017-CONTEXT.md`
- `.planning/phases/018-a-b-c/018-CONTEXT.md`
- `.planning/phases/019-gaps-1/019-CONTEXT.md`

</canonical_refs>
