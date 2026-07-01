//! Explicit backend-specific Tari lane.
//!
//! This namespace keeps concrete Tari contracts behind an opt-in path while
//! reusing the canonical advanced helper lane from [`crate::expert`].

pub mod tari {
    pub use crate::expert::encoding::{from_hex, to_hex, ByteArray, Hex, SafePassword};
    pub use crate::expert::hash_domain;
    pub use crate::expert::keys::{RistrettoPublicKey, RistrettoSecretKey};
    pub use crate::expert::traits::{
        DerivedKeyDomain, DomainSeparatedHasher, DomainSeparation, PublicKeyTrait, SecretKeyTrait,
    };
    pub use tari_crypto::commitment::HomomorphicCommitmentFactory;
    pub use tari_crypto::dhke::DiffieHellmanSharedSecret;
    pub use tari_crypto::extended_range_proof::{
        AggregatedPublicStatement, ExtendedRangeProofService, Statement,
    };
    pub use tari_crypto::range_proof::RangeProofService;
    pub use tari_crypto::ristretto::bulletproofs_plus::{
        BulletproofsPlusService, RistrettoAggregatedPublicStatement, RistrettoStatement,
    };
    pub use tari_crypto::ristretto::pedersen::commitment_factory::PedersenCommitmentFactory;
    pub use tari_crypto::ristretto::pedersen::extended_commitment_factory::ExtendedPedersenCommitmentFactory;
    pub use tari_crypto::ristretto::{
        CompressedRistrettoComAndPubSig, RistrettoComAndPubSig, RistrettoComSig, RistrettoSchnorr,
    };
    pub use tari_crypto::signatures::{
        CommitmentAndPublicKeySignature, CommitmentSignature, SchnorrSignature,
    };
}
