# Phase 047: Wallet Redesign - Context

**Gathered:** 2026-05-15
**Re-audited:** 2026-05-16
**Status:** Implementation landed; closeout packet under re-audit for evidence reconciliation
**Source:** PRD Express Path (`.planning/phases/047-wallet-redesign/047-wallet-redesign-spec.md`)

## Phase Boundary

Phase 047 replaces snapshot-owned `claimed_assets` persistence with wallet-native
encrypted `.wlt` domain objects and matching indexes while preserving the live
`wallet.tx.*`, receive/scan, backup/restore, TOFU, wallet-config, and simulator
Stage 13 behavior established by the Phase 046 proof lane.

## Explicit Scope

- Add `.wlt` schema support for `WalletProfile`, `OwnedAsset`, optional
  `WalletTx`, optional `WalletTxEvent`, and backup manifest payloads.
- Rebuild wallet-owned asset persistence, lookup, reservation, cancellation,
  reconcile, and restore around `.wlt` objects.
- Keep receive/scan persistence centered on `WalletService::recv_range(...)`.
- Keep transaction input selection centered on `wallet.tx.build_transaction`.
- Cut simulator Stage 13 report language and checks over in the same phase.
- Preserve backup/restore while tx history remains an explicit JSONL sidecar.

## Explicit Non-Goals

- Do not change the storage-root or JMT implementation in `z00z_storage`.
- Do not move ownership detection into a remote JMT or aggregator service.
- Do not claim durable persisted seed-rotation or master-key rotation.
- Do not let `wallet.asset.send_asset`, `split_asset`, `merge_assets`,
  `stake_assets`, `swap_assets`, or `unstake_assets` become confirmed ledger
  mutation authority unless they are rewired through `wallet.tx.*`.

## Source Inputs

- [047-wallet-redesign-spec.md](./047-wallet-redesign-spec.md)
- [047-wallet-addon-spec.md](./047-wallet-addon-spec.md)
- [047-TODO.md](./047-TODO.md)
- [047-SPEC-COVERAGE.md](./047-SPEC-COVERAGE.md)
- [046-CONTEXT.md](../000/046-wallet-addons/046-CONTEXT.md)

## Locked Invariants

- `OwnedAssetPayload` objects are the live wallet-owned asset authority.
- `WalletProfilePayload` replaces Snapshot for live non-asset wallet metadata.
- `WalletPersistenceState.claimed_assets` must leave the normal live write path.
- JSONL tx history remains an explicit sidecar until a later phase moves tx
  records into `.wlt`.
- `recv_range(...)` plus `StealthOutputScanner` stay wallet-side ownership
  authority; remote JMT/aggregator data remains chunk/proof input only.
- Secrets stay in the `secrets` table and are never duplicated into asset, tx,
  profile, or backup manifest objects.
- Runtime defaults must come from `wallet_config.yaml` plus explicit env
  overrides, not hardcoded Rust literals.
- Stage 13 simulator text, report flags, and tests must cut over in the same
  patch series as the runtime wallet implementation.
- No edits may touch `crates/z00z_crypto/tari/**`.

## Architectural Choice

- Keep `.wlt` as the canonical encrypted wallet database.
- Keep the `secrets` table separate from public or encrypted object payloads.
- Keep `WalletProfilePayload` narrow: profile, config, restore metadata only;
  no live `claimed_assets`, tx-history blob, or scan cursor payload belongs
  there.
- Store each wallet-owned asset as its own encrypted `OwnedAssetPayload`
  object.
- Keep JSONL tx history as an explicit sidecar plane until a later dedicated
  `.wlt` tx-record cutover is planned and executed; schema-forward kinds may
  exist, but they must not introduce a parallel live authority in this phase.
- Keep `ScanStatePayload` separate from both profile and asset payloads.
- Add explicit indexes over asset id, asset definition id, spend status,
  pending tx id, and scan position so build/reconcile does not depend on
  rewriting a full snapshot blob.

## Current Code Reality Anchors

- `.wlt` currently persists through `meta`, `secrets`, `objects`, and derived
  index tables such as `index_asset_out_by_def`,
  `index_asset_out_by_spentflag`, and `index_tracked_asset_by_spentflag`.
- Existing encrypted wallet objects already cover wallet root, account,
  derivation state, scan state, app/chain, keys, stealth metadata, TOFU pins,
  and the deprecated Snapshot container.
- Canonical tx history currently remains outside `.wlt` as JSONL and must stay
  an explicit second persistence plane until a later planned migration says
  otherwise.

## Verified Current Seams

- Current `.wlt` substrate still centers on `META_TABLE`, `SECRETS_TABLE`,
  `OBJECTS_TABLE`, `INDEX_MANIFEST_TABLE`, and `ObjectKindId::{WalletRoot,
  Account, DerivationState, ScanState, App, Chain, Keys, StealthMeta,
  TofuPins, Snapshot}`.
- Current encrypted object container still uses `EncryptedObjectPayload` and
  `EncryptedObjectRecord`; the redesign extends this substrate rather than
  inventing a second encryption envelope.
- Current generic object writer is `write_object(...)`, and it allocates a new
  `object_id`. That is acceptable for inserts and wrong for owned-asset status
  transitions, profile rewrites, and singleton updates.
- The existing internal `write_object_with_indexes(...)` pattern already does
  object insert, index-manifest refresh, stale-index replacement, save-sequence
  bump, and integrity update in one RedB write transaction; future waves must
  reuse that path rather than duplicate index logic.
- Current Snapshot persistence still flows through `write_wallet_snapshot(...)`,
  `read_wallet_snapshot(...)`, and `WalletPersistenceState`, which still carries
  `claimed_assets: Vec<AssetWire>`.
- Current receive and claimed-asset mutation boundary still centers on
  `recv_range(...)`, `put_claimed_asset(...)`, `set_claimed_assets(...)`, and
  `list_claimed_assets(...)`.
- Current scan cursor boundary still centers on `read_scan_state(...)` and
  `upsert_scan_state(...)`.
- Current tx RPC compatibility surfaces still include `build_transaction(...)`,
  `cancel_transaction(...)`, `broadcast_transaction(...)`,
  `import_transaction(...)`, and `reconcile_transaction(...)`.
- Current asset RPC compatibility surfaces still include `list_assets(...)`,
  `import_asset(...)`, `receive_asset(...)`, `get_asset_balance(...)`, and
  `get_asset_details(...)`.
- Current tx-history path seam still centers on `wallet_history_jsonl_name(...)`
  and `wallet_history_jsonl_path(...)`.
- Current backup restore entry point still centers on
  `restore_backup_with_mode(...)`.
- Today the live owned-asset path is:
  `recv_range(...) -> StealthOutputScanner -> recv_route(..., ReceiveNext::PersistClaim) -> put_claimed_asset(...) -> wallet_claimed_assets -> create_snapshot(...) -> WalletPersistenceState.claimed_assets -> write_wallet_snapshot(...)`.
- Today transaction building is:
  `wallet.tx.build_transaction -> list_claimed_assets(...) -> exclude pending reserved asset ids from tx history -> AssetSelector::select(...) -> build inputs / outputs / change / fee / proof package`.
- Today reconcile is:
  `wallet.tx.reconcile_transaction -> validate stored tx package and confirmation evidence -> remove spent claimed asset ids -> scan tx outputs with wallet receiver keys -> add wallet-owned change/import outputs -> set_claimed_assets(...) -> rewrite Snapshot claimed_assets`.
- These verified seams are the allowed extension points for Phase 047. Do not
  fork them into a new side path, hidden cache authority, or simulator-only
  rewrite.

## Architectural Rejection Reasons

- Snapshot-owned assets are the wrong ownership boundary: assets are operational
  state, not passive wallet-profile metadata.
- Rewriting one large snapshot blob is the wrong write granularity for
  reservation, reconcile, and restore flows.
- Snapshot-owned assets create poor query shape for spendable lookup and pending
  exclusion.
- Snapshot-centered writes make atomicity across asset state, tx state, and
  scan progress harder to reason about.
- Snapshot as a hidden asset database makes future extension and simulator
  honesty harder.
- Existing `.wlt` index infrastructure already points toward object-per-asset
  design; the redesign must follow that direction instead of adding another
  compatibility-shaped blob.

## Gap-Coverage Promises

- Prove the canonical `wallet.tx.*` lifecycle instead of relying on simulator
  helpers alone.
- Prove transaction building selects from wallet-owned assets.
- Prove pending reservation, cancellation, rebuild, broadcast, and reconcile.
- Prove receiver export/import parity.
- Prove tamper and fail-closed behavior for bad packages or evidence.
- Prove backup restore with `WalletPlusHistory`.
- Preserve TOFU and payment-request negative paths.
- Preserve session-hardening and key-rotation boundaries without overclaiming
  new persisted key semantics.
- Keep `recv_range(...)` plus `StealthOutputScanner` as wallet-side authority.
- Do not imply a JMT-side ownership scanner.
- Reuse existing storage and settlement root helpers; do not introduce custom
  root math.
- Remove stale placeholder or stub wording from runtime docs, simulator text,
  tests, and phase-local spec copies.

## Preserved Runtime Obligations

- Create and open wallets from `.wlt`.
- Keep secrets separated from non-secret object payloads.
- Unlock the session and derive receiver keys.
- Scan chunks with `StealthOutputScanner`.
- Persist scan cursor state.
- Persist newly owned assets discovered by scan.
- List owned assets for balance and UX.
- Select spendable input assets for `wallet.tx.build_transaction`.
- Exclude pending or reserved assets from selection.
- Cancel pending tx and release reservations.
- Broadcast tx and store submitted or admitted status.
- Reconcile confirmed tx and update asset states.
- Add wallet-owned change outputs.
- Import portable tx and detect receiver-owned outputs.
- Restore wallet assets and tx history with all-or-nothing behavior.
- Reject wrong password, corrupted snapshot input, corrupted tx history, and
  tampered packages without mutating existing state.
- Keep TOFU and payment-request validation checks in front of tx-output build
  logic.
- Keep `wallet.asset.*` compatibility surfaces explicitly non-canonical until
  they are backed by the same tx/reconcile authority plane.
- Keep storage and settlement root logic delegated to the existing helpers.
- Keep remote JMT or aggregator inputs as data/proof sources only; wallet-side
  keys still decide ownership.

## Config Cutover Obligations

- Preserve runtime priority
  `Z00Z_WALLET_CONFIG_PATH -> crate-local src/wallet_config.yaml -> embedded fallback mirror`.
- Treat the first two sources as live runtime inputs; the embedded fallback is a
  mirror, not a separate competing config truth.
- Invalid YAML or invalid typed live values must fail closed.
- Existing env overrides may remain highest priority, but YAML becomes the
  documented baseline.
- `wallet.settings.auto_lock_timeout_secs` and
  `wallet.auto_lock.timeout_secs` must resolve from one authoritative source.
- `wallet.backup.location` is a base directory, not a per-wallet absolute path.
- `wallet.recovery.gap_limit` replaces the hardcoded `20` recovery scan width.
- Numeric `RuntimePasswordPolicy` knobs move to YAML even if broader password
  heuristics stay code-owned.
- Chain, network, Tor, receiver-cache, and materially user-visible wallet
  runtime defaults move to YAML or are explicitly justified if temporarily
  excluded.
- `wallet_history_jsonl_name(...)` and `wallet_history_jsonl_path(...)` may
  stay derived from wallet stem and `output_dir`; do not invent a second
  unrelated tx-history root in this phase.
- The hardcoded live defaults currently rooted in create, open, runtime,
  backup, and recovery paths must be removed from production execution, not
  only from tests.

## Plan Map

| Plan | Focus | Primary spec implementation phase |
| --- | --- | --- |
| `047-01` | schema, object kinds, payload versions, index vocabulary, debug decode | Phase 1 |
| `047-02` | object-by-id writes, production index builders, indexed reads | Phase 2 |
| `047-03` | `WalletProfilePayload`, create/open/save cutover, YAML-backed defaults | Phase 3 + wallet config cutover |
| `047-04` | `OwnedAssetPayload` and `.wlt` wallet asset authority | Phase 4 |
| `047-05` | `recv_range(...)`, scan cursor, and receive compatibility boundary | Phase 5 |
| `047-06` | build, reserve, cancel, reconcile, asset list/balance/details views | Phase 6 |
| `047-07` | backup manifest, export pack, staged restore, compatibility bridge removal | Phase 7 |
| `047-08` | Stage 13/doc cutover, existing-test migration, final validation | Phase 8 |

## Execution Order

1. Align schema, payload ids, and debug tooling first.
2. Land the safe low-level update/query primitives before any live asset writes.
3. Move live wallet metadata out of Snapshot and make runtime defaults
   YAML-backed before asset cutover.
4. Introduce the owned-asset store and switch wallet authority to it.
5. Rewire receive and scan persistence to owned assets plus scan-state updates.
6. Move tx lifecycle and user-facing asset views onto owned-asset status
   authority.
7. Rebuild backup/restore and export around profile + owned assets + JSONL
   history.
8. Cut the simulator, docs, and existing tests over and close the final
   validation matrix.

## Cross-Cutting Rules

- Every task in every `047-0N-PLAN.md` must embed exact spec line ranges that
  engineers reread before implementation.
- If the spec names rules, instructions, numeric ids, or code-shape snippets,
  the owning plan task must carry them forward explicitly.
- Do not duplicate existing runtime, store, RPC, or simulator logic when the
  current codebase already has a seam that can be extended.
- Do not introduce a second live authority plane, hidden bridge database, or
  parallel write path to avoid touching the real code path.
- No placeholder or stub behavior is allowed when the specification defines the
  expected implementation shape.
- Existing tests that encode Snapshot or `claimed_assets` as the live authority
  must be upgraded in the same patch series; adding only new tests is
  insufficient.
- Every auto task must use the mandatory verify order from
  [047-TODO.md](./047-TODO.md).

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase-local design authority

- `.planning/phases/047-wallet-redesign/047-wallet-redesign-spec.md` — normative
  storage redesign specification
- `.planning/phases/047-wallet-redesign/047-wallet-addon-spec.md` — Phase 046
  proof-lane spec copy that must stop implying Snapshot is the target authority
- `.planning/phases/047-wallet-redesign/047-TODO.md` — canonical backlog and
  requirement routing for execution
- `.planning/phases/047-wallet-redesign/047-SPEC-COVERAGE.md` — section-by-section
  proof that the full spec is routed into the planning packet

### Wallet store and schema

- `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs` — object kinds,
  payload structs, table constants, index update types
- `crates/z00z_wallets/src/db/redb_wallet_store/mod.rs` —
  `is_supported_payload_version(...)` and store exports
- `crates/z00z_wallets/src/db/redb_wallet_store/objects/mod.rs` — live object
  write path and index manifest replacement logic
- `crates/z00z_wallets/src/db/redb_wallet_store/queries.rs` — object reads and
  future indexed lookup helpers
- `crates/z00z_wallets/src/db/redb/schema/redb-schema.yaml` — on-disk schema
  truth that must match Rust ids exactly
- `crates/z00z_wallets/src/db/schema_keys.rs` — `IndexTable` and meta pointer
  vocabulary
- `crates/z00z_wallets/src/db/index_codecs/mod.rs` — canonical index key/value
  validation and size contracts

### Runtime wallet store, receive, tx, and backup touchpoints

- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock_open.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_runtime.rs`
- `crates/z00z_wallets/src/services/app/app_wallet_lifecycle.rs`
- `crates/z00z_wallets/src/services/wallet_paths.rs`
- `crates/z00z_wallets/src/wallet_config.yaml`

### RPC and tx-history surfaces

- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`

### Simulator and final proof lane

- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/report.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/storage.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/runner_contract_table.in`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`

## Spec Coverage Routing

- `047-01` carries the schema ids, payload versions, YAML parity, and debug
  decode contract for new object and index vocabulary.
- `047-01` carries `REQ-012` through `REQ-014` and the schema/index drift fix.
- `047-02` carries the low-level `write_object_by_id(...)` and indexed read
  helper contract that keeps one asset on one `object_id`.
- `047-03` carries `WalletProfilePayload` plus YAML default cutover across
  create/open/runtime/backup/recovery paths.
- `047-04` carries `OwnedAssetPayload`, `WalletAssetStore`, duplicate-id
  rejection, and service authority replacement.
- `047-05` carries `recv_range(...)` persistence, scan-state coupling, and
  receive compatibility boundaries.
- `047-06` carries the build/reservation/cancel/reconcile transaction authority,
  preserves TOFU/payment-request validation ordering, and keeps asset-facing RPC
  and compatibility ops non-canonical unless they route through the same tx
  lane.
- `047-07` carries backup manifest, export pack, staged restore, and snapshot
  compatibility bridge retirement.
- `047-08` carries simulator cutover, doc alignment, existing-test migration,
  final phase validation, and stale authority-string removal from phase-local
  docs.

## Validation Obligations

### Unit and store coverage

- Insert one `OwnedAsset`, reopen the wallet, and list it.
- Reject duplicate asset ids unless replay is explicitly idempotent and payload
  matches.
- Prove status updates preserve one stable `object_id` and replace stale index
  rows.
- Prove index reads return the same records as full object scans.
- Fail closed on corrupt object payloads.
- Detect missing index rows during open or index validation.
- Keep scan cursor and asset insert atomic, or prove replay-safe idempotence for
  any temporary two-step fallback.
- Exclude `PendingSpend` assets from selection.
- Release pending assets on cancel.
- Mark spent inputs and add change outputs on reconcile.
- Reject duplicate asset ids during restore.
- Leave the existing wallet unchanged on wrong-password restore failure.
- Keep Snapshot Rust kind ids aligned with schema documentation.
- Parse expanded `wallet_config.yaml` successfully and fail closed on invalid
  coupled fields.
- Read wallet create/open/settings defaults from the same YAML-backed values.
- Derive backup defaults from `wallet.backup.*`, including `location`.
- Use `wallet.recovery.gap_limit` instead of the hardcoded recovery width.
- If any temporary Snapshot compatibility remains, it must convert legacy data
  into `OwnedAssetPayload` objects exactly once and never re-enter the normal
  live write path.

### Wallet RPC coverage

- `wallet.tx.build_transaction` selects only `Spendable` assets.
- `wallet.tx.list_pending_transactions` reflects reserved inputs.
- `wallet.tx.cancel_transaction` releases inputs.
- `wallet.tx.reconcile_transaction` updates asset states and tx status.
- `wallet.tx.import_transaction` creates receiver-owned assets through
  wallet-side scan logic.
- `wallet.asset.list_assets` reads from `OwnedAsset` objects, not Snapshot.
- `wallet.asset.import_asset` persists through `OwnedAsset` objects.
- `wallet.asset.get_asset_balance` computes total, pending, and available from
  `OwnedAsset` state plus pending reservation bindings.
- `wallet.asset.get_asset_details` reads from `OwnedAsset` objects.
- Wallet create/recover/backup RPC flows honor YAML-backed defaults on live
  paths.

### Simulator coverage

- Stage 13 proves the live `wallet.tx.*` lifecycle.
- Stage 13 proves assets survive lock and reopen.
- Stage 13 proves backup and restore with wallet state plus tx history.
- Stage 13 proves tamper failure does not mutate asset objects.
- Stage 13 reports must distinguish wallet-owned asset object state, tx-history
  state, scan-cursor state, and storage or JMT roots.
- Stage 13 contract, config, and report text must stop calling encrypted
  snapshot payloads the live claimed-asset authority.
- If Stage 13 claims reopen or `WalletPlusHistory` proof, it must execute those
  operations for real, not only log a report note.

## Existing Test Migration Rules

- Rewrite stale assertions that require Snapshot or `claimed_assets` to remain
  the live post-cutover authority.
- Keep compatibility tests only where they explicitly prove one-shot import or
  export migration from legacy Snapshot data.
- Do not keep dual assertions where both Snapshot authority and `OwnedAsset`
  authority appear canonical in the same green suite.

## Acceptance Coverage Notes

- `AC-001` closes in `047-03`.
- `AC-002` through `AC-004` close in `047-04` and `047-05`.
- `AC-005` through `AC-009` close in `047-06`.
- `AC-010` and `AC-011` close in `047-07`.
- `AC-012` closes in `047-08`.
- `AC-013` closes across `047-03` and `047-07`.

## Drift Bars

- Do not add a simulator-only asset table.
- Do not make remote JMT ownership detection authoritative.
- Do not store wallet secrets inside asset records.
- Do not keep growing `WalletPersistenceState` into a hidden database.
- Do not infer pending reservations only from JSONL once asset records exist.
- Do not recompute storage roots with custom simulator math.
- Do not describe `wallet.asset.send_asset` as canonical confirmed spend until
  it is backed by the same tx/reconcile path.
- Do not keep the name Snapshot for live mutable wallet state if legacy
  compatibility is no longer needed.
- Do not leave a dual authority where Snapshot and `OwnedAssetPayload` both look
  canonical in green code or green tests.
- Do not add production logic that depends on `#[cfg(test)]` constructors or
  test-only index helpers.
- Do not overload old asset-output index names if the owned-asset semantics are
  different enough to require explicit new index variants.
- Do not treat Stage 13 report flags as proof for reopen/restore behavior unless
  the simulator actually executes those operations.
- Do not preserve hardcoded `auto_lock_timeout`, `default_fee`,
  `currency_display`, or recovery gap width on live runtime paths after the YAML
  cutover lands.

## Test Artifacts

- `crates/z00z_wallets/src/services/wallet_service_tests.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_tests.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/tests.rs`
- `crates/z00z_wallets/src/db/redb_wallet_store/tests.rs`
- `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`

---

*Phase: 047-wallet-redesign*
*Context gathered: 2026-05-15 via PRD Express Path*
