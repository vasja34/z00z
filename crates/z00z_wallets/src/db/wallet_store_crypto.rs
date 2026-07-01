//! RedB-SPEC.v4 wallet crypto (key schedule + envelopes).
//!
//! This module implements the crypto/key-management requirements defined in:
//! - `specs/006-z00z-wallets/RedB-SPEC.md`
//! - `crates/z00z_wallets/src/config/redb-schema.yaml` (authoritative)
//!
//! Notes:
//! - This is Z00Z-owned code (Tari is reference only).
//! - The `.wlt` (redb) persistence path is native-only, but the key schedule and
//!   envelopes are platform-agnostic and are required for browser WASM backends.
//!
//! # AAD Construction Policy
//!
//! All AEAD AAD MUST be unambiguous.
//!
//! For any variable-length field included in AAD, encode it as:
//! - length: `u32` little-endian (4 bytes)
//! - bytes: the field payload
//!
//! This prevents AAD collision attacks caused by concatenation ambiguity.
//!
//! Migration note: any non-canonical AAD decoding must stay isolated to explicit migration-only
//! helpers and must never leak back into the default open path.
//!

use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use z00z_crypto::expert::encoding::SafePassword;
use z00z_crypto::kdf::Argon2Params;
use z00z_crypto::{CryptoError, Hidden};
use z00z_utils::logger::{Logger, TracingLogger};

#[cfg(test)]
#[path = "test_wallet_store_crypto.rs"]
mod tests;
#[path = "wallet_store_crypto_aad.rs"]
mod wallet_store_crypto_aad;
#[path = "wallet_store_crypto_kdf.rs"]
mod wallet_store_crypto_kdf;
#[path = "wallet_store_crypto_models.rs"]
mod wallet_store_crypto_models;

pub use self::wallet_store_crypto_aad::{aad_master_key, aad_object, aad_secret, wallet_aad_id};
use self::wallet_store_crypto_kdf::{
    derive_key_zero_padding, derive_wallet_keys as derive_wallet_keys_current,
};
pub use self::wallet_store_crypto_models::{
    AeadEnvelope, MasterKeyRecord, SecretsKind, SecretsRecord, WalletDerivedKeys,
};

/// Current schema version for wallet database structure
pub const REDB_WALLET_SCHEMA_VERSION: u32 = 4;

/// Current HKDF `info` scheme version used for persisted RedB wallets.
pub const HKDF_INFO_VERSION: u32 = 2;

/// Domain identifiers for AEAD AAD labels.
///
/// These values are part of the persistent wallet crypto contract. Changing them will make
/// previously created `.wlt` files unreadable.
pub const AAD_MASTER_KEY_LABEL: &[u8] = b"master-key:v1";
/// AEAD AAD prefix for secret records
pub const AAD_SECRET_PREFIX: &[u8] = b"secret:v1:";
/// Current persisted secret-AAD format version.
pub const AAD_SECRET_VERSION: u32 = 2;
/// Algorithm identifier for XChaCha20-Poly1305 AEAD
pub const AEAD_ALGO_XCHACHA: &str = "xchacha20poly1305";

// Hash domains live in `crate::domains`.

/// Hard upper bounds for persisted wallet KDF parameters.
///
/// These values are used to validate *untrusted* `wallet.kdf` metadata before running any
/// expensive KDF computation. The goal is to prevent attacker-controlled `.wlt` files from
/// forcing pathological CPU/memory usage during open/unlock.
pub const MAX_MEM_LIMIT_KIB: u32 = z00z_crypto::kdf::MAX_MEM_LIMIT_KIB;
/// Maximum Argon2 iterations accepted from untrusted persisted metadata.
pub const MAX_OPS_LIMIT: u32 = z00z_crypto::kdf::MAX_OPS_LIMIT;
/// Maximum Argon2 parallelism accepted from untrusted persisted metadata.
pub const MAX_PARALLELISM: u32 = z00z_crypto::kdf::MAX_PARALLELISM;
/// Maximum estimated KDF time accepted from untrusted persisted metadata.
pub const MAX_KDF_TIME_MS: u64 = z00z_crypto::kdf::MAX_KDF_TIME_MS;

/// Minimum memory limit for KDF (16 MiB)
pub const MIN_MEM_LIMIT_KIB: u32 = 16 * 1024; // 16 MiB
/// Minimum operations limit for KDF
pub const MIN_OPS_LIMIT: u32 = 1;
/// Minimum parallelism for KDF
pub const MIN_PARALLELISM: u32 = 1;

#[cfg(feature = "test-params-fast")]
const DEFAULT_MEM_BYTES: u64 = 16 * 1024 * 1024;
#[cfg(feature = "test-params-fast")]
const DEFAULT_OPS: u32 = 1;
#[cfg(feature = "test-params-fast")]
const DEFAULT_PAR: u32 = 2;

#[cfg(not(feature = "test-params-fast"))]
const DEFAULT_MEM_BYTES: u64 = 128 * 1024 * 1024;
#[cfg(not(feature = "test-params-fast"))]
const DEFAULT_OPS: u32 = 5;
#[cfg(not(feature = "test-params-fast"))]
const DEFAULT_PAR: u32 = 8;

/// Key derivation function algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KdfAlgo {
    /// Argon2id memory-hard KDF
    Argon2id,
    /// Scrypt memory-hard KDF
    Scrypt,
}

/// Key derivation function parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParams {
    /// KDF algorithm to use
    pub algo: KdfAlgo,
    /// 16..32 bytes.
    pub salt: Vec<u8>,
    /// Bytes (must be a multiple of 1024 for Argon2 parameters).
    pub mem_limit: u64,
    /// Iterations/time cost.
    pub ops_limit: u32,
    /// Parallelism.
    pub parallelism: u32,
    /// Schema-defined version of this KDF record.
    pub version: u16,
}

impl KdfParams {
    /// Current KDF parameters version.
    ///
    /// The wallet persists only the zero-padded Argon2id lane in Phase 036.
    pub const VERSION: u16 = 2;

    /// Default RedB-SPEC.v4-ish Argon2id parameters.
    ///
    /// Memory: 128 MiB, Iterations: 5, Parallelism: 8, Salt: caller-provided.
    /// In test mode (test-params-fast feature), uses reduced parameters for speed.
    pub fn default_argon2id_with_salt(salt: Vec<u8>) -> Self {
        Self {
            algo: KdfAlgo::Argon2id,
            salt,
            mem_limit: DEFAULT_MEM_BYTES,
            ops_limit: DEFAULT_OPS,
            parallelism: DEFAULT_PAR,
            version: Self::VERSION,
        }
    }

    /// Validate KDF parameters coming from untrusted persisted metadata.
    ///
    /// This must be called before running any expensive KDF derivation.
    pub fn validate_untrusted_persisted(&self) -> Result<(), CryptoError> {
        if self.version != Self::VERSION {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_version",
            });
        }

        if self.algo != KdfAlgo::Argon2id {
            return Err(CryptoError::InvalidParameters { param: "kdf_algo" });
        }

        if !(16..=32).contains(&self.salt.len()) {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_salt_length",
            });
        }

        if self.mem_limit == 0 {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_mem_limit",
            });
        }

        if !self.mem_limit.is_multiple_of(1024) {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_mem_limit",
            });
        }

        let mem_kib_u64 = self.mem_limit / 1024;
        if mem_kib_u64 < MIN_MEM_LIMIT_KIB as u64 {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_mem_limit",
            });
        }
        if mem_kib_u64 > MAX_MEM_LIMIT_KIB as u64 {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_mem_limit",
            });
        }

        if self.ops_limit == 0 {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_ops_limit",
            });
        }

        if self.ops_limit < MIN_OPS_LIMIT {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_ops_limit",
            });
        }

        if self.ops_limit > MAX_OPS_LIMIT {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_ops_limit",
            });
        }

        if self.parallelism == 0 {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_parallelism",
            });
        }

        if self.parallelism < MIN_PARALLELISM {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_parallelism",
            });
        }

        if self.parallelism > MAX_PARALLELISM {
            return Err(CryptoError::InvalidParameters {
                param: "kdf_parallelism",
            });
        }

        // Hard time budget to avoid DoS when opening attacker-controlled `.wlt` files.
        let mem_kib_u64 = self.mem_limit / 1024;
        let params = Argon2Params {
            memory: mem_kib_u64
                .try_into()
                .map_err(|_| CryptoError::InvalidParameters {
                    param: "kdf_mem_limit",
                })?,
            iterations: self.ops_limit,
            parallelism: self.parallelism,
        };

        if params.estimate_time_seconds() > (MAX_KDF_TIME_MS as f64 / 1000.0) {
            return Err(CryptoError::InvalidParameters { param: "kdf_time" });
        }

        Ok(())
    }

    fn validate_for_kdf(&self) -> Result<(), CryptoError> {
        self.validate_untrusted_persisted()
    }

    /// Enhanced RedB-SPEC.v4-ish Argon2id parameters (V2).
    ///
    /// Memory: 128 MiB, Iterations: 5, Parallelism: 8, Salt: caller-provided.
    /// Used for new wallets and re-encryption on unlock.
    pub fn enhanced_argon2id_with_salt(salt: Vec<u8>) -> Self {
        #[cfg(feature = "test-params-fast")]
        {
            // Fast parameters for testing: 16 MiB, 1 iteration, parallelism 2
            Self {
                algo: KdfAlgo::Argon2id,
                salt,
                mem_limit: 16 * 1024 * 1024,
                ops_limit: 1,
                parallelism: 2,
                version: Self::VERSION,
            }
        }

        #[cfg(not(feature = "test-params-fast"))]
        {
            // Enhanced production parameters: 128 MiB, 5 iterations, parallelism 8
            Self {
                algo: KdfAlgo::Argon2id,
                salt,
                mem_limit: 128 * 1024 * 1024,
                ops_limit: 5,
                parallelism: 8,
                version: Self::VERSION,
            }
        }
    }
}

/// Fixed-size key material used by RedB-SPEC.
pub type RedbKey32 = [u8; 32];

/// Derive `PW_KEY` (32 bytes) from a password and stored `KdfParams`.
pub fn derive_pw_key(
    password: &SafePassword,
    params: &KdfParams,
) -> Result<Hidden<RedbKey32>, CryptoError> {
    params.validate_for_kdf()?;

    match params.algo {
        KdfAlgo::Argon2id => {
            let mem_kib_u64 = params.mem_limit / 1024;
            let mem_kib: u32 =
                mem_kib_u64
                    .try_into()
                    .map_err(|_| CryptoError::InvalidParameters {
                        param: "kdf_mem_limit",
                    })?;

            let argon2_params = Argon2Params {
                memory: mem_kib,
                iterations: params.ops_limit,
                parallelism: params.parallelism,
            };

            let est_s = argon2_params.estimate_time_seconds();
            if est_s > 3.0 {
                Logger::warn(
                    &TracingLogger,
                    &format!(
                        "Wallet KDF may be slow: estimated {:.1}s (mem_kib={}, iterations={}, parallelism={})",
                        est_s,
                        argon2_params.memory,
                        argon2_params.iterations,
                        argon2_params.parallelism
                    ),
                );
            }

            Logger::info(
                &TracingLogger,
                &format!(
                    "Running wallet KDF (estimated {:.1}s)",
                    est_s.min(MAX_KDF_TIME_MS as f64 / 1000.0)
                ),
            );

            if params.version != KdfParams::VERSION {
                return Err(CryptoError::InvalidParameters {
                    param: "kdf_version",
                });
            }

            let out = derive_key_zero_padding(password, params, &argon2_params)?;

            Ok(Hidden::hide(out))
        }
        KdfAlgo::Scrypt => Err(CryptoError::InvalidParameters { param: "kdf_algo" }),
    }
}

/// Derive DATA/INDEX/INTEGRITY keys from `MASTER_KEY` using HKDF-SHA256.
pub fn derive_wallet_keys(master_key: &RedbKey32) -> Result<WalletDerivedKeys, CryptoError> {
    derive_wallet_keys_current(master_key)
}
