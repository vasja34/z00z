/// BIP-44 path structure with explicit hardening
///
/// Represents a complete BIP-44 derivation path with all components:
/// `m / purpose' / asset_type' / account' / change / address_index`
///
/// # Examples
///
/// ```
/// use z00z_wallets::key::{Bip44Path, Z00Z_BIP44_ASSET};
/// use std::str::FromStr;
///
/// // Parse from BIP-44 string format
/// let path = Bip44Path::from_str(&format!(
///     "m/44'/{asset}'/0'/0/0",
///     asset = Z00Z_BIP44_ASSET
/// ))?;
///
/// // Access components
/// assert!(path.purpose().is_hardened());
/// assert_eq!(path.purpose().index(), 44);
/// # Ok::<(), z00z_wallets::key::Bip44Error>(())
/// ```
///
/// # Structure
///
/// - `purpose`: Always 44' (hardened) - BIP-44 standard
/// - `asset_type`: SLIP-0044 coin type (hardened)
/// - `account`: Account index (hardened)
/// - `change`: 0=external chain, 1=internal chain (non-hardened)
/// - `address_index`: Address index within chain (non-hardened)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bip44Path {
    purpose: ChildNumber,
    asset_type: ChildNumber,
    account: ChildNumber,
    change: ChildNumber,
    address_index: ChildNumber,
}

impl ConstantTimeEq for Bip44Path {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.purpose.index().ct_eq(&other.purpose.index())
            & u8::from(self.purpose.is_hardened()).ct_eq(&u8::from(other.purpose.is_hardened()))
            & self.asset_type.index().ct_eq(&other.asset_type.index())
            & u8::from(self.asset_type.is_hardened())
                .ct_eq(&u8::from(other.asset_type.is_hardened()))
            & self.account.index().ct_eq(&other.account.index())
            & u8::from(self.account.is_hardened()).ct_eq(&u8::from(other.account.is_hardened()))
            & self.change.index().ct_eq(&other.change.index())
            & u8::from(self.change.is_hardened()).ct_eq(&u8::from(other.change.is_hardened()))
            & self
                .address_index
                .index()
                .ct_eq(&other.address_index.index())
            & u8::from(self.address_index.is_hardened())
                .ct_eq(&u8::from(other.address_index.is_hardened()))
    }
}

impl std::hash::Hash for Bip44Path {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.purpose.index().hash(state);
        self.asset_type.index().hash(state);
        self.account.index().hash(state);
        self.change.index().hash(state);
        self.address_index.index().hash(state);
    }
}

impl Bip44Path {
    /// Create a new Z00Z BIP-44 path
    pub fn new_z00z(account: u32, change: u8, address_index: u32) -> Result<Self, Bip44Error> {
        if (VIEW_KEY_ACCOUNT_OFFSET..2 * VIEW_KEY_ACCOUNT_OFFSET).contains(&account) {
            return Err(Bip44Error::InvalidPath(format!(
                "account {} is in view key namespace [{}..{}), use account < {} for spend keys",
                account,
                VIEW_KEY_ACCOUNT_OFFSET,
                2 * VIEW_KEY_ACCOUNT_OFFSET,
                VIEW_KEY_ACCOUNT_OFFSET
            )));
        }

        Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(account)
            .change(change)
            .address_index(address_index)
            .build()
    }

    /// Returns the hardened BIP-44 purpose component.
    pub fn purpose(&self) -> ChildNumber {
        self.purpose
    }

    /// Returns the hardened SLIP-0044 asset type component.
    pub fn asset_type(&self) -> ChildNumber {
        self.asset_type
    }

    /// Returns the hardened account component.
    pub fn account(&self) -> ChildNumber {
        self.account
    }

    /// Encodes the path components in little-endian order for domain hashing.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20);
        bytes.extend_from_slice(&self.purpose.index().to_le_bytes());
        bytes.extend_from_slice(&self.asset_type.index().to_le_bytes());
        bytes.extend_from_slice(&self.account.index().to_le_bytes());
        bytes.extend_from_slice(&self.change.index().to_le_bytes());
        bytes.extend_from_slice(&self.address_index.index().to_le_bytes());
        bytes
    }

    #[cfg(test)]
    pub(crate) fn new_unchecked_for_tests(
        purpose: (u32, bool),
        asset_type: (u32, bool),
        account: (u32, bool),
        change: (u32, bool),
        address_index: (u32, bool),
    ) -> Self {
        Bip44Path {
            purpose: ChildNumber::new(purpose.0, purpose.1).unwrap(),
            asset_type: ChildNumber::new(asset_type.0, asset_type.1).unwrap(),
            account: ChildNumber::new(account.0, account.1).unwrap(),
            change: ChildNumber::new(change.0, change.1).unwrap(),
            address_index: ChildNumber::new(address_index.0, address_index.1).unwrap(),
        }
    }

    /// Returns the non-hardened change chain component.
    pub fn change(&self) -> ChildNumber {
        self.change
    }

    /// Returns the non-hardened address index component.
    pub fn address_index(&self) -> ChildNumber {
        self.address_index
    }

    /// Converts the typed path into a `bip32::DerivationPath`.
    pub fn to_derivation_path(&self) -> DerivationPath {
        let mut path = DerivationPath::default();
        path.push(self.purpose);
        path.push(self.asset_type);
        path.push(self.account);
        path.push(self.change);
        path.push(self.address_index);
        path
    }

    /// Maps a spend-key path into its reserved view-key namespace counterpart.
    pub fn to_view_key_path(&self) -> Result<Self, Bip44Error> {
        let base_account = self.account().index();
        if base_account >= VIEW_KEY_ACCOUNT_OFFSET {
            return Err(Bip44Error::InvalidPath(format!(
                "account {} already in view key namespace (>= {})",
                base_account, VIEW_KEY_ACCOUNT_OFFSET
            )));
        }

        let view_account = base_account.checked_add(VIEW_KEY_ACCOUNT_OFFSET).ok_or(
            Bip44Error::IndexOutOfRange {
                field: "account",
                value: base_account,
                max: MAX_BIP32_INDEX,
            },
        )?;

        if view_account > MAX_BIP32_INDEX {
            return Err(Bip44Error::IndexOutOfRange {
                field: "account",
                value: view_account,
                max: MAX_BIP32_INDEX,
            });
        }

        Bip44PathBuilder::new()
            .asset_type(Z00Z_BIP44_ASSET)
            .account(view_account)
            .change(self.change().index() as u8)
            .address_index(self.address_index().index())
            .build()
    }

    /// Maps a reserved view-key path back into the spend-key namespace.
    pub fn to_spend_key_path(&self) -> Result<Self, Bip44Error> {
        let base_account = self.account().index();
        if base_account < VIEW_KEY_ACCOUNT_OFFSET {
            return Err(Bip44Error::InvalidPath(format!(
                "account {} is not in view key namespace (< {})",
                base_account, VIEW_KEY_ACCOUNT_OFFSET
            )));
        }

        let spend_account =
            base_account
                .checked_sub(VIEW_KEY_ACCOUNT_OFFSET)
                .ok_or(Bip44Error::InvalidPath(
                    "account underflow when deriving spend key path".into(),
                ))?;

        if spend_account >= VIEW_KEY_ACCOUNT_OFFSET {
            return Err(Bip44Error::InvalidPath(format!(
                "invalid view key account {}: spend account {} would be >= {}",
                base_account, spend_account, VIEW_KEY_ACCOUNT_OFFSET
            )));
        }

        Bip44Path::new_z00z(
            spend_account,
            self.change().index() as u8,
            self.address_index().index(),
        )
    }

    /// Returns `true` when the account lives in the reserved view-key namespace.
    pub fn is_view_key_path(&self) -> bool {
        let account = self.account().index();
        (VIEW_KEY_ACCOUNT_OFFSET..2 * VIEW_KEY_ACCOUNT_OFFSET).contains(&account)
    }

    /// Returns `true` when the account lives in the standard spend-key namespace.
    pub fn is_spend_key_path(&self) -> bool {
        self.account().index() < VIEW_KEY_ACCOUNT_OFFSET
    }

    /// Returns the companion spend/view path for the current namespace.
    pub fn corresponding_path(&self) -> Result<Self, Bip44Error> {
        if self.is_view_key_path() {
            self.to_spend_key_path()
        } else if self.is_spend_key_path() {
            self.to_view_key_path()
        } else {
            Err(Bip44Error::InvalidPath(format!(
                "account {} is neither in spend key namespace [0, {}) nor view key namespace [{}, {})",
                self.account().index(),
                VIEW_KEY_ACCOUNT_OFFSET,
                VIEW_KEY_ACCOUNT_OFFSET,
                2 * VIEW_KEY_ACCOUNT_OFFSET
            )))
        }
    }

    /// Builds the default external payment path for account `0`.
    pub fn payment(index: u32) -> Result<Self, Bip44Error> {
        Bip44Path::new_z00z(0, 0, index)
    }

    /// Builds an external payment path for the supplied account.
    pub fn payment_for_account(account: u32, index: u32) -> Result<Self, Bip44Error> {
        Bip44Path::new_z00z(account, 0, index)
    }

    /// Builds the default internal change path for account `0`.
    pub fn change_path(index: u32) -> Result<Self, Bip44Error> {
        Bip44Path::new_z00z(0, 1, index)
    }

    /// Builds an internal change path for the supplied account.
    pub fn change_path_for_account(account: u32, index: u32) -> Result<Self, Bip44Error> {
        Bip44Path::new_z00z(account, 1, index)
    }

    /// Returns `true` when the path targets the external payment chain.
    pub fn is_payment(&self) -> bool {
        self.change().index() == 0
    }

    /// Returns `true` when the path targets the internal change chain.
    pub fn is_change(&self) -> bool {
        self.change().index() == 1
    }

    /// Returns `true` when every component satisfies the canonical Z00Z BIP-44 shape.
    pub fn is_standard(&self) -> bool {
        self.purpose().index() == Z00Z_BIP44_PURPOSE
            && self.asset_type().index() == Z00Z_BIP44_ASSET
            && self.purpose().is_hardened()
            && self.asset_type().is_hardened()
            && self.account().is_hardened()
            && !self.change().is_hardened()
            && !self.address_index().is_hardened()
    }
}

impl FromStr for Bip44Path {
    type Err = Bip44Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("m/") {
            return Err(Bip44Error::InvalidPath("must start with 'm/'".to_string()));
        }

        let path_str = s
            .strip_prefix("m/")
            .ok_or_else(|| Bip44Error::InvalidPath("failed to strip prefix".to_string()))?;
        let components: Vec<&str> = path_str.split('/').collect();

        if components.len() != 5 {
            return Err(Bip44Error::InvalidPath(
                "BIP-44 requires exactly 5 components".to_string(),
            ));
        }

        let purpose = components[0]
            .parse::<ChildNumber>()
            .map_err(|e| Bip44Error::InvalidPath(format!("purpose: {}", e)))?;
        let asset_type = components[1]
            .parse::<ChildNumber>()
            .map_err(|e| Bip44Error::InvalidPath(format!("asset_type: {}", e)))?;
        let account = components[2]
            .parse::<ChildNumber>()
            .map_err(|e| Bip44Error::InvalidPath(format!("account: {}", e)))?;
        let change = components[3]
            .parse::<ChildNumber>()
            .map_err(|e| Bip44Error::InvalidPath(format!("change: {}", e)))?;
        let address_index = components[4]
            .parse::<ChildNumber>()
            .map_err(|e| Bip44Error::InvalidPath(format!("address_index: {}", e)))?;

        let path = Bip44Path {
            purpose,
            asset_type,
            account,
            change,
            address_index,
        };

        Bip44Validator::validate(&path)?;
        Ok(path)
    }
}

impl std::fmt::Display for Bip44Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "m/{}/{}/{}/{}/{}",
            self.purpose, self.asset_type, self.account, self.change, self.address_index
        )
    }
}

impl From<Bip44Path> for DerivationPath {
    fn from(path: Bip44Path) -> DerivationPath {
        path.to_derivation_path()
    }
}
