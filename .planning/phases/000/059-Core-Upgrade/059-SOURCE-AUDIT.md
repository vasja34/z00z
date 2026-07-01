# Phase 059 Source Audit

**Phase:** 059-Core-Upgrade
**Date:** 2026-06-16
**Scope:** Assets, Vouchers, Rights, policies, storage, wallets, runtime, and simulator.
**Inputs:** `059-TODO.md`, `059-CONTEXT.md`, the referenced whitepaper corpus,
and live source seams listed below.

## Purpose

This audit satisfies Phase 059 decisions D-38 and D-39 before execution starts.
It separates what exists today from the target object model and the migration
concerns that must be handled by the numbered plans.

Phase 059 must not jump from the whitepaper directly to code changes. The
implementation packet must first preserve the live/target distinction because
target terms such as `VoucherLeaf`, `VoucherPolicy`, `ActionPoolDescriptorV1`,
`PolicyDescriptorV1`, `ObjectWitness`, and `WalletOwnedObject` are not yet a
complete live implementation surface.

For Phase 059, that distinction is not a future-status escape hatch. Target or
future wording inside `059-TODO.md` and the referenced corpus is mandatory live
phase scope now. In this packet, `target` only means "not yet landed in
repository code today", not "optional later work".

## Referenced Corpus Constraints

| Source | Constraint imported into Phase 059 | Where plans must close it |
|---|---|---|
| `docs/Z00Z-Main-Whitepaper.md` | Wallets prepare and recognize objects locally, while checkpoint evidence remains final settlement authority; soft admission is not finality. | Runtime, wallet, simulator, and final evidence plans. |
| `docs/Z00Z-Smart-Cash-Whitepaper.md` | Smart cash is bounded typed settlement, not a universal private VM; fee support remains separate from rights. | Core policy, typed delta, runtime, and docs plans. |
| `docs/Z00Z-UseCases-Whitepaper.md` | Budget, grant, allowance, aid/community voucher, service, and agent-right scenarios are concrete test fuel; external provider truth remains outside core settlement. | Genesis fixtures, wallet RPC/UAT, simulator, and final docs plans. |
| `docs/Z00Z-Uniqueness-Whitepaper.md` | Z00Z's category claim depends on private wallet-local rights and minimal checkpoint evidence, not public accounts or delegated full-wallet authority. | Core object vocabulary, wallet inventory, runtime verdict, simulator, and UAT plans. |
| `docs/tech-papers/done/Z00Z-HJMT-Design.md` | The storage upgrade must extend `SettlementStateRoot`, `SettlementPath`, and `SettlementLeaf` families in place; `AssetStateRoot`/`AssetPath` stay archived compatibility vocabulary; `RightLeaf` and `FeeEnvelope` stay distinct. | Source audit, storage leaf/proof, typed delta, runtime, and final docs plans. |

These corpus constraints are architecture constraints, not optional background.
If implementation discovers a conflict between a local target name and a live
corpus/code contract, the executor must keep the live contract, mark the target
name as proposed, and update the plan summary instead of creating a parallel
authority layer.

## Global Findings

| Area | Live | Target | Migration concern |
|---|---|---|---|
| Object model | Native assets and partial rights support exist. Vouchers are absent. | Assets, Vouchers, and Rights are sibling settlement object classes. | Do not route vouchers through asset metadata or wallet-only payloads. |
| Genesis | `GenesisConfig` carries assets and rights. `genesis_rights.rs` emits right records and `genesis_settlement_manifest.rs` exports right artifacts. | Shared genesis orchestration with per-object generators for assets, rights, vouchers, and policies. | Preserve existing assets/rights config compatibility before adding `vouchers` and `policies`. |
| Storage | `SettlementLeaf` has `Terminal` and `Right`; proof/cache/batch tags cover terminal/right families. | One settlement root with `Terminal`, `Right`, and `Voucher` leaf families. | Stable family tags and old terminal/right decode compatibility are blocking. |
| Wallet | RedB wallet persistence centers on `OwnedAssetPayload`, `ObjectKindId::OwnedAsset`, and asset send/receive RPC. | One owned-object inventory facade with asset, voucher, and right projections. | Preserve `PAYLOAD_VERSION_OWNED_ASSET = 1` and asset spendable balance semantics. |
| Simulator | `scenario_1` validates asset/HJMT transfer lanes and some right artifact checks. | Alice/Bob/Charlie flows for all object families and cross-object interactions. | Extend staged scenario contracts instead of replacing them or forking a parallel simulator. |
| Runtime | Aggregator, validator, watcher, and rollup surfaces consume settlement roots and checkpoint evidence. | Typed package admission, validator verdicts, and watcher alerts for object-family semantics. | Runtime must carry evidence without becoming a second semantic authority. |
| Fee boundary | `FeeEnvelope` exists as processing support. | Fee support remains separate from Asset/Voucher/Right object roles. | Reject any path that makes fees a right value or voucher backing shortcut. |

## Core Live/Target Inventory

| File or module | Live | Target | Migration concern |
|---|---|---|---|
| `crates/z00z_core/src/assets/mod.rs` | Asset facade exports amount, class, definition, policy flags, serials, wire packages, and right config. | Keep `assets` as the asset-specific facade and compatibility re-export surface where needed. | Do not let `assets` become a second semantic owner for policy, voucher, right, or action logic. |
| `crates/z00z_core/src/actions` | Module root exists but is empty today. | Canonical home for action ids, action descriptors, action-pool shapes, and action-selection validation hooks. | Populate one implementation path only; if compatibility re-exports are needed, they must point back to this canonical owner. |
| `crates/z00z_core/src/policies` | Module root exists but is empty today. | Canonical home for policy ids, policy descriptors, condition descriptors, and canonical hash/byte rules. | Do not duplicate descriptor or canonicalization logic under `assets`, `genesis`, or runtime-local helpers. |
| `crates/z00z_core/src/rights` | Module root exists but is empty today. | Canonical home for right-specific types, lifecycle/authority vocabulary, and compatibility facades over existing right config support. | Move toward one rights owner path instead of leaving semantics split between `assets` tests and scattered helpers. |
| `crates/z00z_core/src/vauchers` | Module root exists but is empty today. | Canonical home for voucher config, lifecycle, backing metadata, and voucher-specific semantics. | Keep the current spelling as the canonical repository path for Phase 059; do not create a parallel `vouchers/` owner tree. |
| `crates/z00z_core/src/assets/right_config.rs` | `RightsConfigEntry` exists with scopes, holder/control/beneficiary fixtures, validity windows, policy ids, payload seed, metadata, and forbidden value-bearing keys. | Reuse as the authority-only config pattern for rights and as a rejection model for value-bearing right fields. | Existing right support is live but not complete; do not mark all right interactions done. |
| `crates/z00z_core/src/assets/policy_flags.rs` | Native asset policy flags exist for cash-grade asset behavior. | Fixed `CashPolicy` remains special; voucher/right policy descriptors are separate. | Do not retrofit arbitrary action pools onto native cash. |
| `crates/z00z_core/src/assets/definition_id.rs` and `serial_id.rs` | Asset-style identifiers and serial paths exist. | Voucher and right objects must still map into deterministic `definition_id -> serial_id -> terminal_id` path rules. | Objects without finite supply still need stable serial-bucket derivation rules. |
| `crates/z00z_core/src/assets/wire_pkg*.rs` | Asset wire/package serde exists. | Object package and witness vocabulary must bind family, selected action, descriptor hash, root, and deltas. | Avoid using asset wire packages as generic object packages without typed family checks. |
| `crates/z00z_core/src/genesis/genesis_config.rs` | `GenesisConfig` includes chain, assets, rights, outputs, and performance. | Add typed `policies` and optional/bootstrap `vouchers` sections under one genesis boundary. | Existing configs can break if new arrays are required without defaults. |
| `crates/z00z_core/src/genesis/genesis_config_validate.rs` | Validates chain/assets/rights and current rights requiredness. | Validate policy descriptors, action pools, voucher backing/reserve, lifecycle, and zero-value rights. | Decide compatibility for empty rights and missing vouchers/policies before schema changes. |
| `crates/z00z_core/src/genesis/genesis_rights.rs` | Generates deterministic right leaves and records with domain-separated derivations. | Keep right generation and add missing policy/challenge/forbidden-field tests. | Voucher derivations must use separate labels and cannot reuse right domains. |
| `crates/z00z_core/src/genesis/genesis_settlement_manifest.rs` | Exports `genesis_rights.json` and a single settlement manifest. | Add policy and voucher artifacts while preserving the manifest role. | Manifest consumers must remain able to locate old rights artifacts and new object artifacts. |

## Storage Live/Target Inventory

| File or module | Live | Target | Migration concern |
|---|---|---|---|
| `crates/z00z_storage/src/settlement/record.rs` | `SettlementLeaf` covers `Terminal` and `Right`; `RightLeaf`, `RightAction`, and fee records exist. | Add `VoucherLeaf`, voucher lifecycle/action contexts, family accessors, and typed delta records. | Do not add voucher state outside `z00z_storage::settlement` as a wallet-only truth. |
| `crates/z00z_storage/src/settlement/leaf.rs` | Leaf encoding and serialization cover terminal/right paths. | Add stable voucher family tags, serde labels, and unknown-tag rejection. | Binary tags are durable; old rows must keep decoding. |
| `crates/z00z_storage/src/settlement/proof.rs` | `SettlementLeafFamily` and marker leaves cover terminal/right. | Add `SettlementLeafFamily::Voucher`, family-specific marker leaves, inclusion, deletion, and nonexistence checks. | Prove asset paths cannot validate voucher/right leaves and voucher paths cannot validate other families. |
| `crates/z00z_storage/src/settlement/proof_batch.rs` | Batch proof tags include asset/terminal and right families. | Add voucher batch tags without tag reuse. | Shared proof compatibility and old proofs must remain deterministic. |
| `crates/z00z_storage/src/settlement/proof_batch_verify.rs` | Batch verifier checks current family tags. | Reject wrong-family voucher proofs and stale roots. | Negative tests must cover all cross-family combinations. |
| `crates/z00z_storage/src/settlement/hjmt_cache.rs` | Durable family encode/decode uses `1 = Terminal`, `2 = Right`. | Add a durable voucher code and migration/compat tests. | Never reorder existing codes. |
| `crates/z00z_storage/src/settlement/store.rs` | Store/apply APIs cover terminal/right and fee operations. | Add generic typed object operations or explicit voucher operations with one authority boundary. | Store must not interpret wallet secrets or private openings. |
| `crates/z00z_storage/src/settlement/model.rs` | Semantic model supports current root determinism. | Mixed object deltas must update roots deterministically for assets, vouchers, and rights. | Conservation must include value-bearing assets/vouchers and exclude rights. |
| `crates/z00z_storage/src/settlement/fee_envelope.rs` | Fee support is separate. | Fee support remains outside object roles. | Negative tests must reject fee semantics hidden in rights or voucher backing shortcuts. |

## Wallet Live/Target Inventory

| File or module | Live | Target | Migration concern |
|---|---|---|---|
| `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs` | `ObjectKindId::OwnedAsset = 21`; `OwnedAssetStatus`; `OwnedAssetPayload` embeds `AssetWire`. | Add `WalletOwnedObject` facade or typed `OwnedVoucher`/`OwnedRight` kinds behind one query facade. | Reusing `OwnedAsset` for all object classes is rejected. Preserve asset payload version. |
| `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs` | Asset persistence, reserve, restore, scan batch, and quarantine patterns exist. | Add object-family persistence, indexes, lifecycle transitions, durable quarantine, and typed restore. | Cash spendable lists remain asset-only. |
| `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs` | Receive flow calls `recv_claim_asset` and returns assets. | Classify settlement leaves by family before payload recovery. | Voucher/right discovery must not use asset-only stealth recovery assumptions. |
| `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs` | Asset send/receive RPC and quarantined asset behavior are asset-specific. | Preserve asset RPC as cash-only and add typed object/voucher/right RPC namespaces. | Do not overload `asset.send` or `asset.receive` for voucher/right semantics. |
| `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_history.rs` | Asset quarantine is partly RPC-layer and in-memory. | Durable, family-aware, policy-aware quarantine excluded from balance. | Quarantine must survive restart and restore. |
| `crates/z00z_wallets/src/backup/backup_wire.rs` | Backup wire primarily carries wallet and asset payload state. | Include voucher/right payloads, descriptor refs, openings, and quarantine states. | Prevent secret leakage and keep old backups importable. |

## Runtime And Rollup Live/Target Inventory

| File or module | Live | Target | Migration concern |
|---|---|---|---|
| `crates/z00z_runtime/validators/src/tx_verify.rs` | Validates current transaction/checkpoint assumptions. | Validate policy/action membership, rights, signatures, attestations, lifecycle, typed deltas, conservation, and fee boundary. | Validator rejection reasons must be precise enough for simulator artifacts and watcher alerts. |
| `crates/z00z_runtime/validators/src/verdict.rs` | Verdict surface exists for current validation outcomes. | Add object-family verdict codes for voucher/right/policy/lifecycle failures. | Do not collapse all object failures into generic invalid transaction. |
| `crates/z00z_runtime/aggregators/src/ingress.rs` and `batch_planner.rs` | Routes current packages and batch work. | Carry typed object packages and route-bound deltas. | Aggregator packages work; it is not semantic authority. |
| `crates/z00z_runtime/watchers/src/alerts.rs` | Watcher alert vocabulary exists. | Alert on unknown policy, invalid backing, value-bearing right, wrong-family proof, replay, double redemption, expired use, forced acceptance, and stale roots. | Watchers consume evidence and roots; they do not repair state. |
| `crates/z00z_rollup_node/src/runtime.rs` and `rpc.rs` | Rollup runtime and RPC surface checkpoint/state evidence. | Surface object-family publication/verdict fields where checkpoint verification depends on them. | Keep wire compatibility and avoid exposing wallet secrets. |

## Simulator Live/Target Inventory

| File or module | Live | Target | Migration concern |
|---|---|---|---|
| `crates/z00z_simulator/src/scenario_1/stage_1.rs` | Genesis/config scenario entrypoint exists. | Emit genesis assets, rights, policies, bootstrap vouchers, and settlement manifest evidence. | YAML config and design contract must stay synchronized. |
| `crates/z00z_simulator/src/scenario_1/stage_4.rs` and `stage_4_utils/tx_preparation_core.rs` | Builds asset spend membership witnesses from asset wire/terminal leaves. | Prepare typed object packages and witnesses for assets, vouchers, rights, policies, fees, and required-right links. | Do not force voucher/right actions into asset spend witness types. |
| `crates/z00z_simulator/src/scenario_1/stage_5.rs` and `stage_5_utils/transfer_lane_impl.rs` | Asset transfer lane uses asset receive response types. | Exercise receiver behavior by object family: cash one-sided receive, voucher offer/accept/reject, right grant/delegate/consume. | Receiver safety requires voucher acceptance to be explicit. |
| `crates/z00z_simulator/src/scenario_1/stage_6.rs` | Bundle/checkpoint logic carries current asset deltas. | Carry typed mixed-object deltas into bundle/checkpoint logic. | Storage root evidence must show deleted, created, and updated residual objects. |
| `crates/z00z_simulator/src/scenario_1/stage_11.rs` | Scan/apply and Charlie handoff exist for current flows. | Scan/apply all object families and validate Alice/Bob/Charlie handoffs. | A scenario is incomplete without wallet persistence and scan/apply evidence. |
| `crates/z00z_simulator/src/scenario_1/stage_13.rs` | HJMT examples and negative storage examples exist. | Add examples for voucher leaves and wrong-family proof failures. | Release evidence must include negative artifacts, not only success logs. |
| `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` and `scenario_design.yaml` | Scenario contracts drive current asset/HJMT stages. | Add object policy, action, actor, and expected-verdict coverage. | Runner contract must fail when YAML and executable stages diverge. |

## Blocking Implementation Rules

- Assets remain final spendable value and the only spendable cash projection.
- Vouchers are conditional value claims with backing/reserve evidence.
- Rights are zero-value authority objects.
- Future/target design statements in `059-TODO.md` and the referenced corpus are
  mandatory Phase 059 scope now; `target` means not-yet-landed code only.
- `FeeEnvelope` remains support-only and never becomes object value or authority.
- Vouchers and rights are sibling settlement objects, not nested asset payloads.
- One `SettlementStateRoot` and one `SettlementPath` contract remain authoritative.
- Unknown policies fail closed for validators and quarantine in wallets.
- Every concept must end with one canonical module/function owner path.
  Compatibility re-exports are allowed; duplicate semantic owners are not.
- Runtime, wallets, and simulator consume storage/validator semantics; they do
  not become parallel authorities.

## Required Follow-Up Evidence

The numbered plans must produce and verify:

- stable policy/action descriptor hashes;
- genesis artifacts for policies and bootstrap vouchers;
- voucher family tags across serde, bincode, proof, batch proof, cache, and recovery;
- mixed object deltas with conservation and residual voucher checks;
- wallet object inventory and durable quarantine;
- asset-only spendable balance;
- typed RPC/service package builders;
- validator verdicts and watcher alerts;
- Alice/Bob/Charlie positive and negative simulator artifacts;
- one canonical owner map for `assets`, `actions`, `policies`, `rights`, and
  `vauchers`, with explicit compatibility re-exports where needed;
- full targeted and release validation listed in `059-TEST-SPEC.md`.
