# Recursive-Ready Checkpoint Contract Scenarios

[TOC]

Date: 2026-07-03

Status: new scenario backlog for the replay substrate and
recursive-ready checkpoint contract.

Spec authority: `docs/tech-papers/Recursive-Ready-Checkpoint-Contract.md`

Required config path: `config/checkpoint_contract.yaml`

## 1. Scenario Boundary

These scenarios MUST prove the recursive-ready checkpoint contract without
claiming live recursive admission, live external DA, or complete post-quantum
security.

The scenarios MUST treat compact delta storage as replay, witness, archive,
fraud, audit, and recovery substrate. They MUST NOT treat delta artifacts as the
final proof theorem.

## 2. Current Code Anchors

| Area | Code anchor | Scenario use |
| --- | --- | --- |
| Final checkpoint artifact | `z00z_storage::checkpoint::CheckpointArtifact` | Prove current authoritative artifact lane. |
| Proof system discriminator | `z00z_storage::checkpoint::CheckpointProofSystem` | Keep `OPAQUE_ATTEST` live and `VERIFIED` gated. |
| Execution input | `z00z_storage::checkpoint::CheckpointExecInput` | Preserve exact transaction proof bytes. |
| Checkpoint link | `z00z_storage::checkpoint::CheckpointLink` | Prove continuity and reject detached artifacts. |
| Settlement proofs | `z00z_storage::settlement` batch proof surfaces | Reuse witness DAG and batch proof compression. |
| DA/local publication | `z00z_rollup_node` local DA surfaces | Bind availability refs without treating DA as validity. |
| Validators | `z00z_validators` checkpoint consumers | Consume storage artifacts; MUST NOT duplicate theorem logic. |

## 3. Required Implementation Surfaces

| Area | Required surface | Scenario use |
| --- | --- | --- |
| Config gate | `config/checkpoint_contract.yaml` plus storage-owned validator | Load and reject unsafe recursive-ready policy. |
| Recursive sidecar | `RecursiveCheckpointSidecarV1` or versioned equivalent | Bind mock/adapter proof evidence to the same statement. |
| Recursive verifier API | Storage/proof adapter verifier facade | Reject wrong statement digest, wrong chain link, and unsupported backend. |
| DA reference object | `CheckpointDaReferenceV1` or versioned equivalent | Bind provider locator to `statement_core_digest` and archive manifest root. |
| Archive manifest object | `CheckpointArchiveManifestV1` or versioned equivalent | Freeze retained archive content independently from local filesystem layout. |
| PQ audit anchor | `PostQuantumCheckpointAnchorV1` or versioned equivalent | Enforce cadence-bound audit envelope. |
| Measurement record | Versioned proof measurement object | Record proof size, prover time, verifier time, memory, witness size, and chain length. |
| Authority-promotion gate | `authority_promotion.stage` plus authority-promotion evidence | Prevent stage skips and premature recursive authority. |
| Path and limit gates | `paths.*`, `limits.*`, and PQ stage gate fields | Reject unsafe filesystem and size policy before writes occur. |

## 4. Scenario 12: Canonical Checkpoint With Delta Substrate

### 4.1 Purpose

Prove that the canonical branch remains authoritative while compact delta
artifacts are retained as replay and dispute material.

### 4.2 Given

- Repository config validates through the storage-owned checkpoint contract config validator.
- Canonical branch is enabled and authoritative.
- Recursive branch is enabled but non-authoritative.
- Input gates MUST require statement fields, execution input ID, preparation
  snapshot ID, DA reference, and exact transaction proof bytes.

### 4.3 When

The scenario builds a local checkpoint from package input through execution
input, touched deltas, `statement_core_digest`, DA export, archive manifest,
final statement digest, opaque artifact, and checkpoint link.

### 4.4 Then

- The checkpoint artifact MUST use opaque attest semantics.
- The checkpoint link MUST preserve root continuity.
- Exact transaction proof bytes MUST remain present in canonical replay.
- Delta and witness archives MUST be retained through dispute policy.
- The scenario MUST fail if any input, output, or artifact gate is weakened.

### 4.5 Verification Anchors

The implementation MUST add a storage integration test for the config gate and
MUST keep `cargo test -p z00z_storage --lib -- --nocapture` green.

## 5. Scenario 13: Recursive Shadow Chain Is Non-Authoritative

### 5.1 Purpose

Prove that a recursive sidecar can bind the same statement digest without
becoming checkpoint admission authority.

### 5.2 Given

- The same `CheckpointTransitionStatementV1` used by the canonical branch.
- Recursive branch mode is `shadow_mock`.
- `branches.recursive.is_authoritative` is false.
- `branches.recursive.min_chain_steps >= 3`.
- `branches.recursive.target_chain_steps >= min_chain_steps`.

### 5.3 When

The scenario builds a 3 to 5 step mock recursive chain using prior-output
binding:

```text
prior_recursive_output_root == next_statement.prev_root
```

### 5.4 Then

- The sidecar MUST bind the same statement digest as the canonical artifact.
- The sidecar MUST record backend label, chain length, proof bytes digest, and
  measurements.
- The sidecar MUST NOT pass as `CheckpointProofSystem::VERIFIED`.
- A wrong statement digest MUST reject.
- A wrong prior-output root MUST reject.
- A sidecar marked authoritative MUST reject.

### 5.5 Verification Anchors

The implementation MUST add focused tests for wrong statement digest, wrong
public input digest, wrong prior-output binding, unsupported backend, and
missing measurements.

## 6. Scenario 14: PQ Audit Anchor Cadence

### 6.1 Purpose

Prove that post-quantum audit anchors are produced at configured cadence and do
not overclaim full post-quantum proof security.

### 6.2 Given

- `post_quantum.is_enabled` is true.
- `post_quantum.cadence_blocks` is 1000 by repository policy.
- `authority_promotion.stage` is `pq_anchor_writer` or later.
- `post_quantum.enforce_live_cadence` is true.
- MUST-have PQ artifacts are configured.

### 6.3 When

The scenario checks heights 999, 1000, 1001, and 2000.

### 6.4 Then

- Height 1000 MUST require a PQ audit anchor.
- Height 2000 MUST require a PQ audit anchor.
- Heights 999 and 1001 MUST NOT require a PQ audit anchor.
- Missing `pq_statement_digest`, `pq_delta_root`, `pq_witness_root`,
  `pq_archive_manifest_root`, or `pq_signature_or_commitment` MUST reject.
- The scenario MUST describe the anchor as audit and migration evidence, not as
  complete PQ recursive validity.

### 6.5 Verification Anchors

The implementation MUST add tests that verify cadence at 999, 1000, 1001, and
2000 only when live cadence enforcement is active and without claiming complete
PQ recursive security.

## 7. Scenario 15: Archive And Dispute Retention Fail Closed

### 7.1 Purpose

Prove that recursive readiness does not permit early deletion of raw, witness,
or transaction-proof material.

### 7.2 Given

- Raw transaction packages are `archive_required`.
- Witness data is `archive_required`.
- Transaction proof bytes are `canonical_until_verified_backend`.
- DA blobs are `da_required_until_pruned`.

### 7.3 When

The scenario attempts to weaken any retention mode before verified backend
authority promotion and before dispute policy permits pruning.

### 7.4 Then

- Raw transaction package weakening MUST reject.
- Witness data weakening MUST reject.
- Transaction proof byte weakening MUST reject.
- DA blob weakening MUST reject.
- Compact metadata MUST remain permanent.

### 7.5 Verification Anchors

The implementation MUST add retention weakening tests for raw packages,
witness data, transaction proof bytes, compact metadata, and DA blobs.

## 8. Scenario 16: Future Verified Branch Authority-Promotion Gate

### 8.1 Purpose

Prove that the future verified branch MUST NOT become authoritative until the
full authority-promotion evidence exists.

### 8.2 Given

- `CheckpointProofSystem::VERIFIED` exists conceptually.
- Live codec currently accepts the opaque branch.
- Recursive branch is mock/shadow only.

### 8.3 When

The scenario attempts to promote a verified recursive proof without stable
proof object, verifier API, codec support, negative tests, benchmark evidence,
and rollback policy.

### 8.4 Then

- Authority promotion MUST reject.
- The canonical statement MUST remain unchanged.
- Validators MUST consume storage-owned artifacts.
- Validators MUST NOT define a second checkpoint theorem.
- Authority-promotion stage MUST NOT move to `verified_backend_enabled` without proof
  object, verifier API, codec, negative tests, benchmarks, and rollback policy.
- The failure report MUST name the missing authority-promotion gates.

### 8.5 Verification Anchors

The implementation MUST add authority-promotion tests for authority-promotion
stage skips, premature recursive authority, and clippy coverage when the
verified branch code surface exists.

## 9. Scenario 17: Mixed-Era Artifact Rejection

### 9.1 Purpose

Prove that artifacts from different proof eras, codecs, versions, or branch
policies fail closed unless an explicit compatibility adapter exists.

### 9.2 Given

- Canonical artifacts use opaque attest semantics.
- Recursive sidecars are non-authoritative.
- Future verified artifacts are not enabled.

### 9.3 When

The scenario injects an unsupported proof-system discriminator, unsupported
version, missing replay ID, mismatched root, mismatched DA ref, or sidecar
presented as authority.

### 9.4 Then

- Unsupported proof system MUST reject.
- Unsupported version MUST reject.
- Missing replay ID MUST reject.
- Root mismatch MUST reject.
- DA reference mismatch MUST reject.
- Sidecar-as-authority MUST reject.

### 9.5 Verification Anchors

The implementation MUST add codec and validator tests for unsupported proof
system, unsupported version, missing replay ID, root mismatch, DA ref mismatch,
and sidecar-as-authority.

## 10. Implementation Order

1. Keep the YAML config gate and typed validator green.
2. Freeze canonical byte framing, golden vectors, DA reference object, and archive manifest object.
3. Extend canonical statement fields without changing current authority.
4. Add sidecar object, mock verifier API, codec, and negative tests.
5. Add measurement metadata and 3 to 5 step chain tests.
6. Add PQ anchor object, cadence writer, and PQ stage-gate tests.
7. Add path and limit gate tests before any implementation claims verified readiness.
8. Add local E2E harness only after the lower-level storage gates are tested.

## 11. Anti-Drift Rules

- MUST NOT claim recursive proof authority before verified codec and verifier
  support exist.
- MUST NOT remove exact transaction proof bytes from canonical replay.
- MUST NOT remove raw or witness data before dispute and audit policy allows it.
- MUST NOT treat DA publication as checkpoint validity.
- MUST NOT treat provider-local paths as authoritative archive-manifest bytes.
- MUST NOT claim canonical digests without golden-vector coverage.
- MUST NOT allow absolute or traversing checkpoint-contract paths in V1.
- MUST NOT describe PQ audit anchors as complete PQ recursive proof security.
- MUST NOT add a checkpoint theorem outside storage-owned checkpoint artifacts.

## 12. Scenario 18: Canonical Digest And Golden-Vector Contract

### 12.1 Purpose

Prove that the statement digest and every committed root are derived from one
stable canonical byte contract.

### 12.2 Given

- A fixed checkpoint batch with ordered execution rows.
- A fixed delta journal emission order.
- A fixed witness archive entry order.
- Versioned `CheckpointDaReferenceV1` and `CheckpointArchiveManifestV1`
  examples.

### 12.3 When

The scenario computes `statement_core_digest`, `statement_digest`,
`tx_data_root`, `delta_root`, `witness_root`, `da_ref`, and
`archive_manifest_root` twice from the same authoritative bytes and then
recomputes after tampering one field at a time.

### 12.4 Then

- Repeated canonical computation MUST produce identical digests.
- Any tamper in row order, delta order, witness order, framing, or byte content
  MUST change the corresponding digest and reject dependent artifacts.
- JSON formatting differences MUST NOT change authoritative digests.
- Provider-local path hints MUST NOT change `archive_manifest_root`.

### 12.5 Verification Anchors

The implementation MUST add golden-vector tests for `statement_core_digest`,
`statement_digest`, `tx_data_root`, `delta_root`, `witness_root`,
`CheckpointDaReferenceV1`, and `CheckpointArchiveManifestV1`.

## 13. Scenario 19: DA Reference And Archive Manifest Binding

### 13.1 Purpose

Prove that DA publication is bound through versioned objects rather than through
loose locator strings.

### 13.2 Given

- `CheckpointDaReferenceV1` binds `statement_core_digest`,
  `archive_manifest_root`, and `payload_commitment`.
- `CheckpointArchiveManifestV1` binds archive entries with version, ordinal,
  digest, byte length, encoding kind, and retention class.

### 13.3 When

The scenario mutates locator value, payload commitment, archive manifest root,
`statement_core_digest`, or one archive entry while keeping the rest of the package
apparently plausible.

### 13.4 Then

- DA reference digest mismatch MUST reject.
- Archive manifest root mismatch MUST reject.
- Locator-only publication without statement binding MUST reject.
- Any V1 DA reference or archive manifest that binds artifact-derived
  `checkpoint_id` MUST reject.
- Archive entry byte-length drift MUST reject.
- Provider-specific extension fields MUST NOT silently change authoritative V1
  meaning.

### 13.5 Verification Anchors

The implementation MUST add object roundtrip and negative tests for DA
reference, archive manifest, archive entry, and payload-commitment mismatch.

## 14. Scenario 20: Path, Limit, And PQ Stage Gates

### 14.1 Purpose

Prove that filesystem policy, size bounds, and PQ target-vs-live enforcement all
fail closed before any checkpoint write path becomes authoritative.

### 14.2 Given

- `paths.*` are configured for every authoritative write surface.
- `limits.*` are configured for batch, witness, and recursive-proof bounds.
- `post_quantum.enforcement_stage` is `pq_anchor_writer`.

### 14.3 When

The scenario injects an absolute path, a `..` traversal path, a colliding write
path, a zero limit, an overflowed limit, `post_quantum.enforce_live_cadence`
before `pq_anchor_writer`, and missing live cadence enforcement at or after
`pq_anchor_writer`.

### 14.4 Then

- Absolute paths MUST reject.
- Traversal paths MUST reject.
- Colliding authoritative write paths MUST reject.
- Zero or overflowed limits MUST reject.
- Live PQ cadence enforcement before `pq_anchor_writer` MUST reject.
- Missing live PQ cadence enforcement at or after `pq_anchor_writer` MUST
  reject.

### 14.5 Verification Anchors

The implementation MUST add config tests for path normalization, path
collision, limit overflow, zero bounds, and PQ stage-gate transitions.
