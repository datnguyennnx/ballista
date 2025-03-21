pub mod model;
pub mod view;
pub mod controller;
pub mod http;

// Import the error types from model
pub use model::error::AppError;

// Create a prelude module for commonly used items
pub mod prelude {
    // Re-export from model
    pub use super::model::error::AppError;
    pub use super::model::test::{
        TestConfig, TestResult, TestStatus, TestType, TestMetrics, TestUpdate, ApiTest
    };
    pub use super::model::metrics::{Metrics, new_metrics, add_request, calculate_summary};
    pub use super::model::utils::formatters::format_duration;
    
    // Re-export from view
    pub use super::view::response::{ApiResponse, create_api_response};
    
    // Common imports
    pub use std::result::Result;
    pub use std::future::Future;
    pub use async_trait::async_trait;
    pub use std::sync::Arc;
    pub use tokio::time::Duration;
    pub use serde::{Serialize, Deserialize};
    pub use serde_json::{self, Value, json};
}

// Functional programming utilities
// Function to compose multiple functions
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

// Function to partially apply a function
pub fn partial<A, B, C, F>(f: F, b: B) -> impl Fn(A) -> C
where
    F: Fn(A, B) -> C,
    B: Clone,
{
    move |a| f(a, b.clone())
}

// Higher-order function to map over a Result
pub fn map_result<T, E, U, F>(result: Result<T, E>, f: F) -> Result<U, E>
where
    F: FnOnce(T) -> U,
{
    result.map(f)
}

// Higher-order function to flat_map over a Result
pub fn and_then<T, E, U, F>(result: Result<T, E>, f: F) -> Result<U, E>
where
    F: FnOnce(T) -> Result<U, E>,
{
    result.and_then(f)
}

// Curry function to transform a function with multiple arguments into a chain of functions with single arguments
pub fn curry<A, B, C, F>(f: F) -> impl Fn(A) -> Box<dyn Fn(B) -> C>
where
    F: Fn(A, B) -> C + Copy + 'static,
    A: Clone + 'static,
    B: 'static,
    C: 'static,
{
    move |a| {
        let f = f;
        let a = a.clone();
        Box::new(move |b| f(a.clone(), b))
    }
}

// Function to apply a function n times
pub fn apply_n_times<T, F>(x: T, n: usize, f: F) -> T
where
    F: Fn(T) -> T,
{
    (0..n).fold(x, |acc, _| f(acc))
}

// Function to create a memoized version of a function
pub fn memoize<A, B, F>(mut f: F) -> impl FnMut(A) -> B
where
    F: FnMut(A) -> B,
    A: std::hash::Hash + Eq + Clone,
    B: Clone,
{
    let mut cache = std::collections::HashMap::new();
    move |arg: A| {
        cache
            .entry(arg.clone())
            .or_insert_with(|| f(arg.clone()))
            .clone()
    }
}

// Function to create a pipeline of functions
pub fn pipeline<T>(initial: T) -> Pipeline<T> {
    Pipeline { value: initial }
}

pub struct Pipeline<T> {
    value: T,
}

impl<T> Pipeline<T> {
    pub fn apply<F, U>(self, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> U,
    {
        Pipeline { value: f(self.value) }
    }

    pub fn end(self) -> T {
        self.value
    }
}