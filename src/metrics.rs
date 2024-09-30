use std::time::Duration;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct Metrics {
    response_times: Vec<Duration>,
    status_codes: Vec<u16>,
    json_responses: Vec<Option<Value>>,
}

#[derive(Clone)]
pub struct MetricsSummary {
    pub total_requests: usize,
    pub total_time: Duration,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub median_duration: Option<Duration>,
    pub percentile_95: Option<Duration>,
    pub status_codes: HashMap<u16, usize>,
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

        MetricsSummary {
            total_requests,
            total_time,
            min_duration: sorted_times.first().cloned(),
            max_duration: sorted_times.last().cloned(),
            median_duration: Self::calculate_percentile(&sorted_times, 50.0),
            percentile_95: Self::calculate_percentile(&sorted_times, 95.0),
            status_codes: self.status_codes.iter().fold(HashMap::new(), |mut acc, &code| {
                *acc.entry(code).or_insert(0) += 1;
                acc
            }),
        }
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

impl MetricsSummary {
    pub fn average_duration(&self) -> Option<Duration> {
        if self.total_requests == 0 {
            None
        } else {
            Some(self.total_time / self.total_requests as u32)
        }
    }

    pub fn requests_per_second(&self) -> f64 {
        self.total_requests as f64 / self.total_time.as_secs_f64()
    }

    pub fn success_rate(&self) -> f64 {
        let successful_requests = self.status_codes.iter()
            .filter(|(&code, _)| code >= 200 && code < 300)
            .map(|(_, &count)| count)
            .sum::<usize>();
        
        successful_requests as f64 / self.total_requests as f64
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

        assert_eq!(summary.total_requests, 3);
        assert_eq!(summary.total_time, Duration::from_millis(600));
        assert_eq!(summary.min_duration, Some(Duration::from_millis(100)));
        assert_eq!(summary.max_duration, Some(Duration::from_millis(300)));
        assert_eq!(summary.median_duration, Some(Duration::from_millis(200)));
        assert_eq!(summary.percentile_95, Some(Duration::from_millis(300)));
        assert_eq!(summary.status_codes.get(&200), Some(&2));
        assert_eq!(summary.status_codes.get(&404), Some(&1));
    }

    #[test]
    fn test_metrics_summary_calculations() {
        let mut metrics = Metrics::new();
        metrics.add_request(Duration::from_millis(100), 200, None);
        metrics.add_request(Duration::from_millis(200), 200, None);
        metrics.add_request(Duration::from_millis(300), 404, None);

        let summary = metrics.summary();

        assert_eq!(summary.average_duration(), Some(Duration::from_millis(200)));
        assert_eq!(summary.requests_per_second(), 5.0);
        assert_eq!(summary.success_rate(), 2.0 / 3.0);
    }
}