# 040-TODO

Canonical design source:

- [040-Spend-Proof-Spec](./040-Spend-Proof-Spec.md)

Execution rules:

- execute this file in order unless a dependency note says otherwise;
- treat the spec as normative for requirement meaning and this file as
  normative for execution order;
- do not pull requirements from the retired `7-Spend-Proof.md` draft during
  implementation;
- if execution discovers a new design constraint, update the spec first and
  then update this backlog.

## Decision Summary

The execution baseline for Phase 040 is:

1. keep the current public spend contract and checkpoint pipeline;
2. add a versioned non-empty proof carrier instead of a parallel proof object;
3. place the concrete proof producer on the wallet or Stage-4 side;
4. place the concrete proof consumer on the checkpoint-facing verifier side;
5. add regular-spend nullifier semantics at the state-transition boundary;
6. postpone output-constructor unification until proof seams are stable.

## Dependency Chain

Execution dependency chain:

1. `040-01` proof carrier contract
2. `040-02` canonical spend statement
3. `040-03` producer path
4. `040-04` verifier path
5. `040-05` nullifier semantics
6. `040-06` full regular package verification entry point
7. `040-07` simulator roundtrip and closure tests
8. `040-08` optional output-constructor follow-up

Hard dependencies:

- `040-02` depends on `040-01`
- `040-03` depends on `040-01` and `040-02`
- `040-04` depends on `040-01` and `040-02`
- `040-05` depends on `040-02` and `040-04`
- `040-06` depends on `040-03` and `040-04`
- `040-07` depends on `040-03`, `040-04`, `040-05`, and `040-06`
- `040-08` depends on `040-07`

## File-First Implementation Order

Refactor truth note:

- current code keeps the live spend statement, producer bridge, and public
  verifier logic in `spend_verification.rs`;
- current code keeps spend authorization signing in `prover.rs` and rule or
  nullifier semantics in `spend_rules.rs`;
- current code keeps the canonical full package-admission wrapper in
  `tx_verifier.rs` as `verify_full_tx_package()`;
- do not split work into new `spend_statement.rs`, `spend_prover.rs`, or
  `spend_nullifiers.rs`; keep theorem-backend work on the existing
  `spend_proof_backend.rs` seam unless a later spec update authorizes a
  different split.
- 2026-04-28 local backend hardening note: `CanonicalSpendProofBackend::prove()`
  now rejects witnesses that do not satisfy typed statement public inputs for
  owner tag, theorem `leaf_ad_id`, nullifier, and balance through the existing
  `verify_spend_rules(...)` relation. This is a real fail-closed generation
  improvement, not a final public ZK theorem proof; membership and public
  proof-of-knowledge closure remain under the active `040-10` tasks.
- 2026-04-29 direct-verifier hardening note: `CanonicalSpendProofBackend::verify()`
  now fails closed on forged public relation drift before accepting deterministic
  artifact bytes. Output theorem leaves intentionally project
  `AssetLeaf.asset_id` into the output `leaf_ad_id` namespace for theorem
  overlap checks, while storage/package asset IDs remain carried by the canonical
  statement bytes. This is subordinate to `040-10` internal theorem-relation
  closure and does not claim public proof-of-knowledge, checkpoint theorem
  finality, or full rollup settlement proof closure.

Edit order by file cluster:

1. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
2. `crates/z00z_wallets/src/core/tx/mod.rs`
3. `crates/z00z_wallets/src/core/tx/spend_verification.rs`
4. `crates/z00z_wallets/src/core/tx/prover.rs`
5. `crates/z00z_wallets/src/core/tx/spend_rules.rs`
6. `crates/z00z_wallets/src/core/tx/state_update.rs`
7. `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
8. `crates/z00z_wallets/src/core/tx/witness_gate.rs`
9. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`
10. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
11. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
12. wallet-side tests, including forged public-relation rejection anchors in
  `test_spend_proof_backend.rs`, verifier drift anchors in
  `test_tx_proof_verifier.rs`, and the tx pass/poison/drift/e2e send-scan
  matrix
13. simulator-side tests
14. rollup public-artifact binding tests in `test_settlement_theorem.rs`
15. only then optional output-builder cleanup in `builder.rs` and `output_flow.rs`

## Validation Matrix

This table proves that the implementation-driving instructions from
`040-Spend-Proof-Spec.md` have been migrated into this backlog and remain explicitly
traceable.

| 040-Spend-Proof-Spec source | Required theme | TODO coverage | Status |
| --- | --- | --- | --- |
| `2.5.3`, `2.11`, `4.2.1`, `4.3 Task 1` | versioned non-empty proof carrier | `040-01` | Validated mapped |
| `2.6.1`, `2.7.2`, `2.8.2`-`2.8.4`, `4.3 Task 2` | canonical spend statement and digest discipline | `040-02`, `040-09`, `040-10`, `040-11` | Validated mapped |
| `2.5.1`, `2.9`, `4.3 Task 2`, `4.3 Task 3` | output leaf field binding, output `leaf_ad_id` theorem projection, range relation, and forged public-relation rejection | `040-08`, `040-10`, `040-14`, `040-CG` | Validated mapped |
| `2.4`, `2.7.3`, `4.2.4`, `4.3 Task 3` | wallet or Stage-4 proof producer path | `040-03` | Validated mapped |
| `2.7.3`, `2.7.4`, `4.2.2`, `4.3 Task 3` | checkpoint-facing verifier backend via `TxProofVerifier` | `040-04`, `040-12` | Validated mapped |
| `2.11`, `4.2.3`, `4.3 Task 4` | explicit regular-spend nullifier semantics | `040-05` | Validated mapped |
| `2.7.2`, `2.8.4`, `4.2.2`, `4.3 Task 5` | one canonical full regular-tx verification entry point | `040-06`, `040-11`, `040-12` | Validated mapped |
| `2.10`, `4.4`, `4.5` | end-to-end roundtrip, failure surfaces, and honest closure tests | `040-07`, `Completion Gate` | Validated mapped |
| `2.6.3`-`2.6.5`, `4.1`, `4.3 Task 6` | bounded output-constructor follow-up only after proof seams stabilize | `040-08` | Validated mapped |
| `4.1` | legacy-draft retirement and prohibited stale designs | `040-09`, `040-12`, `040-13`, `040-14` | Validated mapped |
| `2.11` | mandatory future-proof constraints and shortcut prohibitions | `040-09`-`040-14` | Validated mapped |
| `2.12` | missing-code closure obligations | `040-04`, `040-08`, `040-13` | Validated mapped |
| `4.4` | mandatory test surfaces | tests listed under `040-01`-`040-07` | Validated mapped |
| `4.5` | completion criteria | `Completion Gate` | Validated mapped |

## Concrete Execution Tasks

### 040-01 Proof Carrier Contract

Spec references:

- Part 2: `2.5.3 Public regular tx package`, `2.11 Implementation Guidance for Future Non-Empty Spend Proof`
- Part 4: `4.2.1 Proof carrier shape`, `4.3 Task 1`, `4.4 Mandatory Test Surfaces`, `4.5 Phase 040 Completion Rule`

MANDATORY pre-read before implementation: read `040-Spend-Proof-Spec.md` lines
`209-256`, `585-618`, `675-700`, `788-814`, and `931-961` in full.

- [x] Add a non-empty, versioned regular-spend proof carrier to `TxProofWire`.
- [x] Add explicit fields for proof suite or parameters, canonical public
  statement payload, and opaque proof bytes.
- [x] Preserve current package framing and digest compatibility.

Files:

- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
- `crates/z00z_wallets/src/core/tx/mod.rs`

Tests:

- [x] create `crates/z00z_wallets/tests/test_spend_proof_wire.rs`
  - version encode or decode roundtrip
  - unknown version rejects
  - empty placeholder path rejects once non-empty proof-carrier mode is enabled

Exit condition:

- `TxProofWire` can carry a versioned proof payload without creating a parallel
  tx proof object.

### 040-02 Canonical Spend Statement

Spec references:

- Part 2: `2.6.1 Receiver secret to owner/view binding`, `2.7.2 Regular package verifier boundary`, `2.8.2 Package digest`, `2.8.3 Wire digest helper`, `2.8.4 Normative digest decision`, `2.11 Implementation Guidance for Future Non-Empty Spend Proof`
- Part 4: `4.3 Task 2`, `4.4 Mandatory Test Surfaces`

MANDATORY pre-read before implementation: read `040-Spend-Proof-Spec.md` lines
`278-309`, `399-420`, `476-554`, `585-618`, `815-839`, and `931-947` in full.

- [x] Centralize canonical regular-spend statement construction behind a shared
  helper in `spend_verification.rs` so producer and verifier consume the same
  builder.
- [x] Bind chain or root scope, package digest, input refs, output public
  fields, and any public commitment aggregates required by the proof backend.
- [x] Lock the rule that wire digest can only be an internal helper nested under
  the package digest.
- [x] Carry forward current range-proof semantics explicitly.
- [x] Reject ad hoc JSON fragments and unversioned proof transcripts.

Files:

- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
- `crates/z00z_wallets/src/core/tx/mod.rs`

Tests:

- [x] create `crates/z00z_wallets/tests/test_spend_statement.rs`
  - same package gives same statement
  - envelope digest drift changes statement
  - output field drift changes statement
  - statement does not accept bare wire digest as the only public root

Exit condition:

- one canonical statement builder exists and both producer and verifier are able
  to consume it.

### 040-03 Producer Path

Spec references:

- Part 2: `2.4 Current End-to-End Architecture`, `2.7.3 Spend witness gate boundary`, `2.11 Implementation Guidance for Future Non-Empty Spend Proof`
- Part 4: `4.2.4 Proof producer placement`, `4.3 Task 3`, `4.4 Mandatory Test Surfaces`, `4.5 Phase 040 Completion Rule`

MANDATORY pre-read before implementation: read `040-Spend-Proof-Spec.md` lines
`114-156`, `421-440`, `585-618`, `755-785`, `840-870`, and `931-961` in full.

- [x] Add a wallet or Stage-4 producer that emits the new proof carrier.
- [x] Feed it the current spend witness material rather than reconstructing a
  new conceptual witness system.
- [x] Keep the producer aligned with the live Stage-4 regular transaction flow.

Files:

- `crates/z00z_wallets/src/core/tx/prover.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`

Tests:

- [x] create `crates/z00z_wallets/tests/test_spend_prover_contract.rs`
  - producer emits carrier with canonical statement
  - producer fails closed on mismatched witness or package data

Exit condition:

- Stage 4 or an equivalent wallet path can produce a concrete regular-spend
  proof carrier.

### 040-04 Verifier Path

Spec references:

- Part 2: `2.7.3 Spend witness gate boundary`, `2.7.4 Pre-state and checkpoint boundary`, `2.11 Implementation Guidance for Future Non-Empty Spend Proof`, `2.12 Open Questions / Missing Code Support`
- Part 4: `4.2.2 Checkpoint verification boundary`, `4.3 Task 3`, `4.4 Mandatory Test Surfaces`, `4.5 Phase 040 Completion Rule`

MANDATORY pre-read before implementation: read `040-Spend-Proof-Spec.md` lines
`421-468`, `585-631`, `701-728`, `840-870`, and `931-961` in full.

- [x] Add a concrete verifier backend that consumes the new proof carrier.
- [x] Reject suite mismatch, statement mismatch, malformed proof bytes, and
  recomputation drift.
- [x] Wire this backend through `TxProofVerifier` into checkpoint apply.
- [x] Keep the current package-coupled simulator verifier only as a temporary
  adapter until the real backend is active.

Files:

- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/state_update.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`

Tests:

- [x] create or extend `crates/z00z_wallets/tests/test_tx_proof_verifier.rs`
  - suite mismatch rejects
  - statement mismatch rejects
  - malformed bytes reject
  - wrong resolved input rejects
  - wrong root rejects
  - wrong output statement rejects

Exit condition:

- checkpoint apply verifies regular-tx proof semantics through the shared proof
  contract, not through placeholder bytes.

### 040-05 Nullifier Semantics

Spec references:

- Part 2: `2.11 Implementation Guidance for Future Non-Empty Spend Proof`
- Part 4: `4.2.3 Nullifier semantics placement`, `4.3 Task 4`, `4.4 Mandatory Test Surfaces`, `4.5 Phase 040 Completion Rule`

MANDATORY pre-read before implementation: read `040-Spend-Proof-Spec.md` lines
`585-618`, `729-754`, `871-891`, and `931-961` in full.

- [x] Define deterministic regular-spend nullifier derivation.
- [x] Scope nullifiers to the correct chain or root context.
- [x] Carry nullifier commitments in the proof or statement surface.
- [x] Enforce replay or uniqueness at checkpoint or storage level.

Files:

- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/spend_rules.rs`
- `crates/z00z_wallets/src/core/tx/state_update.rs`

Tests:

- [x] create `crates/z00z_wallets/tests/test_spend_nullifier_semantics.rs`
  - same input gives same nullifier in same scope
  - scope drift changes nullifier
  - replay in checkpoint path rejects
  - claim nullifier semantics remain separate

Exit condition:

- regular-spend replay safety is explicit and state-enforced.

### 040-06 Full Regular Package Verification Entry Point

Spec references:

- Part 2: `2.7.2 Regular package verifier boundary`, `2.8.4 Normative digest decision`, `2.11 Implementation Guidance for Future Non-Empty Spend Proof`
- Part 4: `4.2.2 Checkpoint verification boundary`, `4.3 Task 5`, `4.4 Mandatory Test Surfaces`

MANDATORY pre-read before implementation: read `040-Spend-Proof-Spec.md` lines
`399-420`, `516-554`, `585-618`, `701-728`, `892-911`, and `931-947` in full.

- [x] Add one canonical verification entry point that composes local package
  checks and the public spend contract.
- [x] Keep `TxVerifierImpl` accurately scoped if it remains a local-only piece.
- [x] Switch simulator or admission paths to the composed entry point.

Files:

- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
- `crates/z00z_wallets/src/core/tx/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`

Tests:

- [x] extend `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`
  - structurally valid package without spend proof fails the canonical full verifier
- [x] extend `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
  - package passes local wire checks but fails public spend contract
  - range-proof mode stays explicit during backend integration

Residual follow-up note:

- the simulator-side regression for `package passes local wire checks but fails
  public spend contract` is now landed in `test_scenario1_spend_gate.rs`; keep
  that scenario explicit as downstream plans tighten roundtrip wording.

Exit condition:

- no caller can accidentally treat local wire verification as full spend
  validation.

### 040-07 End-to-End Roundtrip And Surface Locks

Spec references:

- Part 2: `2.4 Current End-to-End Architecture`, `2.10 Failure Cases`
- Part 4: `4.4 Mandatory Test Surfaces`, `4.5 Phase 040 Completion Rule`

MANDATORY pre-read before implementation: read `040-Spend-Proof-Spec.md` lines
`114-156`, `572-584`, and `931-961` in full.

- [x] Prove the produced carrier survives Stage 4 through Stage 6 without
  statement drift.
- [x] Keep wording and behavior tests honest about current proof scope.

Files:

- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`

Tests:

- [x] create `crates/z00z_simulator/tests/test_scenario1_tx_proof_roundtrip.rs`
- [x] extend `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
  - current public spend scope wording stays honest
  - post-`040-05` replay-closure wording stays synchronized with the actual
    landed boundary

Exit condition:

- simulator roundtrip demonstrates producer to verifier compatibility and keeps
  honest stage-surface wording.

### 040-08 Optional Output-Constructor Follow-Up

Spec references:

- Part 2: `2.6.3 Wallet-owned leaf_ad and tag16`, `2.6.4 Output construction binding`, `2.6.5 Receiver-side detection binding`, `2.12 Open Questions / Missing Code Support`
- Part 4: `4.1 Legacy Requirement Migration Verdict`, `4.3 Task 6`

MANDATORY pre-read before implementation: read `040-Spend-Proof-Spec.md` lines
`321-382`, `619-631`, `657-672`, and `912-930` in full.

- [x] Revisit `build_tx_stealth_output()` versus `build_output_leaf()` only
  after proof seams are stable.
- [x] Preserve `leaf_ad`, `tag16`, commitment, and range-proof semantics.

Files:

- `crates/z00z_wallets/src/core/tx/builder.rs`
- `crates/z00z_wallets/src/core/tx/output_flow.rs`

Tests:

- [x] extend whichever builder or output tests already lock self-decrypt and
  output-field invariants

Exit condition:

- output-builder cleanup does not change any proof-facing or receiver-facing
  behavior.

## Open-Quest Derived Tasks

This section makes the remaining `open-quest.md` requirements executable and
traceable inside the backlog.

MANDATORY before closing any of `040-09` through `040-14`: read
`040-Spend-Proof-Spec.md` lines `585-631` and `657-672` in full, then re-check the
owning primary tasks named inside each subsection.

### 040-09 Proof-Theorem Preservation

Spec references:

- Part 2: `2.6.1 Receiver secret to owner/view binding`, `2.7.3 Spend witness gate boundary`, `2.11 Implementation Guidance for Future Non-Empty Spend Proof`
- Part 4: `4.1 Legacy Requirement Migration Verdict`, `4.3 Task 2`

- [x] During `040-02`, freeze the canonical proof statement so the shipped
  package digest, chain/root scope, input refs, output public fields, and
  range-proof semantics continue to preserve the currently implemented
  spend-rule boundary carried into later `040-03`/`040-04` checks.
- [x] During `040-03` and `040-04`, reject any producer or verifier path that
  widens, drops, or reorders those equations without an explicit spec update.
- [x] During `040-10-01`, freeze the final canonical theorem target on one
  suite `regular_spend_theorem_bpplus`, one theorem contract `T(S, W)`, and
  one canonical proof carrier with no semantic aliases.
- [x] During `040-10-01`, move authoritative membership against `prev_root`
  into the frozen theorem witness table and keep checkpoint/state-transition
  plus rollup settlement closure inside the active Phase 040 boundary.

Primary owners:

- `040-02`
- `040-03`
- `040-04`

Current boundary from the active `040-10` authority reset:

- the producer and verifier share the canonical public statement, and the
  internal backend proof-generation path now requires one theorem carrier,
  explicit membership witnesses against `prev_root`, and the local relation
  checks for statement shape, nullifier, balance, and range proof.
- the frozen witness table remains deliberately narrow: `receiver_secret +
  ordered s_in[i]` plus the explicit membership sub-witness for `prev_root`.
  No sender-secret inflation, witness replay adapter, or state-only acceptance
  shortcut is allowed while the stronger public/trustless verifier,
  checkpoint theorem finality, and rollup settlement gaps remain open.

### 040-10 Public Input Surface Preservation

Spec references:

- Part 2: `2.5.1 Public leaf contract`, `2.7.1 Output self-check boundary`, `2.11 Implementation Guidance for Future Non-Empty Spend Proof`
- Part 4: `4.3 Task 2`, `4.3 Task 3`

- [x] During `040-02`, bind the proof statement to the same resolved input ids
  and public leaf fields carried by `SpendInputLeaf`.
- [x] During `040-02`, bind the proof statement to the same output `AssetLeaf`
  fields already checked by `verify_self_decrypt()`.
- [x] During `040-02`, keep chain or root scope in the public statement surface
  rather than leaving it implicit in backend-local context.
- [x] During `040-04`, add fail-closed verification for any drift between carried
  public inputs and recomputed tx facts.

Primary owners:

- `040-02`
- `040-04`

### 040-11 Digest-Root Discipline

Spec references:

- Part 2: `2.8.2 Package digest`, `2.8.3 Wire digest helper`, `2.8.4 Normative digest decision`, `2.11 Implementation Guidance for Future Non-Empty Spend Proof`
- Part 4: `4.3 Task 2`, `4.3 Task 5`

- [x] During `040-02`, keep `build_tx_package_digest()` as the only public or
  persisted proof-binding root.
- [x] During `040-02`, allow `compute_tx_digest_from_wire()` only as an internal
  helper nested under the package digest.
- [x] During `040-06`, ensure the full regular package verification entry point
  rejects any proof path that attempts to bind only the wire digest.

Primary owners:

- `040-02`
- `040-06`

### 040-12 Checkpoint-Pipeline Reuse

Spec references:

- Part 2: `2.7.4 Pre-state and checkpoint boundary`, `2.11 Implementation Guidance for Future Non-Empty Spend Proof`, `2.12 Open Questions / Missing Code Support`
- Part 4: `4.1 Legacy Requirement Migration Verdict`, `4.2.2 Checkpoint verification boundary`

- [x] During `040-04`, reuse the existing `prev_root` typed checkpoint pipeline
  and `TxProofVerifier` seam instead of introducing a parallel membership model.
- [x] During `040-06`, keep package admission and checkpoint apply as the two
  explicit verification seams, without inventing a separate regular-tx
  `CheckpointProof` object.

Primary owners:

- `040-04`
- `040-06`

### 040-13 Missing-Code Closure Tasks

Spec references:

- Part 2: `2.12 Open Questions / Missing Code Support`
- Part 4: `4.1 Legacy Requirement Migration Verdict`, `4.3 Task 1`, `4.3 Task 3`, `4.3 Task 4`, `4.3 Task 5`, `4.3 Task 6`

- [x] Close the "non-empty `TxProofWire`" gap through `040-01`.
- [x] Close the "concrete regular tx prover" gap through `040-03`.
- [x] Close the "concrete regular tx verifier" gap through `040-04`.
- [x] Resolve the "concrete regular `CheckpointProof` object" item by keeping it
  explicitly out of scope and closing the proof-consumption gap through the
  existing checkpoint hooks in `040-04` and `040-06`.
- [x] Resolve the "unified output constructor" item only as the bounded
  follow-up in `040-08`.

Primary owners:

- `040-01`
- `040-03`
- `040-04`
- `040-06`
- `040-08`

### 040-14 Prohibited Shortcut Checklist

Spec references:

- Part 2: `2.11 Implementation Guidance for Future Non-Empty Spend Proof`
- Part 4: `4.1 Legacy Requirement Migration Verdict`

- [x] Do not claim STARK proof support until `TxProofWire` is non-empty and wired through `TxProofVerifier`.
- [x] Do not introduce `receiver_cards` into the regular persisted package.
- [x] Do not replace fee-as-output semantics with a separate `C_fee` contract unless verifier logic, tests, and spec are migrated together.
- [x] Do not mix wallet `compute_leaf_ad()` with crypto `derive_leaf_ad()` in the same runtime path without a documented migration plan.
- [x] Re-check the STARK shortcut against the stronger live boundary from `040-CONTEXT.md`: non-empty carrier wiring is necessary, but standalone-proof closure remains prohibited until the remaining checkpoint replay hooks are closed.

Validation hook:

- [x] Re-check this shortcut checklist before marking `040-07` complete.

## Minimal Validation Order

Run validation in this order during execution:

1. focused wallet unit tests for wire and statement changes
2. producer and verifier contract tests
3. nullifier tests
4. simulator spend-gate tests
5. simulator roundtrip test
6. selected `cargo test -p z00z_wallets --release --features test-fast ...`
7. selected `cargo test -p z00z_simulator --release --features test-fast ...`

## Completion Gate

Phase 040 is execution-ready for closeout only when:

- [x] `040-01` through `040-07` are complete
- [x] all mandatory tests listed above exist and are green
- [x] `040-Spend-Proof-Spec.md` and `040-TODO.md` still agree on design and order
- [x] no remaining implementation step depends on the retired superseded draft
