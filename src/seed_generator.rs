use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GeneratorMode {
    Sequential,   // 10^23, 10^23+1, 10^23+2, ...
    SmartRandom,  // Random generation with heuristics
    PatternBased, // Based on observed patterns
}

pub struct SeedGenerator {
    current: BigUint,
    max: BigUint,
    digits: usize,
    p10_max: BigUint,
    pub mode: GeneratorMode,
    skip_count: u64,
    generated_count: u64,
}

impl SeedGenerator {
    /// Create a new seed generator
    pub fn new(digits: usize, mode: GeneratorMode) -> Self {
        let min = BigUint::from(10u32).pow(digits as u32 - 1);
        let p10_max = min.clone();
        let max = BigUint::from(10u32).pow(digits as u32);

        SeedGenerator {
            current: min,
            max,
            digits,
            p10_max,
            mode,
            skip_count: 0,
            generated_count: 0,
        }
    }

    /// Create generator with custom starting point (for resuming)
    pub fn from_checkpoint(digits: usize, mode: GeneratorMode, current: BigUint) -> Self {
        let p10_max = BigUint::from(10u32).pow(digits as u32 - 1);
        let max = BigUint::from(10u32).pow(digits as u32);

        SeedGenerator {
            current,
            max,
            digits,
            p10_max,
            mode,
            skip_count: 0,
            generated_count: 0,
        }
    }

    pub fn current_position(&self) -> BigUint {
        self.current.clone()
    }

    pub fn get_stats(&self) -> GeneratorStats {
        let total = self.generated_count + self.skip_count;
        let skip_rate = if total > 0 {
            self.skip_count as f64 / total as f64
        } else {
            0.0
        };

        GeneratorStats {
            generated_count: self.generated_count,
            skip_count: self.skip_count,
            skip_rate,
        }
    }

    fn generate_sequential(&mut self) -> BigUint {
        let result = self.current.clone();
        self.current += 1u32;
        result
    }

    pub fn current_p10_max(&self) -> BigUint {
        self.p10_max.clone()
    }

    fn generate_smart_random(&mut self) -> BigUint {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let min = BigUint::from(10u32).pow(self.digits as u32 - 1);

        let mut random_digits = String::new();
        for i in 0..self.digits {
            let digit = if i == 0 {
                rng.gen_range(1..=9)
            } else {
                rng.gen_range(0..=9)
            };
            random_digits.push(std::char::from_digit(digit, 10).unwrap());
        }

        random_digits.parse().unwrap_or(min)
    }

    fn generate_from_pattern(&mut self) -> BigUint {
        self.generate_sequential()
    }

    pub fn next_raw_batch(&mut self, size: usize) -> Vec<BigUint> {
        let mut batch = Vec::with_capacity(size);
        for _ in 0..size {
            if self.current >= self.max {
                break;
            }
            let candidate = match self.mode {
                GeneratorMode::Sequential => self.generate_sequential(),
                GeneratorMode::SmartRandom => self.generate_smart_random(),
                GeneratorMode::PatternBased => self.generate_from_pattern(),
            };
            batch.push(candidate);
        }
        batch
    }
}

/// Free function to check if a number is a potential seed
pub fn is_potential_seed(n: &BigUint, p10_max: Option<&BigUint>) -> bool {
    use num_traits::ToPrimitive;

    // Fast arithmetic path
    if let Some(p10) = p10_max {
        let last = (n % 10u32).to_u32().unwrap();
        let first = (n / p10).to_u32().unwrap();
        if last < first {
            return false;
        }
        if last > first {
            return true;
        }
    }

    let digits = n.to_radix_le(10);
    let len = digits.len();
    for i in 0..len / 2 {
        let left = digits[len - 1 - i]; // Most significant
        let right = digits[i]; // Least significant
        if right < left {
            return false; // reverse(n) < n
        }
        if right > left {
            return true; // reverse(n) > n
        }
    }
    true // Palindrome
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

            if candidate >= self.max {
                return None;
            }

            if is_potential_seed(&candidate, Some(&self.p10_max)) {
                self.generated_count += 1;
                return Some(candidate);
            } else {
                self.skip_count += 1;
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
    use crate::lychrel::reverse_number;

    #[test]
    fn test_is_potential_seed() {
        let p10_4 = BigUint::from(10000u32);

        // 19991 reversed = 19991 (palindrome, should pass)
        assert!(is_potential_seed(&BigUint::from(19991u32), Some(&p10_4)));

        // 12345 reversed = 54321 > 12345 (should pass)
        assert!(is_potential_seed(&BigUint::from(12345u32), Some(&p10_4)));

        // 54321 reversed = 12345 < 54321 (should fail)
        assert!(!is_potential_seed(&BigUint::from(54321u32), Some(&p10_4)));

        // 10000 reversed = 1 < 10000 (should fail)
        assert!(!is_potential_seed(&BigUint::from(10000u32), Some(&p10_4)));
    }

    #[test]
    fn test_generator_sequential() {
        let mut gen = SeedGenerator::new(3, GeneratorMode::Sequential);
        let first = gen.next();
        assert!(first.is_some());
        let mut count = 0;
        for _ in gen.by_ref().take(10) {
            count += 1;
        }
        assert_eq!(count, 10);
    }

    #[test]
    fn test_generator_filters_reversed() {
        let mut gen = SeedGenerator::new(3, GeneratorMode::Sequential);
        let numbers: Vec<BigUint> = gen.by_ref().take(50).collect();
        for n in &numbers {
            let reversed = reverse_number(n);
            assert!(reversed >= *n);
        }
    }
}
