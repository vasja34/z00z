use z00z_storage::settlement::ObjectRejectCode;

const OBJECT_DOC: &str = include_str!("../../../wiki/05-storage-runtime/object-package-rejects.md");
const OBJECT_RPC: &str = include_str!("../../z00z_wallets/src/rpc/object_rpc_impl.rs");
const OBJECT_SRC: &str = include_str!("../src/settlement/object_package_contract.rs");
const VALIDATOR_DOC: &str = include_str!("../../z00z_runtime/validators/README.md");

fn expect_pair(code: ObjectRejectCode, rpc_code: &str) {
    let variant = format!("ObjectRejectCode::{code:?}");
    assert!(
        OBJECT_SRC.contains(&format!("{},", code_name(code))),
        "missing enum variant {variant}"
    );
    assert!(
        OBJECT_RPC.contains(&format!("{variant} => \"{rpc_code}\"")),
        "missing wallet RPC mapping for {variant}"
    );
}

fn code_name(code: ObjectRejectCode) -> &'static str {
    match code {
        ObjectRejectCode::UnknownPolicy => "UnknownPolicy",
        ObjectRejectCode::UnknownAction => "UnknownAction",
        ObjectRejectCode::InvalidBacking => "InvalidBacking",
        ObjectRejectCode::WrongFamilyProof => "WrongFamilyProof",
        ObjectRejectCode::VoucherUsedAsCash => "VoucherUsedAsCash",
        ObjectRejectCode::RightUsedAsValue => "RightUsedAsValue",
        ObjectRejectCode::MissingRight => "MissingRight",
        ObjectRejectCode::RightOutOfScope => "RightOutOfScope",
        ObjectRejectCode::RightExpired => "RightExpired",
        ObjectRejectCode::RightRevoked => "RightRevoked",
        ObjectRejectCode::RightConsumed => "RightConsumed",
        ObjectRejectCode::Replay => "Replay",
        ObjectRejectCode::DoubleRedeem => "DoubleRedeem",
        ObjectRejectCode::ResidualMismatch => "ResidualMismatch",
        ObjectRejectCode::ForcedAcceptance => "ForcedAcceptance",
        ObjectRejectCode::StaleRoot => "StaleRoot",
        ObjectRejectCode::FeeBoundary => "FeeBoundary",
        ObjectRejectCode::MissingSignature => "MissingSignature",
        ObjectRejectCode::MissingAttestation => "MissingAttestation",
        ObjectRejectCode::ExpiredVoucherUse => "ExpiredVoucherUse",
    }
}

#[test]
fn reject_codes_mapped() {
    assert!(!OBJECT_RPC.contains("match code {\n        _ =>"));
    assert!(OBJECT_DOC.contains("| `ObjectRejectCode` | Stable reject taxonomy."));
    assert!(OBJECT_DOC
        .contains("| Validator boundary | Reuses storage-owned proof and route contracts. |"));
    assert!(OBJECT_DOC
        .contains("Wallet RPC converts storage reject codes into stable `OBJECT_*` RPC errors"));
    assert!(OBJECT_SRC.contains("pub struct ObjectValidatorVerdict"));
    assert!(OBJECT_SRC.contains("pub fn inspect_object_package("));
    assert!(VALIDATOR_DOC.contains("typed settlement objects"));

    let pairs = [
        (ObjectRejectCode::UnknownPolicy, "OBJECT_UNKNOWN_POLICY"),
        (ObjectRejectCode::UnknownAction, "OBJECT_UNKNOWN_ACTION"),
        (ObjectRejectCode::InvalidBacking, "OBJECT_INVALID_BACKING"),
        (
            ObjectRejectCode::WrongFamilyProof,
            "OBJECT_WRONG_FAMILY_PROOF",
        ),
        (
            ObjectRejectCode::VoucherUsedAsCash,
            "OBJECT_VOUCHER_USED_AS_CASH",
        ),
        (
            ObjectRejectCode::RightUsedAsValue,
            "OBJECT_RIGHT_USED_AS_VALUE",
        ),
        (ObjectRejectCode::MissingRight, "OBJECT_MISSING_RIGHT"),
        (
            ObjectRejectCode::RightOutOfScope,
            "OBJECT_RIGHT_OUT_OF_SCOPE",
        ),
        (ObjectRejectCode::RightExpired, "OBJECT_RIGHT_EXPIRED"),
        (ObjectRejectCode::RightRevoked, "OBJECT_RIGHT_REVOKED"),
        (ObjectRejectCode::RightConsumed, "OBJECT_RIGHT_CONSUMED"),
        (ObjectRejectCode::Replay, "OBJECT_REPLAY"),
        (ObjectRejectCode::DoubleRedeem, "OBJECT_DOUBLE_REDEEM"),
        (
            ObjectRejectCode::ResidualMismatch,
            "OBJECT_RESIDUAL_MISMATCH",
        ),
        (
            ObjectRejectCode::ForcedAcceptance,
            "OBJECT_FORCED_ACCEPTANCE",
        ),
        (ObjectRejectCode::StaleRoot, "OBJECT_STALE_ROOT"),
        (ObjectRejectCode::FeeBoundary, "OBJECT_FEE_BOUNDARY"),
        (
            ObjectRejectCode::MissingSignature,
            "OBJECT_MISSING_SIGNATURE",
        ),
        (
            ObjectRejectCode::MissingAttestation,
            "OBJECT_MISSING_ATTESTATION",
        ),
        (
            ObjectRejectCode::ExpiredVoucherUse,
            "OBJECT_EXPIRED_VOUCHER_USE",
        ),
    ];

    assert_eq!(pairs.len(), 20, "ObjectRejectCode coverage count drifted");
    for (code, rpc_code) in pairs {
        expect_pair(code, rpc_code);
    }
}
