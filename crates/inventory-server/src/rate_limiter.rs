// Rate limiter module
// TODO: Implementar en Fase 3

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    attempts: RwLock<HashMap<String, (u32, Instant)>>,
    max_attempts: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            attempts: RwLock::new(HashMap::new()),
            max_attempts,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut attempts = self.attempts.write().unwrap();
        let now = Instant::now();

        if let Some((count, first_attempt)) = attempts.get(key) {
            if now.duration_since(*first_attempt) > self.window {
                attempts.remove(key);
                return true;
            }
            if *count >= self.max_attempts {
                return false;
            }
        }

        true
    }

    pub fn record_attempt(&self, key: &str) {
        let mut attempts = self.attempts.write().unwrap();
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
    }
}
