use blake2::{Blake2b512, Digest};
use once_cell::sync::Lazy;
use p3_field::PrimeField64;
use p3_goldilocks::{
    Goldilocks, Poseidon2GoldilocksHL, HL_GOLDILOCKS_8_EXTERNAL_ROUND_CONSTANTS,
    HL_GOLDILOCKS_8_INTERNAL_ROUND_CONSTANTS,
};
use p3_poseidon2::{ExternalLayerConstants, Poseidon2};
use p3_symmetric::Permutation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConsensusHash([u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WalletHash([u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HashFunction {
    Poseidon2,
    Blake2b,
}

pub fn hash_fn_for_domain(domain: &[u8]) -> HashFunction {
    if domain.starts_with(b"Z00Z/")
        || domain.starts_with(b"z00z.consensus.")
        || domain.starts_with(b"z00z.payment.")
        || domain.starts_with(b"z00z.receiver.")
    {
        HashFunction::Poseidon2
    } else {
        HashFunction::Blake2b
    }
}

impl ConsensusHash {
    pub(crate) fn from_poseidon2(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl WalletHash {
    pub(crate) fn from_blake2b(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

pub fn poseidon2_hash(domain: &[u8], data: &[&[u8]]) -> [u8; 32] {
    const WIDTH: usize = 8;
    const RATE: usize = WIDTH - 1;
    const OUT_WORDS: usize = 4;

    let mut packer = WordPacker::new();
    packer.push_frame_bytes(domain);
    packer.push_u64_le(data.len() as u64);
    for item in data {
        packer.push_frame_bytes(item);
    }

    let words = packer.finalize();
    let poseidon = poseidon2_perm();
    let mut state = [Goldilocks::new(0); WIDTH];
    let mut rate_index = 0usize;

    for word in words {
        state[rate_index] += word;
        rate_index += 1;

        if rate_index == RATE {
            poseidon.permute_mut(&mut state);
            rate_index = 0;
        }
    }

    poseidon.permute_mut(&mut state);

    let mut out = [0u8; 32];
    for (index, item) in state.iter().take(OUT_WORDS).enumerate() {
        out[index * 8..(index + 1) * 8].copy_from_slice(&item.as_canonical_u64().to_le_bytes());
    }
    out
}

fn poseidon2_perm() -> &'static Poseidon2GoldilocksHL<8> {
    static INSTANCE: Lazy<Poseidon2GoldilocksHL<8>> = Lazy::new(|| {
        Poseidon2::new(
            ExternalLayerConstants::<Goldilocks, 8>::new_from_saved_array(
                HL_GOLDILOCKS_8_EXTERNAL_ROUND_CONSTANTS,
                Goldilocks::new_array,
            ),
            Goldilocks::new_array(HL_GOLDILOCKS_8_INTERNAL_ROUND_CONSTANTS).to_vec(),
        )
    });

    &INSTANCE
}

struct WordPacker {
    words: Vec<Goldilocks>,
    block: [u8; 8],
    used: usize,
    total_len: u64,
}

impl WordPacker {
    fn new() -> Self {
        Self {
            words: Vec::new(),
            block: [0u8; 8],
            used: 0,
            total_len: 0,
        }
    }

    fn push_u64_le(&mut self, value: u64) {
        self.push_bytes(&value.to_le_bytes());
    }

    fn push_frame_bytes(&mut self, bytes: &[u8]) {
        self.push_bytes(&(bytes.len() as u32).to_le_bytes());
        self.push_bytes(bytes);
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        self.total_len = self.total_len.saturating_add(bytes.len() as u64);

        for &byte in bytes {
            self.block[self.used] = byte;
            self.used += 1;
            if self.used == 8 {
                self.words
                    .push(Goldilocks::new(u64::from_le_bytes(self.block)));
                self.block = [0u8; 8];
                self.used = 0;
            }
        }
    }

    fn finalize(mut self) -> Vec<Goldilocks> {
        let mut out = Vec::with_capacity(self.words.len() + 3);
        out.push(Goldilocks::new(self.total_len));
        out.append(&mut self.words);

        if self.used > 0 {
            out.push(Goldilocks::new(u64::from_le_bytes(self.block)));
        }

        out.push(Goldilocks::new(1));
        out
    }
}

pub fn compute_consensus_hash(domain: &[u8], data: &[&[u8]]) -> ConsensusHash {
    ConsensusHash::from_poseidon2(poseidon2_hash(domain, data))
}

pub fn blake2b_hash(domain: &[u8], data: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Blake2b512::new();
    hasher.update((domain.len() as u32).to_le_bytes());
    hasher.update(domain);

    for item in data {
        hasher.update((item.len() as u32).to_le_bytes());
        hasher.update(item);
    }

    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest[..32]);
    out
}

pub fn compute_wallet_seed_hash(seed: &[u8]) -> WalletHash {
    WalletHash::from_blake2b(blake2b_hash(b"z00z/wallet/seed", &[seed]))
}

pub fn hash_db_record_id(record_type: &str, key: &[u8]) -> WalletHash {
    WalletHash::from_blake2b(blake2b_hash(
        b"z00z/wallet/db_id",
        &[record_type.as_bytes(), key],
    ))
}

pub fn hash_cache_key(leaf_hash: &[u8; 32]) -> WalletHash {
    WalletHash::from_blake2b(blake2b_hash(b"z00z/wallet/cache", &[leaf_hash]))
}
