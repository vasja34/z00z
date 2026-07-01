# 044-TODO

Self-contained backlog contract:

- this file is the canonical execution artifact for Phase 044;
- this file embeds the required design decisions, requirements, acceptance
  criteria, test scenarios, tx-history correction overlay, and closeout gates;
- no external phase design document is required to implement or review the
  tasks below;
- if another document disagrees with this backlog, pause and update this file
  before implementation continues.

Execution rules:

- execute this file in order unless a dependency note says otherwise;
- keep Phase 044 wallet-centered and consume Phase 043 seams instead of
  duplicating assembler, verifier, receive persistence, output builder, or
  checkpoint behavior;
- extend the live authority surface embedded below before adding a new
  helper module, data store, admission path, or lifecycle vocabulary;
- if execution discovers a missing Phase 043 prerequisite, fix the Phase 043
  seam first and then consume it here;
- keep simulated admission explicit and trait-backed; do not describe it as
  real chain broadcast, consensus, or public proof-of-knowledge closure;
- keep sender and receiver submission role-neutral for the same canonical tx
  bytes and tx hash;
- do not introduce `Validated` as a wallet asset state or persisted tx status;
- store tx packages outside `.wlt`;
- make `wallet_<stem>_tx_history.jsonl` the canonical live tx-history store for
  that wallet stem;
- do not create a new broad database for Phase 044 tx packages;
- do not use `wallet_<stem>_tx_history/<tx_hash>.json` as the live tx store;
- backup and restore must preserve the live JSONL history and exact tx package
  bytes instead of rebuilding history from per-transaction JSON files;
- do not modify `crates/z00z_crypto/tari/**`;
- before starting any numbered task, read its embedded requirement anchors and
  task-local requirement summary.

## 🎯 Decision Summary

The execution baseline for Phase 044 is:

1. wallet assets are spendable only in `Available` state;
2. reservation, exported, pending spend, final spent, pending change, pending
   receive, quarantine, and rollback states are distinct lifecycle semantics;
3. tx build and send must reuse `AssetSelectorImpl`, `TxAssemblerImpl`,
   `CanonicalSpendProofBackend`, and `verify_full_tx_package(...)`;
4. fee and change are real role-tagged outputs, not scalar-only metadata;
5. tx details, pending lists, history, and balance are views over wallet
   journal data, persisted tx bytes, and lifecycle rows;
6. report-only receive stays non-persistent, while final receiver ownership
   still routes through `recv_route(..., ReceiveNext::PersistClaim)`;
7. portable package export and receiver import preserve canonical tx bytes,
   tx hash identity, chain id, version, checksum, and redacted metadata only;
8. sender-side and receiver-side submission call one `WalletTxAdmitter` path
   and converge idempotently by tx hash;
9. simulated confirmation produces typed checkpoint evidence, and wallet
   finalization happens only through reconciliation against that evidence;
10. final spendable value remains `Available`, final consumed input remains
    `Spent`, and final tx storage remains `Confirmed`;
11. cancellation releases inputs only when admission or policy proves release
    is safe;
12. all new file I/O, serialization, time, logging, metrics, config, and RNG
    boundaries must use `z00z_utils` or existing project abstractions;
13. closure is wallet-level lifecycle closure only, not trustless public
    proof-of-knowledge or production consensus closure.
14. `.wlt` stores wallet snapshot, identity, encrypted seed material, and
    wallet-level restore state only; it must not become the tx package history
    store.
15. `wallet_<stem>_tx_history.jsonl` is the canonical append-ordered live
    wallet tx-history file, not a derivative backup artifact.
16. every package-bearing tx-history entry stores the exact canonical tx
    package bytes and a hash of those bytes, including encrypted package fields
    and encrypted asset data that are part of the package.
17. backup reads and preserves existing JSONL bytes plus a manifest; restore
    writes JSONL back instead of expanding records into per-tx JSON files.
18. legacy `wallet_<stem>_tx_history/` directories are migration input only and
    must not be used for new writes.

## 🔗 Dependency Chain

Execution dependency chain:

1. `044-01` spec coverage and no-duplicate lock
2. `044-02` wallet asset ledger and reservation layer
3. `044-03` sender build and send lifecycle
4. `044-04` tx journal, details, pending lists, and history
5. `044-04A` canonical JSONL live tx-history authority and path contract
6. `044-04B` JSONL-backed tx storage and folded reads
7. `044-04C` backup, restore, and forensic JSONL preservation
8. `044-04D` legacy per-tx JSON migration and guards
9. `044-05` portable package export, import, and role-neutral submission
10. `044-06` admission and confirmation boundary
11. `044-07` storage-backed wallet reconciliation
12. `044-08` receiver pending, report-only, and finalization behavior
13. `044-09` balance and user-facing lifecycle views
14. `044-10` regression matrix and source-shape guards

Hard dependencies:

- `044-02` depends on `044-01`
- `044-03` depends on `044-02`
- `044-04` depends on `044-02` and `044-03`
- `044-04A` depends on `044-04`
- `044-04B` depends on `044-04A`
- `044-04C` depends on `044-04B`
- `044-04D` depends on `044-04B`
- `044-05` depends on `044-03`, `044-04`, `044-04A`, and `044-04B`
- `044-06` depends on `044-03`, `044-04`, and `044-05`
- `044-07` depends on `044-02`, `044-04`, and `044-06`
- `044-08` depends on `044-04`, `044-05`, `044-06`, and `044-07`
- `044-09` depends on `044-02`, `044-04`, `044-07`, and `044-08`
- `044-10` depends on `044-01` through `044-09`, including `044-04A`
  through `044-04D`

## 🗂️ File-First Implementation Order

Edit order by file cluster:

1. `.planning/phases/044-wallet-assets/044-coverage.md`
2. `crates/z00z_wallets/src/persistence/assets/asset_storage.rs`
3. `crates/z00z_wallets/src/persistence/assets/asset_storage_impl.rs`
4. `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_assets.rs`
5. `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
6. `crates/z00z_wallets/src/db/redb/store/redb_wallet_store_objects.rs`
7. `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`
8. `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs`
9. `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
10. `crates/z00z_wallets/src/tx/selection/asset_selector.rs`
11. `crates/z00z_wallets/src/tx/verify/tx_wire_types.rs`
12. `crates/z00z_wallets/src/tx/verify/tx_verifier.rs`
13. `crates/z00z_wallets/src/tx/verify/tx_verifier_helpers.rs`
14. `crates/z00z_wallets/src/tx/state/state_resolved_input.rs`
15. `crates/z00z_wallets/src/tx/spend/spend_verification.rs`
16. `crates/z00z_wallets/src/tx/tx_assembler.rs`
17. `crates/z00z_wallets/src/tx/output/output_flow.rs`
18. `crates/z00z_wallets/src/stealth/output/output_build.rs`
19. `crates/z00z_wallets/src/stealth/output/output_validator.rs`
20. `crates/z00z_wallets/src/tx/proof/spend_proof_backend.rs`
21. `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs`
22. `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
23. `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
24. `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
25. `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
26. `crates/z00z_wallets/src/backup/crypto/backup_wire.rs`
27. `crates/z00z_wallets/src/backup/export/backup_exporter_impl.rs`
28. `crates/z00z_wallets/src/backup/import/backup_importer_impl.rs`
29. `crates/z00z_wallets/src/backup/import/backup_importer.rs`
30. `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`
31. `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_history.rs`
32. `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
33. `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
34. `crates/z00z_wallets/src/chain/client/chain_client.rs`
35. `crates/z00z_wallets/src/chain/client/chain_client_impl.rs`
36. `crates/z00z_wallets/src/chain/broadcast/broadcast.rs`
37. `crates/z00z_wallets/src/chain/broadcast/broadcast_impl.rs`
38. `crates/z00z_wallets/src/tx/state/state_update.rs`
39. `crates/z00z_wallets/src/persistence/scans/storage.rs`
40. `crates/z00z_wallets/src/persistence/scans/storage_impl.rs`
41. `crates/z00z_storage/src/checkpoint/build.rs`
42. `crates/z00z_storage/src/checkpoint/exec_input.rs`
43. `crates/z00z_storage/src/checkpoint/artifact_types.rs`
44. `crates/z00z_storage/src/assets/store_internal/store_query.rs`
45. `crates/z00z_storage/src/assets/store_internal/store_rows.rs`
46. `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`
47. wallet-side focused tests
48. simulator and release-style validation gates
49. `.planning/phases/044-wallet-assets/044-SUMMARY.md`

## ✅ Validation Matrix

This table proves that the implementation-driving instructions embedded in this
backlog remain traceable section by section.

| Embedded section or IDs | Required theme | TODO coverage | Status |
| --- | --- | --- | --- |
| `Purpose`, `Source Evidence`, `Live Authority Surface` | wallet-centered lifecycle closure over existing selectors, assemblers, verifiers, storage, receive, and checkpoint paths | execution rules; `044-01`; all task pre-read blocks | Validated mapped |
| `Scope`, `Out Of Scope`, `Risk Watchpoints`, `No Logical Weak Spots` | forbid vendor edits, duplicate authorities, fake broadcast, public proof overclaiming, raw secret leakage, and `Validated` state drift | execution rules; `Explicit Phase Boundary`; `044-10`; `Completion Gate` | Mapped as release guardrail |
| `D-044-001`, `D-044-002`, `D-044-013`, `Definitions`, `Sender Input State Machine`, `PH44-LEDGER`, `PH44-CANCEL` | asset lifecycle states, atomic reservation, safe release, exported lock behavior, final spent semantics | `044-02`; `044-03`; `044-07`; `044-10` | Validated mapped |
| `D-044-003`, `D-044-004`, `PH44-SEND`, `Sender Output State Machine` | sender build creates real recipient, change, and fee outputs and verifies package before admission | `044-03`; `044-04`; `044-10` | Validated mapped |
| `D-044-011`, `D-044-012`, `PH44-OFFLINE`, `Portable Tx Package`, `Offline Package Flow` | canonical tx bytes are portable; sender and receiver submission are equal and idempotent | `044-05`; `044-06`; `044-10` | Validated mapped |
| `D-044-007`, `PH44-ADMIT`, `Admission Trait`, `Simulated Validator Boundary` | simulated admission is explicit, trait-backed, deterministic, and replaceable by real chain implementation later | `044-06`; `044-10` | Validated mapped |
| `PH44-RECONCILE`, `Confirmation Receipt And Reconciler`, `Phase Gate 4A` | wallet finalization scans typed storage/checkpoint evidence and applies idempotent pending-to-final transitions | `044-07`; `044-10` | Validated mapped |
| `D-044-005`, `D-044-006`, `PH44-RECEIVE`, `Receiver State Machine`, `Receiver Flow` | report-only receive stays non-persistent; receiver finalization uses canonical persist-claim route | `044-08`; `044-10` | Validated mapped |
| `D-044-010`, `PH44-BALANCE`, `Phase Gate 6` | balance keeps compatibility fields while deriving available and pending from lifecycle state | `044-09`; `044-10` | Validated mapped |
| `D-044-008`, `PH44-HISTORY`, `Tx Journal Record`, `Wallet Tx Status` | details, history, pending lists, receipts, origin, submitter role, and tx bytes derive from a richer journal | `044-04`; `044-05`; `044-06`; `044-07`; `044-09` | Validated mapped |
| `PH44-DRIFT`, `Required Commands`, `Required Test Scenarios`, `Acceptance Criteria` | source-shape guards, narrow-first validation, and all AC/T scenarios are closure obligations | `044-01`; `044-10`; `Completion Gate` | Mapped as validation contract |
| `Required Outputs`, `Completion Definition` | coverage, tests, updated RPC behavior, admission, reconciliation, summary, and honest closeout language must land | all tasks; `Completion Gate` | Validated mapped |
| `Patch Correction / P-044-001` through `P-044-012` | JSONL live tx-history store, exact package preservation, backup/restore JSONL preservation, legacy per-tx migration, and source-shape guards | `044-04A`; `044-04B`; `044-04C`; `044-04D`; `044-10`; `Completion Gate` | Mapped as corrected storage contract |
| `AC-044-001` through `AC-044-008` | Alice reservation, cancel, submit, change, fee, verification failure, admission failure, and confirmation behavior | `044-02`; `044-03`; `044-06`; `044-07`; `044-10` | Mapped as sender criteria |
| `AC-044-009` through `AC-044-012` | Bob report-only, persist-claim, pending receive, and duplicate finalization behavior | `044-08`; `044-10` | Mapped as receiver criteria |
| `AC-044-013` through `AC-044-015` | pending balance, populated details, and explicit simulated admission boundary | `044-04`; `044-06`; `044-09`; `044-10` | Mapped as RPC criteria |
| `AC-044-016` through `AC-044-020` | offline package export/import, role-neutral submit, duplicate submit, safe cancellation, and bad package rejection | `044-05`; `044-06`; `044-10` | Mapped as offline criteria |
| `AC-044-021` through `AC-044-024` | storage-backed reconciliation, idempotent finalization, and conflict fail-closed behavior | `044-07`; `044-08`; `044-10` | Mapped as reconciliation criteria |

## 📚 Embedded Requirement Corpus

This section embeds the normative Phase 044 requirements needed for
implementation. Do not require another phase document to understand these
requirements.

Purpose:

- close the sender and receiver wallet asset lifecycle around real tx package
  bytes;
- reserve sender inputs before tx bytes are exposed;
- preserve tx packages for history, details, forensic replay, backup, restore,
  offline import, and role-neutral submission;
- keep report-only receive separate from receiver claim persistence;
- use simulated admission and storage-backed reconciliation honestly without
  claiming real chain consensus or public proof-of-knowledge closure.

PH44 requirement clauses:

| ID | Requirement |
| --- | --- |
| `PH44-LEDGER` | Wallet asset ledger must distinguish `Available`, `Reserved`, `Exported`, `ClaimPending`, `PendingSpend`, `Spent`, `PendingChange`, `PendingReceive`, `ReorgPending`, `Quarantined`, and `Dropped`; only `Available` is spendable. |
| `PH44-SEND` | Sender build/send must perform selection, reservation, output construction, tx package assembly, verification, journal persistence, and admission through the canonical lifecycle engine; `BuiltTxStub` must not be returned as live behavior. |
| `PH44-OFFLINE` | A verified tx package must be exportable as portable canonical tx bytes; receiver import must decode, verify, validate chain/version, scan owned outputs, and write only pending receiver state before confirmation. |
| `PH44-ADMIT` | Real chain absence must be represented by one explicit trait-backed simulated admission adapter, not RPC-local fake success. |
| `PH44-RECONCILE` | Wallet finalization must scan typed storage/checkpoint evidence and transition pending rows idempotently to final states only when evidence matches journal expectations. |
| `PH44-RECEIVE` | Report-only receive must not mutate claims, tx history, pending rows, or balances; receiver finalization must use `recv_route(..., ReceiveNext::PersistClaim)`. |
| `PH44-BALANCE` | Balance must derive `available` and `pending` from wallet lifecycle rows; `pending = 0` is forbidden when pending lifecycle rows exist. |
| `PH44-HISTORY` | Tx details, pending lists, and history must be backed by real journal data, role-tagged rows, receipts, tx bytes, and lifecycle status. |
| `PH44-CANCEL` | Cancellation may release inputs only before admission or when no-admission evidence or fail-closed policy proves release is safe; exported packages keep sender inputs non-spendable until resolved. |
| `PH44-DRIFT` | Phase 044 must not duplicate assembler, verifier, tx schema, receive persistence, wallet asset authority, chain broadcast, or scan-cursor proof semantics. |

Patch correction clauses:

| ID | Requirement |
| --- | --- |
| `P-044-001` | `.wlt` stores wallet snapshot, identity, encrypted seed material, and wallet-level restore state only; it must not store tx packages or unbounded tx history. |
| `P-044-002` | `wallet_<stem>_tx_history.jsonl` is the canonical live tx-history store for that wallet stem, not merely a backup export or sidecar generated from another live store. |
| `P-044-003` | Phase 044 must not introduce a new broad database for tx packages. |
| `P-044-004` | New writes must not use `wallet_<stem>_tx_history/<tx_hash>.json` or any per-transaction JSON directory as the live tx store. |
| `P-044-005` | Every package-bearing JSONL entry must preserve the exact canonical tx package bytes as built or imported, including encrypted package fields and encrypted asset data that are part of the package. |
| `P-044-006` | Decrypted wallet secrets, private blindings, plaintext seed material, and non-package private wallet state must never be added to tx-history JSONL. |
| `P-044-007` | `TxStorageImpl` must be refactored into a single JSONL-file store whose reads fold rows by `tx_hash` into the current view. |
| `P-044-008` | Status updates append new rows; physical deletion is replaced by forensic tombstone rows unless a future explicit purge policy is approved. |
| `P-044-009` | JSONL writes must be atomic through a temporary sibling file and rename, guarded by a per-wallet file lock or existing wallet session lock. |
| `P-044-010` | Backup must read existing live JSONL bytes, validate them, and include the exact JSONL bytes plus manifest in the encrypted forensic backup payload; `Vec<TxRecord>` is a derived view, not the authority. |
| `P-044-011` | Restore must write archived JSONL bytes back to the restored wallet stem's live JSONL path; it must not reconstruct history from extracted records or expand rows into per-tx JSON files. |
| `P-044-012` | Existing `wallet_<stem>_tx_history/` directories are legacy migration input only; migration preserves tx bytes, writes `Migrated` JSONL entries, and does not delete legacy directories automatically. |

Patch non-negotiable invariants:

- .wlt must not become the tx package history store.
- wallet_<stem>_tx_history.jsonl must not be a derivative backup artifact.
- Phase 044 must not introduce a broad new wallet database for tx packages.
- Tx package bytes must be stored exactly and remain recoverable.
- Backup must preserve the live JSONL history, not synthesize it from private
  implementation files.
- Restore must write the live JSONL history, not expand it into per-tx JSON
  files.
- Report-only receive must still not mutate claimed assets or balances.
- Receiver import must still verify package bytes before writing pending
  receiver state.
- Sender and receiver submission must still converge by the same tx hash and
  admission path.

Live JSONL entry contract:

```rust
pub struct WalletTxHistoryJsonlEntry {
    pub schema_version: u32,
    pub wallet_stem: String,
    pub sequence: u64,
    pub recorded_at_ms: u64,
    pub tx_hash: String,
    pub entry_kind: WalletTxHistoryEntryKind,
    pub record_hash: [u8; 32],
    pub tx_bytes_hash: [u8; 32],
    pub previous_entry_hash: Option<[u8; 32]>,
    pub entry_hash: [u8; 32],
    pub record: TxRecord,
}

pub enum WalletTxHistoryEntryKind {
    Created,
    Imported,
    Exported,
    Submitted,
    Admitted,
    Confirmed,
    Failed,
    Cancelled,
    Tombstoned,
    Migrated,
}
```

Live JSONL rules:

- `record.tx_bytes` must contain complete canonical tx package bytes when a
  package exists;
- `tx_bytes_hash` must hash the exact bytes in `record.tx_bytes`;
- `record_hash` must hash the serialized `TxRecord`;
- `entry_hash` must cover the entry without `entry_hash` plus
  `previous_entry_hash`;
- `previous_entry_hash` links to the previous physical JSONL row for the same
  wallet history file;
- `sequence` is monotonically increasing within one wallet history file;
- `record.tx_hash` and top-level `tx_hash` must match;
- malformed rows fail closed unless a future recovery mode explicitly allows
  partial salvage.

Core wallet data contracts:

Exact Rust placement may vary, but these contract names and meanings must not
drift. If implementation uses equivalent names, `044-coverage.md` must map the
equivalent type back to the contract name below.

```rust
pub struct WalletAssetLedgerRow {
    pub wallet_id: String,
    pub asset_id_hex: String,
    pub serial_id: u32,
    pub asset_class: String,
    pub amount: u64,
    pub state: WalletAssetState,
    pub tx_id: Option<String>,
    pub reservation_id: Option<String>,
    pub output_role: Option<TxOutRole>,
    pub block_height: Option<u64>,
    pub state_root_hex: Option<String>,
}

pub struct WalletTxAdmissionRequest {
    pub tx_hash_hex: String,
    pub tx_bytes: Vec<u8>,
    pub chain_id: String,
    pub submitter_role: WalletTxSubmitterRole,
    pub idempotency_key: String,
}

pub struct WalletReconcileRequest {
    pub wallet_id: String,
    pub chain_id: String,
    pub pending_tx_scope: Vec<String>,
}

pub struct WalletReconcileReport {
    pub confirmed_tx_hashes: Vec<String>,
    pub unchanged_pending_tx_hashes: Vec<String>,
    pub failed_or_quarantined_tx_hashes: Vec<String>,
    pub already_applied_tx_hashes: Vec<String>,
}
```

Core wallet data contract rules:

- `WalletAssetLedgerRow` stores wallet-local lifecycle state and must fail
  closed on unknown state values.
- `WalletTxAdmissionRequest` must carry tx hash, canonical tx bytes, chain id,
  submitter role, and idempotency key; submitter role is audit data only.
- `WalletReconcileRequest` must identify the wallet, chain/checkpoint scope,
  and pending/admitted tx rows to reconcile.
- `WalletReconcileReport` must distinguish confirmed transitions, unchanged
  pending rows, failures or quarantines, and idempotent already-applied effects.

Acceptance criteria:

| ID | Criterion |
| --- | --- |
| `AC-044-001` | Given Alice has one available asset, when Alice builds a tx that spends it, then the selected asset becomes `Reserved` and cannot be selected by a second tx. |
| `AC-044-002` | Given Alice cancels a tx before admission, when cancellation succeeds, then all reserved inputs return to `Available` and pending outputs are dropped. |
| `AC-044-003` | Given Alice submits a verified tx, when admission accepts it, then inputs become `PendingSpend` and tx details show non-empty input and output rows. |
| `AC-044-004` | Given Alice submits a tx with change, when the tx is built, then one `TxOutRole::Change` output exists and is sender-owned. |
| `AC-044-005` | Given Alice submits a tx with a non-zero fee, when the tx is built, then one or more `TxOutRole::Fee` coin outputs sum to the declared fee. |
| `AC-044-006` | Given a fee mismatch exists, when the wallet verifies the package, then the tx is rejected and reservations are released. |
| `AC-044-007` | Given simulated confirmation succeeds, when checkpoint semantics consume inputs and create outputs, then Alice inputs become `Spent`, Alice change becomes `Available`, and tx status becomes `Confirmed`. |
| `AC-044-008` | Given simulated admission fails before confirmation, when the failure is recorded, then Alice inputs become `Available` and tx status becomes `Failed`. |
| `AC-044-009` | Given Bob calls report-only receive, when an owned output is detected, then no claimed asset, balance, or pending receive row is persisted. |
| `AC-044-010` | Given Bob imports or scans a confirmed owned output, when `recv_route(..., PersistClaim)` succeeds, then the asset becomes `Available` exactly once. |
| `AC-044-011` | Given Bob sees a verified but unconfirmed owned tx output, when the package is accepted but checkpoint confirmation is pending, then the output is `PendingReceive` and not spendable. |
| `AC-044-012` | Given a duplicate receiver output is processed twice, when the second finalization runs, then it fails closed without double-counting balance. |
| `AC-044-013` | Given balance is requested during Alice pending spend, then `available` excludes reserved inputs and `pending` includes the non-spendable lifecycle value according to `PH44-BALANCE`. |
| `AC-044-014` | Given tx details are requested for a built, submitted, failed, cancelled, or confirmed tx, then details contain journaled inputs, outputs, amount, fee, status, and evidence fields for that state. |
| `AC-044-015` | Given real chain submit remains unimplemented, when wallet send or broadcast runs, then the only acceptance path is the explicit simulated admission adapter. |
| `AC-044-016` | Given Alice exports a verified portable tx package before admission, when Bob receives it offline, then Bob can import the package and see the receiver-owned output as non-spendable pending state. |
| `AC-044-017` | Given Bob imports Alice's portable tx package, when Bob submits it, then admission uses the same `WalletTxAdmitter` path and tx hash as Alice sender submission. |
| `AC-044-018` | Given Alice and Bob both submit the same tx bytes, when the second submission reaches admission, then it is idempotent for the same tx hash and does not duplicate spend or receive state. |
| `AC-044-019` | Given Alice exports a tx package and then tries to cancel locally, when no no-admission proof exists, then selected inputs remain non-spendable and cannot be returned to `Available`. |
| `AC-044-020` | Given Bob imports a tampered, wrong-chain, malformed, or no-owned-output package, when import validation runs, then the wallet rejects it without changing claimed assets, pending rows, or available balance. |
| `AC-044-021` | Given a pending tx has not appeared in storage checkpoint evidence, when reconciliation scans storage, then all wallet asset rows stay pending and tx status remains non-confirmed. |
| `AC-044-022` | Given storage evidence proves Alice's selected inputs were spent and change output was created, when reconciliation runs, then Alice inputs become `Spent`, Alice change becomes `Available`, and the tx becomes `Confirmed`. |
| `AC-044-023` | Given storage evidence proves Bob's receiver-owned output was created, when reconciliation runs, then Bob's output is persisted through `recv_route(..., PersistClaim)` and becomes `Available` exactly once. |
| `AC-044-024` | Given storage evidence conflicts with the tx journal, when reconciliation runs, then the wallet fails closed, records typed failure or quarantine evidence, and does not increase `available`. |

Required test scenarios:

| ID | Scenario | Required result |
| --- | --- | --- |
| `T-044-001` | same input selected by two Alice tx builds | second build fails before tx bytes are exposed |
| `T-044-002` | Alice tx build then cancel before admission | reservation releases and balance returns to pre-build state |
| `T-044-003` | Alice tx build with exact input | no change output is created |
| `T-044-004` | Alice tx build with oversized input | one sender-owned change output is created |
| `T-044-005` | Alice tx build with fee | fee output exists, is coin class, and declared fee matches verifier rule |
| `T-044-006` | malformed package after reservation | verifier rejects and reservation releases |
| `T-044-007` | simulated admission accepted | tx journal records admission receipt and pending spend |
| `T-044-008` | simulated confirmation accepted | inputs spent, change available, tx confirmed |
| `T-044-009` | simulated confirmation rejected | deterministic rollback or retry state with no double spend |
| `T-044-010` | Bob report-only receive | detection returned, no persistence and no balance change |
| `T-044-011` | Bob persist claim after confirmation | claimed asset available exactly once |
| `T-044-012` | Bob duplicate finalization | duplicate rejected with no double balance |
| `T-044-013` | tx details for pending tx | non-empty inputs and outputs with roles |
| `T-044-014` | balance during pending outgoing | available excludes reserved; pending reflects lifecycle state |
| `T-044-015` | source-shape guard for stubs | no live `BuiltTxStub`, `pending = 0`, empty details, or RPC-local fake success remains |
| `T-044-016` | Alice exports portable package and Bob imports offline | Bob records receiver-owned output as pending and non-spendable |
| `T-044-017` | Bob submits imported package | same admission trait, tx hash, and receipt path as sender submission |
| `T-044-018` | Alice and Bob duplicate-submit same package | second submission is idempotent and cannot duplicate spend or receive rows |
| `T-044-019` | sender cancellation after external export | inputs stay locked unless no-admission proof exists |
| `T-044-020` | tampered or wrong-chain offline package import | import rejects with no balance or claim mutation |
| `T-044-021` | reconciliation scan with no matching storage evidence | pending rows and tx status remain unchanged |
| `T-044-022` | reconciliation scan with matching sender spend/change evidence | inputs spent, change available, tx confirmed |
| `T-044-023` | reconciliation scan with matching receiver created-output evidence | receiver claim persists through `PersistClaim` and becomes available once |
| `T-044-024` | reconciliation scan with wrong root or missing output evidence | typed failure/quarantine with no available balance increase |

Patch-specific required test scenarios:

| ID | Scenario | Required result |
| --- | --- | --- |
| `PT-044-001` | wallet path contract after Phase 044 tx-history correction | `wallet_<stem>_tx_history.jsonl` is the live tx-history path and no new live `wallet_<stem>_tx_history/` directory is created |
| `PT-044-002` | tx-history write with full package bytes | JSONL row contains exact `TxRecord.tx_bytes`, `tx_bytes_hash`, `record_hash`, sequence, entry hash, and previous-entry linkage |
| `PT-044-003` | tx status update by appending new rows | update appends a new JSONL row and folded reads return the latest status without mutating prior rows |
| `PT-044-004` | tx delete request | delete appends a `Tombstoned` row and folded reads hide the tx by default without erasing package bytes |
| `PT-044-005` | malformed JSONL row or hash mismatch | corrupted `record_hash`, `tx_bytes_hash`, `entry_hash`, sequence, or previous-entry linkage fails closed |
| `PT-044-006` | concurrent JSONL writes and concurrent appends | per-wallet lock or session lock prevents duplicate sequence numbers, partial rows, and interleaved writes |
| `PT-044-007` | backup creation with live JSONL history | backup reads live JSONL bytes, validates them, stores exact bytes plus manifest, and does not collect normal history from per-tx JSON |
| `PT-044-008` | backup JSONL sidecar copy | sidecar, when requested, is a verified copy of live JSONL bytes rather than a regenerated export |
| `PT-044-009` | restore `WalletOnly` mode | wallet snapshot restores without inventing tx-history rows |
| `PT-044-010` | restore `WalletPlusHistory` mode | wallet snapshot restores and archived JSONL bytes are written to the restored wallet stem's live JSONL path |
| `PT-044-011` | restore `TxHistoryOnly` mode | tx-history JSONL is restored for forensic inspection and no `.wlt` is created |
| `PT-044-012` | restore tampering | tampered JSONL bytes, manifest, row hashes, tx bytes hashes, sequence, wallet identity, or chain identity fail closed without wallet mutation |
| `PT-044-013` | legacy per-tx JSON migration | legacy records migrate to `Migrated` JSONL rows with exact package bytes and deterministic ordering |
| `PT-044-014` | legacy directory preservation | legacy directory is preserved: migration does not delete or mutate `wallet_<stem>_tx_history/` automatically |
| `PT-044-015` | JSONL already exists with legacy directory present | JSONL is authoritative and legacy rows are not merged automatically |
| `PT-044-016` | source-shape guard for old tx-history model | guards fail if live code writes one JSON file per tx hash or backup collects normal history from per-tx JSON |

## 🔎 Full Source Coverage Index

Source evidence coverage:

| Source ID | TODO coverage | Coverage meaning |
| --- | --- | --- |
| `EV-044-001` | `044-02`; `044-03`; `044-10` | selector reuse, input reservation, and no double selection |
| `EV-044-002` | `044-03`; `044-04`; `044-10` | reference-only inputs, resolved metadata, and output role journaling |
| `EV-044-003` | `044-03`; `044-06`; `044-10` | package verifier before admission or submitted state |
| `EV-044-004` | `044-03`; `044-10` | fee output class and declared-fee equality |
| `EV-044-005` | `044-02`; `044-07`; `044-09` | lifecycle vocabulary promoted into wallet runtime state |
| `EV-044-006` | `044-03`; `044-06`; `044-07` | resolved inputs for membership and conservation |
| `EV-044-007` | `044-06`; `044-07`; `044-10` | checkpoint semantics for consumed inputs and created outputs |
| `EV-044-008` | `044-03`; `044-06`; `044-10` | canonical proof backend reuse and no parallel proof representation |
| `EV-044-009` | `044-01`; `044-03`; `Completion Gate` | wallet-local lifecycle closure without public proof-of-knowledge overclaim |
| `EV-044-010` | `044-03`; `044-10` | live send RPC rewired to the wallet tx lifecycle engine |
| `EV-044-011` | `044-03`; `044-04`; `044-06`; `044-10` | build, broadcast, and details stub removal |
| `EV-044-012` | `044-04`; `044-10` | richer tx journal beyond shallow RPC tx metadata |
| `EV-044-013` | `044-04`; `044-05`; `044-07` | persistent tx bytes retained as canonical package storage |
| `EV-044-014` | `044-02`; `044-08`; `Completion Gate` | asset reservation must not fork receive persistence authority |
| `EV-044-015` | `044-02`; `044-07`; `044-09` | pending/reserved state added without abusing confirmed spent state |
| `EV-044-016` | `044-09`; `044-10` | balance pending value derived from lifecycle rows |
| `EV-044-017` | `044-03`; `044-08`; `044-10` | asset send routes through tx lifecycle; receive stays report-only |
| `EV-044-018` | `044-07`; `044-08`; `044-10` | receiver finalization preserves `recv_range` and `PersistClaim` |
| `EV-044-019` | `044-02`; `044-08`; `044-10` | report-only and persist-claim remain distinct |
| `EV-044-020` | `044-03`; `044-10` | recipient and change outputs use approved builders |
| `EV-044-021` | `044-06`; `044-10` | chain and broadcast stubs replaced by explicit admission trait |
| `EV-044-022` | `044-06`; `044-07`; `044-10` | receipts compatible with storage execution and claim-nullifier rows |
| `EV-044-023` | `044-04`; `044-05`; `044-06` | import/export reuses tx bytes and one lifecycle entrypoint |
| `EV-044-024` | `044-04`; `044-07`; `044-10` | storage finalizes txs as `Confirmed`, not `Validated` |
| `EV-044-025` | `044-02`; `044-07`; `044-09` | final spendability remains `Available`, pending state remains separate |
| `EV-044-026` | `044-07`; `044-10` | reconciliation must not overload scan cursor storage as tx evidence |

Acceptance and required-test coverage:

| Source IDs | TODO coverage | Coverage meaning |
| --- | --- | --- |
| `AC-044-001`, `T-044-001` | `044-02`; `044-03`; `044-10` | second spend build cannot reserve the same input |
| `AC-044-002`, `T-044-002` | `044-02`; `044-10` | pre-admission cancel releases reservations and drops pending outputs |
| `AC-044-003`, `T-044-007`, `T-044-013` | `044-03`; `044-04`; `044-06`; `044-10` | accepted admission records pending spend and populated details |
| `AC-044-004`, `T-044-003`, `T-044-004` | `044-03`; `044-10` | exact input has no change; oversized input creates sender-owned change |
| `AC-044-005`, `T-044-005` | `044-03`; `044-10` | non-zero fee creates coin fee output matching declared fee |
| `AC-044-006`, `T-044-006` | `044-03`; `044-10` | malformed or mismatched package rejects and releases reservation |
| `AC-044-007`, `T-044-008`, `T-044-022` | `044-06`; `044-07`; `044-10` | confirmation consumes inputs, creates change, and confirms tx |
| `AC-044-008`, `T-044-009` | `044-06`; `044-10` | admission or confirmation failure rolls back deterministically |
| `AC-044-009`, `T-044-010` | `044-08`; `044-10` | report-only receive has no persistence or balance mutation |
| `AC-044-010`, `T-044-011` | `044-08`; `044-10` | persist-claim finalization makes receiver asset available once |
| `AC-044-011`, `T-044-016` | `044-05`; `044-08`; `044-10` | imported unconfirmed owned output remains non-spendable pending |
| `AC-044-012`, `T-044-012` | `044-08`; `044-10` | duplicate receiver finalization fails closed |
| `AC-044-013`, `T-044-014` | `044-09`; `044-10` | pending outgoing is excluded from available and included in pending |
| `AC-044-014` | `044-04`; `044-10` | details contain journaled rows and available state evidence |
| `AC-044-015`, `T-044-015` | `044-06`; `044-10` | no fake RPC-local success path remains |
| `AC-044-016` | `044-05`; `044-10` | Alice export and Bob import preserve pending receiver state |
| `AC-044-017`, `T-044-017` | `044-05`; `044-06`; `044-10` | Bob submit uses the same admission trait and tx hash |
| `AC-044-018`, `T-044-018` | `044-05`; `044-06`; `044-10` | duplicate sender/receiver submit is idempotent |
| `AC-044-019`, `T-044-019` | `044-02`; `044-05`; `044-10` | exported sender inputs stay locked without no-admission proof |
| `AC-044-020`, `T-044-020` | `044-05`; `044-10` | bad package import rejects without state mutation |
| `AC-044-021`, `T-044-021` | `044-07`; `044-10` | missing checkpoint evidence leaves rows pending |
| `AC-044-022` | `044-07`; `044-10` | matching sender evidence spends inputs and makes change available |
| `AC-044-023`, `T-044-023` | `044-07`; `044-08`; `044-10` | receiver evidence persists through `PersistClaim` exactly once |
| `AC-044-024`, `T-044-024` | `044-07`; `044-10` | conflicting evidence fails closed with no available increase |

Patch correction coverage:

| Patch ID | TODO coverage | Coverage meaning |
| --- | --- | --- |
| `P-044-001` | execution rules; `044-04A`; `Completion Gate` | `.wlt` does not store tx packages or unbounded tx history |
| `P-044-002` | execution rules; `044-04A`; `044-04B` | `wallet_<stem>_tx_history.jsonl` is the canonical live tx-history store |
| `P-044-003` | execution rules; `044-04A`; `044-10` | no new broad tx package database is introduced |
| `P-044-004` | `044-04A`; `044-04D`; `044-10` | per-tx JSON directory is not a live store |
| `P-044-005` | `044-04A`; `044-04B`; `044-04C` | exact canonical tx package bytes are preserved |
| `P-044-006` | `044-04A`; `044-05`; `044-10` | tx history excludes decrypted wallet secrets and private non-package state |
| `P-044-007` | `044-04B`; `044-04D` | `TxStorageImpl` becomes a JSONL-backed folded read store |
| `P-044-008` | `044-04B`; `044-10` | status changes append rows and delete becomes tombstone |
| `P-044-009` | `044-04B` | JSONL writes are atomic and lock-guarded |
| `P-044-010` | `044-04C`; `044-10` | backup preserves exact JSONL bytes plus manifest |
| `P-044-011` | `044-04C`; `044-10` | restore writes JSONL, not extracted per-tx JSON |
| `P-044-012` | `044-04D`; `044-10` | legacy per-tx directories migrate without deletion |

Patch-specific test coverage:

| Patch test IDs | TODO coverage | Coverage meaning |
| --- | --- | --- |
| `PT-044-001` | `044-04A`; `044-10` | wallet path contract and no live per-tx directory writes |
| `PT-044-002` through `PT-044-006` | `044-04B`; `044-10` | JSONL-backed `TxStorageImpl`, folded reads, tombstones, hashes, and write atomicity |
| `PT-044-007` through `PT-044-012` | `044-04C`; `044-10` | backup/restore exact JSONL preservation, restore modes, and tamper rejection |
| `PT-044-013` through `PT-044-016` | `044-04D`; `044-10` | legacy migration, no deletion, JSONL authority, and source-shape guards |

## 🧪 Existing Test Impact Matrix

Every existing test file below is already tied to one or more Phase 044 feature
areas. Update the listed files where behavior changes, or record a no-change
evidence note in `044-coverage.md` if the existing test remains valid as-is.

Existing-test discovery gate:

- [ ] Before implementation, run a repository test sweep for wallet asset,
  tx-history, JSONL, backup, restore, `.wlt`, RPC tx, scanner, reconciliation,
  and simulator wallet-output terms.
- [ ] Add every feature-linked existing test file found by the sweep to this
  matrix, or record explicit no-change/out-of-scope evidence in
  `044-coverage.md`.
- [ ] Repeat the sweep before completion and prove that no newly discovered
  feature-linked test file is missing from the matrix or from coverage evidence.

| Existing test file | Required Phase 044 update or evidence |
| --- | --- |
| `crates/z00z_wallets/src/persistence/assets/test_asset_storage_impl_suite.rs` | add asset lifecycle state, reservation, release, unknown-state fail-closed, and final-spent distinction coverage |
| `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs` | update inline `TxStorageImpl` unit tests from per-tx JSON directory semantics to the canonical JSONL live store, including append-only writes, folded reads, `get`, `list`, `list_by_status`, `update_status`, tombstones, idempotency, corrupt-tail quarantine, lock behavior, and invalid hash/path traversal rejection |
| `crates/z00z_wallets/src/services/wallet/tests/test_wallet_paths_suite.rs` | add or update wallet path tests so the resolved wallet stem yields `wallet_<stem>.wlt` and the sibling live tx-history path `wallet_<stem>_tx_history.jsonl`, with no live `wallet_<stem>_tx_history/` directory contract |
| `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs` | update wallet stem path contract, report-only versus persist behavior, backup JSONL creation, restore modes, tamper rejection, and no live per-tx directory assumptions |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_wallet_impl_suite.rs` | update wallet RPC create/open/import/export tests for the sidecar JSONL path contract, no tx packages inside `.wlt`, and no assumptions that opening a wallet creates tx-history files without tx events |
| `crates/z00z_wallets/src/tx/selection/test_asset_selector_suite.rs` | ensure selector only sees `Available` candidates and double selection cannot reserve the same input |
| `crates/z00z_wallets/src/tx/selection/test_asset_selector_multi_suite.rs` | ensure multi-input selection respects reservation and exported locks across aggregate selections |
| `crates/z00z_wallets/src/tx/verify/test_tx_verifier_suite.rs` | ensure canonical package verification still rejects malformed, wrong-fee, wrong-chain, tampered, and conflicting packages used by send/import/admission |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs` | update send/build assertions for real selected inputs, reservation before tx bytes exposure, package verification, failure release, and export lock behavior |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs` | update broadcast assertions so admission trait evidence replaces RPC-local fake success and sender/receiver submit are role-neutral |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs` | update pending list assertions for reserved, exported, admitted, pending-spend, pending-change, and pending-receive rows |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_body.rs` | update history/details assertions for non-empty journal-backed inputs/outputs, package bytes linkage, receipts, status, and evidence fields |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs` | preserve and extend cursor, status, amount, and date filters after JSONL folded reads replace the old store shape |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs` | preserve and extend receipt visibility and sort behavior after journal/JSONL details are joined |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_body.rs` | update aggregate tx RPC body tests for real package-backed responses instead of stub details |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_impl_suite.rs` | update tx RPC integration tests for lifecycle delegation, admission evidence, cancellation, and history/details views |
| `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs` | update asset RPC tests for report-only non-persistence, persist-claim finalization, duplicate finalization, and pending balance semantics |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs` | mirror asset RPC coverage where the duplicate suite is active, or document why one suite supersedes the other |
| `crates/z00z_wallets/src/tx/state/test_state_update_suite.rs` | add checkpoint evidence coverage for selected inputs, sender change, receiver outputs, absent evidence, conflict evidence, and idempotent reconciliation |
| `crates/z00z_wallets/tests/test_tx_store_integration.rs` | refactor from per-tx JSON assumptions to JSONL live store, folded reads, hashes, tombstones, legacy migration, and exact tx package bytes |
| `crates/z00z_wallets/tests/test_tx_balance.rs` | update Alice/Bob available and pending assertions across build, export, import, submit, confirm, cancel, fail, and reconcile |
| `crates/z00z_wallets/tests/test_stealth_output.rs` | ensure recipient and change outputs still use approved stealth output builders |
| `crates/z00z_wallets/src/stealth/output/test_output.rs` | preserve lower-level output builder invariants used by recipient/change construction |
| `crates/z00z_wallets/src/stealth/output/test_output_extra.rs` | preserve extra output builder and validation edge-case coverage affected by sender change output construction |
| `crates/z00z_wallets/src/stealth/output/test_facade_zkpack_suite.rs` | ensure facade-backed package/output helpers do not leak secrets and remain compatible with canonical package bytes |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_backup_impl_suite.rs` | update RPC backup create/list/restore coverage for exact JSONL preservation, restore modes, and no invented tx history |
| `crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs` | update exporter tests so forensic backup stores exact live JSONL bytes plus manifest and rejects export-only regeneration as authority |
| `crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs` | update importer tests for archived JSONL byte validation, manifest/hash tamper rejection, and restore-mode behavior |
| `crates/z00z_wallets/src/backup/crypto/test_wallet_backup_suite.rs` | keep encrypted backup envelope and integrity tests passing after forensic JSONL bytes are added to the encrypted payload |
| `crates/z00z_wallets/src/backup/crypto/wallet_backup.rs` | update inline or companion backup-envelope tests, or record no-change evidence, so encrypted wallet backup payload changes cannot bypass JSONL manifest/hash validation or weaken `.wlt` snapshot integrity |
| `crates/z00z_wallets/tests/test_rpc_types_serialization.rs` | update backup and tx-history related RPC type roundtrips if response payloads grow, and preserve the guard that backup settings do not introduce a user toggle that silently changes forensic tx-history behavior |
| `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs` | update boundary tests so wallet export pack stays separate from tx-history JSONL and secrets remain excluded from operator JSONL |
| `crates/z00z_wallets/tests/test_backup_restore_identity.rs` | preserve chain and wallet identity checks while restoring JSONL history to the restored wallet stem path |
| `crates/z00z_wallets/tests/test_backup_metadata_policy.rs` | ensure backup metadata remains truthful when forensic JSONL manifest data is present |
| `crates/z00z_wallets/tests/test_backup_kdf_contract.rs` | keep KDF contract unchanged while payload shape gains forensic JSONL bytes |
| `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs` | ensure wallet persistence backup service does not move tx package history into `.wlt` |
| `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | preserve `.wlt` open semantics without requiring or creating tx-history JSONL, and add evidence that tx packages remain outside the RedB `.wlt` container |
| `crates/z00z_wallets/src/db/redb/tests/redb_wallet_store.rs` | update or document no-change evidence for RedB `.wlt` schema/object tests so no tx-history table, tx package blob, or canonical JSONL data is introduced into `.wlt` storage |
| `crates/z00z_wallets/src/db/backends/wallet_store.rs` | update inline wallet-store boundary tests, or record no-change evidence, so `.wlt` persistence remains independent from tx-history JSONL sidecars |
| `crates/z00z_wallets/src/db/redb/tests/test_redb_wallet_crypto_suite.rs` | record no-change or update evidence that encrypted RedB wallet crypto still protects only `.wlt` payloads and does not become the tx-history package store |
| `crates/z00z_wallets/src/db/redb/tests/test_storage_backend_suite.rs` | record no-change or update evidence for backend file I/O semantics after sibling JSONL sidecars are introduced |
| `crates/z00z_wallets/src/db/redb/tests/test_index_codecs_suite.rs` | record no-change or update evidence that wallet index codecs do not gain hidden tx-history/package fields |
| `crates/z00z_wallets/tests/test_wlt_validator.rs` | ensure the offline `.wlt` validator remains scoped to the `.wlt` container and neither requires nor validates the sibling tx-history JSONL sidecar |
| `crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs` | ensure wallet discovery ignores tx-history JSONL sidecars as candidate wallet files and does not confuse sidecars with `.wlt` sources |
| `crates/z00z_wallets/tests/test_key_manager_storage_unlock.rs` | record no-change or update evidence that key-manager unlock flows remain `.wlt`-only and do not depend on tx-history sidecar presence |
| `crates/z00z_wallets/src/key/manager/test_key_manager_password_suite.rs` | record no-change or update evidence that password and key-manager tests still treat tx-history as outside the encrypted `.wlt` key material boundary |
| `crates/z00z_wallets/tests/test_wallet_service_errors.rs` | update wallet service error tests for JSONL sidecar tamper, missing/corrupt history, restore-mode failures, and no per-tx directory assumptions where applicable |
| `crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs` | update end-to-end RPC create/backup/restore roundtrip so restored tx-history JSONL behavior is visible through public RPC |
| `crates/z00z_wallets/src/services/app/test_app_service_suite.rs` | update app-level create/open tests so `.wlt` creation remains independent from tx-history sidecar creation and no live per-tx directory is expected |
| `crates/z00z_wallets/tests/test_app_service_create_wallet.rs` | preserve create-wallet persistence assertions while adding evidence that a new wallet does not embed or require tx package history in `.wlt` |
| `crates/z00z_wallets/tests/test_create_wallet_crypto_e2e.rs` | preserve encrypted create-wallet E2E checks and add evidence that tx-history sidecars are absent until real tx records exist |
| `crates/z00z_wallets/tests/test_deterministic_derivation_across_restarts.rs` | preserve deterministic wallet identity and restart behavior while ensuring sidecar JSONL path derivation is stable across restarts |
| `crates/z00z_wallets/tests/test_phase2_production_hardening.rs` | update or record no-change evidence for `.wlt` tamper/hardening tests after tx-history is kept outside `.wlt` |
| `crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs` | record no-change evidence that plaintext seed display tests remain scoped to wallet secrets and cannot expose tx package sidecar material |
| `crates/z00z_wallets/src/wallet/snapshot/test_snapshot_suite.rs` | update or record no-change evidence for wallet snapshot tests so snapshots remain wallet state only and do not become tx package history authority |
| `crates/z00z_wallets/src/adapters/rpc/methods/test_storage_impl_suite.rs` | update or record no-change evidence for storage RPC tests when checkpoint/store-op evidence becomes wallet reconciliation input |
| `crates/z00z_wallets/tests/test_tx_fee.rs` | ensure fee output semantics still match declared fee and verifier rules |
| `crates/z00z_wallets/src/tx/fees/test_fee_estimator_suite.rs` | update or record no-change evidence so fee-estimation tests remain compatible with declared-fee output semantics and verifier checks |
| `crates/z00z_wallets/src/tx/fees/fee_estimator.rs` | update inline or companion fee-estimator tests, or record no-change evidence, if fee estimation changes due to package-backed output construction |
| `crates/z00z_wallets/tests/test_tx_serial.rs` | ensure canonical tx serialization remains stable for stored package bytes |
| `crates/z00z_wallets/tests/test_tx_digest_framing.rs` | ensure tx hash/digest framing remains stable for JSONL idempotency keys and package hashes |
| `crates/z00z_wallets/src/tx/ids/tx_id.rs` | update inline tx-id tests, or record no-change evidence, so tx hash derivation remains stable for JSONL idempotency and sender/receiver duplicate submission |
| `crates/z00z_wallets/src/tx/ids/pay_ref.rs` | update inline pay-ref tests, or record no-change evidence, if package export/import metadata or receipt linking touches payment references |
| `crates/z00z_wallets/src/tx/multi_io.rs` | update inline multi-input/output tests, or record no-change evidence, so aggregate input/output semantics stay compatible with reservation, change, fee, and JSONL package retention |
| `crates/z00z_wallets/tests/test_tx_tamper.rs` | ensure tampered package bytes fail verification before wallet state mutation |
| `crates/z00z_wallets/tests/test_tx_wrong_root.rs` | ensure wrong-root evidence fails reconciliation/admission without increasing available balance |
| `crates/z00z_wallets/tests/test_tx_stealth_flow.rs` | ensure stealth sender/receiver package flow remains compatible with stored canonical tx bytes |
| `crates/z00z_wallets/tests/test_tx_roundtrip.rs` | ensure package roundtrip preserves exact bytes used by JSONL and portable package import/export |
| `crates/z00z_wallets/tests/test_tx_parity.rs` | ensure sender and receiver submission parity holds for the same tx bytes and tx hash |
| `crates/z00z_wallets/tests/test_tx_drift.rs` | extend drift guards for duplicate tx schema, duplicate verifier, and per-tx JSON live store reintroduction |
| `crates/z00z_wallets/src/tx/spend/spend_rules.rs` | update inline spend-rule tests, or record no-change evidence, so `Available`/pending/spent lifecycle semantics cannot make non-spendable rows selectable |
| `crates/z00z_wallets/src/tx/spend/spending.rs` | update inline spending tests, or record no-change evidence, for selected-input, change, fee, and conservation behavior used by package assembly |
| `crates/z00z_wallets/src/tx/spend/witness_gate.rs` | update inline witness-gate tests, or record no-change evidence, so witness/proof gates still block admission and reconciliation without required evidence |
| `crates/z00z_wallets/src/tx/claim/test_claim_tx.rs` | update claim tx tests, or record no-change evidence, for package verification and receiver claim interactions |
| `crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl_proof.rs` | update inline claim verifier tests, or record no-change evidence, so proof-backed claim validation remains compatible with Phase 044 package verification |
| `crates/z00z_wallets/src/claim/test_nullifier_store.rs` | update nullifier store tests, or record no-change evidence, when receiver finalization and checkpoint evidence rely on claim-nullifier semantics |
| `crates/z00z_wallets/src/receiver/scan/test_stealth_scanner.rs` | preserve report-only scan behavior and ensure scanning does not become tx confirmation evidence |
| `crates/z00z_wallets/src/receiver/scan/stealth_scanner/test_stealth_scanner.rs` | mirror scanner behavior coverage where the nested scanner suite is active, or document why one suite supersedes the other |
| `crates/z00z_wallets/src/receiver/scan/test_stealth_scan_support_suite.rs` | update scan support tests, or record no-change evidence, so scan helpers preserve report-only behavior and do not become tx validation evidence |
| `crates/z00z_wallets/src/receiver/manager/test_receiver_manager_suite.rs` | update receiver manager tests, or record no-change evidence, when sender-owned change outputs derive/select receivers through the approved receiver manager path |
| `crates/z00z_wallets/src/receiver/manager/test_canonical_snapshot_suite.rs` | update canonical receiver snapshot tests, or record no-change evidence, if receiver state snapshots are used by report-only, pending receive, or persist-claim flows |
| `crates/z00z_wallets/src/receiver/request/test_stealth_request.rs` | update stealth request tests, or record no-change evidence, for request-based receive flows that must stay report-only until explicit persist-claim/finalization |
| `crates/z00z_wallets/src/receiver/card/test_stealth_card.rs` | update card stealth tests, or record no-change evidence, so card-bound output construction stays separate from tx confirmation and package history authority |
| `crates/z00z_wallets/src/receiver/card/test_stealth_trust_suite.rs` | update card trust tests, or record no-change evidence, where trusted receiver material intersects with sender change or receiver-owned outputs |
| `crates/z00z_wallets/src/key/receiver/test_stealth_keys_suite.rs` | update receiver key tests, or record no-change evidence, if receiver-owned output detection or sender change ownership touches receiver key derivation |
| `crates/z00z_wallets/tests/test_direct_tx_receive.rs` | update direct tx receive flow for pending receive versus final persist-claim behavior |
| `crates/z00z_wallets/tests/test_e2e_req_flow.rs` | preserve request-based report-only receive and sender/receiver flow semantics after package-backed tx lifecycle changes |
| `crates/z00z_wallets/tests/test_e2e_runtime_parity.rs` | ensure runtime parity remains true for report-only receive, package-backed send, and JSONL-backed history views |
| `crates/z00z_wallets/tests/test_e2e_send_scan.rs` | update end-to-end send/scan expectations for pending receive and reconciliation evidence |
| `crates/z00z_wallets/tests/test_phase040_spend_proof_support.rs` | ensure Phase 040 spend proof support remains compatible with canonical package verification and wallet-local lifecycle closure |
| `crates/z00z_wallets/tests/test_phase14_pipeline.rs` | preserve pipeline invariants affected by package build, proof, and verification routing |
| `crates/z00z_wallets/tests/test_phase24_gate.rs` | preserve phase gate assumptions that overlap tx package, receiver, and storage proof boundaries |
| `crates/z00z_wallets/tests/test_s5_sender_examples.rs` | update sender examples for reservation, exported locks, package bytes, and role-tagged outputs |
| `crates/z00z_wallets/tests/test_s5_spec6_bridge.rs` | preserve S5/S6 bridge semantics for report-only receive and package-backed sender output flow |
| `crates/z00z_wallets/tests/test_s6_recv_examples.rs` | update receiver examples for report-only, pending receive, persist-claim, and duplicate finalization behavior |
| `crates/z00z_wallets/tests/test_spec_terms_guard.rs` | extend terminology guards for `Available`, `Spent`, `Confirmed`, JSONL live store, and forbidden `Validated` spendable-state drift |
| `crates/z00z_wallets/tests/test_spend_statement.rs` | ensure spend statement semantics remain compatible with selected inputs, resolved input evidence, and package verification |
| `crates/z00z_wallets/tests/test_spend_witness_gate.rs` | preserve witness gate behavior required before admission or reconciliation can advance wallet state |
| `crates/z00z_wallets/tests/test_tx_pass.rs` | keep passing tx package behavior compatible with exact JSONL package retention |
| `crates/z00z_wallets/tests/test_tx_poison.rs` | ensure poisoned or malformed tx inputs fail before wallet state mutation or JSONL advancement |
| `crates/z00z_wallets/tests/test_tx_spent_gate.rs` | ensure spent-gate tests distinguish final spent from local reservation, exported, and pending spend states |
| `crates/z00z_wallets/tests/test_claim_snapshot_core.rs` | preserve claim snapshot reconciliation invariants and add evidence that claim snapshots do not become tx-history confirmation authority |
| `crates/z00z_core/src/assets/test_asset_suite.rs` | record no-change or update evidence if wallet lifecycle work touches core asset construction, stealth consistency, asset IDs, or amount semantics |
| `crates/z00z_core/src/assets/test_registry_suite.rs` | record no-change or update evidence if wallet lifecycle work touches asset definition lookup, registry sharing, or asset-class metadata |
| `crates/z00z_core/tests/assets/test_integration_assets_test13.rs` | record no-change or update evidence for concurrent wallet asset generation and registry sharing if reservation or receiver-output work touches asset creation concurrency |
| `crates/z00z_storage/tests/test_serialization_restore.rs` | record no-change or update evidence for lower-level asset restore roots if wallet recovery evidence or reconciliation root handling touches storage serialization |
| `crates/z00z_simulator/tests/test_wallet_integration.rs` | update simulator wallet integration expectations for `.wlt` plus sibling JSONL output, preserving tx package history outside `.wlt` and avoiding live per-tx history directories |
| `crates/z00z_simulator/tests/test_claim_persist.rs` | update or document no-change evidence for claim persistence outputs so wallet asset state changes do not imply tx confirmation or tx-history package persistence |
| `crates/z00z_simulator/tests/test_claim_post.rs` | update post-claim wallet export/import checks so `.wlt` sync remains separate from tx-history JSONL preservation |
| `crates/z00z_simulator/tests/test_claim_snapshot.rs` | preserve snapshot reconciliation invariants while ensuring snapshot evidence is not treated as tx-history authority |
| `crates/z00z_simulator/tests/test_e2e_phase4.rs` | update end-to-end Phase 4 receive persistence expectations for report-only versus persist-claim and the external tx-history sidecar boundary |
| `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | update scenario stage-surface expectations and generated artifact checks for `.wlt` plus tx-history JSONL outputs where transactions are present |
| `crates/z00z_simulator/tests/test_stage4_output_crypto.rs` | preserve wallet output crypto checks while adding evidence that tx packages/history are not embedded into `.wlt` artifacts |

Live authority checklist:

- [ ] preserve `AssetSelectorImpl` as the selected-input and change calculator;
- [ ] preserve `TxAssemblerImpl` as the canonical tx package assembler;
- [ ] preserve `verify_full_tx_package(...)` as the package verification gate;
- [ ] preserve `CanonicalSpendProofBackend` as the proof backend authority;
- [ ] preserve `ResolvedInput` for membership and conservation pre-state;
- [ ] preserve `apply_batch_checkpoint(...)` for simulated confirmation effects;
- [ ] preserve `TxRecord` as canonical persisted tx-byte retention;
- [ ] preserve `TxStorage::list_by_status(...)` and
  `TxStorage::update_status(...)` as tx status access seams;
- [ ] preserve `AssetStorage` as the wallet asset storage seam;
- [ ] preserve `ScanStorage` for scan cursor state only, not tx validation
  evidence;
- [ ] preserve `CheckpointDraft`, `CheckpointPubIn`, `CreatedEnt`, and
  `SpentEnt` as checkpoint evidence vocabulary;
- [ ] preserve `wallet_claimed_assets` through `put_claimed_asset(...)` as the
  wallet-owned claim authority;
- [ ] preserve `recv_route(..., ReceiveNext::PersistClaim)` as the receiver
  claim persistence route;
- [ ] preserve `build_tx_stealth_output_validated(...)` for tx stealth output
  construction;
- [ ] preserve `build_card_stealth_output_validated(...)` for card-bound output
  construction;
- [ ] preserve `ChainClient` and `TransactionBroadcast` as future real-chain
  trait boundaries.

Forbidden authority checklist:

- [ ] do not add a second transaction wire schema;
- [ ] do not add a second transaction assembler;
- [ ] do not add a second public spend verifier;
- [ ] do not add a second receiver scan or claim persistence path;
- [ ] do not add a second wallet asset database that competes with claimed
  assets and asset storage;
- [ ] do not add a raw-builder sender path that bypasses validated receiver,
  card, or request checks;
- [ ] do not keep RPC-local broadcast simulation that bypasses a trait-backed
  admission boundary;
- [ ] do not create a sender-only submission path that treats receiver import
  as less authorized for the same tx bytes.

Scope checklist:

- [ ] sender-side asset reservation for selected inputs;
- [ ] sender-side transaction build with recipient, change, and fee outputs;
- [ ] sender-side tx proof and verifier routing through existing canonical
  seams;
- [ ] wallet tx journal records with inputs, outputs, status, tx bytes, tx hash,
  fees, timestamps, and confirmation metadata;
- [ ] pending, cancellation, failure, retry, and confirmation transitions for
  sender assets;
- [ ] receiver-side report-only detection, claim-pending state, confirmation
  reconciliation, and claim persistence;
- [ ] balance RPC semantics for `total`, `available`, and `pending`;
- [ ] transaction history, pending list, and details RPC semantics backed by
  real journal data;
- [ ] portable tx package export for offline transfer and receiver-side import
  of canonical tx bytes;
- [ ] role-neutral tx submission with one admission trait and tx hash
  idempotency;
- [ ] simulated validator/admission adapter over existing tx/state/checkpoint
  semantics;
- [ ] wallet tx reconciliation scanner over storage/checkpoint evidence;
- [ ] tests and source-shape guards for double spend, phantom confirmation,
  fake pending balances, report-only persistence, and tx detail gaps.

Data contract and definition coverage:

| Contract or term | TODO coverage | Required preservation |
| --- | --- | --- |
| `Available` | `044-02`; `044-07`; `044-09` | confirmed spendable wallet value only |
| `Reserved` | `044-02`; `044-03`; `044-10` | locally selected input, non-spendable until released or advanced |
| `Exported` | `044-02`; `044-05`; `044-10` | exported tx bytes keep sender inputs locked until safe resolution |
| `Pending spend` | `044-02`; `044-06`; `044-07` | admitted spend remains non-spendable before checkpoint proof |
| `Spent` | `044-02`; `044-07`; `044-09` | final consumed input state only |
| `Pending change` | `044-03`; `044-07`; `044-09` | sender-owned change remains non-spendable until confirmation |
| `Pending receive` | `044-05`; `044-08`; `044-09` | receiver-owned output remains non-spendable until confirmation |
| `Reorg pending` | `044-02`; `044-07`; `044-09` | rollback state remains non-spendable until checkpoint evidence resolves it |
| `Report-only receive` | `044-08`; `044-10` | preview or scan result must not mutate claims or balance |
| `Tx journal` | `044-04`; `044-05`; `044-06`; `044-07` | links tx id, hash, bytes, rows, fees, receipts, and metadata |
| `Portable tx package` | `044-05`; `044-10` | canonical tx bytes plus redacted metadata only |
| `Submission actor` | `044-05`; `044-06` | sender or receiver role is audit metadata, not validator behavior |
| `Admission receipt` | `044-06`; `044-04` | acceptance evidence stored with tx journal |
| `Confirmation receipt` | `044-06`; `044-07` | checkpoint effect evidence for finalization |
| `Storage reconciliation scan` | `044-07`; `044-10` | scans typed evidence and applies final wallet transitions |
| `Validated evidence` | `044-07`; `Completion Gate` | evidence wording only, never persisted spendable state |
| `WalletAssetState` | `044-02`; `044-07`; `044-09` | total state mapping that fails closed on unknown values |
| `WalletTxStatus` | `044-04`; `044-06`; `044-07` | internal status maps to existing persistent/public vocabularies |
| `WalletTxJournalRecord` | `044-04`; `044-05`; `044-06`; `044-07` | rows include inputs, outputs, tx bytes, receipts, failure, origin, and submitter |
| `PortableWalletTxPackage` | `044-05` | version, chain id, tx hash, canonical bytes, metadata hash |
| `WalletTxAdmitter` | `044-05`; `044-06` | one sender/receiver admission path with idempotency |
| `ConfirmationReceipt` | `044-06`; `044-07` | tx hash, height, checkpoint id, roots, spent ids, created ids |
| `WalletTxReconciler` | `044-07` | typed pending-to-final reconciliation service |

Layer ownership checklist:

- [ ] `z00z_wallets::tx::selection` owns candidate selection and change
  calculation, not persistence, broadcast, or receiver claim finalization;
- [ ] `z00z_wallets::tx::verify` owns tx wire/package structure, digest, fee,
  output proof, and public checks, not wallet reservation or balance UI
  semantics;
- [ ] `z00z_wallets::tx::state` owns resolved inputs, membership witness,
  checkpoint apply, and pending/confirmed projection, not RPC-local fake
  broadcast success;
- [ ] `z00z_wallets::stealth::output` owns recipient/change output construction
  through approved builders, not wallet journal persistence;
- [ ] `z00z_wallets::services::wallet` owns wallet claimed assets, receive
  route, session, and receiver manager integration, not raw chain consensus or
  storage backend internals;
- [ ] `z00z_wallets::adapters::rpc` owns request validation, response mapping,
  and lifecycle delegation, not core lifecycle decisions or local tx simulation;
- [ ] `z00z_storage` owns checkpoint execution, storage roots, store ops, and
  claim-nullifier rows, not wallet UI pending balances or report-only receive
  behavior.

## 🚫 Explicit Phase Boundary

The following topics are intentionally out of scope for this backlog:

- modifying `crates/z00z_crypto/tari/**`;
- replacing Phase 043 assembler, verifier, receive, stealth-output, or
  storage membership behavior;
- claiming public or trustless proof-of-knowledge closure;
- implementing production consensus, networked aggregation, email transport,
  file-sync transport, or a real chain broadcaster;
- changing canonical `.wlt` semantics or reopening Phase 042 address-era
  naming decisions;
- turning report-only receive detection into claim persistence;
- creating sender-only tx submission priority over receiver-imported tx bytes;
- using scan cursor progress as proof that tx effects were validated;
- introducing a broad wallet database when a narrow lifecycle index over
  existing claimed assets is sufficient.

## ⚙️ Concrete Execution Tasks

### 044-01 Embedded Coverage And No-Duplicate Lock

Embedded requirement anchors:

- `Purpose`
- `Source Evidence`
- `Live Authority Surface`
- `Implementation Guide / Phase Gate 0`
- `Required Outputs`

Task-local requirement summary:

- prove every embedded requirement has an owner, a test home, and an evidence
  slot before code edits;
- preserve existing Phase 043 authorities instead of creating duplicates;
- keep vendor crypto read-only.

- [ ] Create `.planning/phases/044-wallet-assets/044-coverage.md` before code
  edits.
- [ ] Map every `EV-044-*`, `D-044-*`, `PH44-*`, and `AC-044-*` identifier to
  one owner file, one test home, and one evidence slot.
- [ ] Record each Phase 043 prerequisite as implemented, missing, or in
  progress.
- [ ] If a prerequisite is missing, route the fix to the Phase 043 authority
  path before Phase 044 consumes it.
- [ ] Run the embedded inventory commands and paste the relevant evidence into
  `044-coverage.md`.
- [ ] Confirm the execution plan does not touch `crates/z00z_crypto/tari/**`.

Files:

- new `.planning/phases/044-wallet-assets/044-coverage.md`

Tests:

- [ ] run spec-exact Phase Gate 0 inventory commands:

```bash
rg -n "BuiltTxStub|pending = 0|inputs: vec!\[\]|outputs: vec!\[\]|simulate|not implemented in Phase 1" crates/z00z_wallets/src
rg -n "AssetSelection|TxOutRole::Change|TxOutRole::Fee|ResolvedInput|apply_batch_checkpoint|CanonicalSpendProofBackend" crates/z00z_wallets/src
rg -n "ReceiveNext::ReportOnly|ReceiveNext::PersistClaim|put_claimed_asset|wallet_claimed_assets" crates/z00z_wallets/src
```

- [ ] run stub and fake-success inventory:
  `rg -n "BuiltTxStub|pending = 0|inputs: vec!\\[\\]|outputs: vec!\\[\\]|simulate retry success" crates/z00z_wallets/src`
- [ ] run canonical authority inventory:
  `rg -n "AssetSelectorImpl|TxAssemblerImpl|verify_full_tx_package|CanonicalSpendProofBackend|ResolvedInput|apply_batch_checkpoint" crates/z00z_wallets/src crates/z00z_storage/src`
- [ ] run receive persistence inventory:
  `rg -n "recv_route|ReceiveNext::ReportOnly|ReceiveNext::PersistClaim|put_claimed_asset|wallet_claimed_assets" crates/z00z_wallets/src`

Exit condition:

- Phase 044 has a coverage ledger proving every source identifier has an owner,
  a test home, and no planned duplicate authority.

### 044-02 Wallet Asset Ledger And Reservation Layer

Embedded requirement anchors:

- `D-044-001`, `D-044-002`, `D-044-013`
- `Definitions`
- `Sender Input State Machine`
- `Wallet Asset State`
- `PH44-LEDGER`
- `PH44-CANCEL`
- `Implementation Guide / Phase Gate 1`

Task-local requirement summary:

- only `Available` assets are spendable;
- reservation, exported, pending, final spent, and rollback states are distinct;
- final spendability remains `Available`, not `Validated`.

- [ ] Add or extend a wallet-owned asset lifecycle record for `Available`,
  `Reserved`, `Exported`, `ClaimPending`, `PendingSpend`, `Spent`,
  `PendingChange`, `PendingReceive`, `ReorgPending`, `Quarantined`, and
  `Dropped`.
- [ ] Keep `wallet_claimed_assets` and `put_claimed_asset(...)` as the
  ownership authority; use the lifecycle ledger as an index over wallet spend
  state rather than a replacement store.
- [ ] Add atomic reserve and release operations for selected input rows.
- [ ] Fail closed when any selected input is not `Available`.
- [ ] Keep `mark_spent_for_wallet(...)` or equivalent final-spent storage for
  confirmed deletion evidence only.
- [ ] Add migration or derived-index behavior so existing claimed assets appear
  as `Available` until explicitly reserved, exported, pending, spent, or
  quarantined.
- [ ] Add redaction rules for sensitive lifecycle data in debug exports.

Files:

- `crates/z00z_wallets/src/persistence/assets/asset_storage.rs`
- `crates/z00z_wallets/src/persistence/assets/asset_storage_impl.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_assets.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
- `crates/z00z_wallets/src/db/redb/store/redb_wallet_store_objects.rs`
- `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`

Tests:

- [ ] extend `crates/z00z_wallets/src/persistence/assets/test_asset_storage_impl_suite.rs`
  - active wallet row uniqueness
  - unknown state fails closed
  - confirmed spent differs from local reservation
- [ ] extend `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
  - two simultaneous builds cannot reserve the same input
  - report-only receive does not create lifecycle rows
- [ ] extend `crates/z00z_wallets/src/tx/selection/test_asset_selector_suite.rs`
  - selector only sees `Available` candidates

Exit condition:

- Two simultaneous build attempts against the same wallet-owned input cannot
  both reserve it, and reservation does not masquerade as final spend.

### 044-03 Sender Build And Send Lifecycle

Embedded requirement anchors:

- `D-044-003`, `D-044-004`, `D-044-009`
- `Sender Output State Machine`
- `PH44-SEND`
- `Wallet Tx Status`
- `Sender Flow`
- `Implementation Guide / Phase Gate 2`

Task-local requirement summary:

- sender build selects and reserves inputs before exposing tx bytes;
- recipient, change, and fee outputs are real role-tagged outputs;
- package assembly, proof material, and verification reuse canonical Phase 043
  seams.

- [ ] Replace `BuiltTxStub` with canonical tx package bytes plus journal
  metadata.
- [ ] Query only available wallet assets and call `AssetSelectorImpl::select(...)`
  with `target_amount` and `fee`.
- [ ] Reserve selected input rows before output construction.
- [ ] Build recipient output through the validated receiver, card, or request
  builder path.
- [ ] Build sender-owned `TxOutRole::Change` output when `change_amount > 0`.
- [ ] Build `TxOutRole::Fee` coin output when `fee > 0`.
- [ ] Assemble canonical tx wire/package through `TxAssemblerImpl` or the
  approved Phase 043 assembler path.
- [ ] Produce or attach proof material through `CanonicalSpendProofBackend` or
  its approved wrapper.
- [ ] Call `verify_full_tx_package(...)` before persistence advances past
  reserved state.
- [ ] Release reservations and journal a typed failure on every build,
  verification, output, proof, or reservation error.

Files:

- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/tx/selection/asset_selector.rs`
- `crates/z00z_wallets/src/tx/verify/tx_wire_types.rs`
- `crates/z00z_wallets/src/tx/verify/tx_verifier.rs`
- `crates/z00z_wallets/src/tx/verify/tx_verifier_helpers.rs`
- `crates/z00z_wallets/src/tx/state/state_resolved_input.rs`
- `crates/z00z_wallets/src/tx/spend/spend_verification.rs`
- `crates/z00z_wallets/src/tx/tx_assembler.rs`
- `crates/z00z_wallets/src/tx/output/output_flow.rs`
- `crates/z00z_wallets/src/stealth/output/output_build.rs`
- `crates/z00z_wallets/src/stealth/output/output_validator.rs`
- `crates/z00z_wallets/src/tx/proof/spend_proof_backend.rs`

Tests:

- [ ] extend `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`
  - non-empty input and output rows
  - reservation before tx bytes are exposed
  - verification failure releases reservations
- [ ] extend `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs`
  - build does not imply fake broadcast success
- [ ] extend `crates/z00z_wallets/tests/test_stealth_output.rs`
  - recipient and change outputs use approved builder surfaces

Exit condition:

- A built tx has real selected inputs, role-tagged recipient/change/fee outputs
  when required, verifier-approved tx bytes, and non-spendable selected inputs.

### 044-04 Tx Journal, Details, Pending Lists, And History

Embedded requirement anchors:

- `D-044-008`, `D-044-010`
- `PH44-HISTORY`
- `Tx Journal Record`
- `Wallet Tx Status`
- `Implementation Guide / Phase Gate 3`

Task-local requirement summary:

- tx details, pending lists, and history are views over tx journal metadata,
  lifecycle rows, and preserved package bytes;
- empty input/output detail arrays are forbidden for built or submitted txs;
- this task defines the journal data shape consumed by JSONL storage tasks.

- [ ] Add a versioned tx journal row or extend existing tx metadata without
  breaking `TxRecord` storage.
- [ ] Persist selected input refs, resolved-input metadata hashes, output refs,
  output roles, amount, fee, origin, submitter role, reservation ids, and
  lifecycle status.
- [ ] Store canonical tx package bytes in `TxRecord` when package bytes exist.
- [ ] Join journal rows and tx bytes for history, pending, and details
  responses.
- [ ] Replace empty input/output arrays in `get_transaction_details_impl(...)`
  with decoded and journal-backed rows.
- [ ] Preserve existing pagination, cursor, and filter behavior.
- [ ] Surface partial-details diagnostics when tx bytes exist but journal
  metadata is incomplete.

Files:

- `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_history.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`

Tests:

- [ ] extend `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
  - reserved, exported, submitted, admitted, pending-spend, pending-change, and
    pending-receive rows appear as pending
- [ ] extend `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_body.rs`
  - stable ordering by timestamp plus tx id
  - details are populated for built, submitted, failed, cancelled, and
    confirmed txs
- [ ] extend `crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - tx bytes and journal metadata rehydrate together

Exit condition:

- Every tx created by build, send, export, import, admission, failure, or
  confirmation can be inspected with truthful input, output, receipt, and status
  details.

### 044-04A Canonical JSONL Live Tx-History Authority And Path Contract

Embedded requirement anchors:

- `P-044-001` through `P-044-006`
- `Live JSONL entry contract`
- `PH44-HISTORY`
- `PH44-DRIFT`

Task-local requirement summary:

- `.wlt` remains wallet snapshot and restore state only;
- `wallet_<stem>_tx_history.jsonl` is the live tx-history store;
- `wallet_<stem>_tx_history/` is legacy migration input only;
- exact tx package bytes are preserved in JSONL when a package exists.

- [ ] Replace any runtime assumption that `wallet_<stem>_tx_history/` is the
  live store with `wallet_<stem>_tx_history.jsonl`.
- [ ] Keep `wallet_<stem>.wlt` and `wallet_<stem>_tx_history.jsonl` as sibling
  wallet-stem artifacts in the wallet output directory.
- [ ] Restrict helpers named like `wallet_tx_history_dir(...)` to legacy
  migration only, or remove/rename them so new write paths cannot call them.
- [ ] Ensure helpers that open tx history accept or resolve the JSONL file path,
  not a directory.
- [ ] Keep the tx-history store file-backed; do not add RedB, SQLite, RocksDB,
  or another broad database for tx packages in Phase 044.
- [ ] Store encrypted tx fields and encrypted asset data when they are part of
  the canonical tx package bytes.
- [ ] Reject any attempt to store decrypted wallet secrets, private blindings,
  plaintext seed material, or non-package private wallet state in JSONL.

Files:

- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`

Tests:

- [ ] update wallet stem naming tests:
  - `wallet_<stem>.wlt` remains the wallet snapshot name
  - `wallet_<stem>_tx_history.jsonl` is the live tx-history path
  - `wallet_<stem>_tx_history/` is not returned for new live writes
- [ ] add a source-shape guard that fails if new-write code calls a live
  `wallet_tx_history_dir(...)` helper.

Exit condition:

- New wallet tx activity has one canonical live tx-history path:
  `wallet_<stem>_tx_history.jsonl`, with no new per-tx JSON directory writes
  and no tx package storage inside `.wlt`.

### 044-04B JSONL-Backed TxStorageImpl And Folded Reads

Embedded requirement anchors:

- `P-044-005`, `P-044-007`, `P-044-008`, `P-044-009`
- `Live JSONL entry contract`
- `PH44-HISTORY`

Task-local requirement summary:

- `TxStorageImpl` is a single-file JSONL store;
- writes append ordered rows through atomic temp-file replacement;
- reads fold rows by `tx_hash` to produce the current tx view.

- [ ] Refactor `TxStorageImpl::new(...)` so its root is the JSONL file path,
  not a directory path.
- [ ] Add `WalletTxHistoryJsonlEntry` fields for wallet stem, sequence,
  recorded timestamp, entry kind, record hash, tx bytes hash, previous entry
  hash, entry hash, and full `TxRecord`.
- [ ] Add `WalletTxHistoryEntryKind` values for `Created`, `Imported`,
  `Exported`, `Submitted`, `Admitted`, `Confirmed`, `Failed`, `Cancelled`,
  `Tombstoned`, and `Migrated`.
- [ ] Implement `put(record)` by appending a JSONL row that contains the full
  `TxRecord` and exact `tx_bytes`.
- [ ] Implement `update_status(tx_hash, status)` by folding the latest record,
  changing only current status and timestamp fields, then appending a new row.
- [ ] Implement `delete(tx_hash)` as a `Tombstoned` row, not physical erasure of
  the package bytes.
- [ ] Implement `get`, `list`, and `list_by_status` by reading, validating, and
  folding JSONL rows by `tx_hash`.
- [ ] Reject invalid tx hash labels before writing.
- [ ] Treat duplicate tx hashes as legal lifecycle rows that fold
  deterministically by sequence.
- [ ] Fail closed on malformed rows, hash mismatches, sequence gaps, or
  unexpected entry kinds unless a future recovery mode is explicitly added.
- [ ] Write updates by reading the existing file, appending one serialized line
  in memory, writing a temporary sibling file, and renaming it into place.
- [ ] Guard writes with a per-wallet file lock or an existing wallet session
  lock so concurrent writers cannot interleave sequences.

Files:

- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
- `crates/z00z_wallets/src/backup/crypto/backup_wire.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`

Tests:

- [ ] `TxStorageImpl::put` writes one JSONL row with exact `tx_bytes`.
- [ ] `TxStorageImpl::update_status` appends a second row and folded reads
  return the latest status.
- [ ] `TxStorageImpl::delete` appends a tombstone and folded reads hide the tx
  by default.
- [ ] corrupted `tx_bytes_hash`, `record_hash`, `entry_hash`, or
  `previous_entry_hash` is rejected.
- [ ] duplicate lifecycle rows fold deterministically by sequence.
- [ ] concurrent write simulation cannot create duplicate sequence numbers or
  interleaved partial rows.

Exit condition:

- `TxStorage` callers remain compatible while the backing authority is a single
  tamper-evident JSONL file that preserves exact tx package bytes.

### 044-04C Backup, Restore, And Forensic JSONL Preservation

Embedded requirement anchors:

- `P-044-010`, `P-044-011`
- `Live JSONL entry contract`
- `Required output checklist`

Task-local requirement summary:

- backup copies or embeds exact live JSONL bytes plus a manifest;
- restore writes JSONL back to the live wallet-stem path;
- `Vec<TxRecord>` is only a folded view, not forensic authority.

- [ ] Change backup creation so it reads the existing live
  `wallet_<stem>_tx_history.jsonl` bytes instead of collecting records from
  per-tx JSON files.
- [ ] Validate JSONL rows, sequence, hashes, wallet identity, and tx package
  hashes before including history in an encrypted backup.
- [ ] Store exact JSONL bytes plus a manifest in the forensic backup payload.
- [ ] Do not make `Vec<TxRecord>` the backup authority; use it only as a
  derived compatibility view after JSONL validation.
- [ ] If a backup-side JSONL sidecar is requested, copy the live JSONL bytes to
  the backup destination and verify the copied hash.
- [ ] Do not overwrite the wallet-directory live JSONL as part of backup
  creation unless the file is missing and an empty history file must be
  initialized.
- [ ] Implement `WalletOnly` restore so it restores only the wallet snapshot and
  does not invent tx history.
- [ ] Implement `WalletPlusHistory` restore so it restores the wallet snapshot
  and writes archived JSONL bytes to the restored wallet stem's live JSONL path.
- [ ] Implement `TxHistoryOnly` restore so it writes only tx-history JSONL for
  forensic inspection and does not create `.wlt`.
- [ ] Validate manifest hashes, row hashes, tx package hashes, sequence, chain
  identity, and wallet identity before writing restored JSONL.
- [ ] Preserve original wallet id, tx hash, package bytes, event order, and
  forensic hashes in explicit fields when restored wallet stem rewriting is
  required.

Files:

- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
- `crates/z00z_wallets/src/backup/crypto/backup_wire.rs`
- `crates/z00z_wallets/src/backup/export/backup_exporter_impl.rs`
- `crates/z00z_wallets/src/backup/import/backup_importer_impl.rs`
- `crates/z00z_wallets/src/backup/import/backup_importer.rs`
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
- `crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs`

Tests:

- [ ] backup embeds or copies exact live JSONL bytes and does not rebuild the
  file from per-tx JSON records.
- [ ] tampering with archived JSONL bytes, row hash, tx bytes hash, manifest,
  sequence, wallet identity, or chain identity fails closed.
- [ ] `WalletPlusHistory` recreates the JSONL file and preserves exact tx
  package bytes.
- [ ] `TxHistoryOnly` creates only JSONL history and no `.wlt`.
- [ ] backup of an empty wallet creates a valid empty history artifact without
  inventing tx rows.

Exit condition:

- Backup and restore preserve live tx-history JSONL byte-for-byte where the
  wallet stem does not change, and preserve exact package bytes and event order
  when stem metadata must be rewritten.

### 044-04D Legacy Per-Tx JSON Migration And Guards

Embedded requirement anchors:

- `P-044-004`, `P-044-012`
- `PH44-DRIFT`
- `T-044-015`

Task-local requirement summary:

- legacy per-tx JSON directories may be read only to migrate old data;
- migration writes JSONL and never deletes legacy directories automatically;
- guards prevent the old live-store model from returning.

- [ ] Add an explicit migration path for existing
  `wallet_<stem>_tx_history/` directories.
- [ ] If `wallet_<stem>_tx_history.jsonl` exists, treat JSONL as authoritative
  and do not merge legacy directory rows automatically.
- [ ] If only the legacy directory exists, read every valid per-tx JSON record,
  validate tx hash labels, and preserve each record's `tx_bytes`.
- [ ] Sort legacy records by timestamp, then tx hash, for deterministic
  migration order.
- [ ] Write `Migrated` JSONL entries with manifest evidence or metadata for the
  old directory hash set.
- [ ] Do not delete or modify the legacy directory during Phase 044 migration.
- [ ] After successful migration, send all new writes only to JSONL.
- [ ] Add source-shape guards for:
  `wallet_{wallet_stem}_tx_history`, `format!("{tx_hash}.json")`,
  `collect_tx_history_records`, and live `wallet_tx_history_dir` usage.
- [ ] Document any remaining matches as legacy migration code, test fixtures, or
  compatibility-only strings.

Files:

- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`

Tests:

- [ ] legacy per-tx JSON migration preserves tx packages and writes `Migrated`
  JSONL rows.
- [ ] migration does not delete the legacy directory.
- [ ] if JSONL already exists, migration treats JSONL as authoritative.
- [ ] guards fail if `TxStorageImpl` writes one JSON file per tx hash.
- [ ] guards fail if backup collects normal history from per-tx JSON files.

Exit condition:

- Existing incorrectly-shaped history can be rescued, but all normal Phase 044
  runtime, backup, restore, and forensic flows use JSONL as the only tx-history
  authority.

### 044-05 Portable Package Export, Import, And Role-Neutral Submission

Embedded requirement anchors:

- `D-044-011`, `D-044-012`
- `PH44-OFFLINE`
- `Portable Tx Package`
- `Offline Package Flow`
- `Receiver Flow`
- `Implementation Guide / Phase Gate 3A`

Task-local requirement summary:

- portable packages preserve canonical tx bytes, tx hash identity, chain id,
  version, checksum, and redacted metadata;
- sender and receiver submission have equal authority at the admission
  boundary;
- receiver imports are pending and non-spendable until confirmation evidence
  finalizes them.

- [ ] Add a narrow portable tx package export function that returns canonical
  tx bytes plus redacted metadata, tx hash, chain id, package version, and
  checksum.
- [ ] Keep sender-selected inputs locked as `Reserved` or `Exported` after
  package export.
- [ ] Reject package export that includes wallet secrets, decrypted asset packs,
  private blindings, plaintext seed material, or conflicting metadata.
- [ ] Add receiver-side import that decodes package bytes, validates chain id
  and version, runs `verify_full_tx_package(...)`, and scans outputs for wallet
  ownership.
- [ ] Store receiver-owned imports as `ClaimPending` or `PendingReceive` rows,
  never as immediately `Available`.
- [ ] Add receiver-side submit that calls `WalletTxAdmitter` with submitter
  role `Receiver` and the exact imported tx bytes.
- [ ] Make sender and receiver submission share tx hash idempotency, admission
  receipt storage, and lifecycle transition rules.
- [ ] Reject tampered, wrong-chain, malformed, duplicate-with-conflict, or
  no-owned-output packages without mutating available balance.

Files:

- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`

Tests:

- [ ] extend `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`
  - export preserves tx hash and keeps sender inputs non-spendable
- [ ] extend `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs`
  - sender and receiver submit the same tx bytes through the same path
  - duplicate submit is idempotent by tx hash
- [ ] extend `crates/z00z_wallets/tests/test_tx_balance.rs`
  - imported receiver output is pending and non-spendable
  - wrong-chain or tampered package changes no balance

Exit condition:

- Bob can import Alice's offline package, inspect the receiver-owned pending
  output, and submit the same canonical tx bytes through the same admission path
  Alice would use.

### 044-06 Admission And Confirmation Boundary

Embedded requirement anchors:

- `D-044-007`, `D-044-012`
- `PH44-ADMIT`
- `Admission Trait`
- `Confirmation Receipt And Reconciler`
- `Simulated Validator Boundary`
- `Implementation Guide / Phase Gate 4`

Task-local requirement summary:

- simulated admission is explicit, trait-backed, deterministic, and
  replaceable by real chain implementation later;
- RPC handlers must not fake broadcast success locally;
- admission receipts and confirmation receipts are persisted as tx evidence.

- [ ] Add `WalletTxAdmitter` or an equivalent narrow wallet-runtime trait.
- [ ] Define admission request fields for tx hash, canonical tx bytes, chain id,
  submitter role, and idempotency key.
- [ ] Implement a simulated adapter that validates package bytes and uses
  resolved inputs plus checkpoint apply semantics.
- [ ] Make `broadcast_transaction_impl(...)` call the admission trait instead
  of returning RPC-local simulated success.
- [ ] Store admission receipts in the tx journal.
- [ ] Confirm simulated txs by applying checkpoint semantics and returning
  confirmation receipts.
- [ ] Persist or expose typed checkpoint evidence that reconciliation can match
  against journal expectations.
- [ ] Keep real `ChainClient` and `TransactionBroadcast` as later trait-backed
  implementations without changing wallet lifecycle code.

Files:

- `crates/z00z_wallets/src/chain/client/chain_client.rs`
- `crates/z00z_wallets/src/chain/client/chain_client_impl.rs`
- `crates/z00z_wallets/src/chain/broadcast/broadcast.rs`
- `crates/z00z_wallets/src/chain/broadcast/broadcast_impl.rs`
- `crates/z00z_wallets/src/tx/state/state_update.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/scans/storage.rs`
- `crates/z00z_wallets/src/persistence/scans/storage_impl.rs`
- `crates/z00z_storage/src/checkpoint/build.rs`
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
- `crates/z00z_storage/src/assets/store_internal/store_rows.rs`

Tests:

- [ ] extend `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs`
  - broadcast requires admission trait evidence
  - sender and receiver submitter roles do not change validation
  - admission rejection rolls back deterministically
- [ ] extend `crates/z00z_wallets/src/tx/state/test_state_update_suite.rs`
  - simulated confirmation binds selected inputs and created outputs
- [ ] extend `crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - admission receipts rehydrate with tx journal rows

Exit condition:

- RPC broadcast can no longer mark a tx accepted or pending without a
  `WalletTxAdmitter` result and stored admission evidence.

### 044-07 Storage-Backed Wallet Reconciliation

Embedded requirement anchors:

- `PH44-RECONCILE`
- `Confirmation Receipt And Reconciler`
- `Simulated Validator Boundary`
- `Implementation Guide / Phase Gate 4A`

Task-local requirement summary:

- wallet finalization scans typed storage/checkpoint evidence;
- `SpentEnt` and `CreatedEnt` coverage must match journal expectations before
  state transitions;
- scan cursor progress is never proof of tx validation.

- [ ] Add a narrow `WalletTxReconciler` or equivalent service that scans
  pending and admitted journal rows.
- [ ] Query storage/checkpoint adapters for typed tx-effect evidence rather
  than ad hoc file or cursor parsing.
- [ ] Match pending txs by tx hash, chain id, checkpoint height, previous root,
  new root, expected input ids, expected output ids, output roles, and journal
  expectations before mutating wallet state.
- [ ] Require `SpentEnt` coverage for every selected sender input before
  marking inputs `Spent`.
- [ ] Require `CreatedEnt` coverage for every sender change and receiver output
  before moving outputs to `Available`.
- [ ] Route receiver-owned output finalization through
  `recv_route(..., ReceiveNext::PersistClaim)`.
- [ ] Update persistent tx records from `Pending` to `Confirmed` only after all
  required asset transitions commit.
- [ ] Leave rows pending when evidence is absent, and fail closed or quarantine
  only when evidence conflicts with journal expectations.
- [ ] Make reconciliation resumable and idempotent across repeated scans.

Files:

- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
- `crates/z00z_wallets/src/persistence/assets/asset_storage.rs`
- `crates/z00z_wallets/src/persistence/assets/asset_storage_impl.rs`
- `crates/z00z_wallets/src/persistence/scans/storage.rs`
- `crates/z00z_wallets/src/persistence/scans/storage_impl.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
- `crates/z00z_wallets/src/tx/state/lifecycle.rs`
- `crates/z00z_storage/src/checkpoint/build.rs`
- `crates/z00z_storage/src/checkpoint/exec_input.rs`
- `crates/z00z_storage/src/checkpoint/artifact_types.rs`
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
- `crates/z00z_storage/src/assets/store_internal/store_rows.rs`

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - no matching evidence leaves rows pending
  - matching sender spend and change evidence finalizes exactly once
  - wrong root or missing output evidence fails closed
- [ ] extend `crates/z00z_wallets/src/tx/state/test_state_update_suite.rs`
  - checkpoint evidence includes spent and created entity coverage
- [ ] extend `crates/z00z_wallets/tests/test_tx_balance.rs`
  - reconciliation updates Alice and Bob balances only after evidence matches

Exit condition:

- A storage-backed scan can turn pending tx effects into `Spent`,
  `Available`, and `Confirmed` without inventing `Validated`, using scan cursor
  state as proof, or double-applying checkpoint evidence.

### 044-08 Receiver Pending, Report-Only, And Finalization Behavior

Embedded requirement anchors:

- `D-044-005`, `D-044-006`
- `Receiver State Machine`
- `PH44-RECEIVE`
- `Receiver Flow`
- `Implementation Guide / Phase Gate 5`

Task-local requirement summary:

- report-only receive detects but does not persist claims or balances;
- verified receiver-owned package outputs may become pending receive rows;
- final receiver persistence uses `recv_route(..., ReceiveNext::PersistClaim)`.

- [ ] Keep `receive_asset_impl(...)` report-only and make the non-persistence
  contract explicit in response metadata or docs.
- [ ] Ensure report-only detection does not update claimed assets, asset
  storage, tx journal balances, pending receive rows, or available balance.
- [ ] Add receiver pending rows for verified tx packages or imports that are
  not yet spendable.
- [ ] Finalize receiver pending rows only through
  `recv_route(..., ReceiveNext::PersistClaim)`.
- [ ] Record claim reservation, claim finalization, quarantine, and release
  evidence.
- [ ] Prevent duplicate finalization by output id, serial id, claim nullifier,
  and tx output row.
- [ ] Classify import origin and confirmation source honestly; do not silently
  treat arbitrary DTO import as chain-confirmed unless existing policy
  explicitly allows it.

Files:

- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs`
- `crates/z00z_wallets/src/receiver/scan/types_receive.rs`
- `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs`

Tests:

- [ ] extend `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`
  - report-only receive returns detection without persistence
  - duplicate claim finalization fails closed
- [ ] extend `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
  - `ReportOnly` and `PersistClaim` stay behaviorally distinct
  - pending receiver rows finalize only through canonical persist-claim route
- [ ] extend `crates/z00z_wallets/tests/test_tx_balance.rs`
  - Bob pending receive is non-spendable until confirmation and persistence

Exit condition:

- Bob can preview an owned output without persistence, store a verified
  pending receive without making it spendable, and finalize exactly once through
  the canonical claim route after confirmation evidence.

### 044-09 Balance And User-Facing Lifecycle Views

Embedded requirement anchors:

- `D-044-008`, `D-044-010`
- `PH44-BALANCE`
- `PH44-HISTORY`
- `Implementation Guide / Phase Gate 6`

Task-local requirement summary:

- public balance keeps `total`, `available`, and `pending`;
- `available` includes confirmed spendable wallet value only;
- `pending` is derived from non-spendable lifecycle rows, never forced to zero.

- [ ] Replace `pending = 0` with ledger-derived pending calculation.
- [ ] Compute `available` from confirmed spendable wallet assets only.
- [ ] Compute `pending` from `Reserved`, `Exported`, `ClaimPending`,
  `PendingSpend`, `PendingChange`, `PendingReceive`, and `ReorgPending` values
  for the queried asset class.
- [ ] Keep `total = available + pending` for compatibility unless this backlog
  is updated with a public API expansion.
- [ ] Preserve an internal breakdown for reserved, exported, claim-pending,
  pending-spend, pending-change, pending-receive, reorg-pending, quarantined,
  and spent values.
- [ ] Return a typed diagnostic when pending rows cannot be decoded or joined to
  tx journal entries instead of reporting `pending = 0`.
- [ ] Ensure tx details and internal diagnostics expose the richer breakdown
  without changing compact public balance compatibility prematurely.

Files:

- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/asset.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_history.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_tx_balance.rs`
  - Alice pending outgoing excludes reserved input from available
  - Alice pending change is pending until confirmation
  - Bob pending receive is pending until confirmation
  - confirmed balances match final lifecycle state
- [ ] extend `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`
  - compact public balance remains `total`, `available`, and `pending`
  - malformed pending rows produce diagnostics instead of fake zero pending

Exit condition:

- Alice and Bob balance responses change predictably at build, export, import,
  submit, confirm, cancel, fail, and reconcile transitions.

### 044-10 Regression Matrix And Source-Shape Guards

Embedded requirement anchors:

- `Acceptance Criteria`
- `Validation Strategy`
- `Required Test Scenarios`
- `Risk Watchpoints`
- `No Logical Weak Spots`
- `Implementation Guide / Phase Gate 7`
- `Completion Definition`
- `P-044-004`, `P-044-010`, `P-044-011`, `P-044-012`

Task-local requirement summary:

- all acceptance criteria and required scenarios embedded above must have test
  coverage;
- source-shape guards must reject both old RPC stubs and the old per-tx JSON
  live-store model.

- [ ] Implement or update focused tests for `T-044-001` through `T-044-024`.
- [ ] Implement or update focused tests for `PT-044-001` through `PT-044-016`.
- [ ] Ensure every `AC-044-*` row is represented in `044-coverage.md` with a
  test home and execution evidence slot.
- [ ] Ensure every existing test file in the `Existing Test Impact Matrix` is
  updated or explicitly marked as no-change evidence in `044-coverage.md`.
- [ ] Add source-shape guards for `BuiltTxStub`, `pending = 0`, empty tx
  details, RPC-local fake success, and forbidden receive persistence drift.
- [ ] Add source-shape guards for per-tx JSON live storage, backup collection
  from per-tx JSON, and live `wallet_tx_history_dir(...)` usage.
- [ ] Run the bootstrap gate before broader validation:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
- [ ] Format the wallet crate with `cargo fmt -p z00z_wallets`.
- [ ] Run focused wallet tests for balance, tx store integration, and stealth
  output:
  `cargo test -p z00z_wallets --features test-fast --test test_tx_balance -- --nocapture`,
  `cargo test -p z00z_wallets --features test-fast --test test_tx_store_integration -- --nocapture`,
  and `cargo test -p z00z_wallets --features test-fast --test test_stealth_output -- --nocapture`.
- [ ] Run focused module tests for tx send, tx broadcast, tx pending, tx
  history, asset impl, wallet service, asset selection, and state update
  coverage.
- [ ] Run the release-style wallet gate:
  `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump`.
- [ ] Run simulator validation when touched:
  `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
  and `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`.
- [ ] If Phase 044 touches `z00z_storage`, add and run focused storage tests
  for checkpoint execution and store-op evidence before the wallet release
  gate.
- [ ] Record all commands, outputs, remaining risks, and deferrals in
  `.planning/phases/044-wallet-assets/044-SUMMARY.md`.

Release blocker checklist:

- [ ] reject fake public-input amount recovery;
- [ ] reject `mark_spent_for_wallet(...)` before confirmation;
- [ ] reject any second receive persistence path;
- [ ] reject report-only receive mutating claimed assets or balance;
- [ ] reject accepted sender flows that reach a raw output builder where a
  validated builder is required;
- [ ] reject tx details responses that fabricate rows from incomplete metadata;
- [ ] reject RPC methods that report broadcast success without admission trait
  evidence;
- [ ] reject wording that describes simulated admission as real chain
  consensus;
- [ ] reject sender-only broadcast authority over receiver submission for the
  same tx bytes;
- [ ] reject offline package export that includes wallet secrets, decrypted
  packs, private blindings, or plaintext seed material;
- [ ] reject local cancellation that unlocks exported inputs without
  no-admission proof;
- [ ] reject duplicate sender/receiver submission creating two admission
  records for one tx hash;
- [ ] reject scanners that turn pending rows spendable without matching
  checkpoint/storage evidence;
- [ ] reject `Validated` as a spendable wallet asset status;
- [ ] reject direct `std::fs`, raw serde, raw time, raw logging, metrics,
  config, or RNG in new business logic unless an existing documented exception
  applies.

No-logical-weak-spots checklist:

- [ ] do not spend from `ClaimPending`, `PendingReceive`, `PendingChange`,
  `Reserved`, `Exported`, `PendingSpend`, or `ReorgPending`;
- [ ] do not count `Quarantined`, `Dropped`, `Failed`, or `Spent` as
  available;
- [ ] do not create fee metadata without fee output semantics;
- [ ] do not create change outside sender-owned receiver controls;
- [ ] do not store receiver preview results as claims;
- [ ] do not finalize receiver claims without
  `recv_route(..., ReceiveNext::PersistClaim)`;
- [ ] do not use the tx RPC metadata store as the only source for details;
- [ ] do not leave tx bytes unlinked from journal records;
- [ ] do not translate portable package bytes into a second tx schema during
  import;
- [ ] do not reject receiver-side submission because the tx originated from
  sender build/export;
- [ ] do not release exported sender inputs just because the sender did not
  submit first;
- [ ] do not rename final spendable wallet value to `Validated`; use
  `Available` after confirmation evidence is reconciled;
- [ ] do not treat scan cursor progress as proof that a tx effect was
  validated;
- [ ] do not add a broad wallet database if a narrow lifecycle index over
  existing claimed assets is sufficient;
- [ ] do not bypass Phase 043 authority seams to make Phase 044 appear complete
  faster.

Required output checklist:

- [ ] `.planning/phases/044-wallet-assets/044-coverage.md`;
- [ ] updated or new focused wallet asset lifecycle tests;
- [ ] updated existing tests named in the `Existing Test Impact Matrix`, or
  explicit no-change evidence for each one;
- [ ] new focused tests for `PT-044-001` through `PT-044-016`;
- [ ] updated sender RPC build, send, broadcast, and details behavior;
- [ ] updated receiver report-only versus persist behavior where needed;
- [ ] portable tx package export and receiver import behavior;
- [ ] role-neutral sender/receiver submission path using `WalletTxAdmitter`;
- [ ] updated balance and tx-history semantics;
- [ ] JSONL-backed live tx-history store at `wallet_<stem>_tx_history.jsonl`;
- [ ] backup and restore preserve exact live JSONL history bytes plus manifest;
- [ ] legacy per-tx JSON migration path with no automatic deletion;
- [ ] simulated admission adapter with explicit receipts;
- [ ] storage-backed wallet reconciliation scanner with idempotent pending-to-
  final transitions;
- [ ] source-shape guards for removed stubs and fake success paths;
- [ ] source-shape guards for rejected per-tx JSON live storage;
- [ ] `.planning/phases/044-wallet-assets/044-SUMMARY.md` after implementation.

Files:

- `crates/z00z_wallets/src/tx/state/test_state_update_suite.rs`
- `crates/z00z_wallets/src/tx/selection/test_asset_selector_suite.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_body.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
- `crates/z00z_wallets/tests/test_tx_balance.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- `crates/z00z_wallets/tests/test_stealth_output.rs`
- all existing test files listed in the `Existing Test Impact Matrix`
- `.planning/phases/044-wallet-assets/044-coverage.md`
- new `.planning/phases/044-wallet-assets/044-SUMMARY.md`

Tests:

- [ ] run spec-exact Phase Gate 7 source-shape guard:

```bash
rg -n "BuiltTxStub|pending = 0|inputs: vec!\[\]|outputs: vec!\[\]" crates/z00z_wallets/src/adapters/rpc
```

- [ ] run source-shape guard:
  `rg -n "BuiltTxStub|pending = 0|inputs: vec!\\[\\]|outputs: vec!\\[\\]" crates/z00z_wallets/src/adapters/rpc`
- [ ] run source-shape guard:
  `rg -n "simulate retry success|status: TxStatus::Pending" crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- [ ] run source-shape guard:
  `rg -n "ReceiveNext::ReportOnly" crates/z00z_wallets/src/adapters/rpc/methods crates/z00z_wallets/src/services/wallet/actions`
- [ ] run JSONL live-store guard:
  `rg -n "wallet_\\{wallet_stem\\}_tx_history|format!\\(\\\"\\{tx_hash\\}\\.json\\\"\\)|collect_tx_history_records|wallet_tx_history_dir" crates/z00z_wallets/src`
- [ ] run patch-exact JSONL live-store guards:

```bash
rg -n "wallet_\\{wallet_stem\\}_tx_history\"" crates/z00z_wallets/src
rg -n "format!\\(\"\\{tx_hash\\}\\.json\"\\)" crates/z00z_wallets/src/persistence/tx
rg -n "collect_tx_history_records" crates/z00z_wallets/src/services/wallet
rg -n "wallet_tx_history_dir" crates/z00z_wallets/src
```

- [ ] run the required commands embedded in this task and record evidence in
  `044-SUMMARY.md`

Exit condition:

- Guards either return no forbidden live matches or every remaining match is
  explicitly documented as a test fixture or compatibility-only string, and the
  Phase 044 summary records the passing validation evidence.

## ✅ Completion Gate

Phase 044 is complete only when all of the following hold:

- `044-coverage.md` maps every `EV-044-*`, `D-044-*`, `PH44-*`, and
  `AC-044-*` identifier to owner files, test homes, and evidence slots;
- all tasks `044-01` through `044-10`, including `044-04A` through `044-04D`,
  are implemented or explicitly deferred by a backlog update;
- Alice sender assets have reservation, pending, confirmation, cancellation,
  failure, export, and rollback semantics;
- Bob receiver assets have report-only, import, pending receive, persist claim,
  quarantine, duplicate, and confirmation semantics;
- `.wlt` remains wallet snapshot and restore state only, with no tx package
  history storage;
- `wallet_<stem>_tx_history.jsonl` is the only live tx package history store
  for the wallet stem;
- every package-bearing JSONL row preserves exact canonical tx package bytes
  and validates tx bytes hash, record hash, sequence, and entry hash;
- backup preserves exact live JSONL bytes plus manifest, and restore writes
  JSONL back instead of expanding txs into per-hash JSON files;
- legacy `wallet_<stem>_tx_history/` directories are migration input only and
  are not deleted automatically;
- build, send, broadcast, details, pending list, history, and balance RPCs no
  longer rely on stub tx bytes, empty detail arrays, fake success, or fake
  pending values;
- portable package export/import keeps canonical tx bytes and allows sender or
  receiver submission through the same admission trait without sender priority;
- reconciliation finalizes wallet state as `Available`, `Spent`, and
  `Confirmed` without introducing `Validated`;
- no duplicate tx assembler, verifier, receive persistence path, output builder
  facade, wallet asset authority, admission path, or tx schema exists;
- required tests and source-shape guards pass and are recorded in
  `044-SUMMARY.md`;
- every `T-044-*`, every `PT-044-*`, and every existing test file named in the
  `Existing Test Impact Matrix` has passing evidence or an explicit no-change
  evidence note in `044-coverage.md`;
- closeout language claims wallet-level lifecycle closure only and keeps public
  proof-of-knowledge, production consensus, and real network broadcast outside
  the Phase 044 claim.
