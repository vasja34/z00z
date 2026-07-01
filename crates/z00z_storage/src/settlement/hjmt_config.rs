use crate::settlement::BucketPolicy;
use crate::settlement::{RootGeneration, HJMT_PROOF_ENVELOPE_VERSION};
use z00z_utils::config::{ConfigSource, EnvConfig};

use super::SettlementStoreError;

pub const SETTLEMENT_BACKEND_MODE_ENV: &str = "Z00Z_SETTLEMENT_BACKEND_MODE";
pub const SETTLEMENT_BUCKET_BITS_ENV: &str = "Z00Z_SETTLEMENT_BUCKET_BITS";
pub const LIVE_BACKEND_GEN: u64 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SettlementBackendMode {
    #[default]
    Hjmt,
}

impl SettlementBackendMode {
    pub const HJMT_NAME: &'static str = "hjmt";

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Hjmt => Self::HJMT_NAME,
        }
    }

    pub fn from_name(raw: &str) -> Result<Self, SettlementStoreError> {
        match raw.trim() {
            "" | Self::HJMT_NAME => Ok(Self::Hjmt),
            _ => Err(SettlementStoreError::Backend(
                "unsupported settlement backend mode".to_string(),
            )),
        }
    }

    pub fn from_env_or_default() -> Result<Self, SettlementStoreError> {
        match env_value(SETTLEMENT_BACKEND_MODE_ENV)? {
            Some(raw) => Self::from_name(&raw),
            None => Ok(Self::default()),
        }
    }
}

pub(super) fn env_value(key: &str) -> Result<Option<String>, SettlementStoreError> {
    EnvConfig
        .get(key)
        .map_err(|err| SettlementStoreError::Backend(format!("invalid {key}: {err}")))
}

pub(crate) fn env_opt(key: &str) -> Option<String> {
    env_value(key).ok().flatten()
}

pub fn bucket_policy_from_env() -> Result<BucketPolicy, SettlementStoreError> {
    let raw = match env_value(SETTLEMENT_BUCKET_BITS_ENV)? {
        Some(raw) => raw,
        None => return Ok(BucketPolicy::default_fixed()),
    };

    let bucket_bits = raw.trim().parse::<u8>().map_err(|err| {
        SettlementStoreError::Backend(format!("invalid {SETTLEMENT_BUCKET_BITS_ENV}: {err}"))
    })?;
    BucketPolicy::new(
        bucket_bits,
        BucketPolicy::DEFAULT_MIN_BUCKET_COUNT,
        BucketPolicy::DEFAULT_MAX_TARGET_LEAF_COUNT,
        BucketPolicy::DEFAULT_COMPATIBILITY_GENERATION,
    )
    .map_err(|err| {
        SettlementStoreError::Backend(format!("invalid {SETTLEMENT_BUCKET_BITS_ENV}: {err}"))
    })
}

pub fn check_live_startup_contract(
    backend: &str,
    generation: u64,
    root_generation: u8,
    proof_version: u16,
) -> Result<(), SettlementStoreError> {
    SettlementBackendMode::from_name(backend)?;
    if generation != LIVE_BACKEND_GEN {
        return Err(SettlementStoreError::Backend(format!(
            "unsupported settlement backend generation: {generation}"
        )));
    }
    if RootGeneration::from_version(root_generation) != Some(RootGeneration::SettlementV1) {
        return Err(SettlementStoreError::Backend(format!(
            "unsupported settlement root generation: {root_generation}"
        )));
    }
    let want = HJMT_PROOF_ENVELOPE_VERSION as u16;
    if proof_version != want {
        return Err(SettlementStoreError::Backend(format!(
            "unsupported settlement proof version: {proof_version}"
        )));
    }
    Ok(())
}
