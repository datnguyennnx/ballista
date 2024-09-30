use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, interval};
use crate::utils::{get_cpu_usage, get_memory_usage, get_network_usage};

pub struct ResourceMonitor {
    is_finished: Arc<AtomicBool>,
}

impl ResourceMonitor {
    pub fn new(is_finished: Arc<AtomicBool>) -> Self {
        Self { is_finished }
    }

    pub async fn start(self) -> (Vec<f64>, Vec<f64>, Vec<(f64, f64)>) {
        let mut cpu_samples = Vec::new();
        let mut memory_samples = Vec::new();
        let mut network_samples = Vec::new();

        let mut interval = interval(Duration::from_secs(1));

        while !self.is_finished.load(Ordering::Relaxed) {
            interval.tick().await;

            let (cpu, memory, network) = tokio::join!(
                get_cpu_usage(),
                get_memory_usage(),
                get_network_usage()
            );

            if let Ok(cpu) = cpu {
                cpu_samples.push(cpu);
            }
            if let Ok(memory) = memory {
                memory_samples.push(memory);
            }
            if let Ok(network) = network {
                network_samples.push(network);
            }
        }

        (cpu_samples, memory_samples, network_samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_resource_monitor() {
        let is_finished = Arc::new(AtomicBool::new(false));
        let monitor = ResourceMonitor::new(Arc::clone(&is_finished));

        let monitor_handle = tokio::spawn(async move {
            monitor.start().await
        });

        // Allow the monitor to run for 2 seconds
        tokio::time::sleep(Duration::from_secs(2)).await;
        is_finished.store(true, Ordering::Relaxed);

        // Wait for the monitor to finish with a timeout
        let result = timeout(Duration::from_secs(1), monitor_handle).await;

        assert!(result.is_ok(), "Monitor did not finish within the expected time");

        let (cpu_samples, memory_samples, network_samples) = result.unwrap().unwrap();

        assert!(!cpu_samples.is_empty(), "CPU samples should not be empty");
        assert!(!memory_samples.is_empty(), "Memory samples should not be empty");
        assert!(!network_samples.is_empty(), "Network samples should not be empty");
    }
}