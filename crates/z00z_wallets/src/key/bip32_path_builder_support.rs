struct Bip44PathComponents {
    purpose: u32,
    asset_type: u32,
    account: u32,
    change: u8,
    address_index: u32,
}

fn required_path_component<T>(value: Option<T>, field: &'static str) -> Result<T, Bip44Error> {
    value.ok_or_else(|| Bip44Error::InvalidPath(format!("{field} not set")))
}

fn required_components(builder: Bip44PathBuilder) -> Result<Bip44PathComponents, Bip44Error> {
    Ok(Bip44PathComponents {
        purpose: required_path_component(builder.purpose, "purpose")?,
        asset_type: required_path_component(builder.asset_type, "asset_type")?,
        account: required_path_component(builder.account, "account")?,
        change: required_path_component(builder.change, "change")?,
        address_index: required_path_component(builder.address_index, "address_index")?,
    })
}

fn validate_path_components(components: &Bip44PathComponents) -> Result<(), Bip44Error> {
    if components.change > 1 {
        return Err(Bip44Error::InvalidChangeValue(u32::from(components.change)));
    }

    if components.account > MAX_BIP32_INDEX {
        return Err(Bip44Error::IndexOutOfRange {
            field: "account",
            value: components.account,
            max: MAX_BIP32_INDEX,
        });
    }

    if components.address_index > MAX_BIP32_INDEX {
        return Err(Bip44Error::IndexOutOfRange {
            field: "address_index",
            value: components.address_index,
            max: MAX_BIP32_INDEX,
        });
    }

    Ok(())
}
