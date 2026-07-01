# Crypto Audit Report — z00z_core

## Scope

📌 Auditor: GitHub Copilot (GPT-5.4)

📌 Date: 2026-03-26

📌 Target: `crates/z00z_core/**/*.rs` only

📌 Exclusions: non-Rust documents, other crates, and all vendor code under
`z00z_crypto/tari/`

📌 Coverage note: the crate contains 29 Rust source files under
`crates/z00z_core/src`; findings below focus on the cryptography-relevant
surfaces that define security semantics in this crate: asset commitments,
range-proof handling, owner-signature binding, stealth/wire boundaries,
registry snapshot integrity, and genesis determinism/consensus binding.

## Executive Verdict

🚨 Verdict: `Risky but salvageable`

🚨 This crate does not contain an S0 fund-theft primitive by itself, but it does
contain multiple S1 composition failures that break stated security goals under a
realistic attacker model. The most important issues are: weak registry snapshot
integrity binding, self-asserted owner signatures that are not cryptographically
bound to commitment ownership, and disabled genesis consensus hash enforcement.

🚨 Final decision: `Blocked` until the S1 items below are resolved or explicitly
re-scoped with named owners and external enforcement evidence.

## Input Classification

📌 Input type: implementation review of a Rust crate.

📌 Applied review layers: primitive selection review, composition review,
serialization/binding review, and implementation review.

## Security Goals Extracted

📌 The crate explicitly or implicitly aims to provide these properties:

| Goal | Status in `z00z_core` | Notes |
| --- | --- | --- |
| Commitment binding and amount hiding | Partial | Delegated to `z00z_crypto`, but composition is owned here |
| Range-proof sound acceptance | Partial | Verified at asset/genesis level, but boundary semantics matter |
| Owner authorization binding | Broken | Signature proves self-selected key control, not asset ownership |
| Snapshot integrity for validator-to-wallet state | Broken | Hash covers only IDs, not full policy payload |
| Deterministic genesis reproducibility | Present | Implemented via domain-separated derivations |
| Genesis consensus anchoring | Broken | Verification function currently fails open |
| Nonce uniqueness and replay resistance | Partial | Local derivation exists; global enforcement is outside this crate |
| Stealth payload boundary safety | Partial | Public DTO is safer than internal wire, but semantics are inconsistent |

## Threat Model Summary

📌 Adversaries considered in this audit:

- malicious validator or snapshot sender
- malicious wallet/client crafting inbound asset payloads
- network attacker replaying or substituting unauthenticated state
- malicious party re-signing asset state with arbitrary keys
- operator or integration code using the wrong API boundary

📌 Trust boundaries relevant to this crate:

- `z00z_core` trusts `z00z_crypto` for primitive correctness
- `z00z_core` itself owns the binding between primitives and protocol meaning
- registry snapshots cross a validator-to-wallet boundary
- public asset DTOs cross import/export boundaries
- genesis security depends on external consensus constants that are not enforced here

📌 Failure assumptions relevant to this crate:

- missing or malformed proofs/signatures can appear from untrusted input
- clock/time provider failure can happen
- snapshot transport may be tampered with unless cryptographically authenticated
- downstream consumers may mistake local state flags for transport-safe fields

## Critical And High Findings

### S1-1 Registry Snapshot Integrity Hash Covers Only Definition IDs

📌 Component: `crates/z00z_core/src/assets/snapshot.rs:74-92` and
`crates/z00z_core/src/assets/registry.rs:867-972`

| Field | Content |
| --- | --- |
| Severity | S1 |
| Problem | `RegistryVersion::compute_hash()` hashes only the sorted `[u8; 32]` definition IDs. `update_from_snapshot()` rebuilds full `AssetDefinition` records from wire data, but verifies integrity against the ID-only hash again. That means policy fields such as `decimals`, `nominal`, `crypto_version`, `policy_flags`, `domain_name`, and `metadata` can change without changing the snapshot hash, as long as the IDs stay the same. |
| Impact | A malicious validator, relay, or compromised channel can alter asset policy or metadata while still passing the snapshot integrity check. This breaks the documented goal of using version/hash to validate registry updates and prevent downgrade or corruption. Wallet behavior can diverge from validator intent without detection. |
| Fix | Hash a canonical serialization of every full `DefinitionWire` entry, in deterministic order, not just the IDs. Then authenticate the snapshot transport itself with a validator signature or other authenticated channel. |

📌 Confidence: High.

📌 Evidence that would change confidence: a second integrity mechanism elsewhere that
cryptographically authenticates the full `definitions` payload before this crate accepts
it.

### S1-2 Owner Signature Is Self-Asserted And Not Bound To Commitment Ownership

📌 Component: `crates/z00z_core/src/assets/assets.rs:865-893` and
`crates/z00z_core/src/assets/assets.rs:1532-1648`

| Field | Content |
| --- | --- |
| Severity | S1 |
| Problem | `Asset::new()` creates `owner_pub` from the commitment blinding and signs with the same secret, but the verifier side does not enforce this relation. `verify_owner_signature()` only checks that `owner_signature` is valid under the provided `owner_pub` over `to_owner_message()`. Any party can replace `owner_pub` with a new public key, sign the same asset message with a different secret, and still pass `verify_owner_signature()` and `verify_complete()`. |
| Impact | The crate's documented ownership story is stronger than what the code proves. If downstream logic treats `owner_signature` as authorization or proof of rightful ownership, an attacker can rebind ownership to an arbitrary key without proving knowledge of the commitment opening. This breaks a stated security goal under a realistic attacker model. |
| Fix | Do not treat plain Schnorr over `to_owner_message()` as proof of asset ownership. Either: (1) add an explicit proof of knowledge that binds the signing key to the commitment opening, or (2) require an externally anchored owner key from ledger state and verify equality to that expected key before accepting the signature. Update the docs to stop claiming ownership binding until that exists. |

📌 Confidence: High.

📌 Evidence that would change confidence: a downstream protocol proof showing that
`owner_pub` is independently anchored and cannot be attacker-chosen at verification time.

### S1-3 Genesis Consensus Hash Verification Is Effectively Disabled

📌 Component: `crates/z00z_core/src/genesis/validator.rs:939-981`

| Field | Content |
| --- | --- |
| Severity | S1 |
| Problem | `verify_genesis_consensus()` defines `MAINNET_GENESIS_STATE_HASH` and `TESTNET_GENESIS_STATE_HASH` as `None`. In that state, the function always returns `Ok(())`, including for mainnet and testnet. This is a fail-open implementation of a consensus-critical check. |
| Impact | Divergent genesis states can pass local verification, defeating the stated goal of preventing incompatible genesis generation and chain splits. A realistic deployment mistake or malicious packaging of genesis data would not be caught here. |
| Fix | Inject the expected mainnet/testnet hash from consensus parameters and fail closed whenever the network type is mainnet or testnet and the expected hash is missing or mismatched. Missing constants must be a startup error, not a silent skip. |

📌 Confidence: High.

📌 Evidence that would change confidence: proof that this function is never used in any
deployment path and that another mandatory verifier enforces the same check before node
startup.

## Medium Findings

### S2-1 Public Asset DTO Drops Frozen And Slashed State On Rehydration

📌 Component: `crates/z00z_core/src/assets/wire_pkg.rs:17-18` and
`crates/z00z_core/src/assets/wire_pkg.rs:275-298`

| Field | Content |
| --- | --- |
| Severity | S2 |
| Problem | `AssetPkgWire` explicitly excludes `is_frozen`, `is_slashed`, and `secret`. When it is converted back via `to_wire()`, the code hard-sets `is_frozen: false`, `is_slashed: false`, and `secret: None`. At the same time, the owner-signature message in `Asset` includes `is_frozen` and `is_slashed` as signed state. |
| Impact | Crossing this public boundary can silently clear punitive or lock state. If any external workflow accepts `AssetPkgWire` for import, verify, or transaction packaging, it can downgrade asset state semantics. At minimum this creates signature/state inconsistency; at worst it becomes a policy bypass if consumers rely on the stripped flags. |
| Fix | Either include these flags in the canonical external DTO, or explicitly reject any asset whose state cannot be faithfully represented by `AssetPkgWire`. Do not silently coerce signed state back to `false`. |

📌 Confidence: Medium-high.

📌 Evidence that would change confidence: proof that `AssetPkgWire` is never accepted on
security-sensitive ingest paths and is used only for display or benign export.

### S2-2 Genesis Seed Entropy Threshold Is Impossible For A 32-Byte Sample

📌 Component: `crates/z00z_core/src/genesis/validator.rs:462-531`

| Field | Content |
| --- | --- |
| Severity | S2 |
| Problem | `validate_genesis_seed()` computes empirical Shannon entropy from exactly 32 bytes, then requires at least 200 bits for production. For a 32-sample byte string, the maximum empirical total Shannon entropy is 160 bits when all 32 bytes are distinct. That makes the current production threshold unreachable for any 32-byte seed. |
| Impact | The production validation rule cannot succeed as written. That blocks valid genesis generation or incentivizes bypassing the validator. In security terms, it weakens trust in the seed-validation path because operators must eventually route around it. |
| Fix | Remove sample-based Shannon estimation for fixed 32-byte seeds. Replace it with provenance requirements plus blacklist checks for obvious bad seeds, or validate entropy at the RNG/source level before the 32-byte seed is materialized. |

📌 Confidence: High.

📌 Evidence that would change confidence: none, unless the input length changes to a much
larger sampled dataset and the threshold logic is rewritten accordingly.

### S2-3 Production Registry Config Uses A Test-Only Asset-ID Domain

📌 Component: `crates/z00z_core/src/assets/assets_config.rs:56` and
`crates/z00z_core/src/assets/assets_config.rs:201-219`, reached from
`crates/z00z_core/src/assets/registry.rs:789-805`

| Field | Content |
| --- | --- |
| Severity | S2 |
| Problem | `compute_asset_id_from_config()` uses `TestAssetIdDomain` with label `test_asset`, but the function is used by the real YAML registry loading path. That places runtime-config asset IDs into a test namespace instead of a dedicated production domain. |
| Impact | Domain confusion increases the risk of cross-environment collisions, inconsistent identity derivation across genesis/runtime paths, and future migration mistakes. This is not an immediate primitive break, but it is a protocol-identity binding weakness. |
| Fix | Introduce a dedicated production asset-config domain and migrate the registry loader to it. Keep the current test domain only for tests and explicitly named fixtures. |

📌 Confidence: High.

📌 Evidence that would change confidence: proof that `load_from_config()` is test-only and
never used in wallet or validator runtime paths.

## Low Findings

### S3-1 Silent Time Fallbacks Hide Nonce-Derivation Failures

📌 Component: `crates/z00z_core/src/assets/nonce.rs:468-479` and
`crates/z00z_core/src/assets/nonce.rs:520-626`

| Field | Content |
| --- | --- |
| Severity | S3 |
| Problem | `get_timestamp_micros()`, `derive_nonce_simple()`, and `derive_nonce_minimal()` silently fall back to timestamp `0` on time-provider failure. Fallible variants exist, but the easy APIs hide the failure. |
| Impact | This is mostly an auditability and misuse-resistance issue. Counter- or RNG-derived uniqueness still exists, so the most likely outcome is hidden operational failure rather than direct nonce collision. |
| Fix | Use the fallible APIs in production paths and make the infallible helpers test-only or clearly marked as non-production convenience wrappers. |

📌 Confidence: High.

📌 Evidence that would change confidence: proof that all production callers already use the
fallible variants only.

## Additional Observations

### S4-1 Internal `AssetWire` Still Carries Secret Material

📌 Observation: `AssetWire` still includes `secret: Option<[u8; 32]>` and forwards it in
`to_asset()` in `crates/z00z_core/src/assets/wire.rs:128-197`, while the safer public DTO
filters it out in `AssetPkgWire`.

📌 This is not elevated above S4 because the code comments describe `AssetWire` as an
internal mutable transport type, but it is still a footgun: if external code uses the
wrong wire type, the secret-bearing path reappears.

## Open Ambiguities

📌 The following items remain outside what this crate alone proves:

- global nonce uniqueness enforcement across transactions and chain history
- nullifier uniqueness and spend-set enforcement
- authenticated transport for registry snapshots
- whether any downstream verifier anchors `owner_pub` externally before accepting
  `owner_signature`
- whether `AssetPkgWire` is accepted on spend-critical import paths or only on benign
  interchange paths

## Concrete Remediation Plan

📌 Minimum safe sequence:

1. Redesign registry snapshot integrity to hash canonical full definitions and add an
   authenticated sender binding.
2. Re-scope owner signatures so they are either clearly advisory or actually bound to
   commitment ownership with a proof the verifier can check.
3. Make genesis consensus verification fail closed on mainnet/testnet.
4. Decide whether frozen/slashed state is protocol state or wallet-local state. Then make
   the DTO boundary faithful to that decision.
5. Replace impossible seed-entropy validation with a sound source/provenance rule.
6. Remove the test domain from runtime asset-ID derivation.
7. Route production nonce creation through the fallible APIs only.

## Test Plan

📌 Required tests before sign-off:

- adversarial snapshot test: modify `policy_flags` or `crypto_version` while keeping the
  same IDs and confirm integrity rejection
- adversarial ownership test: take an existing asset, swap in an attacker key/signature,
  and confirm the verifier rejects it once ownership binding is fixed
- fail-closed genesis test: mainnet/testnet startup must error if expected state hash is
  missing
- DTO downgrade test: asset with `is_frozen=true` or `is_slashed=true` must not silently
  round-trip through the external package format
- seed validation tests for realistic 32-byte CSPRNG outputs
- domain-separation regression tests for runtime asset-ID derivation
- negative tests for time-provider failure in nonce derivation

## Confidence By Claim

📌 S1-1 snapshot-integrity finding: High confidence.

📌 S1-2 owner-signature-binding finding: High confidence on the local code claim; medium
confidence on end-to-end exploitability until downstream authorization paths are audited.

📌 S1-3 genesis fail-open finding: High confidence.

📌 S2-1 DTO state-loss finding: Medium-high confidence pending confirmation of import-path
usage.

📌 S2-2 entropy-threshold finding: High confidence.

📌 S2-3 test-domain misuse finding: High confidence.

📌 S3-1 nonce time-fallback finding: High confidence.

## Final Decision

🚨 `Blocked`.

🚨 Required owners for unblock:

- protocol owner: define real ownership-binding semantics for `owner_signature`
- wallet/validator boundary owner: redesign registry snapshot integrity and transport auth
- consensus owner: wire in mandatory genesis-state constants and fail closed
- asset-transport owner: decide whether frozen/slashed state is protocol-visible or local

🚨 Execution-ready status: not reached.
