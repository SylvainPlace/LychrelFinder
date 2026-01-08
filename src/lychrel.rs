use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    pub start_number: BigUint,
    pub is_palindrome: bool,
    pub iterations: u32,
    pub final_number: Option<BigUint>,
    pub is_potential_lychrel: bool,
}

pub fn reverse_number(n: &BigUint) -> BigUint {
    let s = n.to_string();
    let reversed = s.chars().rev().collect::<String>();
    reversed.parse().unwrap()
}

pub fn is_palindrome(n: &BigUint) -> bool {
    let s = n.to_string();
    let reversed: String = s.chars().rev().collect();
    s == reversed
}

pub fn lychrel_iteration(start: BigUint, max_iterations: u32) -> IterationResult {
    let mut current = start.clone();
    let mut iteration_count = 0;

    if is_palindrome(&current) {
        return IterationResult {
            start_number: start,
            is_palindrome: true,
            iterations: 0,
            final_number: Some(current),
            is_potential_lychrel: false,
        };
    }

    while iteration_count < max_iterations {
        let reversed = reverse_number(&current);
        current = current + reversed;
        iteration_count += 1;

        if is_palindrome(&current) {
            return IterationResult {
                start_number: start,
                is_palindrome: true,
                iterations: iteration_count,
                final_number: Some(current),
                is_potential_lychrel: false,
            };
        }
    }

    IterationResult {
        start_number: start,
        is_palindrome: false,
        iterations: iteration_count,
        final_number: Some(current),
        is_potential_lychrel: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_number() {
        assert_eq!(reverse_number(&BigUint::from(123u32)), BigUint::from(321u32));
        assert_eq!(reverse_number(&BigUint::from(100u32)), BigUint::from(1u32));
        assert_eq!(reverse_number(&BigUint::from(505u32)), BigUint::from(505u32));
    }

    #[test]
    fn test_is_palindrome() {
        assert!(is_palindrome(&BigUint::from(121u32)));
        assert!(is_palindrome(&BigUint::from(1u32)));
        assert!(!is_palindrome(&BigUint::from(123u32)));
        assert!(is_palindrome(&BigUint::from(12321u32)));
    }

    #[test]
    fn test_lychrel_iteration_simple() {
        let result = lychrel_iteration(BigUint::from(89u32), 100);
        assert!(result.is_palindrome);
        assert_eq!(result.iterations, 24);
    }

    #[test]
    fn test_lychrel_iteration_196() {
        let result = lychrel_iteration(BigUint::from(196u32), 100);
        assert!(!result.is_palindrome);
        assert!(result.is_potential_lychrel);
        assert_eq!(result.iterations, 100);
    }

    #[test]
    fn test_already_palindrome() {
        let result = lychrel_iteration(BigUint::from(121u32), 100);
        assert!(result.is_palindrome);
        assert_eq!(result.iterations, 0);
    }
}
