### 1. Nullifiers

1. nullifiers ??? nullifier_store {NullifierClaim, NullifierLease  - ponajt otkuda vzjalis nullifiers v sisteme, kogda JMT empty leaf govorit o nalishii asset ili none: ego otsutstvii
   zachem nuzhni eti nullifieri i komu oni prinadlezhat



### 2. Findings

Medium: публичный runtime scanner остаётся footgun-ом по validation layering. В stealth_scanner.rs:193 при отсутствии stealth-полей scanner сразу возвращает NotMine на stealth_scanner.rs:198, а не явный invalid-input. Да, service-layer это компенсирует через precheck в wallet_service.rs:1318 и wallet_service.rs:1324, но сам live surface StealthOutputScanner::scan_leaf() остаётся небезопасным для прямых вызовов: malformed runtime asset можно тихо деградировать в “чужой”, а не в “некорректный”. Это не ломает текущий RPC/service handoff, но нарушает жёсткую идею design foundation про assertion на том слое, где реально происходит invalid-input boundary.

Medium: canonical leaf handoff разделён на два API с разной семантикой ошибки, и receiver_scan_leaf() сам по себе недостаточен для честной классификации tamper/invalid-proof. В leaf_scan.rs:29 и leaf_scan.rs:33 Tag|Decrypt превращаются в Ok(None), тогда как тот же state в leaf_scan.rs:71 и leaf_scan.rs:51 уже честно мапится в InvalidProof. Для корректного handoff caller обязан знать, что pack-path и report-path нельзя рассматривать как взаимозаменяемые. Это не баг текущей реализации, а API-pitfall: прямой пользователь receiver_scan_leaf() легко потеряет proof-failure taxonomy.

Low: sender live surface не дотягивает до полной provider-based determinism/compliance из design foundation. В актуальном дереве интересующий builder.rs переехал в output.rs:203. Там builder использует системную энтропию напрямую для blinding на output.rs:284 и для hedged r через get_rng_bytes() на output.rs:422. С криптографической точки зрения это нормально и даже ожидаемо, но с точки зрения полной архитектурной дисциплины это остаётся non-injected randomness surface. То есть crypto-correctness тут не сломана, но строгая “trait-based determinism / provider-backed state” из полного design doc соблюдена не полностью.

Что проверил

Sender handoff в output.rs:203: request/card bind, M1 ownership binding, leaf_ad, s_out, tag16, commitment binding.
Canonical leaf receive в leaf_scan.rs:63 и leaf_scan.rs:71.
Runtime scanner path в stealth_scanner.rs:193, tag-prefilter branch в stealth_scanner.rs:218, и outward taxonomy mapping в types.rs:285 и types.rs:292.
Итог
Критического crypto-break в reviewed live handoff surfaces я не вижу. Sender→canonical leaf→runtime scanner в текущем виде логически согласован, M1/decrypt/commitment stages не выглядят перепутанными, а публичная receive taxonomy стабильно сходится к Detected / InvalidProof / NotMine. Реальные риски здесь не в core crypto, а в surface discipline: два API требуют правильного способа вызова, и прямой caller может потерять классификацию invalid input или invalid proof, если возьмёт “не тот” entry point в отрыве от service-layer контракта.

Если хочешь, следующим сообщением я могу уже не review, а предложить минимальный hardening patch для этих live surfaces:

сделать scanner-level malformed runtime asset явным reject вместо молчаливого NotMine,
развести canonical leaf pack и leaf report контракт так, чтобы caller не мог случайно потерять InvalidProof,
вынести RNG в provider-injected sender surface без ломки текущего API.



### 3. The backup flow does not restore full wallet state

- **Claim:** The document says a wallet backup snapshot should be enough to restore the wallet fully, including seed, coin maps, pending ACKs, and FSM state, on a new device.
- **Rating:** VERIFIED
- **Finding:** The active backup importer currently reconstructs only wallet identity and network metadata. The imported payload returns empty `keys`, `transactions`, and `assets` vectors, so it does not recreate the document's full wallet-state snapshot.
- **Source:**
  [backup_importer_impl.rs](../../crates/z00z_wallets/src/core/backup/backup_importer_impl.rs#L227-L229),
  [backup_importer.rs](../../crates/z00z_wallets/src/core/backup/backup_importer.rs),
  [wallet.md](./10_Z00Z_Wallet.md)
- **Recommendation:** Reword the backup section to reflect the current implementation until restoration of wallet state, coin records, and pending operations is added. The importer currently reconstructs only metadata-level wallet identity, not the full wallet state.

### 4. The proposed wallet snapshot schema fields are absent from the active backup container

- **Claim:** The document defines a structured `Z00Z_WalletBackup` snapshot containing `wallet_seed`, `spent_index_map`, `pending_ack`, `confirmed_coin_set`, `nullifier_set_seen`, and a backup signature.
- **Rating:** VERIFIED
- **Finding:** The implemented backup container is a smaller file-based JSON object with metadata, encryption parameters, compression parameters, checksum, and ciphertext. The active schema does not expose the document's wallet-state fields or a backup signature field.
- **Source:**
  [backup_exporter_impl.rs](../../crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs),
  [backup_exporter.rs](../../crates/z00z_wallets/src/core/backup/backup_exporter.rs),
  [backup_importer_impl.rs](../../crates/z00z_wallets/src/core/backup/backup_importer_impl.rs),
  [wallet.md](./10_Z00Z_Wallet.md#L52-L56)
- **Recommendation:** Treat the YAML snapshot schema in the document as roadmap material until those wallet-state fields exist in the production backup container. The implemented backup format stores metadata, encryption and compression descriptors, checksum, and ciphertext instead.

