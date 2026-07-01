# Phase Question Sheet

**Phase:** `033-crypto-audit-scenario-2`
**Generated:** `2026-04-06`
**Scope Sources:** `033-32EXAM-QEST-DRAFT.md`, `033-SEMANTIC-FREEZE.md`, `.planning/ROADMAP.md`

## MUST

1. Every answer in this document MUST remain repository-backed.
2. If a proof cannot be closed, the answer MUST state exactly what evidence,
   artifact, or live behavior is missing.
3. Every answer MUST stay tied to the live codebase, tests, logs, manifests,
   and phase artifacts for this repository.
4. Every answer in this document MUST function as a verification exam of the
   implementation, not as freeform commentary.
5. If answering a question reveals a real bug, gap, or overclaim, the answer
   MUST name it explicitly and state the remediation path.
6. This file originated as a question sheet. Now that the `Ans:` sections are
    filled, future edits MUST preserve repository-backed evidence, explicit
    narrowing, and truthful gap reporting.

## 🎯 Challenge

Pressure-test whether the reformulated questions faithfully preserve the Phase
033 draft claims, conclusions, and remediation demands while staying usable for
repository-backed solving.

## ⛔ Constraints

- This file is a structured `Quest/Ans` rewrite of the draft assertions, not a
  new brainstorming pass.
- Hints are allowed when they help preserve the original meaning of a draft
  assertion.
- The draft remains the primary source of meaning.
- Questions may be split when needed so each one stays compact and
  self-contained.

## Scope Note

This exam rewrites the draft audit surface into a standard solving format. It
tests the current claim, spend, checkpoint, ownership, simulator, and
secret-handling story reflected by the draft, with minimal semantic drift from
the original assertions.

## 🔍 Answering Standard

- Answers must discover their own evidence path through the repository.
- Answers must separate current-stack enforcement from stronger future-proof or
  trustless-proof claims.
- Answers must preserve the scope of the original draft claim instead of
  widening it to a broader architecture review.

## 📏 Status Semantics

- `Full Evidence`: The final answer is directly supported by repository
  artifacts after required narrowing.
- `Partial Evidence`: The final answer is still the best repository-backed
  reading, but it depends on bounded inference, incomplete repo-wide closure,
  or unresolved wording at the canonical boundary.

## 🎯 Theme 1: Core Draft Claims

### 1. Passive Receiver Identifiers

🔴 **Quest:** What exactly is the current C3 claim that passive leaf or JMT observers get no direct receiver identifier from the leaf payload, and what evidence supports or limits that claim?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The live repository supports only a narrow C3 claim: the canonical public leaf payload omits a direct receiver identifier such as `owner_handle`, `view_pk`, or `identity_pk`. That narrow claim is supported by the live leaf and wire surfaces. The same code also limits the claim: passive observers still see public stealth metadata such as `r_pub`, `owner_tag`, `tag16`, commitment bytes, and ciphertext, so the repository does not prove a stronger metadata-unlinkability guarantee.

**Evidence Trail:**

1. `crates/z00z_core/src/assets/leaf.rs` defines the canonical public `AssetLeaf` fields as `asset_id`, `serial_id`, `r_pub`, `owner_tag`, `c_amount`, `enc_pack`, `range_proof`, and `tag16`, with no `owner_handle`, `view_pk`, or `identity_pk` field.
2. `crates/z00z_wallets/src/core/address/stealth_card.rs` defines `ReceiverCard` with `owner_handle`, `view_pk`, and `identity_pk`, showing that direct receiver identifiers exist on a separate routing surface rather than inside the leaf payload.
3. `crates/z00z_wallets/src/core/stealth/output_build.rs` validates request/card route agreement with `owner_handle`, `view_pk`, and `identity_pk`, but emits only `TxStealthOutput { r_pub, owner_tag, tag16, enc_pack, c_amount }` into the output contract.
4. `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` shows wallet detection reconstructs ownership from wallet-local `owner_handle`, private `view_sk` values, optional request ids, and public leaf fields. It does not read a direct receiver identifier out of the leaf itself.
5. `crates/z00z_core/src/assets/asset_validation.rs` and `crates/z00z_core/src/assets/wire.rs` show the broader public stealth asset/wire surface requires `r_pub`, `owner_tag`, `enc_pack`, `tag16`, and `leaf_ad_id`, but still not `owner_handle`, `view_pk`, or `identity_pk`.

**Reasoning:**

- A passive observer of the committed leaf or public asset wire can see several public stealth fields, but none of them is the receiver's direct routing identity.
- The receiver's direct identifiers live on the signed `ReceiverCard` and `PaymentRequest` routing surfaces and are consumed before output construction or inside wallet-local approval logic.
- Therefore the exact current C3 claim is not "the leaf proves no metadata leakage at all". It is only that the leaf payload omits a direct receiver identifier.
- A tempting but false overread would be: because `owner_handle` is absent, the leaf is formally unlinkability-proof. The code does not prove that. Public `r_pub`, `owner_tag`, `tag16`, ciphertext shape, and other visible metadata remain observable, so the stronger claim stays unsupported.

**Gap Or Blocker:** None for the narrow claim. The stronger metadata-proof reading is unsupported, but that is a limit of the claim rather than a blocker to answering it.

**Verification:**

- `doublecheck` status: VERIFIED
- Residual caveat: The verified closure applies only to the narrow claim that the leaf omits direct receiver identifiers, not to a broader no-metadata-linkage claim.

### 2. Real Stealth And Range Proofs

🔴 **Quest:** Does the repository implement a real stealth receive path and real Bulletproofs+ range-proof verification, and does that justify the full original claim as stated?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** Yes for the narrow current-tree facts: the repository has a real canonical wallet-side stealth receive path and real Bulletproofs+ range-proof verification on the current accepted public-spend boundary. No for the stronger original claim as stated: those truths do not by themselves justify a finished end-to-end trustless spend or checkpoint verifier claim.

**Evidence Trail:**

1. The canonical receive path is live in wallet code. `receiver_scan_leaf(...)` and `receiver_scan_report(...)` in `crates/z00z_wallets/src/core/address/leaf_scan.rs` run the ownership check through `scan_owned(...)`, while `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` decodes `r_pub`, computes receiver-side DH, derives `k_dh`, checks `owner_tag` and `tag16`, decrypts the encrypted pack, parses it, and verifies the commitment opening before returning `DetectState::Mine`.

2. That receive logic is not isolated helper code. `crates/z00z_wallets/src/core/address/stealth_scanner.rs` routes runtime assets through `scan_leaf(...)`, `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` uses the scanner in `recv_one(...)` and `recv_range(...)`, and `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs` calls the same scanner on the RPC receive path and only returns success on `ScanResult::Mine`.

3. The repository also contains a real Bulletproofs+ verifier. `crates/z00z_wallets/src/core/tx/prover.rs` initializes `BulletproofsPlusService` and `verify_proof(...)` parses the commitment bytes and calls the service verifier directly. `crates/z00z_wallets/src/core/tx/spend_verification.rs` then uses that prover inside `verify_tx_public_spend_contract(...)` and rejects missing or invalid range proofs for `Coin` and `Token` outputs.

4. The same cryptographic verification surface is reused outside that one seam. `crates/z00z_core/src/assets/asset_crypto.rs` calls `z00z_crypto::verify_range_proof(...)` for asset-level validation, and `crates/z00z_core/src/genesis/genesis_verification.rs` batch-verifies genesis asset range proofs through `z00z_crypto::batch_verify_range_proofs(...)`.

5. Live tests confirm these are real checks, not placeholders. `crates/z00z_wallets/src/core/address/stealth_scanner/test_stealth_scanner.rs` exercises owned-output detection, `crates/z00z_core/tests/assets/test_integration_assets_test6.rs` shows wrong-commitment and tampered-proof rejection under `verify_range_proof(...)`, `crates/z00z_wallets/tests/test_spend_witness_gate.rs` verifies the public spend contract on canonical inputs, and `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` proves Scenario 1 stage-4 packages are accepted or rejected by that same wallet public verifier.

6. The stronger original claim still fails. `docs/code-review/032-scenario-1-crypto-status.md` explicitly says the tree does not claim a universal trustless public spend verifier for every wallet-local ownership invariant beyond the current accepted boundary, and the Phase 033 draft `033-32EXAM-QEST-DRAFT.md` separately records that the repository does not justify the full claim as originally stated.

**Reasoning:**

- The receive side is live because the code performs real receiver-key DH, authenticated decrypt, payload parse, and commitment-opening validation before producing a wallet-owned output.

- The proof side is live because Bulletproofs+ verification is wired through real cryptographic services and is enforced on accepted public-spend and asset-validation seams.

- But the original claim is stronger than “receive scanner exists” plus “range proofs verify.” It implicitly reaches toward a finished end-to-end trustless spend or checkpoint-verifier story.

- The repository itself narrows that story: current accepted boundaries are real, but broader whole-chain, checkpoint, and universal ownership-verifier claims remain intentionally unclaimed or still open.

**Gap Or Blocker:** None for the narrow two-part factual claim. The blocker is only against the stronger reading: not every receive entrypoint is canonical, and the repository still does not justify a fully finished trustless public-proof boundary from wallet-local ownership semantics through final chain acceptance.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: “real stealth receive path” is stated as the canonical wallet-side receive flow, not every possible receive entrypoint.
- Required narrowing applied: “real Bulletproofs+ verification” is stated at the current accepted public-boundary seams, not as proof of a fully finished end-to-end trustless verifier.

### 3. Full Claim Justification

🔴 **Quest:** Does the repository justify the full claim as originally stated?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** No. The repository justifies only a narrower current-stack public-boundary claim, not the full original claim as stated.

**Evidence Trail:**

1. The regular spend wire itself defines a limited proof boundary. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` documents `TxProofWire.spend` and `TxAuthWire.spend` as the canonical public spend seam for accepted spend paths, while the same file explicitly says input membership stays in the checkpoint or pre-state path rather than inside the local tx proof.

2. The current public spend verifier is real, but scoped. `crates/z00z_wallets/src/core/tx/spend_verification.rs` checks proof/auth presence, schema versions, nonzero `prev_root`, valid receiver card material, input-to-proof-row pairing, recomputed `leaf_ad` hashes, output stealth-field relations, output range proofs, commitment balance, and final spend authorization. That file does not elevate the contract into a complete whole-chain ownership or universal trustless-verifier proof story.

3. The repository's own status note narrows the claim explicitly. `docs/code-review/032-scenario-1-crypto-status.md` says the tree does not claim a universal trustless public spend verifier for every wallet-local ownership invariant beyond the current accepted boundary, does not claim an authoritative whole-chain or on-chain verifier deployment, and does not claim the broader original `PH32-SPEND` requirement is fully closed because the current regular-spend wire and public statement still do not bind nullifier semantics.

4. Live tests confirm the narrower seam rather than the stronger claim. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` proves the canonical public spend contract accepts valid packages and rejects placeholder gaps, replayed `prev_root`, and tampered `leaf_ad` relations, while `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` proves Scenario 1 stage-4 acceptance reuses that same wallet public verifier.

5. The Phase 033 draft `033-32EXAM-QEST-DRAFT.md` matches the same conclusion: the repository does not justify the full claim as originally stated and still describes the spend or validator path as lacking a complete public ZK verifier. This draft is only corroborating evidence; the controlling proof remains the live code and status note above.

**Reasoning:**

- The honest reading of the repository is not “there is no public verifier,” but rather “the public verifier that exists is narrower than the stronger original claim.”

- The live verifier proves an accepted current-stack spend contract with real cryptographic checks.

- The original claim reaches further than that accepted seam into broader trustless, authoritative, or whole-chain proof semantics.

- The repository itself rejects that stronger reading in both code comments and status documentation, especially around membership split and missing nullifier semantics.

**Gap Or Blocker:** The blocking gap is not absence of all proof checking; it is overclaim beyond the current accepted boundary. The repo still does not justify a fully finished universal trustless public spend verifier, authoritative whole-chain deployment, or full closure of the broader original `PH32-SPEND` wording.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer distinguishes “real current-stack public spend verifier” from “full original claim closure.”
- Required narrowing applied: no claim is made about whole-chain, on-chain, checkpoint-recursive, or universal wallet-local ownership verification beyond the repository's accepted public-boundary seam.

### 4. Three Most Important Conclusions

🔴 **Quest:** What are the three most important current conclusions about sender-side secret knowledge, public spend verification, and the unfinished proof boundary?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The three most important current conclusions are these: Alice does know sender-side output secret material in the live implementation; the current public spend verifier is real but narrower than a complete authoritative public ZK boundary; and the unresolved issue is therefore the unfinished proof boundary, not sender ignorance of `s_out`.

**Evidence Trail:**

1. The sender-side output path derives the secret material directly. `crates/z00z_wallets/src/core/stealth/output_build.rs` states that current Scenario 1 semantics keep sender-side derivation of `k_dh` and `s_out` explicit during output construction, then actually computes `s_out = derive_s_out(...)` and places it into the encrypted `AssetPackPlain` payload.

2. The broader stealth module repeats the same ownership split. `crates/z00z_wallets/src/core/stealth/output.rs` defines wallet-local two-factor ownership verification as `receiver_secret + s_out`, but explicitly warns that this must not be read as proof that the current public verifier already proves the same property end to end.

3. The public spend verifier is not fake. `crates/z00z_wallets/src/core/tx/spend_verification.rs` verifies proof/auth presence, versions, `prev_root`, receiver-card validity, input and output transcript relations, output range proofs, commitment balance, and final authorization signature.

4. But that verifier is narrower than the strongest claim. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` says the local transaction proof carries only local spend-proof material and leaves consumed-input membership in the checkpoint or pre-state path, while `docs/code-review/032-scenario-1-crypto-status.md` explicitly says the tree does not claim a universal trustless public spend verifier beyond the current accepted boundary and does not claim the broader original `PH32-SPEND` wording is fully closed.

5. The Phase 033 draft `033-32EXAM-QEST-DRAFT.md` synthesizes the same reading: the design and report direction support sender knowledge of `s_out`, and the missing piece is the finished public proof boundary rather than sender ignorance.

**Reasoning:**

- Once the sender path explicitly derives `k_dh` and `s_out`, the old story “Alice does not know the asset secret” stops being the honest central model.

- The repository still has a meaningful public-verification seam, so the honest conclusion is not “nothing is verified,” but “the verified seam is narrower than the strongest claim.”

- The strongest unresolved issue is therefore proof-boundary completeness: which ownership, membership, nullifier, and validator-facing semantics are enforced by the accepted public contract versus only by wallet-local or upstream structure.

- That is why the honest audit emphasis shifts away from sender ignorance and toward unfinished public-proof closure.

**Gap Or Blocker:** None for the three conclusions themselves. The blocker sits beneath conclusion two and three: the current accepted boundary is real but incomplete relative to the stronger authoritative end-to-end claim, especially where wallet-local ownership semantics must be promoted into a finished public-proof boundary.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: “not backed by a complete public ZK verifier” is stated as “real current-stack verifier, but not a complete authoritative end-to-end public boundary.”
- Required narrowing applied: the design-direction claim is grounded in live sender code plus repository status and draft artifacts, not in a broader undocumented architectural assumption.

### 5. Alice And The Asset Secret

🔴 **Quest:** Is the claim that Alice does not know the asset secret still defensible, given the sender-side derivation of `k_dh` and `s_out`?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** No, not if “asset secret” means the output secret `s_out` or the derived asset-secret handle `H(s_out)`. In the current implementation Alice's sender-side build path derives `k_dh`, derives `s_out`, and encrypts a payload that already contains that `s_out`. The claim would only stay defensible if “asset secret” were being used to mean Bob's separate `receiver_secret`, which this sender-path evidence does not prove Alice knows.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/stealth/output_build.rs` states the accepted semantics openly: sender-side derivation of `k_dh` and `s_out` remains explicit during output construction and must be documented honestly rather than hidden behind a sender-ignorance story.

2. The same file performs that derivation concretely. It derives `k_dh`, then computes `s_out = derive_s_out(...)`, then places `s_out` into `AssetPackPlain { value, blinding, s_out }` before encrypting the pack.

3. The pack format confirms that `s_out` is the secret being carried. `crates/z00z_core/src/assets/leaf.rs` defines `AssetPackPlain` with a raw `s_out` field in its canonical byte layout.

4. `crates/z00z_core/src/assets/asset_validation.rs` defines the derived asset-secret handle as `H(s_out)`. Once the sender already knows `s_out`, the stronger statement that Alice does not know the asset secret handle tied to `s_out` is also no longer defensible on this path.

5. `crates/z00z_wallets/src/core/stealth/output.rs` reinforces the ownership split: wallet-local ownership is checked with `receiver_secret + s_out`, and the code explicitly warns that this should not be misread as a claim that the public verifier already proves the same property end to end.

6. The Phase 033 draft `033-32EXAM-QEST-DRAFT.md` matches that reading and already marks the old C4 statement as disputed precisely because sender code derives `k_dh` and `s_out` during output construction.

**Reasoning:**

- The live sender path is incompatible with a clean “Alice never knows the asset secret” narrative if that secret is `s_out` or anything derived directly from it.

- The receiver still contributes something essential, but that something is the separate `receiver_secret`, not ignorance of `s_out` on the sender side.

- So the honest model shifts from sender ignorance to two-factor ownership and an unfinished public-proof boundary.

- That makes the original sentence defensible only under a different secret definition than the one the code actually materializes in the output pack.

**Gap Or Blocker:** The only blocker is terminology drift. If the report uses “asset secret” loosely, it can collapse `s_out` and `receiver_secret` into one phrase and overstate what the sender-path evidence proves. The code supports only the narrower statement: Alice knows `s_out`, while Bob's independent `receiver_secret` remains a separate factor.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: “asset secret” is treated as `s_out` or `H(s_out)`, not automatically as Bob's `receiver_secret`.
- Required narrowing applied: the answer says the sender-path evidence defeats the sender-ignorance claim, but does not claim Alice knows Bob's receiver secret.

### 6. Receiver Secret Spend Exclusion

🔴 **Quest:** What evidence supports or disputes the claim that Alice still cannot spend without Bob's `receiver_secret`?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The repository contains real code evidence that wallet-local spend ownership and accepted spend-preparation logic depend on Bob's `receiver_secret`. But that does not justify the stronger end-to-end claim that Alice is already cryptographically excluded by a finished public proof boundary solely because she lacks Bob's `receiver_secret`.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/stealth/output.rs` defines wallet-local ownership as a two-factor rule over `receiver_secret` and `s_out` in `verify_owner_two_factor(...)`, and the surrounding comment explicitly warns that this rule must not be read as proof that the current public verifier already proves the same property end to end.

2. `crates/z00z_wallets/src/core/tx/witness_gate.rs` shows the accepted spend-preparation path really starts from `recv_sec`: it converts `recv_sec` into `ReceiverSecret`, derives `ReceiverKeys`, runs `receiver_scan_input(...)`, and fails closed if the input is not decryptable under that receiver-side secret material.

3. `crates/z00z_wallets/src/core/tx/spend_verification.rs` also wires the receiver secret directly into local spend-rule logic. `build_spend_assets(...)` rejects zero or invalid `recv_sec`, converts it into `ReceiverSecret`, builds `SpendStmt { receiver_secret, spend_ins, c_outs, range_ok }`, and then calls `verify_spend_rules(...)`.

4. The public boundary is narrower than that local rule. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` carries only the public spend proof/auth seam and explicitly leaves consumed-input membership in the checkpoint or pre-state path rather than inside the local tx proof. `crates/z00z_wallets/src/core/tx/spend_verification.rs` verifies that public seam, but it does not expose Bob's receiver secret as a public witness or claim a universal whole-chain exclusion proof from that fact alone.

5. The repository's own status note agrees with the narrower reading. `docs/code-review/032-scenario-1-crypto-status.md` says there is no universal trustless public spend verifier beyond the current accepted boundary and that the broader original `PH32-SPEND` requirement is still not fully closed because nullifier semantics remain outside the finished public statement.

6. The Phase 033 draft `033-32EXAM-QEST-DRAFT.md` reaches the same conclusion in audit language: receiver-secret-gated spend authorization is part of the intended design and local rule logic, but the stronger end-to-end exclusion claim remains unproven while the public proof boundary is unfinished.

**Reasoning:**

- The live code really does use Bob's receiver secret on the wallet-local side; this is not speculative design text.

- That supports the narrower statement that local spend preparation and ownership checks depend on Bob-side secret material.

- But the stronger statement about Alice being cryptographically excluded at the final public verification boundary needs more than local dependency; it needs a finished public proof contract that carries and enforces that exclusion story all the way through accepted state transition semantics.

- The repository itself says that broader closure is not yet claimed.

**Gap Or Blocker:** The blocker is not lack of Bob-secret dependence in local code. The blocker is promotion: the repository does not yet justify that this wallet-local secret gate has become a fully finished authoritative public-proof exclusion rule for every accepted spend or checkpoint boundary.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer distinguishes wallet-local and accepted spend-preparation dependence on `receiver_secret` from a stronger end-to-end public-proof guarantee.
- Required narrowing applied: no claim is made that Alice can or cannot pass every final public boundary solely from the absence of Bob's receiver secret, because the repository still treats that broader proof story as unfinished.

### 7. Trustless Spend Verification

🔴 **Quest:** Can a validator trustlessly verify spend correctness with a real ZK verifier in the current repository state?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Not in the broad strong sense. The repository does support validator-facing verification of a real current-stack public spend contract, but it does not yet justify the stronger statement that full spend correctness is already trustlessly enforced by a complete end-to-end real ZK verifier.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/spend_verification.rs` implements a real public spend verifier. `verify_tx_public_spend_contract(...)` checks proof/auth presence, schema versions, nonzero `prev_root`, receiver-card validity, input-proof pairing, `leaf_ad` transcript binding, output stealth fields, output range proofs, commitment balance, and final spend authorization.

2. This verifier is exercised at live seams, not only as dead code. `crates/z00z_wallets/tests/test_spend_witness_gate.rs` accepts canonical spend statements and rejects structural-only placeholder gaps, while `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs` proves Scenario 1 persisted transaction packages are checked by that same wallet public verifier.

3. The same repository also narrows what that verifier means. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` says the local tx proof carries only local spend-proof material and leaves consumed-input membership in the checkpoint or pre-state path, where validators must separately resolve the consumed leaf and membership witness against `prev_root`.

4. The official status note in `docs/code-review/032-scenario-1-crypto-status.md` makes the limitation explicit: the tree does not claim a universal trustless public spend verifier beyond the current accepted boundary, does not claim authoritative whole-chain or on-chain verifier deployment, and does not claim the broader original `PH32-SPEND` wording is fully closed because nullifier semantics are still missing from the finished public statement.

5. The Phase 033 draft `033-32EXAM-QEST-DRAFT.md` reaches the same dispute classification for C6: the current spend gate should not be promoted into a full public-proof-verifier claim for all validator-facing spend correctness semantics.

**Reasoning:**

- The honest answer is not “there is no verifier.” There is a real public spend verifier in the tree today.

- The honest answer is also not “validators already have a complete trustless ZK story for full spend correctness.” The repository itself rejects that stronger reading.

- The missing pieces are broader than output range checks and authorization; they include the still-open semantics that live outside the local spend proof boundary, especially membership and nullifier closure.

- So the correct classification is narrow acceptance, not full trustless verifier closure.

**Gap Or Blocker:** The blocker is scope. The current verifier proves the accepted current-stack spend contract, but the repository still does not justify a complete end-to-end validator-proof boundary for all spend-correctness semantics that the stronger claim would require.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer affirms a real current-stack validator-facing verifier, but rejects overclaim to full trustless spend-correctness closure.
- Required narrowing applied: the answer explicitly keeps membership, nullifier, and broader checkpoint or whole-chain semantics outside the already-proven spend seam.

### 8. Authoritative Publish Proofs

🔴 **Quest:** Is checkpoint or JMT publish already backed by authoritative proof verification in the current repository state?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** No in the strong sense. The current tree demonstrates real package-coupled integrity and replay-consistency checks at the accepted simulator and storage boundary, but it does not demonstrate a standalone authoritative checkpoint or JMT publish-proof backend.

**Evidence Trail:**

1. `crates/z00z_storage/src/checkpoint/artifact_final.rs` and `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs` both require `cp_proof` to be non-empty, but their own comments describe those bytes as verifier-bound compatibility payload bytes rather than the canonical replay-evidence binding or a checkpoint-identity-defining proof backend.

2. `crates/z00z_storage/src/checkpoint/artifact_types.rs` shows the proof-system taxonomy clearly: `LEGACY_OPAQUE`, `OPAQUE_ATTEST`, and a distinct `VERIFIED` value. The live artifact constructors in the current path use the opaque or attestation modes, not a demonstrated `VERIFIED` publish-proof mode.

3. `crates/z00z_storage/src/checkpoint/codec.rs` enforces fail-closed compatibility checks: artifact encode/decode rejects empty `cp_proof`, validates proof-system compatibility, and preserves statement binding, but it still does not perform cryptographic verification of a standalone publish proof.

4. `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` states the current stack persists transaction-proof bytes as the compatibility payload for checkpoint promotion and says this binds Stage 8 to the package contract, but is not a standalone checkpoint-proof backend.

5. The accepted simulator handoff does contain real integrity checks. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` reuses the current-stack public spend verifier through `CheckpointPackageProofVerifier::verify_pkg_contract(...)`, and `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` rejects mismatches between the exec row and the already verified stage-4 package for tx proof bytes, input refs, and canonical outputs.

6. `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` proves the narrow rejection story: tampered exec-input rows, tampered stage-4 proof material, replay-style reload drift, and package digest tamper are rejected before accepted checkpoint emission or reload. But these tests do not prove that arbitrary stored `cp_proof` rows are authoritatively cryptographically verified as a publish-proof backend.

7. The repository's own status note in `docs/code-review/032-scenario-1-crypto-status.md` explicitly says no recursive checkpoint proof system and no authoritative whole-chain or on-chain verifier deployment are claimed. The Phase 033 draft `033-32EXAM-QEST-DRAFT.md` matches that classification by marking the stronger publish-proof claim as disputed.

**Reasoning:**

- The repository does more than merely store unchecked bytes; accepted handoff and replay-consistency checks are real.

- But those checks are package-coupled and compatibility-oriented, not proof-backend-complete.

- The presence and linkage of `cp_proof` are enforced, yet that is not the same thing as authoritative cryptographic verification of checkpoint or JMT publish.

- So the honest answer is integrity checks yes, authoritative publish-proof closure no.

**Gap Or Blocker:** The blocker is backend completeness. The current path still treats `cp_proof` as opaque or attested compatibility payload tied to prior package checks, rather than as a demonstrated authoritative publish proof that independently closes checkpoint or JMT verification in the strong sense.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: rejection evidence is stated only for tampered exec rows and tampered stage-4 package handoff, not for arbitrary stored `cp_proof` rows under a general cryptographic publish verifier.
- Required narrowing applied: the answer distinguishes package-coupled integrity and attestation checks from a standalone authoritative checkpoint or JMT publish-proof backend.

### 9. Real Theft-Resistance Boundary

🔴 **Quest:** Does the current theft-resistance story come from an additional receiver-secret requirement at spend time rather than from sender ignorance of output-side material?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** Yes, in the live repository the defensible current theft-resistance story comes from an additional `receiver_secret` requirement at spend time, not from sender ignorance of output-side material. But that remains a wallet-local or current-boundary statement, not a fully proven end-to-end public-proof guarantee.

**Evidence Trail:**

1. The sender-ignorance theory is directly contradicted by the live sender path. `crates/z00z_wallets/src/core/stealth/output_build.rs` says current Scenario 1 semantics keep sender-side derivation of `k_dh` and `s_out` explicit, then actually derives `s_out = derive_s_out(...)` and places it into `AssetPackPlain` during output construction.

2. Wallet-local ownership logic explicitly adds a second factor beyond output-side material. `crates/z00z_wallets/src/core/stealth/output.rs` defines local ownership verification through `verify_owner_two_factor(receiver_secret, s_out)` and states that live wallet ownership is modeled as `receiver_secret + s_out`, while also warning that this must not be overstated as an end-to-end public verifier claim.

3. Spend preparation really depends on the receiver secret. `crates/z00z_wallets/src/core/tx/witness_gate.rs` resolves the input pack from `recv_sec`, converts it into `ReceiverSecret`, derives receiver keys, and fails closed if the candidate cannot be decrypted or resolved under that secret material.

4. Local spend-rule construction carries the same dependency into spend verification prep. `crates/z00z_wallets/src/core/tx/spend_verification.rs` builds `SpendStmt { receiver_secret, ... }` for the local spend-rules path and verifies it before the public spend contract is accepted.

5. The repository's own status note narrows the claim. `docs/code-review/032-scenario-1-crypto-status.md` says the tree does not claim a universal trustless public spend verifier beyond the current accepted boundary and does not claim full closure of the broader spend requirement. The Phase 033 draft `033-32EXAM-QEST-DRAFT.md` mirrors the same split between intended design or local rule logic and unfinished public-proof closure.

**Reasoning:**

- Once the sender path explicitly derives `s_out`, the honest anti-theft explanation can no longer be “Alice cannot steal because she never knows the output-side secret material.”

- The live repository instead supports a narrower statement: spend ownership and spend preparation require an additional receiver-held secret, so sender knowledge of `s_out` alone is not enough for wallet-local spend authorization.

- That is the real current theft-resistance boundary the code supports today.

- But the same repository also says the public verifier boundary is not yet broad enough to upgrade that statement into a final end-to-end trustless proof guarantee.

**Gap Or Blocker:** The blocker is proof-boundary completeness. The repository supports receiver-secret-gated spend logic locally and at the current accepted spend-preparation seam, but it still does not justify the stronger claim that the full public verifier or final chain or checkpoint boundary already proves theft exclusion end to end.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer states a real current theft-resistance boundary in live code, but only as a wallet-local or current-boundary rule rather than a finished universal public-proof guarantee.
- Required narrowing applied: sender ignorance is rejected specifically because the sender path derives `s_out`; the surviving anti-theft explanation is the additional `receiver_secret` requirement at spend time.

### 10. Ideas-Document Overclaim

🔴 **Quest:** Is the ideas-document sentence that Alice still cannot steal because `Spend-TxProof` requires `receiver_secret` stronger than the live proof boundary?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Yes. That ideas-document sentence matches the intended ownership model and live wallet-local spend logic, but it is stronger than what the current live proof boundary justifies as a completed verifier-backed property.

**Evidence Trail:**

1. The ideas document really makes that claim. `.planning/temp/Z00Z-ECC-IDEAS.md` says, in substance, that sender-side knowledge is compatible with non-spendability because spend proof or spend constraints require `receiver_secret`, including the statements that sender cannot assemble the witness without `receiver_secret` and that even knowledge of sender-generated output material is not enough to pass ownership constraints.

2. The live wallet code supports that statement as local ownership logic. `crates/z00z_wallets/src/core/stealth/output.rs` defines the accepted wallet-local spend ownership rule through `verify_owner_two_factor(receiver_secret, s_out)` and explicitly warns that this must not be read as proof that the current public verifier path already proves the same property end to end.

3. Spend preparation also carries real receiver-secret dependence. `crates/z00z_wallets/src/core/tx/witness_gate.rs` resolves spend input state from `recv_sec`, while `crates/z00z_wallets/src/core/tx/spend_verification.rs` constructs the local spend statement with `receiver_secret` and verifies that local rule path before the public spend contract is accepted.

4. The live public proof boundary is narrower than the document wording sounds. `crates/z00z_wallets/src/core/tx/spend_verification.rs` does verify a real current-stack public spend contract, but `docs/code-review/032-scenario-1-crypto-status.md` explicitly says the tree does not claim a universal trustless public spend verifier for every wallet-local ownership invariant beyond the current accepted boundary and does not claim full closure of the broader spend requirement.

5. The older Scenario 1 audit says the same thing explicitly. `.planning/phases/000/032-crypto-audit-scenario-1/032-TODO.md` treats the sentence as supported by intended design and local rule logic, but not yet as a finished end-to-end verifier-backed implementation property of the live stack.

**Reasoning:**

- The problem is not that the ideas document invents a false local rule. The receiver-secret-gated spend story is real in the intended model and in the wallet-local code path.

- The problem is that the sentence sounds stronger than the live proof boundary actually is today.

- In the current repository, it can be defended as design intent plus local ownership enforcement.

- It cannot yet be defended as a fully closed authoritative verifier-backed property at the final public spend or checkpoint acceptance boundary.

**Gap Or Blocker:** The blocker is unfinished public-proof closure. The repository still lacks sufficient proof-boundary completeness to promote receiver-secret-gated anti-theft language from intended design and wallet-local enforcement into a finished end-to-end live verifier guarantee.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the sentence is treated as intended protocol direction and wallet-local enforcement, not as a completed end-to-end verifier-backed property of the live stack.
- Required narrowing applied: the answer distinguishes real local receiver-secret gating from stronger authoritative JMT or checkpoint-boundary proof claims.

### 11. Intended Design Versus Public Proof

🔴 **Quest:** Is receiver-secret-gated spend authorization currently supported as intended design and local rule logic, but not yet justified as a full end-to-end public-proof guarantee?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Yes. Receiver-secret-gated spend authorization is supported in the repository today as intended design and live wallet-local or spend-preparation rule logic, but the repository does not yet justify that property as a fully closed end-to-end authoritative public-proof guarantee.

**Evidence Trail:**

1. Intended design is explicit. `.planning/temp/Z00Z-ECC-IDEAS.md` says spend authority depends on `receiver_secret` and that sender-side knowledge alone must not be enough to assemble a valid spend witness.

2. The live wallet code implements that design locally. `crates/z00z_wallets/src/core/stealth/output.rs` defines `verify_owner_two_factor(receiver_secret, s_out)` as the accepted wallet-local ownership rule and explicitly warns that this must not be read as proof that the current public verifier path already proves the same property end to end.

3. Spend preparation depends on the same receiver secret. `crates/z00z_wallets/src/core/tx/witness_gate.rs` resolves spend input state from `recv_sec`, derives receiver keys from that secret, and rejects candidates that cannot be decrypted or resolved under it.

4. Local spend-rule verification also carries the dependency forward. `crates/z00z_wallets/src/core/tx/spend_verification.rs` converts the witness-gate output into `receiver_secret`, builds `SpendStmt { receiver_secret, ... }`, and verifies that local spend-rules path before the public spend contract is accepted.

5. The public proof boundary is explicitly narrower. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` says the local transaction proof carries only local spend-proof material while consumed-input membership remains in the checkpoint or pre-state path, and `docs/code-review/032-scenario-1-crypto-status.md` explicitly disclaims a universal trustless public spend verifier or full closure of the broader spend requirement at the current authoritative boundary.

**Reasoning:**

- The repository clearly supports receiver-secret-gated spend authorization as design intent and as real live local enforcement.

- So the correct answer is not that the property is absent.

- The remaining limitation is boundary scope: the wallet and spend-preparation layers enforce it, but the repository still stops short of proving that the authoritative public spend or checkpoint boundary already closes the same property end to end.

- That is why the honest wording is “supported as intended design and local rule logic, but not yet justified as a full public-proof guarantee.”

**Gap Or Blocker:** The blocker is still public-proof completeness. The repository has real local receiver-secret gating, but it does not yet provide enough proof-boundary closure to claim that validators or final checkpoint acceptance already enforce the entire anti-theft property as a finished end-to-end verifier-backed invariant.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer distinguishes intended design and live wallet-local or spend-preparation logic from the unfinished authoritative public-proof boundary.
- Required narrowing applied: no claim is made that the current public spend or checkpoint verifier already proves the full receiver-secret-gated property end to end.

## ⚖️ Theme 2: Trust Model, Drift, And Ownership Logic

### 12. Incomplete Validator Trust Model

🔴 **Quest:** What exactly is incomplete about the validator-facing spend or checkpoint trust model, and which stronger trustless-verification claim is still unproven?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** What is incomplete is not the existence of validator-facing checking, but the final cryptographic closure of that trust model. The repository has a real current-stack public spend verifier and real package-coupled checkpoint handoff integrity checks, yet it still does not demonstrate a standalone authoritative checkpoint-publish proof backend or a universal validator-grade proof boundary that forces full privacy-preserving spend correctness across every accepted JMT transition.

**Evidence Trail:**

1. The repository does contain a real validator-facing public spend verifier. `crates/z00z_wallets/src/core/tx/spend_verification.rs` verifies proof/auth presence, schema versions, `prev_root`, receiver-card validity, input pairing, `leaf_ad` binding, range proofs, balance, and final spend authorization.

2. But the spend wire itself defines a narrower proof surface. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` explicitly says the proof object carries local transaction-proof material only, while consumed-input membership remains in the checkpoint or pre-state path instead of being fully closed inside the local spend proof.

3. The checkpoint layer still treats `cp_proof` as compatibility or attestation-style payload rather than as a demonstrated authoritative standalone publish proof. `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs` and `crates/z00z_storage/src/checkpoint/artifact_final.rs` describe `cp_proof` as compatibility or verifier payload material, while canonical binding still rides on statement data, roots, and replay references.

4. The simulator checkpoint seam reinforces that narrower reading. `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` and `bundle_lane_impl.rs` describe proof bytes and package construction as compatibility or handoff material, while stage-11 and checkpoint tests focus on tamper and replay resistance of the package or handoff path rather than on an independently demonstrated recursive publish-proof backend.

5. The repository's own status note explicitly disclaims the stronger trustless claim. `docs/code-review/032-scenario-1-crypto-status.md` says the tree does not claim a universal trustless public spend verifier beyond the current accepted boundary, does not claim authoritative whole-chain or on-chain verifier deployment, and does not claim full closure of the broader spend requirement.

**Reasoning:**

- The honest current model is therefore split.

- Validators can check a real public spend contract at the current accepted seam.

- Checkpoint promotion and finalization also enforce real package-coupled integrity and replay or tamper checks.

- But the repository still stops short of proving the stronger theorem that an untrusted validator or aggregator can verify full privacy-preserving spend correctness from a proof object alone and that every accepted final JMT transition is cryptographically forced to satisfy that property under an authoritative standalone publish-proof backend.

**Gap Or Blocker:** The missing piece is authoritative final-boundary proof closure. Membership remains split out of the local spend proof, and checkpoint publication still relies on package-coupled integrity and compatibility payloads rather than a fully demonstrated standalone recursive or publish-proof verifier.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer affirms a real current-stack validator-facing verifier and real handoff integrity checks, instead of overstating the gap as total absence of verification.
- Required narrowing applied: the strongest unproven claim is phrased as lack of a standalone authoritative validator-grade proof boundary for full spend correctness across the final accepted checkpoint or JMT transition.

### 13. Aggregator Anti-Theft Scope

🔴 **Quest:** Do current aggregator membership and batch-shape checks justify a stronger anti-theft claim, or is that still blocked on a real proof verifier?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Current aggregator membership, replay, package-shape, and tamper checks are real and meaningful, but they do not justify the stronger anti-theft claim that the aggregator is prevented from theft specifically because it lacks Bob's secrets. That stronger anti-theft statement is still blocked on an authoritative final proof verifier.

**Evidence Trail:**

1. The aggregator-facing checks are not fake. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` re-verifies the package contract before draft creation, and `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs` shows rejection of tampered exec rows, tampered package proof bytes, tampered digest handoff, and replay-style reload violations.

2. Membership continuity and replay controls are also real. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` keeps resolved input state through membership witnesses or resolved-input material, binds that to `prev_root`, and separately fail-closes replay-spent tracking.

3. But the repository itself narrows what those checks mean. `bundle_lane_impl.rs` describes the verifier as a current-stack package-coupled verifier, while `exec_input_builder.rs` says checkpoint proof bytes are compatibility payload and not a standalone checkpoint-proof backend.

4. Bob-secret dependence does exist elsewhere in the stack, but only at a narrower boundary. `crates/z00z_wallets/src/core/tx/spend_verification.rs` builds local spend rules with `receiver_secret`, and `crates/z00z_wallets/src/core/stealth/output.rs` explicitly says the wallet-local two-factor ownership rule must not be overstated as an already-proven end-to-end public-verifier property.

5. The repository's own status and freeze documents reject the stronger anti-theft reading. `docs/code-review/032-scenario-1-crypto-status.md` disclaims a universal trustless public spend verifier and authoritative whole-chain verifier closure, while `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md` explicitly narrows the truthful anti-theft statement to current accepted boundaries.

**Reasoning:**

- The aggregator is constrained today by meaningful structural and continuity checks.

- Those checks help prevent tampering, replay, and malformed package promotion.

- But they are not yet the same thing as a completed theorem that the aggregator cannot steal specifically because it lacks Bob's secrets.

- That Bob-secret-based anti-theft statement only becomes fully defensible once the final public-proof or checkpoint-verifier boundary authoritatively closes the same property.

**Gap Or Blocker:** The blocker is authoritative proof closure at the final boundary. The repository supports Bob-secret gating as wallet-local or spend-preparation logic, but the aggregator-facing path still stops at package-coupled integrity and compatibility checks rather than a final proof object that independently closes theft exclusion.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer affirms meaningful aggregator integrity checks instead of dismissing them as no-op structure.
- Required narrowing applied: the stronger rejected claim is specifically Bob-secret-based theft exclusion at the final authoritative boundary, not generic tamper resistance or replay rejection.

### 14. Boundary Ambiguity

🔴 **Quest:** Which boundary ambiguity, while not a demonstrated break by itself, still makes proof, wire, and runtime drift likely enough to block stronger formal claims?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The key boundary ambiguity is the exact `leaf_ad_id` contract at the decrypt, receive, and spend boundary. The accepted flow works, but the repository still carries a subtle coexistence between canonical decrypt-associated-data semantics and separate asset state-key semantics, and that ambiguity is exactly the kind of proof, wire, and runtime drift risk that blocks stronger formal claims.

**Evidence Trail:**

1. The prior audit already names this as the issue. `.planning/phases/000/032-crypto-audit-scenario-1/032-TODO.md` says `leaf_ad_id` semantics are not yet strong enough to support a nearly formal statement without caveats and explicitly identifies the coexistence of `asset.asset_id()` and `leaf_ad_id()` as the source of semantic subtlety.

2. The semantic freeze confirms that this boundary is supposed to be canonical. `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md` says `leaf_ad_id` is the canonical decrypt-associated-data asset identifier and treats drift between the stored leaf asset identifier and the decrypt-associated identifier as a semantic failure rather than as harmless compatibility detail.

3. The live asset and wire code preserve that split explicitly. `crates/z00z_core/src/assets/assets.rs` stores `leaf_ad_id` as an optional parity field, while the asset state key remains separately derived. Full-stealth wire validation in the assets layer requires `leaf_ad_id` when the full stealth path is used.

4. The spend or receive path then rebases onto that decrypt boundary. `crates/z00z_wallets/src/core/tx/witness_gate.rs` explicitly rebases `leaf.asset_id = leaf_ad_id` for decrypt resolution, and its tests prove both that decrypt uses `leaf_ad_id` and that missing `leaf_ad_id` fails closed.

5. Runtime regression tests confirm the same fragility. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` shows that the accepted ownership path works when `leaf_ad_id` matches the canonical decrypt boundary and breaks when that value is tampered, which proves the contract is live but also subtle enough that it must stay frozen consistently across builder, runtime, wire, tests, and proof statements.

**Reasoning:**

- This is not evidence of a live exploit on the accepted path.

- It is evidence that the boundary is subtle enough that multiple neighboring layers must agree exactly on what `leaf_ad_id` means.

- As long as canonical decrypt-boundary meaning and separate state-key meaning coexist without a single fully frozen theorem across proof, wire, and runtime, stronger formal claims remain risky.

- That is why the repository treats this as a semantic-freeze issue rather than as a completed fully formalized invariant.

**Gap Or Blocker:** The blocker is canonical contract freeze across all layers. Until builder, runtime, wire, regression tests, and proof statements all stay pinned to the same `leaf_ad_id` boundary semantics without caveat, the repository cannot honestly make stronger near-formal or universal claims here.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer does not claim a demonstrated break, only a drift-prone semantic ambiguity.
- Required narrowing applied: the ambiguity is identified specifically as the `leaf_ad_id` decrypt-boundary versus separate asset state-key contract, not as a vague general inconsistency.

### 15. Sender Knowledge Of `s_out`

🔴 **Quest:** Does current evidence support sender ignorance of `s_out`, or instead show a need to freeze canonical `s_out` semantics?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Current evidence does not support sender ignorance of `s_out`. It instead shows a need to freeze canonical `s_out` semantics. Both the live repository and the cited design documents allow sender knowledge of `s_out`; the real unresolved issue is drift in how `s_out` is derived or described.

**Evidence Trail:**

1. The live code directly rejects a sender-ignorance model. `crates/z00z_wallets/src/core/stealth/output_build.rs` says current Scenario 1 semantics keep sender-side derivation of `k_dh` and `s_out` explicit, then computes `s_out = derive_s_out(...)` and stores it in `AssetPackPlain`.

2. The canonical code formula is sender-derived, not sender-unknown. `crates/z00z_wallets/src/core/stealth/ecdh.rs` defines `derive_s_out(...)` as the single canonical implementation and derives `s_out` from sender-held context.

3. The anti-theft logic in code relies on an additional receiver-held factor, not on sender ignorance of `s_out`. `crates/z00z_wallets/src/core/stealth/output.rs` defines wallet-local ownership as `receiver_secret + s_out` and explicitly warns against overstating that as a finished end-to-end public-verifier claim.

4. The cited ideas and spec documents also allow sender knowledge of `s_out`. `.planning/temp/Z00Z-ECC-IDEAS.md` describes a model where sender chooses `s_out = random32`, and also says that even if sender knows `s_out` and all sender-generated output material, sender still cannot spend without `receiver_secret`. `.planning/temp/Z00Z-ECC-SPEC_part1.md` follows the same direction.

5. The real mismatch is therefore derivation semantics. The documents describe a sender-known but randomly generated `s_out`, while the live code canonically derives `s_out` from sender-held context. `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md` already freezes the honest current Scenario 1 rule and rejects the overclaim that sender cannot know `s_out`.

**Reasoning:**

- If both the code and the cited design documents allow sender knowledge of `s_out`, then sender ignorance cannot be the honest reading of the evidence.

- The repository-backed issue is instead semantic drift about how `s_out` is produced.

- That drift matters because stronger proof or protocol claims should not sit on an unresolved split between documented derivation semantics and live implementation semantics.

- So the corrective action is semantic freeze and canonical alignment, not a claim that sender should be ignorant of `s_out`.

**Gap Or Blocker:** The blocker is canonical alignment of the `s_out` derivation contract across docs and implementation. Until the repository fully freezes whether `s_out` is described as sender-random or sender-derived, stronger proof-language claims should continue to carry an explicit caveat.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer treats this as a current repository interpretation about design drift, not as a finished end-to-end proof statement.
- Required narrowing applied: sender knowledge of `s_out` is affirmed in both code and cited documents, while the unresolved issue is limited to derivation semantics.

### 16. Receiver Secret At Spend

🔴 **Quest:** Should spending additionally require receiver-secret knowledge, and how is that requirement expressed today?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Yes. Spending should additionally require receiver-secret knowledge. In the current repository that requirement is expressed as receiver-secret-gated wallet-local ownership, spend-input resolution from `recv_sec`, and local spend-rule construction that carries `receiver_secret` into `SpendStmt`.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md` freezes the honest current anti-theft statement as receiver-secret-gated wallet-local ownership layered on top of output secret material.

2. `crates/z00z_wallets/src/core/stealth/output.rs` defines `verify_owner_two_factor(...)` as the accepted wallet-local rule and enforces ownership through `receiver_secret`, `owner_tag`, and expected `s_out`.

3. `crates/z00z_wallets/src/core/tx/witness_gate.rs` resolves the input pack through `recv_sec`, converts it into `ReceiverSecret` and receiver keys, and recovers `s_out` from that same receiver-held path.

4. `crates/z00z_wallets/src/core/tx/spend_verification.rs` turns `wit.recv_sec` into `ReceiverSecret`, builds `SpendStmt { receiver_secret, ... }`, and runs the local spend-rules path with that material.

**Reasoning:** The current repository therefore does not treat receiver-secret knowledge as optional decoration. It is part of the accepted ownership and spend-preparation boundary today.

**Gap Or Blocker:** This is proven as wallet-local and spend-preparation logic, not yet as a finished chain-level authoritative theorem.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer is limited to live local enforcement and current accepted spend-preparation logic.

### 17. Two-Factor Spend Authority

🔴 **Quest:** In the intended B3 logic, does spend authority require both `receiver_secret` and `s_in`, and does the current repository prove that at chain level today?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Yes, in the intended B3 logic spend authority requires both receiver-secret-based ownership material and the spend-side secret `s_in`. But the current repository proves that only as intended design and local rule logic, not yet as a chain-level authoritative property.

**Evidence Trail:**

1. `.planning/temp/Z00Z-ECC-IDEAS.md` defines the intended B3 witness as `receiver_secret`, `view_sk`, and `s_in` plus balance material.

2. `.planning/temp/Z00Z-ECC-SPEC_part1.md` explicitly says ownership requires `receiver_secret + s_in` and describes that as strong ownership.

3. `crates/z00z_wallets/src/core/tx/spend_rules.rs` carries `receiver_secret` inside `SpendStmt`, derives `owner_handle` and `view_sk` from it, checks `owner_tag_in`, and separately binds `s_in`.

4. `docs/code-review/032-scenario-1-crypto-status.md` explicitly says the repository does not yet claim universal trustless public spend verification or authoritative whole-chain closure of that property.

**Reasoning:** So the intended model is clearly two-factor, and the local rule path reflects that model. The unfinished part is promotion into chain-level proof closure.

**Gap Or Blocker:** The blocker is authoritative public-proof or checkpoint-boundary closure, not absence of intended B3 logic.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: “yes” is confined to intended design and local rule logic, while chain-level closure remains unclaimed.

### 18. `s_in`-Only Counterfactual

🔴 **Quest:** If spend authority collapsed to `s_in` alone, would Alice be able to steal, and what would that imply about the intended B3 model?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Yes. If spend authority collapsed to `s_in` alone, that would destroy the intended B3 anti-theft model and make sender-side theft possible in the very scenario the design is trying to prevent.

**Evidence Trail:**

1. `.planning/temp/Z00Z-ECC-IDEAS.md` states directly that Spend-TxProof must require `receiver_secret`, otherwise sender steals.

2. The same ideas document also says that even if sender knows output-side material, ownership constraints still fail without `receiver_secret`.

3. `.planning/temp/Z00Z-ECC-SPEC_part1.md` describes strong ownership in exactly that receiver-secret-plus-spend-secret form.

4. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` shows live local evidence that sender-side substitute material fails the receiver gate.

**Reasoning:** This is a counterfactual about the intended model. If `s_in` alone were sufficient, the extra Bob-held ownership factor would disappear and the intended B3 anti-theft claim would collapse.

**Gap Or Blocker:** The repository does not implement a live alternate `s_in`-only branch; this conclusion is proven as intended-model reasoning backed by the current local enforcement language.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer treats this as intended-model and local-rule reasoning, not as a separate executed implementation branch.

### 19. Secret Inside The Coin Or Not

🔴 **Quest:** Is `receiver_secret` embedded inside the coin, or is ownership instead enforced through fields that only Bob's secret can satisfy?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** `receiver_secret` is not embedded inside the coin. Ownership is instead enforced through derived fields and relations that only Bob's secret can satisfy.

**Evidence Trail:**

1. `.planning/temp/Z00Z-ECC-SPEC_part1.md` says `receiver_secret` is never published and must never be transmitted.

2. The same spec says the encrypted pack contains `s_out`, value, randomness, and related sender-produced material, not `receiver_secret` itself.

3. `.planning/temp/Z00Z-ECC-IDEAS.md` says the leaf does not carry `owner_handle`, `view_pk`, or the full address directly.

4. The intended and local rule model instead derives `owner_handle`, `view_sk`, and related ownership checks from Bob's secret and tests those relations at receive or spend time.

**Reasoning:** The coin therefore does not carry Bob's secret as a public or embedded field. It carries state that only Bob's secret can satisfy when the derived ownership relations are checked.

**Gap Or Blocker:** None for the narrow question itself. The remaining gap is only whether final public-proof boundaries fully enforce the same property authoritatively.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer distinguishes non-public receiver-secret storage from derived ownership relations.

### 20. Post-JMT Anti-Theft Proof

🔴 **Quest:** Does the current repository prove that `receiver_secret + s_in` actually prevents Alice from stealing after the tx is inserted into JMT, or only the intended local rule?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The current repository proves only the intended local rule and the current accepted local boundary. It does not yet prove that `receiver_secret + s_in` is a finished post-JMT authoritative chain-level anti-theft theorem.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md` explicitly separates wallet-local ownership checks from public trustless verification and says the latter is not implied.

2. `crates/z00z_wallets/src/core/stealth/output.rs` warns that `verify_owner_two_factor(...)` must not be read as proof of the same property end to end.

3. `docs/code-review/032-scenario-1-crypto-status.md` rejects claims of universal trustless public spend verification beyond the current accepted boundary and rejects authoritative whole-chain verifier deployment claims.

4. `crates/z00z_wallets/src/core/tx/spend_verification.rs` still shows that the current spend contract is real, so the gap is specifically failure to promote the anti-theft property into a final authoritative post-JMT theorem.

**Reasoning:** The honest answer is therefore not “nothing is proven,” but “the final post-JMT theorem is not yet proven.”

**Gap Or Blocker:** The blocker remains final public-proof or checkpoint-boundary closure.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer affirms current accepted boundary checks while rejecting finished post-JMT authoritative closure.

### 21. Why Alice's Secret Fails

🔴 **Quest:** Why does sender-side substitute receiver material fail to reproduce the `owner_handle`, `view_sk`, and `owner_tag` relation required for Bob's coin?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Sender-side substitute receiver material fails because the required `owner_handle`, `view_sk`, and `owner_tag` relations are derived from Bob's `receiver_secret` and related receiver-held material. A sender-side substitute cannot recreate the exact relation bound into Bob's coin.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/spend_rules.rs` derives `owner_handle` and `view_sk` from `receiver_secret`, requires exact `owner_tag_in` agreement, and separately checks the `s_in` relation.

2. `.planning/temp/Z00Z-ECC-SPEC_part1.md` defines the exact derivation chain from `receiver_secret` into `owner_handle` and `view_sk`, and then into the ownership-tag relation.

3. `.planning/temp/Z00Z-ECC-IDEAS.md` says the wallet must recompute the expected owner fields from its own receiver secret and explains the decryptable-but-not-spendable case when a sender injects mismatched ownership material.

4. `crates/z00z_wallets/tests/test_scenario1_semantics.rs` gives live evidence that sender-side substitute receiver material does not satisfy the receiver gate.

**Reasoning:** Alice can reproduce sender-side material she created herself, but she cannot forge the Bob-secret-derived ownership relations unless she also has Bob's secret.

**Gap Or Blocker:** The remaining gap is not local derivation logic; it is only whether the entire property is already elevated into a finished whole-chain theorem.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer is grounded in derivation and validation seams plus local tests, not in an overclaimed whole-chain theorem.

### 22. Stealth Address Job Description

🔴 **Quest:** In the current design, what security job does stealth address perform, and what security job does it explicitly not perform by itself?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** In the current design, stealth address performs recipient recognition, routing, and decryptability checks. It does not, by itself, prove final spend authority or a finished public trustless anti-theft theorem.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md` defines stealth scan as the accepted ownership-recognition and decryptability surface.

2. `crates/z00z_wallets/src/core/address/leaf_scan.rs` and `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` show the live scan path returning owned or not-owned outcomes and treating decrypt failures as a separate boundary.

3. `crates/z00z_wallets/src/core/stealth/output.rs` marks `verify_owner_two_factor(...)` as a wallet-local ownership rule and explicitly warns against reading it as a full end-to-end public-proof claim.

**Reasoning:** The live stack therefore uses stealth address machinery to decide whether a leaf is Bob's and whether its payload can be opened. Final spend authority still depends on later spend-rule and verifier seams.

**Gap Or Blocker:** None for the narrow split itself. The remaining gap is only final public-proof closure.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer separates stealth-recognition duties from final trustless spend-proof duties.

### 23. B3 Code Seams And Verifier Boundary

🔴 **Quest:** Which live code seams model the intended B3 ownership logic, and which seam still keeps the public verifier boundary narrower than a finished trustless verifier?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The intended B3 ownership logic is modeled across wallet-local and spend-preparation seams, while the public verifier boundary remains narrower in the current wire and verifier contract.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/stealth/output.rs` provides the wallet-local `verify_owner_two_factor(...)` ownership seam.

2. `crates/z00z_wallets/src/core/tx/witness_gate.rs` models decrypt-or-fail witness recovery through `receiver_scan_input(...)`.

3. `crates/z00z_wallets/src/core/tx/spend_rules.rs` encodes the intended ownership relation with `receiver_secret`, `owner_tag_in`, `leaf_ad_id_in`, range checks, and balance checks.

4. `crates/z00z_wallets/src/core/tx/spend_verification.rs` builds spend assets with `receiver_secret` and invokes the spend-rule seam.

5. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` and `docs/code-review/032-scenario-1-crypto-status.md` show that the public verifier boundary is still narrower than a finished universal trustless verifier.

**Reasoning:** The repository therefore does model the intended ownership logic in real code, but it has not yet elevated that full ownership story into a final public-proof theorem.

**Gap Or Blocker:** The blocker remains the narrower public verifier and checkpoint boundary.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: “B3” is treated as planning shorthand for the modeled ownership split, not as a literal code symbol.

### 24. Canonical `s_out` Freeze

🔴 **Quest:** Has the repository frozen one canonical `s_out` model, or does a docs-versus-code semantic split still need to be resolved?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The accepted live code path has frozen one canonical `s_out` model, but a docs-versus-code semantic split still exists in legacy planning text and should be resolved explicitly.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md` says the current accepted stack uses a single canonical `s_out` interpretation and forbids sender-ignorance wording.

2. `crates/z00z_wallets/src/core/stealth/output_build.rs` shows sender-side derivation of `k_dh` and `s_out` in the accepted build path.

3. `crates/z00z_wallets/src/core/stealth/ecdh.rs` labels `derive_s_out(...)` as the single canonical implementation.

4. `.planning/temp/Z00Z-ECC-IDEAS.md` and `.planning/temp/Z00Z-ECC-SPEC_part1.md` still contain older `random32` wording that conflicts with the accepted live code.

**Reasoning:** The live stack is frozen. The unresolved work is documentation cleanup, not ambiguity in accepted code behavior.

**Gap Or Blocker:** Legacy documentation still overstates or contradicts the frozen model.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer distinguishes accepted code semantics from unresolved legacy document cleanup.

## 🗃️ Theme 3: Checkpoint Boundary And Publish Verdicts

### 25. Non-Empty Proof Bytes

🔴 **Quest:** Do storage and checkpoint code paths accept non-empty proof bytes rather than verifying a real recursive proof transcript at the final boundary?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Yes. The current storage and checkpoint code paths still accept non-empty `cp_proof` bytes plus boundary-consistency checks, rather than verifying an authoritative recursive proof transcript at the final boundary.

**Evidence Trail:**

1. `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs` accepts any non-empty `cp_proof` and does not perform recursive-proof verification.

2. `crates/z00z_storage/src/checkpoint/artifact_final.rs` does the same at the final artifact boundary.

3. `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` documents the current `cp_proof` handoff as a compatibility payload, not a standalone checkpoint-proof backend.

4. `docs/code-review/032-scenario-1-crypto-status.md` rejects claims that a recursive checkpoint-proof system is already finished.

**Reasoning:** The current path is not completely unchecked, because it still validates linkage and consistency. But it is not yet the authoritative recursive-proof boundary the stronger claim would require.

**Gap Or Blocker:** Missing final proof-transcript validation.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer says “accepts non-empty bytes plus consistency checks,” not “blindly accepts arbitrary garbage.”

### 26. Synthetic Proof Evidence

🔴 **Quest:** What evidence shows that current checkpoint artifacts accept non-empty `cp_proof` and that storage persists synthetic proof bytes?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The evidence is direct: constructors accept non-empty `cp_proof`, simulator stage 6 persists compatibility proof bytes, and tests encode synthetic byte payloads successfully.

**Evidence Trail:**

1. `crates/z00z_storage/src/checkpoint/artifact_final.rs` and `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs` only require non-empty `cp_proof`.

2. `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` states that current stack persists tx proof bytes as a compatibility payload.

3. `crates/z00z_storage/src/checkpoint/artifact_tests.rs`, `crates/z00z_storage/src/checkpoint/store_tests.rs`, and `crates/z00z_storage/src/checkpoint/ids.rs` contain accepted fixtures with synthetic non-empty proof bytes.

4. `crates/z00z_storage/src/checkpoint/store.rs` persists the raw artifact lane separately from any future stronger proof boundary.

**Reasoning:** This is enough to show that non-empty proof-byte acceptance and persistence are live repository behavior, not just a planning suspicion.

**Gap Or Blocker:** The remaining gap is runtime authoritative proof verification, not evidence of the current synthetic-byte lane.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: this proves acceptance and persistence semantics, not a completed exploit path.

### 27. Stage-6 Placeholder Comments

🔴 **Quest:** Do stage-6 comments explicitly mark checkpoint aggregation and digest construction as placeholders?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Yes. Stage-6 comments explicitly mark the current checkpoint proof handoff as compatibility-shaped and also label parts of the current demo flow as placeholders.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` says the current stack persists tx proof bytes as a compatibility payload and is not a standalone checkpoint-proof backend.

2. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` includes a direct “Placeholder only for the current demo flow” comment.

3. The same bundle-lane file also labels the live verifier as a package-coupled current-stack verifier rather than a stronger final backend.

**Reasoning:** The repository therefore already documents the incompleteness of this boundary in its own stage-6 comments.

**Gap Or Blocker:** None for the question itself.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer uses the exact compatibility and placeholder labels the code already contains.

### 28. Bob-Secret Checkpoint Enforcement

🔴 **Quest:** Does the authoritative checkpoint boundary enforce a real Bob-secret-gated spend proof today?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** No. Bob-secret-gated ownership lives in wallet and spend-rule seams today; the authoritative checkpoint boundary does not yet enforce a standalone Bob-secret-gated spend proof.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/stealth/output.rs`, `crates/z00z_wallets/src/core/tx/witness_gate.rs`, and `crates/z00z_wallets/src/core/tx/spend_rules.rs` show that Bob-secret dependence is modeled before checkpoint publication.

2. `crates/z00z_storage/src/checkpoint/artifact_final.rs` and `crates/z00z_storage/src/checkpoint/store.rs` do not expose a Bob-secret verifier surface at the storage boundary.

3. `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` and `bundle_lane_impl.rs` keep the checkpoint boundary package-coupled and compatibility-shaped, not authoritative in the stronger Bob-secret sense.

**Reasoning:** The real answer is therefore negative: Bob-secret gating is real locally, but not yet elevated into the authoritative checkpoint verifier boundary.

**Gap Or Blocker:** Missing authoritative checkpoint proof backend.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer rejects authoritative checkpoint enforcement without denying the real local Bob-secret gate.

### 29. Finished Authoritative Verifier

🔴 **Quest:** Is spend or checkpoint authorization already backed by a finished authoritative proof verifier?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** No. The repository has real current-stack verification seams, but it does not yet expose a finished authoritative proof verifier for spend or checkpoint authorization.

**Evidence Trail:**

1. `docs/code-review/032-scenario-1-crypto-status.md` rejects claims of a recursive checkpoint proof system, a universal trustless public spend verifier, and an authoritative whole-chain verifier deployment.

2. `crates/z00z_storage/src/checkpoint/artifact_final.rs` currently routes accepted proof systems through opaque and attest lanes rather than a finished verified proof backend.

3. `crates/z00z_wallets/src/core/tx/tx_wire_types.rs` still documents the current proof object as carrying local transaction-proof material only.

**Reasoning:** The repository is therefore not verifier-free, but it is not yet at the stronger “finished authoritative verifier” stage.

**Gap Or Blocker:** Final authoritative checkpoint and spend-proof closure.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer denies only the stronger final-verifier claim.

### 30. JMT Publish Trustlessness

🔴 **Quest:** Does JMT publish satisfy its trustlessness goal, or is the current privacy-oriented leaf path still blocked because checkpoint proof verification is not yet authoritative?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The current evidence supports the blocked reading: JMT publish has not yet satisfied a stronger trustlessness goal because checkpoint proof verification is still not authoritative.

**Evidence Trail:**

1. `docs/code-review/032-scenario-1-crypto-status.md` says no recursive checkpoint proof system and no authoritative whole-chain verifier deployment are claimed.

2. `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs` and `bundle_lane_impl.rs` describe the checkpoint proof lane as compatibility-shaped and package-coupled rather than authoritative.

**Reasoning:** That is enough to reject any strong “publish is already trustless” reading. The privacy-oriented leaf path remains narrower than the final trustless publish theorem.

**Gap Or Blocker:** I did not find a single canonical sentence in the repository that defines the publish goal in exactly those words; the conclusion is an evidence-backed inference from the missing authoritative verifier.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer states the blocked conclusion without pretending the repository already names the goal with the exact exam wording.

### 31. Full ZK Spend Claim

🔴 **Quest:** Does the spend path satisfy its full ZK claim, or only a narrower owner-tag, asset-id, balance, and range contract without a complete public proof verifier?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The spend path currently satisfies only a narrower owner-tag, asset-id, balance, and range contract. It does not yet satisfy the stronger full-ZK claim of a completed public trustless spend verifier.

**Evidence Trail:**

1. `crates/z00z_wallets/src/core/tx/spend_verification.rs` enforces real proof/auth presence, root and leaf-hash relations, stealth output relations, range checks, balance, and authorization signature.

2. `crates/z00z_wallets/src/core/tx/spend_rules.rs` makes the narrower ownership fields explicit through `owner_tag_in`, `leaf_ad_id_in`, balance, and range checks under `receiver_secret`-gated local rules.

3. `docs/code-review/032-scenario-1-crypto-status.md` says no universal trustless public spend verifier is claimed and that `PH32-SPEND` remains open.

**Reasoning:** The honest answer is therefore “narrower accepted spend contract,” not “full finished ZK spend theorem.”

**Gap Or Blocker:** The public verifier boundary remains incomplete.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer keeps the live spend contract real while rejecting the stronger final claim.

## 🧬 Theme 4: Claim And Checkpoint Structural Findings

### 32. Public-Data Claim Forgery

🔴 **Quest:** Does the current accepted genesis-claim flow treat public-statement-only claim proof or authority-signature material as sufficient for acceptance?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** No. The current accepted genesis-claim flow does not treat the legacy public-statement-only placeholder proof or authority-signature material as sufficient for acceptance.

**Evidence Trail:**

1. `crates/z00z_crypto/src/claim/prover.rs`, `verifier.rs`, and `proof.rs` still contain legacy placeholder-style helpers derived from public statement material.

2. But the accepted Scenario 1 claim package path in `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` builds a claim-source-v2 proof, checks root agreement, and signs the rebuilt statement through the accepted authority flow.

3. The accepted consumer and verifier path in `crates/z00z_wallets/src/core/tx/claim_auth.rs` and `crates/z00z_simulator/src/claim_pkg_consumer.rs` uses the newer claim-v2 surface.

**Reasoning:** The placeholder helpers remain in the tree, but they do not prove that the accepted current claim path still accepts them.

**Gap Or Blocker:** Legacy placeholder code still exists and remains a maintenance risk.

**Verification:**

- `doublecheck` status: VERIFIED for the final negative answer; the original positive thesis is contradicted by the accepted claim-v2 path.
- Required narrowing applied: the live answer stays negative even though legacy placeholder helpers still exist elsewhere in the tree.

### 33. Genesis Membership Continuity

🔴 **Quest:** Does the current genesis-claim path prove persisted authenticated genesis membership continuity, or does it still stop at a helper-owned source-root continuity contract?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** It still stops at a helper-owned source-root continuity contract rather than proving persisted authenticated genesis membership continuity.

**Evidence Trail:**

1. `crates/z00z_simulator/src/claim_pkg_consumer.rs` compares the package proof/root against `AssetStore::claim_source_contract_for_item(...)`.

2. `crates/z00z_storage/src/assets/store_internal/store_query.rs` shows that helper building a synthetic off-backend store and deriving the root/proof from that temporary structure.

3. `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` mirrors that narrower contract by checking source-root agreement.

4. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` rejects all-zero roots, so the gap is not “ZERO_ROOT accepted in the final claim-v2 path.”

**Reasoning:** The accepted path is stronger than a pure placeholder root, but it still does not reach persisted authenticated genesis membership continuity.

**Gap Or Blocker:** Missing anchoring of the source root to authoritative genesis or chain state.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer rejects persisted continuity without incorrectly claiming that the accepted path still accepts literal zero roots.

### 34. Checkpoint Placeholder Boundary

🔴 **Quest:** Do checkpoint drafts still use `PassProof` and `NoSpent` placeholders instead of a real proof-verification and spent-set boundary?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The accepted live path has moved beyond literal `PassProof` and `NoSpent` placeholders, but the checkpoint boundary is still unfinished because proof verification remains package-coupled and the spent-set source remains narrower than an authoritative backend.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_11_apply.rs` now builds the draft with `CheckpointPackageProofVerifier` and `CheckpointReplaySpentIndex`.

2. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` says the live proof verifier is package-coupled, not a standalone checkpoint-proof backend.

3. The same bundle-lane file shows a narrow replay-oriented spent index rather than an authoritative external spent-set source.

4. Literal `PassProof` and `NoSpent` placeholders still remain in test scaffolding such as `crates/z00z_storage/tests/test_checkpoint_draft_build.rs` and `crates/z00z_wallets/tests/test_tx_spent_gate.rs`.

**Reasoning:** The strongest accurate answer is therefore “still unfinished checkpoint boundary,” not “accepted path still literally uses the old placeholder structs.”

**Gap Or Blocker:** Missing authoritative proof and spent-set backends.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer distinguishes accepted live path from placeholder test scaffolds.

## 🔐 Theme 5: Secret Handling And Simulator Hygiene

### 35. Plaintext Secret Exposure

🔴 **Quest:** Do Scenario 1 artifacts store passwords, seed phrases, or receiver secrets in plaintext, including debug exports and long-lived fields?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Yes. Scenario 1 still stores highly sensitive material in plaintext in debug exports and long-lived fields, even though the default public lane has already been narrowed.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` stores `password`, `seed_phrase`, and `receiver_secret_hex` in `ActorRun`.

2. The same file writes those values into a private markdown artifact when debug secret dumping is enabled.

3. `crates/z00z_simulator/src/scenario_1/stage_2.rs`, `config_accessors.rs`, and `Cargo.toml` show that this lane is feature-gated rather than part of the default public path.

4. `crates/z00z_simulator/src/config.rs`, `scenario_config.yaml`, and `stage_2_utils/actors.rs` also keep plaintext password material in config/default surfaces.

**Reasoning:** The default path is better than the older behavior, but the thesis remains valid for debug/export lanes and long-lived in-memory fields.

**Gap Or Blocker:** Secret lifecycle hardening is still incomplete.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer does not claim that the default public path still emits the plaintext artifact.

### 36. `SeqSecureRngProvider` Safety

🔴 **Quest:** Is `SeqSecureRngProvider` cryptographically defensible for key or unlinkability-sensitive material, even if it is labeled simulator-only?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** No. `SeqSecureRngProvider` is not cryptographically defensible for key or unlinkability-sensitive material, even if it is labeled simulator-only.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` derives seeds through counter and XOR mixing and then feeds them into `StdRng::seed_from_u64(...)`.

2. The same file enters that deterministic branch under `use_mock_rng`, with a zero-seed fallback if the seed is absent.

3. The code itself scopes the provider as simulator-only rather than as a production-strength cryptographic RNG boundary.

4. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` reaches the same audit conclusion.

**Reasoning:** Simulator-only scope explains why the code exists, but it does not make the construction cryptographically defensible for sensitive material.

**Gap Or Blocker:** The stack still needs consistent use of the stronger existing deterministic-vs-system RNG abstractions.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer critiques cryptographic suitability, not usefulness for deterministic test fixtures.

### 37. Password And Env Hazards

🔴 **Quest:** Do weak actor passwords and process-global `set_var` mutation each create real simulator security or integrity hazards?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Yes. Both weak actor passwords and process-global `set_var` mutation create real simulator security or integrity hazards.

**Evidence Trail:**

1. `crates/z00z_simulator/src/scenario_1/stage_2_utils/actors.rs` and `scenario_config.yaml` ship fixed human-readable actor passwords.

2. `crates/z00z_simulator/src/scenario_1/stage_2.rs` mutates process-global wallet chain and network configuration through `std::env::set_var(...)`.

3. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` identifies both patterns as real simulator security and integrity hazards.

**Reasoning:** Weak credentials increase disclosure and misuse risk, while global environment mutation creates shared-process and concurrency hazards.

**Gap Or Blocker:** The strongest effect of env mutation appears under concurrent or shared-process execution, but the global side effect is already real in code.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer does not overstate the hazard beyond simulator/integrity scope.

### 38. README Placeholder Disclaimer

🔴 **Quest:** Does the simulator README prominently disclaim that claim proof and authority semantics are placeholders, or could readers still overread it as a trustworthy reference implementation?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The simulator README does not prominently disclaim the placeholder claim-proof and authority semantics, so readers could still overread it as a more trustworthy reference than the repository actually warrants.

**Evidence Trail:**

1. The live README surface is `crates/z00z_simulator/README.md`.

2. Workspace search did not find README-level placeholder disclaimers for claim proof, claim authority, or zero-root-style trust-boundary limitations.

3. Those caveats instead live in code comments and status notes such as `exec_input_builder.rs` and `docs/code-review/032-scenario-1-crypto-status.md`.

**Reasoning:** The README does document simulator boundaries and secret-artifact policy, but it still lacks a prominent warning about the placeholder crypto and authority semantics most likely to be overread.

**Gap Or Blocker:** Documentation surface is incomplete.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer does not claim the simulator is presented as production code, only that the README caveat is not prominent enough.

### 39. Properties Worth Preserving

🔴 **Quest:** Which positive properties of the current stack are worth preserving during remediation even though they do not close the structural gaps?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Several positive properties are worth preserving during remediation: explicit domain separation, chain-bound nullifier derivation, proof-before-ownership sequencing, and the real narrower spend and claim validation seams already in the accepted stack.

**Evidence Trail:**

1. `crates/z00z_crypto/src/claim/v2.rs` uses domain-labeled claim-statement hashing.

2. `crates/z00z_wallets/src/core/claim/nullifier.rs` binds nullifier derivation to explicit domain labels and chain context.

3. `crates/z00z_simulator/src/scenario_1/stage_11_utils/jmt_wallet_scan.rs` keeps proof checks ahead of ownership interpretation in the JMT scan path.

4. `crates/z00z_wallets/src/core/tx/claim_tx.rs` preserves careful verifier ordering, and `stage_3.rs` preserves explicit checkpoint and resume discipline.

**Reasoning:** These properties do not close the unresolved structural gaps, but they materially improve correctness and should survive any remediation pass.

**Gap Or Blocker:** None for the preservation question itself.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer treats these as preservation targets, not as proof that the broader system is already complete.

### 40. Unresolved Blocking Questions

🔴 **Quest:** Which unresolved questions still block stronger positive conclusions about authority trust anchor, simulator-only scope, spent-set source of truth, range-proof limits, and debug-artifact policy?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** Stronger positive conclusions are still blocked by unresolved questions about the real authority trust anchor, simulator-only confinement of placeholder paths, authoritative spent-set source of truth, the scope of range-proof assurance outside the current simulator path, and final policy for debug secret artifacts.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` lists these exact blockers in its unresolved-question set.

2. `crates/z00z_wallets/src/core/tx/claim_auth.rs` still uses a simulator-style authority anchor rather than a chain-anchored live trust root.

3. `docs/code-review/032-scenario-1-crypto-status.md` still rejects stronger recursive-proof, authoritative verifier, and full trust-closure claims.

4. `Cargo.toml` plus `config_accessors.rs` show that debug secret dumping is gated, but the repository does not yet show a full retention and cleanup policy for such artifacts.

**Reasoning:** These unresolved questions are exactly why the repository still supports narrower claims than the strongest intended story.

**Gap Or Blocker:** They are themselves the blockers.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: the answer identifies open blockers rather than speculating that they have already been resolved.

## 🔧 Theme 6: Concrete Remediation Sets

### 41. Authoritative Checkpoint Fix Set

🔴 **Quest:** What exact remediation is required to make checkpoint proof verification authoritative at the storage boundary?
🔵 **Ans:**

**Status:** Full Evidence

**Conclusion:** The required remediation is to replace compatibility proof bytes with a canonical checkpoint-proof transcript, verify that transcript during finalize and load, and reject every non-canonical or non-verifiable proof at the storage boundary.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` calls for replacing synthetic proof bytes, verifying during finalize and load, and rejecting non-canonical transcripts.

2. `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs` and `artifact_final.rs` currently stop at non-empty-byte plus consistency checks.

3. `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` and stage-6 code show that the live lane is still compatibility-shaped rather than authoritative.

**Required Fix Set:**

1. Replace current compatibility `cp_proof` payloads with a canonical proof transcript format.

2. Add authoritative proof verification at both finalize-time and load-time boundaries.

3. Fail closed on missing, malformed, or non-canonical transcripts.

4. Remove any final-boundary interpretation that treats opaque compatibility bytes as sufficient.

**Reasoning:** Without those changes, the storage boundary still proves linkage and consistency but not authoritative checkpoint proof validity.

**Gap Or Blocker:** None beyond implementation of the fix set itself.

**Verification:**

- `doublecheck` status: VERIFIED
- Required narrowing applied: this is the exact remediation for authority at the storage boundary, not a broader architecture rewrite.

### 42. Receiver Identity Binding Fix Set

🔴 **Quest:** What exact remediation is required to harden receiver identity binding so the sender cannot be tricked into encrypting to Mallory under Alice's directory entry?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The required remediation is to make receiver identity binding fail closed through signed or pinned card identity, mandatory request and card validation, and explicit rotation handling before encryption proceeds.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` calls for TOFU or signed-directory binding, mandatory request/card validation, and frozen rotation handling.

2. `crates/z00z_wallets/src/core/address/stealth_trust.rs`, `stealth_request.rs`, and `receiver_card_record.rs` show that much of this machinery already exists in the repository.

**Required Fix Set:**

1. Require a signed directory record, TOFU pin, or equivalent trusted receiver-card binding before card-only encryption proceeds.

2. Make request and card validation mandatory on every send path that claims identity safety.

3. Add explicit rotation and epoch mismatch handling that fails closed until the sender confirms the new card.

4. Keep encryption blocked when Alice's expected identity and the presented receiver card diverge.

**Reasoning:** The repo already contains major pieces of this fix, but the send-path policy must be made uniformly mandatory and fail closed.

**Gap Or Blocker:** I did not find proof that every pure card send already enforces the stricter binding rule repo-wide.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer distinguishes existing building blocks from the still-required repo-wide policy closure.

### 43. Request-Bound Tag Fix Set

🔴 **Quest:** What exact remediation is required to make request-bound tag derivation the normal privacy path and materially reduce targeted scan spam?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The required remediation is to promote request-bound derivation from an available option into the normal privacy path, keep `tag16` tied to request identity when requests exist, and make downgrade to card-only mode explicit rather than silent.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` calls for request-bound tag derivation as the normal path.

2. `crates/z00z_wallets/src/core/stealth/output_build.rs` already switches to `RequestBound` mode, derives `tag16` from `req_id`, and uses `derive_k_dh_with_req(...)` when a payment request is present.

3. Repository tests already exercise request-bound divergence semantics in the wallet test suite.

**Required Fix Set:**

1. Make request-bound mode the preferred or default privacy path whenever a valid request exists.

2. Preserve `tag16 <- req_id` and request-bound `k_dh` derivation as the canonical tagged-send behavior.

3. Make card-only fallback an explicit downgraded mode with visible semantics rather than an unmarked alternative.

4. Keep or extend request-bound tests so targeted-scan reduction remains locked in.

**Reasoning:** The core machinery already exists; the remaining work is product and policy closure around the default path.

**Gap Or Blocker:** The repository does not yet prove that request-bound mode is the normal path for all ordinary sends.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer avoids claiming that the fix set starts from zero.

### 44. Real Claim Authority Fix Set

🔴 **Quest:** What exact fix set is required to replace placeholder claim authorization with a real authority model?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The required fix set is to replace simulator-fixed trust anchors with a real chain-anchored authority registry and lifecycle, while preserving the existing canonical statement-signing surface.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` calls for replacing placeholder-style claim authority with a real authority model.

2. `crates/z00z_wallets/src/core/tx/claim_auth.rs` still uses a simulator-style chain anchor and fixed authority key material.

3. The same file and `claim_tx_verifier_impl_proof.rs` already use real `ClaimAuthoritySigV2` signing and verification over canonical `ClaimStmtV2`.

**Required Fix Set:**

1. Replace hardcoded simulator authority anchors with a chain-anchored authority root or registry.

2. Support authority key rotation, revocation, and versioned trust lifecycle.

3. Keep signing bound to canonical `ClaimStmtV2` rather than ad hoc payloads.

4. Verify authority signatures against the anchored live authority set, not a fixed embedded simulator key.

**Reasoning:** The missing part is not the signature primitive; it is the trust anchor and authority lifecycle.

**Gap Or Blocker:** None beyond implementing the authority-root and lifecycle model.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer preserves what is already real and focuses remediation on the unresolved trust-anchor layer.

### 45. Genesis Membership Fix Set

🔴 **Quest:** What exact fix set is required to authenticate genesis membership instead of relying on `ZERO_ROOT` statement binding?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The required fix set is to anchor claim membership to real persisted genesis or chain state, verify inclusion against that anchored root, and preserve the already-correct field binding in the canonical claim statement.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` identifies the need to replace placeholder-style genesis binding with authenticated membership.

2. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` already rejects all-zero roots in the accepted verifier path.

3. `crates/z00z_crypto/src/claim/v2.rs` and `claim_helpers.rs` already bind `chain_id`, asset identity, commitment, source root, and `claim_scope_hash` into the canonical statement.

4. `crates/z00z_storage/src/assets/store_internal/store_query.rs` shows that the current continuity still stops at a helper-owned source-root contract.

**Required Fix Set:**

1. Replace helper-owned synthetic source-root continuity with authenticated genesis or chain-state inclusion proofs.

2. Verify those proofs against an authoritative anchored root.

3. Preserve the existing canonical statement field binding rather than redesigning it.

4. Remove any remaining dependency on placeholder or synthetic continuity shortcuts in accepted claim paths.

**Reasoning:** The repository already binds the right fields, but not yet to an authoritative persisted genesis membership source.

**Gap Or Blocker:** Missing anchored membership proof source.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer does not falsely say the accepted path still depends on literal zero-root acceptance.

### 46. Checkpoint Integrity Fix Set

🔴 **Quest:** What exact fix set is required to remove checkpoint integrity placeholders and fail closed when proof or spent validation is unavailable?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The required fix set is to replace package-coupled and placeholder-style integrity lanes with authoritative proof and spent validation backends, and to fail closed whenever either backend is unavailable.

**Evidence Trail:**

1. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` explicitly calls for removing placeholder proof and spent validation.

2. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` still exposes a package-coupled proof verifier and a narrow replay-oriented spent index.

3. Placeholder scaffolds still exist in checkpoint and spent-gate tests, even though the accepted live path has moved beyond literal `PassProof` and `NoSpent` objects.

**Required Fix Set:**

1. Replace the package-coupled proof verifier with an authoritative checkpoint-proof backend.

2. Replace the narrow replay spent index with an authoritative spent-set source of truth.

3. Fail closed whenever proof or spent validation backends are unavailable or inconclusive.

4. Remove or quarantine placeholder integrity scaffolds from any accepted runtime path.

**Reasoning:** The live path already fail-closes more than the original placeholder thesis suggested, but it still stops short of authoritative checkpoint integrity.

**Gap Or Blocker:** Missing authoritative proof and spent backends.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer targets the real remaining integrity gap rather than pretending the accepted path is unchanged since the oldest placeholder version.

### 47. Secret Lifecycle Fix Set

🔴 **Quest:** What exact fix set is required to harden the secret lifecycle and remove plaintext debug artifacts from the default path?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The required fix set is to preserve the already-hardened default path, confine plaintext debug artifacts to strictly gated private workflows, and move remaining secret handling to explicit protected wrappers with cleanup and retention policy.

**Evidence Trail:**

1. The default plaintext secret artifact is already removed from the normal lane through `wallet_debug_dump` feature gating in `Cargo.toml`, `config_accessors.rs`, `stage_2.rs`, and `test_stage2_secret_artifacts.rs`.

2. `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` still writes passwords, seed phrases, and key material when the debug lane is enabled.

3. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` asks for stronger wrappers, cleanup, and labeling.

**Required Fix Set:**

1. Keep plaintext secret artifacts out of the default path.

2. Restrict debug secret export to explicitly gated private workflows and document that boundary clearly.

3. Replace long-lived plaintext secret fields with protected wrappers such as `Hidden<T>`, `SafePassword`, or equivalent zeroizing containers.

4. Add explicit retention, cleanup, and handling policy for any remaining debug artifacts.

**Reasoning:** The repo already solved the biggest default-path issue. The remaining work is debug-lane confinement and lifecycle hardening.

**Gap Or Blocker:** Remaining long-lived plaintext fields and debug artifact policy.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer does not incorrectly describe the default path as still plaintext by default.

### 48. RNG, Credential, And Config Fix Set

🔴 **Quest:** What exact fix set is required to replace weak simulator randomness, credentials, and configuration handling with existing deterministic test RNG abstractions and explicit configuration objects?
🔵 **Ans:**

**Status:** Partial Evidence

**Conclusion:** The required fix set is to retire the ad hoc simulator RNG and global env mutation path in favor of the repository's existing explicit RNG-mode abstractions and explicit configuration objects, while also eliminating fixed human-readable actor credentials.

**Evidence Trail:**

1. `crates/z00z_simulator/src/rng_mode.rs` and `config.rs` already provide explicit seeded-versus-system RNG abstractions.

2. `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` still uses the older `SeqSecureRngProvider` path.

3. `crates/z00z_simulator/src/scenario_1/stage_2_utils/actors.rs` still ships fixed human-readable passwords.

4. `crates/z00z_simulator/src/scenario_1/stage_2.rs` still mutates process-global configuration with `std::env::set_var(...)`.

5. `.planning/phases/033-crypto-audit-scenario-2/033-32EXAM-QEST-DRAFT.md` calls for this exact cleanup direction.

**Required Fix Set:**

1. Replace `SeqSecureRngProvider` on accepted simulator-sensitive paths with the existing `rng_mode.rs` deterministic-test and system-RNG abstractions.

2. Keep deterministic seeded RNG confined to explicit test or replay modes.

3. Remove hardcoded human-readable actor passwords from defaults and inject credentials through explicit config or protected secret inputs.

4. Replace global `set_var(...)` mutation with explicit configuration objects passed through runtime seams.

**Reasoning:** This is a completion pass, not a greenfield design: most replacement primitives already exist in the repository.

**Gap Or Blocker:** The new abstractions are not yet applied consistently across the Scenario 1 path.

**Verification:**

- `doublecheck` status: PARTIALLY VERIFIED
- Required narrowing applied: the answer treats the fix set as completion and consolidation of existing abstractions, not as invention of a brand-new design.

## 📊 Summary Table

### Overview By Theme

| Theme | Q Range | Full Evidence | Partial Evidence | Final Validation Read |
| --- | --- | ---: | ---: | --- |
| Core Draft Claims | 1-11 | 7 | 4 | Real receive, range-proof, spend, and anti-theft seams exist, but the stronger original end-to-end claim remains narrower than the live proof boundary. |
| Trust Model, Drift, And Ownership Logic | 12-24 | 12 | 1 | The repository supports the intended ownership model and semantic freeze direction, but stronger validator and chain-level closure is still incomplete. |
| Checkpoint Boundary And Publish Verdicts | 25-31 | 5 | 2 | Checkpoint and publish integrity checks are real, but authoritative proof-backend closure is still missing. |
| Claim And Checkpoint Structural Findings | 32-34 | 1 | 2 | The accepted claim path is stronger than the oldest placeholder story, but authoritative genesis-membership and checkpoint-boundary closure remain incomplete. |
| Secret Handling And Simulator Hygiene | 35-40 | 6 | 0 | Sensitive simulator hazards are real, scoped, and repository-backed; preserved positives and unresolved blockers are now stated explicitly. |
| Concrete Remediation Sets | 41-48 | 1 | 7 | The remediation direction is clear and repository-backed, but most fixes are completion or closure steps over partially landed building blocks rather than greenfield changes. |
| **Total** | **1-48** | **32** | **16** | **The exam now consistently supports a narrow, validated current-stack story and rejects broader claims that are still not closed by the live repository.** |

### Partial-Evidence Questions Requiring Caution

| Q | Topic | Why It Stays Partial | Safe Final Reading |
| --- | --- | --- | --- |
| 2 | Real stealth and range proofs | Stronger original claim overreaches the accepted verifier boundary | Real canonical receive and Bulletproofs+ seams exist, but they do not close the full trustless theorem. |
| 5 | Alice and the asset secret | Terminology drift around what “asset secret” means | Alice knows `s_out`; the unresolved split is whether a report means `s_out` or Bob's separate `receiver_secret`. |
| 8 | Authoritative publish proofs | Integrity checks are real, but publish-proof backend is still compatibility-shaped | Package-coupled checkpoint integrity exists; authoritative publish-proof closure does not. |
| 9 | Real theft-resistance boundary | Local anti-theft seam is real, but final public-proof closure is not | The honest current story is receiver-secret gating at spend time, not sender ignorance of `s_out`. |
| 12 | Incomplete validator trust model | Current validator checks are real but not fully authoritative | The missing piece is final cryptographic closure, not total absence of validator-facing verification. |
| 30 | JMT publish trustlessness | The repo supports the blocked reading but does not define the goal in exactly the exam's words | Publish is not yet strong enough to be called fully trustless. |
| 31 | Full ZK spend claim | Public spend verifier remains narrower than the strongest claim | The live contract is real, but it is still narrower than a finished full-ZK spend theorem. |
| 33 | Genesis membership continuity | Accepted path rejects zero-root shortcuts but still stops at helper-owned continuity | Source-root continuity is stronger than placeholder roots, but not yet authoritative persisted genesis membership. |
| 34 | Checkpoint placeholder boundary | Live path moved beyond literal placeholders, but the boundary is still incomplete | The right finding is “unfinished boundary,” not “unchanged old placeholder runtime.” |
| 42 | Receiver identity binding fix set | Important building blocks already exist, but repo-wide fail-closed policy is not proven | Finish the policy layer that makes identity binding uniformly mandatory. |
| 43 | Request-bound tag fix set | Request-bound machinery exists, but default-path closure is not proven | Promote request-bound mode from available option to normal privacy path. |
| 44 | Real claim authority fix set | Signature primitive exists; trust anchor does not | Replace simulator-fixed authority roots with live anchored authority lifecycle. |
| 45 | Genesis membership fix set | Field binding is already stronger than the thesis suggests | Preserve current statement binding and replace helper continuity with authoritative membership proofs. |
| 46 | Checkpoint integrity fix set | Live path already fail-closes more than the old placeholder thesis | Finish authoritative proof and spent backends instead of describing the runtime as fully placeholder-based. |
| 47 | Secret lifecycle fix set | Default plaintext path is already removed, but debug lane remains sensitive | Keep default lane hardened and finish debug-lane confinement, wrapping, and retention policy. |
| 48 | RNG, credential, and config fix set | Replacement abstractions already exist but are not applied consistently | This is a consolidation pass over live abstractions, not a brand-new design. |
