use std::path::{Path, PathBuf};
use crate::prelude::*;

pub fn get_example_path(filename: &str) -> PathBuf {
    Path::new("test_examples").join(filename)
}

pub fn list_examples() -> Result<Vec<String>, AppError> {
    let examples_dir = Path::new("test_examples");
    if !examples_dir.exists() {
        return Err(AppError::FileError("Examples directory not found".to_string()));
    }

    let entries = std::fs::read_dir(examples_dir)
        .map_err(|e| AppError::FileError(e.to_string()))?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                e.file_name()
                    .into_string()
                    .ok()
                    .filter(|name| name.ends_with(".json"))
            })
        })
        .collect();

    Ok(entries)
} 