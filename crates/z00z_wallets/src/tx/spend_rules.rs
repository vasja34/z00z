#![allow(missing_docs)]

use std::collections::HashSet;

use thiserror::Error;
#[cfg(test)]
use z00z_crypto::Z00ZScalar;
use z00z_crypto::{
    derive_domain_hash,
    domains::{AssetIdDomain, SpendNullifierDomain},
    hash_zk::hash_zk,
    Z00ZCommitment,
};
use z00z_storage::settlement::{RightClass, RightLeaf};

use crate::{
    key::{derive_owner_handle, derive_view_secret_key, ReceiverSecret},
    stealth::{
        ecdh::{compute_dh_receiver, decode_r_pub},
        kdf::{compute_owner_tag, derive_k_dh},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpendRule {
    OwnerHandle,
    ViewKey,
    InputKdf,
    OwnerTag,
    AssetId,
    Nullifier,
    Balance,
    Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpendRuleTriplet {
    pub rule: SpendRule,
    pub witness: &'static [&'static str],
    pub public: &'static [&'static str],
    pub domain: &'static str,
}

const OWNER_WIT: &[&str] = &["receiver_secret"];
const OWNER_PUB: &[&str] = &["owner_handle"];
const VIEW_WIT: &[&str] = &["receiver_secret"];
const VIEW_PUB: &[&str] = &["view_sk"];
const KDF_WIT: &[&str] = &["view_sk"];
const KDF_PUB: &[&str] = &["r_pub_in", "k_in"];
const TAG_WIT: &[&str] = &["owner_handle", "k_in"];
const TAG_PUB: &[&str] = &["owner_tag_in"];
const ASSET_WIT: &[&str] = &["s_in"];
const ASSET_PUB: &[&str] = &["leaf_ad_id_in"];
const NULL_WIT: &[&str] = &["chain_id", "s_in"];
const NULL_PUB: &[&str] = &["nullifier_in"];
const BAL_WIT: &[&str] = &["c_in", "c_out"];
const BAL_PUB: &[&str] = &["commit_eq_zero"];
const RNG_WIT: &[&str] = &["range_proofs"];
const RNG_PUB: &[&str] = &["range_ok"];

const SPEND_ORDER: [SpendRule; 8] = [
    SpendRule::OwnerHandle,
    SpendRule::ViewKey,
    SpendRule::InputKdf,
    SpendRule::OwnerTag,
    SpendRule::AssetId,
    SpendRule::Nullifier,
    SpendRule::Balance,
    SpendRule::Range,
];

const SPEND_RULES: [SpendRuleTriplet; 8] = [
    SpendRuleTriplet {
        rule: SpendRule::OwnerHandle,
        witness: OWNER_WIT,
        public: OWNER_PUB,
        domain: "z00z.consensus.receiver_id.v1",
    },
    SpendRuleTriplet {
        rule: SpendRule::ViewKey,
        witness: VIEW_WIT,
        public: VIEW_PUB,
        domain: "z00z.consensus.view_key.v1",
    },
    SpendRuleTriplet {
        rule: SpendRule::InputKdf,
        witness: KDF_WIT,
        public: KDF_PUB,
        domain: "z00z.consensus.dh_key.v1",
    },
    SpendRuleTriplet {
        rule: SpendRule::OwnerTag,
        witness: TAG_WIT,
        public: TAG_PUB,
        domain: "z00z.consensus.owner_tag.v1",
    },
    SpendRuleTriplet {
        rule: SpendRule::AssetId,
        witness: ASSET_WIT,
        public: ASSET_PUB,
        domain: "z00z.consensus.asset_id.v1",
    },
    SpendRuleTriplet {
        rule: SpendRule::Nullifier,
        witness: NULL_WIT,
        public: NULL_PUB,
        domain: "z00z.consensus.nullifier.v1",
    },
    SpendRuleTriplet {
        rule: SpendRule::Balance,
        witness: BAL_WIT,
        public: BAL_PUB,
        domain: "Z00Z/BAL",
    },
    SpendRuleTriplet {
        rule: SpendRule::Range,
        witness: RNG_WIT,
        public: RNG_PUB,
        domain: "z00z.consensus.range_ctx.v1",
    },
];

pub const VALIDATOR_MANDATE_LOCK_PROFILE_ID: &str = "validator_mandate_lock_v1";

pub fn has_validator_mandate_lock_profile(labels: &[String]) -> bool {
    labels
        .iter()
        .any(|label| label == VALIDATOR_MANDATE_LOCK_PROFILE_ID)
}

pub fn validator_mandate_lock_payload_commitment(
    locked_asset_id: &[u8; 32],
    locked_amount: u64,
    right: &RightLeaf,
) -> [u8; 32] {
    let mut data = Vec::with_capacity(32 * 5 + 8 * 5 + 1 + VALIDATOR_MANDATE_LOCK_PROFILE_ID.len());
    data.extend_from_slice(VALIDATOR_MANDATE_LOCK_PROFILE_ID.as_bytes());
    data.push(1);
    data.extend_from_slice(locked_asset_id);
    data.extend_from_slice(&locked_amount.to_le_bytes());
    data.extend_from_slice(&right.valid_from.to_le_bytes());
    data.extend_from_slice(&right.valid_until.to_le_bytes());
    data.extend_from_slice(&right.challenge_from.to_le_bytes());
    data.extend_from_slice(&right.challenge_until.to_le_bytes());
    data.extend_from_slice(&right.use_nonce);
    data.extend_from_slice(&right.transition_policy_id);
    data.extend_from_slice(&right.revocation_policy_id);
    data.extend_from_slice(&right.disclosure_policy_id);
    data.extend_from_slice(&right.retention_policy_id);

    derive_domain_hash("z00z.wallet.validator_mandate_lock.v1", &data)
}

pub fn validator_mandate_lock_matches_asset(
    right: &RightLeaf,
    locked_asset_id: &[u8; 32],
    locked_amount: u64,
) -> bool {
    right.right_class == RightClass::ValidatorMandate
        && right.payload_commitment
            == validator_mandate_lock_payload_commitment(locked_asset_id, locked_amount, right)
}

pub fn validator_mandate_lock_unlock_ready(right: &RightLeaf, now_secs: u64) -> bool {
    right.right_class == RightClass::ValidatorMandate && now_secs > right.valid_until
}

pub fn spend_order() -> &'static [SpendRule; 8] {
    &SPEND_ORDER
}

pub fn spend_triplets() -> &'static [SpendRuleTriplet; 8] {
    &SPEND_RULES
}

#[derive(Debug, Clone)]
pub struct SpendIn {
    pub chain_id: u32,
    pub r_pub_in: [u8; 32],
    pub owner_tag_in: [u8; 32],
    pub leaf_ad_id_in: [u8; 32],
    pub nullifier_in: Option<[u8; 32]>,
    pub s_in: [u8; 32],
    pub c_in: Z00ZCommitment,
}

pub struct SpendStmt {
    pub receiver_secret: ReceiverSecret,
    pub spend_ins: Vec<SpendIn>,
    pub c_outs: Vec<Z00ZCommitment>,
    pub range_ok: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpendRuleErr {
    #[error("view key derivation failed")]
    BadView,
    #[error("invalid r_pub at input {index}")]
    BadRPub { index: usize },
    #[error("invalid dh at input {index}")]
    BadDh { index: usize },
    #[error("owner_tag mismatch at input {index}")]
    BadOwnerTag { index: usize },
    #[error("asset_id mismatch at input {index}")]
    BadAssetId { index: usize },
    #[error("missing nullifier at input {index}")]
    MissingNullifier { index: usize },
    #[error("nullifier mismatch at input {index}")]
    BadNullifier { index: usize },
    #[error("duplicate nullifier in one spend contract")]
    DuplicateNullifier,
    #[error("balance equation mismatch")]
    BadBalance,
    #[error("range condition failed")]
    BadRange,
}

pub fn derive_spend_nullifier(chain_id: u32, s_in: &[u8; 32]) -> [u8; 32] {
    let chain_id_le = chain_id.to_le_bytes();
    hash_zk::<SpendNullifierDomain>("", &[&chain_id_le, s_in])
}

/// These structural spend rules lock only the delivered persisted public spend
/// contract's owner, leaf, nullifier field, balance, and range relations.
/// The deterministic nullifier checked here is a regular-spend proof field and
/// duplicate guard. It does not replace checkpointed storage membership or
/// asset-leaf deletion as the authoritative consumed-state model.
/// The live public spend contract is real but still narrower than a finished
/// full-ZK spend theorem. `PH32-SPEND` stays limited to that shipped boundary.
pub fn verify_spend_rules(stmt: &SpendStmt) -> Result<(), SpendRuleErr> {
    let owner_handle = derive_owner_handle(&stmt.receiver_secret);
    let view_sk =
        derive_view_secret_key(&stmt.receiver_secret).map_err(|_| SpendRuleErr::BadView)?;
    let mut seen_nullifiers = HashSet::new();

    for (index, input) in stmt.spend_ins.iter().enumerate() {
        let r_pub = decode_r_pub(&input.r_pub_in).map_err(|_| SpendRuleErr::BadRPub { index })?;
        let dh =
            compute_dh_receiver(&view_sk, &r_pub).map_err(|_| SpendRuleErr::BadDh { index })?;
        let k_in = derive_k_dh(&dh);

        let expected_tag = compute_owner_tag(&owner_handle, &k_in);
        if expected_tag != input.owner_tag_in {
            return Err(SpendRuleErr::BadOwnerTag { index });
        }

        let expected_leaf_ad_id = hash_zk::<AssetIdDomain>("", &[&input.s_in]);
        if expected_leaf_ad_id != input.leaf_ad_id_in {
            return Err(SpendRuleErr::BadAssetId { index });
        }

        let nullifier = input
            .nullifier_in
            .ok_or(SpendRuleErr::MissingNullifier { index })?;
        let expected_nullifier = derive_spend_nullifier(input.chain_id, &input.s_in);
        if expected_nullifier != nullifier {
            return Err(SpendRuleErr::BadNullifier { index });
        }
        if !seen_nullifiers.insert(nullifier) {
            return Err(SpendRuleErr::DuplicateNullifier);
        }
    }

    if !verify_balance_eq(&stmt.spend_ins, &stmt.c_outs) {
        return Err(SpendRuleErr::BadBalance);
    }
    if !stmt.range_ok {
        return Err(SpendRuleErr::BadRange);
    }
    Ok(())
}

fn verify_balance_eq(spend_ins: &[SpendIn], c_outs: &[Z00ZCommitment]) -> bool {
    if spend_ins.is_empty() || c_outs.is_empty() {
        return false;
    }
    sum_spend_in_commitments(spend_ins) == sum_commitments(c_outs)
}

fn sum_spend_in_commitments(items: &[SpendIn]) -> Z00ZCommitment {
    items
        .iter()
        .skip(1)
        .fold(items[0].c_in.clone(), |acc, item| &acc + &item.c_in)
}

fn sum_commitments(items: &[Z00ZCommitment]) -> Z00ZCommitment {
    items
        .iter()
        .skip(1)
        .fold(items[0].clone(), |acc, item| &acc + item)
}

#[cfg(test)]
fn test_scalar(seed: u64) -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Z00ZScalar::try_from_bytes(bytes).expect("valid scalar")
}

#[cfg(test)]
fn test_secret(seed: u64) -> ReceiverSecret {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    if bytes == [0u8; 32] {
        bytes[0] = 1;
    }
    ReceiverSecret::from_bytes(bytes).expect("secret")
}

#[cfg(test)]
mod tests {
    use super::{
        derive_spend_nullifier, has_validator_mandate_lock_profile, spend_order, spend_triplets,
        test_scalar, test_secret, validator_mandate_lock_matches_asset,
        validator_mandate_lock_payload_commitment, validator_mandate_lock_unlock_ready,
        verify_spend_rules, SpendIn, SpendRule, SpendRuleErr, SpendStmt,
        VALIDATOR_MANDATE_LOCK_PROFILE_ID,
    };
    use z00z_crypto::{
        create_commitment, domains::AssetIdDomain, hash_zk::hash_zk, Z00ZRistrettoPoint,
    };
    use z00z_storage::settlement::{RightClass, RightLeaf, TerminalId};

    use crate::{
        key::derive_view_secret_key,
        stealth::{
            ecdh::compute_dh_sender,
            kdf::{compute_owner_tag, derive_k_dh},
        },
    };

    fn make_stmt() -> SpendStmt {
        let receiver_secret = test_secret(9);
        let view_sk = derive_view_secret_key(&receiver_secret).expect("view");
        let view_pk = Z00ZRistrettoPoint::from_secret_key(&view_sk);

        let r = test_scalar(77);
        let r_pub_pt = Z00ZRistrettoPoint::from_secret_key(&r);
        let r_pub_in = r_pub_pt.to_bytes();

        let dh = compute_dh_sender(&r, &view_pk).expect("dh");
        let k_in = derive_k_dh(&dh);
        let owner_handle = crate::key::derive_owner_handle(&receiver_secret);
        let owner_tag_in = compute_owner_tag(&owner_handle, &k_in);

        let s_in = [5u8; 32];
        let asset_id_in = hash_zk::<AssetIdDomain>("", &[&s_in]);

        let in_blind = test_scalar(41);
        let c_in = create_commitment(12, &in_blind).expect("c_in");
        let c_out = create_commitment(12, &in_blind).expect("c_out");

        SpendStmt {
            receiver_secret,
            spend_ins: vec![SpendIn {
                chain_id: 3,
                r_pub_in,
                owner_tag_in,
                leaf_ad_id_in: asset_id_in,
                nullifier_in: Some(derive_spend_nullifier(3, &s_in)),
                s_in,
                c_in,
            }],
            c_outs: vec![c_out],
            range_ok: true,
        }
    }

    fn validator_lock_leaf() -> RightLeaf {
        RightLeaf {
            version: 1,
            terminal_id: TerminalId::new([0xA1; 32]),
            right_class: RightClass::ValidatorMandate,
            issuer_scope: [0x11; 32],
            provider_scope: [0x12; 32],
            holder_commitment: [0x13; 32],
            control_commitment: [0x14; 32],
            beneficiary_commitment: [0x15; 32],
            payload_commitment: [0u8; 32],
            valid_from: 10,
            valid_until: 20,
            challenge_from: 21,
            challenge_until: 30,
            use_nonce: [0x16; 32],
            revocation_policy_id: [0x17; 32],
            transition_policy_id: [0x18; 32],
            challenge_policy_id: [0x19; 32],
            disclosure_policy_id: [0x1A; 32],
            retention_policy_id: [0x1B; 32],
        }
    }

    #[test]
    fn test_verify_spend_ok() {
        let stmt = make_stmt();
        assert_eq!(verify_spend_rules(&stmt), Ok(()));
    }

    #[test]
    fn test_wrong_owner_tag() {
        let mut stmt = make_stmt();
        stmt.spend_ins[0].owner_tag_in[0] ^= 1;
        assert_eq!(
            verify_spend_rules(&stmt),
            Err(SpendRuleErr::BadOwnerTag { index: 0 })
        );
    }

    #[test]
    fn test_bad_r_pub() {
        let mut stmt = make_stmt();
        stmt.spend_ins[0].r_pub_in = [0u8; 32];
        assert_eq!(
            verify_spend_rules(&stmt),
            Err(SpendRuleErr::BadRPub { index: 0 })
        );
    }

    #[test]
    fn test_wrong_asset_id() {
        let mut stmt = make_stmt();
        stmt.spend_ins[0].leaf_ad_id_in[0] ^= 1;
        assert_eq!(
            verify_spend_rules(&stmt),
            Err(SpendRuleErr::BadAssetId { index: 0 })
        );
    }

    #[test]
    fn test_missing_nullifier() {
        let mut stmt = make_stmt();
        stmt.spend_ins[0].nullifier_in = None;
        assert_eq!(
            verify_spend_rules(&stmt),
            Err(SpendRuleErr::MissingNullifier { index: 0 })
        );
    }

    #[test]
    fn test_wrong_nullifier() {
        let mut stmt = make_stmt();
        let mut wrong = stmt.spend_ins[0].nullifier_in.expect("nullifier");
        wrong[0] ^= 1;
        stmt.spend_ins[0].nullifier_in = Some(wrong);
        assert_eq!(
            verify_spend_rules(&stmt),
            Err(SpendRuleErr::BadNullifier { index: 0 })
        );
    }

    #[test]
    fn test_duplicate_nullifier() {
        let mut stmt = make_stmt();
        let base = stmt.spend_ins[0].clone();
        stmt.spend_ins.push(base);
        stmt.c_outs.push(stmt.c_outs[0].clone());

        assert_eq!(
            verify_spend_rules(&stmt),
            Err(SpendRuleErr::DuplicateNullifier)
        );
    }

    #[test]
    fn test_wrong_balance_eq() {
        let mut stmt = make_stmt();
        stmt.c_outs[0] = create_commitment(10, &test_scalar(17)).expect("c_out");
        assert_eq!(verify_spend_rules(&stmt), Err(SpendRuleErr::BadBalance));
    }

    #[test]
    fn test_rule_order_lock() {
        assert_eq!(
            spend_order(),
            &[
                SpendRule::OwnerHandle,
                SpendRule::ViewKey,
                SpendRule::InputKdf,
                SpendRule::OwnerTag,
                SpendRule::AssetId,
                SpendRule::Nullifier,
                SpendRule::Balance,
                SpendRule::Range,
            ]
        );
    }

    #[test]
    fn test_rule_domains_locked() {
        let rules = spend_triplets();
        assert_eq!(rules[0].domain, "z00z.consensus.receiver_id.v1");
        assert_eq!(rules[1].domain, "z00z.consensus.view_key.v1");
        assert_eq!(rules[2].domain, "z00z.consensus.dh_key.v1");
        assert_eq!(rules[3].domain, "z00z.consensus.owner_tag.v1");
        assert_eq!(rules[4].domain, "z00z.consensus.asset_id.v1");
        assert_eq!(rules[5].domain, "z00z.consensus.nullifier.v1");
        assert_eq!(rules[6].domain, "Z00Z/BAL");
        assert_eq!(rules[7].domain, "z00z.consensus.range_ctx.v1");
    }

    #[test]
    fn test_triplets_non_empty() {
        for rule in spend_triplets() {
            assert!(!rule.witness.is_empty());
            assert!(!rule.public.is_empty());
            assert!(!rule.domain.is_empty());
        }
    }

    #[test]
    fn test_mandate_lock_tag_detect() {
        assert!(has_validator_mandate_lock_profile(&[
            "right".to_string(),
            VALIDATOR_MANDATE_LOCK_PROFILE_ID.to_string(),
        ]));
        assert!(!has_validator_mandate_lock_profile(&[
            "validator_mandate".to_string()
        ]));
    }

    #[test]
    fn test_mandate_lock_payload_changes() {
        let right = validator_lock_leaf();
        let base = validator_mandate_lock_payload_commitment(&[0x31; 32], 50, &right);
        let other_asset = validator_mandate_lock_payload_commitment(&[0x32; 32], 50, &right);
        let other_amount = validator_mandate_lock_payload_commitment(&[0x31; 32], 51, &right);
        assert_ne!(base, other_asset);
        assert_ne!(base, other_amount);
    }

    #[test]
    fn test_mandate_lock_requires_payload() {
        let mut right = validator_lock_leaf();
        let locked_asset_id = [0x41; 32];
        right.payload_commitment =
            validator_mandate_lock_payload_commitment(&locked_asset_id, 77, &right);

        assert!(validator_mandate_lock_matches_asset(
            &right,
            &locked_asset_id,
            77,
        ));
        assert!(!validator_mandate_lock_matches_asset(
            &right,
            &locked_asset_id,
            78,
        ));
        assert!(!validator_mandate_lock_matches_asset(
            &right,
            &[0x42; 32],
            77,
        ));
    }

    #[test]
    fn test_mandate_unlock_after_expiry() {
        let right = validator_lock_leaf();
        assert!(!validator_mandate_lock_unlock_ready(&right, 20));
        assert!(validator_mandate_lock_unlock_ready(&right, 21));
    }

    #[test]
    fn test_nullifier_closure_wording() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let spend_verification =
            z00z_utils::io::read_to_string(repo_root.join("src/tx/spend_verification.rs"))
                .expect("read spend_verification.rs")
                .to_lowercase();
        let spend_rules = z00z_utils::io::read_to_string(repo_root.join("src/tx/spend_rules.rs"))
            .expect("read spend_rules.rs")
            .to_lowercase();
        let requirements =
            z00z_utils::io::read_to_string(repo_root.join("../../.planning/REQUIREMENTS.md"))
                .expect("read REQUIREMENTS.md")
                .to_lowercase();

        assert!(
            spend_verification.contains("persisted public spend contract")
                && spend_verification.contains("deterministic nullifier semantics surface")
                && spend_verification.contains("current proof/auth seam")
                && spend_verification.contains("already live"),
            "public spend verifier wording must describe the shipped deterministic nullifier closure honestly"
        );
        assert!(
            spend_rules.contains("deterministic nullifier semantics surface")
                && spend_rules.contains("structural rule layer"),
            "spend rules documentation must describe the shipped deterministic nullifier closure honestly"
        );
        assert!(
            requirements.contains("- [x] **ph32-spend**")
                && requirements.contains("authenticates one signed nullifier field on the public seam")
                && requirements.contains(
                    "witness bridge and structural spend rules enforce deterministic `chain_id || s_in` derivation"
                ),
            "requirements must reflect the shipped nullifier closure in active wording"
        );
    }
}
