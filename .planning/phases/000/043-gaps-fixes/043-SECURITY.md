---
phase: 043
slug: 043-gaps-fixes
status: verified
threats_open: 0
asvs_level: 1
created: 2026-05-07
audited: 2026-05-08
---

<!-- markdownlint-disable MD060 -->

# Phase 043 - Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

## Scope

This audit covers the landed Phase 043 closeout artifacts, the wallet/storage seams named in the numbered plans, and the regression homes that prove those seams fail closed. The audit is grounded in repository-backed code, tests, and the phase coverage ledger only.

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| spec/backlog -> execution ledger | Missing or misclassified rows can silently drop required work | Phase requirements, evidence slots, and test-home ownership |
| public tx refs -> resolved input summation | Untrusted public refs must not be treated as confidential value/opening evidence | `TxInputWire` bytes and resolved-input witnesses |
| storage proof bytes -> typed membership result | JMT witness verification must not leak into wallet-facing public proof claims | Proof bytes, `ProofBlob`, and typed scan summaries |
| archive bytes -> restore/import path | Untrusted archive data must not mutate canonical wallet state before validation | Encrypted backup bytes, manifest hashes, and import mode |
| detected opening bytes -> public DTO | Internal opening data must not be mislabeled or duplicated through placeholder wrappers | Scan results, redaction states, and RPC status mapping |
| tag-cache metadata -> ownership classification | Best-effort liveness/cache data must not be treated as proof of completeness | `Tag16Cache`, active requests, and strict tag-only mode |
| approved sender flow -> stealth output construction | Receiver approval policy must not be bypassed by raw builders | Validated output builders, raw builder seams, and live RPC paths |
| landed implementation -> regression evidence | Phase closure claims must be tied to replayable narrow gates rather than prose | Named tests, release gates, and coverage rows |
| final closeout narrative -> actual evidence | Summary language must stay subordinate to decisive command outputs and named tests | `043-SUMMARY.md` and the validation outputs it records |

## Evidence Map

| Evidence ID | Files | What it proves |
|------------|-------|----------------|
| E1 | [state_update.rs](../../../crates/z00z_wallets/src/tx/state/state_update.rs), [state_resolved_input.rs](../../../crates/z00z_wallets/src/tx/state/state_resolved_input.rs), [test_tx_tamper.rs](../../../crates/z00z_wallets/tests/test_tx_tamper.rs), [test_tx_wrong_root.rs](../../../crates/z00z_wallets/tests/test_tx_wrong_root.rs) | `prepare_tx_sum(...)` keeps `TxInputWire` reference-only, loads resolved inputs through the typed contract, and fails closed on witness bytes, bad blobs, wrong roots, wrong paths, and missing inputs. |
| E2 | [tx_verifier.rs](../../../crates/z00z_wallets/src/tx/verify/tx_verifier.rs), [tx_assembler.rs](../../../crates/z00z_wallets/src/tx/tx_assembler.rs), [test_tx_balance.rs](../../../crates/z00z_wallets/tests/test_tx_balance.rs), [test_tx_fee.rs](../../../crates/z00z_wallets/tests/test_tx_fee.rs), [test_tx_verifier_suite.rs](../../../crates/z00z_wallets/src/tx/verify/test_tx_verifier_suite.rs) | Public package verification routes through `verify_full_tx_package(...)`, the canonical balance helper keeps fee outputs on the output side, and the verifier tests document the admission boundary precisely. |
| E3 | [proof.rs](../../../crates/z00z_storage/src/assets/proof.rs), [store_query.rs](../../../crates/z00z_storage/src/assets/store_internal/store_query.rs), [test_claim_source_proof.rs](../../../crates/z00z_storage/tests/test_claim_source_proof.rs), [test_whitebox_proofs.rs](../../../crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs) | Storage witness verification stays behind `ProofBlob`/`chk_*`, and the sanitized proof scan path does not expose raw JMT proof internals. |
| E4 | [commit_audit.rs](../../../crates/z00z_wallets/src/tx/commit_audit.rs), [test_tx_pedersen.rs](../../../crates/z00z_wallets/tests/test_tx_pedersen.rs), [test_spend_proof_backend.rs](../../../crates/z00z_wallets/tests/test_spend_proof_backend.rs) | Conservation audit remains explicit and diagnostic, while asset-class recomputation stays outside canonical transaction admission. |
| E5 | [backup_exporter_impl.rs](../../../crates/z00z_wallets/src/backup/export/backup_exporter_impl.rs), [backup_importer_impl.rs](../../../crates/z00z_wallets/src/backup/import/backup_importer_impl.rs), [backup_wire.rs](../../../crates/z00z_wallets/src/backup/crypto/backup_wire.rs), [test_wallet_export_pack_boundary.rs](../../../crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs), [test_backup_exporter_suite.rs](../../../crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs), [test_backup_importer_suite.rs](../../../crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs) | Archive import/export is hash-bound, versioned, explicit about import mode, and uses `z00z_utils` I/O seams instead of direct `std::fs` paths. |
| E6 | [types_receive.rs](../../../crates/z00z_wallets/src/receiver/scan/types_receive.rs), [asset_impl_server_transfer.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs), [test_stealth_scanner_flow.rs](../../../crates/z00z_wallets/tests/test_stealth_scanner_flow.rs), [test_import_error_taxonomy.rs](../../../crates/z00z_wallets/tests/test_import_error_taxonomy.rs), [test_runtime_validation_result.rs](../../../crates/z00z_wallets/tests/test_runtime_validation_result.rs) | Receiver compatibility stays on typed opening/redaction states, while `RECEIVE_INVALID_PROOF` remains an outward-only compatibility label. |
| E7 | [stealth_scanner.rs](../../../crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs), [types_tag_cache.rs](../../../crates/z00z_wallets/src/receiver/scan/types_tag_cache.rs), [test_stealth_scanner_cache.rs](../../../crates/z00z_wallets/tests/test_stealth_scanner_cache.rs), [test_stealth_scanner_prefilter.rs](../../../crates/z00z_wallets/tests/test_stealth_scanner_prefilter.rs) | Strict tag-only scanning is gated by explicit completeness, `add_request(...)` is liveness-only metadata, and direct-scan fallback remains preserved when completeness is absent. |
| E8 | [output.rs](../../../crates/z00z_wallets/src/stealth/output/output.rs), [asset_impl_server_transfer.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs), [test_asset_impl_suite.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs), [test_live_path_enforcement.rs](../../../crates/z00z_wallets/tests/test_live_path_enforcement.rs), [test_e2e_send_scan.rs](../../../crates/z00z_wallets/tests/test_e2e_send_scan.rs) | Approved sender flows use validated builders, raw builders stay explicit seams, and live RPC plus routed simulator entrypoints are guarded against regression. |
| E9 | [043-coverage.md](043-coverage.md), [043-SUMMARY.md](043-SUMMARY.md) | Every EV, PH43, D-043, and AC-043 row has landed evidence or a spec-backed note, and the phase closeout narrative matches the decisive test outputs. |

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Evidence | Status |
|-----------|----------|-----------|-------------|------------|----------|--------|
| T-043-01-01 | T | `043-coverage.md` row set | mitigate | Require every EV, PH43, D-043, and AC-043 identifier to appear with one evidence slot. | E9 | closed |
| T-043-01-02 | E | inventory scope | mitigate | Keep unrelated TODO families and vendor paths explicitly excluded in the ledger. | E9 | closed |
| T-043-01-03 | R | failing-anchor ownership | mitigate | Bind every seam to one named existing or newly added test home before behavior work starts. | E1, E2, E3, E4, E5, E6, E7, E8, E9 | closed |
| T-043-02-01 | T | `sum_inputs` | mitigate | Require an explicit resolved-input contract and reject public `TxInputWire` bytes. | E1 | closed |
| T-043-02-02 | I | canonical balance helper | mitigate | Keep fee outputs on the output side and reject non-zero commitment delta. | E2 | closed |
| T-043-02-03 | R | public verifier surface | mitigate | Route public package checks through `verify_full_tx_package(...)` and document limits precisely. | E2 | closed |
| T-043-03-01 | I | storage proof API | mitigate | Keep `ProofBlob`/`chk_*` as the only witness verification entry and avoid exposing raw JMT proof types. | E3 | closed |
| T-043-03-02 | T | conservation audit | mitigate | Require explicit validated leaves plus commitment/proof evidence and return typed mismatch classes. | E4 | closed |
| T-043-03-03 | D | asset-class audit | mitigate | Keep asset-class recomputation operator-invoked and outside canonical tx admission. | E4 | closed |
| T-043-04-01 | T | forensic import | mitigate | Verify tx bytes, tx hashes, and manifest hashes before any state mutation. | E5 | closed |
| T-043-04-02 | I | `.wlt` semantics | mitigate | Keep tx-history outside `WalletPersistenceState` and require explicit import mode. | E5 | closed |
| T-043-04-03 | I | archive file I/O | mitigate | Route archive persistence through `z00z_utils` abstractions and reject direct `std::fs` seams. | E5 | closed |
| T-043-05-01 | I | receive DTO | mitigate | Use explicit typed opening/redaction states instead of placeholder field semantics. | E6 | closed |
| T-043-05-02 | R | status/report mapping | mitigate | Preserve precise internal reject classes and map to `RECEIVE_INVALID_PROOF` only at the outward compatibility edge. | E6 | closed |
| T-043-05-03 | T | logs/docs | mitigate | Remove wording that implies a downstream proof verifier ran on detector-side failure. | E6 | closed |
| T-043-06-01 | T | `background_scan_strategy(...)` | mitigate | Gate `TagFilterOnly` behind explicit completeness state, not cache size. | E7 | closed |
| T-043-06-02 | I | tag-context materialization | mitigate | Keep `add_request` as liveness-only metadata and require concrete `Tag16Context` coverage. | E7 | closed |
| T-043-06-03 | D | fallback coverage | mitigate | Preserve direct-scan fallback when strict completeness is absent. | E7 | closed |
| T-043-07-01 | E | approved sender flow | mitigate | Replace live accepted-flow raw-builder call sites with validated constructors. | E8 | closed |
| T-043-07-02 | T | raw-builder contract | mitigate | Keep raw builders explicitly raw and documented as requiring prior validation. | E8 | closed |
| T-043-07-03 | R | RPC send path | mitigate | Add source-shape and end-to-end tests that fail on raw-builder regression. | E8 | closed |
| T-043-08-01 | R | `043-coverage.md` evidence slots | mitigate | Rewrite the ledger only from actual narrow-gate outcomes and named tests. | E9 | closed |
| T-043-08-02 | D | regression wave | mitigate | Run the narrow suites before the broad release gate and stop on phase-scoped failures. | E9 | closed |
| T-043-08-03 | I | typed failure classes | mitigate | Preserve separate storage and wallet mismatch categories in both tests and ledger notes. | E2, E3, E4, E9 | closed |
| T-043-09-01 | R | `043-coverage.md` evidence slots | mitigate | Update receive/tag/output rows only from actual narrow-gate outcomes. | E9 | closed |
| T-043-09-02 | T | strict tag/send-path regressions | mitigate | Keep negative coverage for incomplete tag contexts, best-effort fallback loss, and raw-builder/raw-serial-builder use on the live path plus routed simulator entrypoints. | E7, E8 | closed |
| T-043-09-03 | I | compatibility/status claims | mitigate | Preserve precise internal reject tests while checking outward compatibility remains stable. | E6, E9 | closed |
| T-043-10-01 | R | `043-SUMMARY.md` | mitigate | Require copied or linked decisive command outputs plus residual-risk notes. | E9 | closed |
| T-043-10-02 | T | final coverage ledger | mitigate | Mark every EV, PH43, D-043, and AC-043 row with landed evidence or explicit spec-backed deferral. | E9 | closed |
| T-043-10-03 | I | phase-close claim | mitigate | Run the final simulator truth gates after narrow and broad archive validation and document any justified blocker explicitly. | E8, E9 | closed |

## Additive 043-18 Verification

| Threat ID | Category | Component | Disposition | Mitigation | Evidence | Status |
|-----------|----------|-----------|-------------|------------|----------|--------|
| T-043-18-01 | R | evidence ledger | mitigate | Map every claim to command output, test name, or explicit deferral. | E9 | closed |
| T-043-18-02 | I | redaction | mitigate | Re-run summary and coverage redaction grep after all edits. | E9 | closed |
| T-043-18-03 | T | planning authority | mitigate | Reject any `*-2` parallel evidence artifact. | E9 | closed |

## Accepted Risks Log

No accepted risks.

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-05-07 | 30 | 30 | 0 | GitHub Copilot (GPT-5.4 mini) |
| 2026-05-08 | 33 | 33 | 0 | GitHub Copilot |

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-05-08
