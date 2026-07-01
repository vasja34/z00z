# Phase 047 Wallet Storage Redesign Specification

**Status:** Implementation-ready storage specification
**Date:** 2026-05-15
**Reader:** Wallet engineer or simulator engineer implementing the `.wlt` asset-store refactor.
**Post-read action:** Implement wallet-owned asset persistence as first-class encrypted `.wlt` objects without losing the current `wallet.tx.*`, receive/scan, backup/restore, TOFU, or simulator behavior.

This Phase 047 document supersedes the live-asset-authority part of the earlier Phase 046 demo/storage notes while preserving the behavioral proof obligations those notes established.

## 📌 Scope

This document is no longer only a suggestion memo. It is the target storage specification for replacing live Snapshot-owned `claimed_assets` with wallet-native `.wlt` domain objects.

In scope:

1. `.wlt` schema additions for wallet profile, owned assets, optional tx records, optional tx events, and backup manifest payloads.
2. Wallet-owned asset persistence, lookup, reservation, cancellation, reconcile, and restore.
3. Receive/scan persistence boundaries around `WalletService::recv_range(...)`.
4. Transaction input selection through `wallet.tx.build_transaction`.
5. Simulator Stage 13 report language and checks.
6. Backup/restore behavior while tx history is still a JSONL sidecar.

Out of scope:

1. Changing the storage root/JMT implementation in `z00z_storage`.
2. Moving ownership detection into a remote JMT/aggregator service.
3. Claiming durable persisted seed/master-key rotation.
4. Making `wallet.asset.send_asset`, `split_asset`, `merge_assets`, `stake_assets`, `swap_assets`, or `unstake_assets` confirmed ledger mutation authority unless they are rewired through `wallet.tx.*`.

## 🎯 Executive Recommendation

Do not keep wallet-owned assets as a growing `claimed_assets` vector inside the wallet Snapshot blob.

The current implementation is understandable as an incremental bridge: the wallet already had a snapshot/export shape, and adding `claimed_assets: Vec<AssetWire>` made owned assets durable quickly. But as a long-term `.wlt` design, it is the wrong center of gravity. Assets are not wallet profile metadata. They are operational state used by scanning, input selection, pending reservations, cancellation, reconcile, restore, and simulator proofs.

The better design is:

1. Keep `.wlt` as the canonical encrypted wallet database.
2. Keep `secrets` separate from public/encrypted object payloads.
3. Rename or replace Snapshot with a narrow `WalletProfile` object that stores only profile/config/restore metadata.
4. Store each wallet-owned asset as its own encrypted `OwnedAsset` object.
5. Store or migrate transaction lifecycle records into `.wlt` as `WalletTx` / `WalletTxEvent` objects when ready; until then, treat JSONL tx history as an explicit sidecar plane.
6. Keep scan cursor as a separate `ScanState` object, as it already is.
7. Add indexes over asset id, asset definition id, spend status, pending tx id, and scan position so transaction building does not depend on reading and rewriting a full wallet snapshot.

This preserves the current functional behavior while making the storage model more robust, more testable, and easier to extend.

## 🔍 Current Code Reality

The current wallet has three important persistence shapes:

1. `.wlt` RedB tables:
   - `meta`
   - `secrets`
   - `objects`
   - index tables such as `index_asset_out_by_def`, `index_asset_out_by_spentflag`, and `index_tracked_asset_by_spentflag`

2. Encrypted wallet objects:
   - wallet root
   - account
   - derivation state
   - scan state
   - app / chain
   - keys
   - stealth metadata
   - TOFU pins
   - deprecated Snapshot container

3. Transaction history sidecar:
   - canonical JSONL history is currently outside `.wlt`
   - `wallet_<stem>_tx_history.jsonl` stores submitted/admitted/confirmed/imported/exported/cancelled lifecycle records

The important observation is that `.wlt` already has an object-store architecture. The current Snapshot blob is not the natural foundation of the database. It is a compatibility container living inside the object store.

### 🔎 Verified Current Symbols And Signatures

The following symbols and signatures were checked against the current repository before this specification was written. If any of these symbols change, update this section before implementing the plan.

Current `.wlt` table and object substrate:

```rust
pub(crate) const META_TABLE: TableDefinition<&str, &[u8]>;
pub(crate) const SECRETS_TABLE: TableDefinition<&str, &[u8]>;
pub(crate) const OBJECTS_TABLE: TableDefinition<&[u8], &[u8]>;
pub(crate) const INDEX_MANIFEST_TABLE: TableDefinition<&[u8], &[u8]>;

#[repr(u8)]
pub enum ObjectKindId {
    WalletRoot = 1,
    Account = 2,
    DerivationState = 7,
    ScanState = 8,
    App = 15,
    Chain = 16,
    Keys = 17,
    StealthMeta = 18,
    TofuPins = 19,
    Snapshot = 250,
}
```

Current object encryption container:

```rust
pub struct EncryptedObjectPayload {
    pub payload_version: u16,
    pub kind_id: u8,
    pub data: Vec<u8>,
}

pub struct EncryptedObjectRecord {
    pub envelope: AeadEnvelope,
    pub payload_version: u16,
}
```

Current generic object writer:

```rust
pub fn write_object<R: SecureRngProvider>(
    session: &WalletSession,
    kind_id: u8,
    payload_version: u16,
    payload_bytes: Vec<u8>,
    index_updates: &[IndexUpdate],
    rng_provider: R,
) -> WalletResult<u128>;
```

Important implementation constraint:

1. `write_object(...)` allocates a new object id.
2. Asset status updates must not allocate a new object id for the same asset.
3. The implementation must add a production helper that writes an object by an existing object id and refreshes its indexes in the same write transaction.
4. The existing internal `write_object_with_indexes(...)` already performs object insert, index manifest refresh, index row replacement, save-sequence bump, and integrity update inside one RedB write transaction. Use that pattern rather than duplicating index logic.

Current Snapshot persistence:

```rust
pub fn write_wallet_snapshot<R: SecureRngProvider>(
    session: &WalletSession,
    snapshot_bytes: Vec<u8>,
    rng_provider: R,
) -> WalletResult<u64>;

pub fn read_wallet_snapshot(session: &WalletSession) -> WalletResult<SecretBytes>;
```

Current Snapshot payload:

```rust
pub struct WalletPersistenceState {
    pub version: u32,
    pub wallet_id: PersistWalletId,
    pub name: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub password_verifier: PasswordVerifierState,
    pub receiver_deriver: ReceiverDeriverState,
    pub settings: PersistWalletSettings,
    pub seed_salt: Option<[u8; 16]>,
    pub state: WalletState,
    pub claimed_assets: Vec<AssetWire>,
    pub checksum: Option<[u8; 32]>,
}
```

Current receive and claimed-asset mutation boundary:

```rust
pub async fn recv_range(
    &self,
    wallet_id: &PersistWalletId,
    chunks: &[ScanChunk],
    requests: &[PaymentRequest],
    max_ckpt: Option<usize>,
) -> WalletResult<ScanRangeOut>;

pub async fn put_claimed_asset(
    &self,
    wallet_id: &PersistWalletId,
    asset: Asset,
) -> WalletResult<bool>;

pub async fn set_claimed_assets(
    &self,
    wallet_id: &PersistWalletId,
    claimed_assets: Vec<Asset>,
) -> WalletResult<()>;

pub async fn list_claimed_assets(
    &self,
    wallet_id: &PersistWalletId,
) -> WalletResult<Vec<Asset>>;
```

Current scan cursor helpers:

```rust
pub fn read_scan_state(session: &WalletSession) -> WalletResult<Option<ScanStatePayload>>;

pub fn upsert_scan_state<R: SecureRngProvider>(
    session: &WalletSession,
    payload: &ScanStatePayload,
    rng_provider: R,
) -> WalletResult<u128>;
```

Current transaction RPC signatures that must remain behaviorally compatible:

```rust
async fn build_transaction(
    &self,
    session: SessionToken,
    recipient: String,
    amount: u64,
    asset_id: Option<String>,
) -> RpcResult<RuntimeBuildTxResponse>;

async fn cancel_transaction(
    &self,
    session: SessionToken,
    tx_id: PersistTxId,
) -> RpcResult<RuntimeCancelTxResponse>;

async fn broadcast_transaction(
    &self,
    session: SessionToken,
    tx_data: String,
) -> RpcResult<RuntimeBroadcastTxResponse>;

async fn import_transaction(
    &self,
    session: SessionToken,
    tx_data: String,
) -> RpcResult<RuntimeImportTxResponse>;

async fn reconcile_transaction(
    &self,
    session: SessionToken,
    tx_id: PersistTxId,
) -> RpcResult<RuntimeReconcileTxResponse>;
```

Current asset RPC surfaces that must read from the new store where applicable:

```rust
async fn list_assets(
    &self,
    wallet_id: PersistWalletId,
    limit: Option<usize>,
    cursor: Option<String>,
    filter: Option<RuntimeAssetListFilter>,
) -> RpcResult<RuntimeListAssetsResponse>;

async fn import_asset(
    &self,
    session: SessionToken,
    asset_data: String,
) -> RpcResult<RuntimeImportAssetResponse>;

async fn receive_asset(
    &self,
    session: SessionToken,
    asset_id: AssetId,
) -> RpcResult<RuntimeReceiveAssetResponse>;
```

Current asset balance/detail RPC surfaces that also depend on the live owned-asset authority:

```rust
async fn get_asset_balance(
    &self,
    wallet_id: PersistWalletId,
    asset_id: AssetId,
) -> RpcResult<RuntimeAssetBalanceResponse>;

async fn get_asset_details(
    &self,
    wallet_id: PersistWalletId,
    asset_id: AssetId,
) -> RpcResult<RuntimeAssetDetailsResponse>;
```

Current tx-history persistence:

```rust
pub(crate) fn wallet_history_jsonl_name(wallet_stem: &str) -> String;
pub(crate) fn wallet_history_jsonl_path(&self, wallet_id: &PersistWalletId) -> PathBuf;
```

Current backup restore entry point:

```rust
pub async fn restore_backup_with_mode(
    &self,
    backup_path: String,
    password: SafePassword,
    wallet_name: Option<String>,
    mode: ForensicImportMode,
) -> WalletResult<RuntimeRestoreBackupResponse>;
```

Today, wallet-owned assets flow like this:

```text
WalletService.recv_range(...)
  -> StealthOutputScanner detects a wallet-owned output
  -> recv_route(..., ReceiveNext::PersistClaim)
  -> put_claimed_asset(...)
  -> wallet_claimed_assets in memory
  -> create_snapshot(...)
  -> WalletPersistenceState.claimed_assets: Vec<AssetWire>
  -> write_wallet_snapshot(...)
  -> encrypted Snapshot object in .wlt objects table
```

Transaction building then uses:

```text
wallet.tx.build_transaction
  -> list_claimed_assets(...)
  -> exclude pending reserved asset ids from tx history
  -> AssetSelector::select(...)
  -> build inputs / outputs / change / fee / proof package
```

Reconcile uses:

```text
wallet.tx.reconcile_transaction
  -> validate stored tx package and confirmation evidence
  -> remove spent claimed asset ids
  -> scan tx outputs with wallet receiver keys
  -> add wallet-owned change/import outputs
  -> set_claimed_assets(...)
  -> rewrite Snapshot claimed_assets
```

So yes: the wallet currently can function. It can find assets, store them, select inputs, reserve pending assets, cancel and release them, reconcile spent and change outputs, and restore claimed assets through the snapshot path.

The concern is not "this cannot work." The concern is "this is a poor long-term authority boundary."

## ⚠️ Why Snapshot-Owned Assets Are a Weak Design

Snapshot is a good name for an export image or point-in-time backup. It is a bad name and a bad abstraction for the live operational asset set.

Wallet-owned assets are not a profile snapshot. They are the wallet's active spendable inventory. The code needs to query, reserve, mark spent, reconcile, restore, and audit them. Storing them inside one full Snapshot object makes every small asset mutation look like a whole-wallet rewrite.

The main design problems are:

1. **Wrong ownership boundary**
   - `claimed_assets` are domain objects.
   - `wallet name`, `settings`, `receiver_deriver`, and `password_verifier` are wallet profile/config state.
   - Mixing them in one Snapshot couples unrelated mutation rates and unrelated invariants.

2. **Poor write granularity**
   - Adding one newly detected asset rewrites the full Snapshot.
   - Spending one asset and adding one change output rewrites the full claimed set.
   - This is acceptable for tiny demos but gets worse as asset count grows.

3. **Poor query shape**
   - Transaction building wants "spendable assets for wallet, optionally by asset definition, excluding pending reservations."
   - A Snapshot vector forces load/filter behavior instead of a direct indexed lookup.
   - The current code compensates by keeping `wallet_claimed_assets` in memory, but that means open/reload and restore must rebuild this cache correctly every time.

4. **Harder atomicity across related domains**
   - Reconcile changes asset status and tx evidence together.
   - Today tx history is JSONL and assets are Snapshot state, so rollback logic has to bridge two persistence planes.
   - A wallet-native object store can commit asset state, tx record, scan cursor, and index updates in one RedB transaction when those planes are moved into `.wlt`.

5. **Harder future extension**
   - Asset metadata will likely grow: source, confirmation status, spend reservation, proof references, imported/exported lifecycle, claim height, scan provenance, quarantine reason, user labels, policy state.
   - Adding all of that to `WalletPersistenceState.claimed_assets` turns Snapshot into a database hidden inside one field.

6. **Misleading mental model**
   - Calling this object `Snapshot` suggests a point-in-time copy.
   - The live wallet currently treats it as the canonical asset table.
   - That mismatch will confuse future code and future docs.

7. **Index tables already hint at the better design**
   - `.wlt` already defines object and asset-related index tables.
   - Keeping all assets inside Snapshot prevents those indexes from becoming the primary access path.

There are real advantages to the current design, but they are mostly short-term:

1. It was quick to implement.
2. It gives one encrypted payload for backup/restore.
3. It gives an easy checksum boundary.
4. It avoids partial migration work.
5. It keeps the simulator able to prove current behavior without inventing a second asset table.

Those advantages are valid, but they do not outweigh the long-term cost if this wallet is still in development and legacy compatibility is not required.

## 🧭 Gap Coverage From The Two Source Notes

The gap notes identified several real issues. Most of them moved into the Phase 046 spec, but the spec intentionally preserves current storage behavior instead of proposing a better target storage model.

Gaps that did move into the current spec:

1. Prove the canonical `wallet.tx.*` lifecycle instead of relying only on simulator-specific transaction helpers.
2. Show that transaction building selects inputs from the wallet-owned claimed asset set.
3. Show pending reservation, cancellation, rebuild, broadcast, and reconcile.
4. Show receiver export/import parity.
5. Show tamper/fail-closed behavior for bad packages or evidence.
6. Show backup restore with `WalletPlusHistory`.
7. Show TOFU and payment request negative paths.
8. Show session hardening and key-rotation boundary.
9. Keep `recv_range(...)` plus `StealthOutputScanner` as wallet-side scan authority.
10. Do not claim a JMT-side ownership scanner.
11. Reuse existing storage/root proof helpers instead of adding custom root math.
12. Clean stale placeholder/stub wording.

The gap that did not become a design improvement:

1. The notes correctly identified that assets need a real storage authority so future transactions can select inputs.
2. The spec answers that by saying "the current authority is `.wlt` Snapshot `claimed_assets`."
3. That is true as a current-state statement.
4. It is not a strong target architecture.

So the current Phase 046 spec is mostly correct as a demonstration spec, but it should not be treated as endorsement of Snapshot-as-asset-store. Decision 2 should be revised or supplemented:

```text
For Phase 046 demonstration, prove the current `.wlt` Snapshot claimed-asset behavior.
For the wallet storage target, replace Snapshot-owned claimed_assets with native `.wlt`
OwnedAsset objects and indexes.
```

## ⚙️ Requirements The New Design Must Preserve

The replacement design must not lose any behavior that currently works.

Required wallet behaviors:

1. Create/open wallet from `.wlt`.
2. Keep secrets separated from non-secret object payloads.
3. Unlock session and derive receiver keys.
4. Scan chunks with `StealthOutputScanner`.
5. Persist scan cursor.
6. Persist newly owned assets discovered by scan.
7. List owned assets for balance and UX.
8. Select spendable input assets for `wallet.tx.build_transaction`.
9. Exclude pending/reserved assets from selection.
10. Cancel pending tx and release reservations.
11. Broadcast tx and store submitted/admitted status.
12. Reconcile confirmed tx and update asset states.
13. Add wallet-owned change outputs.
14. Import portable tx and detect receiver-owned outputs.
15. Restore wallet assets and tx history with all-or-nothing behavior.
16. Reject wrong password, corrupted snapshot, corrupted tx history, and tampered packages without mutating existing state.
17. Keep TOFU/payment-request checks before building tx outputs.
18. Keep `wallet.asset.*` compatibility surfaces clearly non-canonical until they are tx/reconcile-backed.
19. Keep storage/root proof code delegated to the existing storage and settlement helpers.
20. Keep remote JMT/aggregator as a data/proof source only; wallet-side keys decide ownership.

Required storage properties:

1. Atomic mutation for all fields changed by one wallet operation.
2. Object-level encryption with correct AAD binding.
3. No secret material inside asset records, tx records, or profile objects.
4. Duplicate asset id rejection.
5. Indexed lookup by wallet/account/status/asset definition.
6. Deterministic restore.
7. Idempotent replay or recovery after interrupted writes.
8. Clear migration story while the project is still pre-production.

### ✅ Normative Requirements

Use these requirement ids in implementation plans, tests, and review notes.

1. **REQ-001**: The live authority for wallet-owned assets MUST be encrypted `.wlt` `OwnedAssetPayload` objects, not `WalletPersistenceState.claimed_assets`.
2. **REQ-002**: `WalletPersistenceState.claimed_assets` MUST be removed from the live write path. It MAY exist temporarily only as an import/export compatibility field during a single cutover patch.
3. **REQ-003**: Every owned asset MUST have exactly one canonical `object_id` for its lifetime inside one wallet.
4. **REQ-004**: Asset status transitions MUST update the existing asset object by object id. They MUST NOT create a second object for the same `asset_id`.
5. **REQ-005**: Duplicate `asset_id` insert MUST fail closed unless the existing payload is byte-for-byte or semantically identical and the operation is explicitly idempotent.
6. **REQ-006**: `wallet.tx.build_transaction` MUST select inputs only from `OwnedAssetStatus::Spendable` assets and MUST exclude all assets reserved by the same wallet under `PendingSpend`.
7. **REQ-007**: `wallet.tx.cancel_transaction` MUST release only assets reserved by the cancelled `tx_id`.
8. **REQ-008**: `wallet.tx.reconcile_transaction` MUST atomically mark spent inputs as `Spent`, insert wallet-owned outputs/change as `Spendable`, and update tx status/evidence when tx records are inside `.wlt`.
9. **REQ-009**: While tx history remains JSONL, code MUST preserve the current staged restore behavior and treat `.wlt` plus JSONL as two explicit persistence planes.
10. **REQ-010**: `recv_range(...)` MUST remain the wallet-side ownership authority. Remote JMT/aggregator data MUST remain input chunks/proofs only.
11. **REQ-011**: Asset insert and scan cursor advance SHOULD commit in one `.wlt` write transaction. If this is not possible in the first patch, the code MUST make the operation idempotent on replay and document the temporary two-step failure mode.
12. **REQ-012**: Object payload kind ids and payload versions MUST be updated in Rust and schema YAML in the same patch.
13. **REQ-013**: Schema YAML MUST include every Rust `ObjectKindId` variant. Existing drift around Snapshot, StealthMeta, and TofuPins MUST be fixed before adding new object kinds.
14. **REQ-014**: Index rows MUST be treated as derived accelerators. Object payloads remain canonical, and index rebuild/validation must be possible from object payloads.
15. **REQ-015**: Secrets MUST remain in the `secrets` table. Asset, tx, profile, scan, and backup manifest objects MUST NOT duplicate seed phrase, master key, receiver secret, or signing secret bytes.
16. **REQ-016**: Wallet defaults used on live create/open/recover/backup/runtime paths MUST come from `crates/z00z_wallets/src/wallet_config.yaml` (or an explicit higher-priority override), not from hardcoded Rust literals.
17. **REQ-017**: The existing `wallet_config.yaml` placeholder MUST be expanded in the same patch so it covers every live wallet default introduced or preserved by this redesign.
18. **REQ-018**: The implementation MUST remove or rewrite the current hardcoded defaults in wallet create/open/settings/backup/recovery code paths so the runtime actually consumes YAML-backed values on production execution paths, not only in tests.
19. **REQ-019**: Existing wallet and simulator tests that currently encode Snapshot / `claimed_assets` as the live authority MUST be upgraded in the same patch; adding only new tests is insufficient.
20. **REQ-020**: Runtime wallet code and `crates/z00z_simulator` Stage 13 code/config/reporting MUST cut over together; the refactor is incomplete if only the RedB schema changes.

### 🔧 Wallet Config Cutover

`crates/z00z_wallets/src/wallet_config.yaml` is currently only a partial runtime-config stub. This refactor must turn it into the documented source of wallet defaults actually used on live create/open/recover/backup/runtime paths.

Config-loading rules:

1. Keep runtime priority `Z00Z_WALLET_CONFIG_PATH` -> crate-local `src/wallet_config.yaml` -> embedded fallback mirror.
2. The first two sources are live runtime configuration inputs. The embedded fallback is only a bootstrap mirror and must carry the same documented defaults as the file-backed schema.
3. Invalid YAML or invalid typed values for enabled wallet settings MUST fail closed.
4. Existing env overrides such as `Z00Z_WALLET_OUTPUT_DIR`, `Z00Z_WALLET_NETWORK`, `Z00Z_WALLET_CHAIN`, `Z00Z_WALLET_RECEIVER_CACHE_SIZE`, and the current receiver rate-limit env overrides MAY remain as top-priority deployment overrides, but YAML must become the documented baseline.

Required `wallet_config.yaml` additions for this refactor:

```text
wallet:
  network:
    type
    use_tor_default
    onionet_chain_type
  chain:
    type
    id
    endpoints
  paths:
    output_dir
  receiver:
    cache_size
    async_batch_threshold
    cache_ttl_seconds
    purge_interval_seconds
    purge_min_size
    timing_safe_mode
    rate_limit:
      enabled
      rate_per_sec
      burst
  settings:
    auto_lock_timeout_secs
    default_fee
    currency_display
    policy_rules
  auto_lock:
    timeout_secs
    triggers
  backup:
    auto_backup_enabled
    backup_interval_hours
    location
    encrypt_backups
    create_rate_limit_window_ms
  recovery:
    gap_limit
  security:
    password_policy:
      min_length
      recommended_length
      max_length
  logger:
    rpc:
      enabled
      level
      output.path
      output.rotation.max_bytes
      output.rotation.keep_files
      max_line_bytes
      truncation.non_secret_min_bytes
      truncation.head_chars
      truncation.tail_chars
```

Implementation notes:

1. `wallet.settings.auto_lock_timeout_secs` and `wallet.auto_lock.timeout_secs` MUST be wired from one authoritative config source; do not keep the current drift where persisted wallet settings default to `300` seconds while `AutoLockPolicy::default()` uses `900`.
2. `wallet.backup.location` should be interpreted as a base directory. The default per-wallet backup location may still be derived as `location/<sanitized_wallet_id>` so the config file does not need per-wallet absolute paths.
3. `wallet.recovery.gap_limit` MUST replace the current hardcoded `reconcile_persist_gap_limit(..., 20, ...)` recovery scan width.
4. Password-policy heuristics may remain code-owned, but the numeric `RuntimePasswordPolicy` knobs used by `PasswordValidator::default()` MUST come from YAML.
5. Chain/network transport defaults such as chain endpoints and Tor/onion routing defaults must come from YAML rather than silently falling back to hardcoded `p2p` / `devnet` behavior.
6. Receiver-manager tuning that materially affects runtime behavior should move with the same cutover. If any of `async_batch_threshold`, cache TTL/purge knobs, or timing-safe mode remain code-owned in the first patch, the implementation must document why they are intentionally excluded from the config contract.
7. `wallet_history_jsonl_name(...)` and `wallet_history_jsonl_path(...)` may remain derived from the wallet stem and `output_dir` in the first patch; do not introduce a second unrelated tx-history root unless the refactor also documents backup/restore implications.
8. The implementation patch MUST remove hardcoded live defaults from at least these paths:
   - `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock.rs`
   - `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock_open.rs`
   - `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_runtime.rs`
   - `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
   - `crates/z00z_wallets/src/services/app/app_wallet_lifecycle.rs`

## 🧪 Design Options

### Native `.wlt` Domain Object Store

Interface shape:

```text
.wlt
  meta
  secrets
  objects
    WalletProfile
    Account
    DerivationState
    ScanState
    Keys
    StealthMeta
    TofuPins
    OwnedAsset
    WalletTx / WalletTxEvent
    BackupManifest
  indexes
    asset_id -> object_id
    asset_def_id + status -> object_id set
    status -> object_id set
    tx_id -> object_id set
    scan_position -> object_id set
```

Caller-facing port:

```text
WalletAssetStore {
  put_owned_asset(wallet_id, asset, source, scan_ref)
  list_assets(wallet_id, filter)
  list_spendable(wallet_id, asset_def_id, exclude_pending)
  reserve_inputs(wallet_id, tx_id, asset_ids)
  release_reservation(wallet_id, tx_id)
  confirm_spend(wallet_id, tx_id, spent_asset_ids, new_owned_outputs)
  restore_assets(wallet_id, records)
}

WalletTxStore {
  append_event(wallet_id, tx_id, event)
  get_tx(wallet_id, tx_id)
  list_history(wallet_id, filter, cursor)
  bind_assets(wallet_id, tx_id, input_ids, output_ids)
}
```

What it hides:

1. Encryption envelope.
2. Object ids.
3. Index maintenance.
4. Atomic RedB write transactions.
5. Cache rebuild on open.
6. Rollback/commit ordering.

Pros:

1. Matches the existing `.wlt` object-store direction.
2. Keeps assets encrypted as wallet objects.
3. Gives object-level write granularity.
4. Gives clean indexes for input selection.
5. Makes scan, tx, and restore invariants explicit.
6. Allows future tx history migration into `.wlt`.
7. Avoids a hidden database inside `WalletPersistenceState`.
8. Keeps secrets separate.
9. Gives simulator a better storage contract to prove.

Cons:

1. Requires new object kinds and payload versions.
2. Requires index consistency tests.
3. Requires more migration/refactor work than keeping Snapshot.
4. Requires careful atomic batch APIs so asset and tx mutations do not drift.
5. Requires deciding whether tx JSONL remains a sidecar or becomes `.wlt` objects.

Recommendation:

Choose this. It is the best match for the existing `.wlt` architecture and the future wallet needs.



## ✅ Recommended `.wlt` Organization

The recommended organization is a domain object store inside `.wlt`.

```text
.wlt
  meta
    wallet id
    schema version
    save sequence
    integrity digest
    object pointers for singleton objects
    index manifest

  secrets
    encrypted seed material
    encrypted secret key material
    secret refs only, not duplicated in objects

  objects
    WalletProfile
    Account
    DerivationState
    ScanState
    Keys
    StealthMeta
    TofuPins
    OwnedAsset
    WalletTx
    WalletTxEvent
    BackupManifest

  indexes
    asset_by_id
    asset_by_definition
    asset_by_status
    asset_by_tx
    asset_by_scan_position
    tx_by_status
    tx_by_time
```

### 🔑 `WalletProfile`

`WalletProfile` should replace the live semantic role currently played by Snapshot for non-asset profile state.

Fields:

```text
WalletProfile {
  version: u32
  wallet_id: PersistWalletId
  name: String
  created_at: u64
  updated_at: u64
  password_verifier: PasswordVerifierState
  receiver_deriver: ReceiverDeriverState
  settings: PersistWalletSettings
  seed_salt: Option<[u8; 16]>
  state: WalletState
  checksum: Option<[u8; 32]>
}
```

Rules:

1. No `claimed_assets`.
2. No transaction history.
3. No scan cursor.
4. No secret key material.
5. Preserve all non-asset fields currently required by `WalletPersistenceState`.
6. Persist `WalletState::Locked` when saving an unlocked wallet, matching current Snapshot behavior.
7. Keep `password_verifier` encrypted inside the object. It is not the password, but it is still security-sensitive verifier material.

Name choice:

1. `WalletProfile` is better than `Snapshot` for live state.
2. `Snapshot` may remain as an export/backup term only.
3. If no legacy compatibility is needed, rename now rather than carrying a misleading name.

### 🔑 `OwnedAsset`

Each wallet-owned asset should be one encrypted object.

Suggested fields:

```text
OwnedAssetPayload {
  version: u32
  wallet_id: PersistWalletId
  account_id: Option<u128>
  asset_id: [u8; 32]
  asset_definition_id: [u8; 32]
  asset_wire: AssetWire
  status: OwnedAssetStatus
  source: OwnedAssetSource
  first_seen: Option<AssetSeenRef>
  last_updated_ms: u64
  scan_ref: Option<ScanRef>
  receive_ref: Option<ReceiveRef>
  spend_ref: Option<PersistTxId>
  confirmation_ref: Option<ConfirmRef>
  labels: Vec<String>
  policy: OwnedAssetPolicy
  checksum: Option<[u8; 32]>
}
```

Status enum:

```text
OwnedAssetStatus {
  Spendable
  PendingSpend
  Spent
  PendingReceive
  Quarantined
  Archived
}
```

Source enum:

```text
OwnedAssetSource {
  Scan
  Import
  Change
  Genesis
  Restore
  ManualClaim
}
```

Support structs in this section are new target types. They do not currently exist and must be added by the implementation:

```text
AssetSeenRef:
  height: Option<u64>
  hash_or_root: Option<Vec<u8>>
  local_time_ms: u64

ScanRef:
  start_height: u64
  end_height: u64
  cursor_hash: Vec<u8>

ReceiveRef:
  request_id: Option<String>
  receiver_handle: Option<String>
  import_tx_id: Option<PersistTxId>

ConfirmRef:
  checkpoint_id_hex: Option<String>
  state_root_hex: Option<String>
  evidence_id: Option<String>

OwnedAssetPolicy:
  frozen: bool
  manual_review: bool
  quarantine_reason: Option<String>
```

Field details:

```text
version:
  Payload version for migrations.

wallet_id:
  The owning wallet.

account_id:
  Account/subaccount boundary for future multi-account wallets.

asset_id:
  Canonical asset id derived from the asset leaf/wire data.

asset_definition_id:
  Asset class/definition id used for filtering and balance grouping.

asset_wire:
  The serialized owned asset payload needed to spend or prove ownership. Use
  z00z_core::assets::AssetWire, matching the current Snapshot field type.

status:
  The wallet-local spendability state. Do not infer this only from JSONL once
  OwnedAssetPayload exists.

source:
  Scan
  Import
  Change
  Genesis
  Restore
  ManualClaim

first_seen:
  Optional block/checkpoint height, hash/root, chunk id, and local timestamp.

last_updated:
  Local timestamp or monotonic wallet sequence.

scan_ref:
  Optional pointer to scan cursor/chunk/proof context.

receive_ref:
  Optional payment request id, receiver card id, or import id.

spend_ref:
  Optional tx id if PendingSpend or Spent.

confirmation_ref:
  Optional confirmation evidence id or root binding.

labels:
  User labels or UX tags, if needed later.

policy:
  Wallet-local policy flags such as frozen/quarantined/manual-review.

integrity:
  Record checksum or object-level digest in addition to the encrypted object envelope.
```

Required invariants:

1. `asset_id` MUST equal `asset_wire.to_asset()?.asset_id()`.
2. `asset_definition_id` MUST equal the definition id inside the decoded asset.
3. `status == PendingSpend` MUST have `spend_ref = Some(tx_id)`.
4. `status == Spent` MUST have `spend_ref = Some(tx_id)` unless an explicit repair mode is active.
5. `status == Spendable` MUST NOT have a live pending spend reservation.
6. `status == Quarantined` MUST be excluded from balance and input selection.
7. `asset_wire` MUST be validated with existing asset validation before persistence.

Confidentiality note:

Do not add redundant plaintext amount fields unless the wallet already exposes that information at the same confidentiality level. If amount, commitment, or asset definition metadata is needed for UX, keep it inside the encrypted object and expose it only through wallet APIs that already enforce session rules.

### 🔑 `ScanState`

`ScanState` should remain a separate object.

Suggested fields:

```text
ScanState {
  version
  wallet_id
  account_id
  last_scanned_height
  last_scanned_hash
  last_scanned_root
  scanner_version
  updated_at
}
```

Rules:

1. Scan cursor is not scanner authority.
2. The scanner authority remains wallet-side `recv_range(...)` plus `StealthOutputScanner`.
3. A remote aggregator/JMT service may provide chunks and proofs, but it must not decide wallet ownership.
4. When a scan detects assets, asset inserts and scan cursor update should commit atomically.

### 🔑 `WalletTx` And `WalletTxEvent`

The current JSONL tx history can remain for Phase 046 demonstration, but the target wallet storage should plan for tx records inside `.wlt`.

Suggested `WalletTxPayload` fields:

```text
WalletTxPayload {
  version: u32
  wallet_id: PersistWalletId
  tx_id: PersistTxId
  tx_hash: String
  status: TxStatus
  role: WalletTxRole
  package_bytes: Option<Vec<u8>>
  input_asset_ids: Vec<[u8; 32]>
  output_asset_ids: Vec<[u8; 32]>
  imported: bool
  exported: bool
  submitted_at_ms: Option<u64>
  admitted_at_ms: Option<u64>
  confirmed_at_ms: Option<u64>
  cancelled_at_ms: Option<u64>
  confirmation_evidence_ref: Option<String>
  error_or_reject_reason: Option<String>
}
```

Suggested `WalletTxEventPayload` fields:

```text
WalletTxEventPayload {
  version: u32
  wallet_id: PersistWalletId
  tx_id: PersistTxId
  event_seq: u64
  event_type: WalletTxEventType
  event_time_ms: u64
  payload: Vec<u8>
}
```

Support enums in this section are new target types:

```text
WalletTxRole {
  Sender
  Receiver
  Observer
}

WalletTxEventType {
  Built
  Submitted
  Admitted
  Imported
  Exported
  Cancelled
  Confirmed
  Rejected
}
```

Rules:

1. `WalletTxPayload` gives current state for API queries.
2. `WalletTxEventPayload` gives audit/history and can be optional at first.
3. Asset status and tx status must be updated in the same write transaction whenever one operation changes both.
4. If JSONL remains temporarily, the code must treat `.wlt` plus JSONL as two explicit persistence planes and keep all-or-nothing staged restore semantics.
5. The first implementation MAY keep tx history in JSONL, but it MUST still store pending asset reservations on `OwnedAssetPayload` once the asset store exists.

### 🔑 `Keys` And `Secrets`

Keep keys and secrets separated.

Rules:

1. `secrets` stores encrypted secret material.
2. `Keys` stores public material, fingerprints, and secret references.
3. Asset objects reference key/account context but never embed seed material.
4. Master-key rotation must not be described as durable seed rotation until the secret storage layer actually rewrites persisted seed/master-key material.

## 🔁 Operation Flows In The Recommended Design

### Flow 1: Wallet Create

```text
create_wallet_store
  -> write secrets
  -> write WalletProfile
  -> write Account
  -> write DerivationState
  -> write ScanState
  -> write Keys / StealthMeta / TofuPins
  -> initialize index manifest
```

No asset objects exist at creation unless the wallet is restored from a backup.

### Flow 2: Wallet Open

```text
open_wallet_store
  -> validate object envelopes
  -> read singleton object pointers
  -> load WalletProfile and key refs
  -> load ScanState
  -> lazily load or index-query OwnedAsset objects
  -> rebuild in-memory caches from object store
```

The in-memory cache is a performance optimization, not the authority.

### Flow 3: Scan Detects A New Owned Asset

```text
recv_range(chunks)
  -> read ScanState
  -> derive receiver keys for unlocked wallet
  -> StealthOutputScanner scans public chunk data
  -> scanner identifies Mine leaf
  -> convert leaf to Asset
  -> begin wallet write transaction
  -> insert OwnedAsset object if asset_id is new
  -> update asset indexes
  -> update ScanState cursor
  -> bump save sequence and integrity
  -> commit
```

If the same asset is discovered again, the operation should be idempotent:

1. Same asset id and same wire data: no-op or metadata refresh.
2. Same asset id but conflicting wire data: fail closed and quarantine/report.

### Flow 4: Alice Builds A Transaction To Bob

```text
wallet.tx.build_transaction(Alice, Bob, amount, asset_id?)
  -> validate session, TOFU/payment request, chain binding
  -> query WalletAssetStore.list_spendable(...)
  -> exclude PendingSpend / reserved assets through indexed status
  -> AssetSelector::select(...)
  -> build inputs, recipient output, change output, fee output
  -> create tx package and proofs
  -> begin wallet write transaction
  -> create/update WalletTxPayload as Pending
  -> mark selected OwnedAsset records as PendingSpend with tx_id
  -> commit
```

The important difference from the current design is that pending reservation should live on the asset records and tx record, not only be inferred from a JSONL sidecar.

### Flow 5: Alice Cancels Pending Transaction

```text
wallet.tx.cancel_transaction(tx_id)
  -> validate tx is cancellable
  -> begin wallet write transaction
  -> update WalletTxPayload status to Cancelled
  -> release all OwnedAsset records reserved by tx_id back to Spendable
  -> append WalletTxEventPayload Cancelled if event log is enabled
  -> commit
```

### Flow 6: Broadcast And Reconcile

```text
wallet.tx.broadcast_transaction(tx_data)
  -> verify package
  -> submit package
  -> update WalletTxPayload Submitted / Admitted

wallet.tx.reconcile_transaction(tx_id)
  -> load WalletTxPayload and package
  -> validate confirmation evidence and root binding
  -> scan tx outputs with wallet receiver keys
  -> begin wallet write transaction
  -> mark input OwnedAsset records as Spent
  -> insert wallet-owned change/output OwnedAsset records as Spendable
  -> update WalletTxPayload Confirmed
  -> store confirmation evidence ref
  -> commit
```

This removes the current need to call `set_claimed_assets(...)` with a full replacement vector.

### Flow 7: Bob Receives Or Imports A Transaction

```text
wallet.tx.import_transaction(tx_data)
  -> verify package
  -> check chain id and package digest
  -> scan outputs with Bob receiver keys
  -> begin wallet write transaction
  -> create WalletTxPayload as Imported/Pending
  -> insert Bob-owned output OwnedAsset records as PendingReceive or Spendable
  -> commit
```

Later broadcast/reconcile can move the same tx and asset objects to confirmed states.

### Flow 8: Backup And Restore

Backup should export a coherent wallet pack:

```text
WalletBackupPack {
  manifest
  wallet_profile
  singleton_objects
  owned_assets
  tx_records_or_history
  scan_state
  tofu_pins
  key_refs
  encrypted_secret_material
}
```

Restore should stage everything before mutating live state:

```text
restore_backup
  -> decrypt and decode pack
  -> validate schema versions
  -> validate wallet id / chain binding
  -> validate duplicate asset ids
  -> validate tx references to asset ids
  -> write staged .wlt
  -> write staged tx history if JSONL sidecar still exists
  -> atomically promote staged artifacts
```

If any step fails, the existing wallet must remain unchanged.

## 🧷 Index Strategy

The object store needs indexes that match wallet operations.

Minimum indexes:

```text
asset_by_id:
  key: wallet_id + asset_id
  value: object_id

asset_by_definition_status:
  key: wallet_id + asset_definition_id + status + asset_id
  value: object_id

asset_by_status:
  key: wallet_id + status + asset_id
  value: object_id

asset_by_tx:
  key: wallet_id + tx_id + asset_id
  value: object_id

asset_by_scan_position:
  key: wallet_id + height/root/chunk + asset_id
  value: object_id

tx_by_status:
  key: wallet_id + status + tx_id
  value: object_id

tx_by_time:
  key: wallet_id + timestamp + tx_id
  value: object_id
```

Rules:

1. Indexes are accelerators, not separate truth.
2. Object payload is the authority.
3. Every object write must update indexes in the same RedB write transaction.
4. Open-time validation should detect missing/stale index rows.
5. A repair/rebuild command can rebuild indexes from objects, but normal open should fail on integrity drift unless an explicit repair mode is requested.
6. Index keys MUST use the existing `crate::db::index_codecs` canonical key format.
7. Index values MUST remain pointer-like and fit `MAX_INDEX_VALUE_BYTES`; store larger data in object payloads.

Current `IndexTable` variants already include:

```text
AssetOutByDef = 4
AssetOutBySpentFlag = 5
TrackedAssetBySpentFlag = 6
TxByStatus = 7
TxByTime = 8
PendingByStatusExpiry = 9
ReceiptByTxHash = 10
WalletByWalletId = 11
```

Implementation rule:

1. Reuse existing index table variants only when their names and semantics exactly match the new `OwnedAssetPayload` query.
2. If an index stores wallet-owned `OwnedAssetPayload` records, prefer adding explicit variants instead of overloading `AssetOutByDef`.
3. Recommended new variants:
   - `OwnedAssetById = 12`
   - `OwnedAssetByDefStatus = 13`
   - `OwnedAssetByStatus = 14`
   - `OwnedAssetByTx = 15`
   - `OwnedAssetByScan = 16`
4. If these variants are added, also add matching RedB table constants, schema YAML entries, debug dump entries, and validation branches.
5. Add a non-test production constructor or builder for `IndexUpdate` / `IndexValueBytes`; the current public constructor path is test-only, so production asset-store code must not rely on `#[cfg(test)]` helpers.

## 🧱 Object Kinds And Payload Versions

The numeric ids below are the target ids for implementation. They fit the current `#[repr(u8)]` object-kind encoding and avoid the deprecated Snapshot id.

Current ids that must remain stable unless a deliberate breaking schema reset is performed:

```text
WalletRoot = 1
Account = 2
DerivationState = 7
ScanState = 8
App = 15
Chain = 16
Keys = 17
StealthMeta = 18
TofuPins = 19
Snapshot = 250
```

New ids:

```text
WalletProfile = 20
OwnedAsset = 21
WalletTx = 22
WalletTxEvent = 23
BackupManifest = 24
```

New payload version constants:

```rust
pub const PAYLOAD_VERSION_WALLET_PROFILE: u16 = 1;
pub const PAYLOAD_VERSION_OWNED_ASSET: u16 = 1;
pub const PAYLOAD_VERSION_WALLET_TX: u16 = 1;
pub const PAYLOAD_VERSION_WALLET_TX_EVENT: u16 = 1;
pub const PAYLOAD_VERSION_BACKUP_MANIFEST: u16 = 1;
```

Files that must be updated together:

1. `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs`
   - Add the new payload version constants.
   - Add the new `ObjectKindId` variants.
   - Add payload structs or re-export them from a new payload module.
2. `crates/z00z_wallets/src/db/redb_wallet_store/mod.rs`
   - Extend `is_supported_payload_version(...)`.
3. `crates/z00z_wallets/src/db/redb/schema/redb-schema.yaml`
   - Add all new object kinds.
   - Fix existing schema drift: Rust has `StealthMeta = 18`, `TofuPins = 19`, and `Snapshot = 250`, while the current YAML omits `18`/`19` and lists Snapshot as `254`.
4. `crates/z00z_wallets/src/db/redb_wallet_store/debug/debug_types.rs`
   - Add debug decode support for new payloads behind the existing `wallet_debug_dump` feature path.

The implementation MUST NOT add an object kind id above `255` unless the object-kind encoding changes from `u8`.

### 🔧 Required Low-Level Store Helpers

The refactor needs a small production API over the existing object writer.

Add helpers equivalent to:

```rust
pub fn write_object_by_id<R: SecureRngProvider>(
    session: &WalletSession,
    object_id: u128,
    kind_id: u8,
    payload_version: u16,
    payload_bytes: Vec<u8>,
    index_updates: &[IndexUpdate],
    rng_provider: R,
) -> WalletResult<u64>;

pub fn read_objects_by_index(
    session: &WalletSession,
    table: IndexTable,
    semantic_prefix: &[u8],
    limit: usize,
    cursor: Option<Vec<u8>>,
) -> WalletResult<IndexedObjectPage>;
```

Rules:

1. `write_object_by_id(...)` MUST validate `is_supported_payload_version(kind_id, payload_version)`.
2. It MUST encrypt with `encrypt_object_record(...)` using the existing object id.
3. It MUST call the same index manifest replacement path used by `write_object_with_indexes(...)`.
4. It MUST commit the object write, index updates, save-sequence bump, and integrity update in one RedB write transaction.
5. It MUST NOT be `#[cfg(test)]`.
6. It MUST be used for asset status transitions and singleton profile updates.
7. It MAY be implemented in `objects/mod.rs` by making the existing internal helper reusable instead of duplicating logic.

Do not implement asset status updates by calling `write_object(...)`, because that function allocates a new object id.

## 🧩 Proposed Interfaces

The caller should not care whether assets are stored as encrypted objects, rows, or future event-sourced read models.

Target port. This is a new internal wallet-store interface, not an existing symbol:

```text
WalletAssetStore {
  put_owned_asset(wallet_id, asset, source, context) -> PutAssetResult
  get_owned_asset(wallet_id, asset_id) -> Option<OwnedAssetPayload>
  list_owned_assets(wallet_id, filter, cursor) -> Page<OwnedAssetPayload>
  list_spendable_assets(wallet_id, asset_def_id, amount_hint) -> Vec<OwnedAssetPayload>
  reserve_asset_inputs(wallet_id, tx_id, asset_ids) -> ReservationResult
  release_asset_reservation(wallet_id, tx_id) -> ReleaseResult
  confirm_asset_spend(wallet_id, tx_id, spent_ids, new_outputs) -> ConfirmResult
  replace_assets_for_restore(wallet_id, assets) -> RestoreResult
}
```

Concrete Rust target shape:

```rust
pub(crate) trait WalletAssetStore {
    fn put_owned_asset(
        &self,
        session: &WalletSession,
        asset: Asset,
        source: OwnedAssetSource,
        context: AssetPersistContext,
    ) -> WalletResult<PutAssetOutcome>;

    fn get_owned_asset(
        &self,
        session: &WalletSession,
        asset_id: &[u8; 32],
    ) -> WalletResult<Option<OwnedAssetPayload>>;

    fn list_owned_assets(
        &self,
        session: &WalletSession,
        filter: AssetFilter,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<AssetPage>;

    fn reserve_asset_inputs(
        &self,
        session: &WalletSession,
        tx_id: &PersistTxId,
        asset_ids: &[[u8; 32]],
    ) -> WalletResult<()>;

    fn release_asset_reservation(
        &self,
        session: &WalletSession,
        tx_id: &PersistTxId,
    ) -> WalletResult<()>;

    fn confirm_asset_spend(
        &self,
        session: &WalletSession,
        tx_id: &PersistTxId,
        spent_ids: &[[u8; 32]],
        new_outputs: &[Asset],
    ) -> WalletResult<()>;
}
```

Implementation notes:

1. Prefer `pub(crate)` until the interface stabilizes.
2. Keep this port inside the wallet persistence boundary, not in RPC adapters.
3. RPC adapters should call `WalletService`, and `WalletService` should call this store.
4. The first implementation can be synchronous internally because RedB wallet helpers are synchronous behind session access. Do not expose async at the low-level `.wlt` store unless the existing wallet session boundary requires it.

Suggested filter:

```text
AssetFilter {
  account_id
  asset_definition_id
  status
  source
  tx_id
  min_height
  max_height
}
```

Additional new target types used by the port:

```text
AssetPersistContext:
  scan_ref: Option<ScanRef>
  receive_ref: Option<ReceiveRef>
  confirmation_ref: Option<ConfirmRef>
  now_ms: u64

PutAssetOutcome:
  Inserted { object_id: u128 }
  AlreadyPresent { object_id: u128 }

AssetPage:
  items: Vec<OwnedAssetPayload>
  next_cursor: Option<String>
  has_more: bool

IndexedObjectPage:
  object_ids: Vec<u128>
  next_cursor: Option<Vec<u8>>
  has_more: bool
```

Misuse to prevent:

1. Caller must not directly mutate asset status without tx context.
2. Caller must not insert duplicate asset ids.
3. Caller must not mark an asset spent unless it is PendingSpend for the same tx or an explicit recovery mode is active.
4. Caller must not expose secret material through asset records.
5. Caller must not rebuild balances from untrusted external JMT ownership claims.

## 🧨 What Should Change In The Existing Phase 046 Spec

The current spec is useful as a proof plan, but it should separate "current behavior to demonstrate" from "target architecture to preserve."

Recommended edits:

1. Keep Decision 1.
   - Add the simulator stage. It is still valuable.

2. Revise Decision 2.
   - Current wording: use `.wlt` claimed assets as input authority and introduce no new simulator asset table.
   - Better wording: Phase 046 must prove the current `.wlt` claimed-assets path, but the target wallet storage is first-class `OwnedAsset` objects inside `.wlt`; no simulator-only asset table may be introduced.

3. Keep Decision 3.
   - `wallet.asset.*` send-like methods should remain non-canonical until tx/reconcile-backed.

4. Keep Decision 4.
   - `WalletPlusHistory` restore remains required while tx history is separate JSONL.
   - Add future target: move tx records into `.wlt` or explicitly keep JSONL as an audited sidecar.

5. Keep Decision 5.
   - Key rotation boundary remains separate from storage redesign.

6. Keep Decision 6.
   - Stale placeholder cleanup is still needed.

7. Keep Decision 7.
   - No custom root math.

8. Keep Decision 8.
   - Ownership detection remains wallet-side.
   - Remote JMT/aggregator is only a chunk/proof read adapter.

New decision to add:

```text
Decision 9: Treat Snapshot as an export/profile compatibility container, not as the
target live asset authority. Wallet-owned assets must become first-class encrypted
OwnedAsset objects in `.wlt`, indexed for spend selection and updated atomically with
tx status and scan cursor changes.
```

## 🛠️ Implementation Plan And Migration Sequence

Because the project is still in development and no legacy wallet compatibility is required, prefer a clean cut instead of long compatibility scaffolding.

### Phase 1: Schema And Payload Groundwork

Goal: add the new object vocabulary without changing runtime behavior yet.

Tasks:

1. Update `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs`.
   - Add `PAYLOAD_VERSION_WALLET_PROFILE`, `PAYLOAD_VERSION_OWNED_ASSET`, `PAYLOAD_VERSION_WALLET_TX`, `PAYLOAD_VERSION_WALLET_TX_EVENT`, and `PAYLOAD_VERSION_BACKUP_MANIFEST`.
   - Add `ObjectKindId::WalletProfile = 20`, `OwnedAsset = 21`, `WalletTx = 22`, `WalletTxEvent = 23`, and `BackupManifest = 24`.
   - Add payload structs or move payload structs into a dedicated module re-exported from the wallet store.

2. Update `crates/z00z_wallets/src/db/redb_wallet_store/mod.rs`.
   - Extend `is_supported_payload_version(...)` for every new object kind.
   - Add tests that reject unsupported payload versions for each new kind.

3. Update `crates/z00z_wallets/src/db/redb/schema/redb-schema.yaml`.
   - Add missing current kinds `18` and `19`.
   - Fix Snapshot from `254` to `250`.
   - Add kinds `20` through `24`.
   - Add index table entries for any new owned-asset indexes.

4. Update debug dump support.
   - Extend `crates/z00z_wallets/src/db/redb_wallet_store/debug/debug_types.rs`.
   - Extend `crates/z00z_wallets/src/db/redb_wallet_store/debug/debug_export.rs` if new index tables are added.

Completion criteria:

1. Schema YAML and Rust object kind ids match exactly.
2. `is_supported_payload_version(...)` accepts every new kind/version pair and rejects version drift.
3. No receive, tx, or backup behavior has changed yet.

### Phase 2: Low-Level Object Upsert And Index API

Goal: make status updates safe before adding `OwnedAssetPayload`.

Tasks:

1. Add a production `write_object_by_id(...)` helper in the RedB wallet store object module.
   - It must accept an existing `object_id`.
   - It must reuse `encrypt_object_record(...)`.
   - It must reuse the index replacement logic behind `write_object_with_indexes(...)`.
   - It must commit object write, index updates, save-sequence bump, and integrity update together.

2. Add production constructors/builders for index updates.
   - The current `IndexUpdate::new(...)` and `IndexValueBytes::new(...)` path is test-only.
   - Production owned-asset code needs a non-test way to construct validated index updates.

3. Add index query helpers.
   - Query by exact semantic key where needed.
   - Query by canonical prefix only if the existing index key format safely supports it.
   - Return object ids plus cursor metadata, then load object payloads by id.

Completion criteria:

1. Updating an object by id replaces stale index rows through `index_manifest`.
2. Tests prove that updating one object from status A to status B removes old status indexes.
3. No production code depends on test-only index constructors.

### Phase 3: Wallet Profile Replacement For Snapshot Metadata

Goal: move non-asset Snapshot fields into `WalletProfilePayload`.

Tasks:

1. Add `WalletProfilePayload`.
   - Include all non-asset fields from `WalletPersistenceState`.
   - Do not include `claimed_assets`.

2. Add profile read/write helpers.
   - Add `write_wallet_profile(...)`.
   - Add `read_wallet_profile(...)`.
   - Use `META_WALLET_PROFILE_OBJECT_ID` or a wallet-id index; prefer a meta pointer for the singleton profile object.

3. Update wallet creation.
   - `create_wallet_using_explicit_identity(...)` currently builds `WalletPersistenceState` and writes it via `persist_wallet_snapshot(...)`.
   - Replace that write with profile object creation and an empty owned-asset set.
   - Prefer making wallet creation all-or-nothing by extending the low-level create path to write profile in the same creation transaction. If that is too large for the first patch, use a staged rollback path and document the temporary failure boundary.

4. Update wallet open.
   - Open should load `WalletProfilePayload` instead of Snapshot for name, verifier, settings, seed salt, receiver counters, and state.
   - If no legacy compatibility is required, missing Snapshot must not be fatal after cutover.

Completion criteria:

1. New wallets open without a Snapshot object.
2. Name, settings, verifier, seed salt, receiver counters, and locked state survive close/reopen.
3. `WalletPersistenceState.claimed_assets` is no longer written on normal save.

### Phase 4: Owned Asset Store

Goal: make `OwnedAssetPayload` the live asset authority.

Tasks:

1. Add `OwnedAssetPayload`, `OwnedAssetStatus`, `OwnedAssetSource`, and supporting reference structs.
   - Use `z00z_core::assets::AssetWire` for the owned asset payload.
   - Store `asset_id` and `asset_definition_id` redundantly only as validated indexed fields.

2. Add `WalletAssetStore` over `.wlt`.
   - Implement `put_owned_asset(...)`.
   - Implement `get_owned_asset(...)`.
   - Implement `list_owned_assets(...)`.
   - Implement `list_spendable_assets(...)`.
   - Implement `reserve_asset_inputs(...)`.
   - Implement `release_asset_reservation(...)`.
   - Implement `confirm_asset_spend(...)`.

3. Replace the live authority in `WalletService`.
   - `wallet_claimed_assets` may remain as a cache only.
   - Rename or wrap `list_claimed_assets(...)` so existing callers compile while reading from the object store.
   - Do not let callers mutate the cache directly.

4. Replace `put_claimed_asset(...)`.
   - Keep the public behavior: return `Ok(true)` for new insert and `Ok(false)` for duplicate.
   - Persist to `OwnedAssetPayload`.
   - Roll back cache changes on persistence failure if a cache remains.

5. Replace `set_claimed_assets(...)`.
   - Remove it from normal tx reconcile.
   - Keep a restricted restore/test helper if needed.
   - Any replacement helper must validate duplicate ids before mutating state.

Completion criteria:

1. A scanned/imported asset survives close/reopen without Snapshot.
2. Duplicate asset ids fail closed or return idempotent no-op only for matching payloads.
3. `wallet.asset.list_assets` reads from `OwnedAssetPayload`.
4. `AssetStorageImpl` remains non-authoritative or is rewired to delegate to `.wlt`; it must not become the durable wallet asset store as-is.

### Phase 5: Receive And Scan Integration

Goal: persist scan hits as owned asset objects.

Tasks:

1. Update `WalletService::recv_range(...)` internals.
   - Keep `StealthOutputScanner` as the ownership detector.
   - Keep request registration behavior.
   - Replace `recv_route(..., ReceiveNext::PersistClaim)` internals with owned-asset persistence.

2. Make scan hit insert and scan cursor update robust.
   - Preferred: one RedB transaction updates `OwnedAssetPayload` objects, indexes, and `ScanStatePayload`.
   - Acceptable temporary path: asset insert then scan cursor save, but replay must be idempotent and tests must cover failure between the two writes.

3. Keep `wallet.asset.receive_asset` compatibility-only.
   - It may report ownership.
   - It must not become a second persistence authority unless explicitly routed through the same owned-asset store.

Completion criteria:

1. `recv_range(...)` persists found assets through `OwnedAssetPayload`.
2. Restart/resume tests prove scan cursor and asset set do not drift.
3. No code or docs claim JMT-side ownership detection.

### Phase 6: Transaction Build, Reservation, Cancel, And Reconcile

Goal: make `wallet.tx.*` operate on owned asset object state.

Tasks:

1. Update `build_transaction_impl(...)`.
   - Replace `list_claimed_assets(...)` plus JSONL reservation filtering with `list_spendable_assets(...)`.
   - Keep explicit `asset_id` filtering behavior.
   - Keep default behavior that chooses the first available asset definition when `asset_id` is absent.
   - After package build succeeds, call `reserve_asset_inputs(...)` for selected input asset ids.

2. Update pending transaction handling.
   - Keep JSONL tx history if full `.wlt` tx migration is deferred.
   - Ensure pending tx history and asset reservations cannot disagree silently.
   - On tx store write failure, release asset reservations.
   - On asset reservation failure, do not write pending tx history.

3. Update `cancel_transaction`.
   - Mark tx cancelled in the current tx store.
   - Release only `PendingSpend` assets for that tx id.
   - Return existing RPC response semantics.

4. Update `reconcile_transaction_impl(...)`.
   - Replace full-vector `set_claimed_assets(...)`.
   - Mark input objects as `Spent`.
   - Insert wallet-owned output/change assets as `Spendable`.
   - If tx evidence/history write fails after asset mutation, rollback asset status changes or use one atomic `.wlt` tx if tx records have moved into `.wlt`.

Completion criteria:

1. Build excludes pending assets using owned-asset status.
2. Cancel releases inputs.
3. Reconcile removes spendability from inputs and adds change outputs.
4. Tamper/fail-closed tests prove failed import/reconcile does not mutate asset objects.

### Phase 7: Backup, Restore, And Export

Goal: restore full wallet state without Snapshot-owned assets.

Tasks:

1. Add backup manifest payload.
   - Include schema version, wallet id, chain/network identity, object counts, and tx-history plane marker.

2. Update export pack.
   - Export `WalletProfilePayload`.
   - Export all `OwnedAssetPayload` objects.
   - Export `ScanStatePayload`, `TofuPinsPayload`, key refs, and encrypted secret material.
   - Continue exporting JSONL tx history while it remains the tx-history plane.

3. Update restore.
   - Validate profile.
   - Validate asset ids and duplicate ids.
   - Validate tx references to asset ids when tx records are included.
   - Stage `.wlt` and JSONL writes before promoting.
   - Reject wrong password or corrupt backup without mutating existing wallet state.

4. Remove Snapshot dependency.
   - If no legacy is required, delete Snapshot restore code after tests are updated.
   - If a short compatibility bridge is needed, keep it read-only and convert Snapshot `claimed_assets` into `OwnedAssetPayload` once, then write the new object store.

Completion criteria:

1. `ForensicImportMode::WalletPlusHistory` restores profile, owned assets, scan state, and tx history.
2. Wrong password and corrupt history fail without partial restore.
3. Restored wallet can immediately build a transaction from restored `OwnedAssetPayload` records.

### Phase 8: Simulator And Documentation Cutover

Goal: make Scenario 1 describe the new storage truth.

Tasks:

1. Update Stage 13 storage/report notes.
   - Replace "claimed-asset persistence remains in encrypted .wlt snapshot payloads" with language naming `OwnedAssetPayload` objects.
   - Keep root vocabulary explicit: `prev_root`, `state_root`, and `flat_root` remain storage concepts, not wallet asset-store concepts.

2. Update Stage 13 assertions.
   - Prove live `wallet.tx.*` lifecycle.
   - Prove lock/reopen asset survival through `.wlt` owned asset objects.
   - Prove backup/restore through `WalletPlusHistory`.
   - Prove tamper failure does not mutate owned asset objects.
   - A report flag or note is not sufficient for reopen/restore proof; Stage 13 must actually execute the reopen and restore steps if it claims those proofs.

3. Update Phase 046 spec language.
   - Decision 2 must not imply Snapshot is the target asset authority.
   - Add Decision 9 from this document.

Completion criteria:

1. Simulator reports no longer describe Snapshot as claimed-asset authority.
2. Existing Stage 13 root-binding checks still reuse existing storage helpers.
3. Docs distinguish wallet asset objects from chain/JMT storage leaves.

### 🧭 Mandatory Runtime And Simulator Touchpoints

The implementation is not complete if it only changes the RedB schema. The same patch series must explicitly cover:

1. Wallet store/session/create/open/restore paths:
   - `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock.rs`
   - `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock_open.rs`
   - `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_runtime.rs`
   - `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
   - `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
   - `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
   - `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs`
   - `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
   - `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs`

2. Asset / tx runtime adapters that still depend on claimed-asset or snapshot-backed assumptions:
   - `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`
   - `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
   - `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_ops.rs`
   - `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_registry.rs`
   - `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs`
   - any `load_assets_from_storage(...)`, `list_claimed_assets(...)`, `put_claimed_asset(...)`, or `set_claimed_assets(...)` call sites that would otherwise keep Snapshot as live authority

3. Simulator Stage 13 contract/config/reporting:
   - `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/flow.rs`
   - `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/report.rs`
   - `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/storage.rs`
   - `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs`
   - `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
   - `crates/z00z_simulator/src/scenario_1/runner_contract_table.in`
   - `crates/z00z_simulator/src/scenario_1/runner_verify.rs`

4. The simulator text helper currently returning `claimed-asset persistence remains in encrypted .wlt snapshot payloads` MUST be rewritten as part of the cutover; leaving that string behind would make the simulator contradict the implementation.

## 🧪 Validation Plan

Required unit tests:

1. Insert one `OwnedAsset`, reopen wallet, list it.
2. Duplicate asset id is rejected or idempotently accepted only if payload matches.
3. Updating an existing `OwnedAssetPayload` status preserves the same `object_id` and replaces stale index rows.
4. Asset indexes return the same records as full object scan.
5. Corrupt object payload fails closed.
6. Missing index row is detected on open or index validation.
7. Scan cursor and asset insert commit atomically.
8. If the first patch uses a temporary two-step asset-write / cursor-save fallback, a fault-injection test proves idempotent replay after a failure between the two writes.
9. `PendingSpend` assets are excluded from input selection.
10. Cancel releases pending assets.
11. Reconcile marks inputs spent and inserts change outputs.
12. Restore rejects duplicate asset ids.
13. Restore wrong password leaves existing wallet unchanged.
14. Snapshot object kind id in Rust matches schema documentation.
15. Expanded `wallet_config.yaml` parses successfully and invalid coupled fields fail closed.
16. Wallet create/open/settings defaults read the same YAML-backed values for auto-lock timeout, default fee, and currency display.
17. Backup defaults derive from `wallet.backup.*` YAML values, including `location`.
18. Seed recovery uses `wallet.recovery.gap_limit` from YAML instead of a hardcoded value.
19. If temporary Snapshot compatibility remains, the import path converts legacy snapshot-owned assets into `OwnedAssetPayload` objects without re-entering the normal live write path.

Required wallet RPC tests:

1. `wallet.tx.build_transaction` selects only `Spendable` assets.
2. `wallet.tx.list_pending_transactions` reflects reserved inputs.
3. `wallet.tx.cancel_transaction` releases inputs.
4. `wallet.tx.reconcile_transaction` updates asset states and tx status.
5. `wallet.tx.import_transaction` creates receiver-owned assets through wallet-side scan logic.
6. `wallet.asset.list_assets` reads from `OwnedAsset` objects, not Snapshot.
7. `wallet.asset.import_asset` persists through `OwnedAsset` objects.
8. `wallet.asset.get_asset_balance` computes total / pending / available from `OwnedAsset` state plus pending reservation bindings.
9. `wallet.asset.get_asset_details` reads from `OwnedAsset` objects.
10. Wallet create/recover/backup RPC flows honor YAML-backed defaults on live paths, not hardcoded literals.

Required simulator checks:

1. Stage 13 proves live `wallet.tx.*` lifecycle.
2. Stage 13 proves assets survive lock/reopen.
3. Stage 13 proves backup/restore with wallet state and tx history.
4. Stage 13 proves tamper failure does not mutate asset objects.
5. Stage 13 report distinguishes:
   - wallet-owned asset object state
   - tx history state
   - scan cursor state
   - storage/JMT roots
6. Stage 13 contract/config/report text no longer calls encrypted snapshot payloads the live claimed-asset authority.
7. If Stage 13 claims lock/reopen or `WalletPlusHistory` proof, it must actually execute those operations rather than only recording a config flag or report note.

### 🔄 Existing Test Migration Is Mandatory

New tests alone are not sufficient. The same patch must upgrade existing suites that currently encode Snapshot / `claimed_assets` as the live authority.

Minimum suites to update:

1. `crates/z00z_wallets/src/services/wallet_service_tests.rs`
2. `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_tests.rs`
3. `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
4. `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`
5. `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/tests.rs`
6. `crates/z00z_wallets/src/db/redb_wallet_store/tests.rs`
7. `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs`
8. `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`
9. `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
10. Scenario 1 contract/config/report fixtures under `crates/z00z_simulator/src/scenario_1/`

Rules:

1. Rewrite stale assertions that require Snapshot or `claimed_assets` to be the live post-cutover authority.
2. Keep compatibility tests only where they explicitly cover one-shot import/export migration from old Snapshot data.
3. Do not leave dual assertions where old Snapshot authority and new `OwnedAsset` authority both appear canonical in the same green suite.

## ✅ Acceptance Criteria

1. **AC-001**: Given a newly created wallet, when it is closed and reopened, then wallet profile fields load from `WalletProfilePayload` and no Snapshot object is required for normal open.
2. **AC-002**: Given `recv_range(...)` detects a wallet-owned asset, when persistence succeeds, then exactly one `OwnedAssetPayload` exists for that `asset_id` and the scan cursor is advanced or replay remains idempotent.
3. **AC-003**: Given the same scan hit is replayed, when the stored payload matches, then the operation returns idempotently without creating a second asset object.
4. **AC-004**: Given the same `asset_id` appears with conflicting payload data, when insert is attempted, then the wallet fails closed and does not change spendable balance.
5. **AC-005**: Given Alice has spendable assets, when `wallet.tx.build_transaction` succeeds, then selected input assets become `PendingSpend` for that tx id.
6. **AC-006**: Given a tx is pending, when Alice builds another transaction, then `PendingSpend` assets are not selectable.
7. **AC-007**: Given a pending tx is cancelled, when cancellation succeeds, then only assets reserved by that tx id return to `Spendable`.
8. **AC-008**: Given a tx is reconciled with valid evidence, when reconcile succeeds, then spent inputs become `Spent` and wallet-owned outputs/change become `Spendable`.
9. **AC-009**: Given package/evidence tampering, when import or reconcile is attempted, then no owned asset object changes status and no new owned asset object is inserted.
10. **AC-010**: Given a backup is restored with `WalletPlusHistory`, when restore succeeds, then profile, owned assets, scan state, and tx history match the source wallet.
11. **AC-011**: Given wrong password or corrupt history during restore, when restore fails, then no existing wallet artifact is mutated.
12. **AC-012**: Given simulator Stage 13 runs, when it writes its report, then the report identifies `OwnedAssetPayload` object persistence and does not call Snapshot the live asset authority.
13. **AC-013**: Given wallet defaults are configured in `wallet_config.yaml`, when wallet create/open/recover/backup runtime paths execute, then they use the YAML-backed values instead of hardcoded Rust defaults.

## 🧪 Verification Gates

Every implementation plan generated from this specification must include this verify section.

Required command order:

1. Run the fail-fast gate first:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```

2. If the fail-fast gate fails, stop broader validation, fix the failure, and rerun the gate.

3. Then run focused and broad Rust validation relevant to the changed code. For this refactor, the broad command is:

```bash
cargo test --release --features test-fast --features wallet_debug_dump
```

4. Run `/.github/prompts/gsd-review-tasks-execution.prompt.md` through `/GSD-Review-Tasks-Execution` at least 3 times in YOLO mode.

5. Continue fixing all issues and warnings from the review prompt until at least 2 consecutive runs show no significant code issues.

6. Do not mark the refactor complete until Stage 13 simulator checks and wallet backup/restore tests pass under the new asset-store authority.

## 🚫 What Not To Do

1. Do not add a simulator-only asset table.
2. Do not make remote JMT ownership detection authoritative.
3. Do not store wallet secrets inside asset records.
4. Do not keep growing `WalletPersistenceState` into a hidden database.
5. Do not infer pending reservations only from JSONL once asset records exist.
6. Do not recompute storage roots with custom simulator math.
7. Do not describe `wallet.asset.send_asset` as canonical confirmed spend until it is backed by the same tx/reconcile path.
8. Do not keep the name Snapshot for live mutable wallet state if legacy compatibility is not needed.

## 📌 Final Specification Position

Rewriting a full wallet Snapshot every time an asset is claimed, spent, or reconciled is not the target wallet design.

The current code works because the wallet has an in-memory claimed-asset cache and persists that cache into an encrypted Snapshot object. That is a reasonable bridge, but it should not become the permanent storage model.

The best target is:

```text
WalletProfile object:
  slow-changing wallet metadata

OwnedAsset objects:
  one encrypted object per wallet-owned asset

ScanState object:
  scan cursor and resume metadata

WalletTx / WalletTxEvent objects:
  tx lifecycle state, either now or as the next migration after assets

Secrets table:
  secret material only

Indexes:
  fast spendable/pending/spent lookup and tx binding
```

This design keeps the current wallet functionality, makes transaction input selection natural, gives simulator proofs a real wallet storage boundary, and leaves room for future features without turning Snapshot into an accidental database.
