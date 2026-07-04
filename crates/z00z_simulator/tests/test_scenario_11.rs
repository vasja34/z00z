use tempfile::tempdir;
use z00z_simulator::scenario_11::{
    report::{
        CommitSubjectReport, FaultMatrixReport, LocalDaBindingReport, PackageIngressReport,
        PlacementMembershipReport, QuorumCertificateReport, ReportHonesty, RoutePlanReport,
        SecondaryReplayVotesReport, ValidatorVerdictReport,
    },
    run,
};
use z00z_utils::io::load_json;

#[test]
fn scenario11_happy_path_artifacts_are_subject_consistent() {
    let temp = tempdir().expect("tempdir");
    let run = run(temp.path()).expect("scenario_11 run");
    let root = run.artifact_root();

    let ingress: PackageIngressReport =
        load_json(root.join("package_ingress_report.json")).expect("ingress report");
    let route: RoutePlanReport =
        load_json(root.join("route_plan_report.json")).expect("route report");
    let placement: PlacementMembershipReport =
        load_json(root.join("placement_membership.json")).expect("placement report");
    let subject: CommitSubjectReport =
        load_json(root.join("commit_subject.json")).expect("subject report");
    let votes: SecondaryReplayVotesReport =
        load_json(root.join("secondary_replay_votes.json")).expect("votes report");
    let qc: QuorumCertificateReport =
        load_json(root.join("quorum_certificate.json")).expect("qc report");
    let da: LocalDaBindingReport =
        load_json(root.join("local_da_binding.json")).expect("da report");
    let verdict: ValidatorVerdictReport =
        load_json(root.join("validator_verdict_report.json")).expect("verdict report");

    assert!(root.join("fault_matrix.json").exists());
    assert!(root.join("report_honesty.json").exists());

    assert!(ingress.ingress_recomputed_digest);
    assert_eq!(ingress.package_digest_hex, ingress.route_key_hex);
    assert_eq!(ingress.batch_id_hex, subject.batch_id_hex);
    assert_eq!(route.happy_path.shard_id, ingress.shard_id);
    assert_eq!(
        route.happy_path.route_table_digest_hex,
        subject.route_table_digest_hex
    );
    assert_eq!(route.happy_path.batch_id_hex, subject.batch_id_hex);
    assert_eq!(
        placement.happy_path.membership_digest_hex,
        subject.membership_digest_hex
    );
    assert_eq!(
        qc.happy_path.subject_digest_hex,
        subject.subject_digest_hex
    );
    assert_eq!(
        qc.happy_path.membership_digest_hex,
        subject.membership_digest_hex
    );
    assert_eq!(
        da.publication_binding_digest_hex,
        subject.publication_binding_digest_hex
    );
    assert!(da.resumed_same_certificate);
    assert_eq!(verdict.verdict_kind, "accepted");
    assert_eq!(verdict.subject_digest_hex, subject.subject_digest_hex);
    assert_eq!(
        verdict.certificate_digest_hex,
        qc.happy_path.certificate_digest_hex
    );
    assert_eq!(route.all_shard_sweep.len(), 7);
    assert_eq!(placement.all_shard_sweep.len(), 7);
    assert_eq!(route.dual_primary_owner.shard_ids.len(), 2);
    assert_eq!(
        route.dual_primary_owner.membership_digests_hex.len(),
        route.dual_primary_owner.certificate_digests_hex.len()
    );
    assert!(
        route.dual_primary_owner.membership_digests_hex[0]
            != route.dual_primary_owner.membership_digests_hex[1]
    );
    assert!(
        route.dual_primary_owner.certificate_digests_hex[0]
            != route.dual_primary_owner.certificate_digests_hex[1]
    );
    assert!(
        votes.happy_path_votes
            .iter()
            .all(|vote| vote.verdict == "accept")
    );
}

#[test]
fn scenario11_fault_matrix_covers_rejects_and_crash_paths() {
    let temp = tempdir().expect("tempdir");
    let run = run(temp.path()).expect("scenario_11 run");
    let root = run.artifact_root();

    let faults: FaultMatrixReport =
        load_json(root.join("fault_matrix.json")).expect("fault matrix");
    let votes: SecondaryReplayVotesReport =
        load_json(root.join("secondary_replay_votes.json")).expect("votes report");

    let reject_ids = [
        "wrong_route_digest",
        "wrong_generation",
        "wrong_plan_digest",
        "wrong_state_root",
        "wrong_proof_version",
        "wrong_publication_binding",
        "wrong_theorem_digest",
    ];
    for fault_id in reject_ids {
        let entry = faults
            .entries
            .iter()
            .find(|entry| entry.fault_id == fault_id)
            .expect("fault entry");
        assert_eq!(entry.observed_status, "rejected_as_expected");
        assert!(entry.reject_code.is_some());
    }

    let pre_quorum = faults
        .entries
        .iter()
        .find(|entry| entry.fault_id == "primary_crash_before_quorum")
        .expect("pre quorum crash");
    assert_eq!(pre_quorum.observed_status, "rejected_as_expected");

    let offline_primary = faults
        .entries
        .iter()
        .find(|entry| entry.fault_id == "primary_offline_before_dispatch")
        .expect("offline primary");
    assert_eq!(offline_primary.observed_status, "deferred_as_expected");

    let post_quorum = faults
        .entries
        .iter()
        .find(|entry| entry.fault_id == "primary_crash_after_quorum_before_da")
        .expect("post quorum crash");
    assert_eq!(post_quorum.observed_status, "resumed_same_certificate");

    assert!(
        votes.offline_case_votes
            .iter()
            .any(|vote| vote.verdict == "offline")
    );
    assert!(
        votes.stale_case_votes
            .iter()
            .any(|vote| vote.reject_code.as_deref() == Some("StaleSecondaryState"))
    );
}

#[test]
fn scenario11_report_honesty_rejects_overclaims() {
    let temp = tempdir().expect("tempdir");
    let run = run(temp.path()).expect("scenario_11 run");
    let honesty: ReportHonesty =
        load_json(run.artifact_root().join("report_honesty.json")).expect("honesty report");

    assert!(honesty.supported_claims.iter().any(|line| line.contains("local per-shard 2-of-3 CFT quorum")));
    assert!(honesty
        .forbidden_claims
        .iter()
        .any(|line| line.contains("network BFT")));
    assert!(honesty
        .forbidden_claims
        .iter()
        .any(|line| line.contains("Celestia finality")));
    assert!(honesty
        .deferred_claims
        .iter()
        .any(|line| line.contains("067-07")));
}
