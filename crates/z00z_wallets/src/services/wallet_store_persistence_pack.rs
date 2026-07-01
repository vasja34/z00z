impl WalletService {
    pub(crate) fn wallet_stem(wallet_id: &PersistWalletId) -> String {
        let hash = compute_wallet_file_id(&wallet_id.0);
        hex::encode(&hash[..8])
    }

    pub(crate) fn wallet_file_name(wallet_stem: &str) -> String {
        format!("wallet_{wallet_stem}.wlt")
    }

    pub(crate) fn wallet_history_jsonl_name(wallet_stem: &str) -> String {
        format!("wallet_{wallet_stem}_tx_history.jsonl")
    }

    pub(crate) fn wlt_file_path(&self, wallet_id: &PersistWalletId) -> std::path::PathBuf {
        let wallet_stem = Self::wallet_stem(wallet_id);
        self.output_dir.join(Self::wallet_file_name(&wallet_stem))
    }

    pub(crate) fn wallet_history_jsonl_path(
        &self,
        wallet_id: &PersistWalletId,
    ) -> std::path::PathBuf {
        let wallet_stem = Self::wallet_stem(wallet_id);
        self.output_dir
            .join(Self::wallet_history_jsonl_name(&wallet_stem))
    }

    fn wallet_export_aad() -> WalletResult<Vec<u8>> {
        use z00z_crypto::expert::traits::DomainSeparation;
        let context = [Z00ZKeyBranch::WalletBackup.as_aad_byte()];
        aead::build_aad_multipart(AeadEnvelopeDomain::domain(), &[&context[..]]).map_err(|e| {
            WalletError::InvalidConfig(format!("wallet export AAD construction failed: {e}"))
        })
    }

    pub(crate) const WALLET_EXPORT_PAYLOAD_MAGIC: &'static [u8] = b"z00z-wexp\0";

    pub(crate) fn encode_wallet_export_payload(version: u32, inner: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::WALLET_EXPORT_PAYLOAD_MAGIC.len() + 4 + inner.len());
        out.extend_from_slice(Self::WALLET_EXPORT_PAYLOAD_MAGIC);
        out.extend_from_slice(&version.to_le_bytes());
        out.extend_from_slice(inner);
        out
    }

    pub(crate) fn decode_wallet_export_payload(payload: &[u8]) -> Option<(u32, &[u8])> {
        let magic = Self::WALLET_EXPORT_PAYLOAD_MAGIC;
        if payload.len() < magic.len() + 4 {
            return None;
        }
        if !payload.starts_with(magic) {
            return None;
        }

        let version_offset = magic.len();
        let mut version_bytes = [0u8; 4];
        version_bytes.copy_from_slice(&payload[version_offset..version_offset + 4]);
        let version = u32::from_le_bytes(version_bytes);

        Some((version, &payload[version_offset + 4..]))
    }

    /// Decode the wallet-owned seed salt from an encrypted export payload.
    pub fn decode_export_seed_salt(
        export: &RuntimeEncryptedResponse,
        password: &SafePassword,
    ) -> WalletResult<[u8; 16]> {
        use z00z_utils::codec::BincodeCodec;

        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(export.ciphertext.as_bytes())
            .map_err(|_| {
                WalletError::InvalidParams("Invalid backup payload encoding".to_string())
            })?;

        let aad = Self::wallet_export_aad()?;
        let (version, inner_bytes) = Self::decode_wallet_export_payload(&payload_bytes)
            .ok_or_else(|| {
                WalletError::InvalidParams("Invalid backup payload format".to_string())
            })?;

        if version != crate::security::encryption::EncryptedWalletContainer::VERSION {
            return Err(WalletError::InvalidParams(format!(
                "Unsupported backup payload version: {}",
                version
            )));
        }

        let container = BincodeCodec
            .deserialize::<crate::security::encryption::EncryptedWalletContainer>(inner_bytes)
            .map_err(|_| WalletError::InvalidParams("Invalid backup format".to_string()))?;

        let plaintext = crate::security::encryption::WalletEncryption::decrypt_wallet(
            password, &aad, &container,
        )?;

        let pack = BincodeCodec
            .deserialize::<WalletExportPack>(plaintext.as_slice())
            .map_err(|_| WalletError::InvalidParams("Invalid backup payload".to_string()))?;

        let profile = pack.wallet_profile.ok_or_else(|| {
            WalletError::InvalidConfig("Wallet export payload missing wallet profile".to_string())
        })?;

        profile.seed_salt.ok_or_else(|| {
            WalletError::InvalidConfig("Wallet export profile missing seed salt".to_string())
        })
    }

    /// Save wallet to disk using `.wlt` (RedB) persistence.
    ///
    /// The live wallet profile is stored inside the `.wlt` file as an encrypted object record.
    /// This method keeps normal non-asset saves on the profile-first `.wlt` path.
    ///
    /// # Arguments
    /// * `wallet_id` - ID of wallet to save
    /// * `password` - Password used to unlock `.wlt` and encrypt the profile object
    pub async fn save_wallet(
        &self,
        wallet_id: PersistWalletId,
        password: SafePassword,
        seed_phrase: Option<&str>,
    ) -> WalletResult<()> {
        use z00z_utils::codec::{BincodeCodec, Codec};

        let profile = self.create_profile(&wallet_id).await?;
        let codec = BincodeCodec;
        let profile_bytes = codec.serialize(&profile).map_err(|e| {
            WalletError::InvalidConfig(format!("Binary serialization failed: {}", e))
        })?;

        self.save_wallet_wlt(wallet_id, password, seed_phrase, profile_bytes)
            .await
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn save_wallet_wlt(
        &self,
        wallet_id: PersistWalletId,
        password: SafePassword,
        seed_phrase: Option<&str>,
        profile_bytes: Vec<u8>,
    ) -> WalletResult<()> {
        let wlt_path = self.wlt_file_path(&wallet_id);
        let seed_phrase = seed_phrase.map(ToString::to_string);
        let wlt_store = Arc::clone(&self.wlt_store);
        let identity = self.resolve_persisted_wallet_identity(&wallet_id).await?;
        let wallet_id_for_update = wallet_id.clone();
        let existing_session = {
            let now_ms = self.require_now_ms()?;
            let timeout_ms = self.timeout_ms();
            self.wallet_sessions
                .session_for_wallet(&wallet_id, now_ms, timeout_ms)
                .await
                .ok()
        };

        let wrote_with_existing_session = existing_session.is_some();

        tokio::task::spawn_blocking(move || {
            if !wlt_path.exists() {
                let seed_phrase = seed_phrase.ok_or_else(|| {
                    WalletError::InvalidParams(
                        "seed phrase is required to create a new .wlt file".to_string(),
                    )
                })?;

                wlt_store.create_wallet_store(
                    &wlt_path,
                    &wallet_id,
                    &password,
                    &seed_phrase,
                    &identity,
                )?;

                drop(seed_phrase);
            }

            if let Some(session) = existing_session {
                drop(password);
                session.with_wallet_session(|wlt_session| {
                    wlt_store
                        .write_wallet_profile(wlt_session, profile_bytes)
                        .map(|_| ())
                })
            } else {
                let session =
                    wlt_store.open_wallet_store(&wlt_path, &wallet_id, &password, &identity)?;

                drop(password);

                wlt_store
                    .write_wallet_profile(&session, profile_bytes)
                    .map(|_| ())
            }
        })
        .await
        .map_err(|_| WalletError::InvalidConfig(".wlt persistence task failed".to_string()))??;

        if wrote_with_existing_session {
            self.update_activity(&wallet_id_for_update).await?;
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    async fn save_wallet_wlt(
        &self,
        _wallet_id: PersistWalletId,
        _password: SafePassword,
        _seed_phrase: Option<&str>,
        _profile_bytes: Vec<u8>,
    ) -> WalletResult<()> {
        Err(WalletError::InvalidConfig(
            ".wlt persistence is not supported on wasm32".to_string(),
        ))
    }
}

include!("wallet_store_export_pack.rs");
