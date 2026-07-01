

assets_config.yaml
vauchers_config.yaml
rights_config.yaml
policies_config.yaml
actions_config.yaml

z00z_core/tests/rights
z00z_core/tests/vauchers
dlja chego nuzhni z00z_core/tests/vectors ?

z00z_core/tests/generated_kani_asset_pkg_json.rs --> assets/
z00z_core/tests/test_assets_suite.rs
z00z_core/tests/test_genesis_suite.rs

z00z_core - vse sub folders structures
prover structuru eto crate; mozhet nado sozdat dopolnitelnie moduli. 
mozhet to chto v assets/ i genesis/ nuzhno kak to podrugomu organizovat;





Нет: `crates/z00z_core/src/assets` сейчас не является строго asset-only.

Моя сильная рекомендация: не делать новые подкаталоги вида `assets/rights`, `assets/policies`, `assets/actions`, `assets/vauchers`. Это создало бы второй слой владения. Правильнее завершить уже начатое разделение: owner-модули уже живут на root-уровне в [lib.rs (line 103)](/home/vadim/Projects/z00z/crates/z00z_core/src/lib.rs:103), а из `assets` нужно вынести только остаточные shared cross-object примитивы в один root-owned модуль, например `src/objects/`, плюс shared error на root-уровень. Канонический bootstrap authority должен оставаться один: `genesis`.

**Почему это так**

- [assets/mod.rs (line 14)](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/mod.rs:14) прямо говорит, что canonical bootstrap authority для assets/rights/policies/vouchers уже находится в `z00z_core::genesis`, а `assets_config.yaml` является только secondary compatibility surface.
- При этом общие типы все еще лежат в `assets`: [object_family.rs (line 3)](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/object_family.rs:3) содержит `ObjectFamily` и `ObjectRoleV1`, хотя они описывают не только asset, но и `voucher`/`right`.
- Наличие shared dependency видно по импорту из owner-модулей: [action_pool.rs (line 7)](/home/vadim/Projects/z00z/crates/z00z_core/src/actions/action_pool.rs:7), [policy_descriptor.rs (line 7)](/home/vadim/Projects/z00z/crates/z00z_core/src/policies/policy_descriptor.rs:7), [right_policy.rs (line 5)](/home/vadim/Projects/z00z/crates/z00z_core/src/rights/right_policy.rs:5), [voucher_bootstrap.rs (line 3)](/home/vadim/Projects/z00z/crates/z00z_core/src/vauchers/voucher_bootstrap.rs:3) тянут типы из `assets`.
- Shared error тоже физически привязан к `assets`: [asset_error.rs (line 6)](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/asset_error.rs:6). По смыслу это уже не чисто asset error, раз им валидируются policies/rights/vouchers/actions.
- Даже тестовая привязка еще смешана: [policies/mod.rs (line 17)](/home/vadim/Projects/z00z/crates/z00z_core/src/policies/mod.rs:17) и [vauchers/mod.rs (line 15)](/home/vadim/Projects/z00z/crates/z00z_core/src/vauchers/mod.rs:15) подключают тесты из `../assets/...`.
- `assets_config.yaml` и его schema уже смешанные: [assets_config.yaml (line 122)](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/assets_config.yaml:122) содержит `rights`, а [assets_config_schema.yaml (line 136)](/home/vadim/Projects/z00z/crates/z00z_core/src/assets/assets_config_schema.yaml:136) их же валидирует. Это явный boundary smell.

**Что должно остаться в `assets`**

- AssetDefinition, Asset, registry, commitment/range-proof/wire, serial/nonce, asset policy flags, asset-specific parsing.
- `gas.rs` я бы пока оставил в `assets`, если fee semantics по-прежнему моделируются как свойство native asset definition, а не как отдельный object family.

**Что надо вынести из `assets`**

- `ObjectFamily`, `ObjectRoleV1` и прочую shared object vocabulary в root-owned модуль вроде `src/objects/`.
- Shared validation/config/serialization error из asset-нейминга в root-level error module. Иначе имя `AssetError` продолжает врать о реальной области ответственности.
- Misplaced owner tests из `assets` обратно в `policies/`, `vauchers/`, `rights/`, `actions/`.
- Mixed compatibility fixture `assets_config.yaml` либо сделать строго asset-only, либо переименовать/переместить как compatibility fixture, но не оставлять его в виде “asset file с rights внутри”.

**Про YAML-конфиги**

- Отсутствие отдельных `rights_config.yaml` / `policies.yaml` / `vouchers.yaml` сегодня оправдано только потому, что live authority уже централизована в `GenesisConfig`: [genesis_config.rs (line 47)](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config.rs:47).
- `genesis_config_devnet_small.yaml` уже покрывает текущий live bootstrap: `assets` [3 (line 3)](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml:3), `rights` [57 (line 57)](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml:57), `policies` [168 (line 168)](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml:168), `vouchers` [334 (line 334)](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml:334), `outputs` [409 (line 409)](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml:409), `performance` [414 (line 414)](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml:414).
- Отдельный `actions.yaml` я бы не вводил. Текущая модель уже связывает `action_pool + template` в один `PolicyConfigEntryV1` в [policy_template.rs (line 64)](/home/vadim/Projects/z00z/crates/z00z_core/src/policies/policy_template.rs:64), и в YAML actions уже естественно живут внутри policy entry [genesis_config_devnet_small.yaml (line 168)](/home/vadim/Projects/z00z/crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml:168). Вынести actions в отдельный файл сейчас значит добавить лишние foreign-key связи и новый drift-risk.

**Один большой файл или набор sub-files**

- Для `devnet_small`, тестовых fixture и локальной отладки одного файла достаточно. Это удобно и прозрачно.
- Для production я бы выбрал не “много равноправных YAML”, а `один canonical root manifest + referenced sub-files`.
- То есть authority остается одна, но физически данные можно разнести:
  - `genesis.yaml` содержит `chain`, `outputs`, `performance` и ссылки на `assets.yaml`, `rights.yaml`, `policies.yaml`, `vouchers.yaml`.
  - Loader собирает это в тот же самый `GenesisConfig`, без второй логики и без параллельного object model.

**Моя production-рекомендация**

- `genesis` остается единственным authority.
- `genesis_config_devnet_small.yaml` можно оставить как self-contained fixture.
- Для production перейти на `root manifest + sub-files`.
- `actions` не выносить в отдельный YAML, пока action pools не станут реально independently versioned first-class entities.

**Насколько это усложнит кодовую базу**

- Boundary cleanup модулей: низкая/средняя сложность. Это в основном relocation imports + выправление ownership.
- Переезд shared error: средняя сложность, потому что затронет сигнатуры и error taxonomy.
- `root manifest + refs`: средняя сложность. Нужны path resolution, deterministic merge и единый validation pass.
- Это не смена парадигмы и не parallel layer, если сохранить один canonical `GenesisConfig` и не дублировать схемы/типы.

**Порядок без concept drift**

1. Зафиксировать правило: live bootstrap authority только `genesis`; `assets_config.yaml` не authority.
2. Вынести shared object vocabulary из `assets`.
3. Развязать shared error от asset-only naming.
4. Убрать owner tests и mixed fixtures из `assets`.
5. Только после этого, если production config реально разрастается, добавлять `root manifest + refs`, но без изменения доменной модели.

Код не менял. Если хочешь, следующим сообщением могу дать уже совсем прикладной target-layout: точный список файлов, что оставить в `assets`, что перенести в `objects`, и какой YAML split сделать для production без drift.