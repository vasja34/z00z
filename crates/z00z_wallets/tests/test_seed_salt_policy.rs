#![cfg(not(target_arch = "wasm32"))]

use std::sync::{Arc, MutexGuard};

use base64::Engine as _;
use z00z_crypto::aead;
use z00z_crypto::expert::{encoding::SafePassword, traits::DomainSeparation};
use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};
use z00z_wallets::{
    domains::{hashing::compute_seed_salt, AeadEnvelopeDomain},
    key::Z00ZKeyBranch,
    rpc::types::common::RuntimeEncryptedResponse,
    security::encryption::{EncryptedWalletContainer, WalletEncryption},
    services::{AppService, WalletService},
    wallet::persistence::WalletExportPack,
};

#[path = "test_inc/test_wallet_env_lock.inc"]
mod test_common;

const EXPORT_MAGIC: &[u8] = b"z00z-wexp\0";
const TEST_PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";

struct WalletChainEnvGuard {
    _guard: MutexGuard<'static, ()>,
    prev_network: Option<String>,
    prev_chain: Option<String>,
}

impl WalletChainEnvGuard {
    fn new(network: &str, chain: &str) -> Self {
        let guard = test_common::wallet_env_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let prev_network = std::env::var("Z00Z_WALLET_NETWORK").ok();
        let prev_chain = std::env::var("Z00Z_WALLET_CHAIN").ok();

        std::env::set_var("Z00Z_WALLET_NETWORK", network);
        std::env::set_var("Z00Z_WALLET_CHAIN", chain);

        Self {
            _guard: guard,
            prev_network,
            prev_chain,
        }
    }
}

impl Drop for WalletChainEnvGuard {
    fn drop(&mut self) {
        match &self.prev_network {
            Some(value) => std::env::set_var("Z00Z_WALLET_NETWORK", value),
            None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
        }

        match &self.prev_chain {
            Some(value) => std::env::set_var("Z00Z_WALLET_CHAIN", value),
            None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
        }
    }
}

fn export_aad() -> Vec<u8> {
    let context = [Z00ZKeyBranch::WalletBackup.as_aad_byte()];
    aead::build_aad_multipart(AeadEnvelopeDomain::domain(), &[&context[..]]).expect("export aad")
}

fn decode_export_pack(
    export: &RuntimeEncryptedResponse,
    password: &SafePassword,
) -> (u32, WalletExportPack) {
    let payload = base64::engine::general_purpose::STANDARD
        .decode(export.ciphertext.as_bytes())
        .expect("payload base64");
    assert!(
        payload.starts_with(EXPORT_MAGIC),
        "export payload must be framed"
    );

    let version_offset = EXPORT_MAGIC.len();
    let mut version_bytes = [0u8; 4];
    version_bytes.copy_from_slice(&payload[version_offset..version_offset + 4]);
    let version = u32::from_le_bytes(version_bytes);

    let container: EncryptedWalletContainer = BincodeCodec
        .deserialize(&payload[version_offset + 4..])
        .expect("container decode");
    let plaintext = WalletEncryption::decrypt_wallet(password, &export_aad(), &container)
        .expect("decrypt export");
    let pack = BincodeCodec
        .deserialize::<WalletExportPack>(plaintext.as_ref())
        .expect("export pack decode");
    (version, pack)
}

#[tokio::test]
async fn test_new_wallet_seed_salt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let service_a = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&service_a));
    let password = SafePassword::from(TEST_PASSWORD);

    let created = {
        let _env = WalletChainEnvGuard::new("p2p", "devnet");
        app.create_wallet(
            "wallet-seed-salt".to_string(),
            TEST_PASSWORD.to_string(),
            None,
        )
        .await
        .expect("create wallet")
    };

    let service_b = WalletService::with_output_dir(output_dir);
    {
        let _env = WalletChainEnvGuard::new("p2p", "devnet");
        service_b
            .load_wallet(&created.wallet_id, TEST_PASSWORD)
            .await
            .expect("load wallet")
    }

    let exported = {
        let _env = WalletChainEnvGuard::new("p2p", "devnet");
        service_b
            .export_wallet_payload(&created.wallet_id, &password)
            .await
            .expect("export wallet")
    };

    let (_version, pack) = decode_export_pack(&exported, &password);
    let seed_salt = pack
        .wallet_profile
        .as_ref()
        .and_then(|profile| profile.seed_salt)
        .expect("seed salt present");

    assert_ne!(seed_salt, compute_seed_salt(&created.wallet_id.0));
}

#[tokio::test]
async fn test_raw_profile_json_rejected() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output_dir = temp.path().join("wallets");
    let service_a = Arc::new(WalletService::with_output_dir(output_dir.clone()));
    let app = AppService::with_wallet_service(Arc::clone(&service_a));
    let password = SafePassword::from(TEST_PASSWORD);

    let created = {
        let _env = WalletChainEnvGuard::new("p2p", "devnet");
        app.create_wallet(
            "wallet-prior-seed-salt".to_string(),
            TEST_PASSWORD.to_string(),
            None,
        )
        .await
        .expect("create wallet")
    };

    let exported = {
        let _env = WalletChainEnvGuard::new("p2p", "devnet");
        service_a
            .export_wallet_payload(&created.wallet_id, &password)
            .await
            .expect("export wallet")
    };

    let (_version, pack) = decode_export_pack(&exported, &password);
    let import_json = String::from_utf8(
        JsonCodec
            .serialize(&pack.wallet_profile.clone().expect("wallet profile"))
            .expect("wallet profile json"),
    )
    .expect("json utf8");

    let service_b = WalletService::with_output_dir(temp.path().join("wallets-import"));

    let err = {
        let _env = WalletChainEnvGuard::new("p2p", "devnet");
        service_b
            .import_wallet_payload(&import_json, &password, "raw-profile")
            .await
            .expect_err("raw profile import must fail")
    };

    match err {
        z00z_wallets::WalletError::InvalidParams(msg) => {
            assert_eq!(msg, "Invalid wallet export payload");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn test_seed_source_split_contract() {
    let source = include_str!("../src/key/seed.rs");

    for part in [
        "seed_mnemonic.rs",
        "seed_cipher.rs",
        "seed_backup_format.rs",
    ] {
        let needle = format!("include!(\"{part}\");");
        assert!(
            source.contains(&needle),
            "seed.rs must keep facade include for {part}"
        );
    }
}
