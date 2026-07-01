<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD047 -->
# Phase 019: 019-gaps-1 - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

🎯 Phase 019 closes the wallet-facing contract gaps identified in `todo.md`
 without widening the milestone into general wallet cleanup.

🎯 This phase must define and implement one storage-owned nullifier contract,
 one authoritative receive taxonomy contract, and one public backup contract
 that converges on the existing full wallet export/import path.

🎯 This phase does not create a second canonical storage engine, a second
 wallet backup schema, or a second receive-proof path. It must evolve the
 existing `z00z_storage` and `z00z_wallets` seams.

</domain>

<decisions>
## Implementation Decisions

### Nullifier state boundary
- **D-01:** Claim-domain nullifier state must move toward a storage-owned
  canonical namespace rather than remain wallet-local or simulator-local
  process state.
- **D-02:** The mandatory invariant is atomic asset-plus-nullifier commit: a
  finalized claim publish is accepted only if canonical asset state and
  canonical nullifier state advance together in one storage-owned transition.
- **D-03:** This nullifier track is claim-specific anti-replay, not a generic
  rule for every JMT spend path. Asset presence answers current leaf
  existence; claim nullifier state answers whether one claim identity has
  already been reserved or consumed.

### Receive taxonomy contract
- **D-04:** The authoritative public receive contract must be report-first at
  the runtime boundary.
- **D-05:** Malformed runtime input must never degrade into `NotMine` or
  `MaybeMine` on the public path; it must surface as explicit `InvalidInput`.
- **D-06:** Low-level pack or leaf helpers are secondary helpers only and must
  not remain public taxonomy authorities if they preserve the old silent
  downgrade behavior.
- **D-07:** Service-level validation remains required as defense in depth, but
  scanner-level report semantics must also be explicit so direct callers cannot
  bypass the contract accidentally.

### Backup and restore contract
- **D-08:** The public backup contract must converge on the existing
  `WalletExportPack` path rather than evolve `BackupPayloadV1` independently.
- **D-09:** The target public contract is full restore based on the existing
  encrypted export/import payload: seed or equivalent root secret material,
  snapshot state, and versioned restore metadata travel together.
- **D-10:** Legacy metadata-only backup behavior must be treated as legacy and
  read-compatible, not as the long-term public restore promise.

### Scope guardrails
- **D-11:** Phase 019 remains focused on three primary gaps only: nullifier
  state ownership, receive failure taxonomy, and backup convergence.
- **D-12:** Tightly related refactors are allowed only when they are necessary
  to implement one of those three gaps correctly.
- **D-13:** Broader cleanup such as general RNG-provider discipline or wallet
  documentation refresh is out of scope unless a plan proves that a small local
  refactor is strictly required by one of the primary contracts.

### Execution spine
- **D-14:** The execution order is fixed: first define the storage-owned
  nullifier transition, then harden the public receive taxonomy, then converge
  the public backup contract onto `WalletExportPack`, and only then close the
  phase with migration validation.
- **D-15:** Public receive migration must update the authoritative runtime
  service boundary and every directly coupled adapter or test surface in the
  same plan track; the phase must not leave a half-migrated taxonomy split.
- **D-16:** Backup convergence must keep V1 read compatibility while moving the
  public backup promise to the full-state export path; it must not break legacy
  backup file parsing before the V2 path is available.

### the agent's Discretion
- Exact module split inside `z00z_wallets` and `z00z_storage`, as long as no
  duplicate public contract is introduced.
- Exact DTO naming for report-first receive surfaces, as long as public
  taxonomy remains explicit.
- Exact migration wrapper shape for backup V2, as long as it reuses the
  existing `WalletExportPack` payload instead of inventing a parallel payload.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase inputs
- `.planning/ROADMAP.md` — Phase 019 registration and milestone placement.
- `.planning/PROJECT.md` — project-level storage and crate-boundary
  constraints.
- `.planning/REQUIREMENTS.md` — active requirements and scope constraints.
- `.planning/STATE.md` — current project state, prior decisions, and blockers.
- `.planning/phases/019-gaps-1/todo.md` — the gap list this phase must close.
- `.planning/phases/019-gaps-1/019-RESEARCH.md` — codebase-backed research and
  recommended direction for the three gaps.

### Prior phase decisions
- `.planning/phases/017-scenario-1/017-CONTEXT.md` — storage-owned state,
  typed proof boundary, and checkpoint discipline that Phase 019 must not
  violate.
- `.planning/phases/018-a-b-c/018-CONTEXT.md` — locked proof-validated scan,
  wallet evidence, and finalization gate discipline relevant to receive and
  storage-facing contracts.
- `.planning/phases/016-jmt-search-and-redb/016-CONTEXT.md` — durable storage
  and deterministic search constraints that canonical nullifier ownership must
  respect.

### Nullifier and claim surfaces
- `crates/z00z_wallets/src/core/claim/nullifier.rs` — deterministic nullifier
  derivation.
- `crates/z00z_wallets/src/core/claim/nullifier_store.rs` — current reserve and
  finalize lifecycle state.
- `crates/z00z_wallets/src/core/tx/claim_tx.rs` — claim-side nullifier
  verification.
- `crates/z00z_simulator/src/claim_pkg_store.rs` — current reserve, reload, and
  finalize helper path used by the simulator.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` — current claim publish
  reserve, rollback, and finalize flow.
- `crates/z00z_storage/src/assets/store.rs` — existing canonical storage write
  boundary that a storage-owned nullifier contract must extend rather than
  bypass.

### Receive taxonomy surfaces
- `crates/z00z_wallets/src/core/address/leaf_scan.rs` — canonical leaf scan and
  report helpers.
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs` — runtime scanner
  surface and current downgrade behavior.
- `crates/z00z_wallets/src/services/wallet_service.rs` — current public service
  boundary and prevalidation path.
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs` — direct public
  call sites and parity tests that will need taxonomy-safe migration.
- `crates/z00z_wallets/src/core/address/types.rs` — receive/report taxonomy
  types.

### Backup and restore surfaces
- `crates/z00z_wallets/src/core/backup/backup_exporter.rs` — backup exporter
  contract.
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs` — current
  metadata-only backup payload behavior.
- `crates/z00z_wallets/src/core/backup/backup_importer.rs` — backup importer
  contract.
- `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` — current
  metadata-only restore behavior.
- `crates/z00z_wallets/src/core/wallet/snapshot.rs` — versioned wallet snapshot
  contract.
- `crates/z00z_wallets/src/services/wallet_service.rs` — public backup,
  restore, export, and import entry points.
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl.rs` — public RPC
  boundary for backup create, list, and restore semantics.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `NullifierStateStore`: current reserve, rollback, and finalize semantics that
  planning should preserve while moving ownership downward.
- `claim_pkg_store.rs`: current helper seam for reserve, reload, rollback, and
  finalize orchestration that must not diverge from the canonical storage path.
- `receiver_scan_report(...)`: existing canonical report-first precedent that
  the runtime receive path should mirror.
- `WalletExportPack` and wallet payload import/export: existing full-state
  building block the backup contract should converge on.
- `AssetStore`: existing canonical write boundary that should host or anchor a
  storage-owned nullifier transition instead of adding a second persistence
  engine.

### Established Patterns
- Storage-owned canonical state lives in `z00z_storage`; wallet policy and
  public user-facing semantics live in `z00z_wallets`.
- Public contracts should be typed and explicit; silent downgrade semantics are
  acceptable only for internal helpers, not for authoritative boundaries.
- Backup and restore flows should reuse existing encrypted payload and snapshot
  abstractions instead of inventing a parallel schema.

### Integration Points
- Claim publish finalization path in `claim_pkg_consumer.rs` is the current
  place where nullifier lifecycle and storage apply already meet.
- `wallet_service.rs` is the current public receive and backup boundary and is
  the correct place to preserve or tighten public semantics.
- `asset_impl.rs` and simulator receive/parity tests are migration-sensitive
  callers because they currently exercise `receiver_scan_leaf()` and
  `scan_leaf()` directly.
- `backup_impl.rs` is migration-sensitive because public RPC backup semantics
  must stay explicit while V1 and V2 coexist.
- `snapshot.rs` and wallet payload export/import already provide the snapshot
  contract that backup convergence should build on.

</code_context>

<validation>
## Validation Gates

- **G-01 Nullifier Ownership Gate:** pass only if the chosen storage-owned
  nullifier path advances replay-protection state together with asset state and
  fails closed on partial publish.
- **G-02 Receive Taxonomy Gate:** pass only if malformed runtime input reaches
  the public receive boundary as `InvalidInput` and no authoritative public path
  silently downgrades it to `NotMine` or `MaybeMine`.
- **G-03 Migration Safety Gate:** pass only if every direct adapter or
  high-value caller affected by receive migration is updated or explicitly
  labeled as internal compatibility-only behavior.
- **G-04 Backup Convergence Gate:** pass only if the public backup contract
  moves onto the `WalletExportPack` path while V1 remains read-compatible.
- **G-05 Restore Contract Gate:** pass only if the new public restore promise is
  explicit about restoring root secret material, snapshot state, and versioned
  metadata.
- **G-06 Scope Gate:** pass only if any adjacent refactor included in the phase
  is necessary for one of the three primary contracts and does not widen the
  phase into unrelated cleanup.

## Blockers and Rollback

- If a storage-owned nullifier write path cannot be attached to an existing
  `z00z_storage` canonical write boundary without inventing a parallel state
  engine, that is a blocker and must be labeled explicitly during planning.
- If receive migration cannot preserve one authoritative public taxonomy path
  without leaving adapters or tests in contradictory semantics, the phase must
  stop at an explicit migration checkpoint instead of merging mixed behavior.
- If backup convergence cannot keep V1 restore compatibility while introducing
  the full-state V2 path, the phase must keep the old public contract unchanged
  and treat V2 as incomplete rather than shipping a partial migration.

</validation>

<specifics>
## Specific Ideas

📌 The user wants Phase 019 to answer and close every gap listed in
 `todo.md`, not just document them.

📌 The user explicitly chose a storage-owned target for nullifier state and a
 report-first public receive taxonomy.

📌 The nullifier discussion for Phase 019 is intentionally narrow: it covers
 claim-package replay protection where the current publish flow inserts claim
 outputs, not a generic statement that every ordinary JMT spend path requires
 a nullifier.

📌 The user explicitly chose a full-restore backup target based on
 `WalletExportPack`, not a continued metadata-only backup story.

📌 The user wants the phase to stay focused, but allows tightly related local
 refactors when they are necessary to make one of the three core contracts
 correct.

📌 The required execution spine for this phase is:

1. Attach nullifier ownership to a storage-owned canonical transition.
2. Harden the authoritative public receive taxonomy.
3. Migrate direct adapters and high-value callers that rely on the old receive
  semantics.
4. Converge public backup create and restore onto `WalletExportPack` while
  keeping V1 readable.
5. Close the phase only after the validation gates above are green.

📌 Direct receive callers in simulator tests, examples, and RPC glue are part
 of the rollout hazard for this phase and must be handled deliberately rather
 than left as an implicit cleanup task.

</specifics>

<deferred>
## Deferred Ideas

### Deferred follow-up cleanup
- General RNG-provider discipline cleanup outside the minimal changes required
  by the three primary Phase 019 gaps.
- Broad wallet documentation refresh and placeholder-surface cleanup not
  required by nullifier, receive taxonomy, or backup convergence work.

</deferred>

---

*Phase: 019-gaps-1*
*Context gathered: 2026-03-24*