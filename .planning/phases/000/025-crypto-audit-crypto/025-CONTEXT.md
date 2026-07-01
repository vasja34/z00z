<!-- markdownlint-disable MD001 MD022 MD032 MD033 -->

# Phase 025: crypto-audit-crypto - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning
**Source:** PRD Express Path (`.planning/phases/025-crypto-audit-crypto/025-FUSION.md`)

<domain>
## Phase Boundary

Turn the fused `z00z_crypto` audit into one execution-ready remediation phase that removes placeholder claim and custom-zkpack security boundaries from default production use, introduces authoritative `claim_v2` contracts, hardens fail-open helper paths, and consolidates crypto-owned stealth and range binding APIs across `z00z_crypto`, `z00z_storage`, `z00z_wallets`, and `z00z_simulator`.

</domain>

<decisions>
## Implementation Decisions

### Claim Surface

- Treat the current `claim_v1` proof and authority artifacts as non-production placeholder APIs.
- Introduce `claim_v2` around canonical statement bytes, a real Tari-backed Schnorr `ClaimAuthoritySigV2`, and a versioned `ClaimSourceProof` container.
- Reject zero-root or transitional-root claims in production verification and builder flows.
- Load claim authority trust from repository-owned state or config, not from attacker-controlled tx payloads.

### Source Proof Ownership

- Keep authoritative claim-source roots and inclusion proofs storage-owned.
- Extend storage proof and checkpoint surfaces with typed claim-source root or proof contracts instead of opaque blobs.
- Reuse existing checkpoint, snapshot, and asset proof primitives rather than introducing a second proof storage stack.

### ZkPack Boundary

- The blessed production `ZkPack` path is the wallet ChaCha20-Poly1305 facade in `z00z_wallets`.
- The custom `z00z_crypto::aead_zkpack` path must become experimental and non-default.
- `ZkPackEncrypted` wire handling must return explicit version or decode errors rather than `Option`-style ambiguity.

### Fail-Closed Derivation

- Convert scalar derivation, scalar-from-hash, HMAC initialization, and random-scalar helpers to fail-closed typed error paths.
- Remove constant fallback values such as `Z00ZScalar::one()` or `[0u8; 32]` from stable production helper behavior.
- Remove unbounded retry loops from production random-scalar generation.

### Stealth And Range Binding

- Promote `tag16` and `leaf_ad` formulas into one crypto-owned canonical API consumed by wallets and simulator code.
- Freeze vectors for claim statement framing, `tag16`, `leaf_ad`, deterministic wallet `ZkPack`, and DST or Poseidon-sensitive derivations.
- Add `range_ctx_hash` binding so `asset_id`, `chain_id`, versions, policy, commitment, and proof bytes are explicitly covered by outer claim or tx digests.

### Dependency Policy

- Reuse the existing Tari Ristretto Schnorr stack, wallet AEAD implementation, typed error surfaces, and storage proofs already present in the workspace.
- Do not add a new signature, AEAD, Merkle, or zk-proof dependency in this phase.

### the agent's Discretion

- Choose the exact module split for `claim_v2`, stealth-binding helpers, and source-proof request types as long as crate ownership stays consistent.
- Choose the smallest feature names and compatibility shims that satisfy the default-production gating goal.
- Decide whether frozen vectors live in existing test files or new focused regression files per crate.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Audit And Phase Inputs

- `.planning/phases/025-crypto-audit-crypto/025-FUSION.md` — Canonical fused audit, remediation architecture, and ordered implementation phases.
- `.planning/phases/025-crypto-audit-crypto/FUSION.audit.md` — Traceability back to source audits and merged finding inventory.
- `.planning/ROADMAP.md` — Phase 025 requirement IDs and planned execution order.
- `.planning/REQUIREMENTS.md` — Requirement definitions for `PH25-*` coverage.

### Claim Reachability And Consumers

- `crates/z00z_crypto/src/claim/mod.rs` — Current placeholder public claim facade.
- `crates/z00z_crypto/src/claim/proof.rs` — Placeholder proof, signature, and verification result types.
- `crates/z00z_crypto/src/claim/statement.rs` — Canonical statement bytes currently missing `claim_v2` version or root policy fields.
- `crates/z00z_wallets/src/core/tx/claim_tx.rs` — Production claim verification path that still consumes placeholder claim artifacts.
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` — Simulator claim package builder currently hard-coding `ZERO_ROOT` and placeholder proof or signature generation.

### ZkPack And Stealth Binding

- `crates/z00z_crypto/src/lib.rs` — Root crypto facade currently exposing `aead_zkpack` and legacy helpers.
- `crates/z00z_crypto/src/aead.rs` — Custom zkpack implementation and export path.
- `crates/z00z_crypto/src/zkpack.rs` — `ZkPackEncrypted` wire type with ambiguous `Option` parsing.
- `crates/z00z_wallets/src/core/stealth/facade_zkpack.rs` — Existing production-grade ChaCha20-Poly1305 wallet facade.
- `crates/z00z_wallets/src/core/stealth/tag.rs` — Canonical wallet owner of `tag16` and `leaf_ad` formulas that must move under crypto ownership.

### Storage-Owned Proofs And Roots

- `crates/z00z_storage/src/assets/types.rs` — Root and typed asset proof contracts.
- `crates/z00z_storage/src/assets/proof.rs` — Storage-owned proof blob and proof-check errors.
- `crates/z00z_storage/src/assets/store.rs` — Store trait, claim nullifier state, and natural API seam for source-proof requests.
- `crates/z00z_storage/src/checkpoint/artifact.rs` — Checkpoint artifact surface that can publish authoritative root metadata.
- `crates/z00z_storage/src/checkpoint/build.rs` — Member witness bridge from storage proofs to higher-level verification.
- `crates/z00z_core/src/assets/snapshot.rs` — Registry or snapshot versioning surface that can anchor trusted authority metadata.

</canonical_refs>

<specifics>
## Specific Ideas

- Current production-reachable claim flows exist in both wallet and simulator code, so placeholder claim APIs are active boundary flaws, not dormant scaffolding.
- The repository already contains the formulas and tests for deterministic wallet `ZkPack`, `tag16`, and `leaf_ad`; the gap is ownership and blessed-surface selection, not primitive absence.
- `hash_to_scalar_zk()` is already fail-closed, but compatibility wrappers such as `hash_to_scalar_domain`, `kdf::hash_to_scalar_domain`, `Z00ZScalar::from_hash`, `hmac_sha256`, and `Z00ZScalar::random` still preserve fail-open behavior that must be retired or fenced.

</specifics>

<deferred>
## Deferred Ideas

- If product requirements later demand a hidden-witness zk claim proof beyond authoritative source membership and a real authority signature, that second proof system is deferred to a future phase.
- Full native-versus-circuit Poseidon2 equivalence beyond frozen vector coverage is deferred until the repository exposes circuit-side verification inputs.

</deferred>

---

*Phase: 025-crypto-audit-crypto*
*Context gathered: 2026-03-27 via PRD Express Path*
