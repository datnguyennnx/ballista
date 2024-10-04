# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

-   New API testing functionality
-   `api_tester.rs` module for handling API tests
-   Support for running API tests from JSON configuration files
-   `--api-test` command-line option for specifying API test JSON files
-   Example API test JSON file in `examples/sample_restfulAPI_test.json`

### Changed

-   Updated `main.rs` to incorporate API testing logic
-   Modified `args.rs` to include the new `--api-test` option
-   Updated README.md with information about the new API testing feature

### Updated

-   Dependencies in Cargo.toml, including new crates: `async-trait` and `mockall` (for testing)

## [0.1.0] - 2024-09-30

### Added

-   New `ResourceMonitor` module for more efficient resource usage tracking
-   Utility functions `format_duration` and `format_size` for human-readable output
-   More comprehensive unit tests for utility functions
-   This CHANGELOG file to track project changes
-   Support for standalone resource usage monitoring

### Changed

-   Refactored codebase to follow functional programming principles
-   Improved error handling with custom error types using `thiserror`
-   Updated `metrics.rs` to use more immutable data structures
-   Enhanced `structure_output.rs` for better formatting of test results
-   Updated `http_client.rs` to be more functional and modular
-   Improved `args.rs` with better encapsulation and immutability

### Updated

-   Dependencies in Cargo.toml, including new crates: `num_cpus`, `sys-info`, and `thiserror`
-   README.md with more detailed information about the tool's features and usage

### Fixed

-   Resource monitor test to avoid timeouts and ensure proper functionality

## [0.0.1] - 2023-05-01

### Added

-   Initial release of Target CLI
-   Basic load testing functionality
-   Support for testing single URLs and sitemaps
-   Simple resource usage monitoring
-   Command-line interface for configuring tests
