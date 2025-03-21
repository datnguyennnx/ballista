pub mod router;
pub mod health;
pub mod test;
pub mod websocket;

// Re-export the router for main.rs
pub use router::create_router; 