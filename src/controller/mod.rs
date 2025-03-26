pub mod router;
pub mod health;
pub mod websocket;

mod test_common;
mod test_operations;
mod api_test_controller;
mod load_test_controller;
mod stress_test_controller;

// Re-export the router for main.rs
pub use router::create_router;
pub use api_test_controller::start_api_test;
pub use load_test_controller::start_load_test;
pub use stress_test_controller::start_stress_test;
pub use test_operations::get_all_test_results; 