use std::time::Duration;
use serde_json::Value;
use crate::metrics::summary::MetricsSummary;

#[derive(Clone, Default)]
pub struct Metrics {
    response_times: Vec<Duration>,
    status_codes: Vec<u16>,
    json_responses: Vec<Option<Value>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_request(&mut self, duration: Duration, status: u16, json: Option<Value>) {
        self.response_times.push(duration);
        self.status_codes.push(status);
        self.json_responses.push(json);
    }

    pub fn summary(&self) -> MetricsSummary {
        let total_requests = self.response_times.len();
        let total_time = self.response_times.iter().sum();

        let mut sorted_times = self.response_times.clone();
        sorted_times.sort_unstable();

        MetricsSummary::new(
            total_requests,
            total_time,
            sorted_times.first().cloned(),
            sorted_times.last().cloned(),
            Self::calculate_percentile(&sorted_times, 50.0),
            Self::calculate_percentile(&sorted_times, 95.0),
            &self.status_codes,
        )
    }

    fn calculate_percentile(sorted_times: &[Duration], percentile: f64) -> Option<Duration> {
        if sorted_times.is_empty() {
            None
        } else {
            let index = ((sorted_times.len() as f64 * percentile / 100.0).round() as usize)
                .saturating_sub(1)
                .min(sorted_times.len() - 1);
            Some(sorted_times[index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_summary() {
        let mut metrics = Metrics::new();
        metrics.add_request(Duration::from_millis(100), 200, None);
        metrics.add_request(Duration::from_millis(200), 200, None);
        metrics.add_request(Duration::from_millis(300), 404, None);

        let summary = metrics.summary();

        assert_eq!(summary.total_requests(), 3);
        assert_eq!(summary.total_time(), Duration::from_millis(600));
        assert_eq!(summary.min_duration(), Some(Duration::from_millis(100)));
        assert_eq!(summary.max_duration(), Some(Duration::from_millis(300)));
        assert_eq!(summary.median_duration(), Some(Duration::from_millis(200)));
        assert_eq!(summary.percentile_95(), Some(Duration::from_millis(300)));
        assert_eq!(summary.status_codes().get(&200), Some(&2));
        assert_eq!(summary.status_codes().get(&404), Some(&1));
    }
}