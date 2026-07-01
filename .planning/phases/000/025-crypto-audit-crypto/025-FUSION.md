# Fused Crypto Audit: z00z_crypto

This document fuses five independent crypto-audit writeups for `crates/z00z_crypto`
into one topic-organized canonical audit. It preserves the strongest shared findings,
records source disagreements explicitly, and keeps source-specific summary artifacts
available for traceability.

## D01 Executive Verdict

`Blocked for production` is the canonical fused verdict.

That verdict is the strongest defensible synthesis across the five sources because:

- All five audits identify the `claim` proof and authority-signature paths as
  placeholder hash constructions rather than real proof/signature systems.
- All five audits flag `ZkPack` as a problematic custom or insufficiently bounded
  cryptographic path, even though they disagree on whether its primary issue is
  construction-level, encoding-level, or lifecycle-level.
- Multiple audits independently flag fail-open scalar derivation or MAC behavior.

The fused decision is conditional in one important way: if the entire `claim`
subsystem and the custom `ZkPack` path are conclusively gated out of production
reachability, the remaining non-claim crypto surface is materially stronger and
some source documents would downgrade the overall release risk to `salvageable`
or even `execution-ready with conditions`.

## D02 Scope and Reviewed Surface

The fused scope is the same across all five source audits:

- Review target: `crates/z00z_crypto/src/**/*.rs`
- Exclusion: `crates/z00z_crypto/tari/**`
- Review mode: source-code cryptographic audit / implementation review

The reviewed surface repeatedly includes the same core modules:

- `lib.rs`
- `backend.rs`
- `backend_tari.rs`
- `error.rs`
- `types.rs`
- `hash.rs`
- `domains.rs`
- `kdf.rs`
- `kdf_domains.rs`
- `aead.rs`
- `zkpack.rs`
- `commitments.rs`
- `range_proofs.rs`
- `ecdh.rs`
- `ecdh_stealth.rs`
- `validation.rs`
- `secret.rs`
- `hidden.rs`
- `claim/mod.rs`
- `claim/proof.rs`
- `claim/prover.rs`
- `claim/statement.rs`
- `claim/verifier.rs`

Two source-specific scope notes are preserved:

- One source carried blog-style front matter and author metadata; that metadata is
  retained in D17 for traceability only.
- Two sources added appendix-style file inventories and one source added a dependency
  graph; those artifacts are normalized into this scope section and D17.

## D03 Security Goals

Across the five audits, the intended security goals are consistent:

- Confidential transaction amounts via Pedersen commitments and Bulletproofs+
  range proofs.
- Stealth-address privacy via Ristretto-based ECDH and domain-separated key derivation.
- Authenticated encryption for transport and pack payloads.
- Domain separation across hash, KDF, and transcript contexts.
- Strong secret lifecycle handling, including zeroization and reduced accidental leakage.
- Claim integrity and authority binding for genesis-claim workflows.

The fused conclusion is that these goals are only partially achieved today. The
non-claim primitives align well with their intended goals, but the `claim`
subsystem does not currently meet authenticity or proof-of-knowledge expectations.

## D04 Threat Model

The shared threat model covers:

- Passive observers correlating commitments, tags, and stealth-address outputs.
- Active attackers tampering with ciphertexts, envelopes, proofs, and public statements.
- Malicious provers or claimants attempting to forge proofs or authority artifacts.
- DoS attackers supplying oversized or malformed encodings, proofs, AAD, or points.
- Side-channel attackers exploiting non-constant-time comparisons or excess secret copies.

The fused trust boundary is also consistent across sources:

- Tari-backed low-level primitives are treated as the trusted backend.
- The public `z00z_crypto` facade is expected to be fail-closed and protocol-safe.
- Downstream crates should not be forced to distinguish placeholder or experimental
  crypto from production-safe crypto only by comments or convention.

## D05 Claim Subsystem Failures

This is the strongest and most consistent production blocker.

All five audits identify two central problems:

- `GenesisClaimProof` is currently derived from statement hashing rather than from
  a real proof system or a witness-bound proof-of-knowledge.
- `ClaimAuthoritySig` is currently derived from public statement material rather
  than from a secret signing key and verifier-held public key.

The strongest formulation preserved from the source set is:

- Any actor who knows the public statement can recreate the current proof artifact.
- Any actor who knows the statement hash can recreate the current authority artifact.
- If these paths are reachable in production or trusted by downstream callers,
  claim authorization is forgeable.

The fused audit also preserves a secondary but still important point:

- The current verifier uses non-constant-time equality for some claim-digest and
  signature-style comparisons. That timing issue is secondary today because the
  underlying constructions are already placeholders, but it still must be corrected
  before any real claim proof or real claim signature is deployed.

Canonical remediation:

- Replace the proof placeholder with a real proof-of-knowledge or real ZK proof.
- Replace the authority placeholder with a real asymmetric signature.
- Remove or gate these APIs from the stable public facade until they are real.

## D06 ZkPack and Envelope Design Risks

Every source flags `ZkPack`, but they emphasize different failure modes. The fused
section keeps all of them:

- Construction risk: several audits classify `ZkPack` as a hand-rolled authenticated
  encryption scheme or custom stream-cipher-plus-tag construction without a formal
  security proof.
- Envelope risk: one audit flags the separate storage of ciphertext and authentication
  tag as an architectural hazard because it diverges from the crate's standard opaque
  envelope model.
- Encoding risk: one audit flags `serde` exposure and opaque version-mismatch handling
  around `ZkPackEncrypted` as consensus and diagnostics hazards.
- Visibility risk: one audit explicitly warns that the standard XChaCha20-Poly1305
  path and the custom `ZkPack` path coexist in a way that invites wrong-surface use.

Canonical fused guidance:

- Prefer the standard XChaCha20-Poly1305 path unless there is a hard and documented
  requirement for a circuit-friendly custom construction.
- If `ZkPack` must remain, treat it as experimental until its construction,
  provenance, canonical wire format, and authentication boundary are explicitly
  proven or externally audited.
- Keep tag handling opaque at the API boundary and use explicit version mismatch errors.

## D07 Fail-Open Scalar, MAC, and RNG Failure Paths

Multiple audits independently converge on a fail-closed design requirement that the
current code does not always meet.

The shared issues are:

- `h2scalar_zk` / `hash_to_scalar` style helpers may silently fall back to
  `Z00ZScalar::one()`.
- `Z00ZScalar::from_hash` may silently fall back to `one()`.
- `hmac_sha256` may silently return `[0u8; 32]` on key-construction failure.
- One audit additionally flags an infinite-loop fallback in `Z00ZScalar::random`
  as a liveness failure if RNG behavior degrades unexpectedly.

The strongest fused reading is:

- A cryptographic derivation failure must halt the operation.
- Replacing a failure with a known constant scalar or known constant tag is not
  an acceptable production behavior.

Canonical remediation:

- Convert these APIs to explicit `Result`-returning fail-closed interfaces.
- Remove constant fallback values.
- Remove infinite-loop fallback behavior from random scalar generation.

## D08 Stealth Addressing and View-Tag Gaps

The source set agrees that the ECDH foundation is structurally solid but incomplete
at the privacy-optimization layer.

Preserved points:

- Identity-point rejection and zero-scalar rejection are broadly praised.
- The byte-oriented compatibility wrapper should still validate points explicitly as
  defense in depth.
- A meaningful view-tag optimization appears to be missing from the active scan path,
  even though `Tag16Domain` or similar domain material already exists in the codebase.

The fused conclusion is:

- The crate has a sound base stealth-address construction.
- It still lacks the short-tag filtering expected for large-scale wallet scanning,
  leaving both privacy ergonomics and scan performance weaker than they need to be.

## D09 Range-Proof and Context-Binding Gaps

Only part of the source set raises this point, but it is concrete enough to preserve.

The fused statement is:

- The range-proof layer appears sound with respect to Bulletproofs+ verification,
  but the proof statement itself does not obviously bind `asset_id`, `chain_id`,
  or equivalent asset context inside the proof object.
- If that contextual binding is delegated to caller-layer blinding-factor derivation,
  transaction composition, or AAD, that contract must be explicit.

This is retained as a medium-confidence design gap rather than a proven exploit.

## D10 Domain Separation and Hash Framing

The audits strongly agree that domain separation is one of the best aspects of the crate.

Positive consensus:

- The typed `hash_domain!` registry is extensive and disciplined.
- HKDF info labels are distinct and purposeful.
- Domain and label framing for Blake2b / SHA-256 style hashing is intentional.

Caveats that must not be lost:

- Two coexisting domain-separation systems remain in use: typed `hash_domain!`
  domains and custom `dst()` framing.
- Some labels are too short or insufficiently namespaced.
- One audit flags the need for a frozen DST test vector so refactors cannot silently
  alter consensus-relevant framing.
- One audit flags potential Poseidon2 absorption-mode compatibility questions across
  native and circuit implementations.

Canonical guidance:

- Preserve the current strong domain-separation discipline.
- Reduce ambiguity between the two systems.
- Freeze framing behavior with explicit vectors or tests.

## D11 Additional API, Encoding, and Ergonomic Risks

The source set also records a series of lower-severity but still useful hardening items:

- `SecretBytes::dangerous_clone()` should remain auditable because it increases secret-copy risk.
- `SecretBytes32::into_inner()` transfers zeroization responsibility to the caller.
- `Z00ZScalar::reveal()` may expose backend details too widely.
- `Z00ZRistrettoPoint` debug formatting may leak more public traceability than desired.
- `ZkPackEncrypted` serde exposure, version handling, and opaque wire-format behavior
  need tighter boundaries.
- `ClaimVerifyReport::owner_bind_checked` currently overstates what is actually verified.
- Some error aggregation choices reduce diagnosability even when they do not change safety.
- One audit notes a heap allocation in a small hot-path nonce derivation.

These are not the canonical release blocker, but they are real hardening work.

## D12 Positive Security Properties

The fused document preserves the substantial positive overlap across the audits:

- The standard XChaCha20-Poly1305 path is widely assessed as sound.
- Tari-backed commitments and Bulletproofs+ integration are treated as the strongest
  part of the crate.
- Identity-point rejection and zero-scalar rejection are consistent strengths.
- Zeroization discipline and constant-time helper usage are repeatedly praised.
- AEAD, AAD, batch, and proof-size limits provide credible DoS resistance.
- Error strings generally avoid sensitive data leakage.
- Canonical byte encodings and explicit validation checks are better than average.

This positive overlap is the main reason the fused verdict is conditional rather than
uniformly catastrophic: outside the claim placeholders and custom-path concerns,
the crate contains a lot of sound engineering.

## D13 Resolved Ambiguities and Remaining Blockers

The post-fusion codebase check resolves several of the earlier ambiguities.

Resolved points:

- The `claim` subsystem is not merely dormant scaffolding. It is production-reachable
  through wallet and simulator flows, which means the placeholder proof and placeholder
  authority signature must be treated as active boundary flaws rather than hypothetical ones.
- A production-grade AEAD path already exists in the wallet-side `ZkPack` facade,
  separate from the custom `z00z_crypto::aead_zkpack` path. The problem is therefore
  boundary selection and export policy, not absence of a standard AEAD implementation.
- View-tag machinery is not missing from the repository as a whole. `tag16` and `leaf_ad`
  already exist in wallet code, but they are not yet elevated into one crypto-owned,
  protocol-canonical API.

The remaining true blockers are narrower and more concrete:

- `claim_v2` cannot be considered complete until there is an authoritative source root,
  because the current claim statement path still uses a transitional zero root in at least
  one reachable flow.
- The repository does not yet expose a storage-owned inclusion-proof API for the same
  authoritative root that claim verification is supposed to trust.
- The current codebase does not prove whether asset / chain binding is enforced inside the
  range-proof object itself or only at the outer transaction / statement layer.
- Poseidon2 native-versus-circuit compatibility still needs explicit vectors or circuit-side
  confirmation before any stronger claim is made about that path.

Those remaining blockers justify the continued conditional blocked verdict, but they do not
prevent the fused document from specifying an execution-ready remediation architecture.

## D14 Canonical Remediation Architecture

This section upgrades the earlier priority list into a concrete solution path.

### D14.1 Canonical target architecture

The recommended target state is:

- `claim_v1` becomes explicitly non-production. It must be feature-gated, hidden from the
  default public surface, or moved to test/simulator-only reachability.
- `claim_v2` becomes the only production claim path. It consists of three distinct layers:
  canonical statement bytes, a real authority signature over those bytes, and a versioned
  source-proof container proving that the claim references an authoritative source allocation.
- The authority layer should reuse the existing Tari-backed Ristretto Schnorr stack already
  exported by `z00z_crypto`, rather than introducing a second signature ecosystem.
- The source-proof layer should not be a generic “proof blob”. It should be a versioned enum
  such as `ClaimSourceProof`, with variants like `MerkleInclusionV1` or
  `GenesisAllocationRecordV1`, so the verifier knows exactly which proof system and root policy
  is being checked.
- The existing owner-attestation layer in wallets remains useful, but it is not a replacement
  for authority authorization or source-membership proof.
- `zkpack_v1` production use should route to the already existing wallet-side
  ChaCha20-Poly1305 facade. The custom `z00z_crypto::aead_zkpack` path should remain only as an
  explicitly experimental interface until it has a separate proof or audit basis.
- `tag16` and `leaf_ad` should move into one crypto-owned canonical module so that wallets,
  simulator, and future nodes all consume the same formulas and vectors.
- Bulletproofs+ should remain the range-proof system. Instead of replacing it, the outer
  transaction and authority-signature layers should bind an explicit `range_ctx_hash` covering
  proof bytes, commitment, `asset_id`, `chain_id`, amount policy, and version fields.

### D14.2 Codebase reuse matrix

The repository already contains most of the building blocks needed for this plan:

- Real authority signatures: `z00z_crypto` already exports `Z00ZSchnorrSignature`, and wallet
  key flows already sign and verify identity material. The fix is therefore message framing and
  trust distribution, not primitive selection.
- Production AEAD: `z00z_crypto::aead` already exposes the standard XChaCha20-Poly1305 path,
  and wallet code already implements a deterministic ChaCha20-Poly1305 `ZkPack` facade. No new
  AEAD crate is required.
- View tags and leaf binding: wallet code already contains `compute_tag16`, `compute_leaf_ad`,
  and a scanner cache that uses them. The gap is ownership and protocol-level consolidation.
- Fail-closed error model: `CryptoError` and other typed error surfaces already exist, so the
  remaining work is converting lossy helper APIs to `Result` and cleaning up their call sites.
- Domain separation: typed `hash_domain!` registries already exist. The remaining work is to
  stop drift between typed domains and manual `dst()` framing, not to invent a new policy.

### D14.3 Crates.io alternatives and verdict

External crates were considered only where the existing codebase was insufficient.

- New signature stacks such as `ed25519-dalek`, `schnorrkel`, or `k256` are not recommended.
  They would introduce a second curve and second signing ecosystem without solving any current
  blocker better than the already present Tari Ristretto Schnorr stack.
- Alternative AEAD crates such as `aes-gcm-siv` are not required. The codebase already has a
  standard AEAD implementation and an acceptable deterministic-facade pattern.
- Generic Merkle crates are not the right immediate answer for claim proofs, because the real
  blocker is not tree hashing alone but agreement on the authoritative root and the storage/API
  boundary that owns it.
- A second ZK proof stack such as `bulletproofs`, `ark-groth16`, or `halo2_proofs` should not be
  pulled in for the current remediation. It adds major complexity and still does not solve the
  authoritative-source-root problem by itself.

Canonical verdict on dependencies:

- Use the existing workspace for signatures, AEAD, typed errors, domain separation, and most
  wallet-stealth logic.
- Do not add a new crypto crate unless the product later requires a genuinely hidden-witness
  claim proof that cannot be represented as an authenticated source-membership proof.

### D14.4 Blocked decisions and required evidence

The following items still require explicit evidence before the remediation can be called complete:

- An authoritative source root for claims: where it is stored, who publishes it, how it is
  versioned, and how verifiers learn to trust it.
- A storage-owned inclusion-proof API that proves membership in that exact authoritative source.
- A final decision on whether production claim proofs require a true zero-knowledge witness proof
  or only a source-membership proof plus authority signature.
- Frozen compatibility vectors for any Poseidon2 path that is expected to match circuit behavior.

### D14.5 Ordered implementation plan

Phase 0:

- Immediately gate `claim_v1` and remove placeholder claim re-exports from the default public
  production surface.
- Mark `z00z_crypto::aead_zkpack` experimental and make the wallet AEAD facade the only blessed
  production pack path.

Phase 1:

- Introduce `ClaimAuthoritySigV2` as a real Schnorr signature over canonical framed statement bytes.
- Define one canonical statement digest that includes `chain_id`, root version, source-proof
  version, transaction version, and any range-binding context.

Phase 2:

- Replace `GenesisClaimProof` with `ClaimSourceProof` and define concrete accepted variants.
- Reject transitional zero-root statements in production verification.
- Add explicit verifier errors for unsupported proof version, unsupported root version, and
  source-proof mismatch.

Phase 3:

- Promote `compute_tag16` and `compute_leaf_ad` into `z00z_crypto` as canonical protocol APIs.
- Convert fail-open scalar, HMAC, and random-helper paths to typed `Result`-returning APIs.
- Remove or deprecate lossy helper exports from the stable facade.

Phase 4:

- Add `range_ctx_hash` to the transaction / claim digest path so range proofs are bound to
  `asset_id`, `chain_id`, versioning, and policy fields even if the proof object remains unchanged.
- Consolidate domain-separation policy so typed domains are authoritative and manual `dst()`
  framing is either frozen and narrowly scoped or migrated away.

This plan is sufficient to describe not only what is wrong but how the repository can actually
get to a production-safe state using mostly existing code.

## D15 Test and Validation Plan

The merged test plan keeps the strongest recurring asks from the five audits:

- Add claim-forgery, replay, and authority-authentication regression tests.
- Add fail-open regression tests for scalar derivation, scalar-from-hash, HMAC,
  and random-scalar generation.
- Add view-tag filter tests and large-wallet scan tests.
- Add malformed-envelope and malformed-proof fuzzing.
- Add Wycheproof or standards-vector coverage where practical.
- Add cross-implementation and framing-vector tests for Poseidon / DST-sensitive paths.
- Add explicit version-mismatch and canonical-serialization tests for `ZkPack`-adjacent types.

The minimum additional validation artifacts required by the remediation architecture are:

- A regression proving that placeholder claim APIs cannot be called from the default production
  surface.
- Golden vectors for canonical claim statement framing and `ClaimAuthoritySigV2` verification.
- Golden vectors for `tag16`, `leaf_ad`, and deterministic wallet `ZkPack` encryption.
- Rejection tests for zero-root production claims.
- Rejection tests for unsupported root versions, unsupported source-proof variants, and
  mismatched authoritative roots.
- End-to-end tests proving that `range_ctx_hash` changes when any bound field changes.

Completion gate for this fused remediation plan:

- `claim_v1` is gated.
- `ClaimAuthoritySigV2` uses a real signature.
- `ClaimSourceProof` exists and verifies against an authoritative root.
- Production `ZkPack` uses the standard AEAD path.
- Fail-open helpers are removed from the stable production facade.
- Canonical vectors exist for the newly frozen framing rules.

## D16 Confidence, Disagreements, and Release Decision

The source set disagrees materially about severity labels and final release posture.

The most important disagreement is:

- `crypto-audit-minimax27.md` treats the crate as execution-ready with conditions.
- `crypto-audit-gpt54.md` and `crypto-audit-sonet46.md` treat the claim placeholders as
  production-blocking flaws.
- `crypto-audit-glm5.md` and `crypto-audit-mimov2.md` sit in the middle: broadly
  salvageable, but still blocked until specific issues are resolved.

The fused document resolves that disagreement this way:

- If placeholder claim artifacts or custom `ZkPack` are production-reachable,
  the crate is blocked for production.
- If those paths are demonstrably gated out, the remaining crypto surface trends toward
  salvageable or conditionally releasable.

That conditional blocked verdict is the most conservative synthesis that still preserves
the positive consensus around the non-claim primitives.

## D17 Source Metadata and Summary Artifacts

The source set also contains artifacts that are not new security findings but still carry
traceability value:

- Blog-style front matter in the GPT-5.4 source document.
- File-inventory appendices.
- A dependency-graph appendix.
- A source-specific severity-summary appendix.
- Consolidated finding tables and source-local summary rollups.

Those artifacts are preserved for audit traceability in `FUSION.audit.md` and are not
discarded merely because their underlying findings were normalized into topic-based sections.
