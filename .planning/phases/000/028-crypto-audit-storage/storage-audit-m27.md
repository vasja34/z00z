# Crypto Audit Report: `z00z_storage`

**Phase**: 028-crypto-audit-storage  
**Target**: `crates/z00z_storage/` — only `*.rs` implementation; `tari/` vendor code excluded  
**Date**: 2026-03-26  
**Confidence**: HIGH — full source read with cross-reference to `z00z_crypto` and `z00z_utils`

---

## 1. Executive Verdict

**Safe enough** — no S0 or S1 findings. The crate correctly delegates crypto-heavy lifting to `z00z_crypto` (Poseidon2, `hash_domain!`, `AssetIdDomain`) and uses a battle-tested JMT (Jellyfish Merkle Tree) via `jmt` crate backed by `sha2::Sha256` for internal merkle operations. All identity derivations use canonical `BincodeCodec` serialization + SHA-256 digest. `redb` is used as a plain key-value store without cryptographic operations. The design follows ONE SOURCE OF TRUTH principle.

---

## 2. Input Classification & Scope

| File | Role |
| --- | --- |
| `src/lib.rs` | Key derivation (`definition_key`, `serial_key`, `asset_key`) via `hash_zk` |
| `src/assets/keys.rs` | Thin re-export facade |
| `src/assets/leaf.rs` | `AssetLeaf` wrapper around `z00z_core::assets::AssetLeaf` |
| `src/assets/model.rs` | `AssetModel` — merkle-sum-tree hierarchy; `hash_many` for root composition |
| `src/assets/types.rs` | Strongly-typed IDs (`DefinitionId`, `AssetId`, `SerialId`), `AssetPath`, `RootRec`, `DefinitionRootLeaf`, `SerialRootLeaf` |
| `src/assets/proof.rs` | `ProofBlob` encoding/decoding; `leaf_hash` via `jmt::ValueHash::with::<Sha256>`; JMT proof verification via `SparseMerkleProof::verify_existence` |
| `src/assets/store.rs` | `AssetStore` — JMT reader/writer; namespace keys; `compute_secret_tag`; `MemTreeStore`; `TreeStore` |
| `src/assets/store_internal/redb_backend.rs` | `RedbBackend` — `redb` embedded DB; raw table persistence |
| `src/assets/store_internal/tree_store.rs` | `TreeStore`, `MemTreeStore`, `PrepCommit` — JMT batch commit |
| `src/checkpoint/artifact.rs` | `CheckpointDraft`, `CheckpointProof`, `CheckpointPubIn`, `SpentEnt`, `CreatedEnt` |
| `src/checkpoint/ids.rs` | ID derivations via SHA-256 of canonical binary encoding |
| `src/checkpoint/codec.rs` | JSON + Bincode codecs with version gating |
| `src/checkpoint/build.rs` | `TxPkgSum`, `MemberWit`, `ResolvedInput`, `InputResolver`, `SpentIndex`, checkpoint apply state machine |
| `src/checkpoint/exec_input.rs` | `CheckpointExecInput`, `CheckpointExecTx`, `CheckpointExecOut` |
| `src/checkpoint/link.rs` | `CheckpointLink` — chains artifact, snapshot, exec-input ids |
| `src/serialization/codec.rs` | JMT artifact codec + `derive_artifact_id` via SHA-256 |
| `src/serialization/build.rs` | JMT node/edge serialization artifact builder |
| `src/snapshot/codec.rs` | `PrepSnapshot` codec + `derive_id` via SHA-256 |
| `src/snapshot/store.rs` | `PrepSnapshotStore`, `PrepFsStore`, `PrepReplayEntry` |
| `src/error.rs` | Typed error enums |

---

## 3. Security Goals Assumed

1. **Integrity** — All stored asset state is verifiable against a commitment root (JMT root hash)
2. **Determinism** — Same inputs always produce bit-identical serialization + root output
3. **Replay resistance** — Nullifier tracking prevents double-spend / claim replay
4. **Non-malleability** — Checkpoint IDs derived from canonical binary form; JSON wire format rejected for ID derivation
5. **Auditability** — Checkpoint artifact encodes full delta (spent + created) with membership witnesses

---

## 4. Threat Model Summary

| Adversary | Goal | Mitigation |
| --- | --- | --- |
| Network observer | Learn asset paths / balances | All sensitive data in encrypted `AssetLeaf` (range proofs, commitments) from core layer |
| Malicious storage modifier | Corrupt DB, swap leaf bytes | JMT root verification on read; `SparseMerkleProof::verify_existence` checks namespace-keyed membership |
| Replay attacker | Re-submit spent claim nullifier | `ClaimNullRec` tracked in `redb`; `sync_store` rejects duplicates |
| Blob modifier | Swap proof bytes to redirect asset | `ProofBlob` encodes `asset_leaf_hash` independently; `chk_blob` cross-validates typed item against JMT branch proofs |
| ID collision attacker | Craft two different objects with same checkpoint id | SHA-256 of canonical binary encoding; reject JSON wire format for ID path |
| Downgrade attacker | Force older schema version | Version gating on all deserialization paths |

---

## 5. Critical & High Findings (S0/S1)

**None.**

---

## 6. Medium Findings (S2)

### S2-A: `leaf_hash` uses `jmt::ValueHash::with::<Sha256>` — output not domain-separated from other JMT uses

**Component**: `src/assets/proof.rs:138`

```rust
fn leaf_hash(leaf: &AssetLeaf) -> Result<[u8; 32], CodecError> {
    let codec = BincodeCodec;
    let payload = codec.serialize(leaf)?;
    Ok(jmt::ValueHash::with::<Sha256>(&payload).0)
}
```

**Problem**: `jmt::ValueHash` with `Sha256` is a plain hash — no domain separation tag. The same 32-byte hash is used as the JMT leaf value key and stored in `ProofBlob::asset_leaf_hash`. If an attacker can find a different `AssetLeaf` payload that serializes to the same bytes, they could swap the leaf without detection at the `leaf_hash` level. The `ProofBlob.asset_leaf_hash` field provides a secondary check (it must match `leaf_hash(leaf)?`), so the swap is blocked at the blob level. However, there is no explicit domain tag.

**Impact**: Under a chosen-plaintext attack on `BincodeCodec` serialization (unlikely), or a second-preimage attack on SHA-256 of a crafted leaf payload, an attacker could potentially substitute a leaf. The JMT proof verification would still pass because the value hash is the JMT leaf key, not the leaf content itself.

**Fix**: Domain-separate the leaf hash using a `hash_domain!` tag before the SHA-256:

```rust
use z00z_crypto::{hash_domain, hash_zk::hash_zk};

hash_domain!(StorLeafDom, "z00z.storage.leaf.hash.v1", 1);

fn leaf_hash(leaf: &AssetLeaf) -> Result<[u8; 32], CodecError> {
    let codec = BincodeCodec;
    let payload = codec.serialize(leaf)?;
    Ok(hash_zk::<StorLeafDom>(&payload))
}
```

**Confidence**: Medium — attack requires breaking SHA-256 second preimage OR controlling serialization input; JMT proof cross-check mitigates but does not eliminate.

---

### S2-B: `compute_secret_tag` uses bare `hash_zk` without result binding check

**Component**: `src/assets/store.rs:compute_secret_tag`

```rust
#[allow(dead_code)]
pub fn compute_secret_tag(s_out: &[u8; 32]) -> [u8; 32] {
    hash_zk::<AssetIdDomain>("", &[s_out])
}
```

**Problem**: `s_out` is a component of the encrypted pack inside `AssetLeaf`. The function is `#[allow(dead_code)]` — not used in current codebase, which is good. However, if it were used, the output is not validated against any stored commitment or proof. A caller who passes a wrong `s_out` would silently get a wrong tag with no error signal.

**Impact**: Low in current state (function unused). If used in a future wallet integration path, misuse could link assets incorrectly.

**Fix**: Either remove the dead function entirely, or add a comment that it must only be called with the verified `s_out` from a decrypted `AssetLeaf` with a prior range-proof validation.

---

### S2-C: Namespace keys use different domain tags but share the same `hash_zk` sponge

**Component**: `src/assets/store.rs:ns_key` and `src/assets/store.rs`

```rust
hash_domain!(StorAssetNsDom, "z00z.storage.ns.asset.v1", 1);
hash_domain!(StorSerNsDom, "z00z.storage.ns.serial.v1", 1);
hash_domain!(StorDefNsDom, "z00z.storage.ns.definition.v1", 1);
hash_domain!(StorPathNsDom, "z00z.storage.ns.path.v1", 1);
```

**Problem**: Each namespace uses a distinct `hash_domain!` tag. However, the JMT key is `ns_key(TreeId, KeyHash)` — the namespace key is derived by hashing the `tree_id` enum discriminator plus the semantic key. If `TreeId` enum has the same byte discriminant as a future added variant, collision could occur.

**Impact**: Low — `TreeId` enum variants are well-defined and not extensible without a code change. The domain tags provide adequate separation.

---

## 7. Low Findings (S3)

### S3-A: `sha2::Sha256` used directly for ID derivations — no HMAC

**Component**: `src/checkpoint/ids.rs`, `src/snapshot/codec.rs`, `src/serialization/codec.rs`

All three use `Sha256::digest(&bytes).into()` for content-addressed ID derivation. This is correct — content-addressed IDs don't need a MAC, they just need collision resistance. SHA-256 is still safe for this under the birthday-bound only (2^128 for 32-byte output). No action needed.

---

### S3-B: `BincodeCodec` is the canonical form for all ID derivations

**Component**: Multiple codec sites

`BincodeCodec` is used as the canonical encoding for ID derivation (checkpoint IDs, artifact IDs, snapshot IDs). JSON is supported only as a human-readable wire format and is explicitly rejected for ID derivation paths. This is correct — `encode_draft_bin`, `encode_art_bin`, `encode_snap` are used for ID derivations, not the JSON variants.

---

### S3-C: `sha256` in `hash_many` — local helper uses bare SHA-256

**Component**: `src/assets/model.rs`

```rust
fn hash_many<D: DomainSeparation>(domain: &[u8], parts: &[Vec<u8>]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(domain);
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}
```

**Assessment**: This is a local helper within `model.rs`, not exported. It uses a domain parameter but the domain is always passed as a `hash_domain!`-generated static string. The use of SHA-256 directly (not `hash_zk`) is appropriate here since this is computing a composite root from already-committed leaves (each leaf is a Poseidon2 commitment). The threat model for this hash is collision resistance only, which SHA-256 provides.

---

## 8. Findings from Mandatory Checklist

| Check | Status | Notes |
| --- | --- | --- |
| Threat model exists | ✅ | Documented above |
| Security goals explicit | ✅ | Integrity, determinism, replay resistance, non-malleability, auditability |
| Construction is standard | ✅ | JMT (Jellyfish Merkle Tree) via `jmt` crate; SHA-256; `redb` |
| Domain separation | ✅ | `hash_domain!` for storage keys; separate domain tags per namespace |
| Serialization canonical | ✅ | `BincodeCodec` is canonical; JSON is display-only |
| Transcript includes all fields | ✅ | `CheckpointPubIn` binds `prev_root`, `new_root`, `spent_delta`, `created_delta` |
| Nonce policy restart-safe | ✅ | Version counter in `StateMeta`; JMT version is monotonic |
| Secret lifecycle defined | ✅ | No raw secrets in this crate — all crypto assets come from `z00z_core` |
| Error handling no oracle | ✅ | All errors return typed enums; no padding or timing oracles |
| Multi-party assumptions | N/A | Single-node storage |
| Proof statement complete | ✅ | `MemberWit` binds `ProofItem` (root + path + def_leaf + ser_leaf + leaf) |
| ZK circuit constraints | N/A | No circuits in this crate |
| Trusted setup provenance | N/A | No ZK setup required |
| Dependencies audited | ⚠️ | `jmt` crate source not reviewed; `redb` is plain storage |

---

## 9. Z00Z-Specific Checks

| Check | Status |
| --- | --- |
| `hash_domain!` used for all storage key derivations | ✅ `StorDefKeyDom`, `StorSerKeyDom`, `StorAssetNsDom`, `StorSerNsDom`, `StorDefNsDom`, `StorPathNsDom` |
| `z00z_crypto::hash_zk::hash_zk` used (Poseidon2) for semantic keys | ✅ |
| `StorAssetDom`, `StorSerialDom`, `StorDefDom`, `StorStateDom`, `StorCompatDom` domain tags in `model.rs` | ✅ |
| `hash_zk::<AssetIdDomain>` for `compute_secret_tag` | ✅ but function is dead (`#[allow(dead_code)]`) |
| No direct `std::fs` usage | ✅ Uses `z00z_utils::io` |
| No direct `serde_json` / `serde_yaml` | ✅ Uses `z00z_utils::codec` (BincodeCodec + JsonCodec) |
| No `SystemTime` direct usage | ✅ Uses `z00z_utils::time` |
| `redb` backend is plain key-value, no crypto | ✅ |
| JMT proof verification on all read paths | ✅ `SparseMerkleProof::verify_existence` in `chk_blob` |
| Nullifier tracking via `ClaimNullRec` in `redb` | ✅ |
| `CheckpointPubIn` binds fee (via root transition) | ✅ Root transition implicitly covers all state changes |
| `AssetIdDomain` from `z00z_crypto::domains` used for secret tag | ✅ |
| Range proof parameters match genesis config | N/A — range proof generation is in `z00z_core`; storage only stores the proof bytes |

---

## 10. Open Ambiguities

1. **`jmt` crate provenance**: The `jmt` crate is used for the Merkle tree implementation. Its source is not vendored; the SHA-256 usage within JMT for value hashing is standard but the crate has not been audited as part of this review. The JMT design (based on Jellyfish Merkle Tree from Diem/Starcoin) uses SHA-256 for internal hashing, which is conservative and appropriate for a Merkle tree.

2. **`redb` durability**: `redb` is an embedded key-value store. It does not perform cryptographic operations. The integrity of data at rest depends on the integrity of the filesystem. If filesystem-level tampering occurs, the SHA-256-based JMT root provides detectability on next read (proof verification will fail).

3. **Upgrade path for `BincodeCodec`**: If a future version of `BincodeCodec` changes serialization format, stored checkpoint/snapshot IDs would diverge from new artifacts. The version gating mitigates this for supported versions.

---

## 11. Test Plan Validation

| Test Type | Coverage | Notes |
| --- | --- | --- |
| Codec roundtrip | ✅ | `encode_*` / `decode_*` symmetry tested in `codec.rs`, `ids.rs`, `snapshot/codec.rs` |
| ID stability across re-encode | ✅ | `test_draft_id_stable_across_reencode`, `test_art_id_stable_across_reencode` in `ids.rs` |
| Version gating | ✅ | Unsupported versions rejected with `VersionMix` |
| Malformed input rejection | ✅ | `test_bad_transport_rejects`, `test_malformed_root_rejects`, `test_malformed_link_id_rejects`, `test_malformed_version_tag_rejects` |
| Proof verification | ✅ | `test_whitebox_proofs.rs` tests JMT proof paths |
| Storage CRUD | ✅ | `test_whitebox_crud.rs` |
| Model root computation | ✅ | `test_whitebox_state.rs` |

**Gaps**: No property-based tests for serialization determinism across platform variations (endianness). No fuzzing for `ProofBlob::decode` or `chk_blob`.

---

## 12. Confidence Level

| Claim | Confidence | Evidence |
| --- | --- | --- |
| All identity derivations use SHA-256 of canonical Bincode | HIGH | Manual trace of all `derive_*` functions |
| Namespace keys are domain-separated via `hash_domain!` | HIGH | All call sites reviewed |
| JMT proof verification covers membership | HIGH | `SparseMerkleProof::verify_existence` called in `chk_blob` with correct namespace keys |
| Nullifier replay is blocked | MEDIUM | `ClaimNullRec` dedup check in `sync_store` — not formally verified against concurrent write races |
| No secret key material in this crate | HIGH | Full source scan negative |
| `AssetLeaf` wrapper preserves core crypto | HIGH | `AssetLeaf` is a zero-cost wrapper; actual crypto in `z00z_core` and `z00z_crypto` |

**Evidence that would increase confidence**: Full audit of `jmt` crate; formal verification of dedup race condition in `sync_store`.

---

## 13. Final Decision

**Execution-ready** — the crate is well-structured with correct use of `hash_domain!`, canonical serialization for IDs, JMT proof verification on reads, and typed error hierarchies. No S0/S1 issues found. The S2 findings are minor and either already mitigated (S2-A via `ProofBlob.asset_leaf_hash` cross-check) or not exploitable in the current state (S2-B, dead code). The architecture follows ONE SOURCE OF TRUTH correctly — all I/O goes through `z00z_utils::io` and `z00z_utils::codec`.

**Owner for S2-A fix**: `z00z_storage` maintainer — domain-tag the `leaf_hash` output if the function is ever used in a hot path.

---

## 14. Severity Table

| ID | Severity | Component | Problem | Fix |
| --- | --- | --- | --- | --- |
| S2-A | MEDIUM | `src/assets/proof.rs:leaf_hash` | No domain separation on JMT value hash | Add `hash_domain!(StorLeafDom)` wrapper |
| S2-B | MEDIUM | `src/assets/store.rs:compute_secret_tag` | Dead function with no validation hook; misuse risk if activated | Remove or document constraints |
| S2-C | LOW | `src/assets/store.rs:ns_key` | `TreeId` enum variants not verified extensible-safe | Add enum exhaustive match warning |
| S3-A | INFO | Multiple ID derivation sites | Direct SHA-256 without HMAC — acceptable for content-addressing | No action needed |
| S3-B | INFO | Codec architecture | JSON is wire-format only; Bincode is canonical — correct design | No action needed |
| S3-C | INFO | `src/assets/model.rs:hash_many` | Local helper uses bare SHA-256 — acceptable for composite root | No action needed |
