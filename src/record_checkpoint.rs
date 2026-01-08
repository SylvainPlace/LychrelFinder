use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::record_hunt::{HuntStatistics, RecordCandidate};
use crate::seed_generator::GeneratorMode;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordHuntCheckpoint {
    pub generator_state: GeneratorState,
    pub statistics: CheckpointStatistics,
    pub thread_cache_file: String,
    pub timestamp: String,
    pub config: CheckpointConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratorState {
    pub current_value: String,  // BigUint as String for serialization
    pub digits: usize,
    pub mode: GeneratorMode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointStatistics {
    pub numbers_tested: u64,
    pub seeds_tested: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub best_iterations_found: u32,
    pub best_digits_found: usize,
    pub candidates_above_200: Vec<RecordCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointConfig {
    pub min_digits: usize,
    pub target_iterations: u32,
    pub max_iterations: u32,
    pub target_final_digits: usize,
    pub cache_size: usize,
    pub checkpoint_interval: u64,
}

impl RecordHuntCheckpoint {
    pub fn new(
        current_position: &BigUint,
        digits: usize,
        mode: GeneratorMode,
        stats: &HuntStatistics,
        cache_file: &str,
        config: CheckpointConfig,
    ) -> Self {
        RecordHuntCheckpoint {
            generator_state: GeneratorState {
                current_value: current_position.to_string(),
                digits,
                mode,
            },
            statistics: CheckpointStatistics {
                numbers_tested: stats.numbers_tested,
                seeds_tested: stats.seeds_tested,
                cache_hits: stats.cache_hits,
                cache_misses: stats.cache_misses,
                best_iterations_found: stats.best_iterations_found,
                best_digits_found: stats.best_digits_found,
                candidates_above_200: stats.candidates_above_200.clone(),
            },
            thread_cache_file: cache_file.to_string(),
            timestamp: chrono::Local::now().to_string(),
            config,
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let checkpoint = serde_json::from_reader(reader)?;
        Ok(checkpoint)
    }

    pub fn get_current_position(&self) -> Result<BigUint, num_bigint::ParseBigIntError> {
        self.generator_state.current_value.parse()
    }

    pub fn display_info(&self) {
        println!("📂 Checkpoint Information");
        println!("  Timestamp: {}", self.timestamp);
        println!("  Numbers tested: {}", self.statistics.numbers_tested);
        println!("  Seeds tested: {}", self.statistics.seeds_tested);
        println!("  Best iterations: {}", self.statistics.best_iterations_found);
        println!("  Best digits: {}", self.statistics.best_digits_found);
        println!("  Candidates (200+): {}", self.statistics.candidates_above_200.len());
        println!("  Current position: {}", self.generator_state.current_value);
        println!("  Cache file: {}", self.thread_cache_file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record_hunt::HuntStatistics;
    use std::time::Instant;

    #[test]
    fn test_checkpoint_save_load() {
        let temp_file = "test_checkpoint_temp.json";
        
        let stats = HuntStatistics {
            numbers_tested: 50000,
            seeds_tested: 25000,
            cache_hits: 10000,
            cache_misses: 15000,
            best_iterations_found: 150,
            best_digits_found: 75,
            start_time: Instant::now(),
            candidates_above_200: vec![],
        };
        
        let config = CheckpointConfig {
            min_digits: 23,
            target_iterations: 289,
            max_iterations: 300,
            target_final_digits: 142,
            cache_size: 1000000,
            checkpoint_interval: 100000,
        };
        
        let checkpoint = RecordHuntCheckpoint::new(
            &BigUint::from(123456789u64),
            23,
            GeneratorMode::Sequential,
            &stats,
            "cache.json",
            config,
        );
        
        // Save
        checkpoint.save(Path::new(temp_file)).unwrap();
        
        // Load
        let loaded = RecordHuntCheckpoint::load(Path::new(temp_file)).unwrap();
        
        assert_eq!(loaded.statistics.numbers_tested, 50000);
        assert_eq!(loaded.statistics.seeds_tested, 25000);
        assert_eq!(loaded.generator_state.digits, 23);
        
        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_get_current_position() {
        let stats = HuntStatistics {
            numbers_tested: 0,
            seeds_tested: 0,
            cache_hits: 0,
            cache_misses: 0,
            best_iterations_found: 0,
            best_digits_found: 0,
            start_time: Instant::now(),
            candidates_above_200: vec![],
        };
        
        let config = CheckpointConfig {
            min_digits: 20,
            target_iterations: 200,
            max_iterations: 250,
            target_final_digits: 100,
            cache_size: 10000,
            checkpoint_interval: 10000,
        };
        
        let position = BigUint::from(99999999999999999999u128);
        let checkpoint = RecordHuntCheckpoint::new(
            &position,
            20,
            GeneratorMode::Sequential,
            &stats,
            "cache.json",
            config,
        );
        
        let loaded_position = checkpoint.get_current_position().unwrap();
        assert_eq!(loaded_position, position);
    }
}
