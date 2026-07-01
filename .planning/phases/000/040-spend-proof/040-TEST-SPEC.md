---
phase: 040-spend-proof
artifact: test-spec
status: plan-active
source: context-todo-plans-summaries-validation-rollup-guard-and-live-test-anchors
updated: 2026-04-29
---

# Phase 040 Test Spec

## 🎯 Purpose

📌 This document defines the phase-local unit, integration, and end-to-end
coverage required for Phase 040 Spend Proof.

📌 It is directly usable by another engineer or agent without guessing
scenario boundaries, cryptographic invariants, rejection paths, pass oracles,
or the correct test-file destination.

📌 Phase 040 end-to-end coverage is Rust integration coverage, not browser
automation. The canonical end-to-end lane is the Scenario 1 flow from Stage 4
tx preparation through Stage 6 bundle handling and Stage 11 checkpoint apply.

## ⚠️ Workflow Status

- Mode: `plan-active`, with `040-10-PLAN.md` freezing the internal
  theorem-relation target for the current phase authority chain.
- Source artifacts used:
  - `040-CONTEXT.md`
  - `040-TODO.md`
  - `040-01-PLAN.md` through `040-08-PLAN.md`
  - `040-01-SUMMARY.md` through `040-09-SUMMARY.md`
  - `040-VALIDATION.md`
  - `040-UAT.md`
  - live spend-proof seams in `crates/z00z_wallets` and `crates/z00z_simulator`
  - existing wallet and simulator test anchors already present in the repo
  - landed Phase 040 test files already present in the repo
- `040-VERIFICATION.md` is still absent, but strict fallback conditions no
  longer apply because the phase already has summary-backed execution plus
  phase-local validation and UAT artifacts.
- The current authority target is final-state driven for the internal wallet and
  simulator relation: test ownership must prove one canonical theorem suite,
  one theorem contract, one checkpoint-apply boundary, and one rollup
  public-artifact binding guard while keeping public proof-of-knowledge,
  checkpoint theorem finality, and full rollup settlement proof closure
  explicitly open.
- Browser E2E classification is intentionally empty. Scenario proof for this
  phase must stay in Rust integration tests because the workflow is package,
  proof, and checkpoint driven.

## 📌 Classification

### ✅ TDD And Integration Targets

- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
  because Phase 040 starts by making `TxProofWire` carry a versioned non-empty
  regular-spend proof surface.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
  because it owns `build_public_spend_contract(...)`, statement binding,
  public-input checks, authorization checks, and fail-closed verifier errors.
- `crates/z00z_wallets/src/core/tx/witness_gate.rs`
  because it owns `prepare_spend_public_inputs(...)` and
  `verify_spend_witness_gate(...)`, which bridge selected inputs, tx wires, and
  canonical public-input preparation.
- `crates/z00z_wallets/src/core/tx/prover.rs`
  because the regular spend producer contract must emit proof and auth material
  that the verifier later consumes without shadow seams.
- `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`
  because Phase 040 now has one canonical backend seam for live suite
  selection, theorem-artifact acceptance, and fail-closed legacy rejection.
- `crates/z00z_wallets/src/core/tx/spend_rules.rs`
  because Phase 040 must preserve the currently implemented spend-rule
  equations, nullifier semantics, and scope-sensitive replay safety.
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
  because Phase 040 requires one canonical full verification entry point that
  composes local package checks and the public spend contract.
- `crates/z00z_wallets/src/core/tx/builder.rs`
  because the optional `040-08` follow-up may touch `build_output_leaf(...)`
  and must keep output-facing proof data stable.
- `crates/z00z_wallets/src/core/tx/output_flow.rs`
  because the same follow-up must preserve `leaf_ad`, `tag16`, commitment,
  range-proof, and `verify_self_decrypt(...)` semantics.
- `crates/z00z_rollup_node/src/lib.rs`
  because `verify_settlement_theorem(...)` owns the Phase 040 rollup
  public-artifact binding guard without upgrading it into full settlement proof
  closure.
- `crates/z00z_rollup_node/tests/test_settlement_theorem.rs`
  because it anchors the checkpoint artifact, link, execution input, spend root,
  tx inclusion, and public spend-theorem package binding for the rollup guard.

### 🔗 E2E Targets

- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
  because Stage 4 is the canonical producer seam for public spend input
  preparation, proof generation, structural checks, and witness-gate checks.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
  because Stage 6 must consume the same proof carrier through the canonical
  full verifier rather than only local wire checks.
- `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs`
  because Stage 11 is where authoritative checkpoint mutation must remain
  blocked when the spend package or proof path drifts.
- `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
  because it already anchors the current public spend contract against the
  Scenario 1 tx package.
- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
  because it already proves authoritative checkpoint emission is blocked when
  tx package continuity drifts.
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
  because Phase 040 requires wording and behavior guards that stay honest about
  current proof scope and unresolved closure items.

### ⛔ Skip Targets

- Browser automation.
- Speculative shadow modules or second proof lanes such as:
  - `crates/z00z_wallets/src/core/tx/spend_statement.rs`
  - `crates/z00z_wallets/src/core/tx/spend_prover.rs`
  - `crates/z00z_wallets/src/core/tx/spend_nullifiers.rs`
  because `040-CONTEXT.md` explicitly forbids a parallel layer outside the
  live owner seams.
- `040-INTEGRITY-GATES.md` and `040-CLOSEOUT-GATES.md` as standalone test
  binaries. They are planning evidence ledgers and should be validated through
  source-shape checks, scenario outcomes, and synchronized test/docs coverage.

## ⭐ Existing Test Anchors To Reuse

- `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`
- `crates/z00z_wallets/tests/test_tx_digest_framing.rs`
- `crates/z00z_wallets/tests/test_tx_wrong_root.rs`
- `crates/z00z_wallets/tests/test_tx_tamper.rs`
- `crates/z00z_wallets/tests/test_view_key_contract.rs`
- `crates/z00z_wallets/tests/test_s5_misuse_gate.rs`
- `crates/z00z_wallets/tests/test_e2e_tag_auth.rs`
- `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/test_tx_handoff_integration.rs`
- `crates/z00z_rollup_node/tests/test_settlement_theorem.rs`

📌 Reuse or extend those anchors before proposing new test files. New files are
only justified when the scenario cannot truthfully fit the current owning seam.

## ✅ Landed Phase 040 Anchors Already Present

- `crates/z00z_wallets/tests/support/test_phase040_spend_proof_support.rs`
- `crates/z00z_wallets/tests/test_spend_proof_wire.rs`
- `crates/z00z_wallets/tests/test_spend_statement.rs`
- `crates/z00z_wallets/tests/test_spend_prover_contract.rs`
- `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
- `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
- `crates/z00z_wallets/tests/test_spend_nullifier_semantics.rs`
- `crates/z00z_simulator/tests/test_scenario1_tx_proof_roundtrip.rs`
- `crates/z00z_rollup_node/tests/test_settlement_theorem.rs`

📌 These files mean the primary Phase 040 carrier, statement, producer,
backend, verifier, nullifier, and roundtrip seams are no longer greenfield
planning targets. Remaining work is synchronization, focused reruns, and any
residual audit-reopened gap closure still called out below.

## 🧪 Test File Placement

| Scenario Family | Test File Path | Extend Or Create | Why This Is The Correct Home |
| --- | --- | --- | --- |
| W1 proof carrier contract | `crates/z00z_wallets/tests/test_spend_proof_wire.rs` | Extend | Landed phase-local integration seam for non-empty `TxProofWire` and `TxAuthWire` carrier encoding. |
| S1 statement and public input binding | `crates/z00z_wallets/tests/test_spend_statement.rs` | Extend | Landed phase-local seam for exact canonical statement composition and public-input drift detection. |
| P1 producer contract | `crates/z00z_wallets/tests/test_spend_prover_contract.rs` | Extend | Producer-specific contract already has a dedicated file so proof/auth emission, statement reuse, and fail-closed producer behavior stay isolated. |
| B0 backend theorem boundary | `crates/z00z_wallets/tests/test_spend_proof_backend.rs` | Extend | Backend-specific seam owns canonical suite selection, theorem-artifact acceptance, and legacy rejection without inventing a second proof lane. |
| V1 public spend verifier contract | `crates/z00z_wallets/tests/test_tx_proof_verifier.rs` | Extend | Verifier-specific contract already has focused coverage separate from the broader tx verifier suite. |
| N1 nullifier semantics | `crates/z00z_wallets/tests/test_spend_nullifier_semantics.rs` | Extend | Phase 040 already has a dedicated regular-spend nullifier seam with replay separation from claim logic. |
| F1 full regular package verifier | `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs` | Extend | Existing canonical verifier suite already owns `verify_full_tx_package(...)`. |
| E1 scenario spend gate | `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` | Extend | Existing Scenario 1 gate already proves current public spend contract against produced tx packages. |
| E2 producer-to-consumer roundtrip | `crates/z00z_simulator/tests/test_scenario1_tx_proof_roundtrip.rs` | Extend | Landed phase-local simulator seam already owns the Stage 4 to Stage 6 roundtrip and should absorb any remaining statement-stability coverage instead of reopening a greenfield file. |
| E3 stage wording and honesty locks | `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | Extend | Existing stage-surface file already owns source-shape and phase wording contracts. |
| O1 optional output-constructor follow-up | `crates/z00z_wallets/src/core/tx/builder.rs` and `crates/z00z_wallets/src/core/tx/output_flow.rs` | Extend | The low-level builder and output-flow seams already own `build_output_leaf(...)`, `verify_self_decrypt(...)`, `leaf_ad`, and `tag16` invariants. |
| R1 rollup public-artifact binding guard | `crates/z00z_rollup_node/tests/test_settlement_theorem.rs` | Extend | The rollup test seam owns checkpoint artifact/link/exec-input binding and tx inclusion without claiming full settlement proof closure. |
| C1 shortcut and closeout guards | `crates/z00z_wallets/tests/test_phase040_spend_proof_guards.rs` | Create only if needed | Phase-local duplicate-surface and shortcut quarantine should get a dedicated file only if existing anchors cannot truthfully host the source-shape assertions. |

## ✅ Required End-To-End Behaviors

| Behavior ID | Requirement | Primary Path | Pass Signal | Fail Signal |
| --- | --- | --- | --- | --- |
| B1 | Stage 4 emits a non-empty, versioned spend proof and spend auth carrier | `prepare_spend_public_inputs(...) -> build_public_spend_contract(...) -> tx_alice_to_bob_pkg.json` | produced tx package contains non-empty `tx.proof.spend` and `tx.auth.spend` with canonical fields populated | carrier remains empty, placeholder-like, or omitted |
| B2 | The canonical statement binds the exact currently implemented spend equations and public fields | `build_public_spend_contract(...)` plus `verify_spend_rules(...)` | recomputed statement verifies against the signed authorization and exact input/output public fields | any dropped, reordered, or widened field still verifies |
| B3 | The verifier rejects structurally valid but semantically incomplete spend packages | `verify_tx_public_spend_contract(...)` | malformed proof/auth/root/leaf/nullifier/range data fails closed with specific error classes | package passes local checks and semantic contract simultaneously despite drift |
| B4 | One canonical full package verifier exists and composes local and public checks | `verify_full_tx_package(...)` | full verifier rejects packages that pass local wire checks but fail the public spend contract | any caller can treat local wire verification as full validation |
| B5 | Nullifier semantics are deterministic in-scope and reject replay cross-state or cross-scope | `derive_spend_nullifier(...)` plus state and checkpoint hooks | same `(chain_id, s_in)` gives same nullifier, scope drift changes it, replay rejects | regular spend nullifier is unstable or collapses into claim semantics |
| B6 | Stage 4 to Stage 6 roundtrip keeps statement, digest, root, and proof compatibility intact | `stage_4_utils::tx_lane_runtime_flow -> stage_6_utils::bundle_lane_impl` | Stage 6 consumes the Stage 4 package through the canonical verifier without rewriting canonical theorem fields | drift is hidden by rewrites or only local wire checks |
| B7 | Stage-surface wording stays honest about current scope and unresolved closure items | source-shape plus behavior tests | docs and error text explicitly say structural plausibility is weaker than semantic acceptance where still true | wording overclaims proof completeness or nullifier closure |
| B8 | Output-constructor cleanup preserves proof-facing and receiver-facing output semantics | `build_output_leaf(...)`, `bind_output_wire(...)`, `verify_self_decrypt(...)` | `leaf_ad`, `tag16`, commitment, and range-proof relations stay invariant | cleanup silently changes receiver or proof-facing semantics |
| B9 | Closeout keeps checkpoint-pipeline reuse and blocks prohibited shortcuts | Stage 6 admission and Stage 11 apply | no shadow checkpoint proof object or receiver-card insertion appears, and authoritative apply stays fail-closed | Phase 040 closes with parallel seams or shortcut drift |
| B10 | Rollup admission binds the public spend-theorem package to checkpoint artifacts without claiming final settlement proof closure | `verify_settlement_theorem(...)` | canonical bundle verifies package theorem, checkpoint proof payload, link, roots, exec input, and tx inclusion | output-range-only or package-only proof is accepted as full settlement closure |

## 🔗 Critical Integration Paths

1. `prepare_spend_public_inputs(...) -> build_public_spend_contract(...) -> verify_tx_public_spend_contract(...)`
2. `build_public_spend_contract(...) -> sign_spend_authorization(...) -> verify_spend_authorization(...)`
3. `verify_tx_public_spend_contract(...) -> verify_spend_rules(...) -> derive_spend_nullifier(...)`
4. `verify_full_tx_package(...) -> verify_tx_package(...) -> verify_tx_public_spend_contract(...)`
5. `stage_4_utils::tx_lane_runtime_flow -> tx_alice_to_bob_pkg.json -> stage_6_utils::bundle_lane_impl`
6. `stage_6_utils::bundle_lane_impl -> verify_full_tx_package(...) -> checkpoint draft handoff`
7. `stage_11_apply -> checkpoint acceptance -> post_tx storage mutation`
8. `build_output_leaf(...) -> bind_output_wire(...) -> verify_self_decrypt(...)`
9. `verify_settlement_theorem(...) -> verify_package_public_spend_contract(...) -> checkpoint artifact/link/exec-input inclusion`

## 📥 Input Fixtures And Preconditions

| Scenario Family | Inputs | Preconditions | Fixture Source |
| --- | --- | --- | --- |
| W1, S1, P1, V1 | canonical tx inputs, outputs, `prev_root`, receiver keys, selected inputs, proof inputs | use the same chain metadata and tx helpers already present in wallet tx test suites | `test_spend_witness_gate.rs`, `test_tx_verifier_suite.rs`, inline helpers in wallet tx modules |
| N1 | one or more deterministic `s_in` values, multiple `chain_id` values, replay candidate state | regular-spend nullifier logic is still distinct from claim nullifier logic | `spend_rules.rs` helpers and any wallet state-update fixtures |
| F1 | serialized `TxPackage` bytes with local-wire-valid but public-contract-invalid variants | canonical `verify_full_tx_package(...)` stays the only full verifier | `test_tx_verifier_suite.rs` package builders |
| E1, E2, E3 | Scenario 1 design doc, fixed Stage 4 config, tx package artifacts, checkpoint outputs | deterministic simulator config, release-style simulator test path | `test_scenario1_spend_gate.rs`, `test_checkpoint_acceptance.rs`, `test_scenario1_stage_surface.rs` |
| O1 | output-builder keys, sender wallet, output bundle fixtures, self-decrypt fixtures | output cleanup remains explicitly optional until proof seams settle | inline builder/output tests plus `test_s5_misuse_gate.rs` and `test_e2e_tag_auth.rs` |
| R1 | canonical `TxPackage`, checkpoint artifact, execution input, and checkpoint link | wallet public spend-theorem package already verifies and checkpoint execution input replays the package row | `test_settlement_theorem.rs` |

## 📤 Expected Outputs And Produced Artifacts

| Scenario Family | Expected Output | Persisted Artifact | Observable Signal |
| --- | --- | --- | --- |
| W1 | non-empty versioned proof/auth carrier roundtrips through JSON/package encoding | `tx_alice_to_bob_pkg.json` or wallet serialization fixture | decoded carrier preserves proof and auth fields exactly |
| S1 | canonical statement bytes remain deterministic for fixed tx facts | none unless test stores bytes for comparison | signed statement verification passes only for exact facts |
| P1 | producer emits proof/auth compatible with canonical verifier | tx package or tx wire fixture | `verify_tx_public_spend_contract(...)` accepts untouched producer output |
| V1 | verifier emits precise typed errors for drifted fields | none | exact `SpendPublicErr` match or canonical full-verifier rejection text |
| N1 | nullifier remains deterministic and scope-sensitive | state/update or checkpoint replay fixture | replay rejection or state-enforced duplicate refusal |
| F1 | full verifier blocks semantic drift even when local package structure passes | none | `result.valid == false` and error text or typed result names canonical boundary failure |
| E1 | simulator-produced package passes or fails exactly with wallet verifier truth | `transactions/tx_alice_to_bob_pkg.json` | Scenario 1 test and wallet verifier verdict agree |
| E2 | Stage 6 and Stage 11 accept only continuity-preserving package artifacts | `transactions/checkpoint_s7.json`, `transactions/checkpoint_s8.json`, post-tx checkpoint dirs | authoritative artifacts exist only on valid path |
| E3 | phase wording and failure text remain honest about current scope | source files and stage output strings | source-shape assertions and error wording checks stay green |
| R1 | rollup guard accepts only checkpoint-bound public artifacts | none beyond canonical test fixtures | `verify_settlement_theorem(...)` returns `Ok(())` only for package, artifact, link, root, and tx-row alignment |

## 🚫 Negative Scenario Matrix

| Scenario ID | Drift Or Failure | Owning Test File | Expected Result |
| --- | --- | --- | --- |
| N01 | missing spend proof | `test_spend_proof_wire.rs` and `test_scenario1_spend_gate.rs` | `SpendPublicErr::MissingProof` |
| N02 | missing spend auth | `test_scenario1_spend_gate.rs` | `SpendPublicErr::MissingAuth` |
| N03 | bad proof version | `test_spend_proof_wire.rs` and `test_scenario1_spend_gate.rs` | `SpendPublicErr::BadProofVersion` |
| N04 | bad auth version | `test_spend_proof_wire.rs` | `SpendPublicErr::BadAuthVersion` |
| N05 | zero or malformed `prev_root_hex` | `test_tx_proof_verifier.rs` and `test_scenario1_spend_gate.rs` | `SpendPublicErr::BadPrevRoot` or exact invalid-hex label |
| N06 | input count mismatch between tx inputs and proof inputs | `test_tx_proof_verifier.rs` | `SpendPublicErr::InputCountMismatch` |
| N07 | malformed or empty nullifier hex | `test_tx_proof_verifier.rs` and `test_scenario1_spend_gate.rs` | `SpendPublicErr::InvalidHex { label: "proof.inputs[].nullifier_hex" }` |
| N08 | signed nullifier drift after auth is created | `test_scenario1_spend_gate.rs` | `SpendPublicErr::StatementMismatch` |
| N09 | leaf-ad hash drift on inputs | `test_spend_witness_gate.rs` | `SpendPublicErr::InputLeafAdHashMismatch { idx }` |
| N10 | duplicate nullifier in one spend contract | `test_spend_witness_gate.rs` | `SpendPublicErr::DuplicateNullifier` |
| N11 | missing output `leaf_ad_id`, `r_pub`, or `owner_tag` | `test_tx_proof_verifier.rs` | `SpendPublicErr::MissingOutputField { idx, field }` |
| N12 | output `leaf_ad_id` drift on the canonical verifier seam | `test_tx_proof_verifier.rs`; canonical statement drift must reject without emitting a standalone `BadOutputLeafAd` code path | `SpendPublicErr::StatementMismatch` |
| N13 | missing or invalid output range proof | `test_tx_proof_verifier.rs` | `SpendPublicErr::MissingRangeProof { idx }` or `BadRangeProof { idx, .. }` |
| N14 | balance equation mismatch | `test_tx_proof_verifier.rs` | `SpendPublicErr::BadBalance` |
| N15 | package passes local wire checks but fails public spend contract | `test_tx_verifier_suite.rs` and `test_scenario1_spend_gate.rs` | canonical full verifier rejects and simulator harness blocks authoritative path |
| N16 | Stage 11 package digest or package payload drift | `test_checkpoint_acceptance.rs` and `test_scenario1_tx_proof_roundtrip.rs` | Stage 11 fails and authoritative checkpoint artifacts stay absent |
| N17 | checkpoint pipeline replaced by parallel proof object | `test_phase040_spend_proof_guards.rs` if needed | source-shape or integration guard fails closeout |
| N18 | output cleanup changes `leaf_ad`, `tag16`, or self-decrypt semantics | builder/output tests | output follow-up rejects as behavior drift |
| N19 | rollup checkpoint execution input no longer matches checkpoint artifact statement | `test_settlement_theorem.rs` | `SettlementError::CheckpointReplay` |
| N20 | rollup checkpoint execution input omits the tx package row | `test_settlement_theorem.rs` | `SettlementError::TxMissing` |
| N21 | rollup checkpoint root no longer matches the spend theorem `prev_root` | `test_settlement_theorem.rs` | `SettlementError::CheckpointRoot` |
| N22 | rollup checkpoint link no longer binds the artifact and execution input | `test_settlement_theorem.rs` | `SettlementError::CheckpointLink` |
| N23 | rollup tx package public spend-theorem evidence is malformed | `test_settlement_theorem.rs` | `SettlementError::TxTheorem(_)` |

## 🔐 Cryptographic And Security Invariants To Observe

| Invariant | Why It Matters | Assertion Shape |
| --- | --- | --- |
| Proof statement preserves the exact current spend-rule theorem | prevents silent widening or weakening of the implemented spend proof contract | statement tests and prover/verifier tests compare exact inputs, order, and semantic fields |
| Public input surface preserves input refs, leaf fields, chain scope, and root scope | keeps verifier bound to the tx facts already checked elsewhere | mutate any bound public field and require fail-closed rejection |
| `build_tx_package_digest()` remains the only public or persisted proof-binding root | prevents wire-digest-only shortcuts from becoming authoritative | full verifier tests reject proof paths that bind only the local wire digest |
| Regular spend nullifiers remain scope-sensitive and separate from claim semantics | prevents replay collapse and semantic mixing between lanes | same scope is deterministic, scope drift changes value, replay rejects through state/checkpoint path |
| Package admission and checkpoint apply remain the only canonical verification seams | prevents a parallel membership or checkpoint proof model | simulator tests and source-shape guards reject parallel proof objects or bypasses |
| Non-empty proof carrier is required before claiming future proof backend support | prevents overclaiming ZK support while carrier is still placeholder-like | wire tests and source-shape guards block empty-carrier acceptance |
| Output cleanup cannot change receiver-facing or proof-facing output semantics | preserves downstream receiver detection and proof verification | builder/output tests assert `leaf_ad`, `tag16`, commitment, and range-proof parity before and after cleanup |

## 🧭 Scenario Catalog

### 1. Maintain `crates/z00z_wallets/tests/test_spend_proof_wire.rs`

Core assertions to keep or extend:

1. `spend_proof_wire_roundtrip_persists_non_empty_regular_spend_carrier`
   Demonstrates: a canonical regular-spend proof/auth carrier survives
   serialization without dropping versioned fields.
   Success conditions:
   - `TxProofWire::spend` and `TxAuthWire::spend` are present after roundtrip;
   - all mandatory proof/auth fields are non-empty;
   - the encoded package still uses one canonical proof carrier surface.

2. `legacy_empty_spend_carrier_is_not_treated_as_valid_phase040_output`
   Demonstrates: placeholder-like or empty spend carriers are not considered
   valid Phase 040 output.
   Success conditions:
   - canonical verifier path rejects the placeholder carrier;
   - no test treats `Default::default()` proof/auth as proof-complete success.

3. `spend_wire_version_drift_is_visible_to_verifier_contract`
   Demonstrates: proof/auth version fields matter and are consumed by the
   verifier contract.
   Success conditions:
   - bad proof version is rejected;
   - bad auth version is rejected.

### 2. Maintain `crates/z00z_wallets/tests/test_spend_statement.rs`

Core assertions to keep or extend:

1. `statement_is_deterministic_for_identical_tx_facts`
   Demonstrates: the canonical statement is stable for fixed tx inputs, outputs,
   chain metadata, and root scope.
   Success conditions:
   - repeated construction yields identical statement bytes;
   - changing no field leaves the statement unchanged.

2. `statement_binds_exact_input_refs_and_leaf_fields`
   Demonstrates: statement integrity covers the same resolved input ids and
   public leaf fields carried by `SpendInputLeaf`.
   Success conditions:
   - drift in `serial_id`, `asset_id`, `leaf_ad_id`, `owner_tag`, `r_pub`, or
     commitment changes the statement and breaks verification.

3. `statement_binds_output_leaf_surface_and_chain_root_scope`
   Demonstrates: output leaf fields, `chain_id`, and `prev_root` stay inside the
   public statement surface instead of living only in backend-local context.
   Success conditions:
   - mutating any of those values changes the statement and invalidates auth.

### 3. Maintain `crates/z00z_wallets/tests/test_spend_prover_contract.rs`

Core assertions to keep or extend:

1. `producer_emits_proof_and_auth_that_public_verifier_accepts`
   Demonstrates: the canonical producer path and canonical verifier path are
   compatible without translation shims.
   Success conditions:
   - producer output verifies through `verify_tx_public_spend_contract(...)`;
   - producer output verifies through `verify_spend_witness_gate(...)` when
     routed through the witness gate.

2. `producer_rejects_empty_inputs_or_outputs_before_emitting_contract`
   Demonstrates: producer remains fail-closed for the primary `SpendBuildErr`
   conditions.
   Success conditions:
   - empty inputs reject;
   - empty outputs reject;
   - duplicate input references reject.

3. `producer_does_not_reorder_or_widen_statement_equations`
   Demonstrates: producer stays subordinate to `verify_spend_rules(...)` and the
   exact public statement ordering.
   Success conditions:
   - local changes to equation order or bound field order cause rejection;
   - no alternate statement encoder is accepted.

### 4. Maintain `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`

Core assertions to keep or extend:

1. `public_spend_verifier_accepts_canonical_producer_output`
2. `public_spend_verifier_rejects_the_current_seam_local_negative_matrix`
3. `public_spend_verifier_does_not_overclaim_distributed_negative_case_ownership`

Required assertions:

- prefer exact `SpendPublicErr` matches where the error type already exists;
- reject any attempt to make local wire validity imply semantic proof validity;
- prove fail-closed recomputation for the negative cases this seam actually
  owns today;
- keep N09 through N14 ownership truthful when a negative case lives in
  `test_spend_witness_gate.rs`, `test_scenario1_spend_gate.rs`, or remains an
  explicit uncovered gap on the current live verifier seam.

### 5. Maintain `crates/z00z_wallets/tests/test_spend_nullifier_semantics.rs`

Core assertions to keep or extend:

1. `same_scope_same_input_yields_same_nullifier`
2. `scope_drift_changes_nullifier`
3. `checkpoint_or_state_replay_rejects_duplicate_regular_spend_nullifier`
4. `claim_nullifier_semantics_remain_separate_from_regular_spend`

Pass conditions:

- determinism holds inside one scope;
- cross-scope drift changes the nullifier;
- replay is explicitly rejected;
- claim and regular-spend semantics do not collapse into one domain.

### 6. Extend `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`

Tests or assertions to add:

1. `full_verifier_rejects_local_wire_only_success`
   Demonstrates: local wire validity is not enough for canonical acceptance.
2. `full_verifier_rejects_wire_digest_only_binding`
   Demonstrates: `build_tx_package_digest()` stays the only public binding root.
3. `full_verifier_error_surface_mentions_public_spend_boundary`
   Demonstrates: callers can see that the semantic contract, not just the local
   wire checks, caused the failure.

### 7. Extend `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`

Tests or assertions to add:

1. `scenario1_local_wire_ok_but_public_spend_fail_blocks_acceptance`
   Demonstrates: the simulator path covers the residual follow-up named in
   `040-TODO.md`, not only the wallet-side verifier unit test.
   Success conditions:
   - construct a package variant that keeps local package structure valid;
   - `verify_tx_public_spend_contract(...)` rejects;
   - authoritative simulator path treats it as failure.

2. `scenario1_range_proof_mode_stays_explicit`
   Demonstrates: backend integration does not hide whether the current mode is
   still partial, opaque-test, or otherwise bounded.
   Success conditions:
   - the scenario surface records the active proof mode explicitly;
   - no test wording overclaims backend maturity.

### 8. Maintain `crates/z00z_simulator/tests/test_scenario1_tx_proof_roundtrip.rs`

Core assertions to keep or extend:

1. `stage4_to_stage6_roundtrip_preserves_public_statement_surface`
   Demonstrates: Stage 4 producer output survives handoff to Stage 6 without
   statement drift.
   Success conditions:

   - Stage 6 consumes the Stage 4 package through the canonical full verifier;
   - no bound public field changes between stages.

1. `stage11_rejects_roundtrip_after_canonical_chain_scope_drift`
   Demonstrates: authoritative checkpoint mutation stays blocked after drift.
  Success conditions: mutate a canonical statement field such as `chain_id`
  and recompute `tx_digest_hex` instead of drifting `fee`, because fee drift
  trips an earlier local verifier gate before the public spend-contract
  boundary; Stage 11 fails; authoritative checkpoint artifacts remain absent;
  Stage 4 observational artifacts may remain, but do not become
  authoritative.

1. `roundtrip_uses_checkpoint_pipeline_reuse_not_parallel_membership_model`
   Demonstrates: Phase 040 keeps the existing `prev_root` and checkpoint hooks.
   Success conditions:

   - no alternate membership or checkpoint proof object is introduced in the
     runtime path;
   - existing checkpoint hook outputs remain the only authoritative path.

### 9. Extend `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

Tests or assertions to add:

1. `stage_surface_keeps_current_public_spend_scope_wording_honest`
2. `stage_surface_keeps_replay_closure_and_open_public_boundaries_explicit`
3. `stage_surface_does_not_claim_stark_support_before_non_empty_carrier_and_verifier_wiring`

Pass conditions:

- wording matches the current bounded implementation truth;
- wording does not regress once the code lands;
- wording explicitly blocks overclaiming future-proof support.

### 10. Extend `crates/z00z_wallets/src/core/tx/builder.rs` and `crates/z00z_wallets/src/core/tx/output_flow.rs`

Tests or assertions to add only after `040-01` through `040-07` are stable:

1. `output_cleanup_preserves_leaf_ad_tag16_commitment_and_range_proof_semantics`
2. `build_output_leaf_and_verify_self_decrypt_remain_behaviorally_equivalent`
3. `output_cleanup_does_not_change_receiver_visible_fields_or_self_decrypt_success`

Success conditions:

- `leaf_ad` stays computed from the same fields;
- `tag16` stays computed from the same `k_dh` plus `leaf_ad` relation;
- self-decrypt and receiver-facing output behavior remain unchanged.

### 11. Maintain `crates/z00z_rollup_node/tests/test_settlement_theorem.rs`

Core assertions to keep or extend:

1. `test_settlement_accepts_bundle`
  Demonstrates: the rollup guard accepts a canonical package, checkpoint
  artifact, checkpoint link, execution input, and tx inclusion tuple.

2. `test_settlement_rejects_checkpoint_replay`
  Demonstrates: checkpoint artifact statements cannot be replayed against a
  drifted execution input.

3. `test_settlement_rejects_tx_missing`
  Demonstrates: a verified package is not accepted unless the checkpoint
  execution input includes the matching tx row.

4. `test_settlement_rejects_root_mismatch`
  Demonstrates: the package spend root and checkpoint execution root must stay
  aligned.

5. `test_settlement_rejects_bad_link`
  Demonstrates: the checkpoint link must bind the artifact, snapshot, and
  execution input.

6. `test_settlement_rejects_bad_package`
  Demonstrates: malformed public spend-theorem package evidence fails before
  rollup admission.

Success conditions:

- the rollup guard verifies public-artifact binding only;
- output range proofs alone never count as settlement closure;
- public proof-of-knowledge and full rollup settlement proof closure remain
  explicit open boundaries.

## 📏 Measurable Pass Conditions

- Every landed or newly extended phase-local test file compiles and runs under
  the owning crate.
- Wallet-side tests use `cargo test -p z00z_wallets --release --features test-fast ...`.
- Simulator-side tests use
  `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump ...`.
- At least one simulator scenario test proves the residual `local wire ok,
  public spend fail` case named in `040-TODO.md`.
- At least one simulator roundtrip test proves Stage 4 to Stage 6 compatibility
  and Stage 11 fail-closed authoritative blocking.
- At least one rollup test proves checkpoint artifact, link, execution input,
  spend root, tx inclusion, and public package theorem binding without claiming
  full settlement proof closure.
- No scenario introduces or depends on a shadow proof layer, receiver-card
  package field, or parallel checkpoint proof object.

## 🧪 Command Gates

- Wallet focused runs:
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_proof_wire -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_statement -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_prover_contract -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_proof_backend -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --test test_tx_proof_verifier -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast --test test_spend_nullifier_semantics -- --nocapture`
- Simulator focused runs:
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_tx_proof_roundtrip -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`
- Scenario sanity run:
  - `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
- Rollup focused run:
  - `cargo test -p z00z_rollup_node --release --test test_settlement_theorem -- --nocapture`

## ✅ Done Condition

- `040-01` through `040-07` each have at least one owning test anchor and an
  explicit pass oracle.
- `040-08` remains clearly marked optional and is implemented only after the
  proof seams stabilize.
- `040-09` through `040-14` are represented in executable seams, invariants,
  negative cases, source-shape guards, or explicit audit-reopened closeout
  assertions rather than being silently dropped.
- The resulting coverage proves one canonical producer lane, one canonical
  public verifier lane, one canonical full package verifier boundary, one
  canonical checkpoint-apply boundary, and one rollup public-artifact binding
  guard while keeping full rollup settlement proof closure open.
