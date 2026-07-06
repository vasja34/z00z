#[path = "test_transport_common.rs"]
mod test_transport_common;

use z00z_aggregators::{
    ConsensusAdapter, InMemoryVoteTransport, ReplayVerifiedVoteService, ShardPlacementTable,
    ShardVoteKind, ShardVoteRole, TransportDeliveryPlan, TransportFaultEvidenceKind,
    VoteExchangeOutcome, VoteTransport, VoteTransportEnvelope,
};

use self::test_transport_common::{
    local_vote, transport_fixture, vote_exchange_context, vote_exchange_context_with_placement,
};

#[test]
fn test_fault_scheduler() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let mut transport = InMemoryVoteTransport::default();
    let immediate = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[0].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );
    let delayed = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[1].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );
    let dropped = VoteTransportEnvelope::missing_payload(
        fixture.primary,
        fixture.ready_secondaries[0].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::Commit,
        "scheduled drop",
    );

    transport.enqueue_planned(delayed.clone(), TransportDeliveryPlan::delayed(2));
    transport.enqueue_planned(
        immediate.clone(),
        TransportDeliveryPlan::default()
            .front_of_queue()
            .with_duplicate_after(2),
    );
    transport.enqueue_planned(dropped, TransportDeliveryPlan::default().drop_on_delivery());

    let first = transport.step();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].message_id, immediate.message_id);

    let second = transport.step();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].message_id, delayed.message_id);

    let third = transport.step();
    assert_eq!(third.len(), 1);
    assert_eq!(third[0].message_id, immediate.message_id);

    let kinds = transport
        .fault_records()
        .iter()
        .map(|entry| entry.kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TransportFaultEvidenceKind::Delay));
    assert!(kinds.contains(&TransportFaultEvidenceKind::Reorder));
    assert!(kinds.contains(&TransportFaultEvidenceKind::Duplicate));
    assert!(kinds.contains(&TransportFaultEvidenceKind::Drop));

    Ok(())
}

#[test]
fn test_replay_bound() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let mut transport = InMemoryVoteTransport::default();
    let mut service = ReplayVerifiedVoteService::local();
    let envelope = VoteTransportEnvelope::available(
        fixture.primary,
        fixture.ready_secondaries[0].aggregator_id,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );
    transport.enqueue_planned(
        envelope,
        TransportDeliveryPlan::default()
            .with_duplicate_after(1)
            .with_replay_after(2),
    );

    let first = transport.step().into_iter().next().expect("first delivery");
    let first_result = service.process_envelope(
        &first,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[0].aggregator_id,
            &fixture.subject,
        ),
    );
    assert!(matches!(first_result.outcome, VoteExchangeOutcome::Vote(_)));

    let duplicate = transport
        .step()
        .into_iter()
        .next()
        .expect("duplicate delivery");
    let duplicate_result = service.process_envelope(
        &duplicate,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[0].aggregator_id,
            &fixture.subject,
        ),
    );
    assert_eq!(
        duplicate_result.outcome,
        VoteExchangeOutcome::DuplicateMessage
    );

    let replay = transport
        .step()
        .into_iter()
        .next()
        .expect("replay delivery");
    let replay_result = service.process_envelope(
        &replay,
        vote_exchange_context(
            &fixture,
            fixture.ready_secondaries[0].aggregator_id,
            &fixture.subject,
        ),
    );
    assert_eq!(replay_result.outcome, VoteExchangeOutcome::DuplicateMessage);
    assert!(service.evidence_records().is_empty());

    let kinds = transport
        .fault_records()
        .iter()
        .map(|entry| entry.kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TransportFaultEvidenceKind::Duplicate));
    assert!(kinds.contains(&TransportFaultEvidenceKind::Replay));

    Ok(())
}

#[test]
fn test_partition_heal() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let recipient = fixture.ready_secondaries[0].aggregator_id;
    let mut transport = InMemoryVoteTransport::default();
    let mut service = ReplayVerifiedVoteService::local();
    let envelope = VoteTransportEnvelope::available(
        fixture.primary,
        recipient,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );

    transport.partition_peer(recipient);
    transport.enqueue(envelope);
    assert!(transport.step().is_empty());

    transport.heal_peer(recipient);
    let delivered = transport
        .step()
        .into_iter()
        .next()
        .expect("delivery after heal");
    let result = service.process_envelope(
        &delivered,
        vote_exchange_context(&fixture, recipient, &fixture.subject),
    );
    assert!(matches!(result.outcome, VoteExchangeOutcome::Vote(_)));

    let kinds = transport
        .fault_records()
        .iter()
        .map(|entry| entry.kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TransportFaultEvidenceKind::PartitionDeferred));
    assert!(kinds.contains(&TransportFaultEvidenceKind::Heal));

    Ok(())
}

#[test]
fn test_minority_no_qc() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let mut transport = InMemoryVoteTransport::default();
    let mut adapter =
        ConsensusAdapter::from_placement(&fixture.placement).expect("placement adapter");

    for secondary in &fixture.ready_secondaries {
        transport.partition_peer(secondary.aggregator_id);
        transport.enqueue(VoteTransportEnvelope::available(
            fixture.primary,
            secondary.aggregator_id,
            fixture.subject.clone(),
            ShardVoteKind::LocalCommit,
        ));
    }

    assert!(transport.step().is_empty());
    let err = adapter
        .commit(
            &fixture.subject,
            &[local_vote(
                &fixture.subject,
                fixture.primary,
                ShardVoteRole::Primary,
            )],
        )
        .expect_err("minority side must not form a certificate");
    assert!(err.detail.contains("below quorum"));

    Ok(())
}

#[test]
fn test_restart_reconnect() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = transport_fixture()?;
    let recipient = fixture.ready_secondaries[0].aggregator_id;
    let mut transport = InMemoryVoteTransport::default();
    let mut service = ReplayVerifiedVoteService::local();
    let envelope = VoteTransportEnvelope::available(
        fixture.primary,
        recipient,
        fixture.subject.clone(),
        ShardVoteKind::LocalCommit,
    );

    transport.restart_peer(recipient);
    transport.reconnect_peer(recipient);
    transport.enqueue(envelope);

    let delivered = transport
        .step()
        .into_iter()
        .next()
        .expect("delivery after reconnect");
    let mut drifted_placement = fixture.placement.clone();
    drifted_placement
        .secondaries
        .retain(|secondary| secondary.aggregator_id == recipient);
    let mut drifted_table = ShardPlacementTable::default();
    drifted_table.insert(drifted_placement);

    let result = service.process_envelope(
        &delivered,
        vote_exchange_context_with_placement(&fixture, &drifted_table, recipient, &fixture.subject),
    );
    match result.outcome {
        VoteExchangeOutcome::ReplayRejected(reject) => {
            assert!(matches!(
                reject.code,
                z00z_aggregators::SecondaryReplayRejectCode::MembershipDrift
            ));
        }
        other => panic!("expected membership-drift rejection, got {other:?}"),
    }

    let kinds = transport
        .fault_records()
        .iter()
        .map(|entry| entry.kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&TransportFaultEvidenceKind::Restart));
    assert!(kinds.contains(&TransportFaultEvidenceKind::Reconnect));

    Ok(())
}
