use colored::{Colorize, ColoredString};
use crate::utils::format_size;

pub fn status_color(status: u16) -> ColoredString {
    match status {
        200..=299 => status.to_string().green(),
        300..=399 => status.to_string().yellow(),
        400..=599 => status.to_string().red(),
        _ => status.to_string().normal(),
    }
}

pub fn format_cpu_usage(value: f64, cpu_cores: f64) -> String {
    format!("{:.2}% ({:.2} cores)", value, value * cpu_cores / 100.0)
}

pub fn format_memory_usage(value: f64, total_memory: f64) -> String {
    format!("{:.2}% ({:.2} GB)", value, value * total_memory / 100.0)
}

pub fn format_network_usage(value: f64) -> String {
    format!("{}/s", format_size((value * 1_000_000.0) as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_color() {
        assert_eq!(status_color(200).to_string(), "200".green().to_string());
        assert_eq!(status_color(302).to_string(), "302".yellow().to_string());
        assert_eq!(status_color(404).to_string(), "404".red().to_string());
        assert_eq!(status_color(600).to_string(), "600");
    }

    #[test]
    fn test_format_cpu_usage() {
        assert_eq!(format_cpu_usage(50.0, 4.0), "50.00% (2.00 cores)");
    }

    #[test]
    fn test_format_memory_usage() {
        assert_eq!(format_memory_usage(75.0, 16.0), "75.00% (12.00 GB)");
    }

    #[test]
    fn test_format_network_usage() {
        assert_eq!(format_network_usage(1.5), "1.50 MB/s");
    }
}