use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub start_number: BigUint,
    pub current_number: BigUint,
    pub iterations_completed: u64,
    pub max_iterations: u64,
    pub progress_interval: u64,
    pub checkpoint_interval: Option<u64>,
    pub elapsed_secs: f64,
    pub timestamp: String,
}

impl Checkpoint {
    pub fn new(
        start_number: BigUint,
        current_number: BigUint,
        iterations_completed: u64,
        max_iterations: u64,
        progress_interval: u64,
        checkpoint_interval: Option<u64>,
        elapsed_secs: f64,
    ) -> Self {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        Checkpoint {
            start_number,
            current_number,
            iterations_completed,
            max_iterations,
            progress_interval,
            checkpoint_interval,
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
        let checkpoint: Checkpoint = serde_json::from_str(&contents)?;
        Ok(checkpoint)
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.max_iterations == 0 {
            0.0
        } else {
            (self.iterations_completed as f64 / self.max_iterations as f64) * 100.0
        }
    }

    pub fn digit_count(&self) -> usize {
        self.current_number.to_string().len()
    }

    pub fn iterations_remaining(&self) -> u64 {
        if self.iterations_completed >= self.max_iterations {
            0
        } else {
            self.max_iterations - self.iterations_completed
        }
    }
}
