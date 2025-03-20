use crate::prelude::*;
use crate::metrics::summary::MetricsSummary;
use prettytable::{Table, Row, Cell};

pub fn format_test_results(
    summary: Option<&MetricsSummary>,
    total_duration: Option<Duration>,
) -> String {
    let mut output = String::new();

    if let Some(summary) = summary {
        output.push_str(&format_summary(summary));
    }

    if let Some(duration) = total_duration {
        output.push_str(&format!("\nTotal duration: {}\n", format_duration(duration)));
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_test_results() {
        let summary = MetricsSummary::new(
            100,
            Duration::from_secs(10),
            Some(Duration::from_millis(10)),
            Some(Duration::from_millis(100)),
            Some(Duration::from_millis(50)),
            Some(Duration::from_millis(90)),
            &[200, 200, 404]
        );

        let result = format_test_results(
            Some(&summary),
            Some(Duration::from_secs(60)),
        );

        assert!(result.contains("Total Requests"));
    }
}