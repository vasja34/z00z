# 043-TODO

Canonical design source:

- [043-fixes-spec](./043-fixes-spec.md)

Execution rules:

- execute this file in order unless a dependency note says otherwise;
- treat the spec as normative for requirement meaning and this file as
  normative for execution order;
- do not pull requirements from unrelated wallet, storage, backup, fee,
  prover, or repository-wide TODO cleanup notes during implementation;
- do not add a parallel tx assembler, JMT public verifier, `.wlt`
  tx-history schema, receive decode path, cache-completeness shortcut, or
  second validated sender-flow facade when the existing truthful seam can be
  extended;
- keep `TxAssemblerImpl`, `verify_full_tx_package(...)`,
  `ProofBlob::{decode, chk_item, chk_blob}`, `TxRecord`, `WalletExportPack`,
  `WalletPersistenceState`, `ReceiveStatus::InvalidProof`, `Tag16Cache`,
  `ordered_request_candidates(...)`,
  `build_card_stealth_output_validated(...)`, and
  `build_tx_stealth_output_validated(...)` as the live authority surface
  unless the spec is updated first;
- if execution discovers that a planned new file is not required, land the
  behavior on the existing truthful seam and remove the speculative file entry
  from this backlog instead of creating a placeholder module;
- keep canonical `.wlt` semantics limited to wallet snapshot state and claimed
  assets; forensic archive remains an explicit optional envelope;
- keep storage membership and commitment conservation as separate verification
  layers; do not flatten them into one proof claim in code, docs, or tests;
- preserve request-bound candidate ordering and best-effort direct-scan
  fallback unless the spec changes first;
- before starting any numbered task, complete its `MANDATORY pre-read` block.

## 🎯 Decision Summary

The execution baseline for Phase 043 is:

1. keep JMT membership and Pedersen conservation as separate verification
   layers;
1. keep canonical `.wlt` as wallet state and place full tx-history export in
   an explicit forensic archive envelope;
1. make `TxAssemblerImpl` consume wallet-local resolved inputs rather than
   inferring confidential amounts from public input references;
1. keep `RECEIVE_INVALID_PROOF` only as an outward compatibility mapping while
   internal code uses an honest detector-versus-proof failure taxonomy;
1. allow strict `TagFilterOnly` only when concrete tag context completeness is
   proven for the scan domain;
1. require accepted sender flows to use validated stealth-output builders while
   keeping raw builders explicit and narrow;
1. expose asset-class Pedersen total recomputation only as an operator-invoked
   diagnostic surface, not as canonical tx admission.

## 🔗 Dependency Chain

Execution dependency chain:

1. `043-01` coverage ledger and failing-test lock-in
1. `043-02` transaction assembler closure
1. `043-03` storage membership and conservation separation
1. `043-04` optional forensic archive envelope
1. `043-05` receive DTO and status honesty
1. `043-06` tag16 completeness gate
1. `043-07` stealth output builder contract hardening
1. `043-08` tx and conservation regression wave
1. `043-09` receive, tag, and output regression wave
1. `043-10` archive closure and phase closeout

Hard dependencies:

- `043-02` depends on `043-01`
- `043-03` depends on `043-01` and `043-02`
- `043-04` depends on `043-01`
- `043-05` depends on `043-01`
- `043-06` depends on `043-05`
- `043-07` depends on `043-01` and should land after receiver approval seams
  are frozen by `043-05`
- `043-08` depends on `043-02` and `043-03`
- `043-09` depends on `043-05`, `043-06`, and `043-07`
- `043-10` depends on `043-04`, `043-08`, and `043-09`

Order rationale:

- assembler truth lands before storage-backed conservation auditing so the
  canonical tx-admission equation is frozen before operator diagnostics grow;
- receive-status honesty lands before tag completeness so strict tag-only mode
  is not wired on top of ambiguous status or placeholder opening semantics;
- archive closure stays late so `.wlt` preservation, tx-history integrity, and
  import isolation are proven only after the envelope contract itself is real.

## 🗂️ File-First Implementation Order

Edit order by file cluster:

1. `crates/z00z_wallets/src/tx/tx_assembler.rs`
1. `crates/z00z_wallets/src/tx/verify/tx_wire_types.rs`
1. `crates/z00z_wallets/src/tx/verify/tx_verifier.rs`
1. `crates/z00z_wallets/src/tx/balance.rs`
1. `crates/z00z_wallets/src/tx/state/state_resolved_input.rs`
1. `crates/z00z_wallets/src/tx/mod.rs`
1. `crates/z00z_storage/src/assets/proof.rs`
1. `crates/z00z_storage/src/assets/store_internal/proof_help.rs`
1. `crates/z00z_storage/src/assets/store_internal/store_query.rs`
1. new `crates/z00z_storage/src/assets/proof_scan.rs` only if typed membership
   scan results cannot live truthfully on the existing storage-proof seams
1. new `crates/z00z_wallets/src/tx/commit_audit.rs` only if operator audit
   logic cannot live truthfully in the existing tx/spend or verify seams
1. `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
1. `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
1. `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`
1. `crates/z00z_wallets/src/backup/export/backup_exporter.rs`
1. `crates/z00z_wallets/src/backup/export/backup_exporter_impl.rs`
1. `crates/z00z_wallets/src/backup/import/backup_importer.rs`
1. `crates/z00z_wallets/src/backup/import/backup_importer_impl.rs`
1. `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_transfer_import.rs`
1. `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
1. `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs`
1. `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs`
1. `crates/z00z_wallets/src/receiver/scan/types_receive.rs`
1. `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs`
1. `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs`
1. `crates/z00z_wallets/src/receiver/scan/types_tag_cache.rs`
1. new `crates/z00z_wallets/src/receiver/scan/tag_context.rs` only if
   completeness state cannot live truthfully in the current scan type modules
1. `crates/z00z_wallets/src/stealth/output/output.rs`
1. `crates/z00z_wallets/src/stealth/output/output_build.rs`
1. `crates/z00z_wallets/src/stealth/output/output_validator.rs`
1. `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
1. `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_assets.rs`
1. `crates/z00z_storage/tests/test_assets_suite.rs`
1. `crates/z00z_storage/tests/test_claim_source_proof.rs`
1. `crates/z00z_wallets/tests/test_tx_balance.rs`
1. `crates/z00z_wallets/tests/test_tx_tamper.rs`
1. `crates/z00z_wallets/tests/test_tx_fee.rs`
1. `crates/z00z_wallets/tests/test_tx_pedersen.rs`
1. `crates/z00z_wallets/tests/test_tx_store_integration.rs`
1. `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
1. `crates/z00z_wallets/tests/test_redb_wlt_open.rs`
1. `crates/z00z_wallets/tests/test_stealth_scanner_cache.rs`
1. `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs`
1. `crates/z00z_wallets/tests/test_stealth_output.rs`
1. `crates/z00z_wallets/tests/test_live_path_enforcement.rs`

## ✅ Validation Matrix

This table proves that the implementation-driving instructions from
`043-fixes-spec.md` have been migrated into this backlog and remain explicitly
traceable.

| 043-fixes-spec section | Required theme | TODO coverage | Status |
| --- | --- | --- | --- |
| `Purpose` and `Scope` | this phase is narrow, security-relevant, and must not turn into a generic wallet cleanup | execution rules; `043-01` through `043-10`; explicit phase boundary | Validated mapped |
| `Source Evidence` EV-001 to EV-005 | assembler stubs and storage-proof semantics must be closed without overstating JMT guarantees | `043-02`; `043-03`; `043-08` | Validated mapped |
| `Source Evidence` EV-006 to EV-007 | tx history and wallet snapshot remain separate persistence surfaces | `043-04`; `043-10` | Validated mapped |
| `Source Evidence` EV-008 to EV-009 | receive DTOs and public status compatibility must become semantically honest | `043-05`; `043-09` | Validated mapped |
| `Source Evidence` EV-010 to EV-012 | tag16 acceleration cannot imply completeness without materialized context | `043-06`; `043-09` | Validated mapped |
| `Source Evidence` EV-013 to EV-014 | raw and validated output builders must stay distinct on accepted sender flows | `043-07`; `043-09` | Validated mapped |
| `Required Design Decisions` | freeze the one-live-path architecture before coding starts | decision summary; dependency chain; task exit conditions | Mapped as execution guardrail |
| `PH43-TXASM`, `PH43-CONSERVE`, and `PH43-ASSETAUDIT` | keep canonical tx admission, manual asset-class audit, and storage membership layered honestly | `043-02`; `043-03`; `043-08` | Validated mapped |
| `PH43-ARCHIVE` | keep `.wlt` canonical and make forensic archive optional, explicit, and hash-bound | `043-04`; `043-10` | Validated mapped |
| `PH43-RECEIVE`, `PH43-TAG`, and `PH43-OUTPUT` | keep receive/report semantics, tag completeness, and output-builder approval precise and fail closed | `043-05`; `043-06`; `043-07`; `043-09` | Validated mapped |
| `Architecture` and `Conservation Flow` | preserve storage, wallet, receiver, stealth, and persistence ownership boundaries | execution rules; `043-03`; `043-04`; `043-05`; `043-07` | Mapped as preservation constraint |
| `Implementation Plan` | phase gates, inventory, and targeted tests must drive execution order | `043-01` through `043-10`; completion gate | Validated mapped |
| `Acceptance Criteria` | code, tests, docs, and archive behavior must all close with explicit signals | `043-08`; `043-09`; `043-10`; completion gate | Validated mapped |
| `Validation Strategy` | narrow-first source-shape, Rust, scenario, and security gates must stay explicit | `043-08`; `043-09`; `043-10`; concrete validation commands | Validated mapped |
| `Risks And Mitigations` | known failure modes must remain visible during execution, not only in the spec | risk watchpoints; task exit conditions; completion gate | Validated mapped |
| `No Logical Weak Spots Checklist` | anti-drift prohibitions must stay visible during execution | execution rules; no logical weak spots | Validated mapped |
| `Required Outputs` | coverage ledger, summary, focused tests, docs, and validation logs must be produced | `043-01`; `043-10`; required closeout outputs | Validated mapped |
| `Completion Definition` | wording-only closure is forbidden; code, tests, and docs must agree | completion gate | Validated mapped |

## 🚫 Explicit Phase Boundary

The following topics are intentionally out of scope for this backlog:

- no edits under `crates/z00z_crypto/tari/**`;
- no replacement of the current JMT backend and no exposure of raw `jmt`
  proof internals to wallet callers;
- no silent expansion of canonical `.wlt` into a mandatory tx-history store;
- no claim that wallet-local, pre-broadcast, or operator audit paths are a
  public trustless verifier;
- no broad wallet UI placeholder cleanup unrelated to the verified Phase 043
  seams;
- no repository-wide cleanup of unrelated `# TODO` blocks in fee estimation,
  prover internals, generic backup traits, or persistence comments unless the
  spec is first updated to include them.

## ⚠️ Risk Watchpoints

- fake resolved-input summation or fake public-input amount recovery is a
  release blocker; `043-02` must fail closed instead of inventing values;
- any code or docs that blend JMT membership with Pedersen conservation are a
  release blocker; `043-03` owns that separation;
- any change that makes the manual asset-class total audit an implicit
  canonical tx-admission dependency is a release blocker; `043-03` and
  `043-08` must keep it explicit and out-of-band;
- any change that silently expands canonical `.wlt` semantics into tx-history
  storage is a release blocker; `043-04` and `043-10` must preserve the
  separate forensic envelope;
- any change that drops public receive compatibility codes before an explicit
  RPC decision is a release blocker; `043-05` must improve internal honesty
  without breaking the outward contract;
- any strict tag-only path that can miss owned outputs because completeness is
  inferred from request metadata or cache size is a release blocker; `043-06`
  must fail closed;
- any accepted sender flow that still reaches a raw stealth-output builder is a
  release blocker; `043-07` and `043-09` must lock this down;
- any archive/export path that logs, leaks, or partially imports sensitive tx
  data on failed validation is a release blocker; `043-04` and `043-10` own
  the mitigation.

## 🧱 No Logical Weak Spots

- do not claim confidential input amounts can be recovered from public input
  refs;
- do not claim fee metadata alone proves economic conservation;
- do not use a JMT existence proof as a Pedersen commitment equation;
- do not turn the manual asset-class total audit into an implicit canonical
  admission dependency;
- do not use `backend_root` as a public semantic root;
- do not make forensic archive import mutate wallet state before all tx-history
  hashes verify;
- do not regress the existing request-bound candidate ordering by evaluating
  the request-less fallback first;
- do not let public `InvalidProof` compatibility labels erase internal failure
  precision;
- do not let raw stealth output builders become accepted approval paths by
  convention;
- do not bypass `z00z_utils` abstractions for new file I/O, time,
  serialization, or RNG boundaries.
- do not modify Tari vendor code;
- do not describe Phase 043 as closing unrelated wallet or storage TODO
  markers outside EV-001 through EV-014 and the explicitly named live call
  sites.

## ⚙️ Concrete Execution Tasks

### 043-01 Coverage Ledger And Failing-Test Lock-In

Spec references:

- `Purpose`
- `Source Evidence`
- `Scope`
- `Implementation Plan`, `Phase Gate 0: Inventory And Failing Tests`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `Purpose`
- section `Source Evidence`
- section `Scope`
- section `Implementation Plan`

- [ ] Create `.planning/phases/043-gaps-fixes/043-coverage.md` and map every
  EV, PH43, D-043, and AC requirement to one implementation owner, one test
  home, one evidence slot, and Notes that tie any related Risk Watchpoint or
  No Logical Weak Spots item back to the owning row.
- [ ] Run and record the inventory commands from the spec before any behavior
  changes so the starting residue is evidence-backed.
- [ ] Assign one truthful failing or gap-revealing test home per verified seam
  instead of widening the phase into a repository-wide TODO sweep.
- [ ] Record the explicit exclusions for unrelated fee, prover, generic
  backup, and broad persistence TODO blocks so they do not re-enter through
  implementation drift.
- [ ] Confirm that no planned edit touches `crates/z00z_crypto/tari/**`.

Coverage ledger schema:

| Column | Meaning |
| --- | --- |
| Requirement | one EV, PH43, D-043, or AC identifier |
| Owner files | exact live files that own the implementation seam |
| Task | the numbered TODO task that closes the requirement |
| Test home | exact test file or module that proves the requirement |
| Evidence slot | one reproducible evidence pointer such as a command, named test case, or summary reference |
| Status | `open`, `in-progress`, `blocked`, `green`, or `deferred-by-spec` |
| Notes | short anti-drift note or deferral reason |

Evidence slot contract:

- use one reproducible pointer per row, such as a named test case, a focused
  command from `Concrete Validation Commands`, or a short reference into
  `043-SUMMARY.md`;
- do not use vague prose like `covered by tests` without a concrete anchor;
- if a row is deferred, the evidence slot must point to the exact spec update
  or closeout note that authorizes the deferral.

Failing anchor definition:

- one failing anchor means at least one named test case, one module-local test
  function, or one explicit assertion path that reproduces the gap before the
  fix and turns green after the fix;
- a generic promise to add tests later is not a valid failing anchor.

Commands:

```bash
rg -n "not implemented in Phase 1|# TODO|placeholder|best-effort only|does not assert that a downstream proof verifier ran here" crates/z00z_wallets/src crates/z00z_storage/src
rg -n "build_tx_stealth_output\(|build_tx_stealth_output_serial\(|build_tx_stealth_output_validated\(|build_card_stealth_output_validated\(" crates/z00z_wallets/src crates/z00z_wallets/tests
rg -n "TxStorageImpl|TxRecord|WalletExportPack|WalletPersistenceState" crates/z00z_wallets/src
```

Files:

- new `.planning/phases/043-gaps-fixes/043-coverage.md`
- `.planning/phases/043-gaps-fixes/043-fixes-spec.md` only if a source-truth
  ambiguity is discovered during inventory

Tests:

- [ ] identify or add failing anchors in existing homes before behavior work:
  - `crates/z00z_wallets/tests/test_tx_balance.rs`
  - `crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - `crates/z00z_wallets/tests/test_stealth_scanner_cache.rs`
  - `crates/z00z_wallets/tests/test_stealth_output.rs`

Exit condition:

- the phase has a repository-backed coverage ledger, one selected test home per
  seam, and no unscoped cleanup drift hidden behind generic inventory noise.

Focused-scope rule:

- `focused` tests and docs in this phase means only the files, suites, and
  docs directly tied to EV-001 through EV-014, the explicitly named live call
  sites, and the required closeout artifacts;
- do not widen a focused validation or docs pass into unrelated wallet,
  storage, backup, prover, or persistence families that are outside the Phase
  043 seam inventory.

### 043-02 Transaction Assembler Closure

Spec references:

- `Source Evidence` EV-001 to EV-003
- `Required Design Decisions` D-043-001, D-043-003, D-043-007
- `PH43-TXASM`
- `Implementation Plan`, `Phase Gate 1: Tx Assembler Closure`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `Required Design Decisions`
- section `PH43-TXASM: Transaction Assembly Closure`
- section `Implementation Plan`

- [ ] Add a wallet-local resolved-input wire contract on the existing tx seam,
  or in one tightly related helper file, that carries the value or opening or
  commitment data actually required by `sum_inputs`.
- [ ] Decode `TxAssemblyParams.inputs_bytes` through that resolved-input
  contract instead of public `TxInputWire` bytes.
- [ ] Decode `TxAssemblyParams.tx_outputs_bytes` through the canonical output
  wire path and reject malformed bytes, empty inputs, empty outputs, empty
  lanes, duplicate refs, duplicate output state keys, duplicate nonces,
  invalid fee-output shape, and invalid chain metadata.
- [ ] Keep `sum_tx_outputs` on the existing typed conversion path and sum only
  values that are semantically visible in the decoded asset lane instead of
  inventing a second output-value interpretation rule.
- [ ] Build canonical `TxWire` and `TxPackage` bytes through the existing
  digest helpers, then route public package checks through
  `verify_full_tx_package`.
- [ ] Add or tighten the resolved-input balance helper so it consumes resolved
  input commitments and produced output commitments explicitly, keeps fee
  outputs on the output side, and fails closed on non-zero delta.
- [ ] Remove or rewrite every reachable `not implemented in Phase 1` path and
  any stub wording that overstates public tx-package conservation.

Files:

- `crates/z00z_wallets/src/tx/tx_assembler.rs`
- `crates/z00z_wallets/src/tx/verify/tx_wire_types.rs`
- `crates/z00z_wallets/src/tx/verify/tx_verifier.rs`
- `crates/z00z_wallets/src/tx/balance.rs`
- `crates/z00z_wallets/src/tx/state/state_resolved_input.rs`
- `crates/z00z_wallets/src/tx/mod.rs`

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_tx_balance.rs`
  - canonical resolved-input delta passes with fee outputs included on the
    output side
  - public `TxInputWire` bytes reject on the assembler input path
  - public `TxPackage` verification does not infer hidden input values without
    resolved pre-state or spend-proof evidence
- [ ] extend `crates/z00z_wallets/tests/test_tx_tamper.rs`
  - malformed input or output bytes reject before package emission
  - duplicate input or output identifiers reject
- [ ] extend `crates/z00z_wallets/tests/test_tx_fee.rs`
  - declared fee and fee-output mismatch reject before balance success
- [ ] add `crates/z00z_wallets/tests/test_tx_assembler.rs` only if the current
  tx suites cannot host the active trait-path closure truthfully

Exit condition:

- the active `TxAssemblerImpl` path no longer returns reachable Phase 1 stub
  errors and no code path infers confidential input amounts from public tx
  references.

### 043-03 Storage Membership And Conservation Separation

Spec references:

- `Source Evidence` EV-004 and EV-005
- `Required Design Decisions` D-043-001 and D-043-007
- `PH43-CONSERVE`
- `PH43-ASSETAUDIT`
- `Architecture`, `Layer Boundaries`, `Conservation Flow`, and
  `Manual Audit vs Canonical Path`
- `Implementation Plan`, `Phase Gate 2: Membership And Conservation Audit`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `PH43-CONSERVE: Storage Membership And Conservation Audit`
- section `PH43-ASSETAUDIT: Manual Asset-Class Pedersen Total Audit`
- section `Architecture`
- section `Implementation Plan`

- [ ] Add a typed storage-side scan result that exposes semantic root,
  backend-root binding, path, leaf hash, and verified leaf without leaking raw
  JMT proof internals into wallet callers.
- [ ] Keep `ProofBlob::decode`, `chk_item`, and `chk_blob` as the only proof
  verification gateway for storage witness bytes.
- [ ] Add wallet or tx-layer conservation audit logic that consumes validated
  leaves together with tx or proof or commitment evidence and reports distinct
  membership, root-binding, leaf, spend-proof, commitment, missing-evidence,
  and asset-class-audit mismatch failures.
- [ ] Add the operator-invoked asset-class Pedersen total audit on the existing
  tx or verify seam, or one narrow helper file, so full asset-class totals are
  explicit diagnostics rather than canonical tx admission.
- [ ] Update docs and comments so no storage proof path claims to prove Pedersen
  conservation by itself.

Files:

- `crates/z00z_storage/src/assets/proof.rs`
- `crates/z00z_storage/src/assets/store_internal/proof_help.rs`
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
- new `crates/z00z_storage/src/assets/proof_scan.rs` only if typed scan results
  cannot live truthfully in the current proof modules
- `crates/z00z_wallets/src/tx/spend/spend_verification.rs`
- `crates/z00z_wallets/src/tx/verify/tx_verifier.rs`
- new `crates/z00z_wallets/src/tx/commit_audit.rs` only if the operator audit
  cannot live truthfully on the current tx seams

Tests:

- [ ] extend `crates/z00z_storage/tests/test_assets_suite.rs`
  - semantic-root, backend-root, path, and leaf tampering fail with distinct
    evidence classes
- [ ] extend `crates/z00z_storage/tests/test_claim_source_proof.rs`
  - proof decode and blob verification remain the only storage witness gateway
- [ ] extend `crates/z00z_wallets/tests/test_tx_pedersen.rs`
  - commitment mismatch rejects separately from proof membership success
- [ ] extend `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
  - missing spend-proof or missing commitment evidence fails closed
- [ ] extend `crates/z00z_wallets/tests/test_tx_wrong_root.rs`
  - wrong semantic root does not masquerade as a balance mismatch

Exit condition:

- storage membership, root binding, and Pedersen conservation are separate typed
  outcomes in code, docs, and tests, and asset-class total recomputation exists
  only as an explicit diagnostic path.

### 043-04 Optional Forensic Archive Envelope

Spec references:

- `Source Evidence` EV-006 and EV-007
- `Required Design Decisions` D-043-002
- `PH43-ARCHIVE`
- `Architecture`, `Forensic Archive Flow`
- `Implementation Plan`, `Phase Gate 3: Optional Forensic Archive`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `PH43-ARCHIVE: Optional Forensic Archive`
- section `Forensic Archive Flow`
- section `Implementation Plan`

- [ ] Add a versioned forensic envelope around an encrypted
  `WalletExportPack` payload, tx-history records, tx-hash manifest data,
  chain identity, schema version, and export metadata without changing
  canonical `.wlt` semantics.
- [ ] Reuse `WalletExportPack` for wallet restore state instead of duplicating
  seed or identity payload fields.
- [ ] Add bounded tx-record hash verification and explicit import modes for
  wallet-only, tx-history-only, and wallet-plus-history flows.
- [ ] Keep exporter-side and importer-side archive verification on the existing
  backup seams so the forensic envelope does not introduce a second integrity
  stack.
- [ ] Ensure invalid tx-history payloads reject without mutating restored wallet
  state.
- [ ] Route any new archive or import file access through `z00z_utils::io`
  abstractions and reject direct `std::fs` usage on the new forensic path.
- [ ] Document the archive as diagnostic or replay support rather than chain
  truth or checkpoint proof verification.

Files:

- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs`
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
- `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`
- `crates/z00z_wallets/src/backup/export/backup_exporter.rs`
- `crates/z00z_wallets/src/backup/export/backup_exporter_impl.rs`
- `crates/z00z_wallets/src/backup/export/backup_exporter_verify.rs`
- `crates/z00z_wallets/src/backup/import/backup_importer.rs`
- `crates/z00z_wallets/src/backup/import/backup_importer_impl.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_transfer_import.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs`
- new `crates/z00z_wallets/src/wallet/snapshot/forensic_types.rs` only if the
  envelope types cannot live truthfully in the existing snapshot or backup
  seams

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
  - canonical `.wlt` semantics remain wallet-state-only
  - forensic envelope versioning and hash manifest roundtrip succeed
- [ ] extend `crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - tx-history-only import requires explicit caller intent
  - tx-history hash mismatch rejects without partial import
- [ ] extend `crates/z00z_wallets/tests/test_redb_wlt_open.rs`
  - wallet snapshot restore remains valid when forensic history is absent
- [ ] extend `crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs`
  - exporter-side container integrity and metadata verification stays aligned
    with the forensic envelope contract
- [ ] extend `crates/z00z_wallets/tests/test_wallet_json_export.rs` only if the
  existing export tests already own the archive serialization seam

Exit condition:

- forensic archive exists as an optional, versioned, hash-bound envelope and
  canonical `.wlt` semantics remain unchanged.

### 043-05 Receive DTO And Status Honesty

Spec references:

- `Source Evidence` EV-008 and EV-009
- `Required Design Decisions` D-043-004
- `PH43-RECEIVE`
- `Implementation Plan`, `Phase Gate 4: Receive DTO And Status Closure`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `PH43-RECEIVE: Receive DTO And Status Honesty`
- section `Implementation Plan`

- [ ] Replace placeholder `asset_secret` and `blinding` semantics with an
  explicit decoded opening contract or an explicit redacted or unavailable
  state, without creating a second decode path for bytes already carried by
  `DetectedAssetPack`.
- [ ] Add an internal detector-failure taxonomy such as candidate-invalid,
  decrypt-failed, or proof-check-failed, then map to public
  `ReceiveStatus::InvalidProof` only at compatibility boundaries that still
  require that outward code.
- [ ] Update receive-report conversion, logs, and docs so detector-side failure
  does not claim that downstream proof verification ran.
- [ ] Preserve outward compatibility while making internal ownership and
  validation reporting precise enough for diagnostics and tests.

Files:

- `crates/z00z_wallets/src/receiver/scan/types_receive.rs`
- `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs`
- `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs`
  - detector failure reports do not claim proof-verifier success
- [ ] extend receive-report or scanner tests for malformed leaf input, status
  mapping, and alert or log-code behavior if those assertions already live on
  the existing receive test seam
- [ ] extend receive or scanner tests for proof-verifier failure paths as a
  separate internal reason when such a path exists on the live seam
- [ ] extend `crates/z00z_wallets/tests/test_import_error_taxonomy.rs`
  - internal reject classes remain more precise than the public compatibility
    code
- [ ] extend `crates/z00z_wallets/tests/test_runtime_validation_result.rs`
  - compatibility mapping preserves outward status stability
- [ ] extend local receive scan suites under
  `crates/z00z_wallets/src/receiver/scan/stealth_scanner/` and
  `crates/z00z_wallets/src/receiver/scan/stealth_scan_support/`

Exit condition:

- receive outputs use honest opening semantics and internal reject reasons no
  longer masquerade as downstream proof verification.

### 043-06 Tag16 Completeness Gate

Spec references:

- `Source Evidence` EV-010 to EV-012
- `Required Design Decisions` D-043-005
- `PH43-TAG`
- `Implementation Plan`, `Phase Gate 5: Tag16 Completeness Closure`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `PH43-TAG: Tag16 Cache Completeness`
- section `Implementation Plan`

- [ ] Add an explicit completeness state for tag contexts, keeping liveness and
  completeness separate.
- [ ] Keep `add_request` as request-liveness metadata only; it must never
  upgrade cache completeness.
- [ ] Add or extend a context-materialization path that records concrete
  `Tag16Context` entries and marks the scan domain complete only when all
  required contexts are present.
- [ ] Change `background_scan_strategy` so strict `TagFilterOnly` is selected
  only when completeness is proven, not when cache size or hit count is high.
- [ ] Preserve existing request-bound candidate ordering and direct-scan
  fallback for best-effort paths.
- [ ] Keep cache statistics and completeness status as separate reported
  concepts.

Files:

- `crates/z00z_wallets/src/receiver/scan/types_tag_cache.rs`
- `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs`
- `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs`
- new `crates/z00z_wallets/src/receiver/scan/tag_context.rs` only if the
  completeness state cannot live truthfully in the existing scan-type modules

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_stealth_scanner_cache.rs`
  - active request ids alone do not authorize strict ownership classification
  - cache size alone does not authorize `TagFilterOnly`
  - expired or stale requests do not imply completeness
- [ ] extend `crates/z00z_wallets/tests/test_stealth_scanner_prefilter.rs`
  - request-bound candidates stay ahead of request-less fallback
- [ ] extend local suites in
  `crates/z00z_wallets/src/receiver/scan/stealth_scanner/test_stealth_scanner.rs`
  and
  `crates/z00z_wallets/src/receiver/scan/stealth_scan_support/test_stealth_scan_support_suite.rs`
  - strict tag-only mode fails closed when context state is incomplete

Exit condition:

- strict `TagFilterOnly` cannot be selected from cache size or request liveness
  alone, and best-effort scans still fall back to direct scan.

### 043-07 Stealth Output Builder Contract Hardening

Spec references:

- `Source Evidence` EV-013 and EV-014
- `Required Design Decisions` D-043-006
- `PH43-OUTPUT`
- `Implementation Plan`, `Phase Gate 6: Output Builder Contract Hardening`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `PH43-OUTPUT: Stealth Output Builder Contract Hardening`
- section `Implementation Plan`

- [ ] Audit every active call site of `build_tx_stealth_output(...)` and
  `build_tx_stealth_output_serial(...)` on accepted sender flows.
- [ ] Replace accepted-flow call sites with the explicit validated constructors
  `build_card_stealth_output_validated(...)` and
  `build_tx_stealth_output_validated(...)`, or document a stricter named
  successor in `043-SUMMARY.md`, while keeping raw builders narrow and
  explicitly documented for test-only or already-validated internal
  construction paths.
- [ ] Add source-shape guards that fail if the live RPC send path, or any
  scenario entrypoint that routes through it, calls
  `build_tx_stealth_output(...)` or `build_tx_stealth_output_serial(...)`
  directly where receiver approval is required.
- [ ] Add behavior tests proving malformed or unapproved card and request paths
  fail in validated builders.
- [ ] Update Rustdoc so every compatibility helper names the validated
  replacement and the missing policy checks.

Files:

- `crates/z00z_wallets/src/stealth/output/output.rs`
- `crates/z00z_wallets/src/stealth/output/output_build.rs`
- `crates/z00z_wallets/src/stealth/output/output_validator.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_assets.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3.rs`

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_stealth_output.rs`
  - validated builders reject malformed or unapproved inputs
- [ ] extend `crates/z00z_wallets/tests/test_live_path_enforcement.rs`
  - live RPC send path and routed simulator entrypoints cannot call raw or
    serial raw builders directly where approval is required
- [ ] extend `crates/z00z_wallets/tests/test_e2e_send_scan.rs`
  - accepted sender flow uses validated builders end to end
- [ ] extend `crates/z00z_wallets/tests/test_tx_stealth_flow.rs` only if that
  suite already owns the sender-flow approval seam more truthfully than a new
  dedicated test file

Exit condition:

- active approved sender flows no longer call raw builders directly and raw or
  serial raw builder contracts remain explicit instead of partially validated.

## 🧪 Concrete Test Execution Tasks

### 043-08 Tx And Conservation Regression Wave

Spec references:

- `PH43-TXASM`
- `PH43-CONSERVE`
- `PH43-ASSETAUDIT`
- `Implementation Plan`, `Phase Gate 1` and `Phase Gate 2`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `PH43-TXASM: Transaction Assembly Closure`
- section `PH43-CONSERVE: Storage Membership And Conservation Audit`
- section `PH43-ASSETAUDIT: Manual Asset-Class Pedersen Total Audit`

- [ ] Run the tightened wallet tx suites for assembler decode, fee-output
  shape, resolved-input delta, and verifier routing.
- [ ] Run the storage and wallet proof suites that tamper semantic root,
  backend root, leaf hash, branch proof, output commitment, fee output, and
  asset-class audit target independently.
- [ ] Record the green or failing evidence back into `043-coverage.md` so each
  PH43 tx or conservation requirement has a named verification anchor.

Files:

- `.planning/phases/043-gaps-fixes/043-coverage.md`
- `crates/z00z_storage/tests/test_assets_suite.rs`
- `crates/z00z_storage/tests/test_claim_source_proof.rs`
- `crates/z00z_wallets/tests/test_tx_balance.rs`
- `crates/z00z_wallets/tests/test_tx_tamper.rs`
- `crates/z00z_wallets/tests/test_tx_fee.rs`
- `crates/z00z_wallets/tests/test_tx_pedersen.rs`
- `crates/z00z_wallets/tests/test_spend_proof_backend.rs`
- `crates/z00z_wallets/tests/test_tx_wrong_root.rs`

Tests:

- [ ] wallet tx assembly and fee-shape regressions are green
- [ ] storage membership tamper regressions are green
- [ ] commitment and asset-class audit mismatch regressions are green

Exit condition:

- assembler closure and conservation separation have targeted regression
  evidence with distinct typed failure classes and no residual stub path.

### 043-09 Receive, Tag, And Output Regression Wave

Spec references:

- `PH43-RECEIVE`
- `PH43-TAG`
- `PH43-OUTPUT`
- `Implementation Plan`, `Phase Gate 4` through `Phase Gate 6`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `PH43-RECEIVE: Receive DTO And Status Honesty`
- section `PH43-TAG: Tag16 Cache Completeness`
- section `PH43-OUTPUT: Stealth Output Builder Contract Hardening`

- [ ] Run the scanner-flow, tag-cache, and source-shape test homes after the
  receive and builder tasks land.
- [ ] Add or tighten negative coverage for detector failure mapping, incomplete
  tag context in strict mode, cache-size-only strategy selection, best-effort
  direct-scan fallback when strict completeness is absent, and raw or serial
  raw builder usage on the live send path and routed simulator entrypoints.
- [ ] Record the final test homes for each EV-008 through EV-014 seam in the
  coverage ledger so future drift does not reopen ambiguous ownership of these
  checks.

Files:

- `.planning/phases/043-gaps-fixes/043-coverage.md`
- `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs`
- `crates/z00z_wallets/tests/test_import_error_taxonomy.rs`
- `crates/z00z_wallets/tests/test_runtime_validation_result.rs`
- `crates/z00z_wallets/tests/test_stealth_scanner_cache.rs`
- `crates/z00z_wallets/tests/test_stealth_scanner_prefilter.rs`
- `crates/z00z_wallets/tests/test_stealth_output.rs`
- `crates/z00z_wallets/tests/test_live_path_enforcement.rs`
- `crates/z00z_wallets/tests/test_e2e_send_scan.rs`

Tests:

- [ ] receive/status honesty regressions are green
- [ ] tag completeness and direct-scan fallback regressions are green
- [ ] validated-builder routing regressions are green

Exit condition:

- receive compatibility, tag completeness, and validated-builder routing are
  all locked by named regression homes with no ambiguous ownership.

### 043-10 Archive Closure And Phase Closeout

Spec references:

- `Scope`
- `Out Of Scope`
- `PH43-ARCHIVE`
- `Implementation Plan`

MANDATORY pre-read in `043-fixes-spec.md`:

- section `Scope`
- section `Out Of Scope`
- section `PH43-ARCHIVE: Optional Forensic Archive`

- [ ] Finalize archive/export/import docs and evidence without widening
  canonical `.wlt` semantics.
- [ ] Re-run the wallet export, tx-store, and wallet-open test homes after the
  archive envelope lands.
- [ ] Re-run the exporter-side backup verification gate after the archive
  envelope lands so exporter/importer archive integrity stays symmetric.
- [ ] Create `.planning/phases/043-gaps-fixes/043-SUMMARY.md` with final
  coverage, changed files, validation commands, residual risks, and any
  spec-backed deferrals.
- [ ] Update `043-coverage.md` so every EV, PH43, and D-043 item is marked with
  one landed code path, one test anchor, or one explicit spec-backed deferral.
- [ ] Capture the exact validation command outputs in `043-SUMMARY.md` so
  closeout evidence is reproducible.
- [ ] Record any intentionally skipped tests in `043-SUMMARY.md` with exact
  reason and next owner.
- [ ] Run an explicit closeout redaction gate on `043-SUMMARY.md` and
  `043-coverage.md` after the validation outputs have been copied into
  `043-SUMMARY.md` so copied evidence stays hash-bound or redacted rather than
  embedding raw secret or tx-history fields.
- [ ] If execution discovered a new design constraint, update
  `043-fixes-spec.md` first, then this backlog, before calling the phase
  complete.

Files:

- `.planning/phases/043-gaps-fixes/043-coverage.md`
- `.planning/phases/043-gaps-fixes/043-SUMMARY.md`
- `.planning/phases/043-gaps-fixes/043-TODO.md`
- `.planning/phases/043-gaps-fixes/043-fixes-spec.md` only if the source of
  truth changes
- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- `crates/z00z_wallets/tests/test_redb_wlt_open.rs`

Tests:

- [ ] forensic archive roundtrip and hash-manifest checks are green
- [ ] invalid tx-history import leaves wallet restore state unchanged
- [ ] canonical `.wlt` open or restore paths remain green without forensic
  history present

Exit condition:

- the optional forensic archive is proven without mutating canonical `.wlt`
  semantics and the phase ledger can point to one evidence-backed closeout map.

## 🧪 Concrete Validation Commands

Run these in order. Do not widen scope until the narrower gate is green or the
blocking failure is explained in `043-SUMMARY.md`.

### Source-Shape Gates

```bash
rg -n "not implemented in Phase 1" crates/z00z_wallets/src/tx/tx_assembler.rs
rg -n "Asset secret bytes placeholder|Blinding bytes placeholder" crates/z00z_wallets/src/receiver/scan/types_receive.rs
rg -n "TagFilterOnly" crates/z00z_wallets/src/receiver/scan
rg -n "build_tx_stealth_output\(|build_tx_stealth_output_serial\(" crates/z00z_wallets/src crates/z00z_wallets/tests crates/z00z_simulator/src
```

Expected handling:

- the first two commands must return no active production stub or placeholder
  hits;
- `TagFilterOnly` hits are allowed only where completeness is checked or tests
  assert fail-closed behavior;
- raw-builder and raw-serial-builder hits are allowed only in raw-builder
  definitions, tests, or call sites with explicit pre-validation evidence.

### Targeted Rust Gates

```bash
cargo fmt --all
cargo check -p z00z_wallets --all-targets
cargo test -p z00z_wallets tx_assembler --lib -- --nocapture
cargo test -p z00z_wallets tx_verifier --lib -- --nocapture
cargo test -p z00z_wallets stealth_scanner --lib -- --nocapture
cargo test -p z00z_wallets --test test_stealth_request -- --nocapture
cargo test -p z00z_storage assets:: --lib -- --nocapture
```

### Scenario Gates

Run simulator gates only after targeted wallet and storage tests pass.

```bash
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump
```

### Security And Regression Gates

```bash
rg -n "JMT.*conservation|conservation.*JMT|backend_root.*public" crates docs .planning/phases/043-gaps-fixes
rg -n "std::fs" crates/z00z_wallets/src/backup crates/z00z_wallets/src/persistence
rg -n "RECEIVE_INVALID_PROOF" crates/z00z_wallets/src crates/z00z_wallets/tests
rg -n '"seed_phrase"|"wallet_identity"|"tx_bytes"|"enc_pack"|"asset_secret"|"blinding"' .planning/phases/043-gaps-fixes/043-SUMMARY.md .planning/phases/043-gaps-fixes/043-coverage.md
```

Expected handling:

- no docs or code may say JMT proves Pedersen conservation;
- new backup or persistence work must use `z00z_utils::io` instead of direct
  `std::fs`;
- `RECEIVE_INVALID_PROOF` may remain only as a public compatibility code with
  precise internal mapping;
- closeout artifacts must not embed raw secret or tx-history field payloads;
  only redacted or hash-bound evidence is allowed.

## 📦 Required Closeout Outputs

- `.planning/phases/043-gaps-fixes/043-coverage.md`
- `.planning/phases/043-gaps-fixes/043-SUMMARY.md`
- focused wallet and storage tests for every acceptance criterion touched by
  the phase
- updated Rustdoc or docs for changed public or crate-visible APIs
- validation logs or copied command outputs referenced from the summary

Artifact distinction:

- the `Validation Matrix` in this TODO proves planning coverage from spec to
  task structure;
- `043-coverage.md` is the execution ledger that maps each requirement to live
  files, tests, and evidence;
- `043-SUMMARY.md` is the closeout artifact that records decisions, changed
  files, decisive command outputs, residual risks, and any spec-backed
  deferrals.

`043-SUMMARY.md` minimum structure:

- phase scope and closeout verdict;
- coverage summary with counts for EV, PH43, D-043, and AC rows;
- key decisions or design clarifications landed during execution;
- changed files grouped by tx, storage, receive, tag, output, archive, and
  docs or tests;
- focused tests run, their outcomes, and any intentionally skipped tests with
  exact reason and next owner;
- decisive validation commands and copied or referenced outputs;
- residual risks, explicit deferrals, and owner follow-ups if any remain.

## ✅ Completion Gate

Phase 043 is complete only when all of the following hold:

- every numbered task in this backlog is implemented or explicitly deferred by
  a source update to `043-fixes-spec.md`;
- `.planning/phases/043-gaps-fixes/043-coverage.md` maps every EV, PH43, and
  D-043 requirement to landed code and named regression evidence;
- the active `TxAssemblerImpl` path has no reachable Phase 1 stub behavior;
- storage membership, Pedersen conservation, and asset-class total audit are
  separated honestly in code, docs, and tests;
- canonical `.wlt` semantics remain wallet-state-only, while forensic archive
  remains optional and explicit;
- public receive compatibility stays stable while internal receive and tag
  semantics become precise and fail closed;
- active accepted sender flows use validated stealth-output builders where
  receiver approval is required;
- `.planning/phases/043-gaps-fixes/043-SUMMARY.md` exists and points to the
  decisive validation outputs;
- updated Rustdoc or focused docs exist for every changed public or
  crate-visible API or proof-boundary contract;
- copied validation logs or command outputs are linked from the summary rather
  than left implicit;
- no forbidden parallel layer, JMT public-proof claim, or out-of-scope cleanup
  sweep was introduced;
- focused wallet and storage regression homes are green and no vendor Tari file
  was modified.
