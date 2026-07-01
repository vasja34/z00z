# 032-04 Architecture Note: Public Spend Contract Without Backend Lock-In

## Purpose

This note reconciles Wave 3 with the current repository truth and with
`032-TODO.md` item `2`.

The correction is simple: `032-04` should be described as delivering a
current-stack public spend-contract verifier at the accepted boundary. It should
not be described as locking Scenario 1 to DLEQ or to any other future proof
backend.

## What 032-04 Actually Delivered

Wave 3 removed the old structural-only spend-acceptance story from the accepted
Scenario 1 path.

The implemented boundary now requires persisted spend proof and authorization
data and verifies a canonical public spend statement before the accepted flow
can proceed.

In current-stack terms, the accepted boundary binds these spend-contract inputs:

- previous root
- input references
- output leaves
- `leaf_ad_id` relation
- owner-tag relation
- asset-id relation
- balance equation
- range-proof checks
- nullifier semantics
- `chain_id`
- versioning
- canonical transcript or public-input framing

That is the architectural commitment of `032-04`.

## What 032-04 Does Not Claim

`032-04` does not claim that Scenario 1 is now committed to DLEQ.

`032-04` does not claim that a STARK/FRI stack landed.

`032-04` does not claim that every future stronger verifier must use the same
internal proof primitive as the current implementation.

`032-04` does not reopen the false witness-gate narrative. The witness path is
demoted to preparation and input-shaping; the accepted boundary is the public
spend contract.

## Backend-Agnostic Rule

The durable architecture rule is:

Any future verifier backend may replace the current proof object only if it
proves the same canonical spend statement and preserves the same fail-closed
acceptance semantics.

This keeps the architecture honest and avoids premature lock-in.

The repository can therefore evolve toward DLEQ, another proof system, or a
stronger recursive construction later, but none of those should be implied by
the `032-04` narrative unless they are actually implemented.

## Cross-Wave Reconciliation Against 032-TODO.md

### Wave 3

- `032-TODO.md` item `2` requires replacing the structural spend gate with a
  real public verifier.
- `032-04` satisfies the current-stack version of that requirement by enforcing
  an accepted public spend contract instead of structural witness-only success.
- The correct summary language is therefore `public spend-contract verifier` or
  `accepted public verifier boundary`, not `DLEQ verifier`.

### Wave 4

- `032-TODO.md` item `4` remains open.
- Wave 4 still needs authoritative checkpoint proof and spent-set enforcement at
  the storage boundary.
- Wave 4 must consume the Wave 3 spend contract as an input truth, but it must
  not restate Wave 3 as unresolved placeholder witness logic.

### Wave 5

- Secret-hygiene and simulator-RNG honesty work remain open.
- Wave 5 is operational hardening, not proof-backend selection.
- Wave 5 must not use seeded simulator behavior or debug exports to imply a
  stronger cryptographic or production posture than the code really provides.

### Deferred Claim

- `032-TODO.md` item `3` remains explicitly deferred unless a real STARK/FRI
  stack lands.
- No Wave 3, Wave 4, or Wave 5 summary may smuggle that deferred capability in
  through shorthand architecture language.

## Recommended Wording For Phase 032 Documents

Use this phrasing when describing `032-04`:

`Wave 3 replaced structural spend acceptance with a current-stack public
spend-contract verifier that binds the canonical spend statement at the accepted
boundary. This records the contract that future proof backends must satisfy,
without committing the phase to DLEQ or any other specific backend.`

Avoid this phrasing:

- `Wave 3 landed DLEQ spend verification`
- `Wave 3 committed the final proof backend`
- `Wave 3 completed trustless validator verification for the whole chain`

## Planning Consequence

Wave 3 is closed as an honesty and accepted-boundary remediation.

Wave 4 and Wave 5 remain the blocking closeout work for checkpoint truthfulness,
spent-set authority, secret-artifact hygiene, and honest final language.

---

Продолжаю в том месте, где осталась главная путаница: у вас смешиваются три разных уровня.

**1. Три разных слоя**
Вот жёсткое разделение.

| Слой | Что это такое | Что он решает | Что можно менять |
|---|---|---|---|
| `witness layer` | подготовка данных, извлечение, декодирование, связывание полей | привести входы к форме, пригодной для проверки | внутренние helper-ы, bridge-логику, формат подготовки |
| `verifier backend` | механизм доказательства и проверки | как именно доказывается canonical spend statement | proof primitive, transcript engine, proving system |
| `DA backend` | слой доступности данных | где публикуются/хранятся данные доказательства или batch | Celestia, другой DA, локальный storage path |

То есть:

- witness_gate.rs это `witness layer`
- spend_verification.rs это `accepted verifier boundary`
- Celestia, если появится, будет относиться к `DA/backend for data publication`, а не к этой фразе из note
- STARK/FRI, DLEQ и подобное относятся к `proof backend`

**2. Что значит “witness_gate не место окончательного trust decision”**
Это значит:

- код в witness_gate.rs может:
  - собрать inputs
  - расшифровать pack
  - вытащить `leaf_ad_id`
  - построить proof inputs
  - проверить локальные подготовительные инварианты
- но он не должен быть тем местом, где система говорит:
  - “этот spend принят окончательно”
  - “этот tx cryptographically valid на accepted boundary”

Это должен говорить verifier в spend_verification.rs, потому что именно там проверяется публичный контракт целиком:
- previous root
- input refs
- output leaves
- `leaf_ad_id` relation
- owner-tag relation
- asset-id relation
- balance equation
- range proofs
- nullifier semantics
- `chain_id`
- version
- transcript framing

То есть `witness_gate` отвечает на вопрос:
“я смог подготовить материал?”

А `spend_verification` отвечает на вопрос:
“этого материала достаточно, чтобы сеть/accepted flow доверял spend?”

**3. Где тут Celestia**
Напрямую здесь её нет.

Celestia отвечает на другой вопрос:
“данные batch/proof доступны и опубликованы?”

Она не отвечает на вопрос:
“какое именно криптографическое утверждение считается доказанным?”
и не отвечает на вопрос:
“какой verifier принимает spend?”

Поэтому если говорить очень строго:

- Celestia может быть частью Wave 4/дальше, где речь про checkpoint truth, publication, blob/data availability
- но она не является ответом на фразу про `accepted public verifier boundary`

**4. Где тут STARK/FRI**
STARK/FRI это уже пример возможного `proof backend`.

Если завтра вы решите:
- вместо текущего proof object использовать STARK/FRI,
- или recursive construction,
- или другой verifier stack,

то architecture note говорит:
это нормально, **если** новый backend доказывает тот же canonical spend statement и не ослабляет fail-closed semantics.

Именно поэтому note в .planning/phases/032-crypto-audit-scenario-1/032-04-ARCHITECTURE-NOTE.md запрещает писать, будто “Wave 3 already landed STARK/FRI”. Потому что это был бы claim про конкретный backend, а Wave 3 зафиксировала boundary и statement, а не финальный proving stack.

**5. Почему backend вообще могут захотеть менять**
Не потому что “любим переписывать”, а потому что у backend-ов разные tradeoff-ы.

Например:

1. Один backend проще и быстрее доставить сейчас.
2. Другой даёт прозрачность assumptions, например без trusted setup.
3. Один лучше для recursion/aggregation.
4. Один дешевле для verifier-а.
5. Один лучше подходит для on-chain verification.
6. Один лучше для large-batch proving.
7. Один проще для аудита и поддержки прямо сейчас.

То есть вопрос не “почему не сделать максимально хороший прямо сейчас?”, а:
“что именно считать хорошим под текущую фазу?”

Если цель текущей фазы:
- перестать врать про accepted spend verification,
- заменить structural witness-only acceptance на реальный verifier boundary,
- закрыть honesty gap,

то для этого не обязательно уже сейчас тащить самый тяжёлый и самый долгий backend из всех возможных.

**6. Почему не сделать “максимально хороший backend” сейчас**
Потому что это почти всегда плохая инженерная ловушка.

Есть три причины.

1. **Не стабилизирован statement**
Сначала надо жёстко зафиксировать, **что именно** доказывается.
Именно это и делает Wave 3.
Если сначала зашить большой backend, а statement потом ещё поплывёт, вы получите дорогую переделку.

2. **Цена интеграции может быть неадекватной текущей цели**
Если задача фазы честно закрыть boundary, то тащить сразу большой STARK/FRI stack может быть слишком дорогим по:
- времени
- сложности
- аудиту
- тестированию
- риску новых багов

3. **Можно переусложнить до того, как доказан базовый контракт**
Сначала команда должна уметь честно сказать:
“accepted path проходит только через реальный verifier”.
Потом уже можно сказать:
“теперь улучшаем сам proving backend”.

Именно поэтому note продвигает последовательность:
- сначала честный boundary
- потом backend upgrade при необходимости

**7. Что значит “если новый backend доказывает тот же canonical spend statement”**
Это значит, что можно менять **двигатель**, но нельзя менять **смысл доказательства** без архитектурного пересмотра.

Хороший пример замены backend:
- раньше verifier принимал один формат proof/auth object
- потом вы переходите на другой proving system
- но verifier всё ещё проверяет тот же набор публичных фактов о spend

Это архитектурно допустимо.

Плохой пример “замены backend”, которая уже не просто замена backend:
- новый backend больше не связывает `leaf_ad_id`
- или перестаёт связывать `chain_id`
- или ослабляет nullifier semantics
- или меняет acceptance с fail-closed на “best effort”

Это уже не просто новый backend. Это уже изменение самого trust contract.

**8. Очень короткий ответ на ваш вопрос “зачем менять backend?”**
Менять backend имеет смысл только если появляется конкретная выгода:

- дешевле verifier
- сильнее security assumptions
- лучше recursion
- лучше масштабируемость
- лучше on-chain/integration story
- честнее/проще future architecture

Если такой выгоды нет, менять backend не надо.

То есть note не говорит:
“backend обязательно будут менять”.

Он говорит:
“не врите, будто текущая фаза уже зафиксировала backend навсегда”.

**9. Самая короткая формула**
Вот правильная ментальная модель:

- `witness_gate` = подготовка
- `spend_verification` = доверительное решение на accepted boundary
- STARK/FRI/DLEQ = варианты proof backend
- Celestia = возможный DA/backend, но это другой слой
- Wave 3 = зафиксировала honest verifier boundary
- Wave 3 = не зафиксировала окончательный proof backend навсегда

---

Вот жёсткая практическая таблица.

| Изменение | Это просто смена backend | Это уже смена архитектуры | Это вообще другой слой | Почему |
|---|---|---|---|---|
| Заменить внутренний proof object в verifier на другой, но оставить тот же canonical spend statement | Да | Нет | Нет | Меняется механизм доказательства, но не меняется смысл того, что считается доказанным |
| Перейти с текущего verifier stack на DLEQ, если он доказывает тот же набор публичных связей | Да | Нет | Нет | Это замена proof backend, а не trust contract |
| Перейти на STARK/FRI, если проверяются те же `prev_root`, input refs, output leaves, `leaf_ad_id`, owner-tag, asset-id, balance, nullifier, `chain_id`, version, transcript framing | Да | Нет | Нет | Тот же statement, другой proving system |
| Добавить recursion/aggregation поверх того же spend statement | Да | Нет | Нет | Это улучшение backend-а по масштабу и packaging |
| Изменить размер proof, формат proof blob, сериализацию внутренних proof данных | Обычно да | Нет, если semantics не меняются | Нет | Это internal backend detail |
| Ускорить prover/verifier, не трогая public contract | Да | Нет | Нет | Производственная оптимизация без смены смысла |
| Убрать из проверяемого statement `leaf_ad_id` relation | Нет | Да | Нет | Это уже ослабление доверительного контракта |
| Убрать binding к `chain_id` | Нет | Да | Нет | Меняется безопасность statement, это не просто backend swap |
| Убрать nullifier semantics из acceptance | Нет | Да | Нет | Это уже другой spend contract |
| Перестать fail-closed и принимать “best effort” или placeholder witness success | Нет | Да | Нет | Меняется acceptance semantics |
| Перенести окончательное решение из spend_verification.rs обратно в witness_gate.rs | Нет | Да | Нет | Меняется trust boundary |
| Оставить тот же verifier, но изменить meaning input refs или output leaves | Нет | Да | Нет | Это уже новый архитектурный контракт |
| Добавить Celestia для публикации batch/proof data | Нет | Нет | Да | Это DA/publication layer, не proof backend |
| Перенести checkpoint blobs в Celestia | Нет | Нет | Да | Это data availability / publication plumbing |
| Сменить storage/checkpoint publication path без изменения spend verifier | Нет | Нет | Да | Это не тот слой, где принимается spend |
| Заменить local storage witness source на другой resolver, сохранив тот же accepted verifier boundary | Нет или частично | Обычно нет | Частично другой слой | Это data/input preparation layer, а не обязательно proof backend |
| Переписать `witness_gate` так, чтобы он только готовил input shape, а не решал acceptance | Нет | Нет, это приведение к правильной архитектуре | Нет | Это не смена backend, а исправление boundary discipline |
| Ввести новый canonical spend statement и под него новый verifier | Нет | Да | Нет | Это уже не swap backend, а новый архитектурный договор |
| Добавить внешний DA плюс новый recursive prover одновременно | Частично | Может быть | Частично | Это смешанный случай: proof backend и DA layer меняются независимо и их нельзя сваливать в одно |

Самая короткая интерпретация:

- **Смена backend**: меняется “чем доказываем”.
- **Смена архитектуры**: меняется “что именно должно быть доказано” или “где проходит trust boundary”.
- **Другой слой**: меняется “где лежат/публикуются данные”, а не “как проверяется spend”.

Ещё короче на ваших примерах:

- `DLEQ` или `STARK/FRI` при том же statement: это **смена backend**.
- `Celestia`: это **другой слой**, DA.
- Возврат к structural witness acceptance: это **слом архитектуры**, не backend swap.
- Удаление `chain_id` или nullifier binding из acceptance: это **смена архитектуры**, потому что вы ослабляете сам contract.

Если хотите, я следующим сообщением заполню ту же таблицу уже **конкретно для вашего репо** примерами вида:

- witness_gate.rs change
- spend_verification.rs change
- state_update.rs change
- `Celestia integration` change

---
---


**Вердикт**
- У вас в live code сейчас не STARK/FRI backend. Это прямо согласовано и кодом, и planning notes: current accepted path опирается на public spend-contract verifier в spend_verification.rs, а Phase 032 отдельно запрещает описывать Wave 3 как STARK/FRI-landed в 032-04-ARCHITECTURE-NOTE.md и прямо говорит, что live STARK/FRI stack ещё не implemented в 032-TODO.md.
- Live proof primitive, который реально видно в коде, это Bulletproofs+ для range proofs. Это зафиксировано в prover.rs, в crate overview lib.rs, и в vendored backend docs README.md.
- Тезисы вроде “этот backend дешевле для verifier-а”, “тот лучше для on-chain”, “тот проще для аудита”, “тот лучше для recursion” в прошлом ответе не были repo-backed фактами. Это общие архитектурные эвристики, а не утверждения, которые кодовая база сама доказывает. Их надо считать не верифицированными относительно этого репо.

**Что проверено, а что нет**
- Проверено: текущий accepted boundary реально делает много публичных проверок, а не просто structural witness prep. В spend_verification.rs он связывает `prev_root`, input refs, output leaves, `leaf_ad` hash relation, uniqueness constraints, balance equation, range proofs и spend authorization signature.
- Проверено: current stack uses Bulletproofs+ range proofs, а не STARK/FRI. Это видно в prover.rs и подтверждено в 032-TODO.md.
- Проверено: underlying Bulletproofs+ backend умеет aggregation и batch verification. Это видно в backend_tari.rs, в README.md, и в tests test_bulletproofs.rs и perf_guards.rs.
- Не проверено как repo fact: что именно STARK/FRI был бы “дешевле verifier-у” или “лучше on-chain” именно для вашего проекта. Это может быть правдоподобно как общее крипто-сравнение, но не выводится из текущего дерева.
- Не проверено как repo fact: что текущий live stack уже даёт полноценную recursive/trustless validator verification for whole chain. Phase docs как раз запрещают так говорить сейчас в 032-04-ARCHITECTURE-NOTE.md и 032-CONTEXT.md.

**Ответы на ваши прямые вопросы**
1. Почему STARK/FRI не делают прямо сейчас
   Repo-backed ответ такой: Phase 032 сознательно сначала закрыла honesty gap на accepted boundary, не фиксируя final proof backend. Это прямо написано в 032-04-ARCHITECTURE-NOTE.md: Wave 3 landed current-stack public spend-contract verifier, но не committed to DLEQ/STARK/FRI. А в 032-TODO.md даже расписаны две честные опции: либо реально внедрить STARK/FRI stack, либо явно снять этот claim. Сейчас репо находится во второй ветке.

2. Нужен ли current backend trusted setup
   Для live range-proof component — признаков trusted setup нет. Наоборот, текущий live proof piece это Bulletproofs+, а по официальному Tari Labs explainer “Bulletproofs provide short, non-interactive, zero-knowledge proofs without a trusted setup”: https://tlu.tarilabs.com/cryptography/the-bulletproof-protocols
   На уровне репо это согласуется с тем, что в live spend path видны Bulletproofs+ range proofs и signatures, а не SNARK ceremony-dependent stack. То есть честная формулировка: в текущем live spend path trusted-setup dependency не surfaced.

3. Насколько current backend вообще сильный
   Сильный:
   - для скрытия сумм и доказательства диапазона сумм,
   - для explicit public statement binding,
   - для fail-closed checks по input/output relations, balance и authorization.

   Несильный в том смысле, в каком вы спрашиваете про “final trustless zk backend”:
   - это не live STARK/FRI stack,
   - это не recursive checkpoint proof,
   - это не единый succinct proof того, что весь spend/state transition доказан одной прозрачной схемой.

   То есть текущий стек нельзя честно называть “слабым” или “игрушечным”, но и нельзя честно продавать как завершённую trustless recursive proving architecture.

4. Подходит ли current backend для on-chain verification
   Repo-backed ответ: это не доказано и не реализовано как deliverable. Я не нашёл live on-chain verifier path, verifier contract, EVM verifier integration или похожую поверхность. Текущий код — Rust-side verification над текущими wire/public objects и Ristretto/Bulletproofs+ stack в spend_verification.rs и prover.rs. Поэтому честно говорить так: current backend on-chain-ready не подтверждён кодовой базой.

5. Подходит ли current backend для large-batch proving
   Здесь ответ двойной.
   - На уровне primitive/backend: да, Bulletproofs+ backend умеет aggregation и batch verification. Это видно в README.md и backend_tari.rs.
   - На уровне live wallet spend path: пока нет полноценного large-batch pipeline. В prover.rs `create_batch_proofs()` и `verify_batch_proofs()` сейчас фактически просто итерируют по одиночным proof’ам, а не используют сильный batch path из crypto backend.

   Поэтому честная формулировка: backend capability есть, но live accepted spend path пока не использует её как полноценный large-batch verifier.

Итог в одну строку: сейчас у вас live stack = current-stack public spend-contract verifier + Bulletproofs+ range proofs + spend authorization signatures; STARK/FRI, recursive checkpoint proofs, on-chain verifier suitability и “final backend superiority claims” пока нельзя считать подтверждёнными фактами репозитория.

Если хотите, следующим сообщением я могу сжать это в короткую таблицу вида “тезис → VERIFIED / PLAUSIBLE / NOT REPO-BACKED / FALSE”.

---

## 🎯 PROC

1. Здесь важно разделить **следующую phase** и **следующий milestone**.
2. Для `z00z` полноценный переход на `STARK/FRI` — это не локальный refactor, а замена proof backend-а:
   - формализация canonical statement,
   - AIR/trace design,
   - новый proof object,
   - новый verifier path,
   - новые тесты, perf и security review.
3. Поэтому как **обычная следующая GSD implementation phase** это слишком тяжело.
4. Как **следующая GSD research/spike/prototype phase** — да, это нормально.
5. По `Winterfell`: он даёт STARK prover/verifier, но **recursive proving не является его “default turnkey mode”**.

## ✅ CONC

**Коротко:**
- **Да**, это можно брать **следующей GSD phase**, но **только как spike / design / PoC phase**.
- **Нет**, это не выглядит как “ещё одна обычная phase с кодом”. Скорее это **новый mini-project**, а часто и **новый milestone** из нескольких phases.

**Про recursion:**
- **Winterfell proof не является рекурсивным по умолчанию.**
- То есть ответ: **нет, не в том смысле, что вы просто берёте Winterfell и сразу получаете recursive proof stack**.
- Если нужна recursion, её придётся проектировать отдельно, и это заметно усложняет работу.

## 🚩 Практический вывод для `z00z`

Если цель следующей работы:
- **сделать первый реальный STARK/FRI PoC для spend statement** → `Winterfell` ещё нормальный кандидат;
- **сразу идти в recursive checkpoint proofs / aggregation** → это уже выглядит **слишком тяжело для одной phase**, и `Winterfell` тут **не самый удобный старт**.

## 📌 Как правильно оформить в GSD

Лучше так:

1. **Phase 1 — STARK spike**
   - freeze minimal statement,
   - выбрать narrow PoC,
   - поднять tiny Winterfell prover/verifier.

2. **Phase 2 — compatibility review**
   - понять, что остаётся вне STARK,
   - как жить с range proofs / authorization / roots / nullifiers.

3. **Phase 3 — decision gate**
   - оставаться на current stack,
   - или идти дальше в full backend migration,
   - или менять кандидат backend-а.

**Итог:**
**следующей phase — да, как spike; следующей “обычной feature phase” — нет.**
**Winterfell — не recursive-by-default.**

Если нужно, следующим сообщением могу дать **очень жёсткий 3-phase GSD plan именно под Winterfell PoC для `z00z`**.

---

Проверю именно recursion-часть у `plonky2` и `plonky3`: registry metadata плюс docs по recursion crates, чтобы ответить жёстко — это готовая нужная вам рекурсия или только infrastructure для неё.

Ran terminal command:  cargo info plonky2

Ran terminal command:  cargo info p3-recursion

Ran terminal command:  cargo info p3-circuit-prover



Fetched 3 resources

Доберу один уровень выше по источникам — readme/репозитории самих Plonky2 и Plonky3 recursion, чтобы развести “headline recursion feature” и “готовый production-friendly stack для вашего statement”.

Ran terminal command:  cargo search plonky3 --limit 30

Ran terminal command:  cargo search plonky2 --limit 20



Fetched 3 resources

Compacted conversationСинтезирую в короткий вывод: что реально умеет каждый стек, и что из этого годится как следующий шаг для z00z, а что пока только для spike/R&D.

Ran terminal command:  ./scripts/play_tone.sh

**Проверка**

По источникам ответ такой:

1. `Plonky2` действительно даёт recursion.
2. `Plonky3` тоже действительно даёт recursion.
3. Но для `z00z` это не означает “можно просто быстро заменить текущий backend и ехать в production”.

**Точный вывод**

`Plonky2`:
- Да, recursion у него встроена как ключевая фича.
- Это прямо видно по описанию crate: “Recursive SNARKs based on PLONK and FRI”.
- В документации у него есть отдельный `recursion` module и even `bench_recursion`.
- Но стратегически это плохой выбор для нового внедрения: официальный README помечает `Plonky2` как deprecated и рекомендует переходить на `Plonky3`.

`Plonky3`:
- Да, recursion тоже есть, и уже не только “в теории”.
- Для этого есть отдельный стек `Plonky3-recursion` / `p3-recursion` / `p3-circuit-prover`.
- Там прямо заявлены:
  - recursive verification,
  - multi-layer recursion,
  - 2-to-1 recursive aggregation,
  - in-circuit FRI verification.
- То есть если тебе нужен именно recursive proof / proof composition / folding path, то `Plonky3` ближе к нужной цели, чем `Winterfell`.

Но критический practical caveat:
- `Plonky3-recursion` сам пишет, что код under active development,
- не audited,
- и не recommended for production software.

**Что это значит для z00z**

Для текущего `z00z`:
- живой accepted path всё ещё current-stack, а не `Plonky2/3`, см. prover.rs и spend_verification.rs.
- Поэтому ответ на “дают ли Plonky2/3 recursion, который мне нужен?” такой:

Да, дают.
Но:
- `Plonky2` даёт recursion, но как база для нового production-перехода уже не годится, потому что deprecated.
- `Plonky3` даёт recursion и даже aggregation, это правильнее как направление.
- Однако `Plonky3-recursion` сейчас годится скорее для `R&D / spike / next-phase prototype`, а не для немедленной замены accepted verifier boundary в production.

**Практическая рекомендация**

Если формулировать жёстко:

1. Если нужен research prototype recursive backend: брать `Plonky3`, не `Plonky2`.
2. Если нужен production-ready swap прямо сейчас: ни `Plonky2`, ни `Plonky3-recursion` я бы сейчас не назвал безопасным “drop-in choice”.
3. Если нужен следующий разумный GSD phase: делать отдельный spike на `Plonky3-recursion` с одной узкой целью:
   - доказать, что ваш canonical spend statement можно завернуть в recursive aggregation pipeline без ломки accepted contract.

**Источники**
- Plonky2 README: deprecated, recursion benchmarks, PLONK+FRI positioning
  https://github.com/0xPolygonZero/plonky2
- Plonky2 docs.rs: отдельный `recursion` module
  https://docs.rs/plonky2/latest/plonky2/
- Plonky3 core README: toolkit for PIOPs / STARK-based systems
  https://github.com/Plonky3/Plonky3
- Plonky3-recursion README: recursive verification, aggregation, explicit “not recommended for production”
  https://github.com/Plonky3/Plonky3-recursion

Если хочешь, я следующим сообщением дам уже совсем прикладную матрицу:
1. `Winterfell vs STWO vs Plonky3` для `z00z`
2. что брать для `PoC`
3. что брать для долгого production roadmap.

---

## Practical Comparison For Your Requirements

### 1. No trusted setup
- **Plonky3-recursion:** better fit from the checked public evidence.
  Reason: it is a recursive STARK/FRI stack and the checked materials do not introduce a setup ceremony.
- **Microsoft/Nova:** only conditionally fits.
  Reason: the official project supports both transparent-looking and setup-requiring modes, and the README explicitly documents universal setup for HyperKZG and Mercury.

### 2. Recursion
- **Plonky3-recursion:** strong yes.
  Recursive verification, recursive aggregation, and multi-layer recursion are all explicit.
- **Microsoft/Nova:** strong yes.
  Recursive SNARK / IVC is the core purpose of the project.

### 3. Simplicity of integration into current z00z
- **Plonky3-recursion:** likely harder.
  Reason: separate recursion stack, multi-component architecture, incomplete docs, explicit builder/config plumbing, and explicit production warning.
- **Microsoft/Nova:** likely easier.
  Reason: clearer recursive core, documented frontends, and no checked production-use warning. But this comes with the setup caveat above.

---

## Conc

If I compress the whole check into one decision-oriented conclusion:

1. **If your absolute top constraint is “no trusted setup, no ambiguity”**:
   - Plonky3-recursion is the cleaner conceptual match.
   - But it is still an R&D choice, not a production choice, because the project explicitly says so.

2. **If your top constraint is “working recursive system with a simpler integration story”**:
   - Microsoft/Nova looks better.
   - But you must lock the exact instantiation and avoid casually drifting into HyperKZG or Mercury if your architecture forbids trusted setup.

3. **For current z00z specifically**:
   - The repository is still on a Bulletproofs+ plus public spend-contract path, so either direction is a major subsystem addition.
   - The safest next step is not immediate adoption but a narrow spike:
     - one state-transition proof,
     - one recursive step,
     - one explicit setup policy decision,
     - one measurement of proof size and verifier cost.

---

## Limitations

- This tool accelerates human verification; it does not replace it.
- Web search results may not include the most recent information or paywalled sources.
- The adversarial review uses the same underlying model that may have produced the original output. It catches many issues but cannot catch all of them.
- A claim rated VERIFIED means a supporting source was found, not that the claim is definitely correct. Sources can be wrong too.
- Claims rated PLAUSIBLE may still be wrong. The absence of contradicting evidence is not proof of accuracy.

Если хочешь, следующим сообщением я сведу это в совсем короткую инженерную рекомендацию на 5 строк:
- что брать для spike,
- что брать для long-term roadmap,
- и какой риск ты покупаешь в каждом варианте.

-----------


Если твой выбор это `Plonky3-recursion`, то от тебя требуется не “подключить библиотеку”, а зафиксировать новый proof layer для `z00z`.

Сейчас у тебя активный путь это `Bulletproofs+` для range proofs и wire-level accepted verifier в prover.rs и spend_verification.rs. `Plonky3-recursion` не заменяет это одним патчем. Оно потребует от тебя выбрать, что именно ты доказываешь рекурсивно: скорее всего не приватный spend per tx, а state-transition / checkpoint correctness поверх текущего transaction layer.

**Что это от тебя потребует**

1. Зафиксировать точный statement.
   Нужно письменно определить одну проверяемую функцию перехода:
   `root_old + block_delta/nullifier_delta -> root_new`.
   Без этого recursion stack не к чему прикручивать.

2. Выбрать granularity recursion.
   Ты должен решить:
   - proof на каждый блок,
   - proof на epoch,
   - или aggregation нескольких блоков в один recursive step.
   Это влияет на proof size, latency и сложность интеграции.

3. Принять, что первый этап будет spike, не production rollout.
   По официальному README `Plonky3-recursion` ещё under active development, not audited, not recommended for production use.
   Значит, твой первый commit-level goal должен быть:
   “доказать один recursive state transition end-to-end”, а не “перевести всю сеть”.

4. Согласиться на отдельный crate / subsystem.
   Практически это почти наверняка новый модуль уровня вроде:
   - `z00z_recursive_proofs`
   - или отдельный proof adapter crate
   с собственными types для:
   - public inputs,
   - transition witness,
   - recursion proof object,
   - verifier API.

5. Определить public inputs как канон.
   Тебе надо решить, что будет публично входить в proof:
   - `prev_root`
   - `new_root`
   - epoch/block number
   - chain/domain separator
   - optional digest of block contents
   - optional nullifier-set root
   Это должен быть стабильный контракт, как у тебя уже сделано для accepted public spend contract.

6. Выбрать hash/field stack.
   `Plonky3-recursion` тащит STARK/FRI circuit stack и явно завязан на свою конфигурацию, включая FRI params и circuit builder operations.
   То есть от тебя нужен выбор:
   - поле,
   - hash family,
   - commitment path,
   - recursion parameters.
   Это уже crypto-architecture decision, не просто dependency management.

7. Разделить “что остаётся как есть” и “что уходит в recursion”.
   Самый реалистичный путь для `z00z`:
   - текущий tx/privacy/range-proof слой оставить,
   - recursion добавить на уровень checkpoint/state validity.
   Это сильно проще, чем пытаться немедленно переписать все spends в новый proof backend.

8. Подготовить proving infrastructure.
   Тебе понадобится:
   - prover runtime,
   - benchmark harness,
   - proof serialization,
   - checkpoint artifact integration,
   - verifier entrypoint.
   Сейчас в фазовых заметках recursive checkpoint pipeline ещё roadmap-only, а не live implementation, см. 20-Recursive checkpoint proof.md.

9. Принять operational cost.
   `Plonky3-recursion` это не “дешёвая прозрачность”.
   Ты покупаешь:
   - большую arithmetization complexity,
   - более тяжёлую prover integration surface,
   - incomplete docs,
   - необходимость internal validation before trust.

10. Принять честную rollout strategy.
   Для тебя правильная последовательность выглядит так:
   - Phase A: one-block state-transition proof
   - Phase B: recursive chaining over 2-5 steps
   - Phase C: checkpoint artifact embedding
   - Phase D: benchmark and failure model
   - Phase E: only then discuss production gating

**Коротко: что требуется лично от тебя как от архитектора**

- выбрать exact proof statement;
- выбрать block vs epoch recursion;
- согласиться на spike-first rollout;
- не пытаться сразу заменить текущий spend verifier;
- принять, что `Plonky3-recursion` сейчас это R&D-ставка, а не готовый infra commodity.

**Мой практический совет**

Если выбор уже сделан, твой следующий правильный шаг не “интегрировать Plonky3-recursion везде”, а утвердить очень узкий target:

1. Доказать один `checkpoint transition`.
2. Публичные входы: `prev_root`, `new_root`, `epoch_id`, `delta_commitment`.
3. Без замены текущего `Bulletproofs+` spend path.
4. С recursion chain хотя бы на 3-5 шагов.
5. С измерением proof size, prover time и verifier time.

Это и будет минимальный честный входной билет в твой выбор.

Если хочешь, я следующим сообщением разложу это уже в формате:
1. `что надо решить тебе сегодня`
2. `что надо сделать мне в коде`
3. `какой первый GSD phase открыть под Plonky3-recursion`.