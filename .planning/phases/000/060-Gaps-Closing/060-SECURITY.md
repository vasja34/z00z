---
phase: 060
slug: gaps-closing
status: reopened
threats_open: 1
asvs_level: 1
created: 2026-06-23
register_authored_at_plan_time: true
---

# Phase 060 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## 🔒 Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Docs gate -> live scope | The docs gate is part of the security posture for Phase 060 because bootstrap authority, HJMT lineage, wallet object-family boundaries, and replay-sensitive claims must stay traceable instead of merely lint-clean. | `check-docs.sh`, `ZINV` anchors, Markdown corpus, closeout wording |
| Core authority -> downstream bootstrap consumers | `z00z_core::genesis` is the only canonical bootstrap authority and must stay distinct from compatibility or fixture YAML surfaces. | `GenesisConfig`, `assets_config.yaml`, rights config ownership, module docs |
| Runtime config -> HJMT process topology | HJMT YAML and checked manifests define the live topology contract and must not imply a different process model than the executed node and simulator paths. | `execution.shard_mapping`, shard ownership, process count, same-lineage rules |
| Storage publication truth -> validators/watchers/rollup preflight | Storage owns the live settlement-root and shard-route publication contract while validators, watchers, and preflight consume one shared truth path. | route digest, shard identity, publication checkpoints, incomplete vs reject semantics |
| Repo-owned security inputs -> generated verification reports | Supply-chain and adversarial evidence must come from one repository-owned source of truth rather than report-local snapshots or prose-only findings. | reviewed advisories, vet store, adversarial closure artifacts, generated reports |
| Wallet typed-object plane -> asset projection and object builders | Wallet typed-object operations must stay on one authority plane while cash projection remains cash-only and source-sensitive issue/create paths avoid forked APIs. | `wallet.object.*`, `wallet.asset.*`, object packages, refund/source bindings, delegation |
| Bench and profiling packet -> production defaults | HJMT measurement lanes and verification-pipeline timings may influence defaults only through comparable, durable, and resource-honest evidence. | throughput metrics, cache roots, CPU/RSS, restart/failover metrics, run roots |
| Phase packet -> status consumers | Summaries, `STATE.md`, `ROADMAP.md`, EVAL review, and this security ledger must tell one closeout story on one canonical phase folder. | status, plan completion, accepted risks, operator-owned follow-up |

---

## 🚨 Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Evidence | Status |
|-----------|----------|-----------|-------------|------------|----------|--------|
| T-060-01 | Tampering | Docs gate topology | mitigate | Keep strict docs mode topology-honest and reject fake mdBook scaffolding as a closure shortcut. | `060-01-SUMMARY.md`; `.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh` | closed |
| T-060-02 | Repudiation | Security-claim traceability | mitigate | Add real `ZINV` anchors so security claims stay auditable instead of only lint-clean. | `060-01-SUMMARY.md`; `docs/Z00Z-Litepaper.md`; `docs/tech-papers/Z00Z-Multi-DA-and-Checkpoint-Architecture.md` | closed |
| T-060-03 | Repudiation | Docs baseline comparability | mitigate | Stabilize the docs posture before supply-chain, adversarial, and verification-performance comparisons. | `060-01-SUMMARY.md`; `.planning/STATE.md`; `.planning/ROADMAP.md` | closed |
| T-060-04 | Tampering | Bootstrap authority wording | mitigate | Freeze one bootstrap story rooted in `z00z_core::genesis` and `GenesisConfig`. | `060-02-SUMMARY.md`; `crates/z00z_core/README.md`; `crates/z00z_core/src/genesis/README.md`; `crates/z00z_storage/src/settlement/README.md` | closed |
| T-060-05 | Elevation of Privilege | Config authority surface | mitigate | Reject symmetry-driven config sprawl and keep compatibility YAML from becoming a second authority plane. | `060-02-SUMMARY.md`; `060-04-SUMMARY.md`; `crates/z00z_core/src/assets/assets_config.yaml`; `crates/z00z_core/src/rights/config.rs` | closed |
| T-060-06 | Tampering | HJMT process-model contract | mitigate | Keep `aggregator_owned` explicit as the live default and avoid narrating `1 shard = 1 process` as already deployed behavior. | `060-03-SUMMARY.md`; `crates/z00z_rollup_node/src/config.rs`; `config/hjmt_runtime/sim_5a7s/manifest.json` | closed |
| T-060-07 | Repudiation | Shard-mapping comparability | mitigate | Fail closed on mixed or invalid mapping inputs so topology evidence remains comparable across runs. | `060-03-SUMMARY.md`; `crates/z00z_rollup_node/tests/test_hjmt_process.rs`; `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`; `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs` | closed |
| T-060-08 | Tampering | Rights config owner path | mitigate | Move rights config ownership under `rights/` and prevent `assets/` from silently reclaiming semantic ownership. | `060-04-SUMMARY.md`; `crates/z00z_core/src/rights/mod.rs`; `crates/z00z_core/src/rights/config.rs` | closed |
| T-060-09 | Tampering | Dual-authority YAML drift | mitigate | Close the mixed-YAML bootstrap drift instead of only rewriting docs around it. | `060-04-SUMMARY.md`; `crates/z00z_core/src/assets/assets_config_load.rs`; `crates/z00z_core/src/assets/assets_config.yaml` | closed |
| T-060-10 | Tampering | Decommission hygiene | mitigate | Remove stale owner, route, and standby state when aggregators are removed or reassigned. | `060-05-SUMMARY.md`; `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`; `crates/z00z_runtime/aggregators/tests/test_hjmt_split_brain_fencing.rs` | closed |
| T-060-11 | Repudiation | Lineage-safe topology scaling | mitigate | Keep explicit lineage assertions during `3A7S -> 2A7S -> 5A7S` transitions so green throughput cannot hide illegal ownership rewrites. | `060-05-SUMMARY.md`; `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs`; `crates/z00z_simulator/tests/test_scenario_settlement.rs` | closed |
| T-060-12 | Repudiation | Supply-chain authority source | mitigate | Make repository-owned advisory records canonical instead of relying on one generated report root. | `060-06-SUMMARY.md`; `.reviews/reviewed-advisories.toml`; `reports/z00z-verification-orchestrator-20260623-075715/supply-chain/supply-chain-summary.json`; `.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh` | closed |
| T-060-13 | Repudiation | Vet trust honesty | mitigate | Replace bootstrap-only cargo-vet signaling with a repository-owned store and explicit residual backlog. | `060-06-SUMMARY.md`; `.reviews/config.toml`; `.reviews/audits.toml`; `reports/z00z-verification-orchestrator-20260623-075715/supply-chain/supply-chain-summary.json` | open |
| T-060-14 | Repudiation | Wallet profile semantics | mitigate | Publish MVP profile names together with wallet projection rules so product semantics cannot fork by interpretation. | `060-07-SUMMARY.md`; `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`; `crates/z00z_wallets/src/adapters/rpc/methods/object.rs` | closed |
| T-060-15 | Elevation of Privilege | Cash-only wallet projection | mitigate | Keep `wallet.asset.*` cash-only so rights and vouchers cannot re-enter as value by projection alone. | `060-07-SUMMARY.md`; `crates/z00z_wallets/src/adapters/rpc/types/asset.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs` | closed |
| T-060-16 | Elevation of Privilege | Lock enforcement semantics | mitigate | Keep the `validator_mandate_lock_v1` profile protocol-visible and fail closed on ordinary spend instead of relying on UI-only lock semantics. | `060-08-SUMMARY.md`; `crates/z00z_wallets/src/tx/spend_rules.rs`; `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs` | closed |
| T-060-17 | Elevation of Privilege | Slashable-v1 overreach | mitigate | Keep v1 out of slashable logic until a dedicated proof model exists. | `060-08-SUMMARY.md`; `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`; `crates/z00z_simulator/tests/test_scenario1_object_flows.rs` | closed |
| T-060-18 | Repudiation | Adversarial finding ownership | mitigate | Convert high findings from prose into explicit closure artifacts with owners and exact closure modes. | `060-09-SUMMARY.md`; `.github/skills/z00z-verification-orchestrator/scripts/run-security-brainstorm.py` | closed |
| T-060-19 | Tampering | Adversarial count integrity | mitigate | Fix count drift by proving ownership and report consistency rather than editing reports cosmetically. | `060-09-SUMMARY.md`; `reports/z00z-verification-orchestrator-20260620-123133/security/adversarial-summary.json`; `reports/z00z-verification-orchestrator-20260620-123133/security/adversarial-review.md` | closed |
| T-060-20 | Repudiation | Measurement-lane separation | mitigate | Separate proof timing, worker-local throughput, and durable publication throughput so release claims stay truthful. | `060-10-SUMMARY.md`; `crates/z00z_storage/benches/settlement_benches.md`; `crates/z00z_storage/tests/test_bench_lanes.rs` | closed |
| T-060-21 | Repudiation | A/B fairness contract | mitigate | Run mapping A/B comparisons under same-hardware, same-cache, same-persistence, and same-route-generation conditions only. | `060-10-SUMMARY.md`; `crates/z00z_storage/scripts/run_storage_settlement_bench.py`; `crates/z00z_storage/outputs/settlement/hjmt_mapping_ab.md` | closed |
| T-060-22 | Tampering | Verification cache soundness | mitigate | Keep verification acceleration bounded by soundness checks and one canonical run-root instead of target/cache reuse by assumption. | `060-11-SUMMARY.md`; `reports/z00z-verification-orchestrator-20260622-072654/profiling/events.tsv` | closed |
| T-060-23 | Repudiation | Verification cost honesty | mitigate | Treat CPU, RSS, and failure-rate honesty as part of optimization closure instead of reporting wall-clock wins alone. | `060-11-SUMMARY.md`; `reports/z00z-verification-orchestrator-20260622-072654/profiling/events.tsv`; `060-z00z-verification-report.md` | closed |
| T-060-24 | Tampering | Live settlement-root authority | mitigate | Make generation-1 root-of-shard-roots the live storage-owned truth instead of a proof-only side surface. | `060-12-SUMMARY.md`; `crates/z00z_storage/src/settlement/test_live_recovery.rs` | closed |
| T-060-25 | Spoofing | Durable shard-route identity | mitigate | Bind recovery export and replay acceptance to shard route identity and lineage instead of bucket-centric ambiguity. | `060-12-SUMMARY.md`; `crates/z00z_storage/src/settlement/test_live_recovery.rs`; `crates/z00z_storage/tests/test_hjmt_import_export.rs` | closed |
| T-060-26 | Tampering | Exact shard-set acceptance | mitigate | Move exact shard coverage into one shared publication acceptance path rather than only node preflight. | `060-12-SUMMARY.md`; `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`; `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`; `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs` | closed |
| T-060-27 | Tampering | Publication truth authority | mitigate | Keep runtime glue subordinate to the storage-owned publication route checker after root-of-shard-roots activation. | `060-12-SUMMARY.md`; `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`; `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` | closed |
| T-060-28 | Tampering | Prepared tx negative balance coverage | mitigate | Add explicit negative tests on the canonical assembler path so malformed plaintext or commitment combinations still reject. | `060-13-SUMMARY.md`; `crates/z00z_wallets/src/tx/tx_assembler.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs` | closed |
| T-060-29 | Tampering | Voucher conservation rejection | mitigate | Prove conservation-mismatch rejection on the typed object package path rather than trusting implicit value symmetry. | `060-13-SUMMARY.md`; `crates/z00z_storage/src/settlement/test_model.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs` | closed |
| T-060-30 | Tampering | FeeEnvelope multi-surface truth | mitigate | Keep malformed `FeeEnvelope` rejection aligned across storage, wallet, and validator surfaces instead of treating storage-only checks as sufficient. | `060-13-SUMMARY.md`; `crates/z00z_storage/tests/test_fee_envelope.rs`; `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs` | closed |
| T-060-31 | Tampering | Native fee-lane drift | mitigate | Preserve the shipped split between native cash fee outputs and typed-object `FeeEnvelope` support semantics. | `060-13-SUMMARY.md`; `crates/z00z_storage/src/settlement/fee_envelope.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs` | closed |
| T-060-32 | Repudiation | Fee invariant confusion | mitigate | Keep `FeeEnvelope` structural support semantics distinct from arithmetic or Pedersen-balanced native-fee correctness. | `060-13-SUMMARY.md`; `crates/z00z_storage/src/settlement/fee_envelope.rs`; `crates/z00z_core/tests/genesis/test_claim_flow.rs` | closed |
| T-060-33 | Elevation of Privilege | Rights pseudo-cash drift | mitigate | Keep rights outside asset-like value checks so authority objects do not gain cash semantics through symmetric testing. | `060-13-SUMMARY.md`; `crates/z00z_storage/src/settlement/object_package_contract.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs` | closed |
| T-060-34 | Repudiation | Post-reopen closeout freshness | mitigate | Refresh Wave 4 attestation after later reopen slices instead of reusing stale `060-11` closeout language on a different tree. | `060-14-SUMMARY.md`; `060-15-SUMMARY.md`; `.planning/STATE.md`; `.planning/ROADMAP.md` | closed |
| T-060-35 | Elevation of Privilege | Refund/source binding (broad reopen) | mitigate | Keep refund and reject outputs bound to declared refund target and source context so vouchers cannot strip restrictions into clean value. | `060-14-SUMMARY.md`; `060-15-SUMMARY.md`; `crates/z00z_storage/src/settlement/test_model.rs`; `crates/z00z_storage/tests/test_store_api.rs` | closed |
| T-060-36 | Elevation of Privilege | One-plane issue/create API (broad reopen) | mitigate | Reuse the existing typed object-package RPC surface instead of adding a second issue/create API. | `060-14-SUMMARY.md`; `060-15-SUMMARY.md`; `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`; `crates/z00z_wallets/src/adapters/rpc/types/object.rs` | closed |
| T-060-37 | Denial of Service | Incomplete-state visibility (broad reopen) | mitigate | Surface retry-pending and publication-gap states as explicit incomplete states instead of collapsing them into accepted/rejected silence. | `060-14-SUMMARY.md`; `060-15-SUMMARY.md`; `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`; `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` | closed |
| T-060-38 | Tampering | Invalid-artifact fail-closed boundary | mitigate | Keep binding drift and invalid publication artifacts on hard-reject paths instead of misclassifying them as retriable incomplete states. | `060-14-SUMMARY.md`; `060-15-SUMMARY.md`; `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`; `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` | closed |
| T-060-39 | Elevation of Privilege | Refund/source exactness (narrowed MVP) | mitigate | Keep the narrowed MVP refund and restricted-source paths bound to truthful target/source context on the live storage and wallet seams. | `060-15-SUMMARY.md`; `crates/z00z_storage/src/settlement/test_model.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs`; `crates/z00z_storage/tests/test_store_api.rs` | closed |
| T-060-40 | Elevation of Privilege | One-plane issue/create API (narrowed MVP) | mitigate | Keep `wallet.object.preview_package` and `wallet.object.build_package` as the only typed-object construction RPCs while covering truthful voucher issue and right create paths. | `060-15-SUMMARY.md`; `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`; `crates/z00z_wallets/src/adapters/rpc/types/object.rs` | closed |
| T-060-41 | Denial of Service | Incomplete-state visibility (narrowed MVP) | mitigate | Expose real validator and watcher incomplete states on the existing publication contract instead of accepted/rejected/silent collapse. | `060-15-SUMMARY.md`; `crates/z00z_runtime/watchers/src/alerts.rs`; `crates/z00z_runtime/watchers/src/engine.rs`; `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`; `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs` | closed |
| T-060-42 | Elevation of Privilege | Delegation widening | mitigate | Keep `delegate_right` on the shipped transfer path and reject any widening that is falsely narrated as attenuation. | `060-15-SUMMARY.md`; `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`; `crates/z00z_storage/tests/test_right_leaf.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/object.rs` | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## ✅ Accepted Risks Log

No accepted risks are being used to hide the current supply-chain state.

The operator-owned full `z00z-verification-orchestrator` rerun and the semver
decision recorded in `060-11-SUMMARY.md` remain ordinary closeout follow-up,
but they are separate from the remaining live-tree cargo-vet maturity blocker
tracked as `T-060-13`.

---

## 🧾 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-23 | 42 | 41 | 1 | Codex `/gsd-secure-phase 060` + `/GSD-Audit-4` re-audit |

## 🧪 Verification Evidence

- All 15 numbered Phase 060 plan files contain a parseable `<threat_model>`
  block, so `register_authored_at_plan_time: true` is correct for this phase.
- No `## Threat Flags` sections were present in `060-01-SUMMARY.md` through
  `060-15-SUMMARY.md`; this audit therefore verified the plan-authored threat
  register against the executed summaries, live owner files, and cited test
  homes instead of generating a retroactive STRIDE register.
- The local GSD runtime query path was not usable in this workspace during the
  audit because `.github/gsd-core/bin/gsd-tools.cjs` failed on a missing
  `../../../package.json` dependency. The phase-local register and phase
  folder were still explicit and complete, so this security pass proceeded as a
  repository-owned manual verification pass over the plan-authored threats.
- `060-01-SUMMARY.md` through `060-10-SUMMARY.md` close the original workstream
  A/B/C/D threat surfaces directly, while `060-12-SUMMARY.md` through
  `060-15-SUMMARY.md` close the later audit-driven reopen packet without
  creating a second authority layer.
- `060-14-SUMMARY.md` is treated as review-context-only closure for the
  superseded overlap, and `060-15-SUMMARY.md` is treated as the actual
  implementation closeout for the overlapping MVP security subset.
- The implementation packet remains synchronized across the numbered summaries,
  but the current live tree is not fully security-closed because repo-owned
  advisory authority is now restored while repo-owned cargo-vet trust is still
  exemption-heavy, with `.reviews/audits.toml` empty and
  `cargo vet check --store-path .reviews` still reporting
  `Vetting Succeeded (776 exempted)`.

## 🚫 Blocking Findings

One blocking finding remains on the current live tree for the plan-authored
Phase 060 threat register:

- `T-060-13`: cargo-vet trust is now repository-owned by path, but the current
  live store is still explicit-exemption backlog rather than mature trust:
  `.reviews/audits.toml` is empty and direct repo vet still reports
  `Vetting Succeeded (776 exempted)`.

The following remain separate follow-up outside this register:

- operator-owned future full `z00z-verification-orchestrator` reruns by user
  instruction;
- operator-owned semver decision on the `z00z_crypto` API break recorded in
  `060-11-SUMMARY.md`.

---

## ✅ Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 1` confirmed
- [x] `status: reopened` set in frontmatter

**Approval:** reopened 2026-06-23
