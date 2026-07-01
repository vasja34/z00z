use z00z_utils::{prelude::TimeProvider, rng::RngCoreExt, time::TimeError};

use crate::domains::{GenesisNonceDomain, NonceDerivationDomain};
use z00z_crypto::hash::DomainHasher;

use super::nonce_type::{try_get_timestamp_micros, Nonce};

pub fn derive_nonce(
    wallet_seed: &[u8; 32],
    counter: u64,
    timestamp: u64,
    prev_output_hash: &[u8; 32],
) -> Nonce {
    let hash = DomainHasher::<NonceDerivationDomain>::new_with_label("asset_nonce")
        .chain(wallet_seed)
        .chain(counter.to_le_bytes())
        .chain(timestamp.to_le_bytes())
        .chain(prev_output_hash)
        .finalize();

    let mut nonce = [0u8; 32];
    nonce.copy_from_slice(&hash.as_ref()[..32]);
    nonce
}

pub fn derive_nonce_simple(
    wallet_seed: &[u8; 32],
    counter: u64,
    time_provider: &dyn TimeProvider,
) -> Result<Nonce, TimeError> {
    try_derive_nonce_simple(wallet_seed, counter, time_provider)
}

pub fn try_derive_nonce_simple(
    wallet_seed: &[u8; 32],
    counter: u64,
    time_provider: &dyn TimeProvider,
) -> Result<Nonce, TimeError> {
    let timestamp = try_get_timestamp_micros(time_provider)?;
    let prev_hash = [0u8; 32];
    Ok(derive_nonce(wallet_seed, counter, timestamp, &prev_hash))
}

pub fn derive_nonce_minimal(
    rng: &mut (impl rand::RngCore + rand::CryptoRng),
    time_provider: &dyn TimeProvider,
) -> Result<Nonce, TimeError> {
    try_derive_nonce_minimal(rng, time_provider)
}

pub fn try_derive_nonce_minimal(
    rng: &mut (impl rand::RngCore + rand::CryptoRng),
    time_provider: &dyn TimeProvider,
) -> Result<Nonce, TimeError> {
    let mut random_seed = [0u8; 32];
    let mut counter_bytes = [0u8; 8];

    rng.fill_bytes_ext(&mut random_seed);
    rng.fill_bytes_ext(&mut counter_bytes);

    let timestamp = try_get_timestamp_micros(time_provider)?;
    let counter = u64::from_le_bytes(counter_bytes);
    let prev_hash = [0u8; 32];
    Ok(derive_nonce(&random_seed, counter, timestamp, &prev_hash))
}

pub fn derive_genesis_nonce(
    genesis_seed: &[u8; 32],
    definition_id: &[u8; 32],
    serial_id: u32,
) -> Nonce {
    let hash = DomainHasher::<GenesisNonceDomain>::new_with_label("genesis_nonce")
        .chain(genesis_seed)
        .chain(definition_id)
        .chain(serial_id.to_le_bytes())
        .finalize();

    let mut nonce = [0u8; 32];
    nonce.copy_from_slice(&hash.as_ref()[..32]);
    nonce
}
