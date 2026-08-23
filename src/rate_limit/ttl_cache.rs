//! Time-to-live cache used for tracking order creation times to calculate cancellation penalties.

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// A cache that automatically expires entries after a configurable TTL.
#[derive(Debug)]
pub struct TtlCache<K, V> {
    cache: HashMap<K, (V, Instant)>,
    ttl: Duration,
}

impl<K, V> TtlCache<K, V>
where
    K: Hash + Eq,
{
    /// Create a new TTL cache with the specified time-to-live duration.
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            ttl,
        }
    }

    /// Create a new TTL cache with a specific initial capacity.
    pub fn with_capacity(ttl: Duration, capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            ttl,
        }
    }

    /// Insert a key-value pair, timestamped with the current time.
    pub fn insert(&mut self, key: K, value: V) {
        self.cache.insert(key, (value, Instant::now()));
    }

    /// Get a reference to a value if it exists and hasn't expired.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.cache.get(key).and_then(|(value, timestamp)| {
            if timestamp.elapsed() < self.ttl {
                Some(value)
            } else {
                None
            }
        })
    }

    /// Get a mutable reference to a value if it exists and hasn't expired.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let ttl = self.ttl;
        self.cache.get_mut(key).and_then(|(value, timestamp)| {
            if timestamp.elapsed() < ttl {
                Some(value)
            } else {
                None
            }
        })
    }

    /// Get the timestamp when the entry was inserted, or `None` if missing or expired.
    pub fn get_timestamp(&self, key: &K) -> Option<Instant> {
        self.cache.get(key).and_then(|(_, timestamp)| {
            if timestamp.elapsed() < self.ttl {
                Some(*timestamp)
            } else {
                None
            }
        })
    }

    /// Get the age of an entry, or `None` if missing or expired.
    pub fn get_age(&self, key: &K) -> Option<Duration> {
        self.cache.get(key).and_then(|(_, timestamp)| {
            let age = timestamp.elapsed();
            if age < self.ttl {
                Some(age)
            } else {
                None
            }
        })
    }

    /// Remove an entry, returning the value if it existed and had not expired.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.cache.remove(key).and_then(|(value, timestamp)| {
            if timestamp.elapsed() < self.ttl {
                Some(value)
            } else {
                None
            }
        })
    }

    /// Remove an entry and return both the value and its age.
    pub fn remove_with_age(&mut self, key: &K) -> Option<(V, Duration)> {
        self.cache.remove(key).and_then(|(value, timestamp)| {
            let age = timestamp.elapsed();
            if age < self.ttl {
                Some((value, age))
            } else {
                None
            }
        })
    }

    /// Check if a key exists and hasn't expired.
    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Remove all expired entries from the cache.
    pub fn cleanup(&mut self) {
        let ttl = self.ttl;
        self.cache.retain(|_, (_, timestamp)| timestamp.elapsed() < ttl);
    }

    /// Get the number of entries in the cache (including expired ones).
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get the number of non-expired entries.
    pub fn active_count(&self) -> usize {
        let ttl = self.ttl;
        self.cache
            .values()
            .filter(|(_, timestamp)| timestamp.elapsed() < ttl)
            .count()
    }

    /// Clear all entries from the cache.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get the TTL duration for this cache.
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Set a new TTL duration, affecting all future checks without modifying existing timestamps.
    pub fn set_ttl(&mut self, ttl: Duration) {
        self.ttl = ttl;
    }
}

impl<K, V> Default for TtlCache<K, V>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        // 5 minute TTL matching Kraken's order penalty window.
        Self::new(Duration::from_secs(300))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_insert_and_get() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), 100);
        assert_eq!(cache.get(&"key1".to_string()), Some(&100));
        assert_eq!(cache.get(&"key2".to_string()), None);
    }

    #[test]
    fn test_remove() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), 100);
        assert_eq!(cache.remove(&"key1".to_string()), Some(100));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_expiration() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_millis(50));

        cache.insert("key1".to_string(), 100);
        assert!(cache.get(&"key1".to_string()).is_some());

        thread::sleep(Duration::from_millis(60));
        assert!(cache.get(&"key1".to_string()).is_none());
    }

    #[test]
    fn test_cleanup() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_millis(50));

        cache.insert("key1".to_string(), 100);
        cache.insert("key2".to_string(), 200);
        assert_eq!(cache.len(), 2);

        thread::sleep(Duration::from_millis(60));

        assert_eq!(cache.len(), 2);

        cache.cleanup();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_get_age() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), 100);
        thread::sleep(Duration::from_millis(50));

        let age = cache.get_age(&"key1".to_string()).unwrap();
        assert!(age >= Duration::from_millis(50));
        assert!(age < Duration::from_millis(100));
    }

    #[test]
    fn test_remove_with_age() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), 100);
        thread::sleep(Duration::from_millis(50));

        let (value, age) = cache.remove_with_age(&"key1".to_string()).unwrap();
        assert_eq!(value, 100);
        assert!(age >= Duration::from_millis(50));
    }

    #[test]
    fn test_contains() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), 100);
        assert!(cache.contains(&"key1".to_string()));
        assert!(!cache.contains(&"key2".to_string()));
    }

    #[test]
    fn test_active_count() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_millis(50));

        cache.insert("key1".to_string(), 100);
        cache.insert("key2".to_string(), 200);
        assert_eq!(cache.active_count(), 2);

        thread::sleep(Duration::from_millis(60));
        assert_eq!(cache.active_count(), 0);
    }
}
