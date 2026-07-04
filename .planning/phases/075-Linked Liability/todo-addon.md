Проверил глубоко. Короткий вывод: **075 связан с 067-072 концептуально, но до сегодняшней правки связь не была явно зафиксирована в самом 075**. Я добавил в Z00Z-Linked-Liability-Whitepaper.md секцию `10.4 Cross-Phase Compatibility With 067-072` и таблицу `Fraud Flow Closure Status`, чтобы не потерять traceability.

**Что происходит при fraud**
По 075 flow такой:

1. Локально/оффлайн возникает конфликт: duplicate spend, ACK mismatch, missed deadline, duplicate claim, policy breach.
2. Конфликтная evidence доходит до settlement/checkpoint сравнения.
3. Строится `FraudProof`.
4. Валидный proof активирует `LiabilityDomain`.
5. Пишется `LockRegistry` entry.
6. `BondRef` freeze/slash, `PenaltyPolicy` решает penalty/compensation/unlock.
7. Future rights из того же domain reject/quarantine.
8. Case закрывается через penalty paid, victims compensated, dispute closed, proof invalidated, cooldown, unlock/retire.

**Где это пишется/хранится сейчас**
Фундамент есть в 067-072:

- 067: quorum/commit subject/publication evidence, но не guilt.
- 068: checkpoint artifact/link/exec input, archive manifest, DA ref, publication evidence, challenge/dispute statuses.
- 069: recursive sidecars must not delete raw packages/witness/proof/archive evidence.
- 070: rollup DA/archive/retrieval/challenge windows; provider receipts are support evidence only.
- 071: request-bound inbox is advisory and privacy-sensitive; cannot mutate wallet or open liability.
- 072: canonical `TxPackage`, `OfflineTxBundleV1` wrapper, package conflicts, storage apply, theorem-bound checkpoint validation.

Но 075 still needs its own canonical liability objects.

**Что закрыто**
- Концептуальный fraud lifecycle.
- Object model: `LiabilityDomain`, `HiddenLiabilityCommitment`, `FraudProof`, `BondRef`, `PenaltyPolicy`, `LockRegistry`.
- Privacy constraints: selective reveal, exculpability, domain-scoped freeze.
- Cross-phase authority boundaries now documented.
- Maturity honesty: doc clearly says full pipeline is target architecture, not live implementation.

**Что подвешено**
- Нет `FraudEvidenceRecordV1`.
- Нет canonical `FraudProofV1` wire format / canonical bytes / reject reasons.
- Нет `FraudProofVerifier` implementation contract tied to checkpoint/package/archive evidence.
- Нет durable `LockRegistryEntryV1` owner/path/storage contract.
- Нет `BondDebitRecordV1`, `PenaltySettlementRecordV1`, `CompensationReceiptV1`.
- Не закрыт payment flow: кто платит штраф, каким объектом, как proof of payment validates, что с partial/failed payment.
- Нет explicit bridge между checkpoint finality и liability case finality.
- Нет retention/pruning blocker для liability evidence.
- Нет wallet/storage enforcement hook: future-right freeze must be consulted before package creation/import/apply.
- Phase 070 path inconsistency remains external: actual folder is 70-Rollup-Node, while docs call it `070 Rollup Node`.

**Рекомендация**
Делать **2 implementation phases**, не одну.

Phase 075A: **Fraud Evidence Bridge**

- Define `FraudEvidenceRecordV1`.
- Define `FraudProofCandidateV1` / `FraudProofV1` shell.
- Map conflict evidence to 067-072 surfaces.
- Store references to checkpoint, archive, DA publication, package digest, request/ACK anchor, leakage contract.
- Tests: two conflicting offline packages produce one retained evidence record; fake/single/incomplete evidence rejects; retained evidence survives archive policy.

Phase 075B: **Liability Case And Penalty MVP**

- Implement `LockRegistryEntryV1`.
- Add domain-scoped future-right rejection.
- Add `PenaltySettlementRecordV1`, `BondDebitRecordV1`, `CompensationReceiptV1`.
- Define unlock/release/retire transitions.
- Tests: valid proof opens lock; unrelated domain still works; locked domain future spend rejects; penalty+compensation unlocks; invalidated proof clears.

