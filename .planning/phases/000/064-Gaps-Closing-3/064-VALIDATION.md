---
phase: 064
slug: gaps-closing-3
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-30
---

<!-- markdownlint-disable MD013 MD033 -->

# Phase 064 — Validation Strategy

> Reconstructed Nyquist validation contract for the completed `064-Gaps-Closing-3` phase.

## ✅ Validation Basis

- State B reconstruction was used: all `064-0*-SUMMARY.md` files existed, while `064-VALIDATION.md` did not.
- The audit read `064-01..05-PLAN.md`, `064-01..05-SUMMARY.md`, `064-TEST-SPEC.md`, `064-TESTS-TASKS.md`, and `064-SECURITY.md`.
- Coverage result: all `28/28` Phase 064 `REC-064-*` requirements map to concrete automated release-mode commands or executable audit scripts.
- Gap result: `0 missing`, `0 partial`, `0 manual-only`.
- `gsd-nyquist-auditor` was not spawned because the reconstruction found no uncovered requirement to fill.
- Current-tree note: the phase summaries record that broad `cargo test --release` reruns still reproduce pre-existing `z00z_core` genesis/config blockers outside the Phase 064-owned slices; all requirement-level commands below are green for the implemented phase surface.

## 🔧 Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test --release` integration tests plus executable shell/Python boundary audits |
| **Config file** | Workspace `Cargo.toml` manifests and standalone audit scripts; no extra phase-local harness file was required |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release` |
| **Estimated runtime** | ~10-20 minutes for the full Phase 064 validation packet |

## ⏰ Sampling Rate

- After every task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` and the requirement-specific release-mode command from the map below.
- After every plan wave: rerun the grouped wave-local commands recorded in the corresponding `064-0N-SUMMARY.md`.
- Before `/gsd-verify-work`: rerun `cargo test --release` and confirm that any remaining failures are still the known out-of-scope `z00z_core` blockers, not Phase 064 regressions.
- Max feedback latency: <= 20 minutes for the whole phase packet on the current tree.

## 📌 Coverage Summary

| Metric | Count |
|--------|-------|
| Requirements audited | 28 |
| Automated green requirements | 28 |
| Partial requirements | 0 |
| Missing requirements | 0 |
| Manual-only requirements | 0 |
| Plans covered | 5 |

## ✅ Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `REC-064-P0-01` | `PLAN-064-G01` | 1 | Default `scenario_1` publication is final by default | `T-064-01` | Canonical finalize flow must emit truthful final checkpoint evidence instead of draft/incomplete defaults | integration | `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface -- --nocapture`<br>`cargo test --release -p z00z_simulator --test scenario_1 test_scenario_settlement -- --nocapture` | ✅ | ✅ green |
| `REC-064-P0-02` | `PLAN-064-G01` | 1 | Remove `step_stub` fallback from canonical stages 9-12 | `T-064-01` | Filtered runs must fail closed if canonical stages still self-heal through synthetic fallback events | integration | `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_filtered_runs -- --nocapture` | ✅ | ✅ green |
| `REC-064-P0-03` | `PLAN-064-G01` | 1 | Emit exact-home packet files for object flows | `T-064-01` | Public packet anchors must move from pending placeholders to truthful emitted homes | integration | `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_object_flows -- --nocapture` | ✅ | ✅ green |
| `REC-064-P0-08` | `PLAN-064-G01` | 1 | Reject plaintext leakage from the default release packet | `T-064-02` | Default release packet must stay secret-clean for seed phrases, receiver secrets, and lock-byte artifacts | integration | `cargo test --release -p z00z_simulator --test scenario_1 test_stage2_secret_artifacts -- --nocapture` | ✅ | ✅ green |
| `REC-064-P2-07` | `PLAN-064-G01` | 1 | Keep simulator harness imports on stable public facades | `T-064-09` | Simulator-owned harness entrypoints must stay on owner facades instead of opening deep-import drift | integration | `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_filtered_runs -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-01` | `PLAN-064-G02` | 2 | Move `wallet.asset.*` mutations to the live wallet-local path | `T-064-03` | Asset mutations must produce real local tx packages and durable tx lifecycle state instead of `stub_tx_*` responses | integration | `cargo test --release -p z00z_wallets --test test_asset_rpc_mutations -- --nocapture`<br>`cargo test --release -p z00z_wallets --test test_chain_client_sim -- --nocapture`<br>`cargo test --release -p z00z_wallets --test test_chain_broadcast_retry -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-02` | `PLAN-064-G02` | 2 | Keep `wallet.object.*` as the live post-genesis typed-object path | `T-064-03` | Object package docs and route surface must reject stale stub wording and keep the live namespace explicit | integration | `cargo test --release -p z00z_wallets --test test_object_rpc_packages -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-03` | `PLAN-064-G02` | 2 | Repair RPC audit truth and wire `open_wallet_source` publicly | `T-064-03` | Include-based route registration counting and app dispatcher wiring must stay truthful and complete | script + integration | `python3 crates/z00z_wallets/scripts/audit_rpc_method_wiring.py`<br>`cargo test --release -p z00z_wallets --test test_rpc_route_coverage -- --nocapture` | ✅ | ✅ green |
| `REC-064-P2-01` | `PLAN-064-G03` | 2 | Collapse placeholder services to honest live owners or explicit non-live seams | `T-064-09` | Placeholder-only wallet service names must not drift into a fake live ownership story | executable scan | `rg -n "BackupService|KeyService|NetworkService|StorageService|WalletService|non-live|placeholder" crates/z00z_wallets/src/services wiki/04-wallet-and-rpc/wallet-stub-surface.md` | ✅ | ✅ green |
| `REC-064-P0-04` | `PLAN-064-G03` | 2 | Add restore fault injection around `.wlt` and history commit | `T-064-05` | Restore must roll back atomically across staged history, `.wlt`, and publish failure points | integration | `cargo test --release -p z00z_wallets --test test_wallet_restore_atomic -- --nocapture` | ✅ | ✅ green |
| `REC-064-P0-05` | `PLAN-064-G03` | 2 | Prove all sensitive RPCs pass through session verification | `T-064-04` | Sensitive wallet RPC surfaces must route through `verify_session(...)` or `verify_session_no_touch(...)` | integration | `cargo test --release -p z00z_wallets --test test_sensitive_rpc_session -- --nocapture` | ✅ | ✅ green |
| `REC-064-P0-06` | `PLAN-064-G03` | 2 | Ban raw stealth-output builders from production app/RPC flows | `T-064-06` | Raw builder usage must stay out of approval-authority production paths | integration | `cargo test --release -p z00z_wallets --test test_payment_request -- --nocapture` | ✅ | ✅ green |
| `REC-064-P0-07` | `PLAN-064-G03` | 2 | Keep native-only TOFU/inbox guarantees out of wasm/browser claims | `T-064-04` | Browser-facing docs and code must stay explicit about native-only session and `.wlt` guarantees | integration + scan | `cargo test --release -p z00z_wallets --test test_wallet_capability_matrix -- --nocapture`<br>`rg -n "browser builds do not get this live session model|native-only today|Rejects wasm32 and routes native load through spawn_blocking|\\.wlt persistence is not supported on wasm32|\\.wlt owned-asset loading is not supported on wasm32" wiki/04-wallet-and-rpc crates/z00z_wallets/src/services` | ✅ | ✅ green |
| `REC-064-P1-04` | `PLAN-064-G03` | 2 | Preserve quarantine roundtrip and explicit promotion behavior | `T-064-05` | Quarantined objects must survive backup, restore, export, import, and promote only explicitly | integration | `cargo test --release -p z00z_wallets --test test_object_quarantine -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-05` | `PLAN-064-G03` | 2 | Prove stable `ObjectRejectCode` exhaustiveness | `T-064-05` | Validator-facing reject taxonomy and RPC-facing mapping must stay aligned on stable codes | integration | `cargo test --release -p z00z_storage --test test_object_reject_codes -- --nocapture` | ✅ | ✅ green |
| `REC-064-P2-02` | `PLAN-064-G04` | 3 | Expand local DA/runtime simulation without claiming a real DA network | `T-064-08` | Local DA and runtime simulation must reject forged labels, digests, and failover misuse without pretending live network transport | integration | `cargo test --release -p z00z_rollup_node --test test_da_local_sim -- --nocapture`<br>`cargo test --release -p z00z_aggregators --test test_recovery_failover -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-06` | `PLAN-064-G04` | 3 | Guard canonical seal usage on the checkpoint truth path | `T-064-07` | `seal_artifact()` must remain the canonical statement-bound checkpoint path and raw save must stay noncanonical | integration | `cargo test --release -p z00z_storage --test test_checkpoint_store -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-07` | `PLAN-064-G04` | 3 | Cover every explicit `PrepSnapshot` adversarial lane | `T-064-07` | Duplicate ids, family drift, root mix, witness decode, and terminal/path mismatch lanes must fail independently | integration | `cargo test --release -p z00z_storage --test test_prep_snapshot -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-08` | `PLAN-064-G04` | 3 | Preserve settlement proof boundary separation | `T-064-07` | Semantic settlement roots must not collapse into backend proof state or raw storage internals | integration | `cargo test --release -p z00z_storage --test test_settlement_proof_boundaries -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-09` | `PLAN-064-G04` | 3 | Add theorem-boundary negative tests for detached or mismatched inputs | `T-064-08` | Theorem verification must reject detached statements, wrong proof payloads, wrong ids, and broken link roots | integration | `cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-10` | `PLAN-064-G04` | 3 | Cover every explicit recovery failover reject branch | `T-064-08` | Recovery and failover branches must fail closed on stale lineage, split-brain, standby-down, and stale-root cases | integration | `cargo test --release -p z00z_aggregators --test test_recovery_failover -- --nocapture` | ✅ | ✅ green |
| `REC-064-P1-11` | `PLAN-064-G04` | 3 | Extend `PublicationBinding` anti-fork guardrails | `T-064-08` | Publication binding must stay the single route-acceptance and anti-fork digest authority | integration | `cargo test --release -p z00z_aggregators --test test_publication_binding -- --nocapture` | ✅ | ✅ green |
| `REC-064-P2-03` | `PLAN-064-G05` | 4 | Keep core/genesis cleanup truthful and subordinate to earlier closure work | `T-064-09` | Core/genesis wording must not reopen simulator or wallet truth gaps as hidden prerequisites | integration | `cargo test --release -p z00z_core --test test_live_guardrails -- --nocapture` | ✅ | ✅ green |
| `REC-064-P2-04` | `PLAN-064-G05` | 4 | Preserve honest deferred boundaries for OnionNet, remote chain, DA, slashing, and fraud-engine claims | `T-064-09` | Deferred network and fraud boundaries must stay explicitly non-live across docs and wallet anchors | integration | `cargo test --release -p z00z_wallets --test test_live_boundary_claims -- --nocapture` | ✅ | ✅ green |
| `REC-064-P2-05` | `PLAN-064-G05` | 4 | Add CI guardrails for `z00z_utils` boundary discipline | `T-064-09` | Infrastructure-only boundaries must fail closed on business-logic drift | executable script | `bash scripts/audit_z00z_utils_boundary.sh` | ✅ | ✅ green |
| `REC-064-P2-06` | `PLAN-064-G05` | 4 | Enforce the `z00z_crypto` facade | `T-064-09` | Workspace crates must use `z00z_crypto` rather than direct vendor crypto imports | executable script | `bash scripts/audit_crypto_facade.sh` | ✅ | ✅ green |
| `REC-064-P2-08` | `PLAN-064-G05` | 4 | Prevent `z00z_extensions` from absorbing core semantics | `T-064-09` | Extension boundaries must reject semantic dumping-ground drift without an explicit extension plan | executable script | `bash scripts/audit_extensions_boundary.sh` | ✅ | ✅ green |
| `REC-064-P2-09` | `PLAN-064-G05` | 4 | Replace internal GitHub links with local-path refs | `T-064-09` | Wiki and planning citations must stay local-path and offline-safe | executable script | `bash scripts/audit_local_docs_links.sh` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## 📦 Wave 0 Requirements

Existing infrastructure covers all phase requirements.

- No additional framework install was required.
- No missing test file or missing runner gap remained after reconstruction.
- `wave_0_complete: true` is set because no uncovered Phase 064 requirement required follow-up scaffolding.

## 🧪 Manual-Only Verifications

All phase behaviors have automated verification.

## ✅ Validation Sign-Off

- [x] All tasks have automated verification commands or executable audit scripts
- [x] Sampling continuity preserved across all five Phase 064 plans
- [x] Wave 0 gap count is zero
- [x] No watch-mode flags are used
- [x] Phase-local evidence is release-mode or executable-script based
- [x] `nyquist_compliant: true` is set in frontmatter

**Approval:** verified 2026-06-30
