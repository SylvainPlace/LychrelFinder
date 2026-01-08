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
└── search.rs     # Search engine with parallelization

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