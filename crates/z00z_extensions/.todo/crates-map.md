Сейчас сведу только те имена, которые уже закреплены в repo/planning, и отброшу все мои прошлые нормализации. Результат будет жёстко repo-backed: где имя crate зафиксировано, а где зафиксирован только module family.

**2. Celestia DA stack: имя crate уже зафиксировано**

- planned z00z_da_celestia:
  dedicated Celestia adapter extraction
  Роль: отдельный Celestia-specific DA provider crate.
  Источник: 097-system-design.md, 120-rollup-node.md, z00z_top-level-design-v1.md

- Что остаётся в z00z_rollup_node до/после extraction:
  da_adapter
  node_api
  mode wiring
  lifecycle
  Источник: rollup node lib, 097-system-design.md

- Что связано с DA/publication на runtime side:
  agg_publish
  da_iface
  pub.sequencer
  verify.stateless
  Источник: 097-system-design.md, 19_z00z-modules.md

Итог по Celestia без догадок:
- wiring/composition: z00z_rollup_node
- dedicated provider crate: z00z_da_celestia
- runtime semantics around publication/verification: z00z_runtime/aggregators и z00z_runtime/validators

**3. Offline subsystem: top-level crate name в repo ещё не заморожено**

Здесь repo фиксирует module family, но не окончательный workspace crate name.

Подтверждённые module families:
- offline.tee:
  reserve_issue, counters, attest
- offline.redeem:
  approval, policy
- offline.dispute:
  conflict_check, merchant_checks
- offline.wallet:
  verify_voucher, relay

Источник: 19_z00z-modules.md

Подтверждённые repo-native candidate names, которые уже встречаются в docs:
- z00z_offline_cash
  Источник: Z00Z-Ideas-From-Articles.md
- z00z_offline_vouchers
  Источник: Z00Z-Ideas-From-Articles.md

Поэтому без догадок итог такой:
- верхний crate name: не зафиксирован окончательно
- зафиксированные module families: offline.tee, offline.redeem, offline.dispute, offline.wallet
- repo-native naming candidates: z00z_offline_cash или z00z_offline_vouchers

**4. Committee / BLS / finality subsystem: top-level crate name тоже не зафиксирован**

Подтверждённые module families:
- committee.bls:
  keyset, threshold_sig
- verify.stateless:
  bls_committee
- checkpoints.service:
  build_epoch, sign_threshold

Источник: 19_z00z-modules.md

Итог без догадок:
- верхний crate name: не зафиксирован
- зафиксированная family name: committee.bls
- зафиксированные adjacent modules: verify.stateless, checkpoints.service

То есть сейчас честная карта здесь такая:
- validators-side hook family:
  verify.stateless
  checkpoints.service
- separate authority family:
  committee.bls

**5. Lockers / bridge-control subsystem: crate names не зафиксированы, но split model зафиксирован**

Подтверждённые repo-native сущности:
- Locker
- LockerID
- LockerVault
- Z00ZBridge
- Z00Z_CrossChain_Integration

Источники:
- What Is Genuinely New in Z00Z - short.md
- What Is Genuinely New in Z00Z-long.md
- Z00Z-Incentive-Games.md

Подтверждённая split architecture:
- Внутри Z00Z:
  locker-note ownership semantics
  LockerID → owner_pubkey
  locker state in roots/checkpoints
- Внешняя chain/integration side:
  LockerVault contract
  Z00ZBridge deposit/withdraw flow
  threshold or rollup-style bridge authorization

Итог без догадок:
- верхний crate name для internal locker family: не зафиксирован
- верхний crate name для external bridge family: не зафиксирован
- зафиксированные nouns/modules:
  Locker
  LockerVault
  Z00ZBridge
  Z00Z_CrossChain_Integration

**6. Финальная crate map в строгой форме**

Жёстко зафиксировано:
- z00z_rollup_node:
  config, da_adapter, lifecycle, mode, rpc, status
- z00z_runtime/aggregators:
  agg_ingress, agg_ordering, agg_scheduler, agg_recovery, agg_publish
- z00z_runtime/validators:
  val_engine, claim_pkg_verify, spend_rules, checkpoint_flow, verify.stateless, checkpoints.service
- z00z_runtime/watchers:
  watcher_engine, anti_fraud_policy
- z00z_storage:
  claim_nulls, snapshot_sync, history_compact, object_reconcile
- z00z_telemetry:
  telemetry_ops
- planned z00z_da_celestia:
  dedicated Celestia adapter extraction

Зафиксированы только module families, но не final crate names:
- offline:
  offline.tee, offline.redeem, offline.dispute, offline.wallet
- committee/finality:
  committee.bls, verify.stateless, checkpoints.service
- lockers/cross-chain:
  Locker, LockerVault, Z00ZBridge, Z00Z_CrossChain_Integration

Если свести это к одному правилу:
- если имя top-level crate уже есть в planning и повторяется в design flow, его можно считать зафиксированным
- если в repo есть только module family или product noun, top-level crate name пока не надо придумывать

Самый короткий итог по твоим четырём вопросам:
1. Celestia DA:
   z00z_rollup_node + planned z00z_da_celestia
2. Offline vouchers:
   crate name не frozen, family frozen как offline.tee / redeem / dispute / wallet
3. Lockers / bridge:
   crate names не frozen, nouns frozen как Locker / LockerVault / Z00ZBridge
4. Committee / BLS / finality:
   crate name не frozen, family frozen как committee.bls + checkpoints.service + verify.stateless

Если хочешь, следующим сообщением я могу сделать ещё более практичную версию:
- “что уже можно создавать как реальные workspace crates прямо сейчас”
- и отдельно
- “что пока нельзя называть top-level crate, потому что repo ещё не закрепил имя”.

---



Разделю ответ на две части: сначала коротко про BLS, потом про то, возможен ли для Z00Z-like системы запуск с последующей передачей community при pure PoS и что это реально меняет по рискам.

Ran terminal command:  ./scripts/play_tone.sh

**BLS**

BLS обычно означает **Boneh–Lynn–Shacham signatures**.  
Это схема цифровой подписи, у которой для блокчейн-систем особенно важны 3 свойства:

1. Подписи короткие.  
Они компактнее многих других схем.

2. Подписи можно агрегировать.  
Если 100 валидаторов подписали один и тот же checkpoint, можно собрать это в одну агрегированную подпись вместо 100 отдельных.

3. Удобно для committee/finality layer.  
Именно поэтому BLS часто предлагают для:
- `k-of-n` finality
- committee certificates
- checkpoint signatures
- threshold-style approval

В твоём контексте Z00Z это нужно не для “обычных user signatures”, а для слоя:
- `committee.bls`
- `sign_threshold`
- `finality_cert`
- `RedeemApproval`-подобных коллективных подтверждений

То есть очень грубо:  
**Schnorr/обычные подписи** удобны для пользовательских действий,  
**BLS** удобен для коллективной подписи комитета.

---

