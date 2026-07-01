use once_cell::sync::Lazy;
use tari_crypto::{
    commitment::ExtensionDegree,
    ristretto::{
        bulletproofs_plus::BulletproofsPlusService,
        pedersen::extended_commitment_factory::ExtendedPedersenCommitmentFactory,
    },
};

use crate::types::{AGGREGATION_FACTOR, RANGE_PROOF_BITS};

pub(crate) static BULLETPROOF_SERVICE: Lazy<BulletproofsPlusService> = Lazy::new(|| {
    BulletproofsPlusService::init(
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        ExtendedPedersenCommitmentFactory::default(),
    )
    .expect(
        "FATAL: Bulletproof+ service initialization failed. \
         This indicates corrupt cryptographic parameters, \
         insufficient memory, or incompatible system. \
         Application cannot operate safely and will terminate.",
    )
});

pub(crate) static COMMITMENT_FACTORY: Lazy<ExtendedPedersenCommitmentFactory> =
    Lazy::new(ExtendedPedersenCommitmentFactory::default);

pub(crate) fn bulletproof_service() -> &'static BulletproofsPlusService {
    &BULLETPROOF_SERVICE
}

pub(crate) fn initialize_backend() {
    let _ = bulletproof_service();
    let _ = &*COMMITMENT_FACTORY;
    let _ = ExtensionDegree::DefaultPedersen;
}
