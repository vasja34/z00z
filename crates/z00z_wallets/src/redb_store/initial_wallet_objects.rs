use super::{
    encode_bincode, encrypt_object_record, generate_16_bytes, generate_object_id, AccountPayload,
    AppPayload, AppPlatform, ChainPayload, DerivationStatePayload, EncryptedObjectRecord,
    KeysPayload, ObjectKindId, PersistWalletId, ScanStatePayload, StealthMetaPayload,
    TofuPinsPayload, WalletDerivedKeys, WalletIdentity, WalletResult, WalletRootPayload,
    PAYLOAD_VERSION_ACCOUNT, PAYLOAD_VERSION_APP, PAYLOAD_VERSION_CHAIN,
    PAYLOAD_VERSION_DERIVATION_STATE, PAYLOAD_VERSION_KEYS, PAYLOAD_VERSION_SCAN_STATE,
    PAYLOAD_VERSION_STEALTH_META, PAYLOAD_VERSION_TOFU_PINS, PAYLOAD_VERSION_WALLET_ROOT,
};

pub(super) fn create_initial_objects(
    rng: &mut impl rand::RngCore,
    wallet_id: &PersistWalletId,
    derived: &WalletDerivedKeys,
    identity: &WalletIdentity,
    now_ms: u64,
) -> WalletResult<InitialObjects> {
    let data_key = derived.data_key.reveal();

    let wallet_root_id = generate_object_id(rng);
    let main_account_id = generate_object_id(rng);
    let derivation_state_id = generate_object_id(rng);
    let scan_state_id = generate_object_id(rng);
    let app_id = generate_object_id(rng);
    let chain_id = generate_object_id(rng);
    let keys_id = generate_object_id(rng);
    let stealth_meta_id = generate_object_id(rng);

    let wallet_root_payload = WalletRootPayload {
        version: 1,
        main_account_id,
        created_at: now_ms,
        chain: identity.chain.clone(),
    };
    let wallet_root_record = encrypt_object_record(
        rng,
        wallet_id,
        data_key,
        wallet_root_id,
        PAYLOAD_VERSION_WALLET_ROOT,
        ObjectKindId::WalletRoot as u8,
        encode_bincode(&wallet_root_payload)?,
    )?;

    let account_payload = AccountPayload {
        account_id: main_account_id,
        parent_wallet: wallet_root_id,
        name: "Main".to_string(),
        derivation_path: "m/".to_string(),
        public_key: Vec::new(),
        created_at: now_ms,
    };
    let account_record = encrypt_object_record(
        rng,
        wallet_id,
        data_key,
        main_account_id,
        PAYLOAD_VERSION_ACCOUNT,
        ObjectKindId::Account as u8,
        encode_bincode(&account_payload)?,
    )?;

    let derivation_payload = DerivationStatePayload {
        next_account_index: 0,
        next_address_index: 0,
    };
    let derivation_record = encrypt_object_record(
        rng,
        wallet_id,
        data_key,
        derivation_state_id,
        PAYLOAD_VERSION_DERIVATION_STATE,
        ObjectKindId::DerivationState as u8,
        encode_bincode(&derivation_payload)?,
    )?;

    let scan_payload = ScanStatePayload {
        last_scanned_height: 0,
        last_scanned_hash: Vec::new(),
    };
    let scan_record = encrypt_object_record(
        rng,
        wallet_id,
        data_key,
        scan_state_id,
        PAYLOAD_VERSION_SCAN_STATE,
        ObjectKindId::ScanState as u8,
        encode_bincode(&scan_payload)?,
    )?;

    let app_payload = AppPayload {
        app_id: "z00z".to_string(),
        app_name: "Z00Z".to_string(),
        app_version: "0.0.0".to_string(),
        platform: AppPlatform::Linux,
        instance_id: generate_16_bytes(rng),
        created_at: now_ms,
        last_opened_at: None,
        notes: None,
    };
    let app_record = encrypt_object_record(
        rng,
        wallet_id,
        data_key,
        app_id,
        PAYLOAD_VERSION_APP,
        ObjectKindId::App as u8,
        encode_bincode(&app_payload)?,
    )?;

    let chain_payload = ChainPayload {
        chain: identity.chain.clone(),
        chain_id: None,
        genesis_hash: None,
        params: None,
        created_at: now_ms,
    };
    let chain_record = encrypt_object_record(
        rng,
        wallet_id,
        data_key,
        chain_id,
        PAYLOAD_VERSION_CHAIN,
        ObjectKindId::Chain as u8,
        encode_bincode(&chain_payload)?,
    )?;

    let keys_payload = KeysPayload {
        keyset_id: generate_object_id(rng),
        account_id: Some(main_account_id),
        signing_keys: Vec::new(),
        created_at: now_ms,
        updated_at: None,
    };
    let keys_record = encrypt_object_record(
        rng,
        wallet_id,
        data_key,
        keys_id,
        PAYLOAD_VERSION_KEYS,
        ObjectKindId::Keys as u8,
        encode_bincode(&keys_payload)?,
    )?;

    let stealth_payload = StealthMetaPayload {
        view_key_version: 0,
        receiver_mode: "stealth_ecdh".to_string(),
        stealth_activated_at: None,
        mode_audit: Vec::new(),
    };
    let stealth_meta_record = encrypt_object_record(
        rng,
        wallet_id,
        data_key,
        stealth_meta_id,
        PAYLOAD_VERSION_STEALTH_META,
        ObjectKindId::StealthMeta as u8,
        encode_bincode(&stealth_payload)?,
    )?;

    let tofu_pins_id = generate_object_id(rng);
    let tofu_pins_payload = TofuPinsPayload {
        pins: Vec::new(),
        updated_at: now_ms,
    };
    let tofu_pins_record = encrypt_object_record(
        rng,
        wallet_id,
        data_key,
        tofu_pins_id,
        PAYLOAD_VERSION_TOFU_PINS,
        ObjectKindId::TofuPins as u8,
        encode_bincode(&tofu_pins_payload)?,
    )?;

    Ok(InitialObjects {
        wallet_root_id,
        wallet_root_record,
        main_account_id,
        account_record,
        derivation_state_id,
        derivation_record,
        scan_state_id,
        scan_record,
        app_id,
        app_record,
        chain_id,
        chain_record,
        keys_id,
        keys_record,
        stealth_meta_id,
        stealth_meta_record,
        tofu_pins_id,
        tofu_pins_record,
    })
}

pub(super) struct InitialObjects {
    pub(super) wallet_root_id: u128,
    pub(super) wallet_root_record: EncryptedObjectRecord,
    pub(super) main_account_id: u128,
    pub(super) account_record: EncryptedObjectRecord,
    pub(super) derivation_state_id: u128,
    pub(super) derivation_record: EncryptedObjectRecord,
    pub(super) scan_state_id: u128,
    pub(super) scan_record: EncryptedObjectRecord,
    pub(super) app_id: u128,
    pub(super) app_record: EncryptedObjectRecord,
    pub(super) chain_id: u128,
    pub(super) chain_record: EncryptedObjectRecord,
    pub(super) keys_id: u128,
    pub(super) keys_record: EncryptedObjectRecord,
    pub(super) stealth_meta_id: u128,
    pub(super) stealth_meta_record: EncryptedObjectRecord,
    pub(super) tofu_pins_id: u128,
    pub(super) tofu_pins_record: EncryptedObjectRecord,
}
