// ============================================================================
// ENTROPY VALIDATION (PHASE 12)
// ============================================================================

/// Errors returned by entropy validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum EntropyError {
    /// The seed is empty.
    #[error("empty seed")]
    EmptySeed,

    /// The seed bytes are uniform (all 0x00 or all 0xFF).
    #[error("uniform seed bytes (all 0x00 or all 0xFF)")]
    UniformBytes,

    /// One or more heuristic warnings were produced.
    #[error("heuristic warnings detected; use a cryptographically secure random source")]
    HeuristicWarnings(Vec<EntropyWarning>),
}

/// Warnings produced by heuristic entropy checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EntropyWarning {
    /// The number of set bits is outside the acceptable range.
    UnusualBitCount { bits_set: u32 },

    /// The seed is a repeating 4-, 8-, 16-, or 32-byte pattern.
    RepeatingPattern,

    /// The seed contains an overly long run of 0x00 bytes.
    LongZeroRun,

    /// The seed bytes form a simple sequential pattern.
    SequentialBytes,

    /// The seed has too few unique byte values.
    LowUniqueBytes { unique: u32 },

    /// The estimated Shannon entropy is suspiciously low.
    LowShannonEntropy { bits_per_byte_x100: u16 },
}

impl fmt::Display for EntropyWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnusualBitCount { bits_set } => write!(f, "unusual bit count: {bits_set}"),
            Self::RepeatingPattern => write!(f, "repeating byte pattern"),
            Self::LongZeroRun => write!(f, "long zero-byte run"),
            Self::SequentialBytes => write!(f, "sequential byte pattern"),
            Self::LowUniqueBytes { unique } => write!(f, "low unique byte count: {unique}"),
            Self::LowShannonEntropy { bits_per_byte_x100 } => {
                let whole = bits_per_byte_x100 / 100;
                let frac = bits_per_byte_x100 % 100;
                write!(f, "low Shannon entropy: {whole}.{frac:02} bits/byte")
            }
        }
    }
}

fn has_critical_warnings(warnings: &[EntropyWarning]) -> bool {
    warnings.iter().any(|warning| {
        matches!(
            warning,
            EntropyWarning::RepeatingPattern
                | EntropyWarning::LongZeroRun
                | EntropyWarning::SequentialBytes
                | EntropyWarning::LowUniqueBytes { .. }
                | EntropyWarning::LowShannonEntropy { .. }
        )
    })
}

fn warning_texts(warnings: &[EntropyWarning]) -> Vec<String> {
    warnings.iter().map(ToString::to_string).collect()
}

fn warning_error(warnings: &[EntropyWarning]) -> String {
    format!(
        "heuristic warnings detected: {}",
        warning_texts(warnings).join("; ")
    )
}

/// Validate seed entropy and return heuristic warnings.
///
/// Catastrophic cases are returned as hard errors.
pub(crate) fn validate_entropy_with_warnings(
    seed: &[u8],
) -> Result<Vec<EntropyWarning>, EntropyError> {
    fn estimate_shannon_entropy(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut freq = [0u32; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;
        for &count in &freq {
            if count > 0 {
                let p = (count as f64) / len;
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    fn is_repeating_pattern(seed: &[u8], chunk_len: usize) -> bool {
        if chunk_len == 0 || seed.len() < chunk_len * 2 || !seed.len().is_multiple_of(chunk_len) {
            return false;
        }

        let first = &seed[0..chunk_len];
        seed.chunks(chunk_len)
            .all(|chunk| chunk.len() == chunk_len && chunk == first)
    }

    if seed.is_empty() {
        return Err(EntropyError::EmptySeed);
    }

    if seed.iter().all(|&b| b == 0) || seed.iter().all(|&b| b == 0xFF) {
        return Err(EntropyError::UniformBytes);
    }

    let mut warnings = Vec::new();

    let total_bits = (seed.len() as u32) * 8;
    let min_bits_set = total_bits / 8;
    let max_bits_set = total_bits - min_bits_set;

    let bits_set: u32 = seed.iter().map(|b| b.count_ones()).sum();
    if !(min_bits_set..=max_bits_set).contains(&bits_set) {
        warnings.push(EntropyWarning::UnusualBitCount { bits_set });
    }

    if is_repeating_pattern(seed, 4)
        || is_repeating_pattern(seed, 8)
        || is_repeating_pattern(seed, 16)
        || is_repeating_pattern(seed, 32)
    {
        warnings.push(EntropyWarning::RepeatingPattern);
    }

    let mut zero_run = 0u32;
    for &byte in seed {
        if byte == 0 {
            zero_run += 1;
            if zero_run > 8 {
                warnings.push(EntropyWarning::LongZeroRun);
                break;
            }
        } else {
            zero_run = 0;
        }
    }

    if seed.len() >= 2 {
        let is_seq_inc = seed.windows(2).all(|w| w[1] == w[0].wrapping_add(1));
        let is_seq_dec = seed.windows(2).all(|w| w[1] == w[0].wrapping_sub(1));

        if is_seq_inc || is_seq_dec {
            warnings.push(EntropyWarning::SequentialBytes);
        }
    }

    let mut seen = [false; 256];
    let mut unique = 0u32;
    for &byte in seed {
        let slot = &mut seen[byte as usize];
        if !*slot {
            *slot = true;
            unique += 1;
        }
    }

    let min_unique = ((seed.len() as u32) / 4).max(8);
    if unique < min_unique {
        warnings.push(EntropyWarning::LowUniqueBytes { unique });
    }

    let shannon = estimate_shannon_entropy(seed);
    let max_entropy = (seed.len() as f64).log2().min(8.0);
    let min_entropy = max_entropy * 0.70;
    if shannon < min_entropy {
        let bits_per_byte_x100 = (shannon * 100.0).round().clamp(0.0, u16::MAX as f64) as u16;
        warnings.push(EntropyWarning::LowShannonEntropy { bits_per_byte_x100 });
    }

    Ok(warnings)
}

/// Validate seed entropy.
pub(crate) fn validate_entropy(seed: &[u8]) -> Result<(), EntropyError> {
    let warnings = validate_entropy_with_warnings(seed)?;
    if warnings.is_empty() {
        return Ok(());
    }

    if has_critical_warnings(&warnings) {
        return Err(EntropyError::HeuristicWarnings(warnings));
    }

    let logger = TracingLogger;
    for warning in &warnings {
        let msg = format!("Entropy heuristic warning: {:?}", warning);
        logger.warn(&msg);
    }

    Ok(())
}

/// Validate seed entropy and keep non-fatal warnings observable to callers.
pub fn validate_entropy_result(seed: &[u8]) -> RuntimeValidationResult {
    match validate_entropy_with_warnings(seed) {
        Ok(warnings) if warnings.is_empty() => RuntimeValidationResult::valid(),
        Ok(warnings) if has_critical_warnings(&warnings) => {
            RuntimeValidationResult::invalid(warning_error(&warnings))
        }
        Ok(warnings) => RuntimeValidationResult::valid_with_warnings(warning_texts(&warnings)),
        Err(EntropyError::HeuristicWarnings(warnings)) => {
            RuntimeValidationResult::invalid(warning_error(&warnings))
        }
        Err(err) => RuntimeValidationResult::invalid(err.to_string()),
    }
}

#[cfg(test)]
mod entropy_tests {
    use super::*;

    #[test]
    fn test_entropy_all_zero_fails() {
        let seed = [0u8; 64];
        assert!(matches!(
            validate_entropy(&seed),
            Err(EntropyError::UniformBytes)
        ));
    }

    #[test]
    fn test_entropy_all_ff_fails() {
        let seed = [0xffu8; 64];
        assert!(matches!(
            validate_entropy(&seed),
            Err(EntropyError::UniformBytes)
        ));
    }

    #[test]
    fn test_entropy_repeating_pattern_fails() {
        let seed = [0x01u8, 0x02, 0x03, 0x04].repeat(16);
        assert!(matches!(
            validate_entropy(&seed),
            Err(EntropyError::HeuristicWarnings(_))
        ));
    }

    #[test]
    fn test_entropy_repeating_half_fails() {
        let mut half = [0u8; 32];
        for (i, b) in half.iter_mut().enumerate() {
            *b = i as u8;
        }

        let mut seed = [0u8; 64];
        seed[..32].copy_from_slice(&half);
        seed[32..].copy_from_slice(&half);

        assert!(matches!(
            validate_entropy(&seed),
            Err(EntropyError::HeuristicWarnings(_))
        ));
    }

    #[test]
    fn test_entropy_zero_run_fails() {
        let mut seed = [0u8; 64];
        seed[0] = 0x7f;
        seed[1] = 0x3c;
        seed[2..=10].fill(0x00);
        seed[11] = 0xaa;
        assert!(matches!(
            validate_entropy(&seed),
            Err(EntropyError::HeuristicWarnings(_))
        ));
    }

    #[test]
    fn test_entropy_random_ok() {
        let mut seed = [0u8; 64];
        assert!(getrandom::getrandom(&mut seed).is_ok());
        assert!(validate_entropy(&seed).is_ok());
    }

    #[test]
    fn test_entropy_sequential_fails() {
        let mut seed = [0u8; 64];
        for (i, b) in seed.iter_mut().enumerate() {
            *b = i as u8;
        }

        assert!(matches!(
            validate_entropy(&seed),
            Err(EntropyError::HeuristicWarnings(_))
        ));
    }

    #[test]
    fn test_entropy_low_unique_fails() {
        let mut seed = [0u8; 64];
        for (i, b) in seed.iter_mut().enumerate() {
            let block = (i / 8) as u8;
            *b = (block.wrapping_add(i as u8)) % 8;
        }

        assert!(matches!(
            validate_entropy(&seed),
            Err(EntropyError::HeuristicWarnings(_))
        ));
    }
}