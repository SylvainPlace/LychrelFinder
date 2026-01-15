use lychrel_finder::lychrel::{lychrel_iteration, lychrel_iteration_with_cache};
use lychrel_finder::seed_generator::SeedGenerator;
use lychrel_finder::thread_cache::ThreadCache;
use lychrel_finder::{GeneratorMode, HuntConfig};
use num_bigint::BigUint;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct BenchmarkMetrics {
    config_name: String,
    duration_secs: f64,
    candidates_tested: u64,
    seeds_tested: u64,
    cache_hits: u64,
    cache_misses: u64,
    records_found: usize,
    best_iterations: u32,
    candidates_per_sec: f64,
    cache_hit_rate: f64,
}

struct StatsWrapper {
    candidates_tested: AtomicU64,
    seeds_tested: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    best_iterations: AtomicU32,
    records_found: AtomicUsize,
}

fn run_benchmark(
    config: HuntConfig,
    config_name: &str,
    max_duration: Duration,
) -> BenchmarkMetrics {
    println!("🏃 Running benchmark: {}", config_name);
    println!("   Max digits: {}", config.min_digits);
    println!(
        "   Target iterations: {}-{}",
        config.target_iterations, config.max_iterations
    );
    println!("   Warmup: {}", config.warmup);
    println!();

    let stats = Arc::new(StatsWrapper {
        candidates_tested: AtomicU64::new(0),
        seeds_tested: AtomicU64::new(0),
        cache_hits: AtomicU64::new(0),
        cache_misses: AtomicU64::new(0),
        best_iterations: AtomicU32::new(0),
        records_found: AtomicUsize::new(0),
    });

    let stats_clone = stats.clone();
    let mut generator = SeedGenerator::new(config.min_digits, config.generator_mode);
    let mut cache = ThreadCache::new(config.cache_size);

    // Limite absolue de candidats à tester pour éviter l'infini
    let max_candidates: u64 = 500000;

    if config.warmup {
        println!("🔥 Warming up cache...");
        let warmup_start = Instant::now();

        // Use a separate generator for warmup to target relevant numbers
        let mut warmup_generator = SeedGenerator::new(config.min_digits, config.generator_mode.clone());
        let mut warmup_count = 0;
        let warmup_limit = 10_000; // Warmup with 10k items maximum

        while let Some(candidate) = warmup_generator.next() {
            let elapsed = warmup_start.elapsed();
            if elapsed > max_duration / 2 || warmup_count >= warmup_limit {
                if elapsed > max_duration / 2 {
                    println!("⏱️  Warmup interrupted - taking too long");
                }
                break;
            }

            lychrel_iteration_with_cache(candidate, 1000, &mut cache);
            warmup_count += 1;
            
            if warmup_count % 1000 == 0 {
                println!("  Warmup progress: {}/{}", warmup_count, warmup_limit);
            }
        }

        let warmup_elapsed = warmup_start.elapsed();
        let cache_stats = cache.stats();
        println!("✓ Cache warmed up in {:.2}s", warmup_elapsed.as_secs_f64());
        println!("  Cache entries: {}", cache_stats.entries);
        println!(
            "  Hit rate during warmup: {:.1}%",
            cache_stats.hit_rate * 100.0
        );
        println!();
    }

    let start_time = Instant::now();

    while start_time.elapsed() < max_duration
        && stats_clone
            .candidates_tested
            .load(std::sync::atomic::Ordering::Relaxed)
            < max_candidates
    {
        if let Some(candidate) = generator.next() {
            stats_clone
                .candidates_tested
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let cache_stats_before = cache.stats();

            let quick_result = lychrel_iteration(candidate.clone(), 50);
            let digit_growth = quick_result
                .final_number
                .as_ref()
                .map(|n| n.to_string().len() as f64 / 50.0)
                .unwrap_or(0.0);

            if digit_growth < 0.4 {
                continue;
            }

            if quick_result.is_palindrome {
                continue;
            }

            let result =
                lychrel_iteration_with_cache(candidate.clone(), config.max_iterations, &mut cache);

            let cache_stats_after = cache.stats();
            let hits_delta = cache_stats_after.hits - cache_stats_before.hits;
            let misses_delta = cache_stats_after.misses - cache_stats_before.misses;

            stats_clone
                .cache_hits
                .fetch_add(hits_delta, std::sync::atomic::Ordering::Relaxed);
            stats_clone
                .cache_misses
                .fetch_add(misses_delta, std::sync::atomic::Ordering::Relaxed);
            stats_clone
                .seeds_tested
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            if result.is_palindrome
                && result.iterations >= config.target_iterations
                && result.iterations <= config.max_iterations
            {
                let final_digits = result
                    .final_number
                    .as_ref()
                    .map(|n| n.to_string().len())
                    .unwrap_or(0);

                if final_digits >= config.target_final_digits {
                    stats_clone
                        .records_found
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    let current_best = stats_clone
                        .best_iterations
                        .load(std::sync::atomic::Ordering::Relaxed);
                    if result.iterations > current_best {
                        stats_clone
                            .best_iterations
                            .store(result.iterations, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }

            if result.iterations >= 200 && result.is_palindrome {
                let current_best = stats_clone
                    .best_iterations
                    .load(std::sync::atomic::Ordering::Relaxed);
                if result.iterations > current_best {
                    stats_clone
                        .best_iterations
                        .store(result.iterations, std::sync::atomic::Ordering::Relaxed);
                }
            }
        } else {
            break;
        }
    }

    let elapsed = start_time.elapsed();
    let duration_secs = elapsed.as_secs_f64();

    let candidates_tested = stats
        .candidates_tested
        .load(std::sync::atomic::Ordering::Relaxed);
    let seeds_tested = stats
        .seeds_tested
        .load(std::sync::atomic::Ordering::Relaxed);
    let cache_hits = stats.cache_hits.load(std::sync::atomic::Ordering::Relaxed);
    let cache_misses = stats
        .cache_misses
        .load(std::sync::atomic::Ordering::Relaxed);
    let records_found = stats
        .records_found
        .load(std::sync::atomic::Ordering::Relaxed);
    let best_iterations = stats
        .best_iterations
        .load(std::sync::atomic::Ordering::Relaxed);

    BenchmarkMetrics {
        config_name: config_name.to_string(),
        duration_secs,
        candidates_tested,
        seeds_tested,
        cache_hits,
        cache_misses,
        records_found,
        best_iterations,
        candidates_per_sec: candidates_tested as f64 / duration_secs.max(0.1),
        cache_hit_rate: if cache_hits + cache_misses > 0 {
            cache_hits as f64 / (cache_hits + cache_misses) as f64
        } else {
            0.0
        },
    }
}

fn print_table(metrics: &[BenchmarkMetrics]) {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                           RECORD HUNT BENCHMARK RESULTS                       ║");
    println!("╠═══════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Config                                 │ Time     │ Cand/s   │ Seeds     │ Cache Hit │ Best    ║");
    println!("║                                        │ (s)      │          │ Tested    │ %         │ Iter    ║");
    println!("╠═══════════════════════════════════════════════════════════════════════════════╣");

    for m in metrics {
        println!(
            "║ {:<38} │ {:<8.2} │ {:<8.0} │ {:<9} │ {:<7.1}%  │ {:<7} ║",
            m.config_name,
            m.duration_secs,
            m.candidates_per_sec,
            m.seeds_tested,
            m.cache_hit_rate * 100.0,
            m.best_iterations
        );
    }

    println!("╠═══════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Metrics:                                                                      ║");
    println!("║ - Duration:  Total execution time                                             ║");
    println!("║ - Cand/s:    Candidates tested per second (higher is better)                  ║");
    println!("║ - Seeds:     Total seeds tested after filtering                               ║");
    println!("║ - Cache %:   Cache hit rate (higher is better)                                ║");
    println!("║ - Best Iter: Best iterations found during run                                 ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn save_to_file(metrics: &[BenchmarkMetrics], filename: &str) {
    let header = format!(
        "╔═══════════════════════════════════════════════════════════════════════════════╗\n\
         ║                           RECORD HUNT BENCHMARK RESULTS                       ║\n\
         ╠═══════════════════════════════════════════════════════════════════════════════╣\n\
         ║ Config                                 │ Time     │ Cand/s   │ Seeds     │ Cache Hit │ Best    ║\n\
         ║                                        │ (s)      │          │ Tested    │ %         │ Iter    ║\n\
         ╠═══════════════════════════════════════════════════════════════════════════════╣\n"
    );

    let rows: Vec<String> = metrics
        .iter()
        .map(|m| {
            format!(
                "║ {:<38} │ {:<8.2} │ {:<8.0} │ {:<9} │ {:<7.1}%  │ {:<7} ║",
                m.config_name,
                m.duration_secs,
                m.candidates_per_sec,
                m.seeds_tested,
                m.cache_hit_rate * 100.0,
                m.best_iterations
            )
        })
        .collect();

    let footer = "╚═══════════════════════════════════════════════════════════════════════════════════════════════╝\n\
Metrics:\n\
 - Duration:  Total execution time\n\
 - Cand/s:    Candidates tested per second (higher is better)\n\
 - Seeds:     Total seeds tested after filtering\n\
 - Cache %:   Cache hit rate (higher is better)\n\
 - Best Iter: Best iterations found during run\n";

    let output = format!("{}{}\n{}", header, rows.join("\n"), footer);

    match std::fs::write(filename, output) {
        Ok(_) => println!("✓ Results saved to: {}", filename),
        Err(e) => eprintln!("✗ Failed to save results: {}", e),
    }
}

fn main() {
    println!("🔍 Record Hunt Benchmark");
    println!("════════════════════════════════════════════════════════════════════");
    println!();

    // Use clap to parse arguments manually since this is a benchmark
    let args: Vec<String> = std::env::args().collect();
    let mut duration_secs = 300; // Default 5 minutes
    
    // Parse --duration or -d argument
    for i in 0..args.len() {
        if args[i] == "--duration" || args[i] == "-d" {
            if i + 1 < args.len() {
                if let Ok(d) = args[i+1].parse::<u64>() {
                    duration_secs = d;
                }
            }
        }
    }
    
    let max_duration = Duration::from_secs(duration_secs);
    println!("⏱️  Max duration per benchmark: {} seconds", duration_secs);

    let configs = vec![
        (
            "Quick (No Warmup, 100 iters)",
            HuntConfig {
                min_digits: 23,
                max_digits: Some(23),
                target_iterations: 50,
                max_iterations: 100,
                target_final_digits: 50,
                cache_size: 50000,
                generator_mode: GeneratorMode::Sequential,
                checkpoint_interval: 0,
                checkpoint_file: "/dev/null".to_string(),
                warmup: false,
            },
        ),
        (
            "Quick (With Warmup, 100 iters)",
            HuntConfig {
                min_digits: 23,
                max_digits: Some(23),
                target_iterations: 50,
                max_iterations: 100,
                target_final_digits: 50,
                cache_size: 50000,
                generator_mode: GeneratorMode::Sequential,
                checkpoint_interval: 0,
                checkpoint_file: "/dev/null".to_string(),
                warmup: true,
            },
        ),
        (
            "Realistic (No Warmup, 289-300 iters)",
            HuntConfig {
                min_digits: 23,
                max_digits: Some(23),
                target_iterations: 289,
                max_iterations: 300,
                target_final_digits: 142,
                cache_size: 1000000,
                generator_mode: GeneratorMode::Sequential,
                checkpoint_interval: 0,
                checkpoint_file: "/dev/null".to_string(),
                warmup: false,
            },
        ),
        (
            "Realistic (With Warmup, 289-300 iters)",
            HuntConfig {
                min_digits: 23,
                max_digits: Some(23),
                target_iterations: 289,
                max_iterations: 300,
                target_final_digits: 142,
                cache_size: 1000000,
                generator_mode: GeneratorMode::Sequential,
                checkpoint_interval: 0,
                checkpoint_file: "/dev/null".to_string(),
                warmup: true,
            },
        ),
    ];

    let mut all_metrics = Vec::new();

    for (name, config) in &configs {
        let metrics = run_benchmark(config.clone(), name, max_duration);
        all_metrics.push(metrics);
        println!();
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("benchmark_results_{}.txt", timestamp);

    print_table(&all_metrics);
    save_to_file(&all_metrics, &filename);

    println!("✅ Benchmark complete!");
    println!("📊 Results saved to: {}", filename);
    println!();
    println!("Next steps:");
    println!("  1. Implement optimizations");
    println!("  2. Run: cargo run --release --bin record_hunt_benchmark");
    println!("  3. Compare files to measure improvement");
}
