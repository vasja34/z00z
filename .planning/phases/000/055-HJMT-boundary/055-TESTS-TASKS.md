---
phase: 055-HJMT-boundary
artifact: tests-tasks
status: execution-backed
source:
  - 055-TODO.md
  - 055-CONTEXT.md
  - 055-TEST-SPEC.md
  - 055-01-PLAN.md
  - 055-02-PLAN.md
  - 055-03-PLAN.md
  - 055-04-PLAN.md
  - 055-PLAN-REVIEW.md
updated: 2026-06-10
---

<!-- markdownlint-disable MD060 -->

# Phase 055 Test Implementation Tasks

**Phase:** `055-HJMT-boundary`
**Status:** execution-backed planning artifact
**Companion spec:** `055-TEST-SPEC.md`

## Objective

This file translated the Phase 055 packet into an engineer-ready test work
order. Phase 055 is now implemented, so this document remains as the
historical wave order, owner-home map, and verification anchor for future
follow-up work on the same batch-proof surface.

The goal is not to scaffold empty suites. The goal is to add production-grade
proof, fixture, benchmark, and scenario coverage in the same order as the
actual implementation slices.

## Scope Inputs

- `055-TODO.md` is the canonical source for required tests, benchmark names,
  fixture IDs, and mandatory scenario coverage.
- `055-CONTEXT.md` is the canonical source for decisions `D-02` through
  `D-21`, especially additive `ProofBlob` compatibility, no parallel authority
  layers, and mandatory cross-read sections.
- `055-TEST-SPEC.md` freezes the scenario ledger, rejection matrix, Stage 13
  extension contract, and benchmark report contract.
- `055-01-PLAN.md` through `055-04-PLAN.md` freeze the execution order and the
  narrow owner homes for each slice.
- Live anchors already exist in `proof.rs`, `hjmt_proof.rs`,
  `test_hjmt_live_proof_families.rs`, `test_hjmt_proofs.rs`,
  `test_bench_lanes.rs`, `hjmt_examples.rs`, `report.rs`, and
  `runner_verify.rs`.

## Execution Strategy

- This artifact is sequenced to match the implementation order frozen in
  `055-01-PLAN.md` through `055-04-PLAN.md`.
- Wave 1 must land before Wave 2 because the negative verifier and tamper
  corpus need one canonical batch-proof contract to mutate.
- Wave 2 must land before Wave 3 because positive fixtures are not trustworthy
  release evidence until the fail-closed verifier surface is hardened.
- Wave 3 must land before Wave 4 because Stage 13 and bench evidence must point
  at the same live batch-proof builder and baseline comparator.
- The planned batch-proof source files now exist, so this document remains a
  truthful execution-order archive rather than a blocker note.

## Hard Rules

- Reuse existing live owner homes whenever they already own the behavior.
- Do not create a second proof engine, a second Stage 13 lane, a second bench
  runner, or a mirror compatibility layer.
- Do not create inventory-only suites as empty placeholder files.
- Generate positive fixtures from live production code and mutate those bytes
  for negative fixtures whenever possible.
- Keep public technical content in English.
- When a commit is needed, use `/z00z-git-versioning`.

## Verify Block Template

Every Rust or test-affecting change in Phase 055 must verify in this order:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
cargo test --release
```

Then run the wave-specific targeted commands below.

After the targeted commands, run
`/.github/prompts/gsd-review-tasks-execution.prompt.md`
(`/GSD-Review-Tasks-Execution`) in YOLO mode at least three times and continue
until at least two consecutive runs report no significant issues.

## Current Closeout Note

- The live tree now includes the Phase 055 implementation seams named in the
  packet: `proof_batch.rs`, `proof_batch_verify.rs`,
  `hjmt_batch_proof.rs`, `test_hjmt_batch_proof.rs`,
  `test_hjmt_batch_proof_negative.rs`, the batch fixture homes, the canonical
  settlement bench lanes, and the Stage 13 report extensions.
- `055-04-SUMMARY.md` and `055-SUMMARY.md` now record the execution-backed
  benchmark, scenario, runner, and release validation evidence for the phase.
- `/create-tests 055` is no longer blocked by missing live seams; any rerun is
  now purely additive follow-up work, not a prerequisite for Phase 055
  truthfulness.

## Scenario To Plan Crosswalk

| Plan slice | Test scenarios from `055-TEST-SPEC.md` | Main owner homes |
| --- | --- | --- |
| `055-01` | `SC-01`, `SC-02`, `SC-11` | `test_hjmt_batch_proof.rs`, `test_live_guardrails.rs` |
| `055-02` | `SC-03`, `SC-04`, `SC-05` | `test_hjmt_batch_proof_negative.rs`, `batch_proof_v1_negative/` |
| `055-03` | `SC-06`, `SC-07` | `test_hjmt_batch_proof.rs`, `test_hjmt_live_proof_families.rs`, `test_hjmt_proofs.rs`, positive fixture homes |
| `055-04` | `SC-08`, `SC-09`, `SC-10`, `SC-11` | `test_bench_lanes.rs`, `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`, Stage 13 source files |

## Wave 1: `055-01` Contract And Codec Coverage

**Purpose:** freeze the exact batch-proof test surface before builder and bench
work depend on it.

**Files to create or extend**

- `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`

**Implementation tasks**

- Create `test_hjmt_batch_proof.rs` only when the live batch-proof source files
  exist; until then, keep guardrails in existing suites.
- Add deterministic encode, decode, and re-encode tests for `BatchProofBlobV1`
  positive fixtures and inline generated cases.
- Assert exact field-order stability indirectly through canonical bytes and
  fixture round-trip equality.
- Add additive-surface guardrails proving `ProofBlob` is unchanged and
  `BatchProofBlobV1` appears as a separate export, not a replacement.
- Add live-generation-only rejects for unsupported generation tags and any
  shard context under the current generation contract.
- Reuse `settlement_corpus` and existing HJMT fixture helpers instead of
  inventing a new proof-test harness.

**Success conditions**

- `SC-01` and `SC-02` pass with deterministic bytes and accepted roots.
- `SC-11` blocks source-shape drift such as rename waves or standalone Phase
  055 wrapper layers.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture
```

## Wave 2: `055-02` Negative Verifier And Tamper Corpus

**Purpose:** harden the verifier and negative evidence surface before the
builder is trusted as release evidence.

**Files to create or extend**

- `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/`
- `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`

**Implementation tasks**

- Add parser-limit and bounds-check tests for oversize counts, oversize total
  bytes, and out-of-range opening or witness references.
- Cover canonical ordering reject, duplicate-path reject, mixed proof-family
  reject, opening-kind mismatch reject, leaf-family mismatch reject, and exact
  witness-domain or hash-material rejects.
- Generate `BPB-T-001` through `BPB-T-008` from canonical positive bytes, not
  from ad hoc synthetic blobs.
- Record one exact mutation point, one exact reject stage, one exact verdict,
  and one regeneration command in each negative fixture manifest.
- Assert the verifier never returns per-path success on a failed batch and does
  not expose a partial-acceptance API.

**Success conditions**

- `SC-03`, `SC-04`, and `SC-05` pass.
- Every `BPB-T-*` case has live fixture evidence plus a typed rejection path.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture
```

## Wave 3: `055-03` Builder, Positive Fixtures, And Compatibility

**Purpose:** prove the new batch surface is derived from current live proof
truth rather than from a duplicate engine.

**Files to create or extend**

- `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`
- `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs`
- `crates/z00z_storage/tests/test_hjmt_proofs.rs`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/`

**Implementation tasks**

- Add builder-driven positive vectors for `BPB-G-001` through `BPB-G-005`.
- Add direct baseline comparisons for the same logical path set across one
  `ProofBlob`, current `Vec<ProofBlob>`, and `BatchProofBlobV1`.
- Add clustered and scattered witness-reuse examples using live store-derived
  paths and HJMT corpus helpers.
- Add deletion vectors with prior context and non-existence vectors with live
  default commitments.
- Add current-generation accept plus future-generation reject vectors under
  `root_generation_migration/`.
- Re-encode every positive fixture and require byte-stable output.

**Success conditions**

- `SC-06` and `SC-07` pass.
- Positive fixtures become durable release evidence, not just local examples.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_live_proof_families -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_proofs -- --nocapture
```

## Wave 4: `055-04` Bench Lanes, Stage 13 Evidence, And Closeout Guards

**Purpose:** close the phase with durable repository evidence instead of unit
tests alone.

**Files to create or extend**

- `crates/z00z_storage/tests/test_bench_lanes.rs`
- `crates/z00z_storage/benches/settlement_proofs.rs`
- `crates/z00z_storage/benches/settlement_hjmt.rs`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_storage/scripts/run_storage_settlement_bench.py`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/report.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/hjmt_examples.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`

**Implementation tasks**

- Wire the logical lanes `hjmt_batch_proof_bytes` and `hjmt_batch_verify` into
  the existing `settlement_proofs.rs` and `settlement_hjmt.rs` homes.
- Extend bench docs and the runner script so commands, output names, and
  logical lanes stay synchronized.
- Freeze the exact Stage 13 batch `proof_surface` values
  `proof_blob_single`, `proof_blob_vec`, and `batch_proof_v1`.
- Extend Stage 13 examples, proof-size rows, and tamper rows with the exact
  fields and case IDs from `055-TEST-SPEC.md`.
- Add source-shape and JSON-schema assertions that fail on a second Stage 13
  lane, missing representative clustered or scattered rows, missing `2`, `8`,
  or `32` path counts, or missing batch tamper IDs.
- Add runner-verification tests that deliberately remove one required field or
  case and assert contract rejection.

**Success conditions**

- `SC-08`, `SC-09`, `SC-10`, and `SC-11` pass.
- The repository can prove batch-proof behavior through the live bench and
  simulator surfaces without duplicating either of them.

**Targeted commands**

```bash
cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture
cargo bench -p z00z_storage --bench settlement_proofs --no-run
cargo bench -p z00z_storage --bench settlement_hjmt --no-run
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario_settlement -- --nocapture
cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_scenario1_stage_surface -- --nocapture
cargo run --release -p z00z_simulator --bin scenario_1 --features test-params-fast --features wallet_debug_tools
./crates/z00z_storage/scripts/run_storage_settlement_bench.py --bench settlement_proofs -- --sample-size 10 --warm-up-time 0.01 --measurement-time 0.02
./crates/z00z_storage/scripts/run_storage_settlement_bench.py --bench scenario_1
```

## Do Not Materialize As Placeholder Or Parallel Surfaces

The following names remain live Phase 055 authority anchors, but they must not
be added as empty suites, standalone duplicate benches, or second harnesses:

- `test_hjmt_batch_commit.rs`
- `test_hjmt_batch_recovery.rs`
- `test_hjmt_storage_boundary.rs`
- `test_hjmt_backend_conformance.rs`
- `test_hjmt_shard_routing.rs`
- `test_hjmt_failover_same_lineage.rs`
- `test_hjmt_split_brain_fencing.rs`
- `test_hjmt_multi_aggregator_sim.rs`
- `test_hjmt_root_generation.rs`
- `test_hjmt_historical_proofs.rs`
- `test_hjmt_transition_proofs.rs`
- `test_hjmt_privacy_regression.rs`
- `hjmt_bucket_delta_commit.rs`
- `hjmt_backend_boundary.rs`
- `hjmt_shard_parallel_commit.rs`
- `hjmt_root_of_roots_publish.rs`
- `hjmt_transition_locality.rs`
- any standalone `hjmt_batch_proof_bytes.rs` or `hjmt_batch_verify.rs` bench
  file instead of a logical lane inside the live consolidated bench homes

## Closeout Sync Checklist

- If a planned owner home changes, update `055-TEST-SPEC.md`,
  `055-TESTS-TASKS.md`, and the relevant numbered plan in the same change set.
- If a new reject case or fixture ID is added, update the rejection matrix and
  Stage 13 case-ID contract together.
- If Stage 13 schema fields change, update `report.rs`, `runner_verify.rs`,
  `test_scenario_settlement.rs`, and `test_scenario1_stage_surface.rs`
  together.
- If the repo ever adds a standalone Phase 055 bench crate or a second Stage 13
  lane, treat it as a regression against `D-21` and fix the architecture
  instead of adjusting the packet to match the drift.
- If a canonical benchmark home is remapped, update `055-CONTEXT.md`,
  `055-TEST-SPEC.md`, and `055-SOURCE-AUDIT.md` together so the Phase 1
  inventory contract stays exact.
