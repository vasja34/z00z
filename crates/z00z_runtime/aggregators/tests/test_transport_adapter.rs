#[path = "test_transport_common.rs"]
mod test_transport_common;

use z00z_aggregators::{
    InMemoryVoteTransport, ReplayVerifiedVoteService, ShardVoteKind, TransportPayloadStatus,
    VoteEvidence, VoteExchangeOutcome, VoteTransport, VoteTransportEnvelope,
};

use self::test_transport_common::{transport_fixture, vote_exchange_context};

#[test]
fn test_queue_order() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let mut transport = InMemoryVoteTransport::default();
    let delayed = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[0].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );
    let immediate = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[1].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );
    transport.enqueue_delayed(delayed.clone(), 2);
    transport.enqueue_front(immediate.clone());

    let first = transport.step();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].message_id, immediate.message_id);

    transport.requeue(immediate.clone(), 2);
    let second = transport.step();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].message_id, delayed.message_id);

    let third = transport.step();
    assert_eq!(third.len(), 1);
    assert_eq!(third[0].message_id, immediate.message_id);

    Ok(())
}

#[test]
fn test_replay_gate() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let mut service = ReplayVerifiedVoteService::local();
    let envelope = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[0].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );
    let accepted = service.process_envelope(
        &envelope,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[0].aggregator_id,
            &fixture.subject,
        ),
    );
    match accepted.outcome {
        VoteExchangeOutcome::Vote(vote) => {
            assert!(vote.has_valid_signature());
            assert_eq!(vote.voter_id, fixture.ready_secondaries[0].aggregator_id);
        }
        other => panic!("expected vote, got {other:?}"),
    }

    let duplicate = service.process_envelope(
        &envelope,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[0].aggregator_id,
            &fixture.subject,
        ),
    );
    assert_eq!(duplicate.outcome, VoteExchangeOutcome::DuplicateMessage);

    let mut drifted_subject = fixture.subject.clone();
    drifted_subject.theorem_or_settlement_digest = [0x77; 32];
    let drifted_envelope = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[1].aggregator_id,
        drifted_subject,
        ShardVoteKind::LocalCommit,
    );
    let rejected = service.process_envelope(
        &drifted_envelope,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[1].aggregator_id,
            &fixture.subject,
        ),
    );
    match rejected.outcome {
        VoteExchangeOutcome::ReplayRejected(reject) => {
            assert!(matches!(
                reject.code,
                z00z_aggregators::SecondaryReplayRejectCode::WrongTheoremDigest
            ));
        }
        other => panic!("expected replay rejection, got {other:?}"),
    }

    Ok(())
}

#[test]
fn test_payload_evidence() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let mut service = ReplayVerifiedVoteService::local();
    let envelope = VoteTransportEnvelope::missing_payload(
        fixture.primary,
        fixture.ready_secondaries[0].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
        "payload missing before replay",
    );
    assert!(matches!(
        envelope.payload_status,
        TransportPayloadStatus::Missing { .. }
    ));

    let result = service.process_envelope(
        &envelope,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[0].aggregator_id,
            &fixture.subject,
        ),
    );
    match result.outcome {
        VoteExchangeOutcome::Evidence(VoteEvidence::PayloadWithholding(evidence)) => {
            assert_eq!(evidence.accused_id, fixture.primary);
            assert_eq!(
                evidence.reporter_id,
                fixture.ready_secondaries[0].aggregator_id
            );
            assert_eq!(evidence.subject_digest, fixture.subject.digest());
            assert_eq!(evidence.payload_digest, fixture.subject.payload_digest);
        }
        other => panic!("expected payload-withholding evidence, got {other:?}"),
    }
    assert_eq!(service.evidence_records().len(), 1);

    Ok(())
}
