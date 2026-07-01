---
post_title: "Crypto Audit: z00z_crypto GPT-5.4"
author1: "GitHub Copilot"
post_slug: "crypto-audit-z00z-crypto-gpt54"
microsoft_alias: "copilot"
featured_image: "none"
categories:
  - "engineering"
tags:
  - "crypto"
  - "audit"
  - "rust"
  - "z00z"
ai_note: "AI-assisted source-code cryptographic audit"
summary: "Deep source-only audit of crates/z00z_crypto Rust implementation excluding vendor Tari code."
post_date: "2026-03-26"
---

# Crypto Audit: z00z_crypto

## Executive Verdict

🚨 Fundamentally broken.

🚨 Input type: source-code audit of `crates/z00z_crypto/src/**/*.rs` only.
Scope excludes all documentation files and excludes `crates/z00z_crypto/tari/**`.

🚨 Final decision: Blocked.

🚨 Execution-ready status is denied by one S0 structural flaw and one S1
construction-level flaw that must be resolved before this crate can be trusted
as a production cryptographic boundary.

## Scope

📌 Reviewed Rust implementation surface:

- `src/lib.rs`
- `src/backend.rs`
- `src/backend_tari.rs`
- `src/error.rs`
- `src/range_proofs.rs`
- `src/commitments.rs`
- `src/kdf_domains.rs`
- `src/ecdh_stealth.rs`
- `src/types.rs`
- `src/secret.rs`
- `src/aead.rs`
- `src/domains.rs`
- `src/kdf.rs`
- `src/hash.rs`
- `src/hidden.rs`
- `src/ecdh.rs`
- `src/validation.rs`
- `src/zkpack.rs`
- `src/claim/mod.rs`
- `src/claim/proof.rs`
- `src/claim/prover.rs`
- `src/claim/statement.rs`
- `src/claim/verifier.rs`

📌 The audit focus was implementation reality, not comments, plans, or external
specifications. Any security claim below is grounded in Rust code paths,
exported APIs, and directly observable downstream usage from Rust call sites.

## Security Goals Extracted From Code

🎯 The crate appears to target these goals:

- Confidential amounts via Pedersen commitments and Bulletproofs+ wrappers.
- Integrity and authenticity for claim flows.
- Confidential transport via XChaCha20-Poly1305 and pack encryption helpers.
- Stealth-address privacy via Ristretto ECDH and domain-separated KDFs.
- Canonical encoding and invalid-point rejection for consensus-sensitive inputs.
- Zeroization and reduced secret leakage for scalar and byte wrappers.

🎯 These goals are only partially met. The strongest failures are in the claim
authorization path and the custom `zkpack` encryption design.

## Threat Model Summary

⚠️ The reviewed Rust implementation does not contain a single explicit,
internally consistent threat-model artifact. I therefore extracted the practical
threat model from code paths and exported APIs.

⚠️ Relevant adversaries for this crate are:

- A malicious claimant who can craft arbitrary claim statements and proof blobs.
- A chain observer who sees public commitments, proofs, metadata, and pack blobs.
- A malicious transaction producer who can tamper with serialized ciphertexts,
  claim payloads, and public statement fields.
- A malformed-input attacker supplying invalid points, zero scalars, oversized
  proofs, malformed envelopes, and forged authority artifacts.
- A future maintainer who may accidentally trigger silent crypto fallback paths.

⚠️ Trust boundaries implied by the code are:

- Tari-backed range proof and commitment primitives are the trusted low-level
  backend.
- `z00z_crypto` itself is intended to be the protocol-safe facade.
- Downstream crates trust exported `claim::*`, `aead::*`, and KDF surfaces as if
  they were production-safe primitives.

⚠️ Confidence in the extracted threat model is medium. Confidence would rise if
the crate enforced role binding, authority keys, and protocol-state ownership in
types rather than in comments.

## Critical And High Findings

| Severity | Component | Problem | Impact | Fix |
| --- | --- | --- | --- | --- |
| S0 | `src/claim/prover.rs::prove_genesis_claim`, `src/claim/verifier.rs::verify_genesis_claim`, `src/claim/proof.rs::GenesisClaimProof` | The exported “proof” is not a proof. `prove_genesis_claim()` ignores the witness and emits `PROOF_TAG` concatenated with `H(statement_hash)` only. The verifier recomputes the same digest and accepts it. No witness knowledge, no secret, no authority, and no hard problem are involved. | Any actor who knows the public statement can forge a valid claim proof. If downstream logic interprets this as authorization or proof-of-knowledge, unauthorized claim acceptance becomes possible. This matches proof forgery, not mere incompleteness. | Immediately remove or feature-gate this API from production builds. Replace it with a real proof system or rename it to a non-security term such as `statement_digest_artifact` and prevent downstream code from treating it as proof material. |
| S0 | `src/claim/proof.rs::ClaimAuthoritySig::from_statement`, `src/claim/verifier.rs::verify_claim_authority_sig`, `src/claim/mod.rs` | The exported “authority signature” is not a signature. It is `SIG_TAG` concatenated with `H(statement_hash)` with no secret signing key and no public-key verification step. Any caller can mint one locally. | Any actor can forge authority approval for arbitrary statements. This breaks authenticity and can directly subvert any claim, mint, or approval workflow that trusts `ClaimAuthoritySig`. | Replace the type with an actual signature primitive over canonical statement bytes, using an authority secret key and verifier-supplied authority public key. If this path is test scaffolding only, move it behind `cfg(test)` or an explicit `experimental-claim-placeholder` feature and remove it from public re-exports. |
| S1 | `src/aead.rs::zkpack::{seal_zkpack, open_zkpack}` | `zkpack` is home-grown authenticated encryption: deterministic hash-derived keystream plus custom hash tag over ciphertext and metadata. This is not a standard AEAD, has no proof in the codebase, and is exported as a reusable primitive. | Confidentiality and integrity claims depend on an unaudited composition. A design error here can produce plaintext leakage, malleability, replay confusion, or cross-context misuse even if the underlying hash is strong. This is explicitly a red-flag construction. | Replace `zkpack` with a standard AEAD, preferably `XChaCha20-Poly1305` with domain-bound AAD, or `AES-GCM-SIV` if nonce-misuse tolerance is required. If circuit-friendliness is the real reason for this path, mark it experimental, document the security model, and require external audit plus proof of construction before any production use. |

## Medium And Low Findings

| Severity | Component | Problem | Impact | Fix |
| --- | --- | --- | --- | --- |
| S2 | `src/kdf.rs::h2scalar_zk`, `src/hash.rs::h2scalar_zk`, `src/types.rs::Z00ZScalar::from_hash` | Multiple hash-to-scalar helpers fail open by mapping errors or zero results to `Z00ZScalar::one()`. This silently converts derivation failure into a known constant scalar. | If a future encoding, parameter, or backend issue triggers failure, derived blindings, hedged nonces, or derived keys may collapse to a predictable value instead of halting. This is a dangerous failure mode in any scalar derivation path. | Change these APIs to return `Result<Z00ZScalar, CryptoError>` and fail closed. Remove all fallback-to-one branches, including the current `unwrap_or_else` and `Err(_) => Self::one()` logic. |
| S2 | `src/claim/*` public surface as re-exported in `src/lib.rs` | Placeholder claim APIs are exported from the main crate root without an “experimental” boundary, despite code comments explicitly calling them placeholder artifacts. | Downstream crates can and already do consume them as trusted crypto surfaces. That magnifies the S0 issue from an isolated module bug into a boundary-design failure. | Remove public re-exports for placeholder claim APIs or place them behind a non-default feature that clearly labels them non-production. |
| S3 | `src/hash.rs::{blake2b_256_simple, blake2b_512_simple, sha256_simple}`, `src/hash.rs::dst` | The crate publicly exports no-domain-separation hash helpers and also contains a panic-on-NUL path in the DST builder used by convenience hash APIs. | This is primarily a misuse and DoS hazard. A consumer can accidentally bypass domain separation or crash a process by feeding unchecked domain/label input into convenience helpers. | Restrict simple hash helpers to test/internal visibility or mark them `#[doc(hidden)]` and add fallible `Result`-based validation instead of `panic!`. |
| S3 | `src/aead.rs::transport`, `src/aead.rs::zkpack` | Two pack-encryption surfaces coexist: a standard XChaCha20-Poly1305 transport path and a custom `zkpack` path. The naming is close enough to invite wrong-surface selection. | Operator and maintainer misuse becomes more likely, especially in downstream crates where both are accessible from the same crypto facade. | Collapse the public surface to one blessed pack-encryption API. Keep the other internal or explicitly experimental. |

## Supporting Evidence

🔎 Evidence for the S0 claim-proof forgery finding:

- `src/claim/prover.rs::prove_genesis_claim()` ignores `_witness` entirely.
- `src/claim/verifier.rs::proof_bytes()` constructs `PROOF_TAG || proof_digest(stmt_hash)`.
- `src/claim/verifier.rs::verify_genesis_claim()` only checks version, tag,
  length, and digest equality against the public statement hash.
- `src/claim/proof.rs::ClaimAuthoritySig::from_statement()` derives the
  “signature” from the public statement alone.

🔎 Evidence that the forged artifacts are trusted outside this module:

- `crates/z00z_wallets/src/core/tx/claim_tx.rs` verifies these blobs and maps
  them into claim acceptance errors as if they were real proof and authority
  objects.
- `crates/z00z_simulator/src/scenario_1/stage_3.rs` generates both artifacts in
  normal flow construction.

🔎 Evidence for the S1 custom-encryption finding:

- `src/aead.rs::zkpack::gen_keystream()` builds a hash-based XOR stream.
- `src/aead.rs::zkpack::compute_tag()` builds a separate custom MAC from
  `hash_zk`.
- `src/aead.rs::zkpack::{seal_zkpack, open_zkpack}` combine the two into a
  bespoke encryption scheme rather than a standard AEAD.

🔎 Evidence for fail-open scalar derivation:

- `src/kdf.rs::h2scalar_zk()` maps error to `Z00ZScalar::one()` and also maps
  zero output to `one()`.
- `src/hash.rs::h2scalar_zk()` does the same.
- `src/types.rs::Z00ZScalar::from_hash()` also falls back to `Self::one()`.

## Open Ambiguities

❓ The code does not resolve whether the `claim` module is temporary scaffolding
or intended for production authorization. The implementation says placeholder;
the public API and downstream usage say trusted primitive.

❓ The code does not explain whether `zkpack` exists for circuit compatibility,
legacy wire format, or performance. Each motive implies a different acceptable
security model.

❓ The code does not encode authority-key ownership, key rotation, or signer
identity for the claim path. Without that, authenticity cannot be reasoned
about at the type or verifier level.

❓ The code does not contain local validation artifacts for the Poseidon2
parameter set beyond library use. This is not a direct break, but it limits
confidence when Poseidon2 is used outside tightly specified circuit contexts.

## Concrete Fixes

🔧 Minimal safe fix sequence:

1. Remove the `claim` placeholder API from the default public surface.
2. Add a real authority signature scheme over canonical statement bytes.
3. Require the verifier to receive and validate the authority public key.
4. Replace `GenesisClaimWitness` placeholder flow with either:
   - a real proof system, or
   - a renamed non-security artifact that cannot be confused with proof.
5. Delete or internalize `zkpack` unless there is a hard requirement that a
   standard AEAD cannot satisfy.
6. Make all hash-to-scalar helpers fail closed with `Result`.
7. Remove public no-domain-separation helpers from the stable crypto facade.

🔧 Recommended concrete primitives:

- Authority signature: Ristretto Schnorr or another audited signature already
  available through the Tari-backed surface.
- Encrypted pack transport: XChaCha20-Poly1305 with canonical AAD including
  `chain_id`, `asset_id`, `serial_id`, `r_pub`, and format version.
- Scalar derivation: typed domain-separated `h2s_zk::<Domain>() -> Result<_>`
  only, with no deprecated fail-open wrappers.

## Implementation Guidance

⚙️ A safe architecture for this crate should look like this:

- Keep Tari-backed commitments and Bulletproofs+ verification as the only
  production proof backend.
- Treat `z00z_crypto` as a fail-closed facade: no silent fallback to constants,
  no placeholder artifacts exported as crypto primitives, and no panic-based
  validation for externally supplied values.
- Ensure every authenticity primitive has an explicit secret holder, public key,
  canonical message encoding, domain separation tag, and replay-bound context.
- Prefer one public encryption surface per use case. If the standard AEAD path
  is blessed, custom pack ciphers must not remain equally discoverable.

⚙️ Positive implementation notes from the reviewed code:

- `src/backend_tari.rs` correctly routes commitments and range proofs through
  Tari-backed primitives rather than custom proof math.
- `src/ecdh.rs` and `src/validation.rs` consistently reject identity points and
  malformed encodings on the untrusted-input boundary.
- `src/aead.rs::{seal, open}` uses standard XChaCha20-Poly1305 with enforced
  internal nonce generation and AAD size limits.
- `src/commitments.rs` rejects zero blinding factors before commitment creation.

## Test Plan

✅ Required validation before sign-off:

- Add a regression proving that a third party cannot mint a valid claim
  artifact without witness knowledge or authority secret key.
- Add negative tests showing that replay across different `chain_id`,
  `claim_id`, and `scenario_scope_hash` is rejected by the real signature layer.
- Add property tests for hash-to-scalar helpers proving they never return a
  constant fallback on internal failure paths.
- Fuzz all claim blob parsers, compressed-point parsers, and pack parsers.
- Run Wycheproof-style verification where applicable for standard primitives in
  the public surface.
- Add cross-implementation tests for XChaCha20-Poly1305 envelopes if the crate
  interoperates with other stacks.
- If any custom `zkpack` design remains, require a dedicated external audit,
  misuse tests, replay tests, metadata-binding tests, and a written security
  argument before production enablement.

## Confidence

📌 High confidence: the S0 claim finding is real. The implementation is plainly
digest-based and secret-free, and the affected API is exported and consumed by
other crates.

📌 High confidence: the fail-open scalar fallback finding is real. The code
returns `Z00ZScalar::one()` on failure in multiple helper paths.

📌 Medium-high confidence: the `zkpack` design is unsafe to ship as a generic
cryptographic primitive because it is a custom AE construction with no visible
proof or audit boundary. Confidence would become higher with confirmation that
this path is reachable in production flows.

📌 Medium confidence overall for the rest of the crate because the strongest
backend primitives look sound, but the crate boundary still exports dangerous
surfaces that can invalidate downstream security assumptions.

## Final Decision

🛑 Blocked:

- Owner: `z00z_crypto` maintainers must remove or replace placeholder claim
  proof and authority signature APIs.
- Owner: `z00z_crypto` maintainers must decide whether `zkpack` is experimental
  research code or a production primitive, then either remove it or redesign it
  around a standard AEAD.
- Owner: `z00z_crypto` maintainers must convert hash-to-scalar fallback helpers
  to fail-closed `Result` APIs.

🛑 The crate is not execution-ready as a production cryptographic boundary until
those blockers are resolved.
