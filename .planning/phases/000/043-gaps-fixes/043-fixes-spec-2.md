---
phase: 043-gaps-fixes
spec_id: 043-fixes-spec-2
status: planning-spec
created: 2026-05-07
updated: 2026-05-07
owner: Z00Z Wallets and Storage
scope: public conservation witness boundary, manual asset-class audit contracts, and forensic archive naming and transport rules
tags: [wallets, backup, audit, forensics, naming, spec]
---

# Phase 043 Gap Fix Specification 2

This specification formalizes the remaining partial points identified during the doublecheck pass.
It is intentionally strict: it preserves the current honest limits in the public verifier and only
adds new contracts where the current implementation is narrower than the requested behavior.

The goal is not to invent a second archive stack or a second verifier. The goal is to make the
current boundaries explicit, typed, and testable so that future code cannot drift back into vague
claims.

## 1. Purpose & Scope

This document covers four related concerns:

1. Public conservation claims and the boundary between a public verifier and a true conservation proof.
2. Manual asset-class audit contracts, including target selection, status reporting, and mismatch typing.
3. Forensic archive transport, import gating, and canonical JSONL tx-history output.
4. A naming contract that makes wallet snapshot artifacts and tx-history artifacts obviously related.

This spec applies to the wallet, backup, tx, and service layers in `crates/z00z_wallets`, and to the
storage membership seam in `crates/z00z_storage` only where those seams are already the source of
truth.

### Out of scope

- Modifying vendor code under `crates/z00z_crypto/tari/**`.
- Replacing the current JMT backend.
- Making canonical `.wlt` a mandatory tx-history database.
- Introducing a new config key that silently changes backup semantics.
- Introducing a second verifier, a second tx-history store, duplicate wallet/backup logic, or
  parallel planning and coverage artifacts for this slice.
- Claiming that wallet-local or operator-local auditing is a public trustless verifier.
- Rewriting unrelated wallet, storage, fee, or prover TODO items.

## 2. Verified Baseline

The current codebase already establishes the following truths:

- `verify_balance` routes through the public package verifier path, and the code is honest about the
  limits of that path.
- `WalletForensicPack` already exists inside the encrypted backup payload, and import modes already
  distinguish wallet-only, tx-history-only, and wallet-plus-history behavior.
- The encrypted forensic payload already carries the full tx-history record set as `Vec<TxRecord>`;
  there is no separate canonical plaintext forensic history file today.
- `TxStorageImpl` currently stores one JSON file per transaction, keyed by `tx_hash`.
- The live wallet tx-history store remains directory-based through `wallet_tx_history_dir(...)`;
  it is not the same thing as the forensic archive payload.
- `outputs/tx_exports` is a separate RPC export tree and is not the forensic archive.
- `PersistBackupSettings` does not currently contain a forensic toggle.
- `wallet_file_path`, `wlt_file_path`, and `wallet_tx_history_dir` already derive from the same
  wallet-id stem.
- `AssetClassAuditReport` exists, but the current audit helper is narrower than the requested report
  shape because mismatch classification currently lives only on the error path.

This spec does not dispute any of those facts. It builds on them.

## 3. Definitions

- **Public conservation proof**: A verifier claim that public package bytes alone prove conservation.
  This is stronger than the current honest verifier boundary and requires an explicit witness.
- **Resolved pre-state witness**: Explicit evidence that binds a public input reference to validated
  pre-state information that can support conservation checking.
- **Commitment proof**: Explicit evidence that proves a commitment equation rather than merely proving
  storage membership.
- **Manual asset-class audit**: An operator-invoked local diagnostic that aggregates validated leaves
  for one asset class and compares the recomputed total against a target.
- **Mismatch class**: A typed explanation of why an audit or import failed closed.
- **Outcome**: A typed result object that can carry both status and report data without collapsing
  fail-closed information into a bare error.
- **Forensic archive transport**: The encrypted backup payload path that carries the full forensic
  archive inside `BackupContainer` / `BackupPayload`.
- **Forensic history JSONL**: The canonical plaintext tx-history artifact written next to the wallet
  snapshot. It contains a full `TxRecord` replay payload or a validation snippet that is sufficient
  to rebuild that record set, while still remaining redacted of seed phrases and decrypted
  wallet-local secret material. If it carries `TxRecord.tx_bytes`, those bytes are the opaque
  serialized transaction bytes already modeled by `TxRecord`, not decrypted wallet secrets.
- **Live tx-history store**: The wallet-owned directory of per-transaction JSON records keyed by
  `tx_hash`; this is separate from the encrypted forensic archive and from RPC export output.
- **Wallet stem**: The stable wallet-id-derived hex name fragment used to keep wallet and history
  artifact names visibly related. The live code derives this fragment from
  `compute_wallet_file_id(&wallet_id.0)` and uses the first 8 bytes as 16 hex characters.

## 4. Requirements, Constraints & Guidelines

### 4.1 Public conservation witness boundary

- **REQ-PROTO-001**: If the system ever advertises a public proof of conservation for a public
  `TxPackage`, the public package MUST include an explicit conservation witness.
- **REQ-PROTO-002**: The conservation witness MUST be one of:
  - a resolved pre-state witness that binds public input references to validated pre-state leaves, or
  - a commitment proof that binds the package to a checked commitment equation.
- **REQ-PROTO-003**: Until such a witness exists, `verify_balance` MUST remain an honest package
  verifier and MUST report only the guarantees that the current verifier can actually make.
- **REQ-PROTO-004**: The system MUST NOT present public package bytes alone as a proof of hidden-input
  conservation.

### 4.2 Manual asset-class audit

- **REQ-AUD-001**: Add an `AssetClassAuditTarget` enum with at least these variants:
  - `ExpectedTotalCommitment`
  - `CheckpointEquation`
  - `IssuanceBurnDeltaTarget`
- **REQ-AUD-002**: Each audit target MUST resolve to a concrete expected total commitment before the
  recomputation is compared.
- **REQ-AUD-003**: Add an `AssetClassAuditStatus` enum so status is separated from report payload.
- **REQ-AUD-004**: Add an `AssetClassAuditOutcome` type that can return status, report, mismatch class,
  and optional entry index together.
- **REQ-AUD-005**: Extend `AssetClassAuditReport` so it preserves the current live fields and adds
  the missing target or mismatch fields. At minimum it must contain:
  - `asset_class`
  - `semantic_root`
  - `backend_root`
  - `root_bind`
  - `leaf_count` or an explicitly documented compatibility alias such as `verified_leaf_count`
  - `total_commitment` or an explicitly documented compatibility alias such as
    `recomputed_total_commitment`
  - `target`
  - `mismatch_class` as an optional field
- **REQ-AUD-006**: The manual asset-class audit MUST remain out-of-band from canonical tx admission.
  It MUST NOT become an implicit step of normal transaction verification.
- **REQ-AUD-007**: Every fail-closed audit path MUST set a typed mismatch class instead of collapsing
  all failures into one generic error.
- **REQ-AUD-008**: The audit helper MUST keep leaf-order, duplicate-entry, root-binding, asset-class,
  hash, and commitment failures distinguishable.

### 4.3 Forensic archive transport and import

- **REQ-ARC-001**: Canonical `.wlt` semantics MUST remain wallet snapshot state plus claimed assets
  unless a future spec explicitly changes that contract.
- **REQ-ARC-002**: The full forensic archive MUST continue to travel through the existing encrypted
  `BackupContainer` / `BackupPayload` seam.
- **REQ-ARC-003**: The system MUST NOT invent a second encrypted archive container stack.
- **REQ-ARC-004**: Importing a forensic archive MUST require explicit caller intent via import mode.
- **REQ-ARC-005**: A forensic archive section that fails manifest validation, serialized-record hash
  validation, tx-hash label validation, or metadata binding MUST be rejected before wallet state is
  mutated by tx-history import.
- **REQ-ARC-006**: The encrypted forensic archive MAY contain full `TxRecord` entries, including
  opaque `tx_bytes`, because that content stays inside the encrypted transport boundary.
- **REQ-ARC-007**: Any plaintext operator artifact outside the encrypted boundary MUST be hash-bound
  and secret-redacted. If the canonical JSONL file carries `TxRecord.tx_bytes`, those bytes MUST be
  treated as opaque serialized transaction bytes and MUST NOT include plaintext seed phrases,
  decrypted asset-pack fields, or other decrypted wallet-local secrets.
- **REQ-ARC-008**: There is no new persisted config flag for forensic enablement. Enablement MUST be
  driven by explicit constructor or explicit import mode selection such as
  `new_with_forensic_history(...)` on export and `ForensicImportMode::{WalletOnly,TxHistoryOnly,WalletPlusHistory}` on import.
- **REQ-ARC-009**: The canonical forensic archive MUST remain the encrypted backup payload, and the
  canonical plaintext tx-history JSONL MUST also be emitted as a required artifact next to the
  wallet snapshot.
- **REQ-ARC-010**: The canonical JSONL tx-history file MUST be derived from the same validated
  forensic record set as the encrypted archive and MUST use the same wallet stem as the snapshot
  export.
- **REQ-ARC-011**: The canonical JSONL tx-history file MUST be created as part of the normal forensic
  export path; it is not an optional add-on and its absence is a failure.
- **REQ-ARC-012**: The canonical JSONL tx-history file MUST contain the full transaction records or
  transaction snippets needed for forensic review, replay, validation, and audit checks.
- **REQ-ARC-013**: The canonical JSONL tx-history file MUST remain colocated with the `.wlt` file in
  the same output folder so both artifacts are immediately discoverable together.

### 4.4 Naming and layout

- **REQ-NAM-001**: Introduce one shared wallet-stem helper that derives the stable stem from the
  wallet id using the current `compute_wallet_file_id(&wallet_id.0)` convention.
- **REQ-NAM-002**: The canonical wallet snapshot file name MUST remain visibly wallet-specific,
  for example `wallet_<wallet_stem_hex>.wlt`.
- **REQ-NAM-003**: The canonical forensic history artifact name MUST be derived from the same wallet
  stem and MUST be created beside the `.wlt` file in the same folder, for example
  `wallet_<wallet_stem_hex>_tx_history.jsonl`.
- **REQ-NAM-004**: The canonical JSONL history file MUST be treated as the primary plaintext
  forensic history artifact for the wallet, not as an optional add-on or compatibility-only export.
- **REQ-NAM-005**: The legacy order `tx_history_<wallet_stem_hex>.jsonl` MUST NOT be introduced as
  the canonical filename order; it is superseded by `wallet_<wallet_stem_hex>_tx_history.jsonl`.
- **REQ-NAM-006**: The `outputs/tx_exports` tree MUST remain a separate RPC export tree and MUST NOT
  be conflated with forensic history.
- **REQ-NAM-007**: If multiple history versions need to coexist, the version or timestamp MUST be
  appended to the history stem, not embedded into the wallet identity itself.
- **REQ-NAM-008**: The shared wallet stem helper MUST feed the wallet snapshot name, the canonical
  JSONL history file name, and the live tx-history directory name so the artifacts are obviously
  related but not confused.

### 4.5 File placement rules

- **REQ-PATH-001**: The encrypted forensic archive MUST be written to the explicit backup export
  target path chosen by the caller; it is not automatically redirected into the RPC export tree.
- **REQ-PATH-002**: The canonical JSONL tx-history file MUST live beside the wallet snapshot export
  in the same output root and use the shared wallet stem.
- **REQ-PATH-003**: The live tx-history directory MUST remain the existing
  `wallet_<wallet_stem_hex>_tx_history` directory under the wallet output root, with one JSON file
  per transaction.
- **REQ-PATH-004**: The `outputs/tx_exports` tree MUST remain the separate summary-oriented RPC
  export path and MUST NOT become the forensic history location.
- **REQ-PATH-005**: The canonical JSONL tx-history path MUST be a file path such as
  `wallet_<wallet_stem_hex>_tx_history.jsonl`, not the live tx-history directory.

### 4.6 Logging and evidence discipline

- **REQ-SEC-001**: If forensic export/import or closeout evidence is generated, the system MUST NOT
  log or copy plaintext seed phrases, decrypted asset-pack fields, decrypted wallet-local secrets, or
  unredacted tx-history payloads into logs, summaries, or validation notes.
- **REQ-SEC-002**: Only the canonical JSONL tx-history artifact is allowed to carry plaintext
  tx-history replay data outside the encrypted archive, and that artifact must remain hash-bound and
  secret-redacted. All other evidence outside the encrypted archive must be redacted or hash-bound.

### 4.7 Concept-drift guardrails

- **REQ-DRIFT-001**: The spec MUST remain aligned with the current codebase baseline and MUST NOT
  silently reinterpret the public verifier as a public conservation proof.
- **REQ-DRIFT-002**: Any future change that strengthens the public conservation claim MUST be routed
  through a protocol-change spec first.
- **REQ-DRIFT-003**: Any future change that expands canonical tx-history storage MUST be treated as a
  storage contract change, not as a naming-only update.

## 5. Interfaces & Data Contracts

### 5.1 Asset-class audit contracts

The recommended shape is:

```rust
pub enum AssetClassAuditTarget {
    ExpectedTotalCommitment { expected_total: Z00ZCommitment },
    CheckpointEquation { expected_total: Z00ZCommitment, checkpoint_id: String },
    IssuanceBurnDeltaTarget { issued_total: Z00ZCommitment, burned_total: Z00ZCommitment },
}

pub enum AssetClassAuditStatus {
    Pass,
    FailClosed,
}

pub enum AssetClassAuditMismatchClass {
    MissingEvidence,
    RootMismatch,
    LeafMismatch,
    AssetClassMismatch,
    CommitmentMismatch,
    TargetMismatch,
    DuplicateEntry,
    HashMismatch,
    SerializationMismatch,
}

pub struct AssetClassAuditReport {
    pub asset_class: AssetClass,
    pub semantic_root: AssetStateRoot,
    pub backend_root: [u8; 32],
    pub root_bind: [u8; 32],
    pub leaf_count: usize,
    pub total_commitment: Z00ZCommitment,
    pub target: AssetClassAuditTarget,
    pub mismatch_class: Option<AssetClassAuditMismatchClass>,
}

pub struct AssetClassAuditOutcome {
    pub status: AssetClassAuditStatus,
    pub report: AssetClassAuditReport,
    pub mismatch_class: Option<AssetClassAuditMismatchClass>,
    pub entry_index: Option<usize>,
}
```

Implementation note:

- `AssetClassAuditReport` is the typed report.
- `AssetClassAuditOutcome` is the wrapper that keeps status and fail-closed diagnostics together.
- The same mismatch class may appear both in the report and the outcome if the implementation wants a
  compact summary plus a top-level diagnostic.

### 5.2 Forensic archive contracts

The encrypted archive already exists as `WalletForensicPack` inside `BackupPayload`. That contract
must remain the transport contract.

The canonical JSONL tx-history artifact is separate from the encrypted archive. Its recommended
record shape is:

```rust
pub struct WalletTxHistoryJsonlEntry {
    pub schema_version: u32,
    pub tx_hash: String,
    pub record_hash: [u8; 32],
    pub tx_bytes_hash: [u8; 32],
    pub record: TxRecord,
}
```

Rules for the canonical JSONL tx-history file:

- one JSON object per line;
- it contains the full tx record or a tx snippet payload that is sufficient to reconstruct the full
  `TxRecord` needed for forensic replay and validation;
- it must not contain seed phrases, decrypted asset-pack fields, or decrypted wallet secrets;
- it may omit only data that is not part of `TxRecord` and is not needed for standalone replay;
- it must remain a file, not a directory.

### 5.2.1 Canonical JSONL import/view contract

The wallet MUST provide a canonical JSONL import path for
`wallet_<wallet_stem_hex>_tx_history.jsonl` that replays the file into the live tx-history store.

Rules for that import/view path:

- it deserializes each JSONL line into a record set suitable for replay into `TxStorageImpl`;
- it preserves the full `TxRecord` view for inspection after replay, including `tx_hash`,
  `tx_bytes`, `status`, `timestamp_ms`, and `block_height`;
- it fails closed on malformed lines, missing fields, duplicate labels, or hash mismatches before
  any live-store write;
- it treats the JSONL file as plaintext decode/replay data; decryption applies only to the
  encrypted archive seam that can produce the same record set.

Hint for readers and implementers:

- replaying `wallet_<wallet_stem_hex>_tx_history.jsonl` restores plaintext forensic records, not a
  second encrypted archive boundary;
- replay alone does not grant universal decryption of every tx-local encrypted payload;
- sender-side interpretation remains limited to the existing sender-held self-validation/decode
  seams that can derive sender context such as `k_dh` and inspect the encrypted asset pack;
- receiver-side interpretation remains limited to the existing receiver-owned scan/reveal seams
  that can derive the receiver shared secret and recover wallet-local fields intended for that
  receiver, such as amount, blinding, `s_out`, and optional memo;
- this contract therefore promises canonical replay and full `TxRecord` visibility after import,
  while any deeper semantic decode of `tx_bytes` remains role-dependent and key-dependent.

### 5.3 Naming helpers

The following helper contract is recommended:

```rust
fn wallet_stem(wallet_id: &PersistWalletId) -> String;
fn wallet_snapshot_name(wallet_stem: &str) -> String;
fn wallet_history_jsonl_name(wallet_stem: &str) -> String;
fn wallet_tx_history_dir(wallet_id: &PersistWalletId) -> PathBuf;
```

Example outputs:

- `wallet_0a668c3eb2b9c86e.wlt`
- `wallet_0a668c3eb2b9c86e_tx_history.jsonl`
- `wallet_0a668c3eb2b9c86e_tx_history/`

If a timestamp or version suffix is needed, it belongs after the stem and before the extension.

## 6. Acceptance Criteria

### Public conservation witness

- Given a public `TxPackage` without a conservation witness, when `verify_balance` is called, then
  the verifier reports only its honest package-level limits.
- Given a public `TxPackage` with a valid conservation witness, when a future protocol verifier is
  called, then zero-net conservation is checked against that witness and not against public refs
  alone.

### Manual asset-class audit

- Given valid leaves, a selected asset class, and an expected target, when the audit runs, then the
  outcome status is `Pass` and the report includes the asset class, semantic root, backend root,
  root bind, leaf count, total commitment, target, and no mismatch class.
- Given any target mismatch, root mismatch, asset-class mismatch, duplicate entry, or hash mismatch,
  when the audit runs, then the outcome is `FailClosed`, the mismatch class is populated, and the
  entry index is present when the failure is entry-specific.

### Forensic archive

- Given `WalletPlusHistory` or `TxHistoryOnly` mode, when the importer reads a valid encrypted
  archive, then tx-history records are imported only after the archive validates.
- Given a tampered manifest entry, tampered record hash, tampered tx-bytes hash, or mismatched
  `tx_hash` label, when the importer validates the archive, then the tx-history section is rejected
  and wallet state remains unchanged.
- Given the canonical JSONL history artifact, when it is emitted, then it contains the full forensic
  transaction records/snippets needed for replay and validation and no plaintext seed phrase,
  decrypted asset-pack field, or decrypted wallet-local secret.
- Given the canonical JSONL history artifact, when the wallet imports or replays it into the live
  tx-history store, then all `TxRecord` fields remain viewable and tampered lines fail closed before
  mutation.

### Naming

- Given the same wallet id, when the wallet snapshot name, canonical JSONL history file, and live
  tx-history directory are derived, then all three artifacts share the same wallet stem.
- Given `outputs/tx_exports`, when the RPC export tree is produced, then it remains distinct from the
  forensic history artifact and keeps its own summary-oriented format.

## 7. Test Automation Strategy

This spec requires two layers of verification for every new contract: one source-shape test and one
behavioral test.

### Unit tests

Add or extend unit tests for:

- `AssetClassAuditTarget`
- `AssetClassAuditStatus`
- `AssetClassAuditOutcome`
- `AssetClassAuditReport`
- wallet stem helpers
- canonical JSONL history serialization
- no-plaintext leakage rules on any redacted/hash-bound operator artifact

### Integration tests

Add or extend integration tests for:

- encrypted forensic archive roundtrip
- import mode gating
- tamper rejection with no wallet-state mutation
- naming consistency between wallet file, canonical JSONL history file, and live tx-history directory helpers
- tx_exports distinctness from forensic history

### Negative tests

Every new fail-closed path MUST have a negative test for each of these cases where applicable:

- missing evidence
- root mismatch
- leaf mismatch
- asset-class mismatch
- duplicate entry
- commitment mismatch
- target mismatch
- tx-hash label mismatch
- hash mismatch on serialized record or `tx_bytes`

### Required test matrix

| Requirement | Minimum test coverage |
| --- | --- |
| No config toggle for forensic enablement | One unit test proves export/import selection is driven by constructor or import mode, not a persisted backup setting. |
| Encrypted forensic archive completeness | One integration test proves `WalletForensicPack.records` roundtrips as the full tx-history set. |
| Canonical JSONL naming and wallet stem sync | One unit test proves `wallet_snapshot_name`, the canonical JSONL history filename helper, and `wallet_tx_history_dir` all derive the same stem. |
| Canonical JSONL import and full-field view | One integration test proves the canonical JSONL file can be replayed into the live tx store and every `TxRecord` field remains viewable. |
| Separate archive vs live store vs RPC exports | One integration test proves the encrypted archive, live tx-history dir, and `outputs/tx_exports` are distinct paths. |
| No plaintext leakage | One redaction test proves no plaintext seed phrase, decrypted asset-pack field, or decrypted wallet-local secret appears outside the encrypted boundary, including in the canonical JSONL history file. |

### Recommended test files

- `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
- `crates/z00z_wallets/tests/test_tx_pedersen.rs`
- `crates/z00z_wallets/tests/test_tx_wrong_root.rs` only when it remains the truthful wrong-root
  audit home
- `crates/z00z_wallets/tests/test_tx_balance.rs` only when it remains the truthful public-verifier
  honesty home
- `crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs`
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
- `crates/z00z_wallets/tests/test_redb_wlt_open.rs` only for snapshot restore/load boundary checks
- `crates/z00z_wallets/tests/test_wallet_json_export.rs` only if that suite already owns the archive
  serialization seam more truthfully than a new dedicated test file
- a new focused test file for naming helpers if the existing suites become too broad

## 8. Concrete Patch Plan

This is the file-level patch map the implementation should follow.

| File | Required additions |
| --- | --- |
| `crates/z00z_wallets/src/tx/commit_audit.rs` | Add `AssetClassAuditTarget`, `AssetClassAuditStatus`, `AssetClassAuditMismatchClass`, and `AssetClassAuditOutcome`. Expand `AssetClassAuditReport` so it carries the target and optional mismatch class. Keep the helper out-of-band from canonical tx admission. |
| `crates/z00z_wallets/src/tx/mod.rs` | Re-export the new audit types so callers do not need to reach into a private module. |
| `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs` | Add a shared wallet-stem helper plus a canonical JSONL history filename helper. Keep `wallet_file_path` / `wlt_file_path` semantics stable and preserve the existing `wallet_tx_history_dir` helper for live storage. |
| `crates/z00z_wallets/src/backup/crypto/backup_wire.rs` | Keep `WalletForensicPack` as the encrypted archive contract. Add a canonical JSONL history entry type for the wallet-prefixed forensic history file. |
| `crates/z00z_wallets/src/backup/export/backup_exporter_impl.rs` | Write the canonical JSONL history file from the same forensic record set and use the same wallet stem. Reuse the existing encrypted backup transport seam. |
| `crates/z00z_wallets/src/backup/export/backup_exporter_verify.rs` | Keep exporter-side integrity verification as the symmetric validation seam for the encrypted archive. |
| `crates/z00z_wallets/src/backup/import/backup_importer_impl.rs` | Keep explicit mode gating. Add or extend the canonical JSONL replay path if the file is imported directly, and reject missing or malformed forensic sections before wallet-state mutation. |
| `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs` | Keep import/export routing explicit. If the canonical JSONL file is replayed or a filename is emitted for diagnostic purposes, derive it from the shared wallet stem. |
| `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs` | Do not change canonical tx-store semantics unless a separate storage migration spec is written. Keep one JSON file per tx here. |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs` | Leave `outputs/tx_exports` as a separate summary export tree. Add docs or tests only if the boundary needs to be made explicit. |
| `crates/z00z_wallets/tests/**` | Add focused positive and negative tests for the new outcome type, the new target enum, filename helpers, and tamper rejection. |

## 9. Rationale & Concept Drift Guardrails

The current code already draws three important boundaries:

1. storage membership is not conservation,
2. canonical wallet state is not tx history, and
3. a diagnostic helper is not a canonical admission rule.

This spec keeps those boundaries intact.

The new outcome type prevents a common mistake: returning a plain `Result<Report, Err>` when the user
still needs the report fields on fail-closed paths. The new target enum prevents another common
mistake: hardcoding one numeric comparison and then later trying to describe checkpoint or
issuance/burn cases with the same narrow API.

The naming contract is intentionally wallet-first. A filename like
`wallet_<wallet_stem_hex>_tx_history.jsonl` is obviously tied to the same wallet as
`wallet_<wallet_stem_hex>.wlt`, while still making it clear that the file is history, not snapshot
state. If compatibility ever requires a shorter alias, that alias must remain secondary.

The public conservation boundary is intentionally conservative. If a true public conservation proof is
ever needed, the protocol must change first. That change belongs in a separate spec and should add an
explicit witness field to the public package.

## 10. Dependencies & External Integrations

### Internal dependencies

- `BackupContainer`
- `BackupPayload`
- `WalletForensicPack`
- `WalletExportPack`
- `TxRecord`
- `TxStorageImpl`
- `verify_full_tx_package(...)`
- `TxVerifierImpl`
- `ProofBlob::decode(...)`
- `chk_item(...)`
- `chk_blob(...)`

### External dependencies

None are introduced by this spec. The current `z00z_utils` I/O, codec, and time abstractions remain
sufficient.

## 11. Examples & Edge Cases

### Example 1: canonical wallet snapshot plus history JSONL

```text
wallet_0a668c3eb2b9c86e.wlt
wallet_0a668c3eb2b9c86e_tx_history.jsonl
wallet_0a668c3eb2b9c86e_tx_history/
```

The snapshot file is the canonical wallet artifact. The JSONL file is the canonical forensic
history file for that wallet. The directory is the live tx-history store. They share the same wallet
stem and live in the same output root so a human can see that they belong together without
confusing the artifact types.

### Example 2: canonical JSONL history entry

```json
{"schema_version":1,"tx_hash":"tx-1","record_hash":"...","tx_bytes_hash":"...","record":{"tx_hash":"tx-1","tx_bytes":[1,2,3,4],"status":"Pending","timestamp_ms":1715000000000,"block_height":null}}
```

This is allowed only if the file is treated as secret-redacted and hash-bound replay data. The
`tx_bytes` field is the opaque serialized transaction payload from `TxRecord`; it must not contain
plaintext seed phrases, decrypted asset-pack fields, or decrypted wallet-local secrets.

### Example 3: fail-closed audit

If the audit finds a duplicate `tx_hash` label, the outcome must be `FailClosed`, the mismatch class
must be `DuplicateEntry`, and the report must still carry the selected asset class and root context.

### Example 4: public conservation honesty

A public package without a conservation witness may still pass the current package verifier, but it
must never be described as a full public proof of conservation.

## 12. Validation Criteria

This specification is satisfied only when all of the following are true:

- the audit API separates status from report;
- the audit API supports explicit targets beyond one numeric expected total;
- the forensic archive continues to use the encrypted backup seam;
- explicit import mode remains required for tx-history import;
- the canonical JSONL history file can be replayed into the live tx-history store and preserves the full `TxRecord` view;
- the canonical JSONL history file is required, colocated, and wallet-prefixed;
- wallet snapshot naming, JSONL history naming, and live tx-history directory naming share a
  helper-derived stem;
- `outputs/tx_exports` remains distinct from forensic history;
- no code path claims a public conservation proof without an explicit witness.
- `.planning/phases/043-gaps-fixes/043-coverage.md` remains the only execution ledger for this
  slice; no `043-coverage-2.md` or equivalent parallel ledger is created.
- `.planning/phases/043-gaps-fixes/043-SUMMARY.md` remains the only closeout summary for this
  slice; no `043-SUMMARY-2.md` or equivalent parallel summary is created.

## 13. Related Specs / Further Reading

- [043-fixes-spec.md](./043-fixes-spec.md)
- [043-TODO.md](./043-TODO.md)
- [043-TEST-SPEC.md](./043-TEST-SPEC.md)
- [043-coverage.md](./043-coverage.md)
- [043-10-VERIFICATION.md](./043-10-VERIFICATION.md)
- [crate-level backup exporter code](../../../crates/z00z_wallets/src/backup/export/backup_exporter_impl.rs)
- [crate-level backup importer code](../../../crates/z00z_wallets/src/backup/import/backup_importer_impl.rs)
- [tx storage implementation](../../../crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs)
