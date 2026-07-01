use z00z_aggregators::BatchId;
use z00z_core::ObjectFamily;
use z00z_storage::settlement::{SettlementActionV1, SettlementStateRoot, VoucherAction};
use z00z_validators::{ObjectRejectCode, ObjectValidatorVerdict, Verdict, VerdictKind};
use z00z_watchers::{AlertKind, AlertSeverity, EvidenceKey, EvidenceRecord, WatcherBoundary};

#[test]
fn test_maps_object_rejects() {
    let batch_id = BatchId::from_bytes([0xA1; 32]);
    let verdict = Verdict {
        batch_id,
        checkpoint_id: None,
        publication: None,
        kind: VerdictKind::Rejected,
        reject: Some(z00z_validators::RejectClass::PolicyUnknown),
        object_verdicts: vec![
            reject(ObjectRejectCode::UnknownPolicy),
            reject(ObjectRejectCode::MissingRight),
        ],
    };

    let alerts = WatcherBoundary.object_alerts(&verdict);

    assert_eq!(alerts.len(), 2);
    assert_eq!(
        alerts[0].kind,
        AlertKind::ObjectReject(ObjectRejectCode::UnknownPolicy)
    );
    assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    assert_eq!(
        alerts[1].kind,
        AlertKind::ObjectReject(ObjectRejectCode::MissingRight)
    );
    assert_eq!(alerts[1].severity, AlertSeverity::Warn);
}

#[test]
fn evidence_exports_object_reject_codes() {
    let batch_id = BatchId::from_bytes([0xA2; 32]);
    let verdict = Verdict {
        batch_id,
        checkpoint_id: None,
        publication: None,
        kind: VerdictKind::Rejected,
        reject: Some(z00z_validators::RejectClass::ProofInvalid),
        object_verdicts: vec![
            reject(ObjectRejectCode::FeeBoundary),
            reject(ObjectRejectCode::DoubleRedeem),
        ],
    };
    let record = EvidenceRecord {
        evidence_key: EvidenceKey {
            batch_id,
            sequence: 9,
        },
        kind: AlertKind::ObjectReject(ObjectRejectCode::FeeBoundary),
        severity: AlertSeverity::Critical,
        subject: z00z_watchers::AlertSubject::Batch(batch_id),
        publication: None,
        published: None,
        soft_confirmation: None,
        placement: None,
        exec_ticket: None,
        verdict: Some(verdict),
        provider_signal: None,
    };

    assert_eq!(
        record.object_reject_codes(),
        vec![
            ObjectRejectCode::FeeBoundary,
            ObjectRejectCode::DoubleRedeem
        ]
    );
}

#[test]
fn test_maps_replay_expiry() {
    let batch_id = BatchId::from_bytes([0xA3; 32]);
    let verdict = Verdict {
        batch_id,
        checkpoint_id: None,
        publication: None,
        kind: VerdictKind::Rejected,
        reject: Some(z00z_validators::RejectClass::ReplayConflict),
        object_verdicts: vec![
            reject(ObjectRejectCode::Replay),
            reject(ObjectRejectCode::RightExpired),
        ],
    };

    let alerts = WatcherBoundary.object_alerts(&verdict);

    assert_eq!(alerts.len(), 2);
    assert_eq!(
        alerts[0].kind,
        AlertKind::ObjectReject(ObjectRejectCode::Replay)
    );
    assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    assert_eq!(
        alerts[1].kind,
        AlertKind::ObjectReject(ObjectRejectCode::RightExpired)
    );
    assert_eq!(alerts[1].severity, AlertSeverity::Warn);
}

fn reject(code: ObjectRejectCode) -> ObjectValidatorVerdict {
    ObjectValidatorVerdict {
        family: ObjectFamily::Voucher,
        selected_action: SettlementActionV1::Voucher(VoucherAction::RedeemFull),
        policy_descriptor_hash: [0x11; 32],
        action_pool_id: [0x12; 32],
        selected_action_id: [0x13; 32],
        prior_root: SettlementStateRoot::settlement_v1([0x21; 32]),
        expected_new_root: SettlementStateRoot::settlement_v1([0x22; 32]),
        reject: Some(code),
    }
}
