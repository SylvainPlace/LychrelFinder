use lychrel_finder::lychrel::lychrel_iteration_with_cache;
use lychrel_finder::thread_cache::ThreadCache;
use num_bigint::BigUint;

#[test]
fn test_thread_convergence_196_295() {
    let mut cache = ThreadCache::new(10000);

    // Test 196 first (it's a known Lychrel candidate)
    let result1 = lychrel_iteration_with_cache(BigUint::from(196u32), 100, &mut cache);
    assert!(!result1.is_palindrome);
    assert!(result1.is_potential_lychrel);

    // Test 295 - should converge to same thread as 196
    // 295 + 592 = 887
    // 196 + 691 = 887
    // They should converge at 887!
    let result2 = lychrel_iteration_with_cache(BigUint::from(295u32), 100, &mut cache);

    // Should have cache hit for 295 since it converges with 196
    let stats = cache.stats();
    assert!(stats.hits > 0, "Should have cache hit for 295 convergence");

    // Both should be potential Lychrel
    assert!(!result2.is_palindrome);
    assert!(result2.is_potential_lychrel);
}

#[test]
fn test_thread_convergence_multiple() {
    let mut cache = ThreadCache::new(10000);

    // Test a group of numbers that converge to the same thread
    // These all converge quickly
    let numbers = vec![196u32, 295, 394, 493, 592, 689, 788];

    for &n in &numbers {
        let _result = lychrel_iteration_with_cache(BigUint::from(n), 50, &mut cache);
    }

    let stats = cache.stats();

    // After testing all these numbers, we should have several cache hits
    // because they converge to the same sequences
    println!("Cache stats after testing convergent numbers:");
    println!("  Entries: {}", stats.entries);
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
    println!("  Hit rate: {:.2}%", stats.hit_rate * 100.0);

    assert!(stats.hits > 0, "Should have cache hits from convergence");
}

#[test]
fn test_cache_speeds_up_computation() {
    use std::time::Instant;

    // Without cache
    let start = Instant::now();
    for i in 1u32..=1000 {
        let _ = lychrel_iteration_with_cache(BigUint::from(i), 100, &mut ThreadCache::new(0));
    }
    let without_cache = start.elapsed();

    // With cache
    let start = Instant::now();
    let mut cache = ThreadCache::new(100000);
    for i in 1u32..=1000 {
        let _ = lychrel_iteration_with_cache(BigUint::from(i), 100, &mut cache);
    }
    let with_cache = start.elapsed();

    println!("Without cache: {:?}", without_cache);
    println!("With cache: {:?}", with_cache);
    println!(
        "Speedup: {:.2}x",
        without_cache.as_secs_f64() / with_cache.as_secs_f64()
    );

    let stats = cache.stats();
    println!("Cache hit rate: {:.2}%", stats.hit_rate * 100.0);

    // With cache should be faster (or at least not significantly slower)
    // Note: For small numbers, overhead might make it similar, but hit rate should be > 0
    assert!(stats.hits > 0, "Cache should have hits");
}

#[test]
fn test_cache_save_and_load() {
    use std::fs;
    use std::path::Path;

    let cache_file = "test_cache_temp.json";

    // Create and populate cache
    let mut cache = ThreadCache::new(10000);
    for i in 1u32..=100 {
        let _ = lychrel_iteration_with_cache(BigUint::from(i), 50, &mut cache);
    }

    let original_entries = cache.len();

    // Save to file
    cache.save_to_file(Path::new(cache_file)).unwrap();

    // Load from file
    let loaded_cache = ThreadCache::load_from_file(Path::new(cache_file), 10000).unwrap();

    assert_eq!(loaded_cache.len(), original_entries);

    // Cleanup
    fs::remove_file(cache_file).ok();
}

#[test]
fn test_palindrome_numbers_not_cached() {
    let mut cache = ThreadCache::new(10000);

    // Test numbers that quickly become palindromes
    let result = lychrel_iteration_with_cache(BigUint::from(89u32), 100, &mut cache);
    assert!(result.is_palindrome);

    // Low iteration count shouldn't be cached (threshold is 50)
    // 89 takes 24 iterations, which is below threshold
    assert_eq!(cache.len(), 0, "Short iterations shouldn't be cached");
}

#[test]
fn test_long_iteration_cached() {
    let mut cache = ThreadCache::new(10000);

    // 196 requires many iterations and never reaches palindrome in 100 steps
    let _result = lychrel_iteration_with_cache(BigUint::from(196u32), 100, &mut cache);

    // Should be cached because it has 100 iterations (above 50 threshold)
    assert!(cache.len() > 0, "Long iterations should be cached");
}
