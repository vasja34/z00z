# Embedded Versioning and Version-Tagged Naming

This file is the Phase 036 execution surface for version markers that remain
embedded inside identifiers, enum-style value names, module names, local names,
or protocol-tagged literals even when the surrounding scope already carries the
version boundary.

This file is also the exhaustive audit surface for version-tagged names that
show up in live Rust code during Phase 036 verification. Every live non-test
`V0..Vn` or `v0..vn` Rust signature that belongs to this embedded-versioning
wave must appear directly in the raw inventory below.

The `Raw Inventory Appendix` remains the only task-generation surface for this
embedded-versioning wave.

All audit and translation tables in this file carry explicit `#` row IDs so
row-order decisions remain reviewable after edits.

## Raw-First Execution Model

Use the raw inventory directly:

- one non-test raw row = one declaration-backed or module-backed task candidate;
- one literal-backed raw row = one contract-string or domain-label task candidate;
- one local or test-only raw row = one cleanup-only task candidate;
- `Current classification` tells you whether the row is live naming noise,
  a real wire discriminant, a compatibility lane, or only local/test residue;
- `Action now` tells you whether this wave renames the row now, holds it in
  place, or removes it only after a compatibility window closes;
- `Future survivor target` tells you the intended simplified end-state name;
- `Notes` explains why the row exists and what must not be guessed.

Do not recreate a second summary layer outside the raw inventory. If a task
cannot be created from a raw row plus the execution order below, the raw row
must be fixed first.

Do not let a row disappear behind file-boundary ownership language. A live
version-tagged Rust signature for this wave must be visible in the raw
inventory here.

## Manual Broad-Scan Residual Disposition

The broad residual string scan was reviewed manually line by line before this
spec update. Only path-specific strings and errors that still belong to the
Phase 036 embedded-versioning wave are promoted into the raw inventory below.
Everything else stays out of the task surface on purpose.

| # | Residual bucket | Disposition | Reason | Representative paths |
| --- | --- | --- | --- | --- |
| 1 | Bench, example, and bin output strings | exclude from task surface | execution examples and debug output are not production rename targets | `crates/z00z_core/benches/**`, `crates/z00z_core/examples/**`, `crates/**/bin/**` |
| 2 | Generic log and telemetry text that only says `version`, `version=`, or `registry_version` | exclude from task surface | these are operational messages, not version-tagged identifier cleanup rows | `crates/z00z_core/src/assets/registry_core.rs`, `crates/z00z_core/src/assets/registry_snapshot.rs` |
| 3 | Comment and doc prose that mentions versioned RPC or lane names | exclude from task surface | comments do not define the production naming contract by themselves | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` |
| 4 | Domain-separation labels outside the active rename boundary | exclude from this wave | these literals are real contracts, but they belong to a wider domain-literal audit, not this embedded-identifier cleanup wave | `crates/z00z_crypto/src/domains.rs`, `crates/z00z_core/src/domains.rs` |
| 5 | Claim, receipt, tx-type, and RPC literal or error strings tied to the active wallet or crypto lanes | promote into literal-backed inventory | these strings are path-specific and directly participate in live claim, receipt, or RPC contracts | `crates/z00z_crypto/src/claim/v2.rs`, `crates/z00z_wallets/src/core/claim/claim_receipt.rs`, `crates/z00z_wallets/src/core/tx/claim_wire_types.rs`, `crates/z00z_wallets/src/adapters/rpc/**` |
| 6 | Legacy import and compatibility read strings already represented by compatibility declaration rows | do not duplicate unless the literal itself is the contract boundary | the declaration rows already own the cleanup decision; duplicate inventory would recreate a second planning surface | `crates/z00z_wallets/src/core/backup/**`, `crates/z00z_wallets/src/db/**` |

## Execution Order By Row Range

This is the execution order for the embedded-versioning planning surface.
Execute rows in this order and do not reorder them without changing this file.

1. `Step 0 - Freeze explicit wire discriminants and coexisting live lanes`:
   non-test rows `1-18`, `26`, `29-35`, `48-55`; literal rows `1-25`.
   Implement now: no delete and no rename. These rows carry real wire, schema,
   public RPC, explicit claim-lane, coexistence-lane, or future-reserved
   meaning even if the current Rust spelling is noisy.
   Implement later: revisit only during an explicit protocol, schema, RPC, or
   published-contract migration wave.
2. `Step 1 - Preserve compatibility shims and compatibility read-import lanes`:
   non-test rows `20-21`, `24-25`, `27`.
   Implement now: do not rename or delete these rows in the current cleanup.
   Implement later: remove them only when the linked compatibility read,
   public shim, or legacy import window is explicitly closed by code-backed
   retirement proof.
3. `Step 2 - Rename current internal declarations whose surrounding scope already encodes the version`:
   non-test rows `22-23`, `28`.
   Implement now: rename rows whose surrounding module, type, or API scope
   already carries the live version boundary and where the embedded
   `V1/V2/...` marker is only naming noise.
4. `Step 3 - Rename current single-version internal or persisted identifiers while keeping the underlying value contract unchanged`:
   non-test rows `36-43`, `45-47`, `52`.
   Implement now: rename the Rust symbol only. Do not change the encoded byte,
   literal payload, persisted numeric value, or schema-string content that the
   renamed symbol still points to.
5. `Step 4 - Hold current symbols that still pair with an explicit legacy lane or published outward contract`:
   non-test rows `19`, `44`.
   Implement now: no delete and no rename.
   Implement later: rename only after the paired legacy lane or published
   outward contract migration is explicitly approved.
6. `Step 5 - Review local and test-only residue after production decisions`:
   local/test rows `1-92`.
   Implement now: rename only the cleanup-only residue whose names do not carry
   a real legacy lane or explicit version-scenario meaning. Keep explicit test
   helpers that intentionally model legacy, V1, or V2 behavior.

## Implementation Examples

Use these examples as the canonical task-translation patterns for the
embedded-versioning wave.

### Example 1: Current internal wiring type that can be simplified now

- Raw row: non-test row `22` (`KeyExportPublicV2Params`)
- Classification: `production current`
- Action now: `rename now`
- Future survivor target: `KeyExportPublicParams`
- Execution step: `Step 2`
- Task meaning: this params type is internal RPC wiring only; the public RPC
   method already stays explicit outside this internal params struct, so keeping
   `V2` here adds naming noise rather than preserving a separate contract.

### Example 2: Internal diagnostic enum variant that is naming noise

- Raw row: non-test row `23` (`IntegrityMismatchV1`)
- Classification: `production current`
- Action now: `rename now`
- Future survivor target: `IntegrityMismatch`
- Execution step: `Step 2`
- Task meaning: this is an internal diagnostic enum variant, not a wire value,
   public schema tag, or compatibility lane.

### Example 3: Literal contract that must not be renamed blindly

- Raw row: literal row `6` (`CLAIM_TX_TYPE = "claim_tx_v1"`)
- Classification: `literal contract`
- Action now: `hold`
- Future survivor target: `CLAIM_TX_TYPE`
- Execution step: `Step 0`
- Task meaning: the value is part of a live transport contract. It may look
  noisy, but changing it is a protocol migration, not a naming cleanup.

### Example 4: Coexisting version lanes that must stay explicit for now

- Raw row: non-test row `16` (`AssetPackVersion::V1Basic`)
- Classification: `production current`
- Action now: `keep`
- Future survivor target: `AssetPackVersion::V1Basic`
- Execution step: `Step 0`
- Task meaning: both V1 and V2 asset-pack lanes are live in production
   processing. Removing the embedded version marker now would blur an active
   decode boundary rather than clean naming noise.

## Raw Inventory Appendix

This appendix is the raw working set behind the execution order above. It keeps
embedded version markers visible so nothing gets lost when the cleanup wave is
executed.

### Non-Test Signatures

| # | Signature | Type | Path | Current classification | Action now | Future survivor target | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `JmtSerVersion::V1` | assoc const | `crates/z00z_storage/src/serialization/artifact.rs` | wire discriminant | keep | `JmtSerVersion::V1` | canonical serialization schema tag; explicit value marker, not naming noise |
| 2 | `PrepSnapshotVersion::V1` | assoc const | `crates/z00z_storage/src/snapshot/types.rs` | wire discriminant | keep | `PrepSnapshotVersion::V1` | canonical snapshot schema tag |
| 3 | `CheckpointExecVersion::V1` | assoc const | `crates/z00z_storage/src/checkpoint/exec_input.rs` | wire discriminant | keep | `CheckpointExecVersion::V1` | canonical execution-input schema tag |
| 4 | `CheckpointLinkVersion::V1` | assoc const | `crates/z00z_storage/src/checkpoint/link.rs` | wire discriminant | keep | `CheckpointLinkVersion::V1` | canonical checkpoint-link schema tag |
| 5 | `CheckpointAuditVersion::V1` | assoc const | `crates/z00z_storage/src/checkpoint/audit.rs` | wire discriminant | keep | `CheckpointAuditVersion::V1` | canonical checkpoint-audit schema tag |
| 6 | `ClaimRootVer::V1` | assoc const | `crates/z00z_crypto/src/claim/v2.rs` | wire discriminant | keep | `ClaimRootVer::V1` | explicit root-version value inside the current claim-v2 lane |
| 7 | `ClaimRootVer::V2` | assoc const | `crates/z00z_crypto/src/claim/v2.rs` | future-reserved discriminant | keep | `ClaimRootVer::V2` | reserved explicit next root-version value |
| 8 | `ClaimProofVer::V1` | assoc const | `crates/z00z_crypto/src/claim/v2.rs` | wire discriminant | keep | `ClaimProofVer::V1` | explicit proof-version value inside the current claim-v2 lane |
| 9 | `ClaimProofVer::V2` | assoc const | `crates/z00z_crypto/src/claim/v2.rs` | future-reserved discriminant | keep | `ClaimProofVer::V2` | reserved explicit next proof-version value |
| 10 | `ClaimV2Err` | enum | `crates/z00z_crypto/src/claim/v2.rs` | production current public lane | hold | `ClaimErr` | re-exported public error surface for the live claim-v2 API; do not flatten naming until the outer claim lane is intentionally simplified |
| 11 | `CLAIM_V2_TAG` | const | `crates/z00z_crypto/src/claim/v2.rs` | wire helper tied to live lane | hold | `CLAIM_TAG` | current claim-v2 tag helper names the active outer lane explicitly; revisit only with an explicit claim-wire migration |
| 12 | `ZKPACK_V1_CT_LEN` | const | `crates/z00z_crypto/src/zkpack.rs` | wire helper | keep | `ZKPACK_V1_CT_LEN` | fixed-size helper for the documented V1 zkpack wire layout |
| 13 | `ZKPACK_V1_TOTAL_LEN` | const | `crates/z00z_crypto/src/zkpack.rs` | wire helper | keep | `ZKPACK_V1_TOTAL_LEN` | fixed-size helper for the documented V1 zkpack wire layout |
| 14 | `AssetPackVersion::V1Basic` | enum variant | `crates/z00z_core/src/assets/version.rs` | production current lane discriminant | keep | `AssetPackVersion::V1Basic` | `validate_serial_id_version()` and downstream decode logic still distinguish live V1 and V2 lanes explicitly |
| 15 | `AssetPackVersion::V2Memo` | enum variant | `crates/z00z_core/src/assets/version.rs` | production current lane discriminant | keep | `AssetPackVersion::V2Memo` | live V2 memo lane; not mere naming noise |
| 16 | `AssetPackPlainV2Memo` | struct | `crates/z00z_core/src/assets/leaf.rs` | production current coexistence lane | keep | `AssetPackPlainV2Memo` | lives beside `AssetPackPlain` and names the active V2 memo payload contract |
| 17 | `DecodedAssetPack::V1Basic` | enum variant | `crates/z00z_core/src/assets/leaf.rs` | production current lane discriminant | keep | `DecodedAssetPack::V1Basic` | downstream receive/scan code branches on the explicit V1 lane |
| 18 | `DecodedAssetPack::V2Memo` | enum variant | `crates/z00z_core/src/assets/leaf.rs` | production current lane discriminant | keep | `DecodedAssetPack::V2Memo` | downstream receive/scan code branches on the explicit V2 lane |
| 19 | `derive_key_v2_zero_padding` | fn | `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs` | production current compatibility-paired helper | hold | `derive_key_zero_padding` | current master-key derivation helper still lives beside a legacy V1 padding lane; do not rename until that pair is intentionally collapsed |
| 20 | `spend_v1` | mod | `crates/z00z_wallets/src/core/tx/spending.rs` | public compatibility shim | remove later | none | shim module preserves old public path while the unified spending implementation lives elsewhere |
| 21 | `events_v1` | mod | `crates/z00z_wallets/src/core/tx/spending.rs` | public compatibility shim | remove later | none | shim module preserves old public event path until the public cutover wave closes |
| 22 | `KeyExportPublicV2Params` | struct | `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs` | production current internal wiring | rename now | `KeyExportPublicParams` | params type is internal wiring-only; the surrounding RPC surface already carries the public version boundary |
| 23 | `IntegrityMismatchV1` | enum variant | `crates/z00z_wallets/src/core/wallet/errors_types.rs` | production current internal diagnostic | rename now | `IntegrityMismatch` | diagnostic code sits in wallet-internal error scope; embedded version marker is naming noise |
| 24 | `ProofBlobV0` | struct | `crates/z00z_storage/src/assets/proof.rs` | compatibility lane | remove later | none | legacy proof-blob compatibility surface; keep only until legacy decode support is retired |
| 25 | `ClaimNullRecV0` | struct | `crates/z00z_storage/src/assets/store_internal/redb_backend_state.rs` | compatibility lane | remove later | none | legacy claim-nullifier record surface; remove only with explicit compatibility closure |
| 26 | `v2` | mod | `crates/z00z_crypto/src/claim/mod.rs` | live lane module | hold | `claim` | the current claim module explicitly routes the live outer lane through `mod v2`; flatten only during an intentional claim-lane simplification |
| 27 | `multi_v1` | mod | `crates/z00z_wallets/src/core/tx/asset_selector.rs` | public compatibility shim | remove later | none | path-based shim preserves the older multi-asset selector surface and should retire only with explicit public cutover |
| 28 | `export_public_material_v2_impl` | async fn | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_requests.rs` | production current internal wiring | rename now | `export_public_material_impl` | internal helper behind the current RPC surface; the public v2 contract is tracked elsewhere |
| 29 | `CheckpointVersion::V1` | enum variant | `crates/z00z_storage/src/checkpoint/artifact_types.rs` | wire discriminant | keep | `CheckpointVersion::V1` | canonical checkpoint artifact schema version carried as an explicit discriminant |
| 30 | `CheckpointStatement::V1` | enum variant | `crates/z00z_storage/src/checkpoint/artifact_stmt.rs` | live statement discriminant | keep | `CheckpointStatement::V1` | active checkpoint statement lane remains explicit and is not a cosmetic naming candidate |
| 31 | `HKDF_INFO_REDB_DATA_V2` | const | `crates/z00z_crypto/src/kdf_domains.rs` | future-reserved cryptographic domain | keep | `HKDF_INFO_REDB_DATA_V2` | exported version-tagged domain constant must stay explicit until a real domain migration exists |
| 32 | `HKDF_INFO_REDB_INDEX_V2` | const | `crates/z00z_crypto/src/kdf_domains.rs` | future-reserved cryptographic domain | keep | `HKDF_INFO_REDB_INDEX_V2` | exported version-tagged domain constant must stay explicit until a real domain migration exists |
| 33 | `HKDF_INFO_REDB_INTEGRITY_V2` | const | `crates/z00z_crypto/src/kdf_domains.rs` | future-reserved cryptographic domain | keep | `HKDF_INFO_REDB_INTEGRITY_V2` | exported version-tagged domain constant must stay explicit until a real domain migration exists |
| 34 | `RANGE_PROOF_BITS_V2` | const | `crates/z00z_crypto/src/crypto_constants.rs` | future-reserved proof contract helper | keep | `RANGE_PROOF_BITS_V2` | exported proof-width helper is reserved contract surface, not naming noise |
| 35 | `MAX_PROOF_SIZE_V2` | const | `crates/z00z_crypto/src/crypto_constants.rs` | future-reserved proof contract helper | keep | `MAX_PROOF_SIZE_V2` | exported proof-size helper is reserved contract surface, not naming noise |
| 36 | `build_aad_v1` | fn | `crates/z00z_crypto/src/aead_aad.rs` | production current single-version helper | rename now | `build_aad` | current AAD bytes stay `z00z.aead.v1\0`; only the Rust helper name loses redundant suffix noise |
| 37 | `TX_TIME_KEY_VERSION_V1` | const | `crates/z00z_wallets/src/db/index_codecs.rs` | production current single-version internal schema tag | rename now | `TX_TIME_KEY_VERSION` | encoded key byte stays `1`; only the private constant identifier simplifies |
| 38 | `SEMANTIC_KEY_VERSION_V1` | const | `crates/z00z_wallets/src/db/index_codecs.rs` | production current single-version internal schema tag | rename now | `SEMANTIC_KEY_VERSION` | encoded key byte stays `1`; only the private constant identifier simplifies |
| 39 | `META_WALLET_INTEGRITY_V1` | const | `crates/z00z_wallets/src/db/schema_keys.rs` | production current single-version schema key identifier | rename now | `META_WALLET_INTEGRITY` | literal key stays `wallet.integrity.v1`; only the Rust constant name simplifies |
| 40 | `META_WALLET_INTEGRITY_V1` | const | `crates/z00z_wallets/src/wasm/schema_keys.rs` | production current single-version schema key identifier | rename now | `META_WALLET_INTEGRITY` | wasm mirror keeps the same literal value; only the constant identifier simplifies |
| 41 | `VERSION_V1` | const | `crates/z00z_wallets/src/core/key/seed_cipher_container.rs` | production current single-version persisted schema tag | rename now | `VERSION` | container payload version byte stays `1`; only the public constant identifier simplifies |
| 42 | `AAD_VERSION_V1` | const | `crates/z00z_wallets/src/core/key/seed_cipher_container.rs` | production current single-version AAD tag | rename now | `AAD_VERSION` | AAD framing byte stays `1`; only the constant identifier simplifies |
| 43 | `OBJECT_PAYLOAD_HEADER_VERSION_V1` | const | `crates/z00z_wallets/src/db/redb_wallet_store_codecs.rs` | production current single-version payload header tag | rename now | `OBJECT_PAYLOAD_HEADER_VERSION` | payload header byte stays `1`; only the Rust constant identifier simplifies |
| 44 | `ReceiverCardRecordV1` | struct | `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` | production current published contract | hold | `ReceiverCardRecord` | published receiver-card record is outward-facing and can simplify only in an explicit publication migration |
| 45 | `CLAIM_SCHEMA_V1` | const | `crates/z00z_wallets/src/core/claim/claim_receipt.rs` | production current single-version schema tag | rename now | `CLAIM_SCHEMA` | receipt schema byte stays `1`; only the constant identifier simplifies |
| 46 | `FEE_WGT_VER_V1` | const | `crates/z00z_wallets/src/core/tx/fee_estimator.rs` | production current single-version model tag | rename now | `FEE_WGT_VER` | literal model tag stays `fee-weight-v1`; only the Rust constant identifier simplifies |
| 47 | `TxStoreMetaV1` | struct | `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs` | production current single-version internal RPC metadata type | rename now | `TxStoreMeta` | decode shape stays the same; only the type name loses redundant suffix noise |
| 48 | `claim_stmt_hash_v2` | fn | `crates/z00z_crypto/src/claim/v2.rs` | production current lane helper | keep | `claim_stmt_hash_v2` | live claim-v2 helper must stay explicit while the outer claim lane is still version-scoped |
| 49 | `encode_single_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | future-reserved address lane helper | keep | `encode_single_v2` | reserved address-v2 helper is intentionally explicit and not active cleanup scope |
| 50 | `encode_dual_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | future-reserved address lane helper | keep | `encode_dual_v2` | reserved address-v2 helper is intentionally explicit and not active cleanup scope |
| 51 | `decode_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | future-reserved address lane helper | keep | `decode_v2` | reserved address-v2 helper is intentionally explicit and not active cleanup scope |
| 52 | `DENYLIST_BLOOM_V1` | const | `crates/z00z_wallets/src/core/security/password.rs` | production current single-version payload identifier | rename now | `DENYLIST_BLOOM` | embedded bytes stay unchanged; only the Rust constant identifier simplifies |
| 53 | `CLAIM_PROOF_V2` | const | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs` | production current transport tag | hold | `CLAIM_PROOF` | outer proof-type lane is still explicitly V2 in production transport; do not flatten until claim transport migration exists |
| 54 | `export_public_material_v2` | async fn | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` | production current public RPC lane | hold | `export_public_material` | current public RPC surface is explicitly versioned and must stay so until RPC migration exists |
| 55 | `export_public_material_v2` | async fn | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs` | production current public RPC lane implementation | hold | `export_public_material` | implementation stays explicit while the public RPC lane itself remains versioned |

### Literal-Backed Contracts

| # | Signature | Type | Path | Current classification | Action now | Future survivor target | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `DRAFT_ID_LABEL = "checkpoint_draft_id_v1"` | const literal | `crates/z00z_storage/src/checkpoint/ids.rs` | literal contract | hold | `DRAFT_ID_LABEL` | ID derivation label participates in stable checkpoint hashing |
| 2 | `CHECKPOINT_ID_LABEL = "checkpoint_final_id_v1"` | const literal | `crates/z00z_storage/src/checkpoint/ids.rs` | literal contract | hold | `CHECKPOINT_ID_LABEL` | ID derivation label participates in stable checkpoint hashing |
| 3 | `EXEC_ID_LABEL = "checkpoint_exec_id_v1"` | const literal | `crates/z00z_storage/src/checkpoint/ids.rs` | literal contract | hold | `EXEC_ID_LABEL` | ID derivation label participates in stable checkpoint hashing |
| 4 | `LINK_BIND_LABEL = "checkpoint_link_bind_v1"` | const literal | `crates/z00z_storage/src/checkpoint/link.rs` | literal contract | hold | `LINK_BIND_LABEL` | link-binding label participates in stable checkpoint hashing |
| 5 | `SPEND_AUTH_CTX = "Z00Z/SPEND_AUTH_V1"` | const literal | `crates/z00z_wallets/src/core/tx/prover.rs` | literal contract | hold | `SPEND_AUTH_CTX` | signing domain label; renaming changes signature domain separation |
| 6 | `CLAIM_TX_TYPE = "claim_tx_v1"` | const literal | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs` | literal contract | hold | `CLAIM_TX_TYPE` | transport contract string; changing it is a protocol migration |
| 7 | `CLAIM_PROOF_V2 = "claim_source"` | const literal | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs` | literal contract | hold | `CLAIM_PROOF` | outer proof-type transport tag is live and path-specific |
| 8 | `CLAIM_CTX = "z00z.wallet.claim_receipt.v1"` | const literal | `crates/z00z_wallets/src/core/claim/claim_receipt.rs` | literal contract | hold | `CLAIM_CTX` | receipt signing context is a domain-separation boundary, not cosmetic text |
| 9 | `"wallet.key.export_public_material_v2"` | RPC string literal | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` | literal contract | hold | `"wallet.key.export_public_material_v2"` | current RPC method string is a live transport contract and must stay explicit until an intentional RPC migration exists |
| 10 | `"wallet.key.export_public_material_v2"` | RPC string literal | `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs` | literal contract | hold | `"wallet.key.export_public_material_v2"` | dispatcher registration uses the same live transport string and must stay path-visible |
| 11 | `"wallet.key.export_public_material_v2"` | RPC string literal | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_requests.rs` | literal contract | hold | `"wallet.key.export_public_material_v2"` | request-side event and telemetry path still binds the live RPC string explicitly |
| 12 | `"claim-v2 decode failed"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | error surface names the live claim-v2 lane explicitly |
| 13 | `"claim-v2 root version must be non-zero"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 14 | `"claim-v2 proof version must be non-zero"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 15 | `"claim-v2 tx version must be non-zero"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 16 | `"claim-v2 output leaf list must be non-empty"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 17 | `"claim-v2 proof bytes must be non-empty"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 18 | `"claim-v2 authority signature is invalid"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 19 | `"claim-v2 root version mismatch"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 20 | `"claim-v2 proof version mismatch"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 21 | `"claim-v2 source root mismatch"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 22 | `"claim-v2 backend signing failed: {0}"` | error literal | `crates/z00z_crypto/src/claim/v2.rs` | literal error contract | hold | same | path-specific claim-v2 validation error |
| 23 | `"unsupported claim schema version: {}"` | error literal | `crates/z00z_wallets/src/core/claim/claim_receipt.rs` | literal error contract | hold | same | schema mismatch error is tied directly to `CLAIM_SCHEMA_V1` validation |
| 24 | `"tx_type must be claim_tx_v1"` | error literal | `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl.rs` | literal error contract | hold | same | validator error string binds the live claim transaction type expectation |
| 25 | `"claim proof version unsupported"` | error literal | `crates/z00z_crypto/src/claim/proof.rs` | compatibility error literal | hold | same | compatibility error for older claim-proof decoding remains explicit until old proof lane retirement is proven |

### Local And Test-Only Residue

| # | Signature | Type | Path | Current classification | Action now | Future survivor target | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `snapshot_v1` | local variable | `crates/z00z_core/src/assets/registry_tests.rs` | test residue | rename now | `downgrade_snapshot` | downgrade intent is already expressed by the test body |
| 2 | `snapshot_v1` | local variable | `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` | test residue | rename now | `initial_snapshot` | first applied snapshot in the version-sequencing test |
| 3 | `snapshot_v2` | local variable | `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` | test residue | rename now | `second_snapshot` | sequence position is enough; embedded version marker is not needed in the name |
| 4 | `snapshot_v3` | local variable | `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` | test residue | rename now | `third_snapshot` | sequence position is enough; embedded version marker is not needed in the name |
| 5 | `snapshot_v5` | local variable | `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` | test residue | rename now | `latest_snapshot` | test semantics are “higher version wins”, not “keep v5 in the variable name” |
| 6 | `def_v1` | local variable | `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` | test residue | rename now | `first_definition` | local test binding only |
| 7 | `arc_v1` | local variable | `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` | test residue | rename now | `first_arc` | local test binding only |
| 8 | `def_v2` | local variable | `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` | test residue | rename now | `replacement_definition` | local test binding only |
| 9 | `arc_v2` | local variable | `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` | test residue | rename now | `replacement_arc` | local test binding only |
| 10 | `"Asset V1"` | string literal | `crates/z00z_core/tests/assets/test_integration_assets_test12.rs` | test residue | rename now | `"Asset"` | test fixture string carries naming noise rather than protocol meaning |
| 11 | `container_v1` | local variable | `crates/z00z_wallets/src/services/wallet_service_store_transfer_import.rs` | local residue | rename now | `decoded_container` / `encrypted_container` | production local bindings on the import and export paths; both values are current containers rather than wire-lane selectors |
| 12 | `v1_name` / `v2_name` | local variables | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | test residue | rename now | `version_one_name` / `version_two_name` | generated test names can stay descriptive without suffix-style variable naming |
| 13 | `legacy_v1_bytes` | fn | `crates/z00z_wallets/src/services/wallet_service_tests.rs` | test residue with explicit legacy meaning | keep | same | helper intentionally emits legacy backup bytes for compatibility coverage |
| 14 | `SERIAL_V2` | const | `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` | test residue with explicit V2 lane meaning | keep | same | test constant intentionally targets the live V2 asset-pack serial range |
| 15 | `build_v2_case` | fn | `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` | test residue with explicit V2 lane meaning | keep | same | helper intentionally constructs a V2 memo case and should stay explicit |
| 16 | `aad_v2` | local variable | `crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops.rs` | local residue | rename now | `aad` | production-local binding for the current secret-record AAD lane; the suffix does not add contract meaning here |
| 17 | `detected_pack_to_bytes_rejects_oversized_v2_memo` | test fn | `crates/z00z_wallets/src/core/address/stealth_scanner/types_receive.rs` | test residue with explicit V2 lane meaning | keep | same | `#[cfg(test)]` helper verifies explicit V2 memo oversize rejection and should stay version-explicit |
| 18 | `test_v2_memo_roundtrip` | test fn | `crates/z00z_core/src/assets/leaf_tests.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 memo roundtrip coverage should stay version-explicit |
| 19 | `test_v2_memo_empty_roundtrip` | test fn | `crates/z00z_core/src/assets/leaf_tests.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 memo empty-roundtrip coverage should stay version-explicit |
| 20 | `test_v2_memo_rejects_oversize` | test fn | `crates/z00z_core/src/assets/leaf_tests.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 memo rejection coverage should stay version-explicit |
| 21 | `test_v2_memo_rejects_bad_len` | test fn | `crates/z00z_core/src/assets/leaf_tests.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 memo rejection coverage should stay version-explicit |
| 22 | `test_v2_memo_rejects_bad_blind` | test fn | `crates/z00z_core/src/assets/leaf_tests.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 memo rejection coverage should stay version-explicit |
| 23 | `test_decode_asset_pack_v1_lane` | test fn | `crates/z00z_core/src/assets/leaf_tests.rs` | test residue with explicit V1 lane meaning | keep | same | explicit V1 asset-pack decode coverage should stay version-explicit |
| 24 | `test_decode_asset_pack_v2_lane` | test fn | `crates/z00z_core/src/assets/leaf_tests.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 asset-pack decode coverage should stay version-explicit |
| 25 | `snapshot_v5_dup` | local variable | `crates/z00z_core/src/assets/registry_tests.rs` | test residue | rename now | `duplicate_snapshot` | local duplicate snapshot alias does not need embedded version noise |
| 26 | `snapshot_v6` | local variable | `crates/z00z_core/src/assets/registry_tests.rs` | test residue | rename now | `final_snapshot` | local ordinal snapshot alias does not need embedded version noise |
| 27 | `test_v1_low` | test fn | `crates/z00z_core/src/assets/version.rs` | test residue with explicit V1 lane meaning | keep | same | explicit V1 boundary test should stay version-explicit |
| 28 | `test_v1_high` | test fn | `crates/z00z_core/src/assets/version.rs` | test residue with explicit V1 lane meaning | keep | same | explicit V1 boundary test should stay version-explicit |
| 29 | `test_v2_low` | test fn | `crates/z00z_core/src/assets/version.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 boundary test should stay version-explicit |
| 30 | `test_v2_high` | test fn | `crates/z00z_core/src/assets/version.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 boundary test should stay version-explicit |
| 31 | `test_ver_v1_low` | test fn | `crates/z00z_core/tests/assets/test_serial_id_encoding.rs` | test residue with explicit V1 lane meaning | keep | same | explicit V1 serial-id boundary test should stay version-explicit |
| 32 | `test_ver_v1_high` | test fn | `crates/z00z_core/tests/assets/test_serial_id_encoding.rs` | test residue with explicit V1 lane meaning | keep | same | explicit V1 serial-id boundary test should stay version-explicit |
| 33 | `test_ver_v2_low` | test fn | `crates/z00z_core/tests/assets/test_serial_id_encoding.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 serial-id boundary test should stay version-explicit |
| 34 | `test_ver_v2_high` | test fn | `crates/z00z_core/tests/assets/test_serial_id_encoding.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 serial-id boundary test should stay version-explicit |
| 35 | `DEFINITION_WIRE_V1_SNAPSHOT` | test const | `crates/z00z_core/tests/assets/test_wire_format_snapshots.rs` | test residue with explicit V1 wire meaning | keep | same | golden V1 snapshot fixture is intentionally version-explicit |
| 36 | `v1` | local variable | `crates/z00z_core/tests/genesis/test_crypto_security.rs` | test residue | rename now | `first_case` | generic local bucket name does not need embedded version noise |
| 37 | `v2` | local variable | `crates/z00z_core/tests/genesis/test_crypto_security.rs` | test residue | rename now | `second_case` | generic local bucket name does not need embedded version noise |
| 38 | `v1` | local variable | `crates/z00z_crypto/src/commitments.rs` | local residue | rename now | `first_value` | local comparison binding does not need embedded version noise |
| 39 | `v2` | local variable | `crates/z00z_crypto/src/commitments.rs` | local residue | rename now | `second_value` | local comparison binding does not need embedded version noise |
| 40 | `key_v1` | local variable | `crates/z00z_crypto/src/kdf_tests.rs` | test residue with explicit V1 comparison meaning | keep | same | explicit V1 key-comparison local should stay version-explicit |
| 41 | `key_v2` | local variable | `crates/z00z_crypto/src/kdf_tests.rs` | test residue with explicit V2 comparison meaning | keep | same | explicit V2 key-comparison local should stay version-explicit |
| 42 | `max_v1` | local variable | `crates/z00z_crypto/src/types_tests.rs` | test residue with explicit V1 baseline meaning | keep | same | explicit V1 proof-size baseline local should stay version-explicit |
| 43 | `max_v2` | local variable | `crates/z00z_crypto/src/types_tests.rs` | test residue with explicit V2 baseline meaning | keep | same | explicit V2 proof-size baseline local should stay version-explicit |
| 44 | `test_claim_v2_frame_vector` | test fn | `crates/z00z_crypto/tests/test_claim_v2_contract.rs` | test residue with explicit V2 lane meaning | keep | same | explicit claim-v2 contract test should stay version-explicit |
| 45 | `test_claim_v2_sig_check` | test fn | `crates/z00z_crypto/tests/test_claim_v2_contract.rs` | test residue with explicit V2 lane meaning | keep | same | explicit claim-v2 signature contract test should stay version-explicit |
| 46 | `test_claim_v2_ver_mix` | test fn | `crates/z00z_crypto/tests/test_claim_v2_contract.rs` | test residue with explicit V2 lane meaning | keep | same | explicit mixed-version claim-v2 test should stay version-explicit |
| 47 | `test_claim_v2_source_root_mismatch` | test fn | `crates/z00z_crypto/tests/test_claim_v2_contract.rs` | test residue with explicit V2 lane meaning | keep | same | explicit claim-v2 mismatch test should stay version-explicit |
| 48 | `v1` | local variable | `crates/z00z_crypto/tests/test_pedersen.rs` | test residue | rename now | `first_value` | generic local comparison name does not need embedded version noise |
| 49 | `v2` | local variable | `crates/z00z_crypto/tests/test_pedersen.rs` | test residue | rename now | `second_value` | generic local comparison name does not need embedded version noise |
| 50 | `test_proof_blob_decode_legacy_v0_upgrades_root_bind` | test fn | `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy-v0 compatibility test should stay version-explicit |
| 51 | `test_version_v1_is_supported` | test fn | `crates/z00z_storage/src/serialization/artifact.rs` | test residue with explicit V1 support meaning | keep | same | explicit V1 support test should stay version-explicit |
| 52 | `ClaimNullRecV0` | test struct | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy-v0 compatibility test type should stay version-explicit |
| 53 | `export_public_material_v2_stub` | test mod | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs` | test residue with explicit V2 lane meaning | keep | same | explicit v2 test stub boundary should stay version-explicit |
| 54 | `test_snapshot_v3_verify_ok` | test fn | `crates/z00z_wallets/src/core/address/address_manager/tests.rs` | test residue with explicit V3 scenario meaning | keep | same | explicit V3 snapshot scenario should stay version-explicit |
| 55 | `test_decode_v2_single` | test fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | dormant future-reserved helper coverage | remove now | none | the helper block under test is marked not yet active and is reserved for a future migration rather than the current production lane |
| 56 | `test_decode_v2_dual` | test fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | dormant future-reserved helper coverage | remove now | none | the helper block under test is marked not yet active and is reserved for a future migration rather than the current production lane |
| 57 | `test_decode_v2_invalid_type` | test fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | dormant future-reserved helper coverage | remove now | none | the helper block under test is marked not yet active and is reserved for a future migration rather than the current production lane |
| 58 | `test_decode_v2_length_mismatch` | test fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | dormant future-reserved helper coverage | remove now | none | the helper block under test is marked not yet active and is reserved for a future migration rather than the current production lane |
| 59 | `test_decode_v2_rejects_wrong_version` | test fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | dormant future-reserved helper coverage | remove now | none | the helper block under test is marked not yet active and is reserved for a future migration rather than the current production lane |
| 60 | `test_v2_type_discriminates_correctly` | test fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | dormant future-reserved helper coverage | remove now | none | the helper block under test is marked not yet active and is reserved for a future migration rather than the current production lane |
| 61 | `test_import_legacy_v1_backup_is_rejected` | test fn | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy-v1 backup reject contract should stay version-explicit |
| 62 | `test_import_v4_roundtrip_preserves_chain` | test fn | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | test residue with explicit V4 scenario meaning | keep | same | explicit V4 roundtrip contract should stay version-explicit |
| 63 | `build_legacy_v1_bytes` | test fn | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy fixture builder should stay version-explicit |
| 64 | `aad_v1` | local variable | `crates/z00z_wallets/src/core/key/seed_cipher_metadata_tests.rs` | test residue with explicit V1 comparison meaning | keep | same | explicit AAD-version comparison local should stay version-explicit |
| 65 | `aad_v2` | local variable | `crates/z00z_wallets/src/core/key/seed_cipher_metadata_tests.rs` | test residue with explicit V2 comparison meaning | keep | same | explicit AAD-version comparison local should stay version-explicit |
| 66 | `key_v0` | local variable | `crates/z00z_wallets/src/core/key/stealth_keys_tests.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy key local should stay version-explicit |
| 67 | `key_v1` | local variable | `crates/z00z_wallets/src/core/key/stealth_keys_tests.rs` | test residue with explicit V1 comparison meaning | keep | same | explicit V1 key local should stay version-explicit |
| 68 | `hash_v0` | local variable | `crates/z00z_wallets/src/core/key/stealth_keys_tests.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy hash local should stay version-explicit |
| 69 | `v1` | local variable | `crates/z00z_wallets/src/core/wallet/snapshot_tests.rs` | test residue | rename now | `first_snapshot` | generic local snapshot binding does not need embedded version noise |
| 70 | `legacy_v1_restore_fails` | test fn | `crates/z00z_wallets/src/services/wallet_service_tests.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy restore failure coverage should stay version-explicit |
| 71 | `test_adv_serial_relabel_v2_is_not_mine` | test fn | `crates/z00z_wallets/tests/test_adversarial.rs` | test residue with explicit V2 lane meaning | keep | same | explicit adversarial V2 scenario should stay version-explicit |
| 72 | `leaf_ad_v1` | local variable | `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` | test residue with explicit V1 comparison meaning | keep | same | explicit V1 leaf-AAD comparison local should stay version-explicit |
| 73 | `leaf_ad_v2` | local variable | `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` | test residue with explicit V2 comparison meaning | keep | same | explicit V2 leaf-AAD comparison local should stay version-explicit |
| 74 | `test_v2_memo_leaf_scan_detects_owned_pack` | test fn | `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 memo scan coverage should stay version-explicit |
| 75 | `test_v2_memo_runtime_scan_keeps_memo_private` | test fn | `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 memo privacy coverage should stay version-explicit |
| 76 | `test_v2_memo_leaf_scan_rejects_bad_memo_len` | test fn | `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs` | test residue with explicit V2 lane meaning | keep | same | explicit V2 memo reject coverage should stay version-explicit |
| 77 | `test_receiver_card_record_v1_is_canonical_live_contract` | test fn | `crates/z00z_wallets/tests/test_receiver_card_record.rs` | test residue with explicit V1 contract meaning | keep | same | explicit receiver-card V1 contract test should stay version-explicit |
| 78 | `test_open_rejects_kdf_v1` | test fn | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy KDF reject test should stay version-explicit |
| 79 | `record_v1` | local variable | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy record fixture local should stay version-explicit |
| 80 | `kdf_v1_blob` | local variable | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy KDF blob fixture local should stay version-explicit |
| 81 | `record_v1_blob` | local variable | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy record blob fixture local should stay version-explicit |
| 82 | `wallet_key_export_public_material_v2_is_canonical_live_contract` | test fn | `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs` | test residue with explicit V2 contract meaning | keep | same | explicit RPC-v2 contract test should stay version-explicit |
| 83 | `card_v0` | local variable | `crates/z00z_wallets/tests/test_stealth_request.rs` | test residue with explicit legacy meaning | keep | same | explicit rotated-card old-lane local should stay version-explicit |
| 84 | `card_v1` | local variable | `crates/z00z_wallets/tests/test_stealth_request.rs` | test residue with explicit V1 lane meaning | keep | same | explicit rotated-card new-lane local should stay version-explicit |
| 85 | `card_v0` | local variable | `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs` | test residue with explicit legacy meaning | keep | same | explicit old-card lane local should stay version-explicit |
| 86 | `card_v1` | local variable | `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs` | test residue with explicit V1 lane meaning | keep | same | explicit rotated-card lane local should stay version-explicit |
| 87 | `outputs_v0` | local variable | `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs` | test residue with explicit legacy meaning | keep | same | explicit old-output lane local should stay version-explicit |
| 88 | `outputs_v1` | local variable | `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs` | test residue with explicit V1 lane meaning | keep | same | explicit new-output lane local should stay version-explicit |
| 89 | `v1` | local variable | `crates/z00z_wallets/tests/test_tx_poison.rs` | test residue | rename now | `first_payload` | generic poisoned JSON bucket does not need embedded version noise |
| 90 | `v2` | local variable | `crates/z00z_wallets/tests/test_tx_poison.rs` | test residue | rename now | `second_payload` | generic poisoned JSON bucket does not need embedded version noise |
| 91 | `v3` | local variable | `crates/z00z_wallets/tests/test_tx_poison.rs` | test residue | rename now | `third_payload` | generic poisoned JSON bucket does not need embedded version noise |
| 92 | `record_v1` | local variable | `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs` | test residue with explicit legacy meaning | keep | same | explicit legacy record migration fixture local should stay version-explicit |
| 93 | `test_rejects_wlt_open_v2` | test fn | `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs` | test residue with misleading V2 noise | rename now | `test_rejects_wlt_open_invalid_save_seq` | the test corrupts `META_WALLET_SAVE_SEQ`, not a wallet version field, so the current name overstates version semantics |

## Compact Patch-Plan By Disputed Row Group

This table is the compact execution map for the disputed groups raised during
doublecheck. It is not a second task-generation surface; every row-group entry
below resolves back to the raw rows already listed above.

| # | Row group | Raw rows | Delete now | Keep or hold now | Rename now | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | explicit wire and public-lane discriminants | non-test `1-18`, `26`, `29-35`, `48-55`; literal `1-25` | none | all rows | none | protocol, RPC, claim-lane, or literal-domain meaning is still live |
| 2 | compatibility shims and compatibility read-import lanes | non-test `20-21`, `24-25`, `27` | none | all rows | none | delete only after explicit retirement proof closes the linked compatibility window |
| 3 | current internal wiring and diagnostic noise | non-test `22-23`, `28` | none | none | all rows | surrounding scope already carries the version boundary |
| 4 | current single-version identifiers whose values stay versioned | non-test `36-43`, `45-47`, `52` | none | none | all rows | rename symbol only; keep literal bytes, schema bytes, and payload contents unchanged |
| 5 | current symbols blocked by paired legacy lane or published outward contract | non-test `19`, `44` | none | all rows | none | simplify only after outward-contract or paired-lane migration proof exists |
| 6 | local and test-only residue | local/test `1-92` | none | explicit legacy or V2 scenario helpers | cleanup-only residue | production decisions must land first |

## Table-to-Task Translation Protocol

Use this file as the source of truth when turning embedded-versioning rows into
execution tasks.

### Raw Inventory Appendix -> Non-Test Signatures

This is the production execution register for declaration-backed versioning
work that is executed directly from this file.

| # | Column | Task meaning | Required translation rule |
| --- | --- | --- | --- |
| 1 | `#` | raw row ID | Keep this as the per-signature subtask ID. |
| 2 | `Signature` | exact symbol under review | Use as the subtask title suffix. |
| 3 | `Type` | symbol kind | Record whether the subtask touches an assoc const, const, struct, enum, enum variant, module, or function. |
| 4 | `Path` | exact file boundary | One row equals one path-specific execution item. |
| 5 | `Current classification` | present-state contract | Distinguish wire discriminants from naming noise and compatibility-only lanes. |
| 6 | `Action now` | current-wave action | `rename now`, `keep`, `hold`, or `remove later` must be copied literally. |
| 7 | `Future survivor target` | survivor end-state label | The simplified name for the surviving lane after this wave or after a later compatibility closure. |
| 8 | `Notes` | per-signature constraint | Copy into the subtask evidence field. |

Required rule for production execution:

1. Do not rename explicit wire or schema value markers just because they contain `V1` or `V2`.
2. Do rename current-only internal declarations when the surrounding scope already encodes the version boundary.
3. Keep compatibility structs and public shim modules until the linked decode/import or public cutover window is explicitly closed.
4. If a live row belongs to this embedded-versioning wave, keep it directly in the raw inventory here instead of pushing ownership into another planning file.

### Raw Inventory Appendix -> Literal-Backed Contracts

This table exists to keep real protocol and cryptographic labels separate from
pure naming cleanup.

Required rule for literal-backed execution:

1. Any row classified as `literal contract` is frozen by default.
2. Do not rename a literal-backed contract inside a naming-only cleanup wave.
3. If a later phase wants to change one of these values, that later phase must
   be treated as a protocol or signature-domain migration, not as a cosmetic
   rename.

| # | Literal translation field | Task meaning | Required translation rule |
| --- | --- | --- | --- |
| 1 | `#` | raw row ID | Keep path-specific literal sites separate even when the string text is identical. |
| 2 | `Signature` | exact literal or error string | Preserve exact spelling, punctuation, and placeholder format in task evidence. |
| 3 | `Type` | literal kind | Distinguish const literal, RPC string literal, and error literal. |
| 4 | `Path` | exact file boundary | One literal row equals one path-specific execution item. |
| 5 | `Current classification` | contract state | Copy literally so later migration waves know whether the row is a live contract or compatibility error. |
| 6 | `Action now` | current-wave action | All current rows here stay `hold`. |
| 7 | `Future survivor target` | future value label | Use only when a later migration intentionally changes the contract. |
| 8 | `Notes` | migration constraint | Preserve the reason why this literal is frozen in the current wave. |

### Raw Inventory Appendix -> Local And Test-Only Residue

This table exists to capture the cleanup-only residue that the user explicitly
called out: local bindings and fixture strings that still carry needless
version markers.

Required rule for local/test execution:

1. These rows must follow production rename decisions, not lead them.
2. Local and test-only rows may be renamed aggressively only when the row does
   not intentionally model a legacy or V2 lane.
3. String-literal fixture rows are cleanup-only unless the note says they are a
   real protocol contract.

| # | Local/test translation field | Task meaning | Required translation rule |
| --- | --- | --- | --- |
| 1 | `#` | raw row ID | Keep the exact local/test row ID so cleanup tasks stay auditable. |
| 2 | `Signature` | exact local name | Preserve whether the row is a local binding, helper, test constant, or fixture string. |
| 3 | `Type` | residue kind | Use to separate rename-safe locals from intentionally explicit scenario helpers. |
| 4 | `Path` | exact file boundary | Do not merge residue across files. |
| 5 | `Current classification` | scenario meaning | Respect `test residue with explicit legacy meaning` and similar classifications literally. |
| 6 | `Action now` | current-wave action | Apply `rename now` only after the linked production rows are already stable. |
| 7 | `Future survivor target` | simplified end-state | Use as the replacement local name or fixture text. |
| 8 | `Notes` | cleanup constraint | Preserve why the row stays explicit or why it is safe to simplify. |

### Execution Step Completion Rules

1. `Step 0` is complete only when every frozen non-test row `1-18`, `26`, `29-35`, `48-55` and literal row `1-25` has an explicit no-rename justification that still matches live code.
2. `Step 1` is complete only when rows `20-21`, `24-25`, `27` are linked to concrete compatibility lanes and no rename task is emitted for them in this wave.
3. `Step 2` is complete only when rows `22-23`, `28` are renamed in code, call sites are updated, and no public RPC or transport literal changed.
4. `Step 3` is complete only when rows `36-43`, `45-47`, `52` are renamed at the Rust-symbol layer while their encoded bytes, literal strings, and persisted values remain byte-for-byte unchanged.
5. `Step 4` is complete only when rows `19`, `44` are explicitly marked blocked-by-contract and no speculative rename is emitted for them.
6. `Step 5` is complete only when local/test rows `1-92` are rechecked against final production names so cleanup does not invent semantics that production rows do not have.
7. The whole file is complete only when every table row still has a `#` ID, every disputed row group resolves through the raw inventory, and no second task-generation layer is reintroduced.

### Pitfall Guards

Do not violate these rules when creating tasks:

1. Do not split ownership of live embedded-versioning rows across multiple planning files.
1. Do not treat explicit schema-version values as the same problem as noisy type names.
1. Do not rename cryptographic or transport literals in a cosmetic cleanup wave.
1. Do not collapse coexisting V1/V2 production lanes just because the enum or type already mentions “version”.
1. Do not merge multiple path-specific local bindings into one row if the code meaning differs.
1. Do not let local variable cleanup redefine protocol semantics.

### Non-Negotiable Translation Rules

1. The raw inventory is the only task-generation surface.
2. Every rename or keep decision must be justified from the row itself.
3. Explicit wire/schema version discriminants stay frozen unless a later migration phase says otherwise.
4. Literal-backed cryptographic and transport contracts are hold-only in this wave.
5. Local and test-only residue should be cleaned only after production naming is stable.
