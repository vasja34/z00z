## 0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout

**Goal:**

- Establish one storage migration gate before more correctness work binds to the current shared-backend internals.
- Make the three ordered steps explicit: first define the backend boundary, compatibility backend, proof-envelope contract, and equivalence corpus; then close storage and wallet authority slices on that stable facade; then land the forest backend as an early infrastructure lane instead of leaving HJMT-family work as a late measurement sidecar.

**Source:**

- [JMT design, section 2.7: Roadmap And Maturity Requirements](Z00Z-JMT-Design.md#27-roadmap-and-maturity-requirements)
- [JMT design, section 3.1: Semantic Strengths Already Present](Z00Z-JMT-Design.md#31-semantic-strengths-already-present)
- [JMT design, section 3.2: Physical Bottleneck To Remove](Z00Z-JMT-Design.md#32-physical-bottleneck-to-remove)
- [JMT design, section 3.3: Compatibility Backend Requirement](Z00Z-JMT-Design.md#33-compatibility-backend-requirement)
- [JMT design, section 4.2: Public Contract](Z00Z-JMT-Design.md#42-public-contract)
- [JMT design, section 5.1: Asset Path Proof Shape](Z00Z-JMT-Design.md#51-asset-path-proof-shape)
- [JMT design, section 7.1: Forest Commit Journal](Z00Z-JMT-Design.md#71-forest-commit-journal)
- [JMT design, section 7.2: Backend Interface Boundary](Z00Z-JMT-Design.md#72-backend-interface-boundary)
- [JMT design, section 9.1: Rollout Phases](Z00Z-JMT-Design.md#91-rollout-phases)
- [JMT design, section 12: Normative Requirement Summary](Z00Z-JMT-Design.md#12-normative-requirement-summary)
- [State management current spec, execution baseline decisions](../.planning/phases/050-state-mgmt/045-NEW-State-Management-Spec.md#execution-baseline-decisions)
- [State management current spec, required execution order](../.planning/phases/050-state-mgmt/045-NEW-State-Management-Spec.md#required-execution-order)

**Implementation-relevant fragments:**

- Use JMT sections 2.7, 3.1, 3.2, and 3.3 for the core architectural judgment: the current semantic storage vocabulary is already mostly correct, the shared physical commit boundary is the bottleneck, and migration must be staged behind a compatibility backend.
- Use JMT section 4.2 and the normative requirement summary for the stable public contract: keep `AssetPath`, `AssetLeaf`, and `AssetStateRoot` as the live public storage vocabulary; do not promote `SettlementStateRoot` or expose `backend_root` as a public authority substitute.
- Use JMT section 5.1 only for the proof-envelope ownership rule: storage owns the semantic-root to physical-proof binding, envelope versioning, and fail-closed verification contract. Downstream crates should consume typed proof results, not backend layout details.
- Use JMT sections 7.1 and 7.2 for the concrete migration seam: one backend trait, one forest commit journal, crash-safe publication rules, and one stable semantic API regardless of physical backend choice.
- Use JMT section 9.1 for the ordered rollout inside this phase: boundary and compatibility backend first, fixed bucket policy and forest backend second, dual-backend equivalence and configuration switch last.
- Use the state-management execution decisions and required order only to place downstream authority slices after the phase `0` facade is stable. They are not the source for backend topology, bucket policy, or proof-envelope design.

**Locality gate:**

- This entire phase is local implementation and local verification work: traits, codecs, roots, journals, golden tests, crash tests, simulator parity, and configuration-gated backend rollout.
- No live network, external DA, testnet, bridge locker, or production operator is needed.

**Implementation boundary:**

- In scope: storage backend trait design, compatibility backend wrapper, semantic root taxonomy, proof-envelope contract, root-binding rules, compile-time and API guardrails against downstream backend coupling, dual-backend equivalence tests, authority slices consuming the stable facade, fixed bucket policy types, forest backend implementation, commit journal, crash-safe recovery, and configuration-gated enablement.
- Out of scope: live backend rollout, production benchmark claims, generalized `SettlementStateRoot` export, public protocol root changes, wallet logic moved into storage, simulator-owned business rules, or backend-specific details leaked into public wallet or core APIs.

**Ordered internal gates:**

**0.1 Boundary And Compatibility Facade:**

- Freeze one storage-owned semantic API before more correctness work binds to the current shared JMT layout.
- Own the backend trait, compatibility backend, root taxonomy, proof-envelope contract, reject classes, and semantic-to-physical root binding rules.
- Reject downstream dependence on `backend_root`, raw namespace layout, raw branch ordering, or current `TreeId` internals as authority.

**Implementation tasks:**

**0.1 Boundary And Compatibility Facade Tasks:**

1. In `z00z_storage`, introduce one backend trait or equivalent facade that covers semantic root queries, batch put/delete, proof queries, checkpoint-facing reads, reload validation hooks, and any storage-owned state needed by wallet and simulator consumers.
2. Move the current shared namespaced JMT implementation behind that trait as the compatibility backend and treat it as a reference backend, not as the target performance architecture.
3. In `z00z_core` and `z00z_storage`, codify root taxonomy so `AssetStateRoot` remains the only live public asset-state root, `CheckRoot` remains checkpoint evidence, and `backend_root` stays proof-local or diagnostic only.
4. Define one storage-owned proof-envelope contract and version boundary for compatibility mode, including semantic root binding, path context, proof bytes, version checks, and fail-closed reject classes.
5. Add API and type guardrails that prevent downstream crates from depending on raw `backend_root`, raw `TreeId`, namespace prefixes, branch ordering details, or current physical key layout as authority.
6. Add a compatibility test corpus and golden vectors that compare semantic outcomes for insert, delete, reload, proof verification, checkpoint seal, checkpoint reload, and path-index rebuild scenarios.

**Tests and simulation:**

**0.1 Boundary And Compatibility Facade Tests:**

- Trait-compatibility tests proving the same semantic operations produce the same `AssetStateRoot`, delete behavior, replay behavior, and proof-verification outcome across compatibility and forest backends.
- Root-taxonomy tests rejecting backend-root substitution, accidental public export of `SettlementStateRoot`, or APIs that compare authority by physical backend root instead of semantic root.
- Proof-envelope tests for unsupported version, malformed bytes, wrong semantic root binding, wrong path context, wrong checkpoint binding, wrong bucket-policy metadata, wrong branch proof, and detached proof payloads.
- Golden-equivalence tests for insert-many, delete-many, hot-serial, cross-definition, duplicate path, delete-missing, reload-after-crash, checkpoint seal, checkpoint reload, and path-index rebuild scenarios.

**Done when:**

- `0.1` is done when downstream crates depend on one semantic storage facade instead of the current shared-backend layout, and the compatibility backend remains a green semantic reference for storage, checkpoint, and wallet-facing tests.

