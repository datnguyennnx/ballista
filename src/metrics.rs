use std::time::Duration;
use serde_json::Value;

pub struct Metrics {
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub total_time: Duration,
    pub response_times: Vec<Duration>,
    pub json_responses: Vec<Value>,
    pub status_codes: std::collections::HashMap<u16, u32>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_time: Duration::new(0, 0),
            response_times: Vec::new(),
            json_responses: Vec::new(),
            status_codes: std::collections::HashMap::new(),
        }
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
        if self.response_times.is_empty() {
            None
        } else {
            let mut sorted_times = self.response_times.clone();
            sorted_times.sort();
            Some(calculate_percentile(&sorted_times, 50.0))
        }
    }
}

pub fn calculate_percentile(sorted_times: &[Duration], percentile: f64) -> Duration {
    let index = (sorted_times.len() as f64 * percentile / 100.0) as usize;
    sorted_times[index]
}