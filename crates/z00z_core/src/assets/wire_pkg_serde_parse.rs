pub fn payload_has_secret_field(bytes: &[u8]) -> Result<bool, AssetError> {
    ensure_asset_pkg_json_size(bytes, "parse")?;

    let codec = JsonCodec;
    let value: z00z_utils::codec::Value = codec.deserialize(bytes).map_err(|error| {
        AssetError::InvalidAsset(Cow::Owned(format!("asset pkg json parse failed: {error}")))
    })?;

    Ok(value
        .as_object()
        .map(|root| root.contains_key("secret"))
        .unwrap_or(false))
}

fn parse_core_fields<E>(
    definition: DefinitionPkg,
    commitment: String,
    range_proof: Option<String>,
    nonce: String,
) -> Result<CorePkgParsed, E>
where
    E: serde::de::Error,
{
    Ok(CorePkgParsed {
        definition: definition.try_into().map_err(E::custom)?,
        commitment: parse_commitment::<E>(&commitment)?,
        range_proof: parse_optional(range_proof, parse_range_proof::<E>)?,
        nonce: decode_hex_de::<32, E>(&nonce, "nonce")?,
    })
}

fn parse_owner_fields<E>(
    owner_pub: Option<String>,
    owner_signature: Option<String>,
    r_pub: Option<String>,
    owner_tag: Option<String>,
    enc_pack: Option<String>,
) -> Result<OwnerPkgParsed, E>
where
    E: serde::de::Error,
{
    Ok(OwnerPkgParsed {
        owner_pub: parse_optional(owner_pub, parse_owner_pub::<E>)?,
        owner_signature: parse_optional(owner_signature, parse_owner_signature::<E>)?,
        r_pub: parse_optional(r_pub, parse_r_pub::<E>)?,
        owner_tag: parse_optional(owner_tag, parse_owner_tag::<E>)?,
        enc_pack: parse_optional(enc_pack, parse_enc_pack::<E>)?,
    })
}

fn parse_leaf_ad_id<E>(value: Option<String>) -> Result<Option<[u8; 32]>, E>
where
    E: serde::de::Error,
{
    parse_optional(value, parse_r_pub::<E>)
}

fn ensure_full_stealth_ad_id<E>(
    owner: &OwnerPkgParsed,
    leaf_ad_id: Option<[u8; 32]>,
) -> Result<Option<[u8; 32]>, E>
where
    E: serde::de::Error,
{
    let has_full_stealth =
        owner.r_pub.is_some() && owner.owner_tag.is_some() && owner.enc_pack.is_some();
    if has_full_stealth && leaf_ad_id.is_none() {
        return Err(E::custom("full stealth fields require leaf_ad_id"));
    }

    Ok(leaf_ad_id)
}

fn parse_pkg_fields<E>(value: AssetPkgSerde) -> Result<AssetPkgWire, E>
where
    E: serde::de::Error,
{
    let AssetPkgSerde {
        definition,
        serial_id,
        amount,
        commitment,
        range_proof,
        nonce,
        lock_height,
        is_burned,
        is_frozen,
        is_slashed,
        owner_pub,
        owner_signature,
        r_pub,
        owner_tag,
        enc_pack,
        tag16,
        leaf_ad_id,
    } = value;

    let core = parse_core_fields::<E>(definition, commitment, range_proof, nonce)?;
    let owner = parse_owner_fields::<E>(owner_pub, owner_signature, r_pub, owner_tag, enc_pack)?;
    let leaf_ad_id = ensure_full_stealth_ad_id::<E>(&owner, parse_leaf_ad_id::<E>(leaf_ad_id)?)?;

    Ok(AssetPkgWire {
        definition: core.definition,
        serial_id,
        amount,
        commitment: core.commitment,
        range_proof: core.range_proof,
        nonce: core.nonce,
        lock_height,
        is_burned,
        is_frozen,
        is_slashed,
        owner_pub: owner.owner_pub,
        owner_signature: owner.owner_signature,
        r_pub: owner.r_pub,
        owner_tag: owner.owner_tag,
        enc_pack: owner.enc_pack,
        tag16,
        leaf_ad_id,
    })
}

impl serde::Serialize for AssetPkgWire {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !serializer.is_human_readable() {
            return self
                .clone()
                .to_wire()
                .map_err(serde::ser::Error::custom)?
                .serialize(serializer);
        }

        self.to_serde()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AssetPkgWire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !deserializer.is_human_readable() {
            let wire = AssetWire::deserialize(deserializer)?;
            return Self::try_from_wire(&wire).map_err(serde::de::Error::custom);
        }

        let value = AssetPkgSerde::deserialize(deserializer)?;
        Self::from_serde::<D::Error>(value)
    }
}