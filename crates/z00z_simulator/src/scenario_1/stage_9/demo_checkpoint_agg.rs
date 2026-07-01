#[cfg(test)]
use std::collections::HashSet;

#[cfg(test)]
use z00z_crypto::hash_zk::hash_zk as hash_zk_fn;
#[cfg(test)]
use z00z_crypto::{
    domains::TxDigestDomain, expert::encoding::to_hex, frame_bytes, frame_str, frame_u32_le,
    frame_u64_le,
};

#[cfg(test)]
use super::bundle_lane_impl::{demo_spent_key, DemoCheckpoint, FragIn, FragOut, FragTx, MadeEnt};

#[cfg(test)]
pub(crate) fn build_demo_frag(stage: u32, idx: u32, amount: u64, prev_root_hex: &str) -> FragTx {
    let in_asset = hash_zk_fn::<TxDigestDomain>(
        "S6/IN_ASSET",
        &[&frame_u32_le(stage), &frame_u32_le(idx), &frame_str("in")],
    );
    let out_asset = hash_zk_fn::<TxDigestDomain>(
        "S6/OUT_ASSET",
        &[&frame_u32_le(stage), &frame_u32_le(idx), &frame_str("out")],
    );
    let leaf_hash = hash_zk_fn::<TxDigestDomain>(
        "S6/LEAF_HASH",
        &[
            &frame_u32_le(stage),
            &frame_u32_le(idx),
            &frame_u64_le(amount),
            &frame_str("charlie"),
        ],
    );

    FragTx {
        id: format!("frag_{idx}"),
        prev_root_hex: prev_root_hex.to_string(),
        inputs: vec![FragIn {
            asset_id_hex: to_hex(&in_asset),
            serial_id: idx,
            prev_root_hex: prev_root_hex.to_string(),
            leaf_hash_hex: to_hex(&hash_zk_fn::<TxDigestDomain>(
                "S6/DEMO_IN_LEAF",
                &[&frame_u32_le(stage), &frame_u32_le(idx), &frame_str("leaf")],
            )),
            member_ok: true,
        }],
        outputs: vec![FragOut {
            asset_id_hex: to_hex(&out_asset),
            leaf_hash_hex: to_hex(&leaf_hash),
            amount,
        }],
    }
}

#[cfg(test)]
pub(crate) fn build_demo_cp(
    prev_root_hex: &str,
    frags: &[FragTx],
    spent_prev: &HashSet<String>,
) -> Result<DemoCheckpoint, String> {
    if !is_hex32(prev_root_hex) {
        return Err("invalid prev_root hex".to_string());
    }

    let mut spent_seen = spent_prev.clone();
    let mut frag_seen = HashSet::<String>::new();
    let mut spent_delta = Vec::<String>::new();
    let mut created_delta = Vec::<MadeEnt>::new();
    let mut frag_ids = Vec::<String>::new();

    for frag in frags {
        if !frag_seen.insert(frag.id.clone()) {
            return Err(format!("duplicate fragment id {}", frag.id));
        }

        verify_demo_frag(frag, prev_root_hex)?;
        check_demo_member(frag, prev_root_hex)?;
        check_demo_spent(frag, &mut spent_seen)?;

        frag_ids.push(frag.id.clone());
        for input in &frag.inputs {
            spent_delta.push(demo_spent_key(input));
        }
        for output in &frag.outputs {
            created_delta.push(MadeEnt {
                asset_id_hex: output.asset_id_hex.clone(),
                leaf_hash_hex: output.leaf_hash_hex.clone(),
            });
        }
    }

    let demo_digest = calc_demo_digest(prev_root_hex, &frag_ids, &spent_delta, &created_delta);

    Ok(DemoCheckpoint {
        prev_root_hex: prev_root_hex.to_string(),
        demo_digest_hex: to_hex(&demo_digest),
        spent_delta,
        created_delta,
        fragment_ids: frag_ids,
    })
}

#[cfg(test)]
pub(crate) fn verify_demo_frag(frag: &FragTx, prev_root_hex: &str) -> Result<(), String> {
    if frag.prev_root_hex != prev_root_hex {
        return Err(format!("stale root in {}", frag.id));
    }
    if frag.inputs.is_empty() {
        return Err(format!("no inputs in {}", frag.id));
    }
    if frag.outputs.is_empty() {
        return Err(format!("no outputs in {}", frag.id));
    }

    for input in &frag.inputs {
        if !is_hex32(&input.asset_id_hex) {
            return Err(format!("invalid input asset hex in {}", frag.id));
        }
        if !is_hex32(&input.leaf_hash_hex) {
            return Err(format!("invalid input leaf hash in {}", frag.id));
        }
    }

    for output in &frag.outputs {
        if !is_hex32(&output.asset_id_hex) {
            return Err(format!("invalid output asset hex in {}", frag.id));
        }
        if !is_hex32(&output.leaf_hash_hex) {
            return Err(format!("invalid leaf hash hex in {}", frag.id));
        }
    }

    if frag.outputs.iter().any(|out| out.amount == 0) {
        return Err(format!("zero amount in {}", frag.id));
    }
    Ok(())
}

pub(crate) fn is_hex32(value: &str) -> bool {
    value.len() == 64 && value.as_bytes().iter().all(u8::is_ascii_hexdigit)
}

#[cfg(test)]
pub(crate) fn check_demo_member(frag: &FragTx, prev_root_hex: &str) -> Result<(), String> {
    for input in &frag.inputs {
        if input.prev_root_hex != prev_root_hex {
            return Err(format!("membership root mismatch in {}", frag.id));
        }
        if !input.member_ok {
            return Err(format!("membership witness failed in {}", frag.id));
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn check_demo_spent(
    frag: &FragTx,
    spent_seen: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    for input in &frag.inputs {
        let state_key = demo_spent_key(input);
        if spent_seen.contains(&state_key) {
            return Err(format!("double spend in {}", frag.id));
        }
        spent_seen.insert(state_key);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn calc_demo_digest(
    prev_root_hex: &str,
    frag_ids: &[String],
    spent_delta: &[String],
    created_delta: &[MadeEnt],
) -> [u8; 32] {
    let mut bytes = Vec::<u8>::new();
    bytes.extend_from_slice(&frame_str(prev_root_hex));
    for id in frag_ids {
        bytes.extend_from_slice(&frame_str(id));
    }
    for item in spent_delta {
        bytes.extend_from_slice(&frame_str(item));
    }
    for item in created_delta {
        bytes.extend_from_slice(&frame_str(&item.asset_id_hex));
        bytes.extend_from_slice(&frame_str(&item.leaf_hash_hex));
    }
    hash_zk_fn::<TxDigestDomain>("S6/NEW_ROOT", &[&frame_bytes(&bytes)])
}
