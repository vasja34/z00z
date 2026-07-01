# Z00Z HJMT Shared Key Terms

Version: 2026-06-06

## 🎯 Purpose

This file is the canonical naming authority across the HJMT document set:

- [Z00Z-HJMT-Design.md](done/Z00Z-HJMT-Design.md)
- [Z00Z-HJMT-Fixture-Checklist.md](done/Z00Z-HJMT-Fixture-Checklist.md)
- [Z00Z-HJMT-Upgrade.md](done/Z00Z-HJMT-Upgrade.md)

Use this glossary to keep terminology stable across phase plans, Rust symbol names,
fixture manifests, tests, benchmarks, diagrams, and review notes.

If the documents diverge:

1. This glossary wins for term selection and spelling.
2. [Z00Z-HJMT-Design.md](done/Z00Z-HJMT-Design.md) wins for architectural semantics.
3. [Z00Z-HJMT-Upgrade.md](done/Z00Z-HJMT-Upgrade.md) wins for normative upgrade rules.
4. [Z00Z-HJMT-Fixture-Checklist.md](done/Z00Z-HJMT-Fixture-Checklist.md) wins for exact fixture IDs and vector coverage.

## 🔑 Naming Rules

| Rule | Canonical usage | Do not do this |
| --- | --- | --- |
| Semantic/public contract names stay unsuffixed. | Use `SettlementStateRoot`, `SettlementPath`, `BatchProofBlob`, `ShardRouteTable`, `CheckpointPublication` when describing the protocol contract or a phase deliverable. | Do not use `V1` names as the generic architectural term. |
| Exact wire-format types keep the explicit version suffix. | Use `BatchProofBlobV1`, `ShardRouteTableV1`, `ShardRootLeafV1`, `CheckpointPublicationV1` for codecs, fixtures, parser rules, binary layouts, and conformance tests. | Do not drop `V1` when the text is about exact bytes or a concrete struct. |
| Runtime placement names stay runtime-only. | Use `AggregatorId`, `ShardPlacementTable`, `ShardPlacementTableV1`, `ShardExecutor`, and `AggregatorNode` only for execution placement and failover orchestration. | Do not treat runtime placement names as public protocol truth. |
| Storage seams stay backend-neutral. | Use `SettlementTreeBackend`, `StorageBackend`, `JournalBackend`, `ReadTxn`, and `WriteTxn` for boundary traits. | Do not bake backend implementation names into the canonical contract. |
| Archived compatibility aliases remain explicitly archived. | Keep `AssetPath`, `AssetStateRoot`, and `AssetPathProof` marked as compatibility-only names. | Do not reuse archived names as live Phase 053 contract terms. |
| Planning helpers stay below protocol authority. | Keep `ShardKey` and `ShardItem` as planning-only helper vocabulary. | Do not promote planning helpers into the stable route or publication contract. |

## ⚙️ Core Protocol And Semantic Terms

| Canonical term | Kind | Exact symbol or variant | Use rule | Primary source |
| --- | --- | --- | --- | --- |
| `SettlementStateRoot` | public semantic root | live root name | Use as the only live public settlement root term. | design, upgrade |
| `SettlementPath` | canonical path identity | `SettlementPath { definition_id, serial_id, terminal_id }` | Use for the stable three-part semantic path across all phases. | design, upgrade |
| `SettlementLeaf` | terminal leaf family | semantic family with `AssetLeaf` and `RightLeaf` variants | Use when the text is about the generalized terminal settlement family. | design |
| `AssetLeaf` | terminal settlement leaf variant | live coin-like settlement leaf | Use for confidential asset-right leaves only. | design |
| `RightLeaf` | terminal settlement leaf variant | live non-coin right leaf | Use for bounded non-coin rights, not as a fee alias. | design |
| `FeeEnvelope` | separate settlement-adjacent object | fee/payment processing object | Keep distinct from `RightLeaf`. | design |
| `Definition` | semantic namespace | definition-level family namespace | Use for issuer/content/policy family meaning. | design |
| `definition_id` | path component | canonical identifier in `SettlementPath` | Use as the definition namespace identifier. | design, upgrade |
| `Serial bucket` | semantic grouping | serial-level semantic grouping | Use for the semantic grouping under one definition. | design |
| `serial_id` | path component | canonical identifier in `SettlementPath` | Use as the serial group identifier. | design, upgrade |
| `terminal_id` | path component | terminal settlement identity | Use as the terminal path identity, not `asset_id`, in live contract text. | design, upgrade |
| `BucketId` | physical layout identifier | policy-derived bucket identifier | Use for mutable physical layout, never for protocol shard identity. | upgrade |
| `Bucket policy` | verifier-visible rule set | bucket derivation and compatibility rule set | Use for policy generation, digest binding, and verifier recomputation. | design, upgrade |
| `ShardId` | stable protocol routing identity | stable shard identifier | Use for routing generations, journals, shard leaves, and failover lineage. | checklist, upgrade |
| `ShardRouteTable` | abstract route-table contract | conceptual route table | Use for the multi-shard route contract in plans and architecture prose. | upgrade |
| `ShardRootLeaf` | abstract shard publication leaf | conceptual shard leaf | Use for shard-root publication concepts outside exact codec text. | upgrade |
| `CheckpointPublication` | abstract publication contract | conceptual checkpoint publication object | Use for publication flow and checkpoint root chaining outside exact codec text. | checklist, upgrade |
| `ProofBlob` | single-path proof envelope | current proof contract | Keep as the one-path compatibility contract. | design, upgrade |
| `BatchProofBlob` | shared multi-path proof envelope | conceptual shared proof contract | Use when discussing the batch proof concept independent of exact encoding. | upgrade |
| `Compatibility backend` | archived backend lane | archive-gated reference backend | Keep archive-gated and compatibility-only. | design |
| `Path index` | auxiliary lookup plane | internal lookup mapping | Keep rebuildable and non-authoritative unless promoted by a future proof contract. | design |
| `backend_root` | private or diagnostic root | proof payload field | Never present as a substitute for `SettlementStateRoot`. | design |
| `Root generation` | root-version concept | root interpretation version | Use for pre/post shard-publication root meaning. | upgrade |

## 🧱 Exact Wire-Format And Record Names

| Canonical concept | Exact concrete type | Kind | Use rule | Primary source |
| --- | --- | --- | --- | --- |
| `BatchProofBlob` | `BatchProofBlobV1` | wire-format struct | Use `BatchProofBlobV1` for bytes, fixtures, parser rules, codec docs, and exact test vectors. | checklist, upgrade |
| `Batch proof header` | `BatchProofHeaderV1` | wire-format struct | Use only for header layout and conformance rules. | upgrade |
| `Batch path entry` | `BatchPathEntryV1` | wire-format struct | Use for per-path table entries in batch proof vectors. | checklist, upgrade |
| `Batch proof limits` | `BatchProofLimits` | bounds struct | Use for parser and verifier bounds. | upgrade |
| `Witness node` | `WitnessNodeV1` | wire-format struct | Use for deduplicated witness DAG node encoding. | checklist, upgrade |
| `Opening entry` | `OpeningEntryV1` | wire-format struct | Use for per-opening envelope entries. | checklist, upgrade |
| inclusion opening | `InclusionOpeningV1` | wire-format struct | Use for exact inclusion opening payloads. | checklist, upgrade |
| non-existence opening | `NonExistenceOpeningV1` | wire-format struct | Use for exact absence opening payloads. | checklist, upgrade |
| prior proof context | `PriorProofContextV1` | wire-format struct | Use for deletion-family context binding. | upgrade |
| deletion fact | `DeletionFactV1` | wire-format struct | Use for deletion proof payloads. | checklist, upgrade |
| path witness reference | `PathWitnessRefV1` | wire-format struct | Use for ordered witness indexes per path. | checklist, upgrade |
| `ShardRouteTable` | `ShardRouteTableV1` | wire-format struct | Use `V1` for canonical bytes, migration vectors, and route-lookup fixtures. | checklist, upgrade |
| route range rule | `RouteRangeRuleV1` | helper struct | Keep the `V1` suffix when described as a concrete route-table member type. | upgrade |
| `ShardRootLeaf` | `ShardRootLeafV1` | wire-format struct | Use `V1` for publication bytes and carry-forward fixtures. | checklist, upgrade |
| policy-set member | `PolicySetMemberV1` | wire-format struct | Use for policy activation/retirement membership records. | upgrade |
| `CheckpointPublication` | `CheckpointPublicationV1` | wire-format struct | Use `V1` for public checkpoint publication bytes and publication digest fixtures. | checklist, upgrade |
| bucket delta | `BucketDeltaV1` | durable commit record | Use for bucket-local commit deltas and durable replay evidence. | upgrade |
| parent delta | `ParentDeltaV1` | durable commit record | Use for serial/definition/global parent delta records. | upgrade |
| batch journal stage | `BatchJournalStageV1` | durable state enum | Use for crash-cut recovery stages and journal evidence. | upgrade |
| verified asset leaf | `VerifiedAssetLeafV1` | verifier output struct | Use for the exact verified asset-leaf proof output. | upgrade |
| path domains | `PathDomainsV1` | resolved domain struct | Use for route/bucket/path binding output. | upgrade |
| shard placement record | `ShardPlacementRecordV1` | runtime record | Use for exact runtime placement rows, not protocol truth. | upgrade |
| `ShardPlacementTable` | `ShardPlacementTableV1` | runtime table struct | Use `V1` only for the concrete runtime placement record set. | upgrade |

## 🧩 Helper Enums, Tags, And Result Types

| Name | Kind | Use rule | Primary source |
| --- | --- | --- | --- |
| `HjmtProofFamily` | enum | Keep as the proof-family discriminator for batch proof verification. | upgrade |
| `RootGeneration` | enum or tagged type | Keep as the root-interpretation discriminator. | upgrade |
| `SettlementLeafFamily` | enum | Keep as the leaf-family discriminator in proof and verifier code. | upgrade |
| `TerminalFamilyTagV1` | wire-format tag | Use only as the exact terminal-family codec tag type. | upgrade |
| `NodeDomainTagV1` | wire-format tag | Use only as the exact witness-node domain tag type. | upgrade |
| `SiblingSideTagV1` | wire-format tag | Use only as the exact sibling-side codec tag type. | upgrade |
| `OpeningKindTagV1` | wire-format tag | Use only as the exact opening-kind codec tag type. | upgrade |
| `PublicationModeV1` | wire-format tag | Use only as the exact publication-mode codec tag type. | checklist, upgrade |
| `BatchProofOk` | verifier result type | Keep as the accepted batch-verifier success output. | upgrade |
| `ProofChkErr` | verifier error type | Keep as the batch and proof verification reject type. | upgrade |
| `RouteErr` | route error type | Keep as the deterministic route-lookup failure type. | upgrade |
| `StoreErr` | storage seam error type | Keep as the backend-neutral storage boundary error type. | upgrade |
| `AssetLeafOk` | verifier result type | Keep as the accepted asset-leaf verifier success output. | upgrade |
| `JournalCursor` | journal cursor type | Keep as the replay cursor over journal history. | upgrade |
| `BatchPlanned` | runtime input type | Keep as the planned batch execution input type. | upgrade |
| `ShardExecutionOk` | runtime result type | Keep as the accepted shard execution output type. | upgrade |
| `ExecErr` | runtime error type | Keep as the shard execution failure type. | upgrade |

## 🔌 Boundary Traits, Runtime Objects, And Function Names

| Canonical name | Kind | Use rule | Primary source |
| --- | --- | --- | --- |
| `SettlementTreeBackend` | trait | Preferred semantic storage trait boundary for the Phase 053 forest. | design |
| `StorageBackend` | trait | Preferred backend-neutral durable KV and transaction seam. | upgrade |
| `ReadTxn` | trait | Preferred backend-neutral read transaction seam. | upgrade |
| `WriteTxn` | trait | Preferred backend-neutral write transaction seam. | upgrade |
| `JournalBackend` | trait | Preferred backend-neutral journal append and replay seam. | upgrade |
| `ShardExecutor` | trait | Preferred runtime shard execution seam parameterized over storage and journal backends. | upgrade |
| `AggregatorNode` | runtime struct | Preferred runtime node object that owns aggregator placement state. | upgrade |
| `AggregatorId` | runtime identifier | Preferred runtime executor identity; operational metadata only. | upgrade |
| `ShardGroupId` | runtime identifier | Preferred future replication-group identity. | upgrade |
| `settlement_root` | function or method | Preferred live root query name. | design |
| `get_settlement_item` | function or method | Preferred settlement item lookup name. | design |
| `put_settlement_item` | function or method | Preferred settlement insertion or replacement name. | design |
| `del_settlement_item` | function or method | Preferred settlement deletion name. | design |
| `settlement_proof_blob` | function or method | Preferred single-path proof retrieval name. | design |
| `verify_batch_proof` | function | Preferred batch proof verification entry point. | upgrade |
| `lookup` | method | Preferred deterministic route lookup entry point on `ShardRouteTableV1`. | upgrade |
| `verify_policy_member` | function | Preferred policy-set membership verifier name. | upgrade |
| `verify_asset_leaf_opening` | function | Preferred exact asset-leaf opening verifier name. | upgrade |
| `resolve_path_domains` | function | Preferred route and bucket domain resolution entry point. | upgrade |
| `begin_read` | method | Preferred backend-neutral read transaction opener. | upgrade |
| `begin_write` | method | Preferred backend-neutral write transaction opener. | upgrade |
| `append` | method | Preferred journal append entry point. | upgrade |
| `replay_from` | method | Preferred journal replay entry point. | upgrade |
| `execute_planned_batch` | method | Preferred shard-executor batch execution entry point. | upgrade |

## 🚩 Archived, Planning-Only, And Non-Preferred Names

| Name | Status | Replacement or rule | Primary source |
| --- | --- | --- | --- |
| `AssetPath` | archived compatibility | Use `SettlementPath` in all live Phase 053 text and code. | design |
| `AssetStateRoot` | archived compatibility | Use `SettlementStateRoot` in all live Phase 053 text and code. | design |
| `AssetPathProof` | archived sketch name | Use `ProofBlob` for the single-path proof envelope. | design |
| `RightsStateRoot` | non-preferred alternate | Keep `SettlementStateRoot` as the canonical live name. | design |
| `asset_id` as live terminal path name | non-preferred live wording | Use `terminal_id` for the live path contract; mention `asset_id` only in archived or asset-local contexts. | design |
| `ShardKey` | planning-only helper | Keep below protocol authority; use `ShardId` for stable routing identity. | upgrade |
| `ShardItem` | planning-only helper | Keep below protocol authority; do not promote into the public shard contract. | upgrade |

## 🧪 Fixture Family And Abbreviation Authority

| Abbreviation | Canonical expansion | Use rule | Primary source |
| --- | --- | --- | --- |
| `SRT` | `ShardRouteTableV1` fixture family | Use for route-table golden and tamper vectors only. | checklist |
| `SRL` | `ShardRootLeafV1` fixture family | Use for shard-root-leaf golden and tamper vectors only. | checklist |
| `CPP` | `CheckpointPublicationV1` fixture family | Use for publication golden and tamper vectors only. | checklist |
| `FOV` | failover, carry-forward, and crash vector family | Use for lawful failover and recovery vectors only. | checklist |
| `BPB` | `BatchProofBlobV1` fixture family | Use for batch-proof golden and tamper vectors only. | checklist |
| `RGM` | root-generation migration vector family | Use for old/new root-generation bridge and reject vectors only. | checklist |

## ✅ Recommended Reuse In Future Phase Documents

| Situation | Preferred name shape |
| --- | --- |
| Architectural prose and phase deliverables | semantic unsuffixed names such as `BatchProofBlob`, `ShardRouteTable`, `CheckpointPublication` |
| Rust struct, enum, or exact parser contract | concrete `V1` type names such as `BatchProofBlobV1` and `ShardRouteTableV1` |
| Runtime placement and failover logic | runtime-only names such as `AggregatorId`, `ShardPlacementTable`, and `ShardExecutor` |
| Storage seam or backend boundary | trait names such as `SettlementTreeBackend`, `StorageBackend`, and `JournalBackend` |
| Archived migration or historical equivalence notes | explicit archived names such as `AssetPath` and `AssetStateRoot` |

## 🔔 Editorial Rule For New Text

When writing new phase notes, TODOs, specs, or code comments:

1. Pick the semantic unsuffixed term for the concept.
2. Switch to the exact `V1` symbol only when bytes, fixtures, parser rules, or a concrete Rust type are under discussion.
3. Mark archived compatibility names explicitly as archived.
4. Keep runtime placement names out of public-root or route-table authority language.
