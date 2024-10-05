use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, interval};
use crate::utils::{get_cpu_usage, get_memory_usage, get_network_usage};
use crate::core::error::AppError;

type ResourceSample = (f64, f64, (f64, f64));
type ResourceSamples = (Vec<f64>, Vec<f64>, Vec<(f64, f64)>);

// Pure function to collect a single sample
async fn collect_sample() -> Result<ResourceSample, AppError> {
    let (cpu, memory, network) = tokio::join!(
        get_cpu_usage(),
        get_memory_usage(),
        get_network_usage()
    );

    Ok((
        cpu.map_err(|e| AppError::Other(e.to_string()))?,
        memory.map_err(|e| AppError::Other(e.to_string()))?,
        network.map_err(|e| AppError::Other(e.to_string()))?,
    ))
}

// Pure function to add a sample to the collection
fn add_sample(samples: ResourceSamples, new_sample: ResourceSample) -> ResourceSamples {
    let (mut cpu_samples, mut memory_samples, mut network_samples) = samples;
    let (cpu, memory, network) = new_sample;

    cpu_samples.push(cpu);
    memory_samples.push(memory);
    network_samples.push(network);

    (cpu_samples, memory_samples, network_samples)
}

// Higher-order function to collect samples until a condition is met
async fn collect_samples_until<F>(mut interval: tokio::time::Interval, is_finished: F) -> Result<ResourceSamples, AppError>
where
    F: Fn() -> bool,
{
    let mut samples = (Vec::new(), Vec::new(), Vec::new());

    while !is_finished() {
        interval.tick().await;
        let new_sample = collect_sample().await?;
        samples = add_sample(samples, new_sample);
    }

    Ok(samples)
}

// Main function to monitor resources
pub async fn monitor_resources(is_finished: Arc<AtomicBool>) -> Result<ResourceSamples, AppError> {
    let interval = interval(Duration::from_secs(1));
    collect_samples_until(interval, || is_finished.load(Ordering::Relaxed)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_monitor() {
        let result = run_monitor_for_duration(Duration::from_secs(2)).await;

        assert!(result.is_ok(), "Monitor did not finish within the expected time");

        let (cpu_samples, memory_samples, network_samples) = result.unwrap();

        assert!(!cpu_samples.is_empty(), "CPU samples should not be empty");
        assert!(!memory_samples.is_empty(), "Memory samples should not be empty");
        assert!(!network_samples.is_empty(), "Network samples should not be empty");
    }
}