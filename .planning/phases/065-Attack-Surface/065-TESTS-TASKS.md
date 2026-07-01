---
phase: 065-Attack-Surface
artifact: tests-tasks
status: planned
source: 065-TEST-SPEC.md
updated: 2026-06-30
---

# Phase 065 Tests Tasks

## Purpose

📌 This document translates `065-TEST-SPEC.md` into one concrete implementation
order for Phase 065 test work.

📌 It is intended to be directly usable by another engineer or agent without
guessing which wave comes first, which seams must be extended instead of
duplicated, which audit scripts are phase-closing, or which command bundles are
required after each wave.

📌 This artifact plans test implementation only. It does not approve fake
remote truth, docs-only closure for code behavior, or weakening any Phase 065
gate to make a test pass.

## Scope Inputs

📌 This breakdown is derived from:

- `.planning/phases/065-Attack-Surface/065-TEST-SPEC.md`
- `.planning/phases/065-Attack-Surface/065-CONTEXT.md`
- `.planning/phases/065-Attack-Surface/065-TODO.md`
- `.planning/phases/065-Attack-Surface/065-01-PLAN.md`
- `.planning/phases/065-Attack-Surface/065-02-PLAN.md`
- `.planning/phases/065-Attack-Surface/065-03-PLAN.md`
- `.planning/phases/065-Attack-Surface/065-04-PLAN.md`
- `.planning/phases/065-Attack-Surface/065-05-PLAN.md`
- `.planning/phases/065-Attack-Surface/065-06-PLAN.md`
- `.planning/phases/065-Attack-Surface/065-07-PLAN.md`
- `.planning/phases/065-Attack-Surface/065-08-PLAN.md`
- `.planning/phases/065-Attack-Surface/065-09-PLAN.md`

📌 Completion artifacts are still absent for Phase 065, so this implementation
order remains truthful `planned` work rather than executed proof.

## Execution Strategy

📌 Phase 065 tests should be implemented in the same dependency order as the
phase plans, not by alphabetical filename order.

📌 The canonical execution sequence is:

1. Lock the reuse map and fallback-ready status before touching tests.
2. Close theorem-bound validator acceptance before seal-only persistence,
   because `WS-01` defines what the accepted path must carry and `WS-02`
   defines how the accepted artifact is born.
3. Close release build policy before simulator truth and wallet privilege,
   because build flags must stop reopening debug paths while behavior tests are
   being expanded.
4. Close simulator truth before privilege and mutation ownership, because
   public evidence semantics must be stable before wallet flows depend on them.
5. Close privilege before mutation and restore ownership, because the mutation
   and restore surfaces are privileged.
6. Close fail-closed boundary and meta-gates before public RPC truth, because
   transport redaction and source-audit policy are cross-cutting safety rails.
7. Close public RPC truth before the final doc sweep, because the wording audit
   must reflect the final code-facing truth.
8. Re-run seal-only regression anchors during the final sweep so active work
   cannot silently weaken already-closed boundaries.

## Gate Coverage

📌 The implementation waves below close the canonical gates explicitly:

- `G-01` and `G-02` close in `Wave T1`
- `G-03` and `G-04` close in `Wave T2`
- `G-05` closes in `Wave T5`
- `G-06` and `G-07` close in `Wave T6`
- `G-08` and `G-09` close in `Wave T8`

📌 `WS-03`, `WS-04`, `WS-07`, and `WS-09` are companion hardening scopes
without standalone `G-*` ids, but they remain phase-closing and keep their own
scenario bundles in `Waves T3`, `T4`, `T7`, and `T9`.

## Task Waves

### Wave T0: Harness Confirmation And Fallback Lock-In

📌 Objective: confirm the extend-versus-create map and freeze the truthful
`fallback-ready` boundary before any test file generation starts.

📌 Files to inspect before implementation:

- all files listed under `Existing Test Anchors To Reuse` in
  `065-TEST-SPEC.md`
- all files listed under `Proposed New Test Files And Audit Scripts` in
  `065-TEST-SPEC.md`

📌 Deliverables:

- one explicit extend-versus-create decision per `065-S01` through `065-S20`
- one shared note that Phase 065 remains `fallback-ready` until executed
  summaries and verification artifacts exist
- one missing-file inventory for the new scripts and
  `crates/z00z_networks/rpc/tests/test_wasm_client_redaction.rs`

📌 Completion gate:

- no scenario still has an ambiguous home
- no planned test duplicates an existing seam without justification
- no one treats the absent `065-SUMMARY.md` or `065-VERIFICATION.md` as if
  they already exist

### Wave T1: `065-S01` Through `065-S03` Validator Theorem Ownership

📌 Why first: `WS-01` is the top-priority attack surface and defines the
accepted-path proof contract used by later storage and simulator waves.

📌 Extend these files:

- `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs`

📌 Required implementation tasks:

1. Add one coherent theorem-bundle success path that binds artifact, exec
   input, link, route digest, and publication digest together.
2. Add negative coverage for bad link, bad exec-input id, bad prep-snapshot id,
   bad route digest, detached proof bytes, and publication-binding drift.
3. Add one explicit `Accepted`-unreachable path when theorem inputs are absent.

📌 Success conditions:

- validator acceptance cannot succeed without theorem-owned inputs
- rollup, validator, and simulator tests all describe one accepted theorem
  story, not three loosely related stories

📌 Command gate:

```bash
cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture
cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard -- --nocapture
cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture
```

### Wave T2: `065-S04` Through `065-S06` Seal-Only Checkpoint Persistence

📌 Why here: once theorem ownership is fixed, the next truth boundary is how
canonical checkpoint artifacts are created and consumed.

📌 Extend these files:

- `crates/z00z_storage/tests/test_checkpoint_store.rs`
- `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
- `crates/z00z_storage/tests/test_checkpoint_link_injective.rs`
- `crates/z00z_simulator/tests/scenario_1/test_stage6_checkpoint_storage_bridge.rs`

📌 Required implementation tasks:

1. Prove `seal_artifact()` is the only canonical birth lane for final
   artifacts.
2. Add write-time failures for `save_link()` before evidence rows exist.
3. Add rejection coverage for mismatched snapshot ids, exec-input ids, and
   statement or proof pair drift.
4. Add one end-to-end proof that stage-12 finalization and validator acceptance
   reject raw-lane artifacts.

📌 Success conditions:

- raw-lane artifacts remain quarantined and cannot impersonate canonical final
  artifacts
- link bind and store authority are both enforced at write time and load time

📌 Command gate:

```bash
cargo test --release -p z00z_storage --test test_checkpoint_store -- --nocapture
cargo test --release -p z00z_storage --test test_checkpoint_finalization -- --nocapture
cargo test --release -p z00z_storage --test test_checkpoint_link_injective -- --nocapture
cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture
```

### Wave T3: `065-S07` Release-Capable Build Hardening

📌 Why here: simulator, wallet, and RPC truth tests should not be expanded
while forbidden feature combinations can still reopen the closed surface.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_production_hardening.rs`
- `crates/z00z_wallets/tests/test_live_boundary_claims.rs`

📌 Create this file:

- `scripts/audit_release_feature_guards.sh`

📌 Required implementation tasks:

1. Add explicit expected-failure checks for release-capable builds using
   `test-params-fast` and `wallet_debug_tools`.
2. Add source-audit checks proving public cache corruption hooks and wallet
   secret debug exporters do not remain available to release-capable builds.
3. Keep docs and test names honest: a green result here proves build-policy
   closure, not runtime feature delivery.

📌 Success conditions:

- forbidden release-feature combinations fail fast
- public debug export and corruption hooks disappear from release-capable
  surfaces

📌 Command gate:

```bash
bash scripts/audit_release_feature_guards.sh
cargo test --release -p z00z_wallets --test test_production_hardening -- --nocapture
cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture
```

### Wave T4: `065-S08` Through `065-S09` Simulator Evidence Truth

📌 Why here: after build policy is locked, public-lane simulator semantics can
be tightened without a moving debug-feature boundary.

📌 Extend these files:

- `crates/z00z_simulator/tests/scenario_1/test_stage6_checkpoint_final_gate.rs`
- `crates/z00z_simulator/tests/scenario_1/test_wallet_integration.rs`

📌 Required implementation tasks:

1. Add release-mode or public-lane rejection for `Stage6ProofMode::DraftOnly`.
2. Add packet verification rejection for synthetic checkpoint ids and draft-only
   publication evidence.
3. Preserve the existing public secret-lane anti-regression checks.

📌 Success conditions:

- draft and debug runs cannot impersonate final-publication truth
- the default public lane remains secret-free

📌 Command gate:

```bash
cargo test --release -p z00z_simulator --test scenario_1 -- --nocapture
cargo test --release -p z00z_storage --test test_checkpoint_finalization -- --nocapture
```

### Wave T5: `065-S10` Through `065-S12` Privileged Capability Sealing

📌 Why here: privileged wallet paths are the next attack surface after the
public simulator lane is stable.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_sensitive_rpc_session.rs`
- `crates/z00z_wallets/tests/test_wallet_capability_matrix.rs`
- `crates/z00z_wallets/tests/test_stealth_output.rs`
- `crates/z00z_wallets/tests/test_rpc_route_coverage.rs`
- `crates/z00z_wallets/scripts/audit_rpc_method_wiring.sh`

📌 Required implementation tasks:

1. Convert handler-level session checks into capability-typed proofs in tests.
2. Make a new privileged handler without typed guard fail the route audit.
3. Add public-API assertions that the raw stealth builder is visibly
   noncanonical.
4. Add explicit unsupported-wasm capability assertions.

📌 Success conditions:

- privileged wallet code cannot compile or register routes without typed
  capability proof
- raw stealth builders are visibly unsafe, internal, or validation-free
- native and wasm capability truth is explicit

📌 Command gate:

```bash
cargo test --release -p z00z_wallets --test test_sensitive_rpc_session -- --nocapture
cargo test --release -p z00z_wallets --test test_wallet_capability_matrix -- --nocapture
cargo test --release -p z00z_wallets --test test_stealth_output -- --nocapture
cargo test --release -p z00z_wallets --test test_rpc_route_coverage -- --nocapture
bash crates/z00z_wallets/scripts/audit_rpc_method_wiring.sh
```

### Wave T6: `065-S13` Through `065-S15` Mutation Truth And Restore Truth

📌 Why here: local mutation and restore flows are privileged and should follow
after capability sealing.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_asset_rpc_mutations.rs`
- `crates/z00z_wallets/tests/test_wallet_restore_atomic.rs`
- `crates/z00z_wallets/tests/test_chain_broadcast_retry.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`

📌 Required implementation tasks:

1. Prove every public local mutation RPC routes through one executor.
2. Prove tx-id truth comes from the canonical owner, not ad hoc formatting.
3. Add failpoint coverage for `history_commit`, `.wlt` commit, publish,
   rollback failure, and retry.
4. Keep rotate-master-key wording and receipt truth aligned with the persisted
   behavior while the broader mutation contract is hardened.

📌 Success conditions:

- one sealed service owns local mutation truth
- restore crash and retry semantics are explicit and deterministic
- tx-history and broadcast lifecycle stay coherent under repeated failures

📌 Command gate:

```bash
cargo test --release -p z00z_wallets --test test_asset_rpc_mutations -- --nocapture
cargo test --release -p z00z_wallets --test test_wallet_restore_atomic -- --nocapture
cargo test --release -p z00z_wallets --test test_chain_broadcast_retry -- --nocapture
cargo test --release -p z00z_wallets --test test_tx_store_integration -- --nocapture
```

### Wave T7: `065-S16` Through `065-S17` Fail-Closed Boundary And Meta-Gates

📌 Why here: once runtime privilege and lifecycle truth are stable, the
cross-cutting fail-closed and hygiene policies can be frozen with executable
audits.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`

📌 Create these files:

- `crates/z00z_networks/rpc/tests/test_wasm_client_redaction.rs`
- `scripts/audit_secret_type_hygiene.sh`
- `scripts/audit_secret_eq_hygiene.sh`
- `scripts/audit_crypto_rng_hygiene.sh`
- `scripts/audit_boundary_panic_hygiene.sh`
- `scripts/audit_log_redaction_hygiene.sh`

📌 Supporting CI artifact to create:

- `.github/workflows/security-hygiene-guards.yml`

📌 Required implementation tasks:

1. Add redaction tests proving transport layers never log raw params or raw
   responses.
2. Add boundary-failure tests proving public constructors return typed errors
   instead of panicking.
3. Create executable source-audit scripts for secret-type, equality, RNG,
   panic, and logging hygiene.
4. Wire the new scripts into CI through the dedicated workflow.

📌 Success conditions:

- no security-boundary constructor panic survives
- no raw secret-bearing transport JSON survives
- meta-gates become executable and CI-enforced rather than memory-based policy

📌 Command gate:

```bash
cargo test --release -p z00z_storage --test test_live_guardrails -- --nocapture
cargo test --release -p z00z_wallets --test test_rpc_logging_acceptance -- --nocapture
cargo test --release -p z00z_wallets --test test_rpc_logging_risk_policy -- --nocapture
cargo test --release -p z00z_networks_rpc -- --nocapture
bash scripts/audit_secret_type_hygiene.sh
bash scripts/audit_secret_eq_hygiene.sh
bash scripts/audit_crypto_rng_hygiene.sh
bash scripts/audit_boundary_panic_hygiene.sh
bash scripts/audit_log_redaction_hygiene.sh
```

### Wave T8: `065-S18` Through `065-S19` Public RPC Truth And DTO Cleanup

📌 Why here: public contract truth should be addressed only after transport
redaction and repository-wide hygiene rules are in place.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_rpc_truth.rs`
- `crates/z00z_wallets/tests/test_rpc_types_serialization.rs`
- `crates/z00z_wallets/tests/test_stub_behavior.rs`
- `crates/z00z_wallets/tests/test_runtime_validation_result.rs`

📌 Required implementation tasks:

1. Add truth tests proving chain scan and chain tip are either truthful local
   semantics or explicitly demoted from production-looking claims.
2. Remove or demote placeholder proof fields from production DTO defaults.
3. Keep stub-only defaults out of production-facing types.
4. Keep validation result semantics aligned between runtime and RPC layers.

📌 Success conditions:

- public RPCs stop overclaiming network or proof truth
- DTOs stop advertising placeholder proof semantics as if they were finalized

📌 Command gate:

```bash
cargo test --release -p z00z_wallets --test test_rpc_truth -- --nocapture
cargo test --release -p z00z_wallets --test test_rpc_types_serialization -- --nocapture
cargo test --release -p z00z_wallets --test test_stub_behavior -- --nocapture
cargo test --release -p z00z_wallets --test test_runtime_validation_result -- --nocapture
```

### Wave T9: `065-S20` Final Wording Sweep And Seal-Only Regression Pack

📌 Why here: the wording audit must run after code and public contract truth are
settled, and the regression pack must confirm active work did not weaken
already-closed boundaries.

📌 Extend these files:

- `crates/z00z_storage/tests/test_claim_source_proof.rs`
- `crates/z00z_wallets/tests/test_object_quarantine.rs`
- `crates/z00z_storage/tests/test_object_reject_codes.rs`
- `crates/z00z_wallets/tests/test_claim_resume_core.rs`
- `crates/z00z_simulator/tests/scenario_1/test_claim_resume.rs`
- `crates/z00z_simulator/tests/scenario_1/test_wallet_integration.rs`
- `crates/z00z_core/tests/test_live_guardrails.rs`

📌 Create this file:

- `scripts/audit_phase065_narrowed_wording.sh`

📌 Required implementation tasks:

1. Add one wording audit that bans stale references to retired historical
   claims.
2. Re-run or extend claim-source, quarantine, reject-code, resume, and
   public-secret-lane regression anchors under the final Phase 065 code.
3. Keep the regression pack honest: it proves already-closed boundaries stay
   closed, not that new work has been verified independently of the earlier
   waves.

📌 Success conditions:

- no human-readable artifact re-promotes retired claims as current blockers
- all seal-only regression anchors remain green after Phase 065 implementation

📌 Command gate:

```bash
bash scripts/audit_phase065_narrowed_wording.sh
cargo test --release -p z00z_storage --test test_claim_source_proof -- --nocapture
cargo test --release -p z00z_wallets --test test_object_quarantine -- --nocapture
cargo test --release -p z00z_storage --test test_object_reject_codes -- --nocapture
cargo test --release -p z00z_wallets --test test_claim_resume_core -- --nocapture
cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture
```

### Wave T10: Full Verification Bundle

📌 Objective: run the whole Phase 065 proof bundle after all test files and
audit scripts exist.

📌 Required verification order:

1. `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
2. all wave-specific command gates from `T1` through `T9`
3. `cargo test --release`
4. repeated review and fix cycles required by the numbered plans

📌 Completion gate:

- every `WS-01` through `WS-09` command bundle is green
- every new audit script exists and is runnable from repo root
- new CI workflow wiring exists for the meta-gates
- Phase 065 can then produce execution summaries and only then move toward a
  verification-backed state

## Sequencing Notes

- Do not start with `WS-08` or `WS-09`. Those waves depend on the earlier
  runtime and policy truth being locked first.
- Do not create a parallel omnibus test file for Phase 065. Keep one focused
  file per seam and reuse the existing anchors named in `065-TEST-SPEC.md`.
- Do not replace negative runtime tests with source-audit checks where the seam
  is already executable. Phase 065 explicitly requires both runtime proof and
  source-audit proof where the attack surface spans both.
- Do not treat source-audit scripts as optional housekeeping. For `WS-03`,
  `WS-07`, and `WS-09`, the scripts are phase-closing artifacts.
- Keep `WS-08` honest: the tests may prove truthful demotion or removal of
  production-looking claims. They must not force fake remote-backed behavior.
