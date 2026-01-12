use crate::lychrel::IterationResult;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCheckpoint {
    pub start_range: BigUint,
    pub end_range: BigUint,
    pub current_number: BigUint,
    pub max_iterations: u32,
    pub numbers_tested: u64,
    pub potential_lychrel_found: Vec<BigUint>,
    pub checkpoint_interval: Option<u64>,
    pub checkpoint_file: Option<String>,
    pub elapsed_secs: f64,
    pub timestamp: String,
}

impl SearchCheckpoint {
    pub fn new(
        start_range: BigUint,
        end_range: BigUint,
        current_number: BigUint,
        max_iterations: u32,
        numbers_tested: u64,
        potential_lychrel: &[IterationResult],
        checkpoint_interval: Option<u64>,
        checkpoint_file: Option<String>,
        elapsed_secs: f64,
    ) -> Self {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let potential_lychrel_found = potential_lychrel
            .iter()
            .map(|r| r.start_number.clone())
            .collect();

        SearchCheckpoint {
            start_range,
            end_range,
            current_number,
            max_iterations,
            numbers_tested,
            potential_lychrel_found,
            checkpoint_interval,
            checkpoint_file,
            elapsed_secs,
            timestamp,
        }
    }

    pub fn save(&self, filepath: &str) -> std::io::Result<()> {
        crate::io_utils::save_to_file_str(self, filepath)
    }

    pub fn load(filepath: &str) -> std::io::Result<Self> {
        crate::io_utils::load_from_file_str(filepath)
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.end_range <= self.start_range {
            return 100.0;
        }

        let total = if let (Ok(start), Ok(end)) = (
            self.start_range.to_string().parse::<u64>(),
            self.end_range.to_string().parse::<u64>(),
        ) {
            end - start + 1
        } else {
            return 0.0;
        };

        (self.numbers_tested as f64 / total as f64) * 100.0
    }

    pub fn numbers_remaining(&self) -> u64 {
        if let (Ok(current), Ok(end)) = (
            self.current_number.to_string().parse::<u64>(),
            self.end_range.to_string().parse::<u64>(),
        ) {
            end.saturating_sub(current)
        } else {
            0
        }
    }
}
