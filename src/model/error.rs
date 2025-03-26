use thiserror::Error;

/// Main error type for the application
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("HTTP error: status {0}")]
    HttpError(u16), // For specific HTTP status errors if needed elsewhere
    
    #[error("No URLs provided for the test")]
    NoUrls,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Test already running")]
    TestAlreadyRunning,

    #[error("Test execution failed: {0}")] // New variant
    TestExecutionError(String),
}