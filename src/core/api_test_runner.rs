use std::fs;
use std::path::Path;

use crate::api::{ReqwestClient, run_tests, load_tests_from_json, analyze_results};
use crate::api::types::TestResult;
use crate::core::error::{AppError, to_app_error};
use crate::utils::formatters::format_duration;

async fn load_and_run_tests(api_test_path: &Path) -> Result<Vec<TestResult>, AppError> {
    let json_content = fs::read_to_string(api_test_path).map_err(to_app_error)?;
    let tests = load_tests_from_json(&json_content).map_err(to_app_error)?;
    let client = ReqwestClient::new();
    Ok(run_tests(&client, &tests).await)
}

fn print_summary(total: usize, successful: usize, total_duration: std::time::Duration) {
    println!("\nAPI Test Results");
    println!("Total tests: {}", total);
    println!("Successful tests: {}", successful);
    println!("Total duration: {}", format_duration(total_duration));
}

fn print_test_results(results: &[TestResult]) {
    for result in results {
        println!("\nTest: {}", result.name);
        println!("Success: {}", result.success);
        println!("Duration: {}", format_duration(result.duration));
        println!("Status: {}", result.status);
        if let Some(error) = &result.error {
            println!("Error: {}", error);
        }
    }
}

pub async fn run_api_tests(api_test_path: &str) -> Result<(), AppError> {
    let results = load_and_run_tests(Path::new(api_test_path)).await?;
    let (total, successful, total_duration) = analyze_results(&results);
    
    print_summary(total, successful, total_duration);
    print_test_results(&results);

    Ok(())
}