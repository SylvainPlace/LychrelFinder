use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub seed_number: String, // Store as String for serialization
    pub iterations_from_seed: u32,
    pub max_iterations_tested: u32,
    pub final_digits: usize,
    pub reached_palindrome: bool,
    pub palindrome_at_iteration: Option<u32>,
}

#[derive(Debug)]
pub struct ThreadCache {
    known_values: HashMap<BigUint, ThreadInfo>,
    snapshot: Option<Arc<HashMap<BigUint, ThreadInfo>>>,
    max_cache_size: usize,
    hits: u64,
    misses: u64,
}

#[derive(Debug)]
pub struct CacheStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

pub enum DetectionResult {
    NewThread {
        path: Vec<BigUint>,
    },
    KnownThread {
        thread_info: ThreadInfo,
        converged_at_iteration: u32,
    },
}

impl ThreadCache {
    pub fn new(max_size: usize) -> Self {
        ThreadCache {
            known_values: HashMap::new(),
            snapshot: None,
            max_cache_size: max_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Check if a value exists in the cache (local or snapshot)
    pub fn check(&mut self, value: &BigUint) -> Option<ThreadInfo> {
        if let Some(info) = self.known_values.get(value) {
            self.hits += 1;
            Some(info.clone())
        } else if let Some(ref snapshot) = self.snapshot {
            if let Some(info) = snapshot.get(value) {
                self.hits += 1;
                Some(info.clone())
            } else {
                self.misses += 1;
                None
            }
        } else {
            self.misses += 1;
            None
        }
    }

    /// Add a new thread to the cache
    ///
    /// This function adds a sequence of numbers (thread) to the cache along with
    /// information about the thread. Only the first N values of the path are cached
    /// to limit memory usage. If the cache exceeds its maximum size, entries with
    /// the lowest iteration counts are evicted.
    ///
    /// # Arguments
    ///
    /// * `path` - A slice of BigUint representing the sequence of numbers in the thread
    /// * `info` - ThreadInfo containing metadata about the thread
    ///
    /// # Examples
    ///
    /// ```
    /// use lychrel_finder::thread_cache::{ThreadCache, ThreadInfo};
    /// use num_bigint::BigUint;
    ///
    /// let mut cache = ThreadCache::new(1000);
    /// let path = vec![BigUint::from(887u32), BigUint::from(1675u32)];
    /// let info = ThreadInfo {
    ///     seed_number: "196".to_string(),
    ///     iterations_from_seed: 0,
    ///     max_iterations_tested: 100,
    ///     final_digits: 50,
    ///     reached_palindrome: false,
    ///     palindrome_at_iteration: None,
    /// };
    /// cache.add_thread(&path, info);
    /// ```
    pub fn add_thread(&mut self, path: &[BigUint], info: ThreadInfo) {
        // Only cache the first N values of the path (e.g., first 100)
        let cache_limit = 100.min(path.len());

        for (idx, number) in path.iter().take(cache_limit).enumerate() {
            // Create modified info for this specific position in the thread
            let position_info = ThreadInfo {
                seed_number: info.seed_number.clone(),
                iterations_from_seed: info.iterations_from_seed + idx as u32,
                max_iterations_tested: info.max_iterations_tested,
                final_digits: info.final_digits,
                reached_palindrome: info.reached_palindrome,
                palindrome_at_iteration: info.palindrome_at_iteration,
            };

            self.known_values.insert(number.clone(), position_info);
        }

        // Evict if needed
        self.evict_if_needed();
    }

    /// Determine if a thread should be cached based on its properties
    pub fn should_cache(&self, iterations: u32) -> bool {
        // Only cache threads with 50+ iterations (interesting ones)
        iterations >= 50
    }

    /// Evict entries if cache is over capacity
    /// Uses a simple strategy: evict entries with low iterations
    pub fn evict_if_needed(&mut self) {
        if self.known_values.len() > self.max_cache_size {
            // Find entries with lowest max_iterations_tested
            let mut entries: Vec<_> = self.known_values.iter().collect();
            entries.sort_by_key(|(_, info)| info.max_iterations_tested);

            // Remove the bottom 10% to avoid frequent evictions
            let to_remove = (self.max_cache_size / 10).max(1);
            let keys_to_remove: Vec<BigUint> = entries
                .iter()
                .take(to_remove)
                .map(|(key, _)| (*key).clone())
                .collect();

            for key in keys_to_remove {
                self.known_values.remove(&key);
            }
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_requests = self.hits + self.misses;
        let hit_rate = if total_requests > 0 {
            self.hits as f64 / total_requests as f64
        } else {
            0.0
        };

        CacheStats {
            entries: self.known_values.len(),
            hits: self.hits,
            misses: self.misses,
            hit_rate,
        }
    }

    /// Calculate current hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Export important threads (200+ iterations) for sharing
    pub fn export_important(&self) -> Vec<(BigUint, ThreadInfo)> {
        self.known_values
            .iter()
            .filter(|(_, info)| info.max_iterations_tested >= 200)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Save cache to file
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        // Convert keys to string for JSON serialization (JSON keys must be strings)
        let string_map: HashMap<String, ThreadInfo> = self
            .known_values
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        crate::io_utils::save_to_file(&string_map, path)
    }

    /// Load cache from file
    pub fn load_from_file(path: &Path, max_size: usize) -> std::io::Result<Self> {
        let string_map: HashMap<String, ThreadInfo> = crate::io_utils::load_from_file(path)?;

        let known_values: HashMap<BigUint, ThreadInfo> = string_map
            .into_iter()
            .map(|(k, v)| (k.parse::<BigUint>().unwrap_or_default(), v))
            .collect();

        Ok(ThreadCache {
            known_values,
            snapshot: None,
            max_cache_size: max_size,
            hits: 0,
            misses: 0,
        })
    }

    /// Merge another cache into this one
    pub fn merge(&mut self, other: ThreadCache) {
        // Merge stats
        self.hits += other.hits;
        self.misses += other.misses;

        // Merge values
        for (key, info) in other.known_values {
            // Only merge if not exists or if the other has more iterations tested
            if let Some(existing) = self.known_values.get(&key) {
                if info.max_iterations_tested > existing.max_iterations_tested {
                    self.known_values.insert(key, info);
                }
            } else {
                self.known_values.insert(key, info);
            }
        }

        self.evict_if_needed();
    }

    /// Take a snapshot of the current cache
    /// Moves known_values to an Arc and clears local known_values
    pub fn take_snapshot(&mut self) -> Arc<HashMap<BigUint, ThreadInfo>> {
        let values = std::mem::take(&mut self.known_values);
        let arc = Arc::new(values);
        self.snapshot = Some(arc.clone());
        arc
    }

    /// Restore cache from a snapshot/merged values
    pub fn restore_snapshot(&mut self, snapshot: Arc<HashMap<BigUint, ThreadInfo>>) {
        // Try to unwrap to avoid cloning if we are the last owner
        // If not, we have to clone.
        match Arc::try_unwrap(snapshot) {
            Ok(values) => self.known_values = values,
            Err(arc) => self.known_values = (*arc).clone(),
        }
        self.snapshot = None;
    }

    /// Create a new worker cache with a reference to the snapshot
    pub fn new_worker(snapshot: Arc<HashMap<BigUint, ThreadInfo>>, max_size: usize) -> Self {
        ThreadCache {
            known_values: HashMap::new(),
            snapshot: Some(snapshot),
            max_cache_size: max_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Create a new empty cache (helper for reduce)
    pub fn new_empty(max_size: usize) -> Self {
        ThreadCache {
            known_values: HashMap::new(),
            snapshot: None,
            max_cache_size: max_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.known_values.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.known_values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = ThreadCache::new(1000);
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hits, 0);
        assert_eq!(cache.misses, 0);
    }

    #[test]
    fn test_cache_add_and_check() {
        let mut cache = ThreadCache::new(1000);

        let path = vec![
            BigUint::from(887u32),
            BigUint::from(1675u32),
            BigUint::from(7436u32),
        ];

        let info = ThreadInfo {
            seed_number: "196".to_string(),
            iterations_from_seed: 0,
            max_iterations_tested: 100,
            final_digits: 50,
            reached_palindrome: false,
            palindrome_at_iteration: None,
        };

        cache.add_thread(&path, info);

        // Check should find the first value
        let result = cache.check(&BigUint::from(887u32));
        assert!(result.is_some());
        assert_eq!(cache.hits, 1);

        // Check unknown value
        let result = cache.check(&BigUint::from(999u32));
        assert!(result.is_none());
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_should_cache() {
        let cache = ThreadCache::new(1000);
        assert!(!cache.should_cache(10));
        assert!(!cache.should_cache(49));
        assert!(cache.should_cache(50));
        assert!(cache.should_cache(100));
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = ThreadCache::new(10);

        // Add more entries than capacity
        for i in 0..20 {
            let path = vec![BigUint::from(i * 1000u32)];
            let info = ThreadInfo {
                seed_number: i.to_string(),
                iterations_from_seed: 0,
                max_iterations_tested: 50 + i,
                final_digits: 20,
                reached_palindrome: false,
                palindrome_at_iteration: None,
            };
            cache.add_thread(&path, info);
        }

        // Cache should have evicted entries to stay under max_size
        assert!(cache.len() <= 10);
    }

    #[test]
    fn test_hit_rate() {
        let mut cache = ThreadCache::new(1000);

        let path = vec![BigUint::from(887u32)];
        let info = ThreadInfo {
            seed_number: "196".to_string(),
            iterations_from_seed: 0,
            max_iterations_tested: 100,
            final_digits: 50,
            reached_palindrome: false,
            palindrome_at_iteration: None,
        };
        cache.add_thread(&path, info);

        // 2 hits, 3 misses = 40% hit rate
        cache.check(&BigUint::from(887u32));
        cache.check(&BigUint::from(887u32));
        cache.check(&BigUint::from(999u32));
        cache.check(&BigUint::from(998u32));
        cache.check(&BigUint::from(997u32));

        assert_eq!(cache.hits, 2);
        assert_eq!(cache.misses, 3);
        assert!((cache.hit_rate() - 0.4).abs() < 0.01);
    }
}
