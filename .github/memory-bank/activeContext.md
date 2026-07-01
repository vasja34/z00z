# Active Context

## 🎯 Role Of This File

This is the main dashboard of the memory bank. Read this file first on every
task, then read the remaining core memory-bank files and use this dashboard to
decide which deeper repository artifacts must be reopened before making
changes.

## 📅 Last Verified Snapshot

Current snapshot: Phase 051 HJMT Facade is summary-backed complete through
`051-06-SUMMARY.md` and final `051-SUMMARY.md`. Phase 052 HJMT Backend is the
next queued lane from `.planning/phases/052-HJMT-Backend/052-TODO.md`; it must
implement the real forest/HJMT backend only behind the Phase 051
`AssetTreeBackend` facade, with `CompatibilityBackend` as the migration oracle,
and must not reopen facade ownership, root vocabulary, proof envelope,
checkpoint contracts, or downstream authority ownership.

- Last reviewed: 2026-05-28
- Verification basis: Phase 051 storage facade doublecheck confirmed the live
  `z00z_storage::assets` caller-facing seam is `AssetTreeBackend`, implemented
  by `AssetStore`, delegated through `CompatibilityBackend`, backed by the
  compatibility golden corpus, and guarded against public physical-key,
  namespace, fake-forest, stale 053 planning drift, and Stage 4 direct
  `ProofBlob::decode` witness consumption. Additional historical basis: Phase
  042 wallet receiver migration is complete on the
  live wallet-crate source path: the old physical `src/address/` module tree
  was renamed to `src/receiver/`, the `core::receiver` compatibility shim was
  removed, and active RPC, receiver-card, payment-request, policy, DB schema,
  scan, wasm, docs, and tests now use the canonical receiver vocabulary. This
  completed state builds on the earlier Phase 042 Wave 1 control artifacts,
  public derive route cutover, persisted receiver-deriver cleanup, session
  state rename, receiver-manager/cache cleanup, public `stealth_address`
  facade cutover, source-tree routing cleanup, concrete and trait-level
  receiver-manager contract rename, public `Stealth*` type-family rename,
  `StealthAddressError` rename, dead RPC DTO removal, `list_receivers(...)`
  rename, receiver-native config/env seam rename, and stealth-native helper
  filename and binary cleanup. The live state was validated against
  `.planning/phases/042-refactor-wallets/042-z00z_address-remove-spec.md`,
  `.planning/phases/042-refactor-wallets/042-05-PLAN.md`,
  `.planning/phases/042-refactor-wallets/042-05-spec-coverage.md`,
  `.planning/phases/042-refactor-wallets/042-05-wave-log.md`,
  `.planning/STATE.md`,
  `crates/z00z_wallets/src/receiver/mod.rs`,
  `crates/z00z_wallets/src/receiver/card/stealth_card.rs`,
  `crates/z00z_wallets/src/receiver/card/stealth_card_codec.rs`,
  `crates/z00z_wallets/src/receiver/request/stealth_request_types.rs`,
  `crates/z00z_wallets/src/receiver/request/stealth_request_parse.rs`,
  `crates/z00z_wallets/src/receiver/manager/address_manager_impl_builder.rs`,
  `crates/z00z_wallets/src/receiver/manager/address_manager_impl_async.rs`,
  `crates/z00z_wallets/src/receiver/manager/address_manager_trait.rs`,
  `crates/z00z_wallets/src/receiver/manager/address_manager_impl_trait_impl.rs`,
  `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs`,
  `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs`,
  `crates/z00z_wallets/src/receiver/scan/leaf_scan.rs`,
  `crates/z00z_wallets/src/stealth/output/output.rs`,
  `crates/z00z_wallets/src/stealth/output/output_build.rs`,
  `crates/z00z_wallets/src/stealth/output/output_validator.rs`,
  `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`,
  `crates/z00z_wallets/src/wallet/snapshot/snapshot_impl.rs`,
  `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs`,
  `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`,
  `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_rotation.rs`,
  `crates/z00z_wallets/src/services/wallet/session/wallet_service_session.rs`,
  `crates/z00z_wallets/src/services/wallet/paths/wallet_paths.rs`,
  `crates/z00z_wallets/src/services/wallet/types/wallet_service_types.rs`,
  `crates/z00z_wallets/src/services/wallet/wallet_service.rs`,
  `crates/z00z_wallets/src/wallet/mod.rs`,
  `crates/z00z_wallets/src/wallet/entity/wallet_entity_core.rs`,
  `crates/z00z_wallets/src/wallet/entity/wallet_entity_constructor.rs`,
  `crates/z00z_wallets/src/adapters/rpc/types/key.rs`,
  `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_derive.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_admin.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs`,
  `crates/z00z_wallets/src/wallet/policy.rs`,
  `crates/z00z_wallets/src/db/codecs/schema_keys.rs`,
  `crates/z00z_wallets/src/db/redb/store/redb_wallet_store_tables.rs`,
  `crates/z00z_wallets/src/db/redb/store/redb_wallet_store_objects.rs`,
  `crates/z00z_wallets/src/db/redb/store/redb_wallet_store_debug_export.rs`,
  `crates/z00z_wallets/src/db/redb/migrations/redb_wallet_store_migrations_tables.rs`,
  `crates/z00z_wallets/src/db/redb/schema/redb-schema.yaml`,
  `crates/z00z_wallets/src/wasm/storage_backend.rs`,
  `crates/z00z_wallets/src/key/mod.rs`,
  `crates/z00z_wallets/src/lib.rs`,
  `crates/z00z_wallets/README.md`,
  `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs`,
  `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`,
  `crates/z00z_wallets/benches/address_derivation.rs`,
  and `crates/z00z_wallets/benches/async_batch_threshold_bench.rs`
- Validation evidence: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed earlier for the Phase 042 bootstrap gate; `cargo fmt -p z00z_wallets` passed; `cargo check -p z00z_wallets --all-targets --features test-params-fast` passed; strict generated-dir-excluded residue scans found no active `src/address`, no active `crate::address`/`core::address`/`z00z_wallets::address`, and no compatibility shim; the remaining `address` hits are classified as frozen domain strings, BIP44 derivation vocabulary, external Tari reference docs, or generic non-receiver docs; `cargo test -p z00z_wallets --release --features test-params-fast --features wallet_debug_tools` passed through final doc-tests with `32 passed`, `0 failed`, and `10 ignored`
- Confidence level: High for the completed wallet-crate receiver migration and active stealth/receiver cryptographic flow alignment; broader Phase 042 planning artifacts may still need a separate documentation synchronization pass, but the live code path is receiver-canonical with no active alias or shim

## ✅ Verified Baseline

- Phase 042 Wave 1 control artifacts are present and aligned: the source spec,
  execute plan, coverage ledger, wave log, and phase state snapshot all agree
  on the current no-backcompat removal effort
- The receiver-card RPC response no longer exposes an address-shaped display
  field; it now uses `owner_handle_display`, and both targeted tests passed
- The live public derive contract is now receiver-native by route and DTO name:
  `wallet.key.derive_receiver` replaces `wallet.key.derive_key`,
  `wallet.key.derive_dual_address` is removed, and the active response type is
  `RuntimeDeriveReceiverResponse { public_key, path }`
- The active receiver admin/list helper vocabulary is now receiver-native:
  `RuntimeReceiverFilter` is primary, `apply_receiver_filter(...)`,
  `get_receiver_labels(...)`, and `upsert_receiver_label(...)` replaced the
  old address-named helper surface, and `wallet.key.label_receiver` no longer
  accepts the legacy request field `address`
- The persisted wallet snapshot contract is now receiver-native on the live
  path: `ReceiverDeriverState`, `receiver_deriver`, and snapshot version `5`
  are wired through wallet snapshot, store/load/create flows, and the active
  backup/import/export fixtures
- The live wallet session derivation lane now uses
  `create_receiver_deriver_state(...)`,
  `get_create_wallet_receiver_deriver(...)`, and `receiver_manager` instead of
  the active address-owned helper or field names
- The active wallet runtime lane now uses `ReceiverManagerImpl`,
  `AsyncReceiverManagerImpl`, `list_receivers(...)`, and
  `receiver_cache_file_path(...)`; raw `.addr_cache` naming is gone from the
  compiled session path
- The public facade/module cleanup is receiver-native: `key/mod.rs` and
  `lib.rs` no longer re-export the legacy `Z00Z*` address types, the public
  alias is `receiver_manager`, and the live receiver module exports
  receiver/card/request/scan contracts instead of a stealth-address subtree
- No legacy stealth-address family remains on the live path;
  receiver-card, payment-request, and scan-output contracts are the active
  receive surface
- The public and internal manager contracts are now `ReceiverManager`,
  `AsyncReceiverManager`, `ReceiverManagerConfig`, `ReceiverManagerError`, and
  `ReceiverManagerResult`
- The wallet config/env seam is now receiver-native on the live path:
  `wallet.receiver.cache_size`, `wallet.receiver.rate_limit.*`,
  `Z00Z_WALLET_RECEIVER_CACHE_SIZE`, and
  `Z00Z_WALLET_RECEIVER_DERIVE_*` replace the old address-era names
- The dead RPC DTO aliases `RuntimeAddressFilter`, `PersistAddressInfo`, and
  `RuntimeListAddressesResponse` have been removed from the active type layer
- The wallet source tree no longer carries the old address-routing names; the
  live receive code is centered on `card`, `request`, and `scan`, and the
  stale `z00z_address.tar.gz` archive sibling is absent from the current tree
- The wallet source tree now also has a single canonical physical receive path:
  `crates/z00z_wallets/src/receiver/`. The old
  `crates/z00z_wallets/src/address/` tree is absent, `src/lib.rs` declares
  `pub mod receiver;`, and `src/core.rs` directly re-exports
  `crate::receiver` instead of aliasing `crate::address`
- Active wallet terminology is receiver-native beyond module paths:
  `format_receiver_handle`, `return_receiver`, `receiver_mode`,
  `allowed_recipients`, `ReceiverByKind`, `INDEX_RECEIVER_BY_KIND_TABLE`,
  `index_receiver_by_kind`, and `index.receiver_by_kind` replaced the old
  active address-shaped names while keeping frozen domain labels and BIP44
  vocabulary intact
- Receiver/stealth cryptographic flow was rechecked after the physical rename:
  receiver cards, request parsing and signatures, owner tags, tag16 scan
  context, KDF binding, direct scan, and sender output self-validation remain
  aligned with the canonical stealth receive model
- The old `z00z-wallet-addr-convert.rs` helper binary and Cargo manifest entry
  are absent; the targeted wallet-crate residue scan is clean for the old
  address-era symbols, DTO names, config keys, helper binary names, and
  filenames
- The unwired duplicate session files
  `wallet_service_session_seed_derivation.rs` and
  `wallet_service_session_snapshot.rs` have been deleted from the source tree
  after confirming the active session root does not include them

- The repository has a verified completed planning chain from Phase 025 through
  Phase 031, not only an isolated Phase 031 closeout
- That baseline includes crypto, core, utils, storage, and wallet hardening,
  followed by long-file refactoring and architecture cleanup
- Phase 033 is closed on repository-backed summary artifacts
- Phase 034 is complete through Plan 09 on repository-backed summary artifacts
- That completed Phase 034 baseline includes claim continuity, regular
  spend-nullifier semantics, the summary-backed sender-authority retirement
  slice from `034-05`, the backend-owned checkpoint contract chain, the
  semantic validation waves, the active documentation allowlist
  reclassification, the phase-local closure package for Q63/Q64/Q65/Q47, and
  the executed post-closure hygiene chain `034-16` -> `034-17` -> `034-18`
- Phase 035 is complete through Plan 19 on repository-backed summary and review
  artifacts
- That completed Phase 035 baseline includes downstream sender-adapter
  convergence, temp-doc truth correction, the sender validation wave, the
  sender acceptance gate for `035-30` and `035-31`, the bounded stealth scope
  plus receiver-secret inventory or narrowing slice for `035-32` through
  `035-34`, frozen card-bound and request-bound stealth derivation vectors, the
  bounded core-side V2 memo decode contract for `035-35` through `035-37`, the
  live V2 memo receive-path closure for `035-38` through `035-40`, the Plan 17
  rename-authority freeze and Wave A file rename slice for `035-41` through
  `035-43`, the Plan 18 wallet DB plus egui rename and mirror/declaration slice
  for `035-44` through `035-46`, and the Plan 19 declaration/reference sweep,
  validation, and acceptance closure for `035-47` through `035-49`
- The final Phase 035 rename acceptance closure keeps the explicit
  `Doublechecked No-Change Calls` frozen, limits old-name residue outside the
  active lane to historical docs or inventory material, and is backed by a
  green rerun of the mandatory bootstrap gate during the final continuity pass
- Phase 035 now also has a partial validation artifact at
  `.planning/phases/035-mix2-fixes/035-VALIDATION.md`
- Phase 035 now also has an eval applicability audit at
  `.planning/phases/035-mix2-fixes/035-EVAL-REVIEW.md`
- That validation pass added
  `crates/z00z_wallets/tests/test_phase035_rename_guards.rs`, reran the Phase
  035 sender, stealth, and simulator-targeted tests green, and recorded a
  fresh `=== BOOTSTRAP COMPLETE ===` quick-gate result
- That eval audit ran in State B with no `AI-SPEC.md`, confirmed Phase 035 is
  a non-AI wallet, stealth, simulator, and rename-closeout phase, and recorded
  a `PRODUCTION READY` verdict for AI-eval applicability with zero critical
  gaps because no model, prompt, retrieval, or agent runtime exists in scope
- The workspace now also has a validated
  `.github/skills/alert-concept-drift/` skill for historical-anchor concept
  drift audits that separate healthy evolution from suspicious drift and route
  non-trivial findings through `doublecheck`
- The workspace now also has a validated
  `.github/skills/attack-surfaces-create/` skill for seeded static discovery
  of security, cryptography, and threat attack surfaces with append-only JSONL
  inventory updates, skeptical pro-con audit gating, and one-candidate-only
  verified report admission

## 🔄 Active Delta

- Phase 043 is no longer planning-closed at ten plans. The original
  `043-SUMMARY.md` and `043-coverage.md` remain the honest baseline for the
  landed `043-01` through `043-10` chain, but the phase has now been reopened
  for an additive spec-2 slice: `043-fixes-spec-2.md`, `043-TODO-2.md`, and
  new sequential plans `043-11-PLAN.md` through `043-16-PLAN.md` cover the
  remaining public-conservation witness boundary, typed manual asset-class
  audit outcome contract, canonical wallet-prefixed JSONL tx-history artifact,
  and live tx-store versus RPC export naming/boundary work. `.planning/STATE.md`
  now reflects that reopened planning state with `043-11` as the next plan.

- Phase 044 wallet asset lifecycle work is still the active dirty-tree lane,
  and the PH44 send/admit/reconcile or history/offline sub-slice is now
  evidence-backed on the live wallet path. Confirmed tx journal rows persist
  `TxConfirmationEvidence`, broadcast or admission stores typed confirmation
  evidence without marking wallet rows confirmed, reconcile consumes stored
  evidence instead of re-running inline confirmation, and history or details
  receipts now prefer persisted evidence over synthetic tx-hash-only roots.
  Focused verification passed on 2026-05-12 with
  `cargo check -p z00z_wallets --tests --features test-params-fast,wallet_debug_tools`,
  the focused PH44 lib tests
  `test_tx_broadcast_admits_without_confirming`,
  `test_tx_reconcile_requires_confirmation_evidence`,
  `test_tx_reconcile_rejects_mismatched_evidence`,
  `test_tx_import_reconcile_portable`, the receipt checks
  `test_tx_history_includes_receipt` and
  `adapters::rpc::methods::tx_rpc_storage::tests::tx_info_to_details_decodes_package_rows`,
  plus `tx_history_appends_admission_sequence`. A follow-up real-code
  `044-wallets-patch.md` doublecheck fixed the simulator Stage 4 card-gate
  negative assertions, revalidated the Scenario 1 Alice-to-Bob tx package
  history guard, proved multi-transaction JSONL append sequencing, and ran the
  full `z00z_simulator` release test suite green with no forbidden live legacy
  tx-history source patterns found. This is still not a full workspace
  closeout or version-manager release.

- Phase 042 wallet-crate name and path migration is complete on the live code
  path: `wallet.key.list_receivers`, `wallet.key.validate_receiver_card`, and
  `wallet.key.label_receiver` remain the active public receiver methods, and
  the implementation beneath them is clean for the targeted address-era source
  residue inventory
- The remaining active delta is bookkeeping rather than code migration:
  broader `.planning` synchronization and any repository-level release notes
  should be refreshed separately if Phase 042 needs a formal planning closeout

- Phase 040 is the active execution surface on `040-10-PLAN.md`.
  `040-09-SUMMARY.md` remains the honest statement-bound historical baseline.
- The live wallet spend path uses the canonical suite
  `regular_spend_theorem_bpplus` and `CanonicalSpendProofBackend`.
- `crates/tari_utilities_bridge` is deleted from the dirty tree and the
  workspace now uses the direct vendored Tari utilities path through
  `crates/z00z_crypto/tari/utils`; do not restore the bridge crate.
- `SpendProofWitness` now carries explicit membership witnesses in addition to
  `receiver_secret` and ordered `input_s_in` values; the struct intentionally
  has no derives because `ReceiverSecret` does not implement the usual clone or
  debug traits.
- `CanonicalSpendProofBackend` now validates statement shape, per-input
  membership against `prev_root`, nullifier/order/balance rules, and output
  range relation before producing the deterministic canonical artifact.
- Direct backend verification now validates public relations before accepting
  deterministic canonical artifact bytes: output range proofs, duplicate input
  refs, duplicate input theorem leaf IDs, duplicate nullifiers, duplicate output
  theorem leaf IDs, input/output theorem leaf overlap, and balance equation.
- The spend statement builder and Phase 040 test fixture intentionally project
  output theorem leaves into the output `leaf_ad_id` namespace by setting
  `AssetLeaf.asset_id` to the output `leaf_ad_id` inside `SpendProofStmt`, while
  storage/package asset IDs remain bound in the canonical statement bytes.
- The statement-only `SpendProofStmt::new(...)` bypass is closed by statement
  shape validation and backend tamper coverage.
- Scenario 1 Stage 4 now derives membership witnesses from the prep file and
  passes them through the membership-aware wallet witness gate and proof path.
- The wallet lib test `core::tx::witness_gate::tests::test_gate_typed_root`
  now builds the matching membership root instead of using an arbitrary typed
  root fixture, so the full workspace verify gate no longer fails on the new
  membership-root contract.
- The canonical full workspace verify gate passed with exit code 0 and wrote
  the long-running test inventory to `reports/full_verify-report-long-running-tests.txt`.
- `z00z_rollup_node` now has a focused settlement guard:
  `verify_settlement_theorem` checks tx package structure and digest, the
  wallet public spend theorem contract, checkpoint statement proof payload,
  artifact/link/exec-id binding, spend-root-to-checkpoint-root alignment, and
  tx row inclusion in the checkpoint execution input. This closes the previous output-proof-only blind
  spot for rollup admission, but it remains a public-artifact binding guard,
  not a public proof-of-knowledge backend.
- Phase 040 planning, validation, UAT, closeout, and stage-surface guard wording
  has been narrowed to `internal theorem-relation closure` instead of overbroad
  `full theorem-level closure` language.
- Phase 040 UAT and TODO traceability were refreshed on 2026-04-29 so the
  exact stage-surface phrase, direct public-relation verifier hardening, output
  `leaf_ad_id` theorem projection, and forged relation test coverage are all
  visible in the active authority chain.
- The Phase 040 continuation review loop ran three independent passes after the
  current-authority legacy-phrase grep; all three reported no significant
  issues, including two consecutive clean passes and a third clean pass.
- The Phase 040 implementation and tracking changes have been committed and
  synced through the repository-owned version-manager workflow on `z00z-dev`.
  The first minor release commit landed as `v2.113.0`; the follow-up patch
  continuity correction lands as `v2.113.1` so the committed memory bank does
  not preserve stale pre-commit wording.
- The 2026-04-29 review-hardening and traceability sync is currently a working
  tree delta, not a version-manager release commit.

## ⚠️ Open Gaps And Watchpoints

- Phase 043 now has a second planning slice queued but not yet implemented.
  Do not describe the spec-2 contracts as landed until `043-11` through
  `043-16` are executed and the additive evidence is folded back into
  `043-coverage.md` and `043-SUMMARY.md`.
- Until the spec-2 slice lands, current code still keeps the original honest
  limitations: `AssetClassAuditReport` exists but not the full typed outcome
  contract from spec-2, the forensic transport remains encrypted-only without a
  canonical wallet-prefixed JSONL history helper on the live path, and the
  wallet tx-history store remains directory-based while `outputs/tx_exports`
  stays separate.

- Phase 044 is no longer spec-only, but it is not fully closed. Do not describe
  wallet asset lifecycle closure as complete until the broader phase gates,
  planning closeout, and any required full workspace verification are done.
- For Phase 044 implementation, do not introduce `Validated` as a spendable
  wallet asset status and do not use `ScanStorage` cursor progress as tx
  confirmation proof. Reconciliation must match storage/checkpoint evidence
  against tx journal expectations before moving value to `Available` or
  `Spent`; rollback or conflicting evidence must keep value non-spendable under
  `ReorgPending` until resolved.

- The wallet-crate receiver migration is complete and release-tested, but the
  broader `.planning/phases/042-refactor-wallets/` artifacts may still contain
  older intermediate wording until a dedicated planning-sync pass updates them
- Do not rewrite frozen domain labels such as
  `app/z00z_wallets/address/cache_snapshot/...`, BIP44 `address_index` style
  vocabulary, or external Tari reference docs as part of receiver cleanup;
  those are classified non-active residues, not compatibility shims

- Phase 040 may claim only wallet/simulator internal theorem-relation closure
  at this point. Public/trustless proof-of-knowledge remains open.
- Checkpoint theorem finality and full public or trustless rollup settlement
  closure remain open; the new rollup guard is an important checkpoint-linked
  admission check, but do not phrase it as final public theorem closure.
- The verifier still validates a deterministic canonical artifact and statement
  binding rather than a public cryptographic proof of witness knowledge.
- The Phase 026 protected-network positive-anchor success gap and Phase 032
  reopened verification wording remain historical open watchpoints.
- Memory-bank maintenance is still documentation-driven rather than enforced by
  repository automation.

## ⏭️ Next Actions

- For the reopened Phase 043 slice, execute `043-11-PLAN.md` through
  `043-16-PLAN.md` in order, keeping the exact `043-TODO-2.md` task wording
  intact and preserving the original `043-SUMMARY.md`/`043-coverage.md`
  baseline until additive evidence is actually landed.
- Continue Phase 044 from the current dirty implementation state: preserve the
  green focused wallet and simulator gates, reconcile the Phase 044 planning
  artifacts with the landed code, and decide whether the next gate is broader
  wallet tests, simulator tests, or the repository full verify path.
- Keep Phase 044 final states aligned with current code vocabulary:
  `Available`, `Spent`, and tx `Confirmed`; use `ConfirmationReceipt` for
  validation evidence, and keep internal-only `WalletTxStatus` mapping rules
  separate from persisted tx status names.

- If Phase 042 needs formal process closure, refresh the phase ledger, wave
  log, and summaries to match the completed physical `receiver/` migration and
  the release-test evidence now recorded here
- Keep the deterministic BIP32/BIP44 recovery substrate intact; do not treat
  BIP44 `address_index` vocabulary as receiver concept drift

- Keep Phase 040 language scoped to `internal theorem-relation closure` unless
  a real public proof-of-knowledge verifier path is implemented and validated.
- If closure continues, add any missing `040-10` summary or handoff artifact
  only after preserving the current internal-only wording and the full-verify
  evidence.
- Use the focused green evidence first when debugging regressions:
  `test_spend_proof_backend`, `test_tx_proof_verifier`, `test_tx_tamper`,
  `test_tx_wrong_root`, `test_spend_witness_gate`, `scenario_1`, and
  `test_scenario1_stage_surface`.
- For the rollup settlement guard, use
  `cargo test -p z00z_rollup_node --features test-params-fast --all-targets` and
  `cargo clippy -p z00z_rollup_node --features test-params-fast --all-targets -- -D warnings`
  as the focused validation pair.
- Do not restore `crates/tari_utilities_bridge` from git history unless the
  user explicitly requests that.
- Treat Phase 026 and Phase 032 historical watchpoints as still live until
  future repository-backed evidence closes them.

## 🗂️ Source Map

- For Phase 044 wallet asset lifecycle work:
  `.planning/phases/044-wallet-assets/044-wallets-assets-spec.md`,
  `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`,
  `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`,
  `crates/z00z_wallets/src/persistence/assets/asset_storage_impl.rs`,
  `crates/z00z_wallets/src/tx/lifecycle.rs`,
  `crates/z00z_wallets/src/tx/state_checkpoint.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs`,
  `crates/z00z_storage/src/checkpoint/build.rs`, and
  `crates/z00z_storage/src/checkpoint/artifact_types.rs`

- For Phase 042 source-of-truth artifacts: `.planning/phases/042-refactor-wallets/042-z00z_address-remove-spec.md`, `.planning/phases/042-refactor-wallets/042-05-PLAN.md`, `.planning/phases/042-refactor-wallets/042-05-spec-coverage.md`, `.planning/phases/042-refactor-wallets/042-05-wave-log.md`, `.planning/STATE.md`
- For the current completed Phase 042 receiver migration: `crates/z00z_wallets/src/lib.rs`, `crates/z00z_wallets/src/core.rs`, `crates/z00z_wallets/src/receiver/mod.rs`, `crates/z00z_wallets/src/receiver/card/stealth_card.rs`, `crates/z00z_wallets/src/receiver/card/stealth_card_codec.rs`, `crates/z00z_wallets/src/receiver/request/stealth_request_types.rs`, `crates/z00z_wallets/src/receiver/request/stealth_request_parse.rs`, `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs`, `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs`, `crates/z00z_wallets/src/receiver/scan/leaf_scan.rs`, `crates/z00z_wallets/src/stealth/output/output.rs`, `crates/z00z_wallets/src/stealth/output/output_build.rs`, `crates/z00z_wallets/src/stealth/output/output_validator.rs`, `crates/z00z_wallets/src/wallet/policy.rs`, `crates/z00z_wallets/src/db/codecs/schema_keys.rs`, `crates/z00z_wallets/src/db/redb/store/redb_wallet_store_tables.rs`, `crates/z00z_wallets/src/db/redb/store/redb_wallet_store_objects.rs`, `crates/z00z_wallets/src/db/redb/store/redb_wallet_store_debug_export.rs`, `crates/z00z_wallets/src/db/redb/migrations/redb_wallet_store_migrations_tables.rs`, `crates/z00z_wallets/src/db/redb/schema/redb-schema.yaml`, `crates/z00z_wallets/src/wasm/storage_backend.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/key.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/key_impl.rs`, `crates/z00z_wallets/src/adapters/rpc/types/key.rs`, `crates/z00z_wallets/README.md`, and the wallet tests under `crates/z00z_wallets/tests/`

- For project scope and intent: `projectbrief.md`, `productContext.md`
- For architecture and recurring design choices: `systemPatterns.md`
- For tools, constraints, and verified technical baseline: `techContext.md`
- For delivered state and known gaps: `progress.md`
- For execution history and task-level continuity: `tasks/_index.md` and the
  individual task files
- For repository evidence behind this snapshot: `.planning/phases/035-mix2-fixes/035-12-SUMMARY.md`, `.planning/phases/035-mix2-fixes/035-13-REVIEW.md`, `.planning/phases/035-mix2-fixes/035-13-SUMMARY.md`, `.planning/phases/035-mix2-fixes/035-14-SUMMARY.md`, `.planning/phases/035-mix2-fixes/035-15-REVIEW.md`, `.planning/phases/035-mix2-fixes/035-15-SUMMARY.md`, `.planning/phases/035-mix2-fixes/035-16-SUMMARY.md`, `.planning/phases/035-mix2-fixes/035-17-REVIEW.md`, `.planning/phases/035-mix2-fixes/035-17-SUMMARY.md`, `.planning/phases/035-mix2-fixes/035-18-REVIEW.md`, `.planning/phases/035-mix2-fixes/035-18-SUMMARY.md`, `.planning/phases/035-mix2-fixes/035-19-REVIEW.md`, `.planning/phases/035-mix2-fixes/035-19-SUMMARY.md`, `.planning/phases/035-mix2-fixes/035-EVAL-REVIEW.md`, `.planning/phases/035-mix2-fixes/035-VALIDATION.md`, `.planning/phases/036-rename/036-a1-versioning-spec.md`, `.planning/phases/036-rename/036-TODO-2.md`, `.planning/phases/036-rename/036-CONTEXT.md`, `.planning/phases/036-rename/036-04-PLAN.md`, `.planning/phases/036-rename/036-05-PLAN.md`, `.planning/phases/036-rename/036-06-PLAN.md`, `.planning/phases/036-rename/036-07-PLAN.md`, `.planning/phases/036-rename/036-07-SUMMARY.md`, `.planning/phases/036-rename/036-08-PLAN.md`, `.planning/phases/036-rename/036-08-SUMMARY.md`, `.planning/phases/036-rename/036-09-PLAN.md`, `.planning/phases/036-rename/036-09-SUMMARY.md`, `.planning/phases/036-rename/036-10-PLAN.md`, `.planning/phases/036-rename/036-10-SUMMARY.md`, `.planning/phases/036-rename/036-a2-legacy-removing-spec.md`, `.planning/phases/036-rename/036-TODO-3.md`, `.planning/phases/036-rename/036-11-PLAN.md`, `.planning/phases/036-rename/036-12-PLAN.md`, `.planning/phases/036-rename/036-13-PLAN.md`, `.planning/phases/036-rename/036-14-PLAN.md`, `.planning/phases/036-rename/036-15-PLAN.md`, `.planning/phases/036-rename/036-16-PLAN.md`, `crates/z00z_wallets/tests/test_phase035_rename_guards.rs`, `.planning/STATE.md`, `.planning/ROADMAP.md`, and the broader `.planning/` phase artifacts
- For Phase 040 evidence behind the current snapshot: `.planning/STATE.md`,
  `.planning/ROADMAP.md`, `.planning/phases/040-spend-proof/040-10-PLAN.md`,
  `.planning/phases/040-spend-proof/040-VALIDATION.md`,
  `.planning/phases/040-spend-proof/040-UAT.md`,
  `.planning/phases/040-spend-proof/040-CLOSEOUT-GATES.md`,
  `crates/z00z_wallets/src/core/tx/spend_proof_backend.rs`,
  `crates/z00z_wallets/src/core/tx/spend_verification.rs`,
  `crates/z00z_wallets/src/core/tx/witness_gate.rs`,
  `crates/z00z_wallets/tests/test_spend_proof_backend.rs`,
  `crates/z00z_wallets/tests/test_spend_witness_gate.rs`,
  `crates/z00z_rollup_node/src/lib.rs`,
  `crates/z00z_rollup_node/tests/test_settlement_theorem.rs`,
  `crates/z00z_rollup_node/Cargo.toml`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`,
  `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`,
  and `reports/full_verify-report-long-running-tests.txt`
- For the attack-surface skill addition: `.github/skills/attack-surfaces-create/SKILL.md`,
  `.github/skills/attack-surfaces-create/REFERENCE.md`,
  `.github/skills/attack-surfaces-create/FORMS.md`,
  `.github/skills/attack-surfaces-create/scripts/ssot_attack_surface_scan.py`,
  and `reports/attack-surfaces/skill-self-scan.md`

## 🧭 Update Protocol

- Update this file first whenever the active truth changes
- Keep each section evidence-backed and short enough to scan quickly
- Record only verified baseline in `Verified Baseline`; move speculation or
  future work into `Active Delta` or `Next Actions`
- Reconcile any change here with `progress.md`, `tasks/_index.md`, and the
  affected task file before ending the session
