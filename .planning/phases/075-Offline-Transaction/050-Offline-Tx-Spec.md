# 050 Offline Tx Spec

Status: authoritative current-state specification

## 🎯 Purpose And Authority

This document is the authoritative Phase 050 specification for offline-capable transaction exchange in the current Z00Z repository. Its job is to explain what the codebase actually supports today, what guarantees that implementation really gives, and which older Phase 050 planning ideas are no longer current truth.

This spec supersedes the two earlier Phase 050 planning notes that originally described offline payments and offline transaction packages. Those notes were useful as design history, but they are no longer the source of truth for current behavior.

The rule for conflicts is simple: live code wins over historical planning text. This document therefore does not try to preserve every old design idea. It preserves only the parts that are still visible in the wallet, transaction, and RPC code paths.

This document is also intentionally practical. A junior developer should be able to read it and answer four questions without reopening the retired planning docs:

1. what an offline-capable package is in the current system,
2. which artifacts are authoritative,
3. what local verification proves,
4. how the live implementation is structured.

This spec covers the live delayed-connectivity package flow implemented in the wallet, RPC, and simulator seams.

Privacy compliance rule: this document inherits the live receiver-routing and package-exposure privacy boundary enforced by the current wallet and RPC contracts. Offline portability must not be described as if it upgrades request-bound receive, package secrecy, or transport anonymity into solved protocol guarantees.

## 🔑 Implemented Authority Surface

The current offline-capable flow is not spread across abstract protocol papers. It is anchored in a small set of concrete, already-implemented Rust contracts. This section names those contracts because every later requirement in the spec depends on them.

The authoritative current-state surface is:

- `ReceiverCard` as the signed receiver-routing artifact.
- `ReceiverCardRecord` as the canonical published receiver-card record.
- Compact raw-card transport through `encode_card_compact` and `decode_card_compact`.
- Compact published-record transport through `ReceiverCardRecord::to_compact` and `ReceiverCardRecord::from_compact`.
- `TxPackage` and `TxWire` as the live portable transaction package format.
- `build_tx_package_digest` as the canonical package digest function.
- `TxVerifierImpl` as the package-local structural verifier.
- `build_public_spend_contract` and `verify_tx_public_spend_contract` as the public spend-authorization path.
- `build_tx_stealth_output`, `build_tx_stealth_output_serial`, `build_card_stealth_output_validated`, and `build_tx_stealth_output_validated` as the wallet-owned sender-output constructors.
- `build_output_bundle`, `build_output_bundle_with_rng`, `bind_output_wire`, `decode_output_pack`, and `verify_self_decrypt` as the current sender-side confidential output flow.
- `verify_transaction_package_impl` as the runtime entry point that verifies a package and scans for owned outputs.
- `is_import_ready(status)` as the current import-readiness gate.
- lifecycle helpers that project pending rows into confirmed rows.

When this spec says that something is part of the offline transaction model, it means that it is enforced or consumed by one of those live contracts. If a concept is not wired into that authority surface, it is not current-state protocol truth.

## ⚙️ Scope

The word "offline" is easy to overread, so the scope needs to be explicit. In the current Z00Z codebase, offline-capable transaction handling means delayed-connectivity package exchange built around package preparation, local verification, owned-output discovery, and status-gated import readiness.

More specifically, the implemented flow allows:

1. a sender to prepare a transaction package without immediate chain submission,
2. a recipient or runtime to verify package-local correctness while disconnected from final chain state,
3. local discovery of recipient-owned outputs when the package is structurally and cryptographically valid at the package level,
4. a runtime decision about whether the package is inspectable, verified, or import-ready.

## 📦 Current Transaction Package Model

The transaction package model is the center of the current offline-capable flow. If a developer understands `TxPackage` and `TxWire`, they understand what is actually exchanged between parties and what the runtime can verify locally.

### ✅ Package Contract

`TxPackage` is the canonical portable envelope. It is the object serialized, transported, verified, and later considered for import-readiness decisions.

The system treats the following fields as the canonical digest-bound package material:

- `kind`
- `package_type`
- `version`
- `chain_id`
- `chain_type`
- `chain_name`
- `tx`

That means `tx_digest_hex` is not an arbitrary metadata field. It is only valid when it matches the digest recomputed from exactly that digest-bound material.

The system SHALL therefore reject a package when:

- `tx_digest_hex` is malformed,
- `tx_digest_hex` is not valid hex,
- `tx_digest_hex` does not match the recomputed digest.

The system also requires `status` to be present and non-empty. `status` is not part of the digest-bound envelope, but it is part of the runtime import decision surface.

Transport and privacy rule: `TxPackage` is portable, but it is still sensitive material. Local verification of package bytes still requires the same care as any other confidential transport artifact in logs, exports, forwarding channels, or helper services.

### ✅ Transaction Wire Contract

`TxWire` is the live transaction payload carried inside `TxPackage`. It contains the transaction subtype marker, reference-only inputs, portable outputs, declared fee metadata, sender sequencing nonce, auxiliary context, and the top-level proof and authorization containers used by the public spend path.

Those top-level containers are always present in the live wire contract. What is optional is the nested spend-specific material inside them, exposed as `proof.spend` and `auth.spend`.

The live `TxWire` fields are:

- `tx_type`: transaction subtype marker; the current regular path uses `regular_tx`.
- `inputs`: reference-only inputs.
- `outputs`: `TxOutputWire` values.
- `fee`: declared fee metadata.
- `nonce`: sender sequencing nonce.
- `context`: additional tx context.
- `proof`: public proof object.
- `auth`: public authorization object.

`TxWire.inputs` are not full embedded pre-state leaves. Each `TxInputWire` contains:

- `asset_id_hex`: the canonical consumed state key encoded as 32-byte hex,
- `serial_id`: a leaf-match consistency field that must agree with the resolved pre-state leaf.

This distinction matters. The live code treats `TxInputWire` as a reference into pre-state, not as a standalone membership proof. Global membership remains in the checkpoint or pre-state resolution path.

`TxWire.outputs` are `TxOutputWire` values, and each output carries one semantic role:

- `recipient`
- `change`
- `fee`

The declared `tx.fee` is metadata that must equal the sum of fee-role outputs. The current design keeps economic value conservation in the output set itself rather than creating a second detached fee commitment object.

The live structural verifier also requires the package and tx identity markers to match the regular transaction path exactly:

- package kind must be `TxPackage`,
- package subtype must be `regular_tx`,
- package version must be non-zero,
- `chain_id` must be non-zero,
- `chain_type` must be non-empty,
- `chain_name` must be non-empty,
- tx payload type must be `regular_tx`.

### ✅ Public Proof And Authorization Containers

The regular transaction wire also exposes optional public spend material through `TxProofWire` and `TxAuthWire`. These containers already have concrete live meaning in the current codebase when the public spend path is used.

`TxProofWire` may carry a `SpendProofWire`, and `TxAuthWire` may carry a `SpendAuthWire`.

`SpendProofWire` currently binds:

- `ver`
- `proof_suite`
- `prev_root_hex`
- `statement_hex`
- `proof_hex`
- `inputs`

Each `SpendInputProofWire` carries public per-input proof material such as:

- `input_asset_id_hex`
- `serial_id`
- `nullifier_hex`
- `r_pub_hex`
- `owner_tag_hex`
- `commitment_hex`
- `leaf_ad_id_hex`
- `leaf_ad_hash_hex`

`SpendAuthWire` currently binds:

- `ver`
- `receiver_card_compact`
- `spend_sig_hex`

This means the public spend path is already a real contract surface in the current repository.

## 👁️‍🗨️ Receiver Routing Model

Offline-capable exchange begins with receiver routing. In the live codebase, the sender does not invent recipient stealth material directly. The sender consumes a signed receiver artifact and derives outputs from that trusted routing surface.

### ✅ ReceiverCard

`ReceiverCard` is the signed receiver-routing artifact used by the sender to derive confidential outputs for the recipient. In practical terms, it is the object that tells the sender, "these are the receiver-side public routing values you may trust for output construction."

The system SHALL enforce all of the following on a receiver card:

1. the card version must be supported,
2. the canonical byte structure must be valid,
3. encoded ECC points must decode correctly,
4. the signature over canonical unsigned bytes must verify,
5. the card must fail closed if expired.

That combination gives the sender a verified routing surface. Final spend authority, chain membership, and request-policy approval remain available to later validation surfaces.

### ✅ ReceiverCardRecord

`ReceiverCardRecord` is the canonical published receiver-card record in the live code. This matters because the repository distinguishes between a raw compact card and a publication record that adds publication semantics such as registry entry binding, epoch, and revocation status.

ReceiverCardRecord remains the canonical published receiver trust contract.

The system verifies `ReceiverCardRecord` by checking:

- supported record version,
- successful embedded `ReceiverCard` decode,
- successful embedded `ReceiverCard` verification,
- correct `registry_entry_id` binding,
- non-revoked state,
- non-stale epoch when an epoch floor is supplied.

The system SHALL also treat these as distinct wire contracts:

- raw compact `ReceiverCard` transport,
- compact `ReceiverCardRecord` publication transport.

That distinction is important for junior readers: a card is the routing artifact; a record is the published registry entry that wraps and governs that artifact.

### ⚠️ Privacy Priority For Offline Routing

The offline-capable routing surface has two distinct lanes in the current repository.

- Request-bound receive is the privacy-first lane whenever a validated `PaymentRequest` exists.
- Raw `ReceiverCard` or `ReceiverCardRecord` transport remains an implemented compatibility route for delayed-connectivity exchange.

Rules:

- request-bound receive is the privacy-first lane whenever a validated `PaymentRequest` exists;
- raw `ReceiverCard` or `ReceiverCardRecord` transport is the implemented compatibility route for delayed-connectivity exchange;
- request, card, and record material stay in the routing or admission layer, separate from public `TxPackage` proof roots.

## 💥 Output Construction Model

Once a verified routing artifact exists, the sender-side output flow builds the confidential leaves that will later be transported inside the transaction package. The code already contains a complete local construction pipeline for that step.

The implemented flow derives a Diffie-Hellman shared key, derives output secrets, binds output wire data into a confidential leaf, and supports sender-side local self-verification before the package is finalized.

The live sender-side surface is split between wallet-owned constructors and compatibility/output-flow helpers:

- `build_tx_stealth_output`
- `build_tx_stealth_output_serial`
- `build_card_stealth_output_validated`
- `build_tx_stealth_output_validated`
- `build_output_bundle`
- `build_output_bundle_with_rng`
- `bind_output_wire`
- `decode_output_pack`
- `verify_self_decrypt`

This means the output pipeline is not just serialization. It is a local cryptographic construction and consistency check pipeline.

### ✅ What The Builder Produces

The current builder produces an `OutputBundle`, which combines:

- semantic receiver and role labels,
- asset class and plaintext value,
- the built confidential `AssetLeaf`,
- the derived shared key material,
- the derived output secret.

The leaf is therefore not an opaque blob. It is a structured result that the sender can sanity-check before packaging it for the receiver.

### ✅ Local Self-Verification Semantics

When the local self-verification helper is used, the system checks that the newly built output is internally coherent before packaging.

The helper verifies that:

1. output self-decryption succeeds,
2. `tag16` recomputation matches the leaf,
3. the decrypted pack value matches the requested output value,
4. the decrypted output secret matches the derived output secret,
5. the commitment opening reproduces the stored commitment,
6. the stored range proof verifies successfully for the output.

This is one of the most important practical boundaries in the current design. Before a package is handed to a runtime verifier, the sender can already prove locally that the output leaf is self-consistent.

## ✅ Local Verification Guarantees

The local package verifier is powerful, but it is not omnipotent. This section separates what the live verifier actually proves from what still belongs to later chain or checkpoint logic.

### ✅ What The Live Verifier Enforces

`TxVerifierImpl` verifies package-local correctness. It rejects malformed or internally inconsistent packages before any claim is made about import readiness.

The system SHALL reject packages that violate any of the following:

- empty package payload,
- malformed package encoding,
- invalid package structure,
- empty input set,
- empty output set,
- duplicate consumed input state keys,
- duplicate produced output state keys,
- overlap between produced output state keys and consumed input keys,
- zero output nonce,
- duplicate output nonce,
- invalid amount semantics by asset class,
- invalid owner signatures when present,
- invalid range proofs,
- fee outputs present when declared fee is zero,
- missing fee outputs when declared fee is positive,
- mismatch between declared fee and fee outputs,
- mismatch between declared fee and calculated fee units,
- digest mismatch.

This is already a meaningful safety surface. A package that passes these checks is not arbitrary JSON; it is a structurally and cryptographically coherent transaction package under the current local rules.

### ✅ Verifier Scope

The local package verifier establishes package-local validity. The checkpoint or chain-facing state path handles global membership, chain-level uniqueness, finality, and ancestor resolution.

## 🔐 Public Spend Authorization Path

In addition to local package structure checks, the repository already contains a public spend-authorization path. This path is the live validator-facing or package-admission-style public contract, and it deserves separate explanation because it is stronger than raw structure validation but narrower than the full wallet-local secret theorem.

### ✅ Public Spend Contract Surface

When the public spend path is used, the canonical live public containers are:

- `SpendProofWire`
- `SpendAuthWire`

`build_public_spend_contract` produces the proof and auth containers, and `verify_tx_public_spend_contract` checks them.

The public spend verification path enforces:

- previous-root binding,
- input-to-proof positional pairing,
- input reference consistency,
- receiver-card decoding and verification,
- spend signature decoding and verification,
- input leaf associated-data consistency,
- output stealth field presence,
- output associated-data consistency,
- output range-proof presence and correctness where required,
- duplicate input rejection,
- duplicate output `leaf_ad_id` rejection,
- input/output leaf overlap rejection,
- balance-equation checks,
- authorization statement integrity.

### ✅ Public Spend Boundary

The current public spend verifier is a real contract surface that authenticates the accepted public statement, binds the receiver card, and carries replay-sensitive state into the later lifecycle path.

## 📡 Current Offline-Capable Receive Cycle

The receive cycle is where the word "offline" becomes operational. The live system already supports a delayed-connectivity exchange pattern in which a sender prepares a portable package, a recipient or runtime verifies it locally, and owned outputs are discovered before final import.

The implemented flow is:

1. the receiver publishes or shares a verified receiver card,
2. the sender builds confidential outputs against that receiver card,
3. the sender serializes a live `TxPackage`,
4. the recipient or another runtime calls `wallet.tx.verify_transaction_package`,
5. owned outputs are scanned and reconstructed only when package-local verification succeeds,
6. the runtime reports whether the package is import-ready,
7. only import-ready packages are candidates for actual asset import.

This is the practical current-state offline-capable path. It is already useful for delayed delivery and local pre-checking.

### ✅ Runtime Verification Response

The runtime verification response currently returns:

- `tx_digest_hex`
- `package_status`
- `is_valid`
- `import_ready`
- `all_owned_spendable`
- `owned_outputs`
- `errors`

The semantics are important:

- `owned_outputs` are only populated when local package verification succeeds,
- `all_owned_spendable` is true only when at least one owned output exists and every owned output contains sufficient spending material.

This means the runtime does not blindly scan malformed packages for wallet outputs. Local validity gates the ownership-discovery path.

### ✅ Import Readiness Gate

Import readiness is currently status-driven, but it is not status-only. A package must first be locally valid.

The system SHALL mark a package as import-ready only when both conditions hold:

1. local package verification succeeds,
2. `status` is accepted by `is_import_ready(status)`.

The currently accepted import-ready statuses are matched case-insensitively and normalize to:

- `confirmed`
- `verified`

By contrast, a locally valid `prepared` package is inspectable but not import-ready.

For junior readers, the practical rule is: import-ready requires both local validity and an accepted status. Status still matters.

## ♨️ Lifecycle Projection

The codebase also contains an explicit lifecycle projection helper that turns pending transaction rows into confirmed rows. This is relevant because offline-capable packages are not only transported and verified; they also participate in the user-facing lifecycle model.

### ✅ Pending And Confirmed States

The implemented pending states are:

- `pending_spend`
- `pending_receive`
- `pending_change`
- `pending_fee`

The implemented confirmed states are:

- `confirmed_spend`
- `confirmed_receive`
- `confirmed_change`
- `confirmed_fee`

The helper accepts the exact pending-to-confirmed transitions encoded by the lifecycle validation helper.

### ✅ Why Lifecycle Projection Matters Here

This matters to Phase 050 because the current offline-capable flow feeds reporting and state projection surfaces that distinguish what is pending from what is already confirmed.

The transport and verification contract stay aligned with the lifecycle vocabulary expected by downstream reporting logic.

## ⚙️ Current Implementation Model

The live Phase 050 model is anchored to the existing single-package `TxPackage` path.

The current repository model consists of:

- verified receiver routing via `ReceiverCard`,
- canonical published routing via `ReceiverCardRecord`,
- sender-side confidential output construction and local self-verification,
- portable `TxPackage` serialization with a canonical digest,
- package-local validation through `TxVerifierImpl`,
- optional public spend authorization through `SpendProofWire` and `SpendAuthWire`,
- runtime-owned-output scanning only after local validity,
- import only after both local validity and an accepted import-ready status.

The transport and lifecycle contract stay aligned with the live wallet, RPC, and simulator seams listed above.

## 🔐 Current Implementation Guarantees

The current codebase already provides several strong local guarantees:

1. receiver cards are signed and fail closed on malformed structure, invalid ECC points, bad signatures, and expiry,
2. receiver-card publication has one canonical live record type with revocation and stale-epoch checks,
3. output construction verifies self-decryption, commitment consistency, and range-proof integrity,
4. local package verification rejects digest mismatch, malformed structure, duplicate keys, zero nonce, duplicate output nonce, and fee inconsistency,
5. public spend authorization provides a real structured public contract with receiver-card binding and statement integrity,
6. runtime verification only reports import readiness when both local validity and accepted lifecycle status hold,
7. lifecycle projection preserves the pending and confirmed vocabulary used by downstream reporting.

## ⭐ Normative Current-State Summary

The current Z00Z offline-capable transaction model is a portable confidential transaction package flow built around a verified receiver-routing artifact, sender-side confidential output construction, package-local verification, runtime-owned-output scanning, and status-gated import readiness.

In short, the live Phase 050 model consists of:

- verified receiver routing via `ReceiverCard`,
- canonical published routing via `ReceiverCardRecord`,
- confidential output construction and local self-verification,
- portable `TxPackage` serialization with a canonical digest,
- package-local validation through `TxVerifierImpl`,
- optional public spend authorization through `SpendProofWire` and `SpendAuthWire`,
- runtime-owned-output scanning only after local validity,
- import only after both local validity and an accepted import-ready status.

This is the only authoritative current-state Phase 050 model.
