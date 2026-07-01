---
phase: 059
slug: core-upgrade
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-18
register_authored_at_plan_time: true
---

# Phase 059 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## 🔒 Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Whitepaper authority -> live implementation | `059-TODO.md`, `059-CONTEXT.md`, and the cited corpus are live Phase 059 scope, but they must resolve to one repository owner path instead of speculative future wording. | canonical module homes, D-IDs, TODO micro-coverage, non-goals |
| Core descriptors -> downstream consumers | `z00z_core::{actions,policies,rights,vauchers}` define object semantics and compatibility facades must not fork a second policy truth. | descriptor bytes, policy hashes, action pools, unknown-policy rules |
| Genesis packet -> storage/runtime/simulator | One `z00z_core::genesis` boundary publishes typed assets, rights, policies, vouchers, and one settlement manifest. | config sections, derivation labels, bootstrap voucher backing, manifest digests |
| Storage settlement root -> runtime and wallets | Storage remains the only settlement-root authority while runtime and wallets consume typed object packages and proofs. | `SettlementLeaf`, `SettlementPath`, typed deltas, reject codes, proofs |
| Wallet inventory -> object RPC/package builder | Wallet inventory may project typed objects and build packages, but it must not make vouchers spendable cash or create wallet-only policy semantics. | `WalletOwnedObject`, quarantine state, validator-readable package fields, backup payloads |
| Runtime validators/watchers -> rollup/status | Validators and watchers expose object verdicts and alerts without becoming a second settlement authority or leaking wallet-local secrets. | `RuntimeObjectPackageV1`, `ObjectRejectCode`, alert evidence, rollup RPC/status rows |
| Simulator evidence -> closeout packet | `scenario_1` is the only executable Phase 059 simulator lane and its YAML, stages, emitted artifacts, and canonical `pending_exact_home` public anchors must stay synchronized. | `object_flow_matrix`, `voucher_flow.json`, stage artifacts, Alice/Bob/Charlie traces |
| Final phase packet -> roadmap/state consumers | Security, summaries, UAT, evidence ledger, `STATE.md`, and `ROADMAP.md` must tell one closeout story. | final verdict, validation commands, phase status, accepted risks |

---

## 🚨 Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Evidence | Status |
|-----------|----------|-----------|-------------|------------|----------|--------|
| T-059-01 | Repudiation | Source-audit authority | mitigate | Freeze the live-vs-target contract before code changes and require every later slice to route through one canonical owner path. | `059-SOURCE-AUDIT.md`; `059-01-SUMMARY.md`; `059-EVIDENCE-LEDGER.md` | closed |
| T-059-02 | Elevation of Privilege | Rights lifecycle/value boundary | mitigate | Keep rights zero-value and close delegation, lifecycle, fee-boundary, and value-rejection gaps through later storage/runtime/wallet coverage instead of treating old rights support as complete. | `059-02-SUMMARY.md`; `059-05-SUMMARY.md`; `crates/z00z_core/src/rights/test_rights_config.rs`; `crates/z00z_storage/src/settlement/tx_plan_types.rs` | closed |
| T-059-03 | Tampering | Voucher authority boundary | mitigate | Keep vouchers as first-class settlement objects and typed wallet inventory rows instead of asset metadata or wallet-only payload truth. | `059-04-SUMMARY.md`; `059-07-SUMMARY.md`; `crates/z00z_storage/src/settlement/record.rs`; `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs` | closed |
| T-059-04 | Elevation of Privilege | Native cash policy profile | mitigate | Keep native cash on a fixed narrow policy profile and reject arbitrary voucher-style programmability on assets. | `059-02-SUMMARY.md`; `crates/z00z_core/src/assets/policy_flags.rs`; `crates/z00z_core/src/assets/test_policy_descriptor.rs` | closed |
| T-059-05 | Elevation of Privilege | Right config validation | mitigate | Reject value-like keys on rights so rights cannot masquerade as vouchers or fee budgets. | `059-02-SUMMARY.md`; `crates/z00z_core/src/rights/test_rights_config.rs`; `crates/z00z_storage/src/settlement/object_package_contract.rs` | closed |
| T-059-06 | Tampering | Policy descriptor hashing | mitigate | Make action pools and policy descriptors canonical, content-addressed, and shared across core, storage, wallets, runtime, and simulator. | `059-02-SUMMARY.md`; `crates/z00z_core/src/policies/policy_descriptor.rs`; `crates/z00z_core/src/assets/test_policy_descriptor.rs` | closed |
| T-059-07 | Denial of Service | Genesis config compatibility | mitigate | Add typed config sections additively so old fixtures remain valid while Phase 059 fixtures gain explicit policy and voucher sections. | `059-03-SUMMARY.md`; `crates/z00z_core/src/genesis/genesis_config.rs`; `crates/z00z_core/src/genesis/genesis_config_validate.rs`; `crates/z00z_core/src/genesis/test_genesis_suite.rs` | closed |
| T-059-08 | Spoofing | Genesis derivation domains | mitigate | Domain-separate voucher derivation from right derivation so object identity and audit lineage cannot collide. | `059-EVIDENCE-LEDGER.md`; `crates/z00z_core/src/genesis/genesis_derivation.rs`; `crates/z00z_core/src/genesis/genesis_policies.rs`; `crates/z00z_core/src/genesis/genesis_vouchers.rs` | closed |
| T-059-09 | Tampering | Genesis voucher backing | mitigate | Allow bootstrap vouchers only as explicit backed exceptions tied to manifest-backed reserve evidence, not free conditional claims. | `059-03-SUMMARY.md`; `crates/z00z_core/src/genesis/genesis_vouchers.rs`; `crates/z00z_core/src/genesis/genesis_settlement_manifest.rs`; `crates/z00z_core/src/genesis/test_genesis_suite.rs` | closed |
| T-059-10 | Tampering | Genesis descriptor ownership | mitigate | Reuse canonical core descriptor types under `z00z_core::genesis` and verify Stage 1 artifacts against config-derived expectations. | `059-03-SUMMARY.md`; `crates/z00z_core/src/genesis/mod.rs`; `crates/z00z_core/src/genesis/genesis_policies.rs`; `crates/z00z_core/src/genesis/genesis_vouchers.rs`; `crates/z00z_simulator/src/scenario_1/runner_verify.rs` | closed |
| T-059-11 | Tampering | Settlement leaf family tags | mitigate | Extend the existing settlement leaf-family model in place with `VoucherLeaf` and dedicated family tags across record, proof, batch, and cache paths. | `059-04-SUMMARY.md`; `crates/z00z_storage/src/settlement/record.rs`; `crates/z00z_storage/src/settlement/leaf.rs`; `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs` | closed |
| T-059-12 | Tampering | Nonexistence-proof family binding | mitigate | Keep nonexistence and batch-proof checks family-aware so one object family cannot prove absence for another. | `059-04-SUMMARY.md`; `crates/z00z_storage/src/settlement/proof.rs`; `crates/z00z_storage/src/settlement/proof_batch.rs`; `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs` | closed |
| T-059-13 | Tampering | One-settlement-root contract | mitigate | Extend the existing root vocabulary in place and block any parallel voucher tree or second settlement authority. | `059-04-SUMMARY.md`; `crates/z00z_storage/src/settlement/root_types.md`; `059-EVIDENCE-LEDGER.md` | closed |
| T-059-14 | Elevation of Privilege | Voucher cash boundary | mitigate | Reject voucher-as-cash paths in the shared object package contract, wallet RPC guards, and simulator negative matrix. | `059-05-SUMMARY.md`; `crates/z00z_storage/src/settlement/object_package_contract.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`; `crates/z00z_simulator/tests/test_scenario1_object_flows.rs` | closed |
| T-059-15 | Elevation of Privilege | Right value boundary | mitigate | Reject right-as-value flows at storage contract, validator verdict, wallet guard, and simulator negative coverage. | `059-05-SUMMARY.md`; `crates/z00z_storage/src/settlement/object_package_contract.rs`; `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`; `crates/z00z_simulator/tests/test_scenario1_object_flows.rs` | closed |
| T-059-16 | Tampering | Fee-support separation | mitigate | Keep `FeeEnvelope` separate from vouchers and rights and surface fee-boundary violations as explicit reject codes. | `059-05-SUMMARY.md`; `crates/z00z_storage/src/settlement/fee_envelope.rs`; `crates/z00z_storage/tests/test_fee_envelope.rs`; `crates/z00z_storage/src/settlement/object_package_contract.rs` | closed |
| T-059-17 | Information Disclosure | Storage/wallet secret boundary | mitigate | Persist only settlement-owned hashes, roots, proofs, and typed deltas while keeping wallet-local secrets outside storage authority. | `059-05-SUMMARY.md`; `crates/z00z_storage/src/settlement/object_package_contract.rs`; `059-EVIDENCE-LEDGER.md` | closed |
| T-059-18 | Elevation of Privilege | Runtime semantic authority | mitigate | Keep aggregators as typed evidence carriers and leave semantic acceptance authority to validators and storage-backed contract checks. | `059-06-SUMMARY.md`; `crates/z00z_runtime/aggregators/src/types.rs`; `crates/z00z_runtime/aggregators/src/batch_planner.rs`; `crates/z00z_runtime/validators/README.md` | closed |
| T-059-19 | Repudiation | Validator verdict precision | mitigate | Emit specific object reject classes for unknown policy, wrong-family proof, value misuse, double redemption, and fee-boundary failures instead of generic invalid verdicts. | `059-06-SUMMARY.md`; `crates/z00z_runtime/validators/src/verdict.rs`; `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs` | closed |
| T-059-20 | Repudiation | Watcher alert visibility | mitigate | Emit watcher alerts for critical object reject families so consensus-critical failures cannot stay silent. | `059-06-SUMMARY.md`; `crates/z00z_runtime/watchers/src/engine.rs`; `crates/z00z_runtime/watchers/tests/test_object_alerts.rs`; `crates/z00z_runtime/watchers/README.md` | closed |
| T-059-21 | Elevation of Privilege | Wallet payload typing | mitigate | Introduce explicit asset, voucher, and right wallet payloads plus one typed inventory facade instead of reusing `OwnedAssetPayload` for every object class. | `059-07-SUMMARY.md`; `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs`; `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs`; `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md` | closed |
| T-059-22 | Tampering | Wallet quarantine durability | mitigate | Keep unknown-policy vouchers and rights in durable quarantine across restart, rotation, and index rebuild flows. | `059-07-SUMMARY.md`; `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs`; `crates/z00z_wallets/src/db/redb_wallet_store/test_mod.rs`; `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md` | closed |
| T-059-23 | Tampering | Wallet migration compatibility | mitigate | Preserve legacy asset payload versioning and additive migration instead of rewriting old asset rows into a new storage truth. | `059-07-SUMMARY.md`; `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs`; `crates/z00z_wallets/src/db/redb_wallet_store/test_mod.rs`; `059-EVIDENCE-LEDGER.md` | closed |
| T-059-24 | Elevation of Privilege | Wallet RPC namespace | mitigate | Keep `wallet.asset.*` cash-only and route voucher/right behavior through typed `wallet.object.*` methods. | `059-08-SUMMARY.md`; `crates/z00z_wallets/src/adapters/rpc/methods/object.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs` | closed |
| T-059-25 | Tampering | Shared object package builder | mitigate | Build wallet object packages on the shared `RuntimeObjectPackageV1` and validator-readable descriptor contract rather than wallet-local semantics. | `059-08-SUMMARY.md`; `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`; `crates/z00z_storage/src/settlement/object_package_contract.rs`; `059-EVIDENCE-LEDGER.md` | closed |
| T-059-26 | Tampering | Backup/import safety | mitigate | Preserve descriptor refs, quarantine reasons, wallet identity, and tamper detection in the additive backup/import packet. | `059-08-SUMMARY.md`; `crates/z00z_wallets/src/services/test_wallet_service.rs`; `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs` | closed |
| T-059-27 | Repudiation | Simulator integration depth | mitigate | Keep simulator evidence end-to-end across create, persist, validate, store, scan, and report flows instead of object creation only. | `059-09-SUMMARY.md`; `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`; `crates/z00z_simulator/tests/test_scenario1_object_flows.rs`; `crates/z00z_simulator/tests/test_scenario_settlement.rs` | closed |
| T-059-28 | Repudiation | Negative interaction coverage | mitigate | Record positive and negative object interactions, including replay, wrong-family, value-misuse, and double-redeem failures. | `059-09-SUMMARY.md`; `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`; `crates/z00z_simulator/tests/test_scenario1_object_flows.rs`; `crates/z00z_simulator/README.md` | closed |
| T-059-29 | Tampering | YAML/stage/report sync | mitigate | Keep one canonical simulator config/design pair and fail-closed stage/report validation for emitted object artifacts. | `059-09-SUMMARY.md`; `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`; `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`; `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`; `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs` | closed |
| T-059-30 | Repudiation | Final test matrix completeness | mitigate | Tie closeout to the Phase 059 test spec, tasks packet, and evidence ledger so no object family or interaction class can be omitted silently. | `059-10-SUMMARY.md`; `059-TEST-SPEC.md`; `059-TESTS-TASKS.md`; `059-EVIDENCE-LEDGER.md` | closed |
| T-059-31 | Repudiation | Docs and evidence honesty | mitigate | Keep TODO and context coverage mapped to live owner paths, explicit deferrals, and synchronized phase-closeout docs rather than overstated future-only claims. | `059-SUMMARY.md`; `059-10-SUMMARY.md`; `059-EVIDENCE-LEDGER.md`; `.planning/STATE.md`; `.planning/ROADMAP.md` | closed |
| T-059-32 | Repudiation | Release-only closeout validation | mitigate | Keep final closeout gated by release-mode bootstrap, targeted crate reruns, full workspace tests, rustdoc, and `full_verify.sh`, plus repeated review passes. | `059-10-SUMMARY.md`; `059-SUMMARY.md`; `reports/full_verify-report-long-running-tests.txt` | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## ✅ Accepted Risks Log

No accepted risks.

---

## 🧾 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-18 | 32 | 32 | 0 | Codex `/gsd-secure-phase 059` |
| 2026-06-18 | 32 | 32 | 0 | Codex `/gsd-secure-phase 059` rerun |

## 🧪 Verification Evidence

- Workspace-first review of `059-01-PLAN.md` through `059-10-PLAN.md`
  confirmed a plan-authored 32-item threat register, so this audit remained in
  verify-mitigations mode and did not invent a retroactive STRIDE register.
- No `## Threat Flags` sections were present in `059-01-SUMMARY.md` through
  `059-10-SUMMARY.md`; the audit therefore used the plan threat models, the
  numbered summaries, `059-EVIDENCE-LEDGER.md`, `059-SUMMARY.md`, and live
  code or test anchors as the security evidence base.
- Workspace-first review of `059-01-SUMMARY.md` through `059-10-SUMMARY.md`,
  `059-EVIDENCE-LEDGER.md`, and the cited implementation files confirmed that
  every plan-time threat closes on the current executed tree without a second
  semantic authority or a parallel object layer.
- The final Phase 059 validation packet already records green release-mode
  evidence for `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`,
  targeted `cargo test -p ... --release` reruns, full `cargo test --release`,
  `cargo doc --release --no-deps`, and
  `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh`.
- This security pass rechecked live workspace anchors for
  `RuntimeObjectPackageV1`, `WalletOwnedObject`, `object_flow_matrix`, and
  `ObjectRejectCode::{UnknownPolicy,WrongFamilyProof,VoucherUsedAsCash,RightUsedAsValue,DoubleRedeem,FeeBoundary}`
  to confirm the closeout packet still points at real repository homes.
- The repeated state-A audit confirmed no drift after SECURITY creation:
  all ten numbered plan files still carry `<threat_model>` blocks, no
  `## Threat Flags` sections appeared in the numbered summaries, and the
  existing 32-row Phase 059 threat register remained synchronized with the
  live tree.

## 🚫 Blocking Findings

No blocking findings remain on the current live tree.

---

## ✅ Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-18
