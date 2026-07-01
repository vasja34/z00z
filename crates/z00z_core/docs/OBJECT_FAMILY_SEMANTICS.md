# Object Family Semantics Matrix

This file is the one canonical semantics matrix for the live `z00z_core`
object corpus. Use it when docs, tests, wallets, storage, or simulator code
need one authoritative statement for assets, rights, policies, and vouchers.

`GenesisConfig` stays the only bootstrap authority. The runtime
`ObjectFamily` enum remains the live leaf-family vocabulary for
`asset | voucher | right`; the policy row is included here because policy
descriptors are part of the same bootstrap corpus and are consumed by the live
runtime validator path.
`z00z_core::ObjectFamily` is the canonical caller-visible path for that
vocabulary. `z00z_core::assets::ObjectFamily` remains a compatibility facade
only.

## Matrix

| Family | Genesis can create it? | Runtime can create or mutate it? | Bootstrap shape vs runtime shape | Value semantics | Policy binding | Backing requirement | Canonical caller path |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Asset | Yes. `GenesisConfig.assets` materializes deterministic asset definitions and bootstrap assets. | Runtime can commit terminal-asset deltas through `SettlementActionV1::AssetMutation` and can create assets as voucher refund or redeem outputs, but the current public runtime does **not** expose a generic post-genesis asset-mint selector. `mintable` is a definition/catalog flag, not a proven general issuance API. | Yes. Bootstrap uses `AssetConfigEntry` plus `PolicyConfig`; runtime persists terminal asset leaves. | Value-bearing final settlement value. | No per-asset bootstrap `policy_id` is embedded in the asset itself. When an asset participates in a runtime object package, that package still binds one `policy_descriptor_hash` and action pool through the validator path. | No explicit reserve/backing field. | `z00z_core::assets::...` |
| Right | Yes. `GenesisConfig.rights` materializes deterministic right leaves. | Yes. Runtime can create, transfer, consume, expire, revoke, and challenge rights. | Yes. Bootstrap uses `RightsConfigEntry`; runtime persists `RightLeaf`. | Zero-value authority only. Rights must never carry declared value units. | Explicit. Runtime right leaves carry per-action policy ids, and runtime packages must bind the matching policy/action contract. | No explicit reserve/backing field. | `z00z_core::rights::...` |
| Policy | Yes. `GenesisConfig.policies` materializes deterministic policy descriptors and action-pool bindings. | No runtime settlement-leaf create or mutate lane exists. Runtime consumes policy descriptors by hash for wallet preview/build and validator checks. | Partly. Bootstrap uses `PolicyConfigEntryV1`, which materializes `PolicyDescriptorV1`; runtime keeps the descriptor/hash contract rather than a settlement leaf. | Descriptor only; not a value object. | Self-authored descriptor contract. The runtime bind point is `policy_descriptor_hash` plus `action_pool_id`. | No explicit reserve/backing field. | `z00z_core::policies::...` |
| Voucher | Yes. `GenesisConfig.vouchers` materializes deterministic bootstrap vouchers under one genesis manifest. | Yes. Runtime can issue, accept, reject, transfer, redeem, refund, and expire vouchers. | Yes. `VoucherBootstrapEntryV1` is a bootstrap-only input that materializes a `VoucherConfigEntry`; the live runtime object is `VoucherLeaf`. | Conditional value-bearing claim with `face_value` and `remaining_value`. | Explicit. Vouchers carry `policy_id` and `action_pool_id`, and validator checks reject wrong-family or unknown-policy packages. | Required. Every live voucher must keep explicit `ConsumedAsset`, `ReserveCommitment`, or `GenesisReserve` backing. | `z00z_core::vouchers::...` |

## Guardrails

- Do not describe vouchers as merely `asset + right`. They are their own
  value-bearing family with explicit backing, lifecycle, and policy/action
  bindings.
- Do not describe rights as value-carrying. The runtime rejects value-bearing
  right deltas.
- Do not describe `mintable` as a live generic runtime mint API. On the
  current tree it is a definition/catalog flag only, and the public runtime
  selectors create vouchers and rights, not arbitrary new assets.
- Do not treat policy descriptors as settlement leaves. They are
  content-addressed runtime validation inputs.

## Live Anchors

- Asset row
  - `crates/z00z_core/src/genesis/genesis_config.rs`
  - `crates/z00z_core/src/assets/object_family.rs`
  - `crates/z00z_storage/src/settlement/record.rs`
  - `crates/z00z_storage/src/settlement/object_package_contract.rs`
  - `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`
- Right row
  - `crates/z00z_core/src/rights/config.rs`
  - `crates/z00z_storage/src/settlement/record.rs`
  - `crates/z00z_storage/src/settlement/tx_plan_types.rs`
  - `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`
- Policy row
  - `crates/z00z_core/src/policies/policy_descriptor.rs`
  - `crates/z00z_core/src/policies/policy_template.rs`
  - `crates/z00z_storage/src/settlement/object_package_contract.rs`
  - `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`
- Voucher row
  - `crates/z00z_core/src/vouchers/voucher_bootstrap.rs`
  - `crates/z00z_core/src/vouchers/voucher_config.rs`
  - `crates/z00z_storage/src/settlement/record.rs`
  - `crates/z00z_storage/src/settlement/tx_plan_types.rs`
  - `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`
