# 036 A6 AS Import Alias Spec

This file records the Rust import-alias audit for Phase 036.
The workspace scan found 96 `use` / `pub use` lines with `as` in `crates/**/*.rs`.

## Decision Summary

Do **not** blanket-rename all import aliases.
Most of the aliases are intentional and should stay because they carry semantic
meaning, avoid type collisions, or follow the standard trait-import pattern `as _`.

The only aliases that look weak enough to change are the short shorthand names in
`test_phase11_derivation.rs`, plus one local helper alias in `wallet_io.rs` that
can be dropped in favor of the original module name.

### Category Counts

- 34 trait-import lines using `as _` are kept as-is.
- 3 `pub use` re-export lines are kept as-is.
- 57 semantic alias lines are kept as-is.
- 2 shorthand alias lines are flagged for rename review.

## Rename Table

| # | File | Line(s) | Old | New / Action | Notes | Comments |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `crates/z00z_wallets/src/db/wallet_io.rs` | 3 | `use z00z_utils::io as zio;` | `use z00z_utils::io;` | Drop the alias and use the original module name. | `zio` is a non-standard shorthand here; the file does not need an extra alias to avoid a collision, so the direct module name is clearer. |
| 2 | `crates/z00z_wallets/tests/test_phase11_derivation.rs` | 4 | `ecdh as cecdh` | `ecdh as crypto_ecdh` | Keep the alias, but expand it to the established crypto naming style. | `crypto_ecdh` matches the existing `crypto_kdf` pattern and is easier to audit than `cecdh`. |
| 3 | `crates/z00z_wallets/tests/test_phase11_derivation.rs` | 4 | `kdf as ckdf` | `kdf as crypto_kdf` | Keep the alias, but expand it to the established crypto naming style. | `crypto_kdf` is already used elsewhere in the workspace, so this keeps the test aligned with the existing semantic alias convention. |

## Coverage Proof

Evidence scan command:

```bash
rg -n --glob '*.rs' '^\s*(pub\s+)?use\s+.*\s+as\s+.*;' crates
```

Raw alias inventory captured from the scan:

```text
crates/z00z_core/bin/assets/assets_generation_cli.rs:28:use z00z_crypto::{create_commitment, DomainHasher, Z00ZScalar as BlindingFactor};
crates/z00z_core/src/assets/definition_validate.rs:3:use super::super::policy_flags::validate_flags as flags_are_valid;
crates/z00z_core/src/assets/test_asset_suite.rs:2:use crate::assets::nonce::derive_nonce_minimal as derive_nonce_minimal_safe;
crates/z00z_core/src/assets/test_wire_phase26.rs:2:use crate::assets::nonce::derive_nonce_minimal as derive_nonce_minimal_safe;
crates/z00z_core/src/assets/test_wire.rs:3:use crate::assets::nonce::derive_nonce_minimal as derive_nonce_minimal_safe;
crates/z00z_core/tests/assets/test_integration_crypto.rs:9:use z00z_core::assets::nonce::derive_nonce_simple as derive_nonce_simple_safe;
crates/z00z_core/tests/assets/test_integration_owner_signature_security.rs:12:use z00z_core::assets::nonce::derive_nonce_minimal as derive_nonce_minimal_safe;
crates/z00z_core/tests/assets/test_property_based.rs:30:use proptest::test_runner::Config as ProptestConfig;
crates/z00z_crypto/src/expert.rs:27:    pub use tari_crypto::keys::{PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
crates/z00z_crypto/src/sha256_hash.rs:1:use sha2::{Digest as _, Sha256};
crates/z00z_crypto/src/test_types_suite.rs:294:    use tari_crypto::keys::PublicKey as _;
crates/z00z_crypto/src/types.rs:55:        use tari_crypto::keys::PublicKey as _;
crates/z00z_crypto/src/types.rs:94:        use tari_crypto::keys::PublicKey as _;
crates/z00z_crypto/src/types.rs:100:        use tari_crypto::keys::PublicKey as _;
crates/z00z_crypto/src/types.rs:285:        use tari_crypto::tari_utilities::ByteArray as _;
crates/z00z_crypto/tests/test_claim_contract.rs:2:use tari_crypto::keys::SecretKey as _;
crates/z00z_networks/rpc/src/wasm_client.rs:15:use z00z_utils::codec::Value as JsonValue;
crates/z00z_simulator/tests/test_stage4_wallet_persist.rs:183:    use std::io::Read as _;
crates/z00z_storage/src/assets/leaf.rs:68:    use z00z_core::assets::{AssetLeaf as CoreLeaf, AssetPackPlain};
crates/z00z_storage/src/assets/store_internal/test_whitebox_help.rs:3:use z00z_core::assets::{AssetLeaf as CoreLeaf, AssetPackPlain};
crates/z00z_storage/src/assets/test_model.rs:11:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/src/checkpoint/codec.rs:179:    use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/src/checkpoint/exec_input.rs:275:    use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/src/serialization/build_temp_tree.rs:6:use anyhow::Result as AnyResult;
crates/z00z_storage/src/serialization/store.rs:115:        use std::fmt::Write as _;
crates/z00z_storage/src/serialization/view.rs:1:use std::fmt::Write as _;
crates/z00z_storage/src/snapshot/store.rs:307:        use std::fmt::Write as _;
crates/z00z_storage/src/snapshot/test_store_suite.rs:2:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/assets/test_store_api.rs:1:use z00z_core::assets::{AssetLeaf as CoreLeaf, AssetPackPlain};
crates/z00z_storage/tests/checkpoint/test_fixtures.rs:4:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/snapshot/test_fix.rs:125:        use std::fmt::Write as _;
crates/z00z_storage/tests/snapshot/test_fix.rs:4:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_checkpoint_draft_build.rs:6:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_checkpoint_leaf_hash.rs:4:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_checkpoint_link_injective.rs:2:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_checkpoint_replay_inputs.rs:4:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_checkpoint_root_binding.rs:4:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_checkpoint_store_api.rs:5:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_redb_mutation.rs:5:use z00z_core::assets::{AssetLeaf as CoreLeaf, AssetPackPlain};
crates/z00z_storage/tests/test_redb_rehydrate.rs:6:use z00z_core::assets::{AssetLeaf as CoreLeaf, AssetPackPlain};
crates/z00z_storage/tests/test_search_api.rs:4:use z00z_core::assets::{AssetLeaf as CoreLeaf, AssetPackPlain};
crates/z00z_storage/tests/test_serialization_determinism.rs:1:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_serialization_restore.rs:1:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_serialization_roundtrip.rs:2:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_storage/tests/test_serialization_visualization.rs:1:use z00z_core::assets::AssetLeaf as CoreLeaf;
crates/z00z_utils/src/config/mod.rs:34:pub use serde_yml::Value as YamlValue;
crates/z00z_utils/src/io/test_fs_io_suite.rs:15:    use std::os::unix::fs as unix_fs;
crates/z00z_utils/src/logger/file_logger.rs:230:        use std::os::unix::fs as unix_fs;
crates/z00z_utils/src/logger/mod.rs:28:    use std::fmt::Write as _;
crates/z00z_utils/src/logger/rotating_file_logger.rs:326:        use std::os::unix::fs as unix_fs;
crates/z00z_utils/src/rng/deterministic.rs:14:use super::traits::DeterministicRngProvider as DeterministicRng;
crates/z00z_utils/src/rng/mock.rs:11:use super::traits::DeterministicRngProvider as DeterministicRng;
crates/z00z_utils/src/rng/mod.rs:42:pub use traits::{DeterministicRngProvider as DeterministicRng, SecureRngProvider};
crates/z00z_utils/src/time/format.rs:11:use chrono::{Local, TimeZone as _, Utc};
crates/z00z_utils/tests/test_io_integration.rs:369:        use std::io::Write as _;
crates/z00z_wallets/src/adapters/rpc/app_dispatcher_wiring.rs:12:use crate::adapters::rpc::methods::chain::ChainScanRpc as _;
crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs:580:use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs:274:    use base64::Engine as _;
crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs:580:use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
crates/z00z_wallets/src/adapters/rpc/methods/test_wallet_impl_suite.rs:9:use base64::Engine as _;
crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs:9:use crate::adapters::rpc::methods::storage::StorageRpc as _;
crates/z00z_wallets/src/core/address/address_manager.rs:67:use tokio::sync::RwLock as TokioRwLock;
crates/z00z_wallets/src/core/address/address_manager.rs:70:use z00z_crypto::{hash::hmac_sha256, hkdf_expand_32, Z00ZRistrettoPoint as PublicKey};
crates/z00z_wallets/src/core/address/stealth_card.rs:1:use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
crates/z00z_wallets/src/core/address/stealth_request.rs:1:use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
crates/z00z_wallets/src/core/address/z00z_address.rs:34:use z00z_crypto::{frame_bytes, DomainHasher256, Z00ZRistrettoPoint as PublicKey};
crates/z00z_wallets/src/core/chain/receiver_card_record.rs:1:use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
crates/z00z_wallets/src/core/key/key_manager.rs:85:use crate::core::domains::WalletSignNonceProdDomain as SignNonceDomain;
crates/z00z_wallets/src/core/key/key_manager.rs:87:use crate::core::domains::WalletSignNonceTestDomain as SignNonceDomain;
crates/z00z_wallets/src/core/storage/claim_registry.rs:4:use std::sync::RwLock as StdRwLock;
crates/z00z_wallets/src/core/tx/claim_tx.rs:36:use z00z_storage::assets::{chk_blob, AssetLeaf as StoreLeaf, ProofBlob};
crates/z00z_wallets/src/core/tx/prover.rs:7:use z00z_crypto::Z00ZCommitment as Commitment;
crates/z00z_wallets/src/core/tx/state_checkpoint.rs:4:use z00z_storage::checkpoint::{CreatedEnt as CpCreatedEnt, SpentEnt as CpSpentEnt, WalletDraft};
crates/z00z_wallets/src/core/wallet/mod.rs:89:        use crate::db::wallet_store::{RedbWalletStore, WltStore as WalletStoreTrait};
crates/z00z_wallets/src/core/wallet/wallet_identity.rs:13:    use serde::de::Error as _;
crates/z00z_wallets/src/db/redb_wallet_store_debug_types.rs:5:use serde_json::Value as DebugJsonValue;
crates/z00z_wallets/src/db/redb_wallet_store.rs:33:use base64::Engine as _;
crates/z00z_wallets/src/db/redb_wallet_store_session.rs:62:    use fs2::FileExt as _;
crates/z00z_wallets/src/db/redb_wallet_store_session.rs:63:    use std::io::{Seek as _, SeekFrom, Write as _};
crates/z00z_wallets/src/db/tests/redb_wallet_store.rs:3226:    use rand::{CryptoRng, Error as RandError, RngCore};
crates/z00z_wallets/src/db/wallet_io.rs:3:use z00z_utils::io as zio;
crates/z00z_wallets/src/services/session_service.rs:8:use crate::db::WalletSession as DbWalletSession;
crates/z00z_wallets/src/services/wallet_service_actions_hardening.rs:30:        use fs2::FileExt as _;
crates/z00z_wallets/src/services/wallet_service_actions_runtime.rs:31:        use fs2::FileExt as _;
crates/z00z_wallets/src/services/wallet_service.rs:75:use base64::Engine as _;
crates/z00z_wallets/src/services/wallet_service_store_transfer_import.rs:28:            use fs2::FileExt as _;
crates/z00z_wallets/src/services/wallet_service_types.rs:63:use base64::Engine as _;
crates/z00z_wallets/src/wasm/indexeddb_backend.rs:4:use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
crates/z00z_wallets/tests/test_phase11_derivation.rs:4:use z00z_crypto::{ecdh as cecdh, hash_to_scalar_domain, kdf as ckdf, Z00ZRistrettoPoint};
crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs:12:use chrono::{TimeZone as _, Utc};
crates/z00z_wallets/tests/test_rpc_logging_configured_path.rs:13:use chrono::{TimeZone as _, Utc};
crates/z00z_wallets/tests/test_rpc_logging_file_sink.rs:22:use chrono::{TimeZone as _, Utc};
crates/z00z_wallets/tests/test_seed_salt_policy.rs:5:use base64::Engine as _;
crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs:6:use base64::Engine as _;
crates/z00z_wallets/tests/test_stealth_kdf_vectors.rs:4:use z00z_crypto::kdf as crypto_kdf;
crates/z00z_wallets/tests/test_stub_behavior.rs:6:use base64::Engine as _;
```

## Notes

- Trait imports written as `as _` are standard Rust and should stay.
- Public `pub use` aliases are part of the crate facade and should stay unless the public API changes.
- Semantic aliases that map upstream types into domain names are intentional and should stay.
- The only aliases that merit rename treatment in this scan are the short shorthand names noted in the rename table.
