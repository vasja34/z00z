//! Network RPC types
//!
//! Request/response types for `network.*` JSON-RPC methods.
//!
//! Note: These DTOs carry `ChainType` (chain selection) but are returned by
//! `network.*` methods for historical reasons.

use serde::{Deserialize, Serialize};

use crate::{rpc::types::common::RuntimeOperationStatus, ChainType};

/// Chain settings.
///
/// These settings are session-scoped runtime configuration for `network.*`.
/// Do not embed them into persisted wallet settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeChainSettings {
    pub chain_type: ChainType,
    pub rpc_endpoint: String,
    pub use_tor: bool,
}

/// Chain settings response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeChainSettingsResponse {
    pub settings: RuntimeChainSettings,
}

/// Switch chain response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSwitchChainResponse {
    #[serde(flatten)]
    pub status: RuntimeOperationStatus,
    pub chain: ChainType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Codec, JsonCodec};

    #[test]
    fn test_runtime_chain_settings() {
        let settings = RuntimeChainSettings {
            chain_type: ChainType::Mainnet,
            rpc_endpoint: "https://mainnet.example.com".to_string(),
            use_tor: false,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&settings).unwrap();
        let deserialized: RuntimeChainSettings = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.chain_type, ChainType::Mainnet);
        assert_eq!(deserialized.rpc_endpoint, "https://mainnet.example.com");
        assert!(!deserialized.use_tor);
    }

    #[test]
    fn test_runtime_chain_settings_response() {
        let response = RuntimeChainSettingsResponse {
            settings: RuntimeChainSettings {
                chain_type: ChainType::Testnet,
                rpc_endpoint: "https://testnet.example.com".to_string(),
                use_tor: true,
            },
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeChainSettingsResponse = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.settings.chain_type, ChainType::Testnet);
        assert!(deserialized.settings.use_tor);
    }

    #[test]
    fn test_switch_chain_response_success() {
        let response = RuntimeSwitchChainResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: "Switched to mainnet".to_string(),
            },
            chain: ChainType::Mainnet,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeSwitchChainResponse = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.status.success);
        assert_eq!(deserialized.chain, ChainType::Mainnet);
    }

    #[test]
    fn test_switch_chain_response_failure() {
        let response = RuntimeSwitchChainResponse {
            status: RuntimeOperationStatus {
                success: false,
                message: "Failed to switch".to_string(),
            },
            chain: ChainType::Testnet,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeSwitchChainResponse = codec.deserialize(&bytes).unwrap();

        assert!(!deserialized.status.success);
        assert_eq!(deserialized.status.message, "Failed to switch");
    }

    #[test]
    fn test_all_chain_types() {
        let chains = [ChainType::Mainnet, ChainType::Testnet];

        for chain in chains {
            let settings = RuntimeChainSettings {
                chain_type: chain,
                rpc_endpoint: "endpoint".to_string(),
                use_tor: false,
            };

            let codec = JsonCodec;
            let bytes = codec.serialize(&settings).unwrap();
            let deserialized: RuntimeChainSettings = codec.deserialize(&bytes).unwrap();

            assert_eq!(deserialized.chain_type, chain);
        }
    }
}
