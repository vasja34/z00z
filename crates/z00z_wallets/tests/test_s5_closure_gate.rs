use std::{fs, path::PathBuf};

const CLAIM_TX: &str = include_str!("../src/tx/claim_tx.rs");
const OUTPUT_TX: &str = include_str!("../src/tx/tx_output.rs");
const WITNESS_TX: &str = include_str!("../src/tx/witness_gate.rs");
const RNG_TRAITS_SRC: &str = include_str!("../../z00z_utils/src/rng/traits.rs");
const RNG_DETERMINISTIC_SRC: &str = include_str!("../../z00z_utils/src/rng/deterministic.rs");
const CONFIG_SRC: &str = include_str!("../../z00z_simulator/src/config.rs");
const RNG_MODE_SRC: &str = include_str!("../../z00z_simulator/src/rng_mode.rs");
const STAGE2_TRANSPORT_SRC: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_2/transport.rs");
const TX_VALIDATION_GATES: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_6/tx_validation_gates.rs");
const CLAIM_SUPPORT: &str =
    include_str!("../../z00z_simulator/tests/scenario_1/claim_pkg_crypto.rs");

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn load_optional_spec(path: &str) -> Option<String> {
    let spec = repo_root().join(path);
    fs::read_to_string(spec).ok()
}

fn has_all(text: &str, need: &[&str]) {
    for item in need {
        assert!(text.contains(item), "missing required text: {item}");
    }
}

fn has_none(text: &str, forbid: &[&str]) {
    for item in forbid {
        assert!(!text.contains(item), "forbidden text present: {item}");
    }
}

fn has_no_overclaim(text: &str) {
    has_none(
        text,
        &[
            "is a final public verifier",
            "is the final public verifier",
            "are final public verifiers",
            "is protocol-complete",
            "are protocol-complete",
            "authenticated manifest authority already exists",
            "final authority closure is complete",
            "final public spend verifier is live",
            "final public fee verifier is live",
        ],
    );
}

fn closure_slice(spec: &str) -> &str {
    let start = spec
        .find("## Phase 3. Publication And Closure Hygiene")
        .expect("closure section start");
    let end = spec
        .find("## Phase 4. Cross-Spec Handoff")
        .expect("closure section end");
    &spec[start..end]
}

#[test]
fn test_s5_track_map() {
    has_all(
        CLAIM_TX,
        &[
            "const ZERO_ROOT: [u8; 32] = [0u8; 32];",
            "impl ClaimTxVerifier for ClaimTxVerifierImpl",
            "fn verify_claim_proof(",
            "fn verify_claim_authority(",
            "fn verify_owner_attest(",
            "fn verify_digest(",
        ],
    );
    has_all(CLAIM_SUPPORT, &["const ZERO_ROOT: [u8; 32] = [0u8; 32];"]);
    has_all(WITNESS_TX, &["pub fn verify_spend_witness_gate("]);
    has_all(OUTPUT_TX, &["pub fn verify_plaintext_balance_with_fee("]);
    has_all(
        TX_VALIDATION_GATES,
        &[
            "fn verify_fee_matches_formula(",
            "fn verify_spend_witness_gate(",
            "fn verify_tx_package(",
        ],
    );
}

#[test]
fn test_s5_honest_docs() {
    let Some(post_spec) = load_optional_spec("specs/011-z00z-ecc-spec-5/post-spec-5.md") else {
        eprintln!("skip test_s5_honest_docs: missing specs/011-z00z-ecc-spec-5/post-spec-5.md");
        return;
    };
    let Some(stage3_map) = load_optional_spec("specs/011-z00z-ecc-spec-5/stage-3-proof-matrix.md")
    else {
        eprintln!(
            "skip test_s5_honest_docs: missing specs/011-z00z-ecc-spec-5/stage-3-proof-matrix.md"
        );
        return;
    };
    let Some(stage3_note) = load_optional_spec("specs/011-z00z-ecc-spec-5/stage-3-frozen-model.md")
    else {
        eprintln!(
            "skip test_s5_honest_docs: missing specs/011-z00z-ecc-spec-5/stage-3-frozen-model.md"
        );
        return;
    };
    let Some(stage4_note) =
        load_optional_spec("specs/011-z00z-ecc-spec-5/stage4-trust-boundary.md")
    else {
        eprintln!(
            "skip test_s5_honest_docs: missing specs/011-z00z-ecc-spec-5/stage4-trust-boundary.md"
        );
        return;
    };
    let Some(stage4_map) =
        load_optional_spec("specs/011-z00z-ecc-spec-5/stage4-verifier-matrix.md")
    else {
        eprintln!(
            "skip test_s5_honest_docs: missing specs/011-z00z-ecc-spec-5/stage4-verifier-matrix.md"
        );
        return;
    };
    let Some(this_spec) = load_optional_spec("specs/012-z00z-ecc-spec-6/E2E-spec56.md") else {
        eprintln!("skip test_s5_honest_docs: missing specs/012-z00z-ecc-spec-6/E2E-spec56.md");
        return;
    };

    let phase3 = closure_slice(&this_spec);

    has_all(
        &post_spec,
        &[
            "typed placeholder claim-proof path",
            "ZERO_ROOT",
            "verify_spend_witness_gate",
            "verify_fee_matches_formula",
            "verify_plaintext_balance_with_fee",
            "still witness-bound",
        ],
    );
    has_all(
        phase3,
        &[
            "still transitional",
            "placeholder authority is not final authority closure",
            "transitional witness acceptance is not equivalent to final public verification",
            "subordinate to `post-spec-5.md`",
        ],
    );

    has_all(
        &stage3_map,
        &[
            "does not promote deferred Stage-1 manifest authority",
            "still the first typed placeholder slice",
            "verify_claim_authority",
            "test_malformed_proof_rejected",
            "test_owner_attest_mismatch_rejected",
        ],
    );
    has_all(&stage3_note, &["owner_attest_hex remains transitional"]);
    has_all(
        &stage4_note,
        &[
            "not yet a third-party public proof",
            "must be treated as temporary",
            "verify_spend_witness_gate",
            "test_stage4_tamper::test_stage4_rejects_bad_witness",
        ],
    );
    has_all(
        &stage4_map,
        &[
            "verify_fee_matches_formula",
            "verify_plaintext_balance_with_fee",
            "verify_spend_witness_gate",
            "not a standalone proof verifier",
            "test_stage4_gates::test_stage4_witness_gate_ok",
        ],
    );

    has_no_overclaim(&stage3_map);
    has_no_overclaim(&stage3_note);
    has_no_overclaim(&stage4_note);
    has_no_overclaim(&stage4_map);

    let notes = [
        stage3_map.as_str(),
        stage3_note.as_str(),
        stage4_note.as_str(),
        stage4_map.as_str(),
    ];
    for note in notes {
        assert!(
            note.contains("tracked only in the canonical `post-spec-5.md` closure spec"),
            "subordinate note must stay under post-spec-5.md"
        );
    }

    has_all(
        phase3,
        &[
            "include the live `ZERO_ROOT` authority placeholder",
            "current witness-gate or stage-gate dependency",
            "Keep the final wording subordinate to `post-spec-5.md`",
        ],
    );
}

#[test]
fn test_seeded_rng_stays_bounded() {
    has_all(
        RNG_TRAITS_SRC,
        &[
            "Deterministic reproducibility (approved genesis/testing only)",
            "Explicit simulator reproducibility flows",
            "Get a new RNG instance for a reproducibility-only caller.",
        ],
    );
    has_all(
        RNG_DETERMINISTIC_SRC,
        &[
            "not a universal secure-entropy abstraction",
            "confined to reproducibility-scoped domains",
        ],
    );
    has_all(
        CONFIG_SRC,
        &[
            "Stage-2 simulator reproducibility toggle",
            "does not establish one repo-wide randomness selector",
            "deterministic zero-seed fallback",
        ],
    );
    has_all(
        RNG_MODE_SRC,
        &[
            "simulator reproducibility",
            "bounded to CI and simulator",
            "does not claim one unified randomness selector",
        ],
    );
    has_all(
        STAGE2_TRANSPORT_SRC,
        &[
            "Simulator-only seeded adapter",
            "must never be treated as universal",
            "mock_rng_seed.unwrap_or(0)",
            "zero-seed reproducibility",
        ],
    );
}

#[test]
fn test_active_reclassification_verified_closure() {
    let requirements =
        fs::read_to_string(repo_root().join(".planning/REQUIREMENTS.md")).expect("requirements");

    has_all(
        &requirements,
        &[
            "Active wording may now reflect the implemented storage-backed claim continuity, deterministic spend nullifier closure, `core::stealth` sender authority, and backend-defined package-coupled checkpoint acceptance because those seams are implemented and re-verified.",
            "Append-only historical audit artifacts remain historical evidence.",
        ],
    );
}
