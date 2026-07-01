# 034-TODO

Phase-local reconciliation status on the live tree:

- [x] `034-01` through `034-14` are summary-backed complete through
  `034-08-SUMMARY.md`, `034-VALIDATION.md`, and `034-CLOSEOUT.md`.
- [x] `034-15` is now executed as a local behavior-preserving sidecar and stays
  outside the semantic closure story.
- [x] `034-16` through `034-18` are summary-backed complete through
  `034-09-SUMMARY.md` and remain post-closure hygiene only.

Canonical design source:

- [034-33AUDIT](./034-33AUDIT.md)
- [034-fix-spec-4](./034-fix-spec-4.md)
- [034-deferred](./034-deferred.md)
- [034-suffixes-V1-Vn](./034-suffixes-V1-Vn.md)

Execution rules:

- execute this file in order unless a dependency note says otherwise;
- treat `034-33AUDIT.md` as normative for the Phase 034 closure gaps and this
  file as normative for execution order;
- treat `034-fix-spec-4.md` as a phase-local refinement for the regular-spend
  and legacy-sender seams only where it stays consistent with the audit-backed
  gaps in `034-33AUDIT.md`;
- follow the Q64 and Q65 semantic gap-closure paths from the audit body and
  `Exact Fixes Required Summary`, not the crossed title-only reading of the two
  summary rows;
- do not pull requirements from older deferred ledgers during implementation;
- do not add a parallel claim-source verifier, spend proof layer, checkpoint
  verifier facade, or sender-construction authority when an existing truthful
  seam can be extended;
- keep the optional `keep_path(...)` cleanup out of the main Phase 034 closure
  path and out of any semantic-completeness claim;
- treat `034-suffixes-V1-Vn.md` as normative only for the optional suffix
  cleanup sidecar, especially its production-current vs reserved-future
  classification and its usage-backed file inventory;
- if implementation discovers a new design constraint, update the canonical
  source first, then this backlog, then the affected tests;
- before starting any numbered task, complete its `MANDATORY pre-read` block;
- documentation allowlist updates are blocked until the Q63, Q64, and Q65
  closure tasks are implemented and re-verified.

## 🎯 Decision Summary

The execution baseline for Phase 034 is:

1. close the Phase 033 audit gaps in strict order: Q63 helper-owned claim
   continuity, Q64 regular-spend nullifier semantics, Q65 authoritative
   checkpoint proof backend, and only then Q47 documentation allowlist review;
2. bind claim-source proofs to persisted storage-backed membership state by
   extending the live storage and claim seams instead of keeping the off-store
   one-item helper as the authority surface;
3. bind deterministic nullifier semantics into the existing regular public
   spend contract, persisted wire contract, witness bridge, and structural
   spend rules rather than inventing a second spend proof system;
4. retire legacy sender-construction authority from `core::tx` only through
   phased migration to the already-live `core::stealth` authority surface;
5. bind checkpoint finalize and reload acceptance to an authoritative proof
   backend instead of compatibility payload bytes and externally supplied
   verifier trust;
6. keep historical deferred items out of scope except for the explicitly
   optional `keep_path(...)` micro-cleanup, which is not part of the completion
   gate for this phase.
7. if the optional `keep_path(...)` cleanup is executed at all, run it only as
  a post-closure sidecar after the main Phase 034 semantic chain is already
  separated, verified, and no longer ambiguous.
8. if identifier-length cleanup is executed at all, run it only as a separate
  post-closure sidecar based on a fresh non-Tari inventory of live signature
  violations, and do not treat naming hygiene as semantic closure evidence for
  Q63, Q64, Q65, or Q47.
9. if suffix cleanup is executed at all, run it only as a separate post-closure
  sidecar that collapses production-current Rust-facing suffixes to canonical
  unsuffixed names, while retiring reserved-future surfaces only where the
  source-backed inventory proves they are not still needed by current
  production read, import, open-session, or migration paths.

## 🔗 Dependency Chain

Execution dependency chain:

1. `034-01` persisted claim-source contract seam
2. `034-02` claim producer and verifier migration
3. `034-03` regular-spend nullifier domain and wire contract
4. `034-04` regular-spend verifier and rule integration
5. `034-05` legacy sender-construction authority retirement
6. `034-06` authoritative checkpoint proof backend contract
7. `034-07` checkpoint finalize or load integration
8. `034-08` harness and seam-reuse lock-in
9. `034-09` claim continuity test wave
10. `034-10` spend nullifier test wave
11. `034-11` checkpoint backend test wave
12. `034-12` documentation allowlist and wording reclassification
13. `034-13` documentation and stage-surface test wave
14. `034-14` closure, regression, and phase-proof sweep
15. `034-15` optional keep-path complexity sidecar
16. `034-16` optional bounded 5-word signature compliance wave
17. `034-17` optional legacy collision retirement sidecar
18. `034-18` optional production-current suffix collapse sidecar

Hard dependencies:

- `034-02` depends on `034-01`
- `034-04` depends on `034-03`
- `034-05` depends on `034-03` and `034-04`
- `034-07` depends on `034-06`
- `034-08` depends on `034-01` through `034-07`
- `034-09` depends on `034-01`, `034-02`, and `034-08`
- `034-10` depends on `034-03`, `034-04`, `034-05`, and `034-08`
- `034-11` depends on `034-06`, `034-07`, and `034-08`
- `034-12` depends on `034-09`, `034-10`, and `034-11`
- `034-13` depends on `034-12`
- `034-14` depends on `034-09` through `034-13`
- `034-15` depends on `034-14` if the optional sidecar is executed
- `034-16` depends on `034-14` if the optional sidecar is executed
- `034-17` depends on `034-16` if the optional sidecar is executed
- `034-18` depends on `034-17` if the optional sidecar is executed

## 🗂️ File-First Implementation Order

Edit order by file cluster:

1. `crates/z00z_storage/src/assets/store_internal/store_query.rs`
2. `crates/z00z_storage/tests/test_claim_source_proof.rs`
3. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
4. `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`
5. `crates/z00z_simulator/src/claim_pkg_consumer.rs`
6. `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`
7. `crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs`
8. `crates/z00z_crypto/src/domains.rs`
9. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
10. `crates/z00z_wallets/src/core/tx/witness_gate.rs`
11. `crates/z00z_wallets/src/core/tx/spend_verification.rs`
12. `crates/z00z_wallets/src/core/tx/spend_rules.rs`
13. `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
14. `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
15. `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
16. `crates/z00z_wallets/src/core/stealth/output.rs`
17. new `crates/z00z_wallets/src/core/stealth/output_leaf.rs` only if the owner split is needed
18. `crates/z00z_wallets/src/core/tx/builder.rs`
19. `crates/z00z_wallets/src/core/tx/output_flow.rs`
20. `crates/z00z_wallets/src/core/tx/mod.rs`
21. `crates/z00z_wallets/src/core/tx/lifecycle.rs`
22. `crates/z00z_wallets/examples/wallet_reload.rs`
23. `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs`
24. `crates/z00z_simulator/src/scenario_1/stage_4_utils/output_construction_balance.rs`
25. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_test_support.rs`
26. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs`
27. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs`
28. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
29. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
30. `crates/z00z_simulator/tests/test_claim_acceptance.rs`
31. `crates/z00z_wallets/src/services/wallet_service_tests.rs`
32. `crates/z00z_wallets/src/core/stealth/test_output_extra.rs`
33. `crates/z00z_wallets/tests/test_s5_sender_examples.rs`
34. `crates/z00z_wallets/tests/test_s5_spec6_bridge.rs`
35. `crates/z00z_wallets/tests/test_adversarial.rs`
36. `crates/z00z_wallets/tests/test_phase14_pipeline.rs`
37. `crates/z00z_wallets/tests/test_phase15_regress.rs`
38. `crates/z00z_wallets/tests/test_tx_serial.rs`
39. `crates/z00z_wallets/tests/test_s5_leaf_gate.rs`
40. `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`
41. `crates/z00z_storage/src/checkpoint/artifact_final.rs`
42. `crates/z00z_storage/src/checkpoint/codec.rs`
43. `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`
44. `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`
45. `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
46. `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
47. `crates/z00z_storage/tests/test_redb_rehydrate.rs`
48. `.planning/REQUIREMENTS.md`
49. `crates/z00z_simulator/src/scenario_1/stage_12.rs`
50. `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
51. only then optional micro-cleanup in `crates/z00z_storage/src/assets/store_internal/store_query.rs` via `034-15`
52. if `034-15` executes, extend `crates/z00z_storage/tests/test_search_api.rs`
53. only after semantic closure, inventory non-Tari Rust signature-like identifiers against `.github/copilot-instructions.md`
54. if `034-16` executes, rename only live >5-word violations outside `crates/z00z_crypto/tari/**`
55. if `034-16` executes, update affected tests and source-text guard surfaces created by honest renames
56. after `034-16`, use `034-suffixes-V1-Vn.md` plus current-tree declaration checks to map every live legacy unsuffixed blocker that occupies a future production-current canonical target
57. if `034-17` executes, retire, rename, or narrow only the live legacy blockers needed to free canonical unsuffixed targets for the later suffix collapse, without deleting compatibility readers or migration support still marked live
58. if `034-17` executes, update all affected tests, docs, grep guards, and source-text contract surfaces after the legacy-collision retirement pass
59. only after `034-17`, collapse production-current Rust-facing suffix-bearing names to unsuffixed canonical names without changing on-wire, on-disk, or discriminator values
60. if `034-18` executes, delete or retire reserved-future suffix surfaces only when the source-backed inventory says the current production path does not still read, import, open, or migrate through them
61. if `034-18` executes, update all affected tests, docs, grep guards, and source-text contract surfaces after the suffix collapse and safe retirements

## ✅ Validation Matrix

| Source section | Required theme | TODO coverage | Status |
| --- | --- | --- | --- |
| `034-33AUDIT.md -> Scope And Source Of Truth` | audit remains canonical for closure truth; narrowed Phase 033 validation is not full implementation closure | execution rules; `034-08`; `034-09`; `Completion Gate` | Mapped as execution guardrail |
| `034-33AUDIT.md -> Critical User Journeys` | claim continuity, public spend acceptance, checkpoint finalize/reload, and wording guards all need direct closure work | `034-01` through `034-14` | Validated mapped |
| `034-33AUDIT.md -> State Transitions` | helper contract, spend statement, and checkpoint reload must move from partial truth to implemented authority | `034-01`; `034-03`; `034-04`; `034-06`; `034-07` | Validated mapped |
| `034-33AUDIT.md -> Proof Paths` | one truthful claim proof path, one truthful spend contract, one truthful checkpoint backend path | `034-01`; `034-02`; `034-04`; `034-06`; `034-07` | Validated mapped |
| `034-33AUDIT.md -> Failure Paths` | no overclaiming may survive in code comments, tests, or docs after closure | `034-12`; `034-13`; `034-14`; `Completion Gate` | Mapped as preservation constraint |
| `034-33AUDIT.md -> Exact Fixes Required Summary` | Q63, Q64, Q65, and Q47 are the hard closure ledger | `034-01` through `034-14` | Validated mapped |
| `034-fix-spec-4.md -> Objective` | nullifier closure and legacy sender-path retirement are the regular-spend execution baseline | `034-03`; `034-04`; `034-05` | Validated mapped |
| `034-fix-spec-4.md -> Anti-Drift Rules` | no stale `core::tx` sender authority and no false claim that nullifier semantics are already done | execution rules; `034-05`; `034-08`; `034-14` | Mapped as anti-drift guardrail |
| `034-fix-spec-4.md -> Workstream A` | regular-spend nullifier semantics must land in domains, wire contract, witness bridge, verifier, and rules | `034-03`; `034-04`; `034-10` | Validated mapped |
| `034-fix-spec-4.md -> Workstream B` | legacy sender helpers must be migrated to `core::stealth` without blind deletion of useful helpers | `034-05`; `034-10`; `034-14` | Validated mapped |
| `034-fix-spec-4.md -> Documentation Updates Required` | documentation can only be narrowed after implementation and tests land | `034-12`; `034-13`; `034-14` | Mapped with explicit closure gate |
| `034-deferred.md -> Intake Decision` | historical deferred ledger is not part of required Phase 034 work, and only one tiny optional sidecar may be attached | execution rules; `034-15`; `🚫 Explicit Phase Boundary` | Mapped as phase-boundary constraint |
| `034-deferred.md -> Optional Tiny Debt Specification` | only one optional sidecar exists: behavior-preserving `keep_path(...)` complexity cleanup | `034-15`; `Completion Gate` | Validated mapped |
| `.github/copilot-instructions.md -> Local Rust Rules` | non-Tari signature-like identifiers must obey the 5-word rule if hygiene cleanup is executed | `034-16`; `Completion Gate`; `🚫 Explicit Phase Boundary` | Mapped as optional post-closure hygiene constraint |
| `034-suffixes-V1-Vn.md -> Fixed Table` | production-current suffix-bearing surfaces must be separated from reserved-future compatibility or future-only surfaces before any rename or deletion is planned | execution rules; `034-17`; `Completion Gate` | Mapped as optional source-backed inventory |
| `034-suffixes-V1-Vn.md -> Bottom Line` | production-current suffix collapse may proceed, but blanket deletion of reserved-future surfaces is not truthful when the source still marks some of them as live read or migration support | `034-17`; `🚫 Explicit Phase Boundary`; `Completion Gate` | Mapped as anti-drift constraint |

## 🚫 Explicit Phase Boundary

The following topics are intentionally out of scope for required Phase 034
closure:

- any historical deferred item beyond the explicitly optional
  `STORQ-KEEP-PATH-COMPLEXITY` micro-cleanup;
- a new speculative claim proof system, a second regular-spend proof stack, or
  a general checkpoint verifier facade outside the live storage or wallet seams;
- a blind deletion of `output_flow.rs` or low-level output helpers before
  callers are migrated and ownership is reassigned truthfully;
- any theorem-level claim that the repository now proves a stronger global
  spend, claim, or checkpoint theorem than the code and tests actually verify;
- documentation relaxation before Q63, Q64, and Q65 are implemented and the
  stage-surface guards are updated to the new truth;
- counting the optional `keep_path(...)` cleanup as evidence that Phase 034 has
  closed its semantic blockers.
- counting optional identifier-length renames as evidence that Phase 034 has
  closed its semantic blockers.
- blind deletion of suffix-bearing compatibility readers or migration constants
  that `034-suffixes-V1-Vn.md` still classifies as live support for current
  production open, import, or migration paths.

## ⚙️ Concrete Execution Tasks

### 034-01 Persisted Claim-Source Contract Seam

Spec references:

- `034-33AUDIT.md -> Critical User Journeys`
- `034-33AUDIT.md -> State Transitions`
- `034-33AUDIT.md -> Proof Paths`
- `034-33AUDIT.md -> Exact Fixes Required Summary -> Q63`

MANDATORY pre-read:

- `034-33AUDIT.md -> Claim package emission and claim-source verification continuity`
- `034-33AUDIT.md -> Store item to claim-source contract tuple`
- `034-33AUDIT.md -> Helper-owned claim-source reconstruction path`
- `034-33AUDIT.md -> Q63`

- [x] Replace the helper-owned off-store one-item reconstruction inside
  `claim_source_contract_for_item(...)` with a storage-backed membership seam
  that derives the root and proof from the persisted store state.
- [x] Keep the live claim-source contract tied to canonical asset paths and
  fail closed when the item or path does not match persisted membership.
- [x] Preserve deterministic proof material and root encoding so downstream
  claim package callers do not need ad hoc compatibility glue.

Files:

- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
- `crates/z00z_storage/tests/test_claim_source_proof.rs`

Tests:

- [x] extend `crates/z00z_storage/tests/test_claim_source_proof.rs`
  - persisted membership roundtrip produces the same root and proof
  - synthetic one-item helper reconstruction is no longer the authority path
  - missing or drifted persisted membership rejects

Exit condition:

- `claim_source_contract_for_item(...)` is backed by persisted membership state
  and no longer derives the authoritative result from an off-store helper-only
  tree.

### 034-02 Claim Producer And Verifier Migration

Spec references:

- `034-33AUDIT.md -> Critical User Journeys`
- `034-33AUDIT.md -> Failure Paths`
- `034-33AUDIT.md -> Exact Fixes Required Summary -> Q63`

MANDATORY pre-read:

- `034-33AUDIT.md -> Claim package emission and claim-source verification continuity`
- `034-33AUDIT.md -> Failure Paths`
- `034-33AUDIT.md -> Doublecheck Results`

- [x] Migrate all live claim package producer and verifier call sites to the new
  storage-backed claim-source seam.
- [x] Remove duplicated local `make_claim_source_proof(...)` helpers where they
  encode the old helper-owned authority path.
- [x] Keep claim package verification fail closed on source-root or proof drift.

Files:

- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`
- `crates/z00z_simulator/src/claim_pkg_consumer.rs`
- `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`
- `crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs`

Tests:

- [x] extend `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`
  - live package path uses the persisted seam
  - helper-owned wording or assumptions no longer match the code path
- [x] extend `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`
  - verifier rejects source-root drift after migration

Exit condition:

- live claim package construction and verification consume one canonical
  storage-backed claim-source seam and no caller relies on the helper-owned
  continuity path as the authority surface.

### 034-03 Regular-Spend Nullifier Domain And Wire Contract

Spec references:

- `034-33AUDIT.md -> Exact Fixes Required Summary -> Q64`
- `034-fix-spec-4.md -> Workstream A`
- `034-fix-spec-4.md -> Recommended regular-tx nullifier contract`

MANDATORY pre-read:

- `034-fix-spec-4.md -> Verified claim 2`
- `034-fix-spec-4.md -> Workstream A: Add Regular Spend Nullifier Semantics`
- `034-fix-spec-4.md -> Files to change for Workstream A`

- [x] Add one dedicated regular-spend nullifier domain under the existing tx and
  consensus-domain family in `z00z_crypto`.
- [x] Extend `SpendInputProofWire` with `nullifier_hex` while preserving the
  existing input proof framing and serial semantics.
- [x] Extend the structural `SpendIn` rule surface with one explicit nullifier
  field so the rule layer and public spend contract can share the same
  semantics.

Files:

- `crates/z00z_crypto/src/domains.rs`
- `crates/z00z_wallets/src/core/tx/tx_wire_types.rs`
- `crates/z00z_wallets/src/core/tx/spend_rules.rs`

Tests:

- [x] extend `crates/z00z_wallets/src/core/tx/spend_rules.rs` internal tests
  - nullifier field required for positive path
  - missing or malformed nullifier field rejects
- [x] extend `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
  - wording reflects the shipped closure for tasks `034-03` and `034-04`

Exit condition:

- the regular-spend wire contract and structural rule contract both expose one
  explicit nullifier surface with no duplicate or shadow nullifier field.

### 034-04 Regular-Spend Verifier And Rule Integration

Spec references:

- `034-33AUDIT.md -> Public spend acceptance for Scenario 1 and validator-style flows`
- `034-33AUDIT.md -> Exact Fixes Required Summary -> Q64`
- `034-fix-spec-4.md -> Workstream A execution order`

MANDATORY pre-read:

- `034-33AUDIT.md -> Persisted tx package to public spend verification`
- `034-fix-spec-4.md -> Recommended regular-tx nullifier contract`
- `034-fix-spec-4.md -> Files to change for Workstream A`

- [x] Add deterministic nullifier derivation from `chain_id || s_in` using the
  new regular-spend nullifier domain.
- [x] Emit `nullifier_hex` in the spend public input bridge and public proof
  construction path.
- [x] Reject malformed nullifier hex, duplicate nullifiers, and post-signature
  nullifier drift inside one spend contract, while the witness bridge and
  structural rule layer enforce the deterministic `chain_id || s_in`
  derivation.
- [x] Mirror the same nullifier rule in `verify_spend_rules(...)` so the public
  and structural contracts stay aligned.

Files:

- `crates/z00z_wallets/src/core/tx/witness_gate.rs`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/spend_rules.rs`

Tests:

- [x] extend `crates/z00z_wallets/src/core/tx/spend_verification.rs` internal
  tests
  - malformed nullifier hex rejects in the public contract verifier
  - post-signature nullifier drift rejects in the public contract verifier
  - duplicate nullifier rejects in the public contract verifier
- [x] extend `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
  - valid deterministic nullifier verifies
  - caller-provided `chain_id` drives witness nullifier derivation
  - malformed hex rejects in the shared public-contract path
  - post-signature nullifier drift rejects
  - duplicate nullifier in one tx rejects
- [x] extend `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
  - Scenario 1 package path enforces the new nullifier semantics

Exit condition:

- the live regular public spend contract, witness bridge, and structural spend
  rules all bind one deterministic regular-spend nullifier semantics surface.

### 034-05 Legacy Sender-Construction Authority Retirement

Spec references:

- `034-fix-spec-4.md -> Verified claim 4`
- `034-fix-spec-4.md -> Verified claim 5`
- `034-fix-spec-4.md -> Workstream B`

MANDATORY pre-read:

- `034-fix-spec-4.md -> Workstream B: Retire Legacy Sender Construction Authority`
- `034-fix-spec-4.md -> Why blind deletion is wrong`
- `034-fix-spec-4.md -> Phase B1` through `Phase B6`

- [x] Re-home low-level leaf-building helpers under `core::stealth` while
  preserving only genuinely low-level helper logic.
- [x] Introduce one canonical full-leaf adapter under `core::stealth` for the
  live sender construction path.
- [x] Migrate simulator, example, lifecycle, and wallet test callers away from
  `sender_create_output_for(...)`, `create_output_bundle(...)`, and
  `create_output_bundle_with_rng(...)` as public construction authority.
- [x] Before deleting or hard-blocking any legacy wrapper, run one workspace
  grep over `sender_create_output_for`, `create_output_bundle`,
  `create_output_bundle_with_rng`, `build_output_leaf`,
  `build_output_with_blind`, and `build_output_with_rng`, then migrate every
  live caller that still treats `core::tx` as the construction owner for those
  seams.
- [x] Narrow `output_flow.rs` to tx-level bridge utilities, remove public
  construction authority from `core::tx` exports, and retire the
  `derive_fee_commitment(...)` alias if migrated callers no longer need it.
- [x] After migrated callers are green, delete the obsolete wrappers or keep
  only a temporary hard-blocked shim with an explicit migration error; do not
  leave the old construction entrypoints as quietly usable public fallbacks.

Files:

- `crates/z00z_wallets/src/core/stealth/output.rs`
- new `crates/z00z_wallets/src/core/stealth/output_leaf.rs` only if the owner split is needed
- `crates/z00z_wallets/src/core/tx/builder.rs`
- `crates/z00z_wallets/src/core/tx/output_flow.rs`
- `crates/z00z_wallets/src/core/tx/mod.rs`
- `crates/z00z_wallets/src/core/tx/witness_gate.rs`
- `crates/z00z_wallets/src/core/tx/lifecycle.rs`
- `crates/z00z_wallets/examples/wallet_reload.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/output_construction_balance.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_test_support.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs`
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
- `crates/z00z_simulator/tests/test_claim_acceptance.rs`
- `crates/z00z_wallets/src/services/wallet_service_tests.rs`
- `crates/z00z_wallets/src/core/stealth/test_output_extra.rs`
- `crates/z00z_wallets/tests/test_s5_spec6_bridge.rs`
- `crates/z00z_wallets/tests/test_adversarial.rs`
- `crates/z00z_wallets/tests/test_tx_serial.rs`
- `crates/z00z_wallets/tests/test_s5_leaf_gate.rs`

Tests:

- [x] extend `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
  - migrated witness-gate helpers no longer depend on retired legacy sender
    construction entrypoints
- [x] extend `crates/z00z_wallets/src/services/wallet_service_tests.rs`
  - service-level output construction no longer imports legacy tx-authority
    constructors
- [x] extend `crates/z00z_wallets/src/core/stealth/test_output_extra.rs`
  - stealth-owned low-level leaf helpers remain reachable only through the new
    canonical owner
- [x] extend `crates/z00z_wallets/tests/test_s5_sender_examples.rs`
  - canonical sender path comes from `core::stealth`
- [x] extend `crates/z00z_wallets/tests/test_s5_spec6_bridge.rs`
  - bridge support uses the migrated canonical sender construction owner
- [x] extend `crates/z00z_wallets/tests/test_adversarial.rs`
  - adversarial receiver-card checks still fail closed after sender-path
    migration
- [x] extend `crates/z00z_wallets/tests/test_phase14_pipeline.rs`
  - no legacy public sender constructor import remains
- [x] extend `crates/z00z_wallets/tests/test_phase15_regress.rs`
  - migrated callers preserve behavior across serial and range constraints
- [x] extend `crates/z00z_wallets/tests/test_tx_serial.rs`
  - moved low-level builders keep serial-sensitive behavior without importing
    them from the old tx owner
- [x] extend `crates/z00z_wallets/tests/test_s5_leaf_gate.rs`
  - full-leaf gate checks still pass after low-level builder ownership moves
    under `core::stealth`
- [x] extend `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_test_support.rs`
  - Stage-4 runtime support no longer routes construction authority through
    legacy tx wrappers
- [x] extend `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs`
  - Stage-4 runtime tests exercise the migrated construction owner
- [x] extend `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
  - seam-map comments and local routing notes no longer describe legacy tx
    construction wrappers as the canonical owner
- [x] extend `crates/z00z_simulator/tests/test_claim_acceptance.rs`
  - source-text guards accept the migrated canonical owner vocabulary and ban
    the retired legacy construction signatures accordingly

Exit condition:

- `core::stealth` is the only public sender-construction authority,
  `core::tx` no longer teaches legacy construction APIs as canonical entry
  points, and a workspace grep shows no live caller still depending on those
  legacy construction wrappers as the owner surface.

### 034-06 Authoritative Checkpoint Proof Backend Contract

Spec references:

- `034-33AUDIT.md -> Checkpoint finalize and reload acceptance`
- `034-33AUDIT.md -> Exact Fixes Required Summary -> Q65`

MANDATORY pre-read:

- `034-33AUDIT.md -> Finalized checkpoint metadata to accepted reload path`
- `034-33AUDIT.md -> Package-coupled checkpoint acceptance path`
- `034-33AUDIT.md -> Q65`

- [x] Define one authoritative checkpoint proof backend contract inside the
  live checkpoint seam instead of relying on compatibility payload bytes as the
  final authority.
- [x] Preserve explicit proof-system typing while removing the requirement for
  externally supplied verifier trust as the closure story.
- [x] Keep legacy compatibility parsing only where strictly needed for backward
  format handling, not as the authoritative validation theorem.

Files:

- `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`
- `crates/z00z_storage/src/checkpoint/artifact_final.rs`
- `crates/z00z_storage/src/checkpoint/codec.rs`
- `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`

Tests:

- [x] extend `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
  - authoritative backend proof path verifies
  - legacy opaque compatibility bytes are not the authority path
- [x] extend `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
  - seal and load reject backend mismatches

Exit condition:

- finalized checkpoint proof acceptance is defined by one authoritative backend
  contract rather than by compatibility payload bytes and external trust.

### 034-07 Checkpoint Finalize Or Load Integration

Spec references:

- `034-33AUDIT.md -> State Transitions`
- `034-33AUDIT.md -> Failure Paths`
- `034-33AUDIT.md -> Exact Fixes Required Summary -> Q65`

MANDATORY pre-read:

- `034-33AUDIT.md -> Finalized checkpoint metadata to accepted reload path`
- `034-33AUDIT.md -> Checkpoint acceptance must not be described as standalone backend authority`
- `034-33AUDIT.md -> Doublecheck Results`

- [x] Bind wallet-side checkpoint finalize or reload acceptance to the new
  backend contract.
- [x] Bind storage reload validation to the same backend contract and reject
  drift between draft, artifact, reload, and simulator promotion surfaces.
- [x] Update Scenario 1 checkpoint promotion helpers so stage paths exercise the
  authoritative backend instead of the compatibility-only wording path.

Files:

- `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`
- `crates/z00z_wallets/src/core/tx/state_update.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`
- `crates/z00z_simulator/src/scenario_1/stage_12.rs`

Tests:

- [x] extend `crates/z00z_storage/tests/test_redb_rehydrate.rs`
  - reload path rejects proof-backend drift
- [x] extend `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
  - semantic failure blocks authoritative checkpoint summary emission

Exit condition:

- checkpoint finalize, seal, reload, and Scenario 1 promotion all consume the
  same authoritative backend closure path.

### 034-08 Harness And Seam-Reuse Lock-In

Spec references:

- `034-33AUDIT.md -> Scope And Source Of Truth`
- `034-fix-spec-4.md -> Anti-Drift Rules`

MANDATORY pre-read:

- `034-33AUDIT.md -> Audit Setup`
- `034-fix-spec-4.md -> Anti-Drift Rules`

- [x] assign one canonical test home for each major seam: claim continuity,
  spend nullifier semantics, legacy sender migration, checkpoint backend, and
  wording guards;
- [x] remove duplicated fixture helpers where they preserve the old truth;
- [x] lock the selected test homes before running the main closure waves.

Files:

- `crates/z00z_storage/tests/test_claim_source_proof.rs`
- `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
- `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

Tests:

- [x] confirm each seam has one selected primary test home and no ambiguous
  duplicate helper path remains

Exit condition:

- every major Phase 034 seam has one truthful primary test home.

## 🧪 Concrete Validation And Closeout Tasks

### 034-09 Claim Continuity Test Wave

Spec references:

- `034-33AUDIT.md -> Q63`

MANDATORY pre-read:

- `034-33AUDIT.md -> Q63`
- `034-33AUDIT.md -> Failure Paths`

- [x] run the storage and simulator claim-continuity tests after `034-01` and
  `034-02`;
- [x] prove persisted membership state, not helper reconstruction, is the live
  source of truth.

Files:

- `crates/z00z_storage/tests/test_claim_source_proof.rs`
- `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`

Tests:

- [x] persisted membership positive path
- [x] helper-owned stale path negative case

Exit condition:

- the Q63 claim continuity blocker is no longer reproducible in its old form.

### 034-10 Spend Nullifier Test Wave

Spec references:

- `034-33AUDIT.md -> Q64`
- `034-fix-spec-4.md -> Required Verification Matrix -> Workstream A`

MANDATORY pre-read:

- `034-33AUDIT.md -> Q64`
- `034-fix-spec-4.md -> Workstream A verification`

- [x] run wallet and simulator spend tests after `034-03` through `034-05`;
- [x] prove deterministic nullifier derivation, duplicate rejection, and
  structural-rule alignment.

Files:

- `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
- `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
- `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`

Tests:

- [x] valid deterministic nullifier path
- [x] malformed or wrong nullifier path
- [x] duplicate nullifier path
- [x] migrated sender authority path still preserves spend gate behavior

Exit condition:

- the Q64 spend-nullifier blocker is closed by code and tests, not only by
  narrowed wording.

### 034-11 Checkpoint Backend Test Wave

Spec references:

- `034-33AUDIT.md -> Q65`

MANDATORY pre-read:

- `034-33AUDIT.md -> Q65`
- `034-33AUDIT.md -> Failure Paths`

- [x] run storage, reload, and simulator checkpoint tests after `034-06` and
  `034-07`;
- [x] prove finalize and load acceptance are driven by the authoritative proof
  backend contract.

Files:

- `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
- `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
- `crates/z00z_storage/tests/test_redb_rehydrate.rs`
- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`

Tests:

- [x] backend proof positive path
- [x] compatibility-only payload negative path
- [x] reload drift negative path

Exit condition:

- the Q65 checkpoint-backend blocker is no longer reproducible in finalize or
  load flows.

### 034-12 Documentation Allowlist And Wording Reclassification

Spec references:

- `034-33AUDIT.md -> Failure Paths`
- `034-33AUDIT.md -> Exact Fixes Required Summary -> Q47`
- `034-fix-spec-4.md -> Documentation Updates Required`

MANDATORY pre-read:

- `034-33AUDIT.md -> Q47`
- `034-fix-spec-4.md -> Documentation Updates Required`
- `034-deferred.md -> Recommended Phase 034 Scope Boundary`

- [x] Re-evaluate what may stay in active documentation only after Q63, Q64,
  and Q65 land and are green.
- [x] Update live requirements and wording targets so they describe the new
  implemented truth without overclaiming beyond the tested seams.
- [x] Review and update the concrete planning docs named by `034-fix-spec-4`
  so they use the canonical wording for regular spend nullifier semantics and
  sender-construction authority.
- [x] Leave append-only historical audit artifacts historical; do not rewrite
  old audit evidence as if it had always said the new truth.

Files:

- `.planning/REQUIREMENTS.md`
- `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md`
- `.planning/temp/Z00Z-ECC-IDEAS.md`
- `.planning/temp/Z00Z-ECC-SPEC_part1.md`
- `crates/z00z_simulator/src/scenario_1/stage_12.rs`
- `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`

Tests:

- [x] prepare the wording surfaces that `034-13` will verify

Exit condition:

- live documentation targets are updated only after the code and prior waves
  prove the new truth.

### 034-13 Documentation And Stage-Surface Test Wave

Spec references:

- `034-33AUDIT.md -> Q47`
- `034-fix-spec-4.md -> Documentation Updates Required`

MANDATORY pre-read:

- `034-33AUDIT.md -> Q47`
- `034-fix-spec-4.md -> Documentation Updates Required`

- [x] run wording and documentation guard tests after `034-12`;
- [x] prove active requirements and stage-surface wording describe the new
  truth and no longer pin the repository to the old open-gap wording.

Files:

- `.planning/REQUIREMENTS.md`
- `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md`
- `.planning/temp/Z00Z-ECC-IDEAS.md`
- `.planning/temp/Z00Z-ECC-SPEC_part1.md`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_wallets/tests/test_scenario1_semantics.rs`

Tests:

- [x] active requirement text matches implemented closure
- [x] stage-surface wording matches implemented closure
- [x] planning-doc wording no longer points future work at `builder.rs` or
  `output_flow.rs` as canonical sender-construction authority
- [x] historical append-only artifacts remain untouched

Exit condition:

- Q47 is closed honestly and the documentation allowlist reflects the shipped,
  tested Phase 034 truth.

### 034-14 Closure, Regression, And Phase-Proof Sweep

Spec references:

- `034-33AUDIT.md -> Final Status`
- `034-33AUDIT.md -> Doublecheck Results`
- `034-fix-spec-4.md -> Required Verification Matrix`

MANDATORY pre-read:

- `034-33AUDIT.md -> Findings Summary`
- `034-33AUDIT.md -> Final Status`
- `034-fix-spec-4.md -> Required Verification Matrix`

- [x] Run the full targeted regression sweep for claim continuity, regular
  spend semantics, legacy sender migration, checkpoint backend closure, and
  wording guards.
- [x] Confirm the Phase 033 partial closure blockers are now either implemented
  or explicitly re-narrowed by source update.
- [x] Produce the phase-local verification summary and closure evidence needed
  to retire the Q63, Q64, Q65, and Q47 blockers.

Files:

- phase-local verification artifact under `.planning/phases/034-mix1-fixes/`
- phase-local summary artifact under `.planning/phases/034-mix1-fixes/`

Tests:

- [x] run the required targeted wallet, storage, and simulator tests from
  `034-09` through `034-13`

Exit condition:

- Phase 034 has one repository-backed closure package that proves the old
  partial blockers are no longer the active truth.

### 034-15 Optional Keep-Path Complexity Sidecar

Status: executed local sidecar on the live tree; intentionally non-semantic
and not part of Phase 034 semantic closure evidence.

Spec references:

- `034-deferred.md -> Intake Decision`
- `034-deferred.md -> Optional Tiny Debt Specification -> STORQ-KEEP-PATH-COMPLEXITY`
- `034-deferred.md -> Final Recommendation`

MANDATORY pre-read:

- `034-deferred.md -> Why this is safe to do`
- `034-deferred.md -> Why this is not core Phase 034 work`
- `034-deferred.md -> Acceptance rule`

- [x] Refactor `keep_path(...)` into smaller local predicates or guard helpers
  inside the same storage query seam without changing any list semantics.
- [x] Preserve the exact filter order and truth table for scope, `start`, `end`,
  and `after` handling so paging and range behavior remain byte-for-byte
  compatible at the observable API level.
- [x] Keep the cleanup local to `store_query.rs`; do not widen it into a search
  API redesign, a new listing abstraction, or a broad cleanup pass.
- [x] Leave the sidecar explicitly non-blocking in phase reporting and do not
  use it as evidence that Phase 034 semantic blockers are closed.

Files:

- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
- `crates/z00z_storage/tests/test_search_api.rs`

Tests:

- [x] extend `crates/z00z_storage/tests/test_search_api.rs`
  - scope filtering still returns the same path set after the refactor
  - `start` and `end` bounds still preserve the same inclusive range behavior
  - `after` token paging still preserves the same deterministic page split

Exit condition:

- `keep_path(...)` is structurally simpler, list behavior is unchanged, and the
  sidecar remains explicitly outside the semantic completion story for Phase
  034.

### 034-16 Optional Bounded 5-Word Signature Compliance Wave

Spec references:

- `.github/copilot-instructions.md -> Local Rust Rules`
- `.github/copilot-instructions.md -> No identifiers longer than 5 words`

MANDATORY pre-read:

- `.github/copilot-instructions.md -> Local Rust Rules`
- `.github/copilot-instructions.md -> Rust Delivery Gate`
- `034-TODO.md -> 🚫 Explicit Phase Boundary`

- [x] Build a fresh declaration-backed inventory of live Rust signature-like
  identifiers longer than 5 words across workspace crates, excluding
  `crates/z00z_crypto/tari/**`, and use that inventory as the only rename
  authority.
- [x] Do not assume any previously cited example is still live; verify each
  candidate in the current workspace before scheduling or renaming it.
- [x] Freeze one bounded execution inventory before renames begin by grouping
  the live violations into explicit crate-local or subsystem-local batches, so
  the wave can complete as a full workspace cleanup without reopening the
  inventory after each rename.
- [x] Rename only the current non-Tari violations selected by that bounded
  execution inventory for functions, methods, constants, failpoint IDs,
  metric IDs, and other signature-like identifiers covered by
  `.github/copilot-instructions.md`.
- [x] Preserve behavior, test intent, and public-contract meaning while
  shortening names; update call sites, assertions, grep guards, and source-text
  contract surfaces that would otherwise drift after the rename.
- [x] Keep this sidecar explicitly separate from the Q63, Q64, Q65, and Q47
  semantic closure story; naming hygiene must not be used as proof that the
  phase fixed claim continuity, spend nullifier semantics, or checkpoint proof
  authority.

Files:

- workspace-wide non-Tari Rust files matched by the inventory
- explicit exclusion: `crates/z00z_crypto/tari/**`

Tests:

- [x] run targeted crate tests for every renamed surface
- [x] run workspace grep checks needed to update source-text guards and rename
  expectations honestly

Exit condition:

- one bounded workspace-wide execution inventory is fully consumed, all selected
  non-Tari >5-word signature violations are renamed consistently, affected test
  and text guards are updated, and the wave remains explicitly outside the
  semantic completion story for Phase 034.

### 034-17 Optional Legacy Collision Retirement Sidecar

Spec references:

- `034-suffixes-V1-Vn.md -> Fixed Table`
- `034-suffixes-V1-Vn.md -> Bottom Line`

MANDATORY pre-read:

- `034-suffixes-V1-Vn.md -> Fixed Table`
- `034-suffixes-V1-Vn.md -> Bottom Line`
- `034-TODO.md -> 🚫 Explicit Phase Boundary`

- [x] Use `034-suffixes-V1-Vn.md` as the authoritative suffix inventory for
  this sidecar and preserve its distinction between `production-current`
  surfaces and `reserved-future` compatibility or future-only surfaces.
- [x] Build a current-tree collision map for every production-current suffix
  target whose future unsuffixed canonical name is already occupied by a live
  legacy symbol.
- [x] Retire, rename, or narrow only those live legacy blockers needed to free
  the canonical unsuffixed targets for the later suffix collapse, without
  changing persisted version values, header bytes, discriminators, or live
  compatibility semantics.
- [x] Do not blindly remove compatibility readers or migration constants that
  the source inventory still marks as live old-format support; limit the wave
  to canonical-source narrowing and blocker retirement needed for the later
  collapse.
- [x] Update all affected tests, source-text guards, grep expectations, and
  documentation so the repository stops advertising the retired legacy blockers
  as canonical current names.
- [x] Keep this sidecar explicitly separate from the Q63, Q64, Q65, and Q47
  semantic closure story; legacy-collision retirement must not be used as proof
  that the phase fixed claim continuity, spend nullifier semantics, or
  checkpoint proof authority.

Files:

- legacy-owner files proven by the collision map, including current confirmed
  blockers such as `crates/z00z_crypto/src/claim/proof.rs` and
  `crates/z00z_wallets/src/core/backup/wallet_backup_container.rs`
- any directly affected tests, docs, and compatibility shims touched by the
  blocker retirement

Tests:

- [x] run targeted crate tests for every retired or renamed legacy blocker
- [x] run workspace grep checks proving the retired legacy blockers no longer
  occupy the future canonical targets needed by the later suffix collapse

Exit condition:

- the live legacy blockers that would collide with the planned unsuffixed
  production-current names are retired, renamed, or narrowed safely, affected
  tests and text guards are updated, and the sidecar remains explicitly outside
  the semantic completion story for Phase 034.

### 034-18 Optional Production-Current Suffix Collapse Sidecar

Spec references:

- `034-suffixes-V1-Vn.md -> Fixed Table`
- `034-suffixes-V1-Vn.md -> Bottom Line`

MANDATORY pre-read:

- `034-suffixes-V1-Vn.md -> Fixed Table`
- `034-suffixes-V1-Vn.md -> Bottom Line`
- `034-TODO.md -> 🚫 Explicit Phase Boundary`

- [x] Rename production-current Rust-facing structs, constants, tags, and
  helper types to unsuffixed canonical names where the source inventory marks
  them as the current production shape, for example `AAD_SECRET_VER_V2` to
  `AAD_SECRET` and `BackupAssociatedDataV1` to `BackupAssociatedData`, without
  changing the underlying persisted version values, header bytes,
  discriminators, or compatibility semantics.
- [x] Delete or retire reserved-future suffix-bearing code and tests only for
  entries whose source-backed inventory says the current production path does
  not still select them for read, import, open-session, or migration support.
- [x] Update all affected tests, source-text guards, grep expectations, and
  documentation so the repository only advertises the unsuffixed
  production-current shapes that remain canonical after the cleanup.
- [x] Keep this sidecar explicitly separate from the Q63, Q64, Q65, and Q47
  semantic closure story; suffix cleanup must not be used as proof that the
  phase fixed claim continuity, spend nullifier semantics, or checkpoint proof
  authority.

Files:

- `crates/z00z_wallets/src/db/redb_wallet_crypto.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store_open_session.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store_migrations.rs`
- `crates/z00z_wallets/src/core/backup/backup_wire.rs`
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs`
- `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs`
- `crates/z00z_wallets/src/core/key/seed_cipher_params.rs`
- `crates/z00z_wallets/src/core/key/seed_cipher_container.rs`
- `crates/z00z_storage/src/checkpoint/artifact_stmt.rs`
- `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`
- `crates/z00z_storage/src/checkpoint/artifact_final.rs`
- `crates/z00z_crypto/src/claim/v2.rs`
- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
- `crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs`
- `crates/z00z_crypto/src/crypto_constants.rs`
- `crates/z00z_crypto/src/types_validation.rs`
- tests and docs touched by the source inventory

Tests:

- [x] run targeted crate tests for every renamed production-current surface
- [x] run targeted compatibility tests for every reserved-future path kept for
  old-format read or migration support
- [x] run workspace grep checks proving the old suffixed production-current
  names no longer remain canonical outside intentional compatibility surfaces

Exit condition:

- production-current suffix-bearing names have unsuffixed canonical Rust-facing
  surfaces, reserved-future cleanup is limited to paths proven safe by the
  source-backed inventory, affected tests and text guards are updated, and the
  sidecar remains explicitly outside the semantic completion story for Phase
  034.

## ✅ Completion Gate

This backlog is complete only when all of the following hold:

- every numbered task `034-01` through `034-14` is completed or explicitly
  deferred by a source update;
- `034-15` is optional; if it is executed, it must stay behavior-preserving and
  must not be counted as semantic closure evidence for Q63, Q64, Q65, or Q47;
- `034-16` is optional; if it is executed, it must stay non-Tari,
  inventory-backed, behavior-preserving, and must not be counted as semantic
  closure evidence for Q63, Q64, Q65, or Q47;
- `034-17` is optional; if it is executed, it must stay source-backed against
  `034-suffixes-V1-Vn.md`, must retire only live legacy blockers needed for the
  later suffix collapse, and must not be counted as semantic closure evidence
  for Q63, Q64, Q65, or Q47;
- `034-18` is optional; if it is executed, it must stay source-backed against
  `034-suffixes-V1-Vn.md`, must not blindly delete compatibility readers that
  are still marked as live migration support, and must not be counted as
  semantic closure evidence for Q63, Q64, Q65, or Q47;
- Q63, Q64, Q65, and Q47 no longer remain open in the live closure ledger;
- claim continuity is storage-backed, regular spend nullifier semantics are
  implemented, and checkpoint finalize or load acceptance is backend-bound;
- `core::stealth` is the only public sender-construction authority and no
  legacy `core::tx` construction entrypoint remains canonical, with the
  corresponding workspace-grep migration sweep completed for the legacy sender
  and output-construction wrappers;
- active requirements and stage-surface wording reflect the new truth without
  overclaiming beyond the implemented seams;
- the optional `keep_path(...)` cleanup, if performed, remains explicitly
  outside the semantic closure claim for Phase 034;
- the optional identifier-length cleanup, if performed, remains explicitly
  outside the semantic closure claim for Phase 034;
- the optional legacy-collision retirement, if performed, remains explicitly
  outside the semantic closure claim for Phase 034;
- the optional suffix cleanup, if performed, remains explicitly outside the
  semantic closure claim for Phase 034.
