---
phase: 052-HJMT-Backend
artifact: tests-tasks
status: implemented
source: 052-TEST-SPEC.md
updated: 2026-05-29
owner: Z00Z Storage
scope: implementation order for Phase 052 test, simulator, and benchmark coverage
---

<!-- markdownlint-disable MD060 -->

# Phase 052 Tests Tasks

## 🎯 Purpose

This document turns `052-TEST-SPEC.md` into one concrete implementation order
for Phase 052 test work.

It stays subordinate to `052-TODO.md`, `052-CONTEXT.md`, and the numbered
`052-01-PLAN.md` through `052-11-PLAN.md` files. The order below is about how
to land coverage without duplicating storage, checkpoint, wallet, validator,
simulator, or future protocol authority.

The waves below are planning artifacts for the real HJMT forest backend phase.
They are designed so another engineer or agent can implement coverage without
guessing which seam must be proven first, which checks are source-shape only,
and which scenarios must run in compatibility, forest, and dual-verify modes.

## 📌 Scope Inputs

- `052-TEST-SPEC.md`
- `052-CONTEXT.md`
- `052-TODO.md`
- `052-01-PLAN.md`
- `052-02-PLAN.md`
- `052-03-PLAN.md`
- `052-04-PLAN.md`
- `052-05-PLAN.md`
- `052-06-PLAN.md`
- `052-07-PLAN.md`
- `052-08-PLAN.md`
- `052-09-PLAN.md`
- `052-10-PLAN.md`
- `052-11-PLAN.md`
- `docs/Z00Z-JMT-Design.md`
- `.planning/phases/000/051-HJMT-Facade/051-TEST-SPEC.md`
- live test anchors and bench harnesses listed in `052-TEST-SPEC.md`
- `052-TEST-SPEC.md` sections `Realistic Examples To Implement` and
  `Coverage Ledger Against The Phase Packet`

## ⚙️ Execution Strategy

- Lock no-duplication and no-parallel-authority guardrails first so test work
  itself cannot drift into a fake forest backend, copied compatibility lane,
  second proof decoder, second checkpoint verifier, or simulator-owned storage
  authority.
- Land backend-mode, bucket-policy, and planner tests before journal or proof
  tests, because recovery and proof correctness both depend on the real forest
  layout and deterministic planner semantics.
- Land recovery and reload tests before equivalence and downstream scenario
  tests, because `scenario_1`, checkpoint, and dual-verify all depend on
  durable forest state and path-index rebuild.
- Land forest proof and absence tests before downstream guardrails and
  simulator validation, because wallet, validator, and simulator consumers
  must read the final proof contract, not an intermediate placeholder.
- Land equivalence and downstream guardrail tests before benchmarks and
  closeout, because Phase 052 cannot make rollout or performance claims until
  the semantic contract is already proven.
- Finish with benchmark evidence, cross-mode `scenario_1`, and closeout
  records only after implementation, focused gates, broad gates, and review
  loops exist.
- After backend closeout, run the green-state audit and preserve adaptive
  buckets, occupancy metadata, generalized root migration, `RightLeaf`, and
  `FeeEnvelope` as first-class future candidates rather than live Phase 052
  exports.
- Every `/GSD-Review-Tasks-Execution` command below means running
  `.github/prompts/gsd-review-tasks-execution.prompt.md` in YOLO mode.
- If execution needs a commit, use `/z00z-git-versioning`; do not use raw git
  commit or push commands.
- File targets named `test_phase052_*` or `forest_*` are proposed homes unless
  already present in the checkout. Extend existing storage, proof, checkpoint,
  wallet, validator, and simulator anchors first, and do not create duplicate
  authority logic or a parallel backend facade.

## 🧪 Task Waves

### Wave T0: Harness Wiring And Authority Guardrails

- priority: first
- why now:
  - Prevents the test harness itself from hiding a fake forest backend,
    duplicate verifier, or unexecuted assets test module behind green output.
- scenario coverage:
  - direct guardrail coverage: `052-SC-03`, `052-SC-14`, `052-SC-20`,
    `052-SC-21`, `052-SC-22`
  - traceability seeding: `052-SC-01` through `052-SC-22`
- files to inspect:
  - `.planning/phases/052-HJMT-Backend/052-TEST-SPEC.md`
  - `.planning/phases/052-HJMT-Backend/052-CONTEXT.md`
  - `.planning/phases/052-HJMT-Backend/052-TODO.md`
  - `crates/z00z_storage/tests/assets/test_assets.rs`
  - `crates/z00z_storage/tests/test_assets_suite.rs`
  - `crates/z00z_storage/tests/test_phase051_guardrails.rs`
  - `crates/z00z_runtime/validators/src/*.rs`
  - `crates/z00z_wallets/src/tx/**/*.rs`
  - `crates/z00z_simulator/src/scenario_1/**/*.rs`
- files to create or extend:
  - `crates/z00z_storage/tests/test_phase052_guardrails.rs`
  - `crates/z00z_storage/tests/assets/test_assets.rs`
  - `crates/z00z_storage/tests/test_assets_suite.rs`
- implementation tasks:
  - Add source-shape guards for no public bucket id authority, no public
    `TreeId`, no namespace helper imports outside storage internals, no raw
    backend-root checkpoint authority, no second proof decoder, no second
    checkpoint verifier, and no fake forest backend lane.
  - Add guardrails for Phase 052 future-only nouns so tests fail if
    `RightLeaf`, generalized-rights exports, or public backend-root authority
    leak into live storage surfaces.
  - Add module wiring checks so new `tests/assets/*.rs` files are routed
    through `test_assets.rs` and `test_assets_suite.rs`.
  - Add a scenario-to-anchor ledger in test names or comments for live
    scenarios `052-SC-01` through `052-SC-17` and planning scenarios
    `052-SC-18` through `052-SC-22`.
- success conditions:
  - Forbidden source shapes are asserted against live source files.
  - New asset-facing tests are guaranteed to run once created.
  - No production logic is introduced in this wave.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_assets_suite -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase051_guardrails -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_guardrails -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T0: Harness Wiring And Authority Guardrails"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T1: Backend Mode, Bucket Policy, And Layout Contract

- priority: after T0 and alongside `052-01-PLAN.md`
- why now:
  - The rest of Phase 052 coverage needs stable backend-mode routing and a
    frozen bucket contract before physical mutation, recovery, or proof logic
    can be tested meaningfully.
- scenario coverage:
  - `052-SC-01`, `052-SC-02`, `052-SC-03`
- files to extend or create:
  - `crates/z00z_storage/tests/assets/test_backend_facade_contract.rs`
  - `crates/z00z_storage/tests/assets/test_store_api.rs`
  - `crates/z00z_storage/tests/test_phase052_forest_backend.rs`
- implementation tasks:
  - Add backend-mode tests for compatibility default, explicit forest,
    explicit dual-verify, and unknown-mode rejection.
  - Add bucket-policy tests for deterministic derivation, invalid bounds,
    single-bucket rejection when forbidden, stable `BucketRootLeaf` encoding,
    and deny-unknown-fields behavior where supported by the chosen codec.
  - Add source-shape tests proving layout identities and bucket ids remain
    storage-private even after the forest layout lands.
- success conditions:
  - Compatibility remains default for existing callers.
  - Bucket derivation and encoding are stable and versioned.
  - No public storage API accepts bucket ids or physical-layout authority.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_assets_suite -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_forest_backend -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T1: Backend Mode, Bucket Policy, And Layout Contract"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T2: Planner Semantics And Reject-Without-Mutation

- priority: after T1 and alongside `052-02-PLAN.md`
- why now:
  - The first meaningful forest correctness proof is that planner workloads are
    real physical forest work and still semantically equivalent to the
    compatibility corpus.
- scenario coverage:
  - `052-SC-04`, `052-SC-05`, `052-SC-12`
- files to extend or create:
  - `crates/z00z_storage/tests/test_phase052_forest_backend.rs`
  - `crates/z00z_storage/tests/assets/test_store_api.rs`
  - `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`
  - `crates/z00z_storage/src/assets/store_internal/test_whitebox_state.rs`
- implementation tasks:
  - Add positive planner workloads for insert-many, delete-many, hot-serial,
    cross-definition, reorder-stable, and no-op flows in compatibility,
    forest, and dual-verify modes.
  - Add negative planner workloads for duplicate path and delete-missing and
    prove that root, version history, path-index state, and rows are unchanged
    after each reject.
  - Add explicit checks that the planner groups by definition, serial, and
    derived bucket instead of copying compatibility logic into a shadow path.
- success conditions:
  - Positive workloads stay semantically equal to compatibility.
  - Rejecting workloads preserve state exactly.
  - Tests would fail if forest mode quietly routed back into compatibility.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_forest_backend -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase051_golden_corpus -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T2: Planner Semantics And Reject-Without-Mutation"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T3: Journal Recovery, Reload, And Path-Index Rebuild

- priority: after T2 and alongside `052-03-PLAN.md`
- why now:
  - Downstream proofs, checkpoint reload, and scenario validation are only
    meaningful once durable forest publication and reload safety exist.
- scenario coverage:
  - `052-SC-06`, `052-SC-07`
- files to extend or create:
  - `crates/z00z_storage/tests/test_phase052_recovery.rs`
  - `crates/z00z_storage/tests/test_redb_rehydrate.rs`
  - `crates/z00z_storage/tests/test_search_api.rs`
  - `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
- implementation tasks:
  - Add a journal interruption matrix for `Prepared`, partial children,
    `ChildrenCommitted`, `ParentsCommitted`, and `RootPublished`.
  - Add digest-mismatch, status-regression, and replay-drift negative tests.
  - Add reload tests for forest root rebuild, path-index rebuild, checkpoint
    seal or reload, and claim replay rows.
  - Add assertions that repeated recovery after `RootPublished` does not
    republish a divergent root.
- success conditions:
  - Recovery either completes to the correct root or rejects fail-closed.
  - Reload reconstructs the committed semantic root and internal lookup state.
  - Path index remains private and rebuildable only from committed leaves.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_recovery -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_redb_rehydrate -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_search_api -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_checkpoint_root_binding -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T3: Journal Recovery, Reload, And Path-Index Rebuild"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T4: Forest Proof Envelope And Absence Matrix

- priority: after T3 and alongside `052-04-PLAN.md`
- why now:
  - Wallet, validator, simulator, and equivalence coverage must consume the
    final proof contract, not a temporary placeholder.
- scenario coverage:
  - `052-SC-08`, `052-SC-09`, `052-SC-10`, `052-SC-11`
- files to extend or create:
  - `crates/z00z_storage/tests/test_phase052_forest_proofs.rs`
  - `crates/z00z_storage/tests/assets/test_store_api.rs`
  - `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs`
  - `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
- implementation tasks:
  - Add valid forest inclusion proof roundtrip coverage for single-leaf and
    shared-parent cases.
  - Add the full proof reject matrix: malformed bytes, wrong path, wrong
    policy, wrong bucket id, wrong bucket root, wrong definition branch, wrong
    serial branch, wrong terminal branch, wrong terminal leaf, wrong terminal
    leaf hash, wrong semantic root, wrong root bind, wrong backend-root
    diagnostic bind, wrong checkpoint context, wrong deletion proof, wrong
    non-existence proof, and unsupported versions.
  - Add deletion proof coverage and absence proof coverage for empty-tree
    absence, absent key after inserts, replay consistency at the same root,
    present-key rejection, tampered deletion-family rejection, and tampered
    non-existence-family rejection.
  - Record proof-size evidence for inclusion from real encodings. For absence,
    record real proof-size and throughput evidence only when absent-key proofs
    are live; otherwise record explicit unsupported fail-closed status. Prove
    that rejecting proof workloads leave state unchanged.
- success conditions:
  - One storage-owned forest proof decoder exists conceptually in tests.
  - Deletion and non-existence proof behavior is real and fail-closed or
    explicitly unsupported.
  - Proof-size evidence is measured, recorded, and never treated as protocol
    truth.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_forest_proofs -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_checkpoint_root_binding -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_assets_suite -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T4: Forest Proof Envelope And Absence Matrix"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T5: Equivalence Corpus And Downstream Semantic Guardrails

- priority: after T4 and alongside `052-05-PLAN.md`
- why now:
  - Once forest mutation, recovery, and proofs are real, the next job is to
    prove there is still one semantic authority lane across storage, wallet,
    validator, and simulator.
- scenario coverage:
  - `052-SC-12`, `052-SC-13`, `052-SC-14`, `052-SC-15`
- files to extend or create:
  - `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`
  - `crates/z00z_storage/tests/test_phase052_forest_backend.rs`
  - `crates/z00z_storage/tests/test_phase051_guardrails.rs`
  - `crates/z00z_storage/tests/test_phase052_guardrails.rs`
  - `crates/z00z_wallets/tests/test_tx_tamper.rs`
  - `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
  - `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`
  - `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`
  - `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`
  - `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
  - `crates/z00z_simulator/tests/test_stage4_tamper.rs`
  - `crates/z00z_simulator/tests/test_scenario1_tx_proof_roundtrip.rs`
  - `crates/z00z_simulator/tests/test_stage4_source_shape.rs`
  - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- implementation tasks:
  - Replace the future-forest placeholder in the golden harness with the real
    forest case and add dual-verify mismatch hard-fail coverage.
  - Compare compatibility and forest for root, check-root, get, lookup, list,
    proof result class, reload behavior, checkpoint behavior, wrong checkpoint
    context rejection, and state preservation after rejects.
  - Extend wallet tests so forest proof bytes, root-bind drift, and audit
    drift reject exactly.
  - Extend simulator and source-shape tests so Stage 4, Stage 6, Stage 7,
    Stage 11, Stage 12, Stage 13, and `runner_verify` remain proof-first and
    semantic-root bound.
  - Extend wallet or simulator tamper tests so signature-bearing and
    digest-bound packages reject before any forest mutation in compatibility,
    forest, and dual-verify modes.
- success conditions:
  - Dual-verify drift is fatal.
  - Downstream code does not learn physical layout authority.
  - `scenario_1` remains a storage-semantics consumer.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase051_golden_corpus -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase051_guardrails -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_forest_backend -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_guardrails -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_tx_tamper -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_spend_proof_backend -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage7_jmt_wallet_scan -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_tamper -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_source_shape -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_tx_proof_roundtrip -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T5: Equivalence Corpus And Downstream Semantic Guardrails"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T6: Benchmarks, Cross-Mode Scenario 1, And Closeout Evidence

- priority: last and alongside `052-06-PLAN.md`
- why now:
  - Benchmarks and final scenario evidence should describe the landed system,
    not a moving intermediate target.
- scenario coverage:
  - `052-SC-15`, `052-SC-16`, `052-SC-17`
- files to extend or create only when absent:
  - `crates/z00z_storage/benches/assets/shard.rs`
  - `crates/z00z_storage/benches/assets/nested.rs`
  - `crates/z00z_storage/benches/assets/assets_benches.md` as the preferred
    benchmark-evidence file, or an equivalent evidence home if absent
  - `.planning/phases/052-HJMT-Backend/052-06-SUMMARY.md`
  - `.planning/phases/052-HJMT-Backend/052-SUMMARY.md`
  - `.planning/STATE.md`
- implementation tasks:
  - Extend the landed harness for random, broad, hot-definition, hot-serial,
    delete-heavy, proof-heavy, recovery, async multi-insert, async
    multi-delete, inclusion proof-size, and non-existence proof-size or
    unsupported-status lanes.
  - Compare compatibility baseline, bucket-width variants, and forest mode;
    keep any definition-sharded-only lane benchmark-only.
  - Run `scenario_1` in compatibility, forest, and dual-verify modes once
    those modes exist and record the exact mode-selection inputs used.
  - Record exact benchmark commands, focused test commands, broad cargo
    command, and repeated review-loop outcomes in the phase summaries.
- success conditions:
  - Benchmark evidence exists for the design's async workload and proof-size
    claims.
  - Compatibility remains default until the full evidence set is green.
  - Closeout truthfully records commands, modes, outputs, and deferred items.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo bench -p z00z_storage --bench assets_shard`
  - `cargo bench -p z00z_storage --bench assets_nested`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump scenario_1`
  - `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
  - `cargo test --release --features test-fast --features wallet_debug_dump`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T6: Benchmarks, Cross-Mode Scenario 1, And Closeout Evidence"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T7: Green-State Audit And Deferred Ledger

- priority: after T6 and alongside `052-07-PLAN.md`
- why now:
  - Future protocol candidates should not start until the fixed-bucket forest
    backend has executed evidence and no placeholder behavior remains.
- scenario coverage:
  - `052-SC-18`
- files to extend:
  - `.planning/phases/052-HJMT-Backend/052-06-SUMMARY.md`
  - `.planning/phases/052-HJMT-Backend/052-SUMMARY.md`
  - `.planning/phases/052-HJMT-Backend/052-TODO.md`
  - `.planning/phases/052-HJMT-Backend/052-CONTEXT.md`
  - `.planning/STATE.md`
- implementation tasks:
  - Audit every `052-01` through `052-06` implementation gate for executed
    evidence.
  - Fail the audit if selected forest operations still return skeleton,
    placeholder, copied compatibility, or fake-success behavior.
  - Record deferred candidates as future work with entry conditions and
    blocked live exports.
- success conditions:
  - Backend work is green by evidence.
  - Deferred future work is visible but not implied shipped.
- command gate:
  - `git diff --check -- .planning/phases/052-HJMT-Backend .planning/STATE.md`
  - rerun `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
    and focused/broad cargo commands if audit evidence is missing
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T7: Green-State Audit And Deferred Ledger"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T8: Adaptive Buckets And Migration Proof Candidate Tests

- priority: after T7 and alongside `052-08-PLAN.md`
- why now:
  - Adaptive buckets depend on fixed-bucket performance, proof-size, and
    recovery evidence.
- scenario coverage:
  - `052-SC-19`
- files to extend:
  - `.planning/phases/052-HJMT-Backend/052-08-PLAN.md`
  - `.planning/phases/052-HJMT-Backend/052-TEST-SPEC.md`
  - `.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md`
- implementation tasks:
  - Define future split proof, merge proof, migration proof, bucket epoch,
    old-policy, new-policy, historical proof, replay, and recovery tests.
  - Define benchmark comparison against fixed buckets and simulator
    `scenario_1` continuity through storage-owned APIs.
  - Keep adaptive runtime code out of Phase 052.
- success conditions:
  - Adaptive migration has complete future test duties and hard entry
    conditions.
  - No adaptive placeholder is treated as shipped Phase 052 behavior.
- command gate:
  - `git diff --check -- .planning/phases/052-HJMT-Backend`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T8: Adaptive Buckets And Migration Proof Candidate Tests"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T9: Bucket Occupancy Metadata Privacy Tests

- priority: after T7 and alongside `052-09-PLAN.md`
- why now:
  - Proof-visible occupancy counters are privacy-sensitive and must not appear
    by accident in the forest proof contract.
- scenario coverage:
  - `052-SC-20`
- files to extend or create only if runtime guardrail needed:
  - `.planning/phases/052-HJMT-Backend/052-09-PLAN.md`
  - `.planning/phases/052-HJMT-Backend/052-TEST-SPEC.md`
  - `.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md`
  - `crates/z00z_storage/tests/test_phase052_followup_guardrails.rs`
- implementation tasks:
  - Define guardrails for no proof-visible `leaf_count`,
    `bucket_occupancy`, or equivalent counter without design update.
  - Define future tests for counter tamper, policy-generation binding,
    root-binding, reload drift, and downstream non-authority.
  - Keep local metrics diagnostic and non-authoritative.
- success conditions:
  - Occupancy metadata cannot become proof-visible silently.
  - Future counter promotion needs design update, privacy review, and tests.
- command gate:
  - `git diff --check -- .planning/phases/052-HJMT-Backend`
  - if runtime guardrail code is added, run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T9: Bucket Occupancy Metadata Privacy Tests"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T10: Generalized Settlement Root Migration Tests

- priority: after T7 and alongside `052-10-PLAN.md`
- why now:
  - Root-vocabulary migration must not be coupled to the backend swap because
    that would destroy the Phase 052 oracle.
- scenario coverage:
  - `052-SC-21`
- files to extend:
  - `.planning/phases/052-HJMT-Backend/052-10-PLAN.md`
  - `.planning/phases/052-HJMT-Backend/052-TEST-SPEC.md`
  - `.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md`
- implementation tasks:
  - Define future tests for old `AssetStateRoot`, new
    `SettlementStateRoot`, mixed generation, downgrade rejection, checkpoint
    migration, proof envelope versioning, wallet or validator migration, and
    simulator `scenario_1` migration mode.
  - Keep `AssetStateRoot` live in Phase 052.
- success conditions:
  - Generalized root migration has a separate oracle and rollback plan.
  - Phase 052 does not export `SettlementStateRoot` as live authority.
- command gate:
  - `git diff --check -- .planning/phases/052-HJMT-Backend`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T10: Generalized Settlement Root Migration Tests"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T11: RightLeaf And FeeEnvelope Protocol Tests

- priority: after T10 and alongside `052-11-PLAN.md`
- why now:
  - Terminal right semantics and fee support depend on generalized root
    migration rules and must remain separate contract families.
- scenario coverage:
  - `052-SC-22`
- files to extend:
  - `.planning/phases/052-HJMT-Backend/052-11-PLAN.md`
  - `.planning/phases/052-HJMT-Backend/052-TEST-SPEC.md`
  - `.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md`
- implementation tasks:
  - Define future `RightLeaf` schema, family marker, transition, revocation,
    expiry, selective-disclosure, proof, checkpoint, wallet, validator, and
    simulator tests.
  - Define future `FeeEnvelope` payer, sponsor, processing guarantee, budget,
    expiry, replay, tamper, wrong binding, and state-preservation tests.
  - Prove fee support does not imply right ownership or mutate terminal right
    semantics.
- success conditions:
  - `RightLeaf` and `FeeEnvelope` are first-class future protocol work.
  - Neither object is live Phase 052 storage export.
- command gate:
  - `git diff --check -- .planning/phases/052-HJMT-Backend`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/052-HJMT-Backend/052-TESTS-TASKS.md current_task="Wave T11: RightLeaf And FeeEnvelope Protocol Tests"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

## ✅ Implementation Evidence

- Wave T0 live guardrails were implemented in
  `crates/z00z_storage/tests/test_phase052_guardrails.rs` for future-only
  exports, no proof-visible occupancy metadata, asset harness wiring, and the
  `052-SC-01` through `052-SC-22` scenario ledger.
- Wave T3 live reload coverage was extended in
  `crates/z00z_storage/tests/test_search_api.rs` with
  `test_forest_search_reload_index`, covering forest-mode durable reload,
  path-index rebuild, root validation, path lookup, asset lookup, and scoped
  pagination.
- Wave T5 live wallet coverage was extended in
  `crates/z00z_wallets/tests/test_spend_proof_backend.rs` with
  `test_forest_audit_scan`, covering valid forest proof-scan audit acceptance
  and root-bind drift rejection.
- Waves T7 through T11 remain future-candidate planning scope, with live
  guardrails proving adaptive buckets, proof-visible occupancy counters,
  `SettlementStateRoot`, `RightLeaf`, and `FeeEnvelope` are not live Phase 052
  exports.
- Validation passed in the required order: bootstrap-first gate, focused
  Phase 052 storage, wallet, and simulator anchors, `scenario_1` test filter,
  `scenario_1` binary, and broad
  `cargo test --release --features test-fast --features wallet_debug_dump`.
- `/GSD-Review-Tasks-Execution` was applied in three YOLO review passes over
  this test-task implementation; the final two passes found no significant
  implementation, boundary, or validation issues.

## ✅ Completion Conditions

- Every scenario `052-SC-01` through `052-SC-17` is implemented in a live
  test or benchmark home from `052-TEST-SPEC.md`.
- Every planning scenario `052-SC-18` through `052-SC-22` is represented in
  follow-up plans, future test duties, and phase closeout notes.
- No phase-owned test creates a fake backend, copied compatibility lane,
  second proof decoder, second checkpoint verifier, or simulator-owned storage
  authority path.
- Focused tests, broad cargo validation, repeated review loops, and cross-mode
  `scenario_1` execution are recorded in phase summaries.
- Compatibility, forest, and dual-verify mode coverage exists where the packet
  requires it.
- Benchmark evidence is attached for async multi-insert, async multi-delete,
  inclusion proof size, and non-existence proof size or throughput duties.
- Adaptive buckets, proof-visible occupancy counters, generalized root
  migration, `RightLeaf`, and `FeeEnvelope` are not live Phase 052 exports and
  have first-class future candidate plans.
