use std::io;
use sysinfo::{System, SystemExt, CpuExt, NetworkExt, NetworksExt};
use tokio::time::{sleep, Duration};

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

#[cfg(test)]
mod tests {
    use super::*;

    // Note: We can't easily test get_cpu_usage, get_memory_usage, and get_network_usage
    // in a unit test environment as they depend on system state.
    // These would be better suited for integration tests.
}