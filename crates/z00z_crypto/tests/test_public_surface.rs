const CARGO_TOML: &str = include_str!("../Cargo.toml");
const AEAD_RS: &str = include_str!("../src/aead/mod.rs");
const AEAD_AAD_RS: &str = include_str!("../src/aead/aead_aad.rs");
const AEAD_ENVELOPE_RS: &str = include_str!("../src/aead/aead_envelope.rs");
const AEAD_ERROR_RS: &str = include_str!("../src/aead/aead_error.rs");
const AEAD_PRIMITIVES_RS: &str = include_str!("../src/aead/aead_primitives.rs");
const ARGON2_KDF_RS: &str = include_str!("../src/kdf/argon2_kdf.rs");
const ARGON2_PARAMS_RS: &str = include_str!("../src/kdf/argon2_params.rs");
const BACKEND_MOD_RS: &str = include_str!("../src/backend/mod.rs");
const BACKEND_BATCH_RS: &str = include_str!("../src/backend/backend_batch.rs");
const BACKEND_COMMITMENT_RS: &str = include_str!("../src/backend/backend_commitment.rs");
const BACKEND_INIT_RS: &str = include_str!("../src/backend/backend_init.rs");
const BACKEND_RANGE_PROOFS_RS: &str = include_str!("../src/backend/backend_range_proofs.rs");
const BACKEND_TARI_RS: &str = include_str!("../src/backend/backend_tari.rs");
const BLAKE2_HASH_RS: &str = include_str!("../src/hash/blake2_hash.rs");
const CRYPTO_CONSTANTS_RS: &str = include_str!("../src/types/crypto_constants.rs");
const HASH_MOD_RS: &str = include_str!("../src/hash/mod.rs");
const HKDF_KDF_RS: &str = include_str!("../src/kdf/hkdf_kdf.rs");
const HMAC_SHA256_RS: &str = include_str!("../src/hash/hmac_sha256.rs");
const LIB_RS: &str = include_str!("../src/lib.rs");
const KDF_DOMAINS_RS: &str = include_str!("../src/kdf/kdf_domains.rs");
const KDF_MOD_RS: &str = include_str!("../src/kdf/mod.rs");
const PROTOCOL_CONSTANTS_RS: &str = include_str!("../src/types/protocol_constants.rs");
const PROTOCOL_MOD_RS: &str = include_str!("../src/protocol/mod.rs");
const README_MD: &str = include_str!("../src/README.md");
const SCALAR_TYPE_RS: &str = include_str!("../src/types/scalar_type.rs");
const SECRET_BYTES_RS: &str = include_str!("../src/kdf/secret_bytes.rs");
const SHA256_HASH_RS: &str = include_str!("../src/hash/sha256_hash.rs");
const VENDOR_RS: &str = include_str!("../src/vendor.rs");

#[test]
fn test_surface_gates_custom_zkpack() {
    assert!(
        CARGO_TOML.contains("experimental-zkpack = []"),
        "Cargo features must expose explicit zkpack experimentation gating"
    );
    assert!(
        CARGO_TOML.contains("readme = \"src/README.md\""),
        "Cargo metadata must point at the same README that documents the gated production surface"
    );
    assert!(
        !LIB_RS.contains("pub mod aead_zkpack"),
        "default facade must not expose the old custom zkpack module name"
    );
    assert!(
        LIB_RS.contains("#[cfg(feature = \"experimental-zkpack\")]"),
        "experimental zkpack surface must be behind an explicit feature gate"
    );
    assert!(
        AEAD_RS.contains(
            "#[cfg(any(test, doctest, feature = \"experimental-zkpack\"))]\npub mod zkpack"
        ),
        "custom zkpack module inside aead.rs must be feature-gated"
    );
    assert!(
        AEAD_RS.contains("#[cfg(any(test, doctest, feature = \"experimental-zkpack\"))]\npub use zkpack::{open_zkpack, seal_zkpack, Pack};"),
        "custom zkpack re-exports inside aead.rs must be feature-gated"
    );
}

#[test]
fn test_readme_documents_production_surface() {
    assert!(
        README_MD.contains("claim_stmt_hash"),
        "README must describe the canonical claim surface as the production claim path"
    );
    assert!(
        README_MD.contains("experimental-zkpack"),
        "README must describe the explicit experimental zkpack feature"
    );
    assert!(
        README_MD.contains("ChaCha20-Poly1305"),
        "README must describe the wallet facade as the blessed production zkpack path"
    );
}

#[test]
fn test_hash_kdf_aead_seams() {
    assert_all_seams_are_present();
    assert_stable_owner_namespaces();
    assert_directory_roots_are_direct();
    assert_aead_facade_seams();
    assert_root_facade_demotes_passthroughs();
}

fn assert_all_seams_are_present() {
    for required in [
        BLAKE2_HASH_RS,
        SHA256_HASH_RS,
        HMAC_SHA256_RS,
        SECRET_BYTES_RS,
        ARGON2_PARAMS_RS,
        ARGON2_KDF_RS,
        HKDF_KDF_RS,
        AEAD_ERROR_RS,
        AEAD_PRIMITIVES_RS,
        AEAD_ENVELOPE_RS,
        AEAD_AAD_RS,
    ] {
        assert!(
            !required.trim().is_empty(),
            "phase 030-03 seam files must exist and be non-empty"
        );
    }
}

fn assert_stable_owner_namespaces() {
    assert!(
        LIB_RS.contains("pub mod hash_policy")
            && LIB_RS.contains("pub mod hash_types")
            && LIB_RS.contains("pub mod kdf_consensus")
            && LIB_RS.contains("pub mod kdf_extended")
            && LIB_RS.contains("pub mod aead_transport")
            && LIB_RS.contains("pub mod expert;")
            && LIB_RS.contains("pub mod vendor;"),
        "lib.rs must keep the approved secondary crypto owner namespaces stable without path aliases"
    );
}

fn assert_directory_roots_are_direct() {
    assert!(
        BACKEND_MOD_RS.contains("mod backend_batch;")
            && BACKEND_MOD_RS.contains("mod backend_commitment;")
            && BACKEND_MOD_RS.contains("mod backend_handles;")
            && BACKEND_MOD_RS.contains("mod backend_tari;")
            && BACKEND_MOD_RS.contains("mod backend_init;")
            && BACKEND_MOD_RS.contains("mod backend_range_proofs;")
            && BACKEND_MOD_RS.contains("pub(crate) use backend_tari::TariCryptoBackend;"),
        "backend module root must own backend helpers and Tari directly"
    );

    assert!(
        HASH_MOD_RS.contains("pub mod convenience;")
            && HASH_MOD_RS.contains("pub mod domains;")
            && HASH_MOD_RS.contains("pub mod policy;")
            && HASH_MOD_RS.contains("pub mod typed;")
            && HASH_MOD_RS.contains("pub mod zk;")
            && HASH_MOD_RS.contains("#[cfg(test)]\nmod test_hash;"),
        "hash module root must own its canonical sibling files directly"
    );

    assert!(
        KDF_DOMAINS_RS.contains("HKDF_INFO_WALLET_KEY")
            && KDF_MOD_RS.contains("pub mod kdf_domains;")
            && KDF_MOD_RS.contains("pub fn derive_pack_nonce")
            && KDF_MOD_RS.contains("pub fn derive_db_encryption_key")
            && KDF_MOD_RS.contains("pub fn derive_encrypt_and_mac_keys")
            && KDF_MOD_RS.contains("pub fn derive_symmetric_key_from_ecdh")
            && KDF_MOD_RS.contains("pub fn generate_hedged_r")
            && KDF_MOD_RS.contains("mod test_kdf;"),
        "kdf module root must own the public helper surface and test seam"
    );

    assert!(
        PROTOCOL_MOD_RS.contains("pub mod commitments;")
            && PROTOCOL_MOD_RS.contains("pub mod ecdh;")
            && PROTOCOL_MOD_RS.contains("pub mod range_proofs;")
            && PROTOCOL_MOD_RS.contains("pub mod stealth_bind;")
            && PROTOCOL_MOD_RS.contains("pub mod zkpack;"),
        "protocol module root must own protocol submodules directly"
    );

    assert!(
        VENDOR_RS.contains("crate::expert::encoding") && !VENDOR_RS.contains("pub mod expert;"),
        "vendor module root must reuse the canonical root expert lane directly"
    );
}

fn assert_aead_facade_seams() {
    assert!(
        AEAD_RS.contains("mod aead_aad;")
            && AEAD_RS.contains("mod aead_envelope;")
            && AEAD_RS.contains("mod aead_error;")
            && AEAD_RS.contains("mod aead_primitives;"),
        "aead facade must delegate to explicit sibling seam modules"
    );

    assert!(
        AEAD_RS.contains("pub mod test_only")
            && !AEAD_RS.contains("pub use aead_test_only::seal_with_nonce_TEST_ONLY;"),
        "test-only AEAD helpers must live under an explicit non-production namespace"
    );
}

fn assert_root_facade_demotes_passthroughs() {
    for forbidden in [
        "PedersenCommitmentFactory",
        "CommitmentSignature",
        "DiffieHellmanSharedSecret",
        "pub use tari_crypto::",
        "pub use expert::hash_domain",
        "pub use expert::encoding::{",
        "pub use expert::keys::{",
        "pub use expert::traits::{",
        "pub mod commitments;",
        "pub mod ecdh;",
        "pub use protocol::{commitments, ecdh, range_proofs, stealth_bind, zkpack};",
        "pub use kdf::kdf_domains;",
        "pub mod range_proofs;",
        "pub mod stealth_bind;",
        "pub mod zkpack;",
    ] {
        assert!(
            !LIB_RS.contains(forbidden),
            "stable root must demote vendor passthrough `{}` out of lib.rs",
            forbidden
        );
    }
}

#[test]
fn test_aead_test_helpers_hidden() {
    assert!(
        AEAD_RS.contains("#[cfg(any(test, feature = \"test-params-fast\", feature = \"test-utils\"))]\npub mod test_only"),
        "test-only AEAD namespace must remain cfg-gated"
    );
    assert!(
        !LIB_RS.contains("seal_with_nonce_TEST_ONLY"),
        "stable root must not expose caller-controlled nonce helpers"
    );
}

#[test]
fn test_supporting_backend_seams_exist() {
    for required in [
        PROTOCOL_CONSTANTS_RS,
        CRYPTO_CONSTANTS_RS,
        SCALAR_TYPE_RS,
        BACKEND_INIT_RS,
        BACKEND_COMMITMENT_RS,
        BACKEND_RANGE_PROOFS_RS,
        BACKEND_BATCH_RS,
    ] {
        assert!(
            !required.trim().is_empty(),
            "phase 030-03 task 2 seam files must exist and be non-empty"
        );
    }

    assert!(
        !BACKEND_TARI_RS.contains("#[path = \"backend_")
            && !BACKEND_TARI_RS.contains("#[path = \"test_backend_tari_suite.rs\"]"),
        "backend_tari.rs must use canonical backend sibling paths without path shims"
    );

    assert!(
        LIB_RS.contains("pub use types::{")
            && !LIB_RS.contains("pub mod protocol_constants")
            && !LIB_RS.contains("pub mod crypto_constants")
            && !LIB_RS.contains("pub mod scalar_type"),
        "lib.rs must keep types.rs as the only public owner facade for supporting crypto types"
    );
}
