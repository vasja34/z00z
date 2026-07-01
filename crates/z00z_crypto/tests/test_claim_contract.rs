use rand::thread_rng;
use tari_crypto::keys::SecretKey as _;
use tari_crypto::ristretto::RistrettoSecretKey;

use z00z_crypto::domains::ClaimStmtDomain;
use z00z_crypto::{
    claim_stmt_hash, ClaimAuthoritySig, ClaimError, ClaimProofVer, ClaimSourceProof, ClaimStmt,
    DomainHasher256, CLAIM_ROOT_VERSION,
};

fn fill(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn test_stmt() -> ClaimStmt {
    ClaimStmt {
        chain_id: 0x0102_0304,
        root_version: CLAIM_ROOT_VERSION,
        proof_ver: ClaimProofVer::V1,
        tx_ver: 2,
        range_ctx_hash: fill(0x11),
        claim_id: fill(0x22),
        claim_source_asset_id: fill(0x33),
        claim_source_commitment: fill(0x44),
        source_root: fill(0x45),
        claim_scope_hash: fill(0x46),
        recipient_binding: fill(0x55),
        nullifier: fill(0x66),
        owner_bind_digest: fill(0x77),
        output_leaf_hashes: vec![fill(0x88), fill(0x99)],
    }
}

fn assert_sig_invalid_for_mutation(
    auth: &ClaimAuthoritySig,
    stmt: &ClaimStmt,
    label: &str,
    mutate: impl FnOnce(&mut ClaimStmt),
) {
    let mut wrong = stmt.clone();
    mutate(&mut wrong);
    let err = auth.verify(&wrong).expect_err(label);
    assert_eq!(err, ClaimError::SigInvalid, "{label}");
}

#[test]
fn test_claim_frame_vector() {
    let stmt = test_stmt();
    let mut want = Vec::new();

    want.extend_from_slice(b"CLM2");
    want.extend_from_slice(&0x0102_0304u32.to_le_bytes());
    want.push(CLAIM_ROOT_VERSION);
    want.push(ClaimProofVer::V1.as_u8());
    want.extend_from_slice(&2u32.to_le_bytes());
    want.extend_from_slice(&fill(0x11));
    want.extend_from_slice(&fill(0x22));
    want.extend_from_slice(&fill(0x33));
    want.extend_from_slice(&fill(0x44));
    want.extend_from_slice(&fill(0x45));
    want.extend_from_slice(&fill(0x46));
    want.extend_from_slice(&fill(0x55));
    want.extend_from_slice(&fill(0x66));
    want.extend_from_slice(&fill(0x77));
    want.extend_from_slice(&2u32.to_le_bytes());
    want.extend_from_slice(&fill(0x88));
    want.extend_from_slice(&fill(0x99));

    let bytes = stmt.to_bytes().expect("claim contract bytes");
    assert_eq!(bytes, want);
    assert_eq!(
        ClaimStmt::from_bytes(&bytes).expect("claim contract decode"),
        stmt
    );
}

#[test]
fn test_claim_hash_contract_label() {
    let stmt = test_stmt();
    let bytes = stmt.to_bytes().expect("claim contract bytes");
    let actual = claim_stmt_hash(&stmt).expect("claim contract hash");

    let expected_hash = DomainHasher256::<ClaimStmtDomain>::new_with_label("claim_contract")
        .chain(&bytes)
        .finalize();
    let legacy_hash = DomainHasher256::<ClaimStmtDomain>::new_with_label("claim_contract_legacy")
        .chain(&bytes)
        .finalize();

    let mut expected = [0u8; 32];
    expected.copy_from_slice(expected_hash.as_ref());

    let mut legacy = [0u8; 32];
    legacy.copy_from_slice(legacy_hash.as_ref());

    assert_eq!(actual, expected);
    assert_ne!(actual, legacy);
}

#[test]
fn test_claim_sig_check() {
    let stmt = test_stmt();
    let mut rng = thread_rng();
    let auth_sk = RistrettoSecretKey::random(&mut rng);
    let auth = ClaimAuthoritySig::sign(&stmt, &auth_sk, &mut rng).expect("claim contract sign");

    auth.verify(&stmt).expect("claim contract verify");

    assert_sig_invalid_for_mutation(&auth, &stmt, "chain_id mutation must fail", |wrong| {
        wrong.chain_id ^= 1;
    });
    assert_sig_invalid_for_mutation(&auth, &stmt, "root_version mutation must fail", |wrong| {
        wrong.root_version = 3;
    });
    assert_sig_invalid_for_mutation(&auth, &stmt, "proof_ver mutation must fail", |wrong| {
        wrong.proof_ver = ClaimProofVer::V2;
    });
    assert_sig_invalid_for_mutation(&auth, &stmt, "tx_ver mutation must fail", |wrong| {
        wrong.tx_ver += 1;
    });
    assert_sig_invalid_for_mutation(&auth, &stmt, "range_ctx_hash mutation must fail", |wrong| {
        wrong.range_ctx_hash[0] ^= 1;
    });
    assert_sig_invalid_for_mutation(&auth, &stmt, "claim_id mutation must fail", |wrong| {
        wrong.claim_id[0] ^= 1;
    });
    assert_sig_invalid_for_mutation(
        &auth,
        &stmt,
        "claim_source_asset_id mutation must fail",
        |wrong| {
            wrong.claim_source_asset_id[0] ^= 1;
        },
    );
    assert_sig_invalid_for_mutation(
        &auth,
        &stmt,
        "claim_source_commitment mutation must fail",
        |wrong| {
            wrong.claim_source_commitment[0] ^= 1;
        },
    );
    assert_sig_invalid_for_mutation(&auth, &stmt, "source_root mutation must fail", |wrong| {
        wrong.source_root[0] ^= 1;
    });
    assert_sig_invalid_for_mutation(
        &auth,
        &stmt,
        "claim_scope_hash mutation must fail",
        |wrong| {
            wrong.claim_scope_hash[0] ^= 1;
        },
    );
    assert_sig_invalid_for_mutation(
        &auth,
        &stmt,
        "recipient_binding mutation must fail",
        |wrong| {
            wrong.recipient_binding[0] ^= 1;
        },
    );
    assert_sig_invalid_for_mutation(&auth, &stmt, "nullifier mutation must fail", |wrong| {
        wrong.nullifier[0] ^= 1;
    });
    assert_sig_invalid_for_mutation(
        &auth,
        &stmt,
        "owner_bind_digest mutation must fail",
        |wrong| {
            wrong.owner_bind_digest[0] ^= 1;
        },
    );
    assert_sig_invalid_for_mutation(
        &auth,
        &stmt,
        "output leaf hash mutation must fail",
        |wrong| {
            wrong.output_leaf_hashes[0][0] ^= 1;
        },
    );
    assert_sig_invalid_for_mutation(
        &auth,
        &stmt,
        "output leaf count mutation must fail",
        |wrong| {
            wrong.output_leaf_hashes.pop();
        },
    );
}

#[test]
fn test_claim_version_mix() {
    let stmt = test_stmt();
    let root_mix = ClaimSourceProof::new(3, fill(0xAB), ClaimProofVer::V1, vec![1u8, 2, 3])
        .expect("root-mix proof");
    let proof_mix = ClaimSourceProof::new(
        CLAIM_ROOT_VERSION,
        fill(0xCD),
        ClaimProofVer::V2,
        vec![4u8, 5, 6],
    )
    .expect("proof-mix proof");

    assert_eq!(
        stmt.chk_source(&root_mix),
        Err(ClaimError::RootVersionMismatch)
    );
    assert_eq!(stmt.chk_source(&proof_mix), Err(ClaimError::ProofVerMix));
}

#[test]
fn test_claim_root_mismatch() {
    let stmt = test_stmt();
    let root_mix = ClaimSourceProof::new(
        CLAIM_ROOT_VERSION,
        fill(0xEE),
        ClaimProofVer::V1,
        vec![1u8, 2, 3],
    )
    .expect("root-mix proof");

    assert_eq!(stmt.chk_source(&root_mix), Err(ClaimError::SourceRootMix));
}
