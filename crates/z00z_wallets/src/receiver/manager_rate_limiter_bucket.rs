#[derive(Debug)]
struct TokenBucket {
    /// Maximum tokens (burst allowance)
    burst: u32,
    /// Refill rate (tokens per second)
    rate_per_sec: u32,
    state: std::sync::Mutex<TokenBucketState>,
}

#[derive(Debug, Clone, Copy)]
struct TokenBucketState {
    tokens: f64,
    last_refill_ms: u64,
    is_init: bool,
}

impl TokenBucket {
    fn new(rate_per_sec: u32, burst: u32) -> Self {
        Self {
            burst,
            rate_per_sec,
            state: std::sync::Mutex::new(TokenBucketState {
                tokens: burst as f64,
                last_refill_ms: 0,
                is_init: false,
            }),
        }
    }

    fn burst(&self) -> u32 {
        self.burst
    }

    fn try_consume(&self, now_ms: u64, n: u32) -> ReceiverManagerResult<bool> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| ReceiverManagerError::RateLimiterPoisoned)?;
        Self::refill(&mut guard, now_ms, self.rate_per_sec, self.burst);

        if guard.tokens >= n as f64 {
            guard.tokens -= n as f64;
            return Ok(true);
        }

        Ok(false)
    }

    fn status(&self, now_ms: u64) -> ReceiverManagerResult<RateLimiterStatus> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| ReceiverManagerError::RateLimiterPoisoned)?;
        Self::refill(&mut guard, now_ms, self.rate_per_sec, self.burst);

        Ok(RateLimiterStatus {
            available_tokens: guard.tokens.floor().max(0.0) as u64,
            max_requests: self.burst as u64,
        })
    }

    fn refill(state: &mut TokenBucketState, now_ms: u64, rate_per_sec: u32, burst: u32) {
        if !state.is_init {
            state.is_init = true;
            state.last_refill_ms = now_ms;
            return;
        }

        let elapsed_ms = now_ms.saturating_sub(state.last_refill_ms);
        if elapsed_ms == 0 {
            return;
        }

        let refill = (elapsed_ms as f64) * (rate_per_sec as f64) / 1000.0;
        state.tokens = (state.tokens + refill).min(burst as f64);
        state.last_refill_ms = now_ms;
    }
}

/// Snapshot of the current token-bucket rate limiter capacity.
pub struct RateLimiterStatus {
    /// Current number of available tokens.
    pub available_tokens: u64,
    /// Maximum allowed requests per time window.
    pub max_requests: u64,
}
