---
phase: 033-crypto-audit-scenario-2
artifact: tests-tasks
status: complete
source: 033-TEST-SPEC.md
updated: 2026-04-09
---

# Phase 033 Tests Tasks

## 🎯 Purpose

📌 This document translates `033-TEST-SPEC.md` into one concrete
implementation order for Phase 033 test work.

📌 The order below is deliberately dependency-driven. Claim tuple and spend
boundary tests land before caution-language or governance checks so that later
documentation and reclassification rows only reuse proven executable seams.

📌 This artifact now records the executed Phase 033 reuse-first closeout.
The original dependency order remains useful, but the phase-local summaries,
UAT, and validation artifacts are the authoritative execution truth.

## 📌 Scope Inputs

- `033-TEST-SPEC.md`
- `033-CONTEXT.md`
- `033-TODO.md`
- `033-01-PLAN.md` through `033-23-PLAN.md`
- `033-EXAM-QUESTIONS-AND-ANSWERS-1.md`
- `033-EXAM-QUESTIONS-AND-ANSWERS-2.md`
- `033-EXAM-QUESTIONS-AND-ANSWERS-3.md`
- `033-32FULL-AUDIT.md`
- `.planning/REQUIREMENTS.md`

## 🧭 Execution Strategy

📌 Phase 033 test work was executed in this dependency order:

1. Lock in reusable test homes before adding any new files.
2. Freeze claim tuple and reject semantics before continuity/governance rows.
3. Freeze ownership, request-route, and spend-boundary behavior before
   checkpoint and caution waves.
4. Freeze checkpoint accepted-path continuity before replay, secret, and RNG
   scope.
5. Run documentation/governance rows only after the executable seams they rely
   on have been proven or explicitly left open.
6. Leave tasks 63-65 as final dedicated closure gates so the row-swapped
   high-severity findings stay isolated and honest.

📌 Later waves depend on earlier ones because Phase 033 contains many caution
and reclassification rows that are only valid if the underlying live seam tests
already exist.

## 🌊 Task Waves

### Wave T0: Harness And Traceability Lock-In

📌 Objective: freeze exact extend-versus-create ownership before writing any new
Phase 033 tests.

📌 Files to inspect:

- `crates/z00z_crypto/tests/test_claim_v2_contract.rs`
- `crates/z00z_storage/tests/test_claim_source_proof.rs`
- `crates/z00z_simulator/tests/test_claim_pkg_runtime.rs`
- `crates/z00z_simulator/tests/test_claim_persist.rs`
- `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
- `crates/z00z_wallets/tests/test_e2e_req_flow.rs`
- `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
- `crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs`
- `crates/z00z_storage/tests/test_checkpoint_link_injective.rs`
- `crates/z00z_simulator/tests/test_stage2_secret_artifacts.rs`

📌 Deliverables:

- one explicit extend-versus-create decision for each scenario in
  `033-TEST-SPEC.md`;
- one fixture note for claim package, spend gate, checkpoint rehydrate, Stage 2
  secret, and RNG selection helpers;
- one note recording which scenarios remain artifact-only gates.

📌 Completion gate:

- no scenario still has an ambiguous test home;
- no new test file duplicates an existing seam without a written reason.

### Wave T1: Claim Verifier Contract And Reject Taxonomy

📌 Priority: highest.

📌 Why first: tasks 1, 3, 28, 29, and 31 define the narrow claim-verifier truth
used later by continuity, authority, and documentation rows.

📌 Executed test homes:

- `crates/z00z_crypto/tests/test_claim_v2_contract.rs`
- `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`
- `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`

📌 Required implementation tasks:

1. Add direct post-sign mutation coverage for `claim_source_asset_id` and
   `chain_id`.
2. Add plausible-package-shape mutation coverage for recipient-binding and
   asset-path drift.
3. Add explicit assertions for `SourceRootVer` and `SourceProofVer`.
4. Tighten stale-proof consumer assertions so they stop relying on loose
   substring matching.
5. Keep canonical slot 31 traceability visible in test names or comments.

📌 Success conditions:

- tuple drift rejects through the live verifier boundary;
- reject taxonomy remains category-specific and stable;
- no test implies broader claim closure than the verifier actually proves.

📌 Command gate:

```bash
cargo test -p z00z_wallets --release claim_tx_tests -- --nocapture
cargo test -p z00z_crypto --release --test test_claim_v2_contract -- --nocapture
cargo test -p z00z_simulator --release --test test_claim_pkg_runtime -- --nocapture
```

### Wave T2: Claim Continuity And Publish-Bound Package Shape

📌 Priority: second.

📌 Why here: tasks 2, 4, 30, 32, 58, 59, and 63 all depend on an honest claim
continuity story and on explicit package-shape rejection.

📌 Extend these files:

- `crates/z00z_storage/tests/test_claim_source_proof.rs`
- `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`
- `crates/z00z_simulator/tests/test_claim_persist.rs`

📌 Required implementation tasks:

1. Separate helper-owned claim continuity from persisted continuity in storage
   assertions.
2. Add tests that reject wrong bundle version and implicit discriminator shapes.
3. Add one explicit assertion that accepted continuity is still helper-owned if
   persisted membership is not yet wired.
4. Keep task-63 high-severity wording isolated from spend and checkpoint gaps.

📌 Success conditions:

- claim continuity cannot be silently described as persisted authority;
- publish-bound package shape rejects implicit or defaulted forms;
- task 63 remains a dedicated explicit open or closed gate.

📌 Command gate:

```bash
cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture
cargo test -p z00z_simulator --release --test test_claim_pkg_crypto_support -- --nocapture
cargo test -p z00z_simulator --release --test test_claim_persist -- --nocapture
```

### Wave T3: Ownership, `leaf_ad_id`, And Request-Route Boundaries

📌 Priority: third.

📌 Why here: tasks 5-9, 33-37, 49, and 57 reuse the same wallet-local
ownership and route-selection boundaries.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
- `crates/z00z_wallets/tests/test_e2e_req_flow.rs`

📌 Required implementation tasks:

1. Add or tighten `leaf_ad_id` drift assertions for canonical flow only.
2. Keep receiver-secret plus `s_out` ownership wording wallet-local.
3. Add route-selection assertions that preserve request-bound preference and
   card-bound compatibility.
4. Preserve the task-9 carry-forward caution row as an active guardrail.

📌 Success conditions:

- ownership drift is visible and fail closed on canonical flow;
- request and card routes are testably distinct;
- no test implies sender ignorance or public trustless exclusivity.

📌 Command gate:

```bash
cargo test -p z00z_wallets --release --test test_spend_witness_gate -- --nocapture
cargo test -p z00z_wallets --release --test test_e2e_req_flow -- --nocapture
```

### Wave T4: Narrow Public Spend Boundary And Nullifier Gap

📌 Priority: fourth.

📌 Why here: tasks 10, 13, 38, 39, 40, 53, and 64 all depend on one truthful
answer about what the current public spend contract does and does not prove.

📌 Extend these files:

- `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`

📌 Required implementation tasks:

1. Freeze the current public spend boundary as the delivered narrow contract.
2. Add direct negative coverage for semantically incomplete spend artifacts if
   code is stronger than tests today.
3. Keep nullifier semantics named as the exact open element.
4. Preserve the task-64 high-severity row wording verbatim in comments or test
   metadata if it is referenced.

📌 Success conditions:

- the narrow public spend boundary is proven without flattening into full
  `PH32-SPEND` closure;
- semantic acceptance precedes state mutation;
- nullifier semantics remain the explicit open gap unless code actually lands.

📌 Command gate:

```bash
cargo test -p z00z_simulator --release --test test_scenario1_spend_gate -- --nocapture
```

### Wave T5: Checkpoint Continuity, Draft-Final Separation, Replay, And Backend Boundary

📌 Priority: fifth.

📌 Why here: tasks 12, 15-18, 41-43, 50-52, 55, 60, and 65 share the same
checkpoint package-coupled boundary and must be tested together to avoid
contradictory claims.

📌 Extend these files:

- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
- `crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs`
- `crates/z00z_storage/tests/test_checkpoint_link_injective.rs`

📌 Reuse-first execution result:

- The backend-boundary assertions were absorbed into
   `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` together with the
   existing persisted replay and link suites; no standalone
   `test_checkpoint_backend_boundary.rs` file was created.

📌 Required implementation tasks:

1. Keep draft/final checkpoint classes disjoint.
2. Prove raw artifact versus persisted binding distinctions stay explicit.
3. Tighten replay/stale tests, especially spent-row rehydrate replay.
4. Keep compatibility-looking payload-only backend closure explicit through the
   accepted-path checkpoint suite, persisted replay/link suites, and late
   wording guards.
5. Preserve the task-65 high-severity row wording verbatim if referenced inside
   test comments or metadata.

📌 Success conditions:

- accepted checkpoint continuity remains package-coupled;
- raw artifact injectivity is not overstated;
- replay and stale paths fail closed on persisted rehydrate;
- no test implies a standalone authoritative backend already exists unless code
  actually adds it.

📌 Command gate:

```bash
cargo test -p z00z_simulator --release --test test_checkpoint_acceptance -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_replay_inputs -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_link_injective -- --nocapture
```

### Wave T6: Default Secret Silence, Debug Lane, And RNG Scope

📌 Priority: sixth.

📌 Why here: tasks 20, 21, 22, 44, 45, 61, and 62 all depend on scoped
operator-facing behavior that should only be tested after the main runtime seams
above are frozen.

📌 Executed test homes:

- `crates/z00z_simulator/tests/test_stage2_secret_artifacts.rs`

📌 Reuse-first execution result:

- The RNG scope contract landed in
   `crates/z00z_simulator/tests/test_transport_rng_boundaries.rs`; no standalone
   `test_rng_scope_contract.rs` file was created.

📌 Required implementation tasks:

1. Keep default plaintext secret-artifact absence explicit and narrow.
2. Prove the debug lane remains the only explicit secret-export surface.
3. Add simulator-bounded seeded RNG tests with `None == zero-seed` nuance where
   applicable.
4. Keep RNG/config assertions framed as consolidation over live abstractions,
   not a brand-new entropy design.

📌 Success conditions:

- default lane remains free of plaintext secret artifacts;
- debug export stays feature-gated and non-default;
- deterministic RNG remains simulator-scoped and non-production.

📌 Command gate:

```bash
cargo test -p z00z_simulator --release --test test_stage2_secret_artifacts -- --nocapture
cargo test -p z00z_simulator --release --test test_transport_rng_boundaries -- --nocapture
```

### Wave T7: Governance, Reclassification, And Caution Rows

📌 Priority: seventh.

📌 Why here: tasks 14, 23, 24, 25, 26, 27, 46, 47, and caution rows 48-62 must
reuse already-landed executable seams instead of inventing new synthetic tests.

📌 Files to inspect and possibly update:

- `.planning/phases/033-crypto-audit-scenario-2/033-CONTEXT.md`
- `.planning/phases/033-crypto-audit-scenario-2/033-TEST-SPEC.md`
- `.planning/phases/033-crypto-audit-scenario-2/033-TODO.md`
- `.planning/REQUIREMENTS.md`

📌 Required implementation tasks:

1. Build a documentation allowlist from waves T1-T6 only.
2. Ensure tasks 25 and 27 stay blocked until tasks 63-65 or formal narrowing
   resolve the governing gaps.
3. Keep caution rows 48-62 phrased as partial, fix-set, or compatibility-lane
   statements that reuse executable evidence already proven.

📌 Success conditions:

- delivered/partial/not-proved buckets stay synchronized;
- no caution row is upgraded into a new theorem;
- reclassification remains blocked unless dependency gaps actually close.

📌 Command gate:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```

### Wave T8: Broad Runtime Sweep

📌 Priority: final.

📌 Why last: this is the broad sanity sweep after all focused phase-owned tests
and artifact guards have landed.

📌 Required implementation tasks:

1. Run the canonical simulator scenario.
2. Run the release-style simulator regression bundle.
3. Recheck that no Phase 033 wording now overstates tasks 63-65.

📌 Success conditions:

- focused test waves are green;
- Scenario 1 still exercises the accepted current-stack path;
- phase artifacts still describe tasks 63-65 as open unless code actually closes
  them.

📌 Command gate:

```bash
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump
```

## ✅ Completion Notes

📌 Phase 033 test implementation should be considered structurally complete only
when:

- waves T1-T6 land focused executable coverage on the live seams;
- wave T7 reuses those proofs for documentation and governance rows;
- wave T8 confirms the accepted runtime path stays intact;
- no artifact upgrades helper-owned claim continuity, the nullifier gap, or the
  checkpoint-backend gap into false closure.
