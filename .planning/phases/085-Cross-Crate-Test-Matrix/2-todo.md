## 8. Core Object, Claim-Root, And Checkpoint Authority Hardening

**Goal:**

- Establish the local protocol floor for every later phase: canonical object contracts, honest `claim_root` handling, checkpoint proof authority, replay rejection, deterministic digests, and storage-owned asset paths.
- Prove through local unit and simulator tests that wallet packages, claim packages, asset leaves, and checkpoint artifacts have fail-closed boundaries before any network or production authority is considered.

**Source:**

- [Main whitepaper, section 3.1: Canonical State Objects](Z00Z-Main-Whitepaper.md#31-canonical-state-objects)
- [Main whitepaper, section 3.2: Cryptographic Integrity And Proof Discipline](Z00Z-Main-Whitepaper.md#32-cryptographic-integrity-and-proof-discipline)
- [Main whitepaper, section 3.3: Checkpoints As Validation Boundary](Z00Z-Main-Whitepaper.md#33-checkpoints-as-validation-boundary)
- [Main whitepaper, section 12.3: Proposed Expansion Path](Z00Z-Main-Whitepaper.md#123-proposed-expansion-path)
- [Main whitepaper, appendix B: Cryptography Detail Boundary](Z00Z-Main-Whitepaper.md#appendix-b-cryptography-detail-boundary)

**Implementation-relevant fragments:**

- Use section 3.1 for canonical object surfaces, asset/claim object boundaries, and the storage-owned key material that must become typed contracts.
- Use section 3.2 for canonical encoding, digest stability, bounded parsing, and fail-closed proof discipline.
- Use section 3.3 for checkpoint authority, replay rejection, and the distinction between checkpoint validation and wallet pre-broadcast checks.
- Use section 12.3 and appendix B only as expansion and crypto-boundary guardrails; do not infer recursive proofs, production DA, or live validator work from this phase.

**Locality gate:**

- This is pure local protocol work: type definitions, codecs, validation, replay checks, digest vectors, checkpoint continuity checks, and simulator fixtures.
- No live network, external DA, testnet, bridge, or production operator is needed.

**Implementation boundary:**

- In scope: current cash/claim object contracts, honest `claim_root` propagation distinct from normal state roots, one checkpoint-owned proof verification path reused at seal and reload, generalized-right extension slots only as bounded metadata or explicitly experimental types, canonical encoding, fail-closed parsing, package digest stability, replay rejection, checkpoint artifact linking, and storage-owned asset paths.
- Out of scope: live consensus admission, recursive proof systems, production DA authority, real validator networking, and performance claims beyond local evidence.

**Implementation tasks:**

1. In `z00z_core`, tighten canonical object contracts around `AssetDefinition`, `AssetLeaf`, `AssetPkgWire`, `AssetWire`, `AssetPackPlain`, `AssetPackPlainMemo`, and asset policy metadata.
2. In `z00z_storage`, keep `AssetPath { definition_id, serial_id, asset_id }` as the canonical committed key and expand negative tests for path/leaf mismatches.
3. In `z00z_storage`, keep `ClaimSourceRoot` and checkpoint `claim_root` distinct from normal state roots; emit `claim_root` only for claim-carrying checkpoint content and reject synthetic aliases.
4. Consolidate one checkpoint-owned proof verification entrypoint inside `z00z_storage::checkpoint` and reuse it during artifact seal, reload, and rehydrate paths.
5. In `z00z_wallets`, keep `TxPackage` and `ClaimTxPackage` as portable package surfaces, but make every import path use bounded parsing and deterministic reject classes.
6. In `z00z_simulator`, keep claim-package consumers and stage evidence bound to storage-owned claim-source truth only.
7. Add golden digest vectors for regular packages, claim packages, asset pack payloads, and checkpoint link IDs.
8. Add a compatibility guard that rejects conversion between unrelated root types, especially `TxDigest` to `CheckRoot`, except through explicit checked APIs.
9. Document and test the difference between wallet pre-broadcast verification and storage/checkpoint authority.
10. Where generalized spendable capability metadata is added, keep it non-authoritative until a concrete verifier exists.

**Tests and simulation:**

- Unit tests for `AssetLeaf` and `AssetPath` mismatch rejection: wrong `asset_id`, wrong `serial_id`, wrong `definition_id`, duplicate path, duplicate terminal asset ID, and delete of missing path.
- Codec tests for asset pack strict length, memo max length, unsupported version, malformed blinding, and unknown fields.
- Claim-root tests for claim-carrying checkpoint batches, non-claim batches, `claim_root != new_root` when claim data exists, and absence of invented claim roots when claim data does not exist.
- Package tests for empty payload, malformed JSON, wrong `kind`, wrong `package_type`, wrong version, wrong digest, duplicate input state keys, duplicate output state keys, input/output overlap, duplicate nonce, and fee-output mismatch.
- Claim package tests for duplicate claim asset path, duplicate claim nullifier, mismatched persisted claim store membership, mismatched claim source root, mismatched proof version, and mismatched proof blob.
- Simulator regression that builds a claim package bundle, verifies it against a local claim store, publishes to `AssetStore`, then rejects replay.
- Checkpoint tests that reject mismatched `prev_root`, mismatched snapshot ID, mismatched exec input ID, mismatched fragment IDs, proof-byte drift, statement drift, root drift, detached proof bytes without package-coupled context, and tampered or unbound artifacts on both memory-backed and `redb` reload paths.

**Done when:**

- Developers can run local unit and simulator tests and see deterministic pass/fail reasons for every canonical object boundary above.
- `claim_root` is real, absent, or rejected according to batch content and no longer silently aliases the state root.
- One checkpoint-owned proof verification path exists and is reused at seal and reload boundaries.
- Storage reload rejects tampered or unbound checkpoint artifacts.
- No test needs a live node, testnet, external DA, or external attester.
- Existing public crate facades remain stable or changes are explicitly versioned.

**Doublecheck:**

- Local condition: satisfied. All work is local code, local storage, local codecs, and local simulator evidence.
- Developer clarity: satisfied. Crates, object surfaces, reject cases, and completion gates are explicit.

## 
