use super::KdfError;

#[cfg(not(target_arch = "wasm32"))]
pub const MAX_MEM_LIMIT_KIB: u32 = 256 * 1024;
#[cfg(target_arch = "wasm32")]
pub const MAX_MEM_LIMIT_KIB: u32 = 64 * 1024;
pub const MAX_OPS_LIMIT: u32 = 5;
pub const MAX_PARALLELISM: u32 = 8;
pub const MAX_KDF_TIME_MS: u64 = 10_000;
pub const MAX_ARGON2_TOTAL_COST: u64 = (256 * 1024 * 5 * 8 * 9) / 10;

#[derive(Debug, Clone, Copy)]
pub struct Argon2Params {
    pub memory: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Argon2Params {
    pub fn interactive() -> Self {
        Self {
            memory: 64 * 1024,
            iterations: 2,
            parallelism: 4,
        }
    }

    pub fn moderate() -> Self {
        Self {
            memory: 128 * 1024,
            iterations: 3,
            parallelism: 6,
        }
    }

    pub fn strong() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                memory: 256 * 1024,
                iterations: 5,
                parallelism: 7,
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self {
                memory: 64 * 1024,
                iterations: 3,
                parallelism: 4,
            }
        }
    }

    #[cfg(test)]
    pub fn debug() -> Self {
        Self {
            memory: 16,
            iterations: 1,
            parallelism: 1,
        }
    }

    #[cfg(any(test, feature = "test-params-fast"))]
    pub fn test_fast() -> Self {
        Self {
            memory: 16,
            iterations: 1,
            parallelism: 1,
        }
    }

    pub fn estimate_time_seconds(&self) -> f64 {
        let mem_factor = (self.memory / (16 * 1024)) as f64;
        let ops_factor = self.iterations as f64;
        let par_div = (self.parallelism as f64).sqrt();
        (mem_factor * ops_factor * 0.3 / par_div).max(0.1)
    }

    pub fn validate_untrusted(&self) -> Result<(), KdfError> {
        self.validate_non_zero()?;
        self.validate_bounds()?;
        self.validate_total_cost()?;
        self.validate_time_budget()?;
        Ok(())
    }

    fn validate_non_zero(&self) -> Result<(), KdfError> {
        if self.memory == 0 || self.iterations == 0 || self.parallelism == 0 {
            return Err(KdfError::Argon2Params);
        }
        Ok(())
    }

    fn validate_bounds(&self) -> Result<(), KdfError> {
        let min_mem_kib = 8u32.saturating_mul(self.parallelism);
        if self.memory < min_mem_kib {
            return Err(KdfError::Argon2Params);
        }
        if self.memory > MAX_MEM_LIMIT_KIB {
            return Err(KdfError::Argon2Params);
        }
        if self.iterations > MAX_OPS_LIMIT {
            return Err(KdfError::Argon2Params);
        }
        if self.parallelism > MAX_PARALLELISM {
            return Err(KdfError::Argon2Params);
        }
        Ok(())
    }

    fn validate_total_cost(&self) -> Result<(), KdfError> {
        let total_cost = (self.memory as u64)
            .checked_mul(self.iterations as u64)
            .and_then(|v| v.checked_mul(self.parallelism as u64))
            .ok_or(KdfError::Argon2Params)?;
        if total_cost > MAX_ARGON2_TOTAL_COST {
            return Err(KdfError::Argon2Params);
        }
        Ok(())
    }

    fn validate_time_budget(&self) -> Result<(), KdfError> {
        if self.estimate_time_seconds() > (MAX_KDF_TIME_MS as f64 / 1000.0) {
            return Err(KdfError::Argon2Params);
        }
        Ok(())
    }
}
