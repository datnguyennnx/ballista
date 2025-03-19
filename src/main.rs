use target_tool::prelude::*;
use target_tool::args::{Args, Command, args_from_json};
use target_tool::core::config::parse_arguments;
use target_tool::core::test_runner::{run_load_test, run_stress_test};
use target_tool::core::api_test_runner::run_api_tests;
use std::path::Path;
use tokio::runtime::Runtime;

// Pure function to convert core::Command to args::Command
fn convert_command(core_command: target_tool::args::Command) -> Command {
    match core_command {
        target_tool::args::Command::LoadTest { url, requests, concurrency } => 
            Command::LoadTest { url, requests, concurrency },
        target_tool::args::Command::StressTest { sitemap, duration, concurrency } => 
            Command::StressTest { sitemap, duration, concurrency },
        target_tool::args::Command::ApiTest { path } => 
            Command::ApiTest { path },
    }
}

// Pure function to parse command-line arguments
async fn parse_args() -> Result<Args, AppError> {
    parse_arguments().await
        .map(|core_args| Args { command: convert_command(core_args.command) })
}

// Pure function to validate arguments
fn validate_args(args: Args) -> Result<Args, AppError> {
    args.validate()
        .map_err(|e| AppError::ArgValidation(e.to_string()))
        .map(|_| args)
}

// Pure function to run the application
async fn run_app(args: Args) -> Result<String, AppError> {
    match args.command {
        Command::LoadTest { url, requests, concurrency } => 
            run_load_test(&url, requests, concurrency)
                .await
                .map(|_| "Load test completed successfully".to_string()),
        Command::StressTest { sitemap, duration, concurrency } => 
            run_stress_test(&sitemap, duration, concurrency)
                .await
                .map(|_| "Stress test completed successfully".to_string()),
        Command::ApiTest { path } => 
            run_api_tests(&path)
                .await
                .map(|_| "API tests completed successfully".to_string()),
    }
}

// Function to compose the application flow
async fn app_flow() -> Result<(), AppError> {
    let args = match parse_args().await {
        Ok(args) => args,
        Err(_) => args_from_json(Path::new("config.json"))
            .map_err(|e| AppError::ArgValidation(format!("Failed to parse arguments from file: {}", e)))?,
    };

    let validated_args = validate_args(args)?;
    let result = run_app(validated_args).await?;
    println!("{}", result);
    Ok(())
}

// Main function using function composition
fn main() -> Result<(), AppError> {
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    
    if let Err(e) = runtime.block_on(app_flow()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}
