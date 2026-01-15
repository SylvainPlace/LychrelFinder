use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::lychrel::{lychrel_iteration, lychrel_iteration_with_cache};
use crate::seed_generator::{GeneratorMode, SeedGenerator};
use crate::thread_cache::ThreadCache;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntConfig {
    pub min_digits: usize,
    #[serde(default)]
    pub max_digits: Option<usize>,
    pub target_iterations: u32,
    pub max_iterations: u32, // Max iterations before considering it a Lychrel
    pub target_final_digits: usize,
    pub cache_size: usize,
    #[serde(default = "default_generator_mode")]
    pub generator_mode: GeneratorMode,
    pub checkpoint_interval: u64,
    pub checkpoint_file: String,
    #[serde(default)]
    pub warmup: bool,
}

fn default_generator_mode() -> GeneratorMode {
    GeneratorMode::Sequential
}

pub struct RecordHunter {
    pub min_digits: usize,
    pub max_digits: Option<usize>,
    pub current_digits: usize,
    pub current_range_tested: u64, // Count for current digit range only
    pub target_iterations: u32,
    pub max_iterations: u32,
    pub target_final_digits: usize,
    pub thread_cache: ThreadCache,
    pub seed_generator: SeedGenerator,
    pub generator_mode: GeneratorMode,
    pub stats: HuntStatistics,
    pub checkpoint_interval: u64,
    pub checkpoint_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntStatistics {
    pub numbers_tested: u64,
    pub seeds_tested: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub best_iterations_found: u32,
    pub best_digits_found: usize,
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
    pub candidates_above_200: Vec<RecordCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordCandidate {
    pub number: String, // Store as String for serialization
    pub iterations: u32,
    pub final_digits: usize,
    pub found_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HuntResults {
    pub numbers_tested: u64,
    pub seeds_tested: u64,
    pub records: Vec<RecordCandidate>,
    pub candidates_above_200: Vec<RecordCandidate>,
    pub best_iterations_found: u32,
    pub elapsed_time: Duration,
}

impl Default for HuntConfig {
    fn default() -> Self {
        HuntConfig {
            min_digits: 23,
            max_digits: None,
            target_iterations: 289,
            max_iterations: 300,
            target_final_digits: 142,
            cache_size: 1_000_000,
            generator_mode: GeneratorMode::Sequential,
            checkpoint_interval: 1_000_000,
            checkpoint_file: "hunt_checkpoint.json".to_string(),
            warmup: false,
        }
    }
}

impl HuntConfig {
    /// Load configuration from a JSON file
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        crate::io_utils::load_from_file(path)
    }

    /// Save configuration to a JSON file
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        crate::io_utils::save_to_file(self, path)
    }
}

impl RecordHunter {
    pub fn new(config: HuntConfig) -> Self {
        RecordHunter {
            min_digits: config.min_digits,
            max_digits: config.max_digits,
            current_digits: config.min_digits,
            current_range_tested: 0,
            target_iterations: config.target_iterations,
            max_iterations: config.max_iterations,
            target_final_digits: config.target_final_digits,
            thread_cache: ThreadCache::new(config.cache_size),
            seed_generator: SeedGenerator::new(config.min_digits, config.generator_mode.clone()),
            generator_mode: config.generator_mode,
            stats: HuntStatistics {
                numbers_tested: 0,
                seeds_tested: 0,
                cache_hits: 0,
                cache_misses: 0,
                best_iterations_found: 0,
                best_digits_found: 0,
                start_time: Instant::now(),
                candidates_above_200: Vec::new(),
            },
            checkpoint_interval: config.checkpoint_interval,
            checkpoint_file: config.checkpoint_file,
        }
    }

    /// Calculate total number of numbers to test from min_digits to max_digits
    /// This is an estimate since we filter seeds (approximately 50% of numbers)
    fn calculate_total_numbers(&self) -> u64 {
        let max_d = self.max_digits.unwrap_or(self.min_digits);
        let mut total = 0u64;

        for d in self.min_digits..=max_d {
            // For d digits: 10^d - 10^(d-1) numbers
            // But we only test seeds (approx 50%), so divide by 2
            let count_for_d = if d <= 18 {
                // For smaller digits, calculate exactly
                let max_val = 10u64.pow(d as u32);
                let min_val = 10u64.pow((d - 1) as u32);
                (max_val - min_val) / 2
            } else {
                // For larger digits, estimate (to avoid overflow)
                u64::MAX / 2
            };
            total = total.saturating_add(count_for_d);
        }

        total
    }

    /// Calculate how many numbers have been processed so far
    fn calculate_processed_numbers(&self) -> u64 {
        let mut processed = 0u64;

        // Add all numbers from completed digit ranges
        for d in self.min_digits..self.current_digits {
            let count_for_d = if d <= 18 {
                let max_val = 10u64.pow(d as u32);
                let min_val = 10u64.pow((d - 1) as u32);
                (max_val - min_val) / 2
            } else {
                u64::MAX / 2
            };
            processed = processed.saturating_add(count_for_d);
        }

        // Add numbers tested in current digit range
        processed.saturating_add(self.current_range_tested)
    }

    /// Calculate overall progress percentage
    fn calculate_progress_percentage(&self) -> f64 {
        if self.max_digits.is_none() {
            return 0.0; // Can't calculate progress without max_digits
        }

        let total = self.calculate_total_numbers();
        if total == 0 {
            return 0.0;
        }

        let processed = self.calculate_processed_numbers();
        (processed as f64 / total as f64) * 100.0
    }

    /// Warmup cache with known range (1 to 1 million)
    pub fn warmup_cache(&mut self) {
        println!("🔥 Warming up thread cache with known range (1-1,000,000)...");
        let warmup_start = Instant::now();

        for n in 1u32..=1_000_000 {
            lychrel_iteration_with_cache(BigUint::from(n), 1000, &mut self.thread_cache, None);

            if n % 100_000 == 0 {
                println!("  Warmup progress: {}/1,000,000", n);
            }
        }

        let cache_stats = self.thread_cache.stats();
        println!(
            "✓ Cache warmed up in {:.2}s",
            warmup_start.elapsed().as_secs_f64()
        );
        println!("  Cache entries: {}", cache_stats.entries);
        println!(
            "  Hit rate during warmup: {:.1}%",
            cache_stats.hit_rate * 100.0
        );
        println!();
    }

    /// Main hunting loop
    ///
    /// This function performs the main hunting loop for finding record-breaking
    /// Lychrel numbers. It generates candidate numbers using the configured generator,
    /// tests them for Lychrel properties, and keeps track of statistics and records.
    /// The function also periodically saves checkpoints and displays progress.
    ///
    /// # Returns
    ///
    /// A `HuntResults` struct containing statistics and results of the hunt
    ///
    /// # Examples
    ///
    /// ```
    /// use lychrel_finder::record_hunt::{RecordHunter, HuntConfig};
    /// use lychrel_finder::seed_generator::GeneratorMode;
    ///
    /// let config = HuntConfig::default();
    /// let mut hunter = RecordHunter::new(config);
    /// let results = hunter.hunt();
    /// println!("Found {} records", results.records.len());
    /// ```
    /// Main hunting loop
    pub fn hunt(&mut self) -> HuntResults {
        use rayon::prelude::*;

        println!("🎯 Starting record hunt (Parallel Mode)...\n");

        loop {
            // 1. Parallel Generate a raw batch of candidates
            // Generate consecutive numbers efficiently
            let batch_size = 500_000;

            // Advance the generator's state sequentially (very fast)
            let raw_batch = self.seed_generator.next_raw_batch(batch_size);

            if raw_batch.is_empty() {
                // ... logic to move to next digit size ...
                if let Some(max_digits) = self.max_digits {
                    if self.current_digits < max_digits {
                        self.current_range_tested = 0;
                        self.current_digits += 1;
                        let progress = self.calculate_progress_percentage();
                        println!(
                            "\n📊 Moving to {}-digit numbers... (Overall progress: {:.2}%)\n",
                            self.current_digits, progress
                        );
                        self.seed_generator =
                            SeedGenerator::new(self.current_digits, self.generator_mode.clone());
                        continue;
                    }
                }
                break;
            }

            // 2. Prepare for parallel processing
            let snapshot = self.thread_cache.take_snapshot();
            let p10_max = self.seed_generator.current_p10_max();
            let worker_cache_size = 10_000;
            let config = HuntConfig {
                min_digits: self.min_digits,
                max_digits: self.max_digits,
                target_iterations: self.target_iterations,
                max_iterations: self.max_iterations,
                target_final_digits: self.target_final_digits,
                cache_size: self.thread_cache.len(),
                generator_mode: self.generator_mode.clone(),
                checkpoint_interval: self.checkpoint_interval,
                checkpoint_file: self.checkpoint_file.clone(),
                warmup: false,
            };

            // 3. Process batch in parallel
            let (results, merged_cache, seeds_tested, max_i, max_d) = raw_batch
                .par_iter()
                .fold(
                    || {
                        (
                            Vec::new(),
                            ThreadCache::new_worker(snapshot.clone(), worker_cache_size),
                            0u64,
                            0u32,
                            0usize,
                        )
                    },
                    |mut acc, candidate| {
                        if !crate::seed_generator::is_potential_seed(candidate, Some(&p10_max)) {
                            return acc;
                        }

                        acc.2 += 1;
                        if let Some(r) = process_candidate(candidate, &mut acc.1, &config) {
                            if r.iterations > acc.3 {
                                acc.3 = r.iterations;
                            }
                            if r.final_digits > acc.4 {
                                acc.4 = r.final_digits;
                            }

                            if r.is_record || r.is_promising {
                                acc.0.push(r);
                            }
                        }
                        acc
                    },
                )
                .reduce(
                    || {
                        (
                            Vec::new(),
                            ThreadCache::new_empty(worker_cache_size),
                            0u64,
                            0u32,
                            0usize,
                        )
                    },
                    |mut a, b| {
                        a.0.extend(b.0);
                        a.1.merge(b.1);
                        a.2 += b.2;
                        a.3 = a.3.max(b.3);
                        a.4 = a.4.max(b.4);
                        a
                    },
                );

            // 4. Update state with results
            drop(snapshot); // CRITICAL: Release the Arc reference so merge is fast
            self.thread_cache.merge(merged_cache);

            // Update stats
            let actual_batch_size = raw_batch.len() as u64;
            self.stats.numbers_tested += actual_batch_size;
            self.stats.seeds_tested += seeds_tested;
            self.current_range_tested += actual_batch_size;

            if max_i > self.stats.best_iterations_found {
                self.stats.best_iterations_found = max_i;
            }
            if max_d > self.stats.best_digits_found {
                self.stats.best_digits_found = max_d;
            }

            for res in results {
                if res.is_record {
                    self.handle_record_found(RecordCandidate {
                        number: res.number.clone(),
                        iterations: res.iterations,
                        final_digits: res.final_digits,
                        found_at: chrono::Local::now().to_string(),
                    });
                }

                if res.is_promising {
                    self.stats.candidates_above_200.push(RecordCandidate {
                        number: res.number,
                        iterations: res.iterations,
                        final_digits: res.final_digits,
                        found_at: chrono::Local::now().to_string(),
                    });
                }
            }

            // Sync cache stats
            let cache_stats = self.thread_cache.stats();
            self.stats.cache_hits = cache_stats.hits;
            self.stats.cache_misses = cache_stats.misses;

            // 5. Periodic actions
            if self
                .stats
                .numbers_tested
                .is_multiple_of(self.checkpoint_interval)
            {
                self.save_checkpoint();
            }
            if self.stats.numbers_tested.is_multiple_of(100_000) {
                self.print_stats();
            }
        }

        self.finalize()
    }

    fn handle_record_found(&mut self, record: RecordCandidate) {
        println!("\n🎉 ═══════════════════════════════════════════");
        println!("   RECORD PALINDROME FOUND!");
        println!("═══════════════════════════════════════════");
        println!("Number:      {}", record.number);
        println!("Iterations:  {} (reached palindrome)", record.iterations);
        println!("Final digits: {}", record.final_digits);
        println!("Found at:    {}", record.found_at);
        println!("═══════════════════════════════════════════\n");

        // Save to dedicated file
        let record_file = format!("record_{}_iter.json", record.iterations);
        if let Ok(mut file) = File::create(&record_file) {
            if let Ok(json) = serde_json::to_string_pretty(&record) {
                let _ = file.write_all(json.as_bytes());
                println!("💾 Record saved to {}\n", record_file);
            }
        }
    }

    fn print_stats(&self) {
        let elapsed = self.stats.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        let rate = if elapsed_secs > 0.0 {
            self.stats.numbers_tested as f64 / elapsed_secs
        } else {
            0.0
        };

        let cache_hit_rate = self.thread_cache.hit_rate() * 100.0;
        let skip_rate = if self.stats.numbers_tested > 0 {
            (self.stats.numbers_tested - self.stats.seeds_tested) as f64
                / self.stats.numbers_tested as f64
        } else {
            0.0
        };

        // Calculate progress percentage if max_digits is set
        if let Some(_max_d) = self.max_digits {
            let progress = self.calculate_progress_percentage();
            println!(
                "[Hunt] Progress: {:.2}% | Digits: {}/{} | Range: {} | Total: {} | Seeds: {} | Cache: {:.1}% | Rate: {:.0}/s | Best: {} iter",
                progress,
                self.current_digits,
                _max_d,
                self.current_range_tested,
                self.stats.numbers_tested,
                self.stats.seeds_tested,
                cache_hit_rate,
                rate,
                self.stats.best_iterations_found
            );
        } else {
            println!(
                "[Hunt] Tested: {} | Seeds: {} | Cache: {:.1}% hit | Rate: {:.0}/s | Best: {} iter ({} digits) | Skip: {:.1}%",
                self.stats.numbers_tested,
                self.stats.seeds_tested,
                cache_hit_rate,
                rate,
                self.stats.best_iterations_found,
                self.stats.best_digits_found,
                skip_rate * 100.0
            );
        }
    }

    pub fn save_checkpoint(&self) {
        use crate::record_checkpoint::{CheckpointConfig, RecordHuntCheckpoint};

        let checkpoint = RecordHuntCheckpoint::new(
            &self.seed_generator.current_position(),
            self.current_digits,
            self.seed_generator.mode.clone(),
            &self.stats,
            &format!("{}_cache.json", self.checkpoint_file),
            CheckpointConfig {
                min_digits: self.min_digits,
                max_digits: self.max_digits,
                target_iterations: self.target_iterations,
                max_iterations: self.max_iterations,
                target_final_digits: self.target_final_digits,
                cache_size: self.thread_cache.len(),
                checkpoint_interval: self.checkpoint_interval,
            },
        );

        // Save checkpoint
        if let Err(e) = checkpoint.save(std::path::Path::new(&self.checkpoint_file)) {
            eprintln!("  ✗ Failed to save checkpoint: {}", e);
        } else {
            // Save thread cache separately
            let cache_file = format!("{}_cache.json", self.checkpoint_file);
            if let Err(e) = self
                .thread_cache
                .save_to_file(std::path::Path::new(&cache_file))
            {
                eprintln!("  ✗ Failed to save cache: {}", e);
            } else {
                println!(
                    "  ✓ Checkpoint saved at {} numbers tested",
                    self.stats.numbers_tested
                );
            }
        }
    }

    fn finalize(&self) -> HuntResults {
        let elapsed = self.stats.start_time.elapsed();

        println!("\n🏁 ═══════════════════════════════════════════");
        println!("   HUNT COMPLETE");
        println!("═══════════════════════════════════════════");
        println!("Numbers tested:      {}", self.stats.numbers_tested);
        println!("Seeds tested:        {}", self.stats.seeds_tested);
        println!("Best iterations:     {}", self.stats.best_iterations_found);
        println!("Best final digits:   {}", self.stats.best_digits_found);
        println!(
            "Candidates (200+):   {}",
            self.stats.candidates_above_200.len()
        );
        println!("Time elapsed:        {:.2}s", elapsed.as_secs_f64());
        println!("═══════════════════════════════════════════\n");

        // Find records (targets met) - all candidates are palindromes, not Lychrels
        let records: Vec<RecordCandidate> = self
            .stats
            .candidates_above_200
            .iter()
            .filter(|c| {
                c.iterations >= self.target_iterations && c.final_digits >= self.target_final_digits
            })
            .cloned()
            .collect();

        HuntResults {
            numbers_tested: self.stats.numbers_tested,
            seeds_tested: self.stats.seeds_tested,
            records,
            candidates_above_200: self.stats.candidates_above_200.clone(),
            best_iterations_found: self.stats.best_iterations_found,
            elapsed_time: elapsed,
        }
    }
}

struct ProcessResult {
    number: String,
    iterations: u32,
    final_digits: usize,
    is_record: bool,
    is_promising: bool,
}

/// Pure function to process a candidate
fn process_candidate(
    candidate: &BigUint,
    cache: &mut ThreadCache,
    config: &HuntConfig,
) -> Option<ProcessResult> {
    // Phase 1: Quick filter (50 first iterations)
    let quick_result = lychrel_iteration(candidate.clone(), 50);

    // Reject if growth too slow
    let start_bits = candidate.bits();
    let end_bits = quick_result
        .final_number
        .as_ref()
        .map(|n| n.bits())
        .unwrap_or(0);

    // Growth threshold: 0.4 digits/iter => 20 digits in 50 iters
    // 20 digits is approx 66 bits (20 / 0.301)
    if (end_bits as i64 - start_bits as i64) < 66 {
        return None; // Growth too slow
    }

    // Reject if palindrome found too quickly
    if quick_result.is_palindrome {
        return None;
    }

    // Phase 2: Full test with cache
    let result =
        lychrel_iteration_with_cache(candidate.clone(), config.max_iterations, cache, None);

    // Check for record or promising candidate
    // A record is a number that:
    // 1. REACHES a palindrome (not a Lychrel)
    // 2. Takes between target_iterations and max_iterations
    // 3. Numbers beyond max_iterations without palindrome are likely true Lychrels
    let is_record = result.is_palindrome
        && result.iterations >= config.target_iterations
        && result.iterations <= config.max_iterations
        && result
            .final_number
            .as_ref()
            .map_or(0, |n| n.to_string().len())
            >= config.target_final_digits;

    let is_promising = result.is_palindrome && result.iterations >= 200;

    Some(ProcessResult {
        number: candidate.to_string(),
        iterations: result.iterations,
        final_digits: result
            .final_number
            .as_ref()
            .map(|n| n.to_string().len())
            .unwrap_or(0),
        is_record,
        is_promising,
    })
}
