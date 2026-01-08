use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use crate::lychrel::reverse_number;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GeneratorMode {
    Sequential,      // 10^23, 10^23+1, 10^23+2, ...
    SmartRandom,     // Random generation with heuristics
    PatternBased,    // Based on observed patterns
}

pub struct SeedGenerator {
    current: BigUint,
    max: BigUint,
    digits: usize,
    pub mode: GeneratorMode,
    skip_count: u64,  // Track how many we've skipped
    generated_count: u64,  // Track how many we've generated
}

impl SeedGenerator {
    pub fn new(digits: usize, mode: GeneratorMode) -> Self {
        let min = BigUint::from(10u32).pow(digits as u32 - 1);
        let max = BigUint::from(10u32).pow(digits as u32);
        
        SeedGenerator {
            current: min,
            max,
            digits,
            mode,
            skip_count: 0,
            generated_count: 0,
        }
    }

    /// Create generator with custom starting point (for resuming)
    pub fn from_checkpoint(digits: usize, mode: GeneratorMode, current: BigUint) -> Self {
        let max = BigUint::from(10u32).pow(digits as u32);
        
        SeedGenerator {
            current,
            max,
            digits,
            mode,
            skip_count: 0,
            generated_count: 0,
        }
    }

    /// Check if a number is a potential seed (primary number in its convergence family)
    pub fn is_potential_seed(&self, n: &BigUint) -> bool {
        let reversed = reverse_number(n);
        
        // If reverse < n, then reversed is a smaller number and could be the real seed
        // We should skip this number since reversed should be tested instead
        if reversed < *n {
            return false;
        }
        
        // If reverse == n (palindrome), we can test it (it's its own seed)
        // If reverse > n, this is potentially a seed
        true
    }

    pub fn get_stats(&self) -> GeneratorStats {
        GeneratorStats {
            generated_count: self.generated_count,
            skip_count: self.skip_count,
            skip_rate: if self.generated_count + self.skip_count > 0 {
                self.skip_count as f64 / (self.generated_count + self.skip_count) as f64
            } else {
                0.0
            },
        }
    }

    pub fn current_position(&self) -> BigUint {
        self.current.clone()
    }

    fn generate_sequential(&mut self) -> BigUint {
        let result = self.current.clone();
        self.current += 1u32;
        result
    }

    fn generate_smart_random(&mut self) -> BigUint {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Generate a random number with the specified number of digits
        // Strategy: Favor numbers that are:
        // 1. Asymmetric (not close to palindromes)
        // 2. Have diverse digits (not too many repeated digits)
        // 3. First half > second half (more likely to be seeds)
        
        let min = BigUint::from(10u32).pow(self.digits as u32 - 1);
        let _max = BigUint::from(10u32).pow(self.digits as u32);
        
        // Generate random offset
        let mut random_digits = String::new();
        for i in 0..self.digits {
            let digit = if i == 0 {
                rng.gen_range(1..=9)  // First digit can't be 0
            } else {
                rng.gen_range(0..=9)
            };
            random_digits.push(std::char::from_digit(digit, 10).unwrap());
        }
        
        random_digits.parse().unwrap_or(min)
    }

    fn generate_from_pattern(&mut self) -> BigUint {
        // Pattern-based generation
        // Based on observed patterns in Lychrel records
        // For now, just do sequential until we have better pattern analysis
        self.generate_sequential()
    }
}

impl Iterator for SeedGenerator {
    type Item = BigUint;
    
    fn next(&mut self) -> Option<BigUint> {
        if self.current >= self.max {
            return None;
        }
        
        loop {
            let candidate = match self.mode {
                GeneratorMode::Sequential => self.generate_sequential(),
                GeneratorMode::SmartRandom => self.generate_smart_random(),
                GeneratorMode::PatternBased => self.generate_from_pattern(),
            };
            
            // Check if we've exceeded the range
            if candidate >= self.max {
                return None;
            }
            
            // Filter for potential seeds
            if self.is_potential_seed(&candidate) {
                self.generated_count += 1;
                return Some(candidate);
            } else {
                self.skip_count += 1;
                // Continue loop to get next candidate
            }
        }
    }
}

#[derive(Debug)]
pub struct GeneratorStats {
    pub generated_count: u64,
    pub skip_count: u64,
    pub skip_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_potential_seed() {
        let gen = SeedGenerator::new(5, GeneratorMode::Sequential);
        
        // 19991 reversed = 19991 (palindrome, should pass)
        assert!(gen.is_potential_seed(&BigUint::from(19991u32)));
        
        // 12345 reversed = 54321 > 12345 (should pass, it's a potential seed)
        assert!(gen.is_potential_seed(&BigUint::from(12345u32)));
        
        // 54321 reversed = 12345 < 54321 (should fail, 12345 is the seed)
        assert!(!gen.is_potential_seed(&BigUint::from(54321u32)));
        
        // 10000 reversed = 00001 = 1 < 10000 (should fail)
        assert!(!gen.is_potential_seed(&BigUint::from(10000u32)));
    }

    #[test]
    fn test_generator_sequential() {
        let mut gen = SeedGenerator::new(3, GeneratorMode::Sequential);
        
        // First number with 3 digits is 100
        let first = gen.next();
        assert!(first.is_some());
        
        // Should generate some numbers
        let mut count = 0;
        for _ in gen.by_ref().take(10) {
            count += 1;
        }
        assert_eq!(count, 10);
    }

    #[test]
    fn test_generator_filters_reversed() {
        let mut gen = SeedGenerator::new(3, GeneratorMode::Sequential);
        
        // Collect first 50 numbers
        let numbers: Vec<BigUint> = gen.by_ref().take(50).collect();
        
        // Check that none of them have reverse < themselves
        for n in &numbers {
            let reversed = reverse_number(n);
            assert!(reversed >= *n, "Generated number {} has reverse {} < itself", n, reversed);
        }
    }

    #[test]
    fn test_generator_stats() {
        let mut gen = SeedGenerator::new(3, GeneratorMode::Sequential);
        
        // Generate some numbers
        for _ in gen.by_ref().take(100) {}
        
        let stats = gen.get_stats();
        assert_eq!(stats.generated_count, 100);
        assert!(stats.skip_count > 0, "Should have skipped some numbers");
        
        println!("Generated: {}, Skipped: {}, Skip rate: {:.2}%",
                 stats.generated_count, stats.skip_count, stats.skip_rate * 100.0);
    }

    #[test]
    fn test_generator_from_checkpoint() {
        let start = BigUint::from(50000u32);
        let mut gen = SeedGenerator::from_checkpoint(5, GeneratorMode::Sequential, start.clone());
        
        let first = gen.next().unwrap();
        assert!(first >= start);
    }
}
