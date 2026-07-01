---
phase: 028-crypto-audit-storage
artifact: test-spec
status: verification-backed
source: context-fusion-plans-and-live-code-seams
updated: 2026-03-30
---

# Phase 028 Test Spec

## Purpose

📌 This document defines the unit, Rust integration, and release-style
acceptance coverage required for Phase 028.

📌 It is intended to be directly usable by another engineer or agent without
guessing scenario boundaries, proof paths, state transitions, negative cases,
or which existing seam should own a given assertion.

📌 Phase 028 is a `z00z_storage` crypto-audit remediation phase. Its end-to-end
proof is not browser automation. The required E2E signal is Rust integration
coverage plus release-style simulator validation that proves checkpoint
artifacts are semantically honest, replay artifacts preserve real execution
bytes, proof blobs bind semantic and backend roots, checkpoint IDs and links
are tamper-detectable, and claim replay protection is keyed by canonical binary
nullifier state.

## Workflow Status

✅ Strict fallback conditions no longer apply because
`.planning/phases/028-crypto-audit-storage/` now contains summary artifacts for
plans `028-01` through `028-05` and a phase-local `028-VERIFICATION.md`
artifact.

📌 This test spec is now verification-backed and uses these inputs as the
current source of truth:

- `.planning/phases/028-crypto-audit-storage/028-FUSION.md`
- `.planning/phases/028-crypto-audit-storage/028-CONTEXT.md`
- `.planning/phases/028-crypto-audit-storage/028-01-PLAN.md`
- `.planning/phases/028-crypto-audit-storage/028-02-PLAN.md`
- `.planning/phases/028-crypto-audit-storage/028-03-PLAN.md`
- `.planning/phases/028-crypto-audit-storage/028-04-PLAN.md`
- `.planning/phases/028-crypto-audit-storage/028-05-PLAN.md`
- `.planning/phases/028-crypto-audit-storage/028-01-SUMMARY.md`
- `.planning/phases/028-crypto-audit-storage/028-02-SUMMARY.md`
- `.planning/phases/028-crypto-audit-storage/028-03-SUMMARY.md`
- `.planning/phases/028-crypto-audit-storage/028-04-SUMMARY.md`
- `.planning/phases/028-crypto-audit-storage/028-05-SUMMARY.md`
- `.planning/phases/028-crypto-audit-storage/028-VERIFICATION.md`
- `.planning/REQUIREMENTS.md`
- Existing seams in `crates/z00z_storage/tests/`,
  `crates/z00z_storage/src/assets/store_internal/`,
  `crates/z00z_simulator/tests/`, and
  `crates/z00z_wallets/src/core/claim/nullifier.rs`

📌 This document remains the phase-local test contract for Phase 028 and is now
backed by executed verification evidence.

## Classification

### TDD And Integration Targets

- `crates/z00z_storage/src/checkpoint/artifact.rs`
  because Phase 028 must prove that opaque checkpoint payloads are attestation
  or transport artifacts, not verified finality proofs.
- `crates/z00z_storage/src/checkpoint/build.rs`
  because the checkpoint builder is the trust-boundary seam for
  `TxProofVerifier` and `SpentIndex`.
- `crates/z00z_storage/src/checkpoint/exec_input.rs`
  because canonical replay artifacts must preserve real upstream `tx_proof`
  bytes.
- `crates/z00z_storage/src/checkpoint/ids.rs`
  because draft, final, and exec-input IDs must become type-separated and
  domain-separated.
- `crates/z00z_storage/src/checkpoint/link.rs`
  because checkpoint-link tampering must become detectable from canonical
  bytes or a derived link commitment.
- `crates/z00z_storage/src/assets/proof.rs`
  because `ProofBlob` must bind the semantic root to the backend JMT root.
- `crates/z00z_storage/src/assets/store_internal/proof_help.rs`
  because the proof-emission path must populate the new root-binding field.
- `crates/z00z_storage/src/assets/store.rs`
  because canonical exec creation, claim replay storage, root-mode behavior,
  and store-facing semantics converge there.
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`
  because RedB persistence must store canonical exec bytes, compatibility-safe
  ID or link bytes, and binary nullifier replay state.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs`
  because the simulator is the boundary that must hand storage canonical
  nullifier bytes without creating a crate cycle.
- `crates/z00z_storage/tests/test_checkpoint_finalization.rs`,
  `test_checkpoint_draft_build.rs`, `test_checkpoint_store_api.rs`,
  `test_checkpoint_replay_inputs.rs`, `test_checkpoint_root_binding.rs`,
  `test_checkpoint_ids.rs`, `test_checkpoint_link_injective.rs`,
  `test_redb_rehydrate.rs`, `test_redb_mutation.rs`, and
  `test_claim_source_proof.rs`
  because these are the primary integration anchors for the phase.
- `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` and
  `test_whitebox_state.rs`
  because Phase 028 must preserve crate-local proof and root parity coverage.
- `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs`,
  `test_claim_tx_pipeline.rs`, `test_scenario1_unified_gate.rs`, and
  `test_stage6_checkpoint_final_gate.rs`
  because Phase 028 must prove downstream release-style behavior under realistic
  claim, checkpoint, and storage flows.

### E2E Browser Targets

- None.

📌 Phase 028 end-to-end proof must remain in Rust integration tests and
release-style simulator commands because the phase hardens storage contracts,
artifact semantics, and replay behavior rather than browser surfaces.

### Skip Targets

- Planning markdown files themselves
  because they are specification inputs, not executable logic.
- Vendor code under `crates/z00z_crypto/tari/`
  because the vendor boundary must remain intact.
- Unrelated wallet UI or RPC surfaces
  unless they are required to prove canonical nullifier compatibility.
- A new external checkpoint proving backend
  because Gate `G-06` requires the phase to close without introducing one.

## Existing Test Structure

📌 Phase 028 already has a direct integration seam for each critical storage
contract that the phase intends to harden:

- `crates/z00z_storage/tests/test_checkpoint_finalization.rs`
- `crates/z00z_storage/tests/test_checkpoint_draft_final.rs`
- `crates/z00z_storage/tests/test_checkpoint_draft_build.rs`
- `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
- `crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs`
- `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
- `crates/z00z_storage/tests/test_checkpoint_ids.rs`
- `crates/z00z_storage/tests/test_checkpoint_link_injective.rs`
- `crates/z00z_storage/tests/test_redb_rehydrate.rs`
- `crates/z00z_storage/tests/test_redb_mutation.rs`
- `crates/z00z_storage/tests/test_claim_source_proof.rs`

📌 The repository already has realistic downstream anchors that should be
reused before creating new files:

- `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs`
- `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`
- `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`
- `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`
- `crates/z00z_wallets/src/core/claim/test_nullifier_store.rs`

📌 Proposed new files are acceptable only when extending one of the anchors
above would blur ownership across unrelated assertions. The default Phase 028
strategy is to extend the existing storage and simulator seams first.

## Canonical Test Commands

📌 Every implementation wave should keep the same top-level validation order:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture`
- `cargo test -p z00z_storage --release --test test_checkpoint_draft_final -- --nocapture`
- `cargo test -p z00z_storage --release --test test_checkpoint_draft_build -- --nocapture`
- `cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture`
- `cargo test -p z00z_storage --release --test test_checkpoint_replay_inputs -- --nocapture`
- `cargo test -p z00z_storage --release --test test_checkpoint_root_binding -- --nocapture`
- `cargo test -p z00z_storage --release --test test_checkpoint_ids -- --nocapture`
- `cargo test -p z00z_storage --release --test test_checkpoint_link_injective -- --nocapture`
- `cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture`
- `cargo test -p z00z_storage --release --test test_redb_mutation -- --nocapture`
- `cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage3_nullifier_store -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture`
- `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
- `cargo test --release --features test-fast --features wallet_debug_dump`

📌 Phase 028 also requires two structural regression scans because part of the
phase closes semantic and compatibility drift that can hide behind otherwise
green tests.

- `rg -n 'vec!\[\(index as u8\)\.saturating_add\(1\)\]' crates/z00z_storage/src crates/z00z_storage/tests`
- `rg -n 'nullifier_hex:\s*String|claim_null_key\([^)]*&str' crates/z00z_storage/src crates/z00z_storage/tests crates/z00z_simulator/src crates/z00z_simulator/tests`

📌 Both structural scans are now absence oracles. After `028-02`, the
placeholder `vec![(index as u8).saturating_add(1)]` replay emitter is expected
to be gone, so the verified green outcome is `0` matches. The legacy text-key
nullifier scan also remains green only when it returns `0` matches.

📌 The exact release-style simulator commands above are authoritative for this
spec because the user requires those forms whenever Scenario 1 participates in
the phase gate.

## Plan-To-Proof Coverage Map

| Plan | Must Be Proven | Primary Seams | Primary Test Ownership |
| ---- | ---- | ---- | ---- |
| `028-01` | Opaque checkpoint artifacts are semantically honest, legacy `OPAQUE` reads follow one explicit policy, and the trust boundary is visible | `artifact.rs`, `build.rs`, `store.rs`, `redb_backend.rs` | `test_checkpoint_finalization.rs`, `test_checkpoint_draft_final.rs`, `test_checkpoint_draft_build.rs`, `test_checkpoint_store_api.rs` |
| `028-02` | Canonical `CheckpointExecInput` artifacts preserve real upstream `tx_proof` bytes and no placeholder proof bytes reach persisted exec state | `exec_input.rs`, `build.rs`, `store.rs`, `redb_backend.rs` | `test_checkpoint_replay_inputs.rs`, `test_checkpoint_draft_build.rs`, `test_redb_rehydrate.rs` |
| `028-03` | `ProofBlob` binds semantic and backend roots through one versioned field and `chk_blob(...)` fails closed on mismatch | `assets/proof.rs`, `proof_help.rs` | `test_checkpoint_root_binding.rs`, `test_claim_source_proof.rs`, whitebox proof/state seams |
| `028-04` | Artifact IDs become class-separated and checkpoint links become canonical tamper-detectable bindings with explicit compatibility | `ids.rs`, `link.rs`, `artifact.rs`, `store.rs`, `redb_backend.rs` | `test_checkpoint_ids.rs`, `test_checkpoint_link_injective.rs`, `test_redb_rehydrate.rs` |
| `028-05` | Claim replay protection uses canonical binary nullifier state, migration parity is explicit, and default-path storage hazards are closed | `assets/store.rs`, `redb_backend.rs`, `claim_pkg_consumer.rs` | `test_stage3_nullifier_store.rs`, `test_claim_tx_pipeline.rs`, `test_redb_rehydrate.rs`, `test_redb_mutation.rs`, release-style simulator gates |

## Critical Workflow Journeys

📌 Another engineer should treat the following as the canonical Phase 028
workflow journeys.

1. Opaque checkpoint journey:
   `CheckpointProof::new(...) -> CheckpointDraft::finalize(...) -> store load`
   where opaque payloads remain attestation-only or legacy-only and never imply
   verified finality.
2. Canonical replay journey:
   accepted execution transcript -> `CheckpointExecTx.tx_proof` bytes ->
   `CheckpointExecInput` canonical bytes -> `derive_exec_id(...)` -> RedB
   persistence -> reload -> link validation.
3. Root-binding journey:
   semantic root plus backend root -> `ProofBlob` versioned binding field ->
   `chk_blob(...)` pre-branch verification -> branch acceptance or typed
   rejection.
4. ID and link journey:
   canonical statement bytes -> typed ID derivation -> checkpoint-link
   commitment -> RedB persistence -> reload parity or explicit compatibility
   rejection.
5. Claim nullifier journey:
   upstream claim package -> canonical nullifier bytes -> storage-owned replay
   row -> reserve or reject -> reload parity -> duplicate replay rejection.
6. Scenario 1 checkpoint journey:
   Stage 4 transaction preparation -> Stage 6 bundle proof mode -> Stages 7
   through 12 checkpoint outputs -> checkpoint artifact, link, and audit decode
   -> storage reload -> release-style consistency checks.

## Security And Crypto Invariants

📌 Phase 028 closes storage-boundary integrity invariants rather than adding a
new proving system.

- Opaque checkpoint payloads must never be treated as self-verifying proofs.
- Legacy `OPAQUE` artifacts must follow one explicit read policy.
- Canonical exec artifacts must preserve the exact upstream `tx_proof` bytes
  that were accepted during checkpoint construction.
- Placeholder, index-derived, or synthetic proof bytes must not enter
  canonical exec persistence or exec-ID derivation.
- `ProofBlob` must carry one versioned, domain-separated commitment that binds
  the semantic root to the backend root.
- `chk_blob(...)` must reject cross-root mismatch before accepting definition,
  serial, or asset branch proofs.
- `CheckpointDraftId`, `CheckpointId`, and `CheckpointExecInputId` must not be
  derived from one undifferentiated raw hash namespace.
- `CheckpointLink` tampering must be detectable from canonical bytes or an
  explicit link commitment.
- `CheckpointAudit` and other audit-only wrappers must remain outside canonical
  artifact IDs and link commitments.
- Replay uniqueness must be keyed by canonical binary nullifier bytes and stay
  claim-domain specific.
- `chain_id` may remain validated metadata, but it must not replace canonical
  nullifier-byte scope.
- Any serialized-byte change must have one explicit compatibility, migration,
  or rollback rule.
- Unsupported root-mode and fault-injection behavior must not surprise the
  production-default path through panics or hidden environment-triggered drift.

## Required End-To-End Behaviors

| Behavior | Requirement Or Gate | Primary Path | Assertions | Pass Signal | Fail Signal |
| ---- | ---- | ---- | ---- | ---- | ---- |
| Opaque artifacts are truthfully labeled | `PH28-CHK-PROOF`, `G-01` | `CheckpointProof::new -> CheckpointDraft::finalize -> store load` | opaque payload is attestation-only or legacy-only; no type, test, or store path describes it as verified finality | storage tests and rustdoc-facing semantics agree on opaque meaning | opaque artifact still appears as a verified proof or finality object |
| Legacy `OPAQUE` compatibility is explicit | `PH28-CHK-PROOF`, `G-01A`, `G-07` | persisted old artifact bytes -> decode or reject policy | already-persisted bytes are preserved as legacy, migrated, or rejected with a typed compatibility rule | one explicit legacy result is asserted | old bytes are silently reinterpreted under new semantics |
| External verifier trust boundary is visible | `PH28-TRUST-HOOK`, `G-06` | `build_cp_draft(...)` and store write path | permissive hooks are test-only or explicit opaque mode; production path does not overclaim self-sufficient verification | build/store tests show explicit trust contract | default path still implies in-crate proof sufficiency |
| Canonical exec artifacts preserve real proof bytes | `PH28-EXEC-PROOF`, `G-02` | accepted transcript -> `CheckpointExecInput` -> RedB -> reload | exact `tx_proof` bytes roundtrip unchanged; exec ID derives from canonical bytes; placeholder pattern is absent | replay-input and rehydrate tests prove stable roundtrip | stored exec bytes are synthetic, truncated, or index-derived |
| Proof blobs bind semantic and backend roots | `PH28-ROOT-BIND`, `G-03` | `ProofBlob` decode -> `chk_blob(...)` | correct root pair passes; wrong semantic root or wrong backend root rejects before branch acceptance | root-binding tests prove fail-closed verification | branch proof can pass under mismatched root pair |
| IDs are class-separated and links are tamper-detectable | `PH28-ID-BIND`, `G-04`, `G-07` | statement bytes -> typed IDs -> link commitment -> reload | same payload under different artifact class yields different ID; any tuple-field tamper invalidates link | ids and link tests prove class separation plus tamper rejection | mixed-era or tampered tuples still look canonical |
| Claim replay protection uses canonical binary nullifier state | `PH28-NULLIFIER`, `G-05` | claim package -> storage replay row -> duplicate claim path | canonical binary key is persisted; duplicate replay rejects on bytes, not text normalization | simulator and storage tests prove duplicate rejection | replay key still depends on raw lower-hex text |
| Migration parity for replay rows is explicit | `PH28-NULLIFIER`, `G-05A`, `G-07` | legacy row set -> migrated or parity-checked row set | old text-key rows and new binary-key rows describe the same replay set or the policy explicitly rejects legacy rows | parity or migration test closes with one explicit result | migration silently drops or broadens replay coverage |
| Release-style simulator gate stays honest | `PH28-CHK-PROOF`, `PH28-EXEC-PROOF`, `PH28-ID-BIND` | Scenario 1 release run -> checkpoint files decode -> storage reload | stage outputs exist, artifact plus link plus audit stay consistent, persisted checkpoint files decode, storage reload remains valid | exact release commands stay green and decoded artifacts agree | release run emits inconsistent checkpoint files or loses reload parity |
| Default-path storage hazards are closed | `PH28-TRUST-HOOK`, `G-07` | root-mode load and RedB fault-injection path | unsupported root mode yields typed failure; production default does not honor hidden fault injection unexpectedly | redb tests prove typed behavior and explicit gating | production path still panics or surprises callers through env hooks |

## Mandatory Negative Scenarios

📌 The following rejection scenarios are mandatory. They are phase-closing
coverage, not optional extras.

- Empty checkpoint proof bytes must reject with an explicit proofless error.
- Tampered checkpoint public input must reject without producing a final
  artifact.
- Legacy `OPAQUE` bytes presented under a forbidden compatibility policy must
  reject deterministically.
- Canonical exec creation must reject placeholder or index-derived `tx_proof`
  payloads.
- A valid branch proof paired with the wrong semantic or backend root must
  reject before branch acceptance.
- Two artifact classes with equal canonical payload bytes must not collapse to
  the same ID.
- Changing any one of `checkpoint_id`, `prep_snapshot_id`, or `exec_input_id`
  must invalidate the checkpoint link.
- A binary nullifier replay lookup must reject duplicates even if text
  normalization, letter case, or metadata fields differ.
- Old replay rows that cannot be proven equivalent to binary rows must not be
  silently accepted.
- Unsupported root-mode configuration must not panic in the production-default
  path.
- Fault injection must not remain silently active through the normal production
  backend path.

## Critical Integration Paths

📌 Another engineer should treat these as the canonical integration paths for
Phase 028. If a new test does not anchor to one of these paths, it is probably
secondary regression coverage rather than phase-closing proof.

1. `CheckpointProof::new(...) -> CheckpointDraft::finalize(...) -> derive_checkpoint_id(...)`
2. `build_cp_draft(...) -> CheckpointExecInput -> derive_exec_id(...) -> check_link_ids(...)`
3. `AssetStore::plan_arts or replacement canonical exec path -> RedB write -> RedB reload`
4. `ProofBlob::decode(...) -> chk_blob(...) -> branch proof acceptance or typed rejection`
5. `derive_draft_id(...) or derive_checkpoint_id(...) -> CheckpointLink canonical bytes -> seal_artifact(...)`
6. `claim_pkg_consumer.rs -> apply_claim_ops(...) -> replay table write -> replay table reload`
7. `test_stage3_nullifier_store.rs -> duplicate claim attempt -> conflict or spent path`
8. `scenario_1 release run -> checkpoint artifact/link/audit decode -> storage/post_tx reload`

## Scenario Oracle Rules

📌 Every scenario in this spec must have a machine-checkable pass or fail
oracle.

1. A scenario passes only when it proves both behavior and invariant.
2. A rejection scenario passes only when rejection is explicit and no silent
   fallback artifact, mixed-era row, or partial-success checkpoint remains
   accepted.
3. An opaque-semantics scenario passes only when the public API shape, tests,
   and persisted behavior all agree that opaque does not mean verified.
4. A replay-artifact scenario passes only when exact upstream `tx_proof` bytes
   roundtrip through canonical exec persistence unchanged.
5. A root-binding scenario passes only when semantic/backend-root mismatch is
   rejected before any branch proof can create a false-positive success.
6. An ID or link scenario passes only when class separation or tuple tampering
   is enforced from canonical bytes rather than by incidental helper behavior.
7. A nullifier scenario passes only when replay state is keyed by canonical
   bytes and old/new row parity is proven or explicitly rejected.
8. A hardening scenario passes only when production-default behavior becomes
   typed or explicitly gated rather than panic-prone or env-hook dependent.
9. A release-style scenario passes only when the exact simulator commands stay
   green and decoded artifact outputs remain mutually consistent.

## Test Files To Add Or Extend

### 1. Extend `crates/z00z_storage/tests/test_checkpoint_finalization.rs` and `test_checkpoint_draft_final.rs`

📌 This file must own the store-facing proof for `PH28-CHK-PROOF`, `G-01`, and
`G-01A`.

Tests to implement or tighten:

- `opaque_artifact_is_attestation_only`
  demonstrates that opaque payloads are not treated as verified finality.
  Assertions: public artifact semantics and test names do not overclaim proof
  validity.
- `legacy_opaque_read_policy_is_explicit`
  demonstrates that persisted old bytes are preserved as legacy, migrated, or
  rejected with one typed compatibility result.
- `verified_semantics_require_real_validation_path`
  demonstrates that any proof-system variant claiming stronger semantics is
  rejected unless one real validator path exists.
- `checkpoint_statement_binds_snapshot_and_exec_identity`
  demonstrates that the canonical checkpoint statement includes the identity
  fields required by Phase 028.
- `draft_and_final_class_surfaces_remain_distinct_under_legacy_policy`
  demonstrates that draft bytes, final bytes, and legacy opaque compatibility
  paths stay class-separated and never collapse into one misleading load path.

Pass condition: these files prove truthful opaque semantics, explicit legacy
behavior, and persistent class separation without introducing a new proof
backend.

### 2. Extend `crates/z00z_storage/tests/test_checkpoint_draft_build.rs` and `test_checkpoint_store_api.rs`

📌 These files must own the trust-boundary proof for `PH28-TRUST-HOOK`.

Tests to implement or tighten:

- `production_write_path_exposes_external_verifier_boundary`
  demonstrates that the default write path does not overclaim in-crate proof
  verification.
- `permissive_verifier_is_test_or_opaque_only`
  demonstrates that permissive hooks remain scoped to tests, simulation, or an
  explicitly named opaque-attestation path.
- `store_created_artifact_inherits_truthful_semantics`
  demonstrates that persisted artifacts from the store path inherit the same
  honest proof contract as the in-memory builder.

Pass condition: the default checkpoint build and store path has one explicit
trust contract.

### 3. Extend `crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs`

📌 This file must own the canonical replay proof for `PH28-EXEC-PROOF` and
`G-02`.

Tests to implement or tighten:

- `canonical_exec_preserves_real_tx_proof_bytes`
  demonstrates byte-for-byte roundtrip of accepted `tx_proof` payloads.
- `placeholder_tx_proof_is_rejected_on_canonical_path`
  demonstrates that index-derived or synthetic proof bytes cannot be persisted
  as canonical exec artifacts.
- `exec_id_changes_when_real_proof_bytes_change`
  demonstrates that canonical exec identity follows canonical payload bytes.
- `link_validation_rejects_exec_bytes_mismatch`
  demonstrates that reload-time link checks catch mismatched exec bytes.

Pass condition: no canonical exec path can persist placeholder proof payloads.

### 4. Extend `crates/z00z_storage/tests/test_redb_rehydrate.rs`

📌 This file must prove persisted compatibility, reload parity, and mixed-era
rejection for `028-02`, `028-04`, and `028-05`.

Tests to implement or tighten:

- `rehydrate_preserves_canonical_exec_identity`
  demonstrates that persisted exec bytes and exec ID survive reload.
- `rehydrate_rejects_mixed_id_and_link_eras`
  demonstrates that old-ID plus new-link or new-ID plus old-link tuples do not
  pass silently.
- `nullifier_row_migration_parity_holds`
  demonstrates that legacy text rows and binary rows describe the same replay
  set or produce one explicit migration result.
- `proof_blob_compatibility_policy_is_explicit_after_root_binding_change`
  demonstrates that proof-format evolution has one explicit compatibility path.

Pass condition: RedB reload proves parity or explicit compatibility policy for
every serialized-byte change introduced by the phase.

### 5. Extend `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`

📌 This file must own the fail-closed root-binding proof for `PH28-ROOT-BIND`
and `G-03`.

Tests to implement or tighten:

- `semantic_backend_binding_mismatch_rejects`
  demonstrates typed rejection when the binding field does not match the root
  pair.
- `valid_branch_wrong_semantic_root_rejects_before_acceptance`
  demonstrates failure before branch proof acceptance.
- `valid_branch_wrong_backend_root_rejects_before_acceptance`
  demonstrates the symmetric failure case.
- `binding_field_is_versioned`
  demonstrates that the binding domain changes explicitly with proof-format
  versioning.

Pass condition: cross-root tampering cannot produce a false-positive success.

### 6. Extend `crates/z00z_storage/tests/test_claim_source_proof.rs` and the whitebox proof/state seams

📌 These seams must prove compatibility under the new `ProofBlob` contract.

Tests to implement or tighten:

- `claim_source_proof_roundtrips_with_root_binding`
  demonstrates that storage-owned proof export remains valid and decodable.
- `whitebox_proof_helpers_emit_binding_field`
  demonstrates that internal proof emission always populates the new field.
- `root_mode_parity_stays_green_after_proof_format_change`
  demonstrates that root-mode parity coverage remains intact.

Pass condition: stronger root-binding semantics do not break storage-owned proof
exports or whitebox parity anchors.

### 7. Extend `crates/z00z_storage/tests/test_checkpoint_ids.rs` and `test_checkpoint_link_injective.rs`

📌 These files must own `PH28-ID-BIND` and `G-04`.

Tests to implement or tighten:

- `same_payload_different_artifact_class_yields_different_id`
  demonstrates class-separated ID derivation.
- `checkpoint_link_detects_tuple_tamper`
  demonstrates that changing any bound ID invalidates the canonical link.
- `audit_metadata_does_not_change_artifact_id_or_link`
  demonstrates that `CheckpointAudit` remains outside canonical commitments.
- `legacy_or_mixed_link_era_requires_explicit_policy`
  demonstrates that compatibility does not happen silently.

Pass condition: checkpoint identities and links harden together and remain
compatible only through one explicit rule.

### 8. Extend `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs` and `test_claim_tx_pipeline.rs`

📌 These files must own the downstream replay proof for `PH28-NULLIFIER`,
`G-05`, and `G-05A`.

Tests to implement or tighten:

- `simulator_hands_storage_canonical_nullifier_bytes`
  demonstrates that the simulator boundary no longer hands storage raw text
  keys.
- `duplicate_claim_rejects_on_binary_nullifier_key`
  demonstrates duplicate replay rejection under canonical bytes.
- `chain_id_is_consistency_metadata_not_scope_key`
  demonstrates that metadata mismatch does not redefine replay-key scope.
- `legacy_text_rows_and_binary_rows_have_parity`
  demonstrates migration equivalence or one explicit rejection policy.

Pass condition: downstream claim flows prove canonical binary replay state under
realistic claim-package handling.

### 9. Extend `crates/z00z_storage/tests/test_redb_mutation.rs`

📌 This file must own the default-path hardening proof for the closeout of
`028-05` and `G-07`.

Tests to implement or tighten:

- `unsupported_root_mode_is_typed_not_panicking`
  demonstrates that unsupported root-mode selection does not panic in the
  default path.
- `fault_injection_is_not_silent_in_production_default_path`
  demonstrates that the fault-injection hook is test-only or explicitly gated.
- `reload_and_mutation_gates_close_after_nullifier_migration`
  demonstrates that backend hardening and migration parity stay green together.

Pass condition: the production-default backend no longer depends on hidden env
hazards.

### 10. Extend `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs` and `test_stage6_checkpoint_final_gate.rs`

📌 These files must own the release-style E2E proof that Phase 028 remains
truthful under a realistic Scenario 1 checkpoint flow.

Tests to implement or tighten:

- `release_gate_decodes_checkpoint_artifact_link_and_audit`
  demonstrates that the emitted files decode and agree on checkpoint identity.
- `release_gate_preserves_canonical_exec_and_checkpoint_files`
  demonstrates that post-transaction storage contains the expected checkpoint,
  exec-input, and artifact files.
- `release_gate_survives_truthful_opaque_semantics`
  demonstrates that release outputs remain green after semantic hardening.
- `release_gate_survives_binary_nullifier_and_reload_parity`
  demonstrates that replay migration and reload parity remain compatible with
  the release workflow.

Pass condition: the exact release-style simulator commands remain green and the
decoded checkpoint outputs stay mutually consistent.

### 11. Add focused new files only if ownership would otherwise blur

📌 New files are allowed only if the extensions above would mix too many
independent assertions into one existing anchor.

Acceptable focused additions are:

- `crates/z00z_storage/tests/test_checkpoint_semantics_phase028.rs`
  for compatibility-era and truthful-semantics cases that do not fit cleanly
  into finalization coverage.
- `crates/z00z_storage/tests/test_claim_nullifier_phase028.rs`
  for storage-side migration or parity cases that would otherwise overload the
  simulator anchors.

Pass condition: any new file must own one tight slice of Phase 028 behavior and
must not duplicate existing anchor intent.

## Realistic Examples To Demonstrate Successful Execution

📌 Examples are required for Phase 028 because the phase changes how storage
artifacts should be interpreted, not only which inputs reject.

1. A truthful opaque checkpoint example in storage tests.
   Demonstrates: an artifact may finalize and persist while still being
   classified as opaque-attestation or legacy-opaque rather than verified
   finality.
2. A canonical replay transcript example with at least two transactions whose
   `tx_proof` bytes differ.
   Demonstrates: canonical exec bytes preserve the real per-transaction proof
   payloads and exec ID changes when those payloads change.
3. A proof-blob root-binding example.
   Demonstrates: the same branch proof succeeds only when paired with the
   correct semantic-root and backend-root binding.
4. A binary nullifier migration example.
   Demonstrates: a legacy replay row set and a binary replay row set resolve to
   the same duplicate-rejection behavior.
5. A Scenario 1 release-style checkpoint example.
   Demonstrates: the exact simulator commands emit decodable checkpoint files,
   reloadable storage artifacts, and mutually consistent checkpoint identity.

## Measurable Success Conditions

📌 Phase 028 test execution is complete only when all of the following are
true.

- All targeted storage integration tests listed in the canonical command block
  pass in release mode.
- All targeted simulator integration tests listed in the canonical command
  block pass in release mode with `test-fast` and `wallet_debug_dump`.
- The exact Scenario 1 release run command completes successfully.
- The full release-style simulator test command completes successfully.
- The full workspace release-style gate completes successfully.
- The placeholder-proof source scan returns no matches on canonical storage
  paths.
- The text-key nullifier scan returns no matches on canonical storage replay
  paths.
- Every gate from `G-01` through `G-07` has at least one explicit owning test
  result.

## Measurable Failure Conditions

🚨 Phase 028 test execution must be treated as failed if any of the following
remain true.

- Any test still describes opaque checkpoint payloads as verified proofs.
- Any canonical exec path still persists placeholder or synthetic proof bytes.
- Any root-binding mismatch can pass branch verification.
- Any artifact-class collision or link tamper can survive canonical validation.
- Any binary nullifier migration changes the effective replay set without one
  explicit policy result.
- Any production-default path still panics or silently honors hidden fault
  injection.
- Any exact release-style simulator command required by this spec stops passing.
