//! Shared simulator actor model.

use std::collections::HashMap;

use z00z_crypto::Hidden;
use z00z_utils::codec::Value;
use z00z_wallets::{key::ReceiverKeys, receiver::ReceiverCard, wallet::WalletRecord};

/// Thin simulator envelope around wallet primitives.
pub struct SimActor {
    /// Display name used in logs/reports.
    pub name: String,
    /// Runtime password captured during stage-2 wallet creation.
    pub password: Option<String>,
    /// Persistent wallet id used by RPC methods.
    pub wallet_id: String,
    /// Persisted wallet metadata.
    pub record: WalletRecord,
    /// Receiver key material.
    pub keys: ReceiverKeys,
    /// Public receiver card.
    pub card: ReceiverCard,
    /// Simulator plaintext ledger by genesis asset id.
    pub balance: HashMap<[u8; 32], u64>,
    /// Receiver secret used for deterministic setup.
    pub receiver_secret: Hidden<[u8; 32]>,
    /// Active RPC session token (filled in stage 2 after unlock).
    pub session: Option<Value>,
}
