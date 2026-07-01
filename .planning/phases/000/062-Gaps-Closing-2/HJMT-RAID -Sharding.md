Ниже — архитектурный вывод по **Z00Z-HJMT-Upgrade** без лишнего “дизайнерского обрамления”. Документ анализировал как нормативную upgrade-spec, не как описание уже готовой реализации.

## 1. Можно ли сделать агрегаторы как RAID: один падает, другие seamless берут его функции, включая sharding?

**Да, но только в узком смысле: как hot-standby / replicated executor для того же `ShardId`, с той же `routing_generation` и той же journal-lineage. Нет — если имеется в виду “любой другой агрегатор автоматически начинает обслуживать чужой shard без committed route migration”.**

В документе это ограничение сформулировано очень жёстко. Агрегатор не является источником истины. Источник истины — committed route generation, published shard-leaf set и accepted public checkpoint. Machine discovery, gossip, executor placement и scheduler-local topology могут решать, **где выполняется код**, но не могут менять shard ownership или verifier-visible state.

Правильная RAID-аналогия такая:

**RAID-like failover допустим на уровне исполнения**, когда standby-агрегатор продолжает **тот же shard**, а не создаёт новую shard-authority. Standby должен восстановиться под тем же `ShardId`, той же `routing_generation` и той же journal-lineage. Тогда это не migration, а lawful failover. Документ прямо говорит, что hot-standby или replicated executor может взять failed shard только при сохранении этих identity/journal boundaries.

**RAID-like “любой берет любого” запрещён на уровне routing/sharding.** Если aggregator/shard executor падает, batches, routed to failed shard, должны вернуть retryable `shard-unavailable`, и система **не должна silently reroute** их в той же routing generation. Другие shards могут продолжать local commits и public checkpoints, но для failed shard они должны carry-forward последний опубликованный `ShardRootLeafV1` byte-for-byte unchanged. Перенос трафика away from failed shard требует новой committed `ShardRouteTableV1` generation и normal route migration.

То есть “seamless для всей системы” делится на два случая:

**Для unaffected shards — да.** Если упал Aggregator B, Aggregators A и C могут продолжать работу. Следующий public checkpoint может включить новые leaves A/C, а leaf B оставить прежним. Это сохраняет public truth и ограничивает blast radius только unpublished progress failed shard.

**Для failed shard — не fully seamless, если нет prepared standby.** Новые batches этого shard должны получить retryable unavailable. Unpublished local state должен быть recovered from durable shard journal до публикации нового shard leaf. Silent takeover другим aggregator без same-lineage запрещён.

**Для добавления/удаления aggregator seamless только если это placement change, а не shard ownership change.** Если Aggregator D подключается как hot-standby existing shard — route table не меняется, нового shard не появляется, D просто mirrors/replays same shard journal lineage. Другие участники не “узнают новый shard”, потому что protocol-level ownership не изменилось. Если D получает новый traffic slice / новый shard — это уже route migration: нужна новая `ShardRouteTableV1`, activation checkpoint и public checkpoint, после которого другие видят D-owned traffic through committed protocol state.

**Вывод:** архитектура может поддержать отказоустойчивые агрегаторы, но не как свободный RAID-router. Правильная модель:
`ShardId` ownership стабилен → executor может failover only with same lineage → permanent reassignment only through committed route migration.
Если требуется “любой aggregator мгновенно берет любой shard без unavailable”, этого в документе нет и это противоречит fail-closed routing model.

## 2. Где сидит batch planner, который организует batches и направляет их к нужному aggregator? Есть ли архитектура под это?

**Да, архитектура есть, но placement planner-а намеренно не зафиксирован как один обязательный процесс.**

В документе planner описан как **deterministic batch-admission component**, который стоит **перед shard-local queues** и создаёт `BatchPlanned` journal record. Его обязанности: canonicalize operations, reject duplicate settlement paths, вычислить `SettlementPath -> route_hash -> ShardId` через committed route table, сгруппировать candidate batch by `ShardId`, reject batch если после grouping остался не ровно один shard, и emit canonical operation digest для journal/delta/proof/recovery replay.

Ключевой момент: документ разделяет **protocol responsibility** и **process placement**. Planner может быть colocated with storage leader, может работать внутри aggregator ingress path, либо быть отдельным coordinator process. Но accepted batch определяется не local scheduler policy, а committed route table, canonical operation order и `BatchPlanned` record.

То есть архитектура такая:

```text
Client / caller
  -> Aggregator ingress или coordinator
  -> deterministic batch planner
      - canonicalize
      - route lookup via ShardRouteTableV1
      - group by ShardId
      - reject cross-shard batch in v1
      - emit BatchPlanned
  -> shard-local queue / shard executor
  -> shard-local journal + local HJMT commit
  -> publication boundary
  -> CheckpointPublicationV1
  -> public SettlementStateRoot
```

Важное разделение: **planner решает, что входит в shard-local batch; publication boundary решает, какие shard-local roots становятся public settlement state.** Эти две функции нельзя схлопывать в runtime heuristic. Planner не решает public visibility. Public state changes only when `CheckpointPublicationV1` is accepted.

Документ также даёт prescriptive module ownership: deterministic planner contract должен жить в `crates/z00z_storage/src/settlement/hjmt/hjmt_plan.rs`; runtime aggregator orchestration — в `z00z_runtime/aggregators` modules; validator checkpoint acceptance — в `z00z_runtime/validators`; watchers — в `z00z_runtime/watchers`. Но сам документ подчёркивает, что это design ownership map, а не claim, что текущий repository уже всё реализует.

**Вывод:** batch planner в архитектуре есть. Он логически сидит перед shard queues. Физически он может быть внутри aggregator ingress, рядом со storage leader или отдельным coordinator. Главное — его output должен быть deterministic, committed-route-based и journaled через `BatchPlanned`.

## 3. Какие архитектурные особенности upgrade реально нуждаются в доработке, а не в косметике?

Общая оценка: **ядро архитектуры правильное и не требует придумывать новые большие слои.** Самые сильные решения уже заложены: stable `ShardId` above mutable `BucketId`, committed route table, per-shard journal, root-of-shard-roots, checkpoint publication, fail-closed proof/root contracts, separation of planner and publication boundary. Это не косметика, это правильный skeletal architecture.

Но документ сам честно говорит: он сейчас в состоянии **specified contract**, а не release-ready implementation. Readiness требует implementation + evidence: deterministic `BatchProofBlob`, negative tests, durable TPS, separation `ShardId/BucketId`, recoverable per-shard journal, root-of-shard-roots migration vectors, adaptive transition proofs, benchmarks and evidence packets.

Существенные доработки такие.

### A. Реальный failover aggregator-а надо довести до protocol-safe runtime, а не оставить prose-level rule

Сейчас правило правильное: same `ShardId`, same `routing_generation`, same journal lineage; иначе route migration. Но для production этого мало. Нужны:

```text
- replicated / mirrored shard journal;
- standby resume protocol;
- wrong-lineage rejection;
- shard-unavailable behavior;
- emergency freeze behavior;
- carry-forward unchanged shard leaf;
- recovery proof that no synthetic mixed root is exposed.
```

Документ прямо требует failover vectors: hot-standby resume under same `ShardId` and same journal lineage, reject wrong journal lineage, reject same local root replayed under wrong `routing_generation`. Также нужны carry-forward vectors for unchanged shard leaf, shard-leaf reordering rejection и mixed old/new child roots rejection during recovery.

Это **не косметика**. Без этого “RAID-like aggregators” останутся небезопасной operational идеей.

### B. Root-of-shard-roots требует реального изменения storage root model

В документе написано, что current code exposes live settlement generation as `RootGeneration::SettlementV1`, а existing root structures do not yet store shard-root leaves. Для generation 1 нужно добавить explicit versioned representation before any proof/root can be accepted as root-of-shard-roots.

Это фундаментальная доработка:

```text
RootGeneration0:
  SettlementStateRoot = current HJMT semantic root

RootGeneration1:
  SettlementStateRoot = Merkle commitment over ShardRootLeaf records
```

Нужны `ShardRootLeafV1`, route-table digest binding, policy-set digest, journal checkpoint, local sequence, transition flags, global shard-root inclusion proof + shard-local HJMT proof. Это не можно заменить косметическим wrapper-ом.

### C. ShardRouteTableV1 и route migration должны стать byte-deterministic

Документ уже задаёт правильную модель: routing через committed route table over deterministic route hashes, а lookup не должен зависеть от load, queue depth, cache, wall-clock или local config.

Но для настоящей реализации нужны golden vectors и migration vectors. Без route-table vectors aggregator join/leave может быть conceptually clear, но byte-contract останется nondeterministic между independent implementations.

Это существенная доработка, потому что sharding authority зависит именно от canonical bytes route table, а не от runtime discovery.

### D. Cross-shard operations сейчас intentionally unsupported

Документ говорит: если batch touches paths in different shard IDs, planner must reject; implementation must not invent distributed transaction protocol between aggregators as implicit fallback.

Это нормально для v1. Но если продуктово нужны atomic операции между assets/rights в разных shards, тогда это **настоящая архитектурная недоделка scope-а**, а не баг. Понадобится отдельная архитектура receipts / cross-shard transaction protocol / two-phase publication semantics. В текущем upgrade этого нет, и документ правильно запрещает “hope-based locking multiple shard queues”.

### E. Batch multiproof — ещё не implementation, текущий batch proof API остаётся independent `Vec<ProofBlob>`

Документ фиксирует, что текущий `settlement_proof_blobs` returns `Vec<ProofBlob>`, то есть это independent proof generation, not shared multiproof. Required implementation — `BatchProofBlobV1` with shared context, path table, witness DAG, opening table, reference table.

Реально надо сделать:

```text
- canonical binary encoding;
- parser limits before allocation-heavy work;
- atomic verification;
- no partial acceptance;
- witness reuse only under complete reuse key;
- golden vectors and tamper vectors;
- benchmarks vs Vec<ProofBlob>.
```

Документ даже рекомендует first implementation: build `BatchProofBlobV1` from already verified independent `ProofBlob` values, then later move builder closer to HJMT internals for native witness reuse.

Это существенная proof-layer работа.

### F. Durable TPS надо мерить на public-root durable boundary, не на scheduler throughput

Документ предупреждает: current scheduler provides bounded parallel work, but that is not durable TPS evidence. Commit path still includes parent recomposition, path-index work, cache work, model history, RedB journal sync, root publication.

Нужны bucket-local batch deltas, parent deltas, staged journal, crash recovery at every boundary, and reports separating worker-local, tree-durable, public-root durable, auxiliary timings. Only public-root durable timing supports wallet/checkpoint/external verifier TPS claims.

Это не косметика. Без этого можно получить красивый worker TPS, но не получить реальный durable settlement TPS.

### G. CheckpointPublicationV1 validator gate должен быть implemented fail-closed

Документ определяет, что every active shard from committed route table must appear exactly once; unchanged shard carries forward previous `ShardRootLeafV1` bytes unchanged; root generation 1 public root is computed only from ordered shard leaves; no subset/reorder/duplicate shard leaves. Monotonicity rules for checkpoint, shard_epoch, local_sequence, journal_checkpoint also exact.

Значит нужен validator-side acceptance gate:

```text
- exact shard-set coverage;
- sorted shard leaves;
- no duplicate shard;
- route_table_digest binding;
- prior_public_root continuity;
- monotonic checkpoint validation;
- carry-forward byte equality;
- reject synthetic mixed roots.
```

Это архитектурная safety surface, не UX.

### H. Adaptive transitions пока имеют record shapes, но не полную verifier/recovery semantics

Документ говорит, что repository already contains record shapes for adaptive transitions, но upgrade work is to define verifier semantics, recovery behavior and benchmark evidence. Split/Merge/PolicyTransition должны доказывать exact leaf preservation, bounded local transition, root-bound policy/epoch, historical proof safety.

Это важно, потому что bucket layout can change, но `SettlementPath` и `ShardId` не должны меняться. Если adaptive split/merge будет implemented как storage optimization без proof/recovery semantics, он может сломать historical proofs или route identity.

### I. Historical metadata retention — обязательная часть, не архивная опция

Old proofs must verify under their historical route generation, policy, epoch and root generation; current node configuration must never reinterpret historical proof. Документ требует migration vectors: last gen0 root, one-shard route table, initial shard leaf, first gen1 root, historical proof sample, rejection sample.

Это существенная доработка, потому что route migration, bucket policy transition и root generation migration без historical metadata создадут “живую” систему, которая не может проверять старые доказательства.

### J. Observability для shard-stall/freeze/route-dispute нужна, но не должна стать source of truth

Watchers/status/evidence-export нужны для эксплуатации: shard stall, freeze mode, route-table dispute, recovery evidence. Но они не должны становиться consensus truth. Документ относит operational observability в runtime watchers, а source of truth оставляет committed route generation, shard-leaf set и accepted checkpoint.

Это не core consensus, но для RAID-like эксплуатации без этого будет трудно понять, когда shard failed, frozen, recovering или disputed.

## Итоговая оценка

**Архитектурно документ в целом правильный.** Я бы не добавлял новых “умных” слоёв, dynamic load-balancer routing, silent shard takeover или implicit distributed transactions. Они прямо противоречат design philosophy документа.

**Главная реальная доработка:** превратить спецификацию в deterministic, tested, recoverable implementation. Самые важные workstreams:

```text
1. BatchProofBlobV1 codec/verifier/golden+negative vectors.
2. ShardRouteTableV1 canonical bytes + route migration vectors.
3. ShardRootLeafV1 + CheckpointPublicationV1 + root generation storage model.
4. Per-shard journal + lawful same-lineage failover + crash matrix.
5. Deterministic planner before shard queues with strict cross-shard rejection.
6. Validator checkpoint acceptance gate.
7. Durable TPS benchmark at public-root boundary.
8. Adaptive transition proofs with historical-proof safety.
```

**Ответ на главный RAID-вопрос:** можно сделать отказоустойчивость как replicated shard executor / hot standby, но нельзя сделать “RAID-style transparent reroute” без committed metadata. Seamless подключение/удаление агрегатора возможно только как executor placement change. Если меняется shard ownership или traffic slice, это route-table generation migration через public checkpoint, а не скрытая runtime операция.



---

---



```
Мой вопрос, он такой: могу ли я обязать каждый из физических агрегаторов машин работать с несколькими шардами, так чтобы, если падает какой-то агрегатор, то всегда есть его бэкап, другие агрегаторы могут взять на себя работу этого шарда. То есть возможно ли такая архитектура? В этом смысле я имел в виду seamless продолжение работы и, собственно, райд-архитектуру.
```

Да — **такая архитектура возможна**, и именно это ближе всего к корректной “RAID-like” модели для агрегаторов. Но важно назвать её правильно:

**Не “агрегатор владеет шардом”, а “физическая машина запускает один или несколько shard-executor’ов для committed `ShardId`”.**
Физический aggregator может обслуживать много shard-executor’ов, а каждый логический `ShardId` может иметь primary + hot-standby / replicas на других физических агрегаторах.

То есть архитектура может быть такой:

```text
Logical shards:
  Shard A
  Shard B
  Shard C
  Shard D

Physical aggregators:

  Agg-1:
    primary executor for Shard A
    standby executor for Shard C

  Agg-2:
    primary executor for Shard B
    standby executor for Shard A

  Agg-3:
    primary executor for Shard C
    standby executor for Shard B

  Agg-4:
    primary executor for Shard D
    standby executor for Shard A / B / C
```

Если падает `Agg-1`, то standby для `Shard A` на `Agg-2` может подняться как active executor **без изменения `ShardId`**. Для протокола это не новый shard и не новая authority. Это просто смена физического места исполнения.

Документ это разрешает: hot-standby или replicated executor может взять failed shard, но обязан продолжить **тот же `ShardId`, ту же `routing_generation` и ту же journal lineage**. Иначе это уже не failover, а route migration.

## Главное условие: резервируется не “агрегатор”, а shard journal lineage

Чтобы failover был seamless, standby должен иметь доступ к той же истории shard-а:

```text
ShardId = B
routing_generation = 7
journal_lineage = JournalB
last_public_leaf = Leaf_B_101
durable local checkpoint = 221
local_sequence = 41
```

Standby может продолжить только если он может доказуемо восстановить состояние из того же durable shard journal. Документ прямо говорит: unpublished shard-local state failed shard должен быть recovered from durable shard journal before any new shard leaf can be published.

Практически это означает, что тебе нужен один из вариантов:

```text
Вариант 1 — shared durable journal:
  все replicas shard-а читают/пишут в один replicated journal layer.

Вариант 2 — replicated journal:
  primary append-ит journal records, standbys получают их синхронно или почти синхронно.

Вариант 3 — consensus group на shard:
  shard имеет маленькую replication group, например 3 machines,
  и journal append принимается quorum-ом.
```

Самый безопасный вариант — **per-shard replicated log / journal**, а не просто “копия агрегатора”.

## Да, можно обязать каждый физический агрегатор работать с несколькими шардами

Это не противоречит документу. Документ даже говорит, что machine discovery, peer gossip, executor placement или scheduler-local topology могут решать, **где выполняется код**, но не могут решать public truth. То есть физическое размещение executor-ов свободно, пока protocol-visible ownership остаётся через `ShardRouteTableV1`, `ShardRootLeafV1` и `CheckpointPublicationV1`.

Иными словами, можно сделать:

```text
каждая physical aggregator machine:
  - держит primary executor для N shards;
  - держит standby executor для K других shards;
  - умеет принимать routing только для тех ShardId, которые ей назначены;
  - не меняет ShardId на основании load / availability;
  - не объявляет себя owner-ом без committed route/checkpoint authority.
```

Это будет корректная архитектура.

## Что значит “seamless” в допустимом смысле

**Seamless да**, если:

```text
Agg-1 падает;
Agg-2 уже является standby для Shard A;
Agg-2 имеет тот же journal lineage;
Agg-2 продолжает тот же ShardId;
routing_generation не меняется;
новый ShardRootLeaf публикуется под той же shard identity.
```

В таком случае для protocol/public root это выглядит как продолжение того же shard-а, а не как миграция ownership. Документ прямо описывает lifecycle: Aggregator D может join как hot-standby existing shard, committed route table остаётся unchanged, D mirrors/replays same shard journal lineage, и при failover обязан resume same `ShardId`, same `routing_generation`, same journal lineage.

## Что нельзя делать

Нельзя сделать так:

```text
Shard A был логически назначен на ShardId=A.
Agg-1 упал.
Agg-2 взял входящие операции Shard A и silently направил их в Shard B
или создал новый локальный root без той же journal lineage.
```

Это документ запрещает. Planner не должен использовать live queue depth, node affinity, peer availability или local load heuristics, чтобы менять shard ownership внутри одной routing generation. Эти факторы могут решать, **какая машина исполняет уже назначенную shard queue**, но не могут менять, какому `ShardId` принадлежит batch.

Также нельзя “тихо” перенаправить traffic failed shard-а в другой shard внутри той же route generation. Если standby не может продолжить ту же lineage, новые batches должны получить retryable `shard-unavailable`, либо должна быть сделана explicit route migration через новую `ShardRouteTableV1` и checkpoint.

## Правильная модель: RAID1/RAID10, не RAID0

Твоя идея корректна, если её формализовать так:

```text
Logical shard = protected unit.
Physical aggregator = execution host.
Shard journal = source of recoverable continuity.
Route table = source of ownership.
Checkpoint = source of public visibility.
```

Ближе всего это к:

```text
RAID1:
  один shard mirrored на нескольких physical aggregators.

RAID10:
  много shards distributed across machines,
  каждый shard имеет replica / standby на другой машине.
```

Но это **не RAID0-style transparent striping**, где при падении одной машины данные просто “растекаются” по другим без committed metadata. В Z00Z так нельзя, потому что route authority и public settlement truth должны оставаться deterministic.

## Минимальная production-схема

Я бы зафиксировал такую архитектуру:

```text
ShardRouteTableV1:
  route_hash range -> ShardId

ShardPlacementTable, non-consensus / operational:
  ShardId -> primary AggregatorId
  ShardId -> standby AggregatorIds[]

Per-shard journal:
  ShardId
  routing_generation
  journal_lineage_id
  local_sequence
  journal_checkpoint
  local_pre_root
  local_post_root

Aggregator runtime:
  runs many ShardExecutor instances

Failover controller:
  detects primary loss
  fences old primary
  promotes standby
  verifies same journal lineage
  resumes from last durable checkpoint
```

`ShardPlacementTable` может быть operational layer, но **не consensus truth**. Истина всё равно остаётся в committed route generation, published shard leaves и accepted public checkpoint. Документ прямо формулирует это как self-check: aggregator ownership is not source of truth; join/leave агрегатора сводится либо к executor failover без `ShardId` change, либо к explicit route migration.

## Самый важный риск: split-brain

Если один и тот же `ShardId` одновременно пишет два active executor-а без fencing/consensus, ты получишь две разные ветки journal lineage:

```text
Agg-1 thinks:
  Shard A local_sequence 42 -> root R1

Agg-2 thinks:
  Shard A local_sequence 42 -> root R2
```

Так делать нельзя. Поэтому нужна одна из защит:

```text
- strict primary lease;
- fencing token;
- quorum journal append;
- deterministic leader election per ShardId;
- reject if journal_lineage_id / routing_generation / local_sequence mismatch.
```

Документ требует failover vectors: hot-standby resume under same `ShardId` and same journal lineage, rejection on wrong journal lineage, and rejection of same local root replayed under wrong `routing_generation`. Это как раз защита от split-brain и неправильного takeover.

## Итог

**Да, ты можешь построить RAID-like архитектуру, где каждый физический агрегатор обслуживает несколько shard-ов и держит backup для других shard-ов.**

Но правильное правило такое:

```text
Можно:
  физический aggregator A упал;
  физический aggregator B продолжает тот же logical ShardId;
  B использует ту же routing_generation;
  B использует ту же journal lineage;
  public checkpoint позже публикует новый ShardRootLeaf для того же ShardId.

Нельзя:
  silently переназначить batch на другой ShardId;
  менять ownership через gossip/discovery/load balancer;
  публиковать synthetic root;
  считать physical aggregator source of truth.
```

Так что ответ: **да, архитектура возможна и хорошо ложится на документ, но только как replicated shard-executor / hot-standby architecture.** Это не свободное динамическое “кто угодно берёт что угодно”, а строгое multi-shard placement поверх committed shard identity.



```
Какой вариант ты рекомендуешь? Вариант 1 — Sharded Durable Journal, вариант 2 — Replicated Journal и вариант 3 — Consensus Group на shard. Какой ты мне посоветовал номер?
```

Я бы рекомендовал:

```text
RAID-модель: RAID10
Journal-модель: №3 — Consensus Group на shard
```

Но с важной оговоркой: **№3 лучше как production-цель**, а **№2 можно использовать как промежуточный MVP**, если нужно быстрее реализовать.

## 1. RAID1 или RAID10?

**Рекомендую RAID10.**

Не RAID1, потому что RAID1 означает почти полное зеркалирование: каждый shard или весь набор shard-ов дублируется как единый блок. Это проще, но плохо масштабируется: ты платишь много storage/CPU и не получаешь нормального распределения write-load.

Тебе нужна такая модель:

```text
Shard A: primary на Agg-1, standby на Agg-2 / Agg-3
Shard B: primary на Agg-2, standby на Agg-3 / Agg-4
Shard C: primary на Agg-3, standby на Agg-1 / Agg-4
Shard D: primary на Agg-4, standby на Agg-1 / Agg-2
```

Это и есть **RAID10-подобная модель**:

```text
RAID0 часть:
  нагрузка размазана по разным shard-ам и физическим агрегаторам

RAID1 часть:
  каждый shard имеет одну или несколько реплик / standby executor-ов
```

Документ хорошо ложится именно на такую модель, потому что он требует, чтобы unrelated shards могли независимо принимать, journal-ить, recover-ить, prove-ить и publish-ить pending state, не ожидая hot shard кроме configured global publication boundary. То есть сама логика документа — не “всё зеркалировать целиком”, а **разделить state на shard-local domains и резервировать каждый domain отдельно**.

## 2. Какой journal-вариант выбрать?

**Моя рекомендация: №3 — Consensus Group на shard.**

То есть:

```text
каждый ShardId имеет свою маленькую replication/consensus group;
journal append считается durable только после quorum;
standby executor может стать active только если он в той же journal lineage;
старый primary должен быть fenced, чтобы не было split-brain.
```

Почему именно №3: документ требует не просто “копию состояния”, а восстановление failed shard-а из durable shard journal перед публикацией нового shard leaf. Он также прямо говорит, что hot-standby или replicated executor может взять shard только если продолжает тот же `ShardId`, ту же `routing_generation` и ту же journal lineage.

А это практически означает: нужен механизм, который гарантирует единственную accepted journal-lineage. Самый чистый способ — **per-shard consensus group**.

## Почему не №1?

Если №1 означает просто:

```text
Shard A journal лежит отдельно
Shard B journal лежит отдельно
Shard C journal лежит отдельно
```

то это хорошая база, но **не полноценная failover-архитектура**. Она даёт sharded durability, но сама по себе не решает:

```text
- кто имеет право писать следующий journal record;
- что делать, если old primary ожил;
- как защититься от двух writers на один ShardId;
- как доказать, что standby продолжает ту же lineage;
- как избежать synthetic mixed root.
```

Документ требует, чтобы после failure система публиковала либо prior visible public root, либо exact later root reconstructed from durable journal state, и не публиковала synthetic mixed root.

Один “sharded durable journal” без quorum/fencing может оказаться single point of failure или split-brain source. Поэтому **№1 годится как storage layout**, но не как полноценная HA/failover модель.

## Почему не №2 как финальный вариант?

№2 — Replicated Journal — лучше, чем №1. Его можно сделать так:

```text
primary пишет journal;
standby получает копии;
при падении primary standby replay-ит journal и продолжает.
```

Это рабочий MVP, но слабое место — **момент признания записи durable**.

Если replication asynchronous, то при падении primary standby может отстать:

```text
Primary accepted operation X
Primary crashed
Standby не получил X
```

Тогда либо клиент думает, что X принят, а shard recovery его не видит, либо ты должен усложнять acknowledgement semantics.

Если replication synchronous, то ты фактически приближаешься к quorum/consensus. Поэтому я бы не делал №2 финальной архитектурой, а использовал бы его только как этап:

```text
MVP:
  №2 replicated journal with strict fencing and sync replication

Production:
  №3 per-shard consensus group
```

Документ требует failover vectors: hot-standby resume under same `ShardId` and same journal lineage, reject wrong journal lineage, reject same local root replayed under wrong `routing_generation`. Это ближе к consensus/quorum discipline, чем к простой репликации.

## Рекомендуемая финальная схема

Я бы зафиксировал так:

```text
Physical layer:
  Aggregator machine runs multiple ShardExecutors.

Routing layer:
  ShardRouteTableV1 maps route_hash -> ShardId.
  Aggregator placement does not change shard ownership.

Durability layer:
  Each ShardId has its own consensus journal group.

Execution layer:
  one active writer per ShardId;
  N standby executors can replay/follow the same journal;
  promotion requires fencing token / quorum leadership.

Publication layer:
  ShardRootLeafV1 binds shard-local root to ShardId,
  routing_generation,
  route_table_digest,
  policy_set_digest,
  journal_checkpoint.

Global root:
  CheckpointPublicationV1 publishes root-of-shard-roots.
```

То есть:

```text
RAID10 over shard executors
+
№3 consensus journal per shard
```

## Конкретная recommendation

**Финальный production-вариант:**

```text
RAID10 + Вариант №3
```

**Практический rollout:**

```text
Phase 1:
  one-shard compatibility mode;
  deterministic ShardRouteTableV1;
  per-shard journal abstraction.

Phase 2:
  RAID10 placement:
    each aggregator has primary shards + standby shards.

Phase 3:
  №2 replicated journal as temporary implementation,
  but only with strict sync ack + fencing.

Phase 4:
  replace / harden into №3:
    consensus group per ShardId,
    quorum append,
    leader lease,
    wrong-lineage rejection,
    recovery vectors.
```

Мой окончательный совет: **не строить финальную систему на №1 и не останавливаться на №2. Делать архитектурную цель №3.**
№2 можно использовать как промежуточный путь, но в design document я бы сразу записал target как:

```text
Per-Shard Consensus Journal Group
with RAID10-style ShardExecutor Placement
```



---

---



Да, **ты понял направление правильно, но формулировку надо чуть поправить**.

Выигрыш TPS в Upgrade идёт **не от того, что “батч целиком становится одним leaf в HJMT”**. Правильнее так:

> **Батч становится единицей commit/recompute/journal/publication, но внутри HJMT он всё равно изменяет набор отдельных terminal settlement leaves по их `SettlementPath`.**

То есть **не каждая транзакция отдельно проходит полный путь: insert leaf → recompute parents → journal sync → publish root**. Вместо этого batch:

```text
validate batch
-> canonicalize operation order
-> reject duplicate settlement paths
-> group by shard / definition / serial / bucket
-> apply bucket-local deltas
-> recompute touched bucket roots once
-> recompute touched serial roots once
-> recompute touched definition roots once
-> recompute touched shard roots if enabled
-> publish public root after journals are durable
```

Именно это в документе названо **Bucket-Local Batch Commit Engine**: текущий mutation path уже группирует работу, а upgrade делает batch deltas explicit, чтобы каждый touched bucket и parent path пересчитывался **один раз на canonical batch**, а не отдельно на каждую операцию.

Ключевая разница:

```text
Неправильная модель:
  batch -> один новый HJMT leaf

Правильная модель:
  batch -> много ordered terminal operations
        -> bucket deltas
        -> parent deltas
        -> один durable public-root publication boundary
```

Документ прямо говорит, что `BucketDelta` содержит **ordered terminal operations**, pre/post bucket roots, operation digest и proof material for affected leaves. А `ParentDelta` содержит минимальный upward update от bucket root к serial root, definition root, shard root и global root. То есть batch не заменяет terminal leaves одним batch-leaf; он агрегирует изменения и пересчёт.

Поэтому TPS-выигрыш возникает из трёх вещей:

1. **Амортизация пересчёта дерева.**
   Если в batch много операций в одном bucket/serial/definition, система пересчитывает touched bucket/serial/definition roots один раз на batch, а не повторяет parent recomposition после каждой операции.
2. **Амортизация durable journal/publication boundary.**
   Public root становится видимым только после durable child roots и journal records. Документ отдельно требует мерить именно `durable-root-published TPS`, а не worker-local throughput.
3. **Параллелизм по bucket/shard.**
   Batch можно разложить по shard/definition/serial/bucket, применить bucket-local deltas параллельно, а потом детерминированно собрать parent roots. Это не “один leaf”, а “много независимых физических commit surfaces внутри одного batch”.

Важно: **каждая операция всё равно относится к конкретному `SettlementPath`**. Upgrade даже запрещает duplicate `SettlementPath` внутри одного mutation batch, чтобы не было споров типа “две операции на один leaf — какая победила?”. Это подтверждает, что leaf-гранулярность не исчезает. Просто операции собираются в canonical batch и затем применяются как ordered terminal operations.

Ещё отдельно: **BatchProofBlob** — это другая оптимизация. Она даёт выигрыш в proof bytes и verify throughput, потому что много proof paths идут через shared witness context. Но это не главный механизм write TPS. Для write TPS главный механизм — **bucket-local batch commit + delta records + durable-root publication batching**. Документ сам разделяет эти upgrade layers: shared hierarchical multiproof для proof/verifier throughput, bucket-local batch commit для durable mutation throughput, stable shard publication для shardability.

Итог:

```text
Да:
  TPS выигрывается за счёт batch-level commit в HJMT:
  группировка операций, bucket-local deltas,
  один пересчёт touched parent paths,
  один durable publication boundary.

Нет:
  batch не должен становиться одним terminal leaf,
  который “содержит все транзакции”.
```

Самая точная формулировка: **не “встраивание батча целиком как leaf”, а “атомарное применение batch-а как canonical ordered set of terminal mutations с одним upward recomposition/publication cycle”.**



---

---



Моя честная оценка: **для одного сильного разработчика/архитектора с LLM, работающей условно 12 часов в день, полный Upgrade до локального production-mode состояния — примерно 5–7 месяцев реалистично.**

Не “деплой в продакшн”, а именно:

```text
полная локальная реализация
+ все фичи Upgrade
+ deterministic bytes
+ golden vectors
+ negative tests
+ crash/recovery simulation
+ shard/failover simulation
+ benchmarks
+ evidence packets
+ локальный multi-aggregator прогон
```

Оптимистично можно уложиться в **3.5–4.5 месяца**, но только если базовый HJMT код очень чистый, тестовая инфраструктура уже хорошая, и не всплывут архитектурные расхождения. Консервативно — **8–10 месяцев**, если будут тяжёлые проблемы в journal/recovery, root migration или sharding.

## Почему не быстрее

Upgrade-документ сам говорит, что это **не completion report**, а specified contract. Готовность требует не только кода, но и source, tests, vectors, benchmarks и recovery evidence для каждого slice. Также документ прямо говорит, что нужно добавить committed `ShardId`, route-table, per-shard journal, shard-root и batch-proof code before enabling or claiming behavior.

То есть задача не такая:

```text
написать BatchProofBlob
добавить пару структур
сделать scheduler
```

А такая:

```text
изменить proof layer
изменить commit engine
изменить journal/recovery model
изменить root generation
добавить shard identity
добавить root-of-shard-roots
добавить fail-closed validator gates
добавить crash matrix
доказать benchmark evidence
```

Readiness в документе жёсткая: upgrade готов только когда `BatchProofBlob` имеет deterministic encoding/verifier/negative tests, batch proofs реально улучшают bytes/verify throughput, bucket-local commits реально улучшают durable TPS, `ShardId` и `BucketId` разделены, per-shard journal recoverable, root-of-shard-roots migration имеет golden vectors, adaptive transitions safe, а все 8/10 или 9/10 claims backed by evidence.

## Моя разбивка по времени

### 1. Подготовка baseline и fixture/evidence harness — 2–3 недели

Это не feature work, но без него дальше всё будет шатко.

Нужно зафиксировать:

```text
current HJMT root fixtures
single ProofBlob fixtures
Vec<ProofBlob> batch fixtures
existing mutation fixtures
historical root/policy fixtures
benchmark harness
crash injection harness
```

Документ говорит, что test/benchmark names сами по себе не evidence; нужны реальные files, commands and reports. Также нужны fixtures for old/new root generations, historical bucket policies, route generations, mixed present/absent paths, tampered witnesses, interrupted journal stages, privacy-reviewed occupancy evidence.

**Оценка:** 2–3 недели.

------

### 2. `BatchProofBlobV1`: codec, parser, verifier, vectors — 4–6 недель

Это один из самых “опасных” кусков, потому что публичный byte contract должен быть canonical. Нужно сделать:

```text
BatchProofBlobV1 binary codec
path_table canonical ordering
witness_dag
opening_table
reference_table
parser limits before allocation-heavy work
atomic verifier
positive vectors
negative vectors
compatibility with existing ProofBlob / Vec<ProofBlob>
```

Документ требует сравнивать `ProofBlob`, `Vec<ProofBlob>` и `BatchProofBlob` для 2, 8, 32, 128 и 1024 paths across clustered, scattered, inclusion, deletion, and non-existence workloads. Нужны serialized bytes, bytes per path, prove time, verify time, peak memory.

**Оценка:** 4–6 недель.
Если делать сначала builder from existing verified `ProofBlob` paths, можно быстрее и безопаснее, но native optimized builder позже всё равно потребует доработки.

------

### 3. Bucket-local batch commit + durable TPS measurement — 4–6 недель

Здесь реализуется реальный TPS-выигрыш:

```text
canonical mutation batch
duplicate SettlementPath rejection
BucketDelta
ParentDelta
one recomposition per touched bucket
journal stages
durable publication boundary
recovery after interrupted commit
```

Важно: документ говорит, что scheduler parallelism is not durable TPS evidence; нужно мерить именно stages including journal sync, cache verification, index update, model history, and root publication.

**Оценка:** 4–6 недель.

------

### 4. `ShardId`, `ShardRouteTableV1`, deterministic planner — 4–5 недель

Это слой, где появляется protocol-level sharding:

```text
ShardId as separate typed identity
RouteTable canonical bytes
route_hash -> ShardId
routing_generation
route_table_digest
one-shard compatibility mode
route migration vectors
planner grouping by ShardId
cross-shard rejection
```

Документ подчёркивает: `BucketId` нельзя превращать в protocol identity; route-table lookup must be deterministic, canonical-encoded and root-bound before multi-shard execution is enabled. Cross-shard operations must be rejected in this upgrade.

**Оценка:** 4–5 недель.

------

### 5. Per-shard journal, shard queues, recovery, failover simulation — 5–8 недель

Это самый сложный runtime/storage кусок. Нужно сделать не просто shards, а recoverable shard domains:

```text
per-shard journals
shard-local queues
BatchPlanned journal records
shard-local checkpoints
same-lineage standby resume
wrong-lineage rejection
shard-unavailable behavior
carry-forward unchanged shard leaf
crash before/after shard-local commit
crash before public publication
route migration recovery
```

Документ требует explicit golden vectors, crash matrix, failover vectors under same `ShardId` and same journal lineage, rejection on wrong lineage, and carry-forward vectors.

**Оценка:** 5–8 недель.

------

### 6. Root-of-shard-roots + `CheckpointPublicationV1` + validator gate — 4–6 недель

Это меняет публичную root semantics:

```text
RootGeneration0:
  current HJMT semantic root

RootGeneration1:
  Merkle commitment over ShardRootLeaf records
```

Нужно добавить `ShardRootLeafV1`, canonical ordering by `shard_id`, route-table digest binding, policy-set digest, journal checkpoint, local sequence, transition flags, and proof/root acceptance rules. Документ прямо говорит, что current code stores no `ShardRootLeaf` list in `HjmtRoots`, so implementation must add explicit versioned representation before root-of-shard-roots can be accepted.

**Оценка:** 4–6 недель.

------

### 7. Adaptive transitions: split/merge/policy transition safety — 4–7 недель

Это можно оставить ближе к концу, но если “со всеми фичами” — оно входит в полный Upgrade.

Нужно доказать:

```text
split: old terminal set == union of new bucket terminal sets
merge: new bucket == union of old sibling buckets
policy transition: bounded terminal set moved without changing proof identity
historical proof safety
occupancy privacy
transition recovery
transition benchmarks
```

Документ говорит, что record shapes уже есть, но upgrade work is to define verifier semantics, recovery behavior, and benchmark evidence.

Также automatic split/merge scheduling должен оставаться disabled, пока нет telemetry showing transition churn does not harm proof size, cache behavior, or publication latency.

**Оценка:** 4–7 недель.

------

### 8. Full-system simulation, fuzzing, crash/recovery matrix, benchmark reports — 4–6 недель

Это финальный этап, который нельзя “сэкономить”, потому что именно он переводит систему из “implemented” в “locally release-ready”.

Нужно прогнать:

```text
1 shard compatibility
2/4/8/16 shard workers
hot serial
hot bucket
hot shard
balanced load
skewed load
aggregator failover
wrong-lineage failover
route migration
root generation migration
historical proofs
tampered batch proofs
crash at bucket/parent/shard/global/aux/final stages
```

Для score claims документ требует evidence packet: baseline commit, candidate commit, workload profile, positive checks, negative checks, measurements, decision. Если одно поле отсутствует, score remains unsupported even if numbers look strong.

**Оценка:** 4–6 недель.

## Итого по календарю

Если считать **один сильный человек + LLM 12 часов в день**, я бы дал такой диапазон:

```text
Оптимистично:
  14–18 недель
  ≈ 3.5–4.5 месяца

Реалистично:
  22–30 недель
  ≈ 5–7 месяцев

Консервативно:
  32–42 недели
  ≈ 8–10 месяцев
```

Мой основной ответ: **5–7 месяцев**.

## Почему LLM не сжимает это до 1–2 месяцев

LLM сильно ускорит:

```text
boilerplate
codec scaffolding
test generation
negative vector generation
benchmark harness
refactoring
documentation
simulation scripts
```

Но LLM плохо “магически сокращает” то, что требует последовательного доказательства:

```text
canonical byte compatibility
crash/recovery correctness
root migration correctness
historical proof compatibility
shard failover without split-brain
durable TPS measurement
state-machine debugging
```

Документ прямо предупреждает, что каждый step должен produce source, tests, vectors and evidence artifacts before the next layer assumes it as baseline; running proof, root, journal and shard work in parallel without shared golden vectors risks incompatible formats that all look plausible in isolation.

То есть LLM можно использовать как 2–3x coding accelerator, но не как 10x correctness accelerator.

## Самая вероятная критическая цепочка

Я бы ожидал, что больше всего времени съедят не `BatchProofBlob`, а вот эта цепочка:

```text
ShardRouteTableV1
  -> per-shard journal
  -> ShardRootLeafV1
  -> CheckpointPublicationV1
  -> recovery matrix
  -> failover vectors
  -> durable TPS benchmark
```

Потому что здесь любая ошибка ломает сразу:

```text
routing
proof context
journal recovery
public root
historical verification
aggregator failover
```

Batch proof сам по себе сложный, но он более изолирован. Shard/root/journal слой сложнее, потому что он cross-cutting.

## Мой практический план реализации

Я бы не пытался сразу делать “всё production-mode”. Я бы делал так:

```text
Milestone A — 4–6 недель:
  BatchProofBlobV1 compatibility slice.
  Existing ProofBlob remains untouched.
  Golden + negative vectors.
  Benchmarks: proof bytes / verify speed.

Milestone B — 4–6 недель:
  Bucket-local batch commit.
  Durable journal stages.
  Crash recovery for batch commit.
  Benchmarks: durable-root-published TPS.

Milestone C — 5–7 недель:
  ShardId + ShardRouteTableV1 + deterministic planner.
  One-shard compatibility.
  Route split vectors.
  Cross-shard rejection.

Milestone D — 6–8 недель:
  Per-shard journals + shard queues + root-of-shard-roots.
  CheckpointPublicationV1.
  Validator acceptance gate.
  Failover simulation.

Milestone E — 4–7 недель:
  Adaptive split/merge/policy transition proofs.
  Historical proof safety.
  Occupancy privacy.
  Transition benchmarks.

Milestone F — 4–6 недель:
  Full local production-mode simulation.
  Fuzzing.
  Crash/recovery matrix.
  Benchmark evidence packets.
  Final readiness report.
```

Это даёт примерно те самые **22–30 недель**.

## Самая короткая разумная версия

Если нужно получить не “все фичи”, а сильный локальный MVP Upgrade, я бы урезал scope:

```text
да:
  BatchProofBlobV1
  bucket-local batch commit
  deterministic planner
  one-shard compatibility
  ShardRouteTableV1
  root generation tags
  limited benchmarks

нет пока:
  full multi-shard failover
  adaptive auto split/merge
  full route migration
  full root-of-shard-roots production gate
```

Такой MVP можно сделать за **8–12 недель**.

Но это будет не “полноценный Upgrade со всеми фичами”. Это будет verified prototype / partial integrated upgrade, а не release-ready local production-mode.

## Финальная оценка

Мой ответ:

```text
Полный Upgrade, если base HJMT уже имплементирован:
  реалистично 5–7 месяцев
  при одном сильном разработчике/архитекторе + LLM 12 часов/день.

Очень хорошо организованный best case:
  3.5–4.5 месяца.

С учётом сложных багов в recovery/sharding:
  8–10 месяцев.
```

Главная причина: документ требует не просто implementation, а **implementation + evidence**. По самому Upgrade readiness, добавление типов, один benchmark или одна доказанная proof family не закрывают upgrade; release-ready состояние требует evidence packets, readiness bullets and compatibility gates.



---

---



## Моя новая оценка

### Агрессивный вариант: 4–6 недель

Это если:

```
- тот же человек, который сделал base HJMT за 2 недели;
- кодовая база чистая;
- LLM реально используется постоянно;
- не пишется собственный distributed consensus;
- цель — локальная полная реализация + tests + basic benchmarks.
```

Разбивка:

```
Неделя 1:
  BatchProofBlobV1 codec/parser/verifier,
  golden/negative tests.

Неделя 2:
  shared witness builder,
  clustered/scattered proof comparison,
  verify benchmarks.

Неделя 3:
  bucket-local batch commit,
  duplicate path rejection,
  BucketDelta / ParentDelta,
  journal stage tests.

Неделя 4:
  ShardId + ShardRouteTableV1,
  one-shard compatibility,
  route digest vectors,
  planner rejection of cross-shard batches.

Неделя 5:
  ShardRootLeafV1,
  root generation tags,
  root-of-shard-roots no-op migration,
  checkpoint continuity tests.

Неделя 6:
  adaptive transition proof tests,
  crash/recovery matrix,
  benchmark reports,
  cleanup.
```

Это tight schedule, но если base действительно был сделан за 2 недели, такой темп уже доказан.

## Новый итоговый график

Я бы теперь поставил такой план:

```
Неделя 1:
  gap map по текущему коду,
  fixture harness,
  BatchProofBlobV1 codec/parser старт.

Неделя 2:
  BatchProofBlobV1 verifier,
  positive/negative vectors,
  comparison vs Vec<ProofBlob>.

Неделя 3:
  bucket-local batch commit,
  duplicate path rejection,
  BucketDelta/ParentDelta,
  durable TPS benchmark skeleton.

Неделя 4:
  ShardId,
  ShardRouteTableV1,
  deterministic planner,
  one-shard compatibility.

Неделя 5:
  per-shard queues/journals,
  local_sequence,
  shard-local recovery.

Неделя 6:
  ShardRootLeafV1,
  root-of-shard-roots,
  generation 0 -> generation 1 migration vector.

Неделя 7:
  multi-shard local simulation,
  carry-forward unchanged shard leaf,
  wrong route/wrong lineage rejection.

Неделя 8:
  adaptive transitions,
  historical proof safety,
  benchmark reports.

Неделя 9–10:
  fuzzing, crash matrix, cleanup,
  readiness report.
```

Это **реалистичный 8–10-недельный план**.



---

---



## Новый итоговый график

Я бы теперь поставил такой план:

```
Неделя 1:
  gap map по текущему коду,
  fixture harness,
  BatchProofBlobV1 codec/parser старт.

Неделя 2:
  BatchProofBlobV1 verifier,
  positive/negative vectors,
  comparison vs Vec<ProofBlob>.

Неделя 3:
  bucket-local batch commit,
  duplicate path rejection,
  BucketDelta/ParentDelta,
  durable TPS benchmark skeleton.

Неделя 4:
  ShardId,
  ShardRouteTableV1,
  deterministic planner,
  one-shard compatibility.

Неделя 5:
  per-shard queues/journals,
  local_sequence,
  shard-local recovery.

Неделя 6:
  ShardRootLeafV1,
  root-of-shard-roots,
  generation 0 -> generation 1 migration vector.

Неделя 7:
  multi-shard local simulation,
  carry-forward unchanged shard leaf,
  wrong route/wrong lineage rejection.

Неделя 8:
  adaptive transitions,
  historical proof safety,
  benchmark reports.

Неделя 9–10:
  fuzzing, crash matrix, cleanup,
  readiness report.
```

Это **реалистичный 8–10-недельный план**.





Мой выбор:

```
best_practical_choice:
  consensus_replicated_journal: "openraft"
  local_journal: "existing RedB custom journal, or orderwal"
  multi_raft_storage_later: "raft-engine"
  membership: "memberlist"
  experimental_replicated_log: "omnipaxos"
```

Самый разумный путь для Z00Z:

```
z00z_path:
  now:
    - "не трогать distributed consensus"
    - "сделать per-shard durable journal локально"
    - "доказать lineage/recovery/root correctness"

  next:
    - "подключить OpenRaft как replicated journal для ShardGroup"

  later:
    - "если shard-ов много и нужен TiKV-style Multi-Raft, смотреть tikv/raft-rs + raft-engine"
```

То есть: **готовые crates есть, но их надо использовать как consensus/log substrate, а не как готовую Z00Z архитектуру.**



## 

Я бы рекомендовал так:

```
recommended_stack:
  local_durable_journal:
    first_choice: "own Z00Z journal on RedB, because HJMT already uses RedB"
    alternative: "raft-engine if you want Multi-Raft-style log storage"

  replicated_journal:
    first_choice: "openraft"
    second_choice: "tikv/raft-rs + raft-engine"
    experimental_candidate: "omnipaxos"

  per_shard_consensus_group:
    first_choice: "openraft per ShardGroup, not necessarily per every tiny ShardId"
    high_performance_heavy_choice: "tikv/raft-rs + raft-engine"

  membership_failure_detection:
    first_choice: "memberlist"
    alternatives:
      - "chitchat"
      - "foca"

  my_recommendation_for_z00z:
    mvp: "custom per-shard RedB journal + simulated failover"
    next_step: "OpenRaft for replicated journal / shard group"
    avoid_initially: "writing your own consensus protocol"
```

## 1. Consensus / replicated log crates

### ✅ `openraft` — мой первый выбор для Z00Z

`openraft` — это advanced Raft implementation in Rust using Tokio. У него есть dynamic membership, batching, pipelined replication, snapshots, storage/network traits, tracing. В документации прямо сказано, что storage and network integration are defined via `RaftStorage` and `RaftNetwork`, то есть ты можешь привязать его к своему RedB/HJMT journal layer. 

Плюсы:

```
openraft:
  good_for:
    - replicated journal
    - per-shard consensus group
    - leader election
    - failover
    - dynamic membership
    - snapshots
    - integration with custom storage/network
  why_good_for_z00z:
    - "можно сделать one Raft group per ShardGroup"
    - "можно хранить committed HJMT journal entries как Raft log commands"
    - "не надо писать consensus с нуля"
```

Минусы:

```
openraft_risks:
  - "API до 1.0 может быть нестабильным"
  - "надо самому написать storage adapter"
  - "надо самому написать network adapter"
  - "Raft не Byzantine-fault-tolerant"
```

Для Z00Z я бы выбрал именно его как **первую практическую интеграцию**.

## 2. Local WAL / journal crates

Если тебе нужен не consensus, а просто durable journal per shard, есть Rust WAL crates.

### ✅ `orderwal`

`orderwal` — ordered Write-Ahead Log implementation for Rust. Документация описывает его как generic-purpose, atomic, ordered, zero-copy read, zero-cost in-place write, concurrent-safe, pre-allocate style mmap WAL for databases. 

Подходит для:

```
orderwal:
  good_for:
    - local durable journal
    - append-only per-shard log
    - fast crash recovery
    - local simulation before replicated journal
```

Но для Z00Z я бы всё равно сначала посмотрел, насколько проще оставить journal на RedB, потому что базовый HJMT уже завязан на RedB durable surface.



Но я бы не тащил их автоматически. Для Z00Z лучше:

```
z00z_local_journal_recommendation:
  first: "use RedB-based own journal if current HJMT already uses RedB"
  second: "orderwal"
  third: "raft-engine if planning Multi-Raft later"
```

## 

### Вариант B — нормальный replicated journal

```
z00z_replicated_journal_stack:
  consensus:
    crate: "openraft"

  storage:
    first: "custom RedB RaftStorage adapter"
    later: "raft-engine if many Raft groups"

  membership:
    crate: "memberlist or chitchat"

  model:
    raft_group: "ShardGroup"
    shard_group_contains:
      - "ShardId A"
      - "ShardId B"
      - "ShardId C"

  not_recommended:
    - "one Raft group per every tiny shard from day one"
```

Ключевой момент: **не обязательно делать one consensus group per every ShardId**. Если shard-ов много, лучше сделать:

```
ShardGroup_01:
  raft_group_id: 1
  shards:
    - ShardId_0001
    - ShardId_0002
    - ShardId_0003

ShardGroup_02:
  raft_group_id: 2
  shards:
    - ShardId_0004
    - ShardId_0005
    - ShardId_0006
```

Иначе ты получишь слишком много Raft groups и overhead. 

## 5. Моя рекомендация номером

Если выбирать из твоих трёх вариантов:

```
variant_1_sharded_durable_journal:
  recommendation: "да, для первого этапа"
  crates:
    - "redb/custom"
    - "orderwal"
    - "raft-engine later"

variant_2_replicated_journal:
  recommendation: "да, как следующий этап"
  crates:
    - "openraft"
    - "omnipaxos experimental"

variant_3_per_shard_consensus_group:
  recommendation: "production target, но не с первого дня"
  crates:
    first_choice: "openraft"
    heavy_choice: "tikv/raft-rs + raft-engine"
```

Мой финальный совет:

```
final_recommendation:
  phase_1:
    choose: "Variant 1"
    implementation: "custom RedB per-shard durable journal"
    goal: "prove HJMT Upgrade, crash recovery, shard lineage"

  phase_2:
    choose: "Variant 2"
    implementation: "OpenRaft replicated journal for ShardGroup"
    goal: "real hot standby / failover"

  phase_3:
    choose: "Variant 3"
    implementation: "OpenRaft or tikv/raft-rs + raft-engine"
    goal: "production-grade per-shard or per-shard-group consensus"
```

## 6. Что не писать самому

Я бы **не писал с нуля**:

```
do_not_write_from_scratch:
  - "Raft"
  - "Paxos"
  - "membership gossip"
  - "network partition handling"
  - "leader election protocol"
  - "joint membership change protocol"
```

Писать самому надо только Z00Z-specific слой:

```
write_custom:
  - "ShardId"
  - "ShardRouteTableV1"
  - "ShardJournalEntry"
  - "JournalLineageId"
  - "BatchPlanned"
  - "BucketDelta"
  - "ParentDelta"
  - "ShardRootLeafV1"
  - "CheckpointPublicationV1"
  - "fail-closed verifier/recovery rules"
```



## 10. Финальный вывод

Моя оценка:

```
final_assessment:
  was_redb_good_choice:
    answer: "да"
    reason:
      - "pure Rust"
      - "ACID"
      - "MVCC"
      - "crash-safe"
      - "simple embedded deployment"
      - "хорошо подходит для HJMT local deterministic storage"

  should_migrate_now:
    answer: "нет"

  production_risk:
    answer: "write-heavy / very-large-data workloads may expose limits"

  best_next_move:
    answer:
      - "оставить redb"
      - "закрыть redb trait abstraction"
      - "сделать Upgrade benchmark matrix"
      - "мигрировать только если benchmark покажет storage bottleneck"

  most_likely_future_alternative:
    answer: "Fjall first, RocksDB only for heavier production scale"
```

Если совсем коротко: **для текущего Z00Z/HJMT `redb` — правильный выбор. Для будущего production я бы не мигрировал заранее, но обязательно сделал бы backend abstraction и benchmark gate.**



---

---



Да, ты понял **правильно**. Более того, это именно тот уровень разбиения, который тебе нужен перед Upgrade.

Но я бы сформулировал точнее:

```yaml
main_rule:
  Wallet: "не обращается напрямую к Storage"
  Aggregator: "принимает операции, планирует batch, направляет в shard executor"
  Settlement/HJMT: "выполняет commit/proof/recovery semantics"
  StorageBackend: "только durable KV / transactions / journal primitives"
```

То есть `StorageBackend` не должен знать бизнес-логику Z00Z. Он не должен “понимать” `AssetLeaf`, `RightLeaf`, `ShardRouteTable`, `BatchProofBlob`. Он должен давать низкоуровневый durable interface. А Z00Z-семантика должна жить выше.

## Рекомендуемая иерархия crates

Я бы сделал так:

```yaml
modules:  in z00z_storage
  types:
    purpose: "общие типы протокола"
    contains:
      - SettlementPath
      - SettlementStateRoot
      - AssetLeaf
      - RightLeaf
      - ShardId
      - ShardRouteTableV1
      - ShardRootLeafV1
      - CheckpointPublicationV1

  api:
    purpose: "абстрактные traits для durable storage"
    contains:
      - StorageBackend
      - ReadTxn
      - WriteTxn
      - JournalBackend
      - Table abstractions
      - scan/get/put/delete/commit_durable

  redb:
    purpose: "конкретная реализация RedB"
    depends_on:
      - z00z-storage-api

  hjmt:
    purpose: "HJMT / proof / tree logic"
    depends_on:
      - z00z-types
      - z00z-storage-api
    contains:
      - HJMT forest logic
      - ProofBlob
      - BatchProofBlob
      - inclusion/deletion/non-existence proofs
      - root recomposition
      - bucket policy

  settlement:
    purpose: "commit engine поверх HJMT"
    depends_on:
      - z00z-types
      - z00z-hjmt
      - z00z-storage-api
    contains:
      - BucketDelta
      - ParentDelta
      - batch commit
      - journal replay
      - root publication
      - historical proof compatibility

  z00z-aggregator:
    purpose: "runtime node / batch planner / shard executor"
    depends_on:
      - z00z-types
      - z00z-settlement
      - z00z-storage-api
    contains:
      - AggregatorNode
      - BatchPlanner
      - ShardExecutor
      - ShardQueue
      - PlacementTable
      - failover simulation
      - shard-unavailable logic

  z00z-wallet:
    purpose: "wallet/client logic"
    depends_on:
      - z00z-types
      - z00z-verifier
      - z00z-aggregator-api
    must_not_depend_on:
      - z00z-storage-redb
      - z00z-storage-api directly

  z00z-verifier:
    purpose: "локальная проверка proof-ов"
    depends_on:
      - z00z-types
      - z00z-hjmt proof verifier only

  z00z-sim:
    purpose: "локальная симуляция multi-aggregator / multi-shard"
    depends_on:
      - z00z-aggregator
      - z00z-storage-redb
```

## Главное: Wallet не должен видеть Storage

Да. Даже в симуляции wallet не должен делать:

```rust
wallet -> redb
wallet -> storage_backend
wallet -> hjmt_commit
```

Правильно:

```text
Wallet
  -> AggregatorClient / AggregatorApi
    -> AggregatorNode
      -> BatchPlanner
        -> ShardExecutor
          -> SettlementEngine
            -> HJMT
              -> StorageBackend
                -> RedB
```

Wallet может делать только две вещи:

```yaml
wallet_allowed:
  submit_operation:
    to: "Aggregator API"

  verify_result:
    using:
      - ProofBlob
      - BatchProofBlob
      - SettlementStateRoot
      - CheckpointPublication
```

То есть wallet может **верифицировать proof**, но не должен сам писать в storage.

Это важно архитектурно: Upgrade-документ требует сохранить `SettlementStateRoot` как semantic authority, а path indexes/caches/metrics/model-history не должны становиться consensus truth. Поэтому wallet должен общаться с public/proof API, а не с внутренними таблицами storage.

## Где должен сидеть batch planner

Я бы разместил `BatchPlanner` внутри `z00z-aggregator`, но сделал бы его отдельным модулем:

```yaml
z00z-aggregator:
  modules:
    batch_planner:
      input:
        - wallet operations
        - current ShardRouteTableV1
      output:
        - BatchPlanned
        - target ShardId
        - canonical ordered operations

    shard_executor:
      input:
        - BatchPlanned
      output:
        - BucketDelta
        - ParentDelta
        - shard-local root update

    publication:
      input:
        - shard-local roots
      output:
        - ShardRootLeafV1
        - CheckpointPublicationV1
```

То есть planner — это не storage. Planner — это часть aggregator/runtime.

Документ как раз требует, чтобы Upgrade добавил stable `ShardId`, per-shard journals, shard queues and root-of-shard-roots. Это означает, что aggregator должен быть местом, где операции превращаются в shard-local batches.

## StorageBackend должен быть тупым

`z00z-storage-api` должен быть максимально скучным:

```rust
pub trait StorageBackend {
    type ReadTxn<'a>: ReadTxn
    where
        Self: 'a;

    type WriteTxn<'a>: WriteTxn
    where
        Self: 'a;

    fn begin_read(&self) -> Result<Self::ReadTxn<'_>>;
    fn begin_write(&self) -> Result<Self::WriteTxn<'_>>;
}

pub trait ReadTxn {
    fn get(&self, table: TableId, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn scan_prefix(&self, table: TableId, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}

pub trait WriteTxn: ReadTxn {
    fn put(&mut self, table: TableId, key: &[u8], value: &[u8]) -> Result<()>;
    fn delete(&mut self, table: TableId, key: &[u8]) -> Result<()>;
    fn commit_durable(self) -> Result<()>;
}
```

А вот это **не должно быть** в storage backend:

```yaml
not_in_storage_backend:
  - validate AssetLeaf
  - validate RightLeaf
  - choose ShardId
  - plan batch
  - verify BatchProofBlob
  - decide checkpoint publication
  - know wallet semantics
  - know aggregator failover rules
```

Storage backend — это просто durable substrate.

## Для симуляции sharding на одном компьютере

Да, именно такая иерархия позволит симулировать весь процесс на одной машине.

Например:

```yaml
simulation:
  machine: "one laptop"
  process: "one binary"
  runtime: "tokio tasks"

  aggregators:
    Agg-1:
      db_path: "./sim/agg1.redb"
      primary_shards: [0, 1]
      standby_shards: [2]

    Agg-2:
      db_path: "./sim/agg2.redb"
      primary_shards: [2, 3]
      standby_shards: [0]

    Agg-3:
      db_path: "./sim/agg3.redb"
      primary_shards: [4, 5]
      standby_shards: [1, 3]

  wallet:
    talks_to:
      - AggregatorApi
    does_not_talk_to:
      - RedB
      - StorageBackend
```

Можно запускать всё в одном процессе:

```text
z00z-sim
  -> AggregatorNode(Agg-1)
  -> AggregatorNode(Agg-2)
  -> AggregatorNode(Agg-3)
  -> WalletSimulator
  -> NetworkSimulator
```

Или в разных процессах:

```text
terminal 1: z00z-aggregator --node agg1 --db ./agg1.redb
terminal 2: z00z-aggregator --node agg2 --db ./agg2.redb
terminal 3: z00z-aggregator --node agg3 --db ./agg3.redb
terminal 4: z00z-wallet-sim --submit 100000 ops
```

## Очень важный момент: Aggregator тоже не должен напрямую знать RedB

Aggregator должен зависеть от trait:

```rust
pub struct AggregatorNode<S>
where
    S: StorageBackend + Send + Sync + 'static,
{
    storage: Arc<S>,
    planner: BatchPlanner,
    settlement: SettlementEngine<S>,
}
```

А не так:

```rust
pub struct AggregatorNode {
    db: redb::Database,
}
```

Почему? Потому что завтра ты сможешь сделать:

```yaml
backends:
  RedbStorageBackend:
    use_now: true

  InMemoryStorageBackend:
    use_for:
      - tests
      - fuzzing
      - fast simulation

  FjallStorageBackend:
    use_later_if:
      - redb bottleneck

  RocksDbStorageBackend:
    use_later_if:
      - heavy production scale

  RaftJournalBackend:
    use_later_if:
      - replicated journal
```

## Где хранить replicated journal later

Сейчас:

```text
Aggregator -> SettlementEngine -> RedB Journal
```

Потом:

```text
Aggregator -> SettlementEngine -> JournalBackend
                               -> RedBJournalBackend
                               -> OpenRaftJournalBackend
                               -> RaftEngineJournalBackend
```

То есть ты заранее не делаешь migration, но оставляешь правильную дырку в архитектуре.

Для replicated journal можно будет сделать:

```yaml
journal_backends:
  local:
    type: "RedBJournalBackend"

  replicated:
    type: "OpenRaftJournalBackend"
    raft_group: "ShardGroupId"
    entries:
      - BatchPlanned
      - BucketDelta
      - ParentDelta
      - ShardCommit
      - ShardCheckpoint
```

## Правильное разделение “protocol truth” и “runtime placement”

Это критично для твоего RAID10/failover.

```yaml
protocol_truth:
  - ShardId
  - ShardRouteTableV1
  - routing_generation
  - ShardRootLeafV1
  - CheckpointPublicationV1
  - SettlementStateRoot

runtime_placement:
  - AggregatorId
  - primary_shards
  - standby_shards
  - local queues
  - liveness
  - failover controller
```

`AggregatorId` не должен быть public settlement truth.
`ShardId` — да.

Так ты сможешь менять физические агрегаторы без поломки proof semantics.

## Минимальная схема зависимостей

Я бы держал зависимости так:

```text
z00z-types
   ↑
z00z-storage-api
   ↑
z00z-storage-redb

z00z-types
   ↑
z00z-hjmt
   ↑
z00z-settlement
   ↑
z00z-aggregator
   ↑
z00z-sim

z00z-wallet -> z00z-aggregator-api
z00z-wallet -> z00z-verifier
z00z-wallet -X-> z00z-storage-redb
```

`-X->` значит запрещено.

## Мой ответ

Да, я бы именно так и делал:

```yaml
answer:
  separate_aggregator_crate:
    decision: "да"
    name: "z00z-aggregator"
    responsibility:
      - batch planning
      - shard routing
      - shard executors
      - queues
      - publication coordination
      - failover simulation

  separate_storage_backend_crate:
    decision: "да"
    names:
      api: "z00z-storage-api"
      redb_impl: "z00z-storage-redb"
    responsibility:
      - RedB integration
      - transactions
      - tables
      - durable commits
      - scans
      - local journal primitives

  wallet_direct_storage_access:
    decision: "нет"
    reason: "wallet должен видеть только protocol API и proofs"

  one_computer_sharding_simulation:
    decision: "да"
    method:
      - multiple AggregatorNode instances
      - each with its own db path or namespace
      - placement table maps ShardId to primary/standby aggregator
      - wallet submits only through Aggregator API
```

Самая короткая правильная формула:

```text
Wallet talks to Aggregator.
Aggregator talks to Settlement Engine.
Settlement Engine talks to HJMT.
HJMT talks to StorageBackend.
StorageBackend talks to RedB.
```

Вот это разделение я бы зафиксировал перед Upgrade.



---

---



Да, это **нормальное и даже более практичное решение на текущем этапе**.

Я бы сделал так:

```yaml
decision:
  separate_crate_for_storage: true
  separate_crates_inside_storage_for_api_redb_types: false
  use_internal_modules: true
```

То есть:

```text
crates/
  z00z-storage/
    src/
      lib.rs
      api.rs
      types.rs
      error.rs
      tables.rs
      journal.rs
      codec.rs
      migration.rs
      backends/
        mod.rs
        redb.rs
        memory.rs
      testkit.rs
```

А не так:

```text
crates/
  z00z-storage-api/
  z00z-storage-types/
  z00z-storage-redb/
  z00z-storage-memory/
```

## Моя оценка

Я согласен с твоим предложением.

Для текущей стадии Z00Z лучше иметь **один crate `z00z-storage`**, внутри которого есть модули:

```yaml
z00z-storage:
  modules:
    api:
      purpose: "traits: StorageBackend, ReadTxn, WriteTxn, JournalBackend"

    types:
      purpose: "storage-level types only"

    backends/redb:
      purpose: "конкретная реализация RedB"

    backends/memory:
      purpose: "in-memory backend for tests/simulation"

    journal:
      purpose: "forest/shard journal records and replay"

    tables:
      purpose: "table IDs, table schema, key namespaces"

    migration:
      purpose: "schema versioning and migration"

    testkit:
      purpose: "common storage tests for all backends"
```

Это проще, быстрее и меньше бюрократии.

## Главное условие

Не количество crates важно, а **границы доступа**.

Даже если всё внутри одного `z00z-storage`, нужно сделать так, чтобы внешний код не работал напрямую с `redb`.

Плохо:

```rust
use z00z_storage::backends::redb::RedbTable;
use redb::Database;
```

Хорошо:

```rust
use z00z_storage::api::StorageBackend;
use z00z_storage::backends::RedbStorage;
```

То есть `redb` должен быть **implementation detail**, а не часть архитектуры aggregator/wallet/HJMT.

## Как должен выглядеть `lib.rs`

Примерно так:

```rust
pub mod api;
pub mod error;
pub mod types;
pub mod journal;
pub mod tables;
pub mod migration;

pub mod backends {
    #[cfg(feature = "redb")]
    mod redb_backend;

    #[cfg(feature = "memory")]
    mod memory_backend;

    #[cfg(feature = "redb")]
    pub use redb_backend::RedbStorage;

    #[cfg(feature = "memory")]
    pub use memory_backend::MemoryStorage;
}
```

Но внутренние детали RedB лучше не экспортировать:

```rust
// не экспортировать наружу:
redb::Database
redb::TableDefinition
redb::ReadableTable
redb::WriteTransaction
```

Экспортировать только свой wrapper:

```rust
pub struct RedbStorage { ... }
```

## Почему один crate лучше сейчас

Потому что у тебя быстрый архитектурный цикл. Разбивка на 4–5 storage crates сейчас может создать лишнюю friction:

```yaml
benefits_of_single_storage_crate:
  - проще refactor
  - быстрее менять API
  - меньше Cargo.toml зависимости
  - меньше version coordination
  - легче гонять тесты
  - проще делать feature flags
  - меньше boilerplate между crates
```

Сейчас важно не “идеально распилить”, а быстро удержать правильные слои:

```text
Aggregator -> Settlement/HJMT -> Storage API -> RedB
```

А не:

```text
Wallet -> RedB
Aggregator -> redb::Database
HJMT -> hardcoded redb tables everywhere
```

## Где нужна осторожность

Есть один риск: когда API, RedB и journal лежат в одном crate, легко начать случайно использовать internal-модули напрямую.

Чтобы этого не произошло, я бы сделал:

```yaml
rules:
  redb_module:
    visibility: "private or crate-private as much as possible"

  public_exports:
    allow:
      - StorageBackend
      - ReadTxn
      - WriteTxn
      - JournalBackend
      - RedbStorage
      - MemoryStorage
      - StorageError

  public_exports:
    deny:
      - redb::Database
      - redb::Transaction
      - redb::TableDefinition
      - internal table names
      - raw schema internals
```

То есть модуль есть, но он не протекает наружу.

## Где держать protocol types

Я бы не складывал в `z00z-storage::types` такие вещи:

```yaml
do_not_put_in_storage_types:
  - SettlementPath
  - SettlementStateRoot
  - AssetLeaf
  - RightLeaf
  - ShardId
  - ShardRouteTableV1
  - BatchProofBlob
```

Это protocol/domain types. Их лучше держать отдельно:

```text
z00z-types
```

А в `z00z-storage::types` держать только storage-level вещи:

```yaml
storage_types:
  - TableId
  - StorageKey
  - StorageValue
  - StorageNamespace
  - StorageVersion
  - TxnId
  - JournalOffset
  - DurableCheckpoint
```

Иначе storage crate начнёт знать слишком много о Z00Z-протоколе.

## Рекомендуемая итоговая структура

```text
crates/
  z00z-types/
    src/
      settlement.rs
      shard.rs
      proof.rs
      leaf.rs

  z00z-storage/
    src/
      lib.rs
      api.rs
      error.rs
      types.rs
      tables.rs
      journal.rs
      migration.rs
      codec.rs
      backends/
        mod.rs
        redb.rs
        memory.rs
      testkit.rs

  z00z-hjmt/
    src/
      forest.rs
      proof.rs
      batch_proof.rs
      bucket.rs
      commit.rs

  z00z-settlement/
    src/
      batch_commit.rs
      publication.rs
      recovery.rs

  z00z-aggregator/
    src/
      node.rs
      batch_planner.rs
      shard_executor.rs
      placement.rs
      failover.rs

  z00z-wallet/
    src/
      client.rs
      verifier.rs

  z00z-sim/
    src/
      main.rs
      multi_aggregator.rs
```

## Финальный ответ

Да, я бы сделал именно так:

```yaml
final_recommendation:
  storage_crate:
    name: "z00z-storage"
    structure: "internal modules, not many small crates"

  aggregator_crate:
    name: "z00z-aggregator"
    separate: true

  wallet:
    direct_storage_access: false

  redb:
    status: "implementation detail inside z00z-storage"

  future_migration:
    possible: true
    how: "add another backend module inside z00z-storage first"
```

Самая правильная формула:

```text
Не дробить Storage на много crates сейчас.
Но жёстко спрятать RedB за Storage API.
```

Это даст тебе и скорость разработки, и возможность потом заменить RedB/Fjall/RocksDB без переписывания aggregator/wallet/HJMT.