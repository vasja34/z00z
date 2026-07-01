---
phase: 032-crypto-audit-scenario-1
artifact: test-spec
status: review-backed
source: context-todo-plans-and-live-test-anchors
updated: 2026-04-05
---

# Phase 032 Test Spec

## 🎯 Purpose

📌 This document defines the phase-local E2E, integration, and focused unit
coverage required to close Phase 032.

📌 It is intended to be directly usable by another engineer or agent without
guessing scenario boundaries, cryptographic invariants, negative paths,
observable outputs, or pass oracles.

📌 Phase 032 is Rust-driven crypto, wallet, simulator, and storage validation.
It is not browser automation. End-to-end proof must come from release-mode
Rust tests that exercise the real Scenario 1 claim, spend, checkpoint, and
storage pipeline.

## 🔔 Workflow Status

✅ This test spec is review-backed and derived from the current Phase 032
planning chain that was already coverage-audited against `032-TODO.md`.

⛔ The literal `gsd-add-tests` workflow is still blocked for Phase 032.

📌 The current phase directory still does not contain a phase-level
`032-SUMMARY.md`, and the broader phase remains reopened on `PH32-SPEND` and
`PH32-CLAIM-TRUST`.

📌 The current phase directory now contains `032-VERIFICATION.md`, but that
artifact is intentionally narrower than full phase closeout and must not be
treated as proof that the reopened spend or claim-trust requirements are
finished.

📌 Until the broader reopened requirement is honestly closed and the phase-level
planning truth realigns, Phase 032 test work must continue to use this spec as
the manual fallback contract instead of pretending the completed-phase workflow
already applies.

📌 Manual fallback progress already implemented and validated in the current
tree:

- `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
- `crates/z00z_simulator/tests/test_transport_rng_boundaries.rs`
- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
- `crates/z00z_simulator/tests/test_stage6_checkpoint.rs`
- `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`

📌 The inputs for this file are:

- `.planning/phases/032-crypto-audit-scenario-1/032-CONTEXT.md`
- `.planning/phases/032-crypto-audit-scenario-1/032-TODO.md`
- `.planning/phases/032-crypto-audit-scenario-1/032-01-PLAN.md`
- `.planning/phases/032-crypto-audit-scenario-1/032-02-PLAN.md`
- `.planning/phases/032-crypto-audit-scenario-1/032-03-PLAN.md`
- `.planning/phases/032-crypto-audit-scenario-1/032-04-PLAN.md`
- `.planning/phases/032-crypto-audit-scenario-1/032-05-PLAN.md`
- `.planning/phases/032-crypto-audit-scenario-1/032-06-PLAN.md`
- `.planning/phases/032-crypto-audit-scenario-1/032-07-PLAN.md`
- Existing live anchors in `crates/z00z_crypto/tests/`,
  `crates/z00z_wallets/tests/`, `crates/z00z_simulator/tests/`, and
  `crates/z00z_storage/tests/`

📌 This document distinguishes between:

- existing test anchors already present in the repository
- proposed new test files already named by the Phase 032 plans

📌 This verification contract does not close `PH32-SPEND` or
`PH32-CLAIM-TRUST` by itself and must not be read as phase-completion evidence
for the broader original spend or claim-trust requirements while
`.planning/REQUIREMENTS.md` still keeps those requirements open.

📌 No file listed as existing below should be treated as hypothetical.

📌 Proposed entries below are explicitly split into:

- planned-but-not-created yet
- created manually under the Phase 032 fallback path

## ⚙️ Classification

### ✅ TDD And Integration Targets

- `crates/z00z_wallets/src/core/stealth/output.rs`
  because Stage 4 output semantics must stop accepting caller-controlled or
  ambiguous ownership metadata.
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
  because output construction must own `leaf_ad_id`, `s_out`, and request/card
  binding semantics.
- `crates/z00z_wallets/src/core/stealth/output_validator.rs`
  because malformed ownership fields must reject before they become accepted
  wallet truth.
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`
  because scanner parity must prove owned-vs-foreign behavior under the frozen
  semantic contract.
- `crates/z00z_wallets/src/core/address/stealth_request.rs`
  because request-bound derivation must become the canonical privacy path.
- `crates/z00z_wallets/src/core/address/stealth_card.rs`
  because card-only flows must not silently bypass request or trust checks.
- `crates/z00z_wallets/src/core/address/stealth_trust.rs`
  because trust decisions must remain explicit when receiver identity and
  request semantics are validated.
- `crates/z00z_crypto/src/claim/v2.rs`
  because `ClaimStmtV2` must bind the exact storage-owned tuple and reject all
  partial or placeholder variants.
- `crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs`
  because claim statement construction must consume the canonical tuple and not
  derive roots from non-authoritative fields.
- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
  because claim verification must reject tuple or digest tampering with precise
  reject classes.
- `crates/z00z_wallets/src/core/tx/claim_auth.rs`
  because the claim authority signature must authenticate the storage-owned
  tuple, not a reconstructed surrogate.
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`
  because Stage 3 emits the portable claim package and must preserve the
  authoritative proof and root binding.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs`
  because downstream claim consumers must continue to trust only the
  authoritative storage-owned statement.
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
  because claim source root and proof must be storage-owned and reproducible.
- `crates/z00z_wallets/src/core/tx/witness_gate.rs`
  because the witness gate must stop acting as a fake proof boundary.
- `crates/z00z_wallets/src/core/tx/spending.rs`
  because spend verification must consume a real public verifier contract.
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
  because the public input statement and verification path live here.
- `crates/z00z_wallets/src/core/tx/prover.rs`
  because prover-side emitted proof bytes must be the exact bytes verified by
  the public verifier.
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`
  because Scenario 1 Stage 4 must reject fake proof success and wrong-root
  acceptance.
- `crates/z00z_storage/src/checkpoint/build.rs`
  because checkpoint execution input must remain authoritative over later
  artifacts.
- `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`
  because draft-only checkpoints must never be mistaken for final proof truth.
- `crates/z00z_storage/src/checkpoint/artifact_final.rs`
  because final artifact emission must bind to the executed checkpoint proof.
- `crates/z00z_storage/src/checkpoint/codec.rs`
  because persisted bytes must decode to the same proof-bearing truth that the
  verifier accepted.
- `crates/z00z_storage/src/checkpoint/store_fs.rs`
  because persisted checkpoint load must reject malformed or tampered proof
  artifacts.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
  because stage-6 bundle assembly must not allow placeholder proof blobs to
  promote into final checkpoint truth.
- `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs`
  because state mutation must occur only after the authoritative checkpoint
  gates pass.
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs`
  because stage-2 outputs must not persist reveal-only or secret-bearing blobs
  beyond the allowed boundary.
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs`
  because deterministic or fixed transport randomness must remain bounded,
  explicit, and test-only.
- `crates/z00z_simulator/src/config.rs`
  because proof mode and debug toggles must remain explicit and honest.
- `crates/z00z_simulator/src/config_accessors.rs`
  because configuration reads must not silently default into insecure or
  misleading proof behavior.

### 🚫 E2E Browser Targets

- None.

### ⛔ Skip Targets

- Planning markdown files
  because they are specification inputs, not executable logic.
- Vendor Tari code under `crates/z00z_crypto/tari/`
  because Phase 032 is a Z00Z-owned contract-hardening phase.
- Pure log-only tests
  unless a required guarantee is otherwise unobservable.

## 🔑 Existing Test Anchors To Reuse

📌 Reuse and extend these existing files before creating additional coverage:

- `crates/z00z_crypto/tests/test_claim_v2_contract.rs`
  for exact `ClaimStmtV2` field binding and tuple-contract assertions.
- `crates/z00z_wallets/tests/test_tx_stealth_flow.rs`
  for Stage 4 sender/receiver parity, foreign-scan rejection, and range-proof
  expectations already called out by `032-TODO.md`.
- `crates/z00z_wallets/tests/test_tx_spent_gate.rs`
  for spent-path rejection, no-partial-mutation guarantees, and canonical input
  path retention.
- `crates/z00z_wallets/tests/test_tx_wrong_root.rs`
  for wrong-root rejection semantics.
- `crates/z00z_wallets/tests/test_tx_tamper.rs`
  for post-build tamper rejection.
- `crates/z00z_wallets/tests/test_stealth_request.rs`
  for request generation, chain binding, TOFU behavior, and compact encoding.
- `crates/z00z_wallets/tests/test_stealth_scan_support.rs`
  for runtime-vs-leaf scan support boundaries.
- `crates/z00z_wallets/tests/test_stealth_output.rs`
  for output-level stealth construction invariants.
- `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`
  for authoritative root emission from Stage 3 claim package construction.
- `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`
  for full claim package accept/reject cases, nullifier stability, digest
  stability, and reject-class precision.
- `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`
  for Scenario 1 checkpoint, scan, diff, and persisted artifact end-to-end
  success proof.
- `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`
  for draft-only vs final-artifact emission and proof-blob continuity.
- `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs`
  for restart/replay and corrupted-nullifier-row closure.
- `crates/z00z_storage/tests/test_claim_source_proof.rs`
  for storage-owned claim root, proof version, and nonzero root-binding checks.
- `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
  for final checkpoint artifact behavior.
- `crates/z00z_storage/tests/test_checkpoint_draft_build.rs`
  for draft-stage checkpoint shape.
- `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
  for root-binding persistence and decode invariants.
- `crates/z00z_storage/tests/test_checkpoint_codec.rs`
  for authoritative persisted byte contract.
- `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
  for storage-facing checkpoint load and reject boundaries.

## ⭐ Phase 032 New Test File Status

📌 These files were originally named by the Phase 032 plans.

### ✅ Created Under The Manual Fallback Path

- `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
- `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
- `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
- `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`
- `crates/z00z_simulator/tests/test_stage2_secret_artifacts.rs`
- `crates/z00z_simulator/tests/test_transport_rng_boundaries.rs`

### 📌 Planned But Not Created Yet

None at the time of the 2026-04-05 Phase 032 verification rerun.

📌 The scenarios below may also extend existing anchors instead of introducing a
new file when the existing seam already matches the assertion burden.

## 🔐 Cryptographic And Contract Invariants To Observe

| Invariant | Why It Matters | Assertion Shape |
| --- | --- | --- |
| `leaf_ad_id` is canonical and not caller-controlled | Prevents forged owner-binding or semantic drift across scan and spend paths | tampering or omission rejects, and emitted output keeps only the canonical asset-definition binding |
| `s_out` semantics are frozen across Stage 4, scanner, and runtime parity | Prevents sender/runtime divergence | same output bytes produce the same owned-vs-foreign result across leaf walk, runtime scan, and Stage 4 roundtrip |
| request-bound tag derivation is the normal privacy path | Prevents card-only fallback from silently becoming default | request path yields accepted owned output; card-only path stays explicit and bounded |
| `ClaimStmtV2` binds the exact storage-owned tuple | Prevents claim replay with reconstructed or placeholder roots | any change to source root, path terminal, proof blob, or auth bytes rejects with explicit reject class |
| canonical claim-source helper seam stays stable across producer and consumer paths | Prevents claim truth from drifting into simulator-local reconstruction while preserving the honest caveat that persisted storage-backed continuity remains open | proof produced by the current helper matches the root later authenticated by wallet and simulator flows, but no test may treat that as proof of the broader persisted-storage requirement |
| spend verification uses the current-stack public verifier boundary | Prevents witness-gate or placeholder success from simulating correctness while preserving the open `PH32-SPEND` nullifier gap | fake or malformed proof bytes reject before state mutation |
| draft checkpoints never masquerade as final | Prevents placeholder proof artifacts from becoming ledger truth | draft mode emits no final artifact or link, and final mode binds exact proof bytes |
| persisted checkpoint bytes remain injectively bound to executed input | Prevents artifact replay or load-time ambiguity | decode and reload recover the same checkpoint id, proof bytes, and bound root |
| simulator artifact hygiene keeps secrets out of persisted stage outputs | Prevents reveal-only blobs from becoming transport or disk truth | artifact files omit secret-bearing fields or reject attempts to persist them |
| deterministic RNG is explicit, bounded, and test-only | Prevents honest-closeout drift and false privacy claims | debug or deterministic modes require explicit config and reject silent insecure defaults |

## ✅ Canonical Commands

📌 Use these commands as the execution contract for the Phase 032 test matrix.

```bash
cargo test -p z00z_wallets --release --features test-fast --test test_scenario1_semantics -- --nocapture
cargo test -p z00z_crypto --release --test test_claim_v2_contract -- --nocapture
cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support -- --nocapture
cargo test -p z00z_wallets --release --features test-fast --test test_spend_witness_gate -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage2_secret_artifacts -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_transport_rng_boundaries -- --nocapture
cargo test -p z00z_wallets --release --features test-fast --test test_tx_spent_gate -- --nocapture
cargo test -p z00z_wallets --release --features test-fast --test test_tx_wrong_root -- --nocapture
cargo test -p z00z_wallets --release --features test-fast --test test_tx_tamper -- --nocapture
cargo test -p z00z_wallets --release --features test-fast --test test_tx_stealth_flow -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_draft_build -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_root_binding -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_codec -- --nocapture
cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture
rg -n "(bootstrap_tests|test-fast|wallet_debug_dump|GSD-Review-Tasks-Execution|2 consecutive clean runs|does not prove|out of scope|PH32-SPEND|PH32-CLAIM-TRUST)" \
  .planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md \
  docs/code-review/032-scenario-1-crypto-status.md \
  .planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md
```

📌 A scenario passes only when the required tests prove the stated invariant.
It does not pass merely because a broader suite happened to stay green.

## 🚩 Scenario Catalog

### 032-E2E-01 Semantic Freeze For Stage 4 Outputs And Receiver Paths

📌 Requirements: `032-01`, `D-01`, `D-02`, `D-03`, `D-04`

📌 Primary anchors:

- Existing: `crates/z00z_wallets/tests/test_tx_stealth_flow.rs`
- Existing: `crates/z00z_wallets/tests/test_stealth_request.rs`
- Existing: `crates/z00z_wallets/tests/test_stealth_scan_support.rs`
- Existing: `crates/z00z_wallets/tests/test_stealth_output.rs`
- Proposed: `crates/z00z_wallets/tests/test_scenario1_semantics.rs`

📌 This scenario demonstrates that output construction, request/card handling,
scanner behavior, and runtime parity all obey one frozen semantic contract.

📌 Setup:

- Build one owned Stage 4 stealth output through the canonical request-bound
  path.
- Build one foreign output for a second receiver.
- Attempt controlled tampering of `leaf_ad_id`, `s_out`, request identity, and
  range-proof-carrying output metadata.

📌 Assertions:

- owned receiver path scans as owned under both leaf walk and runtime scan
- foreign receiver path stays explicit `NotMine`
- request-bound flow remains accepted and reproducible
- card-only fallback stays explicit and does not silently replace the request
  contract
- tampered `leaf_ad_id` or mismatched semantic fields reject explicitly
- Stage 4 range proof remains verified in the same workflow that proves wallet
  ownership parity

📌 Negative coverage:

- wrong chain id, wrong request signature, expired request, or identity
  mismatch reject before output acceptance
- Carol cannot scan Bob-owned asset in the canonical Stage 4 flow
- runtime and leaf scan must not disagree on the same output bytes

📌 Pass oracle:

- all owned-vs-foreign assertions match across Stage 4, leaf scan, and runtime
  scan, and any semantic tamper attempt fails before it becomes wallet truth

### 032-E2E-02 `ClaimStmtV2` Binds The Exact Storage-Owned Tuple

📌 Requirements: `032-02`, `D-05`, `D-06`, `D-07`

📌 Primary anchors:

- Existing: `crates/z00z_crypto/tests/test_claim_v2_contract.rs`
- Existing: `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`

📌 This scenario demonstrates that `ClaimStmtV2` binds the exact storage-owned
claim tuple and that the wallet verifier rejects any drift with precise reject
classes.

📌 Setup:

- Build one valid `ClaimTxPackage` from the canonical Stage 3 path.
- Recompute the expected digest from the package payload.
- Mutate one tuple component at a time: source root, nullifier, proof blob,
  fee, and authenticated digest.

📌 Assertions:

- stable input yields stable digest
- valid package verifies successfully
- identical inputs produce identical nullifier
- different scope or asset inputs produce different nullifier or scope hash
- malformed proof blob rejects as `claim_proof_invalid`
- bad nullifier rejects as `claim_nullifier_invalid`
- nonzero fee rejects as `claim_fee_invalid`

📌 Negative coverage:

- mutation of any statement field that should be authenticated must fail
- no verifier path may classify a semantic fee or proof failure as a generic
  digest mismatch

📌 Pass oracle:

- one valid package passes, every single-field mutation rejects with the
  expected class, and digest/nullifier stability is proven by byte equality

### 032-E2E-03 Canonical Claim Source Helper Preserves The Current Tuple Boundary

📌 Requirements: `032-03`, `D-08`, `D-09`, `D-10`

📌 Primary anchors:

- Existing: `crates/z00z_storage/tests/test_claim_source_proof.rs`
- Existing: `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`
- Existing: `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`

📌 This scenario demonstrates that the current helper-owned claim source seam
emits one canonical root and proof tuple and that Stage 3 plus downstream
claim verification preserve that tuple without reconstructing it from
non-authoritative fields. It does not claim that the broader persisted
storage-backed continuity requirement is already closed.

📌 Setup:

- Insert a canonical leaf into `AssetStore`.
- obtain the current helper-owned claim source root and proof from storage
- build a Stage 3 claim package from the same asset

📌 Assertions:

- storage root version is `ClaimRootVer::V1`
- storage proof version is `ClaimProofVer::V1`
- proof blob binds the same nonzero root returned by the current helper seam
- Stage 3 claim package emits a nonzero canonical root
- downstream claim package verification still succeeds without replacing the
  helper-owned root or proof tuple

📌 Negative coverage:

- zero root must never be accepted as canonical claim source truth
- path-terminal drift or reconstructed path elements must reject
- authority signature must not authenticate a different tuple than storage
  emitted

📌 Honest caveat:

- this scenario proves the current canonical helper-owned tuple boundary only
- it must not be used to claim that `PH32-CLAIM-TRUST` is fully closed for the
  broader persisted storage-backed continuity wording

📌 Pass oracle:

- the root and proof produced by the current helper seam are the same tuple
  observed by Stage 3 claim package output and later verifier acceptance,
  without overstating that result into persisted storage-backed continuity

### 032-E2E-04 Spend Verification Uses The Current-Stack Public Verifier Boundary

📌 Requirements: `032-04`, `D-11`, `D-12`, `D-13`

📌 Primary anchors:

- Existing: `crates/z00z_wallets/tests/test_tx_spent_gate.rs`
- Existing: `crates/z00z_wallets/tests/test_tx_wrong_root.rs`
- Existing: `crates/z00z_wallets/tests/test_tx_tamper.rs`
- Existing: `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
- Proposed: `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`

📌 This scenario demonstrates that the witness gate is no longer treated as a
proof of validity and that Scenario 1 only mutates state after the current
code-delivered public verification contract passes.

📌 Setup:

- build one valid transaction package with canonical public inputs
- exercise one spent-input path, one wrong-root path, and one tampered-proof
  path
- run the same proof contract through the wallet seam and through the Scenario
  1 Stage 4 validation seam

📌 Assertions:

- valid proof path is accepted by the current-stack public verifier
- spent input rejects before mutation and returns the spent-path error class
- wrong root rejects explicitly
- tampered proof bytes reject explicitly
- state-before and state-after snapshots remain identical on rejection
- no output leaf is inserted on a rejected spend path

📌 Negative coverage:

- witness-only success must not be enough to pass the transaction
- placeholder or opaque blobs must not be promoted as proof truth in normal
  honest mode
- error text and reject classes must remain proof-gate specific, not generic

📌 Pass oracle:

- every invalid public-input or proof mutation rejects before mutation, and the
  accepted path is the same path used by the current-stack public verifier

### 032-E2E-05 Checkpoint Acceptance Is Authoritative At Build, Finalize, And Load

📌 Requirements: `032-05`, `D-14`, `D-15`, `D-16`, `D-17`

📌 Primary anchors:

- Existing: `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`
- Existing: `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`
- Existing: `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
- Existing: `crates/z00z_storage/tests/test_checkpoint_draft_build.rs`
- Existing: `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
- Existing: `crates/z00z_storage/tests/test_checkpoint_codec.rs`
- Existing: `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
- Existing: `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
- Existing: `crates/z00z_simulator/tests/test_stage6_checkpoint.rs`

📌 This scenario demonstrates that draft checkpoints stay draft-only, malformed
or tampered tx packages fail closed before checkpoint promotion, final artifact
emission carries the exact executed proof bytes, and persisted storage load
rejects malformed checkpoint truth.

📌 Setup:

- run Scenario 1 once in draft-only mode
- run the same scenario in final opaque-test mode
- inspect emitted `exec_input`, `artifact`, `link`, and `audit` binaries
- reload persisted checkpoint artifacts through the storage API

📌 Assertions:

- draft-only mode emits no final artifact, link, or audit path
- final mode emits artifact, link, and audit with one concrete checkpoint id
- checkpoint id in JSON, link, and audit all match
- `exec_input.txs()[0].tx_proof()` equals the final artifact proof bytes
- persisted decode/load preserves the same bound root and checkpoint id
- unified Scenario 1 run produces the expected storage and checkpoint files

📌 Negative coverage:

- malformed or tampered `tx_digest_hex` must fail closed before `exec_input`,
  `checkpoint_s7.json`, final artifact, link, or audit acceptance
- malformed or replayed persisted checkpoint bytes must reject at load
- stage-11 apply must not mutate state unless authoritative checkpoint gates
  already passed

📌 Pass oracle:

- draft-only stays non-final, digest or package tamper fails closed before
  checkpoint promotion, final mode emits one authoritative checkpoint artifact
  family, and persisted decode/load roundtrips recover the same proof truth
  observed during execution

### 032-E2E-06 Stage-2 Artifact Hygiene And RNG Boundaries Stay Honest

📌 Requirements: `032-06`, `D-18`, `D-19`, `D-20`

📌 Primary anchors:

- Existing: `crates/z00z_wallets/tests/test_stealth_request.rs`
- Existing: `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`
- Proposed: `crates/z00z_simulator/tests/test_stage2_secret_artifacts.rs`
- Existing: `crates/z00z_simulator/tests/test_transport_rng_boundaries.rs`

📌 This scenario demonstrates that stage-2 persistence does not leak
reveal-only or secret-bearing blobs and that deterministic randomness is only
available through explicit, honest configuration.

📌 Setup:

- run Scenario 1 under explicit debug or deterministic test configuration
- inspect stage-2 artifacts and transport outputs
- exercise request generation and compact transport roundtrip under the same
  config surface

📌 Assertions:

- persisted stage-2 artifacts exclude secret-bearing fields that are only
  allowed at outer reveal boundaries
- deterministic RNG or transport seeds require explicit config selection
- default honest mode does not silently use fixed randomness
- configuration accessors recover the exact proof/debug mode chosen by the test

📌 Negative coverage:

- a hidden fallback to deterministic randomness is a phase-blocking failure
- a persisted secret-bearing artifact outside the allowed boundary is a
  phase-blocking failure
- transport roundtrip must not depend on ambient implicit config

📌 Pass oracle:

- stage-2 outputs remain non-secret, deterministic RNG is explicit and
  test-bounded, and honest-mode defaults remain non-deterministic and explicit

### 032-E2E-07 Honest Closeout Rejects Unsupported Trustlessness Claims

📌 Requirements: `032-07`, `D-21`, `D-22`, `D-23`, `D-24`, `D-25`

📌 Primary anchors:

- Existing: `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`
- Existing: `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`
- Existing: `crates/z00z_simulator/tests/test_transport_rng_boundaries.rs`
- Existing: `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`
- Existing: `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md`
- Existing: `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md`
- Existing: `docs/code-review/032-scenario-1-crypto-status.md`

📌 This scenario demonstrates that unsupported STARK/FRI, withheld-data, or
opaque-proof claims are never advertised as if they were production-trustless
verification.

📌 Setup:

- exercise final and draft proof modes through simulator config
- attempt to use unsupported proof mode or ambiguous closeout wording in the
  same execution and config boundaries used by Scenario 1 tests
- assert the closeout and verification artifacts retain explicit out-of-scope
  and does-not-prove language for `PH32-SPEND` and `PH32-CLAIM-TRUST`

📌 Assertions:

- unsupported proof systems reject explicitly or remain isolated behind
  test-only proof modes
- honest mode does not claim more than the executed verifier actually proves
- checkpoint acceptance assertions are phrased in terms of executed proof bytes,
  not aspirational protocol marketing
- closeout and status artifacts keep explicit caveats for unresolved broader
  spend and claim-trust requirements

📌 Negative coverage:

- any path that silently upgrades placeholder or opaque proof behavior into
  trustless acceptance is a failure
- any path that depends on withheld data while presenting itself as complete
  verification is a failure
- any artifact that drops the explicit `PH32-SPEND` or `PH32-CLAIM-TRUST`
  caveat while still claiming current review-backed honesty is a failure

📌 Pass oracle:

- all executable modes are explicit, unsupported trustlessness claims remain
  closed, and test outputs describe only what the current verifier really
  proves

## 🧪 Existing TODO-Driven Acceptance Hooks

📌 The test implementation should preserve or extend the explicit evidence hooks
already named in `032-TODO.md`:

- `test_stage4_alice_sends_asset_to_bob`
- `test_stage4_sender_receiver_roundtrip`
- `test_stage4_runtime_roundtrip`
- `test_stage4_path_parity`
- `test_stage4_carol_cannot_scan_bob_asset`
- `test_e2e_runtime_own`
- `test_e2e_runtime_foreign`
- `test_ex1_leaf_walk`
- `test_ex2_runtime_scan`
- `test_ex5_req_merchant`
- `test_restart_replay_path`
- `test_corrupt_rows_closed`

📌 If any of these hooks move, the replacement test must preserve the same
behavioral proof burden and observable pass condition.

## 🛑 Global Pass Rules

1. A scenario fails if it only proves behavior through comments, logs, or
   planning text without executable assertions.
2. A rejection scenario passes only when the reject path is explicit and no
   partial success artifact survives.
3. A cryptographic scenario passes only when the assertion checks exact bytes,
   exact reject class, exact root, exact checkpoint id, or exact persisted
   state.
4. An honest-closeout scenario passes only when unsupported trustlessness
   claims remain closed in executable config and acceptance surfaces.
5. Phase 032 test-spec execution is complete only when all seven scenario
  families are covered by either existing anchors extended in place or the
  proposed new files listed above. This does not, by itself, close any phase
  requirement that remains open in `.planning/REQUIREMENTS.md`.
