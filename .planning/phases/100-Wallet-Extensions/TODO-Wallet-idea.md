# Wallet Backlog Ideas

Date: 2026-05-31
Scope: `crates/z00z_wallets`
Status: backlog

This note keeps only the still-live wallet ideas extracted from
`docs/archive/10_Z00Z_Wallet.md` and rewrites them in the current wallet
model. The canonical live wallet authority remains the encrypted `.wlt`
database plus the explicit `wallet_<stem>_tx_history.jsonl` sidecar, and the
canonical export surface remains `WalletExportPack`.

## 🎯 Scope

Keep only these two backlog items:

- canonical merge and consolidation flow
- wallet-local compaction policy

Do not carry forward these archive terms as normative design:

- `ACK`, `pending_ack`, or archive-only dispute state as wallet authority
- `context_id` as a required wallet persistence field
- YAML wallet snapshots or YAML storage-policy files
- local Merkle wallet-db, Bloom filters, or bitmaps as authoritative spent
  state
- any second live wallet authority beyond `.wlt`, `WalletExportPack`, and the
  explicit tx-history plane

## 🧾 Phase 060 Catalog Status

This note remains a source-intent document, not the repository-owned wallet
profile catalog. The authoritative Phase 060 catalog lives in
`crates/z00z_wallets/docs/WALLET-GUIDE.md`.

The following profile ids are proposed Phase 060 catalog identifiers and must
not be presented as already-live code identifiers unless a referenced live
anchor says so explicitly:

- `fee_credit_v1`
- `service_entitlement_v1`
- `data_access_v1`
- `agent_budget_v1`
- `validator_mandate_lock_v1`
- `transferable_claim_v1`

The current live repository anchors remain:

- `service_entitlement_v1` -> live `service_entitlement`
- `data_access_v1` -> live `data_access`
- `validator_mandate_lock_v1` -> live `validator_mandate`
- `agent_budget_v1` -> live `machine_capability` plus `one_time_use`
- `fee_credit_v1` -> existing voucher object model plus the `FeeCredit`
  product term
- `transferable_claim_v1` -> existing Phase 059 voucher object model

## 🔀 Idea 1: Canonical Merge And Consolidation Flow

### ✅ Merge Problem

The wallet currently exposes a merge reachability surface and selection logic,
but the live path still returns guard data and `stub_tx_merge` instead of a
canonical transaction plus checkpoint-backed settlement.

### ✅ Merge Goal

Allow one wallet to consolidate many small live assets into fewer live assets
without bypassing the normal wallet transaction, receive, and reconcile
boundaries.

### ✅ Merge Requirements

- Inputs must be selected from live spendable wallet-owned assets in one
  unlocked wallet session.
- The flow must return a real transaction identifier bound to the canonical
  wallet transaction lane, not a merge-only placeholder.
- The resulting output must become authoritative only after publication and
  checkpoint-backed reconcile, not by immediate in-place replacement of owned
  assets.
- Wallet-local ownership for the resulting output must still be recognized by
  wallet-owned scanning and `recv_range(...)`.
- `.wlt`, `WalletProfilePayload`, `OwnedAssetPayload`, `ScanStatePayload`, and
  `wallet_<stem>_tx_history.jsonl` remain the live persistence surfaces.
- Export and restore after a merge must still round-trip through
  `WalletExportPack` plus the existing tx-history plane.
- The convenience RPC surface may stay, but it must resolve into the canonical
  settlement path and must not create a second merge-only database or snapshot.

### 🚫 Merge Non-Goals

- Reintroducing the archive `MergeTx` YAML as a normative wire contract.
- Reintroducing `context_id`, `ACK`, or archive-only challenge material as
  normal wallet state.
- Treating merge as final before checkpoint-backed settlement.

### ✅ Merge Acceptance Criteria

- A wallet with many small spendable assets can request consolidation and
  receive real transaction metadata instead of `stub_tx_merge`.
- The merge lifecycle is visible on the explicit tx-history plane and survives
  reopen, export, and restore without any merge-specific repair path.
- After checkpoint-backed reconcile, the input assets are no longer presented as
  spendable and the consolidated output is present through the normal wallet
  ownership path.

## 🧹 Idea 2: Wallet-Local Compaction Policy

### ✅ Compaction Problem

Wallet-local state can grow across owned-asset storage, scan state, and the
explicit tx-history sidecar. The archive proposed purge, archive pools, and
compact snapshots, but the live equivalent must stay strictly local and must
not redefine protocol authority.

### ✅ Compaction Goal

Reduce local storage growth without changing checkpoint authority, receive
authority, or the canonical export and restore story.

### ✅ Compaction Policy

- Compaction is local housekeeping over `.wlt` payloads and explicit wallet
  sidecar files only.
- Protocol consumed state remains defined by checkpointed storage roots,
  asset-leaf membership, and spent deltas. Wallet-local `is_spent` style flags
  remain cache metadata only.
- Compaction may remove or rewrite only data that is redundant, derivable, or
  explicitly non-authoritative after a retention policy.
- Any archival or offloaded material must stay outside the normal live reopen
  path and must not silently become a second authority source.
- `WalletExportPack` remains the only canonical wallet-state export contract.
  Compaction must not introduce a compact-only export bundle.
- `wallet_<stem>_tx_history.jsonl` remains the explicit live history plane until
  a separate migration lands.
- `recv_range(...)` remains the receive and ownership authority even when
  remote scan helpers or local compaction helpers are added.

### 🚫 Compaction Non-Goals

- Using Bloom filters, bitmaps, or a local Merkle index as authoritative spent
  proof.
- Replacing `.wlt` with a compact snapshot format.
- Hiding required restore state outside `WalletExportPack`.
- Turning wallet-local compaction into protocol settlement logic.

### ✅ Compaction Acceptance Criteria

- Running compaction does not change the wallet's spendable set or ownership
  view after reopen.
- A compacted wallet still exports and restores through the same
  `WalletExportPack` shape and explicit tx-history plane.
- Dropping local spent metadata never becomes proof of protocol spent status and
  never replaces checkpoint evidence.

## 🔒 Invariants

- `.wlt` remains the canonical encrypted wallet database.
- `WalletExportPack` remains the canonical wallet-state transfer bundle.
- `wallet_<stem>_tx_history.jsonl` remains explicit while the current history
  plane is live.
- `recv_range(...)` remains the ownership authority.

## 🔒 ZINV Anchors

### ZINV-ASSET-001

Invariant reference: `ZINV: ASSET-001`

Wallet-local object families must stay attached to one concrete asset lineage.
Any lock, mandate, merge, or compaction idea in this note has to preserve the
fact that rights and policy references stay bound to the same asset family
instead of creating a second ownership plane.

### ZINV-ASSET-002

Invariant reference: `ZINV: ASSET-002`

Any `RightLeaf`, voucher-style wrapper, lock profile, or staking mandate
described here must stay fail-closed and non-transferable unless an explicit
policy-approved transition authorizes movement.

### ZINV-CRYPTO-002

Invariant reference: `ZINV: CRYPTO-002`

Replay-sensitive wallet surfaces such as payment requests, receiver cards,
unlock paths, and reward claims must reject malformed or replayed compact
payloads without panicking or silently reusing prior authority.

---

Разверну идею как Z00Z-native “rights-based staking/lock” и быстро сверю её с текущими `RightLeaf`/settlement anchors, чтобы предложение не уехало в абстрактный staking из другого дизайна.

Да, это сильная идея для Z00Z. Я бы назвал её **Rights-Based Self-Custody Locking** или **Private Stake Rights**.

Ключевая формула:

```text
Coins stay in the wallet.
A RightLeaf encumbers them.
Normal spend is blocked.
Only policy-approved transitions are allowed: unlock, redelegate, challenge, slash, reward claim.
```

Важно: это не должно быть просто “wallet UI не даёт потратить”. Такой soft-lock слабый. Чтобы это было настоящим staking/lock primitive, settlement должен видеть доказательство, что asset либо не encumbered, либо spending transition разрешён lock-policy. Иначе пользователь сможет обойти wallet и собрать tx другим клиентом.

## Как Это Должно Работать

Лучший дизайн — не отправлять Z00Z в staking pool и не делать публичный account lock. Вместо этого wallet делает private rewrap/encumbrance:

```text
AssetLeaf / owned note
  -> locked asset state remains under holder control
  -> RightLeaf::ValidatorMandate or lock profile is created
  -> payload_commitment binds locked asset commitments + amount commitment + terms
  -> transition_policy_id defines unlock/redelegate/slash/reward rules
```

Снаружи это выглядит не как “депозит в контракт”, а как **публично проверяемое право/обременение над приватным владением**. Пользователь всё ещё владеет ключами. Но обычный spend builder и settlement verifier обязаны отклонить попытку потратить locked asset как обычную монету.

`RightLeaf` уже хорошо подходит:

- `right_class = ValidatorMandate` для staking/validator/security bond.
- `holder_commitment` = кто владеет lock.
- `control_commitment` = кто может делать разрешённые transitions.
- `beneficiary_commitment` = куда идут rewards.
- `payload_commitment` = commitment к locked assets, amount, lock terms.
- `valid_from / valid_until` = lock period.
- `challenge_from / challenge_until` = slashing/dispute window.
- `use_nonce` = anti-replay lock id.
- `transition_policy_id` = unlock/redelegate/reward/slash grammar.
- `revocation_policy_id` = emergency revoke or governance-controlled termination.
- `disclosure_policy_id` = optional proof to corporate/auditor that stake exists without showing all wallet assets.
- `retention_policy_id` = how long evidence is retained.

## Самая Чистая Модель Для Staking

Я бы сделал v1 без агрессивного slashing:

```text
StakeLockRight v1:
  type: ValidatorMandate
  locked_asset_commitment
  locked_amount_commitment
  validator_or_pool_scope
  holder_commitment
  reward_beneficiary_commitment
  lock_start
  lock_until
  redelegation_policy_id
  unlock_policy_id
  reward_policy_id
  optional_disclosure_policy_id
```

Что можно доказать:

- “у меня есть stake weight”, без раскрытия всего баланса;
- “этот stake locked до такого checkpoint/window”;
- “эти монеты не участвуют в ordinary spend”;
- “этот validator/operator имеет private delegated mandate”;
- “reward claim связан с active stake right”;
- “corporate treasury держала reserve/stake под policy, но без публичного treasury graph”.

## Как Блокировать Трату

Есть два варианта.

1. **Rewrapped Locked Asset**  
   Самый практичный. При lock обычный asset превращается в locked asset state с тем же holder-ownership смыслом, но другим spend policy. Ordinary spend не проходит, пока не выполнен unlock transition.

2. **Encumbrance Right Over Existing Asset**  
   Более элегантно, но сложнее. Asset остаётся “тем же”, а рядом появляется active `RightLeaf`, который запрещает spend. Тогда каждый spend должен доказывать отсутствие active encumbrance или предъявлять разрешённый transition. Это требует аккуратных non-membership / active-lock proofs и может быть дороже.

Для v1 я бы выбрал **rewrapped locked asset + RightLeaf mandate**. Для пользователя это всё равно “монеты не покинули кошелёк”, но protocol state становится проще и безопаснее.

## Slashing: Осторожно

Если stake должен быть slashable, есть развилка:

- **Non-slashable private stake**: даёт voting/validator weight/reward eligibility, но не является сильным economic security bond.
- **Slashable self-custody bond**: wallet заранее создаёт policy, где доказанное нарушение позволяет выполнить penalty transition. Это мощно, но требует challenge window, fraud proof, appeal policy и очень строгих правил.

Для Z00Z я бы начинал с non-slashable или challenge-bounded stake. Полный slashing только после формального proof model.

## Другие Полезные Features Через Rights

Самые интересные:

| Feature | Как работает через Rights | Почему полезно |
| --- | --- | --- |
| Private governance lock | Монеты locked, голос/weight доказывается через `RightLeaf`, balance не раскрывается | Приватное голосование без публичного whale graph |
| Corporate treasury reserve | Funds stay in corporate wallet, но locked под board/compliance policy | Proof of reserve/segregated funds без раскрытия всей treasury |
| Escrow without custody | Buyer locks funds locally; seller получает proof of lock; release/refund через policy | Private commerce без escrow contract custody |
| Subscription / service entitlement | `ServiceEntitlement` right даёт доступ к API/service до expiry | Z00Z как private access-rights layer |
| Agent budget right | AI agent получает bounded spending/service capability, не весь wallet | Безопасные agent wallets |
| Credit collateral | Collateral stays in wallet but cannot be spent while loan right active | Private lending без публичного collateral graph |
| Insurance/security bond | Locked right доказывает bond; claim/challenge может активировать payout | Для validators, services, bridges |
| Data-room access | `DataAccess` right открывает corporate docs only under policy | Enterprise privacy + auditable access |
| Vesting / payroll lock | Employee/investor allocation locked by time or milestones | Private vesting без публичной cap table |
| Merchant hold / refund window | Payment locked until delivery/challenge window closes | Private commerce UX |
| Bridge reserve right | External custody/bridge reserve represented as bounded right with attestations | Cleaner than pretending bridge custody is native settlement |
| Recovery cooldown | Stolen-key risk: large spends require delay/right transition | Wallet security feature |
| Compliance proof lock | Corporate wallet proves “these funds were not spent during period X” | Useful for audits and reserves |

## Лучший Architecture Takeaway

Z00Z should not add many new primitives like `StakeLeaf`, `EscrowLeaf`, `ReserveLeaf`, `SubscriptionLeaf`.

Лучше:

```text
AssetLeaf = private value object
RightLeaf = private bounded control/permission/encumbrance object
Profiles = staking, escrow, reserve, access, agent budget, governance, compliance
```

То есть staking — это не отдельная вселенная. Это первый killer use-case для **RightLeaf as private encumbrance layer**.

## Что Я Бы Добавил В Roadmap

1. `StakeLockRight` profile over `RightLeaf::ValidatorMandate`.
2. Wallet-local locked asset registry.
3. Spend builder rule: locked assets cannot be selected for ordinary tx.
4. Settlement proof rule: locked asset can move only through approved unlock/redelegate/slash transition.
5. `StakeProof`: proves stake amount/range and active lock without wallet-history reveal.
6. `RewardClaimRight`: rewards are claimed through a bounded right, not public account balance.
7. Tests: cannot spend locked asset, can unlock after expiry, cannot replay unlock, can prove active stake, unrelated assets remain hidden.

This fits Z00Z extremely well: it turns Z00Z from “private cash” into **private programmable ownership constraints** without adding a generic VM or public account staking model.

---

---

```text
В качестве примера можно привести ситуацию, когда казначейства или какие-либо анонимные комиссии могли бы оплачивать сборы за стейкинг монет и динамически регулировать их стоимость в зависимости от состояния рынка, то есть стимулировать удержание, когда все остальные продают, и наоборот, когда все анонимно, что вполне естественно.
```

Быстро сверю идею с текущими treasury/right/fee surfaces в репо, чтобы brainstorming опирался на Z00Z boundaries, а не на абстрактный tokenomics fanfic.

## Коротко

Да, это можно сделать, но в Z00Z это лучше оформлять не как “treasury платит за холд”, а как приватный, rule-bound механизм fee sponsorship и fee credits, обеспеченный locked or bonded Z00Z. В коде уже есть реальный surface для sponsor-paid fees через fee_envelope.rs и его валидацию в fee_envelope.rs, а docs уже разрешают sponsored fee credits и fee smoothing в Z00Z-Tokenomics-Incentives-Whitepaper.md и Z00Z-Tokenomics-Incentives-Whitepaper.md. Но тот же corpus прямо запрещает превращать treasury в постоянный price-support engine или платить за пассивный holding и hype в Z00Z-Tokenomics-Incentives-Whitepaper.md и Z00Z-Proof-of-Useful-Work-Whitepaper.md.

Если говорить строго, в текущем Z00Z ближе не классический staking с доходностью, а locked capacity, fee credits, bonds, sponsor budgets и rights-based encumbrance. То есть хорошая формулировка идеи такая: не “платим людям за падение цены”, а “в стрессовых режимах временно повышаем ценность locked Z00Z как источника приватно потребляемой fee capacity и network stability”.

## Варианты

1. Counter-cyclical FeeCredit Vault. Пользователь, merchant, relay или treasury-locker блокирует Z00Z на эпоху или несколько эпох, а система в режиме stress выдает больше fee credits на ту же заблокированную сумму. Это стимулирует не просто держать монеты, а держать именно processing capacity внутри сети.

2. Use-plus-lock subsidy. Субсидия включается только если одновременно есть lock и реальная полезная активность, например settlement traffic, relay service, merchant flow, agent work. Тогда это не пассивный yield farm, а поддержка полезного поведения во время рыночного стресса.

3. Merchant and relay stability shield. Когда рынок сыпется, treasury временно покрывает часть base fees для relays, merchants, wallets и onboarding surfaces. Это снижает шанс, что сеть потеряет полезную активность именно тогда, когда ей нужна устойчивость.

4. Anonymous sponsor committee. Псевдонимный, bonded, challengeable комитет не двигает деньги руками, а только публикует signed regime result, например stress, neutral, overheated. Дальше treasury-модуль исполняет заранее объявленную кривую субсидии. Так комитет не становится скрытым allocator.

5. Rights-based lock mandate. Сам lock-support surface лучше выражать как профиль поверх RightLeaf, а не как новую primitive family, потому что RightLeaf уже несет holder, control, beneficiary commitments, validity windows и policy ids в types_record.rs и types_record.rs. Это удобно для lock, reclaim, challenge, disclosure и retention policy.

6. Ack-style private fee rebate. Вместо того чтобы награждать “веру в цену”, можно делать приватный cashback на реальные транзакции или service interactions в стресс-эпохах. Это ближе к уже существующему мышлению про bounded cashback and rebate lanes в Z00Z-Proof-of-Useful-Work-Whitepaper.md.

7. Treasury-backed sponsor bands. Treasury не обещает “поддержку цены”, а выделяет ограниченный epoch budget на sponsor commitments. В плохом рынке budget band растет, в перегретом рынке сжимается. Публично видны только policy hash, cap и aggregate spend, а не кто именно получил subsidy.

## Как лучше определять состояние рынка

Лучше не завязывать это на голую цену. Цена слишком легко манипулируется и слишком быстро превращает механику в quasi-market operation.

Лучше брать медленные epoch-сигналы:

1. Net lock versus unlock delta по sponsor vaults и bond surfaces.
2. Burn rate sponsored fee budgets versus organic fee demand.
3. Relay churn, operator uptime, challenger participation.
4. Merchant activity, onboarding success, repeated real usage.
5. Exit pressure, bridge concentration, liquidity stress indicators.
6. External price, только как слабый capped input, а не главный триггер.

Практически это лучше выражать не как continuous oracle, а как редкие regime bands. Например stress, normal, hot. Тогда policy меняет только multiplier, reclaim haircut, sponsor ceiling и cooldown, а не “рисует доходность” на каждом тике.

## Что здесь опасно

1. Платить просто за holding. Это уже близко к запрещенной логике “passive holding reward”, а docs от этого сознательно уходят.
2. Делать perpetual subsidy engine. В tokenomics это прямо запрещено, bootstrap reserve и liquidity reserve не должны стать вечным насосом.
3. Давать anonymous committee discretionary treasury control. Комитет может классифицировать режим, но не должен сам выбирать получателей и суммы.
4. Делать transferable gas vouchers. Docs уже толкают в сторону non-transferable fee credits, и это правильно.
5. Реагировать на цену слишком быстро. Иначе получится oracle-gaming и narrative “protocol manages token price”.
6. Смешивать evaluation rights и transfer rights. Это против всей линии про rule-bound treasury и bounded execution.

## Мой сильный вариант

Если собирать это по-з00zовски, я бы делал так:

1. Liquidity and Fee-Credit Reserve дает только bounded sponsor budgets и fee smoothing, не прямые market operations, как уже задано в Z00Z-Tokenomics-Incentives-Whitepaper.md.
2. Участник или сервис блокирует Z00Z в FeeSupportVault и получает право на private fee sponsorship, а не обещание доходности.
3. Псевдонимный bonded evaluator set публикует MarketRegimeCertificate с timelock и challenge window.
4. Treasury execution module по фиксированной таблице меняет только subsidy multiplier и epoch cap.
5. Конкретное потребление идет через sponsor_commitment в FeeEnvelope, а значит приватность сохраняется на уровне fee support path, а не только на уровне narrative.
6. Lock discipline и reclaim policy живут как RightLeaf profile, а не как новая отдельная денежная система.

Идеологически это очень сильная идея, потому что она стимулирует не “держи, пока всё падает”, а “держи сеть живой, capacity locked, usage flowing, while others panic”. Это намного чище и техничнее.

Если хочешь, следующим сообщением я могу сделать одно из двух:

1. Превратить это в готовую новую секцию для articles-review-ideas.md.
2. Сразу разложить это в quasi-spec с объектами уровня MarketRegimeCertificate, FeeSupportVault, SponsorAuthorization и LockSupportRight.

---

---

```text
ewe odna fishka , takka z00z ne zakritaja sitema v oslichie ot BTC ili ETH ona ne nuzhdaetsja v Link oracle chtob znat' o tom chto ptoishodit v mire eto daet vozmozhnost luchesh programirovat deistvija z00z bez opori na chuzhie platnie servises
```

Сверю это с live docs про off-chain evidence и attestation surfaces, чтобы ответить точно: где Z00Z реально может обойтись без внешнего oracle, а где всё равно нужен свой challengeable evidence lane.

Да, это очень сильная мысль. Но я бы формулировал ее чуть точнее:

Z00Z не устраняет проблему внешнего факта полностью, а радикально уменьшает долю случаев, где нужен чужой глобальный oracle в стиле Chainlink. Во многих системах почти любой факт о мире надо тянуть в chain как внешнюю истину. В Z00Z гораздо больше вещей можно оформить как нативный evidence lane: signed receipt, attestation, service log, provider proof, PaymentRequest, TxPackage, WorkPackage, scoped Evidence Package. Это хорошо видно в Z00Z-Proof-of-Useful-Work-Whitepaper.md, Z00Z-UseCases-Whitepaper.md и Z00Z-Privacy-Threat-Model-Whitepaper.md. То есть Z00Z не говорит: “нам oracle не нужен”, а говорит: “во многих сценариях truth comes from first-party receipts and challengeable attestations, а не от платного внешнего feed-а”.

В этом и есть архитектурное преимущество. Z00Z живет не как закрытая account-chain машина, а как private settlement layer for rights, claims, capacities and external-service-linked objects. Поэтому событие мира может входить в систему не как абстрактный price feed, а как конкретный объект:
merchant receipt, machine usage receipt, relay delivery proof, custody lock confirmation, payroll batch evidence, external provider attestation, bounded reviewer authorization. Это очень сильнее для programmability, потому что можно программировать реакцию на реальные operational facts, а не только на on-chain price candles. Для treasury-идей это особенно ценно: subsidy, fee smoothing, activation support, lock bonuses, sponsor budgets можно вязать к внутренним метрикам сети и first-party evidence, а не к внешнему рыночному API.

Но здесь важна граница. Не все факты мира magically become native. Есть два класса фактов.

1. Локальные и прикладные факты.
Это как раз сильная зона Z00Z. “Услуга оказана”, “пакет доставлен”, “asset locked in vault”, “worker completed task”, “merchant accepted payment”, “device consumed quota”. Такие вещи удобно выражать как receipts plus challenge windows, а не как Chainlink-style oracle dependency.

2. Глобальные и слабо наблюдаемые факты.
Цена актива, CPI, погода, election result, macro index, broad market volatility. Для них oracle problem никуда не исчезает. Просто Z00Z может заменить внешний платный feed своим собственным attestation market, reviewer quorum или regime certificate, но это все равно внешний fact surface, а не бесплатная магия.

Именно поэтому твоя идея про market-reactive treasury становится интереснее. Не надо спрашивать внешний oracle “что там на рынке” в лоб. Можно строить policy mostly from native and near-native signals:
fee demand, sponsor budget burn, relay churn, lock-unlock delta, onboarding success, merchant flow, challenger activity, bridge concentration, settlement stress. А если нужен внешний рынок, использовать его только как слабый secondary input. Тогда system reacts mostly to its own real health, а не к шуму внешней цены.

## Самая сильная версия этой мысли такая

Z00Z умеет не “узнавать мир без oracle”, а превращать большой класс real-world events в protocol-admissible evidence objects. Это переводит programmability из режима “купили чужой data feed” в режим “получили first-party receipt, проверили policy, открыли challenge window, потом исполнили приватное settlement action”.

Отсюда вытекают очень хорошие design consequences:

1. Treasury можно делать rule-bound и evidence-driven, а не oracle-driven.
2. Fee subsidies можно привязывать к measurable network stress, а не к чистой цене.
3. Machine and agent economies можно строить через bounded rights и receipts, а не через full-wallet authority.
4. Corporate и regulated use cases получают сильный путь: scoped disclosure поверх narrow settlement evidence, а не полную зависимость от внешних oracle vendors.
5. Z00Z может строить свои attestation and review markets внутри собственной архитектуры, не отдавая core semantics внешним платным сервисам.

Но я бы обязательно оставил one warning: нельзя из этого делать claim, что Z00Z “знает реальный мир без посредников”. Правильный тезис другой: Z00Z лучше других chains приспособлен к тому, чтобы мир входил в протокол через receipts, rights, attestations and challengeable evidence, а не через один privileged oracle feed.

Если хочешь, я могу следующим сообщением сделать одно из двух:

1. Сжать это в сильный абзац для articles-review-ideas.md.
2. Развернуть это в отдельную mini-architecture: EventReceipt, AttestationLane, RegimeCertificate, PolicyTrigger и TreasuryAction.

---

---

Я специально вычеркнул всё, что уже покрыто в articles-review-ideas.md: privacy linting, compliance-profile wallets, remote-scan proof hints, bounded `RightLeaf` policy runtime, package transcripts, offline e-cash research, sponsor-paid fees, rights stacking и oracle-free evidence lanes. Ниже только те оси, которые либо вообще не проговорены, либо пока существуют в corpus только как слабый намёк.

1. `Proof of Non-Control` как настоящий продукт, а не термин.  
Сейчас в corpus уже есть сам noun в Z00Z-Corpus-Terminology-Reference.md, и есть сильная логика firewall между protocol, steward и treasury в Z00Z-Legal-Architecture-Whitepaper.md и Z00Z-DAO-Whitepaper.md. Но почти нигде это не доведено до формата периодического доказуемого артефакта. Самая недосказанная идея: Z00Z может стать первой системой, где “децентрализация” и “no hidden operator merge” доказываются как `NonControlReceipt`: кто не может одновременно двигать treasury, обновлять policy, контролировать bridge path, менять agent registry и включать emergency override. Это не вариант compliance; это отдельный рынок доверия к самой системе.

2. Опциональный overlay для `solvency / liabilities proofs`, раз base protocol честно от этого отказывается.  
Документы много раз правильно говорят, что protocol не гарантирует issuer solvency, reserve integrity и custody honesty, например в Z00Z-Cross-Chain-Integration-Whitepaper.md и Z00Z-Main-Whitepaper.md. Но не хватает второй половины мысли: как именно ecosystem actor может это всё же сделать доказуемым, не ломая protocol boundary. Сильная новая ось: `ReserveEnvelope` + `LiabilityEnvelope` + selective total proofs + challenge window for mismatch. Тогда Z00Z останется архитектурно честным, но при этом станет лучшей средой для private-but-auditable issuers, bridges, lockers и corporate cash units.

3. `Proof of Non-Occurrence` как killer-feature для enterprise и governance.  
Сейчас весь narrative в основном про “доказать, что событие было”. Но в реальной жизни намного дороже часто доказать, что событие не было: не было второго redemption, не было внеполисного spend, не было emergency override, не было extra mint, не было выплаты claim вне cap. У Z00Z для этого уже есть подходящая низкоуровневая почва в state continuity и path-local proof discipline, см. Z00Z-Main-Whitepaper.md и Z00Z-HJMT-Design.md. Но этот слой пока не осмыслен как отдельный бизнес-primitive. Если Z00Z научится делать bounded absence proofs удобными, это даст не просто privacy, а audit superpower.

4. `Continuity rights` для смерти, потери ключей, смены команды, аварийной передачи полномочий.  
В Z00Z-UseCases-Whitepaper.md inheritance и break-glass проходят как scenario fuel, а не как полноценная архитектура. Это огромная недосказанность. Для private system долгоживущие права без continuity layer хрупки: человек умирает, CFO исчезает, custody team меняется, ключи теряются, предприятие реорганизуется. Здесь нужен не просто “social recovery”, а отдельная семья `ContinuityRight`: staged recovery, mortality/disappearance windows, corporate succession quorum, limited disclosure, reversible pre-activation, challengeable break-glass. Это может стать одной из самых сильных enterprise-функций Z00Z.

5. Рынок `underwriting` для внешней правды.  
Docs уже честно говорят, что service truth, custody truth и issuer honesty остаются снаружи. Но кто тогда превращает этот внешний риск в структурированную экономическую поверхность? Недосказанная идея: `CoverageRight` / `UnderwriterBond` / `FailureClassPolicy` как отдельный слой над lockers, bridges, API providers, machine services и merchant programs. Пользователь или wallet policy мог бы требовать не просто attestation, а наличие страхующего слоя с понятным compensation path. Это не treasury insurance reserve из Z00Z-DAO-Whitepaper.md, а частный рыночный слой, который делает внешний риск priceable, challengeable и сравнимым.

6. Z00Z как private workflow fabric, а не только private settlement fabric.  
Сейчас corpus очень силён в описании transfer of bounded objects, но слабо проговаривает multi-step private operational graphs. А на птичьем полёте именно тут виден большой недосказанный масштаб: purchase order -> delivery receipt -> acceptance right -> invoice claim -> payment release -> warranty right -> dispute trigger. То есть не “один private payment”, а private business process made of chained rights and claims. Это уже просматривается в Z00Z-UseCases-Whitepaper.md и Z00Z-Main-Whitepaper.md, но не доведено до тезиса. Если это развернуть, Z00Z перестаёт выглядеть как “ещё одна privacy chain” и начинает выглядеть как hidden operations layer for procurement, logistics, field work, B2B flows and institutional coordination.

7. Privacy не как feature, а как `budgeted public good`.  
В Z00Z-OnionNet-Whitepaper.md уже есть сильный намёк через cover budget. Но на уровне всей системы эта мысль ещё не доведена до конца. Если privacy реально зависит от cover traffic, witness bundles, decoy retrieval, evidence retention, quiet routing floors и anti-correlation infrastructure, то это должно жить как отдельный budget class, а не как “если операторы добрые”. Иначе privacy всегда будет деградировать при stress и low-load режимах. Очень сильная недосказанная идея: Z00Z должен уметь говорить не только “мы скрываем связи”, а “мы бюджетируем условия, при которых сокрытие остаётся правдой”.

Самые сильные из этого списка, на мой взгляд, три:

1. `Proof of Non-Control`
2. `Proof of Non-Occurrence`
3. `Solvency / Liabilities overlays`

Именно они выводят Z00Z из категории “privacy payments” в категорию “system for proving bounded truths and bounded non-truths without publicizing the whole graph”.

Если хочешь, следующим сообщением я могу сделать одно из двух:

1. Сжать это в новый раздел для articles-review-ideas.md.
2. Развернуть это в жёсткий архитектурный shortlist вида `7 proposals / why new / protocol fit / main risk / first artifact`.
