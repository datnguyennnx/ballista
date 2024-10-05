use colored::{Colorize, ColoredString};
use crate::utils::format_size;

// Higher-order function for status color mapping
pub fn status_color_mapper() -> impl Fn(u16) -> ColoredString {
    |status| match status {
        200..=299 => status.to_string().green(),
        300..=399 => status.to_string().yellow(),
        400..=599 => status.to_string().red(),
        _ => status.to_string().normal(),
    }
}

// Curried function for formatting CPU usage
pub fn format_cpu_usage(cpu_cores: f64) -> impl Fn(f64) -> String {
    move |value| format!("{:.2}% ({:.2} cores)", value, value * cpu_cores / 100.0)
}

// Curried function for formatting memory usage
pub fn format_memory_usage(total_memory: f64) -> impl Fn(f64) -> String {
    move |value| format!("{:.2}% ({:.2} GB)", value, value * total_memory / 100.0)
}

// Function for formatting network usage
pub fn format_network_usage(value: f64) -> String {
    format!("{}/s", format_size((value * 1_000_000.0) as u64))
}

// Higher-order function for generic value formatting
pub fn format_value<T, U, F>(value: T, formatter: F) -> U
where
    F: Fn(T) -> U,
{
    formatter(value)
}

// Composition function for chaining formatters
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_color() {
        let color_mapper = status_color_mapper();
        assert_eq!(color_mapper(200).to_string(), "200".green().to_string());
        assert_eq!(color_mapper(302).to_string(), "302".yellow().to_string());
        assert_eq!(color_mapper(404).to_string(), "404".red().to_string());
        assert_eq!(color_mapper(600).to_string(), "600");
    }

    #[test]
    fn test_format_cpu_usage() {
        let cpu_formatter = format_cpu_usage(4.0);
        assert_eq!(cpu_formatter(50.0), "50.00% (2.00 cores)");
    }

    #[test]
    fn test_format_memory_usage() {
        let memory_formatter = format_memory_usage(16.0);
        assert_eq!(memory_formatter(75.0), "75.00% (12.00 GB)");
    }

    #[test]
    fn test_format_network_usage() {
        assert_eq!(format_network_usage(1.5), "1.50 MB/s");
    }

    #[test]
    fn test_format_value() {
        let double = |x: i32| x * 2;
        assert_eq!(format_value(5, double), 10);
    }

    #[test]
    fn test_compose() {
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;
        let add_one_then_double = compose(add_one, double);
        assert_eq!(add_one_then_double(5), 12);
    }
}