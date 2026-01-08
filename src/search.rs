use crate::lychrel::{lychrel_iteration, IterationResult};
use num_bigint::BigUint;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub struct SearchConfig {
    pub start: BigUint,
    pub end: BigUint,
    pub max_iterations: u32,
    pub parallel: bool,
}

pub struct SearchResults {
    pub total_tested: u64,
    pub potential_lychrel: Vec<IterationResult>,
    pub palindromes_found: Vec<IterationResult>,
}

impl SearchResults {
    pub fn new() -> Self {
        SearchResults {
            total_tested: 0,
            potential_lychrel: Vec::new(),
            palindromes_found: Vec::new(),
        }
    }
}

pub fn search_range(config: SearchConfig) -> SearchResults {
    if config.parallel {
        search_parallel(config)
    } else {
        search_sequential(config)
    }
}

fn search_sequential(config: SearchConfig) -> SearchResults {
    let mut results = SearchResults::new();
    let mut current = config.start.clone();

    while current <= config.end {
        let result = lychrel_iteration(current.clone(), config.max_iterations);
        results.total_tested += 1;

        if result.is_potential_lychrel {
            results.potential_lychrel.push(result);
        } else if result.iterations > 0 {
            results.palindromes_found.push(result);
        }

        current += 1u32;
    }

    results
}

fn search_parallel(config: SearchConfig) -> SearchResults {
    let start_u64 = config.start.to_string().parse::<u64>().unwrap_or(0);
    let end_u64 = config.end.to_string().parse::<u64>().unwrap_or(start_u64);
    
    let potential_lychrel = Arc::new(Mutex::new(Vec::new()));
    let palindromes = Arc::new(Mutex::new(Vec::new()));

    let total_tested = if end_u64 >= start_u64 {
        end_u64 - start_u64 + 1
    } else {
        0
    };

    (start_u64..=end_u64).into_par_iter().for_each(|n| {
        let result = lychrel_iteration(BigUint::from(n), config.max_iterations);

        if result.is_potential_lychrel {
            potential_lychrel.lock().unwrap().push(result);
        } else if result.iterations > 0 {
            palindromes.lock().unwrap().push(result);
        }
    });

    SearchResults {
        total_tested,
        potential_lychrel: Arc::try_unwrap(potential_lychrel)
            .unwrap()
            .into_inner()
            .unwrap(),
        palindromes_found: Arc::try_unwrap(palindromes)
            .unwrap()
            .into_inner()
            .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_small_range() {
        let config = SearchConfig {
            start: BigUint::from(1u32),
            end: BigUint::from(10u32),
            max_iterations: 100,
            parallel: false,
        };

        let results = search_range(config);
        assert_eq!(results.total_tested, 10);
    }

    #[test]
    fn test_search_finds_196() {
        let config = SearchConfig {
            start: BigUint::from(196u32),
            end: BigUint::from(196u32),
            max_iterations: 50,
            parallel: false,
        };

        let results = search_range(config);
        assert_eq!(results.potential_lychrel.len(), 1);
        assert_eq!(results.potential_lychrel[0].start_number, BigUint::from(196u32));
    }
}
