# Settlement Benchmark Notes

Запустил реальные прогоны.

## Invariant Guardrails

### ZINV-CHECKPOINT-001

Invariant reference: `ZINV: CHECKPOINT-001`

Эти benchmark-цифры имеют смысл только если HJMT и checkpoint lineage остаются
одной непрерывной root-chain и ни один batch не потребляет один и тот же input
дважды.

### ZINV-CHECKPOINT-002

Invariant reference: `ZINV: CHECKPOINT-002`

Ни одна оптимизация throughput не должна менять порядок authority:
publication/DA evidence может появляться только после уже committed checkpoint
state, а не как bypass вокруг commit path.

## Commands

```bash
cargo bench -p z00z_storage --bench settlement_shard --features test-params-fast -- --sample-size 10 --warm-up-time 0.01 --measurement-time 0.02
cargo bench -p z00z_storage --bench settlement_proofs --features test-params-fast -- --sample-size 10 --warm-up-time 0.01 --measurement-time 0.02
```

Без `--features test-params-fast` текущий bench build у вас падает на `MockRngProvider MUST NOT be compiled in production builds`, поэтому числа ниже сняты на этом живом compile path.

## Benchmarks

| Surface     | Lane                                         |       Median |          p95 |          p99 |    Throughput |
| ----------- | -------------------------------------------- | -----------: | -----------: | -----------: | ------------: |
| insert-many | `insert_many_definitions/generalized_assets` |  `79.350 ms` |  `83.934 ms` |  `85.083 ms` | `12.60 ops/s` |
| insert-many | `insert_many_serials/generalized_assets`     |  `65.452 ms` |  `67.909 ms` |  `68.652 ms` | `15.28 ops/s` |
| insert-many | `insert_many_hot_serial/asset_batch`         | `115.590 ms` | `120.256 ms` | `120.430 ms` |  `8.65 ops/s` |
| insert-many | `insert_many_hot_serial/right_batch`         |  `47.767 ms` |  `50.374 ms` |  `50.503 ms` | `20.93 ops/s` |
| insert-many | `insert_hot_bucket/asset_batch`              |  `10.584 ms` |  `11.459 ms` |  `11.476 ms` | `94.48 ops/s` |
| delete-many | `delete_many_definitions/generalized_assets` | `148.623 ms` | `149.743 ms` | `149.927 ms` |  `6.73 ops/s` |
| delete-many | `delete_many_hot_serial/generalized_assets`  | `174.091 ms` | `174.760 ms` | `174.771 ms` |  `5.74 ops/s` |
| proof-many  | `prove_many_assets/shared_parent_batch`      |  `11.700 ms` |  `12.682 ms` |  `12.787 ms` | `85.47 ops/s` |

Дополнительно по proof batch из `settlement_proofs`:

| Surface        | Lane                                                |     Median |        p95 |        p99 |     Throughput |
| -------------- | --------------------------------------------------- | ---------: | ---------: | ---------: | -------------: |
| proof-generate | `proof_generate/shared_parent_batch`                | `4.042 ms` | `4.827 ms` | `5.167 ms` | `247.43 ops/s` |
| proof-generate | `proof_generate/mixed_inclusion_nonexistence_batch` | `4.221 ms` | `4.539 ms` | `4.541 ms` | `236.93 ops/s` |
| proof-verify   | `proof_verify/split`                                | `3.559 ms` | `4.569 ms` | `5.130 ms` | `280.98 ops/s` |
| proof-verify   | `proof_verify/merge`                                | `3.587 ms` | `3.906 ms` | `3.994 ms` | `278.77 ops/s` |
| proof-verify   | `proof_verify/policy_transition`                    | `3.610 ms` | `4.726 ms` | `5.081 ms` | `277.02 ops/s` |

## Proof Size Artifacts

Живой artifact snapshot генерируется во время bench/verifier pass и сохраняется в активный run-root под `reports/z00z-verification-orchestrator-<timestamp>/`.

Ниже сохранённая сводка того же bench pass:

| Artifact                         |      Size |
| -------------------------------- | --------: |
| `inclusion_asset`                |  `1372 B` |
| `inclusion_right`                |  `1433 B` |
| `deletion_right`                 |  `1732 B` |
| `nonexistence_asset`             |   `798 B` |
| `split`                          |   `295 B` |
| `merge`                          |   `365 B` |
| `policy_transition`              |   `231 B` |
| `shared_parent_batch_bytes`      | `11314 B` |
| `mixed_batch_inclusion_bytes`    | `11181 B` |
| `mixed_batch_nonexistence_bytes` |  `1191 B` |
| `proof_bytes_total`              |  `5335 B` |

Ещё из того же artifact:

- `shared_parent_batch_count = 8`
- `mixed_batch_inclusion_count = 8`
- `split_time_us = 21`
- `merge_time_us = 5`
- `policy_transition_time_us = 655`

Отдельно по `settlement_shard` во время реального прогона был сгенерирован recovery note с `reload_time_us = 4759` и `root_equal = true`, но следующий bench очистил `outputs/settlement`, потому что этот root managed и пересоздаётся на новый run.

EXPANDABLE: details available for raw criterion JSON paths and preserved-output strategy across multiple bench runs.

---

## Many

`many` здесь не одно фиксированное число. В текущем harness размеры batch захардкожены по lane в [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs). Формы workload задаются в [settlement_corpus.rs](../../crates/z00z_storage/src/fixture_support/settlement_corpus.rs), [settlement_corpus.rs](../../crates/z00z_storage/src/fixture_support/settlement_corpus.rs), [settlement_corpus.rs](../../crates/z00z_storage/src/fixture_support/settlement_corpus.rs), [settlement_corpus.rs](../../crates/z00z_storage/src/fixture_support/settlement_corpus.rs).

| Lane                                 | Сколько item в batch | Что это значит                     |    Median |     Нормализованно |
| ------------------------------------ | -------------------: | ---------------------------------- | --------: | -----------------: |
| `insert_many_definitions`            |                   96 | 96 разных definition, один serial  |  79.35 ms |       ~1210 item/s |
| `insert_many_serials`                |                   96 | 96 serial внутри одного definition |  65.45 ms |       ~1467 item/s |
| `insert_many_hot_serial/asset_batch` |                  128 | 128 asset в одном hot serial       | 115.59 ms |       ~1107 item/s |
| `insert_many_hot_serial/right_batch` |                   64 | 64 right в одном hot serial        |  47.77 ms |       ~1340 item/s |
| `insert_hot_bucket`                  |                    8 | 8 item в одном adaptive bucket     |  10.58 ms |        ~756 item/s |
| `delete_many_definitions`            |                   96 | wide delete по definitions         | 148.62 ms |        ~646 item/s |
| `delete_many_hot_serial`             |                  128 | hot delete в одном serial          | 174.09 ms |        ~735 item/s |
| `prove_many_assets`                  |                   64 | proof batch на 64 path             |  11.70 ms | ~5470 proof-path/s |

Ключевой момент: сравнивать только `ops/s` между этими lane неверно, потому что batch sizes разные. Для production смотри на `item/s`, а не только на `ops/s`.

## Sharding

Кодовая база подтверждает только более узкий тезис: более широкое распределение по независимым definition/serial иногда помогает, но “sharding сам по себе резко поднимет throughput” из текущего кода не следует.

- `settlement_shard` does not measure an external multi-shard deployment. These lanes run through one `SettlementStore::new()` and one `apply_settlement_ops(...)` in [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs).
- `SettlementStore::new()` now uses a managed local RedB root for tests, simulations, and local benches while ignoring `Z00Z_STORAGE_REDB_ROOT`. Operator-bound durable startup still goes through `SettlementStore::try_new()` or `load(...)`, so this bench lane does not exercise env-root reload drift.
- Внутреннее распараллеливание уже есть: планирование идёт через `sched_run_local_queued` в [hjmt_commit.rs](../../crates/z00z_storage/src/settlement/hjmt_commit.rs), tree commits через `sched_map` в [hjmt_commit.rs](../../crates/z00z_storage/src/settlement/hjmt_commit.rs), proof batch тоже через `sched_map` в [hjmt_proof.rs](../../crates/z00z_storage/src/settlement/hjmt_proof.rs).
- По insert’ам spreading помогает умеренно: `insert_many_serials` ~1467 item/s против hot-serial `asset_batch` ~1107 item/s. Это примерно `+32%`, но не порядок величины.
- По delete’ам картина не универсальна: hot-serial delete сейчас даже чуть выше по `item/s`, чем wide-definition delete.
- Свежий targeted bench scheduler path показал, что внутренний parallel proof batch на этой гранулярности не выигрывает: для 48 path `scheduler/serial_batch` ~7.47 ms, `scheduler/parallel_batch` ~7.76 ms в [settlement_hjmt.rs](../../crates/z00z_storage/benches/settlement_hjmt.rs). То есть overhead уже заметен.

Итог:  
`да`, более wide распределение ключей может немного поднять insert throughput.  
`нет`, текущая кодовая база не подтверждает, что просто “добавить sharding” решит проблему кардинально. Сейчас bottleneck выглядит смешанным: hot-key locality, commit/journal overhead и scheduler overhead на мелких job’ах.

Отдельно: назвать текущие цифры “катастрофическими для production” только по repo нельзя, потому что в кодовой базе нет зафиксированного production SLO/TPS target, с которым их сравнивать.

EXPANDABLE: details available for where именно резать write-path overhead next.

Разница такая:

- `ops/s` = сколько **целых bench-операций** проходит за секунду.
- `item/s` = сколько **логических элементов внутри этой операции** проходит за секунду.

Строго по коду одна `op` тут не равна одному asset/right/proof.  
В `settlement_shard` одна `op` это целый batch-вызов:

- insert/delete: один `store.apply_settlement_ops(...)` в [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs), [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs)
- proof: один `store.settlement_proof_blobs(&paths)` в [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs)

Как я считал:

- `ops/s = 1 / median_seconds`
- `items/s = batch_size * ops/s`

`batch_size` берётся из самих lane:

- `many_defs(96)` в [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs)
- `many_sers(..., 96)` в [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs)
- `hot_assets(..., 128)` в [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs)
- `hot_rights(..., 64)` в [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs)
- `hot_bucket_assets(..., 8)` в [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs)
- `hot_assets(..., 64)` для proof batch в [settlement_shard.rs](../../crates/z00z_storage/benches/settlement_shard.rs)

Сами генераторы batch’ей тут: [settlement_corpus.rs](../../crates/z00z_storage/src/fixture_support/settlement_corpus.rs).

Пример из документа [benchmarks.md](../../docs/tech-papers/benchmarks.md):

- `insert_many_definitions`: `Median = 79.350 ms`
- `ops/s = 1 / 0.079350 ≈ 12.60`
- batch size = `96`
- `items/s = 96 * 12.60 ≈ 1210`

То есть:

- `ops/s` нужен, чтобы видеть latency/throughput **целого API-подобного batch вызова**
- `items/s` нужен, чтобы честно сравнивать lane с разными batch sizes (`8`, `64`, `96`, `128`)

Ещё одна тонкость: для `proof-many` метрика `items/s` по смыслу точнее как `paths/s`, потому что там batch доказывает много path, а не вставляет много store-item.

Если хочешь, я могу сразу привести весь `docs/tech-papers/benchmarks.md` к одной строгой терминологии: `ops/s`, `store-items/s`, `proof-paths/s`.  
EXPANDABLE: details available for exact formulas and per-lane recalculation table.
