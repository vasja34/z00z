## 1. Celenium — “видимость и доказуемость” Z00Z на Celestia

**Celenium** — это explorer/API/indexer для Celestia: blobs, namespaces, transactions, blocks, stats. У Celenium есть REST API для исторических данных, blobs и статистики, а их indexer open-source и может быть запущен самостоятельно. ([api-docs.celenium.io](https://api-docs.celenium.io/?utm_source=chatgpt.com))

Для Z00Z это полезно вот так:

### A. Investor dashboard

Ты можешь показать инвестору не просто “мы используем Celestia”, а живой dashboard:

```text
Z00Z testnet epoch: 1245
Celestia height: 5839201
Namespace: z00z-testnet-v1
Blob status: included
prev_root: 0x...
new_root: 0x...
spent_delta_root: 0x...
created_delta_root: 0x...
checkpoint_proof: available
```

Это очень сильно для грантов/инвесторов: **каждый checkpoint Z00Z реально лежит в Celestia DA**.

### B. Watcher service

Z00Z watcher может через Celenium API искать blobs по namespace и проверять:

```yaml
watcher_checks:
  - blob_exists
  - blob_height_confirmed
  - namespace_correct
  - checkpoint_schema_valid
  - prev_root_matches_last_known_root
  - new_root_matches_state_transition
  - proof_ref_available
```

То есть Celenium помогает не строить всё с нуля на первом этапе.

### C. Debugging DA problems

Если aggregator сказал: “я отправил checkpoint в Celestia”, watcher может проверить:

```text
1. tx exists?
2. blob exists?
3. correct namespace?
4. blob decodable?
5. expected checkpoint hash == actual hash?
```

Celestia node API тоже позволяет получать blobs по namespace на заданной высоте, но Celenium удобнее для dashboard/explorer/indexed history. ([Celestia Documentation](https://docs.celestia.org/build/rpc/node-api/?utm_source=chatgpt.com))

### D. Grant optics

Для Celestia grant это важно: ты показываешь **DA usage**.

```text
Z00Z generates continuous useful demand for Celestia blobs:
- checkpoint blobs
- state transition blobs
- proof reference blobs
- optional inbox snapshot CIDs
```

**Что не делать:** не класть в Celestia/Celenium приватные wallet secrets, receiver data, raw ownership data. Только commitments, roots, deltas, encrypted blobs, proof refs.

------

## 
