# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-10-06

### Fixed

-   Resolved type mismatch issues in `src/main.rs` between different `Args` and `Command` types
-   Implemented a `convert_command` function in `src/main.rs` to handle differences between `core::config::Command` and `args::Command`
-   Updated the `parse_args` function in `src/main.rs` to use the new conversion function
-   Fixed compilation errors related to mismatched `Args` types
-   Resolved runtime panic issue in `src/core/api_test_runner.rs` caused by nested runtime creation
-   Fixed type mismatch errors in the `run_api_tests` function

### Changed

-   Refactored `src/main.rs` to use consistent `Args` and `Command` types throughout the file
-   Updated error handling in `src/main.rs` to use the `AppError` type consistently
-   Refactored `run_api_tests` function in `src/core/api_test_runner.rs` to properly handle async operations
-   Simplified the async flow in `run_api_tests` to ensure correct handling of futures

### Improved

-   Enhanced code modularity and type safety in the main application flow
-   Improved error handling and type consistency across the codebase
-   Enhanced error handling in the API test runner
-   Improved adherence to functional programming principles in `src/core/api_test_runner.rs`

### Added

-   Implemented functional programming principles throughout the codebase
-   Added new pure functions for core logic in various modules
-   Introduced higher-order functions and function composition in key areas

### Changed

-   Refactored the entire codebase to follow functional programming paradigms
-   Updated `src/metrics/collector.rs` to use immutable data structures
-   Refactored `src/http/client.rs` to use pure functions instead of methods
-   Modified `src/monitoring/resource.rs` to follow functional principles
-   Restructured `src/core/test_runner.rs` to use function composition
-   Updated `src/app.rs` to use a more functional approach
-   Refactored `src/lib.rs` to include functional programming utilities in the prelude
-   Modified `src/core/api_test_runner.rs` to use pure functions and avoid side effects
-   Updated `src/output/printer.rs` to return formatted strings instead of printing directly
-   Refactored `src/main.rs` to use function composition for the main application flow

### Improved

-   Enhanced code modularity, testability, and maintainability through functional programming techniques
-   Reduced side effects across the codebase, making it easier to reason about and test
-   Improved error handling using functional patterns

### Updated

-   README.md now includes information about the functional programming approach used in the project
