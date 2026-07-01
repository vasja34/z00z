# Phase 057 Full Audit

## 🔔 Audit Run — 2026-06-14 15:43:28

### 📌 Audit Setup

- Phase directory: `.planning/phases/057-HJMT-multi-aggregator`
- Derived FULL-AUDIT path:
  `.planning/phases/057-HJMT-multi-aggregator/057-FULL-AUDIT.md`
- Mandatory context read:
  - `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
  - `.github/copilot-instructions.md`
  - `.github/prompts/gsd-audit-4.prompt.md`
  - `.github/prompts/references/gsd-audit-4-full-audit-report-format.md`
  - `.github/skills/doublecheck/SKILL.md`
- Phase packet read:
  - `057-TODO.md`
  - `057-CONTEXT.md`
  - `057-SOURCE-AUDIT.md`
  - `057-TEST-SPEC.md`
  - `057-TESTS-TASKS.md`
  - `057-VALIDATION.md`
  - `057-UAT.md`
  - `057-SECURITY.md`
  - `057-EVAL-REVIEW.md`
  - `057-01-PLAN.md` through `057-07-PLAN.md`
  - `057-01-SUMMARY.md` through `057-07-SUMMARY.md`
- Execution mode: direct repo audit with manual fallback for all four mandatory
  audit passes, followed by YOLO fix, post-fix reruns, and manual
  `doublecheck` fallback.

> [!IMPORTANT]
> Final in-scope crate list before any audit pass began: `z00z_storage`,
> `z00z_aggregators`, `z00z_rollup_node`, `z00z_validators`,
> `z00z_watchers`, `z00z_simulator`.

- Explicitly excluded crates or modules:
  - `z00z_crypto/tari`: mentioned by the test packet only as cryptographic
    substrate context, is protected vendor code, and is not a Phase 057
    implementation home.
  - `z00z_core`, `z00z_utils`, `z00z_wallets`, `z00z_networks_*`, and other
    workspace crates: phase artifacts treat them as dependencies or broad
    validation collateral, not as phase-owned implementation surfaces.

### 🎯 Scope And Source Of Truth

- Scope was derived from the Phase 057 packet itself, not from workspace-wide
  recency:
  - `057-CONTEXT.md` freezes the cross-crate ownership map:
    `z00z_aggregators` owns route lineage and publication binding,
    `z00z_storage` owns shard-root and proof truth, `z00z_rollup_node` owns
    handoff validation, `z00z_validators` and `z00z_watchers` own downstream
    acceptance surfaces, and `z00z_simulator` owns scenario config and trace
    verification.
  - `057-SOURCE-AUDIT.md` maps live anchors for runtime publication, storage
    proof truth, node preflight, validator/watcher reuse, simulator trace
    homes, and accepted storage bench homes.
  - `057-VALIDATION.md` names the live command matrix for
    `z00z_storage`, `z00z_aggregators`, `z00z_rollup_node`,
    `z00z_validators`, `z00z_watchers`, and `z00z_simulator`.
  - `057-TEST-SPEC.md` and `057-TESTS-TASKS.md` derive the required user
    journeys, failure paths, trace paths, fixtures, and guardrails from the
    same six crate homes.
- The primary live code surfaces audited in this run were:
  - `crates/z00z_storage/src/settlement/proof_batch.rs`
  - `crates/z00z_storage/src/settlement/proof_batch_verify.rs`
  - `crates/z00z_storage/tests/test_hjmt_root_generation.rs`
  - `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`
  - `crates/z00z_storage/tests/test_hjmt_scope_birth.rs`
  - `crates/z00z_storage/tests/test_bench_lanes.rs`
  - `crates/z00z_storage/tests/test_live_guardrails.rs`
  - `crates/z00z_runtime/aggregators/src/service.rs`
  - `crates/z00z_runtime/aggregators/src/types.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
  - `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`
  - `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`
  - `crates/z00z_runtime/validators/src/checkpoint.rs`
  - `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
  - `crates/z00z_runtime/watchers/src/evidence_export.rs`
  - `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`
  - `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
  - `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
  - `crates/z00z_simulator/tests/test_hjmt_runtime_config.rs`
  - `crates/z00z_simulator/tests/test_scenario_settlement.rs`
  - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

### 🧪 Verification Model

#### 🎯 Critical User Journeys

- Canonical publication objects remain singular and byte-canonical across the
  storage-to-runtime handoff.
  - Why it matters: Phase 057 treats `ShardRootLeafV1` and
    `CheckpointPublicationV1` as live contract objects, not future design
    notes.
  - Evidence: `057-TEST-SPEC.md` gate map, `057-VALIDATION.md` task rows
    `057-TT-01` and `057-TT-02`,
    `crates/z00z_storage/src/settlement/proof_batch.rs`,
    `crates/z00z_storage/tests/test_hjmt_root_generation.rs`,
    `crates/z00z_runtime/aggregators/tests/test_hjmt_publish.rs`.
- Publication proof stays layered above shard-local proof truth.
  - Why it matters: the phase forbids flattening storage proof semantics into a
    second runtime-owned proof layer.
  - Evidence: `057-TEST-SPEC.md` scenarios `057-SC-03` and `057-SC-04`,
    `crates/z00z_storage/src/settlement/proof_batch_verify.rs`,
    `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`.
- Runtime publication lineage, downstream reuse, and simulator traces stay on
  one canonical path.
  - Why it matters: the packet explicitly forbids second publication, validator,
    watcher, or simulator truth lanes.
  - Evidence: `057-CONTEXT.md` ownership map,
    `crates/z00z_runtime/aggregators/src/service.rs`,
    `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs`,
    `crates/z00z_runtime/validators/src/checkpoint.rs`,
    `crates/z00z_runtime/watchers/src/evidence_export.rs`,
    `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`.
- Join, transfer, carry-forward, and crash recovery stay lineage-bound and
  fail closed.
  - Why it matters: route-generation transfer and byte-identical carry-forward
    are explicit exit criteria in the phase packet.
  - Evidence: `057-TEST-SPEC.md` scenarios `057-SC-07` through `057-SC-10`,
    `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`,
    `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`,
    `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`,
    `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`.

#### ♻️ State Transitions

- Last pre-shard root -> first lawful shard leaf -> canonical checkpoint
  publication.
  - Preconditions: live storage root-generation and route digest are present.
  - Postconditions: one ordered leaf set, one prior-root story, one
    publication digest.
  - Evidence path: `test_hjmt_root_generation.rs`,
    `test_hjmt_publish.rs`, `proof_batch.rs`.
- Old topology -> standby join -> route generation `N+1` -> owner activation.
  - Preconditions: YAML topology delta and join-mode fields are loaded from
    disk.
  - Postconditions: standby mirrors lineage without owner authority, owner
    activates only after committed generation advance.
  - Evidence path: `test_hjmt_join.rs`,
    `test_hjmt_preflight.rs`, `057-TEST-SPEC.md`.
- Old route table -> new route table -> carry-forward or crash recovery ->
  one lawful publication outcome.
  - Preconditions: live route change, recovery state, and failover fixtures
    exist.
  - Postconditions: unchanged carried-forward bytes stay identical and no
    ambiguous post-crash public checkpoint appears.
  - Evidence path: `test_hjmt_migrate.rs`,
    `test_hjmt_failover_same_lineage.rs`,
    `crates/z00z_storage/src/settlement/test_live_recovery.rs`.
- Canonical publication -> validator acceptance -> watcher export ->
  simulator trace packet.
  - Preconditions: runtime-owned `PublicationBinding` exists.
  - Postconditions: one shared binding digest, one shared verdict map, and
    one trace lineage story.
  - Evidence path: `validators/src/checkpoint.rs`,
    `watchers/src/evidence_export.rs`,
    `watchers/tests/test_hjmt_publication_contract.rs`,
    `simulator/tests/test_scenario_settlement.rs`,
    `simulator/tests/test_scenario1_stage_surface.rs`.

#### 🔐 Proof Paths

- `ShardRootLeafV1` canonical-bytes path.
  - Statement: leaf bytes must round-trip canonically and reject generation,
    route-digest, policy-set, and mutation drift.
  - Evidence: `proof_batch.rs:493-604`,
    `proof_batch_verify.rs:75-82`,
    `test_hjmt_root_generation.rs`.
- `CheckpointPublicationV1` canonical ordered-publication path.
  - Statement: publication must enforce ascending `ShardId`, one route digest,
    prior-root continuity, and monotonic successor semantics.
  - Evidence: `proof_batch.rs:608-760`,
    `test_hjmt_root_generation.rs`,
    `test_hjmt_publish.rs`.
- Runtime-owned `PublicationBinding` path.
  - Statement: `bind_publication_contract(...)` is the only runtime-owned
    binding constructor and downstream consumers must not derive a second local
    digest.
  - Evidence: `service.rs:41-47`,
    `test_live_guardrails.rs:113-187`,
    `validators/src/checkpoint.rs:46-55`,
    `watchers/src/evidence_export.rs:45-55`.
- Trace-home lineage path.
  - Statement: `leaf_flow.json` through `watch_flow.json` remain evidence only
    and must resolve back to the same runtime/config/process/journal lineage.
  - Evidence: `057-SOURCE-AUDIT.md`,
    `runtime_observability.rs:887-955`,
    `test_scenario_settlement.rs`,
    `test_scenario1_stage_surface.rs`.

#### ⚠️ Failure Paths

- Generation confusion, wrong-lineage, cross-shard, and publication-order
  tamper rows must reject.
  - Expected behavior: explicit fail-closed verdicts rather than alternate
    success paths.
  - Validation artifact: `test_hjmt_root_generation.rs`,
    `test_hjmt_historical_proofs.rs`, `test_hjmt_publish.rs`.
- Pre-activation owner, stale route digest, malformed proofs, and unordered
  handoff rows must reject at node/runtime seams.
  - Expected behavior: preflight or runtime rejection with no silent reroute.
  - Validation artifact: `test_hjmt_join.rs`,
    `test_hjmt_migrate.rs`, `test_hjmt_preflight.rs`.
- Missing or tampered trace artifacts must reject stage-surface verification.
  - Expected behavior: missing trace files, tampered path fields, tampered
    scope flags, and tampered recovery checks must fail the simulator guardrail
    suite.
  - Validation artifact: `test_scenario1_stage_surface.rs`.
- Phase-owned audit surfaces must not bypass `z00z_utils` abstractions where a
  project wrapper exists.
  - Expected behavior: no direct `std::fs::remove_file` on the audited Phase 057
    simulator verification surfaces.
  - Validation artifact: `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`,
    targeted grep checks, and the fixed simulator files in this audit run.

### 📊 Findings Summary

| Severity | Count | Meaning |
| --- | ---: | --- |
| 🔴 CRITICAL | 0 | Immediate correctness, security, or truthfulness failure |
| 🟠 HIGH | 0 | Material gap or blocker on required phase behavior |
| 🟡 MEDIUM | 1 | Non-trivial weakness, drift, or incomplete evidence |
| 🔵 LOW | 1 | Minor closure gap outside direct code ownership |
| ⚪ INFO | 8 | Confirmed observations with no immediate remediation |

Initial audit result: the live Phase 057 code already preserved the single
publication-binding path, storage-owned proof truth, route-generation-bound
transitions, downstream digest sameness, and bench-home ownership. One
phase-owned simulator verification drift remained: two test-only deletion sites
still bypassed the repository I/O abstraction and reintroduced a second file
deletion path into the same guardrail surface the phase treats as authority.

### 🔍 Audit Pass Results

#### 📦 z00z_storage

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected:
    `proof_batch.rs`, `proof_batch_verify.rs`,
    `test_hjmt_root_generation.rs`, `test_hjmt_historical_proofs.rs`,
    `test_hjmt_scope_birth.rs`
  - What was checked: `ShardRootLeafV1`, `CheckpointPublicationV1`,
    public-root replay, layered proof composition, scope-birth continuity.
  - Findings:
    - `⚪ INFO`: storage still owns canonical shard-root and proof truth, and
      the live code enforces the exact Phase 057 publication-object contracts.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected:
    `test_hjmt_root_generation.rs`, `test_hjmt_historical_proofs.rs`,
    `test_hjmt_scope_birth.rs`, `test_bench_lanes.rs`,
    `test_live_guardrails.rs`
  - What was checked: wrong-lineage rejects, cross-shard rejects, first-scope
    continuity, second-harness bans, bench-home guardrails.
  - Findings:
    - `⚪ INFO`: no storage-local integrity or replay gap remained on the
      phase-owned storage seam.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `057-CONTEXT.md`, `057-TEST-SPEC.md`, `057-VALIDATION.md`,
    `proof_batch.rs`, `proof_batch_verify.rs`,
    `test_hjmt_root_generation.rs`, `test_hjmt_historical_proofs.rs`
  - What was checked: root-generation, canonical publication, public proof
    layering, first-scope continuity, bench-home ownership.
  - Findings:
    - `⚪ INFO`: storage-side live anchors matched the phase packet with no
      shadow publication or proof owner.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `proof_batch.rs`, `proof_batch_verify.rs`,
    `test_hjmt_root_generation.rs`, `test_hjmt_historical_proofs.rs`,
    `test_bench_lanes.rs`, `test_live_guardrails.rs`
  - What was checked: no direct low-level file I/O, raw time, or second bench
    authority pattern on the phase-owned storage surface.
  - Findings:
    - `⚪ INFO`: no Phase 057 storage-local design-foundation drift remained.
  - Exact fixes required: none.

#### 📦 z00z_aggregators

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected:
    `service.rs`, `types.rs`,
    `test_hjmt_publish.rs`, `test_hjmt_join.rs`,
    `test_hjmt_migrate.rs`, `test_hjmt_failover_same_lineage.rs`
  - What was checked: single publication-binding constructor, route digest
    binding, join/transfer/carry-forward lineage, failover continuity.
  - Findings:
    - `⚪ INFO`: runtime publication binding and transition ownership stayed on
      the existing aggregator seam with no duplicate constructor path.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected:
    `test_hjmt_join.rs`, `test_hjmt_migrate.rs`,
    `test_hjmt_failover_same_lineage.rs`, `test_live_guardrails.rs`
  - What was checked: pre-activation reject rows, silent-reroute bans,
    failover byte identity, second-binding guardrails.
  - Findings:
    - `⚪ INFO`: no aggregator-side second authority or transfer-integrity gap
      remained.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `057-04-PLAN.md`, `057-05-PLAN.md`, `057-06-PLAN.md`,
    `service.rs`, `test_hjmt_publish.rs`, `test_hjmt_join.rs`,
    `test_hjmt_migrate.rs`, `test_live_guardrails.rs`
  - What was checked: canonical publication request assembly, join state
    separation, route-generation transfer, carry-forward continuity.
  - Findings:
    - `⚪ INFO`: live aggregator paths matched the numbered plan packet.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `service.rs`, `types.rs`, `test_live_guardrails.rs`
  - What was checked: no direct file-I/O/time bypass, no serde-owned binding
    construction path, no second digest lane.
  - Findings:
    - `⚪ INFO`: no Phase 057 aggregator-local design-foundation issue was
      found.
  - Exact fixes required: none.

#### 📦 z00z_rollup_node

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected:
    `test_hjmt_preflight.rs`
  - What was checked: publication handoff validation, route digest coverage,
    root-generation and proof-version acceptance boundaries.
  - Findings:
    - `⚪ INFO`: the node remained a validation and composition seam only; it
      did not become a second publication or proof authority.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected:
    `test_hjmt_preflight.rs`
  - What was checked: wrong route digest, wrong journal lineage, malformed
    proof bytes, unordered handoff, unsupported backend generation.
  - Findings:
    - `⚪ INFO`: preflight reject rows stayed fail closed on the current live
      node seam.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `057-03-PLAN.md`, `057-04-PLAN.md`, `test_hjmt_preflight.rs`
  - What was checked: node preflight still proves route-generation and handoff
    requirements named by the packet.
  - Findings:
    - `⚪ INFO`: live node verification aligns with the phase-owned preflight
      requirements.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `test_hjmt_preflight.rs`
  - What was checked: use of repo wrappers and no low-level bypass on the
    audited node surface.
  - Findings:
    - `⚪ INFO`: no design-foundation drift found on the Phase 057 node seam.
  - Exact fixes required: none.

#### 📦 z00z_validators

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected:
    `src/checkpoint.rs`, `tests/test_hjmt_publication_contract.rs`
  - What was checked: validator checkpoint flow reuses runtime-owned binding
    instead of reconstructing a local publication digest.
  - Findings:
    - `⚪ INFO`: validator acceptance remained bound to the canonical
      `PublicationBinding` path.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected:
    `src/checkpoint.rs`, `tests/test_hjmt_publication_contract.rs`
  - What was checked: downstream verdict mapping, batch/checkpoint identity
    reconciliation, wrong-lineage rejection.
  - Findings:
    - `⚪ INFO`: no validator-side digest-fork or verdict-drift issue remained.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `057-05-PLAN.md`, `057-VALIDATION.md`,
    `src/checkpoint.rs`, `tests/test_hjmt_publication_contract.rs`
  - What was checked: `057-G9` and `057-G10` downstream sameness requirements.
  - Findings:
    - `⚪ INFO`: validator evidence stayed aligned with the phase packet.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `src/checkpoint.rs`, `tests/test_hjmt_publication_contract.rs`
  - What was checked: no low-level I/O or local digest-construction bypass.
  - Findings:
    - `⚪ INFO`: no Phase 057 validator-local design-foundation issue was
      found.
  - Exact fixes required: none.

#### 📦 z00z_watchers

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected:
    `src/evidence_export.rs`, `tests/test_hjmt_publication_contract.rs`
  - What was checked: watcher evidence reuses validator/runtime binding and
    exposes only the shared binding digest.
  - Findings:
    - `⚪ INFO`: watcher evidence export remained a consumer of the canonical
      publication contract, not a second digest owner.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected:
    `src/evidence_export.rs`, `tests/test_hjmt_publication_contract.rs`
  - What was checked: binding mismatch rejection, snapshot integrity, runtime
    placement preference.
  - Findings:
    - `⚪ INFO`: watcher publication watch still rejects local binding drift and
      keeps one verdict story.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `057-05-PLAN.md`, `057-TEST-SPEC.md`,
    `src/evidence_export.rs`, `tests/test_hjmt_publication_contract.rs`
  - What was checked: watcher-side `057-SC-11` continuity and downstream digest
    sameness.
  - Findings:
    - `⚪ INFO`: watcher live behavior matched the phase-owned acceptance rules.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `src/evidence_export.rs`, `tests/test_hjmt_publication_contract.rs`
  - What was checked: no local digest constructor and no direct low-level
    bypass on the audited watcher surface.
  - Findings:
    - `⚪ INFO`: no Phase 057 watcher-local design-foundation issue was found.
  - Exact fixes required: none.

#### 📦 z00z_simulator

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected:
    `src/scenario_1/runtime_observability.rs`,
    `src/scenario_1/runner_verify.rs`,
    `tests/test_hjmt_runtime_config.rs`,
    `tests/test_scenario_settlement.rs`,
    `tests/test_scenario1_stage_surface.rs`
  - What was checked: topology-generic `SIM-5A7S-PUB` support, trace-home
    linkage, runtime-owned publication binding reuse, evidence-only trace
    semantics.
  - Findings:
    - `⚪ INFO`: simulator traces and publication evidence still reuse the
      canonical runtime-owned contract and do not replace semantic truth.
  - Exact fixes required: none from the crypto-architect lens.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected:
    `runtime_observability.rs`, `runner_verify.rs`,
    `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`
  - What was checked: missing-trace rejects, tampered-path rejects,
    tampered-scope rejects, tampered-recovery rejects, reserved-profile rejects.
  - Findings:
    - `⚪ INFO`: the live stage-surface suite continued to fail closed on trace
      and evidence tamper scenarios.
  - Exact fixes required: none from the security lens.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `057-03-PLAN.md`, `057-05-PLAN.md`, `057-06-PLAN.md`,
    `057-07-PLAN.md`, `runtime_observability.rs`, `runner_verify.rs`,
    `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`
  - What was checked: trace-home contract names, live `scenario_1` successor
    path, config/design/runtime evidence sync, and `057-G11` continuity.
  - Findings:
    - `⚪ INFO`: simulator live homes matched the packet’s trace and scenario
      authority claims.
  - Exact fixes required: none from the spec-to-code lens.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected:
    `runner_verify.rs`, `test_scenario1_stage_surface.rs`,
    `runtime_observability.rs`

#### 🟡 Phase-Owned Simulator Verification Surfaces Bypassed The Repository I/O Abstraction

**Location:** `crates/z00z_simulator/src/scenario_1/runner_verify.rs:2105`,
`crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:1668`

**Issue:**

```rust
std::fs::remove_file(&metrics_path).expect("remove metrics report");
std::fs::remove_file(out_dir.join("proc_flow.json")).expect("remove proc_flow");
```

**Why This is Critical:**
Phase 057 treats these simulator verification surfaces as canonical guardrails
for trace-home truth. The Design Foundation requires one centralized file-I/O
abstraction through `z00z_utils::io::*`. Leaving direct `std::fs::remove_file`
calls in the same audited surfaces reintroduced a second file-deletion path
exactly where the phase claims design-foundation compliance and single-path
authority. This was not a runtime consensus bug, but it was a material
architecture drift in the phase-owned evidence seam.

**Recommendation:**

```rust
io::remove_file(&metrics_path).expect("remove metrics report");
remove_file(out_dir.join("proc_flow.json")).expect("remove proc_flow");
```

**Severity:** 🟡 Medium
**Category:** Code Quality
**Proof Status:** Full Evidence
**Verification:** VERIFIED

- Exact fixes required:
  - replace direct `std::fs::remove_file` in `runner_verify.rs` with the
    existing `z00z_utils::io::remove_file` wrapper;
  - import and use the same wrapper in
    `test_scenario1_stage_surface.rs`;
  - rerun bootstrap and the touched simulator release lanes after the fix.

## ⚙️ Fixes Applied — 2026-06-14 15:57:42

- Fixed the only actionable code finding from this audit:
  - `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
    now uses `io::remove_file(&metrics_path)` instead of
    `std::fs::remove_file(&metrics_path)`.
  - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
    now imports `remove_file` from `z00z_utils::io` and uses it for the
    missing-trace tamper row.
- Files changed:
  - `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
  - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

> [!IMPORTANT]
> Only the post-bootstrap reruns below are counted as closure evidence. An
> earlier queued cargo attempt under build-lock contention was discarded from
> the audit narrative to keep the rerun story truthful.

- Remaining blocked findings:
  - none in live code.
- Remaining broader gap:
  - `057-UAT.md` is still an open conversational ledger with
    `status: testing`; this is outside direct code-fix scope and remains in the
    final summary table as a low-severity closeout gap.

## ♻️ Re-Audit Results — 2026-06-14 16:09:23

- Re-ran the same crate list and the same four manual-fallback audit lenses
  after the simulator fix.
- Exact commands and results used for post-fix verification:
  - `cargo fmt`
    - completed successfully; only repository rustfmt nightly-option warnings
      were emitted.
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
    - passed with the final banner `=== BOOTSTRAP COMPLETE ===`.
  - `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools s13_metrics_missing_rejects -- --nocapture`
    - passed with `1 passed; 0 failed`.
  - `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture`
    - passed with `22 passed; 0 failed` in `1061.75s`.
- Targeted re-audit disposition:

| Surface | Prior finding | Current status | Evidence |
| --- | --- | --- | --- |
| `runner_verify.rs` phase-owned unit seam | direct `std::fs::remove_file` bypass | fixed | live diff plus `s13_metrics_missing_rejects` green |
| `test_scenario1_stage_surface.rs` phase-owned integration seam | direct `std::fs::remove_file` bypass | fixed | live diff plus `22 passed; 0 failed` green |
| broader simulator trace-home contract | none | still verified | stage-surface suite preserved missing-trace and tamper rejects |
| storage/runtime/node/validator/watcher scopes | none | still verified | source inspection, guardrail checks, and Phase 057 validation matrix remain aligned |

- Re-audit conclusion:
  - the simulator design-foundation drift is closed;
  - no new critical, high, or medium issue appeared on the same scope;
  - the six phase-owned crate homes remain aligned to one canonical authority
    path.

## ✅ Doublecheck Results — 2026-06-14 16:09:23

- `doublecheck` mode: `manual fallback`
- Manual method used:
  - Layer 1 self-audit:
    - extracted every factual claim added by this audit report about scope,
      findings, file locations, fixes, and rerun results;
    - checked the report for internal contradictions, especially around scope
      ownership and rerun ordering.
  - Layer 2 workspace-first verification:
    - rechecked scope claims against `057-CONTEXT.md`,
      `057-SOURCE-AUDIT.md`, `057-TEST-SPEC.md`, and `057-VALIDATION.md`;
    - rechecked finding locations against the live diff in
      `runner_verify.rs` and `test_scenario1_stage_surface.rs`;
    - rechecked rerun evidence against captured command output including
      `=== BOOTSTRAP COMPLETE ===`, `1 passed; 0 failed`, and
      `22 passed; 0 failed`.
  - Layer 3 adversarial review:
    - looked for unsupported scope widening, invented crate ownership, false
      closure claims, and any report wording that implied the open UAT ledger
      was already complete.
- Doublecheck findings:
  - no unsupported code-closure claims remained;
  - no invented crate scope, file path, or command result remained;
  - one broader non-code closeout gap still exists and remains explicit:
    `057-UAT.md` is still open.

> [!CAUTION]
> This audit closes actionable code findings on the live tree. It does not
> replace the still-open conversational UAT ledger.

## 🧾 Exact Fixes Required Summary

| Q | Title | Proof Status | Verification | Severity | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Phase-owned simulator verification surfaces bypassed the repository I/O abstraction | Full Evidence | VERIFIED | 🟡 MEDIUM | None after this audit run | Completed in this audit run by replacing direct `std::fs::remove_file` with `z00z_utils::io::remove_file` on both affected surfaces |
| 2 | Conversational UAT ledger remains open | Full Evidence | PARTIAL | 🔵 LOW | `.planning/phases/057-HJMT-multi-aggregator/057-UAT.md` is still `status: testing` with 8 pending checks | Finish `/gsd-verify-work` and update `057-UAT.md` with actual pass/fail outcomes |
| 3 | Canonical publication-binding path remains singular across runtime, validators, watchers, and simulator traces | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 4 | Storage-owned proof truth and accepted bench-home ownership remain intact | Full Evidence | VERIFIED | ⚪ INFO | None | None |

## 🚩 Final Status

All actionable code findings found by this audit were fixed directly on the
live tree. Post-fix reruns and manual `doublecheck` found no remaining
critical, high, or medium code issue in the Phase 057 six-crate scope.

One low-severity closeout gap remains outside direct code ownership:
`057-UAT.md` still records an open conversational verification session. Phase
057 code-audit closure is therefore complete, while overall phase-closeout
evidence still depends on finishing that UAT ledger honestly.

## 🔔 Audit Run — 2026-06-14 16:14:27

### 📌 Audit Setup

- Phase directory: `.planning/phases/057-HJMT-multi-aggregator`
- Derived FULL-AUDIT path:
  `.planning/phases/057-HJMT-multi-aggregator/057-FULL-AUDIT.md`
- Mandatory context re-read:
  - `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
  - `.github/copilot-instructions.md`
  - `.github/prompts/gsd-audit-4.prompt.md`
  - `.github/prompts/references/gsd-audit-4-full-audit-report-format.md`
- Current worktree state before this rerun:
  - modified:
    `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
  - modified:
    `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
  - untracked:
    `.planning/phases/057-HJMT-multi-aggregator/057-FULL-AUDIT.md`
- Execution mode: append-only rerun over the already-validated Phase 057 audit
  fix state, with manual fallback for all four audit passes and manual
  `doublecheck` fallback.

> [!IMPORTANT]
> Final in-scope crate list remained unchanged for this rerun:
> `z00z_storage`, `z00z_aggregators`, `z00z_rollup_node`,
> `z00z_validators`, `z00z_watchers`, `z00z_simulator`.

- Explicitly excluded crates or modules remained unchanged:
  - `z00z_crypto/tari`
  - `z00z_core`
  - `z00z_utils`
  - `z00z_wallets`
  - non-phase-owned workspace crates outside the Phase 057 packet

### 🎯 Scope And Source Of Truth

- This rerun reused the same phase authority packet and owner map proven in the
  2026-06-14 15:43:28 audit run above.
- Scope was rechecked against:
  - `057-CONTEXT.md` ownership map and live anchors
  - `057-SOURCE-AUDIT.md` live path corrections
  - `057-VALIDATION.md` six-crate command matrix
  - `057-UAT.md` open conversational ledger
- No new live-code delta appeared outside the already-audited simulator fix and
  this append-only audit artifact.

### 🧪 Verification Model

#### 🎯 Critical User Journeys

- Canonical publication objects remain singular across the storage-to-runtime
  seam.
  - Evidence rechecked: `057-VALIDATION.md` task rows `057-TT-01` and
    `057-TT-02`, prior-run live storage/runtime anchors, and current dirty tree
    diff showing no new storage or aggregator code drift.
- Runtime publication binding remains singular across runtime, validators,
  watchers, and simulator traces.
  - Evidence rechecked: `057-CONTEXT.md` ownership map,
    `057-SOURCE-AUDIT.md`, and current live simulator/validator/watcher source
    checks.
- Simulator trace-home guardrails remain evidentiary only and tied to the same
  lineage packet.
  - Evidence rechecked: current simulator source, prior validated rerun
    evidence in this file, and no new simulator code delta beyond the already
    validated I/O-wrapper fix.

#### ♻️ State Transitions

- No new transition implementation was introduced since the prior validated
  run.
  - Evidence path: current `git status --short`, current `git diff`, and the
    unchanged six-crate validation packet.
- The only live-code delta still present is the previously validated
  `remove_file` wrapper usage in the two Phase 057 simulator verification
  surfaces.
  - Evidence path: current `rg` on `runner_verify.rs` and
    `test_scenario1_stage_surface.rs`.

#### 🔐 Proof Paths

- `bind_publication_contract(...)` remains the only runtime-owned
  `PublicationBinding` constructor path.
  - Evidence path: prior-run guardrail proof plus current report and source
    consistency checks.
- `ShardRootLeafV1` and `CheckpointPublicationV1` remain the storage-owned
  canonical publication objects.
  - Evidence path: unchanged storage anchors in `057-CONTEXT.md`,
    `057-SOURCE-AUDIT.md`, and no new storage diff in the current worktree.

#### ⚠️ Failure Paths

- No new bypass reopened the simulator missing-trace and tamper-reject lanes.
  - Evidence path: previously validated `test_scenario1_stage_surface`
    suite results already captured in this file, plus unchanged simulator code
    since that rerun.
- The only still-open broader gap remains the non-code conversational UAT
  ledger.
  - Evidence path: `057-UAT.md` frontmatter and summary counts.

### 📊 Findings Summary

| Severity | Count | Meaning |
| --- | ---: | --- |
| 🔴 CRITICAL | 0 | Immediate correctness, security, or truthfulness failure |
| 🟠 HIGH | 0 | Material gap or blocker on required phase behavior |
| 🟡 MEDIUM | 0 | Non-trivial weakness, drift, or incomplete evidence |
| 🔵 LOW | 1 | Minor closure gap outside direct code ownership |
| ⚪ INFO | 3 | Confirmed observations with no immediate remediation |

Second audit-run result: no new actionable code finding appeared after the
previous validated simulator fix. The only remaining open item is the same
low-severity non-code closeout gap: `057-UAT.md` is still open.

### 🔍 Audit Pass Results

#### 📦 z00z_storage

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected: prior-run storage surfaces plus current `git diff`
    confirmation of no storage changes.
  - What was checked: canonical publication-object and proof-truth ownership
    did not drift.
  - Findings:
    - `⚪ INFO`: no new storage-side drift appeared.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected: prior-run storage test and guardrail surfaces plus current
    dirty-tree diff.
  - What was checked: no new replay, ordering, or second-harness weakness was
    introduced.
  - Findings:
    - `⚪ INFO`: storage-side fail-closed coverage remains intact.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected: `057-CONTEXT.md`, `057-SOURCE-AUDIT.md`,
    `057-VALIDATION.md`, and current storage diff state.
  - What was checked: Phase 057 storage scope remained aligned with the packet.
  - Findings:
    - `⚪ INFO`: no storage-side spec drift appeared.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected: prior-run storage anchors plus current dirty-tree diff.
  - What was checked: no new low-level bypass was introduced on the
    phase-owned storage seam.
  - Findings:
    - `⚪ INFO`: no new design-foundation issue appeared under storage.
  - Exact fixes required: none.

#### 📦 z00z_aggregators

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected: prior-run aggregator surfaces plus current `git diff`
    confirmation of no aggregator changes.
  - What was checked: singular publication-binding path and transition
    ownership remained unchanged.
  - Findings:
    - `⚪ INFO`: no new aggregator-side drift appeared.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected: prior-run transition and guardrail surfaces plus current
    dirty-tree diff.
  - What was checked: no new silent-reroute, split-authority, or transfer
    integrity issue appeared.
  - Findings:
    - `⚪ INFO`: aggregator security posture remained unchanged.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected: `057-04-PLAN.md`, `057-05-PLAN.md`, `057-06-PLAN.md`,
    and current aggregator diff state.
  - What was checked: runtime publication, join, and migration scope remained
    packet-aligned.
  - Findings:
    - `⚪ INFO`: no aggregator-side spec drift appeared.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected: prior-run aggregator anchors plus current dirty-tree diff.
  - What was checked: no new abstraction bypass or local digest path appeared.
  - Findings:
    - `⚪ INFO`: no new design-foundation issue appeared under aggregators.
  - Exact fixes required: none.

#### 📦 z00z_rollup_node

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected: prior-run node preflight surfaces plus current `git diff`
    confirmation of no node changes.
  - What was checked: node stayed a validation/composition seam only.
  - Findings:
    - `⚪ INFO`: no new node-side drift appeared.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected: prior-run node reject-path surfaces plus current diff
    state.
  - What was checked: no new malformed-proof, wrong-lineage, or unordered
    handoff acceptance path appeared.
  - Findings:
    - `⚪ INFO`: node preflight reject coverage remained unchanged.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected: `057-03-PLAN.md`, `057-04-PLAN.md`,
    `057-VALIDATION.md`, and current node diff state.
  - What was checked: node scope remained aligned with the phase packet.
  - Findings:
    - `⚪ INFO`: no node-side spec drift appeared.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected: prior-run node anchors plus current dirty-tree diff.
  - What was checked: no new low-level bypass was introduced on the node seam.
  - Findings:
    - `⚪ INFO`: no new design-foundation issue appeared under node scope.
  - Exact fixes required: none.

#### 📦 z00z_validators

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected: prior-run validator surfaces plus current `git diff`
    confirmation of no validator changes.
  - What was checked: validator reuse of runtime-owned publication binding
    remained unchanged.
  - Findings:
    - `⚪ INFO`: no new validator-side drift appeared.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected: prior-run validator acceptance surfaces plus current diff
    state.
  - What was checked: no local binding fork or verdict-mapping drift appeared.
  - Findings:
    - `⚪ INFO`: validator rejection and digest sameness remained unchanged.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected: `057-05-PLAN.md`, `057-VALIDATION.md`,
    and current validator diff state.
  - What was checked: validator acceptance stayed aligned to `057-G9` and
    `057-G10`.
  - Findings:
    - `⚪ INFO`: no validator-side spec drift appeared.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected: prior-run validator anchors plus current dirty-tree diff.
  - What was checked: no new low-level bypass or local digest constructor
    appeared.
  - Findings:
    - `⚪ INFO`: no new design-foundation issue appeared under validators.
  - Exact fixes required: none.

#### 📦 z00z_watchers

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected: prior-run watcher surfaces plus current `git diff`
    confirmation of no watcher changes.
  - What was checked: watcher evidence remained a consumer of the shared
    publication-binding path.
  - Findings:
    - `⚪ INFO`: no new watcher-side drift appeared.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected: prior-run watcher evidence surfaces plus current diff
    state.
  - What was checked: no watcher-local digest or verdict fork appeared.
  - Findings:
    - `⚪ INFO`: watcher reject and export semantics remained unchanged.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected: `057-05-PLAN.md`, `057-TEST-SPEC.md`,
    and current watcher diff state.
  - What was checked: watcher-side continuity and evidence-export scope stayed
    packet-aligned.
  - Findings:
    - `⚪ INFO`: no watcher-side spec drift appeared.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected: prior-run watcher anchors plus current dirty-tree diff.
  - What was checked: no new abstraction bypass or local digest path appeared.
  - Findings:
    - `⚪ INFO`: no new design-foundation issue appeared under watchers.
  - Exact fixes required: none.

#### 📦 z00z_simulator

- `crypto-architect`
  - Status: `manual fallback`
  - Files inspected: `runner_verify.rs`,
    `test_scenario1_stage_surface.rs`, prior-run simulator anchors, and the
    current dirty-tree diff.
  - What was checked: the previously fixed simulator deletion-path drift stayed
    closed and no new trace-home authority fork appeared.
  - Findings:
    - `⚪ INFO`: the validated simulator fix remains present and no new
      simulator-side drift appeared.
  - Exact fixes required: none.
- `security-audit`
  - Status: `manual fallback`
  - Files inspected: prior-run simulator guardrail surfaces plus current diff
    state.
  - What was checked: no new missing-trace or tamper-acceptance path was
    introduced.
  - Findings:
    - `⚪ INFO`: simulator guardrail coverage remains unchanged from the
      previously validated state.
  - Exact fixes required: none.
- `spec-to-code-compliance`
  - Status: `manual fallback`
  - Files inspected: `057-03-PLAN.md`, `057-05-PLAN.md`,
    `057-06-PLAN.md`, `057-07-PLAN.md`, current simulator diff state.
  - What was checked: live `scenario_1` trace-home and design-sync scope
    remained aligned with the phase packet.
  - Findings:
    - `⚪ INFO`: no simulator-side spec drift appeared.
  - Exact fixes required: none.
- `z00z-design-foundation-compliance`
  - Status: `manual fallback`
  - Files inspected: `runner_verify.rs`,
    `test_scenario1_stage_surface.rs`, and current dirty-tree diff.
  - What was checked: the two Phase 057 simulator deletion sites still route
    through `z00z_utils::io` rather than direct `std::fs`.
  - Findings:
    - `⚪ INFO`: the prior design-foundation finding stayed closed.
  - Exact fixes required: none.

## ⚙️ Fixes Applied — 2026-06-14 16:14:27

- No new code fix was required in this rerun.
- The current dirty state still consists of:
  - the already-validated simulator I/O-wrapper fix;
  - the append-only `057-FULL-AUDIT.md` artifact itself.
- No broader blocker was closed in this rerun.
- The open UAT ledger remains unchanged and stays carried into the final
  summary table.

## ♻️ Re-Audit Results — 2026-06-14 16:14:27

- Re-ran the same six-crate manual-fallback audit lenses on the current
  worktree.
- Because this rerun introduced no new code delta beyond the already-validated
  simulator fix and the audit artifact itself, no fresh cargo rerun was needed
  to keep the report truthful.
- Reused still-applicable verification evidence from the immediately preceding
  validated state because:
  - current `git status --short` showed no new code delta outside the same two
    simulator files;
  - current source checks confirmed the fixed wrapper usage still exists:
    `io::remove_file(&metrics_path)` and
    `remove_file(out_dir.join("proc_flow.json"))`;
  - the prior run in this file already captured green bootstrap and targeted
    release evidence for that exact code state.
- Current rerun disposition:

| Surface | Current status | Evidence |
| --- | --- | --- |
| simulator wrapper fix | still closed | current `rg` confirms wrapper calls remain present |
| six-crate scope ownership | still verified | current packet cross-read plus no new code delta outside simulator fix |
| conversational UAT ledger | still open | `057-UAT.md` still shows `status: testing`, `total: 8`, `pending: 8` |

## ✅ Doublecheck Results — 2026-06-14 16:14:27

- `doublecheck` mode: `manual fallback`
- Manual method used:
  - Layer 1 self-audit:
    - checked this second audit-run text for new unsupported closure claims;
    - checked that it does not pretend a new cargo rerun happened when it did
      not.
  - Layer 2 workspace-first verification:
    - rechecked current `git status --short`;
    - rechecked the current simulator wrapper calls with `rg`;
    - rechecked the open UAT ledger fields in `057-UAT.md`.
  - Layer 3 adversarial review:
    - looked for scope widening, invented new fixes, or false claims that UAT
      was complete.
- Doublecheck findings:
  - no unsupported new code-fix claim was found;
  - no unsupported new verification-result claim was found;
  - the same low non-code closeout gap remains explicit.

## 🧾 Exact Fixes Required Summary

| Q | Title | Proof Status | Verification | Severity | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Conversational UAT ledger remains open | Full Evidence | PARTIAL | 🔵 LOW | `.planning/phases/057-HJMT-multi-aggregator/057-UAT.md` is still `status: testing` with `8` pending checks | Finish `/gsd-verify-work` and update `057-UAT.md` with actual pass/fail outcomes |
| 2 | Previously fixed simulator I/O abstraction drift remains closed | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 3 | Canonical six-crate authority path remains unchanged in the current rerun | Full Evidence | VERIFIED | ⚪ INFO | None | None |

## 🚩 Final Status

This repeat `GSD-Audit-4` rerun found no new actionable code issue beyond the
already-fixed and already-validated simulator design-foundation drift recorded
in the prior run above.

The only remaining open item is still outside direct code-fix scope:
`057-UAT.md` remains an open conversational verification ledger.
