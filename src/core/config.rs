use crate::prelude::*;
use crate::args::{Args, Command};
use crate::utils::parsers::{parse_urls, parse_json, load_example_file, UtilError};
use std::path::{Path, PathBuf};
use tokio::fs;
use clap::Parser;
use serde::Deserialize;
use std::env;

pub async fn parse_arguments() -> Result<Args, AppError> {
    Args::try_parse().map_err(|e| AppError::ArgValidation(e.to_string()))
}

pub async fn prepare_urls(command: &Command) -> Result<Vec<String>, AppError> {
    match command {
        Command::LoadTest { url, .. } => Ok(vec![url.clone()]),
        Command::StressTest { sitemap, .. } => 
            fs::read_to_string(sitemap)
                .await
                .map_err(|e| AppError::FileError(e.to_string()))
                .and_then(|content| parse_urls(&content).map_err(|e| AppError::ParseError(e.to_string()))),
        _ => Err(AppError::InvalidConfig("Invalid command for URL preparation".to_string())),
    }
}

pub fn validate(args: &Args) -> Result<(), AppError> {
    let validate_url = |url: &str| {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            Err(AppError::ArgValidation(format!("Invalid URL: {}", url)))
        } else {
            Ok(())
        }
    };

    let validate_file = |path: &str, extension: &str| {
        if !path.ends_with(extension) {
            Err(AppError::ArgValidation(format!("Invalid file: {}", path)))
        } else {
            Ok(())
        }
    };

    match &args.command {
        Command::LoadTest { url, .. } => validate_url(url),
        Command::StressTest { sitemap, .. } => validate_file(sitemap, ".xml"),
        Command::ApiTest { path } => validate_file(path, ".json"),
    }
}

pub async fn load_config(path: &str) -> Result<Args, AppError> {
    // First try to load from examples directory if it's just a filename
    let content = if Path::new(path).is_relative() && !Path::new(path).exists() {
        load_example_file(path)
    } else {
        fs::read_to_string(path)
            .await
            .map_err(|e| UtilError::Io(e))
    }.map_err(|e| AppError::FileError(e.to_string()))?;

    parse_json(&content)
        .map_err(|e| AppError::ParseError(e.to_string()))
        .and_then(|config: Args| validate(&config).map(|_| config))
}

pub fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .map(|path| path.join("ballista").join("config.json"))
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api_port: u16,
    pub api_host: String,
    pub cors_allowed_origins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_port: 3001,
            api_host: "0.0.0.0".to_string(),
            cors_allowed_origins: vec!["*".to_string()],
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            api_port: env::var("API_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
            api_host: env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_else(|_| vec!["*".to_string()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_prepare_urls_load_test() {
        let command = Command::LoadTest {
            url: "https://example.com".to_string(),
            requests: 10,
            concurrency: 2,
        };
        let urls = prepare_urls(&command).await.unwrap();
        assert_eq!(urls, vec!["https://example.com"]);
    }

    #[test]
    fn test_validate_load_test() {
        let args = Args {
            command: Command::LoadTest {
                url: "https://example.com".to_string(),
                requests: 10,
                concurrency: 2,
            },
        };
        assert!(validate(&args).is_ok());
    }

    #[test]
    fn test_validate_stress_test() {
        let args = Args {
            command: Command::StressTest {
                sitemap: "sitemap.xml".to_string(),
                duration: 60,
                concurrency: 5,
            },
        };
        assert!(validate(&args).is_ok());
    }

    #[test]
    fn test_validate_api_test() {
        let args = Args {
            command: Command::ApiTest {
                path: "tests.json".to_string(),
            },
        };
        assert!(validate(&args).is_ok());
    }

    #[test]
    fn test_validate_invalid_url() {
        let args = Args {
            command: Command::LoadTest {
                url: "invalid-url".to_string(),
                requests: 10,
                concurrency: 2,
            },
        };
        assert!(validate(&args).is_err());
    }

    #[test]
    fn test_validate_invalid_sitemap() {
        let args = Args {
            command: Command::StressTest {
                sitemap: "sitemap.txt".to_string(),
                duration: 60,
                concurrency: 5,
            },
        };
        assert!(validate(&args).is_err());
    }

    #[test]
    fn test_validate_invalid_test_file() {
        let args = Args {
            command: Command::ApiTest {
                path: "tests.txt".to_string(),
            },
        };
        assert!(validate(&args).is_err());
    }

    #[tokio::test]
    async fn test_load_example_config() {
        // Create test configuration in memory
        let config_content = r#"{
            "command": {
                "LoadTest": {
                    "url": "https://example.com",
                    "requests": 10,
                    "concurrency": 2
                }
            }
        }"#;
        
        // Test with in-memory configuration
        let result = parse_json(config_content);
        assert!(result.is_ok());
        let config: Args = result.unwrap();
        assert!(validate(&config).is_ok());
    }
}