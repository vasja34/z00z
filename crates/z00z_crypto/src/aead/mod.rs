#[cfg(test)]
pub(crate) use chacha20poly1305::Key;

mod aead_aad;
mod aead_envelope;
mod aead_error;
mod aead_primitives;
#[cfg(any(test, feature = "test-params-fast", feature = "test-utils"))]
mod aead_test_only;
#[cfg(any(test, doctest, feature = "experimental-zkpack"))]
mod aead_zkpack;
pub mod transport;
#[cfg(any(test, doctest, feature = "experimental-zkpack"))]
pub mod zkpack {
    pub use super::aead_zkpack::*;
}

pub use aead_aad::{build_aad, build_aad_multipart, build_aad_multipart_extended};
pub use aead_envelope::{open, open_extended_aad, seal, seal_extended_aad};
pub use aead_error::AeadError;
pub use aead_primitives::{
    random_nonce, ENVELOPE_HEADER_SIZE, MAX_AAD_SIZE, MAX_AAD_SIZE_EXTENDED,
    MAX_AEAD_ENVELOPE_SIZE, MAX_AEAD_PLAINTEXT_SIZE, MIN_ENVELOPE_SIZE, POLY1305_TAG_SIZE,
    XCHACHA20_POLY1305_ID, XCHACHA_KEY_SIZE, XCHACHA_NONCE_SIZE,
};

#[cfg(not(target_arch = "wasm32"))]
pub use aead_envelope::seal_with_rng;

#[cfg(all(not(target_arch = "wasm32"), test))]
pub(crate) use aead_primitives::random_nonce_with_rng;
#[cfg(any(test, feature = "test-params-fast", feature = "test-utils"))]
pub(crate) use aead_primitives::xchacha20poly1305_encrypt;
#[cfg(test)]
pub(crate) use aead_primitives::{encrypt_with_random_nonce, xchacha20poly1305_decrypt};
#[cfg(any(test, feature = "test-params-fast", feature = "test-utils"))]
pub mod test_only {
    pub use super::aead_test_only::seal_with_nonce_TEST_ONLY;
}

pub mod types {
    pub use chacha20poly1305::{
        aead::{Aead, Payload},
        Key, KeyInit, XChaCha20Poly1305, XNonce,
    };
}

#[cfg(test)]
mod test_aead;

pub use transport::{
    decrypt_asset_pack, decrypt_asset_package_transport, encrypt_asset_pack,
    encrypt_asset_package_transport, AssetPackCt,
};
#[cfg(any(test, doctest, feature = "experimental-zkpack"))]
pub use zkpack::{open_zkpack, seal_zkpack, Pack};
