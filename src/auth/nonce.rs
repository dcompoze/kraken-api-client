//! Nonce generation for Kraken API authentication.
//!
//! Kraken requires a strictly increasing nonce for each authenticated request
//! to prevent replay attacks.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Trait for providing nonces for authenticated requests.
///
/// The nonce must be strictly increasing for each request.
/// Kraken recommends using a timestamp-based approach.
pub trait NonceProvider: Send + Sync {
    /// Generate the next nonce value.
    ///
    /// This value must be greater than any previously returned value.
    fn next_nonce(&self) -> u64;
}

/// A nonce provider that generates strictly increasing nonces based on time.
///
/// Uses microseconds since UNIX epoch, with an atomic counter to ensure
/// uniqueness even for requests made in the same microsecond.
pub struct IncreasingNonce {
    last_nonce: AtomicU64,
}

impl IncreasingNonce {
    /// Create a new increasing nonce provider.
    pub fn new() -> Self {
        Self {
            last_nonce: AtomicU64::new(0),
        }
    }

    /// Get current time in microseconds since UNIX epoch.
    fn current_time_micros() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64
    }
}

impl Default for IncreasingNonce {
    fn default() -> Self {
        Self::new()
    }
}

impl NonceProvider for IncreasingNonce {
    fn next_nonce(&self) -> u64 {
        let time_nonce = Self::current_time_micros();

        // Ensure the nonce is strictly increasing.
        // Use the max of current time and last + 1.
        loop {
            let last = self.last_nonce.load(Ordering::SeqCst);
            let next = time_nonce.max(last + 1);

            if self
                .last_nonce
                .compare_exchange(last, next, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return next;
            }
            // If CAS failed, another thread updated the value. Retry.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::thread;

    #[test]
    fn test_nonce_strictly_increasing() {
        let provider = IncreasingNonce::new();

        let mut last = 0u64;
        for _ in 0..1000 {
            let nonce = provider.next_nonce();
            assert!(nonce > last, "Nonce must be strictly increasing");
            last = nonce;
        }
    }

    #[test]
    fn test_nonce_unique_across_threads() {
        let provider = std::sync::Arc::new(IncreasingNonce::new());
        let mut handles = vec![];

        for _ in 0..4 {
            let p = provider.clone();
            handles.push(thread::spawn(move || {
                let mut nonces = Vec::new();
                for _ in 0..1000 {
                    nonces.push(p.next_nonce());
                }
                nonces
            }));
        }

        let mut all_nonces = HashSet::new();
        for handle in handles {
            let nonces = handle.join().unwrap();
            for nonce in nonces {
                assert!(
                    all_nonces.insert(nonce),
                    "Nonce must be unique across threads"
                );
            }
        }
    }
}
