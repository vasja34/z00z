mod live_reload {
    use std::sync::{Mutex, OnceLock};

    use tempfile::tempdir;
    use z00z_core::assets::{AssetLeaf, AssetPackPlain};
    use z00z_crypto::ZkPackEncrypted;
    use z00z_storage::settlement::{
        AdaptiveProofErr, BatchProofBlobV1, BucketEpoch, BucketId, BucketPolicy, DefinitionId,
        MergeProof, PolicyTransitionProof, SerialId, SettlementPath, SettlementStore, SplitProof,
        StoreItem, TerminalId, TerminalLeaf,
    };

    const BACKEND_ENV: &str = "Z00Z_SETTLEMENT_BACKEND_MODE";
    const BUCKET_BITS_ENV: &str = "Z00Z_SETTLEMENT_BUCKET_BITS";

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        backend_mode: Option<String>,
        bucket_bits: Option<String>,
    }

    impl EnvGuard {
        fn live(bits: &str) -> Self {
            let guard = Self {
                backend_mode: std::env::var(BACKEND_ENV).ok(),
                bucket_bits: std::env::var(BUCKET_BITS_ENV).ok(),
            };
            std::env::set_var(BACKEND_ENV, "hjmt");
            std::env::set_var(BUCKET_BITS_ENV, bits);
            guard
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(mode) = &self.backend_mode {
                std::env::set_var(BACKEND_ENV, mode);
            } else {
                std::env::remove_var(BACKEND_ENV);
            }
            if let Some(bits) = &self.bucket_bits {
                std::env::set_var(BUCKET_BITS_ENV, bits);
            } else {
                std::env::remove_var(BUCKET_BITS_ENV);
            }
        }
    }

    fn bytes(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
        SettlementPath::new(
            DefinitionId::new(bytes(definition)),
            SerialId::new(serial),
            TerminalId::new(bytes(asset)),
        )
    }

    fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
        let payload = AssetPackPlain {
            value,
            blinding: [3u8; 32],
            s_out: [4u8; 32],
        }
        .to_bytes();

        AssetLeaf {
            asset_id: path.terminal_id().into_bytes(),
            serial_id: path.serial_id.get(),
            r_pub: [1u8; 32],
            owner_tag: [2u8; 32],
            c_amount: [5u8; 32],
            enc_pack: ZkPackEncrypted {
                version: 1,
                ciphertext: payload,
                tag: [0u8; 16],
            },
            range_proof: vec![9u8; 4],
            tag16: 11,
        }
        .into()
    }

    fn item(path: SettlementPath, value: u64) -> StoreItem {
        StoreItem::new(path, leaf(path, value)).expect("store item")
    }

    fn put_item(store: &mut SettlementStore, path: SettlementPath, value: u64) {
        store
            .put_settlement_item(item(path, value))
            .expect("put settlement item");
    }

    fn split_ready_count(store: &SettlementStore) -> usize {
        usize::try_from(store.bucket_policy().min_bucket_count()).expect("usize") + 1
    }

    fn sibling_bucket_id(bucket_id: BucketId, bucket_bits: u8) -> BucketId {
        let mut bytes = bucket_id.into_bytes();
        let bit_index = bucket_bits - 1;
        let byte_index = usize::from(bit_index / 8);
        let bit_mask = 1u8 << (7 - (bit_index % 8));
        bytes[byte_index] ^= bit_mask;
        BucketId::new(bytes)
    }

    fn split_ready_paths(store: &mut SettlementStore) -> Vec<SettlementPath> {
        let policy = store.bucket_policy();
        let first = path(41, 9, 1);
        let bucket_id = first.bucket_id(policy);
        let needed = split_ready_count(store);
        let mut selected = vec![(1u8, first)];
        for seed in 2..=255 {
            let candidate = path(41, 9, seed);
            if candidate.bucket_id(policy) == bucket_id {
                selected.push((seed, candidate));
                if selected.len() == needed {
                    break;
                }
            }
        }

        assert_eq!(
            selected.len(),
            needed,
            "failed to find same-bucket split paths"
        );
        for (seed, candidate) in &selected {
            put_item(store, *candidate, 2_100 + u64::from(*seed));
        }
        let paths: Vec<_> = selected.into_iter().map(|(_, path)| path).collect();
        assert!(
            store.split_proof(&paths[0]).is_ok(),
            "split-ready fixture must build a split proof"
        );
        paths
    }

    fn sibling_bucket_pair(store: &mut SettlementStore) -> (SettlementPath, SettlementPath) {
        let mut first_paths = std::collections::BTreeMap::<BucketId, SettlementPath>::new();
        let bucket_bits = store.bucket_policy().bucket_bits();

        for seed in 1..=128 {
            let candidate = path(33, 11, seed);
            put_item(store, candidate, 3_300 + u64::from(seed));
            let bucket = candidate.bucket_id(store.bucket_policy());
            let sibling = sibling_bucket_id(bucket, bucket_bits);
            if let Some(other) = first_paths.get(&sibling).copied() {
                if store.merge_proof(&other, &candidate).is_ok() {
                    return (other, candidate);
                }
            }
            first_paths.entry(bucket).or_insert(candidate);
        }

        panic!("failed to find sibling bucket pair")
    }

    fn next_policy(store: &SettlementStore) -> BucketPolicy {
        BucketPolicy::new(
            store.bucket_policy().bucket_bits() + 1,
            store.bucket_policy().min_bucket_count(),
            store.bucket_policy().max_target_leaf_count(),
            store.bucket_policy().compatibility_generation() + 1,
        )
        .expect("next bucket policy")
    }

    #[test]
    fn test_split_hist_epoch_rejects() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = env_lock().lock().expect("env lock");
        let _env = EnvGuard::live("1");
        let temp = tempdir()?;

        let mut store = SettlementStore::load(temp.path())?;
        let split_paths = split_ready_paths(&mut store);
        let first = split_paths[0];
        let second = split_paths[1];
        let proof = store.split_proof(&first).expect("split proof");
        drop(store);

        let mut reloaded = SettlementStore::load(temp.path())?;
        reloaded
            .validate_split_proof(&proof)
            .expect("reloaded split proof validation");
        put_item(&mut reloaded, path(98, 1, 201), 9_803);
        reloaded
            .validate_split_proof(&proof)
            .expect("historical split proof after reload");

        let tampered_epoch = SplitProof {
            prior_epoch: BucketEpoch::new(proof.prior_epoch.get() + 1),
            ..proof
        };
        let err = reloaded
            .validate_split_proof(&tampered_epoch)
            .expect_err("wrong split epoch must reject");
        assert!(matches!(err, AdaptiveProofErr::WrongEpoch));
        assert_ne!(first, second);

        Ok(())
    }

    #[test]
    fn test_merge_hist_epoch_rejects() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = env_lock().lock().expect("env lock");
        let _env = EnvGuard::live("2");
        let temp = tempdir()?;

        let mut store = SettlementStore::load(temp.path())?;
        let (left, right) = sibling_bucket_pair(&mut store);
        let merge = store.merge_proof(&left, &right).expect("merge proof");
        let next_policy = next_policy(&store);
        let transition = store
            .policy_transition_proof(next_policy)
            .expect("policy transition proof");
        drop(store);

        let mut reloaded = SettlementStore::load(temp.path())?;
        reloaded
            .validate_merge_proof(&merge)
            .expect("reloaded merge proof validation");
        reloaded
            .validate_policy_transition_proof(&transition, next_policy)
            .expect("reloaded policy transition validation");

        put_item(&mut reloaded, path(99, 2, 202), 9_929);
        reloaded
            .validate_merge_proof(&merge)
            .expect("historical merge proof after reload");
        reloaded
            .validate_policy_transition_proof(&transition, next_policy)
            .expect("historical policy transition after reload");

        let tampered_merge = MergeProof {
            prior_epoch: BucketEpoch::new(merge.prior_epoch.get() + 1),
            ..merge
        };
        let err = reloaded
            .validate_merge_proof(&tampered_merge)
            .expect_err("wrong merge epoch must reject");
        assert!(matches!(err, AdaptiveProofErr::WrongEpoch));

        let tampered_transition = PolicyTransitionProof {
            prior_epoch: BucketEpoch::new(transition.prior_epoch.get() + 1),
            ..transition
        };
        let err = reloaded
            .validate_policy_transition_proof(&tampered_transition, next_policy)
            .expect_err("wrong transition epoch must reject");
        assert!(matches!(err, AdaptiveProofErr::WrongEpoch));

        Ok(())
    }

    #[test]
    fn test_baseline_survives_reload() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = env_lock().lock().expect("env lock");
        let _env = EnvGuard::live("1");
        let temp = tempdir()?;

        let mut store = SettlementStore::load(temp.path())?;
        let paths = split_ready_paths(&mut store);
        let proof_paths = paths[..3].to_vec();
        let baseline = store.settlement_proof_blobs(&proof_paths)?;
        let batch = store.settlement_inclusion_batch_v1(&proof_paths)?;
        let batch_bytes = batch.encode()?;
        drop(store);

        let reloaded = SettlementStore::load(temp.path())?;
        let decoded = BatchProofBlobV1::decode(&batch_bytes).expect("decode reloaded batch");
        assert_eq!(decoded.header.settlement_root, reloaded.settlement_root()?);
        assert_eq!(decoded.path_table.len(), proof_paths.len());
        for blob in &baseline {
            reloaded.validate_settlement_proof_blob(blob)?;
            assert_eq!(
                blob.item().settlement_root(),
                decoded.header.settlement_root
            );
        }

        Ok(())
    }
}
