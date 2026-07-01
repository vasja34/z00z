# Phase 044 Attack Surface Report

**Rechecked:** 2026-05-12
**Doublecheck Mode:** workspace-first live-code verification
**Result:** fixed after narrowing one stale broad claim to one real backup-export lock gap

## ✅ Scan Result

The original report admitted one candidate. Doublecheck found that the broad claim is now stale for normal `TxStorageImpl` mutation paths because live code uses the canonical tx-history path mutex around read-modify-write operations. A narrower residual issue remained in backup export: `collect_tx_history_jsonl()` initialized and read the same live JSONL path without acquiring that canonical mutex. That residual issue is now fixed.

### 🚫 Rejected Alternative

- User-controlled backup destination placement remains rejected. It can change where a backup file is written, but it does not create a boundary-crossing integrity break in the canonical live tx-history store.

## ✅ Attack Surface: Backup Export Bypassed The Canonical Tx-History Lock

**Status:** fixed
**Severity:** medium
**Confidence:** high
**Exploitability:** medium
**Category Domain:** availability
**Category CWE:** CWE-362
**Attack Class:** unsafe-serialization-of-sensitive-state
**Scope Level:** repo
**Scope Paths:** `crates/z00z_wallets`, `crates/z00z_utils`
**Boundary Slice:** replay, uniqueness, and state-consumption slice
**Protected Asset:** canonical wallet tx-history integrity and backup snapshot availability
**Trust Boundary:** authenticated wallet RPC and backup requests -> file-backed canonical JSONL store
**Attacker Capability Model:** an authenticated wallet user or operator can issue overlapping backup and wallet mutation requests for the same wallet
**Existing Control State:** complete for in-process wallet storage, backup export, and backup restore
**Main Vulnerability:** the previous broad finding said `TxStorageImpl` had no lock around full-file read-modify-write. Current live code contradicts that broad claim. The real residual issue was narrower: backup export used the same canonical JSONL path but performed missing-file initialization and read/decode without acquiring `tx_history_path_lock()`. If backup export observed a missing file while the first tx mutation was creating the same history file, the export path could still write an empty file outside the single-writer contract.

### 🔎 Threat Model Snapshot

- **Attacker Class:** authenticated wallet user or operator
- **Entry Point:** `wallet.backup.create_backup` interleaved with tx-history mutation paths
- **Sink:** `wallet_<stem>_tx_history.jsonl` via backup export initialization/read and `TxStorageImpl` mutation writes
- **Why This Path Is Realistic:** backup RPC is backed by a shared `Arc<WalletService>`, and backup creation packages the live JSONL history from the same file path used by tx mutations

### ⚙️ Live Code Evidence

- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs:27` defines `tx_history_path_lock(path)` as the canonical per-path mutex helper.
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs:165` defines `with_write_lock(...)`, and all live mutation paths call it before load/mutate/persist.
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs:228` and later `put`, `record_imported`, `record_exported`, `update_status`, `record_submitted`, `record_admitted`, `record_confirmed`, `record_cancelled`, and `delete` are serialized by `with_write_lock(...)`.
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs:89` now acquires `tx_history_path_lock(&live_path)` before backup export creates, reads, or validates the live JSONL file.
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs:162` acquires the same lock before restore writes imported JSONL bytes back to the live path.
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl.rs:24` keeps backup RPC backed by a shared `Arc<WalletService>`, preserving the original reachability model.
- `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs:49` keeps async dispatcher handlers cloning shared RPC state into request futures, preserving the concurrency model.

### 🧭 Doublecheck Correction

The original evidence cited stale function names and line coordinates such as `load_records()` and `persist_records()`. Current code uses `load_rows_unlocked()` and `persist_rows_unlocked()`, but these functions are internal helpers called under `with_write_lock(...)` for mutation paths. Atomic replacement remains important for torn-write protection, while the canonical mutex now protects in-process read-modify-write ordering.

### ✅ Defensive Implementation Contract

- Normal tx-history mutation paths use the canonical `TxStorageImpl` path mutex.
- Backup restore uses the same `tx_history_path_lock()` before replacing live JSONL bytes.
- Backup export now uses the same `tx_history_path_lock()` before parent-directory setup, missing-file initialization, read, and validation.
- Regression coverage proves `wallet.backup.create_backup` waits for the canonical tx-history lock before export can proceed.

### ✅ Validation

```bash
cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_backup_export_waits_for_tx_history_lock
```

Result: `1 passed; 0 failed; 1481 filtered out`.

### ⚠️ Residual Risk

The canonical lock is an in-process coordination control. It closes the wallet service and RPC concurrency surface covered by this report, but it is not an inter-process file lock. A separate OS process writing the same JSONL path outside `TxStorageImpl` and backup helpers would still bypass the contract and should remain out of scope unless Z00Z introduces multi-process wallet writers.
