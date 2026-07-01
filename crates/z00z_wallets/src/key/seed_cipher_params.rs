/// Versioned Argon2id parameters used for persisted encrypted seed containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Argon2idParams {
    /// Argon2 memory cost in KiB.
    pub mem_kib: u32,
    /// Argon2 iterations/time cost.
    pub time: u32,
    /// Argon2 parallelism/lanes.
    pub lanes: u32,
}

impl ConstantTimeEq for Argon2idParams {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.mem_kib.ct_eq(&other.mem_kib)
            & self.time.ct_eq(&other.time)
            & self.lanes.ct_eq(&other.lanes)
    }
}

impl Argon2idParams {
    // Validation thresholds for persisted seed containers.
    // For new wallets, use MOBILE (128 MiB) or higher to meet OWASP 2023 minimums.
    const MIN_MEM_KIB: u32 = 128 * 1024;
    const MIN_TIME: u32 = 3; // OWASP 2023 minimum: 2
    const MIN_LANES: u32 = 1; // OWASP 2023 minimum: 1
    const MAX_MEM_KIB: u32 = 512 * 1024; // DoS protection
    const MAX_TIME: u32 = 10; // DoS protection
    const MAX_LANES: u32 = 16; // DoS protection

    const ENCODED_LEN: usize = 12;

    /// Default parameters: 128 MiB, 4 iterations, 4 lanes.
    pub const DEFAULT: Self = Self {
        mem_kib: 128 * 1024,
        time: 4,
        lanes: 4,
    };

    /// Mobile parameters following OWASP 2023 minimum recommendations.
    ///
    /// **OWASP 2023 Argon2id minimum:** 128 MiB memory, 2 iterations, 1 thread.
    ///
    /// We use slightly stronger parameters:
    /// - Memory: 128 MiB (OWASP minimum)
    /// - Time: 4 iterations (exceeds OWASP minimum of 2)
    /// - Lanes: 4 (exceeds OWASP minimum of 1)
    ///
    /// **Trade-off:** 128 MiB may be tight on low-end devices (< 2GB RAM).
    /// Monitor for OOM crashes. Consider progressive enhancement based on
    /// available memory if constrained devices need a separate product path.
    ///
    /// **Reference:** <https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id>
    pub const MOBILE: Self = Self {
        mem_kib: 128 * 1024,
        time: 4,
        lanes: 4,
    };

    /// Fast parameters for tests: 128 MiB, 3 iterations, 1 lane.
    #[cfg(feature = "test-params-fast")]
    pub const TEST_FAST: Self = Self {
        mem_kib: 128 * 1024,
        time: 3,
        lanes: 1,
    };

    /// High-security parameters: 256 MiB, 8 iterations, 16 lanes.
    pub const HIGH_SECURITY: Self = Self {
        mem_kib: 256 * 1024,
        time: 8,
        lanes: 16,
    };

    /// Validate parameters are within safe bounds.
    ///
    /// **Validation Strategy:**
    /// - MIN thresholds enforce 128 MiB+ persisted parameters
    /// - MAX thresholds provide DoS protection against resource exhaustion
    /// - For new wallets, prefer MOBILE (128 MiB) or higher for OWASP 2023 compliance
    pub fn validate(&self) -> Result<(), CipherSeedError> {
        if self.mem_kib < Self::MIN_MEM_KIB {
            return Err(CipherSeedError::InvalidKdfParams);
        }
        if self.time < Self::MIN_TIME {
            return Err(CipherSeedError::InvalidKdfParams);
        }
        if self.lanes < Self::MIN_LANES || self.lanes > Self::MAX_LANES {
            return Err(CipherSeedError::InvalidKdfParams);
        }

        // Maximum to prevent DoS - hard caps for persisted params
        if self.mem_kib > Self::MAX_MEM_KIB {
            return Err(CipherSeedError::InvalidKdfParams);
        }
        if self.time > Self::MAX_TIME {
            return Err(CipherSeedError::InvalidKdfParams);
        }
        Ok(())
    }

    /// Adapt parameters based on available memory.
    pub fn adapt_to_hardware() -> Self {
        // Placeholder heuristic.
        Self::DEFAULT
    }

    fn encode_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.mem_kib.to_le_bytes());
        out.extend_from_slice(&self.time.to_le_bytes());
        out.extend_from_slice(&self.lanes.to_le_bytes());
    }

    fn decode_from(bytes: &[u8]) -> Result<(Self, usize), CipherSeedError> {
        if bytes.len() < Self::ENCODED_LEN {
            return Err(CipherSeedError::InvalidFormat);
        }

        let mem_kib = u32::from_le_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| CipherSeedError::InvalidFormat)?,
        );
        let time = u32::from_le_bytes(
            bytes[4..8]
                .try_into()
                .map_err(|_| CipherSeedError::InvalidFormat)?,
        );
        let lanes = u32::from_le_bytes(
            bytes[8..12]
                .try_into()
                .map_err(|_| CipherSeedError::InvalidFormat)?,
        );

        Ok((
            Self {
                mem_kib,
                time,
                lanes,
            },
            Self::ENCODED_LEN,
        ))
    }
}

/// Convert Argon2idParams to Argon2Params for KDF operations.
impl From<Argon2idParams> for Argon2Params {
    fn from(params: Argon2idParams) -> Self {
        Self {
            memory: params.mem_kib,
            iterations: params.time,
            parallelism: params.lanes,
        }
    }
}