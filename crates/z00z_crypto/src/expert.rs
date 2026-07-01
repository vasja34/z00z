//! Expert lane for advanced crypto-adjacent utilities.
//!
//! This is a public advanced-API lane, not a backend implementation module.
//! The `backend_<X>.rs` family stays private and implements the internal
//! `CryptoBackend` abstraction. By contrast, `expert` exists only to expose
//! supported specialist-facing helpers that are intentionally kept off the
//! default root facade.
//!
//! In other words:
//! - `expert` answers "what advanced public helpers may callers import?"
//! - `backend_<X>` answers "how is the active crypto backend implemented?"
//!
//! This namespace is intentionally non-default. It groups advanced traits,
//! encoding helpers, and concrete key types that are still supported for
//! internal integration or specialist callers, but do not belong on the stable
//! root facade.

pub use tari_crypto::hash_domain;

pub mod encoding {
    pub use tari_crypto::tari_utilities::hex::{from_hex, to_hex, Hex};
    pub use tari_crypto::tari_utilities::{ByteArray, SafePassword};
}

pub mod traits {
    pub use tari_crypto::hashing::{DerivedKeyDomain, DomainSeparatedHasher, DomainSeparation};
    pub use tari_crypto::keys::{PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
}

pub mod keys {
    pub use tari_crypto::ristretto::{RistrettoPublicKey, RistrettoSecretKey};
}
