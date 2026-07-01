use super::{
    build_claim_store_ops, build_snapshot, path_exists, read_file, to_hex, AssetWire, BTreeMap,
    CheckRoot, Codec, DefinitionId, Deserialize, JsonCodec, Path, PathBuf, PrepFile, PrepRow,
    PrepSnapshot, PrepSnapshotError, PrepSnapshotId, SerialId, SettlementPath, SettlementStateRoot,
    SettlementStore, SnapItem, TerminalId, TerminalLeaf, TxInputWire, TxStorage,
    CLAIM_POST_VIEW_DIR,
};

#[cfg(test)]
use super::{StoreItem, StoreOp};

fn settlement_check_root(store: &SettlementStore) -> Result<CheckRoot, String> {
    store
        .settlement_root()
        .map(CheckRoot::from)
        .map_err(|e| e.to_string())
}

pub(crate) fn build_prep_file(
    claim_store: &SettlementStore,
    selected: &[AssetWire],
    tx_inputs: &[TxInputWire],
) -> Result<PrepFile, String> {
    if selected.len() != tx_inputs.len() {
        return Err("stage4: prep input length mismatch".to_string());
    }

    let prev_root = settlement_check_root(claim_store)
        .map_err(|e| format!("stage4: claim store root load failed: {e}"))?;
    let prev_root_hex = to_hex(prev_root.as_bytes());

    let mut rows = selected
        .iter()
        .zip(tx_inputs.iter())
        .map(|(wire, input_ref)| prep_leaf(wire, input_ref))
        .collect::<Result<Vec<_>, _>>()?;

    for row in &mut rows {
        sync_prep_path(claim_store, row)?;
        row.member_wit_hex = prep_wit_hex(claim_store, row)?;
    }

    Ok(PrepFile {
        prev_root_hex,
        rows,
    })
}

pub(crate) fn prep_leaf(wire: &AssetWire, input_ref: &TxInputWire) -> Result<PrepRow, String> {
    let mut leaf = z00z_wallets::tx::asset_wire_to_leaf(wire)?;
    let asset_id = parse_id_hex(&input_ref.asset_id_hex)?;
    leaf.asset_id = asset_id;

    Ok(PrepRow {
        definition_id_hex: to_hex(&wire.definition.id),
        asset_id_hex: input_ref.asset_id_hex.clone(),
        serial_id: input_ref.serial_id,
        leaf,
        member_wit_hex: String::new(),
    })
}

fn prep_path(row: &PrepRow) -> Result<SettlementPath, String> {
    let definition_id = DefinitionId::new(parse_id_hex(&row.definition_id_hex)?);
    let terminal_id = TerminalId::new(parse_id_hex(&row.asset_id_hex)?);
    Ok(SettlementPath::new(
        definition_id,
        SerialId::new(row.serial_id),
        terminal_id,
    ))
}

fn sync_prep_path(store: &SettlementStore, row: &mut PrepRow) -> Result<(), String> {
    let terminal_id = TerminalId::new(parse_id_hex(&row.asset_id_hex)?);
    let item = store
        .lookup_settlement(z00z_storage::settlement::SettlementLookup::Terminal(
            terminal_id,
        ))
        .map_err(|err| format!("stage4: canonical prep path lookup failed: {err}"))?
        .ok_or_else(|| {
            format!(
                "stage4: canonical prep path missing for {}",
                row.asset_id_hex
            )
        })?;
    let path = item.path();

    let canonical_definition_id_hex = to_hex(path.definition_id.as_bytes());
    if row.definition_id_hex != canonical_definition_id_hex || row.serial_id != path.serial_id.get()
    {
        return Err(format!(
            "stage4: canonical prep path drift for {}: want_def={} want_serial={} got_def={} got_serial={}",
            row.asset_id_hex,
            canonical_definition_id_hex,
            path.serial_id.get(),
            row.definition_id_hex,
            row.serial_id,
        ));
    }

    if row.leaf.asset_id != *path.terminal_id().as_bytes()
        || row.leaf.serial_id != path.serial_id.get()
    {
        return Err(format!(
            "stage4: canonical prep leaf drift for {}",
            row.asset_id_hex,
        ));
    }

    Ok(())
}

#[cfg(test)]
pub(crate) fn prep_store(rows: &[PrepRow]) -> Result<SettlementStore, String> {
    let mut store = SettlementStore::new();
    let ops = rows
        .iter()
        .map(|row| {
            let path = prep_path(row)?;
            let item = StoreItem::new(path, row.leaf.clone()).map_err(|e| e.to_string())?;
            Ok(StoreOp::Put(Box::new(item)))
        })
        .collect::<Result<Vec<_>, String>>()?;
    store
        .apply_settlement_ops(ops)
        .map_err(|e| format!("stage4: prep store build failed: {e}"))?;
    Ok(store)
}

fn prep_state_root(prep: &PrepFile) -> Result<SettlementStateRoot, String> {
    Ok(SettlementStateRoot::settlement_v1(parse_id_hex(
        &prep.prev_root_hex,
    )?))
}

fn checked_prep_wit(
    root: SettlementStateRoot,
    index: usize,
    path: &SettlementPath,
    leaf: &TerminalLeaf,
    proof: &[u8],
) -> Result<z00z_storage::settlement::ProofItem, String> {
    z00z_storage::settlement::chk_blob_settlement_inclusion_bound(proof, root, path, leaf)
        .map(|blob| blob.item().clone())
        .map_err(|err| format!("stage4: prep witness verification failed at row {index}: {err}"))
}

fn prep_wit_hex(store: &SettlementStore, row: &PrepRow) -> Result<String, String> {
    let path = prep_path(row)?;
    let blob = store
        .settlement_proof_blob(&path)
        .map_err(|e| e.to_string())?;
    let bytes = blob.encode().map_err(|e| e.to_string())?;
    Ok(to_hex(&bytes))
}

pub(crate) fn prep_membership_witnesses(
    prep: &PrepFile,
) -> Result<Vec<z00z_wallets::tx::SpendMembershipWitness>, String> {
    let root = prep_state_root(prep)?;

    prep.rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let path = prep_path(row)?;
            let proof = hex::decode(&row.member_wit_hex)
                .map_err(|_| format!("stage4: prep witness hex invalid at row {index}"))?;
            let proof_item = checked_prep_wit(root, index, &path, &row.leaf, &proof)?;
            z00z_wallets::tx::SpendMembershipWitness::new(path, row.leaf.clone(), proof, proof_item)
                .map_err(|err| {
                    format!("stage4: prep witness relation failed at row {index}: {err}")
                })
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn prep_root(rows: &[PrepRow]) -> Result<CheckRoot, String> {
    settlement_check_root(&prep_store(rows)?)
        .map_err(|e| format!("stage4: prep root build failed: {e}"))
}

fn claim_post_store_root(out: &Path) -> PathBuf {
    out.join("storage").join(CLAIM_POST_VIEW_DIR)
}

pub(crate) fn load_claim_post_store(
    out: &Path,
    claim_pkgs: &[z00z_wallets::tx::ClaimTxPackage],
) -> Result<SettlementStore, String> {
    let root = claim_post_store_root(out);
    let persisted = SettlementStore::load(&root).map_err(|e| {
        format!(
            "stage4: claim_post store load failed at {}: {e}",
            root.display()
        )
    })?;
    let persisted_root = settlement_check_root(&persisted)
        .map_err(|e| format!("stage4: claim_post persisted root load failed: {e}"))?;

    let ops = build_claim_store_ops(claim_pkgs)
        .map_err(|e| format!("stage4: claim_post live store rebuild failed: {e}"))?;
    let mut live = SettlementStore::try_new()
        .map_err(|e| format!("stage4: claim_post live store open failed: {e}"))?;
    live.apply_settlement_ops(ops)
        .map_err(|e| format!("stage4: claim_post live store apply failed: {e}"))?;
    let rights_path = out
        .join("genesis")
        .join(z00z_core::genesis::GENESIS_RIGHTS_FILE);
    if path_exists(&rights_path)
        .map_err(|e| format!("stage4: claim_post rights stat failed: {e}"))?
    {
        let rights: Vec<z00z_core::genesis::GenesisRightRecord> = JsonCodec
            .deserialize(
                read_file(&rights_path)
                    .map_err(|e| format!("stage4: claim_post rights read failed: {e}"))?
                    .as_slice(),
            )
            .map_err(|e| format!("stage4: claim_post rights decode failed: {e}"))?;
        crate::scenario_1::stage_4::publish_genesis_rights(&mut live, &rights)?;
    }
    let live_root = settlement_check_root(&live)
        .map_err(|e| format!("stage4: claim_post live root load failed: {e}"))?;

    if live_root != persisted_root {
        return Err(format!(
            "stage4: claim_post root drift persisted={} rebuilt={}",
            to_hex(persisted_root.as_bytes()),
            to_hex(live_root.as_bytes())
        ));
    }

    Ok(live)
}

fn claim_store_snap_items(store: &SettlementStore) -> Result<Vec<SnapItem>, String> {
    let mut after = None;
    let mut items = Vec::new();

    loop {
        let page = store
            .list_settlement(
                z00z_storage::settlement::SettlementListReq::all(256).with_after(after),
            )
            .map_err(|e| format!("stage4: claim store list failed: {e}"))?;

        for item in page.items() {
            let path = item.path();
            let wit = store
                .settlement_proof_blob(&path)
                .map_err(|e| format!("stage4: claim store proof build failed: {e}"))?
                .encode()
                .map_err(|e| format!("stage4: claim store proof encode failed: {e}"))?;
            items.push(
                SnapItem::new(path, item.leaf().clone(), wit)
                    .map_err(|e| format!("stage4: claim store snapshot item build failed: {e}"))?,
            );
        }

        after = page.next();
        if after.is_none() {
            break;
        }
    }

    Ok(items)
}

pub(crate) fn build_canon_snapshot(
    prep: &PrepFile,
    claim_store: &SettlementStore,
) -> Result<(PrepSnapshot, PrepSnapshotId), String> {
    let prev_root = CheckRoot::new(parse_id_hex(&prep.prev_root_hex)?);
    let entries = claim_store_snap_items(claim_store)?;
    let (snapshot, snapshot_id) = build_snapshot(prev_root, entries).map_err(|e| match e {
        PrepSnapshotError::RootMix => "stage4: canonical snapshot root drift".to_string(),
        other => format!("stage4: canonical snapshot build failed: {other}"),
    })?;

    cmp_prep_snapshot(prep, &snapshot)?;
    Ok((snapshot, snapshot_id))
}

fn cmp_prep_snapshot(prep: &PrepFile, snapshot: &PrepSnapshot) -> Result<(), String> {
    if to_hex(snapshot.prev_root.as_bytes()) != prep.prev_root_hex {
        return Err("stage4: canonical snapshot prev_root mismatch".to_string());
    }

    let root = prep_state_root(prep)?;
    let mut by_path = BTreeMap::new();
    for entry in &snapshot.entries {
        by_path.insert(entry.path(), entry);
    }

    for (index, row) in prep.rows.iter().enumerate() {
        let want_path = prep_path(row)?;
        let entry = by_path.get(&want_path).ok_or_else(|| {
            format!("stage4: canonical snapshot missing prep row at index {index}")
        })?;
        let path = entry.path();
        if to_hex(path.definition_id.as_bytes()) != row.definition_id_hex {
            return Err(format!(
                "stage4: canonical definition mismatch at row {index}"
            ));
        }
        if to_hex(path.terminal_id().as_bytes()) != row.asset_id_hex {
            return Err(format!(
                "stage4: canonical asset_id mismatch at row {index}"
            ));
        }
        if path.serial_id.get() != row.serial_id {
            return Err(format!("stage4: canonical serial mismatch at row {index}"));
        }

        let leaf = entry
            .terminal_leaf()
            .map_err(|e| {
                format!("stage4: canonical snapshot wrong leaf family at row {index}: {e}")
            })?
            .clone();
        if leaf != row.leaf {
            return Err(format!("stage4: canonical leaf mismatch at row {index}"));
        }

        if to_hex(entry.wit()) != row.member_wit_hex {
            return Err(format!("stage4: canonical witness mismatch at row {index}"));
        }

        let proof_item = checked_prep_wit(root, index, &path, &row.leaf, entry.wit())?;
        if CheckRoot::from(proof_item.settlement_root()) != snapshot.prev_root {
            return Err(format!(
                "stage4: canonical witness root mismatch at row {index}"
            ));
        }
    }

    Ok(())
}

fn parse_id_hex(hex_id: &str) -> Result<[u8; 32], String> {
    let raw = hex::decode(hex_id).map_err(|_| format!("stage4: invalid asset_id_hex {hex_id}"))?;
    raw.try_into()
        .map_err(|_| format!("stage4: asset_id_hex length mismatch {hex_id}"))
}
