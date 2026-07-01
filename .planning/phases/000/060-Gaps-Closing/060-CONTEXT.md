# Phase 060: Gaps-Closing - Context

**Gathered:** 2026-06-20
**Status:** Planning authority mirrored from `060-TODO.md`; supplemental HJMT plus wallet/object plus runtime audit-reopen packet synchronized through `060-15-PLAN.md`

<domain>
## 🎯 Final Authority Reset

Phase 060 already exists at `.planning/phases/060-Gaps-Closing/`.

`060-TODO.md` is the canonical Phase 060 design and execution authority.
`060-CONTEXT.md` is a derived transfer mirror for pre-implementation review.
It confirms that all implementation-facing tasks, checks, confirmations,
non-goals, and anti-drift constraints from `060-TODO.md` are carried into one
phase-local context file.

`060-12-PLAN.md` is a subordinate audit-driven reopen for the HJMT storage and
publication lane. It does not create a new canonical Phase 060 task id. It
closes live seams discovered by the code-and-docs audit against the HJMT corpus
while staying subordinate to `060-TODO.md` and the existing `B1..B6` task
surface.

`060-13-PLAN.md` is a subordinate audit-driven reopen for wallet prepared-tx
balance coverage, voucher conservation coverage, and typed-object `FeeEnvelope`
reject coverage. It does not create a new canonical Phase 060 task id. It
closes live test and reject-path seams discovered by the code-and-docs audit
against the wallet/object corpus while staying subordinate to `060-TODO.md`
and the existing `D2..D5` task surface.

`060-14-PLAN.md` is a subordinate audit-driven reopen for voucher refund or
source-context binding, one-plane wallet object issue or create construction,
and real validator or watcher incomplete surfacing on the existing publication
contract. It does not create a new canonical Phase 060 task id. It closes late
Phase 059 semantic and runtime seams discovered by the code-and-docs audit
while staying subordinate to `060-TODO.md` and the existing `D2..D5` plus HJMT
publication task surface.

`060-15-PLAN.md` is a subordinate narrowed MVP-successor packet for the still-
open subset revalidated against the live codebase: refund or restricted-source
binding, one-plane voucher issue or right create construction, real
`Incomplete` or `ValidatorIncomplete` runtime surfacing, and monotonic right
delegation on the existing transfer path. It does not create a new canonical
Phase 060 task id. It supersedes the overlapping execution scope of
`060-14-PLAN.md` for these exact gaps, must not run as a parallel duplicate
packet, and stays subordinate to `060-TODO.md` plus the existing `D2..D5` and
HJMT publication task surface.

This context must not become a second spec, a second backlog, or a planner-
invented parallel layer. If Phase 060 scope changes, update `060-TODO.md`
first and then synchronize this file.

## 📌 Phase Boundary

Phase 060 closes four linked workstreams without widening into a second
architecture:

- Workstream A: `z00z_core` authority and ownership cleanup.
- Workstream B: HJMT topology, failover, and measurement evidence.
- Workstream C: verification gate closure plus verification-pipeline
  performance closure.
- Workstream D: wallet typed inventory and MVP rights or vouchers profile
  matrix.

Phase 060 must preserve the current design foundation:

- no duplicate codebase logic;
- no parallel implementation or planning layer;
- no second bootstrap or wallet authority plane;
- no direct edits under `crates/z00z_crypto/tari/**`;
- no concept drift from the live codebase or the canonical Phase 060 sources.

## ⚙️ Task Transfer Confirmation

The table below confirms the transfer of every implementation-facing task from
`060-TODO.md` into this context.

| TODO task | Canonical meaning carried from `060-TODO.md` | Must be completed, checked, or confirmed | Transfer status |
| --- | --- | --- | --- |
| `A1` | Freeze `z00z_core::genesis` as the single bootstrap authority. | Confirm one canonical authority statement and remove conflicting bootstrap stories. | `confirmed` |
| `A2` | Finish the `rights` owner move from `assets` to `rights`. | Complete owner-path cleanup, import migration, and regression guard against owner drift. | `confirmed` |
| `A3` | Demote shim-first imports under `assets/`. | Confirm direct owner-module imports become canonical and shims are compatibility-only. | `confirmed` |
| `A4` | Remove dual-authority YAML risk. | Confirm `rights` bootstrap authority does not remain implied by `assets_config.yaml`. | `confirmed` |
| `A5` | Repair documentation and fixture drift. | Complete doc and fixture sync with the canonical bootstrap boundary. | `confirmed` |
| `B1` | Publish the live HJMT topology contract. | Confirm docs and tests agree that one aggregator is one process by default and may own multiple shards. | `confirmed` |
| `B2` | Add YAML-selectable shard execution mapping. | Complete config option design with `aggregator_owned` default and `shard_process` as opt-in only. | `confirmed` |
| `B3` | Add explicit decommission and removal coverage. | Confirm full-owner removal, route cleanup, and same-lineage failover coverage. | `confirmed` |
| `B4` | Add `3A7S -> 2A7S -> 5A7S` HJMT scenario. | Complete fail-down and fail-up scenario coverage with lawful ownership and lineage assertions. | `confirmed` |
| `B5` | Separate measurement evidence lanes. | Confirm Criterion, `/usr/bin/time`, scenario runtime, and user-facing TPS claims stay distinct. | `confirmed` |
| `B6` | Re-run measurements only after topology and provenance close. | Confirm A/B performance and failover gate before any `shard_process` promotion claim. | `confirmed` |
| `C1` | Close `l0-docs`. | Complete strict docs-gate posture, lint backlog closure, and `ZINV` traceability anchors. | `confirmed` |
| `C2` | Close `l4-supply-chain`. | Complete project-owned advisory handling, reviewed records, and cargo-vet maturity review. | `confirmed` |
| `C3` | Close `l4-adversarial-review`. | Confirm all `11` high findings are normalized, owned, and closed by artifact, not prose. | `confirmed` |
| `C4` | Reduce verification-pipeline runtime for the top `5` slowest events. | Complete measurable optimization backlog without weakening verification evidence. | `confirmed` |
| `D1` | Publish the authoritative MVP profile catalog. | Complete repository-owned profile definitions with actions, policy ids, and reject surfaces. | `confirmed` |
| `D2` | Define wallet projection semantics per profile. | Confirm `wallet.object.*`, `wallet.asset.*`, quarantine, and spendability semantics per profile. | `confirmed` |
| `D3` | Specify `validator_mandate_lock_v1`. | Complete lock-profile contract on top of `RightLeaf::ValidatorMandate` without slashable-v1 drift. | `confirmed` |
| `D4` | Expand validator, watcher, and simulator coverage. | Confirm fail-closed profile tests, lock behavior, replay rejection, and privacy-preserving proof expectations. | `confirmed` |
| `D5` | Keep the one-plane wallet authority. | Confirm `.wlt`, `WalletExportPack`, `wallet.object.*`, and cash-only `wallet.asset.*` remain canonical. | `confirmed` |

## 🗺️ Task To Plan Transfer Matrix

The table below confirms where each canonical Phase 060 task is implemented in
the numbered plan packet.

| TODO task | Primary plan owner | Plan title | Transfer status |
| --- | --- | --- | --- |
| `C1` | `060-01-PLAN.md` | Docs Gate Posture And ZINV Traceability | `confirmed` |
| `A1` | `060-02-PLAN.md` | Canonical Bootstrap Authority Freeze | `confirmed` |
| `B1` | `060-03-PLAN.md` | HJMT Process Model And YAML Shard Mapping Contract | `confirmed` |
| `B2` | `060-03-PLAN.md` | HJMT Process Model And YAML Shard Mapping Contract | `confirmed` |
| `A2` | `060-04-PLAN.md` | Rights Owner Move, Shim Demotion, And Dual-Authority YAML Closure | `confirmed` |
| `A3` | `060-04-PLAN.md` | Rights Owner Move, Shim Demotion, And Dual-Authority YAML Closure | `confirmed` |
| `A4` | `060-04-PLAN.md` | Rights Owner Move, Shim Demotion, And Dual-Authority YAML Closure | `confirmed` |
| `A5` | `060-04-PLAN.md` | Rights Owner Move, Shim Demotion, And Dual-Authority YAML Closure | `confirmed` |
| `B3` | `060-05-PLAN.md` | HJMT Decommission Coverage And `3A7S -> 2A7S -> 5A7S` Scenario | `confirmed` |
| `B4` | `060-05-PLAN.md` | HJMT Decommission Coverage And `3A7S -> 2A7S -> 5A7S` Scenario | `confirmed` |
| `C2` | `060-06-PLAN.md` | Supply-Chain Review Records And Vet Trust Closure | `confirmed` |
| `D1` | `060-07-PLAN.md` | Wallet MVP Profile Catalog And One-Plane Projection Semantics | `confirmed` |
| `D2` | `060-07-PLAN.md` | Wallet MVP Profile Catalog And One-Plane Projection Semantics | `confirmed` |
| `D5` | `060-07-PLAN.md` | Wallet MVP Profile Catalog And One-Plane Projection Semantics | `confirmed` |
| `D3` | `060-08-PLAN.md` | `validator_mandate_lock_v1` Contract And Fail-Closed Profile Coverage | `confirmed` |
| `D4` | `060-08-PLAN.md` | `validator_mandate_lock_v1` Contract And Fail-Closed Profile Coverage | `confirmed` |
| `C3` | `060-09-PLAN.md` | Adversarial High-Finding Closure And Count Reconciliation | `confirmed` |
| `B5` | `060-10-PLAN.md` | HJMT Measurement Lanes And A/B Rerun Packet | `confirmed` |
| `B6` | `060-10-PLAN.md` | HJMT Measurement Lanes And A/B Rerun Packet | `confirmed` |
| `C4` | `060-11-PLAN.md` | Verification-Pipeline Performance And Final Closure Reruns | `confirmed` |

Canonical `A1..D5` routing still ends at `060-11`. `060-12`, `060-13`,
`060-14`, and `060-15` are not new canonical task owners; they are subordinate
HJMT, wallet/object, and runtime audit-reopen slices that close live
storage/publication, wallet/object, refund/source, completeness-surface, and
right-delegation seams left under the existing Workstream B and Workstream D
contracts. `060-15` is the narrowed MVP-successor packet for the overlapping
`060-14` execution scope and must not become a second parallel implementation
layer.

## 🔁 Supplemental HJMT Reopen Coverage

The table below records the additional HJMT closeout seams surfaced by the live
codebase audit and the referenced HJMT docs corpus. These seams do not create
new Phase 060 task ids, but they must be reflected in the packet so the Phase
060 closeout does not claim more HJMT completion than the repository actually
has.

| Supplemental reopen seam | Derived source basis | Plan owner | Why this does not create a new canonical task id | Transfer status |
| --- | --- | --- | --- | --- |
| Live root-of-shard-roots must become the persisted `SettlementStateRoot` and reload truth | `HJMT-RAID -Sharding.md`, `Z00Z-HJMT-Upgrade.md`, `Z00Z-HJMT-Gaps.md`, live storage root code | `060-12-PLAN.md` | It is a late-discovered storage-wiring closeout under the existing HJMT publication and measurement lane, not a new product surface. | `confirmed` |
| Durable journal and recovery export must carry shard identity, routing generation, route digest, and lineage from storage-owned state | `HJMT-RAID -Sharding.md`, `Z00Z-HJMT-Threat-Model.md`, `Z00Z-HJMT-Upgrade.md`, live recovery code | `060-12-PLAN.md` | It closes the lawful failover and same-lineage durability contract already required by `B1..B4`; it does not invent a second failover model. | `confirmed` |
| Exact publication coverage must be validated against the committed route table in one shared acceptance path | `HJMT-RAID -Sharding.md`, `Z00Z-HJMT-Upgrade.md`, `Z00Z-HJMT-Gaps.md`, live publication and validator code | `060-12-PLAN.md` | It closes a split-authority acceptance seam inside the existing HJMT route/publication contract instead of adding a new protocol. | `confirmed` |

## 🔁 Supplemental Wallet/Object Reopen Coverage

The table below records the additional wallet/object closeout seams surfaced by
the live codebase audit and the balance-validation plus fee-support corpus.
These seams do not create new Phase 060 task ids, but they must be reflected
in the packet so the Phase 060 closeout does not claim more prepared-tx,
voucher conservation, or typed-object fee-support coverage than the repository
actually has.

| Supplemental reopen seam | Derived source basis | Plan owner | Why this does not create a new canonical task id | Transfer status |
| --- | --- | --- | --- | --- |
| Ordinary prepared `wallet.tx.build_transaction` must have explicit negative coverage for plaintext and commitment mismatch on the canonical `TxAssembler` path | `🔐 CRYPTOGRAPHIC BALANCE VALIDATION TEST.md`, live wallet tx builder code, live `TxAssembler` code | `060-13-PLAN.md` | It closes a late-discovered reject-coverage seam under existing `D2`, `D4`, and `D5` wallet fail-closed semantics instead of adding a new tx design. | `confirmed` |
| Voucher value-bearing flows must have explicit conservation mismatch coverage on the existing typed object package path, while rights remain outside value-bearing balance scope | `060-TODO.md`, live `ObjectDeltaSetV1` conservation code, live object-package reject mapping, live wallet object RPC code | `060-13-PLAN.md` | It closes a live coverage seam inside the existing voucher and wallet object model instead of introducing a second value-accounting plane or widening rights into cash semantics. | `confirmed` |
| Typed-object `FeeEnvelope` packages must reject malformed structural contracts on preview/build/validator paths while preserving the shipped native fee-output lane for regular cash tx | `docs/Z00Z-Main-Whitepaper.md`, `docs/Z00Z-Litepaper.md`, `docs/tech-papers/done/Z00Z-HJMT-Design.md`, live `FeeEnvelope` support-contract code, and live object-package reject mapping | `060-13-PLAN.md` | It closes a fee-support reject seam inside the existing object package and storage commit contract instead of collapsing regular tx fees into a new universal fee object or widening rights into value-bearing semantics. | `confirmed` |

## 🔁 Supplemental Object/Runtime Reopen Coverage

The table below records the additional Phase 059 semantic and runtime seams
surfaced after the earlier wallet/object reopen. These seams do not create new
Phase 060 task ids, but they must be reflected in the packet so the phase does
not overclaim refund safety, object-package construction completeness, or
validator/watcher liveness truth.

| Supplemental reopen seam | Derived source basis | Plan owner | Why this does not create a new canonical task id | Transfer status |
| --- | --- | --- | --- | --- |
| Voucher reject and refund outputs must bind to the declared `refund_target_commitment`, and restricted backing must also bind to the declared source context | `059-TODO.md`, live `VoucherLeaf` and `VoucherBackingRef` storage code, live object-delta and store API code | `060-15-PLAN.md` | It closes a late-discovered semantic safety seam inside the existing voucher object model instead of inventing a new refund policy layer or a second state object family. | `confirmed` |
| `wallet.object.preview_package` and `wallet.object.build_package` must accept source `Asset` or reserve context for `VoucherAction::Issue` and the same request family should cover `RightAction::Create` without a parallel API | `059-TODO.md`, live wallet object RPC request or builder code, live wallet inventory stores, live object package contract | `060-15-PLAN.md` | It widens the existing one-plane typed object package surface instead of creating a second RPC namespace or a second builder contract. | `confirmed` |
| `VerdictKind::Incomplete` and `AlertKind::ValidatorIncomplete` must become real runtime states for missing-artifact, retry-pending, and publication-gap cases on the existing publication path | `059-TODO.md`, `Z00Z-HJMT-Design.md`, `Z00Z-HJMT-Gaps.md`, `Z00Z-HJMT-Threat-Model.md`, `Z00Z-HJMT-Upgrade.md`, and live validator or watcher publication code | `060-15-PLAN.md` | It closes a late-discovered liveness and visibility seam on the current publication contract instead of adding a second completeness checker or a second evidence path. | `confirmed` |
| `delegate_right` must not widen authority relative to the parent live right, and unsupported attenuation dimensions must fail closed instead of being overclaimed as MVP-complete | `059-TODO.md`, live `RightLeaf` transfer contract, `RightRequirementV1`, live wallet object delegation code, and live right-policy evaluation code | `060-15-PLAN.md` | It closes an authority-widening seam on the existing right transfer path instead of inventing a second right hierarchy, second delegation API, or non-MVP authority lattice. | `confirmed` |

## 🧪 Task To Test Contract Transfer Matrix

The table below confirms where each canonical Phase 060 task is carried into
the phase-local test packet formed by `060-TEST-SPEC.md` and
`060-TESTS-TASKS.md`.

| TODO task | Scenario id | Test-task step | Required proof anchor | Transfer status |
| --- | --- | --- | --- | --- |
| `A1` | `060-S02` | `060-02` | one bootstrap authority story plus phase 059 genesis fixture proof | `confirmed` |
| `A2` | `060-S04` | `060-04` | `rights/` owner-path migration and regression guard | `confirmed` |
| `A3` | `060-S04` | `060-04` | shim demotion and compatibility-only import guard | `confirmed` |
| `A4` | `060-S04` | `060-04` | no dual-authority YAML layer and no symmetry-only `*_config.yaml` drift | `confirmed` |
| `A5` | `060-S04` | `060-04` | docs and fixtures tell one bootstrap story | `confirmed` |
| `B1` | `060-S03` | `060-03` | one aggregator equals one OS process by default, with many shards per process under `aggregator_owned` | `confirmed` |
| `B2` | `060-S03` | `060-03` | canonical YAML default `execution.shard_mapping: "aggregator_owned"` and opt-in `shard_process` rejection paths | `confirmed` |
| `B3` | `060-S05` | `060-05` | owner removal, route cleanup, and same-lineage failover proof | `confirmed` |
| `B4` | `060-S05` | `060-05` | lawful `3A7S -> 2A7S -> 5A7S` stage evidence | `confirmed` |
| `B5` | `060-S10` | `060-10` | separated Criterion, `/usr/bin/time -v`, stage-runtime, and user-facing throughput lanes | `confirmed` |
| `B6` | `060-S10` | `060-10` | fair A/B packet with durable throughput, sync, latency, RSS, CPU, restart, and failover metrics | `confirmed` |
| `C1` | `060-S01` | `060-01` | strict docs gate plus non-zero `ZINV` anchors | `confirmed` |
| `C2` | `060-S06` | `060-06` | repo-owned advisory decisions and `cargo vet` delta evidence | `confirmed` |
| `C3` | `060-S09` | `060-09` | one closure artifact per high finding plus count reconciliation | `confirmed` |
| `C4` | `060-S11` | `060-11` | top-slowest verification runtime reductions with unchanged verification semantics | `confirmed` |
| `D1` | `060-S07` | `060-07` | repository-owned MVP profile catalog with live-versus-proposed labels | `confirmed` |
| `D2` | `060-S07` | `060-07` | wallet.asset.* remains cash-only, and unknown-policy objects remain in durable quarantine | `confirmed` |
| `D3` | `060-S08` | `060-08` | exact `validator_mandate_lock_v1` field grammar and layered-on-live-id contract | `confirmed` |
| `D4` | `060-S08` | `060-08` | wrong-family, replay, spend-block, unlock, and privacy fail-closed coverage | `confirmed` |
| `D5` | `060-S07` | `060-07` | one wallet authority plane across `wallet.object.*`, `wallet.asset.*`, `.wlt`, and `WalletExportPack` | `confirmed` |

The subordinate wallet/object reopen is routed separately into `060-S13`, and
the subordinate refund/source, incomplete-runtime, and monotonic-delegation
reopen is routed into `060-S14`, so the packet can prove these additional
seams without minting a new canonical `D*` task id or a second scenario
family.

## 📚 Source Corpus Transfer Matrix

The table below confirms that the canonical three-source Phase 060 corpus, the
supplemental HJMT plus wallet/object plus runtime audit corpus, and the key
codebase anchors named by `060-TODO.md` are transferred into this context and
the numbered plans without creating a second authority layer.

| Source corpus or live anchor | Required transfer target | Transfer status |
| --- | --- | --- |
| `.planning/phases/060-Gaps-Closing/060-TZ1.md` and `060-TZ1.md` | Drive Workstream A and Workstream B closure across `060-02`, `060-03`, `060-04`, `060-05`, and `060-10`. | `confirmed` |
| `.planning/phases/060-Gaps-Closing/060-TZ2.md` and `060-TZ2.md` | Drive Workstream C and Workstream D closure across `060-01`, `060-06`, `060-07`, `060-08`, `060-09`, and `060-11`. | `confirmed` |
| `.planning/phases/060-Gaps-Closing/060-z00z-verification-report.md` and `060-z00z-verification-report.md` | Supply the docs, supply-chain, adversarial, and top-slowest verification closure lanes reflected in `060-01`, `060-06`, `060-09`, and `060-11`. | `confirmed` |
| `crates/z00z_core/src/genesis/README.md`, `crates/z00z_core/README.md`, `crates/z00z_core/src/assets/assets_config.yaml`, and the `assets_config.yaml` versus `genesis_config*.yaml` authority split | Bound into `060-02` and `060-04` so bootstrap authority drift is closed against the real repository wording and YAML surfaces. | `confirmed` |
| `crates/z00z_core/src/assets/mod.rs` | Bound into `060-02` and `060-04` so bootstrap-authority wording and shim or owner demotion rules stay attached to the live module-doc surface instead of drifting into README-only prose. | `confirmed` |
| `crates/z00z_utils/src/codec/canonical_json.rs`, `crates/z00z_core/src/genesis/genesis_policies.rs`, and `crates/z00z_storage/src/settlement/leaf.rs` | Bound into `060-04` as explicit anti-drift anchors so Phase 060 does not "fix" owner cleanup by re-homing the already shared canonical JSON helper or collapsing the storage-owned leaf boundary. | `confirmed` |
| Rejected symmetry-only filenames `actions_config.yaml`, `policies_config.yaml`, `vouchers_config.yaml`, and `rights_config.yaml` | Bound into `060-04` and this context as explicit non-goals rather than being left as implied style advice. | `confirmed` |
| `crates/z00z_core/src/genesis/**`, `crates/z00z_core/src/rights/mod.rs`, `rights/mod.rs`, and adjacent bootstrap owners | Bound into Workstream A plans so `A1..A5` stay tied to real code seams. | `confirmed` |
| `crates/z00z_core/src/assets/right_config.rs` | Bound into `060-04` as the compatibility-shim and owner-move seam so the packet closes the real `rights` config migration path instead of speaking only at the namespace level. | `confirmed` |
| `crates/z00z_core/src/assets/assets_config.yaml`, `crates/z00z_core/src/rights/mod.rs`, `rights/mod.rs`, `crates/z00z_core/src/genesis/genesis_rights.rs`, and `genesis_rights.rs` | Bound into `060-04`, `060-07`, and `060-08` so rights anchors and profile grammar stay attached to live structures. | `confirmed` |
| `crates/z00z_rollup_node/src/config.rs`, `crates/z00z_rollup_node/tests/test_hjmt_process.rs`, and `crates/z00z_simulator/tests/test_scenario_settlement.rs` | Bound into `060-03`, `060-05`, and `060-12` so the HJMT process-model, topology-mutation, and settlement-scenario contracts stay tied to the live runtime seams rather than to abstract topology prose. | `confirmed` |
| `docs/tech-papers/done/Z00Z-HJMT-Upgrade.md` and HJMT runtime fixtures | Bound into `060-03` and `060-10` so topology and benchmark language stay tied to the live runtime story. | `confirmed` |
| `crates/z00z_storage/benches/settlement_shard.rs` and `crates/z00z_storage/scripts/run_storage_settlement_bench.py` | Bound into `060-10` as the canonical HJMT measurement-lane and throughput evidence homes so benchmark-lane separation stays executable and repository-owned. | `confirmed` |
| `reports/z00z-verification-orchestrator-20260618-170025/profiling/hjmt-summary.json` and `/usr/bin/time -v` measurement-lane evidence | Bound into `060-10` so the TODO-level throughput warning and measurement-lane split are reflected literally in the HJMT rerun plan. | `confirmed` |
| `reports/z00z-verification-orchestrator-20260618-170025/logs/l0-docs.log`, `reports/z00z-verification-orchestrator-20260618-170025/logs/l4-supply-chain.log`, and `reports/z00z-verification-orchestrator-20260618-170025/logs/l4-adversarial-review.log` | Bound into `060-01`, `060-06`, and `060-09` so the TODO-level gate baselines (`97` docs errors, `776` vet exemptions, `392` findings with `11` highs) remain visible in the execution packet. | `confirmed` |
| `reports/z00z-verification-orchestrator-20260618-170025/supply-chain/supply-chain-project.md` and `reports/z00z-verification-orchestrator-20260618-170025/supply-chain/reviewed-advisories.toml` | Bound into `060-06` as the exact project-owned advisory and review-record anchors so supply-chain closure stays tied to the current repository evidence instead of only to generic directory wording. | `confirmed` |
| Supply-chain report artifacts under `reports/z00z-verification-orchestrator-20260618-170025/supply-chain/` | Bound into `060-06` as evidence inputs, while repository-owned review records remain the required authority target. | `confirmed` |
| `reports/z00z-verification-orchestrator-20260618-170025/security/adversarial-review.md` | Bound into `060-09` as the exact high-finding closure ledger input so the packet stays attached to the live adversarial row set and its top-three priority order. | `confirmed` |
| Adversarial report artifacts under `reports/z00z-verification-orchestrator-20260618-170025/security/` | Bound into `060-09` so all `11` highs remain normalized, owned, and closed by artifacts. | `confirmed` |
| `reports/z00z-verification-orchestrator-20260618-170025/profiling/summary.json` and `reports/z00z-verification-orchestrator-20260618-170025/profiling/resources-summary.json` | Bound into `060-11` as the exact wall-time, CPU, RSS, and top-slowest baseline packet so verification-pipeline optimization work stays measured against the named source evidence. | `confirmed` |
| Profiling report artifacts under `reports/z00z-verification-orchestrator-20260618-170025/profiling/` | Bound into `060-11` so the top `5` slowest verification events become explicit optimization work rather than loose advice. | `confirmed` |
| `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md` and `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs` | Bound into `060-07`, `060-08`, `060-14`, and `060-15` as the exact one-plane wallet object authority and quarantine semantics anchors so profile, lock, issue/create, and delegation work stays tied to the live wallet surface. | `confirmed` |
| `docs/tech-papers/TODO-Wallet-idea.md`, `docs/Z00Z-Tokenomics-Incentives-Whitepaper.md`, `docs/Z00Z-Litepaper.md`, and `docs/Z00Z-UseCases-Whitepaper.md` | Bound into `060-07` and `060-08` so wallet profile ids, lock semantics, and product terms stay sourced instead of invented. | `confirmed` |
| `.planning/phases/060-Gaps-Closing/🔐 CRYPTOGRAPHIC BALANCE VALIDATION TEST.md` | Bound into `060-13` as the audit trigger that distinguishes claim-flow cryptographic balance proof from the live ordinary prepared-tx and voucher conservation seams that still needed explicit coverage. | `confirmed` |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`, `crates/z00z_wallets/src/tx/tx_assembler.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`, `crates/z00z_storage/src/settlement/tx_plan_types.rs`, and `crates/z00z_storage/src/settlement/object_package_contract.rs` | Bound into `060-13` as the canonical regular-tx balance and voucher conservation seams, with rights explicitly excluded from value-bearing balance checks. | `confirmed` |
| `docs/Z00Z-Main-Whitepaper.md`, `docs/Z00Z-Litepaper.md`, `docs/tech-papers/done/Z00Z-HJMT-Design.md`, `crates/z00z_storage/src/settlement/fee_envelope.rs`, `crates/z00z_storage/tests/test_fee_envelope.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs`, and `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs` | Bound into `060-13` as the split-fee corpus and typed-object `FeeEnvelope` evidence set so the reopen strengthens early reject coverage without drifting into universal fee unification or a second fee-support layer. | `confirmed` |
| `.planning/phases/000/059-Core-Upgrade/059-TODO.md`, `crates/z00z_storage/src/settlement/record.rs`, `crates/z00z_storage/src/settlement/tx_plan_types.rs`, and `crates/z00z_storage/tests/test_store_api.rs` | Bound into `060-15` as the refund-target and restricted-source semantic authority so reject or refund outputs stay attached to the declared voucher route instead of drifting into unrelated clean outputs. | `confirmed` |
| `crates/z00z_wallets/src/adapters/rpc/types/object.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/object.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`, `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_support.rs`, `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs`, and `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs` | Bound into `060-15` as the one-plane wallet object package construction corpus so voucher issue and right create widen the existing request shape instead of creating a second API or weakening the cash-only `wallet.asset.*` boundary. | `confirmed` |
| `crates/z00z_runtime/aggregators/src/types.rs`, `crates/z00z_runtime/validators/src/verdict.rs`, `crates/z00z_runtime/validators/src/engine.rs`, `crates/z00z_runtime/validators/src/checkpoint.rs`, `crates/z00z_runtime/watchers/src/alerts.rs`, `crates/z00z_runtime/watchers/src/engine.rs`, and `crates/z00z_runtime/watchers/src/publication.rs` | Bound into `060-15` as the live incomplete-state and alert-surface corpus so retry-pending, missing-artifact, and publication-gap cases stay on the existing publication binding path. | `confirmed` |
| `crates/z00z_storage/src/settlement/record.rs`, `crates/z00z_core/src/rights/right_policy.rs`, `crates/z00z_storage/tests/test_right_leaf.rs`, and `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs` | Bound into `060-15` as the monotonic right-delegation corpus so `delegate_right` closes the authority-widening seam on the existing transfer path instead of via a second hierarchy or API. | `confirmed` |
| `.planning/phases/060-Gaps-Closing/HJMT-RAID -Sharding.md` | Bound into `060-12` as the audit trigger for same-lineage failover, route-bound journal, and exact route-table-driven publication coverage without creating a parallel HJMT authority. | `confirmed` |
| `docs/tech-papers/done/Z00Z-HJMT-Design.md` | Bound into `060-12` as the semantic authority for journal-backed publication, fail-closed recovery, and storage-owned truth boundaries. | `confirmed` |
| `docs/tech-papers/done/Z00Z-HJMT-Gaps.md` | Bound into `060-12` as the gap ledger that the reopened HJMT closeout must stop overstating. | `confirmed` |
| `docs/tech-papers/done/Z00Z-HJMT-Threat-Model.md` | Bound into `060-12` as the fail-closed replay and same-lineage threat basis for route-bound durable recovery. | `confirmed` |
| `docs/tech-papers/done/Z00Z-HJMT-Upgrade.md` | Bound into `060-03`, `060-10`, `060-12`, `060-14`, and `060-15` so topology, measurement, root generation, shard publication, route-table acceptance, and publication-liveness wording stay tied to the same upgrade contract. | `confirmed` |

## 🚫 What Phase 060 Must Not Do

The context explicitly carries forward the TODO-level non-goals and rejected
remediations:

- do not add symmetric `*_config.yaml` files just for visual symmetry;
- do not create a second canonical JSON helper or move ownership away from `z00z_utils::codec` without repository-backed need;
- do not rename or split `genesis_policies.rs` on the theory that it is
  asset-only;
- do not collapse `AssetLeaf` and storage-owned terminal leaf boundaries;
- do not treat `1 shard = 1 process` as already-required behavior;
- do not promote `shard_process` to production default without A/B evidence;
- do not treat proof-size wins or verifier timings as end-to-end TPS evidence;
- do not claim scheduler parallelism as operationally proven without live
  evidence;
- do not "fix" docs by adding an artificial mdBook layer unless the repository
  explicitly opts into it;
- do not reduce verification runtime by weakening security or formal gates;
- do not create a second wallet database or new primitive leaves for MVP
  profiles;
- do not redesign `.wlt` persistence from scratch;
- do not rely on wallet UI soft-lock as a substitute for protocol-visible lock
  semantics.
- do not start with fully slashable self-custody staking as v1.

</domain>

<decisions>
## 🧭 Implementation Decisions

### Canonical authority and sync rules

- **D-01:** `060-TODO.md` is the canonical Phase 060 planning and execution
  authority.
- **D-02:** `060-CONTEXT.md` is subordinate to `060-TODO.md` and must not
  compete with it.
- **D-03:** The source intake fixed by `060-TODO.md` remains `14` explicit
  source markers across `060-TZ1.md`, `060-TZ2.md`, and
  `060-z00z-verification-report.md`.
- **D-04:** The canonical workstream task surface is fixed to `A1..A5`,
  `B1..B6`, `C1..C4`, and `D1..D5`. Planning may add execution detail, but
  must not drop, merge away, or reinterpret those task meanings.
- **D-05:** If an implementation wave discovers a new design constraint, update
  `060-TODO.md` first and synchronize this context second.

### Anti-duplication and anti-drift rules

- **D-06:** Do not duplicate the existing codebase or its logic.
- **D-07:** Do not introduce a parallel planning, storage, config, wallet, or
  verification layer.
- **D-08:** Prevent codebase concept drift by tying all planning and later
  implementation to the live repository seams named in `060-TODO.md`.
- **D-09:** TODO-level false gaps and rejected remediations are binding
  non-goals, not optional suggestions.

### Workstream execution rules

- **D-10:** Workstream A must close authority drift by clarifying one canonical
  bootstrap owner, completing the `rights` owner move, and demoting shim-first
  imports without inventing new authorities.
- **D-11:** Workstream B must keep `aggregator_owned` as the production default
  unless the explicit B6 A/B gate proves `shard_process` is neutral or better
  on durable throughput and operational recovery.
- **D-12:** Workstream C must keep the verification closure order encoded in
  `060-TODO.md`: `C1 -> C2 -> C3 -> C4`.
- **D-13:** Workstream D must preserve the current one-plane wallet authority
  and extend it through typed profile semantics rather than storage redesign.

### Acceptance and verification transfer rules

- **D-14:** TODO-level acceptance criteria remain mandatory execution gates.
- **D-15:** TODO-level verification anchors remain mandatory verification
  inputs; they are not advisory notes.
- **D-16:** TODO-level risk controls and pitfalls remain binding phase guards.
- **D-17:** TODO-level global execution order remains the canonical wave order
  unless `060-TODO.md` itself changes first.
- **D-18:** `060-12-PLAN.md` is a subordinate HJMT audit-reopen slice. It adds
  no new canonical task ids, must reuse the existing Workstream B contracts,
  and must trigger refreshed `060-10` and `060-11` evidence if it lands after
  those packets were first prepared.
- **D-19:** `060-13-PLAN.md` is a subordinate wallet/object audit-reopen
  slice. It adds no new canonical task ids, must reuse the existing Workstream
  D contracts, and must keep regular tx balance validation, voucher
  conservation validation, and typed-object `FeeEnvelope` reject validation on
  their current canonical code paths without widening rights into
  value-bearing semantics or collapsing the shipped native fee-output lane. If
  it lands after `060-11` was first prepared, it must also trigger refreshed
  targeted wallet/object, validator, watcher, and simulator closeout evidence
  plus any final Phase 060 closure memo that still cites the older tree.
- **D-20:** `060-14-PLAN.md` is a subordinate refund/source and runtime
  audit-reopen slice. It adds no new canonical task ids, must reuse the
  existing Workstream D and HJMT publication contracts, must keep
  `wallet.object.preview_package` plus `wallet.object.build_package` as the
  one typed-object package surface, and must trigger refreshed targeted
  wallet/object, validator, watcher, and simulator closeout evidence plus any
  final Phase 060 closure memo if it lands after `060-11` was first prepared.
- **D-21:** `060-15-PLAN.md` is a subordinate narrowed MVP-successor slice for
  the still-open subset revalidated after the repeated Phase 059 vs codebase
  audit. It adds no new canonical task ids, must reuse the existing Workstream
  D and HJMT publication contracts, must keep preview/build package
  construction on the one existing wallet object surface, must keep
  `delegate_right` on the current `RightAction::Transfer` path, must not run
  in parallel as a duplicate execution packet beside `060-14`, and must
  trigger refreshed targeted wallet/object, validator, watcher, and simulator
  closeout evidence plus any final Phase 060 closure memo if it lands after
  `060-11` was first prepared.

</decisions>

<specifics>
## 📚 Cross-Workstream Control Transfer

This table confirms that the non-task controls from `060-TODO.md` are also
carried into this context.

| TODO control surface | Context transfer requirement | Transfer status |
| --- | --- | --- |
| `Source TODO Inventory` | Keep the exact source-marker mapping and the four normalized workstreams visible in context. | `confirmed` |
| `Inventory self-audit` | Preserve the `7 / 3 / 4 = 14` source-marker accounting and treat it as a coverage invariant. | `confirmed` |
| `A actual gap` | Treat authority drift, unfinished `rights` ownership, and compatibility-umbrella residue as the real A problems. | `confirmed` |
| `A false gaps` | Carry the explicit rejection of symmetry-only configs, unsupported canonical-JSON re-homing, and leaf-boundary collapse. | `confirmed` |
| `B actual gap` | Carry the five live HJMT gaps: process-model mismatch, missing config mapping, missing removal coverage, missing `3A7S -> 2A7S -> 5A7S`, and mixed measurement lanes. | `confirmed` |
| `B false gaps` | Keep current one-aggregator-many-shards behavior as valid default contract until evidence says otherwise. | `confirmed` |
| `C actual gaps` | Carry docs, supply-chain, adversarial, and verification-pipeline performance closure as four distinct closure lanes. | `confirmed` |
| `C false wins` | Keep the prohibition on speedups that weaken security, fuzz, constant-time, semver, or formal gates. | `confirmed` |
| `D actual gap` | Carry the product or policy matrix gap rather than inventing a new wallet persistence design problem. | `confirmed` |
| `D false gaps` | Preserve the rejection of second wallet DBs, new primitive leaves, UI-only locks, and slashable-v1 scope creep. | `confirmed` |
| `Risk Controls And Pitfalls` | Treat TODO risk table as mandatory pre-implementation guardrail set. | `confirmed` |
| `Global Execution Order` | Preserve Wave 1 through Wave 4 ordering as the canonical implementation sequence. | `confirmed` |
| `Doublechecked Evidence Basis` | Keep the source and repository evidence set named in TODO as the minimum trust base for Phase 060. | `confirmed` |
| `HJMT docs-corpus reopen` | Carry the audit-driven `060-12` closeout as a subordinate HJMT packet instead of letting the plan packet claim completed storage/publication closure that the live code still lacks. | `confirmed` |
| `Wallet/object reopen` | Carry the audit-driven `060-13` closeout as a subordinate wallet/object packet instead of treating the claim-flow cryptographic balance example as full prepared-tx and voucher-conservation coverage or treating storage-only `FeeEnvelope` validation as full typed-object reject coverage. | `confirmed` |
| `Refund/source, runtime, and delegation reopen` | Carry the audit-driven `060-14` broad reopen plus the narrowed `060-15` successor packet as subordinate wallet/object plus runtime closeout slices instead of treating late Phase 059 refund/source semantics, `Incomplete`/`ValidatorIncomplete` runtime surfaces, or monotonic right delegation as already closed by the earlier packet. | `confirmed` |

## 🔍 Verification And Performance Mirror

The context also confirms the TODO-level completion, testing, and performance
checks that later implementation must preserve.

| Workstream | Mandatory completion checks transferred from `060-TODO.md` |
| --- | --- |
| A | Preserve the `z00z_core` genesis, policy, voucher, and rights test anchors named in the TODO verification section. |
| B | Preserve `z00z_rollup_node` topology tests, config tests for `aggregator_owned` and `shard_process`, simulator settlement coverage, and the B6 A/B metric gate on durable throughput, journal sync, latency, RSS, CPU, restart, and failover recovery. |
| C | Preserve strict docs gate rerun, strict supply-chain gate rerun, adversarial-review rerun, and report-consistency verification for `high_risk_count` versus rendered high-finding lists. |
| D | Preserve typed-wallet, validator, watcher, and simulator coverage anchors, keep fail-closed profile testing explicit for lock, replay, quarantine, wrong-family rejection, refund/source binding, runtime incomplete surfacing, and monotonic right delegation, and refresh that targeted closeout evidence if the subordinate `060-13`, `060-14`, or `060-15` reopen lands after `060-11` was first prepared. |

## 📋 Current Live Transfer Status

- `060-TODO.md` remains the only canonical Phase 060 task authority.
- `060-CONTEXT.md` now contains an explicit transfer ledger for every
  implementation-facing task and every cross-workstream control surface that
  must remain in scope before implementation begins.
- `060-12-PLAN.md` is now recorded as a subordinate HJMT reopen slice so the
  packet no longer treats `060-01..11` as the full and final HJMT closeout.
- `060-13-PLAN.md` is now recorded as a subordinate wallet/object reopen slice
  so the packet no longer treats claim-flow balance proof as full coverage for
  the ordinary prepared-tx and voucher conservation paths or treats
  storage-commit-only `FeeEnvelope` validation as full typed-object reject
  coverage.
- `060-14-PLAN.md` is now recorded as a subordinate refund/source and runtime
  reopen slice so the packet no longer treats refund-target or restricted-
  source binding and `Incomplete` or `ValidatorIncomplete` runtime semantics
  as already closed by the earlier Phase 059 or `060-12`/`060-13` slices.
- `060-15-PLAN.md` is now recorded as the narrowed MVP-successor packet for
  the still-open subset of the `060-14` reopen, so refund/source binding,
  one-plane voucher-issue or right-create construction, real incomplete
  runtime emission, and monotonic right delegation are tracked on one
  non-duplicated execution packet instead of drifting into a parallel layer.
- No implementation work is authorized to treat this context as a replacement
  for `060-TODO.md`.
- No task from `A1..D5` may be considered optional, already implied, or safely
  omitted without first changing `060-TODO.md`.

## 🧪 Second Doublecheck Result

The second doublecheck was run against `060-TODO.md` with a workspace-first
audit of this context file, the numbered `060-01..15-PLAN.md` packet,
`060-TEST-SPEC.md`, and `060-TESTS-TASKS.md`.

| Doublecheck dimension | Verification result |
| --- | --- |
| Canonical task transfer | `20 / 20` canonical task ids from `A1..A5`, `B1..B6`, `C1..C4`, and `D1..D5` are represented in the transfer ledger. |
| Plan packet transfer | `20 / 20` canonical task ids are still routed into `11` primary numbered plans with no uncovered task id and no duplicate primary owner slice; `060-12`, `060-13`, `060-14`, and `060-15` are recorded separately as subordinate HJMT, wallet/object, runtime, and narrowed MVP-successor reopen slices with no new canonical task ids. |
| Test packet transfer | `20 / 20` canonical task ids are routed into `060-S01..060-S11` and into ordered test-task steps with explicit pass signals; the subordinate HJMT, wallet/object, and runtime reopens are routed separately into `060-S12`, `060-S13`, and `060-S14`, with `060-15` reusing the `060-S14` semantic/runtime bucket instead of minting a new canonical task id or scenario family. |
| Cross-workstream controls | `14 / 14` control surfaces from the TODO control and audit sections are mirrored in this context. |
| Source corpus transfer | The canonical three-source corpus, the supplemental HJMT plus wallet/object plus runtime audit corpus, and the named live code anchors are mapped into context and the relevant numbered plans. |
| Performance closure coverage | `C4` is carried as the top `5` slowest verification-pipeline optimization lane and remains constrained by the "no weaker verification semantics" rule. |
| HJMT production-default guard | `aggregator_owned` remains the carried production default; `shard_process` remains opt-in pending the explicit B6 A/B gate. |
| HJMT reopen truthfulness | The context no longer claims full HJMT storage/publication closure through `060-11` alone; the reopened `060-12` seams are explicitly routed and constrained to reuse existing HJMT authority. |
| Wallet/object reopen truthfulness | The context no longer treats the claim-flow balance example as full prepared-tx or voucher-conservation coverage, and it no longer treats storage-commit-only `FeeEnvelope` validation as full typed-object reject coverage; the reopened `060-13` seams are explicitly routed and constrained to reuse existing wallet/object authority. |
| Refund/source, runtime, and delegation reopen truthfulness | The context no longer treats refund-target or restricted-source routing, `Incomplete` or `ValidatorIncomplete` runtime semantics, or monotonic right delegation as already covered by the earlier packet; the reopened `060-14` and narrowed `060-15` seams are explicitly routed and constrained to reuse existing wallet/object, right-transfer, and publication authority. |
| Wallet/object closeout refresh | If the subordinate `060-13`, `060-14`, or `060-15` reopen lands after `060-11` was first prepared, the final targeted wallet/object, validator, watcher, and simulator reruns plus any closure memo must be refreshed on the post-`060-13`, post-`060-14`, or post-`060-15` tree. |
| Proposed-vs-existing check | Proposed homes and proposed profile ids are explicitly labeled as proposed where the exact live identifier or repository-owned store is not yet verified. |
| Literal guard normalization | The task packet now spells out the exact operator-facing and fail-closed guard phrases for `execution.shard_mapping: "aggregator_owned"`, wallet.asset.* remains cash-only, durable quarantine, and approved post-expiry unlock handling. |
| Verification-command fit | Integration-test verify anchors that target named test binaries use `--test` where file-name-only bare filters could skip the intended Phase 060 coverage. |
| Execution order | TODO wave order and the internal `C1 -> C2 -> C3 -> C4` order are both preserved. |
| Anti-drift constraints | The context carries the "no duplicate codebase logic", "no parallel layer", and "no concept drift" rules without introducing a competing authority. |
| Parallel-spec check | No new implementation scope, architecture, or task semantics were introduced here beyond transfer, compression, and audit framing of the canonical TODO. |

## 📐 Coverage Dimensions Confirmed

| Dimension that must survive transfer | How this context preserves it | Status |
| --- | --- | --- |
| What to do | `Task Transfer Confirmation` mirrors every canonical task with implementation meaning and required completion or confirmation outcome. | `confirmed` |
| In what order | `Implementation Decisions`, `Verification And Performance Mirror`, and `Second Doublecheck Result` preserve both global wave order and the internal Workstream C order. | `confirmed` |
| How to test | `Verification And Performance Mirror` carries the TODO verification anchors by workstream and keeps them mandatory. | `confirmed` |
| How to check performance | Workstream B preserves the A/B topology gate; Workstream C preserves the top `5` slowest verification-pipeline runtime lane. | `confirmed` |
| Risks and mitigations | `What Phase 060 Must Not Do`, `Cross-Workstream Control Transfer`, and `D-16` preserve TODO risk controls and rejected remediations. | `confirmed` |
| Anti-duplication and anti-drift | `D-06` through `D-09` preserve the no-duplication, no-parallel-layer, and live-codebase-seam constraints. | `confirmed` |
| Supplemental HJMT and wallet/object audit corpus | `Supplemental HJMT Reopen Coverage`, `Supplemental Wallet/Object Reopen Coverage`, `Supplemental Object/Runtime Reopen Coverage`, `Source Corpus Transfer Matrix`, `D-18`, `D-19`, `D-20`, and `D-21` preserve the reopened closeout slices without widening the canonical task surface. | `confirmed` |

## ✅ Second Doublecheck Target

The second doublecheck for Phase 060 is defined as:

1. verify that every canonical task id from `060-TODO.md` appears in this
   context transfer ledger and in the numbered plan packet;
2. verify that the non-goal and anti-drift rules from `060-TODO.md` appear in
   this context without contradiction;
3. verify that acceptance, verification, performance, and wave-order controls
   remain mirrored here and across the numbered plans;
4. verify that the source corpus, supplemental HJMT and wallet/object audit
   corpus, and live-code anchors remain mapped into the relevant plans without
   concept drift;
5. verify that the reopened `060-12`, `060-13`, `060-14`, and `060-15`
   slices do not mint new canonical task ids or second authority paths;
6. verify that this context does not create a second semantic layer or a
   planner-invented replacement for the canonical TODO.

## 🔗 Canonical References

- `.planning/phases/060-Gaps-Closing/060-TODO.md`
- `.planning/phases/060-Gaps-Closing/060-TZ1.md`
- `.planning/phases/060-Gaps-Closing/060-TZ2.md`
- `.planning/phases/060-Gaps-Closing/060-z00z-verification-report.md`
- `.planning/phases/060-Gaps-Closing/060-15-PLAN.md`
- `.planning/phases/060-Gaps-Closing/HJMT-RAID -Sharding.md`
- `docs/tech-papers/done/Z00Z-HJMT-Design.md`
- `docs/tech-papers/done/Z00Z-HJMT-Gaps.md`
- `docs/tech-papers/done/Z00Z-HJMT-Threat-Model.md`
- `docs/tech-papers/done/Z00Z-HJMT-Upgrade.md`
- `.github/copilot-instructions.md`
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`

</specifics>
