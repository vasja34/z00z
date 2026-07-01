# Phase 046 Wallet Addons Implementation Specification

**Status:** Implementation-ready specification  
**Date:** 2026-05-13  
**Source notes:** `.planning/phases/046-wallet-addons/046-wallet-misses.md`, `.planning/phases/046-wallet-addons/046-storage-explain.md`  
**Primary crates:** `z00z_wallets`, `z00z_simulator`  
**Grounding rule:** This specification only uses live repository symbols and paths verified against the current codebase.

## 🎯 Objective

Phase 046 closes the wallet demonstration gaps by proving the real `wallet.tx.*` lifecycle end to end, proving where claimed assets persist, and making simulator evidence match the production wallet boundaries instead of relying only on simulator-specific transaction helpers.

The phase must answer these challenges:

- How are wallet-owned assets found during scan persisted so later transactions can select inputs?
- How can the simulator demonstrate wallet spending if the persisted asset set lives inside `.wlt` snapshots?
- How does a verified `TxPackage` map into canonical storage leaves, `prev_root`, `state_root`, and `flat_root` without inventing a second root engine?
- Which RPC path is canonical for real spends?
- Which `wallet.asset.*` methods are compatibility or UX surfaces rather than confirmed ledger mutation authority?
- Where is the live receive/scan authority, where is scan cursor persistence, and what must not be claimed about `scan_engine` or a JMT/JMT-side scanner?
- How should backup/restore, TOFU/payment requests, session hardening, tx tamper cases, and multi-status history be proven?
- What exactly does `wallet.key.rotate_master_key` prove today, and what must not be claimed until persisted seed rotation exists?
- Which stale placeholder/stub comments must be corrected so simulator output and wallet docs do not overstate or understate live behavior?

## ✅ Verified Current State

### 🔑 Claimed Asset Persistence

Wallet-owned assets are not stored in a separate RedB asset table. The live wallet ownership store is:

1. In memory: `WalletService::wallet_claimed_assets` in `crates/z00z_wallets/src/services/wallet_service_types_core.rs`.
2. Snapshot payload: `WalletPersistenceState.claimed_assets: Vec<AssetWire>` in `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`.
3. Encrypted `.wlt` object: `write_wallet_snapshot(...)` writes the snapshot object in `crates/z00z_wallets/src/db/redb_wallet_store/backup.rs`.

The live mutation boundaries are:

```rust
// crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs
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

pub async fn recv_route(
    &self,
    wallet_id: &PersistWalletId,
    asset: Asset,
    next: ReceiveNext,
) -> WalletResult<bool>;
```

The snapshot bridge is:

```rust
// crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs
pub(crate) async fn create_snapshot(
    &self,
    wallet_id: &PersistWalletId,
) -> WalletResult<WalletPersistenceState>;

// crates/z00z_wallets/src/services/wallet/store/wallet_service_store_support.rs
pub(crate) async fn snapshot_claimed_assets(
    &self,
    wallet_id: &PersistWalletId,
) -> Vec<AssetWire>;

// crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs
async fn restore_snapshot(
    &self,
    snapshot: WalletPersistenceState,
) -> WalletResult<PersistWalletId>;
```

Conclusion: a scan only detects a leaf until code calls `recv_route(..., ReceiveNext::PersistClaim)`. After that, `put_claimed_asset(...)` updates `wallet_claimed_assets` and persists the new `.wlt` snapshot through `persist_snapshot_for_open_session(...)`. There is no separate asset RedB table and no separate `wallet_*.bin.enc` snapshot file in the canonical wallet state path; the durable claimed-asset plane is the encrypted snapshot object inside `.wlt`.

### 🔑 Transaction History Persistence

Transaction history is not embedded in `.wlt`. It is a canonical JSONL sidecar:

```rust
// crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs
pub(crate) fn wallet_history_jsonl_name(wallet_stem: &str) -> String;

pub(crate) fn wallet_history_jsonl_path(
    &self,
    wallet_id: &PersistWalletId,
) -> std::path::PathBuf;
```

The path contract is enforced by `TxStorageImpl`:

```rust
// crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs
// canonical tx-history path must be wallet_<stem>_tx_history.jsonl
pub struct TxStorageImpl<T: TimeProvider> { /* file-backed JSONL store */ }
```

Live storage events are exposed through `TxStorage`:

```rust
// crates/z00z_wallets/src/persistence/tx/tx_storage.rs
fn put(&mut self, record: TxRecord) -> TxStorageResult<()>;
fn record_imported(&mut self, record: TxRecord) -> TxStorageResult<()>;
fn record_exported(&mut self, tx_hash: &str) -> TxStorageResult<()>;
fn record_submitted(&mut self, tx_hash: &str) -> TxStorageResult<()>;
fn record_admitted(&mut self, tx_hash: &str) -> TxStorageResult<()>;
fn record_confirmed(&mut self, tx_hash: &str, block_height: u64) -> TxStorageResult<()>;
fn record_confirmation_evidence(
    &mut self,
    tx_hash: &str,
    evidence: TxConfirmationEvidence,
) -> TxStorageResult<()>;
fn record_cancelled(&mut self, tx_hash: &str) -> TxStorageResult<()>;
```

Conclusion: backup and restore must verify two persistence planes: `.wlt` for `claimed_assets` and the embedded snapshot container, and `wallet_<stem>_tx_history.jsonl` for tx history.

### 🔑 Tx Package to Storage Root Contract

The live tx-to-storage contract already spans wallet wire conversion, simulator bridge helpers, rollup settlement verification, and storage commit persistence. The canonical wire-to-leaf boundary is:

```rust
// crates/z00z_wallets/src/tx/witness_gate.rs
pub fn asset_wire_to_leaf(wire: &AssetWire) -> Result<AssetLeaf, String>;
```

The current simulator proof bridge already reuses that conversion instead of inventing alternate leaf math:

```rust
// crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs
impl CheckpointPackageProofVerifier {
    pub(crate) fn expected_prev_root(pkg: &TxPackage) -> Result<CheckRoot, String>;
    pub(crate) fn from_stage7(pkg: &TxPackage, outputs: &[TxOutputWire]) -> Result<Self, String>;
    pub(crate) fn verify_pkg_contract(pkg: &TxPackage) -> Result<(), String>;
}
```

Rollup settlement verifies the same package against checkpoint execution rows:

```rust
// crates/z00z_rollup_node/src/lib.rs
fn tx_prev_root(tx_package: &TxPackage) -> Result<CheckRoot, SettlementError>;
fn verify_tx_inclusion(
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
) -> Result<(), SettlementError>;
```

Canonical storage apply remains in `z00z_storage`:

```rust
// crates/z00z_storage/src/assets/model.rs
pub fn put_leaf(&mut self, item: StoreItem) -> Result<AssetStateRoot, ModelErr>;
pub fn root(&self) -> Result<AssetStateRoot, ModelErr>;
pub fn del_leaf(&mut self, path: &AssetPath) -> Result<AssetStateRoot, ModelErr>;

// crates/z00z_storage/src/assets/store_internal/tx_plan_engine.rs
fn commit_stage(
    &mut self,
    plan: ShardPlan,
    version: Version,
    inject_fail: bool,
    claims: &[ClaimNullTx],
    txs: Option<Vec<crate::checkpoint::CheckpointExecTx>>,
) -> Result<AssetStateRoot, AssetStoreError>;

// crates/z00z_storage/src/assets/store_internal/redb_backend.rs
pub(super) fn sync_store(
    &self,
    store: &AssetStore,
    write_arts: WriteArts,
    version: u64,
    flat_root: [u8; 32],
    _prev_root: AssetStateRoot,
    state_root: AssetStateRoot,
) -> Result<(), StoreBackendError>;
```

Conclusion: Phase 046 must not add parallel root or digest math in the simulator or wallet docs. New proof/report helpers may reuse existing simulator bridges, but the authoritative chain stays `TxPackage` -> `asset_wire_to_leaf(...)` -> checkpoint exec rows / `StoreOp` replay -> `commit_stage(...)` -> `sync_store(...)`, with `state_root` and `flat_root` recorded as distinct roots.

### 🔑 Canonical Spend Lifecycle

The canonical live spend lifecycle is `wallet.tx.build_transaction` -> `wallet.tx.broadcast_transaction` -> `wallet.tx.reconcile_transaction`.

The full RPC surface exists in `crates/z00z_wallets/src/adapters/rpc/methods/tx.rs`:

```rust
async fn build_transaction(
    &self,
    session: SessionToken,
    recipient: String,
    amount: u64,
    asset_id: Option<String>,
) -> RpcResult<RuntimeBuildTxResponse>;

async fn broadcast_transaction(
    &self,
    session: SessionToken,
    tx_data: String,
) -> RpcResult<RuntimeBroadcastTxResponse>;

async fn verify_transaction_package(
    &self,
    session: SessionToken,
    tx_data: String,
) -> RpcResult<RuntimeVerifyTxPkgResponse>;

async fn cancel_transaction(
    &self,
    session: SessionToken,
    tx_id: PersistTxId,
) -> RpcResult<RuntimeCancelTxResponse>;

async fn get_transaction_details(
    &self,
    session: SessionToken,
    tx_id: PersistTxId,
) -> RpcResult<RuntimeTxDetailsResponse>;

async fn export_transaction(
    &self,
    session: SessionToken,
    tx_id: PersistTxId,
) -> RpcResult<RuntimeExportTxResponse>;

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

async fn get_transaction_history(
    &self,
    session: SessionToken,
    pagination: RuntimePaginationParams,
    filter: Option<RuntimeTxHistoryFilter>,
    sort: Option<RuntimeTxHistorySort>,
) -> RpcResult<RuntimePaginatedResponse<PersistTxInfo>>;

async fn list_pending_transactions(
    &self,
    session: SessionToken,
    pagination: RuntimePaginationParams,
) -> RpcResult<RuntimePaginatedResponse<PersistTxInfo>>;
```

`wallet.tx.send_transaction` exists, but it is a convenience wrapper that validates timestamp/idempotency/policy/rate limit and internally calls build+broadcast. It must not be the only simulator proof of the lifecycle.

### 🔑 Simulator Gap

Scenario 1 currently proves transaction package, proof, checkpoint, and JMT mechanics, but its transaction lane is simulator-specific:

- Stage 5 maps to `stage_5::run_tx_plan`.
- Stage 6 maps to `stage_6::run_tx_prepare`.
- Shared implementation lives under `crates/z00z_simulator/src/scenario_1/stage_4_utils/`.
- It uses `wallet.asset.list_assets`, manually prepares the `TxPackage`, manually persists sender/recipient state, and directly appends history through `TxStorageImpl`.

That is valid for existing checkpoint proof mechanics, but it does not demonstrate the live `wallet.tx.*` RPC path.

### 🔑 Scanner Authority and Scan-State Boundary

The live receive authority is wallet-side and request-aware:

```rust
// crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs
pub async fn recv_range(
    &self,
    wallet_id: &PersistWalletId,
    chunks: &[ScanChunk],
    requests: &[PaymentRequest],
    max_ckpt: Option<usize>,
) -> WalletResult<ScanRangeOut>;
```

`recv_range(...)` derives live receiver keys, builds `StealthOutputScanner`, claims hits only through `recv_route(..., ReceiveNext::PersistClaim)`, and persists `ScanStatePayload` through the wallet DB `read_scan_state` / `upsert_scan_state` path.

The live detector and persistence surfaces are:

```rust
// crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs
pub struct StealthOutputScanner { /* wallet-side detector state */ }

// crates/z00z_wallets/src/persistence/scans/scan_storage_impl.rs
pub struct ScanStorageImpl<T: TimeProvider> { /* state_path + time_provider */ }
```

The deferred seam is explicit:

```rust
// crates/z00z_wallets/src/chain/scan_engine_impl.rs
const PHASE_037_SCAN_ENGINE_DEFERRED: &str =
    "not implemented for the Phase 037 scan-engine seam";
```

Conclusion: Phase 046 must keep receive authority in `WalletService::recv_range(...)` and `StealthOutputScanner`. Wallet DB `scan_state` reads/writes persist the live resume cursor, while `ScanStorageImpl` is a separate local scan-state persistence surface; neither is a second scanner. `ScanEngineImpl` remains a deferred seam and the current wallet/storage receive path does not expose a live JMT-based scanner contract.

### 🔑 Asset RPC UX Boundary

`wallet.asset.list_assets` and `wallet.asset.import_asset` are useful wallet asset surfaces. `wallet.asset.send_asset`, `split_asset`, `merge_assets`, `stake_assets`, `swap_assets`, and `unstake_assets` must not be described as confirmed ledger mutation authority until they become tx/reconcile-backed flows.

The canonical spend must remain `wallet.tx.*`.

### 🔑 Master Key Rotation Boundary

`wallet.key.rotate_master_key` is not full persisted seed rotation today. The live RPC path verifies session/auth, enforces a one-per-hour rate limit, audits denied/rate-limited/success outcomes, confirms the current wallet password, requires the literal confirmation string `ROTATE`, and then calls `finish_rotate(...)`.

The current response is produced by `finish_rotate(...)`, which calls `WalletService::rotate_master_key_in_memory(...)`, rederives cached receiver/key state from the currently unlocked seed material, returns `keys_rederived`, and computes a fresh account public-material fingerprint. The service-level `WalletService::rotate_master_key(...)` in `wallet_service_actions_rpc.rs` is still a compatibility shim whose placeholder response is ignored by the RPC path.

Phase 046 must prove and document this exact boundary:

- The RPC surface is security-relevant and must be covered by auth, rate-limit, audit, wrong-password, wrong-confirmation, and secret-clean log checks.
- The phase must not claim persisted seed rotation, seed reminting, or durable master-key rewrite.
- The compatibility shim comment must be corrected so future readers understand that the live RPC result comes from `finish_rotate(...)` and `rotate_master_key_in_memory(...)`, not from the placeholder response.

## 🧭 Phase Decisions

### ⚙️ Decision 1: Add a New Simulator Stage Instead of Replacing Stage 6

**Decision:** Add a new Scenario 1 stage named `wallet_tx_rpc_lifecycle` after the existing checkpoint stages.  
**Rationale:** Stage 6 is already the canonical handoff producer for checkpoint and storage apply lanes. Replacing it risks concept drift and breaks existing checkpoint tests.  
**Impact:** Stage 6 continues proving package/checkpoint mechanics; the new stage proves the real wallet RPC lifecycle on the same wallet service and persisted wallet files.

### ⚙️ Decision 2: Use `.wlt` Claimed Assets as Input Authority

**Decision:** Transaction building must use `WalletService::list_claimed_assets(...)` through `wallet.tx.build_transaction_impl(...)`; no new simulator asset table is introduced.  
**Rationale:** The live wallet already persists claimed assets in `.wlt` snapshots. Adding a new table would create concept drift and split authority.  
**Impact:** Simulator tests must assert claimed assets survive lock/reopen/backup/restore and are selectable after restore.

### ⚙️ Decision 3: Keep `wallet.asset.*` Send-Like Methods Explicitly Non-Canonical

**Decision:** Phase 046 must update simulator docs/logging and tests so `wallet.asset.send_asset` is labeled compatibility/UX, not full spend lifecycle proof.  
**Rationale:** The current live tx store and reconcile authority lives under `wallet.tx.*`.  
**Impact:** No false user-facing claim that placeholder or compatibility methods perform confirmed ledger mutation.

### ⚙️ Decision 4: Backup Restore Requires `WalletPlusHistory`

**Decision:** Full restore validation must use `ForensicImportMode::WalletPlusHistory` when proving wallet state plus tx history continuity.  
**Rationale:** `.wlt` and tx JSONL are separate persistence planes.  
**Impact:** Backup tests must compare restored claimed assets and restored tx JSONL rows.

### ⚙️ Decision 5: Key Rotation Is a Security Boundary, Not Persisted Seed Rotation

**Decision:** Phase 046 covers the live `wallet.key.rotate_master_key` RPC as an authenticated, rate-limited, audited in-memory rederive flow and explicitly labels persisted seed/master-key rewrite as outside this phase.  
**Rationale:** Current live code reuses unlocked seed material and rebuilds cached derivation state; claiming durable seed rotation would be false.  
**Impact:** Tests and docs must prove the security boundary without overstating cryptographic persistence semantics.

### ⚙️ Decision 6: Stale Placeholder Language Is a Deliverable

**Decision:** The phase includes a cleanup pass for stale `stub`, `Phase 1`, `placeholder`, and `residue` wording in wallet service action comments and simulator-facing docs.  
**Rationale:** The misses file identifies a documentation/UX risk: users can misread compatibility surfaces as either empty stubs or confirmed ledger operations.  
**Impact:** No public behavior changes are required, but false comments and labels must be corrected before closeout.

### ⚙️ Decision 7: Reuse Existing Storage and Settlement Bridges Instead of Adding New Root Math

**Decision:** Any new Phase 046 storage proof or report helper must reuse `asset_wire_to_leaf(...)`, `CheckpointPackageProofVerifier`, and existing `AssetStore` / storage replay helpers instead of introducing custom root or digest code.  
**Rationale:** The current repository already binds `prev_root`, output leaves, checkpoint execution rows, and storage apply semantics across simulator, rollup settlement, and `z00z_storage`. A second root implementation would drift.  
**Impact:** New Stage 13 code may add thin adapters and assertions, but `state_root` and `flat_root` must be read from existing live helpers and persisted artifacts, not recomputed by unrelated code.

### ⚙️ Decision 8: Keep Receive Ownership Detection in `recv_range(...)`

**Decision:** Phase 046 must document and test `WalletService::recv_range(...)` + `StealthOutputScanner` as the live receive/scan authority and must keep `ScanEngineImpl` explicitly future-only.  
**Rationale:** The wallet already persists scan cursor state and claimed hits through the receive service lane. Treating DB `scan_state`, `ScanStorageImpl`, `scan_engine`, or a remote/JMT scanner as authority would overstate live behavior.  
**Impact:** New tests extend existing `recv_range` restart/persistence coverage and Stage 13 report language must not claim a JMT-side or JMT-driven ownership detector.

## 📌 Required Implementation Order

Implement in this exact order to keep dependencies testable and avoid drift.

1. Add Scenario 1 config and design entries for the new wallet tx lifecycle stage.
2. Add the new simulator stage module and runner registration.
3. Add Stage 13 storage/root proof helpers and reuse-visible simulator bridge helpers.
4. Add Stage 13 scanner-boundary/report helpers and extend wallet receive restart coverage.
5. Implement RPC lifecycle helpers that call live `wallet.tx.*` through `LoggedRpcTransport`.
6. Add tamper/fail-closed cases for tx import/reconcile/package verification.
7. Add backup restore with `WalletPlusHistory` checks.
8. Add TOFU/payment request checks against live `wallet.key.*` and `wallet.tx.build_transaction` target parsing.
9. Add session hardening checks for unlock/show-seed/lifecycle/expired sessions.
10. Add `wallet.key.rotate_master_key` boundary checks for auth, confirmation, rate limit, audit, in-memory rederive, and secret-clean logs.
11. Add history status checks for pending/cancelled/confirmed plus imported/exported marker checks.
12. Add asset UX clarity assertions and update docs/comments that still imply stub-only behavior or confirmed mutation where only compatibility behavior exists.
13. Run focused wallet and simulator tests.

## ⚙️ Implementation Details

### 📌 Step 1: Scenario Config

Add a dedicated config section to `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`.

```yaml
stage13_wallet_tx_rpc_lifecycle:
    enabled: true
    sender_actor: "alice"
    receiver_actor: "bob"
    change_actor: "alice"
    amount: 1
    asset_id: null
    memo: "phase046-wallet-tx-rpc-lifecycle"
    tx_exports_dir: "transactions/tx_exports"
    report_file: "transactions/wallet_tx_rpc_lifecycle.json"
    logger_file: "wallet_tx_rpc_lifecycle.log"
    require_cancel_release: true
    require_import_owned_output: true
    require_restore_history: true
    require_tofu_negative: true
    require_session_negative: true
```

Add a matching config type in the simulator config module that follows the existing `ScenarioCfg` style. The type name must stay under the five-word identifier rule.

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Stage13WalletTxCfg {
    pub enabled: bool,
    pub sender_actor: String,
    pub receiver_actor: String,
    pub change_actor: String,
    pub amount: u64,
    pub asset_id: Option<String>,
    pub memo: String,
    pub tx_exports_dir: String,
    pub report_file: String,
    pub logger_file: String,
    pub require_cancel_release: bool,
    pub require_import_owned_output: bool,
    pub require_restore_history: bool,
    pub require_tofu_negative: bool,
    pub require_session_negative: bool,
}
```

Add this optional field to `ScenarioCfg`:

```rust
#[serde(default)]
pub stage13_wallet_tx_rpc_lifecycle: Option<Stage13WalletTxCfg>,
```

### 📌 Step 2: Design Stage Entry

Add a stage to `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` after current stage 12.

```yaml
- stage: 13
  name: wallet_tx_rpc_lifecycle
  description: >
    Prove the live wallet.tx.* RPC lifecycle over the persisted wallet claimed-asset
    store and canonical tx-history JSONL without using simulator-only tx-history writes.
```

Update `crates/z00z_simulator/src/scenario_1/runner_contract_table.in` with a matching `CanonicalStageSpec` entry; keep generated or derived contract artifacts in sync if the local workflow refreshes them. Required step ids:

- `S13-1`: Prepare RPC lifecycle dirs and logged transport.
- `S13-2`: Unlock sender and receiver wallets.
- `S13-3`: Build tx through `wallet.tx.build_transaction`.
- `S13-4`: Verify pending reservation with `wallet.tx.list_pending_transactions`.
- `S13-5`: Cancel first tx and verify cancellation status.
- `S13-6`: Rebuild tx and verify package with `wallet.tx.verify_transaction_package`.
- `S13-7`: Broadcast tx with `wallet.tx.broadcast_transaction`.
- `S13-8`: Reconcile tx with `wallet.tx.reconcile_transaction`.
- `S13-9`: Verify details/history through `wallet.tx.get_transaction_details` and `wallet.tx.get_transaction_history`.
- `S13-10`: Export/import portable tx and assert receiver owned-output behavior.
- `S13-11`: Run tamper fail-closed checks.
- `S13-12`: Run backup restore plus tx-history checks.
- `S13-13`: Run TOFU/payment request checks.
- `S13-14`: Run session hardening checks.
- `S13-15`: Write lifecycle report and logs.

### 📌 Step 3: Runner Registration

Add `pub mod stage_13;` in `crates/z00z_simulator/src/scenario_1/mod.rs` and add the matching module import in `crates/z00z_simulator/src/scenario_1/runner.rs`.

```rust
use crate::{
    config::ScenarioCfgErr, scenario_1::stage_1, scenario_1::stage_10,
    scenario_1::stage_11, scenario_1::stage_12, scenario_1::stage_13,
    scenario_1::stage_2, scenario_1::stage_3, scenario_1::stage_4,
    scenario_1::stage_5, scenario_1::stage_6, scenario_1::stage_7,
    scenario_1::stage_8, scenario_1::stage_9, DesignDoc, DesignErr,
    DesignStage, ScenarioCfg, ScenarioResult, SimContext, StageResult,
    StageState,
};
```

Register the stage in `build_stage_map()`.

```rust
fn build_stage_map() -> BTreeMap<u32, StageFn> {
    let mut stage_map = BTreeMap::new();
    stage_map.insert(1, stage_1::run as _);
    stage_map.insert(2, stage_2::run as _);
    stage_map.insert(3, stage_3::run_claim_prepare as _);
    stage_map.insert(4, stage_4::run_claim_publish as _);
    stage_map.insert(5, stage_5::run_tx_plan as _);
    stage_map.insert(6, stage_6::run_tx_prepare as _);
    stage_map.insert(7, stage_7::run_transfer_receive as _);
    stage_map.insert(8, stage_8::run_transfer_claim as _);
    stage_map.insert(9, stage_9::run_bundle_build as _);
    stage_map.insert(10, stage_10::run_bundle_publish as _);
    stage_map.insert(11, stage_11::run_apply as _);
    stage_map.insert(12, stage_12::run_finalize as _);
    stage_map.insert(13, stage_13::run_wallet_tx as _);
    stage_map
}
```

Add `pub(crate) mod stage_13;` wherever Scenario 1 stage modules are exported.

### 📌 Step 4: Stage 13 Module Layout

Create these files:

- `crates/z00z_simulator/src/scenario_1/stage_13.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/report.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/storage.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/scan.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tamper.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs`

`stage_13.rs` must be a thin facade, matching the existing stage style.

```rust
use crate::{DesignStage, SimContext, StageResult};

#[path = "stage_13_wallet_tx/mod.rs"]
mod stage_13_wallet_tx;

pub fn run_wallet_tx(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    stage_13_wallet_tx::run(ctx, stage)
}
```

### 📌 Step 4A: Storage Contract and Root Binding

Add `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/storage.rs`.

This helper must stay thin. Prefer reusing the existing simulator/storage bridges over duplicating root logic:

```rust
// crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs
impl CheckpointPackageProofVerifier {
    pub(crate) fn expected_prev_root(pkg: &TxPackage) -> Result<CheckRoot, String>;
    pub(crate) fn from_stage7(pkg: &TxPackage, outputs: &[TxOutputWire]) -> Result<Self, String>;
    pub(crate) fn verify_pkg_contract(pkg: &TxPackage) -> Result<(), String>;
}

// crates/z00z_simulator/src/scenario_1/storage_view.rs
fn sync_snapshot_store(view_root: &Path, snapshot: &PrepSnapshot) -> Result<AssetStore, String>;
fn sync_post_store(
    view_root: &Path,
    snapshot: &PrepSnapshot,
    exec: &CheckpointExecInput,
    draft: &CheckpointDraft,
) -> Result<AssetStore, String>;
```

If Stage 13 needs the `storage_view.rs` helpers, widen only these exact helpers to `pub(crate)` and keep lower-level JMT/save internals private.

Stage 13 must prove, in this order:

1. `CheckpointPackageProofVerifier::verify_pkg_contract(&pkg)` passes for the live package bytes.
2. `CheckpointPackageProofVerifier::expected_prev_root(&pkg)` matches the prep snapshot / pre-tx storage root.
3. `CheckpointPackageProofVerifier::from_stage7(&pkg, outputs)` matches checkpoint execution outputs after `asset_wire_to_leaf(...)`.
4. Replayed pre-store state matches the snapshot root and replayed post-store state matches the checkpoint draft new root.
5. Reports and JSON artifacts name `prev_root`, `state_root`, and `flat_root` separately, with `flat_root` loaded from persisted storage metadata rather than `check_root()` replay helpers.

Use the existing store replay helpers rather than custom hashing:

```rust
use crate::scenario_1::stage_6_utils::bundle_lane_impl::CheckpointPackageProofVerifier;
use crate::scenario_1::storage_view::{sync_post_store, sync_snapshot_store};

CheckpointPackageProofVerifier::verify_pkg_contract(&pkg)?;
let expected_prev_root = CheckpointPackageProofVerifier::expected_prev_root(&pkg)?;
if expected_prev_root != snapshot.prev_root {
    return Err("stage13 prev_root drift".to_string());
}

let pre_store = sync_snapshot_store(&view_root, &snapshot)?;
let post_store = sync_post_store(&view_root, &snapshot, &exec_input, &draft)?;
if pre_store.check_root().map_err(|e| e.to_string())? != snapshot.prev_root {
    return Err("stage13 pre-store root mismatch".to_string());
}
if post_store.check_root().map_err(|e| e.to_string())? != draft.new_root() {
    return Err("stage13 post-store root mismatch".to_string());
}
```

These replay helpers prove semantic `state_root` through `check_root()`. If Stage 13 records `flat_root`, load it from persisted storage metadata or artifact files instead of inferring it from `sync_snapshot_store(...)` / `sync_post_store(...)`.

If a Stage 13 helper must materialize output ops, it must reuse `asset_wire_to_leaf(...)` plus `StoreItem::new(...)` / `StoreOp::Put(...)`; do not add alternate leaf encoders or root combiners.

### 📌 Step 4B: Scanner Boundary and Scan-State Proof

Add `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/scan.rs`, but keep it as a proof/report helper only. Do not add a second scan engine.

Extend the existing wallet receive coverage around `recv_range` restart/persistence instead of building a simulator-only scanner:

```rust
// crates/z00z_wallets/src/services/wallet_service_tests.rs
let first = service_a
    .recv_range(&wallet_id, &chunks, &[], Some(1))
    .await
    .unwrap();
let saved_a = session_a
    .with_wallet_session(crate::db::read_scan_state)
    .unwrap()
    .expect("scan state");
let claimed_a = service_a.list_claimed_assets(&wallet_id).await.unwrap();
```

Stage 13 and wallet tests must treat this as the authoritative receive boundary:

- `recv_range(...)` detects ownership.
- `recv_route(..., ReceiveNext::PersistClaim)` persists hits.
- `read_scan_state` / `upsert_scan_state` persist the live wallet resume cursor; `ScanStorageImpl` persists separate local scan state only.
- `ScanEngineImpl` remains deferred and must stay out of Phase 046 success claims.
- If a future remote aggregator/JMT integration is added in this path, it must stay a read adapter that serves chunks/proofs (and related pagination/cache), not a place that performs ownership detection.
- If scanning is ever split into a separate worker for scale, keep it wallet-owned around the same `recv_range(...)` / `StealthOutputScanner` boundary instead of promoting a JMT-side scanner service.

If Phase 046 mentions verification/auth language for this path, reuse the live claim-proof boundary rather than JMT wording:

```rust
// crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl_proof.rs
fn verify_claim_authority(tx: &ClaimTxWire, stmt: &ClaimStmt) -> Result<(), ClaimTxError> {
    let sig = decode_claim_auth(&tx.auth.claim_authority_sig_hex)?;
    sig.verify_with_pk(stmt, &claim_auth_pk())
        .map_err(|e| ClaimTxError::AuthoritySigInvalid(e.to_string()))
}
```

Do not add or document a JMT scanner, a JMT-side ownership detector, or a replacement receive lane in this phase.

### 📌 Step 5: Stage 13 RPC Flow

Use the existing logged local transport builder. Do not construct a parallel dispatcher.

```rust
use crate::scenario_1::stage_2::{actor_password, build_logged_transport_with_wallet};
use crate::scenario_1::stage_4_utils::find_actor;
use serde_json::json;
use std::sync::Arc;
use z00z_networks_rpc::RpcTransport;
use z00z_utils::{codec::{Codec, JsonCodec}, io::{create_dir_all, save_json}};
use z00z_wallets::adapters::rpc::types::{
    common::{PersistTxId, RuntimePaginationParams},
    tx::{RuntimeBuildTxResponse, RuntimeTxDetailsResponse},
};

async fn run_tx_lifecycle(
    ctx: &mut SimContext,
    stage: &DesignStage,
) -> Result<WalletTxReport, String> {
    let cfg = ctx
        .config
        .stage13_wallet_tx_rpc_lifecycle
        .as_ref()
        .ok_or_else(|| "stage13 wallet tx config missing".to_string())?;

    if !cfg.enabled {
        return Ok(WalletTxReport::skipped(stage.stage));
    }

    let wallet_service = ctx
        .wallet_service
        .as_ref()
        .ok_or_else(|| "stage13 requires stage2 wallet_service".to_string())?;

    let out = ctx.outputs_dir.clone();
    let log_dir = out.join("logs");
    let tx_dir = out.join(&cfg.tx_exports_dir);
    create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    create_dir_all(&tx_dir).map_err(|e| e.to_string())?;

    let transport = build_logged_transport_with_wallet(
        Arc::clone(wallet_service),
        &log_dir.join(&cfg.logger_file),
    )?;

    let sender = find_actor(ctx, &cfg.sender_actor)?;
    let receiver = find_actor(ctx, &cfg.receiver_actor)?;

    let sender_session = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({
                "id": sender.wallet_id,
                "password": actor_password(&sender.name)?,
            }),
        )
        .await
        .map_err(|error| format!("stage13 sender unlock failed: {error}"))?;

    let receiver_session = transport
        .call(
            "wallet.session.unlock_wallet",
            json!({
                "id": receiver.wallet_id,
                "password": actor_password(&receiver.name)?,
            }),
        )
        .await
        .map_err(|error| format!("stage13 receiver unlock failed: {error}"))?;

    let receiver_card = transport
        .call(
            "wallet.key.get_receiver_card",
            json!({ "session": receiver_session }),
        )
        .await
        .map_err(|error| format!("stage13 receiver card failed: {error}"))?;

    let recipient = receiver_card["card_compact"]
        .as_str()
        .ok_or_else(|| "stage13 receiver card_compact missing".to_string())?
        .to_string();

    let first_build = build_tx_rpc(
        &transport,
        sender_session.clone(),
        recipient.clone(),
        cfg.amount,
        cfg.asset_id.clone(),
    )
    .await?;

    assert_pending_contains(&transport, sender_session.clone(), &first_build.tx_id).await?;

    transport
        .call(
            "wallet.tx.cancel_transaction",
            json!({
                "session": sender_session,
                "tx_id": first_build.tx_id,
            }),
        )
        .await
        .map_err(|error| format!("stage13 cancel failed: {error}"))?;

    let sender_session = unlock_actor(&transport, sender).await?;
    let second_build = build_tx_rpc(
        &transport,
        sender_session.clone(),
        recipient,
        cfg.amount,
        cfg.asset_id.clone(),
    )
    .await?;

    transport
        .call(
            "wallet.tx.verify_transaction_package",
            json!({
                "session": sender_session,
                "tx_data": second_build.raw_tx,
            }),
        )
        .await
        .map_err(|error| format!("stage13 verify package failed: {error}"))?;

    transport
        .call(
            "wallet.tx.broadcast_transaction",
            json!({
                "session": sender_session,
                "tx_data": second_build.raw_tx,
            }),
        )
        .await
        .map_err(|error| format!("stage13 broadcast failed: {error}"))?;

    transport
        .call(
            "wallet.tx.reconcile_transaction",
            json!({
                "session": sender_session,
                "tx_id": second_build.tx_id,
            }),
        )
        .await
        .map_err(|error| format!("stage13 reconcile failed: {error}"))?;

    let details = transport
        .call(
            "wallet.tx.get_transaction_details",
            json!({
                "session": sender_session,
                "tx_id": second_build.tx_id,
            }),
        )
        .await
        .map_err(|error| format!("stage13 details failed: {error}"))?;

    let details: RuntimeTxDetailsResponse = JsonCodec
        .deserialize(&JsonCodec.serialize(&details).map_err(|e| e.to_string())?)
        .map_err(|e| format!("stage13 details decode failed: {e}"))?;

    Ok(WalletTxReport::from_details(stage.stage, details))
}

async fn build_tx_rpc(
    transport: &impl RpcTransport,
    session: serde_json::Value,
    recipient: String,
    amount: u64,
    asset_id: Option<String>,
) -> Result<RuntimeBuildTxResponse, String> {
    let response = transport
        .call(
            "wallet.tx.build_transaction",
            json!({
                "session": session,
                "recipient": recipient,
                "amount": amount,
                "asset_id": asset_id,
            }),
        )
        .await
        .map_err(|error| format!("wallet.tx.build_transaction failed: {error}"))?;

    JsonCodec
        .deserialize(&JsonCodec.serialize(&response).map_err(|e| e.to_string())?)
        .map_err(|error| format!("build tx response decode failed: {error}"))
}
```

The snippet intentionally uses method names and param fields from `wallet_dispatcher_wiring.rs`:

- `TxBuildParams { session, recipient, amount, asset_id }`
- `TxBroadcastParams { session, tx_data }`
- `TxWalletTxIdParams { session, tx_id }`
- `TxGetHistoryParams { session, pagination, filter, sort }`
- `TxListPendingParams { session, pagination }`

### 📌 Step 6: Pending Reservation and Cancel Release

Add explicit checks that `build_transaction` reserves inputs through pending tx history and that `cancel_transaction` changes status to live JSON value `cancelled`.

```rust
async fn assert_pending_contains(
    transport: &impl RpcTransport,
    session: serde_json::Value,
    tx_id: &PersistTxId,
) -> Result<(), String> {
    let response = transport
        .call(
            "wallet.tx.list_pending_transactions",
            json!({
                "session": session,
                "pagination": { "limit": 50, "cursor": null },
            }),
        )
        .await
        .map_err(|error| format!("list pending failed: {error}"))?;

    let found = response["items"].as_array().is_some_and(|items| {
        items.iter().any(|item| item["tx_id"].as_str() == Some(tx_id.0.as_str()))
    });

    if !found {
        return Err(format!("pending list missing tx_id {}", tx_id.0));
    }

    Ok(())
}
```

Acceptance rule:

- First build creates pending tx.
- Cancel changes status to live JSON value `cancelled`.
- Second build succeeds after cancel, proving the pending-input exclusion does not permanently lock claimed assets.

### 📌 Step 7: Reconcile Must Mutate Claimed Assets

`reconcile_transaction_impl` in `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs` already performs the required mutation:

```rust
let mut next_claimed_assets = self
    .service
    .list_claimed_assets(&wallet_id)
    .await
    .map_err(map_wallet_error_to_rpc)?;
let original_claimed_assets = next_claimed_assets.clone();

next_claimed_assets.retain(|asset| !spent_ids.contains(&asset.asset_id()));

// Wallet-owned outputs are appended.
self.service
    .set_claimed_assets(&wallet_id, next_claimed_assets)
    .await
    .map_err(map_wallet_error_to_rpc)?;
```

Phase 046 must add a simulator assertion around this behavior:

```rust
let before_assets = wallet_service
    .list_claimed_assets(&sender_id)
    .await
    .map_err(|error| format!("stage13 claimed assets before failed: {error}"))?;

// broadcast + reconcile live RPC here

let after_assets = wallet_service
    .list_claimed_assets(&sender_id)
    .await
    .map_err(|error| format!("stage13 claimed assets after failed: {error}"))?;

if before_assets == after_assets {
    return Err("stage13 expected reconcile to update claimed assets".to_string());
}
```

This is the direct answer to the original asset-storage question: the transaction builder selects inputs from `wallet_claimed_assets`; reconcile removes spent inputs and adds wallet-owned change outputs; snapshot persistence writes the updated claimed set back into `.wlt`.

If Stage 13 enables the storage contract helper from Step 4A, compare the replayed post-store root to the checkpoint draft root after reconcile and record `state_root` / `flat_root` separately in the report. Do not treat claimed-asset mutation alone as sufficient storage proof.

### 📌 Step 8: Receiver Import Parity

Add receiver-side import proof using `wallet.tx.export_transaction` and `wallet.tx.import_transaction`.

```rust
let export_response = transport
    .call(
        "wallet.tx.export_transaction",
        json!({
            "session": sender_session,
            "tx_id": confirmed_tx_id,
        }),
    )
    .await
    .map_err(|error| format!("stage13 export failed: {error}"))?;

let export_path = export_response["export_path"]
    .as_str()
    .ok_or_else(|| "stage13 export_path missing".to_string())?
    .to_string();

let tx_data = z00z_utils::io::read_to_string(&export_path)
    .map_err(|error| format!("stage13 export read failed: {error}"))?;

let import_response = transport
    .call(
        "wallet.tx.import_transaction",
        json!({
            "session": receiver_session,
            "tx_data": tx_data,
        }),
    )
    .await
    .map_err(|error| format!("stage13 import failed: {error}"))?;

if !import_response["imported_outputs"]
    .as_array()
    .is_some_and(|items| !items.is_empty())
{
    return Err("stage13 import did not detect receiver-owned outputs".to_string());
}
```

Use the existing live import checks in `import_transaction_impl`: package verification, chain id check, and wallet-owned output scan. Do not add simulator-only ownership rules.

Phase 046 must also prove receiver-side continuation after import:

- `wallet.tx.get_transaction_history` on the receiver session returns the imported `tx_id` in pending state.
- The receiver-side persisted tx record is marked `imported == true`.
- `wallet.tx.broadcast_transaction` from the receiver session submits with `TxSubmitterRole::Receiver`.
- `wallet.tx.reconcile_transaction` from the receiver session confirms the same imported tx id without inventing a second receiver-only tx lane.

If the live broadcast path requires raw package bytes instead of the exported JSON wrapper, use the same conversion helper already exercised by the wallet RPC tests (for example `portable_tx_bytes_from_export(&tx_data)`) instead of changing the RPC contract.

```rust
let receiver_history = transport
    .call(
        "wallet.tx.get_transaction_history",
        json!({
            "session": receiver_session,
            "pagination": { "limit": 50, "cursor": null },
            "filter": { "status": "pending" },
            "sort": { "by": "timestamp", "direction": "desc" },
        }),
    )
    .await
    .map_err(|error| format!("stage13 receiver history failed: {error}"))?;

let receiver_items = receiver_history["items"]
    .as_array()
    .ok_or_else(|| "stage13 receiver history items missing".to_string())?;

if !receiver_items.iter().any(|item| item["id"] == import_response["tx_id"]) {
    return Err("stage13 receiver history missing imported tx".to_string());
}

let receiver_broadcast = transport
    .call(
        "wallet.tx.broadcast_transaction",
        json!({
            "session": receiver_session,
            "tx_data": portable_tx_bytes_from_export(&tx_data),
        }),
    )
    .await
    .map_err(|error| format!("stage13 receiver broadcast failed: {error}"))?;

transport
    .call(
        "wallet.tx.reconcile_transaction",
        json!({
            "session": receiver_session,
            "tx_id": receiver_broadcast["tx_id"],
        }),
    )
    .await
    .map_err(|error| format!("stage13 receiver reconcile failed: {error}"))?;
```

### 📌 Step 9: Tamper and Fail-Closed Tests

Add `stage_13_wallet_tx/tamper.rs` to mutate portable tx/package JSON and assert no wallet state mutation on failure.

Required tamper cases:

- `tx_id` mismatch before reconcile.
- `tx_hash` mismatch before reconcile.
- Wrong `chain_id` in portable tx package before import.
- Bad `verified_block_height == 0` in confirmation evidence path.
- Bad `checkpoint_hex` or `confirmed_root_hex` length/content.
- Wrong `spent_asset_ids` or `created_asset_ids` relative to `TxPackage` summary.

Use the existing evidence validator behavior in `validate_confirmation_evidence(...)`.

```rust
fn assert_claimed_unchanged(
    before: &[z00z_core::Asset],
    after: &[z00z_core::Asset],
    label: &str,
) -> Result<(), String> {
    if before != after {
        return Err(format!("{label}: claimed assets mutated after rejected tx"));
    }
    Ok(())
}
```

Every negative case must:

1. Capture `list_claimed_assets(...)` before.
2. Call the failing RPC.
3. Assert an error result.
4. Capture `list_claimed_assets(...)` after.
5. Assert equality.

If a negative case also exercises the storage proof helper, it must leave the reported pre/post storage root pair unchanged and must not emit a fresh post-commit root artifact.

### 📌 Step 10: Backup Restore with Claimed Assets and Tx History

Use the live backup service. The current implementation already supports history export/import:

```rust
// crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs
pub async fn create_backup(
    &self,
    wallet_id: &PersistWalletId,
    password: SafePassword,
    destination: Option<String>,
) -> WalletResult<RuntimeCreateBackupResponse>;

pub async fn restore_backup_with_mode(
    &self,
    backup_path: String,
    password: SafePassword,
    wallet_name: Option<String>,
    mode: crate::core::backup::ForensicImportMode,
) -> WalletResult<RuntimeRestoreBackupResponse>;
```

Phase 046 must call restore with `WalletPlusHistory`:

```rust
let backup = wallet_service
    .create_backup(&wallet_id, backup_password.clone(), Some(dest_dir))
    .await
    .map_err(|error| format!("stage13 backup failed: {error}"))?;

let restored = restore_service
    .restore_backup_with_mode(
        backup.backup_path.clone(),
        backup_password,
        Some("phase046-restored".to_string()),
        z00z_wallets::core::backup::ForensicImportMode::WalletPlusHistory,
    )
    .await
    .map_err(|error| format!("stage13 restore failed: {error}"))?;
```

Acceptance checks:

- Restored wallet id matches backup metadata semantics.
- Restored `list_claimed_assets(...)` matches source after reconcile.
- Restored `wallet_history_jsonl_path(...)` exists.
- Decoded source and restored JSONL histories are equal after `decode_tx_history_jsonl(...)`.
- Wrong password returns `WalletError::InvalidPassword` through the public RPC/security mapping.

### 📌 Step 11: TOFU and Payment Request Checks

Use live key RPC methods:

```rust
// crates/z00z_wallets/src/adapters/rpc/methods/key.rs
async fn create_payment_request(
    &self,
    session: SessionToken,
    amount: Option<u64>,
    expiry_secs: u64,
    metadata: Option<RuntimePaymentRequestMetaInput>,
) -> RpcResult<RuntimeCreatePaymentRequestResponse>;

async fn validate_payment_request(
    &self,
    session: SessionToken,
    request_compact: String,
) -> RpcResult<RuntimeValidatePaymentRequestResponse>;
```

`wallet.tx.build_transaction` already accepts both receiver-card compact records and payment requests through `parse_tx_build_target(...)`.

```rust
let payment = transport
    .call(
        "wallet.key.create_payment_request",
        json!({
            "session": receiver_session,
            "amount": cfg.amount,
            "expiry_secs": 600,
            "metadata": null,
        }),
    )
    .await
    .map_err(|error| format!("stage13 payment request create failed: {error}"))?;

let request_compact = payment["request_compact"]
    .as_str()
    .ok_or_else(|| "stage13 payment request compact missing".to_string())?
    .to_string();

transport
    .call(
        "wallet.key.validate_payment_request",
        json!({
            "session": sender_session,
            "request_compact": request_compact,
        }),
    )
    .await
    .map_err(|error| format!("stage13 payment request validate failed: {error}"))?;
```

Required negative cases:

- Expired request maps to `REQUEST_EXPIRED`.
- Wrong chain id maps to `REQUEST_CHAIN_MISMATCH`.
- Invalid signature maps to `REQUEST_INVALID_SIGNATURE`.
- Revoked receiver card record maps to receiver-card rejection.
- Changed receiver view key returns `SEND_TOFU_CONFIRM_REQUIRED` when used by `wallet.tx.build_transaction`.

Use existing live types:

- `ReceiverCardRecord::from_compact(...)`
- `ReceiverCardRecord::revoked()`
- `ReceiverCardRecord::to_compact()`
- `ValidatePaymentRequest::validate_all(...)`
- `ValidationOutcome::{Approved, RequiresUserConfirmation, IdentityMismatch}`

### 📌 Step 12: Session Hardening Checks

Use the live session methods and limits:

- `WalletService::unlock_attempt_precheck(...)`: 5 requests per 60 seconds.
- `WalletService::show_seed_phrase_precheck(...)`: 3 requests per 60 seconds.
- `WalletService::rotate_master_key_precheck(...)`: 1 request per hour.
- `WalletService::verify_session(...)`: validates token and touches activity.
- `WalletService::verify_session_no_touch(...)`: validates token without touching activity.
- `WalletService::check_auto_lock(...)`: locks inactive wallets.
- `wallet.lifecycle.on_event`: public RPC lifecycle event bridge.

Stage 13 must prove:

1. Repeated wrong unlock returns rate-limit shaped error and does not leak password.
2. `wallet.session.show_seed_phrase` rate-limits after the configured limit.
3. `wallet.lifecycle.on_event` invalidates or locks as configured by the live service.
4. A stale session cannot build, cancel, broadcast, reconcile, import, or export tx.
5. `wallet.key.rotate_master_key` rejects wrong password as authentication failure, rejects wrong confirmation as invalid params, rate-limits after the configured limit, and audits denied/rate-limited/success outcomes.
6. Successful `wallet.key.rotate_master_key` returns a non-empty `new_fingerprint`, non-zero `rotated_at`, and `keys_rederived` equal to the cached receiver/key paths rederived by `rotate_master_key_in_memory(...)`.
7. The scenario output and comments explicitly say this is in-memory rederive/key-cache rebuild, not persisted seed rotation or durable master-key rewrite.
8. RPC log artifacts contain no password, seed phrase, raw session token, exported mnemonic, or key material.

Do not add a second session manager. Use `WalletRpcImpl`, `WalletService`, and the existing `LoggedRpcTransport`.

### 📌 Step 13: Multi-Status History

Use the live runtime filter and sort DTOs:

```rust
pub struct RuntimeTxHistoryFilter {
    pub status: Option<TxStatus>,
    pub from_date: Option<u64>,
    pub to_date: Option<u64>,
    pub min_amount: Option<u64>,
    pub max_amount: Option<u64>,
}

pub struct RuntimeTxHistorySort {
    pub by: TxHistorySortBy,
    pub direction: SortDirection,
}
```

Required history evidence in one scenario run, using live snake_case JSON values:

- One `pending` row from first build.
- One `cancelled` row from first cancellation.
- One `pending` row from second build.
- One `confirmed` row after broadcast/reconcile.
- One `WalletTxHistoryEntryKind::Imported` row or persisted `imported == true` record on receiver import.
- One `WalletTxHistoryEntryKind::Exported` marker through `record_exported(...)`.

Add assertions against `wallet.tx.get_transaction_history` for the status-bearing surface only:

```rust
transport
    .call(
        "wallet.tx.get_transaction_history",
        json!({
            "session": sender_session,
            "pagination": { "limit": 50, "cursor": null },
            "filter": { "status": "confirmed" },
            "sort": { "by": "timestamp", "direction": "desc" },
        }),
    )
    .await
    .map_err(|error| format!("stage13 confirmed history failed: {error}"))?;
```

Imported lifecycle evidence must be asserted through canonical JSONL row kinds or persisted `TxRecord.imported`, and exported lifecycle evidence must be asserted through canonical JSONL exported rows or `record_exported(...)` persistence, because the live `PersistTxInfo` DTO does not expose separate imported/exported status enums.

If serde enum casing differs in the live DTO, fix the test call to match the DTO serialization, not the other way around.

### 📌 Step 14: Asset UX Clarity

Update only docs/comments/log labels that are false or misleading. Do not change public API semantics in this phase.

Required clarifications:

- `wallet.asset.list_assets`: lists live claimed assets after quarantine filtering; it is not the pending/available balance authority.
- `wallet.asset.get_asset_balance`: reports `total`, `pending`, and `available` using reserved-input state and pending-owner checks.
- `wallet.asset.import_asset`: compatibility import into claimed asset state when validation passes.
- `wallet.asset.receive_asset`: report-only compatibility lane for one asset id.
- `wallet.asset.send_asset`: not the canonical confirmed spend lifecycle.
- `wallet.tx.*`: canonical tx lifecycle.
- `wallet receive`: canonical scan authority is `wallet_service_actions_receive::recv_range`; `scan_asset_report` is compatibility-only; DB scan-state persistence and `ScanStorageImpl` are resume/state stores, not second scanners.
- `wallet receive remote source`: if a future aggregator/JMT integration is added in this path, it must remain a chunk/proof read adapter only; ownership detection stays wallet-side in `recv_range(...)` / `StealthOutputScanner`, and any worker split stays wallet-owned.
- `wallet claim auth wording`: current claim-path verification is `ClaimSourceProof` + `verify_claim_authority(...)` / `claim_auth_pk()`, not JMT-based auth.
- `wallet.key.rotate_master_key`: authenticated/rate-limited/audited in-memory rederive boundary, not persisted seed rotation.

Required stale-comment cleanup targets:

- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_rpc.rs`: replace the `reachability placeholder path` wording for `rotate_master_key(...)` with a compatibility-shim explanation that the RPC result is produced by `finish_rotate(...)` and `rotate_master_key_in_memory(...)`.
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`: replace generic `Phase 1 reachability stub/placeholder` wording with explicit compatibility/reachability language for storage helpers and `show_seed_phrase` surfaces.
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`: replace generic `Phase 1 reachability placeholder` wording for asset list/merge/metadata/receive/send/split/staking/swap/unstaking helpers with exact compatibility wording and state whether each path is report-only, claimed-state import, or non-ledger demo behavior.
- `crates/z00z_wallets/src/chain/scan_engine_impl.rs`: keep the explicit deferred seam wording and remove any label drift that makes the scan engine sound live before implementation exists.
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_assets.rs`: remove or replace the `Phase 030 residue placeholder pending deletion or replacement` header if the file remains part of the build.
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup_rpc.rs`: remove or replace the `Phase 030 residue placeholder pending deletion or replacement` header if the file remains part of the build.
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs`: replace the top-level `Stub implementations for asset.* RPC methods (Phase 1)` wording with compatibility/lifecycle wording that matches the live RPC behavior.
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl.rs`: replace the top-level `Stub implementations for backup.* RPC methods (Phase 1)` wording with wording that matches the live backup RPC behavior.
- Any live wallet/asset/backup RPC entry file surfaced by Phase 046 tests or docs must not keep blanket `stub` / `Phase 1` headers once the phase starts describing the real behavior underneath.
- Any simulator Stage 13 log or README label that says `stub`, `demo spend`, or `asset send completed` must instead name the exact boundary: compatibility asset surface, canonical `wallet.tx.*` lifecycle, or in-memory key rederive.

Do not delete compatibility methods. Do not rename RPC methods.

## 🧪 Required Tests

### ✅ Wallet Unit Tests

Add or extend tests near `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl/tests/`.

Use existing helper patterns:

```rust
async fn setup_session(time: Arc<MockTimeProvider>) -> TestSessionCtx;
async fn seed_spendable_stealth_coin(ctx: &TestSessionCtx, amount: u64, serial_id: u32);
async fn mk_recv_card_compact(ctx: &TestSessionCtx) -> String;
fn tx_history_rows(ctx: &TestSessionCtx) -> Vec<WalletTxHistoryJsonlEntry>;
fn tx_history_kinds(ctx: &TestSessionCtx) -> Vec<WalletTxHistoryEntryKind>;
```

Required tests:

- `test_tx_build_reserves_then_cancel_releases`
- `test_tx_reconcile_updates_claimed_assets`
- `test_tx_reconcile_rejects_bad_evidence_without_asset_mutation`
- `test_tx_import_rejects_wrong_chain_without_history_mutation`
- `test_tx_history_filters_pending_cancelled_confirmed`
- `test_tx_export_import_detects_receiver_owned_outputs`

### ✅ Wallet Service Persistence Tests

Add or extend tests near `crates/z00z_wallets/src/services/wallet_service_tests.rs`.

Required tests:

- `test_claimed_assets_restore_from_wlt_snapshot`
- `test_claimed_assets_checksum_tamper_rejected`
- `test_wallet_plus_history_restore_keeps_tx_jsonl`
- `test_restore_wrong_password_rejects_without_wallet_mutation`
- Extend existing `test_recv_range_restart` so it remains the canonical proof that `recv_range(...)` resumes from persisted `read_scan_state(...)` and only grows claimed assets through `ReceiveNext::PersistClaim`.

### ✅ Simulator Tests

Add tests near the new `stage_13_wallet_tx/tests.rs` and update Scenario 1 smoke checks.

Required tests:

- `test_stage13_storage_contract_matches_prev_root_and_post_store`
- `test_stage13_runs_wallet_tx_rpc_lifecycle`
- `test_stage13_receiver_import_continuation_uses_receiver_path`
- `test_stage13_rejects_tampered_tx_without_claimed_mutation`
- `test_stage13_backup_restore_compares_claimed_assets_and_history`
- `test_stage13_payment_request_negative_paths`
- `test_stage13_session_hardening_negative_paths`
- `test_stage13_history_covers_statuses_and_import_export_markers`

### ✅ Regression Tests That Must Keep Passing

Do not weaken these existing Stage 4/6 checks:

- Canonical history path remains `wallet_<stem>_tx_history.jsonl`.
- No noncanonical tx-history directory is accepted.
- Stage 4/6 transaction package bytes and decoded package remain equal in history assertions.
- Existing tamper options `witness`, `tag16`, `value`, `commit`, `range`, `prev_root_hex` remain fail-closed.
- Existing `test_recv_range_restart` remains the canonical restart/cursor proof for live wallet-side receive scanning.

## 🧾 Acceptance Criteria

### ✅ EARS Requirements

- WHEN a wallet scan detects an owned leaf and the flow reaches `ReceiveNext::PersistClaim`, THE SYSTEM SHALL persist the asset through `wallet_claimed_assets` and `.wlt` snapshot `WalletPersistenceState.claimed_assets`.
- WHERE wallet persistence is described or restored, THE SYSTEM SHALL treat the encrypted snapshot object inside `.wlt` as the canonical claimed-asset persistence plane and SHALL NOT require a separate `wallet_*.bin.enc` file or dedicated asset table.
- WHEN a wallet builds a transaction through `wallet.tx.build_transaction`, THE SYSTEM SHALL select inputs only from claimed assets not reserved by pending tx history.
- WHEN a tx package is prepared and later verified against settlement/storage, THE SYSTEM SHALL use the live `asset_wire_to_leaf(...)` conversion, bind `prev_root` to the spend proof, and prove output inclusion against checkpoint execution rows without introducing alternate root math.
- WHEN storage replay or reporting is emitted, THE SYSTEM SHALL distinguish semantic `state_root` from persisted `flat_root`, SHALL source `flat_root` from persisted storage metadata or artifacts, and SHALL NOT collapse them into one field or explanation.
- WHEN a pending tx is cancelled through `wallet.tx.cancel_transaction`, THE SYSTEM SHALL expose `cancelled` history status and allow later builds to use unspent claimed assets.
- WHEN a tx is broadcast and reconciled through `wallet.tx.broadcast_transaction` and `wallet.tx.reconcile_transaction`, THE SYSTEM SHALL validate package/evidence, remove spent claimed inputs, append wallet-owned outputs, and persist the changed claimed asset set.
- IF tx evidence has mismatched tx id, tx hash, chain id, checkpoint root, spent ids, or created ids, THEN THE SYSTEM SHALL reject fail-closed and leave claimed assets unchanged.
- WHEN a wallet resumes receive scanning after restart, THE SYSTEM SHALL resume from persisted `ScanStatePayload` through the canonical `recv_range(...)` lane and SHALL only persist hits via `ReceiveNext::PersistClaim`.
- WHERE receive scanning is documented or tested, THE SYSTEM SHALL treat `StealthOutputScanner` as the live wallet-side detector, `read_scan_state` as the live wallet resume-cursor persistence path, `ScanStorageImpl` as separate local scan-state persistence, and `ScanEngineImpl` as a deferred seam.
- IF a future receive-scanning integration consumes remote aggregator/JMT data, THEN THE SYSTEM SHALL treat that remote surface as a chunk/proof read adapter only and SHALL keep ownership detection in wallet-side `recv_range(...)` / `StealthOutputScanner`.
- IF docs or simulator output claim a JMT-based or JMT-side wallet scanner in this path, THEN THE SYSTEM SHALL reject that description and use the live wallet-side boundary instead.
- IF docs or simulator output describe auth for this path, THEN THE SYSTEM SHALL reference the live `ClaimSourceProof` plus `verify_claim_authority(...)` / `claim_auth_pk()` boundary and SHALL NOT introduce JMT wording.
- WHEN tx history is requested through `wallet.tx.get_transaction_history`, THE SYSTEM SHALL support pending, cancelled, and confirmed status checks with cursor/filter/sort behavior, imported lifecycle evidence SHALL remain provable through canonical JSONL rows or persisted imported records, and exported lifecycle evidence SHALL remain provable through canonical JSONL exported rows or `record_exported(...)` persistence.
- WHEN a backup is restored with `WalletPlusHistory`, THE SYSTEM SHALL restore both `.wlt` claimed assets and canonical tx-history JSONL with all-or-nothing commit semantics.
- IF backup restore fails at any stage, including wrong password, snapshot decode mismatch, tx-history decode mismatch, JSONL replay failure, or staged write failure, THEN THE SYSTEM SHALL reject without mutating existing wallet state and SHALL discard staged restore outputs for both `.wlt` and tx-history artifacts.
- WHEN a payment request is used as the tx recipient, THE SYSTEM SHALL validate signature, expiry, chain binding, and TOFU status before building tx outputs.
- IF a receiver card is revoked, stale, relabeled, or has changed view/identity material, THEN THE SYSTEM SHALL reject or require confirmation using existing TOFU/payment request errors.
- WHEN session limits are exceeded or a lifecycle lock event occurs, THE SYSTEM SHALL reject sensitive wallet operations without leaking secrets in logs.
- WHEN `wallet.key.rotate_master_key` is called, THE SYSTEM SHALL enforce session auth, password confirmation, secondary `ROTATE` confirmation, one-per-hour rate limiting, audit logging, and secret-clean RPC logs.
- WHEN `wallet.key.rotate_master_key` succeeds in Phase 046, THE SYSTEM SHALL report only the live in-memory rederive result and SHALL NOT claim persisted seed rotation or durable master-key rewrite.
- WHERE `wallet.asset.*` methods remain compatibility or UX operations, THE SYSTEM SHALL not label them as confirmed ledger mutation authority.
- WHERE wallet service comments, live RPC module headers, or simulator labels mention stale `stub`, `placeholder`, `Phase 1`, or `residue` wording, THE SYSTEM SHALL replace them with exact compatibility, report-only, live-state, or canonical-lifecycle wording.

## 🧪 Validation Commands

Run focused tests first:

```bash
cargo test -p z00z_wallets tx_impl --features test-fast
cargo test -p z00z_wallets wallet_service_tests --features test-fast
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump stage13
```

Run the scenario smoke command requested for simulator work:

```bash
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump
```

Before final PR or release gate:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test --all
```

If public Rust API docs change:

```bash
cargo doc --no-deps
```

## 🔍 Doublecheck Register

This table records the live symbols that were verified before writing this specification.

| Area | Verified live symbol/path | Required Phase 046 use |
| --- | --- | --- |
| Tx RPC trait | `crates/z00z_wallets/src/adapters/rpc/methods/tx.rs` | Use exact `wallet.tx.*` method signatures. |
| Tx dispatcher params | `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs` | JSON params must match `TxBuildParams`, `TxBroadcastParams`, `TxWalletTxIdParams`, `TxGetHistoryParams`, `TxListPendingParams`. |
| Tx lifecycle impl | `tx_impl_server_lifecycle.rs` | Build, broadcast, verify, cancel, details are the live implementation. |
| Tx finalize impl | `tx_impl_server_finalize.rs` | Export, import, reconcile, and evidence validation are the live implementation. |
| Tx history impl | `tx_impl_server_history.rs` | History and pending pagination/filter/sort use live RPC. |
| Tx storage | `persistence/tx/tx_storage.rs`, `tx_storage_impl.rs` | Keep canonical `wallet_<stem>_tx_history.jsonl`. |
| Tx leaf conversion | `crates/z00z_wallets/src/tx/witness_gate.rs` | Use `asset_wire_to_leaf(...)`; do not add alternate leaf conversion. |
| Simulator package proof bridge | `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` | Reuse `expected_prev_root`, `from_stage7`, and `verify_pkg_contract`. |
| Storage replay bridge | `crates/z00z_simulator/src/scenario_1/storage_view.rs` | Prefer `sync_snapshot_store` / `sync_post_store` reuse over new root code. |
| Storage semantic root model | `crates/z00z_storage/src/assets/model.rs` | Keep `put_leaf`, `root`, and `del_leaf` as semantic root authority. |
| Storage commit engine | `crates/z00z_storage/src/assets/store_internal/tx_plan_engine.rs` | Keep `commit_stage(...)` as apply authority. |
| Storage root persistence | `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` | Treat `flat_root` and `state_root` as distinct persisted roots. |
| Claimed asset store | `wallet_service_types_core.rs` | `wallet_claimed_assets` is the in-memory ownership set. |
| Claimed asset mutation | `wallet_service_actions_reachability.rs` | Use `put_claimed_asset`, `set_claimed_assets`, and `recv_route`. |
| Receive scan | `wallet_service_actions_receive.rs` | Detection only persists through `ReceiveNext::PersistClaim`. |
| Scan cursor persistence | `db/redb_wallet_store/queries.rs`, `db/redb_wallet_store/mutations/upserts.rs`, `persistence/scans/scan_storage_impl.rs` | Persist `ScanStatePayload` / cursor only, not scanner authority. |
| Wallet-side scanner | `receiver/scan/stealth_scanner.rs` | Use `StealthOutputScanner` as live detector. |
| Deferred scan seam | `chain/scan_engine_impl.rs` | Keep `ScanEngineImpl` future-only; do not treat it as live authority. |
| Snapshot schema | `wallet/snapshot/snapshot_types.rs` | `.wlt` stores `claimed_assets: Vec<AssetWire>`. |
| Snapshot creation | `wallet_service_store_persistence_pack_snapshot.rs` | Snapshot includes `snapshot_claimed_assets`. |
| Snapshot restore | `wallet_service_store_load_restore.rs` | Restore validates/dedups claimed assets and repopulates `wallet_claimed_assets`. |
| `.wlt` write | `db/redb_wallet_store/backup.rs` | Snapshot object is encrypted and atomically updated. |
| Backup history | `wallet_service_actions_backup.rs` | Use `WalletPlusHistory` for `.wlt` plus tx JSONL restore. |
| Receiver card record | `chain/receiver_card_record.rs` | Use `ReceiverCardRecord::from_compact`, `to_compact`, `revoked`, and verifier behavior. |
| Payment request RPC | `methods/key.rs`, `key_impl/server_requests.rs` | Use `wallet.key.create_payment_request` and `wallet.key.validate_payment_request`. |
| Key rotation RPC | `key_impl/server_admin.rs`, `key_impl/support.rs` | Cover auth/rate-limit/audit/password/confirmation and `finish_rotate(...)`. |
| Key rotation service boundary | `wallet_service_actions_rpc.rs`, `wallet_service_session_derivation_recovery.rs` | Label service shim vs in-memory rederive accurately. |
| TOFU bridge | `tx_impl_server_lifecycle.rs`, `wallet_service_actions_tofu.rs` | Build target parsing and TOFU checks are live. |
| Session hardening | `wallet_service_session_runtime_limits.rs`, `wallet_service_session_runtime.rs` | Use live rate limits and auto-lock/session validation on the compiled runtime include chain, not duplicate session helper files. |
| Logged simulator RPC | `stage_2_utils/transport.rs` | Reuse `build_logged_transport_with_wallet`. |
| Current simulator tx lane | `stage_4_utils/tx_lane_runtime_flow.rs` | Do not replace existing proof/checkpoint mechanics. Add Stage 13 proof. |

## 🚫 Non-Goals

- Do not modify `crates/z00z_crypto/tari/`.
- Do not add a new wallet asset RedB table.
- Do not replace `.wlt` claimed asset persistence with a simulator-only store.
- Do not add parallel root or digest logic in simulator/report helpers.
- Do not collapse `state_root` and `flat_root` into one concept or one report field.
- Do not rename public RPC methods.
- Do not claim `wallet.asset.send_asset` is the canonical confirmed spend lifecycle.
- Do not claim `ScanEngineImpl`, `ScanStorageImpl`, or a JMT/JMT-side service is the live wallet receive authority in this path.
- Do not move wallet-secret ownership detection into an aggregator/JMT-side service; if a worker split is ever added, keep it wallet-owned.
- Do not claim `wallet.key.rotate_master_key` performs persisted seed rotation or durable master-key rewrite in Phase 046.
- Do not weaken existing Stage 4/6 checkpoint and tx package tests.
- Do not make Phase 046 depend on Phase 044 being fully closed.

## ✅ Definition of Done

Phase 046 is complete only when:

1. Scenario 1 includes a stage that calls live `wallet.tx.*` RPC methods through `LoggedRpcTransport`.
2. The stage proves build, pending, cancel, rebuild, verify, broadcast, reconcile, details, history, export, and import.
3. Reconcile evidence failure cases leave claimed assets unchanged.
4. `.wlt` claimed asset persistence is proven across lock/reopen and backup restore.
5. `WalletPlusHistory` restore proves tx JSONL continuity.
6. TOFU/payment request negative cases are covered with existing live errors.
7. Session hardening negative cases are covered and logs are secret-clean.
8. `wallet.key.rotate_master_key` boundary checks prove auth, rate limit, audit, confirmation, in-memory rederive, and no persisted-rotation claim.
9. Asset UX docs/log labels distinguish compatibility surfaces from canonical tx lifecycle.
10. Stale placeholder/stub/Phase 1/residue comments in the required wallet action targets are corrected or explicitly justified.
11. Stage 13 or adjacent wallet tests prove the tx-package-to-storage contract and keep `state_root` vs `flat_root` distinct in outputs.
12. Existing `recv_range` restart proof and Phase 046 docs keep receive authority in the wallet-side `StealthOutputScanner` / `ScanStatePayload` lane without promoting `ScanEngineImpl`.
13. Focused `z00z_wallets` and `z00z_simulator` tests pass.
14. Final verification output is recorded before claiming completion.
