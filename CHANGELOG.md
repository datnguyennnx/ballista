# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-03-25

### Breaking Changes
- Completely refactored project structure to follow MVC architecture pattern
- Reorganized code into model, view, and controller components
- Moved type definitions into model directory
- Restructured HTTP handling into controller layer

### Added
- New folder structure with dedicated directories:
  - `model/`: All data structures and business logic
  - `controller/`: Request handling and business logic coordination
  - `http/`: HTTP client and server implementations
  - `view/`: Response formatting and presentation logic
- Improved separation of concerns with clear component boundaries
- Enhanced testability with better component isolation
- Stronger type safety with centralized type definitions

### Changed
- Moved test types from custom types module to model/test module
- Refactored request/response handling into the controller layer
- Enhanced metrics collection with improved data structures
- Updated HTTP client with better error handling
- Improved WebSocket handling for real-time test updates

### Improved
- Better code organization with clear architectural boundaries
- Enhanced maintainability through separation of concerns
- Improved testability with isolated components
- Clearer responsibility allocation across modules
- Stronger adherence to functional programming principles within MVC structure

### Fixed
- Resolved import conflicts with clearer module boundaries
- Fixed naming inconsistencies across the codebase
- Corrected type mismatches through centralized type definitions
- Improved error handling with clear error propagation paths

## [0.3.2] - 2025-03-21

### Fixed
- Fixed type mismatches in test router imports
- Corrected WebSocket message handling with proper types
- Fixed field naming consistency (test_id -> id)
- Resolved compilation errors in test handlers

### Added
- Enhanced test metrics structure with detailed statistics:
  - Added min/max response times
  - Added median and p95 response times
  - Added status code tracking
  - Added total requests counter
- Implemented proper enum types for test status and types
- Added timestamp tracking for all test updates
- Added structured error handling in test updates

### Improved
- Standardized test configuration types across the application
- Enhanced WebSocket message handling with type safety
- Improved error handling in test execution
- Better metrics collection and reporting
- Cleaner type definitions and interfaces

### Technical Debt
- Removed redundant test response types
- Standardized metric field types (f32 -> f64)
- Cleaned up unused imports
- Improved code organization in test router

# [0.3.1] - 2025-03-20

### Fixed

- Resolved test case failures in API test runner and URL parser
- Fixed file path handling for test examples
- Corrected XML parsing in URL parser
- Improved test file resolution logic
- Fixed test assertions and error messages

### Added

- New test_examples directory for sample files:
  - sample_load_test.json
  - sample_restfulAPI_test.json
- Enhanced test file handling with fallback paths
- Better error messages in test assertions
- Improved test coverage for API functionality

### Changed

- Updated file reading logic to support test_examples directory
- Refactored test cases to use actual sample files
- Enhanced test assertions with detailed error messages
- Improved test file path resolution strategy

### Improved

- Better test file organization with dedicated examples directory
- Enhanced error reporting in test cases
- More robust file path handling
- Clearer test failure messages
- Stronger test coverage for API functionality

### Technical Debt

- Removed temporary file creation in tests
- Standardized test file handling
- Improved test organization
- Enhanced test maintainability


## [0.3.0] - 2025-03-20

### Breaking Changes

- Removed CLI functionality in favor of API-only interface
- Removed command-line argument parsing and related functionality
- Removed CLI-specific modules and types:
  - Removed `args` module
  - Removed `Command` enum and related types
  - Removed CLI-specific error handling

### Added

- Implemented new REST API using Axum framework
- Added CORS support for cross-origin requests
- Created new API endpoints:
  - `/api/health` for health checks
  - `/api/tests` for retrieving test results
  - `/api/load-test` for running load tests
  - `/api/stress-test` for running stress tests
  - `/api/api-test` for running API tests
- Introduced shared application state management using `AppState`
- Added test result tracking and history
- Implemented asynchronous test execution with status updates
- Added new DTOs for API requests and responses
- Created pure functions for API response handling

### Changed

- Converted application to API-only service
- Removed CLI interface and related code:
  - Removed `convert_command` function
  - Removed `parse_args` function
  - Removed `validate_args` function
  - Removed `run_app` function
  - Removed `app_flow` function
- Simplified main.rs to focus solely on API server
- Updated error handling to focus on API-specific errors
- Removed CLI-related dependencies
- Rename Target-tool to Ballista.

### Improved

- Enhanced modularity by separating API concerns from core functionality
- Strengthened type safety with proper DTO structures
- Improved error handling with consistent API responses
- Enhanced test result tracking and reporting
- Better adherence to functional programming principles in API layer:
  - Pure functions for response creation
  - Immutable state handling
  - Clear separation of concerns
  - Function composition for request handling
- Enhanced error handling in main.rs for server startup
- Simplified application bootstrap process
- Reduced code complexity by removing unused CLI logic
- Simplified codebase by removing dual-mode (CLI/API) complexity
- Enhanced focus on API functionality
- Cleaner architecture with single responsibility

### Fixed

- Resolved runtime issues with Tokio async execution
- Fixed CORS configuration for proper cross-origin support
- Corrected type mismatches in API handlers
- Resolved issues with concurrent test execution
- Fixed state management in async operations

### Technical Debt

- Removed duplicate endpoint definitions
- Cleaned up unused imports
- Standardized error handling across API endpoints
- Improved code organization in server.rs
- Removed unused functions:
  - Removed redundant `tool_info` endpoint
  - Removed duplicate `run_test` handler
  - Cleaned up unused CLI-related functions
- Simplified main application flow to focus on API server
- Removed unused types and structs
- Removed legacy CLI code from main application entry point
- Streamlined main.rs to single responsibility (API server)
- Removed all CLI-related code
- Simplified application architecture
- Reduced complexity by focusing on single interface (API)
- Removed unused CLI-related dependencies

### Migration Guide

Users previously using the CLI interface should now:
1. Use the REST API endpoints instead
2. Migrate CLI scripts to API calls
3. Use the following endpoint mappings:
   - Load Test: POST /api/load-test
   - Stress Test: POST /api/stress-test
   - API Test: POST /api/api-test

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

