//! RPC types (DTOs) for JSON-RPC methods.
//!
//! Canonical location per redesign spec: `rpc/<domain>_types.rs`.
//! Callers should import DTOs from their owning submodule path, for example
//! `rpc::types::wallet::SessionToken`, instead of relying on flat
//! wildcard re-exports from this module root.

#[path = "app_types.rs"]
pub mod app;
#[path = "asset_types.rs"]
pub mod asset;
#[path = "backup_types.rs"]
pub mod backup;
#[path = "chain_types.rs"]
pub mod chain;
#[path = "common_types.rs"]
pub mod common;
#[path = "events_types.rs"]
pub mod events;
#[path = "key_types.rs"]
pub mod key;
#[path = "network_types.rs"]
pub mod network;
#[path = "object_types.rs"]
pub mod object;
#[path = "security_types.rs"]
pub mod security;
#[path = "storage_types.rs"]
pub mod storage;
#[path = "tx_types.rs"]
pub mod tx;
#[path = "wallet_types.rs"]
pub mod wallet;
