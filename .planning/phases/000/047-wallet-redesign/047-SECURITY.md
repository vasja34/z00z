---
phase: 047
slug: wallet-redesign
status: verified
threats_open: 0
asvs_level: 1
created: 2026-05-20
---

# Phase 047 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
| -------- | ----------- | ------------- |
| Live `.wlt` store | RedB-backed encrypted wallet authority for profile, owned assets, scan state, TOFU pins, key refs, and backup manifest. | Wallet profile data, owned assets, scan cursor, secret-bearing metadata |
| Tx-history sidecar | Explicit JSONL transaction-history plane kept outside `.wlt` and validated separately during export and restore. | Pending or confirmed tx records, confirmation evidence, history replay bytes |
| Backup export and restore | Encrypted wallet export payload plus staged restore promotion path. | Password-protected backup payloads, staged `.wlt` bytes, staged JSONL history |
| Receive and tx RPC lifecycle | Runtime scanner, asset catalog, build, cancel, import, and reconcile surfaces that mutate or expose wallet-owned state. | Checkpoint scan chunks, payment requests, tx packages, owned asset ids |
| Simulator and phase-local truth surfaces | Stage 13 reports, logs, tests, and local spec copies that can misstate the live authority model if they drift. | User-facing proof text, storage-root claims, lifecycle assertions |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
| --------- | -------- | --------- | ----------- | ---------- | ------ |
| T-047-01 | Tampering | Schema object ids and payload vocabulary | mitigate | `ObjectKindId`, payload versions, and the YAML schema stay locked together, with `test_phase047_new_payload_versions_supported` and `test_phase047_schema_yaml_matches_rust_object_kinds` guarding drift. | closed |
| T-047-02 | Tampering | Supported-version gate | mitigate | `is_supported_payload_version(...)` plus the store-level supported-version tests reject unsupported kind or version drift before object writes land. | closed |
| T-047-03 | Repudiation | Debug dump and schema introspection | mitigate | `wallet_debug_dump` decode coverage was widened for the new wallet-profile, owned-asset, tx, tx-event, and backup-manifest kinds, with `test_phase047_debug_decode_supports_new_object_kinds` proving operator visibility. | closed |
| T-047-04 | Tampering | Owned-asset object writes and index rows | mitigate | `write_object_by_id(...)` / `write_payload_at_id(...)` reject unsupported versions, and owned-asset index write coverage proves object and index rows are committed together. | closed |
| T-047-05 | Tampering | Owned-asset index reads and validation | mitigate | Exact-query and cursor-page tests plus `validate_object_index_rows(...)` coverage prove missing or stale index rows are detected instead of silently accepted. | closed |
| T-047-06 | Tampering | Duplicate owned-asset ingest and service wrappers | mitigate | The `.wlt` owned-asset store is the only write authority; duplicate same-wire inserts become idempotent and conflicting duplicates fail closed in `test_phase047_owned_asset_store_duplicate_reserve_release_confirm`. | closed |
| T-047-07 | Tampering | Create and open profile cutover | mitigate | The live open path reads `WalletProfilePayload` first, falls back to Snapshot only for one-shot compatibility, backfills a profile when needed, and `test_save_skips_snapshot_rewrite` proves normal saves no longer rewrite snapshot claimed assets. | closed |
| T-047-08 | Tampering | Backup defaults and config resolution | mitigate | `resolve_wallet_backup_defaults()` now resolves backup settings from checked wallet YAML or env-backed config and rejects zero or malformed values instead of using hardcoded runtime defaults. | closed |
| T-047-09 | Denial of Service | Recovery gap-limit enforcement | mitigate | `resolve_wallet_recovery_gap_limit()` rejects invalid zero values, and `test_recover_from_seed_uses_wallet_yaml_gap_limit` proves recovery obeys wallet YAML gap-limit settings instead of hardcoded scan widths. | closed |
| T-047-10 | Tampering | Duplicate owned-asset ids | mitigate | `put_owned_asset(...)` and `persist_scan_batch(...)` reject conflicting duplicate asset ids and accept exact duplicates idempotently, preventing a second live asset row for the same id. | closed |
| T-047-11 | Tampering | Reserve, release, and confirm transitions | mitigate | Owned assets move through `Spendable -> PendingSpend -> Spent` only through the live store transitions, with release and confirm behavior covered by `test_phase047_owned_asset_store_duplicate_reserve_release_confirm`. | closed |
| T-047-12 | Tampering | Asset catalog authority | mitigate | `list_assets_impl`, `get_asset_balance_impl`, and `get_asset_details_impl` load from stored owned assets plus reserved-id state, so compatibility stubs do not become a second catalog plane. | closed |
| T-047-13 | Tampering | Canonical receive persistence lane | mitigate | `recv_range(...)` remains the canonical receive path, persists detected owned assets and `ScanStatePayload` together through `persist_scan_batch(...)`, and keeps single-asset helpers explicitly compatibility-only. | closed |
| T-047-14 | Tampering | Scan cursor replay and duplicate receive recovery | mitigate | Restart and replay coverage in wallet-service tests proves receive resumes from persisted scan state and does not create duplicate live owned assets when a cursor is rewound. | closed |
| T-047-15 | Repudiation | Compatibility receive helper boundary | mitigate | `recv_route(...)`, `scan_asset_report(...)`, and `receive_asset(...)` explicitly mark report-only or compatibility behavior so only the canonical receive lane can claim live persistence authority. | closed |
| T-047-16 | Tampering | Tx build reservation and rollback | mitigate | `build_transaction_impl(...)` reserves live inputs before persisting the pending tx, and on pending-tx persistence failure it rolls the reservation back through `release_claimed_asset_reservation(...)`. | closed |
| T-047-17 | Tampering | Tx reconcile and claimed-asset mutation | mitigate | `reconcile_transaction_impl(...)` only confirms spent inputs and wallet-owned outputs after confirmation evidence validates, while reconcile rejection tests prove missing or mismatched evidence leaves claimed-asset state unchanged. | closed |
| T-047-18 | Repudiation | Compatibility asset-op surface | mitigate | `wallet.asset.send_asset`, `stake_assets`, `swap_assets`, and `unstake_assets` are documented as compatibility-only, and the stake or unstake tests prove they stay on a non-canonical UX round-trip surface instead of claiming ledger mutation authority. | closed |
| T-047-19 | Tampering | Backup manifest and export-pack contract | mitigate | Backup export now carries explicit profile, owned-asset, scan-state, key, and JSONL-history-plane semantics via `BackupManifestPayload`, and restore validation rejects packs that smuggle Snapshot back in as explicit authority. | closed |
| T-047-20 | Tampering | Atomic staged restore promotion | mitigate | `restore_wallet_pack_atomic(...)` stages `.wlt` and JSONL history first, commits history before `.wlt`, and rolls back or preserves prior bytes on commit or publish failure; restore tests prove wrong password, tamper, and stage failure do not mutate live wallet state. | closed |
| T-047-21 | Tampering | Snapshot compatibility bridge | mitigate | Snapshot survives only as read-only compatibility input for one-shot migration or export packaging; live restore writes explicit owned-asset payloads and explicit export packs reject Snapshot authority when a profile is present. | closed |
| T-047-22 | Repudiation | Stage 13 report and log wording | mitigate | `test_scenario1_stage_surface` and `assert_no_stage13_snapshot_authority_language(...)` require Stage 13 report and log output to describe `OwnedAssetPayload` plus JSONL truth and reject stale Snapshot-authority phrasing. | closed |
| T-047-23 | Repudiation | Existing wallet and simulator suite truth surface | mitigate | The live-path enforcement tests and Stage 13 scenario surface now assert the same storage story: persisted owned-asset objects, explicit tx-history sidecar, real cancel-release, real import-owned-output, and real restore-history checks. | closed |
| T-047-24 | Repudiation | Phase-local spec and RPC doc alignment | mitigate | `047-wallet-addon-spec.md` Decision 9 and the updated RPC doc comments align the local spec, simulator wording, and compatibility asset surfaces around one rule: Snapshot is compatibility-only and confirmed spend authority lives on `wallet.tx.*` plus reconcile. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
| ---------- | ------------- | ------ | ---- | ------ |
| 2026-05-20 | 24 | 24 | 0 | GitHub Copilot |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-05-20
