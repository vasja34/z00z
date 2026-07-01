use super::AssetDefinition;

pub(super) fn serialize<S>(
    arc: &std::sync::Arc<AssetDefinition>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    <AssetDefinition as serde::Serialize>::serialize(arc.as_ref(), serializer)
}

pub(super) fn deserialize<'de, D>(
    deserializer: D,
) -> Result<std::sync::Arc<AssetDefinition>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    <AssetDefinition as serde::Deserialize<'de>>::deserialize(deserializer).map(std::sync::Arc::new)
}
