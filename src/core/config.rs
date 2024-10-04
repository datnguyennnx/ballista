use clap::Parser;
use std::path::Path;
use std::sync::Arc;

use crate::core::error::AppError;
use crate::utils::parsers::parse_sitemap;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug, Clone)]
pub enum Command {
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
    ApiTest {
        /// Path to API test JSON file
        #[arg(long)]
        path: String,
    },
    ResourceUsage,
}

impl Args {
    pub fn from_json(_path: &Path) -> Result<Self, AppError> {
        // Implement JSON parsing logic
        // ...
        Err(AppError::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "JSON parsing not yet implemented",
        ))))
    }

    pub fn validate(&self) -> Result<(), AppError> {
        match &self.command {
            Command::LoadTest { url, .. } => {
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(AppError::ArgValidation("URL must start with http:// or https://".into()));
                }
            },
            Command::StressTest { sitemap, .. } => {
                if !sitemap.ends_with(".xml") {
                    return Err(AppError::ArgValidation("Sitemap file must have .xml extension".into()));
                }
            },
            Command::ApiTest { path } => {
                if !path.ends_with(".json") {
                    return Err(AppError::ArgValidation("API test file must have .json extension".into()));
                }
            },
            Command::ResourceUsage => {},
        }
        Ok(())
    }
}

pub async fn parse_arguments() -> Result<Args, AppError> {
    let args = Args::parse();
    args.validate()?;
    Ok(args)
}

pub fn prepare_urls(command: &Command) -> Result<Arc<Vec<String>>, AppError> {
    match command {
        Command::LoadTest { url, .. } => Ok(Arc::new(vec![url.to_string()])),
        Command::StressTest { sitemap, .. } => {
            let urls = parse_sitemap(sitemap).map_err(|e| AppError::Other(Box::new(e)))?;
            if urls.is_empty() {
                Err(AppError::NoUrls)
            } else {
                Ok(Arc::new(urls))
            }
        },
        _ => Err(AppError::ArgValidation("Invalid command for preparing URLs".into())),
    }
}