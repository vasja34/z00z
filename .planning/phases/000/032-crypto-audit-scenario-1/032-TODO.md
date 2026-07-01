# Crypto Security Report

[TOC]



## Scope

📌 This report audits the concrete repository implementation for the chain `Alice -> leaf build -> JMT publish -> Bob scan -> spend -> validator`.

📌 The target question is whether the current code makes it almost formally true that no sender, aggregator, validator, or outside observer can link a leaf to a wallet or steal the asset, and that only Bob can recover and spend it.

📌 The review follows a doublecheck-style workflow: extract the claims, verify them against repository code and executed tests, then switch to an adversarial posture and look for overstatements, gaps, and hallucination-risk claims.

📌 Sources used here are repository code, repository design notes, repository memory, and executed release-mode test logs. The concept documents reviewed were `.planning/temp/Z00Z-ECC-IDEAS.md` and `.planning/temp/Z00Z-ECC-SPEC_part1.md`. They were treated as design intent, not as proof that the implementation already satisfies that intent.

## Executive Verdict

📌 The repository does implement a real stealth receive path and real Bulletproofs+ range-proof verification.

📌 The repository does not justify the full claim as originally stated.

🚨 The three most important conclusions are these.

1. Alice does know sender-side output secret material in the current implementation.
2. The spend and validator path is not yet backed by a complete public ZK verifier.
3. The design documents themselves support sender knowledge of `s_out`; the unresolved issue is not sender ignorance, but the missing finished public proof boundary.
4. A live STARK/FRI stack is not implemented in repository code.

## Final Ratings

📌 Claim register with final ratings.

1. `C1` Bob can recover his stealth output after scanning.
   VERIFIED.
   Reason: implemented and covered by send/scan and runtime-parity tests.

2. `C2` A foreign receiver cannot decrypt Bob's output.
   VERIFIED.
   Reason: explicit negative tests pass.

3. `C3` Passive leaf or JMT observers get no direct receiver identifier from the leaf payload.
   PLAUSIBLE.
   Reason: the leaf format is privacy-oriented, but this is not a formal metadata-proof.

4. `C4` Alice does not know the asset secret.
   DISPUTED.
   Reason: sender code derives `k_dh` and `s_out` during output construction.

5. `C5` Alice still cannot spend without Bob's receiver secret.
   DISPUTED.
   Reason: local ownership logic depends on `receiver_secret`, but the report cannot promote this to an end-to-end proof-backed guarantee while the public spend boundary remains incomplete.

6. `C6` Validator can trustlessly verify spend correctness with a real ZK verifier.
   DISPUTED.
   Reason: the current spend gate is structural and placeholder-like, not a full public proof verifier.

7. `C7` STARK/FRI ZK is implemented and working in the live stack.
   FABRICATION RISK.
   Reason: it appears in `.todo` and concept docs, not in active proof code or dependencies.

8. `C8` Checkpoint or JMT publish is already backed by authoritative proof verification.
   DISPUTED.
   Reason: the storage or checkpoint path enforces non-empty proof bytes, but not full cryptographic proof validation.

## Severity Findings

### S0 Critical

🚨 `STARK/FRI is implemented and working` is not supported by the repository.

📌 Evidence.

1. `crates/z00z_crypto/src/claim/proof.rs` says `Placeholder witness container for the first typed claim-proof API slice`.
2. `crates/z00z_crypto/src/claim/prover.rs` says `Build the first canonical typed placeholder proof for a genesis claim statement`.
3. `crates/z00z_crypto/src/claim/verifier.rs` says `Verify the canonical placeholder proof against the canonical statement hash`.
4. `crates/z00z_crypto/.todo/pq-Whitepaper.md` contains FRI and STARK references as future architecture, but no corresponding active proof stack surfaced in Cargo manifests.

📌 Impact: any statement that validators currently verify spend or checkpoint correctness through a live STARK/FRI proof system would be materially overstated.

### S1 High

🚨 `Alice does not know the asset secret` is false for the current sender flow.

📌 Evidence.

1. `crates/z00z_wallets/src/core/stealth/output.rs` derives `k_dh` in `derive_mat(...)`.
2. The same file derives `s_out = derive_s_out(&build_mat.k_dh, &build_mat.r_pub, serial_id)` before encrypting the pack.
3. `crates/z00z_simulator/src/scenario_1/stage_3.rs` shows the same sender-side derivation of `k_dh`, `owner_tag`, and `s_out`.

📌 Impact: the privacy statement must be rewritten honestly. The sender knows sender-generated secret material for the output it creates. The theft-resistance story must therefore come from an additional receiver-secret requirement at spend time, not from sender ignorance.

🚨 The ideas-document sentence `Alice still cannot steal because Spend-TxProof requires receiver_secret` matches the intended B3 ownership model, but it is stronger than the live proof boundary.

📌 Evidence.

1. `.planning/temp/Z00Z-ECC-IDEAS.md` explicitly says sender may know everything it generated, explicitly states that even if sender knows `s_out` it still cannot spend without `receiver_secret`, and even includes a sender-side `s_out = random32` step.
2. `.planning/temp/Z00Z-ECC-SPEC_part1.md` defines `s_out` as a random asset secret and separately states that sender knows everything about the output but still cannot spend without `receiver_secret`.
3. `crates/z00z_wallets/src/core/stealth/output.rs` derives `k_dh` and `s_out` on the sender side.
4. The same module exposes `verify_owner_two_factor(...)`, which really does require `receiver_secret` plus `s_out` for spend-level ownership checks.
5. `crates/z00z_wallets/src/core/tx/spending.rs` follows the same receiver-secret-gated rule model.
6. But `crates/z00z_wallets/src/core/tx/witness_gate.rs` and `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` still leave the final proof boundary structural or placeholder-like rather than fully authoritative.

📌 Impact: the correct statement today is that receiver-secret-gated spend authorization is part of both the intended design and the local rule logic, but the repository does not yet justify the stronger end-to-end claim that Alice is cryptographically excluded by a finished public spend proof.

🚨 The validator-facing spend or checkpoint trust model is not yet cryptographically complete.

📌 Evidence.

1. `crates/z00z_wallets/src/core/tx/spending.rs` verifies spend rules directly from witness data and receiver-secret derivation.
2. `crates/z00z_wallets/src/core/tx/witness_gate.rs` implements `SpendCs`, but its checks are structural: non-zero secrets, non-empty commitments, and basic balance presence, not a complete public proof verification.
3. Repository memory already captured that `cp_proof` is accepted as non-empty bytes and that synthetic proof bytes are emitted in the storage or checkpoint path.
4. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` explicitly marks parts of checkpoint aggregation as placeholder-only.

📌 Impact: the current code does not support the strong claim that an untrusted validator can fully verify privacy-preserving spend correctness from a real proof object alone.

🚨 The ideas-document sentence `aggregator cannot steal because it lacks Bob's secrets` is not the property currently enforced at the authoritative checkpoint boundary.

📌 Evidence.

1. `crates/z00z_storage/src/checkpoint/build.rs` does enforce real membership-witness consistency for resolved inputs before applying state transitions.
2. The same path delegates spend-proof acceptance through `proof_chk.verify_tx(tx)?` before deleting old leaves and inserting new ones.
3. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` currently implements that verifier as a non-empty-proof check and explicitly documents placeholder checkpoint aggregation.
4. `crates/z00z_storage/src/checkpoint/artifact.rs` and `crates/z00z_storage/src/checkpoint/codec.rs` reject empty proof bytes, but that is not equivalent to a real Bob-secret-gated authorization proof.

📌 Impact: today the aggregator is constrained by membership and batch-shape checks, but the report cannot honestly say the aggregator is prevented from theft specifically because it lacks Bob's secrets. That anti-theft statement remains blocked on a real proof verifier.

### S2 Medium

⚠️ `leaf_ad_id` semantics are not yet strong enough to support a nearly formal statement without caveats.

📌 Evidence.

1. `crates/z00z_core/src/assets/assets.rs` defines `leaf_ad_id` as an optional decrypt associated-data identifier and requires it when full stealth fields are present.
2. `crates/z00z_wallets/src/core/tx/witness_gate.rs` rebases `wire_decrypt_leaf()` so that `leaf.asset_id = leaf_ad_id` for decryption.
3. Runtime parity tests set `asset.leaf_ad_id = Some(leaf.asset_id)` in multiple places.
4. The coexistence of `asset.asset_id()` and `leaf_ad_id()` means the decrypt contract is subtle and must be frozen canonically.

📌 Impact: this is not a demonstrated break by itself, but it is exactly the sort of boundary ambiguity that blocks strong formal claims and makes proof, wire, and runtime drift more likely.

⚠️ Targeted scan-DoS remains relevant when `tag16` is card-bound rather than request-bound.

📌 Evidence.

1. `crates/z00z_wallets/src/core/stealth/output.rs` uses card-bound `tag16` when no payment request is present.
2. The idea documents correctly warn that public-view-key-derived tags can enable targeted spam and recommend request-bound tag derivation.

📌 Impact: this does not directly enable theft, but it weakens operational privacy and receiver robustness.

⚠️ The documents and the live code agree that sender can know `s_out`, but they currently diverge on how `s_out` is produced.

📌 Evidence.

1. `.planning/temp/Z00Z-ECC-IDEAS.md` describes `s_out = random32` in the sender workflow.
2. `.planning/temp/Z00Z-ECC-SPEC_part1.md` defines `s_out` as a random 32-byte asset secret.
3. `crates/z00z_wallets/src/core/stealth/ecdh.rs` defines the canonical implementation as `derive_s_out(k_dh, r_pub, serial_id)`.
4. `crates/z00z_wallets/src/core/stealth/output.rs` uses that deterministic derivation during output construction.

📌 Impact: this is not evidence that sender should be ignorant of `s_out`; both document and code allow sender knowledge. The real issue is design drift in `s_out` derivation semantics, which should be frozen canonically before stronger proof claims are made.

## Claim Register

### C1. Bob can recover his stealth output after scanning

📌 Repository evidence supports this claim.

📌 Code path.

1. Sender constructs `k_dh`, `owner_tag`, `leaf_ad`, encrypted pack, and `tag16` in `crates/z00z_wallets/src/core/stealth/output.rs`.
2. Receiver scan uses `scan_owned`, `compute_kdf_ad`, `decrypt_pack`, `parse_pack`, and `verify_pack_commitment` in `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`.
3. Runtime scanner uses `scan_leaf`, `scan_with_tag`, `scan_direct`, and `scan_report` in `crates/z00z_wallets/src/core/address/stealth_scanner.rs`.

📌 Executed evidence.

1. `/tmp/wallets_e2e_send_scan.log` contains `test_e2e_send_scan ... ok`.
2. `/tmp/wallets_tx_stealth_flow.log` contains `test_stage4_alice_sends_asset_to_bob ... ok`, `test_stage4_sender_receiver_roundtrip ... ok`, `test_stage4_runtime_roundtrip ... ok`, and `test_stage4_path_parity ... ok`.
3. `/tmp/wallets_runtime_parity.log` contains `test_e2e_runtime_own ... ok`.
4. `/tmp/wallets_s6_recv.log` contains `test_ex1_leaf_walk ... ok`, `test_ex2_runtime_scan ... ok`, and `test_ex5_req_merchant ... ok`.

📌 Rating: VERIFIED.

### C2. A foreign receiver cannot decrypt Bob's output

📌 Repository evidence supports this claim for the tested scenarios.

📌 Code and test evidence.

1. `crates/z00z_wallets/tests/test_tx_stealth_flow.rs` includes `test_stage4_carol_cannot_scan_bob_asset`.
2. `/tmp/wallets_tx_stealth_flow.log` confirms that test passed.
3. `/tmp/wallets_runtime_parity.log` confirms `test_e2e_runtime_foreign ... ok`.

📌 Rating: VERIFIED.

### C3. Passive observers cannot directly map a leaf to a wallet

📌 This claim is broadly consistent with the current leaf design, but it is not proven in a formal metadata model.

📌 Supporting points.

1. The leaf and runtime stealth tuple is centered on `r_pub`, `owner_tag`, `enc_pack`, commitment, and `tag16` rather than an explicit wallet identifier.
2. `validate_stealth_consistency()` in `crates/z00z_core/src/assets/assets.rs` enforces a coherent stealth tuple.
3. The tested scanner path requires Bob's receiver secrets to derive the right key and tag.

📌 Limitation.

1. This does not amount to a formal anonymity proof against timing, amount pattern, request reuse, directory metadata, or network side channels.

📌 Rating: PLAUSIBLE.

### C4. Alice does not know the output secret

📌 This claim is contradicted by both the sender implementation and the reviewed design documents.

📌 Supporting evidence.

1. `.planning/temp/Z00Z-ECC-IDEAS.md` explicitly includes sender-side `s_out = random32` and separately states that sender may know everything it generated.
2. `.planning/temp/Z00Z-ECC-SPEC_part1.md` defines `s_out` as an asset secret and states that sender knows everything about the output but still cannot spend without `receiver_secret`.
3. Sender derives `k_dh` and then derives `s_out` before encrypting the pack in `crates/z00z_wallets/src/core/stealth/output.rs`.
4. `crates/z00z_simulator/src/scenario_1/stage_3.rs` shows the same sender-side logic explicitly.

📌 Corrected statement.

1. Alice knows sender-generated or sender-available output secret material for the output she creates.
2. The intended protection in both docs and code is not sender ignorance, but that spending should additionally require receiver-secret knowledge.

📌 Rating: DISPUTED.

### C5. Alice still cannot spend Bob's coin

📌 The repository points in this direction at the local rule level, but the statement is too strong as an end-to-end implementation claim.

📌 Supporting evidence.

1. `crates/z00z_wallets/src/core/tx/spending.rs` derives the owner handle and view key from `receiver_secret`, recomputes the DH-derived key, rechecks the owner tag, rechecks asset id from `s_in`, and enforces balance plus range condition.
2. `crates/z00z_wallets/src/core/stealth/output.rs` implements `verify_owner_two_factor(...)`, which directly models spend-level ownership as receiver-secret knowledge plus the DH-derived output secret.
3. The concept documents also frame ownership as `s_in + receiver_secret`, not `s_in` alone.

📌 Limitation.

1. The public proof or verifier layer for this property is not complete yet. The rule logic exists, but the final trustless verifier claim does not.
2. The specific ideas-document wording that `Spend-TxProof requires receiver_secret` should therefore be treated as design intent plus local rule logic, not as a completed verifier-backed property of the live stack.

📌 Rating: DISPUTED.

### C6. Validators can trustlessly verify spend correctness with a real ZK verifier

📌 This claim is not supported by the current implementation.

📌 Evidence.

1. `crates/z00z_wallets/src/core/tx/witness_gate.rs` uses `SpendCs` with simple structural checks instead of a full cryptographic verifier.
2. `crates/z00z_crypto/src/claim/*` is explicitly placeholder-based.
3. Storage and checkpoint code accepts non-empty proof bytes rather than verifying a real recursive proof transcript at the final boundary.

📌 Rating: DISPUTED.

### C7. STARK/FRI is implemented and working in the live stack

📌 This claim matches a classic hallucination pattern if stated without qualification.

📌 Evidence.

1. Many STARK or FRI references live in `crates/z00z_crypto/.todo/pq-Whitepaper.md`.
2. No active STARK or FRI dependency or live verifier surfaced in Cargo manifests.
3. The live proof code that does exist is labeled placeholder for claim verification and Bulletproofs+ for range proofs.

📌 Rating: FABRICATION RISK.

### C8. JMT publish or checkpoint path is already authoritative and trustless

📌 This claim is overstated today.

📌 Evidence.

1. `crates/z00z_storage/src/checkpoint/artifact.rs` and `crates/z00z_storage/src/checkpoint/codec.rs` enforce non-empty proof bytes.
2. Repository memory records that current checkpoint artifacts accept non-empty `cp_proof` and that storage persists synthetic proof bytes.
3. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` contains explicit placeholder comments for checkpoint aggregation and digest construction.

📌 Rating: DISPUTED.

## Ideas-Document Claim Cross-Check

📌 This section answers the narrow follow-up question about `.planning/temp/Z00Z-ECC-IDEAS.md`: whether its anti-theft claims about Alice and the aggregator are confirmed by live code, by document logic, or only by an unfinished proof boundary.

### D1. `Alice may know what she generated, but still cannot steal because Spend-TxProof requires receiver_secret`

📌 The sentence is directionally consistent with both reviewed documents, but it is not yet justified as a finished end-to-end verifier-backed property of the live stack.

📌 What is supported.

1. Alice does know sender-generated `k_dh` and `s_out` during output construction.
2. Local spend-level ownership logic really is receiver-secret-gated in `crates/z00z_wallets/src/core/stealth/output.rs` and `crates/z00z_wallets/src/core/tx/spending.rs`.
3. The reviewed design documents themselves explicitly support sender knowledge of `s_out`; they do not require sender ignorance of `s_out`.

📌 What is not yet supported.

1. The live public proof boundary does not yet prove this property to validators through a finished spend verifier.
2. The documents and code disagree on whether `s_out` is sender-random or deterministically derived, so the derivation semantics are still drifting even though sender knowledge is not.

📌 Verdict: supported by document logic and local rule logic, but not yet as a completed proof-backed implementation claim.

### D1a. Which `s_out` model is more correct for this repository: `random32` or `derive(k_dh, r_pub, serial_id)`?

📌 For the live repository, model `2` is the canonical implementation.

📌 Supporting points.

1. `crates/z00z_wallets/src/core/stealth/ecdh.rs` defines the single canonical formula as `derive_s_out(k_dh, r_pub, serial_id)`.
2. `crates/z00z_wallets/src/core/stealth/output.rs`, `crates/z00z_wallets/src/core/tx/builder.rs`, and `crates/z00z_wallets/src/core/tx/output_flow.rs` all build outputs from that deterministic derivation.
3. `crates/z00z_wallets/src/core/stealth/output_validator.rs` re-derives `s_out` from sender-held context and rejects the output if decrypted `pack.s_out` does not match the deterministic formula.

📌 Cryptographic interpretation.

1. The document model `s_out = random32` and the live-code model `s_out = H(k_dh || r_pub || serial)` differ in derivation semantics, but they do not differ on the question the user asked about Alice.
2. In both models Alice still knows `s_out`: in model `1` because she sampled it, and in model `2` because she knows `k_dh`, `r_pub`, and `serial_id`.
3. Therefore the difference between `1` and `2` is a design-drift issue, not a theft-resistance issue.

📌 Verdict.

1. If the question is `what does the live stack actually implement`, model `2` is correct.
2. If the question is `what do the current documents literally say`, model `1` is the document text.
3. If the question is `which model makes Alice ignorant of s_out`, neither model does.

### D1b. Does `receiver_secret + s_in` actually guarantee that Alice cannot steal after the tx is inserted into JMT?

📌 At the intended protocol level, yes.

📌 At the current authoritative implementation boundary, not yet.

📌 Why the intended logic works.

1. `crates/z00z_wallets/src/core/tx/spending.rs` models spend authorization as knowledge of `receiver_secret` plus `s_in`.
2. The same rule recomputes `owner_handle` and `view_sk` from `receiver_secret`, recomputes `k_in` from `view_sk` and `R_pub`, and then checks `owner_tag` and `asset_id == H(s_in)`.
3. This means sender knowledge of `s_out` alone is intentionally insufficient. The missing factor is Bob's `receiver_secret`.

📌 Why the current repository still cannot claim this as a final guarantee.

1. `crates/z00z_wallets/src/core/tx/witness_gate.rs` resolves `s_in` by decrypting the input pack with `receiver_secret`, but its proof API remains structural rather than cryptographically authoritative.
2. The JMT or checkpoint path ultimately relies on a proof-verifier hook that is still placeholder-like in the current stack.
3. So the intended B3 rule exists, but the repository does not yet prove that every accepted JMT state transition must satisfy it under an untrusted validator or aggregator.

📌 Verdict.

1. `receiver_secret + s_in` is the correct anti-theft rule for the intended protocol.
2. It is not yet a completed end-to-end guarantee at the authoritative JMT acceptance boundary.

### D1c. What does Bob have to re-encode after scanning to deprive Alice of access?

📌 In the intended B3 model, Bob does not need to re-encode the received coin at all.

📌 Supporting points.

1. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` stores `asset_secret: Some(pack.s_out)` and `blinding: Some(pack.blinding)` after a successful scan.
2. `crates/z00z_wallets/src/core/tx/witness_gate.rs` later resolves `s_in` by decrypting the same received input with `receiver_secret` and checking that the pack secret matches the witness secret.
3. The protocol intent is therefore `receive -> store s_out locally -> spend with receiver_secret + s_in`, not `receive -> re-encode into a new Bob-only secret before the coin becomes safe`.

📌 The user's logic gap is real, and it points to the actual blocker.

1. If Bob had to re-encode the coin just to make Alice unable to steal it, that would mean the incoming note was not actually Bob-only spendable under the original B3 rule.
2. In a correct B3 protocol, Alice is excluded immediately by the spend verifier because she lacks `receiver_secret`.
3. In the current repository, the reason this still feels unresolved is that the authoritative proof boundary is unfinished, not that Bob forgot to re-encode something.

📌 If the project really wants `Alice must not know the final spend secret`, that is a different protocol.

1. It would require an explicit `claim/rekey` step where Bob consumes the incoming note and creates a fresh Bob-only note with a new secret unknown to the sender.
2. That is not the current no-handshake direct-spend design.

### D2. `Aggregator cannot steal because it lacks Bob's secrets`

📌 The repository does not support this as the right current explanation for anti-theft.

📌 What is supported.

1. Checkpoint application verifies resolved-input membership consistency.
2. Empty proof bytes are rejected at artifact boundaries.

📌 What is not yet supported.

1. The authoritative checkpoint boundary does not yet enforce a real Bob-secret-gated spend proof.
2. The current simulator verifier accepts non-empty proof bytes and documents placeholder checkpoint aggregation.

📌 Verdict: contradicted as a statement about what current code actually enforces.

### D3. `Aggregator may censor, but cannot forge valid state transitions without proof`

📌 The repository only partially supports this statement.

📌 What is supported.

1. Membership consistency for inputs is real.
2. Range-proof verification exists for output commitments.

📌 What is not yet supported.

1. Spend or checkpoint authorization is not yet backed by a finished authoritative proof verifier.
2. Non-empty proof bytes are not equivalent to cryptographic anti-forgery.

📌 Verdict: supported only in a limited structural sense; not yet supported as a full trustless-proof claim.

### D4. `Aggregator can censor or withhold data without directly stealing`

📌 This caveat from the ideas document is consistent with the current implementation and should remain explicit in the report.

📌 Supporting points.

1. The checkpoint path records created-leaf hash-level state transitions.
2. That does not by itself guarantee full data availability for Bob if a malicious publisher withholds leaf body material.

📌 Verdict: PLAUSIBLE and important to keep explicit.

## Phase-by-Phase Security Map

📌 Phase verdicts for the exact chain requested by the task.

1. Alice.
   Goal: the sender should create a Bob-only spendable output without learning enough to steal.
   Implemented: real ECDH, owner tag, encrypted pack, and range-proof leaf build are present.
   Gap: sender does know `k_dh` and `s_out`; anti-theft relies on later receiver-secret checks, and the final proof boundary still does not make that exclusion authoritative.
   Verdict: WARN.

2. Leaf build.
   Goal: the leaf should be unlinkable and internally consistent.
   Implemented: sender build and receiver decrypt path are coherent and tested.
   Gap: `leaf_ad_id` semantics still need canonical freeze.
   Verdict: PASS WITH CAVEAT.

3. JMT publish.
   Goal: the aggregator should publish without learning the recipient and without requiring trust for correctness.
   Implemented: a privacy-oriented leaf payload exists.
   Gap: checkpoint proof path is not yet authoritative; placeholder proof semantics remain.
   Verdict: FAIL FOR TRUSTLESSNESS.

4. Bob scan.
   Goal: Bob should detect, decrypt, and verify his outputs.
   Implemented: yes, and this is covered by canonical or runtime parity tests.
   Gap: card-bound `tag16` leaves a DoS caveat when no request is used.
   Verdict: PASS.

5. Spend.
   Goal: spend should require hidden witness and preserve privacy.
   Implemented: rule logic exists for owner-tag, asset-id, balance, and range conditions.
   Gap: no complete public spend ZK verifier exists yet, so Bob's local recovery of `s_in` does not automatically become an authoritative chain-level exclusion of Alice.
   Verdict: FAIL FOR FULL ZK CLAIM.

6. Validator.
   Goal: validator should verify without learning Bob's secrets.
   Implemented: some structural validation exists.
   Gap: no live STARK/FRI stack and no finished public spend or claim verifier.
   Verdict: FAIL.

## What Was Verified Directly

📌 ECC sender or receiver agreement is real.

📌 Evidence.

1. `crates/z00z_crypto/src/ecdh.rs` implements `compute_stealth_dh_sender`, `recover_stealth_dh_receiver`, and `derive_dh_key` with identity and zero-scalar rejection.
2. `/tmp/crypto_golden.log` shows 7 golden-vector tests passed, including DH and tag-related vectors.

📌 Bulletproofs+ range proof verification is real.

📌 Evidence.

1. `crates/z00z_wallets/tests/test_tx_stealth_flow.rs` verifies the leaf range proof in the stage-4 workflow.
2. `/tmp/crypto_bulletproofs.log` shows `test_wrong_commitment`, `test_range_values`, and `test_batch_verify_100` passed.

📌 Runtime or canonical receive parity is real in tested flows.

📌 Evidence.

1. `/tmp/wallets_runtime_parity.log` shows owned and foreign runtime parity tests passed.
2. `/tmp/wallets_s6_recv.log` shows canonical and runtime examples passed.

📌 Nullifier replay-related storage behavior is tested.

📌 Evidence.

1. `/tmp/simulator_nullifier.log` shows `test_restart_replay_path ... ok` and `test_corrupt_rows_closed ... ok`.

## Adversarial Review

📌 If I assume the current implementation contains an error, the highest-probability failure modes are not in ECDH equality or simple pack decryption. They are in trust boundaries and proof completeness.

📌 The main adversarial objections are these.

1. The sender-side code itself disproves the strongest sender-ignorance narrative.
2. The spend and validator story depends on proof layers that are not finished.
3. The checkpoint or JMT publish path still contains placeholder proof semantics, so trustless state-transition claims are premature.
4. `leaf_ad_id` is important enough that any ambiguity there must be removed before making almost-formal claims.
5. `tag16` remains a spam surface unless request-bound derivation becomes the default path.
6. The ideas-document anti-theft story is directionally coherent, but its strongest sentences currently outrun the live verifier boundary.
7. The real logic gap is not `Bob forgot to re-encode the coin`; it is that the repository has not yet made the B3 ownership rule authoritative at the proof boundary.

## Precise Work Required To Make The Security Statement Nearly Formally True

### 1. Canonicalize `leaf_ad_id`

📌 Required outcome: one canonical definition of decrypt associated data across builder, runtime asset, wire format, tests, and proof statements.

📌 Minimum work.

1. Freeze whether `leaf_ad_id` is always equal to canonical leaf `asset_id` or some other formally named field.
2. Enforce it at constructors and import boundaries.
3. Remove any test or setup patterns that can silently diverge.
4. Bind it explicitly in spend and checkpoint proof statements.

### 1.5 Freeze `s_out` semantics and state explicitly whether sender knowledge is acceptable

📌 Required outcome: one canonical statement of what `s_out` is, who knows it, and why that does or does not matter for theft-resistance.

📌 Minimum work.

1. Choose between `sender-sampled random32` and `deterministic derive(k_dh, r_pub, serial_id)` as the only canonical `s_out` model.
2. Update the documents and the live code so they agree.
3. State explicitly that sender knowledge of `s_out` is acceptable only if the authoritative spend proof also requires `receiver_secret`.
4. If sender ignorance of the final spend secret is a hard requirement, design a separate `claim/rekey` protocol instead of overloading the current direct-spend model.

### 2. Replace the current spend gate with a real public spend verifier

📌 Required outcome: validators verify a proof object, not a local witness-style helper.

📌 Minimum work.

1. Replace structural `SpendCs` checks with a real proof verifier.
2. Bind `prev_root`, input refs, output leaves, owner-tag relation, asset-id relation, balance equation, range proofs, nullifier semantics, chain id, and version into the proved statement.
3. Define exact public inputs and transcript binding.

### 3. Either implement a real STARK/FRI stack or explicitly drop that claim

📌 Required outcome: no ambiguity between roadmap and implementation.

📌 Two valid paths exist.

1. Implement a real STARK/FRI prover or verifier stack with dependencies, parameter choices, proof object format, recursive checkpoint verification, and tests.
2. Explicitly document that the current live stack uses Bulletproofs+ for range proofs and does not yet implement STARK/FRI for spend or checkpoint verification.

📌 What must not happen: marketing or design language claiming live STARK/FRI support while the code remains placeholder-based.

### 4. Make checkpoint proof verification authoritative at the storage boundary

📌 Required outcome: `cp_proof` and `tx_proof` are cryptographically validated, not only checked for non-empty bytes.

📌 Minimum work.

1. Replace synthetic proof bytes in storage or checkpoint paths.
2. Verify proof objects during finalize and load operations.
3. Reject artifacts that do not match canonical proof transcripts.

### 5. Harden receiver identity binding

📌 Required outcome: sender cannot be tricked into encrypting to Mallory under Alice's directory entry.

📌 Minimum work.

1. Add TOFU plus pinning or signed directory binding for `ReceiverCard`.
2. Treat request or card validation as mandatory, not just caller discipline.
3. Freeze rotation behavior and mismatch handling.

### 6. Make request-bound tag derivation the normal privacy path

📌 Required outcome: targeted scan spam becomes materially harder.

📌 Minimum work.

1. Prefer payment-request mode for normal sends.
2. Bind `tag16` and optionally `k_dh` derivation to `req_id`.
3. Add tests that show card-bound and request-bound behavior diverge in the intended way.

### 7. Rewrite the security statement honestly

📌 Required outcome: repository claims match the actual trust model.

📌 Safer wording.

1. The sender derives sender-side output secret material during construction.
2. Spending is intended to require receiver-secret knowledge in addition to output secret material.
3. Passive state observers do not receive a direct wallet identifier from the leaf.
4. Aggregator anti-theft and anti-forgery claims remain conditional on replacing placeholder proof acceptance with a real verifier.
5. Trustless validator verification of spend or checkpoint privacy is not complete until the real proof stack lands.

## Recommended Repository-Level Verdict

📌 The current repository is good enough to say that the stealth receive path is real, tested, and materially privacy-oriented.

📌 The current repository is not good enough to say that the full Alice-to-validator chain is already almost formally secure under the stronger statement in the task.

📌 The most accurate single-sentence verdict today is this: `Stealth send/scan and Bulletproofs+ amount proofs are real; receiver-secret-gated ownership exists at rule level, but sender-ignorance, aggregator anti-theft at the checkpoint boundary, full public spend verification, and STARK/FRI-backed validator trustlessness are not yet real in code.`

## Limitations Of This Verification

📌 This report accelerates human verification; it does not replace a formal cryptographic audit.

📌 The adversarial review is grounded in the same repository and test artifacts available here. It catches important overclaims, but it is not a machine-checked proof.

📌 Claims rated VERIFIED mean supporting code paths and tests were found. They do not mean the implementation is immune to all side channels, metadata leaks, or future regressions.

📌 Claims rated PLAUSIBLE may still fail under a stronger threat model, especially around metadata, network leakage, and incomplete proof binding.

---

---

# HOW IT MUST BE DONE

**Executive Verdict**

Risky but salvageable.

Короткий ответ: нет, это работает не так, что stealth-address Bob “вкладывает” внутрь монеты знание `receiver_secret`. Правильнее так: Alice строит монету по публичной карточке Bob, Bob потом узнаёт “это моё” и расшифровывает содержимое, но право траты в замысле должно проверяться отдельным условием: нужен не только `s_in`/`s_out`, но и секрет Bob `receiver_secret`. То есть stealth-address отвечает за распознавание и доставку, а не сам по себе за окончательное право траты.

**Что Здесь Происходит**

Это mixed review: дизайн + код + threat model.

Цель безопасности здесь такая:

- Alice может создать монету для Bob без handshake.
- Наблюдатель не должен понять, кому она принадлежит.
- Bob должен уметь её распознать и открыть.
- Alice не должна уметь её потратить после публикации в JMT.

В текущем замысле системы это достигается не через “Alice не знает `s_out`”, а через “Alice знает `s_out`, но не знает `receiver_secret` Bob”.

**Строго По Шагам**

1. Bob публикует не свой секрет, а публичную карточку:
   - `owner_handle = H(receiver_secret)`
   - `view_pk = f(receiver_secret) * G`

2. Alice берёт эту карточку и строит output:
   - выбирает/получает `R_pub`
   - вычисляет общий секрет `k_dh`
   - формирует `owner_tag`
   - шифрует pack
   - в текущем коде получает `s_out` детерминированно из `k_dh + r_pub + serial`

3. Bob при скане leaf:
   - по своему `receiver_secret` восстанавливает `view_sk`
   - по `R_pub` получает тот же `k_dh`
   - проверяет `owner_tag`
   - расшифровывает pack
   - достаёт `s_out`

4. При трате, по intended logic, нужно доказать две вещи:
   - я знаю `s_in`, которое даёт `asset_id`
   - я знаю `receiver_secret`, которое даёт правильные `owner_handle` и `view_sk`

Именно второе условие должно выбрасывать Alice.

**Главная Мысль**

`receiver_secret` не “зашивается” в монету так, чтобы Alice его узнала. Он используется как скрытый второй фактор владения.

Аналогия, но точная:

- `s_out` это как номер ячейки и ключ от внешней дверцы.
- `receiver_secret` это как отпечаток пальца для внутреннего замка.
- Alice может знать номер ячейки и даже внешний ключ.
- Но если внутренний замок реально проверяется, без отпечатка Bob она не откроет ячейку до конца.

Проблема текущего репозитория не в том, что эта логика неверная. Проблема в том, что “охранник у двери” пока не полностью реализован как настоящий публичный proof verifier.

**Почему У Вас Возникает Разрыв Логики**

Потому что интуитивно кажется так:

- если Alice знает `s_out`,
- а `asset_id = H(s_out)`,
- значит Alice уже знает “секрет монеты”,
- значит она должна мочь украсть.

Это было бы правдой, если бы право траты определялось только `s_out`.

Но в текущем B3-замысле право траты определяется не `s_out` alone, а парой:

$$
\text{spend authority} = (\text{receiver\_secret}, \text{s\_in})
$$

То есть:

- `s_in` привязывает тебя к конкретной монете,
- `receiver_secret` привязывает тебя к конкретному владельцу.

Если убрать `receiver_secret`, Alice действительно сможет красть.
Если `receiver_secret` реально обязателен в verifier, Alice не сможет красть, даже зная `s_out`.

**Где Это Видно В Коде**

Локальная логика траты это реально моделирует в spending.rs.

Распознавание и извлечение секрета Bob после скана идёт через stealth_scan_support.rs.

Построение sender-side output и derivation `s_out` видно в output.rs и ecdh.rs.

А вот место, где становится видно, почему у вас остаётся недоверие: proof boundary всё ещё structural/placeholder-like в witness_gate.rs.

**Самый Важный Ответ На Ваш Вопрос**

Если протокол корректен, Alice должна быть отрезана уже на момент первой допустимой траты, потому что:

- монета уже Bob-owned по правилу,
- а validator принимает spend только если доказано знание `receiver_secret`.

**То Есть Два Разных Дизайна**

Текущий intended design:

 - Alice знает `s_out`
 - Bob тоже узнаёт `s_out`
 - но только Bob знает `receiver_secret`
 - spend requires both
 - re-encode не нужен

**Критические Findings**

| Field     | Content                                                      |
| --------- | ------------------------------------------------------------ |
| Severity  | S1                                                           |
| Component | B3 ownership model                                           |
| Problem   | Анти-кража зависит не от сокрытия `s_out`, а от обязательности `receiver_secret` в spend-verifier |
| Impact    | Если verifier не требует `receiver_secret`, Alice сможет тратить, зная sender-side material |
| Fix       | Сделать authoritative public verifier, который жёстко биндует `receiver_secret`, `s_in`, `owner_tag`, `asset_id`, `prev_root`, outputs |

| Field     | Content                                                      |
| --------- | ------------------------------------------------------------ |
| Severity  | S1                                                           |
| Component | JMT/checkpoint boundary                                      |
| Problem   | Репозиторий пока не доказывает end-to-end, что все принятые state transitions реально enforceят B3 |
| Impact    | Intended logic есть, но нет полной chain-level криптографической гарантии |
| Fix       | Довести proof boundary до настоящего verifier, а не structural checks / non-empty proof bytes |

| Field     | Content                                                      |
| --------- | ------------------------------------------------------------ |
| Severity  | S2                                                           |
| Component | `s_out` semantics                                            |
| Problem   | Документы говорят `random32`, код говорит `derive(k_dh, r_pub, serial)` |
| Impact    | Это путает reasoning, но не меняет сам факт, что Alice знает `s_out` в обеих моделях |
| Fix       | Канонизировать одну модель и переписать docs/code в одно состояние |

**Ответ В Одной Фразе**

Alice не должна быть отрезана от монеты потому, что она “не знает `s_out`”. Она должна быть отрезана потому, что даже зная `s_out`, она не знает `receiver_secret` Bob, а verifier должен требовать оба секрета вместе.

**Final Decision**

Blocked:

1. Нет завершённого authoritative public spend verifier на границе принятия state transition.
2. Не зафиксирована одна каноническая семантика `s_out`.
3. Поэтому intended anti-theft logic понятна, но end-to-end доказанной гарантией в текущем live stack ещё не является.

---

---

# MORE EXPLANATIONS

**Executive Verdict**

Risky but salvageable.

Да, понимать это нужно так: монета Bob связана не с тем, что внутри нее “лежит” секрет Bob, а с тем, что ее публичные поля согласованы только с секретом Bob. Это как замок, который не хранит ключ внутри себя, но открывается только ключом с правильной нарезкой.

**Что Именно Связывает Монету С Bob**

В текущей логике есть две разные вещи.

1. Распознавание монеты.
   Bob по своей карточке и своему секрету понимает: это монета для меня, я могу ее открыть.

2. Право траты.
   Чтобы потратить монету, мало знать секрет монеты. Нужно еще показать правильный секрет владельца.

Это видно в output.rs и spending.rs.

Логика там такая:

$$
owner\_handle_B = H(RID \parallel receiver\_secret_B)
$$

$$
view\_sk_B = H2Scalar(VIEW \parallel receiver\_secret_B)
$$

Дальше из монеты берется ее публичный эпhemeral ключ $R_{pub}$, и при трате проверка считает:

$$
k_{in} = DH(view\_sk_B, R_{pub})
$$

Потом проверяется, что owner tag монеты совпадает с тем, что должен получиться именно для Bob:

$$
owner\_tag = H(TAG \parallel owner\_handle_B \parallel k_{in})
$$

И отдельно проверяется, что секрет монеты действительно соответствует ее asset id:

$$
asset\_id = H(ASSET \parallel s_{in})
$$

То есть spend-rule в замысле требует сразу две вещи:

$$
spend\ authority = (receiver\_secret,\ s_{in})
$$

**Почему Секрет Alice Не Подходит**

Потому что если Alice подставит свой собственный receiver secret, получится другой owner handle и другой view key.

Значит:

- из того же самого $R_{pub}$ она получит другой $k_{in}$,
- из него получится другой owner tag,
- и проверка owner tag не пройдет.

Проще говоря:

- Alice может знать секрет монеты,
- но монета “помечена” как принадлежащая тому, чей receiver secret воспроизводит правильную пару owner handle плюс view key,
- для монеты Bob это секрет Bob, не Alice.

**Самая Простая Аналогия**

Монета устроена как сейф с двумя условиями.

Первое условие:
нужно знать номер сейфа. Это роль секрета монеты, то есть $s_{in}$.

Второе условие:
нужно приложить правильный отпечаток владельца. Это роль receiver secret.

Alice знает номер сейфа, потому что сама его создала.
Но ее отпечаток не совпадает с тем, под который сейф был выписан.
Поэтому сейф должен не открыться.

**Где Возникает Ваш Разрыв Логики**

Он нормальный. Он появляется из вопроса:

“Если Alice знает $s_{out}$, почему она не может потратить?”

Ответ:
потому что в этой схеме $s_{out}$ сам по себе не является полным правом собственности.

Если бы правило было такое:

$$
spend\ authority = s_{in}
$$

тогда Alice действительно могла бы украсть.

Но в текущем B3-замысле правило другое:

$$
spend\ authority = (receiver\_secret,\ s_{in})
$$

Именно поэтому receiver secret и важен.

**Что Делает Stealth Address И Чего Он Не Делает**

Stealth address:

- помогает Alice создать output для Bob,
- помогает Bob потом узнать “это мое”,
- помогает Bob расшифровать содержимое.

Но stealth address сам по себе не является доказательством права траты.

Право траты появляется только там, где verifier требует receiver secret как часть witness.

**Значит Ли Это, Что Все Уже Надежно**

Вот здесь самое важное ограничение.

На уровне intended logic ответ такой:
да, Alice не должна мочь тратить монету Bob, потому что у нее нет receiver secret Bob.

На уровне текущего репозитория как окончательной chain-level гарантии ответ слабее:
это пока не доведено до конца.

Причина в том, что публичная proof boundary еще не завершена. Локальная логика ownership есть в spending.rs, но финальная verifier boundary все еще structural или placeholder-like в witness_gate.rs.

Именно поэтому формулировка должна быть строгой:

- как криптографическая логика дизайна: да, Bob’s receiver secret должен отсекать Alice;
- как уже завершенная гарантия live stack: пока нет, потому что финальный verifier не доведен.

**Пример Совсем По-Простому**

Пусть:

- Alice отправляет монету Bob,
- Alice знает секрет монеты,
- Bob знает секрет монеты после расшифровки,
- но только Bob знает свой receiver secret.

Тогда:

- Alice пытается потратить,
- verifier спрашивает: покажи секрет монеты и секрет владельца,
- Alice показывает секрет монеты, но секрет владельца показать не может,
- spend должен быть отвергнут.

Если verifier вдруг проверяет только секрет монеты, тогда Alice украдет.
В этом и состоит текущий риск незавершенной proof boundary.

**Ключевой Итог**

Вы правильно понимаете главное:

не монета хранит внутри себя receiver secret Bob.
Правильнее сказать так:
монета построена так, что только Bob’s receiver secret согласуется с ее owner-tag и stealth-структурой при spend-проверке.

То есть связь монеты с Bob идет через проверяемое соответствие, а не через явное хранение секрета внутри монеты.

**Final Decision**

Blocked.

1. Криптологическая логика B3 понятна и внутренне связна: Alice должна отсекаться через отсутствие receiver secret Bob.
2. Но текущая кодовая база еще не дает полной end-to-end гарантии этого на границе принятия state transition.
3. Поэтому правильный ответ на ваш вопрос: “почему Alice не может тратить?” — потому что должен проверяться receiver secret Bob, а не потому что Alice не знает секрет монеты.

Если хотите, я могу следующим сообщением нарисовать это как мини-схему в формате:
Alice знает что,
Bob знает что,
что проверяет validator,
и на каком именно шаге Alice должна ломаться.

---

---

# SHORT SUMMARY

1. Alice знает: что она создала монету и кому хотела ее отправить, но этого мало, чтобы потом ею воспользоваться.
2. Bob знает: свой личный секрет, который подходит именно к этой монете.
3. Bob знает: внутренний секрет самой монеты, который он получает при чтении своей входящей истории.
4. Validator проверяет: что у того, кто тратит монету, есть и секрет Bob, и секрет самой монеты сразу.
5. Validator проверяет: что оба секрета указывают на одну и ту же монету, а не на две разные вещи.
6. Alice ломается ровно здесь: у нее может быть секрет монеты, но нет секрета Bob, поэтому проверка не сходится.
7. Значит: увидеть или даже создать монету не равно уметь ее потратить; потратить может только тот, у кого совпали обе части.

Если хочешь, следующим сообщением сведу это еще жестче в формат:
`Alice has / Bob has / Validator checks / Alice fails here` в 4 строки.



---

---

---



# Scenario 1 Crypto Architect Report

## Review Scope

**Executive verdict:** `Fundamentally broken`

**Final decision:** `Blocked: placeholder claim proof and authority signature, non-authoritative genesis root, checkpoint proof and spent-set stubs, and secret-dump behavior must be resolved before this flow can be treated as cryptographically trustworthy.`

**Input type and scope:** mixed implementation review of Rust source code under `crates/z00z_simulator/src/scenario_1`, with supporting verifier and proof code in `crates/z00z_crypto` and `crates/z00z_wallets` where the scenario depends on them.

This review treats `scenario_1` as a cryptographic workflow, not just a simulator demo, because it constructs claim statements, range proofs, nullifiers, checkpoint artifacts, wallet secrets, and verifier inputs that are accepted by adjacent crates.

## Security Model

**Security goals extracted from code:** claim authenticity, claim soundness, chain binding, nullifier uniqueness, committed-state integrity, wallet secret confidentiality, stealth ownership privacy, crash-safe replay resistance, and deterministic restart safety for simulator checkpoints.

**Assets at risk:** wallet passwords, seed phrases, receiver secrets, claim source commitments, nullifiers, claim statements, checkpoint drafts, spent-set state, post-transaction committed outputs, and any asset balance reachable through scenario-generated claim packages.

**Adversaries considered:** malicious claimant, malicious simulator operator, local filesystem reader, accidental artifact publisher, rollback attacker replaying scenario artifacts, and concurrent execution races across wallet service initialization and environment mutation.

**Trust boundaries extracted from code:** `scenario_1` orchestration, wallet RPC transport, filesystem artifact store, checkpoint store, `z00z_wallets` claim verifier, and `z00z_crypto::claim` placeholder proof/signature logic.

**Failure assumptions extracted from code:** simulator restarts, malformed artifacts, replayed checkpoint inputs, weak deterministic entropy, proof generation failure, concurrent task interleaving, and manual inspection of exported artifacts.

**Threat-model assessment:** usable but incomplete. The code reveals an implicit threat model for simulator happy-path validation, but it does not clearly state what guarantees are intentionally missing. Confidence is therefore reduced whenever simulator placeholders are accepted by verifiers outside a strictly isolated test boundary.

**Confidence:** medium for scope extraction, because the code clearly exposes goals and boundaries, but there is no explicit threat model document defining which cryptographic guarantees are intentionally out of scope.

## Critical And High Findings

| Severity | Component                                                    | Problem                                                      | Impact                                                       | Exploit path                                                 | Fix                                                          |
| -------- | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| S0       | `crates/z00z_crypto/src/claim/prover.rs`, `crates/z00z_crypto/src/claim/verifier.rs`, `crates/z00z_simulator/src/scenario_1/stage_3.rs` | `prove_genesis_claim` and `verify_genesis_claim` implement a deterministic placeholder digest over the public statement, not a proof tied to a secret witness. `ClaimAuthoritySig::from_statement` is likewise forgeable from public data. | Any party able to assemble a statement can generate a passing proof and authority signature without possessing secret authority material or a real witness. This is direct proof forgery and invalid authorization. | Build any desired `GenesisClaimStatement`, derive `proof_bytes(statement_hash)` and `sig_bytes(statement_hash)`, then submit through the normal claim verifier path. | Replace the placeholder claim proof and signature with a real authorization model. At minimum, require a real signing key for claim authority and an authenticated genesis-membership witness. If the design remains temporary, isolate it behind a verifier mode that cannot be enabled outside simulator tests. |
| S0       | `crates/z00z_simulator/src/scenario_1/stage_3.rs`, `crates/z00z_wallets/src/core/tx/claim_tx.rs` | The claim statement binds `genesis_root` to `ZERO_ROOT`, and wallet verification explicitly documents it as a transitional non-authoritative root. | Claim soundness is broken because the verifier does not authenticate the claimed source against a real genesis commitment set. A forged or arbitrary source commitment can be represented as if it came from genesis. | Construct a claim package for an arbitrary source commitment and asset id, then rely on the placeholder proof and zero-root statement binding to satisfy verification. | Bind claim statements to a real authenticated genesis root or equivalent commitment root, and require verifier-side membership validation against that root before any claim is accepted. |
| S1       | `crates/z00z_simulator/src/scenario_1/stage_6.rs`, `crates/z00z_simulator/src/scenario_1/stage_7.rs` | Checkpoint construction uses `PassProof` and `NoSpent` placeholders when building the checkpoint draft. | The checkpoint pipeline can accept state transitions without a real proof check or spent-set validation, undermining double-spend resistance and state integrity in the simulator's finalization flow. | Feed duplicate or invalid fragment inputs into the checkpoint flow and rely on the stubbed verifier path to produce a valid-looking draft and follow-on artifact chain. | Replace placeholders with the real proof verifier and real spent-index checks, or make the draft builder refuse execution unless an explicit simulator-only test mode is proven impossible to ship. |
| S1       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs`, `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs`, `crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs` | Scenario artifacts store passwords, seed phrases, and receiver secrets in plaintext, including a Markdown dump and long-lived `String` fields. | Anyone with filesystem access, CI artifact access, or accidental repository exposure can recover wallet material and fully compromise scenario wallets. | Run the scenario with debug export enabled, read the generated secrets artifact, unlock or import wallets, and spend funds or decrypt owned outputs. | Remove plaintext secret dumps from the default workflow. Use `Hidden<T>` or `SecretBytes` wrappers, zeroizing containers, and explicit one-shot encrypted export flows only. If debug export is unavoidable, gate it behind a separate test-only binary and write to a guaranteed throwaway location with loud runtime confirmation. |

These are structural findings, not style concerns. The first two are enough on their own to block any claim that the scenario exercises a sound claim protocol.

**Confidence:** high for all four findings. The relevant code paths are direct, short, and unambiguous.

## Medium And Low Findings

| Severity | Component                                                    | Problem                                                      | Impact                                                       | Exploit path                                                 | Fix                                                          |
| -------- | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| S2       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` | `SeqSecureRngProvider` derives seeds by XORing a fixed seed with a counter-based mixer, then feeds them into `StdRng`. | This is not a cryptographically defensible randomness construction for key material or unlinkability-sensitive flows, even if labeled simulator-only. The pattern is likely to be copied. | Predict future RNG streams from the small seed and public counter schedule, then reproduce ephemeral choices across runs. | Delete the custom RNG wrapper. Use `MockRngProvider` for deterministic tests and `CryptoRng`-backed sources for any key or stealth output generation. |
| S2       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/actors.rs` | Actor passwords and mock seeds are fixed, human-readable, and low entropy. | The scenario normalizes weak credential habits and makes exported artifacts trivially reusable by anyone who knows the defaults. | Read default actor names, derive matching passwords from source, unlock exported wallets, and recover state. | Generate per-run test credentials, or derive them from a single test seed with a KDF and role-separated labels. |
| S2       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs` | `std::env::set_var` mutates global wallet network and chain settings during runtime assembly. | Concurrent scenario execution can cross-contaminate wallet configuration and silently misbind artifacts to the wrong chain or network context. | Run multiple scenario flows in the same process or runtime, interleave initialization, and observe configuration bleed-through. | Pass network and chain settings through explicit configuration objects instead of process-global environment mutation. |
| S3       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` | Sensitive values live in ordinary `String` fields without zeroization. | Secret exposure surface is larger than necessary and survives longer in heap memory. | Memory dump, panic dump, or later debug logging can reveal material that should have been erased after use. | Replace with zeroizing secret wrappers and avoid cloning or formatting secrets. |
| S3       | `crates/z00z_simulator/src/scenario_1/Readme.md`             | The scenario is described as a happy-path baseline without a prominent disclaimer that claim proof and authority semantics are placeholders. | Readers may over-trust the scenario and propagate broken constructions into adjacent code or docs. | Developers use the simulator as a reference implementation rather than a test harness. | Add an explicit cryptography status section listing placeholder components and banned reuse surfaces. |
| S4       | `crates/z00z_simulator/src/scenario_1/stage_7.rs`, `crates/z00z_simulator/src/scenario_1/jmt_wallet_scan.rs` | Several parts of the pipeline are otherwise disciplined: proof validation precedes ownership detection, state transitions are checkpointed, and report artifacts are emitted consistently. | This reduces operational ambiguity but does not compensate for the structural proof and root-binding failures. | None.                                                        | Preserve this sequencing while replacing placeholder crypto. |

Positive observations worth preserving after remediation:

- BLAKE3 domain separation for claim output leaf hashes and owner-binding hashes is explicit and distinct.
- Nullifier derivation is chain-bound in the scenario flow instead of being a bare claim identifier.
- Range-proof generation for claim outputs is tied to the derived blinding value rather than an obvious dummy scalar.
- Post-transaction JMT scanning validates proof data before ownership checks, which is the right direction for avoiding semantic false positives.

**Confidence:** high for the S2 findings, medium-high for the S3/S4 observations.

## Open Ambiguities

The following questions prevent stronger positive conclusions even after reading the full scenario module:

1. Who owns the long-term authority key that should authorize genesis claims, and where is that trust anchor anchored in chain state or configuration?
2. Is `scenario_1` expected to remain permanently simulator-only, or is it a migration path toward a production claim flow?
3. What exact spent-set invariant should `stage_6` enforce, and which storage component is the intended source of truth for it?
4. What are the maximum allowed amounts and asset classes for the stage-3 range-proof statements, and are those limits consensus-bound anywhere outside the simulator?
5. Is there an approved artifact-handling policy for `wallet_debug_dump`, including retention, cleanup, CI publishing, and accidental commit prevention?

**Confidence:** high that these are real blockers, because each ambiguity maps to a missing trust anchor or enforcement rule.

## Concrete Fixes

**Fix set A: replace placeholder claim authorization.**

1. Introduce a real genesis claim authority keypair and bind its public key in configuration or chain state.
2. Sign the canonical claim statement with a real signature scheme already present in `z00z_crypto`, not a statement-hash wrapper.
3. Make the verifier reject any placeholder mode unless compiled into an isolated simulator-only test target.

**Fix set B: authenticate genesis membership.**

1. Replace `ZERO_ROOT` with a real genesis commitment root.
2. Add membership witnesses or authenticated inclusion data for the source asset.
3. Ensure the claim statement binds asset id, source commitment, chain id, scenario or ruleset version, and the authenticated genesis root.

**Fix set C: remove checkpoint integrity placeholders.**

1. Replace `PassProof` with the real proof validation result used by storage finalization.
2. Replace `NoSpent` with an actual spent-set lookup against the checkpoint source of truth.
3. Add a fail-closed path so draft creation aborts if either proof validation or spent-set validation is unavailable.

**Fix set D: harden secret lifecycle.**

1. Delete plaintext secret Markdown export from the default path.
2. Use `Hidden<T>`, `SecretBytes`, or equivalent zeroizing wrappers for passwords, seed phrases, and receiver secrets.
3. Keep any debug export behind a separate feature and separate executable with runtime confirmation, artifact cleanup, and explicit non-production labeling.

**Fix set E: fix simulator randomness and configuration handling.**

1. Remove `SeqSecureRngProvider` and use existing deterministic test RNG abstractions already provided by the workspace.
2. Derive actor credentials from a scenario seed plus domain-separated labels instead of hard-coded literals.
3. Stop mutating process-global environment variables during runtime assembly.

**Confidence:** high that these changes address the identified issues without requiring vendor edits under `z00z_crypto/tari/`.

## Implementation Guidance

A minimally safe architecture for this scenario is:

1. Treat `scenario_1` as a harness around real verifier components, never as the place where proof semantics are weakened.
2. Keep proof generation and proof verification in `z00z_crypto` and `z00z_wallets`, but make simulator code supply only authenticated inputs, fixtures, and deterministic test configuration.
3. Make the source of truth for nullifier uniqueness and spent-set checks live in storage code, not scenario glue.
4. Keep domain separation explicit and versioned for every hash, proof transcript, KDF label, and authorization message.
5. Use canonical serialization before every signed or hashed statement and make the verifier reject any ambiguous encoding.

Recommended crate and primitive direction:

- Reuse existing Tari-backed signing and commitment primitives already exposed via `z00z_crypto`.
- Use Merlin-style transcript binding if the claim proof evolves beyond the current placeholder API.
- Keep `z00z_utils` as the only layer for filesystem and serialization concerns.

**Confidence:** medium-high. The structural direction is clear, but the exact claim-membership witness format still needs an owner decision.

## Test Plan

**Positive tests:**

1. Valid genesis claim with real authenticated root and authority signature passes end-to-end.
2. Valid restart after stage-3 or stage-6 resume points reproduces identical accepted state.
3. Charlie and Bob receive only outputs that they can actually decrypt after proof verification.

**Negative and misuse tests:**

1. Forged claim statement with arbitrary source commitment must fail because membership proof does not verify.
2. Forged authority signature must fail with a typed authorization error.
3. Replayed nullifier or duplicate spent input must fail before checkpoint draft creation.
4. Wrong chain id, wrong ruleset version, or wrong authenticated root must fail verifier binding checks.
5. Any attempt to enable placeholder proof mode in non-test builds must fail compilation or startup.

**Property and adversarial tests:**

1. Property-test canonical serialization and statement hashing for round-trip stability.
2. Fuzz claim package decoders, checkpoint artifact loaders, and resume-state files.
3. Test concurrent scenario execution to ensure network and chain configuration cannot bleed across runs.
4. Add artifact hygiene tests confirming that debug secret files are never produced in default runs.

**Dependency and primitive checks:**

1. Run `cargo audit` and `cargo deny` for crypto-relevant crates.
2. Add negative tests around range-proof parameter mismatches and malformed proof bytes.
3. If standard primitive wrappers are added, include Wycheproof coverage where applicable for signatures and ECDH-related flows.

**Confidence:** high that this test plan would expose regressions in the areas reviewed.

## Section Confidence Summary

| Section                                 | Confidence | What would raise confidence                                  |
| --------------------------------------- | ---------- | ------------------------------------------------------------ |
| Threat model extraction                 | Medium     | An explicit scenario threat model and statement of simulator-only exclusions |
| Placeholder proof and signature finding | High       | No additional evidence needed; the code is conclusive        |
| Zero-root claim soundness finding       | High       | A verifier-side authenticated membership check that is not currently present |
| Checkpoint placeholder finding          | High       | Proof that `PassProof` and `NoSpent` are unreachable in all accepted flows |
| Secret lifecycle finding                | High       | Removal of plaintext exports and zeroizing secret containers |
| RNG and config hygiene findings         | High       | Replacement with explicit deterministic test RNG and non-global config plumbing |

## Final Decision

`Blocked:`

- Owner: `scenario_1` maintainers must remove placeholder claim proof and placeholder authority signature semantics.
- Owner: claim-verifier owners must bind claims to a real authenticated genesis root and membership witness.
- Owner: checkpoint pipeline owners must replace `PassProof` and `NoSpent` with real enforcement.
- Owner: simulator and wallet-debug owners must remove or quarantine plaintext secret artifacts.

Until these items are resolved, `scenario_1` can serve as a functional smoke harness, but it must not be treated as evidence of cryptographic correctness.
