use num_bigint::BigUint;
use num_traits::Zero;
use std::time::Instant;

pub struct VerifyConfig {
    pub number: BigUint,
    pub max_iterations: u64,
    pub progress_interval: u64,
}

pub struct VerifyResult {
    pub start_number: BigUint,
    pub is_palindrome: bool,
    pub iterations_completed: u64,
    pub final_number: Option<BigUint>,
    pub is_potential_lychrel: bool,
    pub total_time: std::time::Duration,
}

fn reverse_number(n: &BigUint) -> BigUint {
    let s = n.to_string();
    let reversed = s.chars().rev().collect::<String>();
    reversed.parse().unwrap()
}

fn is_palindrome(n: &BigUint) -> bool {
    let s = n.to_string();
    let reversed: String = s.chars().rev().collect();
    s == reversed
}

pub fn verify_lychrel<F>(config: VerifyConfig, mut progress_callback: F) -> VerifyResult
where
    F: FnMut(u64, &BigUint, std::time::Duration),
{
    let start_time = Instant::now();
    let mut current = config.number.clone();
    let mut iteration_count: u64 = 0;

    if is_palindrome(&current) {
        return VerifyResult {
            start_number: config.number,
            is_palindrome: true,
            iterations_completed: 0,
            final_number: Some(current),
            is_potential_lychrel: false,
            total_time: start_time.elapsed(),
        };
    }

    let mut last_progress_report = 0u64;

    while iteration_count < config.max_iterations {
        let reversed = reverse_number(&current);
        current = current + reversed;
        iteration_count += 1;

        if is_palindrome(&current) {
            progress_callback(iteration_count, &current, start_time.elapsed());
            return VerifyResult {
                start_number: config.number,
                is_palindrome: true,
                iterations_completed: iteration_count,
                final_number: Some(current),
                is_potential_lychrel: false,
                total_time: start_time.elapsed(),
            };
        }

        if iteration_count - last_progress_report >= config.progress_interval {
            progress_callback(iteration_count, &current, start_time.elapsed());
            last_progress_report = iteration_count;
        }
    }

    progress_callback(iteration_count, &current, start_time.elapsed());

    VerifyResult {
        start_number: config.number,
        is_palindrome: false,
        iterations_completed: iteration_count,
        final_number: Some(current),
        is_potential_lychrel: true,
        total_time: start_time.elapsed(),
    }
}
