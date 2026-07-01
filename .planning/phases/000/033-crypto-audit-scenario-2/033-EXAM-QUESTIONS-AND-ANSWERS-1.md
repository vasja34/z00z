# Phase Entrance Exam

**Phase:** `032-crypto-audit-scenario-1`
**Generated:** `2026-04-06`
**Scope Sources:** `032-CONTEXT.md`, `032-TODO.md`, `032-TEST-SPEC.md`, `032-VERIFICATION.md`, `032-VALIDATION.md`, `032-HONEST-CLOSEOUT.md`, `032-07-SUMMARY.md`, `docs/code-review/032-scenario-1-crypto-status.md`, and live repository code and tests for the Scenario 1 claim, spend, checkpoint, and simulator boundaries

## MUST

1. Every final answer in this document MUST be independently re-checked through
   the `doublecheck` skill before it is accepted as final.
2. Every answer MUST be a repository-backed proof system, using factual,
   mathematical, cryptographic, and logical proof where applicable.
3. If a proof cannot be closed, the answer MUST state exactly what evidence,
   artifact, mathematical argument, cryptographic assumption, or repository
   behavior is missing.
4. Every answer MUST stay tied to the live codebase, tests, logs, manifests,
   and phase artifacts for this repository.
5. Every answer in this document MUST function as a verification exam of the
   correct implementation of this phase, not as freeform commentary.
6. If answering a question reveals a real bug, gap, or overclaim, the answer
   MUST name it explicitly and state the remediation path.
7. This file is generated as a question sheet. The `Ans:` sections MUST remain
   blank until a later agent or model fills them.

## 🎯 Challenge

Pressure-test whether Phase 032 actually closed the Scenario 1 crypto-audit
seams across the full chain `Alice -> leaf build -> JMT publish -> Bob scan ->
spend -> validator`, while preserving honest language about what remains
partial, placeholder-scoped, theft-sensitive, or still outside the delivered
trust boundary.

## ⛔ Constraints

- The questions test implementation truth, not planning intent alone.
- The solver must discover the evidence path independently from the repository.
- The wording must stay adversarial and specific without embedding file-path,
  helper-name, test-name, requirement-id, or stage-label breadcrumbs.
- A negative or partial answer is acceptable only when the missing proof is
  precisely named and evidence-backed.

## Scope Note

This exam verifies the live Scenario 1 claim path, spend boundary, checkpoint
acceptance path, semantic-freeze seams, secret-handling boundary, and the
honesty of the phase closeout language. It is also intended to expose where the
phase delivered a narrower current-stack guarantee instead of fully closing the
broader claim-trust and spend-verifier ambitions carried by the planning set.
It explicitly asks whether any actor can steal Bob's coin before publication,
after publication, after scanning, or during later verifier and checkpoint
handoffs, and whether the cryptography for the whole chain is genuinely closed
end to end or only partially so.

## 🔍 Answering Standard

- Answers must discover their own evidence path through the repository.
- Answers must distinguish delivered guarantees, local-rule guarantees,
  partially closed requirements, and documentation-only corrections.
- Answers must use code, tests, logs, manifests, and phase artifacts together
  when one source alone would hide a trust-boundary gap or overclaim.

## 🔐 Theme 1: Claim Authenticity And Source Authority

### 1. Signed Root Or Unsigned Baggage

🔴 **Quest:** What exact accepted claim statement is now authenticated by the authority signature, and what repository evidence proves that the authoritative source root is inside that signed statement rather than appended after the fact?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The accepted authority-authenticated statement is the canonical `ClaimStmtV2` rebuilt from one verified claim package. Its signed tuple is `chain_id`, `root_ver`, `proof_ver`, `tx_ver`, `range_ctx_hash`, `claim_id`, `claim_source_asset_id`, `claim_source_commitment`, `source_root`, `claim_scope_hash`, `recipient_binding`, `nullifier`, `owner_bind_digest`, and `output_leaf_hashes`. The authoritative source root is inside that signed statement, not appended after signature creation, because the signature is computed over the hash of `ClaimStmtV2::to_bytes()`, and `to_bytes()` serializes `source_root` in-band before hashing.

**Evidence Trail:**

1. `crates/z00z_crypto/src/claim/v2.rs` defines `ClaimStmtV2`, serializes `source_root` inside `to_bytes()`, hashes the full statement with `claim_stmt_hash_v2(...)`, and signs or verifies that hash in `ClaimAuthoritySigV2::sign(...)` and `ClaimAuthoritySigV2::verify(...)`.
2. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` rebuilds the accepted `ClaimStmtV2`, fills `source_root`, and rejects any mismatch between `stmt.source_root()`, `proof.source_root()`, and the proof-blob root.
3. `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` refuses to sign the package if `stmt.source_root()` does not equal the storage-owned proof root.
4. `crates/z00z_crypto/tests/test_claim_v2_contract.rs` proves the frame layout of the signed statement, including in-band `source_root`, and proves that mutating the signed statement breaks signature verification.
5. `crates/z00z_storage/tests/test_claim_source_proof.rs` and `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` prove that the proof root equals the storage authoritative root and that stale proof or wrong-signature packages fail closed.

**Reasoning:**

- The authority signature does not sign a loose JSON package or a proof blob separately; it signs `claim_stmt_hash_v2(stmt)`, where `stmt` is the canonical `ClaimStmtV2` value.
- Because `ClaimStmtV2::to_bytes()` includes `source_root` before hashing, `source_root` is part of the authenticated message.
- The accepted build and verify paths also cross-check `stmt.source_root == proof.source_root == proof_blob.root`, so the repository rejects any split-brain layout where the signature covers one statement while the proof carries another root.
- The tempting but false reading is that `source_root` lives only in `claim_source_proof` and is merely attached next to the signature later. That reading fails because the statement type itself stores `source_root`, the statement serializer emits it, the signature hashes that emitted byte sequence, and the stage-3 builder will not sign when the statement root and proof root diverge.

**Gap Or Blocker:** None

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The repository proves this by serialization and hash/sign contract plus root-consistency checks; it does not include a dedicated test that flips `source_root` after signing, but the in-band frame test and hash/sign path close that proof.

### 2. Full Tuple Or Partial Story

🔴 **Quest:** Beyond the source root itself, which claim fields are cryptographically bound into the accepted claim tuple, and does any accepted path still allow asset scope, chain binding, or version semantics to drift without invalidating the signature contract?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The accepted authority-signed tuple binds every field of the canonical `ClaimStmtV2`: `chain_id`, `root_ver`, `proof_ver`, `tx_ver`, `range_ctx_hash`, `claim_id`, `claim_source_asset_id`, `claim_source_commitment`, `source_root`, `claim_scope_hash`, `recipient_binding`, `nullifier`, `owner_bind_digest`, and `output_leaf_hashes`. In the live accepted verifier path, drift of the actual source asset, source commitment, numeric chain binding, scope hash, or version bytes does not survive verification: those mutations are re-derived and rejected before or at the authority-signature check. The honest residual seam is narrower and sits outside that tuple: textual package anchor metadata such as `chain_type` and `chain_name` are not part of the authority-signed claim statement, although simulator consumer flows add a separate anchor gate for them.

**Evidence Trail:**

1. `crates/z00z_crypto/src/claim/v2.rs` defines the full `ClaimStmtV2` field set, serializes all of it in `to_bytes()`, and signs or verifies `claim_stmt_hash_v2(stmt)` through `ClaimAuthoritySigV2`.
2. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` rebuilds the canonical statement from package material, re-derives `nullifier`, recomputes `claim_scope_hash`, recomputes `owner_bind_digest`, reconstructs `output_leaf_hashes`, checks source-proof and source-root consistency, and only then verifies the authority signature against that rebuilt statement.
3. `crates/z00z_wallets/src/core/tx/claim_helpers.rs`, `crates/z00z_wallets/src/core/claim/nullifier.rs`, and `crates/z00z_wallets/src/core/tx/claim_tx.rs` show that numeric chain binding is folded into the scope hash, the nullifier, and the owner-attestation message.
4. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl.rs` and `crates/z00z_wallets/src/core/tx/claim_wire_types.rs` show that package version is fixed to `CLAIM_PKG`, while `claim_tx_verifier_impl_proof.rs` hard-pins `root_ver` and `proof_ver` to `V1` in the rebuilt statement and cross-checks them against the proof header.
5. `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` proves several negative paths directly: bad version rejects as `claim_structure_invalid`, bad scope hash rejects as `claim_structure_invalid`, source-commitment drift rejects as `claim_proof_invalid`, proof-root mix rejects as `claim_proof_invalid`, bad authority bytes reject as `claim_authority_invalid`, and scope hashes differ across chain IDs.
6. `crates/z00z_wallets/src/core/tx/claim_auth.rs` and `crates/z00z_simulator/src/claim_pkg_consumer.rs` show that `chain_type` and `chain_name` are not authority-signed tuple members, but consumer flows additionally enforce a simulator-only anchor over `chain_id`, `chain_type`, and `chain_name` before accepting authoritative claim packages.

**Reasoning:**

- The signature contract is not a loose JSON envelope. The verifier rebuilds one canonical `ClaimStmtV2` and checks the authority signature against that exact typed tuple.
- Asset scope is bound by multiple tuple members, not just `source_root`: the verifier couples `claim_source_asset_id`, `claim_source_commitment`, the reconstructed `source_root`, `range_ctx_hash`, and `output_leaf_hashes` to the same signed statement and fails closed when those relationships drift.
- Numeric chain binding is also multi-coupled. `chain_id` is itself signed, and the verifier re-derives chain-sensitive values such as `claim_scope_hash`, `nullifier`, and the owner-attestation bind digest from that same `chain_id`.
- Version semantics are not floating metadata in the accepted path. `tx_ver` is signed and must equal the supported package version, while `root_ver` and `proof_ver` are verifier-pinned and then checked against the proof header.
- The remaining honest caveat is that not every chain-facing label is part of the authority signature. `chain_type` and `chain_name` live outside `ClaimStmtV2`; they are guarded in consumer flows by a separate simulator-anchor rule rather than by the authority-signature contract itself.

**Gap Or Blocker:** The repository does not surface a dedicated targeted negative test that mutates `claim_source_asset_id` or rewrites `chain_id` after signing and then asserts the exact reject class at the end-to-end verifier boundary. The code logic closes those drifts, but that last test-level proof is missing, so the answer stays `Partial Evidence` rather than `Full Evidence`.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status comes from missing targeted negative tests for some drift cases, not from an observed accepted-path bypass of the tuple-bound fields.

### 3. Authoritative Store Or Local Reconstruction

🔴 **Quest:** Where does the live claim flow still depend on a locally rebuilt membership picture rather than persisted storage-backed continuity, and why is that difference strong enough to keep the broader trust claim only partially closed?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The live claim flow still depends on locally rebuilt membership in two places. The generic wallet-side accepted verifier reconstructs `source_root` from the package leaf by creating a fresh one-item store in memory, and the storage helper used by Stage 3 production and simulator consumption also derives root and proof from an off-backend one-item store. So the current tree proves one canonical synthetic-contract semantics across the accepted claim flow, but it does not yet prove persisted storage-backed membership continuity for the broader original trust claim. That difference is strong enough to keep `PH32-CLAIM-TRUST` only partially closed, because the accepted package is still being validated against a deterministic local reconstruction contract rather than a surviving persisted-membership contract.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` shows that `derive_claim_source_root(...)` converts the package leaf into a `StoreItem`, builds a fresh `AssetStore`, inserts that one item, and derives the proof root from that synthetic store.
2. `crates/z00z_storage/src/assets/store_internal/store_query.rs` shows that `AssetStore::claim_source_contract_for_item(...)` also builds an off-backend store, inserts exactly one item, and derives both `claim_source_root()` and `claim_source_proof(...)` from that synthetic contract.
3. `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` uses that helper during claim-package emission, while `crates/z00z_simulator/src/claim_pkg_consumer.rs` re-checks packages against the same helper-owned contract plus the simulator authority anchor.
4. `crates/z00z_storage/tests/test_claim_source_proof.rs` and `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` prove helper roundtrip consistency and fail-closed rejection of stale proof, wrong anchor, and wrong signature, but those tests still operate on the helper-derived contract rather than on persisted continuity.
5. `032-03-SUMMARY.md`, `032-VALIDATION.md`, `032-HONEST-CLOSEOUT.md`, `docs/code-review/032-scenario-1-crypto-status.md`, and `032-FULL-AUDIT.md` all explicitly record that the broader original `PH32-CLAIM-TRUST` wording stays partial because the current helper still re-derives a synthetic one-item store contract instead of proving persisted storage-backed continuity.

**Reasoning:**

- Phase 032 did remove the old split-brain situation where Stage 3 could mint a simulator-local proof meaning and the consumer could read something else. Producer and consumer now converge on one shared synthetic-contract seam.
- But the accepted trust surface is still narrower than “persisted storage-backed continuity.” The package leaf can be turned into a locally consistent root/proof picture without consulting a durable membership store.
- The wallet verifier mirrors the same reconstruction semantics directly instead of asking an already-persisted authoritative store for proof continuity.
- That is enough to prove deterministic helper consistency and to reject stale or forged packages relative to that helper contract.
- It is not enough to honestly claim that Scenario 1 already proves the broader original requirement that accepted claim packages are anchored in persisted storage-backed membership continuity.

**Gap Or Blocker:** The repository still lacks a persisted-membership-backed claim-source contract for the accepted claim flow. Until the helper derives root/proof data from persisted store-backed state, or the original requirement is formally narrowed to the current synthetic-contract boundary, the broader trust claim cannot be closed honestly.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status comes from the still-open persisted-continuity gap and from keeping the wording narrower than any unproven “published historical JMT” claim.

### 4. Precise Reject Semantics

🔴 **Quest:** Which negative paths prove that forged root material, forged proof material, forged authority bytes, and claim-tuple drift fail closed as distinct integrity failures instead of collapsing into one vague mismatch outcome?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository strongly proves that tested integrity failures do not collapse into one vague mismatch outcome. At the low-level claim contract, mutated signed tuples, root-version drift, proof-version drift, and source-root drift are separated explicitly. At the accepted verifier boundary, tested proof/root corruption lands in the proof-integrity family, forged authority bytes land in the authority family, nullifier drift lands in the nullifier family, output-binding drift lands in the output family, and structure drift lands in the structure family. The verifier also preserves precedence, so an early proof or fee failure is not later overwritten by a digest mismatch. The answer stays partial because not every proof-version/root-version verifier branch is closed by a dedicated end-to-end test, and simulator stale-proof rejection is asserted with a slightly looser matcher.

**Evidence Trail:**

1. `crates/z00z_crypto/tests/test_claim_v2_contract.rs` proves contract-level distinctions directly: mutated signed tuple returns `ClaimV2Err::SigInvalid`, root-version drift returns `ClaimV2Err::RootVerMix`, proof-version drift returns `ClaimV2Err::ProofVerMix`, and source-root drift returns `ClaimV2Err::SourceRootMix`.
2. `crates/z00z_wallets/src/core/tx/claim_errors.rs` defines distinct verifier-side error families such as `StructureMalformed`, `VersionUnsupported`, `SourceProofMismatch`, `AuthoritySigInvalid`, `NullifierMismatch`, output-binding errors, and `DigestMismatch`.
3. `crates/z00z_wallets/src/core/tx/claim_tx.rs` maps those families into distinct public reject classes: `claim_proof_invalid`, `claim_authority_invalid`, `claim_nullifier_invalid`, `claim_output_invalid`, `claim_fee_invalid`, and `claim_structure_invalid`.
4. `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` proves concrete tested classifier behavior: bad version rejects as `claim_structure_invalid`, bad scope hash rejects as `claim_structure_invalid`, nullifier mismatch rejects as `claim_nullifier_invalid`, proof-root mix rejects as `claim_proof_invalid`, source-commitment drift rejects as `claim_proof_invalid`, bad authority bytes reject as `claim_authority_invalid`, and owner-binding or nonce drift rejects as `claim_output_invalid`.
5. The same test file proves precedence rather than vague collapse: proof corruption beats digest mismatch, fee corruption beats digest mismatch, and authority-stage failure stops later stages before owner-attest or digest checks run.
6. `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` adds consumer-level distinction: wrong authority signature is rejected through the authority-invalid lane, wrong authority anchor is rejected separately, and stale storage-proof corruption is rejected fail closed without being accepted as a normal package.

**Reasoning:**

- The low-level `claim_v2` contract keeps precise failure causes instead of flattening them into one generic corruption result.
- The accepted wallet verifier intentionally groups some root/proof failures into the proof-integrity family, but it still keeps that family separate from authority corruption, nullifier corruption, output-binding corruption, and structure corruption.
- That means the repository preserves semantically useful reject classes. Forged authority bytes are not mislabeled as structure errors, proof corruption is not mislabeled as digest drift, and tested tuple drifts land in the family that actually broke.
- The precedence tests matter because they show the verifier does not overwrite a more informative early integrity failure with a later `tx_digest_hex` mismatch.

**Gap Or Blocker:** The accepted verifier does have explicit branches for source-root-version and source-proof-version failures, but the repository does not currently surface dedicated end-to-end tests that pin those exact verifier-side branches. The simulator stale-proof test also allows several fail-closed substrings rather than one stable classifier string.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status reflects missing targeted end-to-end tests for some version-specific verifier branches, not a collapse of the tested failure families into one vague mismatch.

### 5. Publish-Bound Claim Continuity

🔴 **Quest:** Across the `Alice -> leaf build -> JMT publish` portion of Scenario 1, what proves that the published claim package preserves one canonical authority-bound truth surface instead of allowing a hardened lane and an older placeholder-compatible lane to coexist silently?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository strongly proves one canonical accepted Scenario 1 claim-package lane at the package-payload and publish/load gate levels. Stage 3 emits the hardened `TxPackage` / `claim_tx` / `claim_tx_v1` / `claim_source` payload with real proof bytes and a real authority signature, and both write-time and load-time consumption run that payload back through the same verification and authoritative-anchor checks. The old placeholder proof/signature stubs are no longer emitted and are rejected if injected. The answer stays partial because the persisted bundle wrapper version is not enforced on load, some discriminator fields can arrive through serde defaults rather than explicit file presence, and replay/nullifier tests are useful publication guards but are not direct proof that a placeholder-compatible lane no longer exists.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` builds the canonical claim package payload with `tx_type = claim_tx_v1`, `proof_type = claim_source`, real proof bytes, and a real authority signature over the rebuilt claim statement.
2. `crates/z00z_simulator/src/claim_pkg_consumer.rs` defines the canonical persisted bundle envelope, serializes and loads package rows through one path, verifies each package with `ClaimTxVerifierImpl`, and then applies authoritative anchor and source-root/proof checks before accepting claim packages.
3. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl.rs` and `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` prove that bad kind, bad version, bad proof type, legacy proof stubs, and legacy signature stubs are rejected fail closed at the accepted verifier boundary.
4. `crates/z00z_simulator/tests/test_claim_emit.rs` proves Stage 3 no longer emits the old placeholder proof and signature stub bytes.
5. `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` proves that wrong simulator anchor, stale storage proof, and wrong authority signature are rejected on consumption.

**Reasoning:**

- A silent coexistence problem would require either producer-side emission of placeholder-compatible claim artifacts or consumer-side acceptance of them.
- The repository closes both halves for the tested accepted path: producer tests prove the old placeholder stubs are no longer emitted, and verifier/consumer tests prove those non-canonical forms are rejected when injected.
- The publish/load path is also not a free byte dump. Stage 3 writes packages through a guarded bundle path, and load-time consumption re-verifies the package payload before treating it as accepted truth.
- That is strong evidence for one canonical accepted authority-bound package lane in the current tree.

**Gap Or Blocker:** The loader does not currently enforce bundle-wrapper `version`, and some canonical discriminator values can be supplied by serde defaults rather than by explicit serialized field presence. Those facts do not reopen the old placeholder proof/signature lane, but they are enough to keep the answer from claiming fully exhaustive closure of every possible serialization-side ambiguity.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: replay/nullifier publication tests strengthen the write gate, but they were not used here as the main proof that the legacy placeholder lane is gone.

## 🧭 Theme 2: Ownership Truth And Semantic Freeze

### 6. Sender Knowledge Versus Anti-Theft

🔴 **Quest:** What live repository evidence disproves the old sender-ignorance story, and what stronger evidence shows that the anti-theft boundary now depends on an additional receiver-held secret rather than on the sender being unable to derive output-side material?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The old sender-ignorance story is directly disproved by live accepted-flow code. The canonical output builder derives sender-side `k_dh`, then derives `s_out`, and only then encrypts the output pack, so Scenario 1 cannot honestly claim that the sender is cut off from output-secret material. The stronger live replacement is a narrower two-factor ownership story: wallet-local ownership requires `receiver_secret` plus the output secret material, and the spend-rule / witness-prep layers re-derive receiver-bound owner/view material before they accept the owner-tag and asset-secret relations. The answer stays partial because the repository does not yet prove that same receiver-secret-gated two-factor exclusion as a finished validator-level trustless boundary; Phase 032 explicitly keeps that stronger public-verifier claim open.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/stealth/output_build.rs` makes the sender-side semantics explicit in both comment and code: the accepted build path derives `k_dh`, then computes `s_out = derive_s_out(&build_mat.k_dh, &build_mat.r_pub, serial_id)`, and only then encrypts the asset pack.
2. `crates/z00z_simulator/src/scenario_1/stage_3.rs` mirrors the same accepted-flow semantics at Scenario 1 runtime by deriving `k_dh`, recomputing `owner_tag`, deriving `s_out`, and rebuilding the encrypted pack from that sender-visible material.
3. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` freezes the same honesty boundary explicitly: Phase 032 must not claim sender ignorance of `s_out`, and the truthful anti-theft statement is instead receiver-secret-gated wallet-local ownership.
4. `crates/z00z_wallets/src/core/stealth/output.rs` defines that wallet-local two-factor rule in live code: `verify_owner_two_factor(...)` derives `owner_handle` and `view_sk` from `receiver_secret`, recomputes `k_dh`, rechecks `owner_tag`, and then re-derives the expected `s_out` before accepting ownership.
5. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves the distinction operationally in `scenario1_wallet_local_two_factor_ownership_needs_receiver_secret_and_s_out`: the real receiver secret passes, sender material fails even with the same `s_out`, and a tampered `s_out` also fails.
6. `crates/z00z_wallets/src/core/tx/spend_rules.rs` strengthens the same model at the spend-rule layer: `receiver_secret` is declared as witness material for owner-handle and view-key derivation, and `verify_spend_rules(...)` re-derives receiver-bound owner/view material, recomputes `k_in`, then checks `owner_tag` and the asset-secret relation.
7. `crates/z00z_wallets/src/core/tx/witness_gate.rs` shows the accepted spend-witness lane also depends on `receiver_secret` when it decrypts the input pack and constructs receiver keys before building the current public spend contract.
8. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` and `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md` keep the caveat explicit: Scenario 1 does not yet prove an end-to-end trustless public verifier for every wallet-local receiver-secret rule or two-factor ownership invariant.

**Reasoning:**

- The sender-ignorance story fails on direct code inspection, because the sender-side constructor itself computes the output-side secret material needed to populate the encrypted pack.
- The live repository therefore cannot honestly explain theft-resistance as “Alice cannot spend because she never knows `s_out`.”
- The accepted replacement story is stronger and more precise: ownership is modeled as a receiver-secret-gated two-factor relation. At the wallet-local layer this is `receiver_secret + s_out`; at the spend-rule / witness-prep layer it is receiver-derived owner/view material plus the input secret relation.
- Those layers are real and tested, so the repository does show that sender knowledge of output-side material alone is intentionally insufficient.
- But the current public spend contract still does not prove the full two-factor exclusion end to end at the final validator-level trustless boundary, so the repository must keep the stronger anti-theft claim narrower than the old narrative it replaces.

**Gap Or Blocker:** The repository proves receiver-secret-gated two-factor ownership at wallet-local, spend-rule, and spend-witness-preparation seams, but it does not yet prove that same exclusion as a completed validator-level trustless public-verifier invariant. Until that boundary is delivered, the honest answer is that sender ignorance is disproved strongly, while the final anti-theft statement remains only partially closed.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status comes from the still-open validator-level proof boundary, not from any surviving evidence that the sender is ignorant of `s_out` in the accepted flow.

### 7. Canonical Output-Secret Semantics

🔴 **Quest:** Which output-secret derivation model is actually canonical in the current tree, and what evidence proves that this semantics is frozen consistently across construction, scanning, ownership checks, and scenario execution?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** On the accepted live implementation surface, the canonical output-secret model is deterministic `derive_s_out(k_dh, r_pub, serial_id)`, not a sender-random `random32` model. That semantics is frozen coherently across the live builder, sender self-validation, receiver scan materialization, wallet-local ownership checks, and Scenario 1 runtime/e2e execution. The answer stays partial because the repository still contains legacy temp design artifacts that describe a competing `random32` model, so it is not yet honest to say every document-only surface in the tree already agrees with the accepted live code.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/stealth/ecdh.rs` defines `derive_s_out(k_dh, r_pub, serial_id)` as the single canonical implementation and explicitly says that formula must not be duplicated with competing semantics.
2. `crates/z00z_wallets/src/core/stealth/output_build.rs` uses that exact formula in the accepted output constructor when it computes `s_out` before encrypting the asset pack.
3. `crates/z00z_wallets/src/core/stealth/output_validator.rs` re-derives `exp_s_out` from the same `(k_dh, r_pub, serial_id)` tuple and rejects the built output if the decrypted pack secret does not match.
4. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` keeps the same semantics on the receive side by materializing `asset_secret: Some(pack.s_out)` after a successful scan instead of translating the pack secret into a different output-secret model.
5. `crates/z00z_wallets/src/core/stealth/output.rs` preserves the same rule at the wallet-local ownership seam: `verify_owner_two_factor(...)` re-derives the expected `s_out` from the recomputed `k_dh`, `r_pub`, and `serial_id` and rejects mismatches.
6. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves the frozen wallet-local rule still depends on the re-derived `s_out`, while `crates/z00z_simulator/src/scenario_1/stage_3.rs` mirrors the same deterministic derivation during Scenario 1 runtime output handling.
7. `crates/z00z_simulator/tests/test_e2e_phase4.rs` proves runtime and canonical scan parity at Stage 3 and Stage 4 by asserting `wallet_output.asset_secret == pack.s_out`, and `crates/z00z_simulator/tests/test_stage4_output_crypto.rs` proves wrong receiver-secret ownership checks fail instead of silently accepting a different output-secret interpretation.
8. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md`, `032-01-SUMMARY.md`, and `032-VALIDATION.md` freeze and validate the accepted semantics explicitly: `s_out` is part of the encrypted pack, current output construction derives it from sender-side material, the semantic freeze is canonical for Scenario 1, and the regression suite is recorded green for keeping that contract stable and test-backed.

**Reasoning:**

- The canonical live formula is not inferred indirectly; it is declared once in the KDF layer and then reused by every accepted seam that matters for Scenario 1 output handling.
- Construction and sender self-validation agree on the same deterministic `s_out` derivation.
- Receiver scan does not reinterpret the output secret; it surfaces the decrypted `pack.s_out` directly as the wallet-visible asset secret.
- Wallet-local ownership checks re-derive the same value again, so the accepted ownership story depends on one stable semantic rather than on builder-only convention.
- Scenario runtime and end-to-end tests then confirm that Stage 3 emission, Stage 4 runtime reception, and wallet scan parity all preserve that same `s_out` meaning.

**Gap Or Blocker:** The accepted live tree is internally consistent, but the repository still carries legacy temp design notes that describe a `random32` output-secret model. Until those historical document-only artifacts are removed, rewritten, or fenced as obsolete, the honest statement is that deterministic `derive_s_out(k_dh, r_pub, serial_id)` is canonical on the accepted live implementation surface, not yet unanimously across every residual artifact in the repository.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status reflects legacy document drift, not a live-code disagreement across the accepted construction, scan, ownership, and Scenario 1 runtime surfaces.

### 8. Associated-Data Identity Freeze

🔴 **Quest:** Where does the accepted flow turn associated-data identity from a compatibility field into a hard integrity boundary, and can any output still remain accepted when that identity diverges from the ownership semantics later consumed by scan or spend paths?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** In the accepted Scenario 1 flow, associated-data identity stops being a compatibility alias at two live seams: the scanner/decrypt path consumes `leaf_ad_id` as the canonical decrypt-associated-data identity, and the spend witness/public-contract path later reuses and authenticates that same identity through input decrypt plus `leaf_ad_hash` binding. Once a full-stealth asset enters that canonical owned/spendable path, omission or tampering of `leaf_ad_id` no longer survives as harmless metadata drift: it either destroys ownership detection or rejects later spend acceptance. The answer stays partial because the broader statement “no crafted output can ever still look accepted under any surface” is stronger than the specific canonical-path proof now pinned by code and tests.

**Evidence Trail:**

1. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` freezes `leaf_ad_id` as the canonical decrypt-associated-data asset identifier rather than a compatibility detail.
2. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves that freeze directly in `scenario1_leaf_ad_id_is_the_canonical_decrypt_boundary`: after a valid owned output is built, changing `asset.leaf_ad_id` causes the scanner to stop returning `Mine`.
3. `crates/z00z_wallets/src/core/tx/witness_gate.rs` turns that same identity into the accepted spend-bridge contract: `wire_decrypt_leaf(...)` rebases the decrypt leaf so `leaf.asset_id = leaf_ad_id`, and `resolve_input_pack(...)` refuses to decrypt without `leaf_ad_id` and then consumes it as the associated-data identifier.
4. The same file’s tests prove fail-closed behavior: missing `leaf_ad_id` rejects, and `crates/z00z_wallets/tests/test_spend_witness_gate.rs` shows tampered `leaf_ad_id` rejects at the witness gate instead of silently remaining spendable.
5. `crates/z00z_wallets/src/core/tx/spend_verification.rs` binds `leaf_ad_id` into the accepted public spend contract by requiring `leaf_ad_id_hex` and `leaf_ad_hash_hex` for each input proof row, recomputing the input leaf associated-data hash, and requiring `leaf_ad_id` on outputs before computing output leaf hashes.
6. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` adds the matching public-boundary negative test: tampering `leaf_ad_hash_hex` in the spend proof is rejected fail closed.
7. `crates/z00z_core/src/assets/asset_validation.rs`, `crates/z00z_core/src/assets/wire.rs`, and `crates/z00z_core/tests/assets/stealth_consistency.rs` prove the structural admission rule that full stealth fields require `leaf_ad_id`; omission is not treated as an acceptable compatibility shortcut.
8. `.planning/phases/032-crypto-audit-scenario-1/032-TEST-SPEC.md` states the intended invariant explicitly: `leaf_ad_id` is canonical and not caller-controlled, and tampering or omission must reject instead of leaving owner-binding semantics ambiguous.

**Reasoning:**

- The freeze becomes operational, not merely documentary, when scanner/decrypt logic starts consuming `leaf_ad_id` as the associated-data identity needed to recover ownership.
- The witness gate then carries the same identity forward into spend preparation, so later spend handling is tied to the same decrypt boundary rather than to a parallel compatibility alias.
- The accepted public spend contract strengthens that further by authenticating the input-side associated-data identity through `leaf_ad_hash` binding and by requiring output-side `leaf_ad_id` for full stealth outputs.
- That means a canonical output whose later ownership semantics depend on one associated-data identity cannot quietly drift to another and still stay on the normal owned/spendable path.
- The important nuance is narrower than “every repeated `leaf_ad_id` is forbidden.” Input-side duplicate `leaf_ad_id` values may still be legitimate because that field is a decrypt/authentication namespace, not the unique consumed-state key. That does not reopen acceptance of identity drift; it only prevents a false uniqueness rule on honest claim-origin inputs.

**Gap Or Blocker:** The repository strongly proves that `leaf_ad_id` drift does not survive the canonical owned/scanned/spendable flow, but it does not yet justify the wider universal claim that any artificially crafted output artifact is rejected everywhere by an independently duplicated ownership oracle. The honest closed statement is therefore about the accepted canonical flow, not every hypothetical artifact surface.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status comes from keeping the claim scoped to the canonical constructed-and-later-consumed flow while preserving the legitimate input-namespace nuance.

### 9. Request Privacy Versus Card Fallback

🔴 **Quest:** What evidence shows that request-bound scanning metadata is treated as the preferred privacy path, and where does the repository still tolerate a card-only fallback that must be described as bounded compatibility rather than equal-strength privacy behavior?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The live repository proves a narrower statement than “request-bound is globally the default privacy path.” What is actually pinned is that, for Scenario 1 request or invoice flows, the accepted validated path binds route approval, `k_dh`, and `tag16` to `req_id`, and a request-aware scanner accepts the output exactly where plain scan and wrong-request scan do not. That is strong evidence that request-bound metadata is the preferred privacy lane for the accepted request flow. But the repository still keeps a live card-bound or base lane in raw build APIs, plain scan APIs, address-manager helpers, and no-request/no-tag compatibility behavior. So card-only must be described as an explicit bounded compatibility lane with weaker, or at least not equally proven, privacy semantics rather than as an equal-strength privacy path.

**Evidence Trail:**

1. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` freezes the intended accepted-flow language directly: a signed payment request is the accepted-flow privacy path when request-bound behavior is available, while request-bound and card-bound `tag16` semantics must remain distinct.
2. `.planning/phases/032-crypto-audit-scenario-1/032-TEST-SPEC.md` turns that freeze into an explicit invariant: request-bound tag derivation is the normal privacy path, and card-only fallback must stay explicit instead of silently replacing the request contract.
3. `crates/z00z_wallets/src/core/stealth/output.rs` and `crates/z00z_wallets/src/core/stealth/output_build.rs` define the accepted validated request lane: `build_tx_stealth_output_validated(...)` is the accepted-flow constructor for request-bound Scenario 1 output building, and request/card route validation plus request-aware key derivation only activate when a verified payment request is present.
4. `crates/z00z_wallets/src/core/address/stealth_request.rs` keeps that request lane wallet-local but explicit by enforcing signature, chain-id, expiry, and TOFU or pinning approval before a request is treated as approved input.
5. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves the intended split operationally: request-bound `tag16` diverges from card-bound mode, the foreign request/card mismatch fails the accepted flow, and the validated request path is the tested strict lane.
6. `crates/z00z_wallets/tests/test_e2e_req_flow.rs` proves the privacy preference in live receive behavior: the request-aware scanner with the right request detects the output, while the plain scanner and wrong-request scanner reject it.
7. `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs`, `crates/z00z_wallets/tests/test_stealth_scanner_cache.rs`, and `crates/z00z_wallets/examples/payment_request_invoice.rs` repeat the same accepted-flow rule: plain scan is insufficient for request-bound invoice semantics, and adding the active request makes the output detectable.
8. The repository also proves that card-only has not been removed. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` still computes the no-request path with card-bound `tag16 = compute_tag16(k_dh, leaf_ad)` and only uses `compute_tag16_with_req(...)` when request metadata is present.
9. The same scan-support module still tries the base/card lane before iterating active request ids during direct scan, so request-aware detection is additive rather than globally replacing the base lane.
10. `crates/z00z_wallets/src/core/address/address_manager/address_manager_trait.rs` keeps plain checkpoint or range scan helpers as the default wrapper surface and offers request-aware variants beside them rather than instead of them.
11. `.planning/phases/032-crypto-audit-scenario-1/032-TODO.md` records the unresolved privacy caveat explicitly: targeted scan-DoS remains relevant when `tag16` is card-bound rather than request-bound, which is exactly why card-only cannot honestly be narrated as equal-strength privacy behavior.

**Reasoning:**

- The strongest live proof of request preference is behavioral, not aspirational: request-bound outputs are accepted only when the sender used the approved request route and the receiver scans with the matching active request.
- That makes request metadata more than optional decoration for invoice-style privacy semantics; it changes the accepted key or tag derivation and therefore the observable ownership path.
- But the repository does not collapse everything into that mode. It still supports raw builder calls without a request, plain scanner flows without active requests, and direct scan logic that continues to check the card-bound lane.
- That means the honest statement is asymmetric. Request-bound metadata is the preferred privacy contract for the accepted request flow, while card-only remains a live compatibility path.
- Because that compatibility path is still active and tested, it cannot be honestly described as already reduced to a dead or purely historical stub. The narrower truthful claim is that it remains explicit and bounded, while the repository does not prove equivalent privacy strength for it.

**Gap Or Blocker:** The repository documents and tests request-bound behavior as the normal privacy lane for request or invoice flows, but it does not prove that request-bound has become the global default across every accepted scan surface. Card-bound or base behavior remains first-class in several APIs and scanner paths, and targeted-scan privacy caveats for that lane remain open in phase notes. Until those base-lane semantics are either formally demoted or given equivalent privacy proof, the stronger global-default claim cannot be closed honestly.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status comes from the still-live card-bound/base lane in builder and scanner APIs, not from any failure of the accepted request-bound request-flow evidence.

### 10. End-To-End Ownership Through The Chain

🔴 **Quest:** Across the full chain `Alice -> leaf build -> JMT publish -> Bob scan -> spend -> validator`, what evidence shows where Bob-only ownership remains stable, and at which handoff the repository still stops short of proving that ownership cryptographically end to end?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The live repository proves a stable accepted-flow ownership chain, not a fully closed end-to-end Bob-only cryptographic theorem. Alice builds outputs with Bob-bound `owner_tag` and deterministic `s_out`; after publication and persistence, Bob’s scanner re-derives the same ownership or decrypt path, recovers the same `pack.s_out` as wallet `asset_secret`, and Bob-secret-gated spend preparation can produce a persisted Stage 4 package that passes the current public spend contract. The proof stops at the witness-to-validator handoff. At that point the repository verifies a public spend statement, hashes, commitments, range proofs, and receiver-card authorization, but it does not independently re-prove the same receiver-secret plus output-secret ownership invariant end to end at the validator boundary.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/stealth/output_build.rs` shows the first Bob-bound seam directly: the sender-side build path derives `k_dh`, computes the Bob-bound `owner_tag`, derives deterministic `s_out`, and encrypts the output pack from that material.
2. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` proves the publish or scan-side continuity: scanner candidates only survive if their derived `owner_tag` matches, successful decrypt yields `AssetPackPlain`, and `make_wallet_output(...)` persists `asset_secret: Some(pack.s_out)` instead of changing the ownership secret model after receipt.
3. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` pins the wallet-local Bob-only rule: `verify_owner_two_factor(...)` requires the real `receiver_secret` plus the re-derived secret relation, sender material fails, and tampered `s_out` fails.
4. `crates/z00z_simulator/tests/test_e2e_phase4.rs` proves that this ownership picture survives publication and reload on the accepted Scenario 1 path. Stage 3 and Stage 4 rows both scan as `Detected`, and Bob recovers the same `asset_id`, `serial_id`, `amount`, `r_pub`, `owner_tag`, `asset_secret = pack.s_out`, and `blinding`, with the same receive verdict preserved across reload.
5. `crates/z00z_wallets/src/core/tx/spend_rules.rs` and `crates/z00z_wallets/src/core/tx/witness_gate.rs` prove the spend-preparation continuity is still Bob-secret-gated: the spend rules derive owner and view material from `receiver_secret`, recompute `owner_tag` and `leaf_ad_id` relations, and the witness gate only accepts inputs that are actually decryptable for the provided receiver secret.
6. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` proves the accepted spend path rejects tampered `leaf_ad_id`, missing owner fields, and placeholder-style public contract gaps before treating the input as valid spend material.
7. `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` proves the persisted Stage 4 package really reaches the current public spend contract: the package contains spend proof and spend auth, and `verify_tx_public_spend_contract(...)` accepts the canonical package while rejecting placeholder gaps.
8. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md`, `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md`, and `032-04-SUMMARY.md` all keep the same caveat explicit: the current stack hardened the accepted spend verifier boundary, but Scenario 1 still does not prove an end-to-end trustless public verifier for every wallet-local ownership rule or two-factor receiver-secret invariant.

**Reasoning:**

- The repository does show continuity from build through publish and scan. The same Bob-bound ownership fields that Alice emits are what Bob later uses to detect ownership and recover the same output-secret material.
- It also shows continuity from Bob scan into spend preparation. The spend-rule and witness-gate layers still require Bob’s receiver secret and the matching decrypted input relation before the spend path can proceed.
- That is enough to prove an accepted-flow chain where Bob-only ownership remains stable operationally from output construction to scan and then into spend preparation.
- The weaker point is the final validator seam. Once the witness-bearing local path is packaged into the public spend statement, the validator verifies the current public contract rather than re-running the full Bob-secret ownership relation.
- So the repository proves current-stack spend acceptance continuity, but not that the validator independently reconstructs or cryptographically closes the same `receiver_secret + s_out` two-factor ownership theorem end to end.

**Gap Or Blocker:** The exact shortfall appears at the handoff from secret-bearing local or witness validation into the validator-facing public contract. Before that handoff, Bob-secret-gated ownership is real and test-backed. After that handoff, the repository validates public spend-proof/auth objects and canonical statement hashes, but it does not yet prove that the validator independently enforces the same receiver-secret-gated two-factor ownership relation or the broader original `PH32-SPEND` nullifier semantics. That is why the chain-wide ownership claim stays partial.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status comes from the validator-side ownership gap, not from any observed break in the accepted build, publish, scan, or spend-preparation continuity.

## ⚖️ Theme 3: Spend Boundary And Replay Semantics

### 11. What The Current Public Boundary Actually Proves

🔴 **Quest:** What does the accepted spend boundary really verify today, and how does the repository distinguish that current-stack contract from a stronger claim of a finished universal trustless public verifier?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The accepted spend boundary now proves a narrower but real current-stack public contract. An accepted regular spend must carry persisted spend proof and spend authorization, pass wire-version checks, bind a nonzero previous root, match each proof row to the canonical tx input refs, recheck input `leaf_ad` hashes, require canonical output stealth fields and output `leaf_ad` relations, verify output range proofs, satisfy commitment-balance equality, reject duplicate or overlapping namespaces, validate the receiver card, and verify an authorization signature over one canonical framed statement. The repository also distinguishes that delivered contract from a stronger claim explicitly and repeatedly: it does not yet prove nullifier semantics inside the regular-spend public contract, it does not yet prove the wallet-local two-factor ownership rule as an end-to-end public-verifier theorem, and it does not justify calling the current boundary a finished universal trustless public verifier.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/spend_verification.rs` makes proof/auth persistence part of the accepted boundary itself: missing spend proof or missing spend auth rejects immediately, not as optional metadata.
2. The same verifier enforces the concrete current-stack contract directly: proof/auth version checks, nonzero `prev_root`, receiver-card decode and validation, canonical input/proof-row pairing, recomputed input `leaf_ad` hash binding, required output stealth fields, output `leaf_ad` relation checks, range-proof verification, commitment-balance equality, duplicate-input and namespace-overlap rejection, and final authorization verification over a canonical framed statement.
3. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` pins fail-closed behavior at the wallet boundary: structural-only placeholder acceptance is rejected when auth is missing, replayed previous-root material rejects, and tampered input `leaf_ad` hash rejects as a distinct integrity failure.
4. `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` proves the simulator accepted path reaches the same boundary honestly: the Stage 4 package really persists spend proof and spend auth, the canonical package passes `verify_tx_public_spend_contract(...)`, and placeholder auth gaps are rejected.
5. `.planning/phases/032-crypto-audit-scenario-1/032-04-SUMMARY.md` states the honest scope correction explicitly: the plan hardened the current-stack spend gate, but the live regular-spend wire and persisted spend proof still do not carry a nullifier field, so this evidence cannot be used to claim full `PH32-SPEND` closure.
6. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` and `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md` reinforce the same distinction in review language: Scenario 1 now proves a materially stronger accepted spend boundary, but it still does not prove an end-to-end trustless public verifier, a general-purpose on-chain verifier, or the broader original spend contract.

**Reasoning:**

- The strongest repository-backed statement is about the verifier that actually runs today. It is no longer a structural placeholder gate; it authenticates a concrete public spend statement assembled from persisted tx proof/auth data, previous-root context, receiver-card identity, canonical inputs, and canonical outputs.
- That means accepted spend success now depends on a real public-contract check rather than on witness-only preparation or placeholder acceptance.
- But the repository also documents the remaining limit precisely. The live wire format for regular spend still omits nullifier semantics, and the public verifier does not independently reconstruct the wallet-local two-factor ownership rule as a public theorem.
- The honest distinction is therefore asymmetric: the current public boundary is real, specific, and test-backed, but it is still narrower than the stronger universal trustless-verifier claim that phase notes explicitly forbid.

**Gap Or Blocker:** The still-open blocker is not the current public-contract check itself; it is the stronger contract that remains undelivered. The live regular-spend wire and public spend statement still do not carry nullifier semantics, and the validator-facing verifier still does not prove the full wallet-local two-factor ownership rule end to end. Until those pieces land, the repository can honestly claim a hardened current-stack spend boundary, but not a finished universal trustless public verifier.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: the accepted public-contract proof is strong, but it remains intentionally narrower than the broader `PH32-SPEND` and universal-trustless-verifier claims.

### 12. Theft Windows Before And After Publication

🔴 **Quest:** If an attacker tries to steal Bob's coin before publication, after JMT publication but before Bob scans, after Bob scans, or during the spend-to-validator handoff, what repository evidence closes or fails to close each theft path?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository closes these theft windows unevenly. Before publication, it does not prove a trustless anti-theft theorem at all; instead it keeps an honest caveat that withholding or publication refusal is still out of scope, and it explicitly forbids the false story that theft is impossible because Alice never knows `s_out`. After publication but before Bob scans, the repository gives strong wallet-local evidence that foreign keys do not satisfy the receiver-bound detection and spend-preparation path, but it still stops short of elevating that into a full public anti-theft theorem. After Bob scans, the local ownership picture is stronger again because Bob recovers stable `asset_secret = pack.s_out` and receiver-secret-gated ownership checks remain stable across reload and replay handling. During the spend-to-validator handoff, the accepted current-stack public contract and later checkpoint handoff close concrete rewrite, replay, and tamper lanes, but the repository still does not prove universal theft impossibility because nullifier semantics and the full receiver-secret public-proof theorem remain open.

**Evidence Trail:**

1. Before publication, the repository explicitly refuses to close the anti-theft story through sender ignorance. `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` and `032-HONEST-CLOSEOUT.md` both state that sender ignorance of `s_out` is not a valid claim, and `032-HONEST-CLOSEOUT.md` separately keeps withholding or publication refusal out of scope.
2. That pre-publication limit matches live code. `crates/z00z_wallets/src/core/stealth/output_build.rs` derives sender-side `k_dh` and `s_out` during the accepted output build path, so the repository cannot honestly say that pre-publication theft is prevented because the sender lacks output-secret material.
3. After publication but before Bob scans, the repository still gives strong wallet-local receiver binding. `crates/z00z_wallets/src/core/address/claim_own.rs` and `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` route ownership detection through the receiver scanner, and `crates/z00z_simulator/tests/test_stage4_output_crypto.rs` proves that the real receiver keys pass while wrong keys fail on live simulator outputs.
4. The same window also remains narrower than a public theorem. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves `leaf_ad_id` is the canonical decrypt boundary and that wallet-local two-factor ownership requires `receiver_secret` plus the correct `s_out`, but `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` keeps the explicit caveat that Scenario 1 still does not prove an end-to-end trustless public verifier for every receiver-secret or two-factor invariant.
5. After Bob scans, the repository materially strengthens the local theft picture. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` persists `asset_secret: Some(pack.s_out)` and `blinding` into the wallet-visible output, `crates/z00z_wallets/src/core/stealth/output.rs` enforces receiver-secret-gated two-factor ownership, and `crates/z00z_wallets/tests/test_scenario1_semantics.rs` proves the correct receiver secret passes while sender-side material and tampered `s_out` fail.
6. `crates/z00z_simulator/tests/test_e2e_phase4.rs` shows that post-scan ownership remains stable across publication, reload, and duplicate-delivery handling: Bob recovers the same `asset_secret`, `blinding`, and receive verdict without silent ownership inflation.
7. During the spend-to-validator handoff, the repository closes specific tamper paths with current-stack public-contract enforcement. `crates/z00z_wallets/src/core/tx/spend_verification.rs` requires persisted spend proof and spend auth, validates previous-root, input refs, `leaf_ad` hashes, output stealth relations, range proofs, balance, and authorization signature over the canonical framed statement.
8. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` and `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` prove that structural-only placeholder acceptance, replayed `prev_root`, and tampered `leaf_ad` relations reject fail closed instead of silently passing.
9. The later checkpoint handoff closes downstream rewrite or replay lanes on the accepted path. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` re-verifies the stage-4 package contract and rejects proof drift, input-ref drift, and output drift before checkpoint draft creation, while `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves tampered exec-input and package-proof cases fail.
10. Even with those handoff protections, the repository still keeps the broader caveat open. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` explicitly says that the live regular-spend wire does not yet carry nullifier semantics and that Scenario 1 still does not prove a finished trustless public verifier.

**Reasoning:**

- Before publication, there is no honest basis for a finished theft-prevention theorem because the repository keeps the publication or withholding caveat open and directly rejects the old sender-ignorance story.
- After publication, the strongest live guarantee becomes receiver-bound local ownership: wrong keys do not detect or satisfy the accepted ownership and spend-preparation path, and Bob's receiver-secret-gated local state remains stable once he scans.
- After Bob scans, the repository shows stronger persistence and replay stability for Bob's local ownership state, but that still lives on the wallet-local side of the trust boundary.
- During the spend and later checkpoint handoff, the repository closes concrete rewrite, replay, and tamper lanes through the current public spend contract and the stage-11 handoff verifier.
- The answer stays partial because the repository itself separates those delivered protections from the broader unproven claims: withholding remains out of scope, sender ignorance remains false, and the final public verifier still lacks the broader nullifier and full receiver-secret-theorem closure.

**Gap Or Blocker:** The missing proof is not that nothing is protected. The missing proof is that these protections do not yet add up to a universal trustless anti-theft statement across all windows. Pre-publication withholding and publication refusal remain outside the delivered contract, and the post-publication public verifier still does not carry the broader nullifier semantics or prove the full wallet-local receiver-secret ownership theorem end to end.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status comes from the still-open publication/withholding caveat and the unfinished public-verifier theorem, not from a lack of concrete local or handoff protections.

### 13. Proof Continuity Across Handoff

🔴 **Quest:** Do the accepted proof and authorization artifacts preserve one continuous meaning from spend creation through later state-application steps, or is there still any seam where later code can reinterpret, rebuild, or downgrade the verified statement?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** On the accepted current-stack path, the proof-and-authorization meaning remains continuous from spend creation through later state application. Stage 11 re-validates the original stage4 public spend contract before it accepts the handoff, then requires the later checkpoint execution row to match the already verified upstream package exactly in proof bytes, input references, and outputs. There is still a narrow packaging caveat: later checkpoint execution artifacts do not carry `spend_auth` as a separate standalone replay artifact, but that does not create a live downgrade seam on the accepted path because later apply code stays package-coupled and rejects tamper before draft build.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/spend_verification.rs` defines the original meaning of the accepted spend boundary: the verifier requires authorization, rebuilds the canonical spend statement, and verifies the authorization signature over that statement instead of trusting loose proof-shaped bytes.
2. `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` proves the accepted stage4 package really carries both spend proof and spend authorization, passes the same verifier on the happy path, and rejects when authorization is missing.
3. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` enforces handoff continuity directly. `verify_stage7_handoff(...)` first re-runs the current public spend verifier on the original stage4 package, then requires exact equality between the checkpoint exec row and the already verified upstream package for `tx_proof`, `input_refs`, and the bridged outputs.
4. `crates/z00z_storage/src/checkpoint/exec_input.rs` documents that `tx_proof` in checkpoint replay artifacts is the exact upstream proof byte sequence and must not be reconstructed or synthesized later in the storage path.
5. `crates/z00z_storage/src/checkpoint/build_prepare.rs` preserves that contract in code by copying `tx_proof` bytes directly into the deterministic checkpoint summary instead of rebuilding them.
6. `crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs` proves checkpoint exec codec roundtrips preserve exact `tx_proof` bytes, and `crates/z00z_storage/tests/test_checkpoint_draft_build.rs` proves draft building forwards the same proof bytes to the verifier without reinterpretation.
7. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves the later state-application seam rejects proof tamper, digest tamper, and replay-style exec-row tamper instead of silently accepting a downgraded or reinterpreted statement.
8. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` shows the same continuity model on the checkpoint-side verifier surface: later checkpoint proof verification uses the carried `tx_proof`, resolved inputs, and outputs as the deterministic summary of the already verified upstream package rather than inventing a new spend meaning.

**Reasoning:**

- The accepted current-stack path does not treat later checkpoint apply as a fresh, weaker interpretation of an earlier spend. It first re-checks the original stage4 public spend contract, including authorization semantics, and only then permits the checkpoint handoff.
- After that re-check, the handoff is held together by exact equality constraints on proof bytes, input references, and outputs, so later state-application steps cannot silently swap in a semantically different transaction row.
- Storage and replay layers preserve the carried proof bytes without reconstructing them, and the regression suite confirms that tampered proof, digest, or replayed exec rows fail closed.
- The honest nuance is that checkpoint exec artifacts are package-coupled rather than self-contained authorization carriers. But on the accepted path that is a packaging boundary, not a live downgrade seam, because later apply logic still rebinds the accepted meaning back to the original verified package before state changes proceed.

**Gap Or Blocker:** There is no live evidence of a reinterpret/rebuild/downgrade seam on the accepted current-stack path. The only remaining caveat is narrower: checkpoint exec artifacts are not designed as standalone authorization carriers outside the original package-coupled verification path.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: the verified result applies to the accepted package-coupled current-stack path; later checkpoint artifacts are not intended to stand alone as an independent replacement for the original stage4 authorization artifact.

### 14. The Requirement That Remains Open

🔴 **Quest:** What exact spend-verifier element is still missing from the regular accepted public statement, and what code-and-test evidence proves that this missing element is the reason the broader spend requirement remains only partially closed?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The exact missing element is nullifier semantics inside the accepted regular-spend public contract. The current regular-spend wire, persisted spend proof, and encoded public statement carry previous-root, canonical input/output, leaf-associated-data, range-proof, balance, and authorization data, but they do not carry a spend nullifier field or any equivalent nullifier binding. The broader `PH32-SPEND` requirement therefore remains only partially closed because the repository now proves a real narrowed spend verifier boundary while explicitly leaving the nullifier portion of the original requirement open.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` shows the accepted regular input wire carries only `asset_id_hex` and `serial_id`, and the wire comment explicitly says `serial_id` must not be treated as checkpoint nullifier material.
2. The same file shows `SpendProofWire` contains only `ver`, `prev_root_hex`, and input proof rows. There is no nullifier field on the regular spend proof object, no nullifier field on `TxProofWire`, and no separate regular-spend nullifier object on `TxAuthWire`.
3. `crates/z00z_wallets/src/core/tx/spend_verification.rs` proves the current public spend verifier is real but narrower: it encodes and verifies a canonical spend statement over chain id, version, tx metadata, receiver card, previous root, canonical tx inputs, proof inputs, outputs, and output leaf hashes, then verifies authorization. There is no decode, reconstruction, or validation path for a spend nullifier field in that verifier.
4. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` proves the narrower accepted public-contract boundary is live and fail closed: missing auth, replay-style `prev_root` drift, and `leaf_ad_hash` tamper reject instead of succeeding structurally.
5. `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` proves Scenario 1 persists real spend proof/auth into the accepted stage4 package and reuses the same wallet verifier on both the green path and fail-closed paths.
6. `.planning/REQUIREMENTS.md` keeps `PH32-SPEND` open and defines the broader spend requirement as including nullifier semantics together with the other public-verifier bindings.
7. `.planning/phases/032-crypto-audit-scenario-1/032-04-SUMMARY.md`, `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md`, `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md`, and `.planning/phases/032-crypto-audit-scenario-1/032-VALIDATION.md` all repeat the same correction: current-stack spend hardening landed honestly, but `PH32-SPEND` remains partial because the live regular-spend wire and public spend statement still do not carry nullifier semantics.

**Reasoning:**

- The repository does not leave this gap ambiguous. The regular spend wire shape itself excludes nullifier material, and the current public statement encoder/verifier never reconstructs or checks a spend nullifier.
- At the same time, the repository also proves the current verifier is not placeholder-only. It validates persisted proof/auth, previous root, canonical input pairing, associated-data integrity, range proofs, balance, and authorization signature with real fail-closed tests.
- That combination matters for the answer: the broader requirement is still open not because spend verification is fake, but because one exact required element from the original `PH32-SPEND` wording is still absent from the accepted regular public contract.
- The honest conclusion is therefore precise: the narrowed current-stack spend boundary is real and verified, while the original nullifier-semantics portion of `PH32-SPEND` remains undelivered.

**Gap Or Blocker:** The blocker is not missing proof/auth enforcement in general. The blocker is specifically that the accepted regular-spend public statement still does not carry and validate nullifier semantics, so the original broader `PH32-SPEND` contract cannot honestly be marked complete.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: broader honesty notes also keep end-to-end public ownership and claim-trust caveats open, but those are separate open statements and are not the reason `PH32-SPEND` itself remains partial.

### 15. Full-Chain Crypto Closure Versus Partial Security

🔴 **Quest:** When the entire `Alice -> leaf build -> JMT publish -> Bob scan -> spend -> validator` chain is audited as one security story, which parts are backed by genuinely live cryptography today, which rely on wallet-local or structural checks, and which are still honest placeholders or future-proof aspirations?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The honest full-chain answer is a layered classification, not a claim of end-to-end trustless closure. The repository does contain genuinely live cryptography today: canonical claim statement signing and verification, stealth output construction, Bob-side scan/decrypt semantics, and the current-stack public spend verifier with range-proof, balance, and authorization checks. But the chain is not uniformly cryptographic from end to end: claim-root continuity, receiver-secret ownership, checkpoint handoff, and replay/spent gating still rely in part on wallet-local or structural boundaries, while final proof-backend claims, universal validator trustlessness, withheld-data guarantees, full spend nullifier closure, and full persisted claim-trust continuity remain explicitly open.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` together with the claim statement/auth layers proves that claim-package cryptography is live at the statement level: the verifier rechecks proof/version/root coherence, rebuilds the canonical claim statement, and verifies the authority signature over that statement.
2. `crates/z00z_wallets/src/core/stealth/output_build.rs` and `crates/z00z_wallets/src/core/stealth/output.rs` show Alice-side leaf build is backed by live cryptographic derivation and encryption: `k_dh`, `s_out`, canonical `leaf_ad`, encrypted pack construction, and tag behavior are all real accepted-flow seams.
3. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` shows Bob-side scan/decrypt is also live cryptography: the scan path recomputes the canonical decrypt-associated-data boundary, checks tag material, decrypts the pack, and verifies commitment-opening consistency before returning owned output state.
4. `crates/z00z_wallets/src/core/tx/spend_verification.rs` proves the spend leg has a real current-stack public verifier: previous-root, canonical input pairing, associated-data hashes, output stealth relations, range proofs, balance equality, and authorization signature are checked at the accepted public boundary.
5. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` and `docs/code-review/032-scenario-1-crypto-status.md` explicitly place checkpoint apply in a narrower category: stage11 now rejects placeholder proof and placeholder spent-state success, but the phase still does not claim a recursive proof backend, final on-chain verifier, or universal trustless validator boundary.
6. `crates/z00z_wallets/src/core/stealth/output.rs` together with `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md` proves receiver-secret-gated ownership is real, but wallet-local: the repository explicitly forbids treating that local two-factor rule as already equivalent to a finished public verifier theorem.
7. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` and the Phase 032 closeout artifacts prove the claim leg remains only partially trustless at the continuity seam: claim-package cryptography is live, but source-root continuity still depends on a synthetic one-item store contract rather than persisted storage-backed membership continuity.
8. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` proves the validator-adjacent handoff is package-coupled and structural: stage11 rechecks the original stage4 spend contract and then enforces exact proof/input/output continuity before checkpoint draft emission, rather than promoting a separate final proof backend.
9. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md`, `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md`, and `docs/code-review/032-scenario-1-crypto-status.md` explicitly forbid overclaims about live STARK/FRI, recursive checkpoint proofs, universal trustless verification, withheld-data recovery, and full closure of `PH32-SPEND` or `PH32-CLAIM-TRUST`.

**Reasoning:**

- Alice leaf build, Bob scan/decrypt, claim statement binding, and the accepted current-stack spend verifier all sit in the genuine live-crypto bucket because the repository actually derives, encrypts, signs, verifies, and rejects cryptographic material at those seams today.
- The chain stops being purely cryptographic when it crosses into wallet-local ownership rules, synthetic claim-root continuity helpers, checkpoint exec packaging, and spent-index or replay bookkeeping. Those are still real protections, but they are structural or local enforcement layers rather than one universal public proof.
- The honest placeholder/future bucket is also explicit, not inferred. The repository itself says that live STARK/FRI, recursive checkpoint proofs, universal validator trustlessness, withheld-data safety, full spend-nullifier closure, and full persisted claim-trust continuity are not yet delivered.
- So the correct full-chain security story is mixed: materially stronger and more truthful than the pre-032 state, but still layered and intentionally narrower than a finished trustless end-to-end zk-verifier architecture.

**Gap Or Blocker:** The remaining blockers are not hidden. The chain still lacks live recursive or succinct checkpoint proof backends, full `PH32-SPEND` nullifier semantics, full persisted `PH32-CLAIM-TRUST` continuity, and a universal public verifier that subsumes all wallet-local ownership invariants.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: the verified result supports the layered classification itself; it does not imply that the entire chain is already fully closed as one end-to-end trustless public-proof system.

## 🧱 Theme 4: Checkpoint Integrity And Artifact Continuity

### 16. Placeholder Success Paths Truly Closed

🔴 **Quest:** What evidence proves that checkpoint acceptance no longer succeeds merely because proof-shaped bytes exist or placeholder spent-state logic happens to look well-formed?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The live repository proves that accepted checkpoint success is no longer driven by proof-shaped bytes or placeholder spent-state that merely looks structurally plausible. Stage 11 now revalidates the original stage4 package contract, then fail-closes on proof drift, canonical input-ref drift, canonical output drift, and replay-style exec-row tamper before it emits checkpoint artifacts. The honest scope is narrower than a final recursive proof backend, but within the accepted current-stack path the placeholder success lanes are closed.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` proves stage11 is no longer a passive pass-through. `verify_stage7_handoff(...)` re-runs the stage4 public spend verifier and then rejects any mismatch between the checkpoint exec row and the already verified package for proof bytes, canonical input refs, and canonical outputs.
2. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` shows the checkpoint verifier surface itself is no longer proof-shaped-byte tolerant: it binds verification to exact `tx_proof`, deterministic input refs, and deterministic outputs rather than accepting any non-empty compatibility blob.
3. The same file also shows placeholder spent-state logic is gone from the accepted path. `CheckpointReplaySpentIndex` only reports spent status when both the expected root and canonical input ids match; root drift or mismatched ids return lookup failure instead of silently passing.
4. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves exec-row tamper rejects fail closed and blocks checkpoint emission, including `checkpoint_s7.json` and post-tx draft persistence.
5. The same test file proves package-proof tamper and package-digest tamper also reject before checkpoint emission, so acceptance no longer survives because bytes merely look proof-like.
6. The same integration suite proves replay-style post-tx exec-row reuse later reloads as `ReplayMix` instead of being accepted as fresh truth.
7. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` and `docs/code-review/032-scenario-1-crypto-status.md` state the same delivered boundary explicitly: accepted checkpoint apply no longer succeeds through `PassProof`, `NoSpent`, or equivalent placeholder logic.

**Reasoning:**

- The current accepted checkpoint path now depends on revalidation plus exact continuity checks, not on superficial artifact shape.
- Proof bytes must match the verified upstream package, input refs must match the canonical consumed pre-state, outputs must match the bridged outputs, and replay-style exec tamper is rejected before state promotion.
- Spent-state handling is also fail closed: root/id mismatch becomes a lookup failure, not an implicit green path.
- That means the relevant placeholder acceptance modes are closed in the live current-stack implementation even though the repository still avoids claiming a stronger final recursive or standalone checkpoint proof backend.

**Gap Or Blocker:** The remaining caveat is not a surviving placeholder success lane. The remaining caveat is that this accepted checkpoint boundary is still package-coupled and current-stack scoped, not a final recursive or universal checkpoint proof architecture.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: this answer proves placeholder proof/spent-state success is closed on the accepted path; it does not upgrade the result into full nullifier-complete spend closure or live STARK/FRI checkpoint proofs.

### 17. Draft Versus Final Truth

🔴 **Quest:** Which repository artifacts and tests show that draft-only checkpoint outputs cannot masquerade as final authoritative state, either at emission time or when later loaded and interpreted?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The repository enforces draft-versus-final separation at three distinct levels: class-specific loading, proof-bound finalization, and the canonical authoritative id/link path. Draft bytes do not load as final artifacts, final bytes do not load as drafts, proofless or statement-mixed finals are rejected, and the canonical authoritative path additionally requires statement-bound sealing and replay-linked binding. A legacy opaque compatibility lane still exists for explicitly final artifact shapes, but there is no live evidence that draft-only checkpoint outputs can masquerade as final authoritative state.

**Evidence Trail:**

1. `crates/z00z_storage/tests/test_checkpoint_draft_final.rs` proves direct class separation in both directions: draft bytes reject final-artifact loading, and final bytes reject draft loading.
2. `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs` and `crates/z00z_storage/tests/test_checkpoint_finalization.rs` prove that finalization is statement-bound rather than shape-only. Tampered public inputs, mismatched statements, and proofless finals reject with `ProofMix` or `ProoflessFinal` instead of silently sealing a draft into accepted final truth.
3. `crates/z00z_storage/src/checkpoint/artifact_final.rs` distinguishes statement-bound attested artifacts from `LegacyOpaque` artifacts at the interpretation layer, so later loads do not implicitly upgrade draft-like material into the authoritative statement-bound class.
4. `crates/z00z_storage/tests/test_checkpoint_ids.rs` proves that draft objects, partial-statement shells, proofless shells, and incompatible attestation shells cannot derive canonical checkpoint ids or pass canonical encode paths.
5. The same id tests also prove that checkpoint identity is tied to the canonical statement path rather than to arbitrary compatibility proof bytes alone.
6. The store/seal path requires statement-bound finalization for the canonical authoritative lane, and the filesystem/link path rejects legacy artifacts as authoritative replay-linked checkpoints.
7. `crates/z00z_storage/tests/test_checkpoint_finalization.rs` separately proves that explicit legacy opaque final artifacts remain legacy when loaded; they do not become attested statement-bound finals by reinterpretation.

**Reasoning:**

- A draft cannot masquerade as final if it fails at load time, fails at finalization time, and fails at authoritative identity or replay-link time. The repository enforces all three of those boundaries.
- Final truth is not inferred from “checkpoint-shaped bytes.” It is tied either to an explicitly final legacy artifact shape or, in the canonical authoritative lane, to a statement-bound attested artifact that survives id, encode, and link checks.
- That is enough to close the specific masquerade claim in this question: draft-only outputs do not silently cross into final authoritative interpretation.
- The remaining nuance is narrower and explicit: legacy opaque final artifacts still exist as compatibility artifacts, but they are already-final shapes and they do not reopen a draft-as-final confusion path.

**Gap Or Blocker:** There is no live evidence that draft bytes can be reinterpreted as final authoritative state. The honest residual caveat is only that the repository still preserves an explicit legacy-final compatibility lane, which must not be conflated with the canonical statement-bound authoritative path.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: legacy opaque final artifacts still exist as compatibility artifacts, but they do not allow draft-only checkpoint outputs to masquerade as statement-bound authoritative final state.

### 18. Injective Persistence Contract

🔴 **Quest:** Do persisted checkpoint bytes remain injectively bound to one executed checkpoint identity, proof payload, and bound root, or can semantically different checkpoint states still serialize into acceptance-compatible artifacts?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository does not prove a fully injective checkpoint artifact surface across identity, proof payload, and bound root. On the raw checkpoint-artifact path, canonical checkpoint identity is bound to the statement (or legacy public input) plus proof-system class, not to the full `cp_proof` byte payload, so different proof payloads can serialize into acceptance-compatible artifacts with the same checkpoint id. The binding becomes much stronger only on the canonical persisted RedB path, where rehydrate validation fail-closes on statement drift, proof-byte drift, exec-id drift, and bound-root drift.

**Evidence Trail:**

1. `crates/z00z_storage/src/checkpoint/ids.rs` shows checkpoint identity is derived from a canonical statement or public-input shape together with proof-system class, not from the full compatibility proof payload bytes.
2. `crates/z00z_storage/tests/test_checkpoint_finalization.rs` proves this explicitly: changing `cp_proof` bytes changes serialized artifact bytes but does not change the derived checkpoint id.
3. `crates/z00z_storage/tests/test_checkpoint_ids.rs` independently confirms that checkpoint id derivation ignores attestation proof bytes on the raw id layer.
4. `crates/z00z_storage/src/checkpoint/artifact_final.rs` and `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs` document and enforce that the canonical statement binds artifact identity, while compatibility proof bytes do not replace that statement-owned binding.
5. The same draft/finalization code also proves that statement or public-input drift does not survive finalization: draft mismatch and proof/public-input drift reject fail closed instead of silently producing a valid attested artifact.
6. `crates/z00z_storage/src/checkpoint/codec.rs` and the raw store surface show that raw artifact encode/load enforce structural compatibility, proof-system class, and non-empty proof bytes, but they do not by themselves prove semantic correctness of arbitrary `cp_proof` payload bytes.
7. `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` and the corresponding rehydrate validator show the stronger persisted path: persisted artifacts are written in a deterministic exec-id/state-root-bound form, and RedB rehydrate rejects proof-byte drift, statement drift, mixed-id eras, and legacy-plus-link mismatches.
8. `crates/z00z_storage/tests/test_redb_rehydrate.rs` proves those stronger persisted guarantees in practice by rejecting proof drift, statement drift, and incompatible artifact/link combinations during rehydrate.

**Reasoning:**

- The raw artifact contract is not fully injective because checkpoint identity intentionally ignores `cp_proof` byte differences. That means two artifacts can share one canonical checkpoint id while carrying different compatibility proof payload bytes.
- At the same time, the repository does not leave the stronger persisted path loose. The canonical RedB bundle ties artifact interpretation back to the reconstructed statement, exec identity, and bound root, then rejects drift fail closed during rehydrate.
- So the honest answer is split. Full byte-level injectivity is not delivered across the whole raw artifact surface, but stronger fail-closed identity/root/proof binding does exist on the canonical persisted path used for authoritative reload.

**Gap Or Blocker:** The missing property is full artifact-surface injectivity by proof payload. Raw checkpoint artifacts can still differ in `cp_proof` bytes while remaining loadable and id-compatible, so only the persisted RedB path, not the entire raw artifact surface, gives the stronger executed-identity and bound-root closure.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: if the question is narrowed only to the canonical RedB-persisted rehydrate path, the binding is materially stronger than the answer’s global `PARTIAL` verdict suggests; the `PARTIAL` label applies to the whole checkpoint artifact surface.

### 19. Replay And Stale-Artifact Resistance

🔴 **Quest:** Where does the live repository prove that stale checkpoint artifacts, replayed proof-bearing bundles, or old spent-state rows cannot be reintroduced as if they were fresh accepted truth?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The live repository proves replay and stale-artifact resistance strongly on the checkpoint artifact, exec-input, and link tuple boundaries, and it proves that tampered or replay-style proof-bearing bundles fail closed on both unit and simulator paths. The answer stays partial only for the old spent-state-row part of the question: the code clearly reloads old spent/nullifier rows as already-spent replay keys and has replay guards for reused nullifiers, but the inspected tests do not show the full rehydrate-then-replay denial as one explicit end-to-end assertion.

**Evidence Trail:**

1. `crates/z00z_storage/src/checkpoint/store.rs` and `crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs` prove that stale or mismatched checkpoint tuples do not reload as accepted truth: link/snapshot mismatch rejects as `LinkMix`, exec-id mismatch rejects as `ReplayMix`, and previous-root mismatch rejects as `RootMix`.
2. The same store path recomputes and rechecks checkpoint tuple components on load, including the exec-input id and root-bound statement, so tampered stored bytes do not survive under the same external identity.
3. `crates/z00z_storage/src/checkpoint/store_fs.rs` adds two more fail-closed guards: one exec-input id cannot be silently reused across different links, and legacy opaque artifacts cannot satisfy the persisted replay-linked authoritative path.
4. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves the live simulator boundary rejects tampered exec rows, package-proof tamper, package-digest tamper, and replay-style post-tx exec reuse before fresh acceptance artifacts are emitted or later reloaded.
5. `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` together with `crates/z00z_storage/tests/test_redb_rehydrate.rs` proves canonical RedB rehydrate rejects mixed checkpoint-id eras, missing snapshot/exec rows, proof-byte drift, statement drift, and incompatible legacy-plus-link bundles.
6. The same RedB/storage code shows that spent-state rows are restored as replay-keyed spent records rather than dropped or interpreted as fresh spendable truth, and reused nullifier keys map to replay rejection instead of silent reuse.
7. `crates/z00z_storage/tests/test_redb_rehydrate.rs` proves legacy spent/nullifier rows reload back into spent state rather than being forgotten or converted into fresh accepted state.

**Reasoning:**

- For stale checkpoint artifacts and replayed proof-bearing bundles, the proof is direct: ids are recomputed, roots are rechecked, links are tuple-bound, and tamper or replay causes fail-closed rejection.
- That same discipline carries through the canonical RedB reload path, which blocks mixed-era or drifted bundles from re-entering accepted state as if they were fresh.
- The weaker part is the old spent-state-row clause. The repository clearly treats restored nullifier rows as already-spent replay keys and has replay guards for reused nullifiers, but the inspected evidence is partly code-backed inference rather than one dedicated rehydrate-then-replay integration assertion.

**Gap Or Blocker:** The remaining blocker is not stale checkpoint or replayed bundle acceptance; those are strongly closed. The residual gap is only that the inspected tests stop short of one explicit end-to-end case showing an old spent/nullifier row is rehydrated and then immediately blocks a second replay attempt in the same test narrative.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: checkpoint and replayed-bundle freshness enforcement is directly test-backed; the old spent-state-row part is supported by code-path discipline and partial tests rather than one single explicit rehydrate-then-replay regression.

### 20. Post-Scan And Post-Spend Theft Resistance

🔴 **Quest:** After Bob has already scanned the asset and later tries to spend it, what exactly prevents a bundler, aggregator, checkpoint builder, or validator-adjacent actor from stealing, rewriting, or replaying the coin, and which part of that answer is still narrower than a final trustless-verifier claim?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** After Bob has scanned the asset, theft or rewrite is prevented by a composition of wallet-local ownership/decryptability checks and the current-stack public spend/checkpoint boundary. Wallet-local ownership still requires the real `receiver_secret`, successful recomputation of `k_dh`, matching `owner_tag`, successful pack decryptability, and in the stronger two-factor path the correct `s_out`; public spend and stage11 then bind the later handoff to `prev_root`, exact input refs, `leaf_ad` integrity, exact outputs, balance/range checks, and receiver-bound authorization signature. The answer remains partial because the repository itself does not claim that this composition is already a final end-to-end trustless verifier for every wallet-local ownership invariant or a full nullifier-complete spend theorem.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/stealth/output.rs` proves the wallet-local layer is real and receiver-secret-gated. `verify_owner_two_factor(...)` requires `receiver_secret`, re-derives `k_dh`, rechecks `owner_tag`, and verifies the expected `s_out` instead of treating a loose tag hit as sufficient ownership.
2. The same module also distinguishes a weaker M1 owner-tag filter from stronger ownership, so the repository does not confuse “looks mine” with “can actually prove spendable ownership.”
3. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` proves Bob’s scan/decrypt path does more than classify metadata. It recomputes the canonical decrypt-associated-data boundary, checks tag material, decrypts the pack, and verifies commitment-opening consistency before returning owned output state.
4. `crates/z00z_wallets/src/core/address/claim_own.rs` routes wallet ownership through successful scan/decrypt results rather than through a weaker compatibility shortcut.
5. `crates/z00z_simulator/tests/test_stage4_output_crypto.rs` proves correct receiver keys pass and wrong keys fail on live Stage 4 outputs, while Stage 4 tamper hooks also fail when output crypto material is corrupted.
6. `crates/z00z_wallets/src/core/tx/spend_rules.rs` and `crates/z00z_wallets/src/core/tx/witness_gate.rs` prove the spend-preparation layer still depends on real receiver-secret-bound decryptability and owner-tag/associated-data consistency before it builds and verifies the public spend contract.
7. `crates/z00z_wallets/src/core/tx/spend_verification.rs` proves the public spend layer is real: it binds spend acceptance to nonzero `prev_root`, canonical input pairing, `leaf_ad` hashes, canonical outputs, range proofs, balance equality, duplicate/overlap guards, and receiver-bound authorization signature.
8. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` proves the validator-adjacent handoff is not free to rewrite or replay the spend after Bob owns it. Stage 11 rechecks the original stage4 spend contract and then requires exact proof/input/output continuity before checkpoint draft emission.
9. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves tampered exec rows, tampered package proof, tampered package digest, and replay-style exec reuse all fail closed and do not produce fresh checkpoint acceptance artifacts.
10. `crates/z00z_simulator/tests/test_e2e_phase4.rs` proves duplicate-delivery handling is stable on the wallet side, so repeated delivery does not silently create a second accepted owned state for the same asset after reload.

**Reasoning:**

- A bundler or validator-adjacent actor cannot simply steal the coin after Bob scans it by rewriting metadata, because Bob-side ownership is tied to receiver-secret-gated decryptability, not to a public marker alone.
- The actor also cannot silently push a rewritten or replayed later spend through the accepted current-stack path, because the public spend verifier and stage11 handoff bind acceptance to concrete previous-root, input, output, and authorization semantics and reject tamper or replay fail closed.
- That said, the security story is layered rather than absolute. The repository’s own honest closeout keeps the current answer narrower than a finished trustless verifier theorem, because wallet-local two-factor ownership has not yet been fully lifted into one universal validator-grade proof and the regular spend contract still lacks nullifier semantics.

**Gap Or Blocker:** The remaining shortfall is not the absence of protection. The shortfall is that the delivered protections are still compositional and current-stack scoped rather than one final end-to-end trustless public theorem. The repository explicitly keeps both the broader wallet-local ownership theorem and the broader nullifier-complete `PH32-SPEND` contract open.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the accepted current-stack path strongly resists rewrite and replay after Bob scans, but the repository does not yet claim that every wallet-local/two-factor ownership rule is already subsumed by one final validator-grade trustless proof boundary.

## 🔒 Theme 5: Secret Hygiene, RNG Boundaries, And Documentation Honesty

### 21. Default Secret Silence

🔴 **Quest:** What proves that a default Scenario 1 run no longer emits a public plaintext wallet-secret artifact, and is there any remaining ordinary output lane where secret-bearing material can still persist without an explicit debug-only decision?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository directly proves that a default Scenario 1 run no longer emits the old public plaintext wallet-secret markdown artifact. The public path `wallets/wlt_secrets_debug.md` must stay absent, and the private markdown secret table is not emitted unless `wallet_debug_dump` is enabled. What remains only partial is the second half of the question: ordinary Stage 2 still persists normal wallet-state artifacts and backup/export lanes, but the reviewed Stage 2 files do not prove that a single remaining ordinary lane can be named as the unique secret-bearing persistence surface. The honest answer is therefore: public plaintext secret export is gone by default, while ordinary wallet-state persistence still exists in non-public operational lanes.

**Evidence Trail:**

1. `crates/z00z_simulator/tests/test_stage2_secret_artifacts.rs` proves the public plaintext path is gone. `stage2_secrets_skip_public_path()` requires that `wallets/wlt_secrets_debug.md` does not exist after a normal run.
2. The same test file proves the private markdown artifact is feature-gated. With `wallet_debug_dump` enabled, `stage2_debug_secrets_stay_private()` requires `wallets/private/wlt_secrets_debug.md`, mode `0600`, and an explicit `[DEBUG]` banner. Without that feature, `stage2_secrets_need_debug_dump()` requires the private secret file to stay absent.
3. `crates/z00z_simulator/src/config_accessors.rs` binds the Stage 2 markdown secret artifact to `cfg!(feature = "wallet_debug_dump")` and only returns a path under `wallets/private/wlt_secrets_debug.md`.
4. `crates/z00z_simulator/src/scenario_1/stage_2.rs` preserves the same boundary at runtime: it writes the markdown secret table only when the optional private path exists, and otherwise logs that the default lane emitted no plaintext wallet secret artifact.
5. `crates/z00z_simulator/README.md` and `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` both freeze that policy at the contract/document layer: plaintext wallet-secret artifacts are not part of the default public scenario contract.
6. The remaining ordinary persistence surfaces are still real. `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs` computes a normal `wallet_*.wlt` path and verifies it through `open_wallet_source(...)`, and the same file runs ordinary export/import and encrypted backup roundtrips as part of default Stage 2 execution.

**Reasoning:**

- The repository no longer allows the old public markdown secret table to appear on the default wallet lane.
- It also does not silently fall back to the private markdown file in default mode; that lane is absent unless a debug-only feature is turned on.
- But Stage 2 still must persist normal wallet-state artifacts and run operational wallet export or backup behavior, so the honest story is not “no secret-bearing state is ever persisted.”
- The honest closed claim is narrower: default Scenario 1 no longer emits a public plaintext wallet-secret artifact.

**Gap Or Blocker:** The repository strongly proves removal of the public plaintext secret artifact, but it does not prove that one single ordinary artifact can be named as the unique remaining secret-bearing persistence lane. The normal `.wlt` store, export roundtrip, and encrypted backup roundtrip are all live operational surfaces in the reviewed Stage 2 code.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the partial status comes from the second half of the question about remaining ordinary secret-bearing persistence, not from any doubt that the default public plaintext markdown artifact is gone.

### 22. Explicit Debug Lane Only

🔴 **Quest:** Where is the secret-export lane still reachable, and what exact controls keep it explicit, private, and non-default rather than silently coupled to normal scenario execution?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The remaining secret-export lane is reachable only through the explicit `wallet_debug_dump` feature, and the repository pins that lane as debug-only, private-path-only, and non-default. It is not silently coupled to normal scenario execution: the simulator feature set defaults to empty, the Stage 2 secret path resolves only when the feature is enabled, the writer uses the private-write API, the produced file is branded as `[DEBUG]`, and the tests assert both private-path-only behavior and absence when the feature is off. The same feature gate also controls later Stage 3 wallet debug dump export helpers.

**Evidence Trail:**

1. `crates/z00z_simulator/Cargo.toml` makes the feature non-default. `default = []`, and `wallet_debug_dump` is documented as a debug-only feature with a `SECURITY: NEVER enable in production builds` warning.
2. `crates/z00z_simulator/src/config_accessors.rs` gates the Stage 2 secret artifact path through `cfg!(feature = "wallet_debug_dump")` and resolves it only to `wallets/private/wlt_secrets_debug.md`.
3. `crates/z00z_simulator/src/scenario_1/stage_2.rs` calls `debug_write_wallet_secrets_md(...)` only when that optional private path exists; otherwise it logs that the default lane emitted no plaintext artifact.
4. `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` makes the file unmistakably debug-only: the markdown header is `# Wallet Secrets (Stage 2) [DEBUG]`, the warning text says the artifact is sensitive, and the write path uses `atomic_write_file_private(...)`.
5. `crates/z00z_utils/src/io/atomic_write.rs` proves the private-write contract is real on Unix: `atomic_write_file_private(...)` sets mode `0o600` before persisting the file.
6. `crates/z00z_simulator/tests/test_stage2_secret_artifacts.rs` proves the contract operationally: the public path must stay absent, the private file must exist only with `wallet_debug_dump`, its mode must be `0600`, and the debug banner must be present.
7. `crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs` shows the later wallet debug export helpers are also behind `#[cfg(feature = "wallet_debug_dump")]`, so post-claim wallet debug dumps are not part of normal execution either.
8. `crates/z00z_simulator/README.md` repeats the same policy in prose: any retained debug-only secret artifact must be behind the feature gate, written to a private-permission path, and absent from the default release-style contract.

**Reasoning:**

- The remaining secret-export lane is not merely discouraged; it is feature-gated, path-gated, and permission-gated.
- Normal scenario execution does not automatically resolve the secret markdown path, and the runtime branch explicitly records the no-artifact default case.
- Even when enabled, the artifact is routed to a private subdirectory, branded as debug-only, and written through the private-write helper rather than through a normal public file path.
- The same gating discipline carries forward into Stage 3 wallet debug dump exports.

**Gap Or Blocker:** None on the requested scope. The repository directly proves the remaining secret-export lane is explicit, private, and non-default.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: none for the requested simulator/debug-lane scope.

### 23. Seeded RNG Stays Bounded

🔴 **Quest:** What repository evidence proves that seeded randomness remains confined to simulator-only reproducibility behavior and cannot silently become part of a production-looking entropy story?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The repository strongly bounds seeded randomness to simulator or test reproducibility flows, but the honest answer is still partial rather than absolute. The positive side is clear: simulator config, simulator RNG mode selection, Stage 2 transport wiring, RNG-provider docs, compile guards around `MockRngProvider`, and transport-boundary tests all say that seeded randomness is for deterministic simulation or testing while ordinary wallet entropy uses `SystemRngProvider`. The reason this remains partial is that the confinement is enforced mostly by simulator-only scoping and call-path discipline, not by a repository-wide impossible-by-construction type boundary: the simulator-local deterministic `SeqSecureRngProvider` still implements `SecureRngProvider`.

**Evidence Trail:**

1. `crates/z00z_simulator/src/config.rs` states the intended split directly: `use_mock_rng` is deterministic RNG for CI, while `mock_rng_seed: None` corresponds to `SystemRngProvider` maximum-randomness mode.
2. `crates/z00z_simulator/src/rng_mode.rs` repeats the same mapping in code and docs: `Some(seed)` becomes deterministic mock RNG, `None` becomes system RNG.
3. `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` keeps the seeded path inside simulator-only code. When `simulation.use_mock_rng` is false, it constructs the wallet service through the ordinary `WalletService::with_output_dir(...)` path; when mock mode is on, it uses the simulator-local deterministic provider.
4. The same file also shows an important nuance: in Stage 2 mock mode, missing seed falls back to `0`, so the simulator still treats that path as deterministic reproducibility, not as system entropy.
5. `crates/z00z_simulator/tests/test_transport_rng_boundaries.rs` proves same-seed reproducibility, `None == Some(0)` in mock mode, and different-seed divergence.
6. `crates/z00z_utils/src/rng/traits.rs` distinguishes unpredictable production RNG from deterministic reproducibility-only RNG and includes explicit security warnings that deterministic RNG must not be used for nonces, ephemeral secrets, or salts.
7. `crates/z00z_utils/src/rng/system.rs` defines `SystemRngProvider` as the production RNG using `OsRng`.
8. `crates/z00z_utils/src/rng/mock.rs` keeps `MockRngProvider` behind compile-time guards that reject it in production builds and documents it as deterministic testing infrastructure.
9. `crates/z00z_wallets/src/services/wallet_service_session_build.rs` and `crates/z00z_wallets/src/services/app_seed_password.rs` show the ordinary wallet entropy path is system-backed in normal service construction and seed-phrase generation.
10. `crates/z00z_simulator/README.md` and `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` both preserve the honest scoping: seeded RNG behavior is a simulator fixture boundary and does not support a production entropy claim.

**Reasoning:**

- On the ordinary path, the wallet service uses system entropy and the utility layer documents deterministic RNG as testing or genesis-only infrastructure.
- The simulator explicitly exposes mock RNG only through simulation config and simulator-only transport wiring.
- The repository also test-locks the deterministic semantics, so seeded mode is not an accidental side effect.
- But the boundary is still scoped rather than absolute, because simulator-local code can present a deterministic provider through the `SecureRngProvider` trait shape.

**Gap Or Blocker:** The remaining blocker to a stronger claim is not missing documentation; it is that the repository does not make deterministic-as-secure impossible everywhere by type system. The bound is strong, explicit, and simulator-scoped, but not universal.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: Stage 2 mock mode treats missing seed as zero-seed determinism, and simulator-local `SeqSecureRngProvider` means the bound depends on simulator scoping and discipline rather than on a globally impossible type boundary.

### 24. Verification Discipline Versus Overclaim

🔴 **Quest:** Does the recorded verification evidence actually satisfy the stated sign-off discipline for this phase, and where do the artifacts themselves admit that targeted clean reruns are stronger than the currently blocked or historically failing broad-suite story?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The recorded artifact story supports a narrower “yes” than a casual reader might assume. The Phase 032 closeout artifacts define the required sign-off order as bootstrap first, then named release-style simulator reruns, then a three-pass review loop with two consecutive clean passes, and they explicitly record that this narrowed targeted-closeout contract was followed. At the same time, those same artifacts also say the broader workspace release-suite story is not authoritative closeout evidence: historical manifests still contain `RESULT[18]=FAIL`, a fresh full-suite rerun hit host-level disk exhaustion, and the long-running report is supporting evidence only. The reason this answer stays partial is that the support is mostly self-reported at the closeout-artifact layer rather than independently re-proved by raw primary logs inside the same artifact set.

**Evidence Trail:**

1. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` defines the required sign-off order exactly: bootstrap first, then named release-style simulator validation, then the review loop.
2. The same file records that the review-loop equivalent was executed manually through `crypto-architect`, `security-audit`, and `doublecheck` criteria, and that the last two review passes were consecutive clean runs.
3. The same closeout also lists the targeted validation commands as the actual evidence surface for this wave, including bootstrap, `test_checkpoint_acceptance`, `test_stage2_secret_artifacts`, and `test_transport_rng_boundaries`.
4. `docs/code-review/032-scenario-1-crypto-status.md` repeats the same verification discipline publicly: bootstrap first, then release-style simulator validation, then the review loop.
5. `.planning/phases/032-crypto-audit-scenario-1/032-07-SUMMARY.md` explicitly narrows the evidence surface further: targeted review-fix validation is clean, historical checked-in manifests still contain `RESULT[18]=FAIL`, and a fresh full-suite rerun hit `No space left on device`.
6. `032-HONEST-CLOSEOUT.md` and `032-07-SUMMARY.md` both demote the broad suite to non-authoritative status for this closeout and say the long-running report is supporting evidence only.
7. `reports/full_verify-report-long-running-tests.txt` independently supports the broader failure or abort story, but it does not itself overturn the closeout’s decision to treat targeted reruns as the authoritative evidence surface.

**Reasoning:**

- The honest closeout does not say “the whole workspace broad suite passed, therefore the phase is closed.”
- Instead, it says the required sign-off discipline is satisfied at the targeted-closeout level and then carefully explains why broad-suite success cannot be claimed as settled evidence.
- That is exactly the anti-overclaim move the question asks about: targeted clean reruns are treated as stronger and more honest than a blocked or historically drifting broad-suite narrative.
- The limitation is that this is still mainly artifact-declared sign-off rather than first-order proof from raw log bundles embedded in the same reviewed file set.

**Gap Or Blocker:** The remaining blocker to a fully stronger claim is evidentiary shape. The closeout artifacts record that the sign-off contract was followed, but they are still summaries rather than a complete set of primary verification logs in one place.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the sign-off discipline is well supported as the honest targeted-closeout artifact story, but not fully independently re-proved from raw first-order logs inside the same evidence bundle.

### 25. Is The Whole Scheme Really Secure

🔴 **Quest:** If someone summarized Scenario 1 as “the full `Alice -> leaf build -> JMT publish -> Bob scan -> spend -> validator` scheme is secure and cryptographically closed end to end,” which parts of that sentence can actually be proved from the repository, which parts remain only partial, and which parts would be overclaim if the caveats were ignored?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The honest repository-level answer is a three-way classification, not a yes-or-no slogan.

- **What can actually be proved:** the accepted current-stack path has real cryptographic and verifier-backed seams. Claim packages bind the accepted claim statement to the canonical source-root contract; output build, scan, wallet-local ownership checks, and spend preparation are all real and test-backed at their current scope; the current public spend contract is live and enforced; Stage 11 closes placeholder proof and placeholder spent-state success on the accepted checkpoint handoff path; default plaintext wallet-secret artifact emission is removed; and seeded RNG is explicitly treated as simulator-scoped reproducibility behavior rather than as a production guarantee.
- **What remains only partial:** persisted storage-backed claim-membership continuity for the broader original claim-trust wording; a validator-grade end-to-end Bob-only ownership theorem; some replay and stale-artifact closure surfaces; the broader original `PH32-SPEND` contract because regular spend still lacks nullifier semantics; and the fact that the whole chain is still layered rather than one finished trustless theorem.
- **What would be overclaim if caveats were ignored:** saying the whole flow is “cryptographically closed end to end” as one final theorem; claiming live STARK, FRI, recursive checkpoint proof, or on-chain verifier closure; claiming full `PH32-SPEND` or full `PH32-CLAIM-TRUST` closure; claiming sender ignorance of all output-secret material; or claiming censorship resistance, withheld-data recovery, or production entropy guarantees.

**Evidence Trail:**

1. `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md` is the canonical honest boundary file. It lists what Scenario 1 now proves, what it does not prove, and what remains explicitly out of scope.
2. `docs/code-review/032-scenario-1-crypto-status.md` repeats the same three-bucket logic in user-facing form: delivered current-tree claims, claims not made, and explicit out-of-scope items.
3. `.planning/phases/032-crypto-audit-scenario-1/032-04-SUMMARY.md` keeps the key spend caveat explicit: the original broader `PH32-SPEND` wording remains open until nullifier semantics are implemented or the requirement is narrowed.
4. The already-solved exam rows in this same file support the same layered classification: Q10, Q12, Q15, Q18, Q19, and Q20 are all narrower or partial by design, while Q11, Q13, Q14, Q16, and Q17 describe specific seams that are fully closed at their current scope.
5. The same closeout files also explicitly reject whole-chain overclaims such as live STARK/FRI, recursive proof-backend closure, sender-ignorance absolutism, censorship resistance, withheld-data recovery, and broader production entropy claims.

**Reasoning:**

- The repository does not support one blanket sentence saying the entire Alice-to-validator chain is fully and finally closed.
- What it does support is a layered story: several important seams are real, current-stack, and well tested, while some broader invariants remain only partially delivered or explicitly out of scope.
- That is why the correct answer must separate delivered local closures from broader chain-wide trustless claims.
- Any summary that collapses those buckets into one triumphant sentence becomes misleading.

**Gap Or Blocker:** The whole-chain claim stays partial because the repository still carries live caveats on broader claim-trust continuity, full Bob-only validator-grade ownership closure, nullifier semantics, replay or stored-state completeness on every surface, and final trustless proof-backend architecture.

**Verification:**

- `doublecheck` status: PARTIAL
- Residual caveat: the three-way classification is the right honest shape, but some elements that casual summaries might fold into “partial” are actually stronger than that in the closeout files: they are explicitly not proved or out of scope, and must stay in the overclaim bucket if asserted positively.

## Summary Table

| Q | Title | Proof Status | Verification | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- |
| 1 | Signed Root Or Unsigned Baggage | Full Evidence | VERIFIED | None | None |
| 2 | Full Tuple Or Partial Story | Partial Evidence | PARTIAL | Missing targeted negative tests for `claim_source_asset_id` and post-sign `chain_id` drift at the end-to-end verifier boundary | Add explicit verifier tests that mutate those fields after signing and assert the reject class |
| 3 | Authoritative Store Or Local Reconstruction | Partial Evidence | PARTIAL | Accepted flow still depends on synthetic one-item claim-membership reconstruction instead of persisted storage-backed continuity | Move claim-source root/proof derivation onto persisted store-backed state or formally narrow `PH32-CLAIM-TRUST` |
| 4 | Precise Reject Semantics | Partial Evidence | PARTIAL | Missing dedicated end-to-end tests for verifier-side root-version and proof-version branches; stale-proof consumer test allows multiple fail-closed substrings | Add explicit verifier tests for `SourceRootVer` and `SourceProofVer`, and stabilize consumer stale-proof classifier expectations |
| 5 | Publish-Bound Claim Continuity | Partial Evidence | PARTIAL | Bundle-wrapper version is not enforced on load, and some discriminator values may arrive through serde defaults instead of explicit serialized fields | Enforce bundle version at load time and add serialization-shape tests that require explicit canonical discriminator fields |
| 6 | Sender Knowledge Versus Anti-Theft | Partial Evidence | PARTIAL | Receiver-secret-gated two-factor ownership is proven only at wallet-local / spend-rule / witness-prep seams, not as a finished validator-level trustless invariant | Deliver a validator-level public proof boundary that proves the same receiver-secret-gated exclusion end to end |
| 7 | Canonical Output-Secret Semantics | Partial Evidence | PARTIAL | Accepted live code is frozen on deterministic `derive_s_out(k_dh, r_pub, serial_id)`, but legacy temp docs still describe a competing `random32` model | Remove or mark obsolete the legacy `random32` artifacts so the repository has one explicit output-secret model everywhere |
| 8 | Associated-Data Identity Freeze | Partial Evidence | PARTIAL | Canonical owned/spendable flow rejects or loses ownership on `leaf_ad_id` drift, but the stronger universal claim about every crafted artifact surface is not separately pinned | Keep the claim scoped to canonical flow or add broader artifact-surface tests that prove no independent acceptance seam survives `leaf_ad_id` divergence |
| 9 | Request Privacy Versus Card Fallback | Partial Evidence | PARTIAL | Request-bound metadata is the preferred privacy lane for accepted request or invoice flows, but card-bound or base build and scan paths remain live compatibility behavior rather than a removed stub | Either demote card-bound APIs and scan order to a clearly non-default compatibility mode or add proof that the remaining card-only lane carries equivalent privacy guarantees |
| 10 | End-To-End Ownership Through The Chain | Partial Evidence | PARTIAL | Bob-only ownership stays stable across accepted build, publish, scan, and Bob-secret-gated spend preparation, but the validator verifies a public spend contract rather than independently re-proving the same two-factor ownership invariant end to end | Deliver a validator-facing proof boundary that closes receiver-secret plus output-secret ownership, or formally narrow the chain-wide ownership claim to the current accepted spend-contract scope |
| 11 | What The Current Public Boundary Actually Proves | Full Evidence | VERIFIED | Current public spend boundary is real and test-backed, but intentionally narrower than full PH32-SPEND or universal trustless-verifier closure | Distinguish delivered public-contract checks from still-open nullifier and end-to-end ownership claims |
| 12 | Theft Windows Before And After Publication | Partial Evidence | PARTIAL | Local receiver-bound ownership and handoff tamper rejection are strong, but pre-publication withholding and full public anti-theft closure remain open | Keep theft prevention separate from withholding and from the still-open universal public-verifier claim |
| 13 | Proof Continuity Across Handoff | Full Evidence | VERIFIED | Accepted current-stack path preserves spend meaning through stage11 via package revalidation and exact proof/input/output continuity; later exec artifacts stay package-coupled rather than standalone auth carriers | Keep scope on accepted package-coupled path; do not overstate standalone checkpoint authorization |
| 14 | The Requirement That Remains Open | Full Evidence | VERIFIED | The exact missing element is nullifier semantics inside the regular spend public contract; current proof/auth boundary is real but PH32-SPEND stays open until nullifier binding is added or the requirement is narrowed | Keep answer narrowly on the nullifier gap; do not imply proof/auth or current spend verification is missing in general |
| 15 | Full-Chain Crypto Closure Versus Partial Security | Full Evidence | VERIFIED | The honest whole-chain story is layered: real crypto exists at claim/stealth/scan/spend seams, while ownership continuity, checkpoint handoff, and replay/spent gating stay partly structural and major trustless-proof claims remain open | Classify by bucket; do not flatten the validator leg into fully live crypto or claim full end-to-end trustless closure |
| 16 | Placeholder Success Paths Truly Closed | Full Evidence | VERIFIED | Checkpoint acceptance is now bound to stage4 package revalidation plus exact proof/input/output continuity and replay-consistent spent-state checks; placeholder proof/spent-state success lanes are closed on the accepted path | Keep scope on accepted current-stack checkpoint path; do not expand into recursive-proof or full PH32-SPEND claims |
| 17 | Draft Versus Final Truth | Full Evidence | VERIFIED | Draft and final remain separated by class-specific loading, proof-bound finalization, and canonical statement/id/link checks; draft-only outputs do not masquerade as final authoritative state | Distinguish draft-vs-final separation from the separate existence of explicit legacy-final compatibility artifacts |
| 18 | Injective Persistence Contract | Partial Evidence | PARTIAL | Raw checkpoint artifacts are not fully injective by proof payload because checkpoint identity ignores cp_proof bytes, while the canonical RedB-persisted path adds stronger fail-closed binding to statement, exec identity, and bound root | Separate the weak raw artifact surface from the stronger canonical persisted rehydrate path |
| 19 | Replay And Stale-Artifact Resistance | Partial Evidence | PARTIAL | Stale checkpoint tuples and replayed proof-bearing bundles are fail-closed by id/root/link/rehydrate checks, while old spent-state rows are strongly code-backed but not covered by one explicit rehydrate-then-replay end-to-end test | Keep the strong replay/bundle closure, but state the spent-row portion more narrowly |
| 20 | Post-Scan And Post-Spend Theft Resistance | Partial Evidence | PARTIAL | Post-scan/post-spend theft resistance is real but layered: receiver-secret-gated wallet ownership plus current-stack public spend and stage11 replay/tamper checks block theft on the accepted path, while the final trustless ownership/nullifier theorem remains open | Describe the defense as compositional; do not collapse it into a finished universal trustless-verifier claim |
| 21 | Default Secret Silence | Partial Evidence | PARTIAL | The default public plaintext wallet-secret markdown artifact is gone, but the reviewed Stage 2 files do not prove a single unique remaining ordinary secret-bearing persistence lane because normal wallet-state, export, and encrypted backup surfaces still exist | Keep the closed claim narrow: no default public plaintext secret artifact; do not overstate ordinary persistence as fully eliminated |
| 22 | Explicit Debug Lane Only | Full Evidence | VERIFIED | The remaining secret-export lane is explicitly feature-gated, private-path-only, permission-guarded, debug-branded, and absent by default across Stage 2 and later debug helpers | Keep the claim on the simulator debug-export surface; do not generalize it into unrelated wallet persistence behavior |
| 23 | Seeded RNG Stays Bounded | Partial Evidence | PARTIAL | Simulator and utility layers strongly bound seeded reproducibility to simulator/test flows, but simulator-local deterministic `SeqSecureRngProvider` still presents through a secure-RNG trait shape, so the boundary depends on scoping and discipline rather than on a universal impossible-by-construction type rule | State the bound as strong and simulator-scoped, not absolute; include the `None == zero-seed` mock-mode nuance |
| 24 | Verification Discipline Versus Overclaim | Partial Evidence | PARTIAL | The closeout artifacts record bootstrap-first targeted reruns and review-loop completion, and they explicitly demote the broad-suite story, but this is still mostly self-reported at the closeout-artifact layer rather than fully re-proved from a complete raw-log bundle in the same evidence set | Treat the sign-off as an honest targeted-closeout contract, not as a claimed broad-suite PASS narrative |
| 25 | Is The Whole Scheme Really Secure | Partial Evidence | PARTIAL | The honest answer is a three-way classification: some current-stack seams are fully delivered, several broader chain-wide invariants remain partial, and multiple stronger claims are explicitly not proved or out of scope; the blanket “cryptographically closed end to end” sentence is overclaim if stated without caveats | Preserve the three-bucket summary and keep “not proved / out of scope” items out of any positive end-to-end closure claim |
