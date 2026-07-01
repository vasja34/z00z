# Phase 034 Test Tasks

## 🎯 Objective

Turn the Phase 034 planning bundle into an implementation-order test backlog for
unit, integration, source-text, and closeout coverage.

This file assumes the engineer will implement test code later. It defines what
to write, where to place it, which scenarios to cover, and how to verify that
the resulting tests are meaningful.

The live repository now tracks execution truth in `034-VALIDATION.md`,
`034-CLOSEOUT.md`, and `034-09-SUMMARY.md`. This file remains the planning
backlog for test ownership and implementation order, not the canonical ledger
for what Phase 034 already completed on the live tree.

## ✅ Reviewed Status

- [x] Verification commands below now align with the exact named-binary proof
  commands used by the live Phase 034 validation package.
- [x] Optional sidecar coverage now tracks `034-15`, `034-16`, `034-17`, and
  `034-18`, with only `034-15` still deferred on the live tree.
- [x] Spend-nullifier task wording keeps the shipped narrow public boundary and
  does not overclaim verifier-side deterministic recomputation.

## 📚 Inputs

- `034-TEST-SPEC.md`
- `034-CONTEXT.md`
- `034-TODO.md`
- `034-01-PLAN.md` through `034-09-PLAN.md`

## 🧭 Execution Policy

- Prefer extending truthful existing test homes.
- Create proposed test files only when no truthful existing home exists.
- Keep RED-GREEN discipline: if a new test exposes an implementation bug,
  record it as a Phase 034 implementation blocker rather than weakening the
  test.
- Do not write semantic-closure tests for optional sidecars until the semantic
  chain `034-01` through `034-14` is already covered.

## 🌊 Wave Breakdown

### Wave 0: Anchor Inventory And Harness Freeze

#### Task T0-01 Existing Anchor Truth Check

- Objective: confirm which plan-named test homes already exist and which are
  still proposed.
- Files:
  - existing homes listed in `034-TEST-SPEC.md`
  - proposed-only homes listed in `034-TEST-SPEC.md`
- Actions:
  - verify all existing homes still compile in the current branch context
  - mark proposed-only files as create-if-needed instead of assumed anchors
  - confirm source-text guard tests can read planning docs via relative paths
  - assign one canonical test home for each major seam: claim continuity,
    spend nullifier semantics, legacy sender migration, checkpoint backend, and
    wording guards
  - remove duplicated fixture helpers where they preserve the old truth
  - lock the selected seam homes before running the main closure waves
- Done when: the implementation engineer has one frozen map of existing vs
  proposed anchors, one canonical seam home per major closure surface, no
  duplicate old-truth helper remains in the active test harness, and later
  waves do not guess file placement or ownership again.

### Wave 1: Claim Continuity Coverage

#### Task T1-01 Storage Claim Contract Tests

- Objective: implement `UT-034-CLAIM-01` and `UT-034-CLAIM-02`.
- Files:
  - `crates/z00z_storage/tests/test_claim_source_proof.rs`
- Required assertions:
  - persisted membership root equals emitted proof root
  - proof blob root bind matches the authoritative root bytes
  - missing membership or path drift rejects fail closed
  - helper-owned reconstruction is not treated as the authority seam
  - every live `make_claim_source_proof(...)` helper used by phase-owned tests
    is either migrated to the persisted seam or explicitly non-authoritative
- Verification:
  - `cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture`

#### Task T1-02 Claim Package Producer Or Consumer Integration

- Objective: implement `IT-034-CLAIM-03`.
- Files:
  - `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`
- Required assertions:
  - Scenario 1 claim package uses the persisted seam
  - emitted proof version and root version remain canonical
  - wrong authority anchor or malformed bundle version rejects
- Verification:
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support -- --nocapture`

#### Task T1-03 Wallet Claim Verifier Drift Rejection

- Objective: implement `IT-034-CLAIM-04`.
- Files:
  - `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`
- Required assertions:
  - wallet verifier rejects source-root drift
  - wallet verifier rejects proof drift using the same canonical seam
  - no fallback helper path remains accepted
- Verification:
  - `cargo test -p z00z_wallets --release test_claim_tx -- --nocapture`

### Wave 2: Spend Nullifier Semantics Coverage

#### Task T2-01 Deterministic Nullifier Unit Coverage

- Objective: implement `UT-034-SPEND-01` through `UT-034-SPEND-04`.
- Files:
  - `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
  - internal tests in `spend_rules.rs`
  - internal tests in `spend_verification.rs`
- Required assertions:
  - `nullifier_hex` matches deterministic derivation from `chain_id || s_in`
  - one canonical helper owns the exact domain symbol, fixed-width `chain_id`
    encoding, and byte framing for the derivation
  - missing or malformed hex rejects
  - signed nullifier drift rejects on the public contract and deterministic
    mismatch rejects in structural validation
  - duplicate nullifier rejects
  - the public seam authenticates one signed nullifier field and rejects
    malformed, duplicate, and post-signature drift without being described as a
    verifier-side deterministic-derivation seam
  - witness and structural paths stay aligned on deterministic derivation and
    on their intended reject semantics for deterministic mismatch
- Verification:
  - `cargo test -p z00z_wallets --release --test test_spend_witness_gate -- --nocapture`

#### Task T2-02 Scenario 1 Spend Integration Coverage

- Objective: implement `IT-034-SPEND-05`.
- Files:
  - `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
  - `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
- Required assertions:
  - valid signed nullifier path passes on the public seam while witness and
    structural deterministic derivation stays aligned
  - missing, malformed, duplicate, and signed-drift nullifier paths reject on
    the public seam
  - deterministic-mismatch coverage remains bound to the witness and structural
    seam rather than being overstated as a standalone public-verifier property
  - migrated sender authority path still preserves spend gate behavior
  - wording guard only flips to closed wording after semantic acceptance is
    actually implemented
- Verification:
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture`
  - `cargo test -p z00z_wallets --release --test test_scenario1_semantics -- --nocapture`

### Wave 3: Sender-Authority Migration Coverage

#### Task T3-01 Canonical Sender Example Coverage

- Objective: implement `IT-034-SENDER-01`.
- Files:
  - `crates/z00z_wallets/tests/test_s5_sender_examples.rs`
- Required assertions:
  - canonical sender path comes from `core::stealth`
  - old tx-owned constructor vocabulary is no longer canonical
  - `core::tx` no longer re-exports retired sender constructors as silent
    public owner surfaces, or any retained shim fails closed with an explicit
    migration error
  - examples still preserve tag16, owner_tag, serial-sensitive behavior
- Verification:
  - targeted wallet test command for `test_s5_sender_examples`

#### Task T3-02 Runtime And Source-Text Guard Coverage

- Objective: implement `IT-034-SENDER-02`.
- Files:
  - `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_tests.rs`
  - `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_test_support.rs`
  - `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
  - `crates/z00z_simulator/tests/test_claim_acceptance.rs`
- Required assertions:
  - runtime helpers no longer route authority through legacy wrappers
  - source-text guards ban retired legacy construction signatures
  - stage 4 or stage 6 support paths still behave correctly after migration
- Verification:
  - targeted simulator tests plus release simulator gate

#### Task T3-03 Module-Local Ownership Guards

- Objective: implement `UT-034-SENDER-03`.
- Files:
  - `crates/z00z_wallets/src/services/wallet_service_tests.rs`
  - `crates/z00z_wallets/src/core/stealth/test_output_extra.rs`
  - `crates/z00z_wallets/tests/test_s5_spec6_bridge.rs`
  - `crates/z00z_wallets/tests/test_adversarial.rs`
  - `crates/z00z_wallets/tests/test_phase14_pipeline.rs`
  - `crates/z00z_wallets/tests/test_phase15_regress.rs`
  - `crates/z00z_wallets/tests/test_tx_serial.rs`
  - `crates/z00z_wallets/tests/test_s5_leaf_gate.rs`
- Required assertions:
  - service layer no longer imports legacy tx-owner constructors
  - stealth-owned helper path remains reachable only through the new owner
  - serial and leaf constraints remain intact after owner move
- Verification:
  - targeted wallet test commands plus broader simulator gate if cross-crate
    behavior moves

### Wave 4: Checkpoint Backend Coverage

#### Task T4-01 Authoritative Backend Contract Tests

- Objective: implement `UT-034-CKPT-01` and `UT-034-CKPT-02`.
- Files:
  - `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
  - `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
- Required assertions:
  - authoritative backend positive path succeeds deterministically
  - legacy decode remains readable only as non-authoritative compatibility
    classification
  - compatibility-only payload is not accepted as authority
  - proof-system mismatch rejects
  - statement-shape drift rejects
  - exec-identity drift rejects
  - snapshot or link-tuple drift rejects
  - payload-shape drift rejects
  - backend mismatch rejects
- Verification:
  - `cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture`
  - `cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture`

#### Task T4-02 Reload And Promotion Integration Tests

- Objective: implement `IT-034-CKPT-03` and `IT-034-CKPT-04`.
- Files:
  - `crates/z00z_storage/tests/test_redb_rehydrate.rs`
  - `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
  - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- Required assertions:
  - reload rejects backend, proof-system, statement-shape, exec-identity,
    snapshot or link-tuple, or payload-shape drift
  - stage 12 acceptance blocks semantic drift and blocks authoritative summary
    emission on failure
  - stage-surface wording reflects the backend-bound truth
- Verification:
  - `cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture`

### Wave 5: Documentation Truth And Closeout Coverage

#### Task T5-01 Documentation Allowlist Guards

- Objective: implement `DOC-034-01` through `DOC-034-03`.
- Files:
  - `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
  - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
  - active planning docs named in `034-12` and `034-13`
- Required assertions:
  - active requirement text matches implemented closure
  - active `034-CONTEXT.md` truth matches implemented closure
  - live code wording surfaces no longer advertise helper-owned claim authority
    or pre-closure checkpoint wording
  - concrete planning docs named by `034-fix-spec-4.md` use the updated
    canonical wording for regular spend nullifier semantics and
    sender-construction authority
  - no active planning doc still points future work at `builder.rs` or
    `output_flow.rs` as canonical sender authority
  - historical append-only artifacts remain untouched
- Verification:
  - same targeted wording guard commands as Wave 2 and Wave 4

#### Task T5-02 Closeout Artifact Proof

- Objective: implement `IT-034-CLOSE-01`.
- Files:
  - `034-VALIDATION.md`
  - `034-CLOSEOUT.md`
  - `.planning/ROADMAP.md`
  - `.planning/STATE.md`
- Required assertions:
  - closeout artifact reconciles every mandatory task `034-01` through `034-14`
  - roadmap and state reflect the same phase status
  - optional sidecars are excluded from semantic closure claims
- Verification:
  - repository-side artifact checks plus final simulator release gate

### Wave 6: Optional Sidecar Coverage

#### Task T6-01 Keep-Path Search Regression

- Objective: implement `OPT-034-SEARCH-01` only if `034-15` executes.
- Files:
  - `crates/z00z_storage/tests/test_search_api.rs`
- Required assertions:
  - identical path set under scope filtering
  - identical inclusive `start`/`end` range behavior
  - identical `after` paging split
- Verification:
  - `cargo test -p z00z_storage --release --test test_search_api -- --nocapture`

#### Task T6-02 Rename Hygiene Guards

- Objective: implement `OPT-034-REN-01` only if `034-16` executes.
- Files:
  - targeted crate tests selected by the fresh identifier inventory
  - grep and source-text guard surfaces
- Required assertions:
  - no non-Tari >5-word selected identifier remains canonical
  - renamed surfaces preserve behavior
  - call sites, assertions, grep guards, and source-text contract surfaces are
    updated together so rename fallout does not drift
  - historical references remain clearly intentional

#### Task T6-03 Suffix Collapse Guards

- Objective: implement `OPT-034-SFX-01` only if `034-18` executes.
- Files:
  - targeted crate tests selected by `034-suffixes-V1-Vn.md`
  - grep and source-text guard surfaces
- Required assertions:
  - use `034-suffixes-V1-Vn.md` as the authoritative suffix inventory and keep
    any normalized execution list subordinate to that source table
  - unsuffixed names are canonical for production-current surfaces
  - compatibility readers or migration paths remain where still required
  - no on-wire or on-disk semantic value changes occur
  - grep expectations and documentation are updated so only the unsuffixed
    production-current shapes remain canonical after cleanup

## ✅ Completion Conditions For This Test Backlog

- Every mandatory semantic scenario from `034-TEST-SPEC.md` is implemented in a
  truthful existing home or an explicitly created proposed home.
- Every negative case names a concrete reject condition and observes it in test
  output.
- Every source-text guard names the stale claim it is banning.
- RED-GREEN results are recorded honestly: passing tests, implementation bugs,
  or blocked execution all remain distinguishable.
- Optional sidecar tests are clearly separated from the mandatory semantic
  closure suite.

## 📌 Recommended Verification Order

1. `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
2. targeted claim tests
3. targeted spend tests
4. targeted sender-migration guards
5. targeted checkpoint tests
6. targeted documentation guards
7. `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
8. `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump` when scenario smoke evidence is needed
9. `cargo test --release --features test-fast --features wallet_debug_dump` only when cross-crate fallout requires the broader sweep

Exact named-binary commands are the proof commands for the Phase 034 task
surfaces above. The broader simulator and workspace reruns are corroborating
smoke gates and must not replace those proof commands.
