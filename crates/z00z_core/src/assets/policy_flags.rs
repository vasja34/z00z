// crates/z00z_core/src/assets/policy_flags.rs
//
//! # Asset Policy Flags
//!
//! Named constants for asset policy bit flags, replacing magic numbers
//! throughout the codebase for improved readability and maintainability.
//!
//! ## Flag Definitions
//!
//! Policy flags are stored as a single `u8` with the following bit layout:
//!
//! ```text
//! Bit Position:  7  6  5  4  3  2  1  0
//!                │  │  │  │  │  │  │  │
//! Flag:          -  -  - BR  -  M  F  G
//!
//! G  = Gas (bit 0)       - Can be used for transaction fees
//! F  = Fungible (bit 1)  - Units are interchangeable
//! M  = Mintable (bit 2)  - New units can be created
//! BR = Burnable (bit 4)  - Units can be destroyed
//! -  = Reserved for future use
//! ```
//!
//! ## Examples
//!
//! ```rust
//! use z00z_core::assets::policy_flags::{NONE, GAS, BURNABLE, FUNGIBLE, MINTABLE, has_flag};
//!
//! // Define a standard coin (gas + fungible + burnable)
//! let coin_flags = GAS | FUNGIBLE | BURNABLE;
//! assert_eq!(coin_flags, 0b0001_0011);
//!
//! // Define a governance token (fungible + mintable)
//! let token_flags = FUNGIBLE | MINTABLE;
//! assert_eq!(token_flags, 0b0000_0110);
//!
//! // Check if asset can be burned
//! let flags = 0b0001_0000;
//! assert!(has_flag(flags, BURNABLE));
//! assert!(!has_flag(flags, GAS));
//! ```

/// 🚫 No policy flags set (default/base case)
pub const NONE: u8 = 0b0000_0000;

/// ⛽ Gas flag (bit 0) - Asset can be used for transaction fees
///
/// Assets with this flag can be spent to pay for transaction execution.
/// Typically set for native blockchain tokens.
pub const GAS: u8 = 0b0000_0001;

/// 🔄 Fungible flag (bit 1) - Asset units are interchangeable
///
/// Set for Coins and Tokens where all units have equal value.
/// Not set for NFTs and Void assets which are unique.
pub const FUNGIBLE: u8 = 0b0000_0010;

/// 🪙 Mintable flag (bit 2) - New asset units can be created
///
/// Allows authorized parties to create new units after initial issuance.
/// Used for inflationary tokens, governance tokens, etc.
pub const MINTABLE: u8 = 0b0000_0100;

/// 🔥 Burnable flag (bit 4) - Asset units can be destroyed
///
/// Allows asset holders to permanently destroy units, reducing total supply.
/// Common for deflationary tokenomics and fee burning mechanisms.
pub const BURNABLE: u8 = 0b0001_0000;

/// Native cash keeps one fixed, non-programmable policy-flag profile.
pub const NATIVE_CASH_POLICY_FLAGS: u8 = GAS | FUNGIBLE | BURNABLE;

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if a specific flag is set in the policy flags byte
///
/// # Arguments
///
/// * `flags` - The complete policy flags byte
/// * `flag` - The specific flag constant to check (e.g., GAS, BURNABLE)
///
/// # Returns
///
/// `true` if the flag is set, `false` otherwise
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::policy_flags::{has_flag, GAS, BURNABLE};
///
/// let flags = 0b0001_0001; // Gas + Burnable
/// assert!(has_flag(flags, GAS));
/// assert!(has_flag(flags, BURNABLE));
/// assert!(!has_flag(flags, 0b0000_0010)); // Not fungible
/// ```
#[inline]
pub const fn has_flag(flags: u8, flag: u8) -> bool {
    (flags & flag) != 0
}

/// Combine multiple flags into a single policy byte
///
/// # Arguments
///
/// * `flags` - Variable number of flag constants to combine
///
/// # Returns
///
/// Combined policy flags byte
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::policy_flags::{combine_flags, GAS, FUNGIBLE, BURNABLE};
///
/// let coin_flags = combine_flags(&[GAS, FUNGIBLE, BURNABLE]);
/// assert_eq!(coin_flags, 0b0001_0011);
/// ```
#[inline]
pub const fn combine_flags(flags: &[u8]) -> u8 {
    let mut result = 0u8;
    let mut i = 0;
    while i < flags.len() {
        result |= flags[i];
        i += 1;
    }
    result
}

/// Validate that policy flags only use defined bits
///
/// # Arguments
///
/// * `flags` - Policy flags byte to validate
///
/// # Returns
///
/// `true` if all set bits correspond to defined flags, `false` if unknown bits are set
///
/// # Examples
///
/// ```rust
/// use z00z_core::assets::policy_flags::validate_flags;
///
/// assert!(validate_flags(0b0001_0111)); // Valid: all defined flags
/// assert!(!validate_flags(0b1000_0000)); // Invalid: bit 7 is undefined
/// assert!(!validate_flags(0b0010_0000)); // Invalid: bit 5 is undefined
/// ```
#[inline]
pub const fn validate_flags(flags: u8) -> bool {
    const VALID_MASK: u8 = GAS | FUNGIBLE | MINTABLE | BURNABLE;
    (flags & !VALID_MASK) == 0
}

/// Native cash does not accept arbitrary action pools in Phase 059.
#[inline]
pub const fn native_cash_uses_action_pools() -> bool {
    false
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_constants() {
        assert_eq!(NONE, 0b0000_0000);
        assert_eq!(GAS, 0b0000_0001);
        assert_eq!(FUNGIBLE, 0b0000_0010);
        assert_eq!(MINTABLE, 0b0000_0100);
        assert_eq!(BURNABLE, 0b0001_0000);
    }

    #[test]
    fn test_has_flag() {
        let flags = GAS | BURNABLE; // 0b0001_0001

        assert!(has_flag(flags, GAS));
        assert!(has_flag(flags, BURNABLE));
        assert!(!has_flag(flags, FUNGIBLE));
        assert!(!has_flag(flags, MINTABLE));
    }

    #[test]
    fn test_combine_flags() {
        let flags = combine_flags(&[GAS, FUNGIBLE, BURNABLE]);
        assert_eq!(flags, 0b0001_0011);

        let all_flags = combine_flags(&[GAS, FUNGIBLE, MINTABLE, BURNABLE]);
        assert_eq!(all_flags, 0b0001_0111);

        let no_flags = combine_flags(&[]);
        assert_eq!(no_flags, NONE);
    }

    #[test]
    fn test_validate_flags() {
        // Valid combinations
        assert!(validate_flags(NONE));
        assert!(validate_flags(GAS));
        assert!(validate_flags(GAS | FUNGIBLE));
        assert!(validate_flags(GAS | FUNGIBLE | MINTABLE | BURNABLE));

        // Invalid - undefined bits set
        assert!(!validate_flags(0b1000_0000)); // Bit 7
        assert!(!validate_flags(0b0100_0000)); // Bit 6
        assert!(!validate_flags(0b0010_0000)); // Bit 5
        assert!(!validate_flags(0b0000_1000)); // Bit 3
    }

    #[test]
    fn test_common_combinations() {
        // Standard coin: gas + fungible + burnable
        let coin = GAS | FUNGIBLE | BURNABLE;
        assert_eq!(coin, 0b0001_0011);
        assert!(has_flag(coin, GAS));
        assert!(has_flag(coin, FUNGIBLE));
        assert!(has_flag(coin, BURNABLE));
        assert!(!has_flag(coin, MINTABLE));

        // Governance token: fungible + mintable
        let token = FUNGIBLE | MINTABLE;
        assert_eq!(token, 0b0000_0110);
        assert!(has_flag(token, FUNGIBLE));
        assert!(has_flag(token, MINTABLE));
        assert!(!has_flag(token, GAS));
        assert!(!has_flag(token, BURNABLE));
    }

    #[test]
    fn test_native_cash_policy_profile() {
        assert_eq!(NATIVE_CASH_POLICY_FLAGS, GAS | FUNGIBLE | BURNABLE);
        assert!(!native_cash_uses_action_pools());
    }
}
