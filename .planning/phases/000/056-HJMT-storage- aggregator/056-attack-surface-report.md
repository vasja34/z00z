# Phase 056 Attack Surface Report

**Rechecked:** 2026-06-12
**Doublecheck Mode:** workspace-first live-code skeptical verification
**Result:** the admitted runtime observability trace-pack validation candidate is now resolved in live code with regression coverage and closure evidence

## ✅ Scan Result

One candidate passed the pro-con audit and verification gate and is now closed in the live implementation.

## Attack Surface: Runtime observability validator accepts tampered trace-specific payload

**Status:** resolved
**Severity:** medium
**Confidence:** high
**Exploitability:** medium
**Category Domain:** validation
**Category CWE:** CWE-345
**Attack Class:** fail-open-validation
**Scope Level:** crate
**Scope Paths:** `crates/z00z_simulator`, `.planning/phases/056-HJMT-storage- aggregator`
**Boundary Slice:** external input and parser slice
**Protected Asset:** runtime observability trace-pack integrity and truthful Phase 056 evidence
**Trust Boundary:** emitted runtime trace JSON files -> validation and audit consumers
**Attacker Capability Model:** a malicious operator, CI artifact editor, or workspace user can modify emitted trace JSON files after the run and before validation or review
**Existing Control State:** historical partial control, now closed for the live validator path
**Main Vulnerability:** `validate_runtime_observability_artifacts()` accepts a runtime trace pack when shared header fields match the current spec, but it never recomputes or validates trace-specific payload fields, so body-only tampering survives the advertised validation gate.

### Threat Model Snapshot

- **Attacker Class:** malicious operator or artifact editor
- **Entry Point:** post-run JSON trace files such as `tx_flow.json`, `plan_flow.json`, `scope_flow.json`, and `recovery_flow.json`
- **Sink:** `runner::validate_runtime_observability_artifacts(...)` and downstream audit or review workflows that trust its verdict
- **Why This Path Is Realistic:** Phase 056 explicitly treats the trace pack as required acceptance evidence, the validator is a public entrypoint, and the traces are ordinary mutable JSON files under the scenario outputs directory

### Implementation Nuance

The current control surface looks stronger than it is because every trace carries `semantic_digest_hex`, `config_digest_set_hex`, `route_table_digest`, `process_topology_digest_hex`, and `journal_lineage_digest_hex`. In live code, however, `semantic_digest_hex` is derived only from the shared spec tuple and not from each trace body. The validator then checks only those shared fields plus `scenario_config_path`, `design_path`, and `hjmt_home`. It does not validate the fields that make each trace semantically meaningful, including transaction artifact paths, planner directories and limits, scope-owner markers, leak flags, failover owner homes, or required startup-check rows.

### Evidence

- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs:172` defines trace-specific payload surfaces in `TxFlow`, `RouteFlow`, `PlanFlow`, `JournalFlow`, `ScopeFlow`, and `RecoveryFlow`.
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs:402` emits those trace-specific fields, including `tx_package_path`, `planner_evidence_dir`, `semantic_owner`, `private_tree_id_exposed`, and `recovery_owner_homes`.
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs:531` starts validation, but the checks stop at shared header and digest fields plus three common paths.
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs:760` computes `semantic_digest_hex` only from the shared spec tuple and excludes every trace-specific body field.
- `.planning/phases/056-HJMT-storage- aggregator/056-TODO.md:585` requires every trace to resolve back to one config-digest set, one route-table digest, one journal-lineage view, and one process-topology view, and says missing, stale, or cross-linked traces fail the phase.
- `.planning/phases/056-HJMT-storage- aggregator/056-SECURITY.md:47` records `runner::validate_runtime_observability_artifacts(...)` as the closed control for `T-056-16` runtime observability trace linkage.

### Security Control Review

- **Controls Checked:** trace file path relativity, trace kind and mode, shared semantic digest, config digest set, route digest, process-topology digest, journal-lineage digest, and common config/design/home path equality
- **Why Existing Controls Are Insufficient:** the validator never authenticates or recomputes the trace-specific body. An attacker can keep the shared fields intact while changing fields such as `private_tree_id_exposed`, `semantic_owner`, `startup_checks_required`, `planner_evidence_dir`, or `tx_package_path`, and the current validation gate still returns success.

### Pro-Con Audit

**Pros**
- The gap is in production code on a public validation entrypoint rather than in docs or tests alone.
- The omitted fields are security-meaningful Phase 056 evidence, not cosmetic metadata.
- The checked `semantic_digest_hex` explicitly excludes the omitted body fields, so the bypass is structural rather than hypothetical.

**Cons**
- The attacker needs artifact-write access after the scenario run, not a remote unauthenticated edge.
- The impact is evidence-integrity and audit-truthfulness drift rather than direct live-state corruption or unauthorized settlement.

**Decision:** accepted

### Verification

**Gate:** passed
**Reason:** the candidate is backed by concrete production-code evidence, crosses a real artifact-to-validator trust boundary, and does not duplicate an existing accepted trace-pack finding in the current attack inventory.

### Defensive Implementation Contract

- Recompute and validate a per-trace digest over the full canonical body, not just the shared spec tuple.
- Compare trace-specific fields against independently recomputed expectations from `RuntimeTraceSpec` and the current output directory.
- Add tamper regression coverage proving that edits to `tx_package_path`, `planner_evidence_dir`, `private_tree_id_exposed`, `semantic_owner`, and `startup_checks_required` all fail `validate_runtime_observability_artifacts()`.

### Resolution Evidence

- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` now recomputes the full canonical payload for every runtime trace kind and rejects body drift instead of trusting shared header fields alone.
- The tx-trace path contract now uses the same runtime output remap semantics as the stage-4 producer, so `tx_package_path` resolves to the live runtime artifact instead of a config-shaped alias.
- `crates/z00z_simulator/src/test_support/stage13_shared_cases.rs` now rewrites stabilized shared Stage-13 trace packs so `scenario_config_path`, `config_digests[*].path`, and tx-flow output paths stay truthful after fixture promotion from `.tmp` roots to stable cache roots.
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` now contains explicit tamper regressions for `tx_package_path`, `planner_evidence_dir`, `private_tree_id_exposed`, and `startup_checks_required`, plus live assertions that `tx_flow.json` points at the real stage-4 and stage-13 artifacts.
- Validation re-ran on the hardened path with `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test -p z00z_simulator --release --test test_scenario1_stage_surface -- --nocapture`, and `cargo test --release`, all green.

### Residual Risk

Historical trace packs that previously passed validation may already contain undetected body-only tampering from the pre-fix validator. Live code is now closed, but pre-existing artifacts should still be treated as partially authenticated evidence unless they are regenerated or revalidated under the hardened contract.
