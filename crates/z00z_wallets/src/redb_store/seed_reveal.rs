use super::{
    commit_redb_write_txn_flush, decode_bincode, decode_bincode_bounded,
    decrypt_secret_record_post_unlock, encode_bincode, encrypt_secret_record,
    validate_seed_main_record, PersistWalletId, ReadableTable, SecretBytes, SecretsKind,
    SecretsRecord, SecureRngProvider, SeedMainEntropyPayload, SeedPhrase24, TimeProvider,
    WalletError, WalletResult, WalletSession, SECRETS_SEED_MAIN, SECRETS_SEED_MAIN_REVEALED_AT,
    SECRETS_TABLE, WALLET_SECRET_INVALID,
};

fn decode_seed_phrase_plaintext(plaintext: &mut SecretBytes) -> WalletResult<String> {
    let seed_phrase = if let Ok(mut payload) = decode_bincode::<SeedMainEntropyPayload>(plaintext) {
        let phrase = SeedPhrase24::from_bip39_entropy_bytes(
            &payload.entropy_bytes,
            payload.mnemonic_language.to_mnemonic_language(),
        )
        .map_err(|_| WalletError::InvalidConfig(WALLET_SECRET_INVALID.to_string()))?
        .with_phrase(|phrase| phrase.to_string());

        payload.entropy_bytes.fill(0);
        phrase
    } else {
        match std::str::from_utf8(plaintext) {
            Ok(s) => s.to_string(),
            Err(_) => {
                plaintext.wipe();
                return Err(WalletError::InvalidConfig(
                    WALLET_SECRET_INVALID.to_string(),
                ));
            }
        }
    };

    plaintext.wipe();
    Ok(seed_phrase)
}

fn load_seed_record(secrets: &mut redb::Table<'_, &str, &[u8]>) -> WalletResult<SecretsRecord> {
    let seed_record_raw = secrets
        .get(SECRETS_SEED_MAIN)
        .map_err(|e| WalletError::InvalidConfig(format!("redb secrets read failed: {e}")))?
        .ok_or_else(|| WalletError::InvalidConfig("missing secrets.seed_main".to_string()))?
        .value()
        .to_vec();
    let seed_record: SecretsRecord = decode_bincode_bounded(
        &seed_record_raw,
        WalletError::InvalidConfig(WALLET_SECRET_INVALID.to_string()),
    )?;

    validate_seed_main_record(
        &seed_record,
        WalletError::InvalidConfig(WALLET_SECRET_INVALID.to_string()),
    )?;

    Ok(seed_record)
}

fn load_seed_phrase(
    secrets: &mut redb::Table<'_, &str, &[u8]>,
    session: &WalletSession,
) -> WalletResult<String> {
    let seed_record = load_seed_record(secrets)?;

    let mut plaintext = decrypt_secret_record_post_unlock(
        &session.opened.wallet_id,
        SECRETS_SEED_MAIN,
        session.opened.master_key.reveal(),
        &seed_record,
    )?;

    decode_seed_phrase_plaintext(&mut plaintext)
}

pub(super) fn store_seed_revealed_at_secret(
    secrets: &mut redb::Table<'_, &str, &[u8]>,
    rng: &mut impl rand::RngCore,
    wallet_id: &PersistWalletId,
    master_key: &[u8; 32],
    revealed_at_ms: u64,
) -> WalletResult<()> {
    let plaintext = encode_bincode(&revealed_at_ms)?;

    let envelope = encrypt_secret_record(
        rng,
        wallet_id,
        SECRETS_SEED_MAIN_REVEALED_AT,
        master_key,
        &plaintext,
    )?;

    let record = SecretsRecord {
        kind: SecretsKind::Custom,
        label: "seed_main_revealed_at".to_string(),
        version: 1,
        envelope,
    };

    secrets
        .insert(
            SECRETS_SEED_MAIN_REVEALED_AT,
            encode_bincode(&record)?.as_slice(),
        )
        .map_err(|e| WalletError::InvalidConfig(format!("redb secrets insert failed: {e}")))?;

    Ok(())
}

pub fn reveal_seed_phrase_once<R: SecureRngProvider>(
    session: &WalletSession,
    rng_provider: R,
    time_provider: &dyn TimeProvider,
) -> WalletResult<String> {
    let revealed_at_ms = time_provider.compat_unix_timestamp_millis();
    let write_txn = session
        .db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    let seed_phrase = {
        let mut secrets = write_txn
            .open_table(SECRETS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open secrets failed: {e}")))?;

        if secrets
            .get(SECRETS_SEED_MAIN_REVEALED_AT)
            .map_err(|e| WalletError::InvalidConfig(format!("redb secrets read failed: {e}")))?
            .is_some()
        {
            return Err(WalletError::InvalidParams(
                "seed phrase can only be shown once".to_string(),
            ));
        }

        let seed_phrase = load_seed_phrase(&mut secrets, session)?;

        let mut rng = rng_provider.rng();
        store_seed_revealed_at_secret(
            &mut secrets,
            &mut rng,
            &session.opened.wallet_id,
            session.opened.master_key.reveal(),
            revealed_at_ms,
        )?;

        seed_phrase
    };

    commit_redb_write_txn_flush(session, write_txn)?;

    Ok(seed_phrase)
}

pub fn reveal_seed_phrase<R: SecureRngProvider>(
    session: &WalletSession,
    rng_provider: R,
    time_provider: &dyn TimeProvider,
) -> WalletResult<String> {
    let revealed_at_ms = time_provider.compat_unix_timestamp_millis();
    let write_txn = session
        .db
        .begin_write()
        .map_err(|e| WalletError::InvalidConfig(format!("redb begin_write failed: {e}")))?;

    let seed_phrase = {
        let mut secrets = write_txn
            .open_table(SECRETS_TABLE)
            .map_err(|e| WalletError::InvalidConfig(format!("redb open secrets failed: {e}")))?;

        let should_write_marker = secrets
            .get(SECRETS_SEED_MAIN_REVEALED_AT)
            .map_err(|e| WalletError::InvalidConfig(format!("redb secrets read failed: {e}")))?
            .is_none();

        let seed_phrase = load_seed_phrase(&mut secrets, session)?;

        if should_write_marker {
            let mut rng = rng_provider.rng();
            store_seed_revealed_at_secret(
                &mut secrets,
                &mut rng,
                &session.opened.wallet_id,
                session.opened.master_key.reveal(),
                revealed_at_ms,
            )?;
        }

        seed_phrase
    };

    commit_redb_write_txn_flush(session, write_txn)?;

    Ok(seed_phrase)
}
