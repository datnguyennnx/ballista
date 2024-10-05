use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum AppError {
    ArgValidation(String),
    NoUrls,
    Util(String),
    Other(String),
    FileError(String),
    ParseError(String),
    ThresholdError(String),
    CheckError(String),
    InvalidConfig(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_message = match self {
            AppError::ArgValidation(msg) => format!("Argument validation error: {}", msg),
            AppError::NoUrls => "No valid URLs found to test".to_string(),
            AppError::Util(e) => format!("Utility error: {}", e),
            AppError::Other(e) => format!("Error: {}", e),
            AppError::FileError(msg) => format!("File error: {}", msg),
            AppError::ParseError(msg) => format!("Parse error: {}", msg),
            AppError::ThresholdError(msg) => format!("Threshold error: {}", msg),
            AppError::CheckError(msg) => format!("Check error: {}", msg),
            AppError::InvalidConfig(msg) => format!("Invalid configuration: {}", msg),
        };
        write!(f, "{}", error_message)
    }
}

impl Error for AppError {}

// Higher-order function to convert any error type to AppError
pub fn to_app_error<E, F>(error_constructor: F) -> impl Fn(E) -> AppError
where
    E: Error + 'static,
    F: Fn(String) -> AppError,
{
    move |error: E| error_constructor(error.to_string())
}

// Utility functions for creating specific AppErrors
pub fn arg_validation_error(msg: &str) -> AppError {
    AppError::ArgValidation(msg.to_string())
}

pub fn no_urls_error() -> AppError {
    AppError::NoUrls
}

pub fn util_error(msg: &str) -> AppError {
    AppError::Util(msg.to_string())
}

pub fn file_error(msg: &str) -> AppError {
    AppError::FileError(msg.to_string())
}

pub fn parse_error(msg: &str) -> AppError {
    AppError::ParseError(msg.to_string())
}

pub fn threshold_error(msg: &str) -> AppError {
    AppError::ThresholdError(msg.to_string())
}

pub fn check_error(msg: &str) -> AppError {
    AppError::CheckError(msg.to_string())
}

pub fn invalid_config_error(msg: &str) -> AppError {
    AppError::InvalidConfig(msg.to_string())
}

// Function to chain multiple error conversions
pub fn chain_errors<T, E1, E2>(
    result: Result<T, E1>,
    error_converter: impl Fn(E1) -> E2,
) -> Result<T, E2> {
    result.map_err(error_converter)
}

// Monadic operations for AppError
impl AppError {
    pub fn and_then<F>(self, op: F) -> AppError
    where
        F: FnOnce(AppError) -> AppError,
    {
        op(self)
    }

    pub fn map<F>(self, op: F) -> AppError
    where
        F: FnOnce(String) -> String,
    {
        match self {
            AppError::ArgValidation(msg) => AppError::ArgValidation(op(msg)),
            AppError::Util(msg) => AppError::Util(op(msg)),
            AppError::Other(msg) => AppError::Other(op(msg)),
            AppError::FileError(msg) => AppError::FileError(op(msg)),
            AppError::ParseError(msg) => AppError::ParseError(op(msg)),
            AppError::ThresholdError(msg) => AppError::ThresholdError(op(msg)),
            AppError::CheckError(msg) => AppError::CheckError(op(msg)),
            AppError::InvalidConfig(msg) => AppError::InvalidConfig(op(msg)),
            AppError::NoUrls => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_app_error() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_error = to_app_error(AppError::FileError)(error);
        assert!(matches!(app_error, AppError::FileError(_)));
    }

    #[test]
    fn test_chain_errors() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"));
        let app_result = chain_errors(result, to_app_error(AppError::FileError));
        assert!(matches!(app_result, Err(AppError::FileError(_))));
    }

    #[test]
    fn test_app_error_and_then() {
        let error = AppError::ArgValidation("Invalid argument".to_string());
        let result = error.and_then(|e| e.map(|msg| format!("Error: {}", msg)));
        assert!(matches!(result, AppError::ArgValidation(msg) if msg == "Error: Invalid argument"));
    }
}