# Динамическое расширение HJMT-структуры

Коротко: да, в `storage` динамическое расширение HJMT-структуры заложено. Но новый asset не всегда означает новое дерево. Новые внутренние HJMT subtree появляются только если впервые возникает новый scope: новый `definition_id`, новый `serial_id` или новый `bucket`. Если запись попадает в уже существующий `(definition, serial, bucket)`, создаётся не новое дерево, а новый leaf в существующем terminal-tree с каскадным пересчётом root-ов вверх.

По коду путь такой:

- Публичная запись идёт через [query.rs](../../../crates/z00z_storage/src/backend/query.rs) или батчем через [store.rs](../../../crates/z00z_storage/src/settlement/store.rs).
- `StoreItem` сначала проверяет согласованность path/leaf; для terminal leaf `serial_id` внутри leaf обязан совпасть с `path.serial_id` в [record.rs](../../../crates/z00z_storage/src/settlement/record.rs) и [record.rs](../../../crates/z00z_storage/src/settlement/record.rs).
- Затем `next.model.put_leaf()` динамически создаёт отсутствующие уровни `definition_id -> serial_id` через `entry(...).or_default()` в [model.rs](../../../crates/z00z_storage/src/settlement/model.rs).
- Планировщик выводит `bucket_id` из path и готовит batch для `BucketTerminal(def, serial, bucket)` в [hjmt_plan.rs](../../../crates/z00z_storage/src/settlement/hjmt_plan.rs) и [hjmt_plan.rs](../../../crates/z00z_storage/src/settlement/hjmt_plan.rs).
- Коммит идёт снизу вверх: `BucketTerminal -> Bucket -> Serial -> Definition` в [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs), [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs), [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs), [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs), [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs).
- Child-root реально встраивается в parent-leaf:
  - `BucketRootLeaf.terminal_jmt_root` в [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs)
  - `SerialRootLeaf.serial_root` в [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs)
  - `DefinitionRootLeaf.definition_root` в [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs)
- Недостающие физические деревья создаются динамически в `ensure_live_hjmt_tree_versions`, где прямо перечислены `Definition`, `Serial(def)`, `Bucket(def, serial)`, `BucketTerminal(def, serial, bucket)` в [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs).

Важный нюанс: публичный `settlement_root` не равен просто верхнему физическому HJMT root. Он считается из семантической модели `SettlementModel::root()` в [model.rs](../../../crates/z00z_storage/src/settlement/model.rs), а потом пишется как `sem_root` и `settlement_root` в [hjmt_commit.rs](../../../crates/z00z_storage/src/settlement/hjmt_commit.rs). То есть физический HJMT-лес и публичный settlement root согласованы, но это не простое `settlement_root = def_root`.

Если говорить буквально про “новую экономику / новый класс / новый serial”:

- В storage измерения дерева это `definition_id`, `serial_id`, `terminal_id` в [identity.rs](../../../crates/z00z_storage/src/settlement/identity.rs); сами типы лежат в [identity.rs](../../../crates/z00z_storage/src/settlement/identity.rs) и [identity.rs](../../../crates/z00z_storage/src/settlement/identity.rs).
- В самом storage-пути записи я не нашёл реестра “разрешённых definition”. По факту новый `definition_id` начинает жить первой успешной записью.
- Отдельный `AssetDefinitionRegistry` существует выше, в `z00z_core`, и simulator держит его в контексте в [registry.rs](../../../crates/z00z_core/src/assets/registry.rs) и [context.rs](../../../crates/z00z_simulator/src/context.rs), но сам storage write-path его не вызывает.

По `aggregator` вывод жёсткий: текущий runtime-aggregator это не место, где создаются эти HJMT scopes.

- В crate есть только traits сервиса в [service.rs](../../../crates/z00z_runtime/aggregators/src/service.rs).
- `OrderedBatch.created_leaves` существует, но `BatchPlanner::make_batch()` заполняет его пустым `Vec::new()` в [types.rs](../../../crates/z00z_runtime/aggregators/src/types.rs) и [batch_planner.rs](../../../crates/z00z_runtime/aggregators/src/batch_planner.rs).
- В `crates/z00z_runtime/aggregators` я не нашёл вызовов `put_settlement_item` или `apply_settlement_ops`.

Итог: storage уже умеет живо наращивать и схлопывать HJMT scopes под новые `definition/serial/bucket`; aggregator layer в текущем коде этим не управляет и сам новые settlement-tree scopes не коммитит. Если вопрос в том, заложен ли use case новых asset economies в storage, ответ: да. Если вопрос, доведён ли он до рабочего aggregator orchestration, ответ: по текущему коду нет.
