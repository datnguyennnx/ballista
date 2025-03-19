use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::core::error::AppError;

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Serialize, Deserialize, Clone)]
pub enum Command {
    #[command(name = "load-test")]
    LoadTest {
        /// URL to test
        #[arg(long)]
        url: String,

        /// Number of requests to send
        #[arg(short, long, default_value = "100")]
        requests: u32,

        /// Number of concurrent requests
        #[arg(short, long, default_value = "10")]
        concurrency: u32,
    },
    #[command(name = "stress-test")]
    StressTest {
        /// Path to sitemap XML file
        #[arg(long)]
        sitemap: String,

        /// Duration of the stress test in seconds
        #[arg(short, long, default_value = "300")]
        duration: u64,

        /// Number of concurrent requests
        #[arg(short, long, default_value = "50")]
        concurrency: u32,
    },
    #[command(name = "api-test")]
    ApiTest {
        /// Path to API test JSON file
        #[arg(long)]
        path: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub command: Command,
}

// Custom error type for argument validation
#[derive(Debug)]
pub enum ArgError {
    InvalidUrl(String),
    InvalidSitemap(String),
    InvalidTestFile(String),
}

impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArgError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            ArgError::InvalidSitemap(path) => write!(f, "Invalid sitemap file: {}", path),
            ArgError::InvalidTestFile(path) => write!(f, "Invalid test file: {}", path),
        }
    }
}

impl std::error::Error for ArgError {}

// Pure functions for validation
pub fn starts_with_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

pub fn validate_sitemap(path: &str) -> bool {
    path.ends_with(".xml")
}

pub fn validate_test_file(path: &str) -> bool {
    path.ends_with(".json")
}

// Higher-order function for validation
fn validate<F>(value: &str, predicate: F, error: ArgError) -> Result<(), ArgError>
where
    F: Fn(&str) -> bool,
{
    if predicate(value) {
        Ok(())
    } else {
        Err(error)
    }
}

impl Args {
    // Validate the arguments
    pub fn validate(&self) -> Result<(), ArgError> {
        match &self.command {
            Command::LoadTest { url, .. } => 
                validate(url, starts_with_http, ArgError::InvalidUrl(url.to_string())),
            Command::StressTest { sitemap, .. } => 
                validate(sitemap, validate_sitemap, ArgError::InvalidSitemap(sitemap.to_string())),
            Command::ApiTest { path } => 
                validate(path, validate_test_file, ArgError::InvalidTestFile(path.to_string())),
        }
    }
}

// Implement From trait for better error handling
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::FileError(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::ParseError(error.to_string())
    }
}

impl From<ArgError> for AppError {
    fn from(error: ArgError) -> Self {
        AppError::ArgValidation(error.to_string())
    }
}

// Function to parse JSON configuration file and return Args
pub fn args_from_json(path: &Path) -> Result<Args, AppError> {
    std::fs::read_to_string(path)
        .map_err(AppError::from)
        .and_then(|config_str| serde_json::from_str::<Config>(&config_str).map_err(AppError::from))
        .map(|config| Args { command: config.command })
        .and_then(|args| args.validate().map(|_| args).map_err(AppError::from))
}

// Higher-order function to apply a transformation to all commands
pub fn map_command<F, T>(args: &Args, f: F) -> T
where
    F: Fn(&Command) -> T,
{
        f(&args.command)
    }

// Higher-order function to filter commands
pub fn filter_command<F>(args: &Args, predicate: F) -> Option<&Command>
where
    F: Fn(&Command) -> bool,
{
    if predicate(&args.command) {
        Some(&args.command)
    } else {
        None
    }
}