# 🔐 Cryptographic Audit — `z00z_storage`

**Auditor:** Claude Sonnet 4.6  
**Date:** 2025-07-14  
**Scope:** `crates/z00z_storage/src/**/*.rs` — implementation files only, Tari vendor excluded  
**Crate version:** 0.1.0 (edition 2021, `#![forbid(unsafe_code)]`)  
**Skill:** `crypto-architect` — full 7-phase methodology

---

## 📌 Phase 0 — Input Classification

| Attribute | Value |
|---|---|
| Input type | Implementation code review — Rust source |
| Artifact class | Authenticated state store + checkpoint pipeline + serialization |
| External crypto primitives | SHA-256 (sha2 0.10), JMT (jmt 0.12, Jellyfish Sparse Merkle), Poseidon2 (via z00z_crypto) |
| Internal crypto primitives | `hash_zk`, `poseidon2_hash`, `hash_domain!` from z00z_crypto |
| ZK circuits | None in this crate; `CheckpointProofSystem::OPAQUE` = placeholder for future external ZK proofs |
| Persistence | `redb = "2"` embedded key-value database |
| Concurrency | `rayon = "1.10"` parallel batches for asset/serial tree commits |
| Secret material | `compute_secret_tag()` (dead code), `ClaimNullRec.owner_hex` |
| Key external hooks | `TxProofVerifier`, `SpentIndex` traits (callee-supplied validity) |

---

## 🎯 Phase 1 — Scope and Threat Model

### System Description

`z00z_storage` is the authenticated state store for the Z00Z blockchain. It maintains:

1. **Asset state** — a three-level Jellyfish Sparse Merkle Tree (Definition → Serial → Asset) backed by a `MemTreeStore` and persisted to `redb`.
2. **Checkpoint pipeline** — draft → finalize → link artifacts recording every state transition.
3. **Snapshot system** — pre-state witness captures used for checkpoint replay.
4. **Claim nullifier registry** — cross-chain nullifier double-spend protection.

### Dual-Root Architecture

The crate maintains **two independent root types** simultaneously:

```
Semantic root (AssetStateRoot)
  = poseidon2_hash::<StorStateDom>(sorted_def_leaves)
  — ZK-friendly, Poseidon2-based, domain-separated

JMT backend root (flat_root / RootHash)
  = SHA-256 Jellyfish Sparse Merkle root over ns_key(tree_id, key) → jmt::ValueHash::Sha256
  — membership proof source, SHA-256-based
```

These two roots are computed together on every `commit_plan()` but are stored and transmitted independently, with **no cross-root binding proof**.

### Threat Model

| Threat | Where |
|---|---|
| T1 — Forged membership proof | `assets/proof.rs` `chk_blob()` |
| T2 — Invalid checkpoint proof accepted | `checkpoint/artifact.rs`, `redb_backend.rs` |
| T3 — Cross-type ID confusion | `checkpoint/ids.rs`, `snapshot/codec.rs`, `serialization/codec.rs` |
| T4 — Double-spend bypass | `checkpoint/build.rs`, `assets/store.rs` |
| T5 — Root substitution | `assets/proof.rs`, `checkpoint/link.rs` |
| T6 — Storage backend tampering | `assets/store_internal/redb_backend.rs` |
| T7 — Information leakage | `assets/store.rs` (`ClaimNullRec` errors) |
| T8 — Availability via env var abuse | `Z00Z_ASSET_ROOT_MODE`, `Z00Z_STORAGE_REDB_INJ` |

---

## ⚙️ Phase 2 — Construction Selection Review

### SHA-256 for Content-Addressed IDs

| ID type | Derivation | Domain label |
|---|---|---|
| `PrepSnapshotId` | `Sha256::digest(bincode(snapshot))` | ❌ None |
| `CheckpointDraftId` | `Sha256::digest(bincode(draft))` | ❌ None |
| `CheckpointId` | `Sha256::digest(bincode(artifact))` | ❌ None |
| `CheckpointExecInputId` | `Sha256::digest(raw_bytes)` | ❌ None |
| `JmtSerArtifactId` | `Sha256::digest(bincode(artifact))` | ❌ None |

⚠️ **All five ID types use raw SHA-256 without domain separation.** Second-preimage attacks across type boundaries are structurally mitigated by different bincode struct layouts, but not cryptographically prevented. Cross-type collision would enable ID spoofing in proof pipelines.

**Contrast:** The authenticated state roots use `poseidon2_hash::<Domain>(...)` with explicit `hash_domain!` declarations, providing cryptographic domain separation.

### JMT Key Derivation

| Key function | Input | Domain separation |
|---|---|---|
| `definition_key(def_id)` | `[0x01] || def_id.bytes` | `StorDefKeyDom` via `hash_zk` ✅ |
| `serial_key(def_id, ser_id)` | `[0x02] || def_id.bytes || ser_id.le_bytes` | `StorSerKeyDom` via `hash_zk` ✅ |
| `asset_key(asset_id)` | `asset_id.into_bytes()` | ❌ Raw bytes — no pre-hash |

`asset_key` returns raw `AssetId` bytes as `KeyHash`. All downstream uses apply `ns_key(TreeId::Asset(..), key)` which adds `hash_zk::<StorAssetNsDom>(ns_bytes || raw_key)`, providing final domain separation. However the API inconsistency is a maintenance trap: any new caller using `asset_key()` without `ns_key()` would insert a non-domain-separated key directly into the JMT.

### Root Computation Modes

The `Z00Z_ASSET_ROOT_MODE` env-var switches between two implementations:

| Mode | Implementation | Source |
|---|---|---|
| `current_incremental` (default) | `TreeRoots::sem_root` incremental cache | `store.rs:incr_root` |
| `current_full_recompute` | `AssetModel::root()` full Poseidon2 recompute | `store.rs:full_root` |

Both should produce identical `AssetStateRoot` but there is no runtime equality assertion bridging the two modes. Divergence would silently produce different roots for the same state.

---

## 🔑 Phase 3 — Composition Review

### Checkpoint Proof Composition

```
CheckpointDraft
  → finalize(CheckpointProof)
  → CheckpointArtifact
       .checkpoint_proof().cp_proof()
       = OPAQUE bytes (any non-empty Vec<u8>)
```

In production (via `redb_backend.rs::proof_bytes()`):
```rust
fn proof_bytes(exec_id: CheckpointExecInputId, state_root: AssetStateRoot) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(64);
    bytes.extend_from_slice(exec_id.as_bytes());    // 32 bytes
    bytes.extend_from_slice(state_root.as_bytes()); // 32 bytes
    bytes
}
```

⚠️ **The "proof" carried in every storage-generated checkpoint artifact is a 64-byte raw concatenation of two known identifiers — not a cryptographic proof of any kind.** The proof system tag `OPAQUE` signals this is a placeholder, but the field name `checkpoint_proof` creates a false impression that a validity witness is present.

### Proof Blob Root Binding Chain

```
ProofBlob {
    item: ProofItem { root: AssetStateRoot (Poseidon2), path, def_leaf, ser_leaf, leaf },
    asset_leaf_hash: ValueHash (SHA-256 leaf hash),
    backend_root: [u8; 32] (JMT SHA-256 root),   ← self-supplied
    definition_proof: SparseMerkleProof<Sha256>,
    serial_proof: SparseMerkleProof<Sha256>,
    asset_proof: SparseMerkleProof<Sha256>,
}
```

In `chk_blob()` (line 196-262, `assets/proof.rs`):
```
1. item.root() checked against external `root` parameter  ✅
2. JMT proofs verified against blob.backend_root()        ⚠️  (self-supplied)
3. No check: item.root() ↔ backend_root cryptographic binding
```

The semantic root and JMT backend root are **co-created atomically** on each `commit_plan()` but stored as independent fields in the blob with **no cross-binding proof**. An attacker who can forge a blob (with preserved SHA-256 ID) could combine a valid `item.root()` from epoch T with `backend_root` + JMT membership proofs from epoch T-1 without detection.

### Claim Nullifier Pipeline

```
ClaimNullTx { nullifier_hex, claim_id_hex, chain_id, owner_hex, tx_digest_hex }
  → claim_rows_for() validates uniqueness (BTreeSet seen + BTreeMap claim_nulls)
  → commit_claim_rows() inserts into in-memory BTreeMap keyed by nullifier_hex
  → redb persisted in CLAIM_NULL_TABLE keyed by version || nullifier_hex.as_bytes()
```

The double-spend protection uses hex-string keys in an in-memory `BTreeMap`. This is correct semantically but hex encoding adds unnecessary bytes and allocations to a hot path.

---

## ♨️ Phase 4 — ZK Circuit Review

No ZK circuits are implemented in this crate. The `CheckpointProofSystem::OPAQUE` field is a placeholder for future external ZK prover integration. The `compute_secret_tag()` function (domain-separated Poseidon2 commitment to a secret output scalar) is currently `#[allow(dead_code)]` and not integrated into any proof pipeline.

---

## ⭐ Phase 5 — Implementation Review

### Full Findings Inventory

---

#### 🚨 F-01 · **S1 · CRITICAL** — Checkpoint `cp_proof` is a non-cryptographic placeholder in production

| Attribute | Value |
|---|---|
| **File** | [`src/assets/store_internal/redb_backend.rs`](src/assets/store_internal/redb_backend.rs) |
| **Function** | `proof_bytes()`, `sync_store()` |
| **Description** | Every `CheckpointArtifact` written by `AssetStore` carries a `cp_proof` field containing `exec_id.bytes || state_root.bytes` — 64 raw bytes with no cryptographic validity property. The proof system tag is `CheckpointProofSystem::OPAQUE`. |
| **Exploit path** | An external verifier reading `CheckpointArtifact.checkpoint_proof().cp_proof()` as a validity proof would accept any checkpoint regardless of whether the state transition was legitimate. Any component that enforces "non-empty proof required" (which storage does) but does NOT independently validate proof content is trivially bypassed. |
| **Impact** | The entire checkpoint proof field provides no cryptographic commitment to execution validity. State transition integrity depends entirely on trusted callers, not on proof verification. |
| **Fix** | Document `OPAQUE` proof contracts explicitly in `CheckpointProof` rustdoc: this field is a passthrough placeholder until a ZK proving system is integrated. Gate any external proof-relying logic behind a `ProofSystem::Verified` discriminator. Add a `is_verified_proof_system()` predicate. |

---

#### 🚨 F-02 · **S2 · HIGH** — `ProofBlob.backend_root` is self-attesting with no semantic cross-binding

| Attribute | Value |
|---|---|
| **File** | [`src/assets/proof.rs`](src/assets/proof.rs) |
| **Functions** | `chk_blob()`, `proof_blob()` in `proof_help.rs` |
| **Description** | `ProofBlob` carries two independent roots: `item.root()` (Poseidon2 semantic root) and `backend_root` (JMT SHA-256 root). In `chk_blob()`, `item.root()` is verified against the external parameter `root: AssetStateRoot`, but the three JMT membership proofs are verified against `blob.backend_root()` — a self-supplied field. No check confirms that `backend_root` corresponds to the same store epoch as `item.root()`. |
| **Exploit path** | Attacker with blob storage write access constructs a blob with valid `item.root()` from epoch T (passing semantic root check) but replaces `backend_root` and membership proofs with data from epoch T-1. This produces a blob that passes all `chk_blob()` checks yet proves wrong epoch membership. |
| **Impact** | JMT membership proofs (the cryptographic backbone of asset ownership verification) are verified against an unanchored root. The blob provides a weaker guarantee than implied: "JMT proofs valid for SOME backend root" rather than "JMT proofs valid for the state that produced semantic root R." |
| **Fix** | Add a cross-root commitment: store `sha256(semantic_root || backend_root)` or `hash_zk::<Binding>(semantic_root, backend_root)` in the blob and verify it in `chk_blob()` before any JMT proof verification. Alternatively, derive `backend_root` deterministically from `item.root()` (making one redundant). |

---

#### ⚠️ F-03 · **S2 · HIGH** — Content-addressed IDs use undifferentiated SHA-256 with no domain labels

| Attribute | Value |
|---|---|
| **Files** | [`src/checkpoint/ids.rs`](src/checkpoint/ids.rs), [`src/snapshot/codec.rs`](src/snapshot/codec.rs), [`src/serialization/codec.rs`](src/serialization/codec.rs) |
| **Pattern** | `Sha256::digest(&bincode_bytes).into()` — no domain prefix |
| **Affected types** | `PrepSnapshotId`, `CheckpointDraftId`, `CheckpointId`, `CheckpointExecInputId`, `JmtSerArtifactId` |
| **Description** | All five content-addressed ID types derive their 32-byte identifier from raw SHA-256 of bincode-serialized bytes. There is no domain label separating snapshot IDs from checkpoint IDs, exec IDs, or serialization artifact IDs. |
| **Exploit path** | A crafted `CheckpointArtifact` whose bincode serialization is a prefix extension of a `PrepSnapshot`'s serialization would produce ID aliasing. Length-extension is inapplicable to `Sha256::digest` (`sha2` uses `finalize()` internally), but chosen-prefix SHA-256 collisions (feasible under multi-year adversarial effort) would allow cross-type ID substitution in `CheckpointLink` lookups. |
| **Practical risk rating** | Moderate — under current SHA-256 assumptions, structural collision across types is negligible; risk increases if SHA-256 is weakened or replaced. |
| **Fix** | Prefix bincode bytes with a 1-byte or 4-byte type discriminant before hashing: `Sha256::new().chain_update(&[TYPE_TAG]).chain_update(&bytes).finalize()`. Alternatively, route all IDs through `hash_zk` with distinct `hash_domain!` values per artifact type. |

---

#### ⚠️ F-04 · **S2 · HIGH** — `CheckpointLink` provides no cryptographic binding over its three IDs

| Attribute | Value |
|---|---|
| **File** | [`src/checkpoint/link.rs`](src/checkpoint/link.rs) |
| **Description** | `CheckpointLink { checkpoint_id, prep_snapshot_id, exec_input_id }` is a plain struct with no HMAC, hash commitment, or signature binding the three IDs to each other. It is stored in redb `LINK_TABLE` keyed by `checkpoint_id`. |
| **Exploit path** | Attacker with redb write access replaces a `LINK_TABLE` entry, substituting `prep_snapshot_id` of a different snapshot or `exec_input_id` of a different execution. The link would pass all structural checks but point to mismatched artifacts. `check_link_ids()` in `checkpoint/store.rs` verifies `exec_id` derivation but only if the caller passes the correct exec bytes — a direct lookup by `checkpoint_id` would return the forged link without re-verification. |
| **Impact** | State audit trails can be falsified at the link level. |
| **Fix** | Commit the link content into the checkpoint ID derivation (embed `snap_id + exec_id` into the artifact before `derive_checkpoint_id`), OR compute `link_id = sha256(checkpoint_id || snap_id || exec_id)` and store as the lookup key. |

---

#### ⚠️ F-05 · **S2 · HIGH** — `TxProofVerifier` and `SpentIndex` hooks have no storage-layer fallback

| Attribute | Value |
|---|---|
| **File** | [`src/checkpoint/build.rs`](src/checkpoint/build.rs) |
| **Description** | `apply_batch_checkpoint()` depends on caller-supplied `TxProofVerifier` (for per-tx ZK/STARK proof verification) and `SpentIndex` (for inter-epoch double-spend detection). If these hooks return `Ok` for invalid inputs, the checkpoint build succeeds. |
| **Exploit path** | A no-op `TxProofVerifier` implementation (which always returns `Ok`) combined with a no-op `SpentIndex` means `apply_batch_checkpoint()` accepts arbitrary malformed batches. There is no independent validity check inside the storage layer. |
| **Impact** | The storage layer cannot be deployed safely without correct hook implementations; it provides no defense-in-depth. |
| **Fix** | Document the trust assumption explicitly in function-level rustdoc. Consider adding a sealed `InternalOnly { _priv: () }` wrapper for the hook traits so that only library-vetted implementations can be used in production `AssetStore::apply_ops()`. |

---

#### ⚠️ F-06 · **S2 · HIGH** — Dual root mode via env var: divergence risk + panic on bad value

| Attribute | Value |
|---|---|
| **File** | [`src/assets/store.rs`](src/assets/store.rs) — `RootMode::load()` |
| **Description** | `Z00Z_ASSET_ROOT_MODE=current_full_recompute` switches to `AssetModel::root()` (Poseidon2 full recompute); default uses incremental `TreeRoots::sem_root`. Both should produce identical `AssetStateRoot`. An unrecognized value triggers `panic!()`. |
| **Exploit path 1** | If the two implementations diverge on any edge case (empty-state `hash_many`, lazy model pruning, rayon non-determinism), different nodes could compute different roots from the same state, breaking consensus. |
| **Exploit path 2** | Setting `Z00Z_ASSET_ROOT_MODE=invalid` panics the process. This is a trivial denial-of-service via environment variable. |
| **Fix** | Replace `panic!()` with process startup validation returning a typed error. Add debug-build assertion: `assert_eq!(incr_root, full_root, "root mode divergence")` after every commit to catch implementation divergence before production. |

---

#### 🔔 F-07 · **S3 · MEDIUM** — `Z00Z_STORAGE_REDB_INJ` fault injector in production binary

| Attribute | Value |
|---|---|
| **File** | [`src/assets/store_internal/redb_backend.rs`](src/assets/store_internal/redb_backend.rs) |
| **Description** | `if env::var_os(INJ_ENV).is_some() { return Err(StoreBackendError::Commit("inj")); }` — all write commits fail when `Z00Z_STORAGE_REDB_INJ` is set. This fault injection point is unconditionally compiled into the production binary. |
| **Exploit path** | An attacker or accidental operator who sets this env var silently prevents all storage writes from committing while accepting data into the commit path. Dirty state may accumulate without persistence across restarts. |
| **Fix** | Gate with `#[cfg(test)]` or an `integrity_tests` feature flag. Remove from production builds. |

---

#### 🔔 F-08 · **S3 · MEDIUM** — Nullifier metadata leak through error strings

| Attribute | Value |
|---|---|
| **File** | [`src/assets/store.rs`](src/assets/store.rs) — `claim_rows_for()` |
| **Description** | `AssetStoreError::ClaimReplay` embeds `nullifier_hex`, `status`, `tx_digest_hex` from `ClaimNullRec` into the error string. These cross-chain identifiers may leak through application logs at `ERROR` level. |
| **Exploit path** | An observer with log access could enumerate previously spent nullifiers and correlate cross-chain claim activity, breaking privacy of Z00Z's claim pipeline. |
| **Fix** | Sanitize error messages: `ClaimReplay("nullifier already spent")` without embedding hex values. Embed identifiers only at `TRACE` log level behind a dedicated `cfg(feature = "debug_claims")` gate. |

---

#### 🔔 F-09 · **S3 · MEDIUM** — `asset_key()` raw byte passthrough: inconsistent key API

| Attribute | Value |
|---|---|
| **File** | [`src/assets/keys.rs`](src/assets/keys.rs) |
| **Description** | `asset_key(asset_id)` = `KeyHash(asset_id.into_bytes())` — raw asset ID bytes. Compare: `definition_key` = `hash_zk::<StorDefKeyDom>(...)`, `serial_key` = `hash_zk::<StorSerKeyDom>(...)`. All callers of `asset_key` currently apply `ns_key()`, which adds domain-separated hashing before the JMT. But the intermediate `KeyHash` returned by `asset_key` contains raw attacker-influenceable bytes. |
| **Exploit path** | A future refactor introducing a new `TreeStore` method that uses `asset_key()` without `ns_key()` would bypass domain separation entirely, inserting raw asset IDs directly as JMT keys. |
| **Fix** | Apply `hash_zk::<StorAssetKeyDom>(...)` inside `asset_key()` for consistency: `KeyHash(hash_zk::<StorAssetKeyDom>("", &[asset_id.as_bytes()]))`. |

---

#### ⚠️ F-10 · **S4 · LOW** — Bincode as ID commitment source: no formal stability guarantee

| Attribute | Value |
|---|---|
| **Files** | All `derive_*_id()` functions |
| **Description** | `BincodeCodec` layout depends on Rust struct field ordering and bincode library version. Struct field reordering or field type changes silently change historic IDs. There are no explicit bincode schema version markers. |
| **Risk** | Future crate refactoring may silently invalidate all stored content-addressed IDs without schema migration tooling. |
| **Fix** | Add a trailing `_schema_v1: ()` marker field to all ID-source types, or use a custom `encode_for_id()` function that serializes a fixed canonical byte layout independent of struct field ordering. |

---

#### ⚠️ F-11 · **S4 · LOW** — `compute_secret_tag()` dead code: ZK output blinding unused

| Attribute | Value |
|---|---|
| **File** | [`src/assets/store.rs`](src/assets/store.rs) — line 139 |
| **Description** | `compute_secret_tag(s_out: &[u8;32]) -> [u8;32]` computes `hash_zk::<AssetIdDomain>("", &[s_out])` — a domain-separated ZK commitment to a secret output scalar. The function is `#[allow(dead_code)]` and is not called anywhere in the asset production pipeline. |
| **Risk** | If the privacy design requires output blinding (so that `asset_id` in public outputs is a commitment rather than a raw identifier), the current store does NOT apply this blinding. Asset IDs may be stored and logged as raw public values when they should be commitments. |
| **Fix** | Either integrate `compute_secret_tag` into the asset creation pipeline at the appropriate boundary, or explicitly document that raw `asset_id` bytes ARE intended to be public identifiers and remove the dead code to reduce confusion. |

---

#### ⚠️ F-12 · **S4 · INFO** — Mixed endianness in redb storage key encoding

| Attribute | Value |
|---|---|
| **File** | [`src/assets/store_internal/redb_backend.rs`](src/assets/store_internal/redb_backend.rs) |
| **Description** | `ver_key(version) = version.to_be_bytes()` (big-endian), but `ser_row_key()` and `asset_row_key()` embed `serial_id.get().to_be_bytes()` alongside version keys. However in `tree_id.rs::ns_bytes()`, `serial_id.get().to_le_bytes()` is used. Inconsistency: redb key encoding uses BE for serial_id, but JMT key encoding uses LE for serial_id. |
| **Risk** | Not a security issue, but keys derived for proof purposes use LE while keys derived for storage lookup use BE. Sorted range queries across (version, serial_id) in redb would produce unexpected ordering. |
| **Fix** | Standardize on a single endianness (preferably BE for redb range queries, LE for JMT hash inputs to match current crypto convention). |

---

## 🛡️ Phase 6 — Validation Requirements

### Tests Observed to Be Present
- Round-trip codec tests for all artifact types
- Version gate rejection tests
- Duplicate path/asset-id detection in snapshot
- JMT snapshot integrity via model-based reference comparison
- `compute_secret_tag` tested indirectly in z00z_core

### Missing Test Coverage

| Coverage Gap | Risk |
|---|---|
| No test that verifies `chk_blob()` rejects when `backend_root` is swapped to a different epoch's root | F-02 |
| No test that verifies `CheckpointLink` check fails on a forged link (valid IDs, wrong cross-binding) | F-04 |
| No test that verifies identical `AssetStateRoot` from both `Full` and `Incr` root modes after multi-step batch | F-06 |
| No test with no-op `TxProofVerifier` to confirm storage-layer rejection (there is none — documenting gap) | F-05 |
| No test that roundtrips through redb `sync_store` + `load_state` with claim nullifiers and checks root binding | F-01 |
| No test for the `Z00Z_STORAGE_REDB_INJ` env var escape (leaking fault injection into integration tests) | F-07 |

---

## 👍 Positive Findings

The codebase demonstrates several strong security practices:

- ✅ `#![forbid(unsafe_code)]` throughout the entire crate
- ✅ `hash_domain!` macro + `hash_zk` with distinct domain strings used for ALL JMT key derivation and state root computation (17 unique `hash_domain!` declarations found)
- ✅ `ns_key(tree_id, key)` correctly applied at ALL `TreeStore` access points (lines 50, 63, 98 in `tree_store.rs`)
- ✅ `chk_blob()` correctly reconstructs `ns_key(tree_id, raw_key)` for JMT proof verification — no key mismatch between write and verify
- ✅ `serde(deny_unknown_fields)` on `ClaimNullRec` prevents future deserialization confusion
- ✅ Version gates enforced at BOTH encode and decode boundaries for all artifact types
- ✅ Atomic redb write transactions — all 11 tables written inside one `begin_write()` + `commit()`
- ✅ `StoreSnap` rollback capability protects in-memory state on failed commits
- ✅ `BTreeSet` duplicate detection in `check_snapshot()` (paths and asset IDs)
- ✅ In-batch double-spend protection via `spent_seen: BTreeSet` in `apply_batch_checkpoint()`
- ✅ `serial_id.get().to_le_bytes()` in JMT key encoding (platform-independent)
- ✅ Poseidon2-based semantic roots use domain-separated `hash_many::<Domain>` with sorted input ordering (avoidance of order-dependency attacks)
- ✅ Parallelism via `rayon` confined to read-only planning phase; single-threaded atomic commit

---

## 📌 Phase 7 — Deliverable Summary

### Findings Table (ordered by severity)

| ID | Sev | File | Short description | Threat |
|---|---|---|---|---|
| F-01 | **S1** | `redb_backend.rs` | `cp_proof` = `exec_id ∥ state_root` — not a validity proof | T2 |
| F-02 | **S2** | `proof.rs` | `backend_root` in ProofBlob is self-attesting; no semantic↔JMT cross-binding | T1, T5 |
| F-03 | **S2** | `ids.rs`, `codec.rs` ×3 | Undifferentiated `Sha256::digest` for all ID types — no domain labels | T3 |
| F-04 | **S2** | `link.rs` | `CheckpointLink` stores 3 IDs with no cryptographic commitment over the tuple | T5, T6 |
| F-05 | **S2** | `build.rs` | `TxProofVerifier` + `SpentIndex` external hooks — no storage fallback | T4 |
| F-06 | **S2** | `store.rs` | Dual root mode via env var; panic on unrecognized value | T8 |
| F-07 | **S3** | `redb_backend.rs` | `Z00Z_STORAGE_REDB_INJ` fault injector compiled into production binary | T8 |
| F-08 | **S3** | `store.rs` | Claim nullifier hex in error strings leaks cross-chain metadata | T7 |
| F-09 | **S3** | `keys.rs` | `asset_key()` returns raw bytes; inconsistent with `definition_key`/`serial_key` | T1 |
| F-10 | **S4** | `ids.rs` | Bincode no formal stability guarantee as ID commitment source | T3 |
| F-11 | **S4** | `store.rs` | `compute_secret_tag()` is dead code; output blinding unused | T7 |
| F-12 | **S4** | `redb_backend.rs` | Mixed BE/LE endianness in redb key construction vs JMT key construction | — |

---

## 💥 Executive Verdict

`z00z_storage` is a well-structured crate with strong safety discipline (`forbid(unsafe_code)`), consistent use of domain-separated Poseidon2 for authenticated state roots, and correct namespace multiplexing in the JMT layer. The codebase shows clear security-aware architecture in multiple areas.

**However, two structural design gaps require resolution before the checkpoint system can be considered cryptographically sound in production:**

1. **F-01**: The `CheckpointProof.cp_proof` field in every checkpoint artifact written by the storage backend is `exec_id || state_root` — a pair of known identifiers, not a proof. This is by design (`OPAQUE` placeholder), but any downstream component trusting this field as a validity witness is operating on a false assumption. The public API creates a false impression of attestation.

2. **F-02**: The `ProofBlob` carries both a Poseidon2 semantic root and a JMT SHA-256 backend root as independent fields without a cryptographic cross-binding. While co-created atomically at write time, these roots become independently trustworthy at verification time. `chk_blob()` checks the semantic root against an external parameter but verifies JMT membership proofs against the blob's own `backend_root`. A forged blob can pass all checks while proving membership in an unrelated tree epoch.

**S2 findings (F-03 through F-06)** represent design-level weaknesses that do not enable immediate exploitation under SHA-256 assumptions but reduce the depth of the security model. In a production blockchain, where content-addressed IDs form the trust chain, domain-free SHA-256 for all artifact IDs is an avoidable risk.

---

## ✅ Final Decision

| Criterion | Status |
|---|---|
| Unsafe code | ✅ None (`forbid(unsafe_code)`) |
| ZK-friendly root computation | ✅ Poseidon2 domain-separated |
| JMT key namespace separation | ✅ `ns_key()` consistently applied |
| Checkpoint proof validity | ❌ **OPAQUE placeholder — no cryptographic proof** |
| Semantic↔JMT root binding | ❌ Self-attesting — no cross-root commitment |
| ID domain separation | ❌ Raw SHA-256, no domain labels |
| Audit trail integrity | ⚠️ Structural linking only, no cryptographic commitment |
| Production hardening | ⚠️ Fault injector and panic in env-var path |

**Recommendation: CONDITIONAL PASS**

The crate is safe for internal prototype use. Before production deployment, resolve **F-01** (document or replace the OPAQUE proof system with real validity proofs) and **F-02** (add cross-root commitment inside `ProofBlob`). Apply **F-03** (domain labels for all SHA-256 ID derivations) and **F-07** (remove `INJ_ENV` from production binary) as a hard requirement before any externally visible deployment.

---

*Audit by Claude Sonnet 4.6 — 2025-07-14 — z00z_storage crate only — tari/ vendor excluded*
