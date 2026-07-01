# 10. Privacy & Anonymity

## 🎯 10.1 Scope and Canonical Truth

This section consolidates privacy and anonymity ideas from `.planning/temp/Z00Z-ECC-IDEAS.md` into a repository-grounded implementation document.

This document is intentionally not a copy of the draft. Live repository artifacts override older formulas, speculative signatures, and legacy wording.

### ✅ 10.1.1 Status Legend

| Status | Meaning |
| --- | --- |
| `Implemented` | Live in the current repository and should be described as shipped behavior. |
| `Partial` | Live on the current accepted path, but not a full public theorem or not yet generalized. |
| `Compatibility-only` | Supported to preserve current workflows, but not the preferred privacy lane. |
| `Future` | Planned direction, not yet implemented. |
| `Drift corrected` | The source draft contained an older or over-broad concept that must be normalized before reuse. |

### 🔗 10.1.2 Canonical Repository Anchors

- `crates/z00z_crypto/src/ecdh.rs`
- `crates/z00z_crypto/src/kdf.rs`
- `crates/z00z_wallets/src/core/address/stealth_card.rs`
- `crates/z00z_wallets/src/core/address/stealth_request.rs`
- `crates/z00z_wallets/src/core/address/stealth_request_types.rs`
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs`
- `crates/z00z_wallets/src/core/stealth/output_build.rs`
- `crates/z00z_wallets/src/core/stealth/tag.rs`
- `crates/z00z_wallets/src/core/stealth/facade_zkpack.rs`
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/core/chain/receiver_card_record.rs`
- `.planning/STATE.md`
- `docs/code-review/2026-04-08-doublecheck-zkpack-plonky3-audit.md`

### 🔒 10.1.3 Cross-Spec Compliance Anchors

- `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md` is canonical for the regular-tx proof boundary: `TxProofWire` is still empty today, so receiver-secret-gated anti-theft remains wallet-local logic plus future-proof direction rather than a finished public theorem.
- `.planning/phases/050-offline-tx/050-Offline-Tx-Spec.md` is canonical for package-local offline exchange: `TxPackage` is portable and locally verifiable, but transported package bytes remain sensitive material and raw-card routing must not overwrite request-bound privacy guidance.
- `.planning/phases/000/028-crypto-audit-storage/028-01-PLAN.md` and `.planning/phases/000/031-refactor-architecture/031-08-PLAN.md` are canonical for checkpoint-artifact truth: checkpoint semantics remain statement-bound and verifier-hook scoped, while `cp_proof` stays compatibility or attestation payload rather than a standalone authoritative backend today.

### 🚨 10.1.4 Privacy Truth Boundary

The current canonical privacy promise is narrower than the most optimistic wording in older drafts.

- state-level unlinkability is the primary delivered privacy goal
- request-bound receive is the preferred privacy lane
- card-only and plain receive remain compatibility behavior unless separately proved equivalent
- inbox is an optional notification helper, not the source of truth for asset ownership
- regular `TxProofWire` is still empty today, so spend-side receiver-secret exclusion is not yet a finished public proof statement
- checkpoint proof semantics remain package-coupled or attestation-bound and must not be described as a standalone authoritative privacy theorem
- any statement about full trustless end-to-end privacy must be scoped to the exact live wallet and scan path, not widened into a global theorem

## 🔒 10.2 Privacy Objectives

### 🎯 10.2.1 State-Level Unlinkability

Observers of the JMT state should not see a stable receiver identifier in the leaf. The canonical design goal is that receiver identity does not appear directly in public state.

### 💰 10.2.2 Amount Confidentiality

Amounts are hidden behind commitments and receiver-side encrypted pack recovery. The long-term goal is not only confidentiality in software, but circuit-aligned confidentiality for OWF and proof systems.

### 👤 10.2.3 Receiver Identity Minimization

Receiver identity material must stay off-chain and out of helper systems unless explicitly protected.

- `owner_handle` must not appear in public state
- `view_pk` must not appear in public state
- stable wallet identifiers must not appear inside inbox hints or leak-prone metadata

### 🧾 10.2.4 Spendability Assurance

Privacy is not enough if a wallet can decrypt an output but cannot spend it. The practical privacy model therefore includes wallet-local spendability validation, not only ciphertext recovery.

### 🕸️ 10.2.5 Transport Privacy as a Separate Layer

State unlinkability does not automatically provide transport privacy. Submission channels, bundle handling, logs, and offline package forwarding must be treated as a distinct privacy boundary.

## ✅ 10.3 Current Implemented Privacy Surfaces

| Area | Status | Canonical Anchors | Notes |
| --- | --- | --- | --- |
| ECDH stealth delivery and receiver-side recovery | `Implemented` | `crates/z00z_crypto/src/ecdh.rs` | Live sender/receiver DH path with fail-closed point validation. |
| Canonical stealth KDF domains | `Implemented` | `crates/z00z_crypto/src/kdf.rs` | Live derivation of `view_sk`, `view_pk`, `owner_handle`, `owner_tag`, and `leaf_ad`. |
| Signed `ReceiverCard` validation | `Implemented` | `crates/z00z_wallets/src/core/address/stealth_card.rs` | Cards are treated as untrusted input and require validation. |
| Signed `PaymentRequest` with `req_id` | `Implemented` | `crates/z00z_wallets/src/core/address/stealth_request.rs`, `crates/z00z_wallets/src/core/address/stealth_request_types.rs` | The request-bound lane already exists and includes chain, expiry, metadata, and signature validation. |
| Request-aware output building | `Implemented` | `crates/z00z_wallets/src/core/stealth/output_build.rs` | Sender path already binds request-aware data and rejects malformed request contexts. |
| Hedged ephemeral generation and duplicate-`R` suppression | `Implemented` | `crates/z00z_wallets/src/core/stealth/output_build.rs`, `crates/z00z_wallets/src/core/address/ephemeral_cache.rs`, `crates/z00z_wallets/tests/test_adversarial.rs` | Sender output building already hedges ephemeral scalar derivation and rejects repeated `R` values for the same receiver context. |
| Request-aware tag helpers | `Implemented` | `crates/z00z_wallets/src/core/stealth/tag.rs` | Request-bound filtering is a real live surface, not only a planning idea. |
| Wallet scan, decrypt, and commitment re-check | `Implemented` | `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` | The receiver path derives DH, checks ownership-related fields, decrypts pack, and verifies commitment consistency. |
| Wallet-local owner-tag and two-factor spendability checks | `Implemented` | `crates/z00z_wallets/src/core/stealth/output.rs` | The live wallet distinguishes “decryptable” from “mine and spendable”; receiver-secret plus recovered coin secret remain the accepted local spend-ownership rule. |
| Receiver-card publication record | `Implemented` | `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` | `ReceiverCardRecordV1` is the current publication contract. |
| TOFU and pinning policy | `Implemented` | `crates/z00z_wallets/src/core/address/stealth_request.rs` | Identity pinning and validation are already part of the wallet trust model. |
| Receiver-card rotation lifecycle | `Implemented` | `crates/z00z_wallets/src/services/wallet_service_actions_tofu.rs`, `crates/z00z_wallets/src/services/wallet_service_tests.rs` | The repository already supports TOFU confirmation/revocation and receive-view rotation, so privacy documentation must treat card lifecycle as real behavior, not future-only theory. |
| Request-bound receive as preferred lane | `Implemented` | `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs`, `.planning/STATE.md` | This is explicitly the accepted privacy lane in active planning truth. |
| Card/plain receive | `Compatibility-only` | `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` | Must not be described as equivalent to the request-bound lane. |
| `ZkPack_v1` encryption facade | `Implemented` | `crates/z00z_wallets/src/core/stealth/facade_zkpack.rs` | Live today, but still `ChaCha20Poly1305`-based and not yet field-native. |
| Poseidon2-based pack and OWF parity | `Future` | `crates/z00z_wallets/src/core/stealth/facade_zkpack.rs`, `docs/code-review/2026-04-08-doublecheck-zkpack-plonky3-audit.md` | Planned migration, not current truth. |

## 👁️ 10.4 Privacy Boundary by Observer Class

### 🔍 10.4.1 State-Only Observer

This is the strongest current privacy story.

What a state-only observer can see:

- leaf-level public fields such as `R`, commitment data, encrypted pack bytes, and helper tags
- state growth patterns and checkpoint cadence

What a state-only observer should not learn directly:

- `view_pk`
- `owner_handle`
- a stable receiver identifier

### 📦 10.4.2 Package or Bundle Leak Observer

This is a different threat class from state-only observation.

If a `tx_package_v1`, forwarding bundle, backup artifact, or log leaks, the observer may recover correlation between receiver-side artifacts and outputs even if public state itself remains unlinkable.

Therefore:

- state unlinkability must not be described as protection against leaked transaction packages
- outer encryption and wallet hygiene are part of the privacy boundary
- packages and logs must be treated as sensitive material

### 🧑‍⚖️ 10.4.3 Submission Edge or Aggregator View

During submission, the edge may temporarily see request/card context that is not present in the resulting public state. This is a transport and operational privacy concern, not a reason to widen the public-state model.

### 🧠 10.4.4 Wallet-Local Accepted Path

The wallet is allowed to do stronger checks than public state proves. Current receive safety and spendability language must stay scoped to the live wallet path unless separately promoted into a validator-facing theorem.

## 🔑 10.5 Canonical Receive Model

### ✅ 10.5.1 Preferred Privacy Lane: Request-Bound Receive

The canonical privacy-first receive flow is request-bound.

Characteristics:

- sender uses a validated signed `PaymentRequest`
- `req_id` participates in request-aware routing and filtering surfaces
- wallet validates request metadata, expiry, chain, and identity pins
- receive classification remains anchored to the request-aware accepted path

This lane should be treated as the default implementation target for future privacy work.

### ⚠️ 10.5.2 Compatibility Lane: Card-Only or Plain Receive

Card-only and plain receive continue to exist for interoperability and recovery, but they are not the canonical privacy target.

Rules:

- describe them as bounded compatibility behavior
- do not widen them into a claim of equal privacy or equal safety
- do not let convenience-path wording overwrite the request-bound privacy baseline

### 🧪 10.5.3 Wallet Receive Validation Requirements

The accepted receiver path must continue to validate more than mere decryption.

- derive DH from the live stealth leaf and receiver-side material
- verify ownership-related binding before classifying an output as accepted
- verify commitment consistency after pack recovery
- fail closed on malformed points, malformed requests, signature mismatches, and stale request metadata

In particular, the receiver path must preserve the distinction between:

- decryptable output
- output that is actually mine
- output that is both mine and spendable under the accepted wallet-local ownership rule

That distinction is what prevents the practical “decryptable-but-not-spendable” failure mode.

This is the practical anti-soft-burn boundary in the current stack.

### 🏷️ 10.5.4 Local Scan Prefilter Boundary

`tag16` is a local prefilter only.

- it is a performance hint used to reduce scan cost
- it is allowed to collide
- it must not be documented as an address, receiver identifier, or ownership proof
- it must not be treated as a standalone DoS defense

Repository-aligned guidance:

- `tag16` is bound per leaf or per request-aware scan context, not to a permanent receiver identity
- attacker-generated false positives remain possible if attacker-visible receiver material leaks
- wallet implementations must prefer rate limits, backoff, and deferred processing over assuming `tag16` itself stops spam

## 📬 10.6 Inbox and Notification Privacy

### 🚨 10.6.0 Inbox Truth Boundary

Inbox planning in this phase must stay inside the same truth boundary as the rest of the document.

- there is no shipped canonical inbox helper module in the repository today
- inbox remains a future helper surface, not an ownership or consensus surface
- request-bound routing is the only acceptable canonical direction for future inbox work
- any inbox wording must distinguish helper-only planning from already implemented wallet receive behavior

| Inbox Surface | Status | Canonical Interpretation |
| --- | --- | --- |
| Request-bound receive and request validation | `Implemented` | Real wallet behavior and the only approved privacy lane to build inbox helpers around. |
| TOFU, pinning, and card lifecycle | `Implemented` | Live trust material that future inbox helpers must respect instead of bypassing. |
| Helper inbox routing, route buckets, encrypted hints | `Future` | Planning-only surfaces; none should be described as shipped. |
| OPRF or PIR retrieval | `Future` | Research direction only. |

### 🧭 10.6.1 Canonical Role of Inbox

Inbox is an optional notification helper. It is not the authoritative ownership registry and it must not become a public address book.

If inbox exists, it should only reduce scan cost or improve UX.

### 🚫 10.6.2 Inbox Non-Negotiables

An inbox helper must not store or expose:

- `owner_handle`
- `view_pk`
- stable wallet identifiers
- plaintext amounts, asset types, or sender identity

It may store only helper material such as leaf locators, asset locators, or encrypted hints.

### 🪣 10.6.3 Recommended Future Variant: Secret Route Buckets

The strongest draft idea that aligns with the current repository is the bucketed hint system based on request-bound routing.

Repository-aligned normalization:

- routing should bind to the request-aware lane, not to a permanent receiver identity
- route derivation should be per-request and epoch-scoped
- buckets are for scan reduction, not identity publication
- request rotation is a privacy requirement, not only a UX detail

### 🔐 10.6.4 Advanced Future Variants

The draft also proposes OPRF or PIR-style inbox retrieval. These remain future work.

If implemented later:

- they must remain helper-only and off-consensus
- they must not redefine the canonical ownership model
- they must not introduce a stable lookup token that outlives request rotation

### 📏 10.6.5 Current Status

Inbox helper logic is a planning surface, not a shipped canonical module. The document should therefore describe it as a privacy roadmap, not as live behavior.

## 🕸️ 10.7 Transport and Network Privacy

### 🚨 10.7.0 Transport Truth Boundary

Transport-related planning must use the same repository-grounded status discipline.

- there is no shipped protocol-enforced anonymity overlay today
- there is already a reserved OnionNet crate boundary for future work
- encrypted wallet backup and export flows exist, but they do not upgrade the whole transport/privacy story into a solved system-wide guarantee
- package forwarding, bundle handling, and submission-edge privacy remain separate from state unlinkability

| Transport Surface | Status | Canonical Interpretation |
| --- | --- | --- |
| Encrypted wallet backup and export flows | `Implemented` | Real wallet data-protection surface for backup transport, but not equivalent to full transaction transport anonymity. |
| Package/bundle privacy policy across all handoff paths | `Partial` | The repository treats leaks as meaningful, but the policy is not yet unified across all package and forwarding surfaces. |
| OnionNet crate boundary | `Implemented` | Real namespace and ownership boundary for future node-owned anonymity work. |
| Protocol-enforced transport anonymity overlay | `Future` | Not shipped. |

### 🚚 10.7.1 Transport Privacy Is Not Yet a Protocol Guarantee

The current protocol truth is best-effort transport privacy, not protocol-enforced transport anonymity.

Implications:

- TLS, Tor, bridges, and submission relays are deployment choices
- no-logging and secure forwarding are operational requirements
- privacy claims must separate chain-state unlinkability from transport unlinkability

### 🌀 10.7.2 OnionNet Boundary

`crates/z00z_networks/onionnet` already exists as the reserved module boundary for a future privacy overlay, but it is still a placeholder.

This means:

- OnionNet is a real planned architecture boundary
- OnionNet is not yet a shipped anonymity layer
- future transport privacy work should land there instead of leaking into wallet business logic

### 📦 10.7.3 Offline Forwarding and Bundle Hygiene

The ECC ideas draft correctly highlights a practical leak path: forwarding bundles can reintroduce correlation if they are logged, backed up, or shared without re-encryption.

Required guidance:

- treat packages and bundles as sensitive private material
- use outer encryption for export, handoff, and backup workflows
- do not log receiver artifacts or forwarding packages in plaintext

## ⚠️ 10.8 Concept Drift Corrections

| Drifted Draft Direction | Canonical Repository-Aligned Concept |
| --- | --- |
| Treating privacy formulas in the draft as canonical by themselves | Live names, field bindings, and validation rules come from `z00z_crypto` and `z00z_wallets`, not from draft prose. |
| Treating `ReceiverCard` as a generic public blob | The current `ReceiverCard` is signed, validated, and treated as untrusted input. |
| Treating `PaymentRequest` as optional UX only | The request-bound path is already the preferred privacy lane and must remain first-class. |
| Treating repeated ephemeral randomness as harmless | Reused `R` values are a privacy failure because they create immediate linkability; the live sender path already hedges `r` generation and suppresses duplicates. |
| Treating card-only receive as equivalent to request-bound receive | Card-only receive is compatibility-only until separately proved. |
| Treating inbox as an address registry | Inbox is only a notification helper and must not store stable receiver identifiers. |
| Treating `tag16` as an identity token or anti-spam boundary | `tag16` is only a scan prefilter; collisions and attacker-induced false positives are part of the design envelope. |
| Treating transport privacy as solved by state unlinkability | Package leaks, logs, backups, and submission edges are a separate privacy boundary. |
| Treating ECDH-stealth delivery as a PQ privacy claim | The current receive privacy lane is classical ECDH-based; PQ claims must stay scoped to the parts of the stack that actually implement them. |
| Treating current pack crypto as already Poseidon2-based | The live wallet pack remains `ChaCha20Poly1305`-based today; Poseidon2 migration is future work. |
| Treating OnionNet as already deployed privacy transport | OnionNet is a reserved placeholder boundary, not a shipped overlay. |
| Treating all accepted wallet checks as public-state theorems | Current strong anti-theft and anti-burn wording must stay scoped to the accepted wallet path unless explicitly promoted. |

## 🧱 10.9 Implementation Direction and Backlog

This backlog converts the planning conclusions above into implementation-ready items. Each row starts from the current repository truth, names the real gap, defines the next task, and pins an explicit verification rule.

| Backlog ID | Area | Implemented Truth | Implemented Gap | Future Task | Verification Rule | Plan File |
| --- | --- | --- | --- | --- | --- | --- |
| `PRIV-055-01` | Field-native pack parity | `ZkPack_v1` is live in wallet stealth flows and currently uses `ChaCha20Poly1305`-based sealing. | The production pack is not yet aligned with field-native OWF/proof goals, so docs and code still require a split between live wallet crypto and future proof-friendly pack design. | Introduce a crypto-owned field-native pack contract, freeze current fixture truth, and add a migration seam that preserves current wallet behavior until parity is proven. | Release tests must prove wallet scan and decrypt parity across old and new pack fixtures, and documentation must continue to label the old pack as current truth until the migration lands. | `055-01-PLAN.md` |
| `PRIV-055-02` | Request-bound inbox helper I1 | Request-bound receive, request validation, TOFU, and card lifecycle are already live. | No shipped inbox helper exists yet, so scan-reduction ideas remain prose only. | Implement a helper-only inbox module that derives per-request route buckets or encrypted hints from request-bound context only and keeps compatibility paths explicitly secondary. | Tests must prove no persisted inbox record contains `owner_handle`, `view_pk`, or another stable receiver identifier; request rotation must change the helper lookup surface. | `055-02-PLAN.md` |
| `PRIV-055-03` | Package export and forwarding hygiene | Wallet backup and export already use encrypted container flows around `WalletExportPack`. | The repository does not yet enforce one explicit privacy policy across tx packages, forwarding bundles, logs, and export handoff surfaces. | Add a shared package privacy policy for export, forwarding, and redaction, including encrypted handoff wrappers where package bytes may leave wallet-local trust boundaries. | Source and behavior checks must fail if plaintext receiver artifacts or package payloads are logged or exported through the protected paths covered by the new policy. | `055-03-PLAN.md` |
| `PRIV-055-04` | Compatibility-boundary enforcement | Request-bound receive is the accepted privacy lane; card/plain receive still exists for compatibility. | The phase has no dedicated guardrail set that prevents future docs or tests from silently treating compatibility paths as privacy-equivalent. | Add wording guards and behavior tests that keep card/plain receive explicitly below the request-bound path in privacy-sensitive planning and implementation surfaces. | Regression tests and source-shape guards must fail if card/plain receive is described or exported as equivalent to the request-bound privacy lane. | `055-02-PLAN.md` |
| `PRIV-055-05` | Transport anonymity boundary | `crates/z00z_networks/onionnet` is already reserved as the node-owned privacy overlay namespace. | The crate is still placeholder-only and there is no implemented transport anonymity pipeline. | Land future node-owned transport anonymity work inside OnionNet, keeping wallet ownership logic and transport overlay concerns separate. | The first transport-anonymity implementation must live under the OnionNet crate boundary and must not introduce wallet business-logic ownership checks into transport modules. | `055-03-PLAN.md` |
| `PRIV-055-06` | Scan prefilter abuse handling | `tag16` exists as a request-aware or leaf-bound local prefilter and repeated `R` suppression already exists on the sender side. | The repository documents the prefilter boundary, but does not yet define one explicit receiver-side abuse policy for `tag16` collision storms and deferred scan behavior. | Add a wallet-local scan abuse policy with rate limiting, backoff, and deferred processing rules for collision-heavy prefilter hits. | Tests must show that collision-heavy candidate streams do not turn `tag16` into an ownership oracle and do not force blocking UI-style receive behavior on the protected path. | `055-02-PLAN.md` |

### 📌 10.9.1 Execution Ordering

Backlog execution should preserve the current truth boundary while reducing the highest-risk gaps first.

1. `PRIV-055-04` to lock the compatibility boundary before future drift reappears.
2. `PRIV-055-03` to reduce package and export leakage risk on already existing handoff surfaces.
3. `PRIV-055-02` to add request-bound helper routing without inventing a second ownership model.
4. `PRIV-055-06` to harden receiver-side scan behavior under adversarial load.
5. `PRIV-055-01` when proof/pack migration resources are available.
6. `PRIV-055-05` when transport anonymity becomes an active implementation wave.

### 📌 10.9.2 Backlog Discipline

Every future privacy task in this phase must preserve the same repository-grounded framing:

- start from an already implemented truth, not from draft formulas
- state the concrete gap without widening claims
- define one future task with one clear ownership boundary
- attach one verification rule that can fail closed

### 📦 10.9.3 Pack Migration Task Split

Plan file: `055-01-PLAN.md`

1. Freeze the current `ZkPack_v1` truth, fixture corpus, and package-privacy wording before any migration begins.
2. Define a crypto-owned field-native pack contract that can coexist with the current wallet pack instead of replacing it in one jump.
3. Add dual-read or single-write migration gates only after decrypt, commitment-open, and associated-data parity are proven.
4. Keep old-pack wording as current truth until the new pack is consumable by the wallet path and any future OWF or proof path that adopts it.

### 📬 10.9.4 Inbox Helper Task Split

Plan file: `055-02-PLAN.md`

1. Derive helper routing only from request-bound context and keep raw-card compatibility routes explicitly secondary.
2. Persist only per-request route buckets, encrypted hints, or equivalent helper material with no stable receiver identifier.
3. Add compatibility-boundary guards so card/plain receive cannot silently be promoted to privacy-equivalent status.
4. Define wallet-local collision, backoff, and deferred-processing rules so `tag16` remains a prefilter rather than an ownership oracle.

### 🕸️ 10.9.5 Transport Privacy Task Split

Plan file: `055-03-PLAN.md`

1. Define one shared package privacy policy for export, forward, relay, backup, and log surfaces.
2. Require encrypted handoff wrappers or explicit redaction whenever package bytes leave wallet-local trust.
3. Keep OnionNet as the only node-owned namespace for future transport-anonymity implementation work.
4. Separate deployment privacy helpers from protocol-level anonymity claims in docs and verification rules.

## 🚨 10.10 Non-Negotiable Privacy Rules

- public state must not contain stable receiver identifiers
- privacy documentation must distinguish state-only unlinkability from transport privacy
- request-bound receive remains the preferred privacy lane
- card-only and plain receive must stay labeled as compatibility-only unless separately proved
- receiver-side acceptance must preserve the difference between decryptable, mine, and spendable
- repeated ephemeral `R` values must be treated as privacy failures, not as benign retries
- `tag16` must stay documented as a prefilter hint, never as an identity token or sufficient anti-spam control
- inbox must remain a hint system, never an identity registry
- package, bundle, and backup leaks are privacy-relevant and must be treated as sensitive material
- current `ZkPack_v1` must be described honestly as live today, while Poseidon2-based migration stays future-facing until implemented
- concept drift from older drafts must be corrected toward canonical repository concepts before new privacy work is planned
