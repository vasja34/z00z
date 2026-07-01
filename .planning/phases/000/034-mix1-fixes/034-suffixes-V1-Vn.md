Сейчас доберу недостающие usage-evidence по оставшимся строкам, чтобы вернуть уже не частичную критику, а полностью исправленную production-oriented таблицу по всем сигнатурам.

## Fixed Table

Ниже уже исправленная таблица в вашей реальной шкале:

- `production-current` = реально идёт по текущему non-test production path
- `====reserved-future====` = используется текущим production кодом для чтения/миграции старых форматов
- `====reserved-future====` = существует как versioned/protocol constant, но current production path его не выбирает

| Элемент                          | Тип                       | Исправленный статус | Короткое основание                                           | Пути                                                         |
| -------------------------------- | ------------------------- | ------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| AAD_SECRET_VER_V1                | version selector constant | ==reserved-future== | current code still supports old secret AAD fallback          | crates/z00z_wallets/src/db/redb_wallet_crypto.rs; crates/z00z_wallets/src/db/redb_wallet_store_open_session.rs; redb_wallet_store_migrations.rs |
| AAD_SECRET_VER_V2                | version selector constant | production-current  | current target secret AAD version                            | crates/z00z_wallets/src/db/redb_wallet_crypto.rs; redb_wallet_store_migrations.rs |
| AAD_VERSION_V1                   | version constant          | production-current  | current and only supported seed-container AAD version        | seed_cipher_container.rs                                     |
| Argon2idParamsV1                 | struct name               | production-current  | current persisted seed/KDF parameter struct                  | crates/z00z_wallets/src/core/key/seed_cipher_params.rs; crates/z00z_wallets/src/core/key/seed_cipher_container.rs; seed_cipher_persistence.rs |
| BackupAssociatedDataV1           | struct name               | production-current  | current backup AAD struct used by exporter/importer/verify   | crates/z00z_wallets/src/core/backup/backup_wire.rs; crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs; crates/z00z_wallets/src/core/backup/backup_importer_impl.rs; backup_exporter_verify.rs |
| BackupCompressionV1              | struct name               | production-current  | current backup compression wire struct                       | crates/z00z_wallets/src/core/backup/backup_wire.rs; crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs; backup_importer_impl.rs |
| BackupContainerV1                | struct name               | production-current  | current backup container type in exporter/importer/verify    | crates/z00z_wallets/src/core/backup/backup_wire.rs; crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs; crates/z00z_wallets/src/core/backup/backup_importer_impl.rs; backup_exporter_verify.rs |
| BackupEncryptionV1               | struct name               | production-current  | current backup encryption wire struct                        | crates/z00z_wallets/src/core/backup/backup_wire.rs; crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs; backup_importer_impl.rs |
| BackupPayloadV1                  | struct name               | ==reserved-future== | legacy backup payload still accepted by current importer/verify code | crates/z00z_wallets/src/core/backup/backup_wire.rs; crates/z00z_wallets/src/core/backup/backup_importer_impl.rs; backup_exporter_verify.rs |
| BackupPayloadV3                  | struct name               | ==reserved-future== | older export-pack payload still supported by current importer/verify code | crates/z00z_wallets/src/core/backup/backup_wire.rs; crates/z00z_wallets/src/core/backup/backup_importer_impl.rs; backup_exporter_verify.rs |
| BackupPayloadV4                  | struct name               | production-current  | current newest backup payload                                | crates/z00z_wallets/src/core/backup/backup_wire.rs; crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs; crates/z00z_wallets/src/core/backup/backup_importer_impl.rs; backup_exporter_verify.rs |
| CheckpointStmtV1                 | struct name               | production-current  | current checkpoint statement type                            | crates/z00z_storage/src/checkpoint/artifact_stmt.rs; crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs; artifact_final.rs |
| CLAIM_PROOF_V2                   | field/tag constant        | production-current  | current canonical proof type tag for claim tx                | crates/z00z_wallets/src/core/tx/claim_wire_types.rs; crates/z00z_wallets/src/core/tx/claim_tx.rs; claim_tx_verifier_impl.rs |
| CLAIM_SCHEMA_V1                  | schema/version constant   | production-current  | current claim receipt schema                                 | claim_receipt.rs                                             |
| ClaimAuthoritySigV2              | struct name               | production-current  | current claim authority signature type                       | crates/z00z_crypto/src/claim/v2.rs; crates/z00z_wallets/src/core/tx/claim_auth.rs; claim_tx_verifier_impl_proof.rs |
| ClaimSourceProof               | struct name               | production-current  | current claim proof build/verify path                        | crates/z00z_crypto/src/claim/v2.rs; crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs; crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs; store_query.rs |
| ClaimStmtV2                      | struct name               | production-current  | current canonical claim statement                            | crates/z00z_crypto/src/claim/v2.rs; crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs; claim_tx_helpers.rs |
| DENYLIST_BLOOM_V1                | embedded data constant    | production-current  | current password denylist bloom payload                      | crates/z00z_wallets/src/core/security/password.rs; password_checks.rs |
| FEE_WGT_VER_V1                   | version tag constant      | production-current  | current fee weight model tag                                 | fee_estimator.rs                                             |
| HKDF_INFO_REDB_DATA_V2           | HKDF info constant        | ==reserved-future== | exported constant, not directly selected by current wallet storage path | kdf_domains.rs                                               |
| HKDF_INFO_REDB_INDEX_V2          | HKDF info constant        | ==reserved-future== | exported constant, not directly selected by current wallet storage path | kdf_domains.rs                                               |
| HKDF_INFO_REDB_INTEGRITY_V2      | HKDF info constant        | ==reserved-future== | exported constant, not directly selected by current wallet storage path | kdf_domains.rs                                               |
| HKDF_INFO_VER_V1                 | version selector constant | ==reserved-future== | current code still opens/migrates old hkdf-info wallets via V1 | crates/z00z_wallets/src/db/redb_wallet_crypto.rs; crates/z00z_wallets/src/db/redb_wallet_store_open_session.rs; redb_wallet_store_migrations.rs |
| HKDF_INFO_VER_V2                 | version selector constant | production-current  | current target hkdf-info version                             | crates/z00z_wallets/src/db/redb_wallet_crypto.rs; redb_wallet_store_migrations.rs |
| KdfParams::VERSION_V1            | version constant          | ==reserved-future== | old KDF record version still accepted and migrated           | crates/z00z_wallets/src/db/redb_wallet_crypto.rs; crates/z00z_wallets/src/db/redb_wallet_store_migrations.rs; crates/z00z_wallets/src/core/key/key_manager_redb.rs; wallet_backup_kdf.rs |
| KdfParams::VERSION_V2            | version constant          | production-current  | current selected KDF record version                          | crates/z00z_wallets/src/db/redb_wallet_crypto.rs; crates/z00z_wallets/src/core/key/key_manager_redb.rs; wallet_backup_kdf.rs |
| MAX_PROOF_SIZE_V1                | proof size limit constant | production-current  | live single and batch verify paths enforce it                | crates/z00z_crypto/src/backend_range_proofs.rs; crates/z00z_crypto/src/backend_batch.rs; types_validation.rs |
| MAX_PROOF_SIZE_V2                | proof size limit constant | ==reserved-future== | generic version-2 bound exists, but current backend path does not select it | crates/z00z_crypto/src/crypto_constants.rs; crates/z00z_crypto/src/types_validation.rs; crates/z00z_crypto/src/types.rs; lib.rs |
| META_WALLET_INTEGRITY_V1         | schema/meta key constant  | production-current  | current wallet integrity meta key                            | crates/z00z_wallets/src/db/schema_keys.rs; crates/z00z_wallets/src/wasm/schema_keys.rs; crates/z00z_wallets/src/db/wlt_validate.rs; redb_wallet_store_crypto_ops.rs |
| OBJECT_PAYLOAD_HEADER_VERSION_V1 | version constant          | production-current  | current encrypted object payload header version              | redb_wallet_store_codecs.rs                                  |
| RANGE_PROOF_BITS_V1              | constant                  | production-current  | current width of proof path and backend init                 | crates/z00z_crypto/src/backend_init.rs; crates/z00z_crypto/src/backend_range_proofs.rs; amount.rs |
| RANGE_PROOF_BITS_V2              | constant                  | ==reserved-future== | defined/exported, but current backend accepts only V1 bits   | crates/z00z_crypto/src/crypto_constants.rs; crates/z00z_crypto/src/types.rs; lib.rs |
| ReceiverCardRecordV1             | struct name               | production-current  | current receiver-card publication contract                   | receiver_card_record.rs                                      |
| SEMANTIC_KEY_VERSION_V1          | version constant          | production-current  | current semantic index key format                            | crates/z00z_wallets/src/db/index_codecs.rs; index_codecs_body.rs |
| SPEND_AUTH_V1                    | domain separator literal  | production-current  | literal inside current SPEND_AUTH_CTX                        | prover.rs                                                    |
| TX_TIME_KEY_VERSION_V1           | version constant          | production-current  | current tx-time index key format                             | crates/z00z_wallets/src/db/index_codecs.rs; index_codecs_tx_time.rs |
| TxStoreMetaV1                    | struct name               | production-current  | current RPC tx storage metadata format                       | tx_rpc_storage.rs                                            |

## Bottom Line

На ваш прямой вопрос по примеру:

- `MAX_PROOF_SIZE_V1` — это текущий production limit
- `MAX_PROOF_SIZE_V2` — это не current production, а reserved/future V2 bound

И в таком же смысле я исправил всю таблицу выше: не “active vs old”, а “current production vs production compatibility vs future reserved”.

Если хотите, следующим сообщением я могу без дополнительной перепроверки отдать ту же fixed table в одном из двух форматов:

1. TSV для прямой вставки в файл  
2. Markdown с отдельной колонкой “Почему не production-current” только для non-current строк