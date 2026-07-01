use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    actions::ActionPoolId,
    config_name::{validate_domain_name, validate_underscore_name},
    policies::PolicyId,
    AssetError,
};

use super::{VoucherAcceptanceTermsV1, VoucherLifecycleV1, VoucherValidityWindowV1};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VoucherBackingReferenceV1 {
    ReserveCommitment([u8; 32]),
    ConsumedAsset {
        definition_id: [u8; 32],
        serial_id: u32,
    },
    GenesisReserve {
        reserve_id: String,
    },
}

impl Serialize for VoucherBackingReferenceV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            Self::ReserveCommitment(bytes) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("reserve_commitment", bytes)?;
                map.end()
            }
            Self::ConsumedAsset {
                definition_id,
                serial_id,
            } => {
                #[derive(Serialize)]
                struct Wire {
                    definition_id: [u8; 32],
                    serial_id: u32,
                }

                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(
                    "consumed_asset",
                    &Wire {
                        definition_id: *definition_id,
                        serial_id: *serial_id,
                    },
                )?;
                map.end()
            }
            Self::GenesisReserve { reserve_id } => {
                #[derive(Serialize)]
                struct Wire<'a> {
                    reserve_id: &'a str,
                }

                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(
                    "genesis_reserve",
                    &Wire {
                        reserve_id: reserve_id.as_str(),
                    },
                )?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for VoucherBackingReferenceV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ConsumedAssetWire {
            definition_id: [u8; 32],
            serial_id: u32,
        }

        #[derive(Deserialize)]
        struct GenesisReserveWire {
            reserve_id: String,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            ReserveCommitment { reserve_commitment: [u8; 32] },
            ConsumedAsset { consumed_asset: ConsumedAssetWire },
            GenesisReserve { genesis_reserve: GenesisReserveWire },
        }

        match Wire::deserialize(deserializer)? {
            Wire::ReserveCommitment { reserve_commitment } => {
                Ok(Self::ReserveCommitment(reserve_commitment))
            }
            Wire::ConsumedAsset { consumed_asset } => Ok(Self::ConsumedAsset {
                definition_id: consumed_asset.definition_id,
                serial_id: consumed_asset.serial_id,
            }),
            Wire::GenesisReserve { genesis_reserve } => Ok(Self::GenesisReserve {
                reserve_id: genesis_reserve.reserve_id,
            }),
        }
    }
}

impl VoucherBackingReferenceV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        match self {
            Self::ReserveCommitment(bytes) => {
                if bytes.iter().all(|byte| *byte == 0) {
                    return Err(AssetError::InvalidAsset(
                        "voucher reserve commitment must not be zero".into(),
                    ));
                }
            }
            Self::ConsumedAsset { definition_id, .. } => {
                if definition_id.iter().all(|byte| *byte == 0) {
                    return Err(AssetError::InvalidAsset(
                        "voucher consumed-asset definition id must not be zero".into(),
                    ));
                }
            }
            Self::GenesisReserve { reserve_id } => {
                if reserve_id.trim().is_empty() {
                    return Err(AssetError::InvalidAsset(
                        "voucher genesis reserve id must not be empty".into(),
                    ));
                }
                validate_underscore_name("voucher.reserve_id", reserve_id.as_str())?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoucherConfigEntry {
    pub id: String,
    pub domain_name: String,
    pub issuer_fixture: String,
    pub holder_fixture: String,
    pub beneficiary_fixture: String,
    pub backing: VoucherBackingReferenceV1,
    pub face_value: u64,
    pub remaining_value: u64,
    pub policy_id: PolicyId,
    pub action_pool_id: ActionPoolId,
    pub lifecycle: VoucherLifecycleV1,
    pub validity: VoucherValidityWindowV1,
    pub acceptance: VoucherAcceptanceTermsV1,
    pub replay_nonce: [u8; 32],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disclosure_commitment: Option<[u8; 32]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_commitment: Option<[u8; 32]>,
}

impl VoucherConfigEntry {
    pub fn validate(&self) -> Result<(), AssetError> {
        for (field, value) in [
            ("id", self.id.as_str()),
            ("domain_name", self.domain_name.as_str()),
            ("issuer_fixture", self.issuer_fixture.as_str()),
            ("holder_fixture", self.holder_fixture.as_str()),
            ("beneficiary_fixture", self.beneficiary_fixture.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(AssetError::InvalidAsset(
                    format!("voucher.{field} must not be empty").into(),
                ));
            }
        }

        validate_underscore_name("voucher.id", self.id.as_str())?;
        validate_domain_name("voucher.domain_name", self.domain_name.as_str())?;
        validate_underscore_name("voucher.issuer_fixture", self.issuer_fixture.as_str())?;
        validate_underscore_name("voucher.holder_fixture", self.holder_fixture.as_str())?;
        validate_underscore_name(
            "voucher.beneficiary_fixture",
            self.beneficiary_fixture.as_str(),
        )?;

        self.backing.validate()?;
        self.validity.validate()?;
        self.acceptance.validate()?;

        if self.face_value == 0 {
            return Err(AssetError::InvalidAsset(
                "voucher face_value must be greater than zero".into(),
            ));
        }

        if self.remaining_value > self.face_value {
            return Err(AssetError::InvalidAsset(
                "voucher remaining_value must not exceed face_value".into(),
            ));
        }

        if self.remaining_value == 0
            && !matches!(
                self.lifecycle,
                VoucherLifecycleV1::Redeemed
                    | VoucherLifecycleV1::Refunded
                    | VoucherLifecycleV1::Rejected
                    | VoucherLifecycleV1::Expired
            )
        {
            return Err(AssetError::InvalidAsset(
                "live vouchers must keep positive remaining_value".into(),
            ));
        }

        if self.replay_nonce.iter().all(|byte| *byte == 0) {
            return Err(AssetError::InvalidAsset(
                "voucher replay nonce must not be zero".into(),
            ));
        }

        if self
            .disclosure_commitment
            .is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0))
        {
            return Err(AssetError::InvalidAsset(
                "voucher disclosure commitment must not be zero".into(),
            ));
        }

        if self
            .audit_commitment
            .is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0))
        {
            return Err(AssetError::InvalidAsset(
                "voucher audit commitment must not be zero".into(),
            ));
        }

        Ok(())
    }
}
