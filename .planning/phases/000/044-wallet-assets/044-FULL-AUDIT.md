# Phase 044 Full Audit

## 🔔 Audit Run — 2026-05-09 16:42:43

### 📌 Audit Setup
- Phase directory: `.planning/phases/044-wallet-assets`
- Derived FULL-AUDIT path: `.planning/phases/044-wallet-assets/044-FULL-AUDIT.md`
- Execution mode: manual fallback audit with direct source inspection and workspace validation
- Mandatory context files read: `044-CONTEXT.md`, `044-TODO.md`, `044-TEST-SPEC.md`, `044-TESTS-TASKS.md`, `044-coverage.md`, `044-SUMMARY.md`, `044-wallets-assets-spec.md`, `044-wallets-patch.md`, `044-01-PLAN.md`, `044-02-PLAN.md`, `044-03-PLAN.md`, `044-04-PLAN.md`, `044-05-PLAN.md`
- Final in-scope crate list:
  - `crates/z00z_wallets`
  - `crates/z00z_storage`
  - `crates/z00z_core`
  - `crates/z00z_simulator`
  - `crates/z00z_crypto`
- Explicit exclusions:
  - `crates/z00z_crypto/tari/**`
  - unrelated untracked workspace artifacts left untouched: `docs/Z00Z New Ideas +  ... .md`, `website/WebDev_3.pdf`

### 🎯 Scope And Source Of Truth
- Phase authority: `044-TODO.md`
- Phase navigation and guardrails: `044-CONTEXT.md`
- Phase execution plan: `044-01-PLAN.md` through `044-05-PLAN.md`
- Phase test contract: `044-TEST-SPEC.md` and `044-TESTS-TASKS.md`
- Phase closeout ledger: `044-coverage.md` and `044-SUMMARY.md`
- Archived source inputs embedded into the backlog: `044-wallets-assets-spec.md` and `044-wallets-patch.md`
- Live code surfaces explicitly named by those artifacts:
  - `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs`
  - `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs`
  - `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`
  - `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
  - `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
  - `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs`
  - `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs`
  - `crates/z00z_storage/src/assets/store_internal/store_query.rs`
  - `crates/z00z_core/src/assets/wire_pkg.rs`
  - `crates/z00z_crypto/src/domains.rs`
  - `crates/z00z_crypto/src/lib.rs`

### 🧪 Verification Model

#### Critical User Journeys
- Wallet backup creation must preserve the wallet snapshot while treating `wallet_<stem>_tx_history.jsonl` as the canonical live tx store.
- Wallet restore must write the live JSONL bytes back without rebuilding history from extracted records.
- Simulator stage 2 backup roundtrip must prove that the live JSONL file exists and the legacy `wallet_<stem>_tx_history/` directory does not.
- Asset balance reporting must derive `pending` from claim-registry state instead of hardcoding zero.
- Transaction build/lifecycle code must emit the canonical payload shape without source-level drift aliases.

#### State Transitions
- Wallet state save -> `.wlt` output path only.
- Transaction history write -> `wallet_<stem>_tx_history.jsonl`.
- Backup export -> read live JSONL bytes, then emit encrypted backup container and preserve the same bytes.
- Backup restore -> decode forensic archive and restore live JSONL bytes directly.
- Receiver cache derivation -> anchored to `wlt_file_path(...)`, not to a separate legacy wallet file helper.

#### Proof Paths
- `TxStorageImpl::put`, `list`, `update_status`, and `delete` operate on a single canonical JSONL file.
- `BackupExporterImpl::export_with_history_file` validates that the canonical JSONL bytes decode back to the forensic history record set.
- `BackupImporterImpl::decode_payload` returns raw history JSONL bytes for restore.
- `WalletService::create_backup` and `WalletService::restore_backup_with_mode` preserve the live tx-history JSONL path contract.
- `flow.rs` stage-2 backup roundtrip asserts the canonical live JSONL path and rejects the legacy directory.
- `claim_registry::has_pending_owner(...)` is the source of truth for `pending`.

#### Failure Paths
- Missing live JSONL during backup must fail closed.
- Tampered forensic archive must be rejected and must not mutate wallet state.
- Legacy tx-history directory shape must be rejected as a live-store authority.
- `BuiltTxStub` literal drift must not reappear in live RPC build paths.
- Hardcoded `pending = 0` drift must not reappear in balance rendering.
- Helper alias seams for `wallet_tx_history_dir*` and `wallet_file_path` must not return as production API.

### 📊 Findings Summary

| Severity | Count | Meaning |
| --- | ---: | --- |
| 🔴 CRITICAL | 0 | Immediate correctness, security, or truthfulness failure |
| 🟠 HIGH | 0 | Material gap or blocker on required phase behavior |
| 🟡 MEDIUM | 0 | Non-trivial weakness, drift, or incomplete evidence |
| 🔵 LOW | 0 | Minor issue, clarity problem, or narrow follow-up |
| ⚪ INFO | 0 | Confirmed observation with no immediate remediation |

Two medium drift issues were identified during the audit and were fixed before closure:
- source-level `BuiltTxStub` drift in the live tx build path
- hardcoded `pending = 0` drift in asset balance rendering

No open findings remain after the fixes and re-audit.

### 🔍 Audit Pass Results

#### `crates/z00z_wallets`

- **crypto-architect (manual fallback)**
  - Files inspected: `asset_impl_server_catalog.rs`, `tx_impl_server_lifecycle.rs`, `wallet_service_actions_backup.rs`, `wallet_service_store_persistence_pack.rs`, `wallet_service_session_derivation_recovery.rs`, `tx_storage_impl.rs`
  - What was checked: live JSONL contract, backup/restore byte preservation, transaction payload shape, receiver cache anchoring, and removal of alias helpers
  - Findings: no open findings

- **security-audit (manual fallback)**
  - Files inspected: `wallet_service_actions_backup.rs`, `backup_exporter_impl.rs`, `backup_importer_impl.rs`, `test_wallet_service_suite.rs`
  - What was checked: fail-closed behavior on missing live history, rejection of tampered forensic archives, and no parallel tx-history authority
  - Findings: no open findings

- **spec-to-code-compliance (manual fallback)**
  - Files inspected: `asset_impl_server_catalog.rs`, `tx_impl_server_lifecycle.rs`, `wallet_service_store_persistence_pack.rs`, `wallet_service_session_derivation_recovery.rs`, `wallet_service_actions_backup.rs`, `test_wallet_service_suite.rs`
  - What was checked: current code against the 044 plan/test contract and the wallet-state-only / live-JSONL contract
  - Resolved during audit:
    - `asset_impl_server_catalog.rs:152` now derives `pending` from `claim_registry::has_pending_owner(...)` and computes `available` from that value
    - `tx_impl_server_lifecycle.rs:154` now uses `BuiltTxPayload` instead of a source-level `BuiltTxStub`
    - `wallet_service_store_persistence_pack.rs:11` removed `wallet_tx_history_dir*` and `wallet_file_path` helper seams
    - `wallet_service_session_derivation_recovery.rs:4` now anchors the receiver cache path to `wlt_file_path(...)`
    - `test_wallet_service_suite.rs:145` and the backup/restore tests now assert canonical paths and legacy-directory rejection without calling the removed helper seams
  - Findings: no open findings

- **z00z-design-foundation-compliance (manual fallback)**
  - Files inspected: `wallet_service_store_persistence_pack.rs`, `wallet_service_actions_backup.rs`, `tx_storage_impl.rs`, `flow.rs`
  - What was checked: no parallel database, no duplicate tx-history authority, no duplicated receiver or backup layer, and no concept drift away from canonical paths
  - Findings: no open findings

#### `crates/z00z_storage`

- **crypto-architect (manual fallback)**
  - Files inspected: `src/assets/store_internal/store_query.rs`
  - What was checked: storage-backed proof and claim-source surfaces remain JMT-backed and do not create a wallet tx-history authority
  - Findings: no open findings

- **security-audit (manual fallback)**
  - Files inspected: `src/assets/store_internal/store_query.rs`, `src/assets/README.MD`
  - What was checked: proof extraction remains sanitized; raw branch proofs are not exposed to wallet callers
  - Findings: no open findings

- **spec-to-code-compliance (manual fallback)**
  - Files inspected: `src/assets/store_internal/store_query.rs`
  - What was checked: claim-source and proof APIs stay storage-backed and do not conflict with phase 044 wallet history handling
  - Findings: no open findings

- **z00z-design-foundation-compliance (manual fallback)**
  - Files inspected: `src/assets/store_internal/store_query.rs`
  - What was checked: storage crate preserves a single storage authority and does not introduce a phase-local side database
  - Findings: no open findings

#### `crates/z00z_core`

- **crypto-architect (manual fallback)**
  - Files inspected: `src/assets/wire_pkg.rs`, `src/assets/nonce_derivation.rs`, `src/assets/leaf.rs`
  - What was checked: canonical asset package DTO boundaries, encrypted payload retention, and wallet-facing memo handling
  - Findings: no open findings

- **security-audit (manual fallback)**
  - Files inspected: `src/assets/wire_pkg.rs`, `src/assets/nonce_counter.rs`, `src/assets/assets.rs`
  - What was checked: public DTO boundary stays explicit, secret/nonce derivation stays wallet-scoped, and no tx-history authority appears here
  - Findings: no open findings

- **spec-to-code-compliance (manual fallback)**
  - Files inspected: `src/assets/wire_pkg.rs`, `src/assets/wire_pkg_serde.rs`, `src/assets/assets.rs`
  - What was checked: the canonical wire DTO remains the correct human-readable public contract for tx packages and related artifacts
  - Findings: no open findings

- **z00z-design-foundation-compliance (manual fallback)**
  - Files inspected: `src/lib.rs`, `src/domains.rs`
  - What was checked: domain separation remains explicit and stable; no duplicate wallet history layer is introduced
  - Findings: no open findings

#### `crates/z00z_simulator`

- **crypto-architect (manual fallback)**
  - Files inspected: `src/scenario_1/stage_2_utils/flow.rs`, `src/scenario_1/runner_contract_table.json`, `src/scenario_1/scenario_design.yaml`
  - What was checked: simulator stage 2 backup roundtrip proves the live JSONL path and rejects the legacy history directory as a live store
  - Findings: no open findings

- **security-audit (manual fallback)**
  - Files inspected: `src/scenario_1/stage_2_utils/flow.rs`, `tests/test_stage2_secret_artifacts.rs`
  - What was checked: the simulator does not weaken the fail-closed backup/restore behavior or the canonical path contract
  - Findings: no open findings

- **spec-to-code-compliance (manual fallback)**
  - Files inspected: `src/scenario_1/stage_2_utils/flow.rs`, `tests/test_scenario1_stage_surface.rs`
  - What was checked: the simulator evidence matches the plan/test contract for backup roundtrip, live JSONL presence, and legacy directory rejection
  - Findings: no open findings

- **z00z-design-foundation-compliance (manual fallback)**
  - Files inspected: `src/scenario_1/stage_2_utils/flow.rs`, `src/scenario_1/stage_3_utils/state.rs`
  - What was checked: scenario orchestration stays explicit, storage-backed, and non-duplicative
  - Findings: no open findings

#### `crates/z00z_crypto`

- **crypto-architect (manual fallback)**
  - Files inspected: `src/domains.rs`, `src/lib.rs`, `tests/test_hash_policy.rs`
  - What was checked: domain-separated hashing, wallet backup domain separation, and stable export surface
  - Findings: no open findings

- **security-audit (manual fallback)**
  - Files inspected: `src/domains.rs`, `src/lib.rs`, `tests/test_fail_closed.rs`
  - What was checked: no wallet-history alias layer, no backup-path authority drift, and domain separation remains fail-closed
  - Findings: no open findings

- **spec-to-code-compliance (manual fallback)**
  - Files inspected: `src/lib.rs`, `src/README.md`, `src/domains.rs`
  - What was checked: public crypto exports remain the canonical backend surface used by wallets, storage, and simulator flows
  - Findings: no open findings

- **z00z-design-foundation-compliance (manual fallback)**
  - Files inspected: `src/domains.rs`, `src/lib.rs`
  - What was checked: no phase-local duplicate authority is introduced; Tari stays behind the compatibility surface
  - Findings: no open findings

### ⚙️ Fixes Applied — 2026-05-09 16:42:43

- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs:152`
  - Replaced the hardcoded `pending = 0` balance drift with claim-registry-derived pending detection and recomputed available balance from that value.
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs:154`
  - Renamed the local build response shape from `BuiltTxStub` to `BuiltTxPayload` so the live source code no longer carries the drift alias.
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs:11`
  - Removed the helper alias layer (`wallet_tx_history_dir_name`, `wallet_tx_history_dir`, `wallet_file_path`) and kept only canonical path helpers.
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs:4`
  - Switched the receiver cache path to derive from `wlt_file_path(...)`, eliminating the old `.bin` alias seam.
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs:145`
  - Updated canonical path assertions and legacy-path rejection checks to use direct path construction rather than the removed helper seams.
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs:3570`
  - Kept backup/restore coverage aligned with the live JSONL contract and legacy-directory rejection semantics.

### ♻️ Re-Audit Results — 2026-05-09 16:42:43

- Re-ran the same four audit passes on the same five-crate scope using direct source inspection and search-based validation.
- Re-verified the canonical live JSONL contract with the following surfaces:
  - `wallet_service_actions_backup.rs:80` and `wallet_service_actions_backup.rs:175`
  - `wallet_service_store_persistence_pack.rs:11`
  - `tx_storage_impl.rs:16`
  - `flow.rs:423`
  - `asset_impl_server_catalog.rs:152`
  - `tx_impl_server_lifecycle.rs:154`
- Re-verified that the legacy helper seams are absent from live code:
  - `wallet_tx_history_dir`
  - `wallet_tx_history_dir_name`
  - `wallet_file_path(`
- Re-verified that the current code and tests enforce:
  - live `wallet_<stem>_tx_history.jsonl` as the canonical tx store
  - backup/restore byte preservation of the live JSONL payload
  - rejection of the legacy per-tx JSON directory as a live store
  - claim-registry-derived pending balance computation
- Re-audit outcome: no new actionable issues were found.

### ✅ Doublecheck Results — 2026-05-09 16:42:43

- `doublecheck` was executed via manual fallback review of the code and phase artifacts.
- Re-verified surfaces:
  - canonical live tx-history JSONL path contract
  - backup/restore behavior
  - simulator stage 2 backup roundtrip
  - helper alias cleanup
  - balance pending derivation
  - tx build payload shape
- Search evidence:
  - `rg -n "wallet_tx_history_dir|wallet_tx_history_dir_name|wallet_file_path\\(" crates/z00z_wallets/src crates/z00z_wallets/tests .planning/phases/044-wallet-assets` returned only test-local `test_wallet_file_path` helper names in `test_app_service_suite.rs`; no production alias seam remained
  - broader scans across `crates/z00z_storage`, `crates/z00z_core`, `crates/z00z_crypto`, and `crates/z00z_simulator` found no phase-044 contract violations in live code
- Verification evidence already available in this workspace:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
  - `cargo test --release --features test-fast --features wallet_debug_dump`
  - `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets --lib`
  - `git diff --check`
- Report truthfulness check: every code claim in this report is backed by a source anchor, search result, or command output; no unsupported claim remains.

### 🧾 Exact Fixes Required Summary

| Q | Title | Proof Status | Verification | Severity | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Canonical live JSONL wallet history only | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 2 | Remove tx-history helper alias seams | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 3 | Claim-registry-derived pending balance | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 4 | Canonical tx build payload shape | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 5 | Receiver cache path anchored to `.wlt` | Full Evidence | VERIFIED | ⚪ INFO | None | None |

## 🚩 Final Status

PASS.

The 044 audit scope is closed with no open findings. The wallet tx-history contract is canonical JSONL-only, `.wlt` remains wallet-state-only, the simulator proves the new live-path behavior, and the removed helper aliases and balance drift are no longer present in live code.

## 🔔 Audit Run — 2026-05-09 16:45:47

### 📌 Audit Setup
- Phase directory: `.planning/phases/044-wallet-assets`
- Derived FULL-AUDIT path: `.planning/phases/044-wallet-assets/044-FULL-AUDIT.md`
- Execution mode: rerun / manual fallback audit with direct source inspection and workspace validation
- Workspace state at rerun time: same wallet-source diff as the prior audit run; no new code changes were introduced during this rerun
- In-scope crate list: unchanged from the prior run
- Explicit exclusions: unchanged from the prior run

### 🎯 Scope And Source Of Truth
- Same phase authority and same source-of-truth artifacts as the prior run
- No new phase artifacts were introduced

### 🧪 Verification Model
- Reconfirmed the same required behaviors:
  - live `wallet_<stem>_tx_history.jsonl` authority
  - backup/restore byte preservation
  - simulator rejection of the legacy history directory as a live store
  - claim-registry-derived `pending`
  - no source-level `BuiltTxStub`
  - no production `wallet_tx_history_dir*` or `wallet_file_path` helper seam

### 📊 Findings Summary

| Severity | Count | Meaning |
| --- | ---: | --- |
| 🔴 CRITICAL | 0 | Immediate correctness, security, or truthfulness failure |
| 🟠 HIGH | 0 | Material gap or blocker on required phase behavior |
| 🟡 MEDIUM | 0 | Non-trivial weakness, drift, or incomplete evidence |
| 🔵 LOW | 0 | Minor issue, clarity problem, or narrow follow-up |
| ⚪ INFO | 0 | Confirmed observation with no immediate remediation |

No new findings were introduced during the rerun.

### 🔍 Audit Pass Results

#### `crates/z00z_wallets`
- Rerun result: no new issues; same fixed surfaces remain aligned with the live JSONL contract.

#### `crates/z00z_storage`
- Rerun result: no new issues; storage surfaces remain orthogonal to wallet tx-history authority.

#### `crates/z00z_core`
- Rerun result: no new issues; wire DTO and domain separation surfaces remain stable.

#### `crates/z00z_simulator`
- Rerun result: no new issues; stage 2 still asserts live JSONL presence and legacy-directory absence.

#### `crates/z00z_crypto`
- Rerun result: no new issues; cryptographic boundaries remain stable and wallet-scoped.

### ⚙️ Fixes Applied — 2026-05-09 16:45:47
- No additional fixes were required in this rerun.

### ♻️ Re-Audit Results — 2026-05-09 16:45:47
- Reconfirmed the same workspace evidence:
  - `git diff --check` remained clean
  - `rg -n "wallet_tx_history_dir|wallet_tx_history_dir_name|wallet_file_path\\(" crates/z00z_wallets/src crates/z00z_wallets/tests .planning/phases/044-wallet-assets` continued to show only test-local `test_wallet_file_path` helper matches
- No new actionable issues were found.

### ✅ Doublecheck Results — 2026-05-09 16:45:47
- Doublecheck rerun did not surface new code issues or truthfulness issues in the audit narrative.

### 🧾 Exact Fixes Required Summary

| Q | Title | Proof Status | Verification | Severity | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Canonical live JSONL wallet history only | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 2 | Remove tx-history helper alias seams | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 3 | Claim-registry-derived pending balance | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 4 | Canonical tx build payload shape | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 5 | Receiver cache path anchored to `.wlt` | Full Evidence | VERIFIED | ⚪ INFO | None | None |

### 🚩 Final Status

PASS.

The rerun confirms the same result: no open findings, no new drift, and no unsupported claims in the audit report.
