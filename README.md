# LychrelFinder

A Rust software for finding Lychrel numbers using the reverse-add iteration algorithm.

## What is a Lychrel Number?

A Lychrel number is a natural number that, hypothetically, never produces a palindrome through the following iterative process:
1. Take a number (example: 196)
2. Add it to its reverse (196 + 691 = 887)
3. Check if the result is a palindrome
4. If not, repeat the process

The number 196 is the smallest known Lychrel candidate - after millions of iterations, no palindrome has been found.

## Features

- ✅ Test individual numbers for Lychrel property
- ✅ Search ranges of numbers with optional parallel processing
- ✅ Deep verification mode with millions of iterations and live progress tracking
- ✅ **Hunt for record-breaking Lychrel numbers with optimized thread detection**
- ✅ Automatic checkpoint/resume system for long-running operations (verify and sequential search)
- ✅ Support for arbitrarily large numbers (BigInt arithmetic)
- ✅ Parallelized processing for optimal performance (search command)
- ✅ Export results to JSON
- ✅ Built-in performance benchmarks

## Installation

Make sure you have Rust and Cargo installed. If not:
```bash
# Windows (via rustup)
# Download and run: https://rustup.rs/
```

Clone or download this project, then compile:
```bash
cd LychrelFinder
cargo build --release
```

## Usage

### Test a Specific Number

```bash
cargo run --release -- test 196 --max-iterations 1000
```

Example output:
```
Testing number: 196
Max iterations: 1000

Results:
  Iterations: 1000
  Status: POTENTIAL LYCHREL NUMBER
  Final number: 8179...4981 (109 digits)

Time elapsed: 0.042s
```

### Search a Range

**Note:** Checkpoints are saved automatically every 1000 numbers by default when using sequential search (--no-parallel).
Parallel search doesn't support checkpoints.

```bash
# Search from 1 to 10000 (parallel by default, no checkpoints)
cargo run --release -- search 1 10000

# Sequential search with automatic checkpoints
cargo run --release -- search 1 100000 --no-parallel

# Custom checkpoint interval
cargo run --release -- search 1 100000 --no-parallel -c 5000

# With JSON export
cargo run --release -- search 1 10000 --output results.json

# Disable checkpoints
cargo run --release -- search 1 10000 --no-parallel -c 0
```

Example output with checkpoints:
```
Searching range: 1 to 100000
Max iterations: 10000
Parallel processing: disabled
Checkpoint interval: every 1000 numbers
Checkpoint file: search_checkpoint_1_100000.json

[Search] Tested: 1000/100000 (1.0%) | Current: 1000
[Search] Tested: 2000/100000 (2.0%) | Current: 2000 | ✓ Checkpoint saved
[Search] Tested: 3000/100000 (3.0%) | Current: 3000 | ✓ Checkpoint saved
...

Search complete!
  Total tested: 100000
  Potential Lychrel numbers found: 251
  Numbers reaching palindromes: 99749
  Time elapsed: 45.678s
```

### Hunt for Record-Breaking Lychrel Numbers

**NEW!** Search for extremely rare numbers that reach palindromes after many iterations (289-300 by default) using optimized thread convergence detection. This feature implements intelligent caching to avoid redundant computation when numbers converge to the same sequences.

**Important:** A "record" is a number that **reaches a palindrome** within the iteration window. Numbers exceeding the maximum iterations without reaching a palindrome are considered true Lychrel candidates, not records.

#### Using Configuration File (Recommended)

```bash
# Generate a default configuration file
cargo run --release -- init-config

# Edit hunt_config.json to your liking, then run:
cargo run --release -- hunt-record --config hunt_config.json

# Override specific settings from CLI
cargo run --release -- hunt-record --config hunt_config.json --warmup true --min-digits 25
```

Example `hunt_config.json`:
```json
{
  "min_digits": 23,
  "target_iterations": 289,
  "max_iterations": 300,
  "target_final_digits": 142,
  "cache_size": 1000000,
  "generator_mode": "Sequential",
  "checkpoint_interval": 100000,
  "checkpoint_file": "hunt_checkpoint.json",
  "warmup": false
}
```

#### Using CLI Arguments Only

```bash
# Basic hunt for 23+ digit numbers
cargo run --release -- hunt-record --min-digits 23

# With cache warmup (recommended for better performance)
cargo run --release -- hunt-record --min-digits 23 --warmup true

# Custom targets (search for 300+ iterations, 150+ final digits)
cargo run --release -- hunt-record \
    --min-digits 25 \
    --target-iterations 300 \
    --max-iterations 310 \
    --target-final-digits 150 \
    --warmup true

# Different generation modes
cargo run --release -- hunt-record --mode sequential --min-digits 20
cargo run --release -- hunt-record --mode random --min-digits 23
```

Example output:
```
🔍 ═══════════════════════════════════════════
   LYCHREL RECORD HUNT
═══════════════════════════════════════════
Configuration:
  Min digits:          23
  Target iterations:   289 - 300
  Target final digits: 142
  Cache size:          1000000
  Generator mode:      sequential
  Checkpoint interval: 100000 numbers
═══════════════════════════════════════════

🔥 Warming up thread cache with known range (1-1,000,000)...
  Warmup progress: 100,000/1,000,000
  ...
✓ Cache warmed up in 12.45s
  Cache entries: 85423
  Hit rate during warmup: 64.2%

🎯 Starting record hunt...

[Hunt] Tested: 10000 | Seeds: 5234 | Cache: 68.3% hit | Rate: 245/s | Best: 156 iter (87 digits) | Skip: 47.7%
[Hunt] Tested: 20000 | Seeds: 10521 | Cache: 69.1% hit | Rate: 248/s | Best: 178 iter (95 digits) | Skip: 47.4%
...

🎉 ═══════════════════════════════════════════
   POTENTIAL RECORD FOUND!
═══════════════════════════════════════════
Number:      12345678901234567890123
Iterations:  291
Final digits: 148
Found at:    2026-01-08 15:30:45
═══════════════════════════════════════════

📊 FINAL SUMMARY
═══════════════════════════════════════════
Numbers tested:      100000
Seeds tested:        52341
Records found:       1
Candidates (200+):   15
Best iterations:     291
Time elapsed:        408.23s
Average rate:        245 numbers/second
═══════════════════════════════════════════
```

**How it works:**
- **Thread Cache:** Detects when numbers converge to the same sequence and skips redundant computation (~2-5x speedup)
- **Seed Filtering:** Only tests "primary" numbers (smallest in their convergence family) to avoid duplicates
- **Smart Generation:** Three modes available - sequential (exhaustive), random (broad coverage), pattern (experimental)
- **Iteration Window:** Tests numbers within a configurable range (default 289-300). Numbers reaching palindrome in this window are records; numbers exceeding it without palindrome are likely true Lychrels
- **Checkpoints:** Saves progress every 100K numbers tested for resumable searches
- **Live Statistics:** Shows cache hit rate, test rate, best found, and skip percentage

### Verify a Lychrel Candidate (Deep Testing)

For extensive verification with millions of iterations and live progress tracking.
**Checkpoints are saved automatically every 10000 iterations by default.**

```bash
# Test 196 with 1 million iterations (auto-saves checkpoints every 10000 iterations)
cargo run --release -- verify 196 --max-iterations 1000000

# Test with custom checkpoint interval
cargo run --release -- verify 196 -m 10000000 -c 50000

# Disable checkpoint saving
cargo run --release -- verify 196 -m 10000000 -c 0
```

**Auto-Resume Feature:**
- If you run `verify` on a number that already has a checkpoint, you'll be asked if you want to resume
- Press Enter or type 'Y' to continue from where it stopped (with the same checkpoint interval)
- Type 'N' to start fresh and delete the old checkpoint
- Use `--force-restart` to automatically restart without asking

```bash
# Force restart without prompting (useful for scripts)
cargo run --release -- verify 196 -m 10000000 --force-restart

# Or manually resume from a specific checkpoint file
cargo run --release -- resume checkpoint_196.json
```

Example of resuming:
```
========================================
  CHECKPOINT FOUND!
========================================
Found existing checkpoint for number: 196
  Progress: 45.23%
  Iterations completed: 452300
  Iterations remaining: 547700
  Checkpoint interval: every 10000 iterations

Your choice (Y/n): Y

Resuming from checkpoint...

[Progress] Iteration: 453000       | Digits: 186312   | Time: 1.23s    | Speed: 36829 iter/s
[Progress] Iteration: 454000       | Digits: 186789   | Time: 2.45s    | Speed: 36945 iter/s
[Progress] Iteration: 455000       | Digits: 187234   | Time: 3.67s    | Speed: 37021 iter/s
[Progress] Iteration: 460000       | Digits: 189234   | Time: 12.45s   | Speed: 36944 iter/s | ✓ Checkpoint saved
```

Example output:
```
========================================
  LYCHREL NUMBER VERIFICATION
========================================
Number to verify: 196
Max iterations: 1000000
Progress interval: every 10000 iterations
Checkpoint interval: every 10000 iterations
========================================

[Progress] Iteration: 10000        | Digits: 4120     | Time: 0.45s    | Speed: 22222 iter/s | ✓ Checkpoint saved
[Progress] Iteration: 20000        | Digits: 8240     | Time: 1.12s    | Speed: 17857 iter/s | ✓ Checkpoint saved
...
[Progress] Iteration: 100000       | Digits: 41234    | Time: 12.34s   | Speed: 21456 iter/s | ✓ Checkpoint saved
...

========================================
  VERIFICATION COMPLETE
========================================
Iterations completed: 1000000
Total time: 45.234s

Result: NO PALINDROME FOUND
Status: This is LIKELY A LYCHREL NUMBER
Final number (413496 digits):
  81797...

Note: 1000000 iterations is not definitive proof.
      Consider running more iterations for stronger verification.
========================================
```

### Performance Benchmark

```bash
cargo run --release -- benchmark
```

## Available Options

### `test` Command
- `number`: The number to test (required)
- `--max-iterations` or `-m`: Maximum number of iterations (default: 10000)

### `search` Command
- `start`: Start of range (required)
- `end`: End of range (required)
- `--max-iterations` or `-m`: Maximum number of iterations (default: 10000)
- `--output` or `-o`: JSON output file for results
- `--no-parallel`: Disable parallel processing (enables checkpoints)
- `--checkpoint-interval` or `-c`: Save checkpoint every N numbers (default: 1000, use 0 to disable, only works with --no-parallel)
- `--checkpoint-file` or `-f`: Checkpoint file path (default: search_checkpoint_<start>_<end>.json)
- `--force-restart`: Ignore existing checkpoint and start fresh

### `verify` Command
- `number`: The number to verify (required)
- `--max-iterations` or `-m`: Maximum iterations (no default, must be specified)
- `--progress-interval` or `-p`: Show progress every N iterations (default: 10000)
- `--checkpoint-interval` or `-c`: Save checkpoint every N iterations (default: 10000, use 0 to disable)
- `--checkpoint-file` or `-f`: Checkpoint file path (default: checkpoint_<number>.json)
- `--force-restart`: Ignore existing checkpoint and start fresh

### `resume` Command
- `checkpoint_file`: Path to the checkpoint file to resume from (required)

### `hunt-record` Command
- `--config <FILE>`: Load configuration from JSON file (recommended). CLI options override file values
- `--min-digits`: Minimum number of digits to test (default: 23)
- `--target-iterations`: Minimum iterations to be considered a record (default: 289)
- `--max-iterations`: Maximum iterations before considering it a true Lychrel (default: 300)
- `--target-final-digits`: Minimum final digits for a record (default: 142)
- `--cache-size`: Thread cache size in entries (default: 1000000)
- `--warmup`: Warmup cache with 1-1M range before hunting (default: false)
- `--mode`: Generator mode - `sequential`, `random`, or `pattern` (default: sequential)
- `-c` or `--checkpoint-interval`: Save checkpoint every N numbers (default: 100000)
- `-f` or `--checkpoint-file`: Checkpoint file path (default: hunt_checkpoint.json)

**Note:** A record must reach a palindrome **within** the iteration window [target_iterations, max_iterations]. Numbers that don't reach a palindrome by max_iterations are considered potential true Lychrels, not records.

### `init-config` Command
Generate a default hunt configuration file.
- First argument: Output file path (default: hunt_config.json)

### `benchmark` Command
Runs a series of predefined performance tests.

## Tests

Run all tests:
```bash
cargo test
```

Run tests with detailed output:
```bash
cargo test -- --nocapture
```

## Code Architecture

```
src/
├── main.rs               # CLI interface with clap
├── lib.rs                # Public library exports
├── lychrel.rs            # Core algorithm (reverse, palindrome, iteration)
├── search.rs             # Search engine with parallelization
├── verify.rs             # Deep verification with progress tracking and checkpoint support
├── checkpoint.rs         # Checkpoint save/load for resumable computation
├── search_checkpoint.rs  # Checkpoints for search operations
├── thread_cache.rs       # Thread convergence detection cache
├── seed_generator.rs     # Smart seed generation for record hunting
├── record_hunt.rs        # Record hunting engine with optimizations
└── record_checkpoint.rs  # Specialized checkpoints for hunting

tests/
├── integration_tests.rs   # Integration tests
└── thread_cache_tests.rs  # Thread convergence detection tests
```

## Performance

The software uses:
- **num-bigint**: Support for arbitrarily large numbers
- **rayon**: Automatic parallelization across all CPU cores (search command)
- **Thread cache**: Convergence detection for 2-5x speedup in record hunting
- **Seed filtering**: Skip redundant numbers (~50% reduction in candidates)
- **--release** mode: Maximum compiler optimizations

Benchmarks:
- **Basic search:** 1-1000 range takes ~0.1-0.3 seconds (8 cores)
- **Record hunting:** ~200-300 numbers/second (23+ digits, with warmup and cache)
- **Cache effectiveness:** 60-70% hit rate after warmup, reducing computation significantly

## Known Lychrel Numbers

First Lychrel candidates < 10000:
- 196, 295, 394, 493, 592, 689, 691, 788, 790, 879, 887, 978, 986, 1495, 1497, 1585, 1587, 1675, 1677, 1765, 1767, 1855, 1857, 1945, 1947, ...

## References

- [The 196 Algorithm](https://www.p196.org/)
- [Lychrel Number (Wikipedia)](https://en.wikipedia.org/wiki/Lychrel_number)
- [Software Comparisons](https://www.p196.org/html/software_comparisons.html)

## License

This project is open source.