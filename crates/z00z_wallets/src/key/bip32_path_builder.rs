/// Builder for Bip44Path with validation
///
/// Provides a fluent API for constructing BIP-44 paths with automatic
/// hardening based on BIP-44 specification.
#[derive(Debug, Default)]
pub struct Bip44PathBuilder {
    purpose: Option<u32>,
    asset_type: Option<u32>,
    account: Option<u32>,
    change: Option<u8>,
    address_index: Option<u32>,
}

impl Bip44PathBuilder {
    /// Creates a builder seeded with the Z00Z BIP-44 purpose value.
    pub fn new() -> Self {
        Self {
            purpose: Some(Z00Z_BIP44_PURPOSE),
            ..Default::default()
        }
    }

    /// Sets the hardened SLIP-0044 asset type.
    pub fn asset_type(mut self, value: u32) -> Self {
        self.asset_type = Some(value);
        self
    }

    /// Sets the hardened account index.
    pub fn account(mut self, value: u32) -> Self {
        self.account = Some(value);
        self
    }

    /// Sets the non-hardened change chain component.
    pub fn change(mut self, value: u8) -> Self {
        self.change = Some(value);
        self
    }

    /// Sets the non-hardened address index component.
    pub fn address_index(mut self, value: u32) -> Self {
        self.address_index = Some(value);
        self
    }

    /// Validates the configured components and returns a typed BIP-44 path.
    pub fn build(self) -> Result<Bip44Path, Bip44Error> {
        let components = required_components(self)?;
        validate_path_components(&components)?;
        let path = Bip44Path {
            purpose: ChildNumber::new(components.purpose, true)?,
            asset_type: ChildNumber::new(components.asset_type, true)?,
            account: ChildNumber::new(components.account, true)?,
            change: ChildNumber::new(u32::from(components.change), false)?,
            address_index: ChildNumber::new(components.address_index, false)?,
        };

        Bip44Validator::validate(&path)?;
        Ok(path)
    }
}
