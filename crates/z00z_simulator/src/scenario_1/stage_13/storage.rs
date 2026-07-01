use std::collections::BTreeSet;

use z00z_core::{genesis::GenesisRightRecord, rights::RightClassConfig, Asset};
use z00z_storage::settlement::{
    BucketPolicy, DefinitionId, FeeActorCtx, FeeEnvelope, FeeSupportCtx, HjmtProofFamily,
    ProofBlob, ProofItem, RightActionCtx, RightClass, RightLeaf, SerialId, SettlementLeafFamily,
    SettlementPath, StoreItem, TerminalId, TerminalLeaf,
};
pub(crate) fn asset_item_from_asset(asset: &Asset) -> Result<StoreItem, String> {
    let mut leaf = TerminalLeaf::dummy_for_scan(asset.serial_id());
    leaf.asset_id = asset.asset_id();
    leaf.serial_id = asset.serial_id();
    let path = SettlementPath::new(
        DefinitionId::new(asset.definition_id()),
        SerialId::new(asset.serial_id()),
        TerminalId::new(asset.asset_id()),
    );
    StoreItem::new(path, leaf).map_err(|e| format!("stage13 asset item build failed: {e}"))
}

pub(crate) fn right_item_from_record(record: &GenesisRightRecord) -> Result<StoreItem, String> {
    let terminal_id = TerminalId::new(record.leaf.terminal_id);
    let path = SettlementPath::new(
        DefinitionId::new(record.definition_id),
        SerialId::new(record.serial_id),
        terminal_id,
    );
    let leaf = RightLeaf {
        version: record.leaf.version,
        terminal_id,
        right_class: storage_right_class(record.leaf.right_class),
        issuer_scope: record.leaf.issuer_scope,
        provider_scope: record.leaf.provider_scope,
        holder_commitment: record.leaf.holder_commitment,
        control_commitment: record.leaf.control_commitment,
        beneficiary_commitment: record.leaf.beneficiary_commitment,
        payload_commitment: record.leaf.payload_commitment,
        valid_from: record.leaf.valid_from,
        valid_until: record.leaf.valid_until,
        challenge_from: record.leaf.challenge_from,
        challenge_until: record.leaf.challenge_until,
        use_nonce: record.leaf.use_nonce,
        revocation_policy_id: record.leaf.revocation_policy_id,
        transition_policy_id: record.leaf.transition_policy_id,
        challenge_policy_id: record.leaf.challenge_policy_id,
        disclosure_policy_id: record.leaf.disclosure_policy_id,
        retention_policy_id: record.leaf.retention_policy_id,
    };
    StoreItem::new(path, leaf).map_err(|e| {
        format!(
            "stage13 right item build failed for {}#{}: {e}",
            record.right_id, record.right_index
        )
    })
}

pub(crate) fn ensure_expected_right_classes(
    rights: &[GenesisRightRecord],
    expected_classes: &[String],
) -> Result<(), String> {
    let mut missing = Vec::new();
    for class in expected_classes {
        if !rights
            .iter()
            .any(|record| record.leaf.right_class.as_str() == class.as_str())
        {
            missing.push(class.clone());
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "stage13 expected genesis right classes missing: {}",
            missing.join(", ")
        ))
    }
}

pub(crate) fn pick_demo_right<'a>(
    rights: &'a [GenesisRightRecord],
    expected_classes: &[String],
) -> Result<&'a GenesisRightRecord, String> {
    if let Some(record) = expected_classes.iter().find_map(|class| {
        rights
            .iter()
            .find(|record| record.leaf.right_class.as_str() == class.as_str())
    }) {
        return Ok(record);
    }
    rights
        .first()
        .ok_or_else(|| "stage13 requires at least one genesis right".to_string())
}

pub(crate) fn transfer_leaf(prior: &RightLeaf, mark: u8) -> RightLeaf {
    let mut next = *prior;
    next.holder_commitment = [mark; 32];
    next.beneficiary_commitment = [mark.wrapping_add(1); 32];
    next
}

pub(crate) fn right_ctx(leaf: &RightLeaf, now: u64) -> RightActionCtx {
    RightActionCtx {
        now,
        expected_holder: Some(leaf.holder_commitment),
        expected_control: Some(leaf.control_commitment),
        ..RightActionCtx::default()
    }
}

pub(crate) fn fee_actor(mark: u8, now: u64) -> FeeActorCtx {
    FeeActorCtx {
        now,
        payer_commitment: Some([mark; 32]),
        sponsor_commitment: None,
    }
}

pub(crate) fn fee_envelope(mark: u8, support: FeeSupportCtx) -> FeeEnvelope {
    let support_ref = Some([mark.wrapping_add(4); 32]);
    let budget_units = support.required_units.saturating_add(1);
    FeeEnvelope {
        version: 1,
        payer_commitment: [mark; 32],
        sponsor_commitment: [0u8; 32],
        budget_units,
        budget_commitment: FeeEnvelope::budget_bind(budget_units, support_ref),
        domain_id: support.domain_id,
        expires_at: 1_000,
        nonce: [mark.wrapping_add(1); 32],
        transition_id: support.transition_id,
        replay_key: [mark.wrapping_add(2); 32],
        support_ref,
        failure_policy_id: [mark.wrapping_add(3); 32],
    }
}

pub(crate) fn fixture_asset_item(path: SettlementPath) -> Result<StoreItem, String> {
    let mut leaf = TerminalLeaf::dummy_for_scan(path.serial_id.get());
    leaf.asset_id = path.terminal_id().into_bytes();
    StoreItem::new(path, leaf).map_err(|e| format!("stage13 fixture asset item failed: {e}"))
}

pub(crate) fn same_bucket_group(
    store: &mut z00z_storage::settlement::SettlementStore,
    needed: usize,
) -> Result<Vec<SettlementPath>, String> {
    let first = SettlementPath::new(
        DefinitionId::new([0x71; 32]),
        SerialId::new(9),
        TerminalId::new([1u8; 32]),
    );
    store
        .put_settlement_item(fixture_asset_item(first)?)
        .map_err(|e| format!("stage13 split seed failed: {e}"))?;
    let target_bucket = store.bucket_policy().derive_bucket_id(first);
    let mut paths = vec![first];
    if paths.len() == needed {
        return Ok(paths);
    }

    let current = store.bucket_policy();
    let split_policy = BucketPolicy::new(
        current.bucket_bits().saturating_add(1),
        current.min_bucket_count(),
        current.max_target_leaf_count(),
        current.compatibility_generation().saturating_add(1),
    )
    .map_err(|e| format!("stage13 split policy build failed: {e}"))?;
    let mut child_buckets = BTreeSet::from([split_policy.derive_bucket_id(first)]);
    let mut pending = Vec::new();

    for seed in 2u32..=u16::MAX as u32 {
        let mut asset_id = [0u8; 32];
        asset_id[..4].copy_from_slice(&seed.to_be_bytes());
        asset_id[31] = 0xA5;
        let candidate = SettlementPath::new(
            first.definition_id,
            first.serial_id,
            TerminalId::new(asset_id),
        );
        if store.bucket_policy().derive_bucket_id(candidate) != target_bucket {
            continue;
        }
        let child_bucket = split_policy.derive_bucket_id(candidate);
        if !child_buckets.contains(&child_bucket) {
            if child_buckets.len() >= 2 {
                continue;
            }
            child_buckets.insert(child_bucket);
        }
        paths.push(candidate);
        pending.push(candidate);
        if paths.len() >= needed && child_buckets.len() == 2 {
            for candidate in pending {
                store
                    .put_settlement_item(fixture_asset_item(candidate)?)
                    .map_err(|e| format!("stage13 split group seed failed: {e}"))?;
            }
            store
                .split_proof(&first)
                .map_err(|e| format!("stage13 split trigger failed: {e}"))?;
            return Ok(paths);
        }
    }

    Err("stage13 failed to find deterministic same-bucket group".to_string())
}

pub(crate) fn missing_right_path_same_bucket(
    policy: BucketPolicy,
    base: SettlementPath,
) -> Result<SettlementPath, String> {
    let target_bucket = policy.derive_bucket_id(base);
    for mark in 1u8..=u8::MAX {
        let candidate = SettlementPath::new(
            base.definition_id,
            base.serial_id,
            TerminalId::new([mark; 32]),
        );
        if candidate != base && policy.derive_bucket_id(candidate) == target_bucket {
            return Ok(candidate);
        }
    }
    Err("stage13 missing same-bucket right path".to_string())
}

pub(crate) fn path_hex(path: SettlementPath) -> String {
    format!(
        "{}/{}/{}",
        hex::encode(path.definition_id.into_bytes()),
        path.serial_id.get(),
        hex::encode(path.terminal_id.into_bytes()),
    )
}

pub(crate) fn terminal_hex(path: SettlementPath) -> String {
    hex::encode(path.terminal_id.into_bytes())
}

pub(crate) fn parse_path_hex(raw: &str) -> Result<SettlementPath, String> {
    let mut parts = raw.split('/');
    let definition = parts
        .next()
        .ok_or_else(|| "stage13 path missing definition".to_string())?;
    let serial = parts
        .next()
        .ok_or_else(|| "stage13 path missing serial".to_string())?;
    let terminal = parts
        .next()
        .ok_or_else(|| "stage13 path missing terminal".to_string())?;
    if parts.next().is_some() {
        return Err("stage13 path has unexpected segments".to_string());
    }
    let definition_bytes: [u8; 32] = hex::decode(definition)
        .map_err(|e| format!("stage13 definition hex decode failed: {e}"))?
        .try_into()
        .map_err(|_| "stage13 definition id must be 32 bytes".to_string())?;
    let terminal_bytes: [u8; 32] = hex::decode(terminal)
        .map_err(|e| format!("stage13 terminal hex decode failed: {e}"))?
        .try_into()
        .map_err(|_| "stage13 terminal id must be 32 bytes".to_string())?;
    let serial_id = serial
        .parse::<u32>()
        .map_err(|e| format!("stage13 serial parse failed: {e}"))?;
    Ok(SettlementPath::new(
        DefinitionId::new(definition_bytes),
        SerialId::new(serial_id),
        TerminalId::new(terminal_bytes),
    ))
}

pub(crate) fn proof_family_name(family: HjmtProofFamily) -> &'static str {
    match family {
        HjmtProofFamily::Inclusion => "inclusion",
        HjmtProofFamily::Deletion => "deletion",
        HjmtProofFamily::NonExistence => "nonexistence",
    }
}

pub(crate) fn leaf_family_name(family: SettlementLeafFamily) -> &'static str {
    match family {
        SettlementLeafFamily::Terminal => "asset",
        SettlementLeafFamily::Right => "right",
        SettlementLeafFamily::Voucher => "voucher",
    }
}

pub(crate) fn typed_error<E>(err: &E) -> super::report::RedactedError
where
    E: std::fmt::Debug + std::fmt::Display,
{
    super::report::RedactedError {
        class: super::report::redact_error_class(err),
        message: super::report::redact_error_message(&format!("{err}")),
    }
}

pub(crate) fn tampered_blob_present_path(
    blob: &ProofBlob,
    present_path: SettlementPath,
    family: SettlementLeafFamily,
) -> Result<ProofBlob, String> {
    let marker_leaf = family.marker_leaf(present_path);
    let item = ProofItem::new_settlement(
        blob.item().settlement_root(),
        present_path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        marker_leaf.clone(),
    )
    .map_err(|e| format!("stage13 tampered proof item failed: {e}"))?;
    let _ = marker_leaf;
    Ok(blob.clone().rebind(item))
}

const fn storage_right_class(class: RightClassConfig) -> RightClass {
    match class {
        RightClassConfig::MachineCapability => RightClass::MachineCapability,
        RightClassConfig::DataAccess => RightClass::DataAccess,
        RightClassConfig::ServiceEntitlement => RightClass::ServiceEntitlement,
        RightClassConfig::ValidatorMandate => RightClass::ValidatorMandate,
        RightClassConfig::OneTimeUse => RightClass::OneTimeUse,
    }
}
