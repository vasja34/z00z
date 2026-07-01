⭕️
- 052-DAG-Offline-Tx-Spec.md использовать как будущую архитектуру DAG: `TxPackage` как node, ancestor closure, topo apply, transport metadata вне consensus.
- 052-practical-recomendations.md превратить в короткое решение: DAG backlog должен быть отдельным deferred блоком.

**Как Это Ложится На Уже Сделанное**
Очень хорошо ложится, если DAG делать **оберткой вокруг текущего `TxPackage`**, а не вторым форматом транзакции. Текущий live contract уже есть: `TxPackage` в tx_wire_types.rs, полный verifier в tx_verifier.rs, digest в tx_digest.rs. Это надо сохранять почти нетронутым.

Storage уже дает важную основу: `TxPkgSum`, `InputResolver`, `apply_batch_checkpoint`, `build_cp_draft` в build.rs. Это не DAG resolver, но это правильный substrate для working-window apply. Simulator stage 6 уже делает bridge/fragment/exec_input handoff через mod.rs и exec_input_builder.rs, но это сценарный прототип, не production DAG.

**Какие Модули Дорабатывать**
1. `z00z_wallets::tx::verify` — минимально: не менять `TxPackage`, `TxWire`, digest и verifier ladder. DAG должен сначала вызывать обычную package verification, потом уже ancestor/window logic.

2. `z00z_wallets` RPC/adapters — средне: сейчас tx_impl_server_lifecycle.rs отделяет verify/report/import, а tx_rpc_impl.rs отдельно gate-ит import. Для DAG нужны новые report/verify surfaces, но import нельзя делать побочным эффектом verify.

3. `z00z_storage::checkpoint` — глубже всего: надо добавить DAG working-window orchestration поверх существующих `CheckpointExecTx` и `CheckpointExecInput` в exec_input.rs. Тут появятся ancestor resolution, cycle rejection, conflict policy, topo ordering, state-anchored vs ancestor-provided input resolution.

4. `z00z_simulator::scenario_1::stage_6_utils` — средне: текущие `Stage6Bridge` и `FragTx` в bundle_lane_impl.rs можно использовать как prototype/test harness, но нельзя выдавать их за production DAG container.

5. `z00z_rollup_node` — средне после storage: lib.rs и lib.rs сейчас проверяют package + checkpoint inclusion для одного accepted path. Для DAG нужно расширение на bundle/window inclusion, но без “proof-only shortcut”.

**Глубина Изменений**
Это **не переписывание транзакций**, если делать правильно. `TxPackage`, digest, public spend verification и output construction остаются основой. Но это **глубокое добавление orchestration layer**: новый DAG wrapper, dependency graph, topological apply, conflict policy, branch packaging, bundle verification reports, storage/checkpoint integration, simulator coverage, rollup inclusion checks.







## 3. Offline Transaction Current Spec

**Goal:**

- Make the offline transaction spec the self-contained source for delayed-connectivity package behavior.
- Prove package verification, reporting, import readiness, lifecycle projection, receiver routing, and final admission boundaries are represented clearly enough for local implementation.

**Source:**

- [Offline transaction current spec, purpose and authority](../.planning/phases/051-offline-tx/050-Offline-Tx-Spec.md#purpose-and-authority)
- [Offline transaction current spec, current transaction package model](../.planning/phases/051-offline-tx/050-Offline-Tx-Spec.md#current-transaction-package-model)
- [Offline transaction current spec, local verification guarantees](../.planning/phases/051-offline-tx/050-Offline-Tx-Spec.md#local-verification-guarantees)
- [Offline transaction current spec, current offline-capable receive cycle](../.planning/phases/051-offline-tx/050-Offline-Tx-Spec.md#current-offline-capable-receive-cycle)

**Implementation-relevant fragments:**

- Use purpose and authority to treat this as the current delayed-connectivity package source.
- Use current transaction package model for `ReceiverCard`, `ReceiverCardRecord`, `TxPackage`, `TxWire`, digest, version, chain, and receiver-binding semantics.
- Use local verification guarantees for structural verification, spend-data checks, public-spend authorization, and report-only behavior.
- Use current offline-capable receive cycle for verify, report, import readiness, persistence, and reconciliation separation.

**Locality gate:**

- The spec describes delayed-connectivity transaction packages that can be created, parsed, verified, reported, imported, and reconciled locally.
- No live transport, mempool, block producer, or testnet is required.

**Implementation boundary:**

- In scope: package contract, transaction wire contract, receiver routing model, output construction, local verifier guarantees, public spend authorization path, runtime verification response, import-readiness gate, and lifecycle projection.
- Out of scope: final chain admission, live receiver publication service, new package family, network delivery protocol, or replacing checkpoint/storage authority.

**Implementation tasks:**

1. Treat this spec as the source for phases `10`, `15`, `16`, and `21`.
2. Keep `ReceiverCard`, `ReceiverCardRecord`, `TxPackage`, `TxWire`, and public proof/authorization containers as the current package vocabulary.
3. Ensure output construction uses current wallet builder semantics and local self-verification.
4. Enforce the live verifier scope: structural package validity, digest checks, spend-data checks where applicable, and local ownership reporting.
5. Implement public spend authorization checks only where package content requires that path.
6. Keep current offline-capable receive cycle distinct across verify response, report-only output, import readiness, and persistence.
7. Map lifecycle projection into wallet status and tx-history/asset convergence tests.
8. Keep package portability distinct from final admission.

**Tests and simulation:**

- Package contract tests for required fields, digest stability, version, chain ID, package type, and malformed bytes.
- Receiver routing tests for raw receiver compatibility, published receiver trust, wrong chain, expired request, wrong identity pin, and amount binding.
- Verifier tests for local validity, public spend authorization, invalid spend data, and report-only behavior.
- Import-readiness tests proving valid/reportable is not enough to persist.
- Lifecycle projection tests for pending, imported, submitted, admitted, confirmed, failed, cancelled, and conflicted.
- Simulator delayed-connectivity path from package creation through local reconciliation.

**Done when:**

- Offline transaction current-state semantics are represented in local package, wallet, and simulator plans.
- Package verification, reporting, import readiness, and lifecycle projection are not conflated.
- The spec can be used directly as the source for local offline transaction work without any external bridge note.

**Doublecheck:**

- Local condition: satisfied. The work is local package and wallet behavior.
- Developer clarity: satisfied. Package model, verifier scope, import gate, and lifecycle tests are explicit.

## 4. Offline Transaction Execution Backlog

**Goal:**

- Convert the offline transaction TODO into a local execution checklist for canonical verifier, receiver publication trust, sender invariants, runtime report/import gates, simulator parity, and seam reuse.
- Prove all 050 backlog items map to one package model, one verifier, one readiness vocabulary, and one import boundary.

**Source:**

- [Offline transaction TODO, decision summary](../.planning/phases/051-offline-tx/050-TODO.md#decision-summary)
- [Offline transaction TODO, concrete execution tasks](../.planning/phases/051-offline-tx/050-TODO.md#concrete-execution-tasks)
- [Offline transaction TODO, implementation boundary](../.planning/phases/051-offline-tx/050-TODO.md#implementation-boundary)
- [Offline transaction TODO, completion gate](../.planning/phases/051-offline-tx/050-TODO.md#completion-gate)

**Implementation-relevant fragments:**

- Use the decision summary for the accepted local direction: one package model, one verifier, and one import boundary.
- Use concrete execution tasks for items 050-01 through 050-07.
- Use implementation boundary to prevent new transport, new finality, or a second package family.
- Use completion gate to decide when phases `15`, `16`, and `21` have enough local evidence.

**Locality gate:**

- The backlog can be completed through wallet package code, local verifier tests, storage-backed import records, and simulator parity scenarios.
- No live P2P transport, receiver registry, mempool, or testnet is needed.

**Implementation boundary:**

- In scope: tasks 050-01 through 050-07, one live package model, raw receiver versus published receiver trust boundary, conditional public-spend reuse, status gate hardening, verify/report versus import/persist separation, simulator parity, and seam reuse.
- Out of scope: new package format family, live receiver publication infrastructure, new finality source, or broad wallet history rewrite beyond convergence required by this plan.

**Implementation tasks:**

1. Complete 050-01 canonical verifier and package authority path.
2. Complete 050-02 receiver publication-trust boundary by separating raw routing compatibility from trusted published receiver metadata.
3. Complete 050-03 sender output-construction invariant lock-in.
4. Complete 050-04 runtime verify, public spend reuse, and report contract.
5. Complete 050-05 import-readiness vocabulary and import-boundary semantics.
6. Complete 050-06 Stage 4 or Stage 5 parity and report-only receive closure in `z00z_simulator`.
7. Complete 050-07 harness and seam-reuse lock-in.
8. Add explicit documentation in code/test names that verify/report is not import/persist.
9. Ensure every backlog task maps to one or more tests in phases `15`, `16`, and `21`.

**Tests and simulation:**

- Canonical verifier tests for digest, kind, type, version, chain, spend data, and receiver binding.
- Receiver boundary tests for raw card compatibility, published receiver record trust, wrong publication metadata, and stale receiver record.
- Sender invariant tests for output binding, fee output, duplicate inputs/outputs, and input/output overlap.
- Runtime report tests for valid-owned, valid-not-owned, unsupported, invalid, and valid-not-import-ready packages.
- Import-boundary tests for state mutation only after readiness gate.
- Simulator parity tests proving Stage 4/Stage 5 behavior matches real wallet surfaces.
- Seam-reuse tests proving no new verifier or status family is introduced.

**Done when:**

- All 050 backlog tasks are represented as local implementation work and tests.
- Offline package handling has one verifier, one readiness vocabulary, and one import boundary.
- Simulator parity proves wallet behavior without redefining it.

**Doublecheck:**

- Local condition: satisfied. Every backlog item is wallet/simulator local work.
- Developer clarity: satisfied. Task IDs, boundaries, and tests are explicit.

## 

---



## 15. Offline `TxPackage` Verify, Report, And Import Hardening

**Goal:**

- Keep one verifier and one import path while making reject reasons machine-readable and redacted.
- Prove that verify is advisory and import is deterministic, rollback-safe, and idempotent on identical payloads.

**Closeout note:**

- `RuntimeVerifyTxPkgResponse` now carries `lifecycle` and `error_codes` alongside backward-compatible string `errors`.
- `RuntimeImportTxResponse` now carries `error_codes`, while import reject paths expose typed JSON-RPC error data through `RuntimeTxRpcErrorData`.
- Exported portable packages now rewrite `status` from the live durable lifecycle (`Created`/`Submitted`/`Admitted`/`Confirmed`) before wrapping, so offline verify/import reads the current canonical wallet state instead of stale stored bytes.
- Verify now distinguishes `InvalidDigest`, `InvalidPublicSpendProof`, `NotImportReady`, and `NoOwnedOutputs`. Import now distinguishes `InvalidEncoding`, `InvalidDigest`, `UnsupportedPackageVersion`, `WrongChain`, `DuplicateConflict`, `AlreadySpent`, and generic internal failure; digest/version/chain rejects stay no-mutation, while already-spent conflicts append the richer failed tx-history kind on the existing journal lane.
- Identical repeated imports remain idempotent: the same package does not duplicate owned assets or append duplicate import rows.

**Evidence anchors:**

- `crates/z00z_wallets/src/rpc/error_mapping.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_support.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_lifecycle.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_finalize.rs`
- `crates/z00z_wallets/src/rpc/tx_rpc_server_helpers.rs`
- `crates/z00z_wallets/src/rpc/tx_types.rs`
- `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs`
- `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
- `crates/z00z_wallets/tests/test_asset_replay_protection.rs`
- `crates/z00z_wallets/tests/test_import_error_taxonomy.rs`
