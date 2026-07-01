use z00z_storage::fixture_support::checkpoint_fixtures;

use z00z_storage::{
    checkpoint::audit::{decode_audit_bin, decode_audit_json, encode_audit_bin, encode_audit_json},
    checkpoint::{
        decode_art_bin, decode_art_json, decode_draft_bin, decode_draft_json, decode_exec_bin,
        decode_exec_json, decode_link_bin, decode_link_json, encode_art_bin, encode_art_json,
        encode_draft_bin, encode_draft_json, encode_exec_bin, encode_exec_json, encode_link_bin,
        encode_link_json, CheckpointExecInputId, CheckpointId,
    },
    CheckpointError,
};

#[test]
fn test_json_roundtrip_keeps_types() {
    assert_eq!(
        decode_draft_json(&encode_draft_json(&checkpoint_fixtures::draft()).expect("draft json"))
            .expect("draft"),
        checkpoint_fixtures::draft()
    );
    assert_eq!(
        decode_art_json(&encode_art_json(&checkpoint_fixtures::artifact()).expect("art json"))
            .expect("artifact"),
        checkpoint_fixtures::artifact()
    );
    assert_eq!(
        decode_link_json(
            &encode_link_json(&checkpoint_fixtures::link(
                CheckpointId::new([6u8; 32]),
                CheckpointExecInputId::new([8u8; 32]),
            ))
            .expect("link json"),
        )
        .expect("link"),
        checkpoint_fixtures::link(
            CheckpointId::new([6u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        )
    );
    assert_eq!(
        decode_exec_json(&encode_exec_json(&checkpoint_fixtures::exec()).expect("exec json"))
            .expect("exec"),
        checkpoint_fixtures::exec()
    );
    assert_eq!(
        decode_audit_json(
            &encode_audit_json(&checkpoint_fixtures::audit(CheckpointId::new([1u8; 32])))
                .expect("audit json"),
        )
        .expect("audit"),
        checkpoint_fixtures::audit(CheckpointId::new([1u8; 32]))
    );
}

#[test]
fn test_bin_roundtrip_keeps_types() {
    assert_eq!(
        decode_draft_bin(&encode_draft_bin(&checkpoint_fixtures::draft()).expect("draft bin"))
            .expect("draft"),
        checkpoint_fixtures::draft()
    );
    assert_eq!(
        decode_art_bin(&encode_art_bin(&checkpoint_fixtures::artifact()).expect("art bin"))
            .expect("artifact"),
        checkpoint_fixtures::artifact()
    );
    assert_eq!(
        decode_link_bin(
            &encode_link_bin(&checkpoint_fixtures::link(
                CheckpointId::new([6u8; 32]),
                CheckpointExecInputId::new([8u8; 32]),
            ))
            .expect("link bin"),
        )
        .expect("link"),
        checkpoint_fixtures::link(
            CheckpointId::new([6u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        )
    );
    assert_eq!(
        decode_exec_bin(&encode_exec_bin(&checkpoint_fixtures::exec()).expect("exec bin"))
            .expect("exec"),
        checkpoint_fixtures::exec()
    );
    assert_eq!(
        decode_audit_bin(
            &encode_audit_bin(&checkpoint_fixtures::audit(CheckpointId::new([1u8; 32])))
                .expect("audit bin"),
        )
        .expect("audit"),
        checkpoint_fixtures::audit(CheckpointId::new([1u8; 32]))
    );
}

#[test]
fn test_wrong_class_payload_rejects() {
    let bytes = encode_audit_json(&checkpoint_fixtures::audit(CheckpointId::new([1u8; 32])))
        .expect("audit json");
    let err = decode_art_json(&bytes).expect_err("audit must not decode as artifact");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_prior_stage6_wrapper_rejects() {
    let bytes = checkpoint_fixtures::prior_stage6_json();

    let draft_err = decode_draft_json(&bytes).expect_err("prior wrapper must not load as draft");
    let art_err = decode_art_json(&bytes).expect_err("prior wrapper must not load as artifact");

    assert!(matches!(draft_err, CheckpointError::Codec(_)));
    assert!(matches!(art_err, CheckpointError::Codec(_)));
}

#[test]
fn test_malformed_transport_rejects() {
    let err = decode_exec_json(br#"{"version":1,"prev_root":"bad"}"#).expect_err("bad transport");

    assert!(matches!(err, CheckpointError::Codec(_)));
}
