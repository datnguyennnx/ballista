use colored::{Colorize, ColoredString};

// Higher-order function for status color mapping
pub fn status_color_mapper() -> impl Fn(u16) -> ColoredString {
    |status| match status {
        200..=299 => status.to_string().green(),
        300..=399 => status.to_string().yellow(),
        400..=599 => status.to_string().red(),
        _ => status.to_string().normal(),
    }
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