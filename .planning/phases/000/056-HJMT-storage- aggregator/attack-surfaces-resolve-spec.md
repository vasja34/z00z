# Attack Surfaces Resolve Spec

**Phase:** 056
**Date:** 2026-06-12
**Source DB:** `056-attack-surface-db.jsonl`
**Target Finding:** `AS-20260612-001`
**Status:** selected solution implemented

## ✅ Target Finding

- `AS-20260612-001`
- Title: `Runtime observability validator accepts tampered trace-specific payload`
- Boundary: emitted runtime trace JSON files -> validation and audit consumers
- Required closure: fail closed on trace-body tampering without weakening the existing Phase 056 evidence contract

## 📊 Ranked Candidates

| Rank | Candidate | Structural Validation | Adversarial Validation | Doublecheck | Net |
| --- | --- | --- | --- | --- | --- |
| 1 | Recompute the full canonical payload per trace kind, validate it against trusted inputs, and harden Stage-13 shared fixture stabilization | pass | pass | pass | positive |
| 2 | Add a new per-trace body digest field and reject mismatches while leaving the rest of the validator shape mostly intact | pass | pass-with-risk | pass-with-risk | positive |
| 3 | Introduce a signed trace-pack envelope with new signing keys and verification material | fail | fail | fail | negative |

## 🔍 Candidate Review

### Candidate 1

**Summary:** Rebuild the full expected payload for each trace kind from `RuntimeTraceSpec`, config, design, planner config, and the current output directory; reject any body drift; align tx-path resolution with live stage-4 runtime semantics; and keep stabilized shared Stage-13 fixtures truthful after promotion from `.tmp` roots.

**Pros**

- Closes the live fail-open gap on the existing public validator entrypoint.
- Reuses the current trace-pack format, so no migration is required for current code consumers.
- Adds direct regression coverage for the exact tamper surfaces named in the finding.
- Fixes the tx-trace path contract to point at the real runtime artifact rather than a config-shaped alias.

**Cons**

- Historical trace packs generated before the fix are still suspect and should be regenerated.
- The validator logic becomes stricter and more coupled to the canonical runtime artifact layout.

**Validation A:** pass
Reason: every trace kind now has a canonical expected payload derived from trusted inputs, and the shared Stage-13 cache remains self-consistent after stabilization.

**Validation B:** pass
Reason: body-only tampering of `tx_flow`, `plan_flow`, `scope_flow`, and `recovery_flow` now fails validation; the tx-path alias gap is also closed.

**Doublecheck:** pass
Reason: workspace evidence matches the implemented code and the validation commands are green.

### Candidate 2

**Summary:** Extend the trace schema with a new body digest per trace and reject mismatches while otherwise keeping the current validator shape.

**Pros**

- Strong cryptographic binding at the trace-body level.
- Easy to reason about during audit once deployed everywhere.

**Cons**

- Requires a schema change and migration path across every trace producer and consumer.
- Still needs independent canonical payload recomputation to avoid hashing the wrong values.
- Larger blast radius than the phase requires.

**Validation A:** pass
Reason: workable, but broader than required.

**Validation B:** pass-with-risk
Reason: without canonical payload recomputation, it can still bless semantically wrong-but-self-consistent trace bodies.

**Doublecheck:** pass-with-risk
Reason: the idea is valid, but the workspace does not need the schema migration to close the current finding.

### Candidate 3

**Summary:** Wrap trace packs in a new signed envelope and add signing key distribution plus signature verification.

**Pros**

- Strong authenticity model if deployed end to end.

**Cons**

- Out of scope for the current phase.
- Introduces new key-management and rollout complexity.
- Does not solve the immediate fixture-truthfulness and runtime-path issues on its own.

**Validation A:** fail
Reason: not a minimal remediation for the identified boundary.

**Validation B:** fail
Reason: creates additional secrets and rollout dependencies that the phase does not define.

**Doublecheck:** fail
Reason: the workspace has no supporting contract for a new trace-signing trust root.

## 🏁 Selected Solution

**Winner:** Candidate 1

**Selection Rationale:** It closes the real fail-open path on the existing validator, keeps the trace-pack format stable, fixes the live tx-path truthfulness bug, and hardens the stabilized shared Stage-13 fixture that the acceptance tests depend on. It also yields direct regression evidence against the exact tamper vectors named in the finding.

## 🛠️ Implemented Changes

- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
  - Validate each trace against a recomputed full canonical payload.
  - Normalize the output root during validation so equivalent paths cannot drift by raw `..` segments.
  - Resolve tx/output artifact paths with the same runtime marker-strip semantics used by the stage-4 producer.
- `crates/z00z_simulator/src/test_support/stage13_shared_cases.rs`
  - Rewrite `scenario_config_path` and `config_digests[*].path` for stabilized shared fixtures.
  - Rewrite tx-flow output paths from `.tmp` roots to the normalized stable cache root.
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
  - Add tamper regressions for `tx_package_path`, `planner_evidence_dir`, `private_tree_id_exposed`, and `startup_checks_required`.
  - Assert that live `tx_flow.json` paths point at the real stage-4 and stage-13 artifacts.

## 🧪 Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` -> pass
- `cargo test -p z00z_simulator --release --test test_scenario1_stage_surface -- --nocapture` -> pass
- `cargo test --release` -> pass

## ⚠️ Residual Risk

- Trace packs generated before this remediation may still contain undetected body-only tampering from the old validator path.
- The live code path is closed, but historical evidence should be regenerated or revalidated under the hardened contract before being treated as trustworthy.
