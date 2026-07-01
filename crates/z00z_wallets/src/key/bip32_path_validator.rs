/// BIP-44 validator with strict hardening and change/index rules.
pub struct Bip44Validator;

impl Bip44Validator {
    /// Validate BIP-44 hardening rules
    ///
    /// # Arguments
    ///
    /// * `path` - Bip44Path to validate
    ///
    /// # Returns
    ///
    /// `Result<(), Bip44Error>` - Ok if valid, error with specific violation
    ///
    /// # Validation Rules
    ///
    /// 1. Purpose must be hardened (44')
    /// 2. Purpose value must be 44
    /// 3. Asset type must be hardened
    /// 4. Asset type value must match `Z00Z_BIP44_ASSET`
    /// 5. Account must be hardened
    /// 6. Change must be non-hardened
    /// 7. Change value must be 0 or 1
    /// 8. Address index must be non-hardened
    pub fn validate(path: &Bip44Path) -> Result<(), Bip44Error> {
        // Purpose must be hardened (44')
        if !path.purpose.is_hardened() {
            return Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::PurposeNotHardened,
                component: "purpose".to_string(),
            });
        }

        // Purpose value must be 44
        if path.purpose.index() != Z00Z_BIP44_PURPOSE {
            return Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::PurposeValueMismatch,
                component: "purpose".to_string(),
            });
        }

        // Asset type must be hardened
        if !path.asset_type.is_hardened() {
            return Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::AssetTypeNotHardened,
                component: "asset_type".to_string(),
            });
        }

        // Asset type value must match Z00Z coin type
        if path.asset_type.index() != Z00Z_BIP44_ASSET {
            return Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::AssetTypeValueMismatch,
                component: "asset_type".to_string(),
            });
        }

        // Account must be hardened
        if !path.account.is_hardened() {
            return Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::AccountNotHardened,
                component: "account".to_string(),
            });
        }

        // Change must be non-hardened
        if path.change.is_hardened() {
            return Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::ChangeIsHardened,
                component: "change".to_string(),
            });
        }

        // Change must be 0 or 1
        let change_value = path.change.index();
        if change_value > 1 {
            return Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::InvalidChangeValue,
                component: "change".to_string(),
            });
        }

        // Address index must be non-hardened
        if path.address_index.is_hardened() {
            return Err(Bip44Error::NonStandardPath {
                reason: Bip44ViolationReason::AddressIndexIsHardened,
                component: "address_index".to_string(),
            });
        }

        Ok(())
    }
}
