use crate::AssetError;

fn is_domain_char(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_digit()
}

#[must_use]
pub(crate) fn is_domain_name(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty()
        || value.starts_with('.')
        || value.ends_with('.')
        || !value.contains('.')
        || value.contains("..")
    {
        return false;
    }

    value.split('.').all(|segment| {
        !segment.is_empty()
            && !segment.starts_with('_')
            && !segment.ends_with('_')
            && !segment.contains("__")
            && segment.chars().all(is_domain_char)
    })
}

#[must_use]
pub(crate) fn is_underscore_name(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty()
        || value.starts_with('_')
        || value.ends_with('_')
        || value.contains("__")
        || value.contains('.')
    {
        return false;
    }

    value.chars().all(|ch| ch == '_' || is_identifier_char(ch))
}

pub(crate) fn validate_domain_name(field_name: &str, value: &str) -> Result<(), AssetError> {
    if !is_domain_name(value) {
        return Err(AssetError::InvalidAsset(
            format!("{field_name} must use dot-separated domain_name format").into(),
        ));
    }

    Ok(())
}

pub(crate) fn validate_underscore_name(field_name: &str, value: &str) -> Result<(), AssetError> {
    if !is_underscore_name(value) {
        return Err(AssetError::InvalidAsset(
            format!("{field_name} must use underscore-separated identifier format").into(),
        ));
    }

    Ok(())
}
