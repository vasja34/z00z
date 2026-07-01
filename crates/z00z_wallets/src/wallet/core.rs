//! Z00ZWallet core entity.
//!
//! Phase 1: data container only.
//!
//! This struct aggregates wallet components (secret storage, persistence, chain/sync,
//! spending pipeline, backup/policy, and utilities) as injectable traits/implementations.

use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use super::{Policy, WalletState};
use crate::backup::{BackupExporter, BackupImporter};
use crate::key::KeyManager;
use crate::persistence::{AssetStorage, ReceiptStorage, TxStorage, WalletStorage};
use crate::receiver::ReceiverManager;
use crate::rpc::types::asset::RuntimeImportAssetResponse;
use crate::rpc::types::common::{PersistWalletId, RuntimeOperationStatus};
use crate::security::SecretStore;
use crate::tx::{AssetSelector, FeeEstimator, LocalVerifier, Prover, Signer, TxAssembler};
use z00z_core::assets::{AssetClass, AssetWire};
use z00z_core::genesis::ChainType;
use z00z_core::Asset;
use z00z_crypto::expert::encoding::{from_hex, to_hex};
use z00z_crypto::DomainHasher;
use z00z_utils::codec::{Codec, JsonCodec};

use crate::domains::WalletIdDomain;

include!("chain_id.rs");
include!("wallet_id.rs");
include!("wallet_kernel.rs");
include!("wallet_record.rs");
include!("entity.rs");
