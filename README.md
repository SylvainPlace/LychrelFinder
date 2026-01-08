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
- ✅ Search ranges of numbers
- ✅ Deep verification mode with millions of iterations and live progress tracking
- ✅ Support for arbitrarily large numbers (BigInt arithmetic)
- ✅ Parallelized processing for optimal performance
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

```bash
# Search from 1 to 10000
cargo run --release -- search 1 10000 --max-iterations 1000

# With JSON export
cargo run --release -- search 1 10000 --output results.json

# Without parallelization
cargo run --release -- search 1 1000 --no-parallel
```

### Verify a Lychrel Candidate (Deep Testing)

For extensive verification with millions of iterations and live progress tracking:

```bash
# Test 196 with 1 million iterations, showing progress every 10000 iterations
cargo run --release -- verify 196 --max-iterations 1000000 --progress-interval 10000

# Test with 10 million iterations
cargo run --release -- verify 196 -m 10000000 -p 50000
```

Example output:
```
========================================
  LYCHREL NUMBER VERIFICATION
========================================
Number to verify: 196
Max iterations: 1000000
Progress interval: every 10000 iterations
========================================

[Progress] Iteration: 10000        | Digits: 4120     | Time: 0.45s    | Speed: 22222 iter/s
[Progress] Iteration: 20000        | Digits: 8240     | Time: 1.12s    | Speed: 17857 iter/s
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
- `--no-parallel`: Disable parallel processing

### `verify` Command
- `number`: The number to verify (required)
- `--max-iterations` or `-m`: Maximum iterations (no default, must be specified)
- `--progress-interval` or `-p`: Show progress every N iterations (default: 10000)

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
├── main.rs       # CLI interface with clap
├── lib.rs        # Public library exports
├── lychrel.rs    # Core algorithm (reverse, palindrome, iteration)
├── search.rs     # Search engine with parallelization
└── verify.rs     # Deep verification with progress tracking

tests/
└── integration_tests.rs  # Integration tests
```

## Performance

The software uses:
- **num-bigint**: Support for arbitrarily large numbers
- **rayon**: Automatic parallelization across all CPU cores
- **--release** mode: Maximum compiler optimizations

On a modern processor (8 cores), searching the range 1 to 1000 takes approximately 0.1-0.3 seconds.

## Known Lychrel Numbers

First Lychrel candidates < 10000:
- 196, 295, 394, 493, 592, 689, 691, 788, 790, 879, 887, 978, 986, 1495, 1497, 1585, 1587, 1675, 1677, 1765, 1767, 1855, 1857, 1945, 1947, ...

## References

- [The 196 Algorithm](https://www.p196.org/)
- [Lychrel Number (Wikipedia)](https://en.wikipedia.org/wiki/Lychrel_number)
- [Software Comparisons](https://www.p196.org/html/software_comparisons.html)

## License

This project is open source.