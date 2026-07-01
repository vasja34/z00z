Сейчас доберу только те куски кода и артефактов, которые нужны для 7 пунктов: полный path Alice→Bob→Charlie, кошелёк до/после, JMT/tx proof включение, claim/regular asset path в JMT, и статусы/scanning по wallet. Потом дам ответ строго по коду и `outputs`, без внешних источников.

**Doublecheck Report**

**Heads up:** в текущем `scenario_1` нет доказательства полного wallet/runtime path `Alice -> Bob -> Charlie` для одного и того же asset flow. По коду и по сгенерированным `outputs` подтверждается полный wallet path только для `Alice -> Bob`; ==ветка `Charlie` в текущем прогоне доведена до checkpoint/storage artifacts, но не до изменения Charlie wallet state.==

**Scope**

- Источник истины в этом отчёте: только код и локальные артефакты из scenario_design.yaml, stage_3.rs, stage_4.rs, stage_5.rs, stage_6.rs, stage_7.rs, stage_8.rs, storage_view.rs и из outputs.
- Нужные артефакты уже были сгенерированы и обновлены в этом сеансе; дополнительная генерация не потребовалась.

**Summary**

| Вопрос                                                       | Вердикт                                                      |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| 1. Полный simulation path `Alice -> Bob -> Charlie` asset    | `PARTIAL / НЕТ полного wallet path до Charlie`               |
| 2. Проверка сумм в кошельках до/после tx у всех участников   | `PARTIAL / Достаточно артефактов для проверки, но нет одного жёсткого global invariant gate по всем wallet sums` |
| 3. Валидация tx через включение в JMT, wallet proof verification, JMT inclusion proofs | `PARTIAL / есть snapshot-membership и JMT artifact export, но нет wallet-side JMT inclusion proof verification` |
| 4. Путь включения всех assets из claim/regular в JMT         | `PARTIAL / есть отдельные JMT views для claim и regular, но не один canonical merged chain-anchor path` |
| 5. Переходы `pending -> confirmed` и JMT scan by wallet via stealth/tag16 | `PARTIAL / pending->confirmed есть, JMT wallet scan нет`     |

**1. Существует ли полный simulation path transactions Alice -> Bob -> Charlie asset**

- **[VERIFIED]** Полный wallet/runtime path для `Alice -> Bob` существует.
  Evidence:
  - Stage 5 делает явный receiver-side путь через `recv_route(..., PersistClaim)` в stage_5.rs.
  - Артефакты pending/confirmed есть в wallets_pending.json и wallets_confirmed.json.
  - Wallet diff и report подтверждают 4 Bob receives: wallets_state_diff.json, wallets_state_report.md.

- **[VERIFIED]** Ветка Stage 6/7/8 не обновляет Charlie wallet runtime.
  Evidence:
  - В Stage 6 прямо зафиксирован `wallet_skip` в stage_6.rs.
  - Charlie не меняется в wallet totals before/after в wallets_state_report.md: `items_before = 26`, `items_after = 26`, `wlt_size_before = wlt_size_after`.
  - leaf_alice_to_charlie_frag1.json и leaf_alice_to_charlie_frag2.json существуют, но это checkpoint fragment artifacts, а не Charlie wallet receive artifacts:
    leaf_alice_to_charlie_frag1.json
    leaf_alice_to_charlie_frag2.json

- **[VERIFIED]** Эти fragment artifacts строятся из Stage 4 input/output pairs, а не из отдельного Charlie receive flow.
  Evidence:
  - Stage 6 пишет `target fragments saved from selected stage_4 input/output pairs` в stage_6.rs.
  - `load_stage6()` строит `frag_a` из `pkg.tx.inputs[0]` и `pkg.tx.outputs[0]`, `frag_b` из `pkg.tx.inputs[1]` и `pkg.tx.outputs[1]` в stage_6.rs.

**Вывод по п.1:** полного path `Alice -> Bob -> Charlie` в смысле `wallet receive -> wallet state change -> confirmed state` нет. Есть `Alice -> Bob` wallet path и отдельный checkpoint/storage fragment path, который не доводится до Charlie wallet.



**3. Есть ли валидация transactions через включение в JMT, проверяет ли кошелёк tx proofs и JMT inclusion proofs**

- **[VERIFIED]** Есть валидация pre-state membership/witness на snapshot уровне.
  Evidence:
  - Stage 4 строит canonical snapshot и сверяет witness/root через `ProofBlob::decode(...)` и root match в stage_4.rs.
  - Stage 6 uses membership material from snapshot replay and имеет проверки `membership root mismatch` / `membership witness failed` в stage_6.rs.

- **[VERIFIED]** Есть JMT artifact generation/export.
  Evidence:
  - `build_artifact(store)` и `JmtFsStore::new(view_root)` используются в storage_view.rs.
  - Реальные JMT artifacts существуют:
    claim_post
    pre_tx
    post_tx

- **[VERIFIED]** В текущем outputs нет финального sealed checkpoint artifact path.
  Evidence:
  - Stage 8 summary в checkpoint_s8.json имеет `status = draft_only`.
  - Это соответствует коду в stage_8.rs.

- **[PARTIAL]** Tx proof verification в Stage 6/7 есть, но она слабая/opaque.
  Evidence:
  - `PassProof::verify_tx()` в stage_6.rs проверяет только что `tx_proof` не пустой.
  - Stage 8 умеет строить opaque checkpoint proof через `build_cp_proof(...)`, но в текущем прогоне до финализации не дошло.

- **[NOT SUPPORTED]** Я не вижу wallet-side verification JMT inclusion proofs в текущем scenario path.
  Evidence:
  - Нет отдельных inclusion proof artifacts в outputs/storage/claim_post, outputs/storage/pre_tx, outputs/storage/post_tx.
  - Wallet-side scanning в scenario_1 идёт по leaf/asset, а не по JMT inclusion proof.

**Вывод по п.3:** есть snapshot membership validation и JMT artifact export, но нет полноценного доказанного wallet-side JMT inclusion proof verification path в текущем outputs.



**4. Есть ли путь включения всех assets из всех типов transactions claim/regular в JMT**

- **[VERIFIED]** Для `claim` есть JMT artifact, но он observational, не canonical consensus anchor.
  Evidence:
  - Stage 3 помечен как `wallet_only_intermediate` в stage_3.rs.
  - Реальный claim JMT artifact есть в outputs/storage/claim_post, summary в claim_post/summary.json подтверждает `root_match = true`.

- **[VERIFIED]** Для `regular tx` есть pre-state и post-state JMT artifacts.
  Evidence:
  - outputs/storage/pre_tx
  - outputs/storage/post_tx
  - `post_tx/summary.json` подтверждает `matches_new_root = true`.

- **[PARTIAL]** Я не вижу одного единого canonical JMT chain path, куда в текущем прогоне последовательно заякорены и claim, и regular tx как единый ledger history artifact.
  Evidence:
  - Есть три раздельных views: `claim_post`, `pre_tx`, `post_tx`.
  - Stage 3 design прямо говорит, что claim “does NOT anchor claim in JMT/GenesisCheckpoint yet”.
  - Значит claim JMT здесь есть как exportable inspection artifact, но не как canonical chain inclusion stage.

**Вывод по п.4:** путь есть только в виде отдельных JMT views для claim и regular. Единого canonical merged inclusion path для всех transaction types в текущем scenario outputs не доказано.

**5. Меняется ли status в кошельках с pending на approved/confirmed после включения в JMT; есть ли scanning JMT by wallet на основе stealth address и tag16**

- **[VERIFIED]** `pending -> confirmed` transition в artifacts есть.
  Evidence:
  - wallets_pending.json
  - wallets_confirmed.json
  - wallets_state_report.md содержит обе таблицы.
  - Alice: `pending_spend -> confirmed_spend`
  - Bob: `pending_receive -> confirmed_receive`
  - Sequencer: `pending_fee -> confirmed_fee`

- **[PARTIAL]** Эти transitions присутствуют как lifecycle overlays сценария, но не доказано, что именно JMT scan кошелька их переводит.
  Evidence:
  - Код Stage 4/5 формирует pending/confirmed rows как часть simulator artifact pipeline.
  - Отдельного wallet scan over JMT после Stage 7 я не вижу.

- **[VERIFIED]** Scanning по stealth fields и `tag16` есть, но он идёт по leaf/asset, не по JMT.
  Evidence:
  - Stage 5 вызывает `receiver_scan_leaf`, `receiver_scan_report`, `StealthOutputScanner::scan_leaf(...)` в stage_5.rs.
  - Stage 4 и Stage 3 строят/проверяют `tag16` bindings.
  - Это подтверждает wallet-level discovery по stealth/tag16 на уровне output leaf, но не по JMT traversal.

- **[NOT SUPPORTED]** JMT-by-wallet scanning для поиска своих assets по stealth address и `tag16` в текущих outputs не найден.
  Evidence:
  - Нет отдельного output artifact или code path в scenario_1, который бы читал JMT artifact, обходил дерево и искал wallet-owned leaves по stealth material.
  - Charlie wallet остаётся неизменным, что дополнительно показывает отсутствие такого path в текущем run.

**Вывод по п.5:** `pending -> confirmed` есть, но это artifact-level transition сценария. Wallet JMT scan по stealth/tag16 в текущем scenario outputs не доказан.



**Прямые ответы на ваши 7 пунктов**

1. Полный `Alice -> Bob -> Charlie` simulation path: **нет**. Есть полный `Alice -> Bob`; Charlie ветка остаётся на checkpoint/storage уровне.
2. Проверка сумм до/после у всех участников: **частично да**. Before/after/diff/report есть, tx economics подтверждены, но нет одного явного global wallet-sum gate.
3. Валидация через включение в JMT и wallet proof verification: **частично да**. Есть snapshot membership и JMT artifact export; wallet-side JMT inclusion proof verification не найден.
4. Путь включения assets `claim/regular` в JMT: **частично да**. Есть отдельные JMT artifacts для `claim_post`, `pre_tx`, `post_tx`; нет одного canonical merged chain anchor.
5. Переход `pending -> confirmed`: **да**, в artifacts. JMT wallet scanning по stealth/tag16: **нет**, не найдено.
6. Артефактов для анализа хватило: **да**. Они были уже сгенерированы/обновлены в этом сеансе.
7. Ответ основан только на коде и локальных outputs: **да**.

**Локальные источники**

- Код:
  scenario_design.yaml
  stage_3.rs
  stage_4.rs
  stage_5.rs
  stage_6.rs
  stage_7.rs
  stage_8.rs
  storage_view.rs

- Артефакты:
  stage_3_snapshot.json
  wallets_state_before.json
  wallets_state_after.json
  wallets_pending.json
  wallets_confirmed.json
  wallets_state_diff.json
  wallets_state_report.md
  leaf_alice_to_bob.json
  leaf_alice_to_charlie_frag1.json
  leaf_alice_to_charlie_frag2.json
  checkpoint_bridge_s6.json
  checkpoint_s7.json
  checkpoint_s8.json
  claim_post/summary.json
  pre_tx/summary.json
  post_tx/summary.json

**Ограничения этой проверки**

- Отчёт основан только на локальном коде и уже сгенерированных outputs.
- ==Отсутствие артефакта я трактую как “не доказано в текущем scenario outputs”, а не как “невозможно в проекте вообще”.==
- Stage 8 в текущем run находится в `draft_only`, поэтому final checkpoint artifact/link/audit path здесь не доказан на артефактах этого прогона.

1. ==Если хотите, следующим сообщением я могу сделать ещё более жёсткую матрицу `claim -> pre_tx -> stage5 receive -> stage7 apply -> stage8 finalize` по каждому asset id, который затронут в текущем run.==
2. ==Если хотите доказать именно JMT wallet scan и Charlie full path на артефактах, нужно расширять сценарий или включать другой режим прогона; я могу сразу указать минимальные кодовые дыры для этого.==
3. 
