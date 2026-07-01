Gathering the storage and transaction processing code paths now.

## Findings

### What we confirmed

- `TxPackage` wire schema is defined in tx_wire_types.rs
  - `TxInputWire` contains `asset_id_hex` and `serial_id`
  - `TxOutputWire` contains `role` plus `asset_wire: AssetPkgWire`
  - `SpendProofWire` carries `prev_root_hex`
  - `TxPackage.tx_digest_hex` is canonical hash over package metadata + normalized tx payload

- Package digest is built by tx_digest.rs
  - It canonicalizes inputs and clears spend-proof statement/proof bytes before hashing
  - Digest includes:
    - `kind`, `package_type`, `version`, `chain_id`, `chain_type`, `chain_name`
    - JSON-serialized normalized `TxWire`

- Full tx package verification is in tx_verifier.rs
  - structure check
  - balance check
  - digest match check
  - signature and range-proof checks
  - plus public spend-contract verification

### How tx package inputs/outputs map into leaves

- `asset_wire_to_leaf(...)` is implemented in witness_gate.rs
  - Converts `AssetWire` into `z00z_core::assets::AssetLeaf`
  - Computes canonical `asset_id` from:
    - nonce
    - commitment
    - definition id
    - serial id
  - Uses `r_pub`, `owner_tag`, commitment bytes, `enc_pack`, `range_proof`, and `tag16`

- In lib.rs
  - `outputs_match(...)` converts `TxOutputWire` back into `AssetWire`
  - then into `StorageLeaf` via `asset_wire_to_leaf(...)`
  - and compares that leaf against checkpoint execution output rows

- In rollup settlement verification
  - `tx_prev_root(...)` reads `spend.prev_root_hex` from the package
  - it must equal `exec_input.prev_root()`
  - the package proof bytes are matched against checkpoint exec rows

### Where root / tree verification lives

- lib.rs is settlement-level verification, not tree update logic
  - it verifies package digest
  - it verifies prev_root binding
  - it verifies tx inclusion and contents against checkpoint rows

- Actual storage tree/root update is inside z00z_storage
  - tx_plan_engine.rs
    - `commit_stage(...)` obtains `prev_root = self.root()`
    - applies `StoreOp::Put` / `StoreOp::Delete`
    - calls `self.commit_all(...)`
    - records `root` for the new version
  - this is where canonical pre-state row changes become a new asset state root

### ==JMT and scanner connection==

- No direct JWT storage-scanner implementation was found in the wallet persistence path
- `ScanStorage` and `TxStorage` remain JSON/file-backed persistence abstractions
- The JWT link is still not visible in the wallet storage path; it is likely elsewhere or not part of the wallet scan-storage boundary

## Next steps for continuation

1. Trace `z00z_storage` storage commit internals:
   - tx_plan_engine.rs
   - store_codec.rs
   - model.rs

2. Inspect how `StorageLeaf` / `AssetPath` are derived and how JMT roots are computed
   - likely in `z00z_storage/src/assets/model.rs` and `z00z_storage/src/assets/leaf.rs`

3. If you want the full “tx package → asset tree → checkpoint root” path, the next handoff should focus on:
   - storage plan generation (`StoreOp` sequences)
   - commit root calculation (`commit_all`, `root()`)
   - proof binding and path canonicalization in `z00z_storage`

If you want, I can continue immediately by reading the storage model/root implementation and the JMT commit path.

## Next step completed

I traced the storage commit path all the way through the model/root layer and backend sync.

### What I confirmed

- [model.rs]
  - [AssetModel::put_leaf()] / [del_leaf()] update the in-memory [defs] tree
  - [AssetModel::root()] computes the canonical [AssetStateRoot]
    - serial roots are Poseidon hashes of asset leaf hashes
    - definition roots are Poseidon hashes of serial root encodings
    - state root is Poseidon hash over sorted definition-root encodings
- [tx_plan_engine.rs]
  - [plan_ops()] prechecks intents and builds [NextState] with the new model
  - [next_roots()] updates [TreeRoots] for affected serial and definition branches
  - [plan_root()] chooses either full root or incremental semantic root depending on [RootMode]
  - [commit_stage()]:
    - reads [prev_root = self.root()]
    - commits raw JMT trees via [commit_all(...)] and stores [flat_root]
    - merges [next] model state into `self`
    - sets [self.tree_roots = tree_roots]
    - calls backend [sync_store(..., flat_root, prev_root, root)]
    - records [root_by_ver.insert(version, root)]
- [redb_backend.rs]
  - [sync_store()] writes both:
    - [flat_root] from the low-level tree commit
    - [state_root] from the [AssetStateRoot] computed by the model
  - it also persists serialized rows, path rows, serial roots, def rows, and checkpoint metadata

### What this means for the tx package / root flow

- [TxPackage] outputs are converted to storage leaves via [asset_wire_to_leaf(...)] in [z00z_wallets]
- those leaves map to [AssetPath] + [AssetLeaf], which are [StoreItem::Put(...)] or delete ops
- [AssetStore] applies these ops in [NextState] and recomputes the tree root
- on successful commit, the backend stores:
  - [prev_root] as the prior state root
  - [flat_root] as the JMT commit root of raw rows
  - [state_root] as the canonical semantic asset state root
- [tx_prev_root()] in [z00z_rollup_node] is therefore checking the prior semantic root against the package’s stated prev_root binding

### Immediate remaining link

I still need one more precise mapping:

- where [TxPackage]/rollup-level [verify_tx_inclusion()] feeds into [AssetStore] op generation in the execution path
- that will show the exact source of [StoreOp::Put/Delete] from tx package outputs and the user-visible storage scanner side

If you want, I can continue from here into the executor/checkpoint path that converts verified package outputs into [StoreOp]s and actual asset-plan commits.

## What I confirmed

### 1. `TxPackage` → `StoreOp` path
- The canonical conversion from tx package outputs/inputs into storage state is through `asset_wire_to_leaf()`.
- In wallet verification / prep code, `AssetWire` is converted into an `AssetLeaf` and then into `StoreOp::Put(...)`.
- Relevant wallet/simulator files:
  - spend_verification.rs
  - witness_gate.rs
  - tx_preparation_core.rs

### 2. Storage commit pipeline
- `StoreOp` execution is handled by `AssetStore` in:
  - tx_plan_engine.rs
- Key flow:
  1. `AssetStore::plan_ops(...)` prechecks ops and builds `ShardPlan`
  2. `AssetStore::apply_put()` / `apply_del()` update the temporary `NextState`
  3. `AssetStore::plan_root(...)` computes the new semantic `AssetStateRoot`
  4. `commit_stage(...)` calls `commit_all(...)` and then `backend.sync_store(...)`

### 3. Root recalculation
- There are two roots in play:
  - `flat_root`: the JMT persistence root from `TreeStore::commit_prep(...)`
  - `state_root`: the semantic asset state root computed from `AssetStore`/`TreeRoots`
- `TreeStore::commit_prep(...)` converts batched asset/serial/def/path updates into namespaced JMT key/value ops and returns the flat root.
- `sync_store(...)` in redb_backend.rs persists:
  - the state root
  - the flat root
  - path rows / asset rows / def rows / serial rows
  - metadata and checkpoint artifacts

### 4. Does storage scanner with JWT work?
- I found no JWT-specific storage scanner in the wallet/storage/settlement path.
- The auth path in claim tx verification is:
  - `ClaimSourceProof` proof decoding from `tx.proof.proof_hex`
  - `verify_claim_authority(...)` verifying a claim authority signature with `claim_auth_pk()`
- That means the current verification/auth model here is claim-proof + signature-based, not JWT-based.

## Short conclusion
- The exact tx package → storage commit mapping is:
  - `AssetWire` → `asset_wire_to_leaf()` → `StoreItem` / `StoreOp::Put`
  - `AssetStore` applies ops to compute next model and semantic root
  - `TreeStore` commits JMT changes and returns flat root
  - `RedbBackend::sync_store()` writes both roots and persistent rows
- There is no obvious JWT scanner in this code path; auth is done via claim source proof and authority signature verification rather than JWT.

If you want, I can now trace the exact caller that builds the first `StoreOp` vector from `TxPackage` outputs in the live settlement/consumer path.



**Да, ты прав: я сформулировал слишком грубо. В wallet code scan — это не внешний JWT-сервис, а несколько внутренних слоёв.**

Коротко по смыслу:

- В chain есть scan_engine, но он прямо помечен как будущий seam, а живой receive path остаётся WalletService.recv_range(...).
- В persistence есть ScanStorageImpl, и это локальное file-backed хранилище scan_state.json.
- В receiver есть StealthOutputScanner и связанные scan-helpers, то есть реальное сканирование выходов живёт внутри кошелька.
- В live receive flow кошелёк читает и пишет свой scan cursor через wallet DB helpers read_scan_state и upsert_scan_state, а не через внешний сервис.
- В wallet sources я не нашёл JWT/JWT references, поэтому связка с JWT в этом пути не подтверждается.

# Verification Report

## ✅ Summary

Да: в storage здесь нет scanner-функции. Storage в кошельке — это слой сохранения состояния скана, а не место, где выполняется сам scan. Само сканирование живёт в wallet receive path через WalletService.recv_range, а scan_engine помечен как будущий seam, не как live authority.

**Проверено:** есть ли в wallet storage scanner-функция и где реально выполняется scanning

---



Коротко: scanner должен жить в wallet-side application service, а не внутри JMT service. JMT у агрегатора — это удалённый read-model / источник чанков и доказательств, но не место, где лежат кошельковые секреты и логика определения “это мой выход”.

В вашем коде это уже почти так и сделано:

- [WalletService.recv_range(...)](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) загружает [ScanStatePayload](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html), создаёт [StealthOutputScanner](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html), сканирует чанки и сохраняет cursor обратно в wallet DB. См. [wallet_service_actions_receive.rs:95](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html).
- [StealthOutputScanner](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) строится из [ReceiverKeys](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html), то есть сканирование завязано на wallet-secret material, а не на JMT. См. [stealth_scanner.rs:55](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html).
- [ScanStorageImpl](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) — это вообще локальный file-backed state для scan cursor, а не сервис сканирования. См. [scan_storage_impl.rs:13](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html).
- [scan_engine](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) в `chain` прямо помечен как proposed-only seam; live authority остаётся [WalletService.recv_range(...)](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html). См. [scan_engine.rs:31](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html).
- В wallet DB [ScanStatePayload](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) хранит только cursor/resume state. См. [redb_wallet_store_tables.rs:137](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html).

Практически правильная архитектура такая:

- У агрегатора/JMT-сервиса: хранение и выдача дерева, чанков, proof’ов, pagination, cache.
- У кошелька: receive/scanner service, который берёт данные из JMT по сети и сам решает ownership по ключам.
- У кошелька же: локальная persistence для cursor, найденных выходов и статуса скана.

Если хочешь вынести это в отдельный процесс ради масштаба, делай не “JMT scanner service”, а “wallet-owned scan worker”, который получает только публичные данные из JMT-адаптера. Ключи и финальное решение о владении должны оставаться на wallet side.

Схема в одну строку:
WalletService.recv_range → JMT read adapter → remote aggregator JMT → chunks/proofs → StealthOutputScanner → wallet DB / ScanStorage

Если нужно, я могу дальше разложить это уже по твоему репо: что оставить в `z00z_wallets`, что должно быть в `z00z_storage`, и какой именно интерфейс нужен между кошельком и удалённым JMT.

----





## 16. Tx-History And `wallet.asset.*` Authority Convergence

**Goal:**

- Keep one append-only `wallet_<stem>_tx_history.jsonl` authority lane for live wallet tx history.
- Project public lifecycle DTOs and wallet responses from durable history rows instead of from a second pending-only authority plane.
- Preserve the cash-only `wallet.asset.*` boundary while keeping voucher and right semantics on `wallet.object.*`.

**Closeout note:**

- Closed on the live wallet JSONL store plus RPC projection path. Public `RuntimeTxLifecycle` values now derive from durable row kinds and current folded records, not from a parallel in-memory-only model.
- The append-only row taxonomy now includes `Conflicted` and `AlreadySpent`. Both preserve coarse `TxStatus::Failed` storage semantics while exposing richer runtime lifecycle states.
- Restart/reload preservation is now explicit: lifecycle projection survives reopen by replaying physical JSONL rows, and rollback paths append compensating restore rows instead of deleting or recreating journal authority.
- `wallet.asset.*` remains the cash-only projection namespace. Voucher and right inventory plus package flows stay on `wallet.object.*`, and asset RPC paths continue to reject non-cash aliases instead of absorbing typed object semantics.

**Evidence anchors:**

- `crates/z00z_wallets/src/persistence/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`
- `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
- `crates/z00z_wallets/src/rpc/tx_types.rs`
- `crates/z00z_wallets/src/backup/backup_wire.rs`
- `crates/z00z_wallets/src/rpc/asset_types.rs`
- `crates/z00z_wallets/src/rpc/asset_rpc_server_ops.rs`
- `crates/z00z_wallets/src/rpc/object_rpc.rs`
- `crates/z00z_wallets/src/rpc/test_asset_impl.rs`
- `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
- `crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`

## 0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout

**Goal:**

- Keep one semantic settlement-root authority and one storage facade while backend rollout stays an internal HJMT concern.
- Treat `SettlementStateRoot` as the live public storage root and `SettlementPath` as the live path vocabulary above the backend.
- Keep `backend_root` proof-local only; there is no second public root authority and no live fallback to asset-only path or root terms.

**Closeout note:**

- Closed as normalized/superseded by live settlement root and HJMT backend.
- Evidence anchors:
  - `crates/z00z_storage/src/settlement/root_types.md`
  - `crates/z00z_storage/src/settlement/store.rs`
  - `crates/z00z_storage/src/settlement/hjmt_config.rs`
  - `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`
  - `crates/z00z_storage/tests/test_live_guardrails.rs`

## 

---

## 10. Wallet Receive, Scan, Import, And History Authority Closure

**Goal:**

- Keep one wallet-local receive mutation lane, one durable tx-history
  authority lane, one public lifecycle projection, and one advisory
  request-bound helper boundary.

**Closeout note:**

- `RuntimeTxLifecycle` is the public lifecycle projection type. It is derived
  from durable tx-history JSONL rows plus the folded tx record state instead of
  from a parallel simulator-only or pending-only lane.
- `RuntimeReceiveScanOutcome` is the public receive status outcome type. It
  reports canonical scan results while leaving receive mutation on the existing
  wallet-local authority path.
- Advisory worker evidence remains non-authoritative. The worker tests prove
  normalized worker data can feed the receive lane, but it cannot bypass the
  canonical wallet mutation path or create a second scan authority plane.
- Asset and object authority boundaries remain explicit: cash projections stay
  on `wallet.asset.*`, typed object inventory stays on `wallet.object.*`, and
  tx-history remains the wallet-owned lifecycle journal.
- The request-bound helper remains advisory only. `recv_range_with_inbox(...)`
  may validate and prioritize request metadata, but it must re-enter the
  canonical receive path instead of owning persistence, final mutation, or
  consensus truth.
- Phase `21` simulator evidence is downstream proof only. It binds receive,
  import, history, restart, and publication digests back to the existing code
  owners without making the simulator the owner of wallet behavior.

**Evidence anchors:**

- `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
- `crates/z00z_wallets/src/rpc/chain_types.rs`
- `crates/z00z_wallets/src/services/wallet_actions_receive.rs`
- `crates/z00z_wallets/tests/test_remote_scan_worker.rs`
- `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- `crates/z00z_wallets/tests/test_e2e_req_flow.rs`
- `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`

## 14. Wallet Scan Orchestration And Runtime Scan Status

**Goal:**

- Keep one canonical wallet receive lane and one public scan-status DTO.
- Keep remote worker evidence advisory, keep asset/cursor persistence atomic,
  and project deterministic receive outcomes without inventing a second scan
  authority plane.

**Closeout note:**

- `WalletService::recv_range_authoritative(...)` remains the only wallet-local
  asset-plus-cursor mutation lane. `recv_range_with_worker(...)` and
  `recv_range_from_worker(...)` only feed normalized worker evidence into that
  same authority path.
- `RuntimeScanStatus` now carries optional `last_receive_outcome` instead of a
  second scan status type. `RuntimeReceiveScanOutcome` stays snake-case
  serialized and covers `Scanned`, `Resumed`, `NoHit`, `ImportedHit`,
  `WorkerEvidenceRejected`, `CursorConflict`, and `UnsupportedVersion`.
- Receive outcome projection is deterministic on the canonical lane:
  origin no-hit scans report `NoHit`, resumed no-hit scans report `Resumed`,
  scans that import new assets report `ImportedHit`, accepted duplicate/no-new
  import scans report `Scanned`, worker validation rejects report
  `WorkerEvidenceRejected`, cursor or persistence conflicts report
  `CursorConflict`, and unsupported receive-version failures map to
  `UnsupportedVersion`.
- Atomicity is proven on the live receive path with a test-only pre-persist
  hook that aborts before commit. Failure leaves both owned assets and cursor
  unchanged, and retry succeeds on the same canonical path.

**Evidence anchors:**

- `crates/z00z_wallets/src/rpc/chain_types.rs`
- `crates/z00z_wallets/src/services/app_chain_network.rs`
- `crates/z00z_wallets/src/services/wallet_actions_receive.rs`
- `crates/z00z_wallets/src/services/wallet_service_core.rs`
- `crates/z00z_wallets/src/services/test_wallet_service.rs`
- `crates/z00z_wallets/tests/test_remote_scan_worker.rs`
- `crates/z00z_wallets/src/redb_store/owned_assets.rs`
- `crates/z00z_wallets/src/rpc/chain_rpc_impl.rs`

---

--



## 9. Storage Claim-Root And Checkpoint Authority Closure

**Goal:**

- Keep claim-source root, checkpoint statement, proof payload, seal, and reload validation on one storage-owned path.

**Closeout table:**

| Criterion                                             | Evidence anchors                                             | Status   |
| ----------------------------------------------------- | ------------------------------------------------------------ | -------- |
| claim source root is bound into proof material        | `crates/z00z_storage/tests/test_claim_source_proof.rs`; `crates/z00z_storage/src/checkpoint/artifact_stmt.rs` | `Closed` |
| checkpoint statement includes claim root              | `crates/z00z_storage/src/checkpoint/artifact_stmt.rs`; `crates/z00z_storage/src/checkpoint/artifact_final.rs` | `Closed` |
| checkpoint proof is statement-bound                   | `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`; `crates/z00z_storage/tests/test_checkpoint_finalization.rs` | `Closed` |
| checkpoint store seals through one canonical path     | `crates/z00z_storage/src/checkpoint/store.rs`; `crates/z00z_storage/tests/test_checkpoint_store_api.rs` | `Closed` |
| checkpoint link reload revalidates artifact identity  | `crates/z00z_storage/src/checkpoint/store.rs`; `crates/z00z_storage/tests/test_checkpoint_store_api.rs`; `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs` | `Closed` |
| tampered claim-root or checkpoint payload is rejected | `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`; `crates/z00z_storage/tests/test_checkpoint_finalization.rs`; `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs` | `Closed` |

## 