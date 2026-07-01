Сначала соберу точный список сигнатур с шаблоном `_for(` по репозиторию и быстро подниму локальный контекст проекта, чтобы таблица была не просто списком, а с внятными и консистентными вариантами rename.

Ниже я взял только объявления Rust-функций с паттерном _for(, без call-sites. Я бы держал такую семантику префиксов: build для конструирования значения, create для лёгкого хэндла, lookup для поиска, derive для детерминированного вывода, select для диспетчеризации по режиму, require для пути с обязательным существованием.

| #    | Path                         | Old-Name                    | Suggested-Name                   | Comments                                                     |
| ---- | ---------------------------- | --------------------------- | -------------------------------- | ------------------------------------------------------------ |
| 1    | output_build.rs              | build_output_ctx_for        | build_output_ctx_with_serial     | Вариант отличается не “for”, а явным serial_id; with_serial читается точнее. |
| 2    | test_model.rs                | root_for                    | build_root_from_path             | Хелпер реально строит root из одного path+leaf, а не “root for something”. |
| 3    | tx_plan_engine.rs            | asset_batches_for           | select_asset_batches             | Эта функция не просто build, а выбирает serial/parallel стратегию по mode. |
| 4    | tx_plan_engine.rs            | serial_batches_for          | select_serial_batches            | Та же логика: это dispatch по PlanMode, а не “for”.          |
| 5    | post_claim.rs                | wlt_path_for                | require_wallet_file_path         | Функция не просто считает path, а падает если файла нет; require это явно отражает. |
| 6    | store_rows.rs                | claim_rows_for              | build_claim_rows                 | Функция конструирует persisted rows из claims и валидирует replay/duplicate. |
| 7    | genesis_verification.rs      | expected_hash_for           | expected_genesis_hash            | Доменный смысл здесь важнее generic hash; имя становится самодокументируемым. |
| 8    | support.rs                   | claim_leaf_for              | decrypt_claim_leaf               | Реальная операция здесь decrypt wire -> leaf, это лучше вынести в глагол. |
| 9    | test_e2e_phase4.rs           | claim_leaf_for              | decrypt_claim_leaf               | Дубликат той же семантики в тесте; лучше держать одинаковое имя. |
| 10   | nullifier_store_global.rs    | lease_for                   | create_nullifier_lease           | Возвращается хэндл-объект; create_nullifier_lease звучит явно и консистентно. |
| 11   | test_stealth_scan_support.rs | make_leaf_for               | build_leaf_with_keys             | В тестовом helper API лучше уйти от make к build и явно назвать зависимость on keys. |
| 12   | builder.rs                   | sender_create_output_for    | build_sender_output_leaf         | Сейчас имя перегружено и смешивает create/output/for; функция реально строит leaf. |
| 13   | output.rs                    | build_tx_stealth_output_for | build_stealth_output_with_serial | Здесь снова отличие именно в explicit serial lane; with_serial намного яснее. |
| 14   | support.rs                   | label_for                   | lookup_address_label             | Это lookup по address, а не generic “label for”; читается быстрее в RPC helper-коде. |
| 15   | asset_impl_tests.rs          | seed_for                    | derive_wallet_mark_seed          | Семантика детерминированного derivation из wallet_id и mark становится явной. |

Если хочешь, следующим сообщением я могу сразу сделать вторую, уже рабочую таблицу:
1. Old-Name → Suggested-Name + оценка риска rename.
2. Old-Name → Suggested-Name + список мест, где каждый символ используется, чтобы потом переименовывать без пропусков.