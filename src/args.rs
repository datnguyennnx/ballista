use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
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
    #[command(name = "resource-usage")]
    ResourceUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub command: Command,
}

impl Args {
    /// Validate the arguments
    pub fn validate(&self) -> Result<(), String> {
        match &self.command {
            Command::LoadTest { url, .. } => {
                if url.starts_with("http://") || url.starts_with("https://") {
                    Ok(())
                } else {
                    Err("URL must start with http:// or https://".to_string())
                }
            },
            Command::StressTest { sitemap, .. } => {
                if sitemap.ends_with(".xml") {
                    Ok(())
                } else {
                    Err("Sitemap file must have .xml extension".to_string())
                }
            },
            Command::ApiTest { path } => {
                if path.ends_with(".json") {
                    Ok(())
                } else {
                    Err("API test file must have .json extension".to_string())
                }
            },
            Command::ResourceUsage => Ok(()),
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
                command: config.command,
            })
            .and_then(|args| args.validate().map(|_| args))
    }
}