use clap::{Parser, Subcommand};
use lychrel_finder::{lychrel_iteration, search_range, verify_lychrel, SearchConfig, SearchResults, VerifyConfig};
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
    },

    #[command(about = "Verify if a number is truly a Lychrel number with extensive testing")]
    Verify {
        #[arg(help = "The number to verify")]
        number: String,

        #[arg(short, long, help = "Maximum iterations to perform (can be very large, e.g., 1000000)")]
        max_iterations: u64,

        #[arg(short, long, default_value = "10000", help = "Show progress every N iterations")]
        progress_interval: u64,
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
        } => {
            search_numbers(start, end, max_iterations, output, !no_parallel);
        }
        Commands::Verify {
            number,
            max_iterations,
            progress_interval,
        } => {
            verify_number(&number, max_iterations, progress_interval);
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

fn verify_number(number_str: &str, max_iterations: u64, progress_interval: u64) {
    let number: BigUint = match number_str.parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: Invalid number '{}'", number_str);
            std::process::exit(1);
        }
    };

    println!("========================================");
    println!("  LYCHREL NUMBER VERIFICATION");
    println!("========================================");
    println!("Number to verify: {}", number);
    println!("Max iterations: {}", max_iterations);
    println!("Progress interval: every {} iterations", progress_interval);
    println!("========================================\n");

    let config = VerifyConfig {
        number: number.clone(),
        max_iterations,
        progress_interval,
    };

    let result = verify_lychrel(config, |iteration, current, elapsed| {
        let digit_count = current.to_string().len();
        let speed = if elapsed.as_secs_f64() > 0.0 {
            iteration as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        println!(
            "[Progress] Iteration: {:<12} | Digits: {:<8} | Time: {:<8.2}s | Speed: {:.0} iter/s",
            iteration,
            digit_count,
            elapsed.as_secs_f64(),
            speed
        );
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
) {
    println!("Searching range: {} to {}", start, end);
    println!("Max iterations: {}", max_iterations);
    println!("Parallel processing: {}", if parallel { "enabled" } else { "disabled" });
    println!();

    let config = SearchConfig {
        start: BigUint::from(start),
        end: BigUint::from(end),
        max_iterations,
        parallel,
    };

    let start_time = Instant::now();
    let results = search_range(config);
    let elapsed = start_time.elapsed();

    print_search_results(&results, elapsed);

    if let Some(filename) = output_file {
        save_results_to_file(&results, &filename);
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
    };
    let start_time = Instant::now();
    let results = search_range(config_intensive);
    let elapsed = start_time.elapsed();
    println!("  Tested: {}", results.total_tested);
    println!("  Potential Lychrel found: {}", results.potential_lychrel.len());
    println!("  Time: {:.3}s", elapsed.as_secs_f64());
}
