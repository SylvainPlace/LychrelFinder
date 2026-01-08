use clap::{Parser, Subcommand};
use lychrel_finder::{lychrel_iteration, search_range, search_range_resumable, verify_lychrel_resumable, resume_from_checkpoint_with_config, SearchConfig, SearchResults, VerifyConfig, Checkpoint, SearchCheckpoint, RecordHunter, HuntConfig, GeneratorMode};
use num_bigint::BigUint;
use serde_json;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "lychrel-finder")]
#[command(about = "Find Lychrel numbers using reverse-add iteration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Test a specific number for Lychrel property")]
    Test {
        #[arg(help = "The number to test")]
        number: String,

        #[arg(short, long, default_value = "10000")]
        max_iterations: u32,
    },

    #[command(about = "Search for Lychrel numbers in a range")]
    Search {
        #[arg(help = "Start of the range")]
        start: u64,

        #[arg(help = "End of the range")]
        end: u64,

        #[arg(short, long, default_value = "10000")]
        max_iterations: u32,

        #[arg(short, long, help = "Output file for results (JSON)")]
        output: Option<String>,

        #[arg(long, help = "Disable parallel processing")]
        no_parallel: bool,

        #[arg(short = 'c', long, help = "Save checkpoint every N numbers tested (default: 1000, use 0 to disable)")]
        checkpoint_interval: Option<u64>,

        #[arg(short = 'f', long, help = "Checkpoint file path (default: search_checkpoint_<start>_<end>.json)")]
        checkpoint_file: Option<String>,

        #[arg(long, help = "Force restart from beginning, ignoring existing checkpoint")]
        force_restart: bool,
    },

    #[command(about = "Verify if a number is truly a Lychrel number with extensive testing")]
    Verify {
        #[arg(help = "The number to verify")]
        number: String,

        #[arg(short, long, help = "Maximum iterations to perform (can be very large, e.g., 1000000)")]
        max_iterations: u64,

        #[arg(short, long, default_value = "10000", help = "Show progress every N iterations")]
        progress_interval: u64,

        #[arg(short = 'c', long, help = "Save checkpoint every N iterations (default: 10000, use 0 to disable)")]
        checkpoint_interval: Option<u64>,

        #[arg(short = 'f', long, help = "Checkpoint file path (default: checkpoint_<number>.json)")]
        checkpoint_file: Option<String>,

        #[arg(long, help = "Force restart from beginning, ignoring existing checkpoint")]
        force_restart: bool,
    },

    #[command(about = "Resume verification from a checkpoint file")]
    Resume {
        #[arg(help = "Path to the checkpoint file")]
        checkpoint_file: String,
    },

    #[command(about = "Hunt for record-breaking Lychrel numbers with optimized thread detection")]
    HuntRecord {
        #[arg(long, help = "Configuration file (JSON) - if provided, CLI options override config file values")]
        config: Option<String>,

        #[arg(long, help = "Minimum number of digits (overrides config file)")]
        min_digits: Option<usize>,

        #[arg(long, help = "Target minimum iterations (overrides config file)")]
        target_iterations: Option<u32>,

        #[arg(long, help = "Maximum iterations before considering it a Lychrel (overrides config file)")]
        max_iterations: Option<u32>,

        #[arg(long, help = "Target minimum final digits (overrides config file)")]
        target_final_digits: Option<usize>,

        #[arg(long, help = "Cache size in entries (overrides config file)")]
        cache_size: Option<usize>,

        #[arg(long, help = "Warmup cache with 1-1M range (overrides config file)")]
        warmup: Option<bool>,

        #[arg(long, help = "Generator mode: sequential, random, pattern (overrides config file)")]
        mode: Option<String>,

        #[arg(short = 'c', long, help = "Checkpoint every N numbers (overrides config file)")]
        checkpoint_interval: Option<u64>,

        #[arg(short = 'f', long, help = "Checkpoint file (overrides config file)")]
        checkpoint_file: Option<String>,
    },

    #[command(about = "Generate a default hunt configuration file")]
    InitConfig {
        #[arg(help = "Output file path (default: hunt_config.json)")]
        output: Option<String>,
    },

    #[command(about = "Run benchmark tests")]
    Benchmark,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            number,
            max_iterations,
        } => {
            test_number(&number, max_iterations);
        }
        Commands::Search {
            start,
            end,
            max_iterations,
            output,
            no_parallel,
            checkpoint_interval,
            checkpoint_file,
            force_restart,
        } => {
            search_numbers(start, end, max_iterations, output, !no_parallel, checkpoint_interval, checkpoint_file, force_restart);
        }
        Commands::Verify {
            number,
            max_iterations,
            progress_interval,
            checkpoint_interval,
            checkpoint_file,
            force_restart,
        } => {
            verify_number(&number, max_iterations, progress_interval, checkpoint_interval, checkpoint_file, force_restart);
        }
        Commands::Resume { checkpoint_file } => {
            resume_verification(&checkpoint_file);
        }
        Commands::HuntRecord {
            config,
            min_digits,
            target_iterations,
            max_iterations,
            target_final_digits,
            cache_size,
            warmup,
            mode,
            checkpoint_interval,
            checkpoint_file,
        } => {
            hunt_records_from_config(
                config,
                min_digits,
                target_iterations,
                max_iterations,
                target_final_digits,
                cache_size,
                warmup,
                mode,
                checkpoint_interval,
                checkpoint_file,
            );
        }
        Commands::InitConfig { output } => {
            init_config_file(output.as_deref().unwrap_or("hunt_config.json"));
        }
        Commands::Benchmark => {
            run_benchmark();
        }
    }
}

fn test_number(number_str: &str, max_iterations: u32) {
    let number: BigUint = match number_str.parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: Invalid number '{}'", number_str);
            std::process::exit(1);
        }
    };

    println!("Testing number: {}", number);
    println!("Max iterations: {}", max_iterations);
    println!();

    let start_time = Instant::now();
    let result = lychrel_iteration(number.clone(), max_iterations);
    let elapsed = start_time.elapsed();

    println!("Results:");
    println!("  Iterations: {}", result.iterations);
    
    if result.is_palindrome {
        if result.iterations == 0 {
            println!("  Status: Already a palindrome");
        } else {
            println!("  Status: Palindrome reached!");
            if let Some(final_num) = &result.final_number {
                println!("  Final number: {}", final_num);
            }
        }
    } else {
        println!("  Status: POTENTIAL LYCHREL NUMBER");
        if let Some(final_num) = &result.final_number {
            let final_str = final_num.to_string();
            if final_str.len() > 100 {
                println!("  Final number: {}... ({} digits)", &final_str[..100], final_str.len());
            } else {
                println!("  Final number: {}", final_num);
            }
        }
    }
    
    println!("\nTime elapsed: {:.3}s", elapsed.as_secs_f64());
}

fn verify_number(
    number_str: &str,
    max_iterations: u64,
    progress_interval: u64,
    checkpoint_interval: Option<u64>,
    checkpoint_file: Option<String>,
    force_restart: bool,
) {
    let number: BigUint = match number_str.parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: Invalid number '{}'", number_str);
            std::process::exit(1);
        }
    };

    let checkpoint_file = checkpoint_file.unwrap_or_else(|| {
        format!("checkpoint_{}.json", number_str)
    });

    // Default checkpoint interval is 10000 if not specified
    let checkpoint_interval = match checkpoint_interval {
        Some(0) => None,  // 0 explicitly disables checkpoints
        Some(n) => Some(n),
        None => Some(10000),  // Default: save every 10000 iterations
    };

    // Check if checkpoint exists and offer to resume
    if !force_restart {
        if let Ok(existing_checkpoint) = Checkpoint::load(&checkpoint_file) {
            println!("========================================");
            println!("  CHECKPOINT FOUND!");
            println!("========================================");
            println!("Found existing checkpoint for number: {}", existing_checkpoint.start_number);
            println!("  Progress: {:.2}%", existing_checkpoint.progress_percentage());
            println!("  Iterations completed: {}", existing_checkpoint.iterations_completed);
            println!("  Iterations remaining: {}", existing_checkpoint.iterations_remaining());
            println!("  Current number digits: {}", existing_checkpoint.digit_count());
            println!("  Time elapsed: {:.3}s", existing_checkpoint.elapsed_secs);
            println!("  Saved at: {}", existing_checkpoint.timestamp);
            println!("========================================");
            println!("\nDo you want to resume from this checkpoint?");
            println!("  [Y] Resume from checkpoint (default)");
            println!("  [N] Start fresh (delete checkpoint)");
            print!("\nYour choice (Y/n): ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();

            if input.is_empty() || input == "y" || input == "yes" {
                println!("\nResuming from checkpoint...\n");
                resume_verification(&checkpoint_file);
                return;
            } else {
                println!("\nDeleting old checkpoint and starting fresh...\n");
                if let Err(e) = std::fs::remove_file(&checkpoint_file) {
                    eprintln!("Warning: Could not delete checkpoint file: {}", e);
                }
            }
        }
    } else if std::path::Path::new(&checkpoint_file).exists() {
        println!("Deleting existing checkpoint (--force-restart)...\n");
        if let Err(e) = std::fs::remove_file(&checkpoint_file) {
            eprintln!("Warning: Could not delete checkpoint file: {}", e);
        }
    }

    println!("========================================");
    println!("  LYCHREL NUMBER VERIFICATION");
    println!("========================================");
    println!("Number to verify: {}", number);
    println!("Max iterations: {}", max_iterations);
    println!("Progress interval: every {} iterations", progress_interval);
    if let Some(interval) = checkpoint_interval {
        println!("Checkpoint interval: every {} iterations", interval);
        println!("Checkpoint file: {}", checkpoint_file);
    } else {
        println!("Checkpoint saving: disabled");
    }
    println!("========================================\n");

    let config = VerifyConfig {
        number: number.clone(),
        max_iterations,
        progress_interval,
        checkpoint_interval,
        checkpoint_file: Some(checkpoint_file.clone()),
    };

    let result = verify_lychrel_resumable(config, |iteration, current, elapsed, is_checkpoint| {
        let digit_count = current.to_string().len();
        let speed = if elapsed.as_secs_f64() > 0.0 {
            iteration as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        if is_checkpoint {
            println!(
                "[Progress] Iteration: {:<12} | Digits: {:<8} | Time: {:<8.2}s | Speed: {:.0} iter/s | ✓ Checkpoint saved",
                iteration,
                digit_count,
                elapsed.as_secs_f64(),
                speed
            );
        } else {
            println!(
                "[Progress] Iteration: {:<12} | Digits: {:<8} | Time: {:<8.2}s | Speed: {:.0} iter/s",
                iteration,
                digit_count,
                elapsed.as_secs_f64(),
                speed
            );
        }
    });

    println!("\n========================================");
    println!("  VERIFICATION COMPLETE");
    println!("========================================");
    println!("Iterations completed: {}", result.iterations_completed);
    println!("Total time: {:.3}s", result.total_time.as_secs_f64());
    println!();

    if result.is_palindrome {
        if result.iterations_completed == 0 {
            println!("Result: The number is ALREADY A PALINDROME");
        } else {
            println!("Result: PALINDROME REACHED!");
            println!("Status: This is NOT a Lychrel number");
            if let Some(final_num) = &result.final_number {
                let final_str = final_num.to_string();
                println!("Final number ({} digits):", final_str.len());
                if final_str.len() > 200 {
                    println!("  {}...", &final_str[..200]);
                } else {
                    println!("  {}", final_num);
                }
            }
        }
    } else {
        println!("Result: NO PALINDROME FOUND");
        println!("Status: This is LIKELY A LYCHREL NUMBER");
        if let Some(final_num) = &result.final_number {
            let final_str = final_num.to_string();
            println!("Final number ({} digits):", final_str.len());
            if final_str.len() > 200 {
                println!("  {}...", &final_str[..200]);
            } else {
                println!("  {}", final_num);
            }
        }
        println!("\nNote: {} iterations is not definitive proof.", result.iterations_completed);
        println!("      Consider running more iterations for stronger verification.");
    }
    println!("========================================");
}

fn resume_verification(checkpoint_file: &str) {
    println!("========================================");
    println!("  RESUME FROM CHECKPOINT");
    println!("========================================");
    println!("Loading checkpoint from: {}\n", checkpoint_file);

    let checkpoint = match Checkpoint::load(checkpoint_file) {
        Ok(cp) => cp,
        Err(e) => {
            eprintln!("Error: Failed to load checkpoint: {}", e);
            std::process::exit(1);
        }
    };

    println!("Checkpoint information:");
    println!("  Start number: {}", checkpoint.start_number);
    println!("  Current number: {} ({} digits)", 
        checkpoint.current_number.to_string().chars().take(50).collect::<String>(),
        checkpoint.digit_count());
    println!("  Iterations completed: {}", checkpoint.iterations_completed);
    println!("  Iterations remaining: {}", checkpoint.iterations_remaining());
    println!("  Progress: {:.2}%", checkpoint.progress_percentage());
    println!("  Elapsed time: {:.3}s", checkpoint.elapsed_secs);
    println!("  Saved at: {}", checkpoint.timestamp);
    
    let checkpoint_interval = checkpoint.checkpoint_interval.unwrap_or(0);
    if checkpoint_interval > 0 {
        println!("  Checkpoint interval: every {} iterations", checkpoint_interval);
    }
    println!("========================================\n");

    let checkpoint_file_owned = checkpoint_file.to_string();
    let result = resume_from_checkpoint_with_config(
        checkpoint,
        checkpoint_file_owned.clone(),
        checkpoint_interval,
        |iteration, current: &BigUint, elapsed: std::time::Duration, is_checkpoint: bool| {
            let digit_count = current.to_string().len();
            let speed = if elapsed.as_secs_f64() > 0.0 {
                iteration as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            if is_checkpoint {
                println!(
                    "[Progress] Iteration: {:<12} | Digits: {:<8} | Time: {:<8.2}s | Speed: {:.0} iter/s | ✓ Checkpoint saved",
                    iteration,
                    digit_count,
                    elapsed.as_secs_f64(),
                    speed
                );
            } else {
                println!(
                    "[Progress] Iteration: {:<12} | Digits: {:<8} | Time: {:<8.2}s | Speed: {:.0} iter/s",
                    iteration,
                    digit_count,
                    elapsed.as_secs_f64(),
                    speed
                );
            }
        });

    println!("\n========================================");
    println!("  VERIFICATION COMPLETE");
    println!("========================================");
    println!("Iterations completed: {}", result.iterations_completed);
    println!("Total time: {:.3}s", result.total_time.as_secs_f64());
    println!();

    if result.is_palindrome {
        if result.iterations_completed == 0 {
            println!("Result: The number is ALREADY A PALINDROME");
        } else {
            println!("Result: PALINDROME REACHED!");
            println!("Status: This is NOT a Lychrel number");
            if let Some(final_num) = &result.final_number {
                let final_str = final_num.to_string();
                println!("Final number ({} digits):", final_str.len());
                if final_str.len() > 200 {
                    println!("  {}...", &final_str[..200]);
                } else {
                    println!("  {}", final_num);
                }
            }
        }
    } else {
        println!("Result: NO PALINDROME FOUND");
        println!("Status: This is LIKELY A LYCHREL NUMBER");
        if let Some(final_num) = &result.final_number {
            let final_str = final_num.to_string();
            println!("Final number ({} digits):", final_str.len());
            if final_str.len() > 200 {
                println!("  {}...", &final_str[..200]);
            } else {
                println!("  {}", final_num);
            }
        }
        println!("\nNote: {} iterations is not definitive proof.", result.iterations_completed);
        println!("      Consider running more iterations for stronger verification.");
    }
    println!("========================================");
}

fn search_numbers(
    start: u64,
    end: u64,
    max_iterations: u32,
    output_file: Option<String>,
    parallel: bool,
    checkpoint_interval: Option<u64>,
    checkpoint_file: Option<String>,
    force_restart: bool,
) {
    let checkpoint_file = checkpoint_file.unwrap_or_else(|| {
        format!("search_checkpoint_{}_{}.json", start, end)
    });

    // Default checkpoint interval is 1000 if not specified
    let checkpoint_interval = match checkpoint_interval {
        Some(0) => None,
        Some(n) => Some(n),
        None => Some(1000),
    };

    // Check if checkpoint exists and offer to resume
    if !force_restart && !parallel {
        if let Ok(existing_checkpoint) = SearchCheckpoint::load(&checkpoint_file) {
            println!("========================================");
            println!("  SEARCH CHECKPOINT FOUND!");
            println!("========================================");
            println!("Search range: {} to {}", existing_checkpoint.start_range, existing_checkpoint.end_range);
            println!("  Progress: {:.2}%", existing_checkpoint.progress_percentage());
            println!("  Numbers tested: {}", existing_checkpoint.numbers_tested);
            println!("  Numbers remaining: {}", existing_checkpoint.numbers_remaining());
            println!("  Potential Lychrel found so far: {}", existing_checkpoint.potential_lychrel_found.len());
            println!("  Elapsed time: {:.3}s", existing_checkpoint.elapsed_secs);
            println!("  Saved at: {}", existing_checkpoint.timestamp);
            
            let checkpoint_interval_val = existing_checkpoint.checkpoint_interval.unwrap_or(0);
            if checkpoint_interval_val > 0 {
                println!("  Checkpoint interval: every {} numbers", checkpoint_interval_val);
            }
            println!("========================================");
            println!("\nDo you want to resume from this checkpoint?");
            println!("  [Y] Resume from checkpoint (default)");
            println!("  [N] Start fresh (delete checkpoint)");
            print!("\nYour choice (Y/n): ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();

            if input.is_empty() || input == "y" || input == "yes" {
                println!("\nResuming search from checkpoint...\n");
                resume_search(&checkpoint_file, output_file);
                return;
            } else {
                println!("\nDeleting old checkpoint and starting fresh...\n");
                if let Err(e) = std::fs::remove_file(&checkpoint_file) {
                    eprintln!("Warning: Could not delete checkpoint file: {}", e);
                }
            }
        }
    } else if !parallel && std::path::Path::new(&checkpoint_file).exists() {
        if force_restart {
            println!("Deleting existing checkpoint (--force-restart)...\n");
            if let Err(e) = std::fs::remove_file(&checkpoint_file) {
                eprintln!("Warning: Could not delete checkpoint file: {}", e);
            }
        }
    }

    if parallel && checkpoint_interval.is_some() {
        println!("Warning: Checkpoints are not supported with parallel processing. Disabling checkpoints.\n");
    }

    println!("Searching range: {} to {}", start, end);
    println!("Max iterations: {}", max_iterations);
    println!("Parallel processing: {}", if parallel { "enabled" } else { "disabled" });
    if !parallel {
        if let Some(interval) = checkpoint_interval {
            println!("Checkpoint interval: every {} numbers", interval);
            println!("Checkpoint file: {}", checkpoint_file);
        } else {
            println!("Checkpoint saving: disabled");
        }
    }
    println!();

    let start_time = Instant::now();
    let results = if parallel {
        let config = SearchConfig {
            start: BigUint::from(start),
            end: BigUint::from(end),
            max_iterations,
            parallel: true,
            checkpoint_interval: None,
            checkpoint_file: None,
        };
        search_range(config)
    } else {
        let config = SearchConfig {
            start: BigUint::from(start),
            end: BigUint::from(end),
            max_iterations,
            parallel: false,
            checkpoint_interval,
            checkpoint_file: Some(checkpoint_file.clone()),
        };
        
        let total_numbers = end - start + 1;
        let mut last_display = 0u64;
        let display_interval = 100;
        
        search_range_resumable(config, |tested, current, is_checkpoint| {
            if is_checkpoint || tested - last_display >= display_interval {
                let progress = (tested as f64 / total_numbers as f64) * 100.0;
                if is_checkpoint {
                    println!(
                        "[Search] Tested: {}/{} ({:.1}%) | Current: {} | ✓ Checkpoint saved",
                        tested, total_numbers, progress, current
                    );
                } else {
                    println!(
                        "[Search] Tested: {}/{} ({:.1}%) | Current: {}",
                        tested, total_numbers, progress, current
                    );
                }
                last_display = tested;
            }
        })
    };
    
    let elapsed = start_time.elapsed();

    print_search_results(&results, elapsed);

    if let Some(filename) = output_file {
        save_results_to_file(&results, &filename);
    }

    // Clean up checkpoint file on successful completion
    if !parallel && std::path::Path::new(&checkpoint_file).exists() {
        if let Err(e) = std::fs::remove_file(&checkpoint_file) {
            eprintln!("Warning: Could not delete checkpoint file: {}", e);
        }
    }
}

fn resume_search(checkpoint_file: &str, output_file: Option<String>) {
    use lychrel_finder::resume_search_from_checkpoint;
    
    let checkpoint = match SearchCheckpoint::load(checkpoint_file) {
        Ok(cp) => cp,
        Err(e) => {
            eprintln!("Error: Failed to load checkpoint: {}", e);
            std::process::exit(1);
        }
    };

    let total_numbers = if let (Ok(start), Ok(end)) = (
        checkpoint.start_range.to_string().parse::<u64>(),
        checkpoint.end_range.to_string().parse::<u64>()
    ) {
        end - start + 1
    } else {
        0
    };

    let mut last_display = checkpoint.numbers_tested;
    let display_interval = 100;
    
    let start_time = Instant::now();
    let results = resume_search_from_checkpoint(checkpoint, |tested, current, is_checkpoint| {
        if is_checkpoint || tested - last_display >= display_interval {
            let progress = (tested as f64 / total_numbers as f64) * 100.0;
            if is_checkpoint {
                println!(
                    "[Search] Tested: {}/{} ({:.1}%) | Current: {} | ✓ Checkpoint saved",
                    tested, total_numbers, progress, current
                );
            } else {
                println!(
                    "[Search] Tested: {}/{} ({:.1}%) | Current: {}",
                    tested, total_numbers, progress, current
                );
            }
            last_display = tested;
        }
    });
    
    let elapsed = start_time.elapsed();

    print_search_results(&results, elapsed);

    if let Some(filename) = output_file {
        save_results_to_file(&results, &filename);
    }

    // Clean up checkpoint file on successful completion
    if std::path::Path::new(checkpoint_file).exists() {
        if let Err(e) = std::fs::remove_file(checkpoint_file) {
            eprintln!("Warning: Could not delete checkpoint file: {}", e);
        }
    }
}

fn print_search_results(results: &SearchResults, elapsed: std::time::Duration) {
    println!("Search complete!");
    println!("  Total tested: {}", results.total_tested);
    println!("  Potential Lychrel numbers found: {}", results.potential_lychrel.len());
    println!("  Numbers reaching palindromes: {}", results.palindromes_found.len());
    println!("  Time elapsed: {:.3}s", elapsed.as_secs_f64());
    
    if !results.potential_lychrel.is_empty() {
        println!("\nPotential Lychrel numbers:");
        for result in &results.potential_lychrel {
            println!("  - {}", result.start_number);
        }
    }
}

fn save_results_to_file(results: &SearchResults, filename: &str) {
    match serde_json::to_string_pretty(&results.potential_lychrel) {
        Ok(json) => {
            match File::create(filename) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(json.as_bytes()) {
                        eprintln!("Error writing to file: {}", e);
                    } else {
                        println!("\nResults saved to: {}", filename);
                    }
                }
                Err(e) => eprintln!("Error creating file: {}", e),
            }
        }
        Err(e) => eprintln!("Error serializing results: {}", e),
    }
}

fn run_benchmark() {
    println!("Running benchmarks...\n");

    let test_cases = vec![
        (89u64, "Number 89 (24 iterations to palindrome)", 1000),
        (196u64, "Number 196 (candidate Lychrel)", 5000),
        (10677u64, "Number 10677 (large iteration count)", 5000),
        (1186060307891929990u64, "Large number (19 digits)", 1000),
    ];

    for (number, description, max_iter) in test_cases {
        println!("Test: {}", description);
        let start_time = Instant::now();
        let result = lychrel_iteration(BigUint::from(number), max_iter);
        let elapsed = start_time.elapsed();
        
        println!("  Iterations: {}", result.iterations);
        println!("  Time: {:.6}s", elapsed.as_secs_f64());
        println!();
    }

    println!("Range search benchmark (1-10000, parallel):");
    let config = SearchConfig {
        start: BigUint::from(1u64),
        end: BigUint::from(10000u64),
        max_iterations: 1000,
        parallel: true,
        checkpoint_interval: None,
        checkpoint_file: None,
    };
    let start_time = Instant::now();
    let results = search_range(config);
    let elapsed = start_time.elapsed();
    println!("  Tested: {}", results.total_tested);
    println!("  Potential Lychrel found: {}", results.potential_lychrel.len());
    println!("  Time: {:.3}s", elapsed.as_secs_f64());
    
    println!("\nIntensive search benchmark (1-100000, 1000 max iterations):");
    let config_intensive = SearchConfig {
        start: BigUint::from(1u64),
        end: BigUint::from(100000u64),
        max_iterations: 1000,
        parallel: true,
        checkpoint_interval: None,
        checkpoint_file: None,
    };
    let start_time = Instant::now();
    let results = search_range(config_intensive);
    let elapsed = start_time.elapsed();
    println!("  Tested: {}", results.total_tested);
    println!("  Potential Lychrel found: {}", results.potential_lychrel.len());
    println!("  Time: {:.3}s", elapsed.as_secs_f64());
}

fn parse_mode(mode_str: &str) -> GeneratorMode {
    match mode_str.to_lowercase().as_str() {
        "sequential" => GeneratorMode::Sequential,
        "random" => GeneratorMode::SmartRandom,
        "pattern" => GeneratorMode::PatternBased,
        _ => {
            eprintln!("Warning: Unknown mode '{}', using sequential", mode_str);
            GeneratorMode::Sequential
        }
    }
}

fn init_config_file(output: &str) {
    let config = HuntConfig::default();
    
    match config.save_to_file(std::path::Path::new(output)) {
        Ok(_) => {
            println!("✓ Default configuration file created: {}", output);
            println!("\nConfiguration:");
            println!("  Min digits:          {}", config.min_digits);
            println!("  Target iterations:   {} - {}", config.target_iterations, config.max_iterations);
            println!("  Target final digits: {}", config.target_final_digits);
            println!("  Cache size:          {}", config.cache_size);
            println!("  Generator mode:      {:?}", config.generator_mode);
            println!("  Checkpoint interval: {}", config.checkpoint_interval);
            println!("  Checkpoint file:     {}", config.checkpoint_file);
            println!("  Warmup:              {}", config.warmup);
            println!("\nYou can now edit this file and use:");
            println!("  cargo run --release -- hunt-record --config {}", output);
        }
        Err(e) => {
            eprintln!("Error creating config file: {}", e);
            std::process::exit(1);
        }
    }
}

fn hunt_records_from_config(
    config_file: Option<String>,
    min_digits_override: Option<usize>,
    target_iterations_override: Option<u32>,
    max_iterations_override: Option<u32>,
    target_final_digits_override: Option<usize>,
    cache_size_override: Option<usize>,
    warmup_override: Option<bool>,
    mode_override: Option<String>,
    checkpoint_interval_override: Option<u64>,
    checkpoint_file_override: Option<String>,
) {
    // Load config from file or use defaults
    let mut config = if let Some(config_path) = config_file {
        match HuntConfig::load_from_file(std::path::Path::new(&config_path)) {
            Ok(c) => {
                println!("✓ Loaded configuration from: {}\n", config_path);
                c
            }
            Err(e) => {
                eprintln!("Error loading config file '{}': {}", config_path, e);
                eprintln!("Using default configuration instead.\n");
                HuntConfig::default()
            }
        }
    } else {
        HuntConfig::default()
    };

    // Apply CLI overrides
    if let Some(v) = min_digits_override {
        config.min_digits = v;
    }
    if let Some(v) = target_iterations_override {
        config.target_iterations = v;
    }
    if let Some(v) = max_iterations_override {
        config.max_iterations = v;
    }
    if let Some(v) = target_final_digits_override {
        config.target_final_digits = v;
    }
    if let Some(v) = cache_size_override {
        config.cache_size = v;
    }
    if let Some(v) = warmup_override {
        config.warmup = v;
    }
    if let Some(v) = mode_override {
        config.generator_mode = parse_mode(&v);
    }
    if let Some(v) = checkpoint_interval_override {
        config.checkpoint_interval = v;
    }
    if let Some(v) = checkpoint_file_override {
        config.checkpoint_file = v;
    }

    hunt_records_with_config(config);
}

fn hunt_records_with_config(config: HuntConfig) {
    println!("🔍 ═══════════════════════════════════════════");
    println!("   LYCHREL RECORD HUNT");
    println!("═══════════════════════════════════════════");
    println!("Configuration:");
    println!("  Min digits:          {}", config.min_digits);
    println!("  Target iterations:   {} - {}", config.target_iterations, config.max_iterations);
    println!("  Target final digits: {}", config.target_final_digits);
    println!("  Cache size:          {}", config.cache_size);
    println!("  Generator mode:      {:?}", config.generator_mode);
    println!("  Checkpoint interval: {} numbers", config.checkpoint_interval);
    println!("  Checkpoint file:     {}", config.checkpoint_file);
    println!("  Warmup:              {}", config.warmup);
    println!("═══════════════════════════════════════════\n");
    
    let warmup = config.warmup;
    
    // Create hunter
    let mut hunter = RecordHunter::new(config);
    
    // Warmup if requested
    if warmup {
        hunter.warmup_cache();
    }
    
    // Start hunting
    let results = hunter.hunt();
    
    // Summary
    println!("\n📊 FINAL SUMMARY");
    println!("═══════════════════════════════════════════");
    println!("Numbers tested:      {}", results.numbers_tested);
    println!("Seeds tested:        {}", results.seeds_tested);
    println!("Records found:       {}", results.records.len());
    println!("Candidates (200+):   {}", results.candidates_above_200.len());
    println!("Best iterations:     {}", results.best_iterations_found);
    println!("Time elapsed:        {:.2}s", results.elapsed_time.as_secs_f64());
    
    if results.elapsed_time.as_secs() > 0 {
        let rate = results.numbers_tested as f64 / results.elapsed_time.as_secs() as f64;
        println!("Average rate:        {:.0} numbers/second", rate);
    }
    
    println!("═══════════════════════════════════════════\n");
    
    if !results.records.is_empty() {
        println!("🎉 {} RECORD(S) FOUND!", results.records.len());
        for record in &results.records {
            println!("  - Number: {} ({} iterations, {} final digits)",
                     record.number, record.iterations, record.final_digits);
        }
        println!();
    }
    
    if !results.candidates_above_200.is_empty() && results.candidates_above_200.len() <= 20 {
        println!("📋 Promising candidates (200+ iterations):");
        for candidate in &results.candidates_above_200 {
            println!("  - {} ({} iter, {} digits)",
                     candidate.number, candidate.iterations, candidate.final_digits);
        }
        println!();
    }
}
