# Z00Z HJMT Fixture Checklist

This document extracts the minimum conformance-safe fixture inventory from
[Z00Z-HJMT-Upgrade.md](Z00Z-HJMT-Upgrade.md). It is a standalone execution
checklist for golden, tamper, carry-forward, crash, and failover vectors.

Canonical naming authority for object, struct, trait, function, and
fixture-family names lives in [Z00Z-HJMT-Key-Terms.md](../Z00Z-HJMT-Key-Terms.md).
This checklist keeps exact wire-format `V1` symbols and exact fixture IDs.

The checklist starts with `ShardRouteTableV1` because route-table bytes are the
first public multi-shard contract. `ShardRootLeafV1`,
`CheckpointPublicationV1`, and generation-1 proof fixtures all bind that
contract transitively.

Completion status below reflects the live repository evidence packet as of
2026-06-16.

If this checklist and the upgrade specification ever disagree, the normative
rules in [Z00Z-HJMT-Upgrade.md](Z00Z-HJMT-Upgrade.md) win.

## Completion Contract

One checklist item is complete only when all of the following exist for that
fixture:

| Artifact | Requirement |
| --- | --- |
| Input description | Exact field values, tags, generation numbers, and preconditions used to build the fixture. |
| Canonical bytes | The exact serialized bytes accepted as authoritative for the fixture. |
| Expected digest or root | The recomputed digest, root, or checkpoint hash that the verifier must match. |
| Expected verdict | Explicit `accept`, `parser_reject`, `verifier_reject`, or `recovery_reject` outcome. |
| Regeneration command | One documented command or harness entry that reproduces the fixture bytes. |
| Evidence pointer | One report, test output, or checked-in manifest proving the fixture was exercised. |

Tamper fixtures additionally MUST record which byte or field was mutated.
Failover and crash fixtures additionally MUST record the cut point, the durable
artifacts that survived, and the only legal recovery outcome.

## Ordered Fixture Inventory

### 1. `ShardRouteTableV1` Golden Vectors

- [x] `SRT-G-001` Generation-0 bridge table.
  MUST contain one `ShardId`, one inclusive `0x00..00` through `0xFF..FF`
  range, `previous_generation_digest = None`, one activation checkpoint, exact
  canonical bytes, and the recomputed `route_table_digest`.

- [x] `SRT-G-002` Split-generation table.
  MUST contain at least two shard IDs in canonical ascending order, gap-free
  ascending `range_rules`, one activation checkpoint, and one
  `previous_generation_digest` that points to the prior committed table.

- [x] `SRT-G-003` Historical old/new migration pair.
  MUST contain the old table bytes, the new table bytes, both digests, and the
  activation checkpoint used by route-migration verification.

- [x] `SRT-G-004` Deterministic re-encode vector.
  MUST prove that the same logical route table re-encodes to the same bytes and
  digest across two independent encoder paths.

### 2. `ShardRouteTableV1` Tamper Vectors

- [x] `SRT-T-001` Unsorted `shard_set` rejects.
  MUST mutate only shard ordering and MUST end in `parser_reject` or canonical
  validation reject.

- [x] `SRT-T-002` Duplicate `ShardId` in `shard_set` rejects.
  MUST prove that uniqueness is enforced before digest acceptance.

- [x] `SRT-T-003` Unsorted `range_rules` reject.
  MUST mutate only `start_hash` ordering.

- [x] `SRT-T-004` Overlapping `range_rules` reject.
  MUST prove that overlapping inclusive ranges are not accepted.

- [x] `SRT-T-005` Gap in route coverage rejects.
  MUST prove that version-1 route coverage is complete over the full 32-byte
  route-hash space.

- [x] `SRT-T-006` Foreign `shard_id` reference rejects.
  MUST prove that one `range_rule` cannot point to a shard outside `shard_set`.

- [x] `SRT-T-007` Wrong `previous_generation_digest` rejects in migration flow.
  MUST prove that historical route linkage is verified, not ignored.

- [x] `SRT-T-008` Digest mismatch rejects.
  MUST prove that verifier-side digest recomputation is authoritative over any
  producer-supplied digest field.

### 3. `ShardRootLeafV1` Golden Vectors

- [x] `SRL-G-001` One-shard bridge leaf.
  MUST bind one `shard_id`, one `shard_root`, one `routing_generation`, one
  `route_table_digest`, one `policy_set_digest`, one `journal_checkpoint`, one
  `local_sequence`, and zeroed reserved transition bits.

- [x] `SRL-G-002` Same-generation changed leaf.
  MUST prove strict monotonic increase of the published
  `(shard_epoch, local_sequence, journal_checkpoint)` tuple when the shard leaf
  changes.

- [x] `SRL-G-003` Transition-state leaf.
  MUST exercise `transition_flags` for one legal in-progress transition and
  prove exact canonical bytes for that flag state.

- [x] `SRL-G-004` Deterministic re-encode vector.
  MUST prove identical bytes and identical Merkle leaf hash across two encoder
  paths.

### 4. `ShardRootLeafV1` Tamper Vectors

- [x] `SRL-T-001` Reserved transition bits set outside `0..2` reject.

- [x] `SRL-T-002` Stale `route_table_digest` rejects.
  MUST prove that a shard leaf bound to one route table is not accepted under a
  different one.

- [x] `SRL-T-003` Stale `shard_epoch` replay rejects.

- [x] `SRL-T-004` Decreasing `journal_checkpoint` rejects.

- [x] `SRL-T-005` Decreasing `local_sequence` rejects.

- [x] `SRL-T-006` Digest mismatch rejects.
  MUST prove that the canonical leaf hash is recomputed from bytes rather than
  trusted by producer claim.

### 5. `CheckpointPublicationV1` Golden Vectors

- [x] `CPP-G-001` First generation-1 bridge publication.
  MUST contain the final visible generation-0 root as `prior_public_root`, one
  shard leaf, one `route_table_digest`, exact canonical bytes, and the expected
  publication digest.

- [x] `CPP-G-002` Carry-forward unchanged shard leaf.
  MUST prove that one unchanged `ShardRootLeafV1` is carried forward byte-for-
  byte into the next checkpoint.

- [x] `CPP-G-003` Changed-one-shard publication.
  MUST prove that only the changed shard leaf bytes differ while unaffected
  shard leaves remain byte-identical.

- [x] `CPP-G-004` Route-generation transition publication.
  MUST bind the new `route_table_digest`, the activation checkpoint, and the
  full post-migration shard-leaf set.

- [x] `CPP-G-005` Prior-root chain vector.
  MUST prove that `prior_public_root` equals the immediately preceding visible
  public root, not a locally durable unpublished shard state.

### 6. `CheckpointPublicationV1` Tamper Vectors

- [x] `CPP-T-001` Reordered shard leaves reject.

- [x] `CPP-T-002` Missing active shard leaf rejects.

- [x] `CPP-T-003` Duplicate shard leaf rejects.

- [x] `CPP-T-004` `route_table_digest` mismatch between publication and shard
  leaves rejects.

- [x] `CPP-T-005` Wrong `prior_public_root` rejects.

- [x] `CPP-T-006` Invalid `root_generation_tag` or `publication_mode_tag`
  rejects.

- [x] `CPP-T-007` One-byte mutation of a carried-forward unchanged shard leaf
  rejects.

### 7. Failover, Carry-Forward, And Crash Vectors

- [x] `FOV-001` Hot-standby same-lineage resume accepts.
  MUST prove resume under the same `ShardId`, the same `routing_generation`,
  and the same durable journal lineage.

- [x] `FOV-T-001` Wrong journal lineage rejects.
  MUST prove that a standby cannot publish under the same `ShardId` with a
  different lineage.

- [x] `FOV-T-002` Same local root replayed under the wrong
  `routing_generation` rejects.

- [x] `FOV-G-002` Failed-shard carry-forward publication accepts.
  MUST prove that unaffected shard leaves carry forward unchanged while the
  failed shard contributes its last published shard leaf byte-for-byte.

- [x] `FOV-G-003` Crash after shard-local durable commit and before public
  publication recovers lawfully.
  MUST prove that recovery exposes only the prior visible public root or the
  exact later root reconstructed from durable journal state.

- [x] `FOV-G-004` Crash during route migration resolves lawfully.
  MUST prove that recovery either completes the committed migration or keeps the
  prior public root without exposing a partial public state.

### 8. `BatchProofBlobV1` Golden Vectors

- [x] `BPB-G-001` Inclusion-family batch vector.
  MUST contain canonical `InclusionOpeningV1` payload bytes and one accepted
  root.

- [x] `BPB-G-002` Non-existence-family batch vector.
  MUST contain canonical `NonExistenceOpeningV1` payload bytes and one accepted
  root.

- [x] `BPB-G-003` Deletion-family batch vector.
  MUST contain canonical `DeletionFactV1` payload bytes and one accepted root.

- [x] `BPB-G-004` Clustered witness-reuse vector.
  MUST prove byte-for-byte stable witness reuse for nearby paths.

- [x] `BPB-G-005` Scattered witness-reuse vector.
  MUST prove stable reference indexing for non-adjacent paths under the same
  root.

### 9. `BatchProofBlobV1` Tamper Vectors

- [x] `BPB-T-001` One mutation per outer header field rejects.

- [x] `BPB-T-002` One mutation per `BatchPathEntryV1` field rejects.

- [x] `BPB-T-003` One mutation per `OpeningEntryV1` field rejects.

- [x] `BPB-T-004` One mutation per `WitnessNodeV1` field rejects.

- [x] `BPB-T-005` One mutation per reference index rejects.

- [x] `BPB-T-006` Mismatched `(proof_family_tag, opening_kind_tag)` rejects.

- [x] `BPB-T-007` `leaf_family_tag` mismatch between path entry and opening
  payload rejects.

- [x] `BPB-T-008` `hash_material_count != 1` rejects.

### 10. Root-Generation Migration Vectors

- [x] `RGM-G-001` Root-generation bridge vector.
  MUST contain the last pre-publication root generation, the first published
  root generation, exact canonical bytes, and the accepted linkage digest.

- [x] `RGM-T-001` Wrong root-generation linkage rejects.
  MUST prove that old/new root-generation material cannot be rebound under the
  wrong migration digest or wrong publication lineage.

## Release Gate

- [x] Every golden fixture above has canonical bytes, expected digest or root,
  expected verdict, and one regeneration command.

- [x] Every tamper fixture records the exact mutation point and the exact reject
  stage.

- [x] Every failover or crash fixture records the cut point, surviving durable
  artifacts, and only legal recovery outcome.

- [x] At least one deterministic re-encode check exists for every new public
  byte format: `ShardRouteTableV1`, `ShardRootLeafV1`,
  `CheckpointPublicationV1`, and `BatchProofBlobV1`.

- [x] Historical old/new route-generation fixtures are wired into migration
  verification rather than stored as documentation-only examples.
