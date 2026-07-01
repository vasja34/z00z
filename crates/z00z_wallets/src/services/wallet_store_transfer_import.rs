impl WalletService {
    /// Delete wallet state from memory and on-disk persistence after password verification.
    pub async fn delete_wallet_data(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
    ) -> WalletResult<()> {
        // Verify password
        let verifiers = self.wallet_password_verifiers.read().await;
        let verifier_state = verifiers
            .get(wallet_id)
            .copied()
            .ok_or(WalletError::NotFound(0))?;
        drop(verifiers);

        let computed = Self::compute_password_verifier(password, &verifier_state.salt);
        if !Self::ct_cmp_32(&computed, &verifier_state.verifier) {
            return Err(WalletError::InvalidPassword);
        }

        // Ensure any in-memory session is dropped before deleting persistence artifacts.
        // This releases the `.wlt.lock` guard held by an unlocked wallet.
        self.lock_wallet(wallet_id).await?;

        // Delete persistence artifacts first (then remove in-memory state).
        #[cfg(not(target_arch = "wasm32"))]
        {
            use fs2::FileExt as _;
            use z00z_utils::io::{read_dir, remove_file, File};

            let wlt_path = self.wlt_file_path(wallet_id);

            let lock_path = {
                let mut os = wlt_path.as_os_str().to_os_string();
                os.push(".lock");
                PathBuf::from(os)
            };

            let tmp_path = {
                let mut os = wlt_path.as_os_str().to_os_string();
                os.push(".tmp");
                PathBuf::from(os)
            };

            // Refuse deletion while another process holds the wallet lock.
            let lock_file = File::options()
                .create(true)
                .read(true)
                .write(true)
                .truncate(false)
                .open(&lock_path)
                .map_err(|_| WalletError::WalletInUse)?;

            if lock_file.try_lock_exclusive().is_err() {
                return Err(WalletError::WalletInUse);
            }

            if wlt_path.exists() {
                remove_file(&wlt_path).map_err(|e| {
                    WalletError::InvalidConfig(format!(
                        "Failed to delete .wlt file {}: {}",
                        wlt_path.display(),
                        e
                    ))
                })?;
            }

            if tmp_path.exists() {
                remove_file(&tmp_path).map_err(|e| {
                    WalletError::InvalidConfig(format!(
                        "Failed to delete .wlt temp file {}: {}",
                        tmp_path.display(),
                        e
                    ))
                })?;
            }

            // Best-effort cleanup of tmpfs work files.
            let shm_dir = Path::new("/dev/shm");
            if let (Some(file_name), Ok(entries)) = (wlt_path.file_name(), read_dir(shm_dir)) {
                let prefix = format!("{}{}", file_name.to_string_lossy(), ".work.");
                for entry in entries {
                    let matches = entry
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|n| n.starts_with(&prefix))
                        .unwrap_or(false);
                    if matches {
                        let _ = remove_file(&entry);
                    }
                }
            }

            drop(lock_file);
            // Best-effort remove the lock file itself.
            if lock_path.exists() {
                let _ = remove_file(&lock_path);
            }
        }

        // Remove from all in-memory stores.
        {
            let mut names = self.wallet_names.write().await;
            names.remove(wallet_id);
        }

        {
            let mut states = self.wallet_states.write().await;
            states.remove(wallet_id);
        }

        {
            let mut verifiers = self.wallet_password_verifiers.write().await;
            verifiers.remove(wallet_id);
        }

        {
            let mut salts = self.wallet_seed_salts.write().await;
            salts.remove(wallet_id);
        }

        {
            let mut derivers = self.wallet_receiver_derivers.write().await;
            derivers.remove(wallet_id);
        }

        {
            let mut counters = self.wallet_receiver_deriver_counters.write().await;
            counters.remove(wallet_id);
        }

        {
            let mut settings = self.wallet_settings.write().await;
            settings.remove(wallet_id);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Defensive: even if callers bypassed explicit locking, do not retain a session.
            self.wallet_sessions.drop_session(wallet_id).await;
        }
        Ok(())
    }

    /// Export wallet as encrypted JSON
    ///
    /// This transfer surface carries the canonical wallet-state pack only. It does not embed
    /// JSONL tx-history sidecar bytes.
    ///
    /// # Arguments
    /// * `wallet_id` - ID of wallet to export
    /// * `password` - Password used to encrypt the canonical export payload
    ///
    /// # Returns
    /// - `Ok(EncryptedResponse)` - Wallet data as JSON
    /// - `Err(WalletError)` - If export fails
    pub async fn export_wallet_payload(
        &self,
        wallet_id: &PersistWalletId,
        password: &SafePassword,
    ) -> WalletResult<RuntimeEncryptedResponse> {
        use crate::security::encryption::{EncryptedWalletContainer, WalletEncryption};
        use z00z_utils::codec::{BincodeCodec, Codec};

        let export_pack = self.build_wallet_export_pack(wallet_id, password).await?;

        let codec = BincodeCodec;
        let plaintext = z00z_crypto::Hidden::hide(codec.serialize(&export_pack).map_err(|e| {
            WalletError::InvalidConfig(format!("Export serialization failed: {e}"))
        })?);

        let aad = Self::wallet_export_aad()?;
        let encrypted_container =
            WalletEncryption::encrypt_wallet(password, &aad, plaintext.reveal().as_slice())?;

        let container_bytes = codec.serialize(&encrypted_container).map_err(|e| {
            WalletError::InvalidConfig(format!("Export container serialization failed: {e}"))
        })?;

        let payload_bytes =
            Self::encode_wallet_export_payload(EncryptedWalletContainer::VERSION, &container_bytes);

        drop(plaintext);

        let ciphertext = base64::engine::general_purpose::STANDARD.encode(payload_bytes);

        let nonce_hex = if encrypted_container.envelope.len() >= 24 {
            format!("0x{}", hex::encode(&encrypted_container.envelope[..24]))
        } else {
            String::new()
        };

        Ok(RuntimeEncryptedResponse {
            ciphertext,
            metadata: RuntimeEncryptionMetadata {
                algorithm: encrypted_container.algorithm,
                nonce: nonce_hex,
                key_derivation: "Argon2id".to_string(),
            },
        })
    }

    /// Import wallet from JSON data
    ///
    /// This transfer surface restores only the canonical wallet-state pack. Full backup restore
    /// is the surface that replays JSONL tx-history sidecar bytes.
    ///
    /// # Arguments
    /// * `data` - JSON string containing an encrypted wallet export payload
    /// * `password` - Password used to decrypt the canonical export payload
    /// * `name` - New name for imported wallet
    ///
    /// # Returns
    /// - `Ok(PersistWalletId)` - ID of imported wallet
    /// - `Err(WalletError)` - If import fails
    pub async fn import_wallet_payload(
        &self,
        data: &str,
        password: &SafePassword,
        name: &str,
    ) -> WalletResult<PersistWalletId> {
        use crate::security::encryption::{EncryptedWalletContainer, WalletEncryption};
        use z00z_utils::codec::{BincodeCodec, Codec, JsonCodec};

        let trimmed = data.trim();
        if trimmed.is_empty() {
            return Err(WalletError::InvalidParams(
                "Wallet import data is required".to_string(),
            ));
        }

        // Canonical format: EncryptedResponse JSON with
        // ciphertext=base64(magic || version || bincode(EncryptedWalletContainer)).
        let json_codec = JsonCodec;
        let encrypted = json_codec
            .deserialize::<RuntimeEncryptedResponse>(trimmed.as_bytes())
            .map_err(|_| WalletError::InvalidParams("Invalid wallet export payload".to_string()))?;

        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(encrypted.ciphertext.as_bytes())
            .map_err(|_| {
                WalletError::InvalidParams("Invalid backup payload encoding".to_string())
            })?;

        let bin_codec = BincodeCodec;
        let aad = Self::wallet_export_aad()?;

        let (version, inner_bytes) = Self::decode_wallet_export_payload(&payload_bytes)
            .ok_or_else(|| {
                WalletError::InvalidParams("Invalid backup payload format".to_string())
            })?;

        if version != EncryptedWalletContainer::VERSION {
            return Err(WalletError::InvalidParams(format!(
                "Unsupported backup payload version: {}",
                version
            )));
        }

        let decoded_container = bin_codec
            .deserialize::<EncryptedWalletContainer>(inner_bytes)
            .map_err(|_| WalletError::InvalidParams("Invalid backup format".to_string()))?;

        let plaintext = WalletEncryption::decrypt_wallet(password, &aad, &decoded_container)?;
        let export_pack = bin_codec
            .deserialize::<WalletExportPack>(plaintext.as_slice())
            .map_err(|_| WalletError::InvalidParams("Invalid backup payload".to_string()))?;
        drop(plaintext);

        let default_identity = crate::services::wallet_runtime_config::resolve_wallet_identity_checked()?;
        let identity = export_pack
            .wallet_identity
            .as_ref()
            .map(|wallet_identity| WalletIdentity {
                network: if wallet_identity.network.trim().is_empty() {
                    default_identity.network.clone()
                } else {
                    wallet_identity.network.clone()
                },
                chain: wallet_identity.chain.clone(),
            })
            .unwrap_or(default_identity);

        self.restore_wallet_export_pack(export_pack, password, Some(name), &identity)
            .await
    }
}
