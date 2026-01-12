use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

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
    /// Create a new checkpoint
    ///
    /// # Arguments
    ///
    /// * `start_number` - The original number being tested
    /// * `current_number` - Current value after iterations
    /// * `iterations_completed` - Number of iterations performed so far
    /// * `max_iterations` - Target number of iterations
    /// * `progress_interval` - How often to show progress updates
    /// * `checkpoint_interval` - How often to save checkpoints
    /// * `elapsed_secs` - Time elapsed in seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use lychrel_finder::Checkpoint;
    /// use num_bigint::BigUint;
    ///
    /// let checkpoint = Checkpoint::new(
    ///     BigUint::from(196u32),
    ///     BigUint::from(887u32),
    ///     1,
    ///     1000,
    ///     100,
    ///     Some(100),
    ///     1.5,
    /// );
    /// ```
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

    /// Save checkpoint to a file
    ///
    /// Serializes checkpoint to JSON and writes it to the specified file path.
    ///
    /// # Arguments
    ///
    /// * `filepath` - Path where the checkpoint file should be saved
    ///
    /// # Returns
    ///
    /// `Ok(())` if save succeeded, `Err(std::io::Error)` if it failed
    ///
    /// # Examples
    ///
    /// ```
    /// use lychrel_finder::Checkpoint;
    /// use num_bigint::BigUint;
    ///
    /// let checkpoint = Checkpoint::new(
    ///     BigUint::from(196u32),
    ///     BigUint::from(887u32),
    ///     1,
    ///     1000,
    ///     100,
    ///     Some(100),
    ///     1.5,
    /// );
    /// checkpoint.save("checkpoint.json").unwrap();
    /// ```
    pub fn save(&self, filepath: &str) -> std::io::Result<()> {
        crate::io_utils::save_to_file_str(self, filepath)
    }

    /// Load checkpoint from a file
    ///
    /// Reads a JSON file and deserializes it into a Checkpoint struct.
    ///
    /// # Arguments
    ///
    /// * `filepath` - Path to the checkpoint file to load
    ///
    /// # Returns
    ///
    /// `Ok(Checkpoint)` if load succeeded, `Err(std::io::Error)` if it failed
    ///
    /// # Examples
    ///
    /// ```
    /// use lychrel_finder::Checkpoint;
    ///
    /// let checkpoint = Checkpoint::load("checkpoint.json").unwrap();
    /// ```
    pub fn load(filepath: &str) -> std::io::Result<Self> {
        crate::io_utils::load_from_file_str(filepath)
    }

    /// Calculate progress as a percentage
    ///
    /// Returns the percentage of completion based on iterations completed
    /// versus max iterations.
    ///
    /// # Returns
    ///
    /// A float between 0.0 and 100.0 representing the percentage complete
    pub fn progress_percentage(&self) -> f64 {
        if self.max_iterations == 0 {
            0.0
        } else {
            (self.iterations_completed as f64 / self.max_iterations as f64) * 100.0
        }
    }

    /// Get the number of digits in the current number
    ///
    /// # Returns
    ///
    /// The number of decimal digits in the current number
    pub fn digit_count(&self) -> usize {
        self.current_number.to_string().len()
    }

    /// Calculate remaining iterations
    ///
    /// Returns the number of iterations still needed to reach max_iterations.
    /// Uses saturating_sub to avoid underflow.
    ///
    /// # Returns
    ///
    /// The number of iterations remaining (0 if already at or beyond max)
    pub fn iterations_remaining(&self) -> u64 {
        self.max_iterations
            .saturating_sub(self.iterations_completed)
    }
}
