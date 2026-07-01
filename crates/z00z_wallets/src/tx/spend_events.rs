//! Unified spending module.
//!
//! This file consolidates Phase 5/7/9 spending logic previously split across:
//! - `spend_constraints.rs`
//! - `spend_verification.rs`
//! - internal event projection helpers.

#[allow(missing_docs)]
mod events {
    //! Canonical event contracts for state reconstruction.

    use std::collections::{BTreeMap, BTreeSet};

    use thiserror::Error;
    use z00z_core::assets::registry::AssetId;
    use z00z_crypto::{domains::TxDigestDomain, hash_zk::hash_zk};

    const EV_VER: u8 = 1;

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct EventSpent {
        pub ver: u8,
        pub pkg: [u8; 32],
        pub idx: u32,
        pub asset_id: AssetId,
        pub burn_com: [u8; 32],
    }

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct EventCom {
        pub ver: u8,
        pub pkg: [u8; 32],
        pub idx: u32,
        pub asset_id: AssetId,
        pub commit: [u8; 32],
        pub jmt_path: Vec<[u8; 32]>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct EventBatch {
        pub spent: Vec<EventSpent>,
        pub commit: Vec<EventCom>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct CreatedRec {
        pub commit: [u8; 32],
        pub jmt_path: Vec<[u8; 32]>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    pub struct EventState {
        pub spent: BTreeMap<AssetId, [u8; 32]>,
        pub created: BTreeMap<AssetId, CreatedRec>,
        pub seen_pkg: BTreeSet<[u8; 32]>,
        pub pkg_fp: BTreeMap<[u8; 32], [u8; 32]>,
    }

    #[derive(Debug, Error, PartialEq, Eq)]
    pub enum EventErr {
        #[error("event version mismatch")]
        Ver,
        #[error("partial package events are forbidden")]
        Atomic,
        #[error("duplicate event in batch")]
        Dup,
        #[error("state collision")]
        Collision,
        #[error("mixed replay is forbidden")]
        MixedReplay,
        #[error("replay data mismatch")]
        ReplayMismatch,
        #[error("event index order is invalid")]
        BadOrder,
    }

    fn dense_idx(items: &BTreeSet<u32>) -> bool {
        if items.is_empty() {
            return false;
        }

        if !items.contains(&0) {
            return false;
        }

        for (expect, idx) in items.iter().enumerate() {
            if *idx != expect as u32 {
                return false;
            }
        }
        true
    }

    fn pkg_fp(batch: &EventBatch) -> BTreeMap<[u8; 32], [u8; 32]> {
        let mut bytes = BTreeMap::<[u8; 32], Vec<u8>>::new();

        for ev in &batch.spent {
            let buf = bytes.entry(ev.pkg).or_default();
            buf.push(b'S');
            buf.extend_from_slice(&ev.idx.to_le_bytes());
            buf.extend_from_slice(&ev.asset_id);
            buf.extend_from_slice(&ev.burn_com);
        }

        for ev in &batch.commit {
            let buf = bytes.entry(ev.pkg).or_default();
            buf.push(b'C');
            buf.extend_from_slice(&ev.idx.to_le_bytes());
            buf.extend_from_slice(&ev.asset_id);
            buf.extend_from_slice(&ev.commit);
            buf.extend_from_slice(&(ev.jmt_path.len() as u32).to_le_bytes());
            for node in &ev.jmt_path {
                buf.extend_from_slice(node);
            }
        }

        let mut out = BTreeMap::new();
        for (pkg, body) in bytes {
            let fp = hash_zk::<TxDigestDomain>("EV/PKG", &[&body]);
            out.insert(pkg, fp);
        }
        out
    }

    pub fn validate_batch(batch: &EventBatch) -> Result<(), EventErr> {
        if batch.spent.is_empty() || batch.commit.is_empty() {
            return Err(EventErr::Atomic);
        }

        let mut spent_pairs = BTreeSet::new();
        let mut com_pairs = BTreeSet::new();
        let mut spent_assets = BTreeSet::new();
        let mut com_assets = BTreeSet::new();
        let mut spent_pkg = BTreeSet::new();
        let mut com_pkg = BTreeSet::new();
        let mut spent_idx = BTreeMap::<[u8; 32], BTreeSet<u32>>::new();
        let mut com_idx = BTreeMap::<[u8; 32], BTreeSet<u32>>::new();

        for ev in &batch.spent {
            if ev.ver != EV_VER {
                return Err(EventErr::Ver);
            }
            if !spent_pairs.insert((ev.pkg, ev.idx)) {
                return Err(EventErr::Dup);
            }
            if !spent_assets.insert(ev.asset_id) {
                return Err(EventErr::Dup);
            }
            spent_pkg.insert(ev.pkg);
            spent_idx.entry(ev.pkg).or_default().insert(ev.idx);
        }

        for ev in &batch.commit {
            if ev.ver != EV_VER {
                return Err(EventErr::Ver);
            }
            if !com_pairs.insert((ev.pkg, ev.idx)) {
                return Err(EventErr::Dup);
            }
            if !com_assets.insert(ev.asset_id) {
                return Err(EventErr::Dup);
            }
            com_pkg.insert(ev.pkg);
            com_idx.entry(ev.pkg).or_default().insert(ev.idx);
        }

        if spent_pkg != com_pkg {
            return Err(EventErr::Atomic);
        }

        for pkg in &spent_pkg {
            let s_idx = spent_idx.get(pkg).expect("spent idx exists");
            let c_idx = com_idx.get(pkg).expect("commit idx exists");
            if !dense_idx(s_idx) || !dense_idx(c_idx) {
                return Err(EventErr::BadOrder);
            }
        }

        Ok(())
    }

    pub fn apply_batch(state: &mut EventState, batch: &EventBatch) -> Result<(), EventErr> {
        validate_batch(batch)?;

        let pkg_set: BTreeSet<[u8; 32]> = batch.spent.iter().map(|x| x.pkg).collect();
        let old_seen = pkg_set
            .iter()
            .filter(|pkg| state.seen_pkg.contains(*pkg))
            .count();

        if old_seen > 0 && old_seen < pkg_set.len() {
            return Err(EventErr::MixedReplay);
        }
        if old_seen == pkg_set.len() {
            let now_fp = pkg_fp(batch);
            for pkg in &pkg_set {
                let old = state.pkg_fp.get(pkg).ok_or(EventErr::ReplayMismatch)?;
                let new = now_fp.get(pkg).ok_or(EventErr::ReplayMismatch)?;
                if old != new {
                    return Err(EventErr::ReplayMismatch);
                }
            }
            return Ok(());
        }

        let mut next = state.clone();

        for ev in &batch.spent {
            if next.spent.contains_key(&ev.asset_id) || next.created.contains_key(&ev.asset_id) {
                return Err(EventErr::Collision);
            }
            next.spent.insert(ev.asset_id, ev.burn_com);
        }

        for ev in &batch.commit {
            if next.created.contains_key(&ev.asset_id) || next.spent.contains_key(&ev.asset_id) {
                return Err(EventErr::Collision);
            }
            next.created.insert(
                ev.asset_id,
                CreatedRec {
                    commit: ev.commit,
                    jmt_path: ev.jmt_path.clone(),
                },
            );
        }

        for pkg in pkg_set {
            next.seen_pkg.insert(pkg);
        }

        for (pkg, fp) in pkg_fp(batch) {
            next.pkg_fp.insert(pkg, fp);
        }

        *state = next;
        Ok(())
    }

    pub fn replay_events(batches: &[EventBatch]) -> Result<EventState, EventErr> {
        let mut state = EventState::default();
        for batch in batches {
            apply_batch(&mut state, batch)?;
        }
        Ok(state)
    }

    #[cfg(test)]
    mod tests {
        use super::{
            apply_batch, replay_events, validate_batch, EventBatch, EventCom, EventErr, EventSpent,
            EventState,
        };
        use z00z_utils::codec::{Codec, JsonCodec};

        fn one_batch() -> EventBatch {
            EventBatch {
                spent: vec![EventSpent {
                    ver: 1,
                    pkg: [7u8; 32],
                    idx: 0,
                    asset_id: [1u8; 32],
                    burn_com: [9u8; 32],
                }],
                commit: vec![EventCom {
                    ver: 1,
                    pkg: [7u8; 32],
                    idx: 0,
                    asset_id: [2u8; 32],
                    commit: [8u8; 32],
                    jmt_path: vec![[3u8; 32], [4u8; 32]],
                }],
            }
        }

        #[test]
        fn test_validate_batch_ok() {
            let batch = one_batch();
            assert_eq!(validate_batch(&batch), Ok(()));
        }

        #[test]
        fn test_validate_batch_atomic_fail() {
            let mut batch = one_batch();
            batch.commit.clear();
            assert_eq!(validate_batch(&batch), Err(EventErr::Atomic));
        }

        #[test]
        fn test_missing_field_reject() {
            let raw = r#"{\"ver\":1,\"pkg\":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],\"idx\":0,\"burn_com\":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]}"#;
            let res = JsonCodec.deserialize::<EventSpent>(raw.as_bytes());
            assert!(res.is_err());
        }

        #[test]
        fn test_unknown_field_reject() {
            let raw = r#"{\"ver\":1,\"pkg\":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],\"idx\":0,\"asset_id\":[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],\"burn_com\":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],\"x\":1}"#;
            let res = JsonCodec.deserialize::<EventSpent>(raw.as_bytes());
            assert!(res.is_err());
        }

        #[test]
        fn test_apply_batch_ok() {
            let mut state = EventState::default();
            let batch = one_batch();
            assert_eq!(apply_batch(&mut state, &batch), Ok(()));
            assert_eq!(state.spent.len(), 1);
            assert_eq!(state.created.len(), 1);
            assert_eq!(state.seen_pkg.len(), 1);
        }

        #[test]
        fn test_apply_batch_idem() {
            let mut state = EventState::default();
            let batch = one_batch();
            assert_eq!(apply_batch(&mut state, &batch), Ok(()));
            let snap = state.clone();
            assert_eq!(apply_batch(&mut state, &batch), Ok(()));
            assert_eq!(state, snap);
        }

        #[test]
        fn test_apply_batch_replay_mismatch() {
            let mut state = EventState::default();
            let batch = one_batch();
            assert_eq!(apply_batch(&mut state, &batch), Ok(()));

            let mut bad = one_batch();
            bad.spent[0].burn_com = [1u8; 32];

            assert_eq!(apply_batch(&mut state, &bad), Err(EventErr::ReplayMismatch));
        }

        #[test]
        fn test_validate_batch_bad_order() {
            let mut batch = one_batch();
            batch.spent[0].idx = 1;
            assert_eq!(validate_batch(&batch), Err(EventErr::BadOrder));
        }

        #[test]
        fn test_apply_batch_mixed_replay() {
            let mut state = EventState::default();
            let batch1 = one_batch();
            assert_eq!(apply_batch(&mut state, &batch1), Ok(()));

            let mut batch2 = one_batch();
            batch2.spent.push(EventSpent {
                ver: 1,
                pkg: [8u8; 32],
                idx: 0,
                asset_id: [5u8; 32],
                burn_com: [9u8; 32],
            });
            batch2.commit.push(EventCom {
                ver: 1,
                pkg: [8u8; 32],
                idx: 0,
                asset_id: [6u8; 32],
                commit: [8u8; 32],
                jmt_path: vec![],
            });

            assert_eq!(apply_batch(&mut state, &batch2), Err(EventErr::MixedReplay));
        }

        #[test]
        fn test_replay_events_rebuild() {
            let mut batch1 = one_batch();
            batch1.spent[0].asset_id = [10u8; 32];
            batch1.commit[0].asset_id = [11u8; 32];

            let mut batch2 = one_batch();
            batch2.spent[0].pkg = [12u8; 32];
            batch2.commit[0].pkg = [12u8; 32];
            batch2.spent[0].asset_id = [13u8; 32];
            batch2.commit[0].asset_id = [14u8; 32];

            let rebuilt = replay_events(&[batch1, batch2]).expect("replay");
            assert_eq!(rebuilt.spent.len(), 2);
            assert_eq!(rebuilt.created.len(), 2);
            assert_eq!(rebuilt.seen_pkg.len(), 2);
        }

        #[test]
        fn test_canonical_event_are_reachable() {
            let apply_fn = crate::tx::apply_batch;
            let replay_fn = crate::tx::replay_events;
            let validate_fn = crate::tx::validate_batch;
            let _ = (apply_fn, replay_fn, validate_fn);
        }

        #[test]
        fn test_canonical_spend_are_reachable() {
            let build_fn = crate::tx::build_public_spend_contract;
            let verify_fn = crate::tx::verify_tx_public_spend_contract;
            let _ = (build_fn, verify_fn);
        }
    }
}

pub use events::{
    apply_batch, replay_events, validate_batch, CreatedRec, EventBatch, EventCom, EventErr,
    EventSpent, EventState,
};
