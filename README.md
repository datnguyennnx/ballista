# Target CLI - Advanced Load Testing Tool (v0.1.0)

Target CLI is a command-line load testing tool written in Rust. It allows you to perform both load tests and stress tests on web applications, providing detailed performance metrics and resource usage statistics. This tool is designed to help developers and system administrators evaluate the performance and scalability of their web applications under various load conditions.

## Features

-   **Flexible Testing Modes**:
    -   Load testing with a specified number of requests
    -   Stress testing for a specified duration
-   **Concurrent Request Handling**: Simulate multiple users accessing the application simultaneously
-   **Multiple URL Support**: Test single URLs or multiple URLs from a sitemap
-   **Real-time Progress Tracking**: Monitor the test progress with a dynamic progress bar
-   **Detailed Performance Metrics**:
    -   Request counts and rates
    -   Response time statistics (min, max, median, 95th percentile)
    -   HTTP status code distribution
-   **Resource Usage Monitoring**:
    -   CPU usage (average and maximum)
    -   Memory usage (average and maximum)
    -   Network usage (received and sent)
-   **Standalone Resource Usage Collection**: Collect system resource usage data independently of load testing
-   **Color-coded Output**: Improve readability with color-coded status codes and formatted tables
-   **Human-readable Formatting**: Durations and sizes are formatted for easy reading
-   **Improved Error Handling**: Custom error types for better error reporting and handling

## Installation

To install Target CLI, you need to have Rust and Cargo installed on your system. Then, follow these steps:

1. Clone the repository:

    ```
    git clone https://github.com/yourusername/target-cli.git
    cd target-cli
    ```

2. Build the project:

    ```
    cargo build --release
    ```

3. The binary will be available at `target/release/target`

## Usage

```
target [OPTIONS] --url <URL> | --sitemap <SITEMAP_PATH>
```

### Options

-   `--url <URL>`: URL to test
-   `--sitemap <SITEMAP_PATH>`: Path to sitemap
-   `-r, --requests <REQUESTS>`: Number of requests to send (default: 100, ignored if --stress is set)
-   `-c, --concurrency <CONCURRENCY>`: Number of concurrent requests (default: 10)
-   `-s, --stress`: Enable stress test mode
-   `-d, --duration <DURATION>`: Duration of the stress test in seconds (default: 60, only used if --stress is set)
-   `--resource-usage`: Collect and display resource usage data for 60 seconds
-   `--config <CONFIG_PATH>`: Path to JSON configuration file

### Example Usage

#### Load Test

```
target --url https://example.com --requests 1000 --concurrency 20
```

This command will perform a load test on https://example.com with 1000 total requests and 20 concurrent requests.

#### Stress Test

```
target --url https://example.com --stress --duration 300 --concurrency 50
```

This command will perform a stress test on https://example.com for 300 seconds (5 minutes) with 50 concurrent requests.

#### Testing with Sitemap

```
target --sitemap path/to/sitemap.xml --requests 500 --concurrency 10
```

This command will perform a load test using URLs from the specified sitemap, with 500 total requests and 10 concurrent requests.

#### Resource Usage Collection

```
target --resource-usage
```

This command will collect and display resource usage data for 60 seconds without performing any load testing.

#### JSON Configuration

You can also specify a JSON configuration file:

```
target --config path/to/config.json
```

An example of a JSON configuration file:

```json
{
    "url": "https://example.com",
    "requests": 1000,
    "concurrency": 20,
    "stress": false,
    "duration": 60,
    "resource_usage": true
}
```

## Output

The tool provides detailed output including test configuration summary, real-time progress bar, comprehensive test results table, HTTP status code distribution, and resource usage statistics. Here's an example of what the output might look like:

```
Test Results
+------------------+------------+------------------------+------------------------+------------------------+------------------------+------------------------+--------------------------------+
| Total requests   | Total time | Requests per second    | Average response time  | Minimum response time  | Maximum response time  | Median response time  | 95th percentile response time  |
+------------------+------------+------------------------+------------------------+------------------------+------------------------+------------------------+--------------------------------+
| 1000             | 15s 230ms  | 65.66                  | 152ms 340µs            | 148ms 320µs            | 523ms 670µs            | 151ms 890µs            | 201ms 450µs                    |
+------------------+------------+------------------------+------------------------+------------------------+------------------------+------------------------+--------------------------------+

HTTP Status Codes
+-------------+-------+------------+
| Status Code | Count | Percentage |
+-------------+-------+------------+
| 200         | 985   | 98.50%     |
| 404         | 10    | 1.00%      |
| 500         | 5     | 0.50%      |
+-------------+-------+------------+

Resource Usage
+------------------+------------------------+------------------------+------------------------+
| Metric           | Average                | Maximum                | Total                  |
+------------------+------------------------+------------------------+------------------------+
| CPU Usage        | 23.45% (1.88 cores)    | 35.67% (2.85 cores)    | N/A                    |
| Memory Usage     | 1.23% (196.8 MB)       | 1.45% (232.0 MB)       | N/A                    |
| Network Received | 5.67 MB/s              | 8.90 MB/s              | 86.45 MB               |
| Network Sent     | 1.23 MB/s              | 2.45 MB/s              | 18.76 MB               |
+------------------+------------------------+------------------------+------------------------+

Success Rate: 98.50%
```

This output provides a comprehensive overview of the test results, including performance metrics and resource usage statistics. The HTTP status codes are color-coded in the actual output (green for 2xx, yellow for 3xx, red for 4xx and 5xx) to improve readability. Durations and sizes are formatted for easy reading using the new utility functions.

## Implementation Details

Target CLI is implemented in Rust and utilizes several key components:

-   `args.rs`: Defines and validates command-line arguments using the `clap` crate
-   `http_client.rs`: Handles HTTP requests and manages the load/stress testing logic
-   `metrics.rs`: Collects and calculates performance metrics
-   `structure_output.rs`: Formats and displays test results using colored output and tables
-   `utils.rs`: Provides utility functions for parsing sitemaps, collecting system resource usage, and formatting durations and sizes
-   `resource_monitor.rs`: Manages the collection of system resource usage data
-   `main.rs`: Orchestrates the overall flow of the application

The tool uses asynchronous programming with `tokio` for efficient concurrent request handling and `reqwest` for HTTP client functionality. It also leverages `sysinfo` for system resource monitoring, `prettytable-rs` for formatted output, and `thiserror` for improved error handling.

## Dependencies

Target CLI relies on the following main dependencies:

-   `clap`: Command-line argument parsing
-   `reqwest`: HTTP client for making requests
-   `tokio`: Asynchronous runtime
-   `serde` and `serde_json`: JSON serialization and deserialization
-   `futures`: Asynchronous programming utilities
-   `indicatif`: Progress bars and indicators
-   `colored`: Colored terminal output
-   `prettytable-rs`: Formatted table output
-   `sysinfo`: System information and resource usage
-   `num_cpus`: CPU information
-   `sys-info`: Additional system information
-   `thiserror`: Custom error type definitions

For a complete list of dependencies, please refer to the `Cargo.toml` file.

## Contributing

Contributions to Target CLI are welcome! Please feel free to submit issues, fork the repository and send pull requests!

To contribute:

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

-   The Rust community for providing excellent crates and documentation
-   All contributors who have helped to improve this tool
