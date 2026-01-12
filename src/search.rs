use crate::lychrel::{lychrel_iteration, IterationResult};
use crate::search_checkpoint::{SearchCheckpoint, SearchCheckpointBuilder};
use num_bigint::BigUint;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct SearchConfig {
    pub start: BigUint,
    pub end: BigUint,
    pub max_iterations: u32,
    pub parallel: bool,
    pub checkpoint_interval: Option<u64>,
    pub checkpoint_file: Option<String>,
}

pub struct SearchResults {
    pub total_tested: u64,
    pub potential_lychrel: Vec<IterationResult>,
    pub palindromes_found: Vec<IterationResult>,
}

impl Default for SearchResults {
    fn default() -> Self {
        Self::new()
    }
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

/// Search for Lychrel numbers in a specified range
///
/// This function performs a search for potential Lychrel numbers in the given range,
/// using either parallel or sequential processing based on the configuration.
/// Parallel processing uses Rayon for automatic CPU core utilization,
/// while sequential processing supports checkpointing.
///
/// # Arguments
///
/// * `config` - SearchConfig containing:
///   - `start`: Starting number of the range
///   - `end`: Ending number of the range
///   - `max_iterations`: Maximum iterations to test each number
///   - `parallel`: Whether to use parallel processing
///   - `checkpoint_interval`: Optional checkpoint frequency (sequential only)
///   - `checkpoint_file`: Optional checkpoint file path (sequential only)
///
/// # Returns
///
/// A `SearchResults` struct containing:
/// - Total numbers tested
/// - Vector of potential Lychrel numbers found
/// - Vector of numbers that reached palindromes
///
/// # Examples
///
/// ```
/// use lychrel_finder::{search_range, SearchConfig};
/// use num_bigint::BigUint;
///
/// let config = SearchConfig {
///     start: BigUint::from(1u32),
///     end: BigUint::from(100u32),
///     max_iterations: 1000,
///     parallel: true,
///     checkpoint_interval: None,
///     checkpoint_file: None,
/// };
///
/// let results = search_range(config);
/// println!("Found {} potential Lychrel numbers", results.potential_lychrel.len());
/// ```
pub fn search_range(config: SearchConfig) -> SearchResults {
    if config.parallel {
        search_parallel(config)
    } else {
        search_sequential(config)
    }
}

/// Search for Lychrel numbers in a range with resumable progress
///
/// This function searches for potential Lychrel numbers in a specified range
/// while providing periodic progress updates through a callback function.
/// It supports checkpointing to allow resuming the search from where it left off.
///
/// # Arguments
///
/// * `config` - Search configuration including range, max iterations, and checkpoint settings
/// * `progress_callback` - Callback function that receives progress updates
///
/// # Returns
///
/// A `SearchResults` struct containing the numbers tested and potential Lychrel candidates
///
/// # Examples
///
/// ```
/// use lychrel_finder::{search_range_resumable, SearchConfig};
/// use num_bigint::BigUint;
///
/// let config = SearchConfig {
///     start: BigUint::from(1u32),
///     end: BigUint::from(1000u32),
///     max_iterations: 100,
///     parallel: false,
///     checkpoint_interval: Some(100),
///     checkpoint_file: Some("checkpoint.json".to_string()),
/// };
///
/// let results = search_range_resumable(config, |tested, current, is_checkpoint| {
///     println!("Tested: {}/1000, Current: {}", tested, current);
/// });
/// ```
pub fn search_range_resumable<F>(config: SearchConfig, mut progress_callback: F) -> SearchResults
where
    F: FnMut(u64, &BigUint, bool),
{
    let start_time = Instant::now();
    let mut results = SearchResults::new();
    let mut current = config.start.clone();
    let mut last_checkpoint = 0u64;

    while current <= config.end {
        let result = lychrel_iteration(current.clone(), config.max_iterations);
        results.total_tested += 1;

        if result.is_potential_lychrel {
            results.potential_lychrel.push(result);
        } else if result.iterations > 0 {
            results.palindromes_found.push(result);
        }

        // Save checkpoint periodically
        let should_save_checkpoint = if let Some(interval) = config.checkpoint_interval {
            results.total_tested - last_checkpoint >= interval
        } else {
            false
        };

        if should_save_checkpoint {
            if let Some(ref file) = config.checkpoint_file {
                let checkpoint = SearchCheckpointBuilder::new()
                    .start_range(config.start.clone())
                    .end_range(config.end.clone())
                    .current_number(current.clone())
                    .max_iterations(config.max_iterations)
                    .numbers_tested(results.total_tested)
                    .potential_lychrel(results.potential_lychrel.clone())
                    .checkpoint_interval(config.checkpoint_interval)
                    .checkpoint_file(config.checkpoint_file.clone())
                    .elapsed_secs(start_time.elapsed().as_secs_f64())
                    .build();

                if let Err(e) = checkpoint.save(file) {
                    eprintln!("Warning: Failed to save checkpoint: {}", e);
                } else {
                    progress_callback(results.total_tested, &current, true);
                    last_checkpoint = results.total_tested;
                }
            }
        } else {
            progress_callback(results.total_tested, &current, false);
        }

        current += 1u32;
    }

    results
}

pub fn resume_search_from_checkpoint<F>(
    checkpoint: SearchCheckpoint,
    mut progress_callback: F,
) -> SearchResults
where
    F: FnMut(u64, &BigUint, bool),
{
    let start_time = Instant::now();
    let mut results = SearchResults::new();
    results.total_tested = checkpoint.numbers_tested;

    // Recreate potential_lychrel from saved numbers
    for num in &checkpoint.potential_lychrel_found {
        let result = IterationResult {
            start_number: num.clone(),
            is_palindrome: false,
            iterations: checkpoint.max_iterations,
            final_number: None,
            is_potential_lychrel: true,
        };
        results.potential_lychrel.push(result);
    }

    let mut current = checkpoint.current_number.clone() + 1u32;
    let mut last_checkpoint = checkpoint.numbers_tested;

    while current <= checkpoint.end_range {
        let result = lychrel_iteration(current.clone(), checkpoint.max_iterations);
        results.total_tested += 1;

        if result.is_potential_lychrel {
            results.potential_lychrel.push(result);
        } else if result.iterations > 0 {
            results.palindromes_found.push(result);
        }

        // Save checkpoint periodically
        let should_save_checkpoint = if let Some(interval) = checkpoint.checkpoint_interval {
            results.total_tested - last_checkpoint >= interval
        } else {
            false
        };

        if should_save_checkpoint {
            if let Some(ref file) = checkpoint.checkpoint_file {
                let new_checkpoint = SearchCheckpointBuilder::new()
                    .start_range(checkpoint.start_range.clone())
                    .end_range(checkpoint.end_range.clone())
                    .current_number(current.clone())
                    .max_iterations(checkpoint.max_iterations)
                    .numbers_tested(results.total_tested)
                    .potential_lychrel(results.potential_lychrel.clone())
                    .checkpoint_interval(checkpoint.checkpoint_interval)
                    .checkpoint_file(checkpoint.checkpoint_file.clone())
                    .elapsed_secs(checkpoint.elapsed_secs + start_time.elapsed().as_secs_f64())
                    .build();

                if let Err(e) = new_checkpoint.save(file) {
                    eprintln!("Warning: Failed to save checkpoint: {}", e);
                } else {
                    progress_callback(results.total_tested, &current, true);
                    last_checkpoint = results.total_tested;
                }
            }
        } else {
            progress_callback(results.total_tested, &current, false);
        }

        current += 1u32;
    }

    results
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
        palindromes_found: Arc::try_unwrap(palindromes).unwrap().into_inner().unwrap(),
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
            checkpoint_interval: None,
            checkpoint_file: None,
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
            checkpoint_interval: None,
            checkpoint_file: None,
        };

        let results = search_range(config);
        assert_eq!(results.potential_lychrel.len(), 1);
        assert_eq!(
            results.potential_lychrel[0].start_number,
            BigUint::from(196u32)
        );
    }
}
