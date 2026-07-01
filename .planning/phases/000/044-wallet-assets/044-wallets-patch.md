# 044 Wallet Tx History Store Patch

Canonical inputs:

- [044-wallets-assets-spec](./044-wallets-assets-spec.md)
- [044-TODO](./044-TODO.md)

Status:

- This patch is a correction overlay for Phase 044 tx-history, backup, and
  forensic package handling.
- Apply this patch before implementing `044-04`, `044-05`, backup restore
  integration, or any tx-history persistence changes.

## 🎯 Reader And Action

Reader:

- the engineer implementing Phase 044 wallet transaction persistence;
- the engineer reviewing backup/restore behavior for wallet tx-history;
- a future agent deciding where tx packages must be stored.

After reading, the engineer must be able to refactor wallet tx-history so that:

- tx packages are not stored inside `.wlt`;
- Phase 044 does not introduce a new database;
- `wallet_<stem>_tx_history.jsonl` is the canonical live tx-history store for
  that wallet stem;
- backup and restore preserve the live tx-history file instead of rebuilding it
  from per-transaction JSON files.

## 🧭 Corrected Understanding

The wallet must persist every transaction package somewhere durable. Storing tx
packages inside `.wlt` is the wrong boundary because `.wlt` would grow without
limit and would mix wallet identity/secret snapshot concerns with high-churn
transaction history.

The corrected storage boundary is:

- `.wlt` stores the wallet snapshot, identity, encrypted seed material, and
  wallet-level state that belongs to wallet restore;
- `wallet_<stem>_tx_history.jsonl` stores that wallet stem's live transaction
  history;
- each created, imported, exported, submitted, admitted, failed, cancelled, or
  confirmed transaction must be recoverable from the JSONL history;
- canonical tx package bytes must be stored exactly as the wallet received or
  built them, including encrypted package fields and encrypted asset data that
  are part of the canonical transaction package;
- decrypted wallet secrets, private blindings, plaintext seed material, and
  non-package private wallet state must not be added to the history file.

This means `wallet_<stem>_tx_history.jsonl` is not merely a backup export and is
not a sidecar generated from another live store. It is the live store.

## 🚫 Current Misinterpretation To Remove

The incorrect model is:

- `wallet_<stem>_tx_history/` is treated as a live tx store;
- `TxStorageImpl` stores one JSON file per transaction hash inside that
  directory;
- `wallet_<stem>_tx_history.jsonl` is treated as a canonical history export
  generated from the live store;
- `create_backup()` collects records from the per-tx JSON directory and then
  writes a JSONL sidecar near `.wlt`;
- restore imports forensic history by expanding records back into the per-tx
  JSON directory.

That model is backwards for Phase 044. It creates an unnecessary second storage
shape, makes the JSONL artifact derivative instead of authoritative, and makes
backup responsible for reconstructing history from implementation detail files.

## 🔑 Target Model

`wallet_<stem>_tx_history.jsonl` is the canonical append-ordered wallet tx
history file.

Required properties:

- one wallet stem has one canonical tx-history JSONL file;
- new tx activity writes directly to that JSONL file;
- no new database is introduced for Phase 044 tx packages;
- new wallets must not create `wallet_<stem>_tx_history/` per-tx JSON
  directories;
- reads fold the JSONL entries into the current wallet tx view;
- forensic recovery can replay JSONL entries in file order to recover the order
  of wallet tx actions;
- every package-bearing entry stores the exact canonical tx package bytes and
  a hash of those bytes;
- tx details, pending lists, history, backup, restore, export, import,
  admission, and reconciliation all read from or write to the same JSONL-backed
  tx-history abstraction.

For Phase 044, the JSONL file may be implemented as a file-backed store, not as
a broad database. If future scale requires indexes, add a separate future spec.

## ⚙️ JSONL Entry Contract

The current `WalletTxHistoryJsonlEntry` concept should become the live history
row contract, not an export-only row.

Minimum fields:

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
```

Minimum entry kinds:

```rust
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

Rules:

- `record.tx_bytes` must contain the complete canonical tx package bytes when a
  package exists;
- `tx_bytes_hash` must hash the exact bytes in `record.tx_bytes`;
- `record_hash` must hash the serialized record;
- `entry_hash` must cover the entry without `entry_hash` plus
  `previous_entry_hash` so the file is tamper-evident in order;
- `previous_entry_hash` links to the previous physical JSONL row for the same
  wallet history file;
- `sequence` is monotonically increasing within one wallet history file;
- `record.tx_hash` and top-level `tx_hash` must match;
- status updates append a new row rather than mutating or deleting older rows;
- `delete()` in the `TxStorage` trait should append a `Tombstoned` row for
  forensic history, not physically erase the package, unless a future explicit
  purge policy is specified.

The read model folds rows by `tx_hash`:

- `get(tx_hash)` returns the latest non-tombstoned record for that hash;
- `list()` returns latest non-tombstoned records;
- `list_by_status(status)` folds first, then filters by current status;
- details/history APIs can optionally expose the event trail for a tx, but the
  default public view remains the latest current record plus journal metadata.

## 🛠️ Step-By-Step Patch Plan

### ✅ Step 1: Rename The Authority In The Plan

Update Phase 044 planning language so the authority is unambiguous:

- replace any "JSONL export" wording with "live tx-history JSONL store" where
  the wallet runtime store is meant;
- keep "export" only for user-requested backup/export copies;
- make `wallet_<stem>_tx_history.jsonl` the named canonical path;
- mark `wallet_<stem>_tx_history/` as legacy migration input only.

### ✅ Step 2: Refactor The Path Contract

Keep the existing wallet stem naming convention.

Required path behavior:

- `wallet_<stem>.wlt` remains the wallet snapshot file;
- `wallet_<stem>_tx_history.jsonl` becomes the live tx-history file;
- `wallet_<stem>_tx_history/` must not be created for new writes;
- any helper named like `wallet_tx_history_dir` must be removed, renamed, or
  restricted to legacy migration only;
- any helper that returns the JSONL path should be the only path used by the
  tx-history store.

### ✅ Step 3: Rebuild `TxStorageImpl` Around JSONL

Change `TxStorageImpl` from a directory-of-JSON-files store into a
single-JSONL-file store.

Required behavior:

- constructor accepts the JSONL file path, not a directory path;
- `put(record)` appends a `Created`, `Imported`, or generic upsert row with the
  full `TxRecord`;
- `update_status(tx_hash, status)` reads the latest record, updates only the
  current status and timestamp fields, then appends a status transition row;
- `delete(tx_hash)` appends a `Tombstoned` row;
- `get`, `list`, and `list_by_status` read and fold the JSONL file;
- invalid tx hash labels are rejected before writing;
- duplicate tx hashes are legal as multiple lifecycle rows but must fold
  deterministically by sequence;
- malformed rows fail closed unless a future recovery mode explicitly allows
  partial salvage;
- all file I/O uses project I/O abstractions.

Atomicity rule:

- write each update by reading the existing file, appending one serialized line
  in memory, writing to a temporary sibling file, and renaming it into place;
- do not append directly to the live file until the project has an explicit
  crash-safe append abstraction;
- guard writes with a per-wallet file lock or an existing wallet session lock so
  concurrent writers cannot interleave sequences.

### ✅ Step 4: Preserve The Full Tx Package

The JSONL store must keep the package bytes, not a reduced view.

Required behavior:

- sender build stores the package bytes produced by the canonical assembler;
- receiver import stores the exact imported package bytes after validation;
- admission stores receipts and lifecycle status without replacing tx package
  bytes with a summary;
- confirmation stores evidence and final status without replacing tx package
  bytes with a summary;
- encrypted tx fields and encrypted asset packs that are part of the canonical
  tx package remain present in `record.tx_bytes`;
- non-package wallet secrets are never added to the record.

Any adapter that needs details should derive display rows from journal metadata
and the stored tx package, but must not treat derived display rows as a
replacement for the original package bytes.

### ✅ Step 5: Fix Backup Semantics

`create_backup()` must not build the canonical JSONL file from another live
store.

Correct behavior:

- read `.wlt` or build the wallet export pack for wallet snapshot backup;
- read the existing live `wallet_<stem>_tx_history.jsonl` bytes;
- validate the JSONL file before including it in a forensic backup payload;
- include the exact JSONL bytes plus a manifest in the encrypted backup
  archive;
- do not make `Vec<TxRecord>` the forensic archive authority; folded records
  are a view derived from JSONL, not the canonical backup payload;
- if a backup-side JSONL sidecar is requested, copy the live JSONL bytes to the
  backup destination and verify the copied hash;
- do not write or overwrite the wallet-directory JSONL as part of backup
  creation unless the file is missing and an empty live history file must be
  initialized;
- do not collect history from `wallet_<stem>_tx_history/` for normal backup.

Backup output should be a snapshot of two authorities:

- encrypted wallet snapshot data from `.wlt` semantics;
- encrypted forensic tx-history data from the live JSONL semantics.

### ✅ Step 6: Fix Restore Semantics

Restore must recreate the live JSONL file, not per-tx JSON files.

Correct behavior:

- `WalletOnly` restores only the wallet snapshot and does not invent tx
  history;
- `WalletPlusHistory` restores the wallet snapshot and writes the forensic
  JSONL history to the restored wallet stem's live JSONL path;
- `TxHistoryOnly` writes only the tx-history JSONL for forensic inspection and
  must not create `.wlt`;
- restore validates manifest hashes, row hashes, tx package hashes, sequence,
  chain identity, and wallet identity before writing;
- restored JSONL must preserve exact tx package bytes;
- restored JSONL should be written from the archived JSONL bytes, not
  reconstructed from extracted record structs;
- restored JSONL may rewrite `wallet_stem` only if the restored wallet receives
  a new stem, but it must preserve original wallet id, tx hash, package bytes,
  event order, and forensic hashes in explicit fields.

### ✅ Step 7: Migrate Legacy Per-Tx JSON Directories

The migration exists only to rescue data already written by the incorrect
implementation.

Migration behavior:

- if `wallet_<stem>_tx_history.jsonl` exists, treat it as authoritative;
- if only `wallet_<stem>_tx_history/` exists, read every valid per-tx JSON
  record;
- sort legacy records by timestamp, then tx hash for deterministic migration;
- write a JSONL file with `Migrated` entries that preserve each record's
  `tx_bytes`;
- include a migration summary row or metadata field with the old directory hash
  set;
- do not delete the legacy directory automatically;
- after successful migration, new writes go only to JSONL.

No destructive cleanup belongs in Phase 044. A future explicit cleanup task may
archive or trash legacy directories after operator confirmation.

### ✅ Step 8: Update RPC And Wallet Views

After the storage refactor:

- tx details read the folded latest record plus tx journal metadata;
- pending tx lists fold JSONL records and lifecycle rows;
- tx history reads from JSONL and sorts by recorded timestamp or sequence;
- balance derives pending/available values from wallet lifecycle state, not from
  JSONL alone;
- backup reads JSONL directly;
- restore writes JSONL directly;
- portable package export/import writes JSONL directly;
- simulated admission and reconciliation append lifecycle rows.

### ✅ Step 9: Add Source-Shape Guards

Add guards that fail if the old model returns.

Required checks:

```bash
rg -n "wallet_\\{wallet_stem\\}_tx_history\"" crates/z00z_wallets/src
rg -n "format!\\(\"\\{tx_hash\\}\\.json\"\\)" crates/z00z_wallets/src/persistence/tx
rg -n "collect_tx_history_records" crates/z00z_wallets/src/services/wallet
rg -n "wallet_tx_history_dir" crates/z00z_wallets/src
```

Expected result after the patch:

- no new-write code path uses `wallet_<stem>_tx_history/`;
- no `TxStorageImpl` code writes one JSON file per tx hash;
- backup no longer collects records from per-tx JSON files;
- any remaining matches are legacy migration code or tests that explicitly name
  the rejected shape.

### ✅ Step 10: Add Tests

Required tests:

- creating a wallet tx creates or updates `wallet_<stem>_tx_history.jsonl` and
  does not create `wallet_<stem>_tx_history/`;
- `TxStorageImpl::put` writes a JSONL row containing the full `tx_bytes`;
- `TxStorageImpl::update_status` appends a second row and folded reads return
  the latest status;
- `TxStorageImpl::delete` appends a tombstone and folded reads hide the tx by
  default;
- corrupted `tx_bytes_hash`, `record_hash`, or `entry_hash` is rejected;
- backup embeds or copies the exact live JSONL bytes and does not rebuild the
  file from per-tx JSON;
- restore with `WalletPlusHistory` recreates the JSONL file and preserves exact
  tx package bytes;
- restore with `TxHistoryOnly` creates only JSONL history and no `.wlt`;
- legacy per-tx JSON migration preserves tx packages and does not delete the
  legacy directory;
- source-shape guards reject reintroduction of per-tx JSON live storage.

## 🚨 Non-Negotiable Invariants

- `.wlt` must not become the tx package history store.
- `wallet_<stem>_tx_history.jsonl` must not be a derivative backup artifact.
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

## ✅ Acceptance Delta

Phase 044 is not complete unless these additional checks pass:

- `wallet_<stem>_tx_history.jsonl` is the only live tx package store for the
  wallet stem;
- every transaction package built or imported by the wallet is represented in
  JSONL with exact package bytes and hash verification;
- no new Phase 044 tx package is stored inside `.wlt`;
- no new Phase 044 tx package is stored as a standalone
  `wallet_<stem>_tx_history/<tx_hash>.json` file;
- backup and restore preserve JSONL history without changing package bytes;
- folded tx-history reads remain compatible with existing `TxStorage` callers;
- forensic replay can reconstruct action order from JSONL sequence and entry
  hashes.
