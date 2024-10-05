use std::time::Duration;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub struct MetricsSummary {
    total_requests: usize,
    total_time: Duration,
    min_duration: Option<Duration>,
    max_duration: Option<Duration>,
    median_duration: Option<Duration>,
    percentile_95: Option<Duration>,
    status_codes: HashMap<u16, usize>,
}

impl fmt::Debug for MetricsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetricsSummary")
            .field("total_requests", &self.total_requests)
            .field("total_time", &self.total_time)
            .field("min_duration", &self.min_duration)
            .field("max_duration", &self.max_duration)
            .field("median_duration", &self.median_duration)
            .field("percentile_95", &self.percentile_95)
            .field("status_codes", &self.status_codes)
            .finish()
    }
}

impl MetricsSummary {
    pub fn new(
        total_requests: usize,
        total_time: Duration,
        min_duration: Option<Duration>,
        max_duration: Option<Duration>,
        median_duration: Option<Duration>,
        percentile_95: Option<Duration>,
        status_codes: &[u16],
    ) -> Self {
        let status_codes = status_codes.iter().fold(HashMap::new(), |mut acc, &code| {
            *acc.entry(code).or_insert(0) += 1;
            acc
        });

        Self {
            total_requests,
            total_time,
            min_duration,
            max_duration,
            median_duration,
            percentile_95,
            status_codes,
        }
    }

    pub fn total_requests(&self) -> usize {
        self.total_requests
    }

    pub fn total_time(&self) -> Duration {
        self.total_time
    }

    pub fn min_duration(&self) -> Option<Duration> {
        self.min_duration
    }

    pub fn max_duration(&self) -> Option<Duration> {
        self.max_duration
    }

    pub fn median_duration(&self) -> Option<Duration> {
        self.median_duration
    }

    pub fn percentile_95(&self) -> Option<Duration> {
        self.percentile_95
    }

    pub fn status_codes(&self) -> &HashMap<u16, usize> {
        &self.status_codes
    }

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
    fn test_metrics_summary_calculations() {
        let summary = MetricsSummary::new(
            3,
            Duration::from_millis(600),
            Some(Duration::from_millis(100)),
            Some(Duration::from_millis(300)),
            Some(Duration::from_millis(200)),
            Some(Duration::from_millis(300)),
            &[200, 200, 404],
        );

        assert_eq!(summary.average_duration(), Some(Duration::from_millis(200)));
        assert_eq!(summary.requests_per_second(), 5.0);
        assert_eq!(summary.success_rate(), 2.0 / 3.0);
    }
}