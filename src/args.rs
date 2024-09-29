use clap::Parser;

/// Command-line arguments for the load testing tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// URL to test. Either this or --sitemap must be provided
    #[arg(long)]
    pub url: Option<String>,

    /// Path to sitemap XML file. Either this or --url must be provided
    #[arg(long)]
    pub sitemap: Option<String>,

    /// Number of requests to send (ignored if --stress is set)
    #[arg(short, long, default_value_t = 100)]
    pub requests: u32,

    /// Number of concurrent requests
    #[arg(short, long, default_value_t = 10)]
    pub concurrency: u32,

    /// Enable stress test mode (runs for a specified duration instead of a fixed number of requests)
    #[arg(short, long)]
    pub stress: bool,

    /// Duration of the stress test in seconds (only used if --stress is set)
    #[arg(short, long, default_value_t = 60)]
    pub duration: u64,
}

impl Args {
    /// Validate the arguments to ensure either URL or sitemap is provided
    pub fn validate(&self) -> Result<(), String> {
        match (&self.url, &self.sitemap) {
            (None, None) => Err("Either --url or --sitemap must be provided".to_string()),
            (Some(_), Some(_)) => Err("Only one of --url or --sitemap should be provided".to_string()),
            _ => Ok(()),
        }
    }
}