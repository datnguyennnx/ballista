use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// URL to test. Either this or --sitemap must be provided
    #[arg(long)]
    url: Option<String>,

    /// Path to sitemap XML file. Either this or --url must be provided
    #[arg(long)]
    sitemap: Option<String>,

    /// Number of requests to send (ignored if --stress is set)
    #[arg(short, long)]
    requests: Option<u32>,

    /// Number of concurrent requests
    #[arg(short, long)]
    concurrency: Option<u32>,

    /// Enable stress test mode (runs for a specified duration instead of a fixed number of requests)
    #[arg(short, long)]
    stress: bool,

    /// Duration of the stress test in seconds (only used if --stress is set)
    #[arg(short, long)]
    duration: Option<u64>,

    /// Collect and display resource usage data for 60 seconds
    #[arg(long)]
    resource_usage: bool,

    /// Path to JSON configuration file
    #[arg(long)]
    config: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub url: Option<String>,
    pub sitemap: Option<String>,
    pub requests: Option<u32>,
    pub concurrency: Option<u32>,
    pub stress: bool,
    pub duration: Option<u64>,
    pub resource_usage: bool,
}

impl Args {
    pub fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }

    pub fn sitemap(&self) -> Option<&String> {
        self.sitemap.as_ref()
    }

    pub fn requests(&self) -> u32 {
        self.requests.unwrap_or(100)
    }

    pub fn concurrency(&self) -> u32 {
        self.concurrency.unwrap_or(10)
    }

    pub fn stress(&self) -> bool {
        self.stress
    }

    pub fn duration(&self) -> u64 {
        self.duration.unwrap_or(60)
    }

    pub fn resource_usage(&self) -> bool {
        self.resource_usage
    }

    pub fn config(&self) -> Option<&String> {
        self.config.as_ref()
    }

    /// Validate the arguments to ensure either URL or sitemap is provided
    pub fn validate(&self) -> Result<(), String> {
        if self.resource_usage {
            return Ok(());
        }
        match (self.url(), self.sitemap()) {
            (None, None) => Err("Either --url or --sitemap must be provided".to_string()),
            (Some(_), Some(_)) => Err("Only one of --url or --sitemap should be provided".to_string()),
            (Some(url), None) => {
                if url.starts_with("http://") || url.starts_with("https://") {
                    Ok(())
                } else {
                    Err("URL must start with http:// or https://".to_string())
                }
            },
            (None, Some(_)) => Ok(()),
        }
    }

    /// Parse JSON configuration file and return Args
    pub fn from_json(path: &Path) -> Result<Self, String> {
        fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))
            .and_then(|config_str| {
                serde_json::from_str::<Config>(&config_str)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))
            })
            .map(|config| Args {
                url: config.url,
                sitemap: config.sitemap,
                requests: config.requests,
                concurrency: config.concurrency,
                stress: config.stress,
                duration: config.duration,
                resource_usage: config.resource_usage,
                config: Some(path.to_string_lossy().into_owned()),
            })
            .and_then(|args| args.validate().map(|_| args))
    }
}