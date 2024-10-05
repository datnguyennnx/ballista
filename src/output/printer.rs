use crate::prelude::*;
use crate::metrics::summary::MetricsSummary;
use prettytable::{Table, Row, Cell};

pub fn format_test_results(
    summary: Option<&MetricsSummary>,
    total_duration: Option<Duration>,
    cpu_samples: &[f64],
    memory_samples: &[f64],
    network_samples: &[(f64, f64)],
) -> String {
    let mut output = String::new();

    if let Some(summary) = summary {
        output.push_str(&format_summary(summary));
    }

    if let Some(duration) = total_duration {
        output.push_str(&format!("\nTotal duration: {}\n", format_duration(duration)));
    }

    output.push_str(&format_resource_usage(cpu_samples, memory_samples, network_samples));

    output
}

fn format_summary(summary: &MetricsSummary) -> String {
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Metric"),
        Cell::new("Value"),
    ]));

    let rows = vec![
        ("Total Requests", summary.total_requests().to_string()),
        ("Total Time", format_duration(summary.total_time())),
        ("Requests/second", format!("{:.2}", summary.requests_per_second())),
        ("Mean Response Time", format_duration(summary.average_duration().unwrap_or_default())),
        ("Median Response Time", format_duration(summary.median_duration().unwrap_or_default())),
        ("95th Percentile", format_duration(summary.percentile_95().unwrap_or_default())),
        ("Min Response Time", format_duration(summary.min_duration().unwrap_or_default())),
        ("Max Response Time", format_duration(summary.max_duration().unwrap_or_default())),
    ];

    for (metric, value) in rows {
        table.add_row(Row::new(vec![
            Cell::new(metric),
            Cell::new(&value),
        ]));
    }

    let mut result = table.to_string();
    result.push_str("\nStatus Code Distribution:\n");
    for (code, count) in summary.status_codes() {
        result.push_str(&format!("{}: {}\n", code, count));
    }

    result
}

fn format_resource_usage(
    cpu_samples: &[f64],
    memory_samples: &[f64],
    network_samples: &[(f64, f64)],
) -> String {
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Resource"),
        Cell::new("Min"),
        Cell::new("Max"),
        Cell::new("Avg"),
    ]));

    let cpu_stats = calculate_stats(cpu_samples);
    let memory_stats = calculate_stats(memory_samples);
    let (network_in_stats, network_out_stats) = calculate_network_stats(network_samples);

    let rows = vec![
        ("CPU Usage (%)", cpu_stats),
        ("Memory Usage (MB)", memory_stats),
        ("Network In (KB/s)", network_in_stats),
        ("Network Out (KB/s)", network_out_stats),
    ];

    for (resource, stats) in rows {
        table.add_row(Row::new(vec![
            Cell::new(resource),
            Cell::new(&format!("{:.2}", stats.min)),
            Cell::new(&format!("{:.2}", stats.max)),
            Cell::new(&format!("{:.2}", stats.avg)),
        ]));
    }

    table.to_string()
}

#[derive(Debug, Clone, Copy)]
struct Stats {
    min: f64,
    max: f64,
    avg: f64,
}

fn calculate_stats(samples: &[f64]) -> Stats {
    samples.iter().fold(Stats { min: f64::INFINITY, max: f64::NEG_INFINITY, avg: 0.0 }, |acc, &x| {
        Stats {
            min: acc.min.min(x),
            max: acc.max.max(x),
            avg: acc.avg + x / samples.len() as f64,
        }
    })
}

fn calculate_network_stats(samples: &[(f64, f64)]) -> (Stats, Stats) {
    let (in_samples, out_samples): (Vec<f64>, Vec<f64>) = samples.iter().cloned().unzip();
    (calculate_stats(&in_samples), calculate_stats(&out_samples))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_format_test_results() {
        let summary = MetricsSummary::new(
            100,
            Duration::from_secs(10),
            Some(Duration::from_millis(10)),
            Some(Duration::from_millis(1000)),
            Some(Duration::from_millis(50)),
            Some(Duration::from_millis(950)),
            &[200, 200, 200, 200, 200, 200, 200, 200, 200, 200, 404, 404, 404, 404, 404],
        );

        let cpu_samples = vec![10.0, 20.0, 30.0];
        let memory_samples = vec![100.0, 200.0, 300.0];
        let network_samples = vec![(1000.0, 2000.0), (1500.0, 2500.0), (2000.0, 3000.0)];

        let result = format_test_results(
            Some(&summary),
            Some(Duration::from_secs(60)),
            &cpu_samples,
            &memory_samples,
            &network_samples,
        );

        assert!(result.contains("Total Requests"));
        assert!(result.contains("Requests/second"));
        assert!(result.contains("Total duration: 1m 0s"));
        assert!(result.contains("CPU Usage (%)"));
        assert!(result.contains("Memory Usage (MB)"));
        assert!(result.contains("Network In (KB/s)"));
        assert!(result.contains("Network Out (KB/s)"));
    }
}