impl<T: TimeProvider, R: SecureRngProvider> BackupExporterImpl<T, R> {
    fn compute_checksum(aad_bytes: &[u8], ciphertext: &[u8]) -> [u8; 32] {
        WalletBackupCrypto::checksum(aad_bytes, ciphertext)
    }

    fn resolve_aad_bytes(container: &BackupContainer) -> BackupExporterResult<Option<Vec<u8>>> {
        let aad = BackupAssociatedData {
            metadata: container.metadata.clone(),
            encryption: container.encryption.clone(),
            compression: container.compression.clone(),
        };

        let aad_bytes = Self::build_aad_bytes(&aad)?;
        if Self::compute_checksum(&aad_bytes, &container.ciphertext) == container.checksum {
            return Ok(Some(aad_bytes));
        };

        Ok(None)
    }

    fn verify_container_integrity(&self, container: &BackupContainer) -> bool {
        matches!(Self::resolve_aad_bytes(container), Ok(Some(_)))
    }

    fn verify_export_pack(
        container: &BackupContainer,
        payload: &BackupPayload,
        expected_network: &str,
        expected_chain: &str,
    ) -> BackupExporterResult<bool> {
        let export_pack = &payload.export_pack;
        if export_pack.seed_phrase.trim().is_empty() {
            return Ok(false);
        }

        let Some(profile) = export_pack.wallet_profile.as_ref() else {
            return Ok(false);
        };
        let profile = match profile.clone().migrate_to_current() {
            Ok(profile) => profile,
            Err(_) => return Ok(false),
        };
        if profile.verify_checksum().is_err() {
            return Ok(false);
        }
        let Some(manifest) = export_pack.manifest.as_ref() else {
            return Ok(false);
        };
        if manifest.verify_checksum().is_err() {
            return Ok(false);
        }
        if export_pack.tx_history_plane.as_deref()
            != Some(crate::db::BackupManifestPayload::TX_HISTORY_JSONL)
        {
            return Ok(false);
        }
        if manifest.wallet_id != profile.wallet_id
            || manifest.network != expected_network
            || manifest.chain != expected_chain
            || manifest.profile_count != 1
            || manifest.owned_asset_count != export_pack.owned_assets.len() as u32
            || manifest.owned_object_count != export_pack.owned_objects.len() as u32
            || manifest.scan_state_count != u32::from(export_pack.scan_state.is_some())
            || manifest.stealth_meta_count != u32::from(export_pack.stealth_meta.is_some())
            || manifest.tofu_pins_count != u32::from(export_pack.tofu_pins.is_some())
            || manifest.key_ref_count
                != export_pack
                    .keys
                    .as_ref()
                    .map(|value| value.signing_keys.len() as u32)
                    .unwrap_or(0)
            || !manifest.has_tx_history_sidecar
            || manifest.tx_history_plane != crate::db::BackupManifestPayload::TX_HISTORY_JSONL
        {
            return Ok(false);
        }

        let mut seen_asset_ids = std::collections::BTreeSet::new();
        for payload in &export_pack.owned_assets {
            let migrated = match payload.clone().migrate_to_current() {
                Ok(value) => value,
                Err(_) => return Ok(false),
            };
            if migrated.verify_checksum().is_err() || migrated.validate_invariants().is_err() {
                return Ok(false);
            }
            if migrated.wallet_id != profile.wallet_id || !seen_asset_ids.insert(migrated.asset_id)
            {
                return Ok(false);
            }
        }

        let mut seen_object_keys = std::collections::BTreeSet::new();
        for payload in &export_pack.owned_objects {
            let migrated = match payload.clone() {
                crate::db::OwnedObjectPayload::Asset(_) => return Ok(false),
                crate::db::OwnedObjectPayload::Voucher(payload) => {
                    let payload = match payload.migrate_to_current() {
                        Ok(value) => value,
                        Err(_) => return Ok(false),
                    };
                    if payload.verify_checksum().is_err()
                        || payload.validate_invariants().is_err()
                        || payload.wallet_id != profile.wallet_id
                    {
                        return Ok(false);
                    }
                    crate::db::OwnedObjectPayload::Voucher(payload)
                }
                crate::db::OwnedObjectPayload::Right(payload) => {
                    let payload = match payload.migrate_to_current() {
                        Ok(value) => value,
                        Err(_) => return Ok(false),
                    };
                    if payload.verify_checksum().is_err()
                        || payload.validate_invariants().is_err()
                        || payload.wallet_id != profile.wallet_id
                    {
                        return Ok(false);
                    }
                    crate::db::OwnedObjectPayload::Right(payload)
                }
            };

            let family_tag = match migrated.family() {
                crate::db::OwnedObjectFamily::Asset => 1u8,
                crate::db::OwnedObjectFamily::Voucher => 2u8,
                crate::db::OwnedObjectFamily::Right => 3u8,
            };
            if !seen_object_keys.insert((family_tag, migrated.stable_object_key())) {
                return Ok(false);
            }
        }

        let wallet_id_matches = profile.wallet_id.0 == container.metadata.wallet_id;

        if let Some(forensic) = payload.forensic.as_ref() {
            if forensic
                .validate(&container.metadata, expected_network, expected_chain)
                .is_err()
            {
                return Ok(false);
            }
        }

        Ok(wallet_id_matches
            && payload.network == expected_network
            && payload.chain == expected_chain
            && !export_pack.seed_phrase.trim().is_empty())
    }

    fn verify_payload_matches_metadata(
        container: &BackupContainer,
        plaintext: &[u8],
        expected_network: &str,
        expected_chain: &str,
    ) -> BackupExporterResult<bool> {
        match container.metadata.version {
            BACKUP_FORMAT_VERSION => {
                let payload: BackupPayload = JsonCodec
                    .deserialize(plaintext)
                    .map_err(|e| BackupExporterError::InvalidFormat(e.to_string()))?;

                Self::verify_export_pack(container, &payload, expected_network, expected_chain)
            }
            _ => Ok(false),
        }
    }

    fn decrypt_payload(
        container: &BackupContainer,
        password: &SafePassword,
    ) -> BackupExporterResult<Option<Vec<u8>>> {
        let kdf = Self::resolve_kdf(&container.encryption)?;
        let Some(aad_bytes) = Self::resolve_aad_bytes(container)? else {
            return Ok(None);
        };

        let key_bytes = WalletBackupCrypto::derive_key_with_kdf(password, &kdf)
            .map_err(|e| BackupExporterError::EncryptionFailed(e.to_string()))?;

        let compressed: Vec<u8> =
            match WalletBackupCrypto::decrypt(&key_bytes, &aad_bytes, &container.ciphertext) {
                Ok(bytes) => bytes,
                Err(_) => return Ok(None),
            };

        if container.compression.algorithm != "zstd" {
            return Ok(None);
        }

        match zstd_decompress_bounded(&compressed, BACKUP_MAX_PLAINTEXT_BYTES) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(_) => Ok(None),
        }
    }
}
