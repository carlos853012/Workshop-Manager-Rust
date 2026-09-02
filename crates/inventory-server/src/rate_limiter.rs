// Rate limiter module
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub struct RateLimiter {
    attempts: RwLock<HashMap<String, (u32, Instant)>>,
    max_attempts: u32,
    window: Duration,
}

#[allow(dead_code)]
impl RateLimiter {
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            attempts: RwLock::new(HashMap::new()),
            max_attempts,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Verifica si una clave puede realizar un intento.
    /// Retorna `true` si está dentro del límite, `false` si excedió.
    pub fn check(&self, key: &str) -> anyhow::Result<bool> {
        let mut attempts = self.attempts.write().map_err(|e| {
            anyhow::anyhow!("Rate limiter lock poisoned: {}", e)
        })?;
        let now = Instant::now();

        if let Some((count, first_attempt)) = attempts.get(key) {
            if now.duration_since(*first_attempt) > self.window {
                attempts.remove(key);
                return Ok(true);
            }
            if *count >= self.max_attempts {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Registra un intento para una clave.
    pub fn record_attempt(&self, key: &str) -> anyhow::Result<()> {
        let mut attempts = self.attempts.write().map_err(|e| {
            anyhow::anyhow!("Rate limiter lock poisoned: {}", e)
        })?;
        let now = Instant::now();

        let new_entry = match attempts.get(key) {
            Some((count, first_attempt)) => {
                if now.duration_since(*first_attempt) > self.window {
                    Some((1, now))
                } else {
                    Some((*count + 1, *first_attempt))
                }
            }
            None => Some((1, now)),
        };

        if let Some(entry) = new_entry {
            attempts.insert(key.to_string(), entry);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_under_limit() -> anyhow::Result<()> {
        let limiter = RateLimiter::new(3, 60);
        assert!(limiter.check("user1")?);
        limiter.record_attempt("user1")?;
        assert!(limiter.check("user1")?);
        Ok(())
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() -> anyhow::Result<()> {
        let limiter = RateLimiter::new(2, 60);
        limiter.record_attempt("user2")?;
        limiter.record_attempt("user2")?;
        assert!(!limiter.check("user2")?);
        Ok(())
    }

    #[test]
    fn test_rate_limiter_resets_after_window() -> anyhow::Result<()> {
        let limiter = RateLimiter::new(1, 0);
        limiter.record_attempt("user3")?;
        // Con ventana de 0 segundos, el intento anterior ya expiró
        assert!(limiter.check("user3")?);
        Ok(())
    }
}
