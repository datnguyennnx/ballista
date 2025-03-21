use serde::{Serialize, Deserialize};

/// Standard API response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

/// Create a standard API response with optional data
pub fn create_api_response<T>(success: bool, message: String, data: Option<T>) -> ApiResponse<T> {
    ApiResponse {
        success,
        message,
        data,
    }
}

/// Create a success response
pub fn create_success_response<T>(message: String, data: Option<T>) -> ApiResponse<T> {
    create_api_response(true, message, data)
}

/// Create an error response
pub fn create_error_response<T>(message: String) -> ApiResponse<T> {
    create_api_response(false, message, None)
}

/// Create a paginated response
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Vec<T>,
    pub pagination: Pagination,
}

/// Pagination information
#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

/// Create a paginated response
pub fn create_paginated_response<T>(
    message: String,
    data: Vec<T>,
    total: usize,
    page: usize,
    per_page: usize,
) -> PaginatedResponse<T> {
    let total_pages = (total as f32 / per_page as f32).ceil() as usize;
    
    PaginatedResponse {
        success: true,
        message,
        data,
        pagination: Pagination {
            total,
            page,
            per_page,
            total_pages,
        },
    }
} 