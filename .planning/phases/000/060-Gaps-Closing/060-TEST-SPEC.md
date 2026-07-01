---
phase: 060-Gaps-Closing
artifact: test-spec
status: planning-ready
source: 060-TODO.md, 060-CONTEXT.md, 060-TZ1.md, 060-TZ2.md, 060-z00z-verification-report.md, 060-01-PLAN.md..060-15-PLAN.md
updated: 2026-06-21
---

# Phase 060 Test Specification

## Purpose

This document turns the Phase 060 planning packet into an executable test
contract. The canonical task inventory remains `060-TODO.md`; `060-CONTEXT.md`
remains the anti-drift transfer mirror; the numbered `060-01` through `060-15`
plans remain the execution packet. This file defines what another engineer or
agent must prove, where to prove it, which failure paths must stay explicit,
and which measurements or artifacts make a scenario pass.

Phase 060 does not add a new product surface. It closes gaps on top of the
current codebase. The tests therefore must reuse existing truthful homes:
genesis and core tests, HJMT topology and simulator coverage, wallet typed
object coverage, supply-chain and adversarial gates, and the current
verification pipeline. No browser automation is required. In this repository,
E2E means live simulator, runtime, CLI or gate execution, filesystem or
artifact roundtrip, and report-consistency proof.

The entire packet is constrained by one rule: do not create a parallel layer.
Do not duplicate wallet authority, HJMT authority, docs topology, benchmark
archives, supply-chain review stores, or closeout ledgers.

## Classification Summary

| Class | Meaning in Phase 060 | Representative homes | Use when |
| --- | --- | --- | --- |
| TDD / unit | Pure Rust contract, config validation, parser, or helper behavior | `crates/z00z_core/src/assets/test_policy_descriptor.rs`, `crates/z00z_core/src/assets/test_voucher_config.rs`, `crates/z00z_wallets/src/db/redb_wallet_store/test_mod.rs`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`, `crates/z00z_storage/tests/test_checkpoint_root_binding.rs` | One seam can be proven without a live simulator run or external artifact packet. |
| Integration / scenario | Multi-file runtime behavior across service, runtime, or simulator boundaries | `crates/z00z_rollup_node/tests/test_hjmt_process.rs`, `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs`, wallet RPC and backup tests | The behavior must be proven through the real stack or a realistic runtime transition. |
| Diagnostics / evidence | Shell gates, report consistency, run-root artifacts, profiling diffs, or repo-owned review ledgers | `.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh`, `.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh`, `cargo vet check`, `run-security-brainstorm.py`, HJMT run-root artifacts, verification before/after reports | The truth is in the artifact or gate output rather than a single Rust assertion. |
| Skip | Planning-only or forbidden seams | `060-TODO.md`, `060-CONTEXT.md`, report-local snapshots as authority, `crates/z00z_crypto/tari/**` | There is no honest runtime assertion to add in Phase 060, or the surface is explicitly read-only. |

## Ordered Execution Packet

Phase 060 tests must preserve the same order as the planning packet:

1. `C1` docs posture and `ZINV` traceability.
2. `A1` bootstrap authority freeze.
3. `B1` and `B2` live HJMT topology and opt-in shard mapping.
4. `A2` through `A5` rights-owner and fixture cleanup.
5. `B3` and `B4` HJMT decommission and `3A7S -> 2A7S -> 5A7S`.
6. `C2` supply-chain review records and vet trust.
7. `D1`, `D2`, and `D5` wallet profile catalog and one-plane semantics.
8. `D3` and `D4` `validator_mandate_lock_v1` fail-closed behavior.
9. `C3` adversarial high-finding closure.
10. `B5` and `B6` measurement-lane separation and A/B rerun packet.
11. `C4` top-slowest verification-pipeline optimization and final reruns.
12. `060-12` audit-driven HJMT core-storage closure and refreshed `060-S10`
    or `060-S11` evidence if that reopen lands after the earlier packets.
13. `060-13` audit-driven prepared-tx balance, voucher conservation, and
    typed-object `FeeEnvelope` coverage closure so the packet does not
    overclaim wallet/object reject-path proof from the claim-flow example
    alone or blur the shipped native-fee split.
14. `060-14` audit-driven refund-target/source binding, one-plane object
    issue-or-create construction, and real validator-or-watcher incomplete
    coverage closure so the packet does not overclaim Phase 059 object-model
    and publication-runtime completeness.
15. `060-15` narrowed MVP-successor closeout for the still-open subset from
    the repeated Phase 059 vs codebase audit: reserve-aware refund/source
    exactness, truthful one-plane issue/create construction, real incomplete
    publication states, and monotonic `delegate_right` enforcement. This slice
    must reuse `060-S14` rather than minting `060-S15`, and it supersedes the
    overlapping `060-14` execution scope for this exact MVP subset instead of
    asking the executor to land the same implementation twice.

Every `auto` slice validates in this order:

1. `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
2. slice-owned narrow tests or gates
3. `cargo test --release` when the slice changes Rust, tests, public APIs, simulator behavior, serialization, or verification scripts
4. `/GSD-Review-Tasks-Execution` in YOLO mode at least 3 times and until 2 consecutive runs show no significant issues
5. `/z00z-git-versioning` if a commit is needed

## Required Invariants

| Invariant | How Phase 060 must prove it |
| --- | --- |
| `z00z_core::genesis` is the only canonical bootstrap authority | Docs, module comments, fixtures, and genesis tests all point to `GenesisConfig` as the single bootstrap story; `assets_config.yaml` is demoted to registry, fixture, example, or compatibility data. |
| Rights config has one semantic owner | `RightClassConfig` and `RightsConfigEntry` resolve under `rights/`; any surviving `assets/` shim is compatibility-only and guarded against owner drift. |
| No dual-authority YAML layer is reintroduced | No `actions_config.yaml`, `policies_config.yaml`, `vouchers_config.yaml`, or `rights_config.yaml` appears; `assets_config.yaml` stops implying bootstrap truth for rights. |
| Live HJMT default remains many shards per aggregator process | `aggregator_owned` stays default; baseline runtime evidence still shows `5` processes for `7` shards on `SIM-5A7S`. |
| Operator-facing mapping default stays explicit | One aggregator equals one OS process by default, and the canonical YAML default remains `execution.shard_mapping: "aggregator_owned"` until the explicit B6 A/B gate says otherwise. |
| `shard_process` is opt-in and fail-closed | Invalid or mixed mapping rejects before runtime; multi-primary owners reject; mapping selection is explicit in config and artifacts. |
| Same-lineage failover stays process-lawful | Decommission, fail-down, and fail-up scenarios preserve lineage, clean stale owner or standby references, and reject split-brain paths. |
| Throughput claims stay bound to durable publication | Promotion claims are made only from `durable_root_published_tps` plus journal sync, latency, blocked time, RSS, CPU, restart, and failover recovery metrics. |
| Docs gate is honest and traceable | `Z00Z_L0_STRICT=1` passes without a fake mdBook layer, and `ZINV references` becomes non-zero on real security-critical docs. |
| Supply-chain closure is repository-owned | Project-owned advisories end as upgrades, removals, or reviewed exceptions in a repo-owned store; report-local snapshots never become the authority. |
| Adversarial highs are closed by artifact, not prose | All `11` highs are normalized into `7` project-owned and `4` protected-vendor items, each with one closure artifact and a final status. |
| Wallet remains one authority plane | `wallet.object.*` is the typed object authority; `wallet.asset.*` is cash-only projection; `.wlt` and `WalletExportPack` remain the only wallet-local authority surfaces. |
| Non-cash or non-available objects fail closed | Rights and vouchers do not surface as cash; `wallet.asset.*` remains cash-only; unknown-policy objects remain in durable quarantine; non-`Available` states stay quarantined. |
| `validator_mandate_lock_v1` is layered, not widening | The lock profile sits on live `validator_mandate`, uses the exact field grammar from the plans, rejects ordinary spend when active, and does not add new primitive leaves or slashable-v1 widening. |
| Regular prepared tx balance validation remains canonical and fail-closed | `TxAssemblerImpl::assemble(...)` rejects plaintext and commitment mismatch before proof/finalization, and no second balance checker or shadow tx path is introduced. |
| Voucher conservation stays on the typed object package contract | Partial redeem or similar voucher value-bearing flows reject conservation mismatch through `ObjectDeltaSetV1` plus the existing object-package reject mapping instead of a parallel validator layer. |
| Rights stay zero-value and outside asset/voucher value-balance scope | Cash tx paths continue rejecting right ids, voucher value checks stay on voucher/object paths, and rights do not gain pseudo-cash symmetry checks. |
| Regular cash fees and typed-object `FeeEnvelope` stay on their shipped two-track contracts | Ordinary cash tx continues proving fee correctness through arithmetic totals plus Pedersen-balanced native fee outputs, while typed object and rights fee support proves structural budget, support, binding, expiry, and replay semantics through `FeeEnvelope`; Phase 060 does not unify those lanes into a new fee plane. |
| Voucher reject and refund outputs stay bound to declared refund and source context | Reject or refund paths prove the created output matches `refund_target_commitment`, and restricted backing proves the output also stays bound to the declared consumed-asset or reserve context instead of stripping restrictions into unrelated clean value. |
| One object-package RPC surface covers existing-object actions plus source-context issue or create actions | `wallet.object.preview_package` and `wallet.object.build_package` remain the only typed-object build surfaces while widening to truthful source or create context for `VoucherAction::Issue` and `RightAction::Create`. |
| Validator and watcher completeness stays explicit on the current publication path | Retry-pending, missing-artifact, and publication-gap states surface as `Incomplete` or `ValidatorIncomplete` on the existing publication binding contract, while true mismatches stay rejected. |
| Right delegation stays monotonic on the shipped transfer contract | `wallet.object.delegate_right` stays on the existing `RightAction::Transfer` path, cannot widen any authority dimension that the live `RightLeaf` contract represents, and rejects or narrows unsupported attenuation claims instead of overclaiming a richer hierarchy. |
| Verification speedups cannot weaken verification meaning | C4 may reduce repeated work and share safe metadata or targets, but it cannot weaken docs, supply-chain, adversarial, semver, unsafe, fuzz, constant-time, or formal evidence. |
| Live settlement root cannot stay on a parallel monolithic authority after root generation 1 activates | Storage, reload, and recovery must export the committed root-of-shard-roots as the live `SettlementStateRoot`, while generation 0 remains historical compatibility only. |
| Durable recovery must prove shard-domain identity from storage-owned state | Restart and failover evidence must roundtrip `ShardId`, routing generation, route-table digest, and journal lineage from the durable journal and recovery export rather than from runtime memory only. |
| Public checkpoint acceptance must verify exact shard coverage against the committed route table | Missing, extra, duplicated, foreign, or wrongly carried shard leaves, plus activation/publication checkpoint drift, must reject in one shared storage-owned acceptance path. |

## Scenario Matrix

| Scenario ID | Plans / tasks | Class | Primary homes | What it proves |
| --- | --- | --- | --- | --- |
| `060-S01` | `060-01` / `C1` | Diagnostics / evidence | docs gate scripts, tracked Markdown files, `l0-docs.log` baseline | Strict docs mode matches live repo topology and `ZINV` anchors exist on real security claims. |
| `060-S02` | `060-02` / `A1` | Diagnostics + integration | core READMEs, settlement README, genesis fixture test | The repo says one bootstrap story and the fixtures still support it. |
| `060-S03` | `060-03` / `B1`, `B2` | Integration / scenario | HJMT process and topology tests, generated fixture helper, settlement scenario | Default process model is explicit and `shard_process` is opt-in only. |
| `060-S04` | `060-04` / `A2`, `A3`, `A4`, `A5` | TDD + integration | rights config tests, genesis tests, shim grep guard, fixture docs | Rights owner move, shim demotion, dual-authority YAML closure, and fixture sync close without storage-boundary drift. |
| `060-S05` | `060-05` / `B3`, `B4` | Integration / E2E | aggregator failover and migration tests, settlement scenario | Owner removal and `3A7S -> 2A7S -> 5A7S` remain lawful and lineage-bound. |
| `060-S06` | `060-06` / `C2` | Diagnostics / evidence | supply-chain gate scripts, the selected repository-owned advisory and vet-store files, and `cargo vet` | Repo-owned advisory decisions and vet trust replace report-local bootstrap signaling. |
| `060-S07` | `060-07` / `D1`, `D2`, `D5` | TDD + integration | wallet typed-object tests, RPC tests, backup tamper test, wallet guide | MVP profile catalog and one-plane wallet semantics are explicit and fail-closed. |
| `060-S08` | `060-08` / `D3`, `D4` | Integration / E2E | wallet send-path tests, validator and watcher tests, simulator object-flow tests | `validator_mandate_lock_v1` is protocol-visible, privacy-aware, and enforced across boundaries. |
| `060-S09` | `060-09` / `C3` | TDD + diagnostics | checkpoint, scheduler, request, stealth tests, adversarial rerun, consistency check | All high findings are owned, reproducible or disproved, and count-consistent. |
| `060-S10` | `060-10` / `B5`, `B6` | Diagnostics / E2E | bench-lane tests, bench runner, HJMT run-root artifacts | The `1 shard = 1 process` switch is measurable, fair, and not promoted by fake TPS wins. |
| `060-S11` | `060-11` / `C4` | Diagnostics / evidence | verify-fast, supply-chain, semver, unsafe, fuzz, constant-time scripts, final reruns | Top-slowest verification work is reduced without changing pass or fail semantics, and any later `060-S12`, `060-S13`, or `060-S14` reopen forces the carried closeout packet to be refreshed on the post-reopen tree. |
| `060-S12` | `060-12` / supplemental HJMT reopen | Integration + diagnostics | storage recovery and proof tests, validator or watcher publication tests, rollup preflight, settlement scenario | Live root-of-shard-roots, route-bound durable recovery, and exact route-table publication coverage close without creating a parallel HJMT authority. |
| `060-S13` | `060-13` / supplemental wallet-object balance reopen | TDD + integration | wallet `TxAssembler` tests, wallet tx RPC tests, wallet object-package RPC tests, storage delta-contract tests, storage `FeeEnvelope` tests, validator object-package tests, claim-flow comparison anchor | Regular prepared tx rejects plaintext and commitment mismatch, voucher value-bearing flows reject conservation mismatch, malformed typed-object `FeeEnvelope` contracts reject fail-closed, and the shipped native cash-fee lane stays distinct from `FeeEnvelope` support semantics. |
| `060-S14` | `060-14` plus narrowed `060-15` successor / supplemental refund-source, issue-create, incomplete-runtime, and monotonic-delegation reopen | TDD + integration | storage refund or reject tests, wallet object-package issue or create tests, validator publication-contract tests, watcher publication-contract tests, right delegation tests | Voucher refund or reject outputs stay bound to declared refund or source context, the existing object-package RPC covers voucher issue and right create without a parallel API, validator or watcher surfaces emit real incomplete states on the current publication path, and the shipped `delegate_right` path cannot widen authority. The narrowed `060-15` successor reuses this scenario family and supersedes the overlapping `060-14` implementation subset rather than duplicating it. |

## Detailed Scenario Contracts

### `060-S01` Docs Gate And `ZINV` Traceability

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh`, tracked docs from `060-01-PLAN.md` | Diagnostics / evidence | strict docs mode -> Markdown backlog closure -> `ZINV` scan | Missing mdBook does not fail strict mode unless the repo explicitly opts into mdBook; tracked docs are clean; `ZINV` anchors exist for bootstrap authority, HJMT lineage, wallet object-family boundaries, fail-closed right or voucher behavior, replay, and checkpoint claims | A dummy `book.toml`, a fake docs root, backlog suppression without fixing docs, or `ZINV references: 0` after the slice | `Z00Z_L0_STRICT=1` exits `0`; `rg -n "ZINV-" docs .github/skills/z00z-l0-spec-gate` returns real anchors; the slice closes the baseline `97` docs errors and no longer emits `ERROR: no mdBook book.toml found` as a strict failure |

### `060-S02` Canonical Bootstrap Authority Freeze

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_core/README.md`, `crates/z00z_core/src/genesis/README.md`, `crates/z00z_core/src/assets/mod.rs`, `crates/z00z_storage/src/settlement/README.md` | Diagnostics / docs guard | repository wording -> module wording -> fixture wording | All docs describe `z00z_core::genesis` and `GenesisConfig` as the only canonical bootstrap authority; `assets_config.yaml` is labeled registry, fixture, example, or compatibility data | Any surviving text presents `assets_config.yaml` as a co-equal bootstrap source or invents a new config home | Grep checks from `060-02-PLAN.md` find one authority story only; there is no new authority file, and wording is stable enough for later owner-path cleanup |
| `cargo test -p z00z_core --release --features deterministic-rng test_genesis_manifest_phase059_fixture -- --nocapture` | Integration | canonical docs -> canonical fixture | The phase 059 fixture still loads and matches the frozen authority story | The fixture or docs imply a different bootstrap owner than the READMEs | The fixture test stays green and does not require a second YAML authority story to pass |

### `060-S03` HJMT Process Model And Opt-In `shard_process`

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_rollup_node/tests/test_hjmt_process.rs`, `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`, `crates/z00z_rollup_node/tests/support/test_hjmt_home.rs` | Integration / scenario | YAML parse -> mapping selection -> topology validation -> generated fixture path | Default or explicit `aggregator_owned` is valid; the operator-facing default is literally `execution.shard_mapping: "aggregator_owned"`; explicit `shard_process` is valid only when requested and generated through `test_hjmt_home.rs` | Mixed mapping, invalid mapping, or multi-primary ownership under `shard_process`; checked-in second canonical runtime home | One aggregator equals one OS process by default; `aggregator_owned` remains the default; `shard_process` is opt-in only; invalid configs fail before runtime; mapping selection is recorded in evidence; no second checked-in canonical home appears |
| `cargo test -p z00z_simulator --release --test scenario_1 test_scenario_settlement -- --nocapture` | E2E / scenario | checked-in `SIM-5A7S` runtime -> baseline process evidence | The default runtime still reports `process_ids.len() == 5` while `shard_ids.len() == 7` | The live default silently behaves as one process per shard or emits ambiguous mapping evidence | The existing baseline remains truthful, which is required before any A/B benchmark or promotion claim |

### `060-S04` Rights Owner Move, Shim Demotion, And YAML Closure

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_core/src/rights/config.rs`, `crates/z00z_core/tests/assets/test_rights_config.rs`, `cargo test -p z00z_core --release --features deterministic-rng test_genesis_manifest_phase059_fixture -- --nocapture`, `cargo test -p z00z_core --release test_policy_descriptor -- --nocapture`, `cargo test -p z00z_core --release test_voucher_config -- --nocapture`, `cargo test -p z00z_core --release test_rights_config -- --nocapture` | TDD + integration | `A2` rights owner move -> `A3` shim demotion -> `A4` YAML closure -> `A5` fixture and doc sync -> regression guard | Rights-config types live under `rights/`; direct owner-module imports become canonical; fixtures and docs agree with the new owner boundary | `crate::assets::right_config` stays the semantic owner, new `*_config.yaml` files appear, or fixture loading still implies asset-registry bootstrap | All listed tests pass; `crates/z00z_core/src/rights/config.rs` remains the semantic owner, the slice lands one regression guard against rights-owner drift, and existing policy and voucher tests remain green on the new owner boundary |
| `rg -n "crate::assets::right_config|assets::right_config|compatibility" crates/z00z_core`, plus manual review of `crates/z00z_utils/src/codec/canonical_json.rs` and `crates/z00z_storage/src/settlement/leaf.rs` boundaries | Diagnostics / evidence | import graph -> compatibility-only shims -> owner anchors -> storage-owner boundary | Any remaining `assets::right_config` hit is explicitly marked compatibility-only; `crates/z00z_utils/src/codec/canonical_json.rs` remains the live canonical-json owner; `leaf.rs` remains storage-owned | Semantic-owner imports survive in normal code paths, the live `z00z_utils::codec::canonical_json.rs` owner is forked or relocated again, `genesis_policies.rs` is split, or storage imports `z00z_core::assets::AssetLeaf` as owner truth | Surviving `assets` hits are compatibility-only; no `actions_config.yaml`, `policies_config.yaml`, `vouchers_config.yaml`, or `rights_config.yaml` is introduced; the live canonical-json owner and storage-owned leaf boundaries remain intact |

### `060-S05` HJMT Decommission And `3A7S -> 2A7S -> 5A7S`

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`, `test_hjmt_migrate.rs`, `test_hjmt_failover_same_lineage.rs`, `test_hjmt_split_brain_fencing.rs` | Integration / scenario | multi-shard owner removal -> ownership redistribution -> standby cleanup -> same-lineage takeover | A decommissioned aggregator that owned multiple shards is fully removed and lawful standby owners take over on the same lineage | Owner, standby, or route ghosts remain; a split-brain path succeeds; lineage is silently rewritten | All seven shards remain owned; removed aggregators disappear from owner and standby tables; same-lineage failover law still fences split-brain cases |
| `crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs` | E2E / scenario | `3A7S` startup -> owner removal to `2A7S` -> re-expansion to `5A7S` -> publication continuity | Runtime evidence records `3A7S`, then `2A7S`, then `5A7S`, with correct route-generation progression and publication continuity | Re-expansion rewrites prior lineage, stale standby state lingers, or process counts do not match the staged scenario | The scenario proves fail-down and fail-up on one HJMT contract, not a second harness, and emits stage-by-stage runtime evidence the bench packet can later reuse |

### `060-S06` Supply-Chain Review Records And Vet Trust

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh` plus the selected repository-owned advisory and vet-store files | Diagnostics / evidence | strict L4 gate -> repo-owned advisory decisions -> repo-owned vet store | `bincode 2.0.1 / RUSTSEC-2025-0141`, `paste 1.0.15 / RUSTSEC-2024-0436`, `derivative 2.2.0 / RUSTSEC-2024-0388`, and `instant 0.1.13 / RUSTSEC-2024-0384` each end as remove, replace, or reviewed exception with explicit fields; vendor `bincode 1.3.3 / RUSTSEC-2025-0141` stays on wrapper or upstream track; if no existing equivalent is discovered first, the proposed default target set is `.reviews/reviewed-advisories.toml`, `.reviews/config.toml`, and `.reviews/audits.toml` | A project-owned advisory remains both unresolved and unreviewed, review records live only under a report root, the packet creates a second repository-owned advisory store, or protected-vendor code is edited | `Z00Z_L4_STRICT=1` passes honestly; the selected repo-owned store is canonical; `bincode 2.0.1` is treated as highest priority because of the `z00z_utils` fan-out, `paste 1.0.15` confirms current `p3-*` ancestry, and each exception records owner, reason, scope, sunset, ancestry, criticality, replacement cost, and temp-exception conditions |
| `cargo vet check`, compared with `reports/z00z-verification-orchestrator-20260618-170025/logs/l4-supply-chain.log` baseline | Diagnostics / evidence | repo-owned vet config -> vet run -> bootstrap-exemption delta | The vet store shrinks or justifies the `776` bootstrap exemptions baseline and the delta is recorded | Green vet output is claimed without showing whether the bootstrap-only trust state changed | The slice records the exemption delta against the baseline and does not call bootstrap green-ness mature trust without proof |

### `060-S07` Wallet MVP Profile Catalog And One-Plane Semantics

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/db/redb_wallet_store/test_mod.rs`, including `test_owned_object_tags_roundtrip`, `test_object_inventory_typed_projections`, and the reject-path inventory tests | TDD + integration | profile catalog -> typed object persistence -> quarantine semantics | The catalog publishes at least `fee_credit_v1`, `service_entitlement_v1`, `data_access_v1`, `agent_budget_v1`, `validator_mandate_lock_v1`, and `transferable_claim_v1`, each labeled live or proposed; unknown-policy objects remain in durable quarantine; unavailable states stay quarantined | Unknown-policy inventory becomes spendable, non-`Available` rows leak into cash balances, or proposed ids are presented as already-live code identifiers | Inventory tests prove typed object tags roundtrip, bad checksums and unquarantined unknown-policy states reject, and live/proposed distinctions stay explicit |
| `cargo test -p z00z_wallets --release --lib test_object_rpc_lists_typed_inventory -- --nocapture`, `cargo test -p z00z_wallets --release --lib test_asset_rpc_rejects_voucher_and_right_ids -- --nocapture`, `cargo test -p z00z_wallets --release test_verify_backup_detects_tamper -- --nocapture` | Integration / scenario | typed object authority -> cash-only asset projection -> wallet-local backup authority | Object RPC exposes typed inventory; asset RPC rejects right and voucher ids as cash; backup tamper still fails closed while `.wlt` and `WalletExportPack` remain authoritative | `wallet.asset.*` starts projecting rights or vouchers as value, a second wallet DB or export plane appears, or tx-history authority is folded into `.wlt` | The wallet retains one authority plane: `wallet.object.*` is typed authority, `wallet.asset.*` remains cash-only, unknown-policy objects remain in durable quarantine, and `.wlt` plus `WalletExportPack` are still the only wallet-local authority surfaces |

### `060-S08` `validator_mandate_lock_v1` And Fail-Closed Profile Coverage

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `cargo test -p z00z_wallets --release --lib test_tx_build_rejects_voucher -- --nocapture`, `cargo test -p z00z_wallets --release --lib test_tx_send_rejects_right -- --nocapture`, plus wallet send-path coverage in `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs` and `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md` | Integration | profile grammar -> builder gating -> wallet-visible state | An active `validator_mandate_lock_v1` object blocks ordinary spend, and the published grammar names `holder_commitment`, `control_commitment`, `beneficiary_commitment`, `payload_commitment`, `valid_from`, `valid_until`, `challenge_from`, `challenge_until`, `use_nonce`, `transition_policy_id`, `revocation_policy_id`, `disclosure_policy_id`, and `retention_policy_id` | Right-as-value or voucher-as-cash paths are accepted, the field grammar drifts, or the lock is only a UI preference | The builder rejects invalid object families, the guide and code expose the same grammar, and the profile is clearly marked as a proposed Phase 060 id layered on live `validator_mandate` |
| `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`, `crates/z00z_runtime/watchers/tests/test_object_alerts.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`, plus `cargo test -p z00z_validators --release --test test_object_policy_verdicts`, `cargo test -p z00z_watchers --release --test test_object_alerts`, `cargo test -p z00z_simulator --release --test scenario_1 test_scenario1_object_flows -- --nocapture`, `cargo test -p z00z_simulator --release --test scenario_1 test_scenario1_object_flows_reject_codes -- --nocapture`, `cargo test -p z00z_simulator --release --test scenario_1 test_scenario1_stage_surface -- --nocapture` | E2E / scenario | active lock -> validator reject -> watcher visibility -> simulator flow and reject codes | Active locks reject ordinary spend; unlock after expiry is accepted only through the approved unlock transition; redelegate paths behave as specified; unrelated assets remain selectable and hidden from lock-only proofs | Wrong-family proofs, replay, stale rights, revoked or expired rights, unknown-policy objects, bypassed post-expiry unlocks, or unrelated-asset privacy leaks are accepted | Validator, watcher, and simulator surfaces agree on fail-closed behavior; `crates/z00z_simulator/tests/scenario_1/test_scenario1_object_flows.rs` and `.../test_scenario1_stage_surface.rs` remain the truthful simulator homes; no new primitive leaf appears; v1 remains non-slashable or challenge-bounded rather than widening into slashable bond logic |

### `060-S09` Adversarial High-Finding Closure And Count Reconciliation

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `cargo test -p z00z_storage --release --test test_checkpoint_root_binding -- --nocapture`, `cargo test -p z00z_storage --release --test test_async_scheduler -- --nocapture`, `cargo test -p z00z_wallets --release --test test_stealth_request -- --nocapture`, `cargo test -p z00z_wallets --release --test test_stealth_scanner_flow -- --nocapture` | TDD + integration | top-three project-owned highs -> reproducer or disprover -> fix or closure memo | The first three highs close with code or proof artifacts for checkpoint lineage and delta integrity, PaymentRequest replay or rebinding, and stealth delivery or inbox confusion | A high finding is closed by prose only, or the reproducer no longer proves the suspected risk boundary | Each priority high gets one artifact with finding id, owner, threat statement, reproduction or proof strategy, linked tests or proof artifacts, and final status |
| `python3 ./.github/skills/z00z-verification-orchestrator/scripts/run-security-brainstorm.py ...`, plus a report-consistency check across JSON and Markdown outputs | Diagnostics / evidence | normalized `11` highs -> rerun -> count reconciliation | The rerun explicitly shows `7` project-owned highs and `4` protected-vendor highs with no hidden unowned item | JSON `high_risk_count`, JSON `top_findings`, and Markdown high rows disagree, or protected-vendor work sneaks into source edits under `crates/z00z_crypto/tari/**` | The rerun closes or reclassifies every high with evidence, preserves the protected-vendor bucket for `crates/z00z_crypto/tari/crypto`, `.../ristretto`, proof-adjacent logging in `ristretto_keys.rs`, and unsafe-block review in `ristretto_keys.rs`, and records any generator bug separately from security closure |

### `060-S10` HJMT Measurement Lanes And The `1 shard = 1 process` A/B Gate

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `cargo test -p z00z_storage --release --test test_bench_lanes -- --nocapture`, `crates/z00z_storage/scripts/run_storage_settlement_bench.py`, HJMT runtime observability and run-root artifacts | Diagnostics / E2E | lane labeling -> fair A/B reruns -> run-root metrics | Every report is labeled as Criterion, `/usr/bin/time -v`, scenario stage runtime, or user-facing throughput, and both `aggregator_owned` and `shard_process` are rerun on identical hardware, profile, shard count, operation mix, cache mode, persistence mode, and route generation | Worker-local wins are marketed as TPS wins, proof-size or single-proof timing is used as throughput evidence, or the two mappings are compared under different hardware or runtime conditions | The A/B packet reports `durable_root_published_tps`, `worker_local_tps`, `hjmt_journal_sync`, publication latency, blocked time, RSS, CPU utilization, restart time, and failover recovery time for both mappings |
| HJMT verdict memo backed by the A/B packet | Diagnostics / evidence | opt-in switch -> production-default decision | `shard_process` remains a YAML-selectable switch while `aggregator_owned` stays default unless durable throughput and recovery are neutral or better | Default flips because one-shard-one-process improves local work while durable publication or failover regresses | This scenario is the only accepted proof path for the production-default question; absent a passing B6 gate, current default remains unchanged and the switch stays opt-in only |

### `060-S11` Verification-Pipeline Runtime Reduction For The Top Slowest Events

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-fast.sh`, `cargo test --release`, `.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh`, `cargo semver-checks check-release --baseline-rev origin/main`, `.github/skills/z00z-l4-security-engineering-gate/scripts/unsafe-report.sh`, `.github/skills/z00z-l4-security-engineering-gate/scripts/run-fuzz-short.sh`, `.github/skills/z00z-l4-security-engineering-gate/scripts/run-constant-time.sh` | Diagnostics / evidence | baseline inventory -> optimization -> before or after packet | The phase captures before or after evidence for the baseline slowest paths: top `5%` equals `4` events consuming `3036.124s` of `4705.708s` total (`64.52%`), led by `l3-verify-fast` wall `1073.121s` and CPU `6101.64s`, `test:workspace` wall `995.056s`, `l4-supply-chain` wall `490.596s`, and `supply:semver:origin/main` wall `477.351s`, plus CPU or RSS-heavy `l4-unsafe`, `l4-fuzz`, and `l4-constant-time` | A gate is skipped, artifact production changes, or a wall-clock improvement hides CPU, RSS, or failure-rate regression | The optimization packet records wall, CPU, CPU percent, max RSS, filesystem output, cache or target roots, and execution mode; every optimized path preserves the same pass or fail semantics and artifact contract |
| Final rerun packet on the optimized tree | Diagnostics / evidence | optimized scripts -> final docs / supply-chain / adversarial / broad release reruns -> post-reopen refresh when `060-S12`, `060-S13`, or `060-S14` lands later | The final tree reruns docs, supply-chain, adversarial review, broad workspace release, and targeted Phase 060 slices on the optimized pipeline, then refreshes that closeout packet on the post-reopen tree if later supplemental slices land | The pipeline is called faster without re-running the final packet, later reopen slices land without refreshing the carried closeout packet, or only green subsets are shown | The final closeout proves Phase 060 on the final optimized tree, and any later `060-S12`, `060-S13`, or `060-S14` reopen refreshes the carried closeout packet before it stays authoritative; any remaining failures are explicitly identified as pre-existing and out-of-slice rather than hidden |

### `060-S12` Audit-Driven HJMT Core-Storage Closure

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_storage/tests/test_hjmt_root_generation.rs`, `test_hjmt_historical_proofs.rs`, `test_hjmt_import_export.rs`, `crates/z00z_storage/src/settlement/test_live_recovery.rs` | Integration / recovery | generation-0 compatibility -> generation-1 activation -> reload -> recovery export | After generation 1 activates, storage reloads the root-of-shard-roots as the live `SettlementStateRoot`; one-shard compatibility migration preserves settlement semantics; recovery export roundtrips shard identity and lineage facts | Live state stays on a monolithic fallback root, recovery omits shard-domain identity, or historical proofs require reinterpretation by current config | Generation-1 reload and semantic continuity tests pass; durable recovery exports `ShardId`, routing generation, route-table digest, journal lineage, and live root-generation metadata from storage-owned state |
| `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`, `test_hjmt_batch_proof_negative.rs`, `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`, `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario_settlement.rs` | Integration / diagnostics | committed route table -> exact shard set -> publication acceptance | Missing, extra, duplicated, foreign, or wrongly carried shard leaves reject through one shared checker; `activation_checkpoint > publication_checkpoint` and route-digest drift reject fail-closed; unchanged shards are carried forward byte-for-byte | Exact shard coverage lives only in preflight, or validator or watcher flows accept a route-table-consistent digest with the wrong shard set | One canonical route-table-driven acceptance path is shared by storage, validator, watcher, and preflight surfaces, and post-`060-S12` reruns refresh any earlier `060-S10` or `060-S11` HJMT closure evidence |

### `060-S13` Audit-Driven Prepared Tx Balance, Voucher Conservation, And `FeeEnvelope` Coverage

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/tx/tx_assembler.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs` | TDD + integration | resolved inputs -> canonical `TxAssembler` gate -> prepared package build | Ordinary prepared tx builds successfully when plaintext totals and commitments balance, and `wallet.tx.build_transaction` stays asset-cash only | A prepared tx with mismatched visible totals is accepted, a commitment drift slips through despite equal visible totals, or voucher/right inventory ids start flowing through the ordinary cash path | Negative `TxAssembler` tests `test_assemble_rejects_plain_mismatch` and `test_assemble_rejects_commit_mismatch` reject the two canonical mismatch families, `test_tx_build_raw_tx` remains green, `test_tx_build_rejects_voucher` stays green, and `test_tx_send_rejects_right` stays green |
| `crates/z00z_storage/src/settlement/test_model.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs`, `crates/z00z_core/tests/genesis/test_claim_flow.rs` | TDD + integration | voucher delta contract -> object-package preview/build reject -> claim-flow comparison anchor | Voucher partial redeem and object-package preview/build both reject residual mismatch while the separate claim-flow balance example still stays valid as a different seam | Voucher conservation mismatch is accepted, reject-code mapping drifts away from `ResidualMismatch`, or rights are forced into asset-like value accounting for fake symmetry | Storage delta tests reject conservation mismatch, object-package RPC preview/build rejects with `ResidualMismatch` / `OBJECT_RESIDUAL_MISMATCH`, and the claim-flow cryptographic balance test remains a comparative proof anchor rather than the only balance coverage in the packet |
| `crates/z00z_storage/src/settlement/test_model.rs`, `crates/z00z_storage/tests/test_fee_envelope.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs` | TDD + integration | typed object package with `FeeEnvelope` -> storage pre-commit contract -> wallet preview/build reject -> validator reject -> storage commit-path support anchor | Typed-object or rights fee support rejects malformed `FeeEnvelope` before storage commit while the existing support-bound commit-path tests still prove replay, transition, and blob-surface semantics | Malformed `FeeEnvelope` survives preview/build or validator inspection, package-level checks drift away from `validate_support()` semantics, or regular cash fees are reinterpreted as `FeeEnvelope` value accounting | Storage delta tests reject malformed `FeeEnvelope`, wallet object-package preview/build rejects with `FeeBoundary` / `OBJECT_FEE_BOUNDARY`, validator contract tests reject the same malformed payload family, storage `test_fee_envelope` keeps pre-mutation, wrong-transition, and blob-surface anchors green, and the evidence packet records that regular native fee outputs remain the cash-lane truth path |

### `060-S14` Audit-Driven Refund Binding, One-Plane Issue/Create, Incomplete Runtime Coverage, And Monotonic Right Delegation

| Test home | Class | State / proof path | Positive example | Negative example | Assertions and pass signal |
| --- | --- | --- | --- | --- | --- |
| `crates/z00z_storage/src/settlement/test_model.rs`, `crates/z00z_storage/tests/test_store_api.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs` | TDD + integration | voucher leaf refund target and backing -> typed delta -> store API reject or refund -> wallet object-package reject mapping | Reject or refund returns value to the declared refund target and preserves any declared consumed-asset or reserve source restriction | A reject or refund package routes value to a different target, strips an asset-backed restriction into unrelated clean cash, or strips a reserve-backed restriction into the wrong clean output family | Storage delta tests reject wrong refund target and wrong restricted source context, store API tests prove positive and negative reject or refund routing for both asset-backed and reserve-backed vouchers, and wallet object-package tests reject wrong target plus wrong restricted-source routing when the package request carries that source context |
| `crates/z00z_wallets/src/adapters/rpc/types/object.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`, `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_support.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs` | TDD + integration | canonical object-package request -> existing live object or truthful source/create context -> preview/build | Voucher issue from source asset or reserve context and right create from create context both build through the existing object-package RPC | A caller must invent a fake non-asset target row, use a second RPC, mix live-object and issue/create selectors, or push voucher or right ids through `wallet.asset.*` | Preview/build succeeds for truthful voucher issue and right create cases, mixed-selector, missing-source, stale-source, reserve-context mismatch, and wrong-family cases reject fail closed, and `wallet.asset.*` still rejects voucher or right inventory ids as cash |
| `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_object_alerts.rs` | TDD + integration | resolved batch or publication record -> validator completeness classification -> watcher alert or evidence projection | Retry-pending, missing-artifact, or publication-gap states surface as `Incomplete` or `ValidatorIncomplete` on the existing publication path | Missing verdict or missing publication binding disappears into raw watch-layer errors, or hard binding drift is mislabeled as incomplete | Validator tests distinguish accepted, rejected, and incomplete states; watcher tests emit `ValidatorIncomplete` for missing verdict, missing binding, missing-artifact, retry-pending, and publication-gap cases; true route or binding drift stays a hard mismatch |
| `crates/z00z_storage/tests/test_right_leaf.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs` | TDD + integration | current right leaf -> existing transfer/delegation path -> successor authority check | A delegated right with equal or narrower validity, scope, and policy commitments succeeds on the existing transfer path | A delegation widens validity, scope, or policy commitments, or requires a second delegation API or child-right hierarchy to express the shipped MVP contract | Storage and wallet tests prove monotonic narrowing succeeds, widening rejects fail closed, and `delegate_right` remains the existing transfer-path surface instead of creating a new delegation plane |

## Test File Placement And Reuse Rules

| Area | Reuse first | New home allowed only when |
| --- | --- | --- |
| Docs and traceability | Existing docs gate scripts and the files named in `060-01-PLAN.md` | A focused helper is needed to assert report consistency and it clearly belongs under the existing docs gate rather than a new docs layer |
| Core bootstrap and rights cleanup | Existing genesis tests, `test_rights_config`, policy and voucher tests, and current README or module-doc surfaces | A regression guard cannot be expressed inside the current owner-path or genesis test homes without making them misleading |
| HJMT topology and failover | `test_hjmt_process.rs`, `test_hjmt_topology.rs`, existing aggregator tests, `test_scenario_settlement.rs`, and generated fixture support under `test_hjmt_home.rs` | A focused decommission or stage-transition assertion cannot be added cleanly to an existing HJMT test, and the new file still lives under the current HJMT test tree |
| HJMT storage, recovery, and publication acceptance | `test_hjmt_root_generation.rs`, `test_hjmt_historical_proofs.rs`, `test_hjmt_import_export.rs`, `test_live_recovery`, `test_hjmt_batch_proof.rs`, `test_hjmt_batch_proof_negative.rs`, validator or watcher publication tests, and `test_hjmt_preflight.rs` | A focused generation-1 reload, route-bound recovery, or exact route-table acceptance assertion cannot be expressed in an existing truthful HJMT home without hiding the seam |
| Wallet typed-object and lock behavior | `redb_wallet_store/test_mod.rs`, wallet RPC test modules, backup tests, validator or watcher tests, and existing simulator object-flow tests | A new file is needed to keep one truthful seam isolated without creating a second wallet authority plane or second simulator story |
| Wallet prepared tx, voucher conservation, refund/source binding, object issue/create, typed-object `FeeEnvelope`, and monotonic right-delegation behavior | `tx_assembler.rs`, wallet tx RPC tests, wallet object-package RPC tests, storage delta-contract tests, `test_fee_envelope.rs`, `test_right_leaf.rs`, validator object-policy tests, and the existing claim-flow anchor | A focused reject-path, reserve-context, incomplete-runtime, or monotonic-delegation assertion cannot be added cleanly to an existing truthful home without blurring the distinction between regular cash tx balance, voucher conservation, refund/source routing, zero-value rights, one-plane object-package construction, typed-object fee-support contracts, and shipped delegation semantics |
| Supply-chain, adversarial, and verification performance | Existing gate scripts, `cargo vet`, existing report generators, and current profiling or run-root homes | A consistency checker or artifact summarizer is required and writes into the current canonical report or bench homes rather than a new authority layer |

## Skip And Reservation Rules

| Item | Status | Reason |
| --- | --- | --- |
| `060-TODO.md`, `060-CONTEXT.md`, and numbered plans as runtime targets | Skip | They are inputs and transfer artifacts, not direct runtime assertions. |
| `crates/z00z_crypto/tari/**` | Skip forever | Protected-vendor code is read-only in this repository for Phase 060; vendor findings close through wrapper, isolation, pin, upstream, or documented exception paths only. |
| A second wallet DB, second export plane, or tx-history authority inside `.wlt` | Forbidden | Phase 060 must preserve one wallet authority plane. |
| A second HJMT runtime home or second publication truth path | Forbidden | `shard_process` must be tested through generated fixtures and existing topology seams. |
| A second docs, supply-chain, benchmark, or summary authority | Forbidden | Repo-owned stores and existing report homes are the only truthful closeout surfaces. |
| Default promotion of `shard_process` before the B6 packet | Forbidden | The YAML switch may exist, but the production default stays `aggregator_owned` until the explicit A/B gate passes. |
| Performance wins obtained by skipping gates | Forbidden | C4 optimization must preserve docs, supply-chain, adversarial, semver, unsafe, fuzz, constant-time, and formal evidence semantics. |

## Completion Criteria

| Criterion | Pass condition |
| --- | --- |
| Coverage linkage | Every canonical task id from `A1..A5`, `B1..B6`, `C1..C4`, and `D1..D5` maps to at least one scenario above and one truthful test or artifact anchor. |
| HJMT and wallet-object reopen linkage | The supplemental `060-S12`, `060-S13`, and `060-S14` scenarios cover the live HJMT storage/publication seams plus the live prepared-tx, voucher-conservation, refund/source, one-plane issue/create, incomplete-runtime, and monotonic right-delegation seams reopened by the docs corpus and code audit. The narrowed `060-15` successor packet is executed through `060-S14` rather than by minting a new scenario family, and it supersedes the overlapping `060-14` MVP subset so the same reopen work is not implemented twice. |
| Journey completeness | Every scenario has at least one positive example and one negative example, and both are implementable in the named homes without guessing new boundaries. |
| Measurement honesty | `060-S10` names all A/B metrics and keeps the production-default verdict tied to durable publication and recovery rather than local work only. |
| Performance closure | `060-S11` explicitly covers the top-slowest verification paths from `060-z00z-verification-report.md` and records before or after evidence for each required gate. |
| One-plane discipline | No scenario widens wallet authority, HJMT authority, or report authority into a second plane. |
| Final closure | The final optimized-tree rerun covers docs, supply-chain, adversarial, broad release validation, and any targeted Phase 060 reruns needed to prove no semantic regression. |

If a requirement cannot be proven in the current homes, record the gap back
into the existing Phase 060 planning artifacts instead of creating a parallel
spec, a parallel coverage ledger, or a second closeout packet.
