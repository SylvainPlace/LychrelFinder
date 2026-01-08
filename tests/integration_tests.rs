use lychrel_finder::{lychrel_iteration, search_range, SearchConfig};
use num_bigint::BigUint;

#[test]
fn test_known_lychrel_candidate_196() {
    let result = lychrel_iteration(BigUint::from(196u32), 1000);
    assert!(!result.is_palindrome);
    assert!(result.is_potential_lychrel);
    assert_eq!(result.start_number, BigUint::from(196u32));
}

#[test]
fn test_number_89_reaches_palindrome() {
    let result = lychrel_iteration(BigUint::from(89u32), 1000);
    assert!(result.is_palindrome);
    assert!(!result.is_potential_lychrel);
    assert_eq!(result.iterations, 24);
}

#[test]
fn test_number_10_reaches_palindrome() {
    let result = lychrel_iteration(BigUint::from(10u32), 1000);
    assert!(result.is_palindrome);
    assert!(!result.is_potential_lychrel);
    assert_eq!(result.iterations, 1);
    assert_eq!(result.final_number, Some(BigUint::from(11u32)));
}

#[test]
fn test_search_range_finds_multiple_lychrel() {
    let config = SearchConfig {
        start: BigUint::from(190u32),
        end: BigUint::from(200u32),
        max_iterations: 100,
        parallel: false,
        checkpoint_interval: None,
        checkpoint_file: None,
    };

    let results = search_range(config);
    
    assert_eq!(results.total_tested, 11);
    assert!(results.potential_lychrel.len() > 0);
    
    let has_196 = results
        .potential_lychrel
        .iter()
        .any(|r| r.start_number == BigUint::from(196u32));
    assert!(has_196);
}

#[test]
fn test_parallel_vs_sequential() {
    let config_seq = SearchConfig {
        start: BigUint::from(1u32),
        end: BigUint::from(100u32),
        max_iterations: 100,
        parallel: false,
        checkpoint_interval: None,
        checkpoint_file: None,
    };

    let config_par = SearchConfig {
        start: BigUint::from(1u32),
        end: BigUint::from(100u32),
        max_iterations: 100,
        parallel: true,
        checkpoint_interval: None,
        checkpoint_file: None,
    };

    let results_seq = search_range(config_seq);
    let results_par = search_range(config_par);

    assert_eq!(results_seq.total_tested, results_par.total_tested);
    assert_eq!(
        results_seq.potential_lychrel.len(),
        results_par.potential_lychrel.len()
    );
}

#[test]
fn test_large_number() {
    let large = BigUint::parse_bytes(b"12345678901234567890", 10).unwrap();
    let result = lychrel_iteration(large.clone(), 10);
    
    assert_eq!(result.start_number, large);
    assert!(result.iterations <= 10);
}
