impl serde::Serialize for Bip44Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Bip44Path", 5)?;
        state.serialize_field("purpose", &self.purpose.index())?;
        state.serialize_field("asset_type", &self.asset_type.index())?;
        state.serialize_field("account", &self.account.index())?;
        state.serialize_field("change", &self.change.index())?;
        state.serialize_field("address_index", &self.address_index.index())?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Bip44Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_bip44_path(deserializer)
    }
}

fn deserialize_bip44_path<'de, D>(deserializer: D) -> Result<Bip44Path, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    struct Bip44PathRaw {
        purpose: u32,
        asset_type: u32,
        account: u32,
        change: u32,
        address_index: u32,
    }

    let raw = <Bip44PathRaw as serde::Deserialize>::deserialize(deserializer)?;
    let purpose = ChildNumber::new(raw.purpose, true).map_err(serde::de::Error::custom)?;
    let asset_type = ChildNumber::new(raw.asset_type, true).map_err(serde::de::Error::custom)?;
    let account = ChildNumber::new(raw.account, true).map_err(serde::de::Error::custom)?;
    let change = ChildNumber::new(raw.change, false).map_err(serde::de::Error::custom)?;
    let address_index =
        ChildNumber::new(raw.address_index, false).map_err(serde::de::Error::custom)?;

    let path = Bip44Path {
        purpose,
        asset_type,
        account,
        change,
        address_index,
    };

    Bip44Validator::validate(&path).map_err(serde::de::Error::custom)?;
    Ok(path)
}
