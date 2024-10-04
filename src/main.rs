mod app;

use target_tool::core::{
    config::parse_arguments,
    error::AppError,
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = parse_arguments().await?;
    app::run_application(args).await.map_err(|e| {
        eprintln!("Error: {}", e);
        e
    })
}
