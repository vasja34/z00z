---
phase: 025-crypto-audit-crypto
artifact: claim-source-proof-boundary
status: completed
source:
  - crates/z00z_storage/src/assets/store.rs
  - crates/z00z_wallets/src/core/tx/claim_tx.rs
  - .planning/phases/000/025-crypto-audit-crypto/025-04-PLAN.md
review_mode: crypto-architect
updated: 2026-03-28
---

# Claim Source Proof Cache Boundary Contract

📌 Strict cache rule: storage or a storage-backed cache may transport
`ClaimSourceProof`, but verifier trust is earned only after reconstructing the
expected canonical leaf from `AssetPkgWire` and validating the cached proof blob
against that leaf, the rooted path, and the storage-owned root metadata.

## Scope

📌 This contract defines the production wallet or storage boundary for
`ClaimSourceProof` in the live `claim_v2` path.

📌 The contract applies to proof production in
`AssetStore::claim_source_proof(...)`, proof transport through simulator or cache
layers, and proof verification in the wallet claim verifier.

## Threat Model

📌 Trusted producer: `z00z_storage::assets::AssetStore` backed by the current
storage root.

📌 Untrusted inputs: tx payload bytes, simulator transport artifacts, cache
entries, proof headers, and any reserialized proof blob delivered outside the
storage seam.

📌 Attacker goal: make the wallet accept a stale, rebound, or cross-asset proof
blob as if it were a valid proof for the current `ClaimStmtV2` and current
claim-source root.

## Role Contract

| Role | Contract |
| ---- | -------- |
| Producer | Storage owns proof construction and root derivation. |
| Cache or Transport | Cache stores opaque proof artifacts but owns no truth and no rewrite authority. |
| Verifier | Wallet reconstructs the expected canonical leaf and validates the cached blob against rooted storage semantics. |

## Producer Requirements

- Producer MUST derive the claim-source root only from
  `AssetStore::claim_source_root()` backed by the current storage root.
- Producer MUST emit `ClaimSourceProof` only through
  `AssetStore::claim_source_proof(&AssetPath)` for the exact canonical path being
  claimed.
- Producer MUST encode the proof blob from the storage-owned `ProofBlob` result
  for that exact path; producer SHALL NOT invent an alternative blob format,
  root, or leaf witness.
- Producer MUST keep header and blob roots identical: `source_root` in
  `ClaimSourceProof` SHALL equal `ClaimSourceRoot::into_bytes()` and SHALL
  equal `blob.item().root().into_bytes()` after decode.
- Producer MUST emit `root_ver = V1` and `proof_ver = V1` for the current live
  seam until the verifier explicitly supports a new version.

## Cache And Transport Requirements

- Cache or transport MUST treat `proof_blob` as opaque bytes and MUST NOT
  re-encode, splice, or partially deserialize-reserialize it.
- Cache or transport MUST NOT substitute `source_root`, `root_ver`,
  `proof_ver`, or path metadata independently of the stored blob.
- Cache or transport MUST bind every cached proof entry to the exact
  `source_root` and exact `AssetPath` it was fetched for.
- Cache or transport MUST invalidate cached proof entries when the storage root,
  snapshot identity, or checkpoint identity changes.
- Cache or transport SHALL NOT act as a second source of truth for root, path,
  leaf, or version semantics.

## Verifier Requirements

- Verifier MUST fail closed in this order: reject zero root, reject unsupported
  root version, reject unsupported proof version, decode `proof_blob`, then
  enforce statement-to-proof version coherence.
- Verifier MUST treat tx payload proof bytes as untrusted until the storage
  proof blob has been validated against rooted storage semantics.
- Verifier MUST reject any mismatch between `ClaimStmtV2`,
  `ClaimSourceProof`, and decoded `ProofBlob` metadata, including header root
  mismatch versus blob root and path `asset_id` mismatch versus
  `stmt.claim_source_asset_id`.
- Verifier MUST reconstruct the canonical terminal leaf from `AssetPkgWire` and
  MUST validate the cached proof blob with `chk_blob(...)` against the exact
  root, path, definition leaf, serial leaf, and terminal leaf bytes.
- Verifier SHALL NOT accept a proof because the blob decodes successfully, the
  header root looks plausible, or the cache says the entry is valid.

## Forbidden Misuse

- Reusing a cached proof after the storage root changes and presenting it as a
  proof for the new state.
- Rebinding a valid blob to a different advertised `source_root` or different
  path metadata.
- Reusing a proof blob for a different `AssetPkgWire` and hoping the verifier
  will trust blob semantics without recomputing the canonical leaf.
- Treating proof decode plus metadata checks as a complete authorization
  decision while skipping full blob validation and the surrounding
  `ClaimStmtV2` or authority-signature checks.

## Current Code Anchors

📌 Producer anchor: `crates/z00z_storage/src/assets/store.rs` via
`claim_source_root()` and `claim_source_proof(...)`.

📌 Verifier anchors: `crates/z00z_wallets/src/core/tx/claim_tx.rs` via
`decode_source_blob(...)`, `verify_source_meta(...)`, and
`verify_source_blob(...)`.

📌 Phase intent anchor:
`.planning/phases/000/025-crypto-audit-crypto/025-04-PLAN.md`.

## Acceptance Check

📌 The seam is compliant only when the producer is storage-owned, cache is
opaque and invalidated on root changes, and the verifier reconstructs the
expected leaf before accepting the cached proof blob.
