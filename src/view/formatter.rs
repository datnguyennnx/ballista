use std::time::Duration;
use crate::model::metrics::MetricsSummary;
use crate::model::test::{TestMetrics, TestResult, TestType, TestStatus};

/// Format test results for display
pub fn format_test_results(summary: Option<&MetricsSummary>, duration: Option<Duration>) -> String {
    let mut output = String::new();
    output.push_str("\n=== Test Results ===\n");
    
    if let Some(summary) = summary {
        output.push_str(&format!("Total requests: {}\n", summary.total_requests));
        output.push_str(&format!("Successful requests: {}\n", summary.successful_requests));
        output.push_str(&format!("Failed requests: {}\n", summary.failed_requests));
        output.push_str(&format!("Average response time: {:.2} ms\n", summary.avg_time_ms));
        output.push_str(&format!("Min response time: {:.2} ms\n", summary.min_time_ms));
        output.push_str(&format!("Max response time: {:.2} ms\n", summary.max_time_ms));
        
        output.push_str("\nStatus code distribution:\n");
        for (status, count) in &summary.status_codes {
            output.push_str(&format!("  {}: {}\n", status, count));
        }
    }
    
    if let Some(duration) = duration {
        output.push_str(&format!("\nTotal duration: {:.2} seconds\n", duration.as_secs_f64()));
    }
    
    output
}

/// Format metrics for API response
pub fn format_metrics(metrics: &TestMetrics) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("Requests: {}/{}\n", metrics.requests_completed, metrics.total_requests));
    output.push_str(&format!("Average response time: {:.2} ms\n", metrics.avg_response_time));
    
    if let Some(min) = metrics.min_response_time {
        output.push_str(&format!("Min response time: {:.2} ms\n", min));
    }
    
    if let Some(max) = metrics.max_response_time {
        output.push_str(&format!("Max response time: {:.2} ms\n", max));
    }
    
    if let Some(median) = metrics.median_response_time {
        output.push_str(&format!("Median response time: {:.2} ms\n", median));
    }
    
    if let Some(p95) = metrics.p95_response_time {
        output.push_str(&format!("95th percentile: {:.2} ms\n", p95));
    }
    
    output.push_str("\nStatus code distribution:\n");
    for (status, count) in &metrics.status_codes {
        output.push_str(&format!("  {}: {}\n", status, count));
    }
    
    output.push_str(&format!("Errors: {}\n", metrics.errors));
    
    output
}

/// Format a test result for display
pub fn format_test_result(result: &TestResult) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("Test ID: {}\n", result.id));
    output.push_str(&format!("Type: {:?}\n", result.test_type));
    output.push_str(&format!("Status: {:?}\n", result.status));
    output.push_str(&format!("Timestamp: {}\n", 
        chrono::DateTime::from_timestamp(result.timestamp, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "Invalid timestamp".to_string())
    ));
    
    if let Some(metrics) = &result.metrics {
        output.push_str("\nMetrics:\n");
        output.push_str(&format_metrics(metrics));
    }
    
    if let Some(error) = &result.error {
        output.push_str(&format!("\nError: {}\n", error));
    }
    
    output
}

/// Format test type to string
pub fn format_test_type(test_type: TestType) -> &'static str {
    match test_type {
        TestType::Load => "Load Test",
        TestType::Stress => "Stress Test",
        TestType::Api => "API Test",
    }
}

/// Format test status to string
pub fn format_test_status(status: TestStatus) -> &'static str {
    match status {
        TestStatus::Started => "Started",
        TestStatus::Running => "Running",
        TestStatus::Completed => "Completed",
        TestStatus::Error => "Error",
    }
} 