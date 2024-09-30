use std::fs::File;
use std::io;
use xml::reader::{EventReader, XmlEvent};
use sysinfo::{System, SystemExt, CpuExt, NetworkExt, NetworksExt};
use tokio::time::{sleep, Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("XML parsing error: {0}")]
    Xml(#[from] xml::reader::Error),
    #[error("No valid URLs found in the XML file")]
    NoValidUrls,
}

/// Get the current CPU usage as a percentage
pub async fn get_cpu_usage() -> io::Result<f64> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    
    sleep(Duration::from_millis(100)).await;
    sys.refresh_cpu();

    Ok(sys.global_cpu_info().cpu_usage() as f64)
}

/// Get the current memory usage as a percentage
pub async fn get_memory_usage() -> io::Result<f64> {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total_memory = sys.total_memory() as f64;
    let used_memory = sys.used_memory() as f64;
    
    Ok((used_memory / total_memory) * 100.0)
}

/// Get the current network usage in MB/s (received, sent)
pub async fn get_network_usage() -> io::Result<(f64, f64)> {
    let mut sys = System::new_all();
    sys.refresh_networks();
    
    let initial_received: u64 = sys.networks().iter().map(|(_, network)| network.total_received()).sum();
    let initial_transmitted: u64 = sys.networks().iter().map(|(_, network)| network.total_transmitted()).sum();
    
    sleep(Duration::from_secs(1)).await;
    sys.refresh_networks();
    
    let final_received: u64 = sys.networks().iter().map(|(_, network)| network.total_received()).sum();
    let final_transmitted: u64 = sys.networks().iter().map(|(_, network)| network.total_transmitted()).sum();
    
    let received_per_second = (final_received - initial_received) as f64 / 1_000_000.0; // Convert to MB/s
    let transmitted_per_second = (final_transmitted - initial_transmitted) as f64 / 1_000_000.0; // Convert to MB/s
    
    Ok((received_per_second, transmitted_per_second))
}

/// Parse a sitemap XML file and extract URLs
pub fn parse_sitemap(path: &str) -> Result<Vec<String>, UtilError> {
    let file = File::open(path)?;
    let parser = EventReader::new(file);
    let url_tags = ["loc", "url", "link"];

    let urls: Vec<String> = parser
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|e| match e {
            XmlEvent::Characters(content) if content.starts_with("http") => Some(content),
            _ => None,
        })
        .filter(|url| url_tags.iter().any(|&tag| url.contains(tag)))
        .filter(|url| !url.is_empty())
        .collect();

    if urls.is_empty() {
        Err(UtilError::NoValidUrls)
    } else {
        Ok(urls)
    }
}

/// Format a duration in a human-readable format
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    let micros = duration.subsec_micros() % 1000;
    
    if secs == 0 {
        if millis == 0 {
            format!("{}µs", micros)
        } else {
            format!("{}ms {}µs", millis, micros)
        }
    } else if secs < 60 {
        format!("{}s {}ms", secs, millis)
    } else {
        let minutes = secs / 60;
        let remaining_secs = secs % 60;
        format!("{}m {}s", minutes, remaining_secs)
    }
}

/// Format a size in bytes to a human-readable format
pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else {
        format!("{:.2} GB", size as f64 / GB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_micros(500)), "500µs");
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms 0µs");
        assert_eq!(format_duration(Duration::from_secs(5)), "5s 0ms");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
        assert_eq!(format_duration(Duration::new(3661, 500_000_000)), "61m 1s");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1500), "1.46 KB");
        assert_eq!(format_size(1500000), "1.43 MB");
        assert_eq!(format_size(1500000000), "1.40 GB");
    }

    #[test]
    fn test_parse_sitemap() {
        // This test would require a sample XML file
        // For now, we'll just test the error case
        let result = parse_sitemap("nonexistent_file.xml");
        assert!(result.is_err());
    }

    // Note: We can't easily test get_cpu_usage, get_memory_usage, and get_network_usage
    // in a unit test environment as they depend on system state.
    // These would be better suited for integration tests.
}