use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Save data to a file using JSON serialization
pub fn save_to_file<T: Serialize>(data: &T, path: &Path) -> std::io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}

/// Load data from a file using JSON deserialization
pub fn load_from_file<T: for<'a> Deserialize<'a>>(path: &Path) -> std::io::Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

/// Save data to a file using JSON serialization (with string path)
pub fn save_to_file_str<T: Serialize>(data: &T, filepath: &str) -> std::io::Result<()> {
    let file = File::create(filepath)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}

/// Load data from a file using JSON deserialization (with string path)
pub fn load_from_file_str<T: for<'a> Deserialize<'a>>(filepath: &str) -> std::io::Result<T> {
    if !Path::new(filepath).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));
    }

    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}
