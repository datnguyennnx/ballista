use std::time::Duration;
use serde_json::Value;
use crate::metrics::summary::MetricsSummary;

#[derive(Clone, Default)]
pub struct Metrics {
    response_times: Vec<Duration>,
    status_codes: Vec<u16>,
    json_responses: Vec<Option<Value>>,
}

pub fn new_metrics() -> Metrics {
    Metrics::default()
}

pub fn add_request(metrics: Metrics, duration: Duration, status: u16, json: Option<Value>) -> Metrics {
    Metrics {
        response_times: metrics.response_times.into_iter().chain(std::iter::once(duration)).collect(),
        status_codes: metrics.status_codes.into_iter().chain(std::iter::once(status)).collect(),
        json_responses: metrics.json_responses.into_iter().chain(std::iter::once(json)).collect(),
    }
}

pub fn calculate_summary(metrics: &Metrics) -> MetricsSummary {
    let total_requests = metrics.response_times.len();
    let total_time = metrics.response_times.iter().sum();

    let mut sorted_times = metrics.response_times.clone();
    sorted_times.sort_unstable();

    MetricsSummary::new(
        total_requests,
        total_time,
        sorted_times.first().cloned(),
        sorted_times.last().cloned(),
        calculate_percentile(&sorted_times, 50.0),
        calculate_percentile(&sorted_times, 95.0),
        &metrics.status_codes,
    )
}

fn calculate_percentile(sorted_times: &[Duration], percentile: f64) -> Option<Duration> {
    sorted_times.get(
        ((sorted_times.len() as f64 * percentile / 100.0).round() as usize)
            .saturating_sub(1)
            .min(sorted_times.len().saturating_sub(1))
    ).cloned()
}

pub fn combine_metrics<I>(metrics_iter: I) -> Metrics
where
    I: IntoIterator<Item = Metrics>,
{
    metrics_iter.into_iter().fold(new_metrics(), |acc, m| Metrics {
        response_times: acc.response_times.into_iter().chain(m.response_times).collect(),
        status_codes: acc.status_codes.into_iter().chain(m.status_codes).collect(),
        json_responses: acc.json_responses.into_iter().chain(m.json_responses).collect(),
    })
}

// Higher-order function to apply an operation to all metrics
pub fn apply_to_metrics<F, T>(metrics: &Metrics, f: F) -> Vec<T>
where
    F: Fn(&Duration, &u16, &Option<Value>) -> T,
{
    metrics.response_times.iter()
        .zip(metrics.status_codes.iter())
        .zip(metrics.json_responses.iter())
        .map(|((duration, status), json)| f(duration, status, json))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_summary() {
        let metrics = add_request(
            add_request(
                add_request(
                    new_metrics(),
                    Duration::from_millis(100),
                    200,
                    None
                ),
                Duration::from_millis(200),
                200,
                None
            ),
            Duration::from_millis(300),
            404,
            None
        );

        let summary = calculate_summary(&metrics);

        assert_eq!(summary.total_requests(), 3);
        assert_eq!(summary.total_time(), Duration::from_millis(600));
        assert_eq!(summary.min_duration(), Some(Duration::from_millis(100)));
        assert_eq!(summary.max_duration(), Some(Duration::from_millis(300)));
        assert_eq!(summary.median_duration(), Some(Duration::from_millis(200)));
        assert_eq!(summary.percentile_95(), Some(Duration::from_millis(300)));
        assert_eq!(summary.status_codes().get(&200), Some(&2));
        assert_eq!(summary.status_codes().get(&404), Some(&1));
    }

    #[test]
    fn test_combine_metrics() {
        let metrics1 = add_request(
            add_request(
                new_metrics(),
                Duration::from_millis(100),
                200,
                None
            ),
            Duration::from_millis(200),
            200,
            None
        );
        let metrics2 = add_request(
            new_metrics(),
            Duration::from_millis(300),
            404,
            None
        );

        let combined = combine_metrics(vec![metrics1, metrics2]);
        let summary = calculate_summary(&combined);

        assert_eq!(summary.total_requests(), 3);
        assert_eq!(summary.total_time(), Duration::from_millis(600));
        assert_eq!(summary.min_duration(), Some(Duration::from_millis(100)));
        assert_eq!(summary.max_duration(), Some(Duration::from_millis(300)));
        assert_eq!(summary.median_duration(), Some(Duration::from_millis(200)));
        assert_eq!(summary.percentile_95(), Some(Duration::from_millis(300)));
        assert_eq!(summary.status_codes().get(&200), Some(&2));
        assert_eq!(summary.status_codes().get(&404), Some(&1));
    }

    #[test]
    fn test_apply_to_metrics() {
        let metrics = add_request(
            add_request(
                new_metrics(),
                Duration::from_millis(100),
                200,
                None
            ),
            Duration::from_millis(200),
            404,
            None
        );

        let result = apply_to_metrics(&metrics, |duration, status, _| {
            (duration.as_millis() as u64, *status)
        });

        assert_eq!(result, vec![(100, 200), (200, 404)]);
    }
}