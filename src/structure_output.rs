use prettytable::{Table, Row, Cell, format};
use std::time::Duration;
use crate::metrics::MetricsSummary;
use colored::{Colorize, ColoredString};
use crate::utils::{format_duration, format_size};

pub fn print_test_results(
    summary: Option<&MetricsSummary>,
    total_duration: Option<Duration>,
    cpu_samples: &[f64],
    memory_samples: &[f64],
    network_samples: &[(f64, f64)]
) {
    if let Some(summary) = summary {
        print_performance_results(summary, total_duration.unwrap());
    }

    print_resource_usage(cpu_samples, memory_samples, network_samples);
}

fn print_performance_results(summary: &MetricsSummary, total_duration: Duration) {
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
        Cell::new(&format_duration(total_duration)),
        Cell::new(&format!("{:.2}", summary.requests_per_second())),
        Cell::new(&format_duration(summary.average_duration().unwrap_or_default())),
        Cell::new(&summary.min_duration.map_or("N/A".to_string(), format_duration)),
        Cell::new(&summary.max_duration.map_or("N/A".to_string(), format_duration)),
        Cell::new(&summary.median_duration.map_or("N/A".to_string(), format_duration)),
        Cell::new(&summary.percentile_95.map_or("N/A".to_string(), format_duration))
    ]));

    table.printstd();

    println!("\nHTTP Status Codes");
    let mut status_table = Table::new();
    status_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    status_table.set_titles(Row::new(vec![
        Cell::new("Status Code").style_spec("b"),
        Cell::new("Count").style_spec("b"),
        Cell::new("Percentage").style_spec("b")
    ]));
    for (status, count) in &summary.status_codes {
        let percentage = (*count as f64 / summary.total_requests as f64) * 100.0;
        status_table.add_row(Row::new(vec![
            Cell::new(&status_color(*status).to_string()).style_spec("r"),
            Cell::new(&count.to_string()).style_spec("r"),
            Cell::new(&format!("{:.2}%", percentage)).style_spec("r")
        ]));
    }
    status_table.printstd();

    println!("\nSuccess Rate: {:.2}%", summary.success_rate() * 100.0);
}

fn print_resource_usage(cpu_samples: &[f64], memory_samples: &[f64], network_samples: &[(f64, f64)]) {
    println!("\nResource Usage");
    let mut resource_table = Table::new();
    resource_table.add_row(Row::new(vec![
        Cell::new("Metric"),
        Cell::new("Average"),
        Cell::new("Maximum"),
        Cell::new("Total")
    ]));
    
    let cpu_cores = num_cpus::get() as f64;
    add_resource_row(&mut resource_table, "CPU Usage", cpu_samples, 
        |v| format!("{:.2}% ({:.2} cores)", v, v * cpu_cores / 100.0),
        |v| format!("{:.2}% ({:.2} cores)", v, v * cpu_cores / 100.0),
        None::<fn(f64) -> String>);
    
    let total_memory = sys_info::mem_info().map(|m| m.total as f64 / 1024.0).unwrap_or(0.0);
    add_resource_row(&mut resource_table, "Memory Usage", memory_samples,
        |v| format!("{:.2}% ({:.2} GB)", v, v * total_memory / 100.0),
        |v| format!("{:.2}% ({:.2} GB)", v, v * total_memory / 100.0),
        None::<fn(f64) -> String>);
    
    let network_received: Vec<f64> = network_samples.iter().map(|&(r, _)| r).collect();
    let network_sent: Vec<f64> = network_samples.iter().map(|&(_, s)| s).collect();
    
    add_resource_row(&mut resource_table, "Network Received", &network_received, 
        |v| format!("{}/s", format_size((v * 1_000_000.0) as u64)),
        |v| format!("{}/s", format_size((v * 1_000_000.0) as u64)),
        Some(|v| format_size((v * 1_000_000.0) as u64)));
    add_resource_row(&mut resource_table, "Network Sent", &network_sent, 
        |v| format!("{}/s", format_size((v * 1_000_000.0) as u64)),
        |v| format!("{}/s", format_size((v * 1_000_000.0) as u64)),
        Some(|v| format_size((v * 1_000_000.0) as u64)));

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