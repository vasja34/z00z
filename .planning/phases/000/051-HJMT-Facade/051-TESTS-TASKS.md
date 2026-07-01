---
phase: 051-HJMT-Facade
artifact: tests-tasks
status: implemented-and-validated
source: 051-TEST-SPEC.md
updated: 2026-05-28
owner: Z00Z Storage
scope: implementation order for Phase 051 test coverage
---

# Phase 051 Tests Tasks

## 🎯 Purpose

This document turns `051-TEST-SPEC.md` into one concrete implementation order
for Phase 051 test work.

It stays subordinate to `051-TODO.md`, `051-CONTEXT.md`, and the numbered
`051-01-PLAN.md` through `051-05-PLAN.md` files. The order below is about how
to land coverage without duplicating storage, checkpoint, wallet, validator, or
simulator authority.

The waves below are implemented for the Phase 051 compatibility backend path.
All Rust validation evidence recorded for this artifact used release-mode cargo
commands.

## 📌 Scope Inputs

- `051-TEST-SPEC.md`
- `051-CONTEXT.md`
- `051-TODO.md`
- `051-01-PLAN.md`
- `051-02-PLAN.md`
- `051-03-PLAN.md`
- `051-04-PLAN.md`
- `051-05-PLAN.md`
- live test anchors listed in `051-TEST-SPEC.md`
- `051-TEST-SPEC.md` section `Realistic Examples To Implement`, especially
  examples `051-EX-01` through `051-EX-06`

## ⚙️ Execution Strategy

- Lock the harness and source-shape guardrails first so implementation cannot
  drift into a parallel backend, verifier, checkpoint schema, wallet scan
  cursor, replay registry, or simulator authority lane.
- Land facade and compatibility backend contract tests before root/proof
  hardening, because the proof and golden corpus must run through the stable
  storage boundary.
- Land root taxonomy and proof-envelope tests before downstream guardrail
  tests, because validator, wallet, and simulator consumers need the storage
  authority contract to exist first.
- Land golden compatibility and reload/checkpoint corpus after the public and
  downstream guardrails, because the corpus must prove end-to-end behavior over
  the final facade shape.
- Finish with documentation and closeout evidence only after implementation,
  focused gates, broad gates, and review loops exist.
- Every `/GSD-Review-Tasks-Execution` command below means running
  `.github/prompts/gsd-review-tasks-execution.prompt.md` in YOLO mode.

## 🧪 Task Waves

### Wave T0: Harness And Guardrail Lock-In

- priority: first
- why now:
  - Prevents test implementation from creating a fake forest backend, copied
    compatibility backend, duplicate checkpoint verifier, or downstream proof
    authority before the real facade is in place.
- files to inspect:
  - `.planning/phases/051-HJMT-Facade/051-TEST-SPEC.md`
  - `.planning/phases/051-HJMT-Facade/051-CONTEXT.md`
  - `.planning/phases/051-HJMT-Facade/051-TODO.md`
  - `crates/z00z_storage/tests/test_assets_suite.rs`
  - `crates/z00z_storage/tests/assets/test_assets.rs`
  - `crates/z00z_storage/tests/assets/test_store_api.rs`
  - `crates/z00z_storage/src/assets/mod.rs`
  - `crates/z00z_storage/src/assets/store.rs`
  - `crates/z00z_runtime/validators/src/*.rs`
  - `crates/z00z_wallets/src/tx/**/*.rs`
  - `crates/z00z_simulator/src/scenario_1/**/*.rs`
- files to create:
  - `crates/z00z_storage/tests/test_phase051_guardrails.rs`
- implementation tasks:
  - Add source-shape guards for no public `TreeId`, no public `ns_key`, no
    downstream namespace-prefix reconstruction, no raw backend-root authority
    method, no live `SettlementStateRoot`, no live `RightLeaf`, no live
    `FeeEnvelope`, no dummy forest backend, and no second checkpoint verifier.
  - Add a scenario-to-anchor ledger inside the new test file comments or test
    names for `051-SC-01` through `051-SC-15`.
  - Keep source-shape tests narrow and file-backed; do not parse planning docs
    as proof of code behavior.
- success conditions:
  - Forbidden source shapes are asserted against live source files.
  - Tests fail if a downstream crate imports storage physical-layout internals
    or creates a second checkpoint/proof authority.
  - No production code is introduced in this wave.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase051_guardrails -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/051-HJMT-Facade/051-TEST-SPEC.md current_task="Wave T0: Harness And Guardrail Lock-In"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T1: Facade And Compatibility Backend Contract

- priority: after T0 and alongside `051-01-PLAN.md`
- why now:
  - The rest of Phase 051 coverage must run through one semantic storage
    boundary rather than the old direct implementation shape.
- files to extend:
  - `crates/z00z_storage/tests/assets/test_store_api.rs`
  - `crates/z00z_storage/tests/assets/test_assets.rs`
  - `crates/z00z_storage/tests/test_assets_suite.rs`
  - `crates/z00z_storage/src/assets/store_internal/test_whitebox_state.rs`
- files to create when useful:
  - `crates/z00z_storage/tests/assets/test_backend_facade_contract.rs`
- implementation tasks:
  - Add tests for the facade methods required by `051-01-PLAN.md`: root,
    check-root, lookup, list, batch put/delete, proof item/blob/scan,
    claim-source proof, checkpoint-facing reads, and reload validation hooks.
  - Add compatibility backend identity tests that prove the current shared
    namespaced JMT is explicitly the compatibility backend and reference
    oracle.
  - Assert public `AssetStore` behavior remains source-compatible for
    `new`, `load`, `put_item`, `del_item`, `apply_ops`, `proof_item`,
    `proof_blob`, `proof_scan`, `root`, `check_root`, and
    `claim_source_contract_for_item`.
  - Assert rollback on failed commits is equivalent to current snapshot/restore
    behavior.
- success conditions:
  - All existing public storage tests still pass.
  - The compatibility backend is named and delegated through one path.
  - There is no old direct authority path plus copied new authority path.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_assets_suite -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/051-HJMT-Facade/051-TEST-SPEC.md current_task="Wave T1: Facade And Compatibility Backend Contract"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T2: Root Taxonomy And Proof Envelope

- priority: after T1 and alongside `051-02-PLAN.md`
- why now:
  - Root taxonomy and proof-envelope failure classes must be stable before
    downstream consumers are cut over.
- files to extend:
  - `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
  - `crates/z00z_storage/tests/assets/test_store_api.rs`
  - `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs`
  - `crates/z00z_storage/tests/test_claim_source_proof.rs`
- files to create when useful:
  - `crates/z00z_storage/tests/assets/test_proof_envelope_contract.rs`
- implementation tasks:
  - Add root taxonomy assertions for `AssetStateRoot`, `CheckRoot`, and
    `TxDigest::to_check()` rejection.
  - Add source-shape assertions that Phase 051 did not export live
    `SettlementStateRoot`, `RightLeaf`, or `FeeEnvelope`.
  - Add a valid proof-envelope roundtrip that asserts semantic root, exact
    path, definition-root leaf, serial-root leaf, terminal leaf, leaf hash,
    backend root, root-bind version, root-bind bytes, and all branch proof
    segments.
  - Add typed rejects for unsupported version, malformed bytes, wrong semantic
    root, wrong path, wrong parent leaves, wrong terminal leaf, wrong terminal
    leaf hash, wrong backend-root binding, wrong branch proofs, wrong
    checkpoint context, unexpected bucket metadata, detached payloads, and
    unsupported deletion/non-existence families.
- success conditions:
  - Proof mismatch tests cover every reject class from `051-TODO.md` and
    `051-CONTEXT.md`.
  - Unsupported proof families cannot pass through placeholder proof bytes.
  - Root taxonomy tests prevent backend-root or future-root substitution.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_checkpoint_root_binding -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_assets_suite -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_claim_source_proof -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/051-HJMT-Facade/051-TEST-SPEC.md current_task="Wave T2: Root Taxonomy And Proof Envelope"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T3: Downstream Authority Consumers

- priority: after T2 and alongside `051-03-PLAN.md`
- why now:
  - Validator, wallet, and simulator consumers must be guarded after storage
    root/proof semantics exist.
- files to extend:
  - `crates/z00z_storage/tests/test_phase051_guardrails.rs`
  - `crates/z00z_wallets/tests/test_tx_tamper.rs`
  - `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
  - `crates/z00z_storage/tests/test_claim_source_proof.rs`
  - `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`
  - `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`
- implementation tasks:
  - Add guardrails proving validators consume storage-owned checkpoint
    contracts and existing `RejectClass` values.
  - Extend wallet witness tests so `MemberWit`, claim-source verification, and
    asset-class audit diagnostics reject tampered `ProofBlob`, root-bind, or
    synthetic source-root evidence.
  - Extend simulator Stage 7 JMT scan tests so empty or tampered proof bytes
    reject before ownership detection and artifact rows remain proof-first.
  - Add source-shape checks that downstream crates do not import `TreeId`,
    `ns_key`, namespace tags, physical branch layout helpers, or new copied
    branch-proof verifiers.
- success conditions:
  - Backend-root/root-bind use in wallet or simulator code is diagnostic and
    paired with semantic root verification.
  - Validator code does not define a second checkpoint proof formula, artifact
    schema, or verdict vocabulary.
  - Simulator remains a proof consumer, not a storage proof authority.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase051_guardrails -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_tx_tamper -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_spend_proof_backend -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage7_jmt_wallet_scan -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/051-HJMT-Facade/051-TEST-SPEC.md current_task="Wave T3: Downstream Authority Consumers"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T4: Golden Compatibility Corpus

- priority: after T3 and alongside `051-04-PLAN.md`
- why now:
  - The compatibility backend must become the semantic reference for later
    forest work only after the facade and downstream guardrails are stable.
- files to extend:
  - `crates/z00z_storage/tests/assets/test_store_api.rs`
  - `crates/z00z_storage/tests/test_search_api.rs`
  - `crates/z00z_storage/src/assets/store_internal/test_whitebox_state.rs`
- files to create:
  - `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`
- implementation tasks:
  - Build a backend-case harness with one executable `compatibility` case.
  - Add a documented future real-forest backend slot that is pending or
    explicitly unsupported, not a fake backend.
  - Cover insert-many, delete-many, hot-serial, cross-definition,
    duplicate path, delete-missing, reorder-stable roots, no-op roots, asset
    lookup, path lookup, list pagination, replay behavior, and proof
    verification outcomes.
  - Assert every rejection workload preserves root, version history, item set,
    path index, and persisted state when applicable.
- success conditions:
  - Compatibility backend is a green semantic reference.
  - No test compares compatibility against copied compatibility logic.
  - Future forest backend can join the same matrix without changing expected
    semantic outcomes.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase051_golden_corpus -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_search_api -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/051-HJMT-Facade/051-TEST-SPEC.md current_task="Wave T4: Golden Compatibility Corpus"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T5: Reload Checkpoint And Path-Index Corpus

- priority: after T4 and alongside the second task in `051-04-PLAN.md`
- why now:
  - Durable reload, checkpoint seal/reload, and path-index rebuild prove the
    facade survives process and artifact boundaries.
- files to extend:
  - `crates/z00z_storage/tests/test_redb_rehydrate.rs`
  - `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
  - `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
  - `crates/z00z_storage/tests/test_serialization_restore.rs`
  - `crates/z00z_storage/tests/test_search_api.rs`
- files to create only if needed:
  - `crates/z00z_storage/tests/test_phase051_recovery_contract.rs`
- implementation tasks:
  - Add reload-after-crash coverage through the facade for committed roots,
    items, `find_asset`, list pagination, proof verification, and checkpoint
    metadata.
  - Add path-index rebuild coverage proving `AssetId` lookup reconstructs from
    committed semantic rows and is not public root truth.
  - Add checkpoint seal/reload coverage for `CheckRoot` agreement with
    `AssetStateRoot`, statement binding, proof bytes, checkpoint ids, exec ids,
    and missing/mixed-era row rejection.
  - Reuse the proof reject corpus from T2 for persisted wrong proof and wrong
    checkpoint binding cases.
- success conditions:
  - `AssetStore::load` rejects root drift, flat-root drift, missing snapshot,
    missing exec, proof-byte drift, draft drift, checkpoint-id drift, and
    statement mismatch.
  - Reloaded path lookup and pagination match pre-reload semantic rows.
  - Checkpoint validation stays storage-owned.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_redb_rehydrate -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_checkpoint_finalization -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_serialization_restore -- --nocapture`
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_search_api -- --nocapture`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/051-HJMT-Facade/051-TEST-SPEC.md current_task="Wave T5: Reload Checkpoint And Path-Index Corpus"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

### Wave T6: Broad Gate And Closeout Evidence

- priority: final, after T0 through T5 are green
- why now:
  - Closeout docs are truthful only after focused and broad validation evidence
    exists.
- files to extend:
  - `.planning/phases/051-HJMT-Facade/051-SUMMARY.md`
  - `.planning/ROADMAP.md`
  - `.planning/STATE.md`
  - `crates/z00z_storage/src/assets/README.MD`
  - `crates/z00z_storage/src/assets/root-types.md`
  - `docs/Z00Z-JMT-Design.md`
- implementation tasks:
  - Run the broad Phase 051 gate after focused storage, wallet, simulator, and
    guardrail commands are green.
  - Record changed files, focused test commands, broad command, review-loop
    evidence, and known deferred forest work in `051-SUMMARY.md`.
  - Update docs to state that the compatibility backend is the reference
    backend, not the target performance architecture.
  - Update roadmap/state only after validation evidence exists.
  - Preserve future forest handoff items: fixed bucket policy,
    verifier-visible bucket metadata, physical forest backend,
    child-before-parent publication, forest commit journal, crash-safe recovery,
    dual-backend equivalence, configuration switch, deletion proofs, and
    non-existence proofs.
- success conditions:
  - Closeout does not claim production forest backend, fixed buckets, adaptive
    buckets, commit journal, crash recovery, deletion/non-existence proof
    families, `RightLeaf`, `FeeEnvelope`, or generalized settlement root
    shipped without implementation evidence.
  - Future forest work is explicitly constrained to join through the facade and
    compatibility corpus.
  - `ROADMAP.md` and `STATE.md` do not mark Phase 051 complete before summaries
    and validation evidence exist.
- command gate:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test --release --features test-fast --features wallet_debug_dump`
  - `git diff --check`
  - `/GSD-Review-Tasks-Execution current_spec=.planning/phases/051-HJMT-Facade/051-TEST-SPEC.md current_task="Wave T6: Broad Gate And Closeout Evidence"` at least 3 times in YOLO mode, stopping only after 2 consecutive clean significant-issue passes

## 🔍 Review Checklist Per Wave

- [x] Does the wave extend an existing anchor before creating a new test file?
- [x] Does the test run through the storage facade instead of old physical
      layout internals?
- [x] Does any backend-root assertion also bind `AssetStateRoot` and root-bind
      evidence?
- [x] Does the test reject unsupported proof families instead of accepting
      placeholder bytes?
- [x] Does the test avoid a dummy forest backend or copied compatibility
      implementation?
- [x] Does downstream coverage consume storage-owned proof or checkpoint
      contracts instead of reconstructing them?
- [x] Does the wave implement the relevant `051-EX-*` example with measurable
      pass and failure conditions, not only the abstract scenario row?
- [x] Does the command gate run bootstrap before focused or broad cargo tests?
- [x] Does the review loop run at least 3 times and stop only after 2
      consecutive clean significant-issue passes?

## ✅ Implementation Evidence

- T0/T3 guardrails landed in
  `crates/z00z_storage/tests/test_phase051_guardrails.rs`.
- T1 facade and compatibility backend contract coverage landed in
  `crates/z00z_storage/tests/assets/test_backend_facade_contract.rs` and
  existing `test_store_api.rs` anchors.
- T2 proof and root checks landed in `test_assets_suite`,
  `test_checkpoint_root_binding.rs`, and `test_claim_source_proof.rs`.
- T4/T5 golden, reject, reload, checkpoint, and path-index coverage landed in
  `crates/z00z_storage/tests/test_phase051_golden_corpus.rs` and existing
  RedB, checkpoint, serialization, and search tests.
- T6 closeout evidence is recorded in `051-06-SUMMARY.md`, `051-SUMMARY.md`,
  `.planning/STATE.md`, `.planning/ROADMAP.md`, and `052-TODO.md`.

## ✅ Done Condition

- `051-TEST-SPEC.md` scenarios `051-SC-01` through `051-SC-15` each have a
  concrete test home and implementation wave.
- `051-TEST-SPEC.md` examples `051-EX-01` through `051-EX-06` each have at
  least one implemented test or explicit future-pending reason matching the
  spec's pass and failure conditions.
- The focused storage, wallet, simulator, and source-shape gates are green.
- The broad `cargo test --release --features test-fast --features
  wallet_debug_dump` command is run when relevant to the implemented Rust/test
  changes.
- No test or helper introduces a parallel storage authority, fake forest
  backend, duplicate checkpoint verifier, duplicate proof decoder, second
  wallet scan cursor model, second replay registry, or simulator-owned proof
  rule.
- Closeout artifacts can be synchronized without guessing what the tests prove.
