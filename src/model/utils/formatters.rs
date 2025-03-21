use std::time::Duration;

/// Format a duration in a human-readable format
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    let micros = duration.subsec_micros() % 1000;
    
    if secs == 0 {
        if millis == 0 {
            format!("{}µs", micros)
        } else {
            format!("{}ms {}µs", millis, micros)
        }
    } else if secs < 60 {
        format!("{}s {}ms", secs, millis)
    } else {
        let minutes = secs / 60;
        let remaining_secs = secs % 60;
        format!("{}m {}s", minutes, remaining_secs)
    }
}

/// Format a size in bytes to a human-readable format
pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else {
        format!("{:.2} GB", size as f64 / GB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_micros(500)), "500µs");
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms 0µs");
        assert_eq!(format_duration(Duration::from_secs(5)), "5s 0ms");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
        assert_eq!(format_duration(Duration::new(3661, 500_000_000)), "61m 1s");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1500), "1.46 KB");
        assert_eq!(format_size(1500000), "1.43 MB");
        assert_eq!(format_size(1500000000), "1.40 GB");
    }
}