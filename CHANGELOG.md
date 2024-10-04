# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

-   Implemented explicit naming for subcommands in the CLI interface
-   Added support for `load-test`, `stress-test`, `api-test`, and `resource-usage` subcommands

### Changed

-   Enhanced performance metrics and statistics
-   Updated `TestConfig` struct to use `Option<u32>` for `total_requests`, allowing more flexible test configurations
-   Refactored `main.rs` to improve code organization and readability
-   Separated application logic into a new `app.rs` file for better modularity
-   Improved type consistency across the project, particularly in the HTTP client module
-   Updated `Args` struct and `Command` enum in `src/args.rs` to use explicit subcommand names

### Fixed

-   Resolved issues related to handling optional total requests in load and stress tests
-   Fixed the `load-test` command recognition in the CLI interface
