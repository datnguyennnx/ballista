use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub url: Option<String>,

    #[arg(long)]
    pub sitemap: Option<String>,

    #[arg(short, long, default_value_t = 100)]
    pub requests: u32,

    #[arg(short, long, default_value_t = 10)]
    pub concurrency: u32,

    #[arg(short, long)]
    pub stress: bool,

    #[arg(short, long, default_value_t = 60)]
    pub duration: u64,
}