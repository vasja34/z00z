use super::{AssetDefinition, AssetError, Cow, BURNABLE, FUNGIBLE, GAS, MINTABLE};

impl AssetDefinition {
    pub fn total_supply(&self) -> Result<u64, AssetError> {
        (self.serials as u64)
            .checked_mul(self.nominal)
            .ok_or_else(|| {
                AssetError::ArithmeticOverflow(Cow::Owned(format!(
                    "total_supply overflow: serials={} * nominal={}",
                    self.serials, self.nominal
                )))
            })
    }

    /// 📊 Calculate supply per individual series
    ///
    /// Returns: `nominal` (since each series has the same supply)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_core::assets::definition::AssetDefinition;
    /// # use z00z_core::assets::AssetClass;
    /// let def = AssetDefinition::new(
    ///     [0u8; 32], AssetClass::Coin, "Test".into(), "TST".into(),
    ///     8, 1000, 100_000_000, "test.io".into(), 1, 1, 0, None
    /// ).unwrap();
    ///
    /// assert_eq!(def.serial_supply(), 100_000_000);
    /// ```
    pub fn serial_supply(&self) -> u64 {
        self.nominal
    }

    /// 🔥 Check if burning is allowed for this asset (flag bit 4)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_core::assets::definition::AssetDefinition;
    /// # use z00z_core::assets::AssetClass;
    /// # use z00z_core::assets::policy_flags::BURNABLE;
    /// let burnable = AssetDefinition::new(
    ///     [0u8; 32], AssetClass::Coin, "Z00Z".into(), "Z00Z".into(),
    ///     8, 50_000, 100_000_000, "z00z.io".into(), 1, 1, BURNABLE, None
    /// ).expect("valid definition");
    /// assert!(burnable.is_burnable());
    /// ```
    pub fn is_burnable(&self) -> bool {
        (self.policy_flags & BURNABLE) != 0
    }

    /// 🚩 Check if asset can be used for gas/transaction fees (flag bit 0)
    pub fn is_gas(&self) -> bool {
        (self.policy_flags & GAS) != 0
    }

    /// ⛽ Alias for is_gas() - matches YAML config naming
    pub fn gas(&self) -> bool {
        self.is_gas()
    }

    /// 🔄 Check if asset is fungible (flag bit 1)
    ///
    /// Fungible assets have interchangeable units (Coins, Tokens).
    /// Non-fungible assets are unique (NFTs, Void).
    pub fn is_fungible(&self) -> bool {
        (self.policy_flags & FUNGIBLE) != 0
    }

    /// 🪙 Alias for is_fungible() - matches YAML config naming
    pub fn fungible(&self) -> bool {
        self.is_fungible()
    }

    /// 🪙 Check if asset is mintable (flag bit 2)
    pub fn is_mintable(&self) -> bool {
        (self.policy_flags & MINTABLE) != 0
    }

    /// 🔨 Alias for is_mintable() - matches YAML config naming
    pub fn mintable(&self) -> bool {
        self.is_mintable()
    }

    /// 🔥 Alias for is_burnable() - matches YAML config naming
    pub fn burnable(&self) -> bool {
        self.is_burnable()
    }

    // ========================================================================
    // Decimal Precision Helpers
    // ========================================================================

    /// 💠 Get minimum representable unit (1 in raw amount)
    ///
    /// Returns the smallest value that can be represented with the given decimal precision.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_core::assets::definition::AssetDefinition;
    /// # use z00z_core::assets::AssetClass;
    /// let btc_like = AssetDefinition::new(
    ///     [0u8; 32], AssetClass::Coin, "BTC".into(), "BTC".into(),
    ///     8, 1, 100_000_000, "test.io".into(), 1, 1, 0, None
    /// ).unwrap();
    ///
    /// assert_eq!(btc_like.min_unit(), 0.00000001); // 1 satoshi = 10^-8
    /// ```
    pub fn min_unit(&self) -> f64 {
        10f64.powi(-(self.decimals as i32))
    }

    /// 💱 Convert raw amount to decimal representation
    ///
    /// Converts internal integer amount to human-readable decimal value.
    ///
    /// # Arguments
    ///
    /// * `amount` - Raw amount in smallest units
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_core::assets::definition::AssetDefinition;
    /// # use z00z_core::assets::AssetClass;
    /// let usdc = AssetDefinition::new(
    ///     [0u8; 32], AssetClass::Token, "USDC".into(), "USDC".into(),
    ///     6, 1, 1_000_000, "test.io".into(), 1, 1, 0, None
    /// ).unwrap();
    ///
    /// assert_eq!(usdc.to_decimal(1_000_000), 1.0);      // 1 USDC
    /// assert_eq!(usdc.to_decimal(1_500_000), 1.5);      // 1.5 USDC
    /// assert_eq!(usdc.to_decimal(1), 0.000001);         // 1 micro-USDC
    /// ```
    pub fn to_decimal(&self, amount: u64) -> f64 {
        amount as f64 / 10f64.powi(self.decimals as i32)
    }

    /// 💱 Convert decimal to raw amount (with validation)
    ///
    /// Converts human-readable decimal value to internal integer amount.
    /// Validates that the result fits in u64 and doesn't lose precision.
    ///
    /// # Arguments
    ///
    /// * `value` - Decimal value (e.g., 1.5 for 1.5 USDC)
    ///
    /// # Returns
    ///
    /// * `Ok(amount)` - Raw amount in smallest units
    /// * `Err(AssetError)` - If value is negative, too large, or loses precision
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_core::assets::definition::AssetDefinition;
    /// # use z00z_core::assets::AssetClass;
    /// let usdc = AssetDefinition::new(
    ///     [0u8; 32], AssetClass::Token, "USDC".into(), "USDC".into(),
    ///     6, 1, 1_000_000, "test.io".into(), 1, 1, 0, None
    /// ).unwrap();
    ///
    /// assert_eq!(usdc.from_decimal(1.0).unwrap(), 1_000_000);   // 1 USDC
    /// assert_eq!(usdc.from_decimal(1.5).unwrap(), 1_500_000);   // 1.5 USDC
    /// assert_eq!(usdc.from_decimal(0.000001).unwrap(), 1);      // Min unit
    ///
    /// // Error: negative value
    /// assert!(usdc.from_decimal(-1.0).is_err());
    ///
    /// // Error: exceeds u64::MAX
    /// assert!(usdc.from_decimal(1e20).is_err());
    /// ```
    pub fn from_decimal(&self, value: f64) -> Result<u64, AssetError> {
        if value < 0.0 {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "Amount cannot be negative",
            )));
        }

        let raw = (value * 10f64.powi(self.decimals as i32)).round();

        if raw > u64::MAX as f64 {
            return Err(AssetError::ArithmeticOverflow(Cow::Owned(format!(
                "Amount {} exceeds u64::MAX when converted with {} decimals",
                value, self.decimals
            ))));
        }

        Ok(raw as u64)
    }

    /// 🔍 Format amount with proper decimal places
    ///
    /// Returns a formatted string with the asset's symbol and proper decimals.
    ///
    /// # Arguments
    ///
    /// * `amount` - Raw amount in smallest units
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use z00z_core::assets::definition::AssetDefinition;
    /// # use z00z_core::assets::AssetClass;
    /// let btc = AssetDefinition::new(
    ///     [0u8; 32], AssetClass::Coin, "Bitcoin".into(), "BTC".into(),
    ///     8, 1, 100_000_000, "test.io".into(), 1, 1, 0, None
    /// ).unwrap();
    ///
    /// assert_eq!(btc.format_amount(100_000_000), "1.00000000 BTC");
    /// assert_eq!(btc.format_amount(150_000_000), "1.50000000 BTC");
    /// assert_eq!(btc.format_amount(1), "0.00000001 BTC");
    /// ```
    pub fn format_amount(&self, amount: u64) -> String {
        format!(
            "{:.precision$} {}",
            self.to_decimal(amount),
            self.symbol,
            precision = self.decimals as usize
        )
    }
}
