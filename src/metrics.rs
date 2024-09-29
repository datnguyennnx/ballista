use std::time::Duration;
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct Metrics {
    total_requests: u32,
    successful_requests: u32,
    failed_requests: u32,
    total_time: Duration,
    response_times: Vec<Duration>,
    json_responses: Vec<Value>,
    status_codes: HashMap<u16, u32>,
}

impl Metrics {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    pub fn update(&mut self, duration: Duration, status: u16, json: Option<Value>) {
        self.total_requests += 1;
        self.total_time += duration;
        self.response_times.push(duration);

        *self.status_codes.entry(status).or_insert(0) += 1;

        if (200..300).contains(&status) {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        if let Some(json) = json {
            self.json_responses.push(json);
        }
    }

    pub fn min_duration(&self) -> Option<Duration> {
        self.response_times.iter().min().cloned()
    }

    pub fn max_duration(&self) -> Option<Duration> {
        self.response_times.iter().max().cloned()
    }

    pub fn median_duration(&self) -> Option<Duration> {
        let len = self.response_times.len();
        if len == 0 {
            None
        } else {
            let mut sorted_times = self.response_times.clone();
            sorted_times.sort_unstable();
            Some(sorted_times[len / 2])
        }
    }

    pub fn percentile_duration(&self, percentile: f64) -> Option<Duration> {
        if self.response_times.is_empty() {
            None
        } else {
            let mut sorted_times = self.response_times.clone();
            sorted_times.sort_unstable();
            let index = ((sorted_times.len() as f64 * percentile / 100.0).round() as usize)
                .saturating_sub(1)
                .min(sorted_times.len() - 1);
            Some(sorted_times[index])
        }
    }

    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_requests: self.total_requests,
            successful_requests: self.successful_requests,
            failed_requests: self.failed_requests,
            total_time: self.total_time,
            min_duration: self.min_duration(),
            max_duration: self.max_duration(),
            median_duration: self.median_duration(),
            percentile_95: self.percentile_duration(95.0),
            status_codes: self.status_codes.clone(),
        }
    }
}

pub struct MetricsSummary {
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub total_time: Duration,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub median_duration: Option<Duration>,
    pub percentile_95: Option<Duration>,
    pub status_codes: HashMap<u16, u32>,
}