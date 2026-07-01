use thiserror::Error;
use z00z_crypto::{expert::hash_domain, hash_zk::hash_zk};

use super::record::FeeEnvelope;

hash_domain!(StorFeeReplayDom, "z00z.storage.settlement.fee.replay.v1", 1);
hash_domain!(
    StorFeeBudgetBindDom,
    "z00z.storage.settlement.fee.budget.bind.v1",
    1
);

const ZERO32: [u8; 32] = [0u8; 32];
const FEE_ENV_VER: u8 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeeSupportCtx {
    pub required_units: u64,
    pub domain_id: [u8; 32],
    pub transition_id: [u8; 32],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeeActorCtx {
    pub now: u64,
    pub payer_commitment: Option<[u8; 32]>,
    pub sponsor_commitment: Option<[u8; 32]>,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct FeeReplayKey([u8; 32]);

impl FeeReplayKey {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeeReplayRec {
    pub replay_key: FeeReplayKey,
    pub replay_digest: [u8; 32],
    pub nonce: [u8; 32],
    pub transition_id: [u8; 32],
    pub domain_id: [u8; 32],
    pub payer_commitment: [u8; 32],
    pub sponsor_commitment: [u8; 32],
    pub budget_units: u64,
    pub budget_commitment: [u8; 32],
    pub support_ref: Option<[u8; 32]>,
    pub failure_policy_id: [u8; 32],
    pub expires_at: u64,
    pub accepted_at_seq: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FeeErr {
    #[error("fee support is required for this transition")]
    SupportRequired,
    #[error("fee envelope version is unsupported")]
    VersionMix,
    #[error("fee envelope must bind a payer or sponsor")]
    SupportBindingMix,
    #[error("fee envelope budget commitment mismatch")]
    BudgetMix,
    #[error("fee envelope budget is insufficient")]
    BudgetShort,
    #[error("fee envelope domain binding mismatch")]
    DomainMix,
    #[error("fee envelope transition binding mismatch")]
    TransitionMix,
    #[error("fee envelope payer binding mismatch")]
    PayerMix,
    #[error("fee envelope sponsor binding mismatch")]
    SponsorMix,
    #[error("fee envelope expiry is stale")]
    Expired,
    #[error("fee envelope replay binding is invalid")]
    ReplayMix,
    #[error("fee envelope nonce binding is invalid")]
    NonceMix,
    #[error("fee envelope failure policy is missing")]
    FailurePolicyMix,
}

impl FeeEnvelope {
    pub fn check(&self) -> Result<(), FeeErr> {
        if self.version != FEE_ENV_VER {
            return Err(FeeErr::VersionMix);
        }
        if self.payer_commitment == ZERO32 && self.sponsor_commitment == ZERO32 {
            return Err(FeeErr::SupportBindingMix);
        }
        if self.budget_units == 0 {
            return Err(FeeErr::BudgetShort);
        }
        if self.budget_commitment != Self::budget_bind(self.budget_units, self.support_ref) {
            return Err(FeeErr::BudgetMix);
        }
        if self.domain_id == ZERO32 {
            return Err(FeeErr::DomainMix);
        }
        if self.transition_id == ZERO32 {
            return Err(FeeErr::TransitionMix);
        }
        if self.replay_key == ZERO32 {
            return Err(FeeErr::ReplayMix);
        }
        if self.nonce == ZERO32 {
            return Err(FeeErr::NonceMix);
        }
        if self.failure_policy_id == ZERO32 {
            return Err(FeeErr::FailurePolicyMix);
        }
        Ok(())
    }

    #[must_use]
    pub fn budget_bind(budget_units: u64, support_ref: Option<[u8; 32]>) -> [u8; 32] {
        let budget = budget_units.to_be_bytes();
        let has_ref = [u8::from(support_ref.is_some())];
        let support_ref = support_ref.unwrap_or(ZERO32);
        hash_zk::<StorFeeBudgetBindDom>("budget", &[&budget, &has_ref, &support_ref])
    }

    #[must_use]
    pub const fn replay_id(&self) -> FeeReplayKey {
        FeeReplayKey::new(self.replay_key)
    }

    #[must_use]
    pub fn replay_digest(&self) -> [u8; 32] {
        let version = [self.version];
        let budget_units = self.budget_units.to_be_bytes();
        let expires = self.expires_at.to_be_bytes();
        let has_ref = [u8::from(self.support_ref.is_some())];
        let support_ref = self.support_ref.unwrap_or(ZERO32);
        hash_zk::<StorFeeReplayDom>(
            "",
            &[
                &version,
                &self.payer_commitment,
                &self.sponsor_commitment,
                &budget_units,
                &self.budget_commitment,
                &self.domain_id,
                &expires,
                &self.nonce,
                &self.transition_id,
                &self.replay_key,
                &has_ref,
                &support_ref,
                &self.failure_policy_id,
            ],
        )
    }

    pub fn validate_support(
        &self,
        support: FeeSupportCtx,
        actor: FeeActorCtx,
        replay_seen: bool,
    ) -> Result<(), FeeErr> {
        self.check()?;

        if actor.payer_commitment.is_none() && actor.sponsor_commitment.is_none() {
            return Err(FeeErr::SupportBindingMix);
        }

        if self.expires_at <= actor.now {
            return Err(FeeErr::Expired);
        }
        if self.budget_units < support.required_units {
            return Err(FeeErr::BudgetShort);
        }
        if self.domain_id != support.domain_id {
            return Err(FeeErr::DomainMix);
        }
        if self.transition_id != support.transition_id {
            return Err(FeeErr::TransitionMix);
        }
        if self.payer_commitment != ZERO32 && actor.payer_commitment.is_none() {
            return Err(FeeErr::PayerMix);
        }
        if self.sponsor_commitment != ZERO32 && actor.sponsor_commitment.is_none() {
            return Err(FeeErr::SponsorMix);
        }
        if self.payer_commitment != ZERO32 && actor.payer_commitment != Some(self.payer_commitment)
        {
            return Err(FeeErr::PayerMix);
        }
        if self.sponsor_commitment != ZERO32
            && actor.sponsor_commitment != Some(self.sponsor_commitment)
        {
            return Err(FeeErr::SponsorMix);
        }
        if replay_seen {
            return Err(FeeErr::ReplayMix);
        }
        Ok(())
    }

    #[must_use]
    pub fn replay_rec(&self, accepted_at_seq: u64) -> FeeReplayRec {
        FeeReplayRec {
            replay_key: self.replay_id(),
            replay_digest: self.replay_digest(),
            nonce: self.nonce,
            transition_id: self.transition_id,
            domain_id: self.domain_id,
            payer_commitment: self.payer_commitment,
            sponsor_commitment: self.sponsor_commitment,
            budget_units: self.budget_units,
            budget_commitment: self.budget_commitment,
            support_ref: self.support_ref,
            failure_policy_id: self.failure_policy_id,
            expires_at: self.expires_at,
            accepted_at_seq,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FeeActorCtx, FeeEnvelope, FeeErr, FeeReplayKey, FeeSupportCtx, ZERO32};

    fn bytes(mark: u8) -> [u8; 32] {
        [mark; 32]
    }

    fn envelope(mark: u8) -> FeeEnvelope {
        let budget_units = u64::from(mark) + 2;
        let support_ref = Some(bytes(mark.wrapping_add(8)));
        FeeEnvelope {
            version: 1,
            payer_commitment: bytes(mark),
            sponsor_commitment: bytes(mark.wrapping_add(1)),
            budget_units,
            budget_commitment: FeeEnvelope::budget_bind(budget_units, support_ref),
            domain_id: bytes(mark.wrapping_add(3)),
            expires_at: 50,
            nonce: bytes(mark.wrapping_add(4)),
            transition_id: bytes(mark.wrapping_add(5)),
            replay_key: bytes(mark.wrapping_add(6)),
            support_ref,
            failure_policy_id: bytes(mark.wrapping_add(7)),
        }
    }

    fn support(mark: u8) -> FeeSupportCtx {
        FeeSupportCtx {
            required_units: u64::from(mark) + 2,
            domain_id: bytes(mark.wrapping_add(3)),
            transition_id: bytes(mark.wrapping_add(5)),
        }
    }

    fn actor(mark: u8, now: u64) -> FeeActorCtx {
        FeeActorCtx {
            now,
            payer_commitment: Some(bytes(mark)),
            sponsor_commitment: Some(bytes(mark.wrapping_add(1))),
        }
    }

    #[test]
    fn test_ok_matching_support() {
        let envelope = envelope(10);
        let replay = envelope.replay_rec(7);

        assert_eq!(
            envelope.validate_support(support(10), actor(10, 12), false),
            Ok(())
        );
        assert_eq!(replay.replay_key, FeeReplayKey::new(bytes(16)));
        assert_eq!(replay.accepted_at_seq, 7);
    }

    #[test]
    fn test_rejects_stale_support() {
        let envelope = envelope(20);
        let err = envelope
            .validate_support(support(20), actor(20, 60), false)
            .expect_err("stale support must reject");

        assert_eq!(err, FeeErr::Expired);
    }

    #[test]
    fn test_rejects_bad_binding() {
        let envelope = envelope(30);
        let mut fee_actor = actor(30, 12);
        fee_actor.sponsor_commitment = Some(bytes(90));

        let err = envelope
            .validate_support(support(30), fee_actor, false)
            .expect_err("wrong sponsor must reject");

        assert_eq!(err, FeeErr::SponsorMix);
    }

    #[test]
    fn test_rejects_unbound_context() {
        let envelope = envelope(35);
        let err = envelope
            .validate_support(
                support(35),
                FeeActorCtx {
                    now: 12,
                    payer_commitment: None,
                    sponsor_commitment: None,
                },
                false,
            )
            .expect_err("unbound caller context must reject");

        assert_eq!(err, FeeErr::SupportBindingMix);
    }

    #[test]
    fn test_rejects_partial_actor_context() {
        let envelope = envelope(34);
        let err = envelope
            .validate_support(
                support(34),
                FeeActorCtx {
                    now: 12,
                    payer_commitment: Some(bytes(34)),
                    sponsor_commitment: None,
                },
                false,
            )
            .expect_err("missing sponsor binding must reject");

        assert_eq!(err, FeeErr::SponsorMix);
    }

    #[test]
    fn test_extra_unbound_actor_context() {
        let mut envelope = envelope(36);
        envelope.payer_commitment = ZERO32;

        envelope
            .validate_support(
                support(36),
                FeeActorCtx {
                    now: 12,
                    payer_commitment: Some(bytes(90)),
                    sponsor_commitment: Some(bytes(37)),
                },
                false,
            )
            .expect("extra unbound payer context must not reject sponsor-bound envelope");
    }

    #[test]
    fn test_rejects_bad_domain() {
        let envelope = envelope(36);
        let err = envelope
            .validate_support(
                FeeSupportCtx {
                    required_units: 38,
                    domain_id: bytes(90),
                    transition_id: bytes(41),
                },
                actor(36, 12),
                false,
            )
            .expect_err("wrong domain must reject");

        assert_eq!(err, FeeErr::DomainMix);
    }

    #[test]
    fn test_fee_envelope_rejects_replay() {
        let envelope = envelope(40);
        let err = envelope
            .validate_support(support(40), actor(40, 12), true)
            .expect_err("replay must reject");

        assert_eq!(err, FeeErr::ReplayMix);
    }
}
