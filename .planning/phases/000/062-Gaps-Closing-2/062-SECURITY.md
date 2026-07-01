---
phase: 062
slug: gaps-closing-2
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-27
register_source: deduplicated-from-plan-time-threat-models
register_authored_at_plan_time: true
---

# Phase 062 — Security

> Per-phase security contract: threat register, accepted risks, and audit
> trail.

This security audit was executed in manual fallback mode because the local
`.github/gsd-core/bin/gsd-tools.cjs` shim currently fails on
`../../../package.json` resolution and the workflow-referenced
`.github/agents/gsd-security-auditor.md` file is not present on the current
tree. The phase itself is fully authored and executed, so the threat register
below is derived from the live `062-*.md` plan packet and verified against the
current implementation and release tests.

Every `062-*.md` plan contains the same three threat classes:
`authority drift`, `placeholder drift`, and `evidence drift`. This file
deduplicates those repeated plan-time threats at the phase level and verifies
them against the live code paths that Phase 062 owns.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Phase packet -> implementation | `062-TODO.md`, `062-CONTEXT.md`, `062-01-PLAN.md` through `062-27-PLAN.md`, and summaries must not become a second truth plane over current code. | Planning authority, artifact paths, closure claims |
| Storage root/backend -> live state | Storage-root naming, backend env gating, checkpoint/publication persistence, and HJMT records must remain bound to one semantic storage authority. | Settlement root names, backend mode, checkpoint/proof state |
| Wallet RPC/thin/local sim -> wallet truth | Wallet policy, tx history, thin/thick transport, chain client, broadcast retry, fee source, and remote worker must stay advisory or canonical exactly where Phase 062 says they are. | Wallet tx history, confirmations, fee rates, remote scan hints, thin snapshots |
| Local simulator/adapter -> authoritative verification | Local simulation and adapter layers must not bypass canonical verification or create transport-only success claims. | Claim/checkpoint artifacts, worker hints, topology/runtime outputs |
| Docs/tests/benchmarks -> runtime closure | Bench docs, metrics, and summaries must not claim runtime closure without executable negative tests or release verification. | Throughput figures, threat closures, residual-gap claims |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-062-01 | authority-drift | Phase 062 canonical authority surfaces across storage, wallet, thin-mode, and simulator seams | mitigate | The phase closes on one existing Phase 062 folder and one current-tree planning packet only. Stale `@proposed` and dead-path drift were removed from the live `062-*.md` plans. Canonical live seams are present on current code paths including `crates/z00z_storage/src/backend/mod.rs`, `crates/z00z_storage/src/settlement/hjmt_config.rs`, `crates/z00z_wallets/src/wallet/policy.rs`, `crates/z00z_wallets/src/chain/chain_client_impl.rs`, `crates/z00z_wallets/src/chain/broadcast_impl.rs`, `crates/z00z_wallets/src/chain/scan_engine_impl.rs`, `crates/z00z_wallets/src/tx/fee_estimator.rs`, and `crates/z00z_simulator/src/scenario_1/runner.rs`. `rg` on the Phase 062 plan packet confirms the stale proposed-target markers are gone. | closed |
| T-062-02 | placeholder-drift | Implementation and docs claimed by Phase 062 | mitigate | Every plan carries anti-placeholder and evidence gates, and the live closeout removed stale contract/TODO wording from touched wallet seams instead of leaving docs-only closure markers. Executable coverage exists on the owned runtime seams through `crates/z00z_wallets/src/rpc/test_tx_send_suite.rs`, `crates/z00z_wallets/tests/test_wallet_policy.rs`, `crates/z00z_wallets/tests/test_remote_scan_worker.rs`, `crates/z00z_wallets/tests/test_chain_client_sim.rs`, `crates/z00z_wallets/tests/test_chain_broadcast_retry.rs`, `crates/z00z_wallets/tests/test_fee_rate_source.rs`, `crates/z00z_wallets/tests/test_thin_cache.rs`, `crates/z00z_wallets/tests/test_thin_equivalence.rs`, `crates/z00z_wallets/tests/test_thin_fallback.rs`, `crates/z00z_wallets/tests/test_thin_index.rs`, `crates/z00z_wallets/tests/test_thin_modes.rs`, `crates/z00z_wallets/tests/test_thin_privacy.rs`, `crates/z00z_wallets/tests/test_thin_support.rs`, `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`, and `crates/z00z_storage/tests/test_live_guardrails.rs`. Scoped `rg` on the touched wallet contract files confirms the stale live TODO contract strings are gone. | closed |
| T-062-03 | evidence-drift | Local simulation, negative tests, and release verification for Phase 062 closures | mitigate | The mandatory `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` gate is green on the current tree. Focused release verification is green for `test_wallet_policy`, `test_remote_scan_worker`, `test_chain_client_sim`, and the remaining targeted Phase 062 security-sensitive suites recorded in the audit trail below. The Phase 062 closeout also already proved a green sequential `cargo test --release -q` on the same implementation state before this SECURITY artifact was added. Simulator-backed evidence remains executable on current paths such as `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`, `crates/z00z_simulator/tests/scenario_1/test_claim_snapshot.rs`, and `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`. The prior full-suite flake on `clear_foreign_live_lock` was removed by serializing the stateful `runner.rs` unit tests, keeping simulator evidence reproducible instead of summary-only. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-27 | 3 | 3 | 0 | codex manual secure-phase fallback |

Summary threat flags: none of the `062-*.md` summaries contain a `## Threat Flags`
section, so no previously-open threat exceptions were inherited into this
audit.

Audit evidence on the current tree:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_wallets test_tx_policy_ -- --nocapture`
- `cargo test --release -p z00z_wallets --test test_wallet_policy`
- `cargo test --release -p z00z_wallets --test test_remote_scan_worker`
- `cargo test --release -p z00z_wallets --test test_chain_client_sim`
- `cargo test --release -p z00z_wallets --test test_chain_broadcast_retry`
- `cargo test --release -p z00z_wallets --test test_fee_rate_source`
- `cargo test --release -p z00z_storage --test test_live_guardrails`
- `cargo test --release -p z00z_simulator --lib clear_foreign_live_lock --features test-params-fast --features wallet_debug_tools -- --nocapture`
- `rg -n "@proposed|proposed target|proposed artifact|proposed target after codebase-fit review" .planning/phases/062-Gaps-Closing-2/062-*-PLAN.md`
- `rg -n "# TODO Implementation Requirements|/// # TODO|// TODO\\(threat\\)" crates/z00z_wallets/src/tx/prover.rs crates/z00z_wallets/src/persistence/scan_storage.rs crates/z00z_wallets/src/persistence/wallet_metadata_storage.rs crates/z00z_wallets/src/persistence/receipt_storage.rs crates/z00z_wallets/src/backup/backup_importer.rs crates/z00z_wallets/src/backup/backup_exporter.rs crates/z00z_wallets/tests/test_spend_statement.rs`

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-27
