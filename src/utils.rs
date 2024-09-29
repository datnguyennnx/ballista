use std::fs::File;
use std::io;
use std::thread;
use std::time::Duration;
use xml::reader::{EventReader, XmlEvent};
use sysinfo::{System, SystemExt, CpuExt, NetworkExt, NetworksExt};

/// Get the current CPU usage as a percentage
pub fn get_cpu_usage() -> io::Result<f64> {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    
    // Wait a bit to get accurate CPU usage
    thread::sleep(Duration::from_millis(100));
    sys.refresh_cpu();

    Ok(sys.global_cpu_info().cpu_usage() as f64)
}

/// Get the current memory usage as a percentage
pub fn get_memory_usage() -> io::Result<f64> {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total_memory = sys.total_memory() as f64;
    let used_memory = sys.used_memory() as f64;
    
    Ok((used_memory / total_memory) * 100.0)
}

/// Get the current network usage in MB/s (received, sent)
pub fn get_network_usage() -> io::Result<(f64, f64)> {
    let mut sys = System::new_all();
    sys.refresh_networks();
    
    let initial_received: u64 = sys.networks().iter().map(|(_, network)| network.total_received()).sum();
    let initial_transmitted: u64 = sys.networks().iter().map(|(_, network)| network.total_transmitted()).sum();
    
    // Wait a bit to get accurate network usage
    thread::sleep(Duration::from_secs(1));
    sys.refresh_networks();
    
    let final_received: u64 = sys.networks().iter().map(|(_, network)| network.total_received()).sum();
    let final_transmitted: u64 = sys.networks().iter().map(|(_, network)| network.total_transmitted()).sum();
    
    let received_per_second = (final_received - initial_received) as f64 / 1_000_000.0; // Convert to MB/s
    let transmitted_per_second = (final_transmitted - initial_transmitted) as f64 / 1_000_000.0; // Convert to MB/s
    
    Ok((received_per_second, transmitted_per_second))
}

/// Get the total number of CPU cores
pub fn get_cpu_cores() -> io::Result<usize> {
    let sys = System::new_all();
    Ok(sys.cpus().len())
}

/// Get the total system memory in GB
pub fn get_total_memory() -> io::Result<f64> {
    let sys = System::new_all();
    Ok(sys.total_memory() as f64 / 1_000_000_000.0) // Convert bytes to GB
}

/// Parse a sitemap XML file and extract URLs
pub fn parse_sitemap(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let parser = EventReader::new(file);
    let url_tags = ["loc", "url", "link"];

    parser.into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| match e {
            XmlEvent::Characters(content) if content.starts_with("http") => Some(content),
            _ => None,
        })
        .collect::<Vec<String>>()
        .into_iter()
        .filter(|url| url_tags.iter().any(|&tag| url.contains(tag)))
        .collect::<Vec<String>>()
        .into_iter()
        .filter(|url| !url.is_empty())
        .map(|url| Ok(url))
        .collect::<Result<Vec<String>, Box<dyn std::error::Error>>>()
        .and_then(|urls| {
            if urls.is_empty() {
                Err("No valid URLs found in the XML file".into())
            } else {
                Ok(urls)
            }
        })
}