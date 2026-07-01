# Progress

## ✅ What Works

- Phase 051 HJMT Facade is summary-backed complete through `051-06-SUMMARY.md`
  and final `051-SUMMARY.md`. The live caller-facing storage seam is
  `AssetTreeBackend`, `AssetStore` delegates through `CompatibilityBackend`,
  the compatibility golden corpus is green, and Phase 052 HJMT Backend is
  queued from `.planning/phases/052-HJMT-Backend/052-TODO.md` behind the same
  facade without reopening root, proof, checkpoint, or downstream authority
  ownership. Stage 4 tx preparation no longer decodes `ProofBlob` directly;
  caller-facing witness consumption now goes through the storage-owned
  `chk_blob_item(...)` verifier.

- Phase 047 core execution is summary-backed complete through `047-08`, and `047-09` through `047-11` are queued only; tx-history migration and `wallet.asset.*` convergence remain later/conditional work.

- Phase 042 Wave 1 control artifacts are in place, and the receiver-card
  display-field rename has been validated with focused wallet tests
- The live public derive contract is now receiver-native:
  `wallet.key.derive_receiver` is active, `wallet.key.derive_dual_address` is
  gone, and the response DTO is `RuntimeDeriveReceiverResponse { public_key, path }`
- The receiver-oriented `wallet.key.list_receivers`,
  `wallet.key.validate_receiver_card`, and `wallet.key.label_receiver` routes
  are active on the public RPC surface
- The receiver validation path checks compact receiver-card payloads, the
  active filter and label helper names are receiver-native, and
  `wallet.key.label_receiver` now rejects the legacy request field `address`
- The persisted wallet snapshot contract now uses `ReceiverDeriverState`,
  `receiver_deriver`, and snapshot version `5` across the active wallet
  snapshot/store/load path and backup/import/export fixtures
- The live wallet session derivation lane now uses
  `create_receiver_deriver_state(...)`,
  `get_create_wallet_receiver_deriver(...)`, and `receiver_manager` on the
  active runtime path
- The active wallet runtime/cache path now uses `ReceiverManagerImpl`,
  `AsyncReceiverManagerImpl`, `list_receivers(...)`, `receiver_manager`, and
  `.receiver_cache`; raw `.addr_cache` naming is gone from the compiled session
  lane
- The public facade cleanup is receiver-native: `key/mod.rs` and `lib.rs` no
  longer leak the legacy `Z00Z*` address re-exports, and the live receiver
  module exports receiver/card/request/scan contracts instead of a
  stealth-address subtree
- No legacy stealth-address family remains; the live receive surface is
  receiver-card, payment-request, and scan-output contracts
- The dead RPC DTO aliases `RuntimeAddressFilter`, `PersistAddressInfo`, and
  `RuntimeListAddressesResponse` are removed from the active RPC type layer
- The wallet source tree no longer carries `z00z_address` routing names or
  `z00z_address_*` helper-file prefixes under `crates/z00z_wallets/src/**`
- The wallet source tree now uses a single canonical physical receiver path:
  `crates/z00z_wallets/src/receiver/`. The old `src/address/` tree is absent,
  `src/lib.rs` declares `pub mod receiver;`, and `src/core.rs` directly
  re-exports `crate::receiver` with no address alias or shim
- Active receiver vocabulary now covers the former address-shaped wallet
  names: `format_receiver_handle`, `return_receiver`, `receiver_mode`,
  `allowed_recipients`, `ReceiverByKind`, `INDEX_RECEIVER_BY_KIND_TABLE`,
  `index_receiver_by_kind`, and `index.receiver_by_kind`
- The completed receiver migration has green validation from
  `cargo fmt -p z00z_wallets`,
  `cargo check -p z00z_wallets --all-targets --features test-params-fast`, strict
  generated-dir-excluded residue scans, and
  `cargo test -p z00z_wallets --release --features test-params-fast --features wallet_debug_tools`
- The unwired duplicate session files
  `wallet_service_session_seed_derivation.rs` and
  `wallet_service_session_snapshot.rs` have been removed from the source tree,
  and the wallet crate compile gate stayed green after their deletion
- The workspace structure is established and multi-crate
- Foundational architecture guidance exists in repository instructions and the
  design foundation
- Core crates and support crates are already present for protocol, crypto,
  storage, wallets, simulation, networking, and utilities
- The memory bank scaffold now exists and can be maintained incrementally
- A dedicated `.github/skills/create-tests/` skill now exists for deriving
  phase-local test specifications, `*-TESTS-TASKS.md` execution plans, optional
  compatibility `*-TEST-PLAN.md` aliases, and approval-backed unit or
  integration coverage from GSD planning artifacts
- A dedicated `.github/skills/spec-to-tasks/` skill now exists for converting
  specs or normalized free-docs into repository-style `TODO.md` or
  `NNN-TODO.md` backlogs with dependency order, file-first sequencing,
  validation traceability, explicit non-goals, exemplar-true validation-wave
  sections, and truthful closure criteria
- A dedicated `.github/skills/alert-concept-drift/` skill now exists for
  comparing the current workspace to a historical Git anchor, separating
  healthy concept evolution from suspicious drift across security, API,
  duplication, architecture, and cryptography, and routing non-trivial
  findings through `doublecheck` before final reporting
- A dedicated `.github/skills/attack-surfaces-create/` skill now exists for
  seeded static discovery of security, cryptography, and threat attack
  surfaces across a module, crate, or whole repository, with append-only JSONL
  inventory updates, skeptical pro-con audit gating, and one strong verified
  candidate admitted per run
- `.planning` contains a completed verified chain across Phase 025 through Phase
  031 covering crypto, core, utils, storage, wallets, long-file refactoring,
  and architecture cleanup
- Phase 025 UAT is fully green, and Phases 027-031 all carry explicit green or
  approved validation closeouts in their phase artifacts
- Phase 033 is closed on repository-backed summary artifacts
- Phase 034 is complete through Plan 09 on repository-backed summary, UAT, and
  validation artifacts
- The completed Phase 034 chain includes the backend-owned checkpoint
  contract, the semantic validation waves, the closure package for
  Q63/Q64/Q65/Q47, and the executed post-closure hygiene chain
  `034-16` -> `034-17` -> `034-18`
- Phase 035 is complete through Plan 16 on repository-backed summary and
  review artifacts, including sender adapter convergence, temp-doc truth
  correction, the sender validation wave, the sender acceptance gate, and the
  first bounded stealth-additions slice for receiver-secret inventory or
  narrowing
- The verified Phase 035 baseline now also includes frozen card-bound and
  request-bound stealth derivation vectors, negative drift coverage, and the
  bounded core-side V2 memo decode contract
- The verified Phase 035 baseline now also includes live V2 memo receive-path
  support on the approved seam, private decrypted memo handling, fail-closed
  malformed memo behavior, and the closed final stealth validation and
  acceptance slice
- Phase 035 is complete through Plan 19 on repository-backed summary and review
  artifacts
- The verified Phase 035 baseline now also includes the full rename closure
  chain: the Plan 17 rename authority fence and Wave A file rename work for
  `035-41` through `035-43`, the Plan 18 wallet DB plus egui rename and mirror
  or declaration slice for `035-44` through `035-46`, and the Plan 19
  declaration/reference sweep plus acceptance closure for `035-47` through
  `035-49`
- Phase 044 now has a focused evidence-backed PH44 lifecycle slice on the live
  wallet path: confirmed journal rows persist `TxConfirmationEvidence`,
  broadcast or admission leaves wallet tx rows pending while storing typed
  confirmation evidence, reconcile consumes stored evidence instead of
  re-running inline confirmation, and history or details receipts project the
  persisted evidence. The focused verification set is green for
  `test_tx_broadcast_admits_without_confirming`,
  `test_tx_reconcile_requires_confirmation_evidence`,
  `test_tx_reconcile_rejects_mismatched_evidence`,
  `test_tx_import_reconcile_portable`, `test_tx_history_includes_receipt`,
  `adapters::rpc::methods::tx_rpc_storage::tests::tx_info_to_details_decodes_package_rows`,
  and `tx_history_appends_admission_sequence`, with
  `cargo check -p z00z_wallets --tests --features test-params-fast,wallet_debug_tools`
  also green on the current tree
- Phase 044 `044-wallets-patch.md` has an additional real-code doublecheck
  pass: the Scenario 1 Alice-to-Bob tx package history guard passed, the
  multi-transaction tx-history append sequencing test passed, the simulator
  Stage 4 card-gate negative assertions were corrected, the full
  `z00z_simulator` release test suite passed, and strict live-source scans
  found no forbidden legacy per-tx tx-history write patterns
- The final Phase 035 closeout is backed by a green rerun of the mandatory
  bootstrap gate during the continuity refresh that synchronized `.planning`
  and the memory bank to the already-written Plan 19 artifacts
- Phase 035 now also has a partial validation artifact and a dedicated rename
  guard integration test at
  `crates/z00z_wallets/tests/test_phase035_rename_guards.rs`
- Phase 035 now also has a State B eval applicability audit at
  `.planning/phases/035-mix2-fixes/035-EVAL-REVIEW.md` with a
  `PRODUCTION READY` verdict for AI evaluation scope because the phase does not
  implement an AI system
- Phase 040 now has a live internal theorem-relation proof-generation path:
  `regular_spend_theorem_bpplus` is the current suite,
  `CanonicalSpendProofBackend` is the default backend, and explicit membership
  witnesses are part of `SpendProofWitness`
- Phase 040 focused validation is green for wallet backend tamper coverage,
  public verifier coverage, tx tamper and wrong-root coverage, witness-gate
  membership coverage, Scenario 1 release execution, the stage-surface guard,
  and the simulator release suite with `test-params-fast` plus `wallet_debug_tools`
- Phase 040 full workspace validation is green after the membership-aware
  wallet typed-root unit test was aligned to the new root contract; the
  canonical `full_verify.sh` gate completed with exit code 0 and refreshed
  `reports/full_verify-report-long-running-tests.txt`
- `z00z_rollup_node` now has a focused rollup settlement guard that binds tx
  package structure and digest, the wallet public spend theorem contract,
  checkpoint statement proof payload, artifact/link/exec-id continuity, root
  alignment including spend-root binding, and checkpoint execution-input tx
  inclusion. Focused rollup tests and clippy passed for this guard.
- The Phase 040 continuation review loop ran three independent passes with no
  significant issues after the current-authority legacy phrase gate returned no
  live matches.

## 🚧 What Is In Progress

- Phase 052 HJMT Backend is the next planning lane after Phase 051. Its real
  forest/HJMT backend must enter behind `AssetTreeBackend`, use
  `CompatibilityBackend` as the semantic oracle, and keep public storage
  behavior on `AssetPath`, `StoreItem`, storage-owned proof contracts, and
  `AssetStateRoot`.

- Phase 044 wallet asset lifecycle is still in active implementation on the
  dirty tree. The current working state now includes the evidence-backed
  PH44 send/admit/reconcile or history/offline slice, earlier wallet tx
  persistence/RPC/export naming work, simulator Stage 5/6 split behavior,
  checkpoint admission coverage, relocated wallet module source guards, and
  refreshed Phase 044 traceability artifacts. Focused PH44 verification is
  complete and the simulator release suite is green after the Stage 4
  card-gate test correction, but broader workspace verification and formal
  phase closeout are still open.

- Phase 042 wallet-crate receiver migration is complete on the live code path:
  receiver-facing RPC, persisted receiver snapshot/session naming, manager
  contracts, cache/rate-limit config seams, public stealth family, helper
  filenames, helper-binary manifest entries, physical module path, DB schema
  names, request metadata, and policy terminology are receiver-native,
  stealth-native, or removed where obsolete
- Remaining Phase 042 work, if continued, is formal process closeout rather
  than active wallet-crate symbol cleanup: broader `.planning` sync and any
  release-note packaging should be handled as a separate documentation slice
- Memory-bank adoption: contributors and agents need to keep these files current
- The memory-bank workflow now uses a dashboard-first `activeContext.md` model,
  while ongoing maintenance is still required to keep the summaries current
- Phase 040 is the active planning and execution surface on `040-10-PLAN.md`.
  The current target is internal theorem-relation closure, not public or
  trustless proof-of-knowledge closure. The implementation, full workspace
  verify gate, and focused rollup public-artifact binding guard are green; a
  final summary or handoff artifact is still separate from the validation
  result.
- Ongoing protocol, wallet, runtime, storage, and integration work across the
  workspace
- Phase 032 remains explicitly reopened in the roadmap for spend-contract and
  claim-trust verification wording.
- Phase 037 output reception now has a summary-backed numbered plan chain
  through `037-10-SUMMARY.md`, but its partial Task 9 backlog plus pending UAT
  and final verification remain open.
- Post-closeout continuity maintenance remains a live discipline so `.planning/`
  state, memory-bank files, and future planning work stay synchronized
- Phase 035 validation is reconstructed, but the phase is not yet fully
  Nyquist-compliant because several planning-authority and acceptance-boundary
  checks are still manual-only by design

## 📌 What Is Left To Build Or Mature

- More detailed task history as real work is captured in `tasks/`
- Deeper feature-level context documents when specific subsystems need them
- Ongoing alignment between docs, architecture, and implemented behavior
- Better continuity summaries for `.planning` phases that materially change the
  repository state
- Closure of residual follow-up gaps that completed phases explicitly left open,
  especially the Phase 026 protected-network anchor success case
- More repo-native planning helpers that reduce manual translation from design
  documents into execution artifacts beyond tests and TODO backlogs
- Explicit routing to the next active planning or implementation slice after
  the now-complete Phase 035 chain
- Optional future automation for the manual-only rows recorded in
  `035-VALIDATION.md` if Phase 035 ever needs a full Nyquist pass
- A future AI-enabled follow-up in the Phase 035 area would still need a real
  `AI-SPEC.md`, eval dimensions, dataset, guardrails, and tracing before any
  AI eval verdict would carry semantic weight

## ⚠️ Known Gaps

- Phase 044 wallet asset lifecycle closure is not yet final. The dirty-tree
  implementation now has focused green PH44 evidence, but it still needs
  broader phase verification and planning closeout before it should be
  described as complete. Keep final vocabulary aligned to `Available`,
  `Spent`, and tx `Confirmed`; do not introduce a spendable `Validated`
  wallet asset status.

- Memory-bank updates are not automated by repository scripts at this point
- Some workspace areas appear thin or scaffold-like and need maturity tracking
- Existing repository documentation is broad, but not all of it is normalized
  into one durable continuity system yet
- The worktree can contain major planning-only changes that are easy to miss if
  the memory bank is updated only from top-level crate docs
- Phase 026 validation is truthfully only partial for protected-network positive
  anchor success because canonical mainnet or testnet anchor values are not yet
  present in the code path described by the phase validation file
- Cross-surface continuity can still drift unless `.planning/` updates and the
  memory bank are reconciled in the same pass
- Phase 035 validation remains only partial until the manual-only rows in
  `035-VALIDATION.md` are either accepted as manual governance checks or
  converted into dedicated guard tests
- Phase 040 public/trustless proof-of-knowledge remains open. The current
  validated state is internal wallet/simulator relation closure over the
  deterministic canonical artifact, not public verifier witness knowledge.
- Phase 040 checkpoint theorem finality and full public or trustless rollup
  settlement closure remain open and must not be described as completed by the
  current wallet relation or focused rollup public-artifact binding work.

## 📅 Current Status

As of 2026-05-12, the active Phase 044 wallet asset lifecycle lane has a live
evidence-backed PH44 send/admit/reconcile or history/offline slice. Confirmed
tx journal rows now persist `TxConfirmationEvidence`, reconcile consumes stored
typed confirmation evidence instead of re-running inline confirmation, and the
focused verification set is green on the current tree for the PH44 lifecycle,
receipt projection, and append-only journal guards. This is still a targeted
lane result, not a full Phase 044 or workspace closeout.

As of 2026-05-05, the live wallet removal effort has completed the
wallet-crate receiver migration. Wave 1 control artifacts remain in place, the
receiver-card response uses `owner_handle_display`, the live derive route is
`wallet.key.derive_receiver`, the persisted snapshot contract uses
`ReceiverDeriverState` / `receiver_deriver` with version `5`, and the active
session helper surface uses `get_create_wallet_receiver_deriver(...)`,
`ReceiverManagerImpl`, `AsyncReceiverManagerImpl`, `ReceiverManager`,
`AsyncReceiverManager`, `ReceiverManagerConfig/Error/Result`,
`list_receivers(...)`, `receiver_manager`, `.receiver_cache`,
`wallet.receiver.*`, and `Z00Z_WALLET_RECEIVER_*`. The physical receive module
is now `crates/z00z_wallets/src/receiver/`; the old `src/address/` module tree
is absent; `core.rs` no longer aliases address as receiver; request metadata,
policy rules, DB schema keys, wasm storage mapping, docs, and tests use active
receiver terminology. Strict residue scans classify remaining `address` hits
as frozen domain labels, BIP44 derivation vocabulary, external Tari reference
docs, or generic non-receiver docs. Validation passed with `cargo fmt`,
`cargo check -p z00z_wallets --all-targets --features test-params-fast`, and
`cargo test -p z00z_wallets --release --features test-params-fast --features wallet_debug_tools`.
A separate `.planning` synchronization slice may still be useful if Phase 042
needs formal process closeout artifacts.

As of 2026-04-05, the memory bank is no longer just a scaffold. It reflects
that the repository completed a verified Phase 025-031 hardening and refactor
chain under `.planning/`, while also preserving the key residual note that the
Phase 026 protected-network anchor-success path is still only partially closed.

As of 2026-04-17, the embedded-versioning slice of Phase 036 is still
summary-backed complete through `036-10-SUMMARY.md` on the canonical authority
chain `.planning/phases/036-rename/036-a1-versioning-spec.md` ->
`.planning/phases/036-rename/036-TODO-2.md` ->
`.planning/phases/036-rename/036-CONTEXT.md`. The separate delete-first
continuation now lives on `.planning/phases/036-rename/036-a2-legacy-removing-spec.md`
-> `.planning/phases/036-rename/036-TODO-3.md` -> `036-11` through `036-16`,
and the live execution pointer was reset to restart that continuation from
`036-11-PLAN.md` instead of overstating that later plans were already active.

As of 2026-04-18, the delete-first continuation itself was found to have drifted
again: `036-13-SUMMARY.md`, the expected `036-11-SUMMARY.md`,
`036-12-SUMMARY.md`, and `036-14-SUMMARY.md` artifacts, `036-a2-legacy-removing-spec.md`,
`036-TODO-3.md`, `.planning/STATE.md`, `.planning/ROADMAP.md`, and a large
dirty storage or wallet or simulator or utility diff no longer agree on what
Phase 036 truth is. The live truth is only that `036-13` and `036-14` are
reopened and `036-15` remains blocked; the current dirty code changes still
need a row-by-row delete-vs-rename audit before any claim about completed
rollback or completed truth repair is trustworthy.

As of 2026-04-22, the roadmap now also registers the pre-existing
`.planning/phases/037-output-reception/` directory as Phase 037 Output
Reception without creating a duplicate folder. The live authority for that
queued phase is currently only `037-TODO.md`; numbered `037-XX-PLAN.md`
artifacts do not exist yet, and Phase 036 still remains the active execution
surface.

As of 2026-04-22, Phase 037 also has a new
`.planning/phases/037-output-reception/037-CONTEXT.md`. The live truth is that
this context intentionally stays simple: it locks `037-TODO.md` as the
canonical planning inventory, requires the future planner to cover every task
sequentially one after another, forbids task-title rewrites or silent task
exclusion, and explicitly blocks duplicate receive layers or concept drift.

As of 2026-04-08, Phase 033 is complete on repository-backed summary artifacts.

As of 2026-04-09, the repository also has a dedicated
`.github/skills/spec-to-tasks/` skill for converting specs or normalized
free-docs into modern repository-style TODO backlogs with stable section order,
dependency mapping, file-first execution sequencing, explicit non-goals,
exemplar-true validation-wave headings, and truthful closure criteria.

As of 2026-04-11, Phase 034 is complete through Plan 09. The live truth is
that the repository now has summary-backed claim continuity and spend-nullifier
closure, a summary-backed sender-authority retirement slice, a backend-owned
checkpoint acceptance chain, a completed closure package for Q63/Q64/Q65/Q47,
and an executed post-closure hygiene chain recorded under `034-09`.

As of 2026-04-13, Phase 035 is complete through Plan 14. The live truth is
that the repository now has repository-backed downstream sender-adapter
convergence, corrected temp sender docs, a completed sender validation wave, a
completed sender acceptance gate, and a completed first stealth-additions slice
for bounded receiver-secret inventory or narrowing before execution continues
at `035-15-PLAN.md`.

As of 2026-04-13, Phase 035 is also complete through Plan 15. The live truth is
that the repository now has frozen card-bound and request-bound stealth
derivation vectors, negative drift coverage for those formula families, and a
bounded side-by-side V2 memo decode contract in `z00z_core`, while wallet
receive support for `V2Memo` remains intentionally deferred before execution
continues at `035-16-PLAN.md`.

As of 2026-04-13, Phase 035 is also complete through Plan 16. The live truth is
that the repository now has the final stealth-additions slice closed: the V2
memo receive path is enabled on the approved wallet seam, memo data remains
private decrypted metadata rather than public leaf metadata, malformed memo
payloads fail closed, and the validation plus acceptance gates close only on
the approved stealth additions before execution continues at `035-17-PLAN.md`.

As of 2026-04-13, Phase 035 is also complete through Plan 17. The live truth is
that the repository now has the first curated rename slice closed: the rename
authority is frozen to the recovered curated table plus the high-confidence
delta, the raw 814-row matrix remains inventory-only, the live rename manifest
is split into file-first and signature-after lanes, and the approved Wave A
test/support file rename set is complete before execution continues at
`035-18-PLAN.md`.

As of 2026-04-13, Phase 035 is complete through Plan 19. The live truth is
that the repository now has the second rename wave, the final declaration and
reference sweep, and the bounded acceptance closure all summary-backed under
`035-18-SUMMARY.md`, `035-19-SUMMARY.md`, `035-18-REVIEW.md`, and the refreshed
`035-19-REVIEW.md`. The top-level `.planning` state, roadmap, and memory-bank
surfaces were then synchronized to that already-written closeout truth after a
green rerun of the mandatory bootstrap gate.

As of 2026-04-13, Phase 035 also has a reconstructed validation artifact at
`.planning/phases/035-mix2-fixes/035-VALIDATION.md`. The live truth is that
the sender, stealth, simulator, and late rename runtime slices were rerun
green, a dedicated rename guard test was added, and the quick bootstrap gate
ended with `=== BOOTSTRAP COMPLETE ===`, but the overall verdict remains
`partial` because planning-authority and acceptance-boundary assertions are
still manual-only.

As of 2026-04-14, Phase 035 also has a reconstructed
`.planning/phases/035-mix2-fixes/035-EVAL-REVIEW.md`. The live truth is that
the eval-review workflow ran in State B, confirmed there is no `AI-SPEC.md`
and no AI system in scope for this phase, and therefore recorded a
`PRODUCTION READY` applicability verdict with zero critical AI-eval gaps while
explicitly deferring all normal Rust, UAT, security, and Nyquist truth to the
existing closeout artifacts.

As of 2026-04-14, the repository also has a dedicated
`.github/skills/alert-concept-drift/` skill. The live truth is that the
workspace can now run a historical-anchor concept-drift audit that reconstructs
baseline concepts from an older commit, compares them to current semantics,
classifies candidates as healthy evolution or suspicious drift, and forces a
`doublecheck` pass on every non-trivial finding before final reporting.

As of 2026-04-28, Phase 040 is active on `040-10-PLAN.md` for internal
theorem-relation closure. The live wallet path uses
`regular_spend_theorem_bpplus` with `CanonicalSpendProofBackend`; membership
witnesses are passed into `SpendProofWitness`; the backend validates each input
against `prev_root` plus nullifier, balance, and range relation checks; and
Scenario 1 Stage 4 now uses prep-derived membership witnesses in the runtime
proof path. Focused wallet tests, the Scenario 1 release run, the stage-surface
guard, and `cargo test -p z00z_simulator --release --features test-params-fast
--features wallet_debug_tools` passed. Public/trustless proof-of-knowledge,
checkpoint theorem finality, and rollup settlement closure remain open.

As of 2026-04-29, the Phase 040 review pass hardened direct backend
verification for the internal relation. `CanonicalSpendProofBackend::verify()`
now validates public relation drift before accepting deterministic artifact
bytes, including output range proofs, duplicate public inputs, input/output
theorem leaf overlap, and balance mismatch. The spend statement builder and
Phase 040 fixture project output theorem leaves through the output `leaf_ad_id`
namespace inside `SpendProofStmt`, while canonical statement bytes still bind
the storage/package asset IDs. Follow-up validation passed for focused wallet
backend and public verifier tests, the simulator stage-surface guard, the
bootstrap gate, and `cargo test --release --features test-params-fast --features
wallet_debug_tools`. The closure remains internal only; public
proof-of-knowledge, checkpoint theorem finality, and full rollup settlement
proof closure remain open.
