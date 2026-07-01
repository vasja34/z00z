---
post_title: "Crypto Audit: z00z_storage GPT-5.4"
author1: "GitHub Copilot"
post_slug: "storage-audit-gpt54"
microsoft_alias: "copilot"
featured_image: "none"
categories:
  - "engineering"
tags:
  - "crypto"
  - "audit"
  - "rust"
  - "z00z_storage"
ai_note: "AI-assisted source-only cryptographic and integrity audit of z00z_storage Rust implementation"
summary: "Deep audit of z00z_storage focused on checkpoint proof authenticity, snapshot witness binding, replay resistance, deterministic IDs, and storage-layer cryptographic trust boundaries."
post_date: "2026-03-26"
---

<!-- markdownlint-disable MD041 -->

## Executive Verdict

🚨 Verdict: `Risky but salvageable`.

🚨 Final decision: `Blocked` for security sign-off.

📌 The strongest blocker is not the Merkle or snapshot layer.

📌 The blocker is the checkpoint attestation path in
`crates/z00z_storage/src/checkpoint/artifact.rs`,
`crates/z00z_storage/src/checkpoint/build.rs`,
`crates/z00z_storage/src/assets/store.rs`, and
`crates/z00z_storage/src/assets/store_internal/redb_backend.rs`.

📌 The crate stores and seals final checkpoint artifacts that look like proof-
bearing objects, but the storage layer does not enforce any cryptographic
verification of those proof bytes, and the default persisted proof bytes are
synthetic data derived from `exec_id || state_root`, not a real proof.

📌 Snapshot witness validation, path binding, leaf binding, JMT namespace
separation, and serialization restore checks are materially stronger than the
checkpoint proof story.

## Input Type And Scope

📌 Input type: implementation audit of Rust source code.

📌 Scope: `crates/z00z_storage/src/**/*.rs` only.

📌 Exclusions: tests, non-Rust documents, all other crates except direct type
use, and all vendor code.

📌 Reviewed module families:

- `assets/` for semantic roots, JMT keying, proof blobs, store mutation, and
  claim replay storage
- `checkpoint/` for execution-input replay, draft construction, final artifact
  sealing, and identity derivation
- `snapshot/` for witness-backed pre-state snapshots and replay materialization
- `serialization/` for deterministic JMT export, restore, and identity binding
- storage backend internals for persisted artifact generation and metadata keys

📌 Method: code-only threat-model reconstruction, cryptographic boundary
inspection, proof-binding analysis, replay-path review, and persistence-layer
integrity review.

## Security Goals Extracted From Code

📌 The crate appears to aim for these properties:

| Goal | Status | Notes |
| --- | --- | --- |
| Deterministic semantic state root over assets, serial buckets, and definitions | ✅ | `assets/model.rs` uses Poseidon2 plus explicit domain separation |
| Existence proofs for concrete asset paths against the backend JMT state | ✅ | `assets/proof.rs` checks definition, serial, and asset proofs against a shared backend root |
| Snapshot replay should be bound to exact path, leaf, and root | ✅ | `snapshot/store.rs` replays `ProofBlob` and revalidates it |
| Checkpoint apply should reject malformed membership witnesses and mismatched pre-state | ✅ | `checkpoint/build.rs` verifies resolved inputs and membership witness consistency |
| Final checkpoint artifacts should represent authenticated state transitions | ❌ | final artifact sealing accepts non-empty opaque bytes without in-crate proof verification |
| Claim nullifier storage should provide replay resistance at the storage boundary | ⚠️ | uniqueness exists, but it is keyed by raw strings rather than canonical binary nullifiers |
| Artifact IDs should be deterministic and content-derived | ✅ | snapshot, checkpoint, exec, and JMT serialization IDs are all derived from canonical bytes |

## Inferred Threat Model

📌 Relevant adversaries for `z00z_storage` are:

| Adversary | Capability | Relevant Surface |
| --- | --- | --- |
| Malicious caller inside the same system boundary | Can provide synthetic execution inputs, opaque proof bytes, and non-canonical replay metadata | `checkpoint/`, `assets/store.rs` |
| Local attacker with storage access | Can tamper with persisted artifacts and attempt cross-file inconsistency | backend persistence, serialization restore |
| Protocol participant trying to replay one already-consumed claim | Can vary nullifier encoding or metadata shape | claim nullifier persistence |
| Integrator treating storage artifacts as stronger than they are | Can assume `CheckpointArtifact` implies verified proof-bearing finality | `checkpoint/artifact.rs`, persisted checkpoint store |

📌 Trust assumptions visible in code:

- snapshot entries are treated as cryptographically meaningful and are validated
  in-crate
- tx proof checking during checkpoint build is delegated to an external
  `TxProofVerifier`
- final checkpoint proof bytes are treated as opaque payloads and are not
  validated in-crate
- claim replay safety currently assumes upstream callers canonicalize
  nullifiers before they hit storage

📌 The crate does not publish an explicit in-code statement that
`CheckpointArtifact` is only an opaque transport wrapper and must not be used as
proof of validity on its own.

## Critical And High Findings

### S1-01: Final Checkpoint Artifacts Can Carry Synthetic, Unverified Proof Bytes

📌 Components:

- `crates/z00z_storage/src/checkpoint/artifact.rs`
- `crates/z00z_storage/src/checkpoint/build.rs`
- `crates/z00z_storage/src/assets/store.rs`
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`

📌 The final-checkpoint path has three separate facts that become dangerous when
composed:

1. `CheckpointProof::new(...)` only checks that the proof-system tag is known
   and `cp_proof` is non-empty.
2. `CheckpointDraft::finalize(...)` only checks that `proof.pub_in == draft.pub_in()`.
3. `RedbBackend::sync_store(...)` manufactures persisted proof bytes with
   `proof_bytes(exec_id, state_root)`, and that helper is literally:

```rust
fn proof_bytes(exec_id: CheckpointExecInputId, state_root: AssetStateRoot) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(64);
    bytes.extend_from_slice(exec_id.as_bytes());
    bytes.extend_from_slice(state_root.as_bytes());
    bytes
}
```

📌 The artifact type is named and shaped like a cryptographic final proof
container, but the storage layer never proves that `cp_proof` is valid for the
stated transition.

📌 This is not a mere documentation gap.

📌 The backend itself persists synthetic bytes that are not a proof system at
all. The only barrier is `non-empty`.

📌 Impact:

- any downstream component that treats `CheckpointArtifact` as an authenticated
  proof-bearing object can be misled
- proof bytes are not cryptographically bound in-crate to the full checkpoint
  statement
- `height`, `snap_id`, and the exact execution transcript are not verified as
  part of any final-proof check in this crate
- the persisted artifact can create a false sense of finality, auditability, or
  transport safety

📌 Why this is high severity:

- the object is not just advisory metadata; it is a persisted canonical final
  artifact class
- the backend automatically generates these artifacts, so this is an ambient
  integrity risk, not a niche misuse path
- the proof-like shape materially exceeds the security actually enforced

📌 Required fix:

1. Replace `CheckpointProofSystem::OPAQUE` as the only production path with a
   proof suite or attestation suite that has a verifier at seal/load time.
2. If the crate is intentionally not a verifier, rename the field and type so
   they do not imply proof validity.
3. Stop auto-generating synthetic proof bytes in `sync_store(...)`.
4. Bind the final attestation to the exact statement. At minimum that statement
   should include `pub_in`, `exec_id`, and the artifact version. If height is
   consensus-relevant, bind height too.

📌 Confidence: High.

### S1-02: Stored Checkpoint Execution Inputs Use Placeholder Tx Proof Bytes

📌 Component: `crates/z00z_storage/src/assets/store.rs` — `build_exec(...)`.

📌 The storage path persists checkpoint execution inputs via `build_exec(...)`.

📌 For each operation it emits one tx with proof bytes built as:

```rust
vec![(index as u8).saturating_add(1)]
```

📌 Those bytes are placeholders, not authenticated per-tx proof material.

📌 They also collide once the batch index exceeds the `u8` range, and they do
not preserve the original proof semantics of the state transition.

📌 In the same crate, `build_cp_draft(...)` delegates tx-proof verification to an
external `TxProofVerifier`, which is a good abstraction boundary for the build
phase, but `build_exec(...)` shows that the persisted execution-input artifact is
not a trustworthy replay of actual proof-carrying tx packages.

📌 Impact:

- the persisted exec artifact cannot safely function as an authoritative replay
  transcript for tx-proof-carrying operations
- a consumer can incorrectly assume that reloaded exec inputs preserve original
  proof material
- combined with S1-01, this makes the whole checkpoint artifact stack appear
  stronger than it is

📌 Required fix:

1. Persist actual proof bytes if the exec artifact is meant to be a canonical
   replay object.
2. If no real tx proof exists at this layer, use a different artifact class
   name that explicitly marks the payload as synthetic plan data.
3. Do not feed placeholder bytes into any path that later looks proof-bearing or
   audit-grade.

📌 Confidence: High.

## Medium Findings

### S2-01: Claim Replay Resistance Depends On Raw String Equality, Not Canonical Nullifier Semantics

📌 Components:

- `crates/z00z_storage/src/assets/store.rs` — `ClaimNullTx`, `ClaimNullRec`,
  `claim_rows_for(...)`
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` —
  `claim_null_key(...)`

📌 Claim replay prevention currently uses `nullifier_hex: String` as the key.

📌 Duplicate rejection is implemented by string equality in memory and by
string-key persistence in Redb.

📌 The storage layer does not validate that the string is:

- fixed-width hex
- lowercase canonical hex
- exactly one encoding of one binary nullifier
- domain-separated by chain or protocol if that is required upstream

📌 The persisted key is also just:

```rust
version || nullifier_hex.as_bytes()
```

📌 Impact:

- replay safety depends on upstream canonicalization rather than storage-layer
  enforcement
- if callers can submit the same binary nullifier in multiple textual forms,
  uniqueness can be bypassed or, at minimum, become inconsistent across paths
- if `chain_id` should scope the nullifier, the current keying ignores it even
  though the record stores it

📌 Required fix:

1. Replace `nullifier_hex: String` as the storage key with a validated binary
   newtype such as `[u8; 32]`.
2. Parse and canonicalize once at the boundary.
3. Decide explicitly whether replay uniqueness is global or `(chain_id,
   nullifier)` scoped, and encode that in the key.

📌 Confidence: Medium-high.

### S2-02: Claim Persistence Stores High-Signal Privacy Metadata In Cleartext

📌 Component: `crates/z00z_storage/src/assets/store.rs` — `ClaimNullRec`.

📌 The replay table stores these fields verbatim:

- `nullifier_hex`
- `claim_id_hex`
- `owner_hex`
- `tx_digest_hex`
- `chain_id`
- `created_at_seq`

📌 For a privacy-focused system, this is a large amount of correlatable metadata
to persist in one place.

📌 This is not a remote cryptographic break by itself, but it is a material
privacy-surface expansion if local backups, debug exports, or forensic dumps are
exposed.

📌 Required fix:

1. Keep only what is strictly needed for replay defense.
2. Consider storing a binary nullifier key plus a compact status record instead
   of owner and tx correlation fields.
3. If the extra metadata is needed operationally, separate it from the replay
   key store and treat it as explicitly sensitive telemetry.

📌 Confidence: Medium.

### S2-03: Checkpoint Proof Statement Is Under-Bound Even Before Real Verification Exists

📌 Components:

- `crates/z00z_storage/src/checkpoint/artifact.rs`
- `crates/z00z_storage/src/checkpoint/exec_input.rs`
- `crates/z00z_storage/src/checkpoint/store.rs`

📌 The typed public inputs reconstructed by `CheckpointDraft::pub_in()` include:

- `prev_root`
- `new_root`
- `spent_delta`
- `created_delta`

📌 They do not include:

- `height`
- `prep_snapshot_id`
- `exec_input_id`

📌 Those values are persisted elsewhere and checked by helper functions such as
`check_link_ids(...)` and `check_exec_root(...)`, but they are not part of the
typed final-proof statement.

📌 If checkpoint validity is intended to mean more than state-root transition
correctness, the final attestation surface is incomplete.

📌 Required fix:

1. Decide whether the proof statement is only about state transition or also
   about replay source and sequencing.
2. If replay source or height matters, bind them into the attested statement.
3. Keep the statement definition in one place and version it explicitly.

📌 Confidence: Medium-high.

## Low And Informational Findings

### S3-01: The Explicit Threat Model For Checkpoint Proof Semantics Is Missing In Code

📌 The implementation clearly distinguishes strong snapshot witness checking from
weak checkpoint proof enforcement, but that difference is not stated as an
explicit threat-model boundary in the source.

📌 This increases the chance that integrators misread the artifact types and
trust levels.

📌 Fix: document, in code comments or module docs, whether `CheckpointArtifact`
is a verified proof container, an opaque transport envelope, or a storage-local
placeholder awaiting external validation.

📌 Confidence: High.

### S3-02: Artifact Identity Depends On Shared Bincode Semantics

📌 Snapshot IDs, exec IDs, checkpoint IDs, and JMT serialization IDs are all
deterministic and correctly content-addressed, which is a strength.

📌 They also implicitly depend on the stability of the shared `BincodeCodec`
configuration across versions.

📌 This is acceptable for local canonical storage, but it should remain an
explicit compatibility consideration if artifact IDs ever cross release or
network boundaries.

📌 Confidence: Medium.

## Positive Security Properties

✅ `assets/model.rs` uses explicit domain separation and Poseidon2 hashing for
semantic roots instead of ad-hoc concatenation.

✅ `assets/keys.rs` and `assets/store.rs` namespace JMT keys by logical tree,
which avoids collisions between definition, serial, asset, and path-index data.

✅ `assets/proof.rs` does real existence-proof checking for definition, serial,
and asset branches against one shared backend root.

✅ `snapshot/store.rs` is materially strong.

✅ Snapshot validation checks root, path, serial, asset id, leaf equality, and
then re-runs `chk_blob(...)` over the witness payload.

✅ `serialization/restore.rs` and the JMT serialization layer are integrity-
oriented rather than dump-only. They validate topology and root binding instead
of trusting serialized shape blindly.

✅ Content-addressed IDs for snapshots, exec inputs, checkpoint artifacts, and
JMT serialization artifacts are deterministic and derived from canonical bytes.

## Open Ambiguities

❓ It is unclear whether any downstream component already treats
`CheckpointArtifact` as only an opaque envelope and never as cryptographic proof
of validity.

❓ It is unclear whether upstream callers already canonicalize nullifiers before
constructing `ClaimNullTx`.

❓ It is unclear whether identical nullifiers on different `chain_id` values are
supposed to be globally conflicting or independently valid.

❓ It is unclear whether `build_exec(...)` is intended only for synthetic local
checkpointing or whether it is meant to survive as a canonical replay object.

## Concrete Fixes

### Fix 1: Separate Verified Checkpoint Proofs From Synthetic Storage Artifacts

📌 Recommended direction:

1. Introduce a proof or attestation trait for final artifact sealing.
2. Verify it at seal time or load time.
3. Bind a versioned statement hash over the exact intended semantics.
4. If real proof verification is out of scope for `z00z_storage`, rename the
   types and fields to remove proof-validity implications.

### Fix 2: Stop Persisting Placeholder Tx Proof Bytes

📌 Replace `vec![(index as u8).saturating_add(1)]` with one of these paths:

1. actual upstream tx proof bytes
2. a typed synthetic-plan marker that cannot be confused with proof material
3. no proof field at all for synthetic local execution transcripts

### Fix 3: Canonicalize Nullifiers At The Storage Boundary

📌 Replace string keys with a validated binary newtype and make replay uniqueness
an explicit protocol rule.

### Fix 4: Minimize Claim Metadata Retention

📌 Keep the replay store focused on what replay defense actually needs and move
optional correlation fields into a separate, explicitly sensitive layer.

## Test Plan

📌 Required validation before sign-off:

1. Add a negative test showing that arbitrary `cp_proof` bytes cannot be sealed
   into a production-grade final artifact once the verifier boundary is added.
2. Add a load-time verification test that tampered checkpoint proof statements
   are rejected.
3. Add canonicalization tests proving that alternate textual encodings of the
   same nullifier collapse to one storage key.
4. Add a scope test for `(chain_id, nullifier)` if chain scoping is required.
5. Add a test proving that persisted exec artifacts preserve real proof bytes if
   they remain part of the canonical replay model.

📌 Recommended regression tests for already-strong areas:

1. Keep snapshot root/path/leaf/witness binding tests.
2. Keep JMT serialization restore root-binding tests.
3. Keep proof-blob corruption tests for definition, serial, and asset proof
   branches.

## Confidence And Final Decision

📌 Highest-confidence findings:

- final checkpoint proof bytes are not verified in-crate
- Redb persistence auto-generates synthetic proof bytes from `exec_id` and
  `state_root`
- persisted exec inputs use placeholder tx proof bytes
- claim replay storage keys uniqueness by raw strings rather than validated
  binary nullifiers

📌 Strongest positive controls:

- snapshot witness validation
- proof-blob branch verification
- namespaced JMT keying
- deterministic content-addressed artifact IDs
- serialization restore integrity checks

🚨 Final decision: `Blocked`.

🚨 The crate has a solid integrity core for asset state, witness replay, and JMT
serialization.

🚨 It does not yet have a trustworthy final checkpoint proof model.

🚨 Until the checkpoint artifact surface is either genuinely verified or renamed
to reflect its weaker semantics, `z00z_storage` should not be signed off as a
cryptographically trustworthy finality or proof-persistence layer.
