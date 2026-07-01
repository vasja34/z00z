---
phase: 062-Gaps-Closing-2
artifact: tests-tasks
status: planning-ready
source: .planning/phases/062-Gaps-Closing-2/062-TEST-SPEC.md, .planning/phases/062-Gaps-Closing-2/062-TODO.md, .planning/phases/062-Gaps-Closing-2/062-CONTEXT.md, .planning/phases/062-Gaps-Closing-2/062-COVERAGE.md, .planning/phases/062-Gaps-Closing-2/062-01-PLAN.md..062-27-PLAN.md
updated: 2026-06-27
---

<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD041 MD047 -->

# Phase 062 Test Tasks

## 🎯 Purpose

This file turns `.planning/phases/062-Gaps-Closing-2/062-TEST-SPEC.md` into an implementation checklist for the
engineer or agent who will land Phase 062 coverage. It keeps the grouped-plan
order, preserves the one-plan-per-scenario boundary, and names the concrete
commands, examples, negative cases, and evidence that must exist before a
scenario can be called closed.

This is not a second design. The normative sources remain:

- `.planning/phases/062-Gaps-Closing-2/062-TODO.md` for task inventory and acceptance language;
- `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md` for anti-drift context;
- `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md` for exact `TASK-NNN -> PLAN-062-GNN` mapping;
- `.planning/phases/062-Gaps-Closing-2/062-TEST-SPEC.md` for scenario boundaries and invariants;
- `.planning/phases/062-Gaps-Closing-2/062-01-PLAN.md` through `.planning/phases/062-Gaps-Closing-2/062-27-PLAN.md` for exact execution
  seams and artifacts.

The Phase 062 referenced design corpus (`.planning/phases/062-Gaps-Closing-2/GAPS.md`, `.planning/phases/062-Gaps-Closing-2/asset-only.md`,
`.planning/phases/062-Gaps-Closing-2/HJMT-REPORT.md`, `.planning/phases/062-Gaps-Closing-2/HJMT-RAID -Sharding.md`,
`.planning/phases/062-Gaps-Closing-2/HJMT-Sharding-Storage-Techpaper.md`, `.planning/phases/062-Gaps-Closing-2/HJMT-структуры.md`, and
`.planning/phases/062-Gaps-Closing-2/Z00Z-Thin-Transaction-Mode.md`) is also live scope wherever the canonical
Phase 062 packet cites it. Do not treat cited design requirements as
future-only placeholders.

## 📌 Ordered Task List

| Step | Scenario / plan | Primary homes | What to implement or extend | Must prove before moving on |
| --- | --- | --- | --- | --- |
| `062-00` | coverage freeze | `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, `.planning/phases/062-Gaps-Closing-2/062-TEST-SPEC.md`, grouped plans | Freeze the one-to-one mapping from `062-S01..062-S27` to `PLAN-062-G01..PLAN-062-G27`, confirm every grouped plan still owns the tasks listed in `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, and normalize the packet to live canonical module and test paths wherever the workspace already has a settled implementation. | The executor can point to one scenario, one grouped plan, one truthful test-home family for every `TASK-001..TASK-125`, and one canonical live path for every module, helper, and test surface named in the packet. |
| `062-01` | `062-S01` / `PLAN-062-G01` | `.planning/phases/062-Gaps-Closing-2/062-TODO.md`, `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, plan corpus | Preserve the planning contract inside the test packet: task order, grouped-plan order, source refs, gate fields, and evidence format. | Coverage counts remain exact and every grouped plan still carries the required gate sections. |
| `062-02` | `062-S02` / `PLAN-062-G02` | storage root types, backend guards, HJMT backend tests | Add or extend tests that freeze one semantic settlement-root authority and current backend env wording. | Storage-root docs and tests agree that only the canonical HJMT root is public truth. |
| `062-03` | `062-S03` / `PLAN-062-G03` | checkpoint store, watcher publication, simulator checkpoint tests | Prove claim-root, checkpoint, publication, restart, and tamper behavior through one verifier path. | Claim-root and checkpoint proofs survive reload and fail closed on tamper or stale publication bindings. |
| `062-04` | `062-S04` / `PLAN-062-G04` | bench-lane tests, stage-surface tests, measurement guard | Separate measurement lanes and reject unmeasured proof-size or throughput claims. | All live performance claims are traceable to measured artifacts and none become semantic authority. |
| `062-05` | `062-S05` / `PLAN-062-G05` | tx storage, lifecycle DTOs, history suites | Add lifecycle DTO and durable-history tests that prove row hash, fold, tombstone, and current-view semantics. | Wallet responses project durable lifecycle state and restart convergence is explicit. |
| `062-06` | `062-S06` / `PLAN-062-G06` | wallet taxonomy, verify/import security suites | Add typed negative tests for unsupported versions, verify-only reports, import rollback, parse failures, and replay. | All decode, verify, import, and RPC failures are typed and deterministic with no premature mutation. |
| `062-07` | `062-S07` / `PLAN-062-G07` | scan status DTOs, authoritative receive path, worker restart suites | Add receive-outcome, cursor persistence, and worker no-mutation tests around the authoritative scan path. | Receive status stays truthful, cursor persistence is atomic, and invalid worker evidence leaves state unchanged. |
| `062-08` | `062-S08` / `PLAN-062-G08` | request validation, inbox helper, e2e request tests | Add inbox helper tests that prove ordering and metadata usefulness while preserving subordinate semantics. | Inbox state cannot replace or mutate authoritative receive outcomes. |
| `062-09` | `062-S09` / `PLAN-062-G09` | wallet simulator scenarios, logging policy tests | Join receive, import, history, and publication evidence into redacted simulator artifacts. | Simulator evidence is secret-safe and points back to live lower-level proofs. |
| `062-10` | `062-S10` / `PLAN-062-G10` | zkpack and canonical package tests | Either implement field-native or Poseidon2 claims with real tests or remove the live claim cleanly. | Canonical package behavior is proven and no future-only crypto naming remains as live truth. |
| `062-11` | `062-S11` / `PLAN-062-G11` | logging, backup, export, request, view-key tests | Add privacy and reveal coverage for logs, backups, exports, request metadata, and docs normalization. | No unowned secret leakage or transport-anonymity overclaim remains. |
| `062-12` | `062-S12` / `PLAN-062-G12` | object package, object RPC, validator object-policy tests | Add cash-vs-object, fee-envelope, unknown-policy, missing-right, voucher, and right-boundary tests. | Typed inventory stays on `wallet.object.*`, cash stays on `wallet.asset.*`, and negatives fail closed. |
| `062-13` | `062-S13` / `PLAN-062-G13` | rights policy, object package contract, validator tests | Add local-simulation tests for payroll/B2B, adapter metadata, agentic rights, and machine capability reuse limits. | Local-only scenarios are explicit and do not drift into live bridge or DA claims. |
| `062-14` | `062-S14` / `PLAN-062-G14` | genesis config, manifest loader, golden tests | Add referenced-manifest loader, schema, duplicate, path, and action-catalog tests. | `GenesisConfig` stays the only bootstrap authority and referenced subfiles merge deterministically. |
| `062-15` | `062-S15` / `PLAN-062-G15` | object vocabulary, generic errors, registry tests | Add owner-cleanup coverage for vocabulary, generic errors, and relocated tests. | Ownership is corrected without semantic rewrite or import breakage. |
| `062-16` | `062-S16` / `PLAN-062-G16` | HJMT planner, proof batch, journal, publication tests | Add local HJMT tests for route digests, publication binding, historical proofs, wallet/storage boundary, and fixture ownership. | Local HJMT evidence closes on real primitives and wallet-facing seams expose only public proof surfaces. |
| `062-17` | `062-S17` / `PLAN-062-G17` | distributed journal and consensus tests, topology tests | Add deterministic local-network simulation for journal replication, quorum, standby catch-up, membership, and split-brain behavior. | Distributed HJMT no longer depends on prose-only claims and uses real route/journal/root inputs. |
| `062-18` | `062-S18` / `PLAN-062-G18` | route rollout, dispatch, watcher publication, runtime configs | Add route-activation, scheduler-wave, remote-dispatch, storage-lock, and observability tests for the local distributed simulator. | Wrong-owner, mixed-generation, stale-lock, cross-shard, and drift cases fail closed. |
| `062-19` | `062-S19` / `PLAN-062-G19` | thin DTOs, thin index, tx store integration | Add signed snapshot, digest pinning, checkpoint binding, and naming-drift tests for thin mode. | Thin snapshots are authenticated helpers, not a second authority plane. |
| `062-20` | `062-S20` / `PLAN-062-G20` | thin cache, thin builder, thin-mode suites | Add thin expansion, cache, fallback-default, and builder-sharing tests. | Thin expands before runtime and shares exactly one meaning with thick mode. |
| `062-21` | `062-S21` / `PLAN-062-G21` | thin fallback, privacy, equivalence, wrong-index tests | Add restart, wrong-index, fallback, equivalence, and privacy negative suites for thin mode. | Stale, wrong, withheld, expired, or equivocated helper data fails closed and safely falls back. |
| `062-22` | `062-S22` / `PLAN-062-G22` | closure register docs, guard tests, repo-wide gates | Normalize stale terms, classify live vs compatibility vs simulation-only claims, and run broad final validation. | No ambiguous or future-only closeout language survives and broad validation status is explicit. |
| `062-23` | `062-S23` / `PLAN-062-G23` | chain client, local node sim, chain RPC tests | Add local node simulation and typed chain-client tests for tip, block, header, submit, status, and network info. | Wallet node capability works locally and stubs are removed from the tested path. |
| `062-24` | `062-S24` / `PLAN-062-G24` | broadcast impl, tx runtime state, tx store, retry suites | Add broadcast submission, retry, timeout, duplicate, replacement, reject, and confirmation tests over durable tx state. | Broadcast lifecycle transitions persist correctly and remain visible in durable history. |
| `062-25` | `062-S25` / `PLAN-062-G25` | fee estimator, `FeeRateSource` seam, chain client sim | Add live-fee source simulation, cache/fallback tests, and pathological-data negatives for fee estimation. | Fee estimation uses a simulated live source safely and keeps real weight logic intact. |
| `062-26` | `062-S26` / `PLAN-062-G26` | scan engine, receive service, remote worker tests | Add local chain/node simulation and trust-boundary tests for the remote scan worker seam. | Remote scan helpers can aid orchestration but cannot bypass local receive validation or persistence rules. |
| `062-27` | `062-S27` / `PLAN-062-G27` | wallet policy, tx storage, broadcast-linked tests | Add daily-spend, confirmation, restart persistence, and multi-send aggregation tests. | Policy enforcement is durable, restart-safe, and fail-closed. |

## 🔧 Required Commands And Evidence By Step

| Step | Mandatory narrow commands or checks | Evidence that must be preserved |
| --- | --- | --- |
| `062-00` | `rg -o "TASK-[0-9]{3}" .planning/phases/062-Gaps-Closing-2/062-TODO.md | sort -u | wc -l`; `rg -o "PLAN-062-G[0-9]{2}" .planning/phases/062-Gaps-Closing-2/062-TODO.md | sort -u | wc -l`; `bash -lc 'for p in crates/z00z_wallets/src/receiver/request_inbox.rs crates/z00z_core/src/genesis/manifest_ref_loader.rs crates/z00z_runtime/aggregators/src/dist_sim.rs crates/z00z_runtime/aggregators/src/consensus_adapter.rs crates/z00z_runtime/aggregators/src/dist_dispatch.rs crates/z00z_runtime/aggregators/src/dist_scheduler.rs crates/z00z_wallets/src/tx/thin_types.rs crates/z00z_wallets/src/tx/thin_snapshot.rs crates/z00z_wallets/src/tx/thin_cache.rs crates/z00z_wallets/src/tx/thin_builder.rs crates/z00z_wallets/src/chain/local_node_sim.rs crates/z00z_wallets/tests/test_wallet_policy.rs; do test -e "$p" || { echo "missing path: $p"; exit 1; }; done'`; `rg -n "RuntimeReceiveScanOutcome" crates/z00z_wallets .planning/phases/062-Gaps-Closing-2/062-TEST-SPEC.md .planning/phases/062-Gaps-Closing-2/062-TESTS-TASKS.md` | Count output proving `125` unique tasks and `27` grouped plans, plus workspace existence output and symbol grep proving the packet names live canonical paths and the current receive-outcome type instead of stale or proposed placeholders. |
| `062-01` | `rg -n "Coverage Appendix|anti_placeholder_gate|simulation_gate|evidence_gate" .planning/phases/062-Gaps-Closing-2/062-*-PLAN.md`; `rg -n "^## (Verdict|Normative Language|Source Corpus|Count Answer|Required GSD Plan Groups|Pre-Plan Blockers|Requirement Gate Contract|Artifact/Test/Result Proof Contract|Current Wallet Path Rewrite Map|Plan Waves|Canonical Task Inventory|Local Full-System Simulation Closure Register|Current Code Evidence Anchors|GSD Plan Generation Contract|Verification Checklist)$" .planning/phases/062-Gaps-Closing-2/062-TODO.md .planning/phases/062-Gaps-Closing-2/062-CONTEXT.md` | Grep output showing that the grouped plan packet preserved the mandatory gate fields and that `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md` still mirrors the required TODO meta-sections. |
| `062-02` | `cargo test --release -p z00z_storage --test test_hjmt_backend_conformance`; `cargo test --release -p z00z_storage --test test_live_guardrails`; `rg -n "AssetStateRoot|Z00Z_SETTLEMENT_BACKEND_MODE|semantic settlement-root" crates/z00z_storage .planning/phases/Z00Z-IMPL-PHASES.md` | Passing storage root tests and grep output proving current backend wording and one semantic root story. |
| `062-03` | `cargo test --release -p z00z_storage --test test_claim_source_proof`; `cargo test --release -p z00z_storage --test test_checkpoint_root_binding`; `cargo test --release -p z00z_storage --test test_checkpoint_finalization`; `cargo test --release -p z00z_simulator --test scenario_1 test_checkpoint_acceptance:: -- --nocapture`; `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_e2e:: -- --nocapture`; `cargo test --release -p z00z_rollup_node --test test_hjmt_node_lifecycle`; `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface::test_scenario1_stage_surface -- --exact` | Proof artifacts or logs showing checkpoint/root/publication binding, tamper rejection, restart-safe reload, and stable stage artifact names. |
| `062-04` | `cargo test --release -p z00z_storage --test test_bench_lanes`; `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface::test_scenario1_stage_surface -- --exact`; `cargo test --release -p z00z_rollup_node --test test_hjmt_node_lifecycle`; `rg -n "proof-size|TPS|latency|RSS|CPU|informational" .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_storage crates/z00z_simulator` | Evidence that benchmark lanes are separated and all performance wording is measured and informational. |
| `062-05` | `cargo test --release -p z00z_wallets --test test_tx_store_integration`; `cargo test --release -p z00z_wallets test_tx_get_ -- --nocapture`; `cargo test --release -p z00z_wallets test_tx_lifecycle_projection_survives_restart -- --nocapture`; `cargo test --release -p z00z_wallets --test test_direct_tx_receive` | Durable lifecycle output showing row hash/fold continuity, tombstones, status projection, and restart-safe history. |
| `062-06` | `cargo test --release -p z00z_wallets --test test_import_error_taxonomy`; `cargo test --release -p z00z_wallets --test test_asset_import_security`; `cargo test --release -p z00z_wallets --test test_asset_replay_protection`; `cargo test --release -p z00z_wallets test_tx_verify_report_lifecycle_and_error_codes -- --nocapture` | Typed error output and no-mutation proofs for verify-only, replay, malformed, and unsupported-version cases. |
| `062-07` | `cargo test --release -p z00z_wallets --test test_asset_scanner_flow`; `cargo test --release -p z00z_wallets --test test_asset_scanner_cache`; `cargo test --release -p z00z_wallets --test test_direct_tx_receive`; `cargo test --release -p z00z_wallets test_recv_range_restart -- --nocapture`; `cargo test --release -p z00z_wallets test_worker_ -- --nocapture` | Logs or assertions showing authoritative receive outcomes, atomic cursor persistence, and worker no-mutation behavior. |
| `062-08` | `cargo test --release -p z00z_wallets --test test_stealth_request`; `cargo test --release -p z00z_wallets --test test_e2e_req_flow`; `cargo test --release -p z00z_wallets --test test_view_key_contract`; `rg -n "request-bound|inbox|authoritative|off-consensus" crates/z00z_wallets/src/receiver crates/z00z_wallets/src/services` | Tests and grep output proving inbox metadata is useful but subordinate and non-authoritative. |
| `062-09` | `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_e2e:: -- --nocapture`; `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_stage_surface::test_scenario1_stage_surface -- --exact`; `cargo test --release -p z00z_wallets --test test_rpc_logging_risk_policy`; `cargo test --release -p z00z_wallets --test test_direct_tx_receive` | Redacted simulator artifacts that join lifecycle, scan, import, and publication evidence without secret leakage. |
| `062-10` | `cargo test --release -p z00z_wallets --test test_zkpack`; `cargo test --release -p z00z_wallets --test test_asset_pack_v2_memo`; `cargo test --release -p z00z_wallets --test test_golden_tag16`; `rg -n "Poseidon2|field-native|future|deferred" crates/z00z_crypto crates/z00z_core .planning/phases/Z00Z-IMPL-PHASES.md` | Proof that real package behavior is covered and unimplemented field-native claims are removed or fully exercised. |
| `062-11` | `cargo test --release -p z00z_wallets --test test_rpc_logging_risk_policy`; `cargo test --release -p z00z_wallets --test test_backup_metadata_policy`; `cargo test --release -p z00z_wallets --test test_view_key_contract`; `cargo test --release -p z00z_wallets --test test_e2e_req_flow`; `rg -n "transport anonymity|selective disclosure|wallet-local|secret|metadata" crates/z00z_wallets .planning/phases/Z00Z-IMPL-PHASES.md` | Logs, backups, request flows, and docs showing bounded privacy claims with no secret leakage. |
| `062-12` | `cargo test --release -p z00z_validators --test test_object_policy_verdicts`; `cargo test --release -p z00z_wallets test_asset_impl -- --nocapture`; `cargo test --release -p z00z_wallets --test test_tx_store_integration`; `rg -n "cash-only|voucher|right|fee envelope|wallet.object|wallet.asset" crates/z00z_wallets crates/z00z_storage` | Validator and wallet outputs proving object inventory, fee envelope, and voucher/right boundaries. |
| `062-13` | `cargo test --release -p z00z_validators --test test_object_policy_verdicts`; `cargo test --release -p z00z_wallets test_asset_impl -- --nocapture`; `rg -n "agentic|machine capability|cross-chain|local adapter|useful-work|payroll|B2B" crates/z00z_core crates/z00z_storage crates/z00z_wallets .planning/phases/Z00Z-IMPL-PHASES.md` | Local-simulation evidence for payroll/B2B, adapter metadata, agentic rights, and capability failure modes. |
| `062-14` | `cargo test --release -p z00z_core test_genesis_manifest_phase059_fixture -- --nocapture`; `cargo test --release -p z00z_core test_genesis_manifest_refs -- --nocapture`; `cargo test --release -p z00z_core test_genesis_manifest_goldens -- --nocapture`; `rg -n "GenesisConfig|root manifest|referenced subfiles|actions_config" crates/z00z_core/src/genesis crates/z00z_core/src/assets` | Fixture, refs, and golden outputs proving deterministic merge, schema validation, and explicit action-catalog treatment. |
| `062-15` | `cargo test --release -p z00z_core test_registry_suite -- --nocapture`; `cargo test --release -p z00z_validators --test test_object_policy_verdicts`; `rg -n "ObjectFamily|ObjectRoleV1|AssetError|Right|Voucher|Genesis" crates/z00z_core crates/z00z_storage crates/z00z_runtime` | Owner-cleanup evidence showing truthful vocabulary and generic-error homes. |
| `062-16` | `cargo test --release -p z00z_validators --test test_hjmt_publication_contract`; `cargo test --release -p z00z_watchers --test test_hjmt_publication_contract`; `cargo test --release -p z00z_rollup_node --test test_hjmt_topology`; `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_e2e:: -- --nocapture`; `rg -n "config/hjmt_runtime|wallet sees public proofs|storage-created scopes|route-table" crates config .planning/phases/Z00Z-IMPL-PHASES.md` | Local HJMT route/proof/publication evidence plus ownership text proving runtime fixture placement and public-only wallet exposure. |
| `062-17` | `cargo test --release -p z00z_aggregators --test test_hjmt_dist_journal`; `cargo test --release -p z00z_aggregators --test test_hjmt_consensus`; `cargo test --release -p z00z_rollup_node --test test_hjmt_topology`; `rg -n "quorum|term|lineage|standby|partition|heal|membership|split-brain" crates/z00z_runtime crates/z00z_rollup_node config/hjmt_runtime` | Local-network simulation logs proving replication, quorum resolution, standby catch-up, membership change, and split-brain handling. |
| `062-18` | `cargo test --release -p z00z_aggregators --test test_hjmt_route_rollout`; `cargo test --release -p z00z_aggregators --test test_hjmt_dispatch`; `cargo test --release -p z00z_watchers --test test_hjmt_publication_contract`; `rg -n "dispatch|scheduler|route rollout|storage lock|observability|drift|stall|freeze" crates/z00z_runtime config/hjmt_runtime` | Route-activation, dispatch, lock, and telemetry evidence from the local distributed simulator. |
| `062-19` | `cargo test --release -p z00z_wallets --test test_thin_index`; `cargo test --release -p z00z_wallets --test test_tx_store_integration -- --nocapture`; `rg -n "thin|signed index|snapshot|checkpoint-bound|root-name" crates/z00z_wallets crates/z00z_storage .planning/phases/Z00Z-IMPL-PHASES.md` | Signed snapshot outputs, digest pins, and typed failures for thin auth or naming drift. |
| `062-20` | `cargo test --release -p z00z_wallets --test test_thin_modes`; `cargo test --release -p z00z_wallets --test test_thin_cache`; `cargo test --release -p z00z_wallets --test test_thin_index`; `rg -n "ThinWorkItem|thin verdict|fallback|expand before runtime" crates/z00z_wallets` | Thin cache and expansion evidence proving one builder meaning and no thin-specific runtime semantics. |
| `062-21` | `cargo test --release -p z00z_wallets --test test_thin_fallback`; `cargo test --release -p z00z_wallets --test test_thin_privacy`; `cargo test --release -p z00z_wallets --test test_thin_equivalence`; `cargo test --release -p z00z_wallets --test test_thin_index` | Fallback, equivalence, privacy, stale, wrong, withheld, expired, and equivocation outputs for thin helper flows. |
| `062-22` | `cargo test --release -p z00z_wallets --test test_spec_terms_guard`; `cargo test --release -p z00z_storage --test test_live_guardrails`; `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh`; `rg -n "Residual gap register|Closeout status:|AssetStateRoot|OnionNet|Linked Liability" .planning docs crates`; `cargo fmt`; `cargo clippy --release --all-targets --all-features`; `cargo test --release --all`; `cargo doc --release --no-deps` | Final closure register, residual-term drift grep, broad repo validation output from release-mode cargo gates, and exact notes for any unrelated failures still outside the slice. |
| `062-23` | `cargo test --release -p z00z_wallets --test test_chain_client_sim`; `cargo test --release -p z00z_wallets --test test_direct_tx_receive`; `rg -n "get_tip_height|get_block|get_header|submit_transaction|get_transaction_status|get_network_info" crates/z00z_wallets/src/chain` | Local node-sim output proving wallet chain methods exist and fail with typed results on missing data. |
| `062-24` | `cargo test --release -p z00z_wallets broadcast_retry -- --nocapture`; `cargo test --release -p z00z_wallets --test test_chain_broadcast_retry`; `cargo test --release -p z00z_wallets --test test_tx_store_integration`; `rg -n "broadcast|confirmation|retry|timeout|duplicate|reorg|replacement" crates/z00z_wallets/src/chain crates/z00z_wallets/src/rpc` | Durable lifecycle evidence for retry, timeout, reject, duplicate, reorg, replacement, and confirmation. |
| `062-25` | `cargo test --release -p z00z_wallets test_rates_ -- --nocapture`; `cargo test --release -p z00z_wallets spike_rate -- --nocapture`; `cargo test --release -p z00z_wallets --test test_fee_rate_source`; `cargo test --release -p z00z_wallets --test test_chain_client_sim`; `rg -n "fee source|cache_ttl|stale|zero|spike|fallback" crates/z00z_wallets/src/tx crates/z00z_wallets/src/chain` | Simulated live-fee output showing cache, fallback, stale, zero, and spike handling on the real estimator path. |
| `062-26` | `cargo test --release -p z00z_wallets --test test_remote_scan_worker`; `cargo test --release -p z00z_wallets test_remote_worker_ -- --nocapture`; `cargo test --release -p z00z_wallets test_worker_ -- --nocapture`; `cargo test --release -p z00z_wallets`; `rg -n "RemoteScanWorker|recv_range_from_worker|recv_range_with_worker|no-mutation|stale|malicious" crates/z00z_wallets/src/chain crates/z00z_wallets/src/services` | Remote-worker evidence showing strict trust boundaries, restart behavior, and no-mutation on malicious or stale inputs. |
| `062-27` | `cargo test --release -p z00z_wallets --test test_wallet_policy`; `cargo test --release -p z00z_wallets --test test_tx_store_integration`; `cargo test --release -p z00z_wallets --test test_chain_broadcast_retry`; `rg -n "max_daily_amount|require_confirmation|DailyLimitExceeded|confirmation" crates/z00z_wallets/src/wallet crates/z00z_wallets/src/persistence` | Policy outputs proving daily caps, confirmation rules, restart persistence, and aggregated spend enforcement. |

## 🔁 Shared Validation Rules

| Rule | Requirement |
| --- | --- |
| Bootstrap fail-fast | Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first for every step. If it fails, stop, fix, rerun, and only then continue. |
| Broad Rust gate | Run `cargo test --release` for any Rust, test, simulator, serialization, config-loader, wallet policy, or verification-script affecting slice. Use `--release` for every cargo validation command in this packet when cargo supports it. |
| Review repetition | Run `./.github/prompts/gsd-review-tasks-execution.prompt.md` (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times, fix all issues and warnings, and continue until at least `2` consecutive runs show no significant issues. |
| Commit discipline | If a step needs a commit after validation, use `/z00z-git-versioning`. |
| Evidence discipline | Preserve the exact output, artifact, log, grep, or simulator packet that proves the pass signal. Do not replace evidence with prose. |
| Real-primitive discipline | Fake only the allowed transport, process-boundary, external network, or scheduler seams. Roots, commitments, signatures, route tables, journals, policy decisions, and wallet persistence must remain real. |

## 🚫 Global Reject Conditions

| Condition | Why it fails the phase |
| --- | --- |
| A scenario adds a new authority plane for wallet state, HJMT truth, genesis authority, or thin semantics | It breaks the anti-drift contract and duplicates existing logic instead of proving it. |
| A helper seam mutates state before authoritative validation | Worker, inbox, remote scan, or node-helper logic must stay subordinate to current truth paths. |
| A performance or cryptographic claim survives without executable proof | Phase 062 must not close on comment-only or docs-only claims. |
| Distributed HJMT is closed by prose or static config only | The local deterministic simulator is mandatory for replication, quorum, rollout, dispatch, and failover semantics. |
| Thin mode introduces runtime-only thin semantics downstream | Thin must collapse into existing runtime vocabulary before admission. |
| Full-system wallet node, broadcast, fee, scan, or policy behaviors remain stubbed | `TASK-121..TASK-125` are mandatory local full-system blockers and cannot be deferred. |
| Final closeout hides unrelated failures or unresolved local blockers | Phase status must stay explicit and phase-failing until blockers are closed or correctly classified as external-adapter-only. |

## 🧭 Implementation Notes For The Next Engineer

| Area | Reuse | Keep explicit |
| --- | --- | --- |
| Wallet lifecycle and policy | Existing persistence, RPC, and receive seams | Durable lifecycle, typed errors, authoritative receive, and restart-safe policy state |
| HJMT local and distributed closure | Current storage, validator, watcher, rollup-node, and aggregator seams | One proof path, route digest integrity, runtime fixture ownership, and real planner or journal inputs |
| Thin mode | Current wallet tx and storage primitives | Authenticated helper semantics, fallback safety, and zero concept drift into runtime semantics |
| Genesis and owner cleanup | Existing genesis loaders and registry suites | One canonical bootstrap authority and truthful module ownership |
| Full-system wallet/node simulation | Current chain, broadcast, fee, receive, and policy seams | Local deterministic node or network simulation around real wallet storage and validation primitives |

<verify>

1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` as
   the mandatory fail-fast gate before any broader validation.
2. Re-run packet consistency checks so the test-task document and the test spec
   both name live canonical paths where those implementations already exist:
   `bash -lc 'for p in crates/z00z_wallets/src/receiver/request_inbox.rs crates/z00z_core/src/genesis/manifest_ref_loader.rs crates/z00z_runtime/aggregators/src/dist_sim.rs crates/z00z_runtime/aggregators/src/consensus_adapter.rs crates/z00z_runtime/aggregators/src/dist_dispatch.rs crates/z00z_runtime/aggregators/src/dist_scheduler.rs crates/z00z_wallets/src/tx/thin_types.rs crates/z00z_wallets/src/tx/thin_snapshot.rs crates/z00z_wallets/src/tx/thin_cache.rs crates/z00z_wallets/src/tx/thin_builder.rs crates/z00z_wallets/src/chain/local_node_sim.rs crates/z00z_wallets/tests/test_wallet_policy.rs; do test -e "$p" || { echo "missing path: $p"; exit 1; }; done'`
   and
   `rg -n "RuntimeReceiveScanOutcome" crates/z00z_wallets .planning/phases/062-Gaps-Closing-2/062-TEST-SPEC.md .planning/phases/062-Gaps-Closing-2/062-TESTS-TASKS.md`
3. Run `cargo test --release` when the touched slice changes Rust, tests,
   simulator behavior, public APIs, serialization, or verification scripts.
   Use release mode for every cargo validation command in this packet when
   cargo supports it.
4. Run `./.github/prompts/gsd-review-tasks-execution.prompt.md`
   (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times. Fix all
   issues and warnings and continue until at least `2` consecutive runs show
   no significant issues.
5. If a commit is required after verification, use `/z00z-git-versioning`.

## ✅ Exit Conditions

The Phase 062 test-task packet is complete when:

1. Every grouped plan `PLAN-062-G01` through `PLAN-062-G27` has one owning
   scenario in `.planning/phases/062-Gaps-Closing-2/062-TEST-SPEC.md`.
2. Every scenario has exact commands, realistic positive examples, required
   negative cases, and measurable pass conditions.
3. The next engineer or agent can implement the coverage without inventing a
   second authority plane, a parallel simulator truth path, or undocumented
   scenario boundaries.
4. `TASK-121` through `TASK-125` are treated as mandatory local full-system
   simulation work, not as optional follow-up.
5. Any seam that still cannot be honestly proven is recorded back into the
   Phase 062 planning packet as a blocker rather than widened into vague
   “future coverage” language.
