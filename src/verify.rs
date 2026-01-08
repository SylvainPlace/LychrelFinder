use crate::checkpoint::Checkpoint;
use num_bigint::BigUint;
use std::time::Instant;

pub struct VerifyConfig {
    pub number: BigUint,
    pub max_iterations: u64,
    pub progress_interval: u64,
    pub checkpoint_interval: Option<u64>,
    pub checkpoint_file: Option<String>,
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

pub fn verify_lychrel_resumable<F>(
    config: VerifyConfig,
    mut progress_callback: F,
) -> VerifyResult
where
    F: FnMut(u64, &BigUint, std::time::Duration, bool),
{
    let start_time = Instant::now();
    let mut current = config.number.clone();
    let mut iteration_count: u64 = 0;
    let total_elapsed = 0.0f64;

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
    let mut last_checkpoint = 0u64;

    while iteration_count < config.max_iterations {
        let reversed = reverse_number(&current);
        current = current + reversed;
        iteration_count += 1;

        if is_palindrome(&current) {
            progress_callback(iteration_count, &current, start_time.elapsed(), false);
            return VerifyResult {
                start_number: config.number.clone(),
                is_palindrome: true,
                iterations_completed: iteration_count,
                final_number: Some(current),
                is_potential_lychrel: false,
                total_time: start_time.elapsed(),
            };
        }

        let should_save_checkpoint = if let Some(checkpoint_interval) = config.checkpoint_interval {
            iteration_count - last_checkpoint >= checkpoint_interval
        } else {
            false
        };

        let should_show_progress = iteration_count - last_progress_report >= config.progress_interval;

        if should_save_checkpoint {
            if let Some(ref checkpoint_file) = config.checkpoint_file {
                let checkpoint = Checkpoint::new(
                    config.number.clone(),
                    current.clone(),
                    iteration_count,
                    config.max_iterations,
                    config.progress_interval,
                    config.checkpoint_interval,
                    total_elapsed + start_time.elapsed().as_secs_f64(),
                );
                
                if let Err(e) = checkpoint.save(checkpoint_file) {
                    eprintln!("Warning: Failed to save checkpoint: {}", e);
                } else {
                    progress_callback(iteration_count, &current, start_time.elapsed(), true);
                    last_checkpoint = iteration_count;
                    if should_show_progress {
                        last_progress_report = iteration_count;
                    }
                }
            }
        } else if should_show_progress {
            progress_callback(iteration_count, &current, start_time.elapsed(), false);
            last_progress_report = iteration_count;
        }
    }

    progress_callback(iteration_count, &current, start_time.elapsed(), false);

    VerifyResult {
        start_number: config.number,
        is_palindrome: false,
        iterations_completed: iteration_count,
        final_number: Some(current),
        is_potential_lychrel: true,
        total_time: start_time.elapsed(),
    }
}

pub fn resume_from_checkpoint<F>(
    checkpoint: Checkpoint,
    checkpoint_file: Option<String>,
    checkpoint_interval: Option<u64>,
    mut progress_callback: F,
) -> VerifyResult
where
    F: FnMut(u64, &BigUint, std::time::Duration, bool),
{
    let start_time = Instant::now();
    let mut current = checkpoint.current_number.clone();
    let mut iteration_count = checkpoint.iterations_completed;
    let base_elapsed = checkpoint.elapsed_secs;

    let mut last_progress_report = iteration_count;
    let mut last_checkpoint = iteration_count;

    while iteration_count < checkpoint.max_iterations {
        let reversed = reverse_number(&current);
        current = current + reversed;
        iteration_count += 1;

        if is_palindrome(&current) {
            progress_callback(iteration_count, &current, start_time.elapsed(), false);
            let total_duration = std::time::Duration::from_secs_f64(
                base_elapsed + start_time.elapsed().as_secs_f64()
            );
            return VerifyResult {
                start_number: checkpoint.start_number,
                is_palindrome: true,
                iterations_completed: iteration_count,
                final_number: Some(current),
                is_potential_lychrel: false,
                total_time: total_duration,
            };
        }

        // Save checkpoint periodically
        let should_save_checkpoint = if let Some(interval) = checkpoint_interval {
            iteration_count - last_checkpoint >= interval
        } else {
            false
        };

        let should_show_progress = iteration_count - last_progress_report >= checkpoint.progress_interval;

        if should_save_checkpoint {
            if let Some(ref file) = checkpoint_file {
                let new_checkpoint = Checkpoint::new(
                    checkpoint.start_number.clone(),
                    current.clone(),
                    iteration_count,
                    checkpoint.max_iterations,
                    checkpoint.progress_interval,
                    checkpoint_interval,
                    base_elapsed + start_time.elapsed().as_secs_f64(),
                );
                
                if let Err(e) = new_checkpoint.save(file) {
                    eprintln!("Warning: Failed to save checkpoint: {}", e);
                } else {
                    progress_callback(iteration_count, &current, start_time.elapsed(), true);
                    last_checkpoint = iteration_count;
                    if should_show_progress {
                        last_progress_report = iteration_count;
                    }
                }
            }
        } else if should_show_progress {
            progress_callback(iteration_count, &current, start_time.elapsed(), false);
            last_progress_report = iteration_count;
        }
    }

    progress_callback(iteration_count, &current, start_time.elapsed(), false);
    let total_duration = std::time::Duration::from_secs_f64(
        base_elapsed + start_time.elapsed().as_secs_f64()
    );

    VerifyResult {
        start_number: checkpoint.start_number,
        is_palindrome: false,
        iterations_completed: iteration_count,
        final_number: Some(current),
        is_potential_lychrel: true,
        total_time: total_duration,
    }
}

pub fn resume_from_checkpoint_with_config<F>(
    checkpoint: Checkpoint,
    checkpoint_file: String,
    checkpoint_interval: u64,
    progress_callback: F,
) -> VerifyResult
where
    F: FnMut(u64, &BigUint, std::time::Duration, bool),
{
    resume_from_checkpoint(checkpoint, Some(checkpoint_file), Some(checkpoint_interval), progress_callback)
}
