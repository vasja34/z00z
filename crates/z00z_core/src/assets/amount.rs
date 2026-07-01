use z00z_crypto::RANGE_PROOF_BITS;

pub type Amount = u64;

pub const fn max_amount_for_bits(bits: usize) -> u64 {
    if bits == 0 {
        0
    } else if bits >= u64::BITS as usize {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

pub const fn is_amount_in_bits(amount: Amount, bits: usize) -> bool {
    amount <= max_amount_for_bits(bits)
}

pub const fn is_amount_in_range(amount: Amount) -> bool {
    is_amount_in_bits(amount, RANGE_PROOF_BITS)
}

pub const MAX_AMOUNT: u64 = max_amount_for_bits(RANGE_PROOF_BITS);

#[cfg(test)]
mod tests {
    use super::{is_amount_in_bits, is_amount_in_range, max_amount_for_bits, MAX_AMOUNT};
    use z00z_crypto::RANGE_PROOF_BITS;

    #[test]
    fn test_max_amount_proof_width() {
        assert_eq!(MAX_AMOUNT, max_amount_for_bits(RANGE_PROOF_BITS));
        assert!(is_amount_in_range(MAX_AMOUNT));
    }

    #[test]
    fn test_max_amount_bits_bounds() {
        assert_eq!(max_amount_for_bits(0), 0);
        assert_eq!(max_amount_for_bits(1), 1);
        assert_eq!(max_amount_for_bits(63), (1u64 << 63) - 1);
        assert_eq!(max_amount_for_bits(96), max_amount_for_bits(64));
        assert!(!is_amount_in_bits(u64::MAX, 63));
    }
}
