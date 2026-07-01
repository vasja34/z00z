---
phase: 047
slug: wallet-redesign
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-20
---

# Phase 047 - Validation Strategy

> Reconstructed Nyquist validation for Phase 047 from State B using the
> repo-local validate-phase workflow, `.planning/config.json`, the full
> `047-01..08-PLAN.md` and `047-01..08-SUMMARY.md` chain, and existing
> phase-local proof tests. No uncovered Phase 047 requirement remained after
> routing `047-wallet-redesign-spec.md` through `047-SPEC-COVERAGE.md`, so this
> artifact records a fully covered phase rather than opening new test work.

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust release-mode unit and integration tests, simulator surface tests, repository bootstrap checks, and lat.md link validation |
| **Config file** | `Cargo.toml` and `.planning/config.json` (`workflow.nyquist_validation: true`) |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | workspace-dependent; release-style sweep |

## Sampling Rate

- After every task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
- After every numbered wave: rerun the strongest focused release command recorded in the matching `047-0N-SUMMARY.md`.
- Before final closeout: rerun `cargo test --release --features test-fast --features wallet_debug_dump`, then rerun phase-local truth anchors when the touched slice is simulator wording, live-path enforcement, or backup or restore boundaries.
- Max feedback latency: bounded by the bootstrap gate plus the strongest focused release command for the active Phase 047 slice.

## Evidence Snapshot

- `.planning/config.json` explicitly enables `workflow.nyquist_validation: true`, so Nyquist validation is active for this repository.
- `047-SPEC-COVERAGE.md` preserves a complete section-by-section crosswalk from `047-wallet-redesign-spec.md` into `047-01..08-PLAN.md` and states that `REQ-001..020` and `AC-001..013` are routed.
- `047-01-SUMMARY.md` through `047-08-SUMMARY.md` all exist and each records a green bootstrap gate plus a green release-style cargo verification pass.
- `crates/z00z_wallets/tests/test_phase047_truth.rs` blocks stale Phase 047 wording and stale authority claims across the RPC, receive, backup, and storage seams.
- `crates/z00z_wallets/tests/test_live_path_enforcement.rs` proves the live asset-import verification path is real and rejects invalid proof bytes instead of falling back to stub behavior.
- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`, `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs`, and `crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs` cover the explicit pack, staged restore, reopen, and identity-boundary seams needed by Waves 07 and 08.
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` proves Stage 13 wording, log markers, restore, reopen, tamper, and no-Snapshot-authority drift on the simulator surface.
- Fresh reconstruction reruns on `2026-05-20` passed for `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase047_truth -- --nocapture`, `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_live_path_enforcement -- --nocapture`, `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture`, and targeted truth-string scans across the Phase 047 docs and live wallet/simulator surfaces.

## Requirement Coverage Summary

| Requirement | Status | Evidence |
| ----------- | ------ | -------- |
| `REQ-001`, `REQ-002` profile cutover and live wallet authority | COVERED | `047-03-SUMMARY.md`, `047-04-SUMMARY.md`, `create_and_save_keep_profile_only_wlt`, `normal_save_does_not_rewrite_snapshot_claimed_assets`, and `test_save_skips_snapshot_rewrite` |
| `REQ-003`, `REQ-004`, `REQ-005` low-level object rewrite and indexed reads | COVERED | `047-02-SUMMARY.md`, `test_phase047_read_objects_by_index_supports_exact_queries_and_cursor_pages`, `test_phase047_validate_object_index_rows_detects_missing_rows`, and store-level `write_object_by_id(...)` coverage in `redb_wallet_store/tests.rs` |
| `REQ-006`, `REQ-007`, `REQ-008` tx build, cancel, import, and reconcile lifecycle | COVERED | `047-06-SUMMARY.md`, `test_tx_import_reconcile_portable`, `test_tx_import_adds_outputs`, `test_tx_reconcile_requires_confirmation_evidence`, `test_tx_reconcile_rejects_mismatched_evidence`, and `test_tx_list_reflects_cancel` |
| `REQ-009` backup/restore continuity and JSONL tx-history plane | COVERED | `047-07-SUMMARY.md`, `test_export_import_roundtrip_restores`, `test_forensic_export_requires_jsonl`, and `WalletPlusHistory` assertions in the backup/import suites |
| `REQ-010`, `REQ-011` receive and scan-state coupling | COVERED | `047-05-SUMMARY.md`, `test_recv_range_restart`, `test_recv_range_*`, and `test_recv_route_gate` |
| `REQ-012`, `REQ-013` schema/object-kind/payload groundwork | COVERED | `047-01-SUMMARY.md`, `test_phase047_new_payload_versions_supported`, `test_phase047_schema_yaml_matches_rust_object_kinds`, `test_phase047_owned_asset_index_tags_roundtrip`, and `test_phase047_debug_decode_supports_new_object_kinds` |
| `REQ-014` stable object-id and index semantics | COVERED | `047-01-SUMMARY.md`, `047-02-SUMMARY.md`, `test_phase047_owned_asset_store_duplicate_reserve_release_confirm`, `test_phase047_validate_object_index_rows_detects_missing_rows`, and the `id_query` focused rerun |
| `REQ-015`, `REQ-016`, `REQ-017`, `REQ-018` YAML-backed defaults, restore identity, and backup policy | COVERED | `047-03-SUMMARY.md`, `047-07-SUMMARY.md`, `wallet_yaml`, `test_recover_from_seed_uses_wallet_yaml_gap_limit`, `test_open_wallet_discovers_unlocks`, `test_open_wallet_bytes_unlocks`, and `test_open_wallet_identity_mismatch` |
| `REQ-019`, `REQ-020` simulator truth, existing-test migration, and final authority wording | COVERED | `047-05-SUMMARY.md`, `047-07-SUMMARY.md`, `047-08-SUMMARY.md`, `test_phase047_truth`, `test_live_path_verify_complete`, `test_live_path_stub_removed`, and `test_scenario1_stage_surface` |

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | ---------- | --------------- | --------- | ----------------- | ----------- | ------ |
| `047-01 T1..T2` | `01` | schema-groundwork | `REQ-012`, `REQ-013`, `REQ-014`, `REQ-019`, `REQ-020` | `T-047-01..03` | Object ids, payload versions, index tags, and debug decode support align exactly across Rust and schema artifacts before later waves rely on them. | bootstrap + focused release tests + workspace release | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test --release --features test-fast --features wallet_debug_dump`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets phase047_ -- --nocapture` | ✅ summary + `redb_wallet_store/tests.rs` | ✅ green |
| `047-02 T1..T3` | `02` | object-update-and-index-api | `REQ-003`, `REQ-004`, `REQ-005`, `REQ-014` | `T-047-04..06` | Production `write_object_by_id(...)` rewrites by stable object id, canonical index paging stays exact, and stale index rows fail closed before owned-asset authority depends on them. | bootstrap + focused release tests + workspace release | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test --release --features test-fast --features wallet_debug_dump`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets phase047_ -- --nocapture` | ✅ summary + `write_object_by_id(...)` store tests | ✅ green |
| `047-03 T1..T3` | `03` | profile-and-yaml-cutover | `REQ-001`, `REQ-002`, `REQ-015`, `REQ-016`, `REQ-017`, `REQ-018` | `T-047-07..09` | Live create/open/save routes through `WalletProfilePayload`, runtime defaults come from YAML-backed config, and normal saves stop rewriting Snapshot claimed-asset state. | bootstrap + focused release tests + workspace release | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test --release --features test-fast --features wallet_debug_dump`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets phase047_ -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets wallet_yaml -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets create_and_save_keep_profile_only_wlt -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets normal_save_does_not_rewrite_snapshot_claimed_assets -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets --test test_phase2_production_hardening test_encrypted_load_fails_tampering -- --nocapture` | ✅ summary + `wallet_service_tests.rs` + `test_app_service_suite.rs` | ✅ green |
| `047-04 T1..T3` | `04` | owned-asset-authority | `REQ-001`, `REQ-002`, `REQ-003`, `REQ-004`, `REQ-005`, `REQ-014`, `REQ-015` | `T-047-10..12` | `OwnedAssetPayload` becomes the canonical live asset authority, duplicate inserts fail closed, and public asset/catalog surfaces read from the object store instead of Snapshot-owned vectors. | bootstrap + focused release tests + workspace release | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test --release --features test-fast --features wallet_debug_dump`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets id_query -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_save_skips_snapshot_rewrite -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_tx_import_reconcile_portable -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_pipeline_genesis_tx test_s4_bob_pending_ok -- --nocapture` | ✅ summary + `owned_assets.rs` + asset id-query tests | ✅ green |
| `047-05 T1..T3` | `05` | receive-and-scan-coupling | `REQ-010`, `REQ-011`, `REQ-019`, `REQ-020` | `T-047-13..15` | `recv_range(...)` persists owned assets through the wallet-side store, scan-state updates stay replay-safe, and compatibility receive helpers never become a second authority plane. | bootstrap + focused release tests + workspace release | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test --release --features test-fast --features wallet_debug_dump`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_recv_range_ -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_recv_route_gate -- --nocapture` | ✅ summary + `wallet_service_tests.rs` | ✅ green |
| `047-06 T1..T3` | `06` | tx-lifecycle-cutover | `REQ-006`, `REQ-007`, `REQ-008`, `REQ-009`, `REQ-014`, `REQ-019` | `T-047-16..18` | Build reserves only spendable owned assets, cancel/import/reconcile mutations fail closed, and asset balance/details/pending views derive from owned-asset lifecycle state instead of Snapshot assumptions. | bootstrap + focused release tests + workspace release | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test --release --features test-fast --features wallet_debug_dump`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_recv_range_restart -- --nocapture` | ✅ summary + `test_tx_pending_body.rs` + `wallet_service_tests.rs` | ✅ green |
| `047-07 T1..T3` | `07` | backup-restore-export-cutover | `REQ-009`, `REQ-015`, `REQ-016`, `REQ-017`, `REQ-018`, `REQ-019` | `T-047-19..21` | Manifest-backed export packs encode profile-first `.wlt` state, `WalletPlusHistory` stages `.wlt` plus JSONL atomically, and Snapshot stays compatibility-only for one-shot legacy input. | bootstrap + workspace release + phase-local proof suites | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test --release --features test-fast --features wallet_debug_dump` | ✅ summary + `test_wallet_export_pack_boundary.rs` + `test_wallet_persistence_backup_service.rs` + `test_open_wallet_source_discovery.rs` | ✅ green |
| `047-08 T1..T3` | `08` | simulator-and-doc-honesty | `REQ-019`, `REQ-020` | `T-047-22..24` | Stage 13 proves the live owned-asset + JSONL truth with restore, tamper, and reopen execution; stale Snapshot-authority wording is rejected; and phase-local docs/tests agree on the same storage boundary. | bootstrap + focused release tests + workspace release | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_session_expired_rpc_code -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets test_derive_session_expired_code -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump` | ✅ summary + `test_phase047_truth.rs` + `test_live_path_enforcement.rs` + `test_scenario1_stage_surface.rs` | ✅ green |

## Wave 0 Requirements

Existing infrastructure already covers the landed Phase 047 validation surface.

No new framework installation was required beyond the repository bootstrap gate,
the existing Rust cargo suites, simulator surface tests, and targeted
truth-string scans.

The local `gsd-sdk` wrapper was unavailable during this reconstruction, so the
workflow was executed manually with the repo-local files
`./.github/get-shit-done/workflows/validate-phase.md` and
`./.github/get-shit-done/templates/VALIDATION.md`. This did not create a phase
gap because `.planning/config.json` explicitly enables Nyquist validation and
all numbered summary artifacts already existed.

## Manual-Only Verifications

No residual manual-only product behaviors remain inside the landed Phase 047
scope. Every routed requirement is backed by either a focused release-style
command recorded in the numbered summaries or by a summary-backed broad
workspace rerun tied to an explicit phase-local proof file.

## Open Gaps And Watchpoints

- No uncovered `REQ-001..020` row remained after crosswalking
  `047-wallet-redesign-spec.md`, `047-SPEC-COVERAGE.md`, and the numbered
  summaries.
- This artifact must not be read as claiming that canonical tx history moved
  into `.wlt`; Phase 047 still keeps `wallet_<stem>_tx_history.jsonl` as the
  live history plane, exactly as `047-07` and `047-08` record.
- The only workflow deviation was orchestration-level: `gsd-sdk` was missing in
  the local environment, so this file is a State-B reconstruction rather than a
  wrapper-generated artifact.

## Validation Sign-Off

- [x] Existing infrastructure detected and reused
- [x] Completed numbered plan waves have command-backed evidence
- [x] Wave 0 dependencies are already satisfied and Nyquist config is enabled
- [x] No watch-mode flags are required
- [x] Existing proof suites cover truth wording, live-path enforcement,
      receive/scan, tx lifecycle, backup/restore, and Stage 13 surfaces
- [x] All Phase 047 plan waves have automated verification evidence
- [x] No new test files were required for closeout
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** verified 2026-05-20

## Reconstruction Notes

This file was reconstructed under validate-phase State B from:

- `.planning/config.json`
- `.planning/phases/047-wallet-redesign/047-wallet-redesign-spec.md`
- `.planning/phases/047-wallet-redesign/047-SPEC-COVERAGE.md`
- `.planning/phases/047-wallet-redesign/047-01-PLAN.md`
- `.planning/phases/047-wallet-redesign/047-02-PLAN.md`
- `.planning/phases/047-wallet-redesign/047-03-PLAN.md`
- `.planning/phases/047-wallet-redesign/047-04-PLAN.md`
- `.planning/phases/047-wallet-redesign/047-05-PLAN.md`
- `.planning/phases/047-wallet-redesign/047-06-PLAN.md`
- `.planning/phases/047-wallet-redesign/047-07-PLAN.md`
- `.planning/phases/047-wallet-redesign/047-08-PLAN.md`
- `.planning/phases/047-wallet-redesign/047-01-SUMMARY.md`
- `.planning/phases/047-wallet-redesign/047-02-SUMMARY.md`
- `.planning/phases/047-wallet-redesign/047-03-SUMMARY.md`
- `.planning/phases/047-wallet-redesign/047-04-SUMMARY.md`
- `.planning/phases/047-wallet-redesign/047-05-SUMMARY.md`
- `.planning/phases/047-wallet-redesign/047-06-SUMMARY.md`
- `.planning/phases/047-wallet-redesign/047-07-SUMMARY.md`
- `.planning/phases/047-wallet-redesign/047-08-SUMMARY.md`
- `.planning/phases/047-wallet-redesign/047-SECURITY.md`
- `crates/z00z_wallets/tests/test_phase047_truth.rs`
- `crates/z00z_wallets/tests/test_live_path_enforcement.rs`
- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
- `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs`
- `crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- the existing validation-file patterns under `.planning/phases/000/*-VALIDATION.md`

Gap audit result: no missing requirement row and no missing automated phase
proof file remained for the landed Phase 047 scope.

Generated test files:

- none

---
*Phase: 047-wallet-redesign*
*Reconstructed: 2026-05-20*
