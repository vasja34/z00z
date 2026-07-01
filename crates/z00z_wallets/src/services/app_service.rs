//! App service (Phase 1 service boundary)
//!
//! This service represents the application shell that owns one or more wallets.
//!
//! Phase 1 focus:
//! - provide a dedicated boundary for app-owned concerns
//! - route chain selection through the app rather than the wallet service
//! - keep wallet creation orchestrated here, with core-level validation and service-level persistence

use std::sync::Arc;

use z00z_utils::rng::{RngCoreExt, SystemRngProvider};
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

use crate::key::{MnemonicLanguage, SeedPhrase24};
use z00z_crypto::expert::encoding::SafePassword;

use crate::app::AppKernel;
use crate::app::Z00ZApp;
use crate::rpc::types::chain::{
    RuntimeBlockInfo, RuntimeScanStatus, RuntimeStartScanParams, RuntimeStartScanResponse,
};
use crate::rpc::types::common::{PersistWalletId, RuntimeOperationStatus};
use crate::rpc::types::network::{
    RuntimeChainSettings, RuntimeChainSettingsResponse, RuntimeSwitchChainResponse,
};
use crate::rpc::types::wallet::{
    PersistWalletDiscovery, PersistWalletInfo, RuntimeCreateWalletResponse,
    RuntimeDeleteWalletResponse, RuntimeExportWalletResponse, RuntimeImportWalletResponse,
    RuntimeRecoverFromSeedResponse, WalletSource,
};
use crate::services::chain_service::ChainServiceError;
use crate::services::{ChainService, NetworkService, WalletService};
use crate::{ChainType, WalletError, WalletResult};

use crate::db::WalletIdentity;

use crate::security::password::PasswordValidator;
use crate::services::seed_phrase::generate_seed_phrase_24_english;

include!("app_service_core.rs");
include!("app_service_construction.rs");
include!("app_chain_network.rs");
include!("app_wallet_lifecycle.rs");
include!("app_seed_password.rs");

#[cfg(test)]
include!("test_app_service_suite.rs");
