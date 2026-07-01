use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use z00z_crypto::DomainHasher;
use z00z_utils::codec::to_canonical_json_bytes;

use crate::{
    config_name::validate_underscore_name, domains::ActionDescriptorHashDomain, AssetError,
    ObjectFamily,
};

use super::ActionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredSignatureV1 {
    Owner,
    Issuer,
    Holder,
    Beneficiary,
    Controller,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessRequirementV1 {
    Signature(RequiredSignatureV1),
    RightReference(String),
    VerifierAttestation(String),
    AcceptanceProof,
    ReplayNonce,
    PriorStateRoot,
    DisclosureCommitment,
}

impl Serialize for WitnessRequirementV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            Self::Signature(value) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("signature", value)?;
                map.end()
            }
            Self::RightReference(value) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("right_reference", value)?;
                map.end()
            }
            Self::VerifierAttestation(value) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("verifier_attestation", value)?;
                map.end()
            }
            Self::AcceptanceProof => serializer.serialize_str("acceptance_proof"),
            Self::ReplayNonce => serializer.serialize_str("replay_nonce"),
            Self::PriorStateRoot => serializer.serialize_str("prior_state_root"),
            Self::DisclosureCommitment => serializer.serialize_str("disclosure_commitment"),
        }
    }
}

impl<'de> Deserialize<'de> for WitnessRequirementV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Label(String),
            Signature { signature: RequiredSignatureV1 },
            RightReference { right_reference: String },
            VerifierAttestation { verifier_attestation: String },
        }

        match Wire::deserialize(deserializer)? {
            Wire::Label(label) => match label.as_str() {
                "acceptance_proof" => Ok(Self::AcceptanceProof),
                "replay_nonce" => Ok(Self::ReplayNonce),
                "prior_state_root" => Ok(Self::PriorStateRoot),
                "disclosure_commitment" => Ok(Self::DisclosureCommitment),
                _ => Err(serde::de::Error::custom(format!(
                    "unknown witness requirement label: {label}"
                ))),
            },
            Wire::Signature { signature } => Ok(Self::Signature(signature)),
            Wire::RightReference { right_reference } => Ok(Self::RightReference(right_reference)),
            Wire::VerifierAttestation {
                verifier_attestation,
            } => Ok(Self::VerifierAttestation(verifier_attestation)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEffectV1 {
    NoStateChange,
    Offer,
    Accept,
    Transfer,
    Redeem,
    PartialRedeem,
    Refund,
    Expire,
    Grant,
    Delegate,
    Use,
    Revoke,
    Challenge,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionDescriptorV1 {
    pub label: String,
    pub allowed_input_families: BTreeSet<ObjectFamily>,
    pub allowed_output_families: BTreeSet<ObjectFamily>,
    pub lifecycle_effect: LifecycleEffectV1,
    pub witness_requirements: BTreeSet<WitnessRequirementV1>,
    pub receiver_must_accept: bool,
    pub preserves_beneficiary: bool,
    pub preserves_refund_authority: bool,
}

impl ActionDescriptorV1 {
    pub fn validate(&self) -> Result<(), AssetError> {
        if self.label.trim().is_empty() {
            return Err(AssetError::InvalidAsset(
                "action descriptor label must not be empty".into(),
            ));
        }
        validate_underscore_name("action.label", self.label.as_str())?;

        if self.allowed_input_families.is_empty() {
            return Err(AssetError::InvalidAsset(
                "action descriptor must declare at least one input family".into(),
            ));
        }

        if self.allowed_output_families.is_empty() {
            return Err(AssetError::InvalidAsset(
                "action descriptor must declare at least one output family".into(),
            ));
        }

        if self.receiver_must_accept
            && !self
                .allowed_output_families
                .contains(&ObjectFamily::Voucher)
        {
            return Err(AssetError::InvalidAsset(
                "receiver acceptance is only meaningful for voucher outputs".into(),
            ));
        }

        for requirement in &self.witness_requirements {
            match requirement {
                WitnessRequirementV1::RightReference(reference)
                | WitnessRequirementV1::VerifierAttestation(reference) => {
                    if reference.trim().is_empty() {
                        return Err(AssetError::InvalidAsset(
                            "witness requirement references must not be empty".into(),
                        ));
                    }
                    validate_underscore_name("action.witness_reference", reference)?;
                }
                WitnessRequirementV1::Signature(_)
                | WitnessRequirementV1::AcceptanceProof
                | WitnessRequirementV1::ReplayNonce
                | WitnessRequirementV1::PriorStateRoot
                | WitnessRequirementV1::DisclosureCommitment => {}
            }
        }

        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, AssetError> {
        self.validate()?;
        to_canonical_json_bytes(self)
            .map_err(|err| AssetError::Serialization(err.to_string().into()))
    }

    pub fn action_id(&self) -> Result<ActionId, AssetError> {
        let bytes = self.canonical_bytes()?;
        let mut hasher = DomainHasher::<ActionDescriptorHashDomain>::new_with_label("action");
        hasher.update(bytes);
        let digest = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&digest.as_ref()[..32]);
        Ok(ActionId::new(id))
    }
}
