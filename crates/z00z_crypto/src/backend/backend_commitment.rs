use tari_crypto::commitment::HomomorphicCommitmentFactory;

use crate::types::{Z00ZCommitment, Z00ZScalar};

use super::backend_init::COMMITMENT_FACTORY;

pub(crate) fn create_commitment_impl(amount: u64, blinding: &Z00ZScalar) -> Z00ZCommitment {
    Z00ZCommitment::from_commitment(COMMITMENT_FACTORY.commit_value(blinding.inner(), amount))
}
