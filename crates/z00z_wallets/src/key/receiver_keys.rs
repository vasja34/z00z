#[cfg(any(test, feature = "test-params-fast"))]
use std::cell::Cell;
use std::path::Path;
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "test-params-fast")]
use std::time::{Duration, Instant};

use subtle::{Choice, ConstantTimeEq};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::expert::traits::DomainSeparation;
use z00z_crypto::{
    aead,
    domains::{ReceiverIdDomain, ViewKeyDomain},
    frame_bytes,
    hash_zk::{hash_to_scalar_zk, hash_zk},
    kdf::{derive_argon2id32_key, Argon2Params},
    sign_kernel_signature, verify_kernel_signature, Hidden,
    KernelSignature as Z00ZSchnorrSignature, Z00ZRistrettoPoint, Z00ZScalar,
};
use z00z_utils::{
    io::{read_file, write_file, IoError},
    rng::{RngCoreExt, SystemRngProvider},
    time::{SystemTimeProvider, TimeProvider},
};

use crate::domains::{
    IdentitySignatureDomain, WalletBIP44Domain, WalletIdentityKeyHashProdDomain,
    WalletViewKeyHashProdDomain,
};
use crate::key::Bip44Path;
use crate::receiver::{ReceiverCard, ReceiverCardError};

const SEC_VER_1: u8 = 1;
const SALT_LEN: usize = 32;
const ENC_AAD: &[u8] = b"z00z.wallet.stealth.receiver_secret.v1";
#[cfg(test)]
static ID_GEN_COUNT: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-params-fast"))]
thread_local! {
    // Keep the failure hook local to the current test thread to avoid
    // cross-test interference in parallel release runs.
    static FAIL_USABLE: Cell<bool> = const { Cell::new(false) };
}

include!("receiver_keys_secret.rs");
include!("receiver_keys_identity.rs");
include!("receiver_keys_bundle.rs");

#[cfg(test)]
mod tests {
    include!("test_receiver_keys_suite.rs");
}
