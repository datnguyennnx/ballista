use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ArgValidation(String),
    NoUrls,
    Util(Box<dyn Error>),
    Other(Box<dyn Error>),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ArgValidation(msg) => write!(f, "Argument validation error: {}", msg),
            AppError::NoUrls => write!(f, "No valid URLs found to test"),
            AppError::Util(e) => write!(f, "Utility error: {}", e),
            AppError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Util(e) | AppError::Other(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl From<Box<dyn Error>> for AppError {
    fn from(error: Box<dyn Error>) -> Self {
        AppError::Other(error)
    }
}

// Helper function to convert any error type to AppError
pub fn to_app_error<E: Error + 'static>(error: E) -> AppError {
    AppError::Other(Box::new(error))
}