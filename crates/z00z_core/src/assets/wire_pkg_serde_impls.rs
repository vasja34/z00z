fn is_false(value: &bool) -> bool {
    !*value
}

fn ensure_asset_pkg_json_size(bytes: &[u8], context: &str) -> Result<(), AssetError> {
    if bytes.len() > ASSET_PKG_JSON_MAX_BYTES {
        return Err(AssetError::InvalidAsset(Cow::Owned(format!(
            "asset pkg json {context} failed: payload too large: {} bytes (limit: {} bytes)",
            bytes.len(),
            ASSET_PKG_JSON_MAX_BYTES,
        ))));
    }

    Ok(())
}

fn decode_hex<const N: usize>(value: &str, field: &str) -> Result<[u8; N], AssetError> {
    let bytes = hex::decode(value)
        .map_err(|_| AssetError::InvalidAsset(Cow::Owned(format!("{field}: invalid hex"))))?;
    bytes
        .try_into()
        .map_err(|_| AssetError::InvalidAsset(Cow::Owned(format!("{field}: invalid length"))))
}

fn sig_to_hex(sig: &KernelSignature) -> String {
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(sig.get_public_nonce().as_bytes());
    bytes[32..].copy_from_slice(sig.get_signature().as_bytes());
    hex::encode(bytes)
}

fn sig_from_hex(value: &str) -> Result<KernelSignature, AssetError> {
    let bytes = decode_hex::<64>(value, "owner_signature")?;
    let nonce = Z00ZRistrettoPoint::try_from_bytes(
        bytes[..32]
            .try_into()
            .map_err(|_| AssetError::InvalidAsset(Cow::Borrowed("owner_signature: invalid nonce bytes")))?,
    )
    .map_err(|_| AssetError::InvalidAsset(Cow::Borrowed("owner_signature: invalid nonce bytes")))?;
    let scalar = Z00ZScalar::try_from_bytes(
        bytes[32..]
            .try_into()
            .map_err(|_| AssetError::InvalidAsset(Cow::Borrowed("owner_signature: invalid scalar bytes")))?,
    )
    .map_err(|_| {
        AssetError::InvalidAsset(Cow::Borrowed("owner_signature: invalid scalar bytes"))
    })?;

    Ok(KernelSignature::new(
        nonce.reveal().clone(),
        scalar.reveal().clone(),
    ))
}

fn decode_hex_de<const N: usize, E>(value: &str, field: &str) -> Result<[u8; N], E>
where
    E: serde::de::Error,
{
    decode_hex::<N>(value, field).map_err(E::custom)
}

fn parse_commitment<E>(value: &str) -> Result<Commitment, E>
where
    E: serde::de::Error,
{
    let bytes = decode_hex_de::<32, E>(value, "commitment")?;
    z00z_crypto::Commitment::from_bytes(&bytes)
        .map(|commitment| commitment.as_commitment().clone())
        .map_err(|_| E::custom("commitment: invalid bytes"))
}

fn parse_range_proof<E>(value: &str) -> Result<RangeProof, E>
where
    E: serde::de::Error,
{
    hex::decode(value).map_err(|_| E::custom("range_proof: invalid hex"))
}

fn parse_owner_signature<E>(value: &str) -> Result<KernelSignature, E>
where
    E: serde::de::Error,
{
    sig_from_hex(value).map_err(E::custom)
}

fn parse_owner_pub<E>(value: &str) -> Result<Z00ZRistrettoPoint, E>
where
    E: serde::de::Error,
{
    let bytes = decode_hex_de::<32, E>(value, "owner_pub")?;
    Z00ZRistrettoPoint::try_from_bytes(bytes).map_err(|_| E::custom("owner_pub: invalid bytes"))
}

fn parse_enc_pack<E>(value: &str) -> Result<ZkPackEncrypted, E>
where
    E: serde::de::Error,
{
    let bytes = hex::decode(value).map_err(|_| E::custom("enc_pack: invalid hex"))?;
    ZkPackEncrypted::from_bytes(&bytes).map_err(|_| E::custom("enc_pack: invalid canonical bytes"))
}

fn parse_r_pub<E>(value: &str) -> Result<[u8; 32], E>
where
    E: serde::de::Error,
{
    decode_hex_de::<32, E>(value, "r_pub")
}

fn parse_owner_tag<E>(value: &str) -> Result<[u8; 32], E>
where
    E: serde::de::Error,
{
    decode_hex_de::<32, E>(value, "owner_tag")
}

fn parse_optional<T, E, F>(value: Option<String>, parse: F) -> Result<Option<T>, E>
where
    E: serde::de::Error,
    F: FnOnce(&str) -> Result<T, E> + Copy,
{
    value.as_deref().map(parse).transpose()
}

impl From<&AssetDefinition> for DefinitionPkg {
    fn from(def: &AssetDefinition) -> Self {
        Self {
            id: hex::encode(def.id),
            class: def.class,
            name: def.name.clone(),
            symbol: def.symbol.clone(),
            decimals: def.decimals,
            serials: def.serials,
            nominal: def.nominal,
            domain_name: def.domain_name.clone(),
            version: def.version,
            crypto_version: def.crypto_version,
            policy_flags: def.policy_flags,
            metadata: def.metadata.clone(),
        }
    }
}

impl TryFrom<DefinitionPkg> for AssetDefinition {
    type Error = AssetError;

    fn try_from(value: DefinitionPkg) -> Result<Self, Self::Error> {
        DefinitionWire {
            id: decode_hex::<32>(&value.id, "definition.id")?,
            class: value.class,
            name: value.name,
            symbol: value.symbol,
            decimals: value.decimals,
            serials: value.serials,
            nominal: value.nominal,
            domain_name: value.domain_name,
            version: value.version,
            crypto_version: value.crypto_version,
            policy_flags: value.policy_flags,
            metadata: value.metadata,
        }
        .try_into()
    }
}

impl AssetPkgWire {
    pub fn try_from_wire(wire: &AssetWire) -> Result<Self, AssetError> {
        if wire.secret.is_some() {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "public asset DTO must not carry secret material",
            )));
        }

        Ok(Self {
            definition: wire.definition.clone(),
            serial_id: wire.serial_id,
            amount: wire.amount,
            commitment: wire.commitment.clone(),
            range_proof: wire.range_proof.clone(),
            nonce: wire.nonce,
            lock_height: wire.lock_height,
            is_burned: wire.is_burned,
            is_frozen: wire.is_frozen,
            is_slashed: wire.is_slashed,
            owner_pub: wire.owner_pub.clone(),
            owner_signature: wire.owner_signature.clone(),
            r_pub: wire.r_pub,
            owner_tag: wire.owner_tag,
            enc_pack: wire.enc_pack.clone(),
            tag16: wire.tag16,
            leaf_ad_id: wire.leaf_ad_id,
        })
    }

    pub fn from_wire(wire: &AssetWire) -> Self {
        Self::try_from_wire(wire).expect("public AssetPkgWire cannot carry secret material")
    }

    pub fn try_from_asset(asset: &Asset) -> Result<Self, AssetError> {
        Self::try_from_wire(&AssetWire::from_asset(asset))
    }

    pub fn from_asset(asset: &Asset) -> Self {
        Self::try_from_asset(asset).expect("public AssetPkgWire cannot carry secret material")
    }

    pub fn to_wire(self) -> Result<AssetWire, AssetError> {
        Ok(AssetWire {
            definition: self.definition,
            serial_id: self.serial_id,
            amount: self.amount,
            commitment: self.commitment,
            range_proof: self.range_proof,
            nonce: self.nonce,
            lock_height: self.lock_height,
            is_burned: self.is_burned,
            owner_pub: self.owner_pub,
            owner_signature: self.owner_signature,
            is_frozen: self.is_frozen,
            is_slashed: self.is_slashed,
            r_pub: self.r_pub,
            owner_tag: self.owner_tag,
            enc_pack: self.enc_pack,
            secret: None,
            tag16: self.tag16,
            leaf_ad_id: self.leaf_ad_id,
        })
    }

    pub fn to_asset(self) -> Result<Asset, AssetError> {
        self.to_wire()?.to_asset()
    }

    pub fn validate(&self) -> Result<(), AssetError> {
        self.clone().to_wire()?.validate()
    }

    fn to_serde(&self) -> Result<AssetPkgSerde, String> {
        Ok(AssetPkgSerde {
            definition: DefinitionPkg::from(&self.definition),
            serial_id: self.serial_id,
            amount: self.amount,
            commitment: hex::encode(self.commitment.as_bytes()),
            range_proof: self.range_proof.as_ref().map(hex::encode),
            nonce: hex::encode(self.nonce),
            lock_height: self.lock_height,
            is_burned: self.is_burned,
            is_frozen: self.is_frozen,
            is_slashed: self.is_slashed,
            owner_pub: self
                .owner_pub
                .as_ref()
                .map(|value| hex::encode(value.to_bytes())),
            owner_signature: self.owner_signature.as_ref().map(sig_to_hex),
            r_pub: self.r_pub.map(hex::encode),
            owner_tag: self.owner_tag.map(hex::encode),
            enc_pack: self
                .enc_pack
                .as_ref()
                .map(|value| {
                    value
                        .to_bytes()
                        .map(hex::encode)
                        .map_err(|_| "enc_pack: invalid v1 bytes".to_string())
                })
                .transpose()?,
            tag16: self.tag16,
            leaf_ad_id: self.leaf_ad_id.map(hex::encode),
        })
    }

    fn from_serde<E>(value: AssetPkgSerde) -> Result<Self, E>
    where
        E: serde::de::Error,
    {
        parse_pkg_fields::<E>(value)
    }
}

pub fn encode_asset_pkg_json(dto: &AssetPkgWire) -> Result<Vec<u8>, AssetError> {
    let codec = JsonCodec;
    codec.serialize(dto).map_err(|error| {
        AssetError::Serialization(Cow::Owned(format!("asset pkg json encode failed: {error}")))
    })
}

pub fn decode_asset_pkg_json(bytes: &[u8]) -> Result<AssetPkgWire, AssetError> {
    ensure_asset_pkg_json_size(bytes, "decode")?;

    if payload_has_secret_field(bytes)? {
        return Err(AssetError::InvalidAsset(Cow::Borrowed(
            "asset pkg json decode failed: forbidden field: secret",
        )));
    }

    let codec = JsonCodec;
    codec.deserialize(bytes).map_err(|error| {
        AssetError::InvalidAsset(Cow::Owned(format!("asset pkg json decode failed: {error}")))
    })
}
