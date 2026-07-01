# Fused Crypto Audit for `z00z_core`

📌 This document fuses five independent audit reports into one canonical view.
It is organized by topic, not by source-file order, and keeps conflicting or
single-source claims explicit instead of flattening them away.

📌 Input set:

- `core-audit-glm5.md`
- `core-audit-gpt54.md`
- `core-audit-m27.md`
- `core-audit-mimov2.md`
- `core-audit-sonet46.md`

## [F-01] Executive Summary

🚨 Canonical verdict: `Risky but salvageable`, but `blocked` for production
release until the confirmed integrity and ownership-binding gaps are fixed.

📌 Cross-report consensus is strong on these themes:

- Genesis consensus anchoring is currently fail-open.
- Ownership and stealth semantics are not fully bound at verification time.
- Untrusted wire and import boundaries expose or mishandle sensitive asset
  state.
- Asset-definition and registry integrity checks are weaker than the protocol
  intent described by the reports.
- Nonce and range-proof safety checks contain fallback or environment-dependent
  behavior that weakens assurance.

📌 Some findings are single-source or partially disputed. Those items remain in
this fusion, but they are labeled as conditional or follow-up items instead of
being promoted silently to confirmed blockers.

## [F-02] Scope and Review Method

📌 All five source reports reviewed `crates/z00z_core/src/**/*.rs` and excluded
vendor code under `z00z_crypto/tari/`.

📌 Coverage emphasis was consistent across the reports: asset commitments,
owner-signature semantics, stealth fields, nonce derivation, asset-definition
identity, registry and snapshot integrity, genesis generation, and genesis
validation.

📌 Report depth differed. The strongest file-level coverage came from the GLM-5,
MiniMax M2, and Sonnet 4.6 audits. The GPT-5.4 audit focused more on integrity
and protocol-meaning composition than file-by-file coverage. The MiMo-V2 report
added specific concerns on fee binding, zero-nonce enforcement, and Poseidon2
domain usage.

## [F-03] Security Goals and Threat Model

📌 The fused audit assumes the crate is trying to provide these properties:

- Confidential amounts through Pedersen commitments and Bulletproofs+.
- Binding between committed values, ownership semantics, and spendable asset
  state.
- Deterministic and consensus-safe genesis generation.
- Safe wire and DTO boundaries for asset import, export, and registry updates.
- Nonce uniqueness sufficient for privacy and replay resistance.
- Domain separation across genesis, asset, metadata, ownership, and nonce
  operations.
- Fee-payment rules that cannot be bypassed or malleated.

📌 The common adversaries in scope are passive observers, malicious clients,
malicious validators or snapshot senders, compromised wallet or storage layers,
and operators who accidentally invoke unsafe or test-oriented code paths.

📌 The fused trust model is also consistent across sources:

- `z00z_core` delegates primitive correctness to `z00z_crypto`.
- `z00z_core` itself owns protocol meaning, binding, and validation order.
- Any wire, JSON, snapshot, or DTO input must be treated as untrusted.
- Time providers, RNGs, and persistent counters are part of the security model,
  not just convenience helpers.

## [F-04] Confirmed Strengths

✅ All reports that commented on the primitive layer agreed that the backend
selection is sound: Pedersen commitments, Bulletproofs+, Schnorr signatures,
and domain-separated hashing come from battle-tested components rather than
local cryptographic reimplementation.

✅ Domain separation is widely regarded as one of the strongest aspects of the
crate. Multiple reports explicitly praised the number of distinct domains,
network-aware genesis separation, and versioned labels.

✅ Deterministic genesis construction was consistently described as a strong
design choice when combined with per-asset derivation inputs and isolated RNG
state.

✅ Error handling quality is generally good. The reports repeatedly note typed
errors, `thiserror`, limited use of panics in production paths, and a general
absence of custom crypto shortcuts.

✅ Batch verification, atomic file-write patterns, and explicit overflow checks
in gas math were all identified as positive operational properties.

📌 Important narrowing of the positive claims:

- Owner-signature verification is syntactically correct Schnorr verification,
  but that does not by itself prove asset ownership.
- Stealth tuple presence checks are useful, but they do not prove the full
  cryptographic binding that later findings say is missing.
- Genesis generation is strong, but genesis consensus enforcement is still
  incomplete.

## [F-05] Confirmed and Near-Confirmed Blockers

### [F-05.1] Genesis consensus anchoring is effectively disabled

🚨 This is the clearest multi-source blocker. Several reports independently
state that mainnet and testnet genesis-state hash constants are `None`, causing
`verify_genesis_consensus()` to fail open instead of fail closed.

📌 Canonical finding:

- Missing expected consensus hashes mean incompatible genesis states can pass
  local verification.
- The current behavior is acceptable only for development flows, not for any
  production or consensus-sensitive deployment.
- The final implementation must treat a missing expected hash as an error for
  protected networks.

### [F-05.2] Ownership and stealth binding are incomplete

🚨 Multiple reports describe the same root problem from different angles:
`owner_signature` is treated as stronger proof than the current code can
justify.

📌 Canonical finding:

- The local signature check proves that the supplied `owner_pub` signed the
  message.
- It does not, by itself, prove that `owner_pub` is cryptographically bound to
  the commitment opening or otherwise anchored by trusted state.
- A separate multi-source concern says stealth-related fields are not fully
  covered by the signed payload, and one audit additionally reports that the
  view tag is not recomputed and verified against the ECDH-derived secret.

📌 Fused decision:

- Treat ownership binding as incomplete until the verifier can prove either
  ownership of the commitment opening or equality with an externally anchored
  owner key.
- Treat stealth-field integrity as incomplete until `r_pub`, `owner_tag`,
  `enc_pack`, `tag16`, and `leaf_ad_id` are either signed, otherwise bound, or
  explicitly documented as mutable and non-authoritative.

### [F-05.3] Registry and definition integrity are under-bound

🚨 The reports describe a broader integrity problem across registry snapshots,
definition identity, and runtime configuration.

📌 Canonical finding:

- One high-confidence audit states that registry snapshot integrity hashes only
  definition IDs, not the full definition payload.
- Other reports say `AssetDefinition.id` can be supplied externally without a
  strict derivation check, and `DefinitionWire` can bypass validation on
  deserialization.
- Another report says runtime config currently uses or exposes a test-oriented
  asset-ID domain in production-relevant paths.

📌 Fused decision:

- The integrity target must be the full canonicalized definition payload, not
  just its identifier list.
- Identity derivation and validation must be centralized so that config, wire,
  and runtime constructors cannot drift.

### [F-05.4] Confidential wire and import boundaries are unsafe

🚨 The strongest boundary-related issues are concentrated around `AssetWire` and
its neighboring DTOs.

📌 Canonical finding:

- One report says plaintext `amount` in `AssetWire` breaks confidentiality.
- Several reports say `secret` is present or still reachable through internal
  wire/import paths and should never be accepted from untrusted payloads.
- Another report says the public DTO path drops frozen and slashed state during
  rehydration, which weakens the integrity of externally represented asset
  state.

📌 Fused decision:

- Secret-bearing and privacy-sensitive fields must be split from general wire
  payloads.
- Untrusted input must never be able to inject asset secrets.
- State flags that affect protocol semantics must either survive the public DTO
  boundary faithfully or be rejected at that boundary.

### [F-05.5] Candidate blocker: fee binding is not proven

⚠️ This item is important but not as well corroborated as the previous groups.

📌 Canonical finding:

- One report explicitly states that fee amounts are not committed or bound into
  the proof statement.
- Another report treats fee binding as unresolved rather than broken.
- A third report says gas-asset validation appears to rely only on `Coin`
  class, not on the canonical native asset ID.

📌 Fused decision:

- Treat fee-binding as `BLOCKED pending tx-layer confirmation`.
- Do not claim fee integrity is solved at the `z00z_core` layer until the
  transaction statement and verification path are audited end-to-end.

## [F-06] Serious Weaknesses Requiring Pre-Production Fixes

### [F-06.1] Nonce derivation, uniqueness, and persistence are too forgiving

📌 Multiple reports describe the same risk cluster:

- Helper paths silently replace time-provider failures with timestamp `0`.
- Some security checks are disabled in test builds, including zero-nonce or
  range-proof checks.
- Nonce uniqueness is deferred to higher layers without a proven current
  mechanism.
- `NonceCounter` persistence lacks integrity protection.
- Some derived nonces are not network-bound.
- Unsafe increment helpers warn instead of enforcing transactional usage.

📌 Fused decision:

- Production nonce construction should use explicit `Result`-returning APIs.
- Zero nonce should be rejected independently of test configuration.
- Counter persistence should have rollback protection.
- The storage contract for uniqueness needs to be explicit and mandatory.

### [F-06.2] Encoding and canonicalization rules are underspecified

📌 The merged reports identify several serialization and comparison issues:

- Non-constant-time commitment equality.
- Possible canonicality uncertainty for signature scalar parsing.
- `AssetPackPlain` default decoding emphasizes length over canonicality or rich
  error reporting.
- `lock_height` encoding is either ambiguous or at least too implicit for a
  consensus-sensitive signing message.
- Variable-length string hashing in asset-definition derivation lacks strong
  framing in at least one reported path.

📌 Fused decision:

- Canonical serialization needs to be made explicit where protocol identity,
  signatures, or state hashes depend on it.
- Audit trails should distinguish proven issues from single-source concerns,
  but the combined direction is consistent: make framing, discriminants, and
  constant-time comparison explicit.

### [F-06.3] Amount and range-proof boundaries are too loose

📌 Several reports object to `MAX_AMOUNT = u64::MAX` as a protocol-facing limit.

📌 Canonical finding:

- The accepted amount range may exceed what the configured proof bit width can
  represent safely.
- Even where the proof system is theoretically 64-bit, the crate currently does
  not tie the amount limit and the proof-width constant together explicitly.
- Additional reports note that `new_confidential()` can produce an ownerless or
  under-documented asset state, and that callers need clearer guidance on how
  blinding and ownership data must be persisted.

### [F-06.4] Genesis seed and network configuration validation need redesign

📌 The fused audit treats the current entropy-validation story as unsound.

- One report states that a 200-bit entropy threshold is mathematically
  impossible to satisfy on a 32-byte empirical sample.
- Another says Shannon estimation on 32 bytes is too weak to support a strong
  policy claim even if the threshold were adjusted.
- A separate report says `ChainType::from(&str)` silently falls back to Devnet,
  which can route configuration mistakes into the wrong network behavior.

📌 Fused decision:

- Replace fragile statistical seed heuristics with source-of-entropy policy,
  explicit blacklist checks, and fail-closed network parsing.

### [F-06.5] Native fee-asset enforcement is incomplete

📌 At least one report says fee validation checks only `AssetClass::Coin` and
not the canonical native coin identifier.

📌 Fused decision:

- Treat fee payment as valid only when both class and identity constraints are
  satisfied.

## [F-07] Moderate and Informational Findings Worth Keeping

📌 The following items are lower severity, but they survived the fusion because
they add unique operational or maintainability value:

- `from_decimal(f64)` precision risk for large values.
- Reserved `policy_flags` bits may not be rejected in all constructors.
- `secret` should likely use `Hidden<T>` or equivalent protection.
- Debug/log behavior may expose nonces or rely on `eprintln!` instead of
  structured logging.
- `state/mod.rs` is empty and some code paths or error variants appear dead or
  incomplete.
- Manual timestamp formatting and partial global-registry mutation are fragile
  operational details.
- One report questions whether blinding or signature parsing docs sufficiently
  document canonicality requirements.
- One report notes that using the same blinding scalar for commitment and owner
  key is a standard Mimblewimble-style tradeoff, not automatically a bug.

📌 These findings do not change the blocker list, but they do shape the cleanup
work that should follow the blocker fixes.

## [F-08] Open Ambiguities and Conflicts

📌 The fusion preserves these unresolved questions instead of forcing false
consensus:

- Whether fee binding is actually broken in the full transaction statement or
  only unproven from the audited slice.
- Whether wire-package scalar canonicality concerns are real in practice,
  depending on the guarantees of the underlying crypto library.
- Whether snapshot authentication already exists elsewhere and simply was not in
  scope for some reports.
- Whether nonce uniqueness enforcement exists in storage or transaction layers
  outside the reviewed files.

📌 The main report-to-report disagreements were semantic rather than factual:

- One audit reported no S1 findings, while others found several. The fusion
  resolves this by using the stronger evidence where multiple reports align.
- One audit praised owner-signature and stealth validation as well-designed.
  Others showed that the existing checks are only partial. The fusion preserves
  both statements by distinguishing syntactic correctness from full semantic
  binding.

## [F-09] Unified Remediation Roadmap

📌 Phase 1, unblockers:

1. Make genesis consensus verification fail closed on mainnet and testnet.
2. Bind ownership and stealth-critical fields to verifiable authority.
3. Remove secret-bearing and plaintext-sensitive fields from untrusted wire
   paths.
4. Redesign registry and asset-definition integrity around canonical full-payload
   hashing and validation.
5. Confirm or redesign fee binding end-to-end.

📌 Phase 2, hardening:

1. Remove timestamp `0` fallbacks and environment-conditional security checks.
2. Add storage-backed nonce uniqueness and integrity guarantees.
3. Tighten canonical encodings, discriminants, and comparison rules.
4. Tie amount limits directly to proof-width and protocol policy.
5. Replace ambiguous config parsing and weak seed heuristics with fail-closed
   validation.

📌 Phase 3, cleanup:

1. Replace unstructured warnings and debug leaks.
2. Remove dead or test-only production symbols.
3. Clarify docs and APIs around blinding, ownerless assets, and metadata hash
   naming.
4. Finish or remove incomplete state and operational scaffolding.

### [F-09.1] Concrete Target Architecture

📌 The fused reports support a concrete target state rather than only a generic
roadmap. The minimum production-ready design is:

- Genesis consensus is fail-closed on protected networks and anchored to one
  canonical genesis-state hash per network.
- Asset-definition identity is derived in exactly one place from canonical,
  framed fields and cannot be overridden by config, wire, or import payloads.
- Registry snapshots are content-addressed from the full ordered definition
  payload, not from definition IDs alone.
- `owner_signature` is treated as an authority attestation over canonical asset
  state, not as a standalone proof of commitment-opening ownership.
- Stealth tuple integrity is enforced in two layers: core binds the tuple into
  canonical signed state, and wallet-side flows recompute stealth helpers from
  recipient secrets before accepting or spending the asset.
- Untrusted wire paths carry only public or explicitly declassified state;
  secret-bearing material stays in trusted internal types only.
- Fee validation is completed at the tx statement layer by binding fee amount
  and native asset identity into the signed or proved transaction digest.

### [F-09.2] Codebase-Reuse-First Solution Path

📌 The existing workspace already contains most of the required building blocks.
No new crate is required to close the main blockers.

| Blocker | Reusable in-repo building blocks | Required change |
| --- | --- | --- |
| Fail-open genesis consensus | `GenesisStateHashDomain`, chain-specific genesis asset-id domains, `DomainHasher` aliases in `hashing.rs` | Populate expected hashes for protected nets and reject missing values |
| Weak definition identity | `AssetIdHasher`, framed hashing helpers, existing genesis asset-id derivation pattern | Replace ad hoc or test-domain derivation with one canonical derivation function |
| Registry hash covers only IDs | `RegistryHashDomain`, `frame_bytes` / `frame_str` / `frame_u32_le` / `frame_u64_le`, `z00z_storage::snapshot::{PrepSnapshot, PrepSnapshotId}` pattern | Derive registry snapshot id from canonical full-definition payload |
| Partial ownership binding | `KernelSignature`, `OwnerSignatureDomain`, current `to_owner_message()` canonical message | Expand signed payload and narrow verifier semantics to trusted authority checks |
| Partial stealth binding | `compute_owner_tag`, `derive_leaf_ad`, `compute_tag16`, `derive_s_out` | Bind stealth fields in core and recompute them in wallet-side acceptance flows |
| Unsafe wire boundaries | Existing internal/public type split patterns across storage and wallet crates | Split trusted secret-bearing types from untrusted public wire types |
| Fee binding gap | Existing framed digest patterns and tx-style framing in simulator flows | Bind fee amount plus canonical native asset id into tx digest or proof statement |

📌 Optional crates.io additions are second-tier hardening only:

- `subtle` can help if the project wants stricter constant-time comparisons in
  protocol-sensitive equality paths.
- `secrecy` is optional if the project prefers an external secret wrapper, but
  blocker resolution can already use the existing `Hidden<T>` direction noted by
  the source audits.

### [F-09.3] Concrete Fix for Genesis Anchoring

📌 The fail-open genesis problem has a direct in-repo fix path:

1. Generate and freeze one canonical expected state hash per protected network.
2. Change `verify_genesis_consensus()` so `None` is accepted only for dev-only
  flows and is an error for mainnet and testnet.
3. Extend the genesis-state preimage from the current narrow asset projection to
  a canonical framed tuple that includes every consensus-relevant asset field
  and the canonical registry-definition fingerprint used by that genesis.
4. Reuse existing chain-specific domains for derivation and keep the final state
  hash under `GenesisStateHashDomain`.
5. Make network parsing fail closed so configuration mistakes cannot silently
  route into Devnet behavior.

📌 The intended result is simple: two nodes with different genesis assets,
definitions, flags, or protected-network configuration must not be able to
reach the same local acceptance decision.

### [F-09.4] Concrete Fix for Definition and Registry Integrity

📌 The solution is to centralize definition identity and then make registry
snapshots content-addressed.

1. Introduce one canonical `derive_definition_id(...)` path that uses framed
  fields and a production domain, not `TestAssetIdDomain`.
2. Require every constructor and import boundary to call that function:
  YAML/config loading, `DefinitionWire` deserialization, runtime constructors,
  and any snapshot rehydration path.
3. Reject supplied IDs when they do not match the derived canonical value.
4. Replace `RegistryVersion::compute_hash(def_ids)` with a canonical snapshot id
  derived from the ordered full payload of each definition, including all
  fields that affect asset semantics.
5. Follow the already-existing `z00z_storage` snapshot model:
  versioned schema tag, content-derived external id, load-then-rederive
  validation, and mismatch rejection on reload.

📌 This is stronger than merely hashing more bytes. It removes constructor drift,
stops ID forgery by configuration, and makes registry synchronization verifiable
as content rather than as a loose list of identifiers.

### [F-09.5] Concrete Fix for Ownership and Stealth Semantics

📌 The fused reports point to one root rule: do not let `owner_signature`
overclaim what it proves.

1. Keep Schnorr signing through the existing `KernelSignature` path, but expand
  the canonical owner message so it covers all authoritative fields that may be
  attacker-mutated in transit.
2. Add the stealth tuple bytes or their canonical fingerprints to the signed
  state: `r_pub`, `owner_tag`, `tag16`, `leaf_ad_id`, and the authenticated
  ciphertext package reference.
3. Make verifier semantics explicit:
  `owner_signature` proves that the declared authority signed this asset state;
  it does not by itself prove knowledge of the commitment opening.
4. Accept the signature only when `owner_pub` is anchored by trusted state,
  policy, or a higher-layer transaction rule.
5. Reuse wallet-side helpers for recipient validation and spend preparation:
  `derive_s_out`, `compute_tag16`, `compute_owner_tag`, and `derive_leaf_ad`
  should be recomputed off-chain wherever the receiver actually possesses the
  necessary view material.

📌 This split resolves the current semantic confusion cleanly:

- Core becomes responsible for binding and tamper detection.
- Wallet or recipient flows become responsible for secret-dependent stealth
  confirmation.
- Tx-layer rules remain responsible for proving spend authority over committed
  value.

### [F-09.6] Concrete Fix for Wire Boundaries and Fee Binding

📌 The wire/import issues should be fixed by redesigning the type boundary, not
by adding more ad hoc field filters.

1. Split public asset transport from trusted secret-bearing transport.
2. Remove `secret` from every untrusted deserialization path.
3. Remove plaintext `amount` from public confidential-asset wire types unless
  the protocol explicitly declares that representation as non-confidential.
4. Preserve or reject state flags such as frozen, burned, and slashed at the
  DTO boundary; never silently drop them during rehydration.
5. Keep any remaining trusted secret carrier behind an internal type and hide it
  by policy rather than by naming convention.

📌 Fee binding needs one cross-crate completion step:

1. Reuse the existing framed-digest pattern already present in transaction-style
  simulator flows.
2. Bind fee amount, fee asset id, and any gas or policy fields into the signed
  transaction digest or proof statement.
3. Reject fee payment unless both `AssetClass::Coin` and the canonical native
  asset id match.

📌 This means the fee blocker is not “needs a new crypto library”; it is “needs
tx-layer completion using the same framing and domain-separation discipline the
workspace already uses elsewhere.”

## [F-10] Consolidated Test Plan

📌 The fused reports converge on this minimum test set:

- Genesis consensus must fail if expected hashes are missing or mismatched.
- Registry snapshot integrity must break on any definition-payload mutation, not
  just ID changes.
- Definition IDs derived from config, wire, and runtime constructors must match
  the same canonical value or be rejected.
- Ownership verification must reject attacker-controlled key substitution.
- Stealth-field tampering must break the canonical signed state, and wallet-side
  stealth recomputation must reject invalid `tag16`, `owner_tag`, or
  `leaf_ad_id`.
- Untrusted wire import must reject or strip `secret` and must not leak
  plaintext-sensitive fields.
- Zero-nonce, missing-range-proof, and clock-failure paths must be exercised in
  the same configuration that production code uses.
- Amount limits must be tested against the configured proof width.
- Nonce-counter rollback or replay must be detectable.
- Fee validation must reject `Coin`-class assets whose id is not the canonical
  native fee asset.

📌 Recommended follow-up tests that appeared in at least one report:

- Wycheproof-style negative tests for signature parsing and canonicality.
- Property tests for nonce collision resistance and deterministic genesis.
- Fuzzing around `AssetPackPlain`, wire decoding, and malformed asset payloads.

## [F-11] Confidence and Release Decision

📌 Confidence is highest on the fused statements about genesis-consensus
fail-open behavior, ownership-binding incompleteness, unsafe wire boundaries,
and nonce-fallback problems because those themes either appear in multiple
reports or are directly tied to concrete code paths named by several sources.

📌 Confidence is medium on fee-binding, scalar-canonicality, and some runtime
snapshot-path assumptions because those claims are either single-source or rely
on code that some reports did not inspect.

🚨 Final fused release decision: `BLOCKED`.

📌 The source reports disagree on some severity labels, but not on the broader
engineering conclusion: the crate has strong primitive choices and useful
defensive structure, yet still needs integrity-boundary and ownership-semantics
work before it can be treated as production-ready.

## [F-12] Fusion Notes

📌 This fused document intentionally keeps stronger formulations where at least
one report contributed unique constraints, caveats, or examples, and it keeps
conflicts explicit instead of silently selecting the most optimistic audit.

📌 Full traceability, section inventory, provision coverage, deduplication
decisions, and the conflict register are recorded in `FUSION.audit.md`.
