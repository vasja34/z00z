# Phase 020: Refactor Scenario 1 - Research

**Researched:** 2026-03-25
**Domain:** Rust simulator pipeline stage-splitting for Scenario 1
**Confidence:** HIGH after user clarification

## User Constraints

### Locked Constraints From User Request

- Refactor `crates/z00z_simulator/src/scenario_1/stage_3.rs` through
  `stage_6.rs` into shorter, more compact, logically consistent units.
- Split large Scenario 1 stages into smaller homogeneous execution stages; do
  not stop at helper extraction only.
- Start with current Stage 3 and Stage 4, then move to current Stage 5 and
  Stage 6.
- Preserve Scenario 1 public behavior, canonical artifact meaning, and
  wallet-visible behavior.
- Deeper synchronized rewrites of `scenario_config.yaml` and
  `scenario_design.yaml` are in scope.
- Reverse Stage 6 reuse by later stages is allowed when it reuses already
  generated artifacts rather than rebuilding them from scratch.
- Broader cleanup around scenario_1 logging and path helpers is in scope.

### Locked Constraints From Roadmap And Requirements

- `SCN1-03`: Scenario 1 regular transfer execution stays on the storage-backed `resolve -> verify -> apply` path, with Stage 6 limited to draft or reload bridging and Stage 7 owning canonical apply.
- `SCN1-04`: Scenario 1 save/load/search plus checkpoint and snapshot reload preserve the same canonical root and canonical-path lookup semantics across RedB reopen.
- `SCN1-05`: Scenario 1 final checkpoint flow keeps draft and final artifacts separate and includes negative tamper coverage for witness, snapshot, and checkpoint materials.

### Non-Goals For This Phase

- No new protocol behavior.
- No new wallet semantics.
- No second storage or checkpoint path.
- No migration of simulator-only adapters into `z00z_core` or `z00z_storage`
  unless that logic is already canonical there.
- No change to wallet RPC meaning or artifact semantics.

## Project Constraints (from copilot-instructions.md)

- All code, comments, documentation, and technical content must remain in English.
- Use `z00z_utils` abstractions for file I/O, serialization, config, and time instead of direct stdlib or raw serde calls in business logic.
- Do not modify `crates/z00z_crypto/tari/`.
- Keep Rust 2021 idioms.
- Group imports from the same crate/module into a single brace-style `use`.
- Keep identifiers to at most 5 words.
- Follow naming conventions: `PascalCase` for types, `snake_case` for functions/modules, `SCREAMING_SNAKE_CASE` for constants, `is_*` / `has_*` for booleans.
- Prefer typed errors via `thiserror`; do not introduce `unwrap()` or `expect()` into production paths.
- Preserve ONE SOURCE OF TRUTH boundaries: simulator orchestration may adapt paths and artifacts, but canonical tx, claim, checkpoint, and storage logic must stay in their owning crates.
- Do not use destructive delete commands.

## Summary

Scenario 1 already has the right macro architecture, but its middle execution
lane is too coarse. The real Phase 020 problem is not just helper placement. It
is that the current Stage 3 through Stage 6 bodies mix too many unrelated
responsibilities inside single stage boundaries.

The codebase already contains the right local precedent for fixing this:
`stage_3_utils` and `stage_4_utils` show how a stage can become smaller and
clearer without abandoning the existing simulator ownership model. The user’s
clarification widens that idea: the runner and YAML are allowed to expose more
stages if that is the cleanest way to turn one oversized stage into several
homogeneous execution units.

The earlier assumption that reverse Stage 6 reuse must be removed was wrong.
If later stages consume Stage 6-produced artifacts or Stage 6-owned helper
logic, that can stay as long as it is explicit and stable. The refactor goal is
clarity and homogeneous stage ownership, not purity for its own sake.

**Primary recommendation:** split the current Stage 3 and Stage 4 lane first by
expanding the runner and YAML stage map, then apply the same discipline to the
current Stage 5 and Stage 6 lane, keeping Stage 6 reuse explicit where it saves
rebuilding prior-stage data. Use broader scenario_1 logging and path helper
cleanup only after the new stage boundaries are stable.

**Recommended execution order:** define the new claim-lane stage surface,
implement the Stage 3 split, implement the Stage 4 split, then split the Stage
5 and Stage 6 lane and widen shared cleanup, and finally close the phase on the
new stage surface and release-style gates.

## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| SCN1-03 | Storage-backed transfer execution remains canonical while stages split | Keep the macro `resolve -> verify -> apply` lane stable even if the runner grows more stage entries |
| SCN1-04 | Save/load/search and checkpoint reload preserve canonical root/path semantics | Preserve machine-readable continuity fields and typed path resolution across the expanded stage surface |
| SCN1-05 | Final checkpoint flow keeps draft/final artifacts separate with tamper coverage | Keep Stage 7 apply and Stage 8 publication semantics stable after the new stage split |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `ScenarioCfg` + typed stage path accessors | workspace | Config source of truth for stage paths and mode flags | Existing code already centralizes stage path lookup through `ctx.config.stage*_paths()`; this is the correct seam for refactor-safe path changes |
| `DesignDoc` / `DesignStage` / `DesignStep` | workspace | Executable design assertions and stable step ids | Runner validation depends on design step coverage; refactor must preserve this contract |
| `z00z_utils::{io, codec, time}` | workspace | Canonical file I/O, serialization, and time abstraction | Project-level ONE SOURCE OF TRUTH requirement explicitly mandates this stack |
| `z00z_wallets::core::{claim, tx, address}` | workspace | Canonical claim, tx, receive, and wallet-facing logic | Existing tests explicitly ban reintroducing local stage crypto/tx implementations |
| `z00z_storage::{assets, snapshot, checkpoint}` | workspace | Canonical prep snapshot, checkpoint draft, and storage-backed apply lane | Phase 017-019 decisions and tests make this the owned storage surface |

### Supporting

| Library | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `tokio` | `1` | Async runtime for RPC-backed stages | Keep only at stage orchestration boundary for stages 2, 4, and 5 |
| `serde` | `1.0` | Typed contracts for stage artifacts | Use for artifact structs and YAML/JSON wire contracts only |
| `thiserror` | `2.0` | Typed error enums where worth stabilizing | Use for public or reused error surfaces; avoid ad-hoc string accumulation when a type adds value |
| `rand` | `0.8` | Deterministic or system RNG adapters | Use only behind existing RNG mode/config seams |
| `rust_xlsxwriter` | `0.79` | Stage 4 reporting export | Keep isolated in report/export helper modules |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Expanding the stage map | Keep the old eight-stage map and only extract helpers | This leaves the core problem unsolved because the execution units remain too coarse |
| Keeping Stage 5 and Stage 6 monolithic until late | Align Stage 3 and Stage 4 first, then move the transfer lane | The user explicitly chose the second option because it reuses the strongest existing split pattern first |
| Forcing reverse Stage 6 reuse out of the design | Keep or formalize explicit Stage 6 reuse | The user explicitly allows reuse when it is artifact-led and avoids needless recomputation |

**Installation:**

```bash
# No new dependencies are recommended for Phase 020.
# Use the existing workspace stack from crates/z00z_simulator/Cargo.toml.
```

## Architecture Patterns

### Recommended Project Structure

```text
crates/z00z_simulator/src/scenario_1/
├── mod.rs
├── runner.rs
├── prep_ref.rs
├── storage_view.rs
├── scenario_paths.rs              # proposed shared path adapters if wider cleanup justifies them
├── scenario_logging.rs            # proposed shared logging helpers if wider cleanup justifies them
├── stage_3.rs
├── stage_3_utils/
├── stage_4.rs
├── stage_4_utils/
├── stage_5.rs
├── stage_5_utils/
├── stage_6.rs
└── stage_6_utils/
```

### Pattern 1: Thin Stage Facade

**What:** Each stage file keeps only the stable public entrypoints, stage-local contract re-exports, and one `run_core(...)` orchestration function.

**When to use:** Use for every numbered stage file. This is the default pattern for Phase 020.

**Why this fits this phase:** The current runner and tests already expect stable entrypoints like `run_claim_genesis`, `run_tx_prepare`, `run_simple`, and `run_bundle`. A facade preserves those paths while allowing internal extraction.

**Concrete guidance:**

- Keep `stage_3.rs` exporting `run_claim_genesis`, `build_claim_package`, `write_claim_bundle`, `verify_resume_wire`, and `Stage3Snapshot` at the same module path.
- Keep `stage_4.rs` exporting `run_tx_prepare`, `resolve_stage4_paths`, and any shared report types that Stage 7 still consumes.
- Keep `stage_5.rs` exporting `run_simple` only; move all other helpers to `stage_5_utils` unless a test imports them directly.
- Keep `stage_6.rs` exporting `run_bundle` while making any later-stage reuse an explicit Stage-6-owned surface through `stage_6_utils` or clear re-exports.

**Example:**

```rust
// Source: crates/z00z_simulator/src/scenario_1/stage_3.rs
pub fn run_claim_genesis(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
  match run_core(ctx, stage.stage) {
    Ok(()) => StageResult::Ok,
    Err(err) => StageResult::Fail(format!(
      "stage {} ({}) failed: {}",
      stage.stage, stage.name, err
    )),
  }
}
```

### Pattern 2: Separate Orchestration From Path And Config Adapters

**What:** Path remapping and config interpretation live in dedicated adapter modules, not in the stage orchestration body.

**When to use:** Use whenever a stage consumes `ScenarioCfg` path strings or needs runtime remapping under `ctx.outputs_dir`.

**Why this fits this phase:** `stage_4_utils/paths.rs` already proves that path remapping is simulator-only infrastructure. Stages 5 and 6 currently duplicate path-resolution behavior and should adopt the same split.

**Concrete guidance:**

- Create `stage_5_utils/paths.rs` for `resolve_input_path`, `rpc_log_path`, and `prep_dirs`-style adapters.
- Move any Stage 6 file lookup helpers (`resolve_input_path`, `direct_path`, `search_path`) into `stage_6_utils/paths.rs` or a broader `scenario_paths.rs` seam if the wider cleanup wave proves the helper is shared across the expanded stage map.
- Do not hardcode artifact paths in orchestration once a typed accessor exists.

**Example:**

```rust
// Source: crates/z00z_simulator/src/scenario_1/stage_4_utils/paths.rs
pub(crate) fn resolve_stage4_paths(
  ctx: &SimContext,
  cfg: &Stage4TxPrepareCfg,
) -> Stage4ResolvedPaths {
  let runtime_out = ctx.outputs_dir.clone();
  let cfg_out = std::path::PathBuf::from(&cfg.paths.outputs_dir);
  let outputs_dir = remap_out(&runtime_out, &cfg_out, &cfg.paths.outputs_dir);
  // ...
  Stage4ResolvedPaths { /* ... */ }
}
```

### Pattern 3: Keep Stage 6 Reuse Explicit Rather Than Accidental

**What:** If Stages 7 and 8 reuse Stage 6-produced artifacts or helper logic,
that reuse should be formalized through a clear Stage 6-owned surface or
re-exports instead of being hidden inside a monolithic file.

**When to use:** Apply in the Stage 5 and Stage 6 wave.

**Why this fits this phase:** the user explicitly allowed reverse Stage 6 reuse
when it avoids recomputing prior-stage data. The problem to solve is not the
existence of reuse. The problem is unclear ownership.

**Concrete guidance:**

- Keep Stage 6 as the owner of Stage 6-produced bridge data and helper logic
  when later stages genuinely consume those artifacts.
- Move purely local fragment or report internals into `stage_6_utils`.
- If later stages import Stage 6 helpers, expose that surface intentionally and
  document it in runner and YAML updates instead of pretending the coupling does
  not exist.

### Pattern 4: Artifact Contracts Live In Dedicated Types Modules

**What:** Snapshot/report/bridge JSON structs should live in explicit contract modules or at the top of the stage facade if they are part of the stable surface.

**When to use:** Use for Stage 5 and Stage 6 immediately. Stage 3 and Stage 4 can be tightened gradually if public paths stay stable.

**Why this fits this phase:** Stage 5 and Stage 6 currently mix artifact structs with orchestration and helper code. Separating contracts reduces cognitive load and makes artifact stability review easier.

**Concrete guidance:**

- Move `Stage5Snap`, `Stage5TxFile`, `RecvCtx`, and `RpcCtx` into `stage_5_utils/contracts.rs` or `context.rs` as appropriate.
- Move `Stage6Report`, `Stage6Bridge`, `FragIn`, `FragOut`, and any demo-only checkpoint shapes into `stage_6_utils/contracts.rs`.
- If a contract is consumed by another stage, either keep it explicitly owned by Stage 6 or move it into a clearly named scenario-level helper module when the wider cleanup wave justifies that promotion.

### Pattern 5: Validation Layering Before Artifact Emission

**What:** Validate in cost order: config/path presence, shape/selection, semantic invariants, canonical crypto/storage gates, then artifact writes.

**When to use:** Use inside `run_core(...)` for all refactored stages.

**Why this fits this phase:** Project design guidance explicitly prefers validation layering, and Scenario 1 tests depend on precise failure classification for tamper and gate checks.

**Concrete guidance:**

- Stage 3 order: config/load -> distribution -> claim package build -> resume/wallet import checks -> artifact write -> snapshot reconcile.
- Stage 4 order: config validation -> sender input selection -> output assembly -> tx verification gates -> prep snapshot generation -> wallet state captures -> artifacts.
- Stage 5 order: tx package load -> output selection -> canonical/runtime receive parity -> report-only RPC path -> claim mutation -> artifacts.
- Stage 6 order: stage4 artifact load -> fragment build -> exec-input generation -> bridge file/report write.

### Pattern 6: YAML Must Be Rewritten To Match The New Stage Surface

**What:** `scenario_config.yaml` owns configurable paths and stage parameters; `scenario_design.yaml` owns stage order, step ids, action semantics, and post-condition declarations. Rust must adapt to these files, not silently diverge from them.

**When to use:** Throughout the whole refactor.

**Concrete synchronization guidance:**

- The earlier assumption that YAML schema and stage ids must stay fixed is no
  longer valid for this phase. The YAML may change more deeply when the stage
  split requires it.
- Every path consumed by refactored Stage 5 and Stage 6 code must come from `ScenarioCfg` accessors or a helper built from them.
- Every log step id emitted after refactor must match the rewritten
  `scenario_design.yaml`, because `runner::validate_step_coverage(...)`
  enforces this.
- If the split changes stage numbering, stage names, or `rust_entry` mappings,
  update YAML and `runner.rs` in the same change set.
- Maintain a one-to-one traceability table while implementing:

| YAML Contract | Rust Consumer |
| --- | --- |
| `stage4_tx_prepare.paths.*` | `resolve_stage4_paths(...)` and its callers |
| `stage5_transfer.paths.*` | new `stage_5_utils::paths` module |
| `stage6_bundle.paths.*` | new `stage_6_utils::paths` or `scenario_paths` |
| Stage step ids `S3-*` to `S6-*` | `push_log(...)` call sites |
| Stage ordering 3 -> 4 -> 5 -> 6 -> 7 -> 8 | `runner::build_stage_map()` and design file |

### Stage-By-Stage Decomposition Guidance

#### Stage 3

- Keep the current `stage_3_utils` split. It is already the best local precedent in the codebase.
- Extract remaining non-orchestration code from `stage_3.rs` into two new internal concerns only if needed:
  - `claim_pkg.rs`: `to_claim_wire`, `build_claim_package*`, `write_claim_bundle*`, claim hash helpers
  - `audit.rs` or `logging.rs`: `parse_reason_code`, `write_audit_log`, `push_log`, `flush_logs`
- Keep the public API path stable by re-exporting from `stage_3.rs`.

#### Stage 4

- This is the main hotspot at 2934 lines and should be refactored first after
  the shared bridge extraction.
- Add more `stage_4_utils` modules instead of creating a second monolith:
  - `selection.rs`: sender input selection, serial targeting, cursor parsing
  - `outputs.rs`: Bob/change/fee output assembly and deterministic nonce helpers
  - `prep.rs`: prep file rows, canonical path sync, snapshot compare
  - `verify.rs`: fee formula, spend witness, commitment balance, tx package verification
  - `tamper.rs`: test-fast tamper injection and root tamper hooks
  - `contracts.rs`: `PendingRow`, `ConfirmRow`, `Stage4ResolvedPaths`, `PrepRow`, `PrepFile`
- Keep `run_tx_prepare` and `run_core` in `stage_4.rs`.

#### Stage 5

- Create `stage_5_utils/` because Stage 5 now mixes context loading, receive parsing, RPC report flow, claim mutation, artifact writes, and log helpers in one file.
- Recommended split:
  - `context.rs`: `StageCtx`, `RecvCtx`, `RpcCtx`, cfg/service accessors
  - `paths.rs`: input resolution, directory prep, rpc log path
  - `receive.rs`: output selection, stealth parsing, canonical/runtime parity checks
  - `rpc_flow.rs`: report-only RPC path and claim list checks
  - `claim_flow.rs`: `run_claim`, `list_claimed`, claim set invariants
  - `artifacts.rs`: write tx leaf artifact and snapshot
  - `logging.rs`: `log_ok`, `push_log`, `flush_logs`, `log_extra`
- Keep all artifact names and JSON keys unchanged.

#### Stage 6

- Split Stage 6 in two directions:
  - later-stage reuse stays explicit through a Stage 6-owned surface;
  - true Stage 6 fragment/report/demo logic moves to `stage_6_utils/`.
- Recommended `stage_6_utils` split:
  - `fragments.rs`: `build_target_frag`, snap item lookup, hash helpers
  - `report.rs`: `Stage6Report`, bridge/report writing
  - `demo.rs`: demo checkpoint builders and validators, ideally under `#[cfg(test)]` if production code does not need them
  - `paths.rs`: if any path lookup remains Stage 6-only after bridge extraction
- This is the root-cause fix for the current Stage 6 monolith without forcing
  the later-stage reuse itself to disappear.

### Anti-Patterns to Avoid

- **Do not move simulator-only adapters into core crates:** Path remap, report export, and YAML glue belong in `z00z_simulator`.
- **Do not rename artifact files for cleanliness:** Existing tests assert files like `checkpoint_bridge_s6.json`, `checkpoint_s7.json`, `checkpoint_s8.json`, `wallets_pending.json`, and `checkpoint_prep.json`.
- **Do not change step ids or silently drop log rows:** `runner::validate_step_coverage(...)` will fail.
- **Do not preserve the old stage map just because it already exists:** the
  user explicitly wants a smaller, more homogeneous stage layout.
- **Do not reintroduce local tx or claim crypto logic:** tests explicitly ban this in Stage 3 and Stage 4.
- **Do not bypass `ScenarioCfg` accessors with new literals:** that creates YAML drift immediately.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| File I/O and JSON/YAML persistence | direct `std::fs`, ad-hoc serde read/write glue | `z00z_utils::io` + `z00z_utils::codec` | Project architecture explicitly centralizes these concerns |
| Claim construction and receive ownership checks | local stealth/claim crypto pipeline in simulator | `z00z_wallets::core::claim`, `z00z_wallets::core::address`, `z00z_wallets::core::tx` | Existing tests enforce this boundary |
| Prep snapshot and checkpoint formats | custom simulator checkpoint schema | `z00z_storage::snapshot` and `z00z_storage::checkpoint` | Canonical storage lane already exists and is covered by integration tests |
| Wallet state reports from raw ad-hoc loops | hand-built row logic in every stage | `stage_4_utils::reports_*` helpers | Existing Stage 4 utilities already define the schema and continuity rules |
| Path remapping logic repeated per stage | new per-file string-join code | dedicated path adapter modules modeled on `stage_4_utils/paths.rs` | Avoids artifact drift and keeps YAML ownership central |

**Key insight:** The phase is a behavior-preserving stage redesign inside the
simulator surface, not merely a helper shuffle. The correct strategy is to
expand the stage map only where it creates homogeneous execution units while
continuing to reuse the canonical crate APIs and the already-proven
`stage_*_utils` pattern.

## Runtime State Inventory

| Category | Items Found | Action Required |
| --- | --- | --- |
| Stored data | No rename-driven datastore migration was identified. Scenario outputs under `crates/z00z_simulator/src/scenario_1/outputs` are generated artifacts, not authoritative state to migrate for this phase. | Code edit only. Re-run scenario/tests after refactor to regenerate artifacts. No data migration. |
| Live service config | None found for this phase. Scenario 1 behavior is configured from in-repo YAML and test harnesses, not from external UI-managed services. | None. |
| OS-registered state | None found. This phase does not rename binaries, services, or OS registrations. | None. |
| Secrets/env vars | Runtime env vars `Z00Z_WALLET_NETWORK` and `Z00Z_WALLET_CHAIN` are set during Stage 2. No rename or semantic change is planned. | Code edit only if helper extraction moves the setup code; env var names stay unchanged. |
| Build artifacts | `target/` outputs and generated Scenario 1 outputs will become stale after refactor, but they are rebuildable. | Re-run targeted tests and scenario binary after refactor. No migration. |

## Common Pitfalls

### Pitfall 1: Reverse Stage Coupling

**What goes wrong:** Stage 7 and Stage 8 depend on internals from `stage_6.rs`, so Stage 6 cannot be simplified without impacting later stages.

**Why it happens:** Shared checkpoint bridge helpers were added to the Stage 6 file because that was the nearest place, but they are no longer Stage 6-only.

**How to avoid:** Extract shared bridge contracts and helpers into a stage-neutral module before shrinking Stage 6.

**Warning signs:** Imports like `use super::stage_6::{...}` in Stage 7 or Stage 8.

### Pitfall 2: YAML Drift During Helper Extraction

**What goes wrong:** The Rust refactor compiles, but `scenario_config.yaml` or `scenario_design.yaml` stops matching reality.

**Why it happens:** Developers move path logic or log calls and forget that runner validation and tests treat YAML as executable contract.

**How to avoid:** Keep a traceability checklist for every extracted helper and do not change step ids or path keys unless the YAML is updated in the same patch.

**Warning signs:** `validate_step_coverage(...)` failures, missing artifact files, or tests that assert specific snapshot/log filenames.

### Pitfall 3: Over-Extracting Into Wrong Crates

**What goes wrong:** Simulator report or path code gets pushed into `z00z_core` or `z00z_storage`, widening boundaries and creating future maintenance cost.

**Why it happens:** The refactor pressure makes it tempting to move anything reused anywhere into a lower crate.

**How to avoid:** Extract only canonical behavior into owning crates. Keep simulator-only adapters, reports, and YAML glue local to `scenario_1`.

**Warning signs:** New public APIs in core/storage that exist only to support Scenario 1 file layout or report formatting.

### Pitfall 4: Behavioral Changes Hidden Inside “Cleanup”

**What goes wrong:** The code becomes cleaner but output files, JSON shape, or receive/apply ordering changes.

**Why it happens:** Refactors mix structural moves with incidental “improvements”.

**How to avoid:** Change shape first, behavior never. Keep public structs, filenames, and stage sequencing stable.

**Warning signs:** Snapshot diffs, changed file names, or Stage 6 doing more than bridge/reload work.

### Pitfall 5: Losing Precise Validation Order

**What goes wrong:** A moved helper changes when a gate runs, so failure classification or tamper behavior changes.

**Why it happens:** Validation and artifact emission are currently interleaved in large files, so extraction can silently reorder them.

**How to avoid:** Refactor around explicit phases in `run_core(...)`: load, validate, apply/report, persist artifacts.

**Warning signs:** Existing tamper tests start failing with different messages or later-stage failures.

### Pitfall 6: Reintroducing Duplicated Local Logic

**What goes wrong:** New helper modules accidentally reimplement selection, confirm-row, or crypto rules already owned by wallet/storage crates.

**Why it happens:** Large files hide which helpers are mere adapters and which are canonical logic.

**How to avoid:** Start each extraction by marking helpers as one of: orchestration, adapter, artifact contract, or canonical call-through.

**Warning signs:** Tests like `test_claim_acceptance` fail source-structure assertions.

## Code Examples

Verified patterns from existing code and official Rust guidance:

### Stage-Local State Extraction

```rust
// Source: crates/z00z_simulator/src/scenario_1/stage_3_utils/state.rs
pub(crate) fn persist_claim_state_file(path: &Path, state: &ClaimStateFile) -> Result<(), String> {
  let next = if let Some(old) = load_claim_state_file(path)? {
    merge_state(&old, state)?
  } else {
    state.clone()
  };
  save_json(path, &next).map_err(|e| format!("claim_state write failed: {e}"))
}
```

**Why this is the right pattern:** Stage 3 keeps crash-safe claim-state behavior in a focused utility module while the stage facade stays stable.

### Path Adapter Extraction

```rust
// Source: crates/z00z_simulator/src/scenario_1/stage_4_utils/paths.rs
pub(crate) fn stage4_runtime_paths(
  ctx: &SimContext,
  cfg: &Stage4TxPrepareCfg,
) -> (std::path::PathBuf, std::path::PathBuf) {
  let paths = resolve_stage4_paths(ctx, cfg);
  (paths.wallets_dir, paths.rpc_logger_file)
}
```

**Why this is the right pattern:** Config interpretation is isolated from orchestration and easy to reuse without duplicating literals.

### Thin Crate-Local Module Boundaries

```rust
// Source: Rust Book, Control Scope and Privacy with Modules
// Organize related code into child modules and keep visibility explicit.
pub mod garden;
```

**Why this matters here:** Rust’s standard module model favors small child modules with explicit visibility. Phase 020 should use more local modules, not bigger files.

### Keep Integration Tests External To Public Behavior

```rust
// Source: Rust Book, Test Organization
// Integration tests live under tests/ and use public APIs only.
```

**Why this matters here:** Existing `crates/z00z_simulator/tests/*.rs` files already protect public behavior. Keep using them as the phase gate while adding a few unit tests for newly extracted local helpers.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Large stage files mixing orchestration, adapters, and contracts | Thin stage facade plus adjacent `stage_*_utils` modules | Already present in Stage 3 and Stage 4 as of current codebase | Phase 020 should extend this pattern to Stages 5 and 6 |
| Direct low-level I/O and serialization in business logic | `z00z_utils` abstraction layer | Project architecture doc and refactors since 2025 | Refactor must keep using `io`, `codec`, and `time` wrappers |
| Simulator-local tx/checkpoint shims | Wallet/storage-owned canonical APIs | Phase 017-019 storage-backed migration | Do not reintroduce local crypto or storage formats during cleanup |
| Treating Stage 6 as both stage and shared library | Stage-neutral shared checkpoint bridge module | Needed now; current code shows transitional smell | This is the main structural correction required in Phase 020 |

**Deprecated/outdated:**

- Monolithic stage files larger than their helper modules combined.
- Cross-stage imports from `stage_6.rs` into later stages.
- New hardcoded path literals outside typed config adapters.

## Open Questions

### Question 0: Which `todo.md` gaps does Phase 020 actually close?

What we know: there is no phase-local `todo.md` under Phase 020, while older
phases still contain functional gap ledgers.

What's unclear: whether the planner should treat those older todo items as
direct completion criteria for this refactor phase.

Recommendation: treat Phase 020 as a structural enabling phase unless the
planner explicitly folds selected older todo items into scope. Do not imply
that this refactor alone closes unresolved functional gaps from phases 018 or
019.

### Gap Coverage Matrix

| Gap Source | Covered Directly In Phase 020? | Why |
| --- | --- | --- |
| Large stage monoliths in Scenario 1 | Yes | This is the direct refactor target |
| Reverse imports from `stage_6.rs` into Stages 7/8 | Yes | This is the root structural dependency defect Phase 020 must remove |
| YAML drift risk between Rust and scenario docs | Yes | The phase explicitly requires same-wave YAML synchronization |
| Missing structure regression coverage for new seams | Yes | Wave 0 gaps already require new structure-focused tests |
| Older nullifier semantics gaps from Phase 019 todo | No, unless explicitly added later | Those are functional semantics, not structural decomposition |
| Older backup restoration or receive taxonomy gaps | No, unless explicitly added later | Those are wallet behavior gaps outside the Phase 020 refactor boundary |

### Question 1: Should Phase 020 update Stage 7 and Stage 8 imports even though the goal names only Stages 3-6?

What we know: Stage 7 and Stage 8 currently import shared helpers from `stage_6.rs`.

What's unclear: Whether the planner wants that counted as in-scope refactor work or as collateral compile maintenance.

Recommendation: Treat it as in scope. The refactor is incomplete otherwise.

### Question 2: Should Stage 3 public helper functions remain in `stage_3.rs` or move behind re-exports?

What we know: tests import `build_claim_package`, `write_claim_bundle`, `verify_resume_wire`, and `Stage3Snapshot` from `scenario_1::stage_3`.

What's unclear: Whether additional public helper cleanup is desired.

Recommendation: Keep the module path stable and use re-exports if extraction is needed.

### Question 3: Can Stage 6 demo helpers move under `#[cfg(test)]`?

What we know: `stage_6.rs` contains demo checkpoint helpers mixed with production bridge logic.

What's unclear: Whether any non-test runtime path still depends on them.

Recommendation: Audit call sites during execution. If only tests use them, move them into a test-only or clearly named demo helper module.

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Rust `cargo test` integration + unit tests |
| Config file | none |
| Quick run command | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture` |
| Full suite command | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| SCN1-03 | Stage 6 stays bridge-only; Stage 7 owns canonical apply | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture` | ✅ |
| SCN1-03 | Checkpoint bridge and storage apply remain wired across stages 6 and 7 | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_storage_bridge -- --nocapture` | ✅ |
| SCN1-04 | Prep snapshot and canonical root/path continuity stay stable | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_pipeline_genesis_tx -- --nocapture` | ✅ |
| SCN1-05 | Draft/final checkpoint separation and tamper gates remain intact | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture` | ✅ |
| REF-020-STRUCT | Stage 3 and Stage 4 continue routing through canonical APIs after refactor | integration/source-structure | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture` | ✅ |
| REF-020-RECV | Stage 5 canonical/runtime receive parity and report-first behavior remain stable | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage5_receive_bridge -- --nocapture` | ✅ |
| REF-020-CHARLIE | Stage 7 Charlie wallet scan continuity remains stable after Stage 6 extraction | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage7_jmt_wallet_scan -- --nocapture` | ✅ |

### Sampling Rate

- **Per task commit:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_acceptance -- --nocapture`
- **Per wave merge:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_pipeline_genesis_tx -- --nocapture`
- **Phase gate:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` plus `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`

### Wave 0 Gaps

- [ ] Add a structure-focused regression test for Stage 5 and Stage 6 similar to `test_claim_acceptance.rs`, so the monolith does not reappear.
- [ ] Add stage-surface tests for the expanded runner and YAML map plus focused unit coverage only where the wider Stage 6 or scenario-level helper cleanup introduces non-trivial branching.
- [ ] Add unit tests for any newly introduced `stage_5_utils::paths` or `stage_5_utils::artifacts` modules if they contain path remap or JSON shape logic.

## Sources

### Primary (HIGH confidence)

- Repository code under `crates/z00z_simulator/src/scenario_1/`.
- Repository tests under `crates/z00z_simulator/tests/`.
- `crates/ONE_SOURCE_OF_TRUTH.md`.
- `crates/Z00Z_DESIGN_FOUNDATION.md` sections on ONE SOURCE OF TRUTH and Validation Layering.
- Rust Book: [Control Scope and Privacy with Modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
- Rust Book: [Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)

### Secondary (MEDIUM confidence)

- Rust API Guidelines checklist: [Checklist](https://rust-lang.github.io/api-guidelines/checklist.html)

### Tertiary (LOW confidence)

- None. No recommendation in this document depends solely on unverified third-party commentary.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - derived from current workspace dependencies, tests, and project architecture constraints.
- Architecture: HIGH - based on existing local precedents (`stage_3_utils`, `stage_4_utils`), runner contracts, and cross-stage import evidence.
- Pitfalls: HIGH - directly evidenced by current code structure and existing integration tests.

**Research date:** 2026-03-25
**Valid until:** 2026-04-24
