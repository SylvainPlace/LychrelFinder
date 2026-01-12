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

#[derive(Debug, Clone, Default)]
pub struct SearchCheckpointBuilder {
    pub start_range: Option<BigUint>,
    pub end_range: Option<BigUint>,
    pub current_number: Option<BigUint>,
    pub max_iterations: Option<u32>,
    pub numbers_tested: Option<u64>,
    pub potential_lychrel: Option<Vec<IterationResult>>,
    pub checkpoint_interval: Option<u64>,
    pub checkpoint_file: Option<String>,
    pub elapsed_secs: Option<f64>,
}

impl SearchCheckpointBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_range(mut self, value: BigUint) -> Self {
        self.start_range = Some(value);
        self
    }

    pub fn end_range(mut self, value: BigUint) -> Self {
        self.end_range = Some(value);
        self
    }

    pub fn current_number(mut self, value: BigUint) -> Self {
        self.current_number = Some(value);
        self
    }

    pub fn max_iterations(mut self, value: u32) -> Self {
        self.max_iterations = Some(value);
        self
    }

    pub fn numbers_tested(mut self, value: u64) -> Self {
        self.numbers_tested = Some(value);
        self
    }

    pub fn potential_lychrel(mut self, value: Vec<IterationResult>) -> Self {
        self.potential_lychrel = Some(value);
        self
    }

    pub fn checkpoint_interval(mut self, value: Option<u64>) -> Self {
        self.checkpoint_interval = value;
        self
    }

    pub fn checkpoint_file(mut self, value: Option<String>) -> Self {
        self.checkpoint_file = value;
        self
    }

    pub fn elapsed_secs(mut self, value: f64) -> Self {
        self.elapsed_secs = Some(value);
        self
    }

    pub fn build(self) -> SearchCheckpoint {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let potential_lychrel_found = self
            .potential_lychrel
            .unwrap_or_default()
            .iter()
            .map(|r| r.start_number.clone())
            .collect();

        SearchCheckpoint {
            start_range: self.start_range.unwrap_or_default(),
            end_range: self.end_range.unwrap_or_default(),
            current_number: self.current_number.unwrap_or_default(),
            max_iterations: self.max_iterations.unwrap_or_default(),
            numbers_tested: self.numbers_tested.unwrap_or_default(),
            potential_lychrel_found,
            checkpoint_interval: self.checkpoint_interval,
            checkpoint_file: self.checkpoint_file,
            elapsed_secs: self.elapsed_secs.unwrap_or_default(),
            timestamp,
        }
    }
}

impl SearchCheckpoint {
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
