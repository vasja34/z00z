# Scenario 1 Storage Follow-Up

Scenario 1 follow-up for storage integration.

This file tracks only the remaining `scenario_1` work after the current
readiness analysis. It is intentionally narrower than
`specs/014-z00z-storage/публикации assets в JMT.md` and must not repeat the
current-state evidence already captured there.

## Scope

- Use this file for unresolved follow-up gaps in Stage 4, Stage 6, and
  wallet/storage boundaries.
- Use `specs/014-z00z-storage/публикации assets в JMT.md` for the current
  readiness picture and code-backed observations.
- Use `specs/014-z00z-storage/snapshot-storage-spec.md` for pre-state snapshot
  requirements.
- Use `specs/014-z00z-storage/checkpoint-storage-spec.md` for checkpoint
  artifact requirements.
- Use `specs/014-z00z-storage/jmt-gaps-workflow.md` for the broader migration
  sequence.

## Remaining Gaps

| Area | Required contract | Current follow-up gap |
| --- | --- | --- |
| Stage 4 root ownership | Pre-state witness preparation must consume a storage-owned root | Stage 4 still reconstructs `prev_root` through compatibility-root style preparation instead of taking the root from one storage-owned source. |
| Stage 4 witness shape | Pre-state witness bytes must stay canonical storage witness bytes | Stage 4 still rebinds `proof_blob()` output to a local `ProofItem` before writing `checkpoint_prep.json`. |
| Snapshot identity | Snapshot identity must be explicit and reusable | `checkpoint_prep.json` is still file-coupled simulator output rather than an explicit content-addressed snapshot contract. |
| Wallet membership boundary | First downstream witness boundary must require canonical proof decoding | `state_update.rs` still accepts opaque non-empty witness bytes and does not perform storage-level `chk_blob(...)` validation itself. |
| Input path binding | Input resolution must stay bound to full `AssetPath` semantics | The downstream boundary is still keyed primarily by compact input refs instead of one typed path-and-proof contract. |
| Stage 6 execution backend | Checkpoint apply must run against storage-backed state | Stage 6 still applies the tx batch through `SimState` rather than a real `AssetStore` adapter implementing the same state-update hooks. |
| Checkpoint artifact separation | Draft execution output and final checkpoint artifact must stay distinct | The simulator path still needs a cleaner separation between local apply output, canonical checkpoint artifact, and audit-only helpers. |
| Tx lifecycle ownership | Storage should own the regular-spend resolve and apply lifecycle | Regular transfer still depends on simulator-managed prep/proof scaffolding instead of one storage-native `resolve -> verify -> apply` path. |

## Non-Goals

- Do not restate whether `claim` outputs are already storage-ready.
- Do not restate whether regular transfer outputs already carry `asset_wire`.
- Do not restate the current Stage 4 and Stage 6 code evidence already listed
  in `specs/014-z00z-storage/публикации assets в JMT.md`.

## Implementation Order

1. Remove compatibility-root reconstruction from Stage 4 witness preparation.
2. Make the first wallet-side witness boundary decode and validate canonical
   storage witness bytes.
3. Preserve full path binding across the pre-state handoff instead of relying
   on compact refs alone.
4. Introduce an `AssetStore`-backed adapter for `AssetState` and `MemberIdx`.
5. Move Stage 6 batch application from `SimState` to the storage-backed
   adapter.
6. Split checkpoint draft output from the final canonical checkpoint artifact.

## TODO

Сначала проверю `scenario_1`, wallet/storage boundary и репо-память по JMT gaps, чтобы ответить не абстрактно, а по текущим типам и точкам интеграции.

Короткий вывод:

- `claim` в `scenario_1` уже готов для публикации assets в JMT на стороне `outputs`.
- перевод `coins` от Alice к Bob тоже уже почти готов, но не как один самостоятельный `TxPackage`.
- для regular transfer канонический JMT-вход сейчас задаётся связкой:
  `TxPackage` + `checkpoint_prep.json`
- если смотреть строго, “готов ли сам формат tx без доп. артефактов”:
  - `claim`: да, для публикации новых leaf в JMT
  - `alice -> bob`: нет, только частично; нужен pre-state/proof слой рядом с tx

**Что уже готово**

- Формат `claim` уже несёт полноценный `output.asset_wire`, который можно превратить в `AssetLeaf` и положить в storage. Это видно в claim_tx.rs и claim_tx.rs.
- Симулятор уже публикует claim-outputs в storage через канонический путь `definition_id -> serial_id -> asset_id`, а не через старый “плоский” alias. Это видно в claim_pkg_consumer.rs и claim_pkg_consumer.rs.
- Формат regular transfer уже делает `inputs` reference-only по каноническому terminal `asset_id` и `serial_id`. Это видно в tx_verifier.rs и в построении Stage 4 stage_4.rs, stage_4.rs.
- Для regular transfer `outputs` тоже уже storage-ready, потому что `TxOutputWire` несёт `asset_wire`. См. tx_verifier.rs.
- Storage-слой уже умеет всё базовое:
  `put_item`, `del_item`, `apply_ops`, `proof_blob`, `chk_blob`. См. README.md и proof.rs.

**Где именно разница между `claim` и `alice -> bob`**

- `claim`-tx по сути создаёт новый leaf.
- Его `input` не является JMT membership-входом. В `ClaimInputWire` лежат `claim_id_hex`, `claim_source_asset_id_hex`, `claim_source_commitment_hex`, а не `prev_root + witness + AssetPath`. См. claim_tx.rs.
- Поэтому `claim` уже готов именно для `publish into JMT`, а не для “consume existing JMT state”.
- Regular tx наоборот тратит уже существующие assets из state.
- Поэтому его compact `inputs` сами по себе недостаточны: они только ссылаются на leaf по `asset_id_hex + serial_id`. См. tx_verifier.rs.
- Реальный pre-state для этих inputs сейчас уходит в отдельный артефакт `checkpoint_prep.json`, который содержит `definition_id_hex`, сам `leaf` и `member_wit_hex`. Это видно в stage_4.rs.

**Готовность по факту**

- `claim`
  - `outputs`: готовы
  - `inputs`: для JMT не нужны в том же смысле, что у transfer
  - verdict: готово для вставки claimed assets в JMT
- `alice -> bob`
  - `outputs`: готовы
  - `inputs`: готовы только как `input_ref`, не как self-contained storage witness
  - verdict: готово для current simulator pipeline, но не готово как полностью автономный canonical JMT tx format

**Что уже делает симуляция для regular tx**

- Stage 4 вычисляет канонический terminal `asset_id` из decryptable input leaf, а не берёт случайный alias. См. stage_4.rs.
- Stage 4 строит `prep`-файл из выбранных входов.
- Stage 4 поднимает временный `AssetStore`, кладёт туда эти pre-state rows и получает `proof_blob` для каждого input. См. stage_4.rs и stage_4.rs.
- Но Stage 4 потом делает compatibility bridge через `rebind(...)` под `prev_root`, а не оставляет storage-owned proof contract как primary source.
- Stage 6 уже умеет валидировать этот witness через `chk_blob(...)`, потом строит `TxPkgSum` через `prepare_tx_sum(...)`, потом делает `apply_batch_checkpoint(...)`. См. stage_6.rs, stage_6.rs, stage_6.rs.

**Главный незавершённый кусок**

Главный gap не в том, что `outputs` плохие.  
Главный gap в том, что regular tx пока не является self-contained storage-native transaction package.

Сейчас модель такая:

- `TxPackage` содержит compact input refs
- `checkpoint_prep.json` содержит resolved pre-state leaves и membership witnesses
- Stage 6 применяет это не к реальному `AssetStore`, а к `SimState` in-memory map

То есть сейчас pipeline уже доказывает, что формат почти правильный, но окончательная canonical ownership ещё размазана между simulator и storage.

**Что надо доделать со стороны симуляции**

1. Перестать считать один только `TxPackage` достаточным для JMT execution.
2. Сделать явный контракт: `TxPackage` всегда идёт вместе с storage-owned prep/proof artifact.
3. Убрать compatibility bridge `blob.rebind(item)` из Stage 4 и опираться на storage-native proof/root contract.
4. В Stage 6 применять tx не к `SimState`, а к адаптеру над реальным `AssetStore`.
5. Перенести canonical proof validation как можно ближе к моменту подготовки tx, а не держать её только в Stage 6.

**Что надо доделать со стороны `z00z_storage`**

1. Базовые примитивы уже есть; здесь не нужен новый proof format с нуля.
2. Нужен storage adapter/API для regular tx lifecycle:
   - resolve `TxInputWire` в canonical `AssetPath + AssetLeaf + ProofBlob`
   - выдать witness под конкретный `prev_root`
   - применить `spent_delta` и `created_delta` прямо к `AssetStore`
   - вернуть новый `AssetStateRoot` и `CheckRoot`
3. Нужна прямая реализация/адаптер для `AssetState` и `MemberIdx` поверх `AssetStore`, чтобы wallet/state-update код работал не через симуляторский `SimState`, а через storage.
4. Желательно, чтобы storage сам владел упаковкой membership witness для transfer path так же, как он уже владеет `proof_blob()` и `chk_blob()`.

**Практический итог**

Если ответить совсем прямо:

- `claim`-формат в `scenario_1` уже готов, чтобы взять его output-assets и встроить их в JMT.
- `alice -> bob`-формат уже правильно движется к JMT, но сам `TxPackage` ещё недостаточен.
- Для transfer у тебя уже есть правильное направление:
  compact `inputs` + storage-backed prep/proof sidecar.
- Чтобы считать задачу завершённой, надо довести ownership до конца:
  симуляция не должна эмулировать storage execution через временные структуры там, где storage уже может стать каноническим владельцем этого шага.

Если хочешь, следующим сообщением я могу разложить это в очень короткую матрицу:
`claim / transfer × inputs / outputs × ready / not ready / what exactly missing`.

---



Compacted conversationНиже сводная таблица по готовности `scenario_1` к встраиванию assets в JMT.

| Поток                                                 | Inputs                                                       | Outputs                                                      | Готовность к JMT                  | Что уже подтверждено в коде                                  | Что еще нужно со стороны симуляции                           | Что еще нужно со стороны z00z_storage                        |
| ----------------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ | --------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| `claim`                                               | Не JMT-spend inputs, а claim-source поля из wire-пакета      | Несут `asset_wire`, из которого уже можно собрать canonical leaf | Да, практически готово уже сейчас | stage_3.rs собирает `ClaimTxPackage`; claim_tx.rs уже держит `ClaimOutputWire.asset_wire`; claim_pkg_consumer.rs уже публикует в storage через canonical `AssetPath` | По сути ничего обязательного для самого формата. Если и доделывать, то только унификацию пайплайна с regular tx | Ничего критичного для claim publish. Базовые операции уже есть: canonical path, put/apply, proof API в README.md |
| `alice -> bob` regular transfer                       | Сейчас inputs компактные: reference-only, в основном `asset_id_hex + serial_id` | Outputs уже несут `asset_wire` и готовы стать новыми leaves  | Частично готово                   | tx_verifier.rs задает компактный input/output wire; stage_4.rs уже вычисляет canonical input `asset_id` и строит prep sidecar; stage_6.rs уже валидирует `member_wit_hex` через storage `chk_blob(...)` | Убрать зависимость от `checkpoint_prep.json` как внешнего обязательного sidecar для исполнения; перенести resolve/apply ближе к storage-backed исполнению; заменить simulator-owned `SimState` на storage-native adapter/flow | Дать storage-native API для regular spend: resolve input leaf по canonical key, принять/проверить membership witness, подготовить state update и применить batch/checkpoint без simulator compatibility bridge |
| `alice -> bob` regular transfer как self-contained tx | Недостаточно данных внутри одного `TxPackage` для полного consumption path | Outputs достаточны                                           | Нет, пока не self-contained       | Проверка правильного направления уже есть: Stage 4 строит proof/prep, Stage 6 проверяет и применяет | Либо встраивать prep/witness внутрь transport-формата, либо определить четкий двухфазный контракт: `TxPackage` + canonical storage prep | Нужен явный публичный контракт на `resolve -> verify witness -> prepare_tx_sum -> apply_batch_checkpoint`, чтобы этим владел storage, а не сценарий |

Короткий вывод:

| Тип tx                       | Вердикт                                                      |
| ---------------------------- | ------------------------------------------------------------ |
| `claim`                      | для claim уже реализованы storage-ready wire и отдельный canonical publish consume |
| `alice -> bob` outputs       | Уже готовы                                                   |
| `alice -> bob` inputs        | Готовы только как compact references, но еще не как полностью self-contained storage-native spend |
| `alice -> bob` весь pipeline | Почти готов, но пока держится на simulator prep/proof scaffolding |

Если свести это к одному предложению: `claim` уже можно считать storage/JMT-ready, а обычный transfer уже правильно кодирует новый asset и canonical input reference, но еще не доведен до состояния, где `z00z_storage` сам полностью владеет исполнением без помощи `scenario_1`.

Если хочешь, следующим сообщением я могу сразу сделать вторую таблицу в формате:

1. `что менять в simulator`
2. `что менять в z00z_storage`
3. `в каком порядке это лучше внедрять`

---

Проверю предыдущую таблицу как набор проверяемых утверждений по коду репозитория и верну полный отчет: что подтверждается, что требует уточнения, и где именно это видно.

**Отчет проверки**

Проверил мой предыдущий ответ как набор утверждений о коде репозитория, а не как внешние факты. Основной итог: базовая логика ответа была верной, но одно место у меня было сформулировано слишком сильно.

**Ключевая поправка**

Утверждение в духе “claim уже опубликованы в storage” надо сужать до более точного:

- в коде есть готовый consumer, который умеет верифицировать claim packages и публиковать их в storage;
- но из просмотренных участков не следует, что именно Stage 3 сам автоматически делает publish в storage как обязательную часть своего пайплайна.

Это не ломает общий вывод “claim-format storage-ready”, но меняет формулировку.

---

**Layer 1: Self-Audit**

Проверяемые утверждения из моего прошлого ответа:

- C1: `ClaimOutputWire` уже несет `asset_wire`, пригодный для JMT/storage publish.
- C2: claim-inputs не являются обычными JMT spend-inputs, а несут claim-source поля.
- C3: есть код, который берет verified claim outputs и публикует их в storage по canonical `AssetPath`.
- C4: regular transfer inputs сейчас compact reference-only: `asset_id_hex + serial_id`.
- C5: Stage 4 вычисляет canonical terminal input `asset_id`.
- C6: Stage 4 строит и сохраняет sidecar `checkpoint_prep.json`.
- C7: Stage 4 получает witness из storage через `proof_blob(...)` и rebinding-ом подгоняет его под свой contract.
- C8: Stage 6 валидирует `member_wit_hex` через `chk_blob(...)`.
- C9: Stage 6 вызывает `prepare_tx_sum(...)` и затем `apply_batch_checkpoint(...)`.
- C10: storage уже экспортирует нужные базовые proof/apply primitives для canonical downstream path.
- C11: claim-path уже “полностью опубликован в storage” в текущем runtime flow.
- C12: regular transfer whole pipeline “почти готов”, но пока не self-contained и зависит от simulator prep/proof scaffolding.

Проверка внутренней согласованности:

- Противоречий между C1-C10 не найдено.
- C11 в исходной формулировке слишком сильное и требует уточнения.
- C12 является архитектурной оценкой, а не прямым фактом из одной строки кода.

---

**Layer 2: Source Verification**

| ID   | Утверждение                                                  | Статус    | Что подтвердилось                                            |
| ---- | ------------------------------------------------------------ | --------- | ------------------------------------------------------------ |
| C1   | `ClaimOutputWire` несет `asset_wire`                         | VERIFIED  | В claim_tx.rs поле `asset_wire` присутствует в `ClaimOutputWire` и описано как carrier for JMT publish and wallet import |
| C2   | claim-inputs не JMT spend-inputs                             | VERIFIED  | В claim_tx.rs `ClaimInputWire` состоит из `claim_id_hex`, `claim_source_asset_id_hex`, `claim_source_commitment_hex` |
| C3   | есть canonical claim publish path в storage                  | VERIFIED  | В claim_pkg_consumer.rs строится `AssetPath::new(DefinitionId, SerialId, AssetId)`, а в claim_pkg_consumer.rs `publish_claims_store(...)` применяет `store.apply_ops(...)` |
| C4   | regular transfer inputs = compact refs                       | VERIFIED  | В tx_verifier.rs `TxInputWire` содержит только `asset_id_hex` и `serial_id`; комментарии ниже в tx_verifier.rs прямо говорят, что consumed leaf не инлайнится |
| C5   | Stage 4 вычисляет canonical terminal input `asset_id`        | VERIFIED  | В stage_4.rs вызывается `canonical_input_asset_id(...)`, а сама функция определена в stage_4.rs и хеширует resolved secret в asset id |
| C6   | Stage 4 пишет `checkpoint_prep.json`                         | VERIFIED  | Константа задана в stage_4.rs, путь собирается в stage_4.rs, сохранение происходит в stage_4.rs |
| C7   | Stage 4 берет proof из storage и делает rebind               | VERIFIED  | В stage_4.rs вызывается `store.proof_blob(&path)`, затем в stage_4.rs создается новый `ProofItem`, и в stage_4.rs делается `blob.rebind(item)` |
| C8   | Stage 6 валидирует `member_wit_hex` через `chk_blob(...)`    | VERIFIED  | В stage_6.rs происходит decode `member_wit_hex`, и в stage_6.rs вызывается `chk_blob(...)` |
| C9   | Stage 6 вызывает `prepare_tx_sum` и `apply_batch_checkpoint` | VERIFIED  | Это видно в stage_6.rs и stage_6.rs                          |
| C10  | storage уже имеет proof/apply primitives                     | VERIFIED  | Контракт перечислен в README.md, README.md, README.md, README.md, README.md; реализация `chk_blob(...)` в proof.rs |
| C11  | claim уже опубликован в storage в текущем flow               | DISPUTED  | Подтверждено только наличие consumer-а publish path в claim_pkg_consumer.rs. В Stage 3 и Stage 4 из просмотренных участков видно использование `load_claim_pkgs(...)`, но не обязательный вызов `publish_claims_store(...)`: stage_3.rs, stage_4.rs |
| C12  | regular transfer pipeline почти готов, но не self-contained  | PLAUSIBLE | Это следует из совокупности фактов: input wire reference-only, prep sidecar обязателен, storage witness rebinding делается в simulator, Stage 6 использует `SimState`. Но это уже архитектурная интерпретация, а не один буквальный контракт |

---

**Layer 3: Adversarial Review**

Проверка на типичные hallucination patterns:

- Fabricated citation: не обнаружено. Все ключевые утверждения привязаны к реальным файлам и местам в коде.
- Precise numbers without source: не применимо.
- Plausible-but-wrong association: обнаружен один риск.
  - Я связал “claim already published into storage” с текущим flow слишком уверенно.
  - Код подтверждает готовый publish consumer, но не подтверждает, что Stage 3 сам завершает эту публикацию как часть обязательного runtime path.
- Temporal confusion: не обнаружено.
- Overgeneralization: есть умеренный риск в C12.
  - Формулировка “почти готов” полезна как архитектурная оценка, но это не бинарный факт из API.
- Missing qualifiers: это главный найденный дефект прошлого ответа.
  - Для claim надо говорить “format и consumer storage-ready”, а не “уже опубликовано” без оговорки.
  - Для regular transfer надо говорить “outputs и proof direction готовы, execution ownership еще split between simulator and storage”.

---

**Итоговая переоценка предыдущего ответа**

**Что было корректно**

- `claim` output format действительно storage-ready.
- Есть canonical claim consumer, который умеет публиковать claim outputs в storage по правильному `AssetPath`.
- regular tx inputs действительно compact reference-only.
- Stage 4 действительно строит `checkpoint_prep.json`.
- Stage 4 действительно берет storage proof blob и делает `rebind(...)`.
- Stage 6 действительно прогоняет `chk_blob(...)`, затем `prepare_tx_sum(...)`, затем `apply_batch_checkpoint(...)`.
- storage действительно уже владеет canonical proof primitives и batched apply primitives.

**Что нужно исправить в моей прошлой формулировке**

Вместо:

- “claim уже опубликованы в storage”

Надо:

- “claim-format и claim consumer уже готовы для canonical publish в storage; из просмотренных мест не видно, что Stage 3 сам завершает publish как обязательную часть текущего scenario_1 flow”

**Исправленный краткий вывод**

- `claim`: VERIFIED как storage-ready format plus ready publish path.
- `alice -> bob` outputs: VERIFIED как portable outputs.
- `alice -> bob` full tx as self-contained storage-native spend: не подтверждено; PLAUSIBLE, что еще нужен simulator/storage handoff refactor.

---

**Финальный вердикт**

Мой прошлый ответ был в целом точным по архитектурному направлению, но содержал 1 завышенную формулировку:

- `claim` не стоит описывать как “уже опубликован в storage” без оговорки.
- Правильнее: “для claim уже реализованы storage-ready wire и отдельный canonical publish consumer”.

Все остальные ключевые технические опоры ответа подтверждаются кодом.

**Limitations of this verification:**

- Эта проверка ускоряет человеческую верификацию, но не заменяет ее.
- Здесь проверялись утверждения по локальному коду репозитория, а не по внешним источникам.
- Adversarial review делает та же модель, поэтому он хорошо ловит переобобщения и натяжки, но не гарантирует абсолютную полноту.
- Утверждение со статусом VERIFIED означает, что для него найдено прямое подтверждение в коде или документации репозитория, а не то, что архитектурно вопрос закрыт навсегда.
- Утверждения со статусом PLAUSIBLE все еще требуют инженерного суждения, особенно когда речь идет о “готовности” пайплайна, а не о наличии конкретной функции.

Если хочешь, я могу теперь переписать ту таблицу в уже исправленном, более строгом варианте без этой завышенной формулировки.

---



- В Stage 6 уже есть реальная canonical proof validation, потому что `member_wit_hex` декодируется и прогоняется через [chk_blob(...)](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html), а затем flow идет в [prepare_tx_sum(...)](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) и [apply_batch_checkpoint(...)](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html).

- В Stage 4 storage witness уже используется, потому что код берет [proof_blob(&path)](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html), но затем делает [rebind(...)](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) на новый [ProofItem](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html), собранный из локального [prev_root](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html); это подтверждает наличие compatibility bridge.  accepted Stage 4 path ещё не полностью “чистый storage-owned” без compatibility bridge

- В [state_update.rs](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) wallet boundary пока не делает [chk_blob(...)](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html): [resolve_inputs(...)](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) принимает membership witness как opaque bytes и требует только, чтобы [wit.proof](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) не был пустым. Поэтому canonical inclusion verification на этом boundary еще не встроена прямо в wallet state-update path.

- В wallet prepare_tx_sum(...) boundary:
  нет, ещё не доведено до конца.
  Сейчас wallet-side resolve_inputs(...) всё ещё проверяет только то, что witness bytes не пустые:

  state_update.rs:257
  Там пока нет chk_blob(...) на первой downstream boundary.
  То есть wallet ещё не использует полную canonical inclusion verification так, как storage уже умеет.
