---
phase: 040-spend-proof
artifact: tests-tasks
status: execution-reviewed
source: 040-TEST-SPEC.md
updated: 2026-04-29
---

# Phase 040 Tests Tasks

## 🎯 Purpose

📌 This document translates `040-TEST-SPEC.md` into one concrete execution
order for test work.

📌 The order stays subordinate to the numbered Phase 040 planning chain. Tests
must reinforce the canonical implementation sequence rather than inventing a
parallel lane.

📌 The primary Phase 040 test files already exist in the repo. This artifact
therefore acts as a synchronization and rerun order, not a greenfield
file-creation checklist.

## ✅ Execution Status

- T1 through T4 are landed and remain the primary focused rerun matrix for the
  current Phase 040 workspace truth.
- T1 ownership now includes the live backend theorem-boundary seam in
  `crates/z00z_wallets/tests/test_spend_proof_backend.rs`, not only the wire
  and statement files.
- T1 ownership now includes the 2026-04-29 direct backend verifier hardening:
  forged deterministic artifacts reject on public range, balance, and
  input/output theorem leaf overlap drift before artifact acceptance.
- T1 and T2 ownership now track the output theorem projection where
  `SpendProofStmt.output_leaves[].asset_id` carries the output `leaf_ad_id`
  namespace for theorem overlap checks while storage/package asset IDs remain
  bound by canonical statement bytes.
- T5 remains conditional and should stay a no-op unless `040-08` changes
  builder, output-flow, or wording-facing output surfaces.
- T6 owns the residual documentation sync, shortcut-quarantine sweep, rollup
  public-artifact binding guard sync, and rerun bundle required to keep the
  Phase 040 test artifacts honest about the internal theorem boundary.

## 📥 Scope Inputs

- `040-TEST-SPEC.md`
- `040-CONTEXT.md`
- `040-TODO.md`
- `040-01-PLAN.md` through `040-08-PLAN.md`
- `040-10-PLAN.md`
- `040-01-SUMMARY.md` through `040-08-SUMMARY.md`
- `040-VALIDATION.md`
- `040-UAT.md`
- live spend-proof seams in `crates/z00z_wallets` and `crates/z00z_simulator`
- existing spend/checkpoint/stage-surface test anchors already present in the
  repo

## 🧭 Execution Strategy

- Freeze reuse anchors first so new tests do not duplicate already-green
  wallet/simulator seams that already express the current fail-closed truth.
- Land wallet proof-carrier and statement tests before simulator roundtrip
  tests, because Stage 4 and Stage 6 should inherit canonical wallet truth,
  not redefine it.
- Keep the simulator residual regression for
  `local wire ok, public spend fail` ahead of broad roundtrip additions,
  because it closes the explicit TODO follow-up at the correct boundary.
- Re-check the live backend theorem boundary before treating the closeout
  waves as documentation-only, because `040-09` stays audit-reopened in
  `040-VALIDATION.md` and `040-UAT.md`.
- Defer `040-08` output-cleanup testing until `040-01` through `040-07` are
  stable. Output cleanup is a bounded follow-up, not a prerequisite.
- Treat shortcut and closeout guards as the last wave so they lock the landed
  slice instead of pre-judging unfinished work.
- Treat T1 through T4 as landed-file ownership waves first: verify the current
  file inventory, rerun focused gates, and only then extend scenarios that are
  still missing from the contract.

## 🌊 Task Waves

### Wave T0: Harness And Reuse Lock-In

- files to inspect:
  - `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
  - `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
  - `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`
  - `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`
  - `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
  - `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
  - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
  - `crates/z00z_rollup_node/src/lib.rs`
  - `crates/z00z_rollup_node/tests/test_settlement_theorem.rs`
- deliverables:
  - confirmed reuse inventory for the wallet producer, backend, and verifier
    seams
  - confirmed reuse inventory for the simulator spend-gate and checkpoint seams
  - explicit note that browser E2E remains out of scope
  - explicit note that shadow proof-layer files remain prohibited
- completion gate:
  - no new test file duplicates an already-owned canonical seam

### Wave T1: Proof Carrier And Statement Contract

- priority: highest
- why now:
  - `040-01` and `040-02` define the proof carrier and statement surface that
    every later scenario depends on
- files already landed:
  - `crates/z00z_wallets/tests/test_spend_proof_wire.rs`
  - `crates/z00z_wallets/tests/test_spend_statement.rs`
  - `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
- implementation tasks:
  - lock non-empty proof/auth roundtrip
  - lock version handling for proof and auth
  - lock deterministic statement encoding for fixed tx facts
  - lock public-input binding for inputs, outputs, chain scope, and root scope
  - lock the live backend suite boundary and fail-closed theorem-artifact
    rejection
  - lock direct backend verification against forged deterministic artifacts for
    public range, balance, and input/output theorem leaf overlap drift
  - lock output theorem leaves to the output `leaf_ad_id` namespace inside
    `SpendProofStmt` without changing storage/package asset ID binding
- success conditions:
  - placeholder carrier is not accepted as Phase 040 success
  - statement drift breaks auth verification
  - no alternate statement encoder is implicitly allowed
  - forged deterministic backend artifacts cannot bypass public relation checks
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_proof_wire -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_statement -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_proof_backend -- --nocapture`

### Wave T2: Producer And Public Verifier Contract

- priority: second
- why now:
  - `040-03` and `040-04` must agree on one producer/verifier contract before
    the simulator can prove end-to-end continuity
- files already landed:
  - `crates/z00z_wallets/tests/test_spend_prover_contract.rs`
  - `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
- implementation tasks:
  - prove producer output verifies through the canonical public verifier
  - cover `SpendBuildErr` fail-closed producer rejects
  - cover the current seam-local negative matrix for public verifier rejects
  - keep distributed N09 through N14 ownership truthful, including explicit
    partial and missing rows on the live verifier seam
  - prove recomputed currently-owned leaf, range-proof, and nullifier checks
    are fail-closed without overclaiming uncovered balance or output-leaf rows
- success conditions:
  - untouched producer output verifies
  - mutated public facts reject with typed canonical errors
  - no local wire-only success is confused with semantic proof success
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_prover_contract -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --test test_tx_proof_verifier -- --nocapture`

### Wave T3: Nullifier Semantics And Full Package Boundary

- priority: third
- why now:
  - `040-05` and `040-06` close replay safety and the canonical full-verifier
    boundary that the simulator later depends on
- files already landed or to extend:
  - `crates/z00z_wallets/tests/test_spend_nullifier_semantics.rs`
  - `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`
- implementation tasks:
  - lock deterministic same-scope nullifier behavior
  - lock scope drift and replay rejection
  - lock separation from claim nullifier semantics
  - extend full verifier coverage for wire-digest-only or local-wire-only
    shortcut attempts
  - prove replay or uniqueness closure at the checkpoint or state boundary
    before treating `040-05` as closed
  - prove the residual simulator regression where local wire checks pass but
    the public spend contract fails before treating `040-06` as closed
- success conditions:
  - replay safety is explicit and state-enforced at the authoritative
    checkpoint or storage seam, not only in wallet-local helpers
  - full verifier remains the only canonical full acceptance seam across both
    wallet and simulator ownership paths
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_nullifier_semantics -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --lib core::tx::tx_verifier::tests -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`

### Wave T4: Simulator Spend Gate And Roundtrip

- priority: fourth
- why now:
  - wallet-side truth must already be pinned before Scenario 1 proves end-to-end
    producer-to-consumer compatibility
- files already landed or to extend:
  - `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
  - `crates/z00z_simulator/tests/test_scenario1_tx_proof_roundtrip.rs`
  - `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
- implementation tasks:
  - prove Stage 4 to Stage 6 roundtrip keeps statement and root continuity
  - prove Stage 11 blocks authoritative mutation after statement-bound
    `chain_id` drift with recomputed `tx_digest_hex`, because fee drift is
    caught earlier by the local verifier gate
  - keep range-proof mode explicit in scenario output and assertions
  - rerun stage-surface honesty locks as part of the same `040-07` wave so
    current proof-scope wording and post-`040-05` truth remain synchronized
- success conditions:
  - simulator path agrees with wallet verifier truth
  - Stage 11 fail-closed behavior is proven through authoritative artifact
    absence
  - no scenario rewrites bound statement fields to make the path pass
- command gate:
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_tx_proof_roundtrip -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`

### Wave T5: Optional Output Follow-Up And Residual Surface Recheck

- priority: fifth
- why now:
  - source-shape honesty is already locked in Wave T4 for `040-07`, while this
    wave keeps only the optional output-cleanup follow-up and any residual
    wording recheck caused by that cleanup
- files to extend:
  - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
  - `crates/z00z_wallets/src/core/tx/builder.rs`
  - `crates/z00z_wallets/src/core/tx/output_flow.rs`
- implementation tasks:
  - only if `040-08` is actually executed, add output-cleanup parity tests for
    `leaf_ad`, `tag16`, commitment, range-proof, and self-decrypt behavior
  - if `040-08` edits wording-facing surfaces, rerun the stage-surface honesty
    check so post-`040-05` and post-cleanup truth stays synchronized
- success conditions:
  - wording does not overclaim proof completeness
  - output cleanup introduces no proof-facing or receiver-facing behavior drift
- command gate:
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast core::tx::builder --lib -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast core::tx::output_flow --lib -- --nocapture`

### Wave T6: Shortcut Quarantine And Final Residual Sweep

- priority: last
- why now:
  - `040-09` through `040-14` are closeout guards over the landed slice, not a
    substitute for the main proof tests
- files to extend or create:
  - existing owning anchors wherever possible
  - `crates/z00z_wallets/tests/test_phase040_spend_proof_guards.rs` only if a
    dedicated phase-local quarantine file is still needed after T0 through T5
- implementation tasks:
  - add source-shape or behavior guards for prohibited shortcuts
  - rerun the backend theorem-boundary seam so the reopened live suite stays
    explicit in the closeout artifacts
  - rerun or account for the rollup public-artifact binding guard so checkpoint
    artifact, link, execution input, spend root, tx inclusion, and public
    package theorem binding stay synchronized without full settlement proof
    overclaim
  - re-check that no browser E2E or shadow proof files were introduced
  - sync `040-TEST-SPEC.md` and this file to the actual landed coverage
- success conditions:
  - no test falsely promotes a parallel proof layer into canonical authority
  - no test claims STARK support before non-empty carrier and verifier wiring
  - planning docs remain synchronized with the actual test surface
- command gate:
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_proof_backend -- --nocapture`
  - `cargo test -p z00z_rollup_node --release --test test_settlement_theorem -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --lib --tests -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --tests -- --nocapture`

## 📋 Review Checklist Per Wave

- [x] Does the wave extend an existing owning seam before creating a new file?
- [x] Does the scenario prove a real Phase 040 gap instead of re-proving an
      already-green baseline behavior?
- [x] Does the assertion map to the canonical function or runtime path named in
      `040-TEST-SPEC.md`?
- [x] Does the wave preserve the no-parallel-layer rule from `040-CONTEXT.md`?
- [x] Does the simulator wave prove authoritative apply blocking rather than
      only observational artifact drift?
- [x] If `040-08` is untouched, does the task list keep output-cleanup tests
      deferred rather than pretending they are already required?

## ✅ Done Condition

- T1 through T4 remain present as real files, with explicit rerun-backed
  evidence against the current codebase truth and T1 kept honest as landed
  carrier, statement, and backend ownership.
- T5 only lands output-cleanup tests when the optional follow-up actually
  changes code.
- T6 leaves no unresolved shortcut guard or stale theorem-boundary wording that
  would allow Phase 040 closeout to overstate proof maturity or
  checkpoint-boundary reuse, and it keeps the rollup guard scoped to public
  artifact binding rather than full settlement proof closure.
- The final coverage still proves one canonical producer path, one canonical
  public verifier path, one canonical full package verifier, and one canonical
  checkpoint-apply boundary plus one rollup public-artifact binding guard while
  keeping the live backend boundary explicit until a real public theorem
  backend exists.
