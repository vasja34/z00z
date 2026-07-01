// ============================================================================
// 4. TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_noncurrent_on_wrap() {
        let km = RedbKeyManager::new();

        let mut params = KdfParams::default_argon2id_with_salt(vec![1u8; 16]);
        params.version = 1;
        let master_key = km.generate_master_key();

        let err = km
            .wrap_master_key(b"wallet-1", &SafePassword::from("pw"), &master_key, &params)
            .expect_err("v1 kdf must be rejected");

        assert!(matches!(err, RedbKeyManagerError::InvalidParameters(_)));
    }

    #[test]
    fn test_unwrap_rejects_record_params() {
        let km = RedbKeyManager::new();

        let params = KdfParams::default_argon2id_with_salt(vec![1u8; 16]);
        let master_key = km.generate_master_key();
        let mut record = km
            .wrap_master_key(
                b"wallet-1",
                &SafePassword::from("pw"),
                &master_key,
                &params,
            )
            .expect("wrap master key");

        record.kdf_params.as_mut().unwrap().version = 1;

        let err = km
            .unwrap_master_key(
                b"wallet-1",
                &SafePassword::from("pw"),
                record.kdf_params.as_ref().unwrap(),
                &record,
            )
            .expect_err("v1 persisted params must be rejected");

        assert!(matches!(err, RedbKeyManagerError::InvalidParameters(_)));
    }

    #[test]
    fn test_kdf_params_mismatch_rejected() {
        let km = RedbKeyManager::new();

        let params_a = KdfParams::default_argon2id_with_salt(vec![1u8; 16]);
        let params_b = KdfParams::default_argon2id_with_salt(vec![2u8; 16]);

        let master_key = km.generate_master_key();
        let record = km
            .wrap_master_key(
                b"wallet-1",
                &SafePassword::from("pw"),
                &master_key,
                &params_a,
            )
            .expect("wrap master key");

        let err = km
            .unwrap_master_key(b"wallet-1", &SafePassword::from("pw"), &params_b, &record)
            .expect_err("mismatch must fail");

        assert!(matches!(
            err,
            RedbKeyManagerError::InvalidParameters(ref msg) if msg.contains("kdf params")
        ));
    }
}