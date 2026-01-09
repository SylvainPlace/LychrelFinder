use crate::lychrel::IterationResult;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

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

    pub fn save(&self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(filepath)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load(filepath: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !Path::new(filepath).exists() {
            return Err("Checkpoint file not found".into());
        }

        let mut file = File::open(filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let checkpoint: SearchCheckpoint = serde_json::from_str(&contents)?;
        Ok(checkpoint)
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
            if end > current {
                end - current
            } else {
                0
            }
        } else {
            0
        }
    }
}
