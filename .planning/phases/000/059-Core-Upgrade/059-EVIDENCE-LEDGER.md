---
phase: 059-Core-Upgrade
artifact: evidence-ledger
status: complete
updated: 2026-06-18
authority:
  - 059-TODO.md
  - 059-CONTEXT.md
  - 059-SOURCE-AUDIT.md
  - 059-TEST-SPEC.md
  - 059-TESTS-TASKS.md
  - 059-01-SUMMARY.md
  - 059-02-SUMMARY.md
  - 059-03-SUMMARY.md
  - 059-04-SUMMARY.md
  - 059-05-SUMMARY.md
  - 059-06-SUMMARY.md
  - 059-07-SUMMARY.md
  - 059-08-SUMMARY.md
  - 059-09-SUMMARY.md
  - 059-10-SUMMARY.md
  - 059-SUMMARY.md
  - docs/Z00Z-Main-Whitepaper.md
  - docs/Z00Z-Smart-Cash-Whitepaper.md
  - docs/Z00Z-UseCases-Whitepaper.md
  - docs/Z00Z-Uniqueness-Whitepaper.md
  - docs/tech-papers/done/Z00Z-HJMT-Design.md
---

<!-- markdownlint-disable MD013 MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD060 -->

# Phase 059 Evidence Ledger

## 🎯 Purpose

This ledger is the canonical Phase 059 closeout map. It proves that every
`059-TODO.md` section, every `059-CONTEXT.md` D-ID, every TODO bullet/table
coverage group, and every referenced corpus constraint resolves to one live
repository path:

- one canonical semantic owner path;
- one summary-backed implementation slice;
- one executable test or simulator anchor when relevant;
- one live doc or explicit deferral when code is intentionally not delivered.

No row in this file may imply a second semantic owner, a second settlement
tree, a wallet-only policy truth, or a simulator-only authority path.

## 🔒 Canonical Rules

- `059-TODO.md` and the referenced corpus are live Phase 059 authority, not
  optional future notes.
- Assets, Vouchers, Rights, and `FeeEnvelope` stay distinct roles: final value,
  conditional value, authority, and processing support.
- `z00z_core::{actions,policies,rights,vauchers}` are canonical vocabulary
  roots. `assets/*` compatibility facades do not become a second semantic home.
- `z00z_storage::settlement` keeps one `SettlementStateRoot`, one
  `SettlementPath`, and one `SettlementLeaf` family surface extended in place.
- Wallet cash projection remains asset-only. Vouchers and Rights never inflate
  spendable balance.
- Aggregators may carry object evidence, but validators and storage remain the
  semantic acceptance authorities.
- `scenario_1` remains the single executable simulator home for Phase 059.

## 🧭 Summary-Backed Delivery Chain

| Plan | Scope closed | Primary live anchors |
|---|---|---|
| `059-01` | source audit, live-vs-target freeze, no-parallel-layer rule | `059-SOURCE-AUDIT.md`, `059-TEST-SPEC.md`, `059-TESTS-TASKS.md`, `059-01-SUMMARY.md` |
| `059-02` | canonical core vocabulary, action/policy/voucher/right descriptors | `crates/z00z_core/src/{actions,policies,rights,vauchers}/`, `crates/z00z_core/src/assets/test_*`, `059-02-SUMMARY.md` |
| `059-03` | additive genesis policies/vouchers, manifest publication | `crates/z00z_core/src/genesis/`, `crates/z00z_core/tests/genesis/test_*`, `059-03-SUMMARY.md` |
| `059-04` | `VoucherLeaf` family, proof and nonexistence support | `crates/z00z_storage/src/settlement/{record,proof,proof_batch,hjmt_cache}.rs`, `059-04-SUMMARY.md` |
| `059-05` | typed deltas, conservation, lifecycle, fee boundary | `crates/z00z_storage/src/settlement/{tx_plan_types,tx_plan_help,store,hjmt_commit}.rs`, `059-05-SUMMARY.md` |
| `059-06` | runtime object packages, validator verdicts, watcher alerts | `crates/z00z_runtime/{aggregators,validators,watchers}/`, `crates/z00z_rollup_node/src/{status,rpc}.rs`, `059-06-SUMMARY.md` |
| `059-07` | wallet typed inventory and persistence | `crates/z00z_wallets/src/db/redb_wallet_store/{owned_objects,tables}.rs`, `059-07-SUMMARY.md` |
| `059-08` | wallet object RPC, package builder, backup/import | `crates/z00z_wallets/src/adapters/rpc/methods/{object,object_impl}.rs`, `059-08-SUMMARY.md` |
| `059-09` | simulator object matrix and Alice/Bob/Charlie evidence | `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`, `crates/z00z_simulator/tests/test_scenario1_object_flows.rs`, `059-09-SUMMARY.md` |
| `059-10` | closeout docs, evidence, UAT, final release verification | `059-EVIDENCE-LEDGER.md`, `059-UAT.md`, `059-10-SUMMARY.md`, `059-SUMMARY.md` |

## 📌 D-ID Coverage Ledger

### Scope And Source Of Truth

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-01` | `059-TODO.md` stays canonical. | `059-01-SUMMARY.md`; `059-10-PLAN.md` `<todo_trace>`; this ledger. |
| `D-02` | Asset/Voucher/Right are sibling settlement objects. | `059-02-SUMMARY.md`, `059-04-SUMMARY.md`, `crates/z00z_storage/src/settlement/root_types.md`. |
| `D-03` | Core, storage, wallets, simulator, runtime, docs, and tests are in scope. | `059-02` through `059-10`; `059-TESTS-TASKS.md` task routing. |
| `D-04` | Every TODO paragraph maps to code, tests, docs, deferral, or non-goal. | `059-CONTEXT.md` micro map; this ledger micro-coverage and deferral sections. |

### Object Semantics

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-05` | Asset is final spendable cash only. | `crates/z00z_wallets/README.md`; `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`; wallet asset-only reject tests from `059-08-SUMMARY.md`. |
| `D-06` | Voucher is conditional value, not final cash. | `crates/z00z_core/src/vauchers/`; `crates/z00z_storage/src/settlement/record.rs`; `059-03`, `059-05`, `059-08`, `059-09`. |
| `D-07` | Right is authority without value. | `crates/z00z_core/src/rights/`; `crates/z00z_storage/src/settlement/root_types.md`; `test_rights_config`; `059-02`, `059-05`. |
| `D-08` | Native cash policy stays fixed and narrow. | `crates/z00z_core/src/policies/policy_descriptor.rs`; `crates/z00z_core/src/actions/action_pool.rs`; `test_policy_descriptor`; `059-02-SUMMARY.md`. |
| `D-09` | Distinguish live code from target semantics. | `059-SOURCE-AUDIT.md`; `059-01-SUMMARY.md`; explicit deferrals in this ledger. |

### Genesis Strategy

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-10` | Keep one `z00z_core::genesis` orchestration boundary. | `crates/z00z_core/src/genesis/mod.rs`; `crates/z00z_core/src/genesis/README.md`; `059-03-SUMMARY.md`. |
| `D-11` | Use per-object generators inside that boundary. | `genesis_rights.rs`, `genesis_policies.rs`, `genesis_vouchers.rs`, existing asset generation in `genesis.rs`. |
| `D-12` | Assets keep finite-supply genesis semantics. | `crates/z00z_core/src/genesis/genesis_config*.yaml`; `genesis_settlement_manifest.rs`; `crates/z00z_core/src/genesis/README.md`. |
| `D-13` | Rights use zero-value authority-instance genesis. | `crates/z00z_core/src/genesis/genesis_rights.rs`; `crates/z00z_core/tests/genesis/test_genesis_rights.rs`. |
| `D-14` | Genesis vouchers are conditional-claim bootstrap exceptions with backing. | `crates/z00z_core/src/genesis/genesis_vouchers.rs`; `crates/z00z_core/tests/genesis/test_genesis_vouchers.rs`. |
| `D-15` | Policy/action descriptors are deterministic and content-addressed. | `crates/z00z_core/src/{actions,policies}/`; `test_policy_descriptor`; `test_genesis_policies.rs`; `059-02`, `059-03`. |

### Storage And Settlement

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-16` | Storage settlement stays canonical authority. | `crates/z00z_storage/src/settlement/README.md`; `059-04-SUMMARY.md`; `059-05-SUMMARY.md`. |
| `D-17` | Extend leaf-family model in place with voucher support. | `SettlementLeaf::Voucher(VoucherLeaf)` in `record.rs`; proof/caching in `proof.rs`, `proof_batch.rs`, `hjmt_cache.rs`; `test_hjmt_live_proof_families.rs`. |
| `D-18` | Split committed state, wallet payload, and witnesses correctly. | `crates/z00z_storage/src/settlement/object_package_contract.rs`; wallet owned-object payloads; runtime object package path. |
| `D-19` | Typed remove/create/update delta is canonical execution shape. | `crates/z00z_storage/src/settlement/tx_plan_types.rs`; `test_store_api.rs`; `059-05-SUMMARY.md`. |
| `D-20` | `FeeEnvelope` stays separate processing support. | `crates/z00z_storage/src/settlement/fee_envelope.rs`; `test_fee_envelope.rs`; validator `FeeBoundary` reject classes. |

### Wallet Model

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-21` | Move from asset-only persistence to typed object inventory. | `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs`; `059-07-SUMMARY.md`. |
| `D-22` | Generalize receive/reserve/quarantine/RPC behind one inventory facade. | `ObjectInventoryStore`; `wallet.object.*` surfaces; `test_object_inventory_*`; `059-07`, `059-08`. |
| `D-23` | Unknown policies quarantine objects and keep them non-spendable. | `OwnedVoucherPayload`/`OwnedRightPayload` quarantine rules in `tables.rs`; `test_object_inventory_rejects_bad_checksum_and_unquarantined_unknown_policy`; wallet docs. |
| `D-24` | Wallet exposes cash, voucher, and right projections. | `crates/z00z_wallets/README.md`; `WALLET-GUIDE.md`; `wallet.object.list_*` surfaces. |
| `D-25` | Wallet package builders bind action, rights, descriptors, proofs, and witnesses. | `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`; shared `RuntimeObjectPackageV1`; `059-08-SUMMARY.md`. |

### Simulator Model

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-26` | Simulator proves all object classes and combined interactions. | `object_flow_matrix` in `scenario_config.yaml`; `test_scenario1_object_flows.rs`; `059-09-SUMMARY.md`. |
| `D-27` | Include minimum voucher/right/fee and negative paths. | `object_flow_matrix` positive and negative rows; `test_scenario1_object_flows_reject_codes`; `059-09-SUMMARY.md`. |
| `D-28` | Include right-gated voucher actions and right-failure paths. | `scenario_config.yaml` combined flows; `test_scenario1_object_flows_wallet_inventory_for_alice_bob_charlie`; runtime reject codes. |
| `D-29` | Adapt existing `scenario_1` rather than replace it. | `crates/z00z_simulator/README.md`; `059-09-SUMMARY.md`; no new scenario root introduced. |

### Runtime, Validators, Watchers

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-30` | Validators verify presence, policy, rights, signatures, lifecycle, conservation, and fee separation. | `crates/z00z_runtime/validators/src/verdict.rs`; `tests/test_object_policy_verdicts.rs`; validator README. |
| `D-31` | Aggregators do not become a second semantic authority. | `059-06-SUMMARY.md`; `crates/z00z_runtime/aggregators/src/{types,batch_planner}.rs`; runtime docs. |
| `D-32` | Watchers detect invalid object actions and alert families. | `crates/z00z_runtime/watchers/src/{engine,alerts,evidence_export}.rs`; `tests/test_object_alerts.rs`; watcher README. |

### Tests And Verification

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-33` | Tests expand by object class and interaction class. | `059-TEST-SPEC.md`; `059-TESTS-TASKS.md`; simulator matrix; per-crate Phase 059 tests. |
| `D-34` | Core tests cover parsing, deterministic genesis, hashes, impossible config rejection. | `test_policy_descriptor`, `test_voucher_config`, `test_rights_config`, `test_genesis_*`, `059-02`, `059-03`. |
| `D-35` | Storage tests cover codec, proof families, deltas, conservation, recovery, failures. | `test_settlement_leaf.rs`, `test_hjmt_live_proof_families.rs`, `test_store_api.rs`, `test_fee_envelope.rs`, `test_live_guardrails.rs`. |
| `D-36` | Wallet tests cover persistence, migration, scan, quarantine, projection, RPC, backup. | `redb_wallet_store/test_mod.rs`, `test_wallet_service`, backup and export tests, `test_wallet_export_pack_boundary.rs`. |
| `D-37` | Simulator tests include Alice/Bob/Charlie full paths and release E2E evidence. | `test_scenario1_object_flows.rs`; `test_scenario_settlement.rs`; simulator artifact contract tests. |

### Second-Pass Source-Audit Expansion

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-38` | Source audit precedes numbered implementation plans. | `059-SOURCE-AUDIT.md`; `059-01-SUMMARY.md`. |
| `D-39` | Audit separates `live`, `target`, and `migration concern`. | `059-SOURCE-AUDIT.md`; `059-PLAN-REVIEW.md`. |
| `D-40` | Existing rights are reusable but incomplete and must be audited. | `059-SOURCE-AUDIT.md`; `059-02`/`059-03`/`059-05` summaries; live right tests. |
| `D-41` | Voucher semantics are explicit, not asset metadata. | canonical `z00z_core::vauchers`, `VoucherLeaf`, wallet owned-object surfaces; `059-02` through `059-08`. |
| `D-42` | Existing config compatibility must be preserved while widening genesis. | `genesis_config.rs`, `genesis_config_validate.rs`, `test_genesis_*`, `059-03-SUMMARY.md`. |
| `D-43` | Wallet must add new object support without reusing `OwnedAsset` for all classes. | `OwnedVoucherPayload`, `OwnedRightPayload`, `WalletOwnedObject`, `059-07-SUMMARY.md`. |

### Object Shape Requirements

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-44` | Voucher leaf carries committed conditional-claim state. | `crates/z00z_storage/src/settlement/record.rs` `VoucherLeaf`; storage docs; voucher tests. |
| `D-45` | Voucher value fields support conservation and residual accounting. | `tx_plan_types.rs`; `test_store_api_voucher_issue_accept_partial_full_and_delta`; `059-05-SUMMARY.md`. |
| `D-46` | Holder and beneficiary stay distinct; transfer must not silently change refund authority. | voucher config and storage record fields; wallet object RPC transfer lane; simulator transfer cases in `object_flow_matrix`. |
| `D-47` | Right leaf stays zero-value and rejects value-like semantics. | `test_rights_config`; `RightLeaf` storage docs; validator and wallet reject classes. |
| `D-48` | Asset/Voucher/Right/FeeEnvelope cannot substitute for one another. | `ObjectFamily`, `ObjectRejectCode::{VoucherUsedAsCash,RightUsedAsValue,FeeBoundary}`; wallet and runtime tests. |

### Policy And Action Model

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-49` | Policies are content-addressed descriptors with stable hashes. | `crates/z00z_core/src/policies/policy_descriptor.rs`; `test_policy_descriptor`; genesis policy tests. |
| `D-50` | Native cash stays special while voucher/right policies use deterministic action pools. | `fixed_cash_action_pool_descriptor`; `native_cash_policy_descriptor`; `test_genesis_policies_reject_asset_side_custom_entries`. |
| `D-51` | MVP conditions stay deterministic or verifier-safe attested only. | `ConditionTrustTierV1`; policy descriptor contracts; explicit non-goals in this ledger. |
| `D-52` | Policy descriptors declare family, rights, signature, lifecycle, replay, and conservation contracts. | `PolicyDescriptorV1`; `RuntimeObjectPackageV1`; validator reject flow. |
| `D-53` | Unknown policies are fail-closed in validators and quarantined in wallets; simulator includes unknown-policy cases. | wallet quarantine rules; `ObjectRejectCode::UnknownPolicy`; simulator reject-code matrix. |

### Genesis And Publication Details

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-54` | Genesis exports object-class artifacts plus one settlement manifest. | `genesis_rights.json`, `genesis_policies.json`, `genesis_vouchers.json`, `genesis_settlement_manifest.json`; genesis README. |
| `D-55` | Deterministic derivation is domain-separated and voucher labels do not reuse right labels. | `genesis_derivation.rs`; `genesis_policies.rs`; `genesis_vouchers.rs`; genesis tests and docs. |
| `D-56` | Asset/right/voucher/policy configs stay typed sections under one genesis boundary. | `GenesisConfig` additive sections; schema yaml; genesis README. |
| `D-57` | Genesis vouchers are bootstrap exceptions, runtime issuance remains live path. | genesis README non-goals and operator notes; `059-03-SUMMARY.md`; runtime package path. |
| `D-58` | Genesis and simulator fixtures include positive and negative typed object fixtures. | `genesis_config_devnet*_phase059.yaml`; `object_flow_matrix`; `test_scenario1_object_flows_stage1_policy_voucher_artifacts`. |

### Storage And Proof Details

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-59` | Voucher support touches all leaf-family boundaries. | `record.rs`, `leaf.rs`, `proof.rs`, `proof_batch.rs`, `hjmt_cache.rs`, `test_live_recovery.rs`, `test_fuzz_seeds.rs`, storage docs. |
| `D-60` | `SettlementPath` stays one shape across object classes. | `crates/z00z_storage/src/settlement/root_types.md`; proof and list APIs; storage README. |
| `D-61` | Cross-family proof misuse rejects fail-closed. | `test_hjmt_live_proof_families.rs`; `test_batch_proof_support.rs`; validator `WrongFamilyProof`. |
| `D-62` | Typed deltas list deletes, creates, residuals, fee, action, policy hash, and roots. | `tx_plan_types.rs`; object package contract; `test_store_api.rs`; `059-05-SUMMARY.md`. |
| `D-63` | Storage does not interpret wallet secrets. | storage README boundaries; `object_package_contract.rs`; `059-05` and `059-06` summaries. |

### Wallet Details

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-64` | One inventory facade with three projections; only assets spendable. | `WalletOwnedObject`; wallet README; `WALLET-GUIDE.md`; object inventory tests. |
| `D-65` | Preserve existing owned-asset rows and additive migration path. | `OwnedAssetPayload::VERSION = 1`; `PAYLOAD_VERSION_OWNED_ASSET = 1`; `059-07-SUMMARY.md`; wallet DB tests. |
| `D-66` | Wallet scan classifies by object family before object-specific recovery. | `wallet_service_store_support.rs`; object inventory and scan paths; wallet docs. |
| `D-67` | Wallet package builder rejects voucher-as-cash, right-as-value, unknown policy, and invalid right state. | `object_impl.rs`; asset RPC reject tests from `059-08`; simulator negative flows. |
| `D-68` | Wallet RPC uses typed object namespace and preserves cash-only asset methods. | `crates/z00z_wallets/src/adapters/rpc/methods/object.rs`; dispatcher registration; asset-only reject tests. |

### Simulator Details

| D-ID | Obligation | Live evidence |
|---|---|---|
| `D-69` | `scenario_1` remains canonical and synchronized. | simulator README; `scenario_design.yaml`; `scenario_config.yaml`; `test_scenario1_stage_surface.rs`. |
| `D-70` | Stages 1/4/5/6/11/13 carry typed object semantics. | `test_scenario1_object_flows_stage1_policy_voucher_artifacts`; `object_flow_matrix`; Stage 13 proof evidence and negative checks. |
| `D-71` | Alice/Bob/Charlie paths must prove full lifecycle with persistence, verdict, root, and watcher evidence. | `test_scenario1_object_flows_wallet_inventory_for_alice_bob_charlie`; simulator artifact lists; `059-09-SUMMARY.md`. |
| `D-72` | Simulator artifacts report failures and proposed fixes, not only successes. | negative rows in `object_flow_matrix`; `test_scenario1_object_flows_reject_codes`; simulator README failure list. |

## 🗺️ `059-TODO.md` Micro Coverage Closeout

| TODO section | D-ID closure | Primary evidence |
|---|---|---|
| Key Terms Used In This Paper | `D-05` through `D-20`, `D-44` through `D-53` | core vocabulary, storage docs, wallet docs, runtime docs, this ledger. |
| 1. Why This Paper Exists | `D-01` through `D-04` | source audit, plan review, evidence ledger. |
| 1.1 Design Problem | `D-02`, `D-03`, `D-16`, `D-21`, `D-26`, `D-31` | core/storage/wallet/simulator/runtime slices. |
| 1.2 Design Thesis | `D-05` through `D-08`, `D-16`, `D-21`, `D-26`, `D-30` | all numbered implementation slices. |
| 1.3 Reader Outcome | `D-01`, `D-04` | `059-CONTEXT.md`; `059-SUMMARY.md`; this ledger. |
| 2. Position In The Z00Z Corpus | `D-01`, `D-03`, `D-38`, `D-39` | `059-SOURCE-AUDIT.md`; corpus map below. |
| 2.1 Corpus Role | `D-38`, `D-39` | plan review corpus traceability; source audit. |
| 2.2 Current Maturity Versus Target Architecture | `D-09`, `D-38`, `D-39`, deferral map | source audit plus explicit non-goals. |
| 3. Core Thesis: Asset, Voucher, And Right | `D-02`, `D-05` through `D-08`, `D-44` through `D-48` | core/storage/wallet/runtime/simulator slices. |
| 3.1 The Minimal Triad | `D-02`, `D-05`, `D-06`, `D-07` | canonical module roots and wallet projections. |
| 3.2 Why This Split Is Minimal | `D-02`, `D-08`, `D-19`, `D-48` | typed delta checks and role-boundary rejects. |
| 3.3 Why Voucher Is Not Redundant With Right | `D-06`, `D-07`, `D-45`, `D-47` | voucher/right separate leaves, separate wallet projections, separate rejects. |
| 3.4 Cross-Object Binding Rules | `D-18`, `D-19`, `D-25`, `D-30`, `D-52` | object package contract, wallet builder, validator verdicts. |
| 4. Asset: Final Value And Cash Boundary | `D-05`, `D-08`, `D-24`, `D-64`, `D-67`, `D-68` | wallet asset-only cash and reject guards. |
| 4.1 What Asset Means | `D-05`, `D-21`, `D-24` | wallet docs and projections. |
| 4.2 Why Asset Must Stay Clean | `D-08`, `D-50` | fixed-cash policy descriptors and tests. |
| 4.3 Cash-Grade Invariants | `D-12`, `D-19`, `D-30`, `D-35` | genesis asset semantics, storage conservation, validator checks. |
| 4.4 What This Paper Does Not Claim About Assets | `D-08`, deferral map | no arbitrary user action pools on native cash. |
| 5. Voucher: Conditional Value, Not Dirty Cash | `D-06`, `D-14`, `D-17`, `D-44` through `D-46` | voucher core/storage/wallet/simulator slices. |
| 5.1 Economic Meaning | `D-06`, `D-24`, `D-64` | wallet voucher projection and non-cash wording. |
| 5.2 Fully Backed Vouchers | `D-14`, `D-45`, `D-57` | genesis voucher tests and typed delta conservation. |
| 5.3 Voucher Is Not Final Cash | `D-06`, `D-23`, `D-24`, `D-67` | durable quarantine and voucher-as-cash rejects. |
| 5.4 Voucher Lifecycle | `D-14`, `D-19`, `D-45`, `D-52`, `D-70`, `D-71` | storage lifecycle transitions, wallet object RPC, simulator matrix. |
| 5.5 Partial Redeem | `D-45`, `D-67`, `D-70` | store API partial redeem and wallet `RedeemPartial` mapping. |
| 5.6 Why Vouchers Are Better Than Encumbered Cash | `D-06`, `D-08`, `D-48` | typed role boundary instead of dirty cash. |
| 6. Right: Authority Without Value | `D-07`, `D-13`, `D-47`, `D-48` | right core/storage docs and tests. |
| 6.1 What Right Means | `D-07`, `D-13`, `D-52` | rights config and genesis rights tests. |
| 6.2 Stateless And Stateful Rights | `D-13`, `D-30`, `D-70` | rights config, validator checks, simulator matrix. |
| 6.3 Rights And Delegation | `D-07`, `D-25`, `D-28`, `D-67`, `D-71` | wallet object RPC right actions and simulator flows. |
| 6.4 Why Right Does Not Duplicate Voucher | `D-07`, `D-47`, `D-48` | zero-value right rules and role-boundary rejects. |
| 7. Policy, ActionPool, And Condition Model | `D-15`, `D-49` through `D-53` | core canonical descriptor modules, validator/wallet behavior. |
| 7.1 Fixed CashPolicy For Native Asset | `D-08`, `D-50` | native cash descriptors and tests. |
| 7.2 VoucherPolicy And ActionPool | `D-15`, `D-49`, `D-52` | policy and action-pool descriptors, genesis policy export. |
| 7.3 Core-Safe Condition Classes | `D-51`, deferral map | deterministic/verifier-safe-only MVP contract. |
| 7.4 Validator And Wallet Responsibilities | `D-23`, `D-25`, `D-30`, `D-32`, `D-53` | wallet quarantine and validator/watcher surfaces. |
| 7.5 Minimum Policy Contract Surface | `D-15`, `D-49`, `D-52` | descriptor types and runtime package contract. |
| 7.6 Minimum Action Semantics | `D-19`, `D-25`, `D-30`, `D-52` | typed deltas, builder, validator checks, simulator actions. |
| 7.7 Package And Witness Boundary | `D-18`, `D-25`, `D-52`, `D-63` | object package contract and wallet object RPC builder. |
| 7.8 Separate Fee-Support Boundary | `D-20`, `D-48` | `FeeEnvelope` docs, storage tests, validator/watcher fee-boundary rejects. |
| 8. Payment, Acceptance, And Receiver Safety | `D-05`, `D-06`, `D-23`, `D-24`, `D-67`, `D-72` | cash/voucher split in wallet docs and simulator failures. |
| 8.1 Clean Payment Versus Voucher Transfer | `D-05`, `D-06`, `D-24`, `D-68` | asset RPC vs object RPC namespace split. |
| 8.2 One-Sided Cash Stays | `D-05`, `D-68` | asset APIs stay cash-only. |
| 8.3 Refund Is Not Arbitrary Clawback | `D-45`, `D-46`, `D-52`, `D-71` | voucher refund lifecycle and beneficiary separation. |
| 8.4 Unknown Policy And Wallet Quarantine | `D-23`, `D-53` | wallet quarantine storage and simulator unknown-policy negative rows. |
| 9. Storage And Settlement Architecture | `D-16` through `D-20`, `D-59` through `D-63` | storage docs, proof tests, typed deltas, runtime package contract. |
| 9.1 One Settlement-Root Contract And Semantic Object View | `D-16`, `D-17`, `D-60` | settlement README and root-types note. |
| 9.2 Live HJMT Leaves And The Voucher Target | `D-17`, `D-59`, `D-61` | `VoucherLeaf`, proof families, family-specific negative tests. |
| 9.3 What Belongs In Canonical State | `D-18`, `D-63` | storage boundaries plus wallet owned-object payload docs. |
| 9.3.1 Per-Object Storage Split | `D-18`, `D-44` through `D-47`, `D-63` | voucher/right payload and storage field surfaces. |
| 9.4 Why Policies And ActionPool Live Mostly Outside The Committed State | `D-18`, `D-49`, `D-52`, `D-63` | descriptor hashes in state, descriptors in package/wallet/runtime docs. |
| 9.5 Conservation And Supply | `D-12`, `D-19`, `D-45`, `D-47` | storage delta checks and rights zero-value rejects. |
| 9.6 Why Not Nested Rights Or Nested Vouchers | `D-02`, `D-16`, `D-17`, `D-60` | sibling leaf families under one root. |
| 9.7 Where Objects Live And Who Uses Them | `D-03`, `D-16`, `D-21`, `D-31`, `D-32`, `D-63` | crate impact matrix and implementation summaries. |
| 9.8 End-To-End Role Path | `D-25`, `D-26`, `D-30`, `D-31`, `D-32`, `D-71` | wallet to runtime to storage to watcher to simulator evidence. |
| 9.9 Admission, Verdict, And Alert Surfaces | `D-30`, `D-31`, `D-32` | validator/watcher/rollup tests and docs. |
| 10. Security Boundary And Non-Goals | `D-20`, `D-23`, `D-30` through `D-32`, `D-48`, `D-51`, `D-53`, `D-72` | reject classes, quarantine, deferral map, negative simulator evidence. |
| 10.1 What Validators Must Verify | `D-30`, `D-52` | validator verdict surface and tests. |
| 10.2 What Core Z00Z Should Refuse | `D-08`, `D-20`, `D-47`, `D-48`, `D-67` | native cash special case, fee boundary, right zero-value, wallet reject paths. |
| 10.3 Residual Risks | deferral map | external issuer solvency, policy availability UX, oracle-heavy conditions remain explicit caps. |
| 10.4 Non-Goals | deferral map | no universal VM, no bridge semantics, no UI-polish overclaim. |
| 11. MVP Recommendation | `D-10` through `D-15`, `D-49` through `D-58` | one clean Asset, backed Voucher, generic Right model. |
| 11.1 MVP Object Set | `D-05` through `D-07`, `D-58` | live core/genesis/storage/wallet/simulator packet. |
| 11.2 MVP Use-Case Priority | `D-24`, `D-27`, `D-58`, `D-71` | simulator/UAT flows cover budget, allowance, grant, contractor/employee, claim-style paths. |
| 11.3 Future Expansion | deferral map | registry/oracle/subjective conditions and broader policy systems remain later work. |
| 11.4 From Whitepaper To Full Spec | `D-15`, `D-18`, `D-25`, `D-30`, `D-32`, `D-34` through `D-37` | all numbered plans plus final docs. |
| 12. Conclusion | `D-02`, `D-16`, `D-21`, `D-26`, `D-30`, `D-33` | integrated phase summary and final docs. |
| Appendix A. Core Claims And Non-Claims | D-ID ledger plus deferral map | this ledger, `059-TEST-SPEC.md`, simulator negative evidence. |
| Appendix B. Reading Map | corpus coverage map | `059-SOURCE-AUDIT.md`, `059-PLAN-REVIEW.md`, this ledger. |

## 📚 TODO Bullet And Table Coverage Groups

| TODO item group | Required closure | Live evidence |
|---|---|---|
| Key-term bullets for object families, policies, action pool, fee support, checkpoints, and HJMT nouns. | glossary terms frozen in code/docs/tests | core module roots, storage docs, wallet docs, runtime docs, simulator docs |
| Section 1 thesis bullets: cash finality, conditional value, delegated authority, refund/partial redeem, supply accounting, bounded settlement surface. | semantics and invariants implemented cross-crate | `059-02` through `059-09` summaries; wallet and storage invariants |
| Corpus-role table rows and maturity bullets. | corpus constraints stay explicit, not implied | `059-SOURCE-AUDIT.md`; `059-PLAN-REVIEW.md`; corpus map below |
| Minimal-triad table rows and split-minimality bullets. | one Asset/Voucher/Right triad only | core vocabulary, wallet projections, storage role boundaries |
| Asset operations, cash-grade invariants, and programmable-cash breakage rows. | native cash stays fixed and asset-only | `test_policy_descriptor`; wallet cash-only RPC guards |
| Voucher economic meaning, fully-backed model, non-final-cash table, lifecycle figure, partial redeem, encumbered-cash comparison. | voucher lifecycle and non-cash semantics are explicit | genesis/storage/wallet/simulator voucher coverage |
| Right action bullets, stateless/stateful right bullets, delegation bullets, and voucher/right distinction bullets. | right stays authority-only | rights config, zero-value guards, right actions, simulator flows |
| Policy/action/condition tables and bullets. | deterministic descriptors and fail-closed unknown-policy handling | core descriptor code, validator rejects, wallet quarantine |
| Package/witness bullets and fee-support bullets. | one package path with separate fee support | runtime object package and `FeeEnvelope` contract |
| Payment, receiver-safety, refund, and quarantine bullets. | cash path and voucher path stay separate | wallet/object RPC split, simulator negative flows |
| Storage architecture bullets/tables. | one root/path/family contract, typed deltas, proof families | storage docs, proof tests, typed delta tests |
| Security checklist bullets, residual risks, and non-goals. | all reject classes explicit; deferred scope named | validator/watcher rejects, simulator failures, deferral map |
| MVP/use-case/future-expansion/full-spec bullets. | MVP delivered; deferred expansions named honestly | phase summaries, UAT packet, deferral map |
| Appendix A and Appendix B rows. | claims/non-claims and reading map remain explicit | this ledger, source audit, plan review |

## 🌐 Referenced Corpus Coverage Map

| Corpus source | Phase 059 constraint | Live evidence or explicit cap |
|---|---|---|
| `docs/Z00Z-Main-Whitepaper.md` | wallet-local possession, checkpoint finality, soft-confirmation limits, package/settlement separation | wallet inventory and package builder, validator/watcher surfaces, simulator checkpoint/report flows |
| `docs/Z00Z-Smart-Cash-Whitepaper.md` | bounded smart-cash only; no universal private VM | fixed native cash policy, typed voucher/right model, deferral of universal VM/oracle-heavy conditions |
| `docs/Z00Z-UseCases-Whitepaper.md` | concrete budget/grant/allowance/aid/service/agent scenarios without overclaiming external issuer truth | simulator matrix and UAT scenarios for voucher/right workflows; issuer solvency remains deferred external trust |
| `docs/Z00Z-Uniqueness-Whitepaper.md` | rights-first private model, no public-account permission system, no full-wallet delegation | right object semantics, wallet-local object inventory, minimal checkpoint evidence |
| `docs/tech-papers/done/Z00Z-HJMT-Design.md` | one `SettlementStateRoot`, one `SettlementPath`, one leaf family surface, no `AssetStateRoot` revival, `RightLeaf`/`FeeEnvelope` separation | storage settlement docs, voucher leaf extension, typed proof-family tests, fee boundary guardrails |

## 🧪 Test And Artifact Evidence Index

### Core And Genesis

| Evidence class | Live anchor |
|---|---|
| policy/action descriptor determinism | `crates/z00z_core/src/assets/test_policy_descriptor.rs` |
| voucher config shape and reject paths | `crates/z00z_core/src/assets/test_voucher_config.rs` |
| right zero-value config guards | `crates/z00z_core/tests/assets/test_rights_config.rs` |
| genesis policy determinism and native-cash custom-policy rejection | `crates/z00z_core/tests/genesis/test_genesis_policies.rs` |
| genesis voucher backing and reject paths | `crates/z00z_core/tests/genesis/test_genesis_vouchers.rs` |
| genesis rights determinism | `crates/z00z_core/tests/genesis/test_genesis_rights.rs` |
| phase 059 manifest fixtures | `crates/z00z_core/src/genesis/test_genesis_suite.rs` |

### Storage

| Evidence class | Live anchor |
|---|---|
| leaf codec and family tags | `crates/z00z_storage/tests/test_settlement_leaf.rs` |
| inclusion and nonexistence proofs per family | `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs` |
| batch proof family tags and negative paths | `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`; `test_hjmt_batch_proof_negative.rs`; `test_batch_proof_support.rs` |
| typed deltas and voucher lifecycle | `crates/z00z_storage/tests/test_store_api.rs` |
| fee boundary and replay | `crates/z00z_storage/tests/test_fee_envelope.rs`; `test_fee_replay.rs` |
| live guardrails and source-shape constraints | `crates/z00z_storage/tests/test_live_guardrails.rs` |
| recovery and journal continuity | `crates/z00z_storage/src/settlement/test_live_recovery.rs` |

### Wallet

| Evidence class | Live anchor |
|---|---|
| typed object inventory and durable quarantine | `crates/z00z_wallets/src/db/redb_wallet_store/test_mod.rs` object inventory tests |
| wallet service package/reject behavior | `crates/z00z_wallets/src/services/test_wallet_service.rs` |
| object RPC namespace and lifecycle wrappers | `crates/z00z_wallets/src/adapters/rpc/methods/object.rs`; `object_impl.rs` |
| asset-only cash boundary rejects | wallet tests recorded in `059-08-SUMMARY.md` |
| backup/export/import typed object boundary | `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`; `test_backup_restore_identity.rs`; `test_backup_metadata_policy.rs`; `test_backup_kdf_contract.rs` |
| rename-guard regression for generated Kani fixtures | `crates/z00z_wallets/tests/test_rename_guards.rs` |

### Runtime And Rollup

| Evidence class | Live anchor |
|---|---|
| validator object reject classes | `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs` |
| watcher object alerts | `crates/z00z_runtime/watchers/tests/test_object_alerts.rs` |
| publication contract continuity | `crates/z00z_runtime/{validators,watchers}/tests/test_hjmt_publication_contract.rs` |
| rollup status and RPC reject-code projection | `crates/z00z_rollup_node/src/{status,rpc}.rs` |

### Simulator And Equivalent Artifact Mapping

| Requested artifact or evidence class | Live owner or equivalent | Proof anchor |
|---|---|---|
| `phase059-fixtures.json` | equivalent live fixture contract in `genesis_config_devnet*_phase059.yaml` and `object_flow_matrix` | `test_scenario1_object_flows_stage1_policy_voucher_artifacts` |
| `phase059-positive-flows.json` | equivalent live rows under `object_flow_matrix.positive` and emitted artifact set | `test_scenario1_object_flows_matrix_contract` |
| `phase059-negative-flows.json` | equivalent live rows under `object_flow_matrix.negative` | `test_scenario1_object_flows_reject_codes` |
| `phase059-proof-evidence.json` | `proof_flow.json` plus Stage 13 HJMT artifacts | `test_scenario_settlement.rs`; Stage 13 checks |
| `phase059-wallet-projections.json` | `wallet_scan.json` plus wallet inventory assertions | `test_scenario1_object_flows_wallet_inventory_for_alice_bob_charlie` |
| `phase059-critical-paths.json` | this ledger plus `object_flow_matrix` evidence files | final closeout packet |
| `asset_flow.json` | canonical simulator public evidence anchor; packet inventory row stays `pending_exact_home` | `runtime_observability.rs`; `test_scenario1_stage_surface.rs` |
| `voucher_flow.json` | canonical simulator voucher-lane evidence anchor; packet inventory row stays `pending_exact_home` | `scenario_config.yaml`; `test_scenario1_object_flows.rs`; `test_scenario_settlement.rs` |
| `right_flow.json` | canonical simulator right-lane evidence anchor; packet inventory row stays `pending_exact_home` | `scenario_config.yaml`; `test_scenario1_object_flows.rs`; `test_scenario_settlement.rs` |
| `wallet_scan.json` | live simulator artifact | stage-surface tests and object-flow tests |
| `val_flow.json` | live validator evidence artifact | `test_scenario_settlement.rs`; `test_hjmt_e2e.rs` |
| `watch_flow.json` | live watcher evidence artifact | `test_scenario_settlement.rs`; `test_hjmt_e2e.rs` |
| `sim_summary.md` | live simulator summary artifact | stage-surface and settlement tests |

## 🚫 Explicit Deferrals And Non-Goals

| Topic | Status | Why not claimed as Phase 059 live behavior |
|---|---|---|
| universal VM-like policy execution | deferred | bounded descriptor/action model is live; arbitrary hidden execution is out of scope |
| subjective or oracle-heavy conditions | deferred | MVP keeps deterministic and verifier-safe attested conditions only |
| cross-chain bridge semantics | deferred | no bridge-specific voucher/right contract is delivered in this phase |
| external issuer solvency or registry truth | deferred external trust | voucher backing is protocol-checked where modeled, but external business solvency is not core protocol truth |
| market pricing or negotiation semantics for vouchers | deferred | object model covers lifecycle and settlement, not marketplace economics |
| UI polish beyond semantic wallet/RPC boundaries | deferred | Phase 059 closes storage/runtime/wallet/simulator semantics, not presentation polish |

## ✅ Final Validation Log

This section is updated only from executed commands on the final tree.

| Command | Status | Notes |
|---|---|---|
| `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` | `passed` | reran green on the final code tree after the closeout `full_verify` clippy fix |
| `cargo test -p z00z_core --release` | `passed` | targeted package rerun completed green during `059-10` closeout validation |
| `cargo test -p z00z_storage --release` | `passed` | targeted package rerun completed green during `059-10` closeout validation |
| `cargo test -p z00z_wallets --release` | `passed` | targeted package rerun completed green during `059-10` closeout validation |
| `cargo test -p z00z_simulator --release` | `passed` | targeted package rerun completed green, including the long `test_scenario1_stage_surface` tail |
| `cargo test -p z00z_aggregators --release` | `passed` | targeted package rerun completed green during `059-10` closeout validation |
| `cargo test -p z00z_validators --release` | `passed` | targeted package rerun completed green during `059-10` closeout validation |
| `cargo test -p z00z_watchers --release` | `passed` | targeted package rerun completed green during `059-10` closeout validation |
| `cargo test -p z00z_rollup_node --release` | `passed` | targeted package rerun completed green during `059-10` closeout validation |
| `cargo test --release` | `passed` | final workspace release rerun completed green on the closeout tree |
| `cargo doc --release --no-deps` | `passed` | exit code green; rustdoc emitted only non-failing existing warnings outside the closeout claims |
| `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` | `passed` | reran green after rustfmt plus simulator clippy cleanup; long-test inventory written to `reports/full_verify-report-long-running-tests.txt` |
| `git diff --check` | `passed` | clean on the final closeout docs, planning-state sync, and simulator verification fixes |

## 🔍 Review Pass Log

| Pass | Mode | Status | Notes |
|---|---|---|---|
| `R1` | `/GSD-Review-Tasks-Execution` equivalent | `fixed` | first pass surfaced rustfmt drift in `stage4_support.rs` and a `clippy::needless_borrows_for_generic_args` reject in `test_scenario1_stage_surface.rs`; both were fixed before the rerun |
| `R2` | `/GSD-Review-Tasks-Execution` equivalent | `passed` | reran `full_verify.sh`, re-audited `059-TODO.md`, `059-CONTEXT.md`, `059-EVIDENCE-LEDGER.md`, `059-UAT.md`, crate docs, and the release outputs; no significant issues remained |
| `R3` | `/GSD-Review-Tasks-Execution` equivalent | `passed` | repeated the same closeout audit after summary/state/roadmap sync; second consecutive clean pass |
| `DC-1` | workspace-first `/doublecheck` | `passed` | verified final closeout claims against command outputs, `reports/full_verify-report-long-running-tests.txt`, simulator paths, and planning-state artifacts |

## 🧾 Closeout Verdict

Phase 059 is complete. Every row in `Final Validation Log` and
`Review Pass Log` is now closed, the final repository packet stays on one
canonical Asset/Voucher/Right authority path, and the remaining long-running
simulator rows are captured as timing evidence in
`reports/full_verify-report-long-running-tests.txt` rather than open failures.
