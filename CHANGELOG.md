# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.6] - 2025-03-26

### Frontend Changes
- Enhanced WebSocket client reliability:
  - Improved error handling with silent error recovery
  - Removed console error logging for better UX
  - Enhanced connection state management
  - Improved reconnection logic
- Enhanced metrics visualization:
  - Added NumberTicker component for animated metrics
  - Improved MetricCard component with React Node support
  - Enhanced real-time updates for all numeric values
  - Added smooth animations for metric changes

### Backend Changes
- Improved WebSocket server stability:
  - Enhanced error handling in test controllers
  - Improved connection management
  - Better handling of disconnections
  - Enhanced message queue processing

### Technical Improvements
- Better type safety in frontend components
- Enhanced error handling across the application
- Improved user experience with animated metrics
- Better state management in WebSocket connections

### Fixed
- Resolved WebSocket connection stability issues
- Fixed type mismatches in MetricCard components
- Corrected NumberTicker integration
- Improved error handling in WebSocket client

## [0.4.5] - 2025-03-26

### Frontend Changes
- Enhanced real-time metrics visualization with new chart components:
  - Implemented responsive area charts for metrics display
  - Added support for dynamic thresholds and warning indicators
  - Introduced live status indicators for connection state
  - Added min/max/avg statistics display
- Improved dashboard layout and organization:
  - Created tabbed interface for different metric views
  - Added fullscreen mode for detailed analysis
  - Implemented grid layout for better space utilization
  - Enhanced mobile responsiveness
- Enhanced data handling and types:
  - Added proper TypeScript interfaces for metrics data
  - Implemented time series data processing
  - Added concurrent users calculation
  - Improved data formatting and display
- Added visual feedback features:
  - Status indicators for test progress
  - Connection state visualization
  - Warning and critical thresholds
  - Real-time data point highlighting
- Implemented theme support:
  - Added CSS variables for chart colors
  - Created consistent color scheme across components
  - Enhanced dark/light mode compatibility

### Backend Changes
- Refactored WebSocket handling for better real-time updates:
  - Improved connection management
  - Enhanced error handling
  - Added support for time series data streaming
- Enhanced test metrics collection:
  - Added detailed statistics tracking
  - Improved concurrent user calculation
  - Enhanced error rate monitoring
  - Added response time analysis
- Improved data structures and types:
  - Added TimeSeriesPoint interface
  - Enhanced test metrics types
  - Improved error handling types
  - Added proper status enums

### Technical Improvements
- Enhanced type safety across the application
- Improved error handling and reporting
- Better separation of concerns in components
- Enhanced code reusability
- Improved performance in real-time updates
- Better state management in frontend
- Enhanced WebSocket communication
- Improved data flow consistency

### Fixed
- Resolved chart rendering issues
- Fixed data type mismatches between frontend and backend
- Corrected WebSocket connection handling
- Fixed metrics calculation accuracy
- Resolved theme inconsistencies
- Fixed fullscreen mode behavior
- Corrected time series data processing

## [0.4.4] - 2025-03-26

### Changed
- Refactored API test controller (`api_test_controller.rs`) for concurrency using `futures::stream` (`buffer_unordered`, `fold`) and incremental metric updates via `TestContext`.
- Refactored Load and Stress test controllers (`load_test_controller.rs`, `stress_test_controller.rs`) to use `mpsc` channels and dedicated aggregator tasks for metric collection, removing the `Arc<Mutex<TestMetrics>>` bottleneck.
- Modified `http::client` functions (`load_test`, `stress_test`, `perform_test`) to support channel-based result passing.
- Changed `send_request` and `send_api_request` in `http::client` to return `anyhow::Result` for consistent error handling.

### Added
- Implemented dedicated aggregator tasks in Load and Stress test controllers for incremental metric calculation and periodic updates via `TestContext`.
- Added `AppError::TestExecutionError(String)` variant for more descriptive test execution failures.
- Added `Default` implementation for `TestMetrics`.
- Added `test_id()` method to `TestContext` for better logging context.

### Fixed
- Resolved numerous compilation errors related to:
    - Incorrect imports and re-exports (`ApiTest` vs `ApiTestRequest`, `RequestResult`, `ApiRequestResult`).
    - Missing `anyhow` dependency and usage.
    - Type mismatches and trait bounds (`Default`, `Sized`).
    - Lifetime errors in stream combinators (`take_while`).
    - Syntax errors in pattern matching.
- Corrected error formatting for `anyhow::Error` when sending updates via `TestContext`.
- Ensured correct usage of `ApiTest` struct from `model::test::api_test`.

### Improved
- Significantly improved concurrency in test execution, especially removing mutex contention in Load/Stress tests.
- Enhanced error handling consistency using `anyhow` and specific `AppError` variants.
- Streamlined integration between asynchronous test execution logic and `TestContext` updates.
- Code clarity through iterative fixing of compiler errors and warnings.

### Technical Debt
- Removed unused function `process_client_message` from `websocket.rs`.
- Removed various unused imports (`AtomicUsize`, `Ordering`, `timeout`, `debug`, `error`, etc.) across refactored files.

## [0.4.3] - 2025-03-25

### Fixed
- Fixed duplicate ApiTest definitions and imports in test.rs
- Resolved moved value issues in test_common.rs by properly cloning metrics
- Fixed type mismatch in api_test_controller.rs by using correct ApiTest type
- Added missing Pending variant to TestStatus match in formatter.rs
- Fixed Send trait issues in load and stress test controllers
- Corrected AppState initialization in main.rs to properly handle returned tuple
- Fixed json macro and StreamExt trait imports in websocket.rs

### Improved
- Enhanced code organization by removing redundant imports
- Better type safety with proper error type bounds (Box<dyn Error + Send + Sync>)
- Improved state management in test controllers
- Enhanced WebSocket message handling with proper imports
- Cleaner code structure with removal of unused imports

### Technical Debt
- Removed unused imports across multiple files:
  - Removed unused HashMap, chrono, and TimeSeriesPoint imports in state.rs
  - Removed unused Duration and Receiver imports in websocket.rs
  - Removed unused Error imports in load_test_controller.rs and stress_test_controller.rs
  - Cleaned up redundant ApiTest imports

## [0.4.2] - 2025-03-25

### Fixed
- Resolved type mismatches in test controllers and WebSocket handling
- Fixed opaque type issues in test context responses
- Corrected module visibility for api_test module
- Fixed WebSocket receiver handling in connection management
- Resolved legacy config conversion issues
- Fixed router return value in create_router function
- Corrected ApiTest type usage in API test controller
- Fixed test configuration structure mismatches

### Changed
- Updated test context to use concrete Response type instead of opaque type
- Improved WebSocket connection handling with proper channel types
- Enhanced legacy config conversion to match new TestConfig structure
- Refactored test controllers to use correct type imports
- Improved error handling in WebSocket message processing

### Improved
- Enhanced type safety across test-related modules
- Better WebSocket connection management
- Cleaner module organization with proper visibility
- More consistent error handling in test controllers
- Improved backward compatibility with legacy configs

### Technical Debt
- Removed unused imports across multiple files
- Cleaned up redundant type definitions
- Standardized test configuration handling
- Improved code organization in test modules
- Enhanced type safety in WebSocket handling

## [0.4.1] - 2025-03-21

### Added
- New configuration module following functional programming principles
- Environment-based configuration system with typed config structures
- Improved logging format with MM/DD/YYYY HH:MM timestamps
- Enhanced error handling for configuration variables

### Changed
- Refactored middleware implementation to use tower_http CORS layer
- Updated logging middleware with more human-readable format
- Replaced hard-coded environment values with configuration module
- Improved middleware application order for better compatibility

### Fixed
- Resolved CORS middleware compatibility issues with Axum 0.7
- Fixed type mismatches in configuration number parsing
- Corrected middleware application order for proper request handling
- Fixed duplicate variable name in logging middleware

### Improved
- Enhanced log readability with formatted timestamps and emoji indicators
- Better request/response logging with clear visual indicators
- Simplified server configuration using the functional config module
- More consistent module exports and imports

## [0.4.0] - 2025-03-21

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
-   Updated the `parse_args` function in `src/main.rs`