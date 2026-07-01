---
phase: 064
slug: gaps-closing-3
status: verified
threats_open: 0
asvs_level: 1
register_origin: retroactive-stride
created: 2026-06-30
---

# Phase 064 — Security

> Per-phase security contract: retroactive STRIDE register, accepted risks, and audit trail for the implemented Phase 064 surface.

## 🧭 Audit Basis

- Phase 064 did not carry a plan-authored `<threat_model>` block inside the `064-*-PLAN.md` files, so this register was built in retroactive-STRIDE mode from:
  - `.planning/phases/064-Gaps-Closing-3/064-CONTEXT.md`
  - `.planning/phases/064-Gaps-Closing-3/064-TODO.md`
  - implemented code, tests, docs, and executable audit scripts
- Security enforcement was verified against the repo-local GSD surface, and the audit used real release-mode tests plus executable boundary scripts instead of docs-only or grep-only closure.

## 🔐 Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| `crates/z00z_simulator/src/scenario_1/**` | Canonical local simulator evidence surface for Phase 064. | checkpoint ids, publication rows, emitted packet inventories, secret-bearing artifacts |
| `crates/z00z_wallets/src/chain/**`, `src/rpc/**`, `src/services/**`, `src/redb_store/**` | Canonical wallet-local mutation, RPC, session, restore, and typed-object durability surface. | session tokens, tx ids, `.wlt` / tx-history state, object packages, quarantine state |
| `crates/z00z_storage/src/**`, `crates/z00z_runtime/aggregators/**`, `crates/z00z_rollup_node/**` | Canonical checkpoint, snapshot, theorem, local DA, recovery, and publication-binding authority surface. | checkpoint artifacts, snapshot ids, semantic roots, theorem bundles, route digests |
| `crates/z00z_crypto/**` and `crates/z00z_crypto/tari/**` | Canonical workspace crypto facade with isolated vendor subtree. | crypto APIs, vendor passthrough risk, direct import drift |
| `docs/**`, `wiki/**`, `scripts/audit_*.sh`, `.github/workflows/boundary-guards.yml` | Canonical live-claim and boundary-guard surface for deferred network/DA/fraud wording and anti-drift CI. | wording authority, boundary audit results, local docs links |

## 🛡️ Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Evidence | Status |
|-----------|----------|-----------|-------------|------------|----------|--------|
| T-064-01 | Tampering | simulator publication truth | mitigate | Default `scenario_1` publication must emit truthful final checkpoint evidence, and canonical filtered lanes must not close through `step_stub` fallback events. | `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs:161`; `crates/z00z_simulator/src/scenario_1/stage_12/finalize_flow.rs:56`; `crates/z00z_simulator/tests/scenario_1/test_scenario1_filtered_runs.rs:48,75`; verified by `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_filtered_runs -- --nocapture` | closed |
| T-064-02 | Information Disclosure | simulator default release packet | mitigate | Default public packet must not leak plaintext seed phrases, receiver secrets, or lock-byte artifacts outside explicit debug-only lanes. | `crates/z00z_simulator/tests/scenario_1/test_stage2_secret_artifacts.rs:153`; verified by `cargo test --release -p z00z_simulator --test scenario_1 test_stage2_secret_artifacts -- --nocapture` | closed |
| T-064-03 | Tampering | wallet mutation and RPC registration | mitigate | `wallet.asset.*` mutations must use live tx packages instead of `stub_tx_*`; dispatcher audit must see include-based routes; `app.wallet.open_wallet_source` and `wallet.object.*` must stay live and registered. | `crates/z00z_wallets/src/rpc/app_dispatcher_wiring.rs:104`; `crates/z00z_wallets/src/rpc/wallet_dispatcher_routes.rs:303`; `crates/z00z_wallets/tests/test_asset_rpc_mutations.rs:172,188`; `crates/z00z_wallets/tests/test_rpc_route_coverage.rs:67-70`; `crates/z00z_wallets/tests/test_object_rpc_packages.rs:10-43`; verified by the corresponding release-mode test group | closed |
| T-064-04 | Spoofing / Elevation of Privilege | sensitive RPC session gates and wasm/native claims | mitigate | Sensitive wallet RPCs must route through `verify_session(...)` or `verify_session_no_touch(...)`; browser-facing surfaces must not claim the native session / `.wlt` guarantee set. | `crates/z00z_wallets/tests/test_sensitive_rpc_session.rs:86-140`; `crates/z00z_wallets/tests/test_wallet_capability_matrix.rs:30-31`; `wiki/04-wallet-and-rpc/wallet-session-locks.md:118`; verified by `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session --test test_wallet_capability_matrix -- --nocapture` | closed |
| T-064-05 | Tampering / Repudiation | wallet restore and typed-object durability | mitigate | Restore must roll back atomically across history, `.wlt`, and publish failure points; quarantined objects must survive restore/export/import and promote only explicitly; reject-code mapping must remain stable. | `crates/z00z_wallets/src/services/wallet_actions_backup.rs:757,852,871,913`; `crates/z00z_wallets/tests/test_wallet_restore_atomic.rs`; `crates/z00z_wallets/tests/test_object_quarantine.rs:209`; `crates/z00z_storage/tests/test_object_reject_codes.rs:1`; verified by the corresponding release-mode tests | closed |
| T-064-06 | Tampering / Elevation of Privilege | raw stealth-output builder misuse | mitigate | Raw `build_tx_stealth_output(...)` must not act as production approval authority; validated request/card paths must remain the canonical approval seam. | `wiki/04-wallet-and-rpc/receiver-request-flow.md:73,138`; `crates/z00z_wallets/src/rpc/asset_rpc_impl.rs:71`; `crates/z00z_wallets/tests/test_s5_misuse_gate.rs`; verified by `cargo test --release -p z00z_wallets --test test_s5_misuse_gate -- --nocapture` | closed |
| T-064-07 | Tampering | checkpoint seal path and semantic-root boundaries | mitigate | `seal_artifact()` must remain the canonical statement-bound checkpoint path; raw save stays explicitly noncanonical; backend roots and storage internals must not become public semantic truth. | `crates/z00z_storage/src/checkpoint/store.rs:212,227,329`; `crates/z00z_storage/tests/test_checkpoint_store.rs:95-115`; `crates/z00z_storage/tests/test_settlement_proof_boundaries.rs`; verified by `cargo test --release -p z00z_storage --test test_checkpoint_store --test test_prep_snapshot --test test_settlement_proof_boundaries -- --nocapture` | closed |
| T-064-08 | Tampering | theorem, local DA, recovery, and publication-binding authority | mitigate | Theorem verification must reject detached or mismatched public artifacts; local DA must reject forged source labels and digests; `PublicationBinding` must keep one runtime-owned construction path and one route-acceptance authority. | `crates/z00z_runtime/aggregators/src/service.rs:46-47`; `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs:35-107`; `crates/z00z_rollup_node/tests/test_da_local_sim.rs`; `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`; `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`; verified by the corresponding release-mode tests | closed |
| T-064-09 | Spoofing / Repudiation | deferred-boundary wording and cross-crate boundary drift | mitigate | OnionNet / remote chain / real DA / slashing / fraud-engine surfaces must stay honestly deferred, and executable audits must block `z00z_utils`, `z00z_crypto`, `z00z_extensions`, and local-doc-link drift. | `crates/z00z_wallets/tests/test_live_boundary_claims.rs:16-67`; `scripts/audit_z00z_utils_boundary.sh`; `scripts/audit_crypto_facade.sh`; `scripts/audit_extensions_boundary.sh`; `scripts/audit_local_docs_links.sh`; all four scripts passed on 2026-06-30 | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

## ✅ Accepted Risks Log

No accepted risks.

## 🧾 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-30 | 9 | 9 | 0 | Codex |

## ✅ Verification Commands

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_simulator --test scenario_1 test_stage2_secret_artifacts -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_filtered_runs -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_asset_rpc_mutations --test test_rpc_route_coverage --test test_object_rpc_packages --test test_wallet_restore_atomic --test test_sensitive_rpc_session --test test_object_quarantine --test test_wallet_capability_matrix --test test_live_boundary_claims -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_s5_misuse_gate -- --nocapture`
- `cargo test --release -p z00z_storage --test test_object_reject_codes --test test_checkpoint_store --test test_prep_snapshot --test test_settlement_proof_boundaries -- --nocapture`
- `cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard --test test_da_local_sim -- --nocapture`
- `cargo test --release -p z00z_aggregators --test test_recovery_failover --test test_publication_binding -- --nocapture`
- `bash scripts/audit_z00z_utils_boundary.sh`
- `bash scripts/audit_crypto_facade.sh`
- `bash scripts/audit_extensions_boundary.sh`
- `bash scripts/audit_local_docs_links.sh`

## 🔏 Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-30
