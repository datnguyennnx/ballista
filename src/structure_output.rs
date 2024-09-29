use prettytable::{Table, Row, Cell, format};
use std::time::Duration;
use crate::metrics::MetricsSummary;
use colored::{Colorize, ColoredString};

pub fn print_test_results(
    summary: &MetricsSummary,
    total_duration: Duration,
    cpu_samples: &[f64],
    memory_samples: &[f64],
    network_samples: &[(f64, f64)] // Add network samples (received, sent)
) {
    println!("\nTest Results");

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Total requests"),
        Cell::new("Total time"),
        Cell::new("Requests per second"),
        Cell::new("Average response time"),
        Cell::new("Minimum response time"),
        Cell::new("Maximum response time"),
        Cell::new("Median response time"),
        Cell::new("95th percentile response time")
    ]));

    table.add_row(Row::new(vec![
        Cell::new(&summary.total_requests.to_string()),
        Cell::new(&format!("{:.2?}", total_duration)),
        Cell::new(&format!("{:.2}", summary.total_requests as f64 / total_duration.as_secs_f64())),
        Cell::new(&format!("{:.2?}", summary.total_time / summary.total_requests)),
        Cell::new(&summary.min_duration.map_or("N/A".to_string(), |d| format!("{:.2?}", d))),
        Cell::new(&summary.max_duration.map_or("N/A".to_string(), |d| format!("{:.2?}", d))),
        Cell::new(&summary.median_duration.map_or("N/A".to_string(), |d| format!("{:.2?}", d))),
        Cell::new(&summary.percentile_95.map_or("N/A".to_string(), |d| format!("{:.2?}", d)))
    ]));

    table.printstd();

    println!("\nHTTP Status Codes");
    let mut status_table = Table::new();
    status_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    status_table.set_titles(Row::new(vec![
        Cell::new("Status Code").style_spec("b"),
        Cell::new("Count").style_spec("b")
    ]));
    for (status, count) in &summary.status_codes {
        status_table.add_row(Row::new(vec![
            Cell::new(&status_color(*status).to_string()).style_spec("r"),
            Cell::new(&count.to_string()).style_spec("r")
        ]));
    }
    status_table.printstd();

    println!("\nResource Usage");
    let mut resource_table = Table::new();
    resource_table.add_row(Row::new(vec![
        Cell::new("Metric"),
        Cell::new("Average"),
        Cell::new("Maximum"),
        Cell::new("Total")
    ]));
    
    // Assuming 8 cores for CPU usage calculation
    let cpu_cores = 8.0;
    add_resource_row(&mut resource_table, "CPU Usage", cpu_samples, 
        |v| format!("{:.2}% ({:.2} cores)", v, v * cpu_cores / 100.0),
        |v| format!("{:.2}% ({:.2} cores)", v, v * cpu_cores / 100.0),
        None::<fn(f64) -> String>);
    
    // Assuming 16 GB total memory for memory usage calculation
    let total_memory_gb = 16.0;
    add_resource_row(&mut resource_table, "Memory Usage", memory_samples,
        |v| format!("{:.2}% ({:.2} GB)", v, v * total_memory_gb / 100.0),
        |v| format!("{:.2}% ({:.2} GB)", v, v * total_memory_gb / 100.0),
        None::<fn(f64) -> String>);
    
    let network_received: Vec<f64> = network_samples.iter().map(|&(r, _)| r).collect();
    let network_sent: Vec<f64> = network_samples.iter().map(|&(_, s)| s).collect();
    
    add_resource_row(&mut resource_table, "Network Received", &network_received, 
        |v| format!("{:.2} MB/s", v),
        |v| format!("{:.2} MB/s", v),
        Some(|v| format!("{:.2} MB", v)));
    add_resource_row(&mut resource_table, "Network Sent", &network_sent, 
        |v| format!("{:.2} MB/s", v),
        |v| format!("{:.2} MB/s", v),
        Some(|v| format!("{:.2} MB", v)));

    resource_table.printstd();
}

fn add_resource_row<F, G, H>(
    table: &mut Table,
    metric_name: &str,
    samples: &[f64],
    avg_format_fn: F,
    max_format_fn: G,
    total_format_fn: Option<H>
) where
    F: Fn(f64) -> String,
    G: Fn(f64) -> String,
    H: Fn(f64) -> String,
{
    if !samples.is_empty() {
        let avg = samples.iter().sum::<f64>() / samples.len() as f64;
        let max = samples.iter().fold(f64::NEG_INFINITY, |m, &v| m.max(v));
        let total = samples.iter().sum::<f64>();
        
        let total_cell = match total_format_fn {
            Some(ref f) => Cell::new(&f(total)),
            None => Cell::new("N/A"),
        };
        
        table.add_row(Row::new(vec![
            Cell::new(metric_name),
            Cell::new(&avg_format_fn(avg)),
            Cell::new(&max_format_fn(max)),
            total_cell,
        ]));
    } else {
        table.add_row(Row::new(vec![
            Cell::new(metric_name),
            Cell::new("N/A"),
            Cell::new("N/A"),
            Cell::new("N/A"),
        ]));
    }
}

fn status_color(status: u16) -> ColoredString {
    match status {
        200..=299 => status.to_string().green(),
        300..=399 => status.to_string().yellow(),
        400..=599 => status.to_string().red(),
        _ => status.to_string().normal(),
    }
}