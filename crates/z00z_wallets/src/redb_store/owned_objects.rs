use super::objects::write_object_by_id;
use super::owned_assets::{collect_object_ids, parse_offset_cursor};
use super::queries::validate_object_index_rows;
use super::{
    decode_bincode, encode_bincode, read_object_by_id, wallet_asset_store, write_object,
    AssetFilter, IndexTable, IndexUpdate, ObjectKindId, OwnedAssetPayload, OwnedNonAssetPayload,
    OwnedObjectFamily, OwnedObjectPayload, OwnedRightPayload, OwnedRightStatus,
    OwnedVoucherPayload, OwnedVoucherStatus, WalletAssetStore, WalletError, WalletInventoryPayload,
    WalletObjectStatus, WalletOwnedObject, WalletPolicyAvailability, WalletResult, WalletSession,
    PAYLOAD_VERSION_OWNED_ASSET, PAYLOAD_VERSION_OWNED_RIGHT, PAYLOAD_VERSION_OWNED_VOUCHER,
};
use crate::db::index_codecs::{encode_index_semantic_kv, IndexValueBytes};
use z00z_utils::rng::SystemRngProvider;

#[derive(Debug, Clone, Default)]
pub struct ObjectInventoryFilter {
    pub account_id: Option<u128>,
    pub family: Option<OwnedObjectFamily>,
    pub status: Option<WalletObjectStatus>,
    pub policy_availability: Option<WalletPolicyAvailability>,
    pub holder_commitment: Option<[u8; 32]>,
}

#[derive(Debug, Clone, Default)]
pub struct ObjectInventoryPage {
    pub items: Vec<WalletOwnedObject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PutOwnedObjectOutcome {
    Inserted { object_id: u128 },
    AlreadyPresent { object_id: u128 },
}

pub trait ObjectInventoryStore {
    fn put_voucher(
        &self,
        session: &WalletSession,
        payload: OwnedVoucherPayload,
    ) -> WalletResult<PutOwnedObjectOutcome>;

    fn put_right(
        &self,
        session: &WalletSession,
        payload: OwnedRightPayload,
    ) -> WalletResult<PutOwnedObjectOutcome>;

    fn replace_voucher(
        &self,
        session: &WalletSession,
        payload: OwnedVoucherPayload,
    ) -> WalletResult<()>;

    fn replace_right(
        &self,
        session: &WalletSession,
        payload: OwnedRightPayload,
    ) -> WalletResult<()>;

    #[cfg(test)]
    fn get_owned_object(
        &self,
        session: &WalletSession,
        family: OwnedObjectFamily,
        stable_key: &[u8; 32],
    ) -> WalletResult<Option<WalletOwnedObject>>;

    fn lookup_non_asset_object(
        &self,
        session: &WalletSession,
        stable_key: &[u8; 32],
    ) -> WalletResult<Option<WalletOwnedObject>>;

    fn list_wallet_inventory(
        &self,
        session: &WalletSession,
        filter: ObjectInventoryFilter,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<ObjectInventoryPage>;

    fn list_voucher_claims(
        &self,
        session: &WalletSession,
        status: Option<OwnedVoucherStatus>,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<Vec<OwnedVoucherPayload>>;

    fn list_right_inventory(
        &self,
        session: &WalletSession,
        status: Option<OwnedRightStatus>,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<Vec<OwnedRightPayload>>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RedbObjectInventoryStore;

pub fn object_inventory_store() -> RedbObjectInventoryStore {
    RedbObjectInventoryStore
}

impl ObjectInventoryStore for RedbObjectInventoryStore {
    fn put_voucher(
        &self,
        session: &WalletSession,
        payload: OwnedVoucherPayload,
    ) -> WalletResult<PutOwnedObjectOutcome> {
        self.put_non_asset_payload(session, OwnedNonAssetPayload::Voucher(payload))
    }

    fn put_right(
        &self,
        session: &WalletSession,
        payload: OwnedRightPayload,
    ) -> WalletResult<PutOwnedObjectOutcome> {
        self.put_non_asset_payload(session, OwnedNonAssetPayload::Right(payload))
    }

    fn replace_voucher(
        &self,
        session: &WalletSession,
        payload: OwnedVoucherPayload,
    ) -> WalletResult<()> {
        self.replace_non_asset_payload(session, OwnedNonAssetPayload::Voucher(payload))
    }

    fn replace_right(
        &self,
        session: &WalletSession,
        payload: OwnedRightPayload,
    ) -> WalletResult<()> {
        self.replace_non_asset_payload(session, OwnedNonAssetPayload::Right(payload))
    }

    #[cfg(test)]
    fn get_owned_object(
        &self,
        session: &WalletSession,
        family: OwnedObjectFamily,
        stable_key: &[u8; 32],
    ) -> WalletResult<Option<WalletOwnedObject>> {
        let Some(object_id) = get_non_asset_object_id(session, family, stable_key)? else {
            return Ok(None);
        };
        let payload = decode_owned_object_payload(session, object_id)?;
        Ok(Some(WalletOwnedObject {
            object_id: Some(object_id),
            payload,
        }))
    }

    fn lookup_non_asset_object(
        &self,
        session: &WalletSession,
        stable_key: &[u8; 32],
    ) -> WalletResult<Option<WalletOwnedObject>> {
        for family in [OwnedObjectFamily::Voucher, OwnedObjectFamily::Right] {
            let Some(object_id) = get_non_asset_object_id(session, family, stable_key)? else {
                continue;
            };
            let payload = decode_owned_object_payload(session, object_id)?;
            return Ok(Some(WalletOwnedObject {
                object_id: Some(object_id),
                payload,
            }));
        }

        Ok(None)
    }

    fn list_wallet_inventory(
        &self,
        session: &WalletSession,
        filter: ObjectInventoryFilter,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<ObjectInventoryPage> {
        if limit == 0 {
            return Ok(ObjectInventoryPage::default());
        }

        let mut items = Vec::new();

        if let Some(asset_filter) = asset_filter_from_inventory_filter(&filter) {
            let assets =
                wallet_asset_store().list_owned_assets(session, asset_filter, None, usize::MAX)?;
            for payload in assets.items {
                let object_id =
                    super::owned_assets::find_owned_asset_object_id(session, &payload.asset_id)?;
                items.push(WalletOwnedObject {
                    object_id,
                    payload: OwnedObjectPayload::Asset(payload),
                });
            }
        }

        for family in non_asset_families_for_filter(&filter) {
            let object_ids = collect_non_asset_object_ids(session, *family, &filter)?;
            for object_id in object_ids {
                let payload = decode_owned_object_payload(session, object_id)?;
                if payload_matches_inventory_filter(&payload, &filter) {
                    items.push(WalletOwnedObject {
                        object_id: Some(object_id),
                        payload,
                    });
                }
            }
        }

        items.sort_by_key(|object| object.payload.inventory_sort_key());
        items.dedup_by(|left, right| {
            left.payload.family() == right.payload.family()
                && left.payload.stable_object_key() == right.payload.stable_object_key()
        });

        let start = parse_offset_cursor(cursor.as_deref())?;
        if start >= items.len() {
            return Ok(ObjectInventoryPage::default());
        }
        let end = start.saturating_add(limit).min(items.len());

        Ok(ObjectInventoryPage {
            items: items[start..end].to_vec(),
        })
    }

    fn list_voucher_claims(
        &self,
        session: &WalletSession,
        status: Option<OwnedVoucherStatus>,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<Vec<OwnedVoucherPayload>> {
        let page = self.list_wallet_inventory(
            session,
            ObjectInventoryFilter {
                family: Some(OwnedObjectFamily::Voucher),
                status: status.map(WalletObjectStatus::Voucher),
                ..ObjectInventoryFilter::default()
            },
            cursor,
            limit,
        )?;

        Ok(page
            .items
            .into_iter()
            .filter_map(|object| match object.payload {
                OwnedObjectPayload::Voucher(payload) => Some(payload),
                OwnedObjectPayload::Asset(_) | OwnedObjectPayload::Right(_) => None,
            })
            .collect())
    }

    fn list_right_inventory(
        &self,
        session: &WalletSession,
        status: Option<OwnedRightStatus>,
        cursor: Option<String>,
        limit: usize,
    ) -> WalletResult<Vec<OwnedRightPayload>> {
        let page = self.list_wallet_inventory(
            session,
            ObjectInventoryFilter {
                family: Some(OwnedObjectFamily::Right),
                status: status.map(WalletObjectStatus::Right),
                ..ObjectInventoryFilter::default()
            },
            cursor,
            limit,
        )?;

        Ok(page
            .items
            .into_iter()
            .filter_map(|object| match object.payload {
                OwnedObjectPayload::Right(payload) => Some(payload),
                OwnedObjectPayload::Asset(_) | OwnedObjectPayload::Voucher(_) => None,
            })
            .collect())
    }
}

impl RedbObjectInventoryStore {
    fn put_non_asset_payload(
        &self,
        session: &WalletSession,
        payload: OwnedNonAssetPayload,
    ) -> WalletResult<PutOwnedObjectOutcome> {
        payload.verify_checksum()?;
        payload.validate_invariants()?;

        let stable_key = payload.stable_object_key();
        let family = payload.family();
        let inventory_payload = WalletInventoryPayload::from(payload.clone());
        let index_updates = owned_object_index_updates(&inventory_payload, None)?;
        if let Some(object_id) = get_non_asset_object_id(session, family, &stable_key)? {
            let stored = decode_owned_object_payload(session, object_id)?;
            if same_object_insert_shape(&stored, &inventory_payload) {
                return Ok(PutOwnedObjectOutcome::AlreadyPresent { object_id });
            }
            return Err(WalletError::InvalidConfig(
                "duplicate owned object id conflicts with stored payload".to_string(),
            ));
        }

        let object_id = write_object(
            session,
            payload.kind_id(),
            payload.payload_version(),
            encode_non_asset_payload(&payload)?,
            &index_updates,
            SystemRngProvider,
        )?;

        Ok(PutOwnedObjectOutcome::Inserted { object_id })
    }

    fn replace_non_asset_payload(
        &self,
        session: &WalletSession,
        payload: OwnedNonAssetPayload,
    ) -> WalletResult<()> {
        payload.verify_checksum()?;
        payload.validate_invariants()?;

        let stable_key = payload.stable_object_key();
        let family = payload.family();
        let Some(object_id) = get_non_asset_object_id(session, family, &stable_key)? else {
            return Err(WalletError::InvalidConfig(
                "owned object replace target not found".to_string(),
            ));
        };

        let inventory_payload = WalletInventoryPayload::from(payload.clone());
        let index_updates = owned_object_index_updates(&inventory_payload, Some(object_id))?;
        let _ = write_object_by_id(
            session,
            object_id,
            payload.kind_id(),
            payload.payload_version(),
            encode_non_asset_payload(&payload)?,
            &index_updates,
            SystemRngProvider,
        )?;

        Ok(())
    }
}

pub(super) fn rotation_owned_object_index_updates(
    object_id: u128,
    payload_version: u16,
    kind_id: u8,
    payload_bytes: &[u8],
) -> WalletResult<Vec<IndexUpdate>> {
    if kind_id == ObjectKindId::OwnedVoucher as u8
        && payload_version == PAYLOAD_VERSION_OWNED_VOUCHER
    {
        let payload: OwnedVoucherPayload = decode_bincode(payload_bytes)?;
        return owned_object_index_updates(&OwnedObjectPayload::Voucher(payload), Some(object_id));
    }

    if kind_id == ObjectKindId::OwnedRight as u8 && payload_version == PAYLOAD_VERSION_OWNED_RIGHT {
        let payload: OwnedRightPayload = decode_bincode(payload_bytes)?;
        return owned_object_index_updates(&OwnedObjectPayload::Right(payload), Some(object_id));
    }

    Ok(Vec::new())
}

pub(super) fn decode_owned_object_payload(
    session: &WalletSession,
    object_id: u128,
) -> WalletResult<OwnedObjectPayload> {
    validate_object_index_rows(session, object_id)?;
    let payload = read_object_by_id(session, object_id)?;
    match payload.kind_id {
        x if x == ObjectKindId::OwnedVoucher as u8
            && payload.payload_version == PAYLOAD_VERSION_OWNED_VOUCHER =>
        {
            let decoded: OwnedVoucherPayload = decode_bincode(&payload.data)?;
            let decoded = decoded.migrate_to_current()?;
            decoded.verify_checksum()?;
            decoded.validate_invariants()?;
            Ok(OwnedObjectPayload::Voucher(decoded))
        }
        x if x == ObjectKindId::OwnedRight as u8
            && payload.payload_version == PAYLOAD_VERSION_OWNED_RIGHT =>
        {
            let decoded: OwnedRightPayload = decode_bincode(&payload.data)?;
            let decoded = decoded.migrate_to_current()?;
            decoded.verify_checksum()?;
            decoded.validate_invariants()?;
            Ok(OwnedObjectPayload::Right(decoded))
        }
        x if x == ObjectKindId::OwnedAsset as u8
            && payload.payload_version == PAYLOAD_VERSION_OWNED_ASSET =>
        {
            let decoded: OwnedAssetPayload = decode_bincode(&payload.data)?;
            let decoded = decoded.migrate_to_current()?;
            decoded.verify_checksum()?;
            let _ = decoded.validate_invariants()?;
            Ok(OwnedObjectPayload::Asset(decoded))
        }
        _ => Err(WalletError::InvalidConfig(
            "owned object payload kind mismatch".to_string(),
        )),
    }
}

pub(super) fn owned_object_index_updates(
    payload: &OwnedObjectPayload,
    object_id: Option<u128>,
) -> WalletResult<Vec<IndexUpdate>> {
    match payload {
        OwnedObjectPayload::Voucher(payload) => owned_voucher_index_updates(payload, object_id),
        OwnedObjectPayload::Right(payload) => owned_right_index_updates(payload, object_id),
        OwnedObjectPayload::Asset(_) => Ok(Vec::new()),
    }
}

fn asset_filter_from_inventory_filter(filter: &ObjectInventoryFilter) -> Option<AssetFilter> {
    if filter.holder_commitment.is_some() {
        return None;
    }

    if matches!(
        filter.family,
        Some(OwnedObjectFamily::Voucher) | Some(OwnedObjectFamily::Right)
    ) {
        return None;
    }

    if let Some(policy_availability) = filter.policy_availability {
        if policy_availability != WalletPolicyAvailability::Available {
            return None;
        }
    }

    let status = match filter.status {
        None => None,
        Some(WalletObjectStatus::Asset(status)) => Some(status),
        Some(WalletObjectStatus::Voucher(_) | WalletObjectStatus::Right(_)) => return None,
    };

    Some(AssetFilter {
        account_id: filter.account_id,
        status,
        ..AssetFilter::default()
    })
}

fn non_asset_families_for_filter(filter: &ObjectInventoryFilter) -> &'static [OwnedObjectFamily] {
    match filter.family {
        Some(OwnedObjectFamily::Voucher) => &[OwnedObjectFamily::Voucher],
        Some(OwnedObjectFamily::Right) => &[OwnedObjectFamily::Right],
        Some(OwnedObjectFamily::Asset) => &[],
        None => &[OwnedObjectFamily::Voucher, OwnedObjectFamily::Right],
    }
}

fn collect_non_asset_object_ids(
    session: &WalletSession,
    family: OwnedObjectFamily,
    filter: &ObjectInventoryFilter,
) -> WalletResult<Vec<u128>> {
    let (table, semantic) = choose_non_asset_query(family, filter)?;
    collect_object_ids(session, table, semantic.as_deref())
}

fn choose_non_asset_query(
    family: OwnedObjectFamily,
    filter: &ObjectInventoryFilter,
) -> WalletResult<(IndexTable, Option<Vec<u8>>)> {
    if let Some(holder_commitment) = filter.holder_commitment {
        return Ok((
            IndexTable::OwnedObjectByHolder,
            Some(encode_owned_object_by_holder(family, &holder_commitment)?),
        ));
    }

    if let Some(status) = filter.status {
        let semantic = match status {
            WalletObjectStatus::Asset(_) => None,
            WalletObjectStatus::Voucher(status) if family == OwnedObjectFamily::Voucher => Some(
                encode_owned_object_by_status(family, owned_voucher_status_tag(status))?,
            ),
            WalletObjectStatus::Right(status) if family == OwnedObjectFamily::Right => Some(
                encode_owned_object_by_status(family, owned_right_status_tag(status))?,
            ),
            WalletObjectStatus::Voucher(_) | WalletObjectStatus::Right(_) => None,
        };
        if let Some(semantic) = semantic {
            return Ok((IndexTable::OwnedObjectByStatus, Some(semantic)));
        }
    }

    if let Some(policy_availability) = filter.policy_availability {
        return Ok((
            IndexTable::OwnedObjectByPolicy,
            Some(encode_owned_object_by_policy(family, policy_availability)?),
        ));
    }

    Ok((
        IndexTable::OwnedObjectByFamily,
        Some(encode_owned_object_by_family(family)?),
    ))
}

fn payload_matches_inventory_filter(
    payload: &OwnedObjectPayload,
    filter: &ObjectInventoryFilter,
) -> bool {
    if let Some(account_id) = filter.account_id {
        if payload.account_id() != Some(account_id) {
            return false;
        }
    }

    if let Some(family) = filter.family {
        if payload.family() != family {
            return false;
        }
    }

    if let Some(status) = filter.status {
        match (status, payload.status()) {
            (WalletObjectStatus::Asset(left), WalletObjectStatus::Asset(right)) => {
                if left != right {
                    return false;
                }
            }
            (WalletObjectStatus::Voucher(left), WalletObjectStatus::Voucher(right)) => {
                if left != right {
                    return false;
                }
            }
            (WalletObjectStatus::Right(left), WalletObjectStatus::Right(right)) => {
                if left != right {
                    return false;
                }
            }
            _ => return false,
        }
    }

    if let Some(policy_availability) = filter.policy_availability {
        if payload.policy_availability() != policy_availability {
            return false;
        }
    }

    if let Some(holder_commitment) = filter.holder_commitment {
        if payload.holder_commitment() != Some(holder_commitment) {
            return false;
        }
    }

    true
}

fn get_non_asset_object_id(
    session: &WalletSession,
    family: OwnedObjectFamily,
    stable_key: &[u8; 32],
) -> WalletResult<Option<u128>> {
    let (table, semantic) = match family {
        OwnedObjectFamily::Asset => {
            return Err(WalletError::InvalidConfig(
                "asset lookup must continue using wallet_asset_store".to_string(),
            ));
        }
        OwnedObjectFamily::Voucher => (
            IndexTable::OwnedVoucherById,
            encode_owned_voucher_by_id(stable_key)?,
        ),
        OwnedObjectFamily::Right => (
            IndexTable::OwnedRightById,
            encode_owned_right_by_id(stable_key)?,
        ),
    };

    let matches = collect_object_ids(session, table, Some(semantic.as_slice()))?;
    match matches.as_slice() {
        [] => Ok(None),
        [object_id] => Ok(Some(*object_id)),
        _ => Err(WalletError::InvalidConfig(
            "owned object id resolved to multiple object ids".to_string(),
        )),
    }
}

fn encode_non_asset_payload(payload: &OwnedNonAssetPayload) -> WalletResult<Vec<u8>> {
    match payload {
        OwnedNonAssetPayload::Voucher(payload) => encode_bincode(payload),
        OwnedNonAssetPayload::Right(payload) => encode_bincode(payload),
    }
}

fn owned_voucher_index_updates(
    payload: &OwnedVoucherPayload,
    object_id: Option<u128>,
) -> WalletResult<Vec<IndexUpdate>> {
    let object_id_value = match object_id {
        Some(object_id) => IndexValueBytes::from_object_id(object_id),
        None => IndexValueBytes::new(Vec::new())?,
    };
    let status_tag = owned_voucher_status_tag(payload.status);
    let mut updates = vec![
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedObjectByFamily,
            encode_owned_object_by_family(OwnedObjectFamily::Voucher)?,
            object_id_value.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedObjectByStatus,
            encode_owned_object_by_status(OwnedObjectFamily::Voucher, status_tag)?,
            object_id_value.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedObjectByPolicy,
            encode_owned_object_by_policy(OwnedObjectFamily::Voucher, payload.policy.availability)?,
            object_id_value.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedObjectByHolder,
            encode_owned_object_by_holder(
                OwnedObjectFamily::Voucher,
                &payload.voucher_leaf.holder_commitment,
            )?,
            object_id_value.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedVoucherById,
            encode_owned_voucher_by_id(payload.terminal_id.as_bytes())?,
            object_id_value,
        )?,
    ];

    if payload.policy.availability != WalletPolicyAvailability::Available
        && payload.status != OwnedVoucherStatus::Quarantined
    {
        return Err(WalletError::InvalidConfig(
            "owned voucher unavailable policy must remain quarantined".to_string(),
        ));
    }

    updates.sort_by(|left, right| {
        left.table
            .store_name()
            .cmp(right.table.store_name())
            .then_with(|| left.semantic_key.cmp(&right.semantic_key))
    });

    Ok(updates)
}

fn owned_right_index_updates(
    payload: &OwnedRightPayload,
    object_id: Option<u128>,
) -> WalletResult<Vec<IndexUpdate>> {
    let object_id_value = match object_id {
        Some(object_id) => IndexValueBytes::from_object_id(object_id),
        None => IndexValueBytes::new(Vec::new())?,
    };
    let status_tag = owned_right_status_tag(payload.status);
    let mut updates = vec![
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedObjectByFamily,
            encode_owned_object_by_family(OwnedObjectFamily::Right)?,
            object_id_value.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedObjectByStatus,
            encode_owned_object_by_status(OwnedObjectFamily::Right, status_tag)?,
            object_id_value.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedObjectByPolicy,
            encode_owned_object_by_policy(OwnedObjectFamily::Right, payload.policy.availability)?,
            object_id_value.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedObjectByHolder,
            encode_owned_object_by_holder(
                OwnedObjectFamily::Right,
                &payload.right_leaf.holder_commitment,
            )?,
            object_id_value.clone(),
        )?,
        IndexUpdate::with_value_bytes(
            IndexTable::OwnedRightById,
            encode_owned_right_by_id(payload.terminal_id.as_bytes())?,
            object_id_value,
        )?,
    ];

    if payload.policy.availability != WalletPolicyAvailability::Available
        && payload.status != OwnedRightStatus::Quarantined
    {
        return Err(WalletError::InvalidConfig(
            "owned right unavailable policy must remain quarantined".to_string(),
        ));
    }

    updates.sort_by(|left, right| {
        left.table
            .store_name()
            .cmp(right.table.store_name())
            .then_with(|| left.semantic_key.cmp(&right.semantic_key))
    });

    Ok(updates)
}

fn encode_owned_object_by_family(family: OwnedObjectFamily) -> WalletResult<Vec<u8>> {
    encode_index_semantic_kv(
        "wallet.owned_object",
        "family",
        &[owned_object_family_tag(family)],
    )
}

fn encode_owned_object_by_status(
    family: OwnedObjectFamily,
    status_tag: u8,
) -> WalletResult<Vec<u8>> {
    encode_index_semantic_kv(
        "wallet.owned_object",
        "family_status",
        &[owned_object_family_tag(family), status_tag],
    )
}

fn encode_owned_object_by_policy(
    family: OwnedObjectFamily,
    availability: WalletPolicyAvailability,
) -> WalletResult<Vec<u8>> {
    encode_index_semantic_kv(
        "wallet.owned_object",
        "family_policy",
        &[
            owned_object_family_tag(family),
            policy_availability_tag(availability),
        ],
    )
}

fn encode_owned_object_by_holder(
    family: OwnedObjectFamily,
    holder_commitment: &[u8; 32],
) -> WalletResult<Vec<u8>> {
    let mut value = Vec::with_capacity(33);
    value.push(owned_object_family_tag(family));
    value.extend_from_slice(holder_commitment);
    encode_index_semantic_kv("wallet.owned_object", "family_holder", value.as_slice())
}

fn encode_owned_voucher_by_id(terminal_id: &[u8; 32]) -> WalletResult<Vec<u8>> {
    encode_index_semantic_kv("wallet.owned_voucher", "terminal_id", terminal_id)
}

fn encode_owned_right_by_id(terminal_id: &[u8; 32]) -> WalletResult<Vec<u8>> {
    encode_index_semantic_kv("wallet.owned_right", "terminal_id", terminal_id)
}

fn owned_object_family_tag(family: OwnedObjectFamily) -> u8 {
    match family {
        OwnedObjectFamily::Asset => 1,
        OwnedObjectFamily::Voucher => 2,
        OwnedObjectFamily::Right => 3,
    }
}

fn policy_availability_tag(availability: WalletPolicyAvailability) -> u8 {
    match availability {
        WalletPolicyAvailability::Available => 1,
        WalletPolicyAvailability::Unknown => 2,
        WalletPolicyAvailability::Missing => 3,
    }
}

fn owned_voucher_status_tag(status: OwnedVoucherStatus) -> u8 {
    match status {
        OwnedVoucherStatus::Offered => 1,
        OwnedVoucherStatus::PendingAccept => 2,
        OwnedVoucherStatus::Accepted => 3,
        OwnedVoucherStatus::Redeemable => 4,
        OwnedVoucherStatus::PartiallyRedeemed => 5,
        OwnedVoucherStatus::Redeemed => 6,
        OwnedVoucherStatus::Rejected => 7,
        OwnedVoucherStatus::Refunded => 8,
        OwnedVoucherStatus::Expired => 9,
        OwnedVoucherStatus::Quarantined => 10,
    }
}

fn owned_right_status_tag(status: OwnedRightStatus) -> u8 {
    match status {
        OwnedRightStatus::Granted => 1,
        OwnedRightStatus::Held => 2,
        OwnedRightStatus::Delegated => 3,
        OwnedRightStatus::Consumed => 4,
        OwnedRightStatus::Revoked => 5,
        OwnedRightStatus::Expired => 6,
        OwnedRightStatus::Challenged => 7,
        OwnedRightStatus::Quarantined => 8,
    }
}

fn same_object_insert_shape(left: &OwnedObjectPayload, right: &OwnedObjectPayload) -> bool {
    match (left, right) {
        (OwnedObjectPayload::Voucher(left), OwnedObjectPayload::Voucher(right)) => {
            left.wallet_id == right.wallet_id
                && left.account_id == right.account_id
                && left.terminal_id == right.terminal_id
                && left.voucher_leaf == right.voucher_leaf
                && left.status == right.status
                && left.source == right.source
                && left.first_seen.as_ref().map(|seen| seen.height)
                    == right.first_seen.as_ref().map(|seen| seen.height)
                && left.scan_ref == right.scan_ref
                && left.receive_ref == right.receive_ref
                && left.confirmation_ref == right.confirmation_ref
                && left.labels == right.labels
                && left.policy == right.policy
                && left.holder_opening == right.holder_opening
                && left.beneficiary_opening == right.beneficiary_opening
        }
        (OwnedObjectPayload::Right(left), OwnedObjectPayload::Right(right)) => {
            left.wallet_id == right.wallet_id
                && left.account_id == right.account_id
                && left.terminal_id == right.terminal_id
                && left.right_leaf == right.right_leaf
                && left.status == right.status
                && left.source == right.source
                && left.first_seen.as_ref().map(|seen| seen.height)
                    == right.first_seen.as_ref().map(|seen| seen.height)
                && left.scan_ref == right.scan_ref
                && left.receive_ref == right.receive_ref
                && left.confirmation_ref == right.confirmation_ref
                && left.labels == right.labels
                && left.policy == right.policy
                && left.holder_opening == right.holder_opening
                && left.control_opening == right.control_opening
                && left.beneficiary_opening == right.beneficiary_opening
        }
        _ => false,
    }
}
