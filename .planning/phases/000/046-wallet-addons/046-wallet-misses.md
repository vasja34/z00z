`Snapshot``250``WalletPersistenceState` bytesDeprecated snapshot container kept inside `.wlt` instead of a separate `wallet_*.bin.enc` file



Ещё одна поправка к картине: transaction lane достаточно глубокий — сборка считает fee, строит recipient/change/fee outputs, готовит witness/proof/auth и сохраняет pending tx; reconcile меняет claimed assets и откатывает их при сбое записи evidence. Это уже не “демо-стаб”, но simulator должен явно показать эти свойства, иначе они остаются скрытыми.

Пробелы лежат дальше: полноценный RPC tx lifecycle, receiver import parity, TOFU/card negative paths и restore/recovery ещё не вынесены в явные runtime stages.


не все user-facing wallet/asset операции являются полноценными end-to-end бизнес-операциями. Главная недостающая демонстрация сейчас не в том, что кошелёк “пустой”, а в том, что simulator ещё не прогоняет весь настоящий wallet.tx.* lifecycle через RPC.



Что не стоит считать 100% завершённым

wallet.asset.send_asset строит stealth output, проверяет TOFU/request/policy/rate-limit, но это не главный полноценный spend lifecycle. Он всё ещё получает guard-style tx id от service compatibility layer. Канонический путь для настоящего spend/reconcile сейчас должен быть wallet.tx.build_transaction -> broadcast -> reconcile.

asset.split/merge/stake/swap/unstake в RPC уже не “пустые” полностью: они валидируют session/assets/amounts и возвращают DTO, stake держит in-memory entry. Но это не полноценные ledger-mutating операции с durable tx/reconcile authority.

В WalletService остались compatibility/reachability placeholder methods и stale comments: wallet_service_actions_receive.rs, wallet_service_actions_reachability.rs, wallet_service_actions_rpc.rs.

rotate_master_key сейчас не полноценная persisted seed rotation. RPC делает auth/rate-limit/audit и вызывает in-memory rederive flow, но service-level fallback в wallet_service_actions_rpc.rs всё ещё placeholder-like.

Есть doc/header drift: некоторые файлы всё ещё пишут “stub/Phase 1”, хотя под ними уже живой код.


Stage 4/5/6/11 показывают tx package/checkpoint/JMT mechanics, но значимая часть tx path идёт через simulator-specific helpers, а не через полный wallet.tx.* RPC lifecycle.


Что я бы добавил в z00z_simulator в первую очередь

Wallet TX RPC lifecycle stage: Alice wallet.tx.build_transaction -> проверить, что input assets зарезервированы и unavailable -> wallet.tx.cancel_transaction -> проверить release -> rebuild -> wallet.tx.broadcast_transaction -> wallet.tx.reconcile_transaction -> wallet.tx.get_transaction_history/details. Это главный missing E2E.

Receiver submit parity stage: sender wallet.tx.export_transaction, receiver wallet.tx.import_transaction, проверить imported=true, owned outputs, receiver-side history, затем broadcast/reconcile через receiver path.

Tamper/fail-closed stage: bad tx id/hash, wrong chain id, bad checkpoint roots, wrong spent/created ids, corrupted portable tx metadata hash. Проверить, что reconcile/import fail-closed и claimed assets не мутируют.

Backup restore with real history: после non-empty tx history создать backup, restore с WalletPlusHistory, сравнить wallet id, claimed assets, tx-history JSONL, плюс wrong password negative case.

TOFU/payment request stage: create payment request -> validate -> send/build tx with request -> changed receiver card rejection -> confirm/revoke/rotate receiver view.

Session hardening stage: failed unlock/backoff, wallet.lifecycle.on_event, auto-lock expiry, show-seed rate limiting and no secret leakage in logs.

Multi-status history stage: pending + cancelled + confirmed + imported transactions, cursor/sort/filter checks, append-only JSONL behavior.

Asset UX clarity stage: either promote split/merge/stake/swap to real tx-backed flows, or keep them explicitly as non-ledger demo operations and assert they do not pretend to mutate confirmed state.

Итог: для функционирования Z00Z wallet уже есть рабочее ядро: persistence, receiver, backup, asset import/receive, tx build/broadcast/cancel/reconcile/history. Чтобы доказать это красиво и без “это только stubs”, simulator нужно прежде всего добавить stage, который проходит именно через wallet.tx.* RPC от spend до reconcile, потому что сейчас это самая большая недопоказанная возможность кошелька. 


отдельной redb-таблицы для assets тут нет. Каноническое “хранилище assets” в wallet-слое — это wallet_claimed_assets в памяти, а на диск они попадают в snapshot payload WalletPersistenceState.claimed_assets: Vec<AssetWire>; сам snapshot лежит как encrypted object kind Snapshot в OBJECTS_TABLE wallet_service_types_core.rs, snapshot_types.rs, tables.rs.

---------

Если wallet просто сканирует и “видит” свой leaf, это само по себе ничего не пишет. asset.receive идёт через scan_asset_report и только возвращает RuntimeReceiveAssetResponse без persistence asset_impl_server_transfer.rs. Запись начинается только в каноническом recv_range(...): claim_scan_hits берёт найденный Mine leaf, прогоняет его через recv_claim_asset и затем вызывает recv_route(..., ReceiveNext::PersistClaim) wallet_service_actions_receive.rs, wallet_service_store_support.rs, wallet_service_actions_reachability.rs.

Что реально меняется при таком claim-пути:

в памяти обновляется wallet_claimed_assets для этого wallet_id wallet_service_types_core.rs;

put_claimed_asset(...) валидирует asset и добавляет его в claimed set wallet_service_actions_reachability.rs;

create_snapshot(...) вытаскивает claimed assets через snapshot_claimed_assets(...) и кладёт их в WalletPersistenceState.claimed_assets wallet_service_store_support.rs, wallet_service_store_persistence_pack_snapshot.rs;

write_wallet_snapshot(...) пишет encrypted snapshot object в OBJECTS_TABLE и использует pointer META_SNAPSHOT_OBJECT_ID; при этом bump_wallet_write_meta(...) обновляет META_WALLET_SAVE_SEQ, META_WALLET_UPDATED_AT и META_WALLET_INTEGRITY backup.rs, meta.rs, schema_keys.rs, schema_keys.rs;

если используется recv_range(...), отдельно сохраняется scan cursor: upsert_scan_state(...) пишет ScanState object и pointer META_SCAN_STATE_OBJECT_ID, а через write_object_with_indexes(...) этот путь тоже bumps save_seq / updated_at / integrity и трогает index_manifest wallet_service_actions_receive.rs, upserts.rs, schema_keys.rs, tables.rs.

Итого по таблицам/объектам: при обнаружении своего leaf assets не пишутся в отдельную asset-table; они идут в wallet_claimed_assets → snapshot payload claimed_assets → encrypted snapshot object в objects. Дополнительно может обновиться scan_state object. В .wlt для этого сценария задействованы meta и objects; secrets не меняется, а отдельной redb asset-table здесь нет asset_rpc_registry.rs, tables.rs.


TUT EST PROBLEMA - KOGDA PROISHODIT CLAIM ILI UZNAVANIE SVOIH ASSETS V JWT-STORAGE VO VREMAJ SCAN TO VSE ASSETS DOLZHNI KLASTSJA V KAKOETO HRANILIWE - OBJEKT ILI TABLICA CHTOB POTOM NA EE BAZE K PRIMERU FORMIROVAT TRANSACIJU 
KOSHELEK ZHE DOLZHEN VIBRAT KAKIE- TO INPUT ASSETS OPREDELEIT OTPUT ASSETS I FEES
KAK ETO VSE FUNKCIONIRUET ? 
KAK ETO MOZHET RABOTAT V SIMULATORE ESLI ONO NE RELAIZOVANO NA UROVEN wlt ?!!










