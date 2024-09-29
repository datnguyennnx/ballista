# Target CLI - Load Testing Tool

Target CLI is a command-line load testing tool written in Rust. It allows you to perform load tests and stress tests on web applications, providing detailed performance metrics and resource usage statistics.

## Features

-   Load testing with a specified number of requests
-   Stress testing for a specified duration
-   Concurrent request handling
-   Support for testing single URLs or multiple URLs from a sitemap
-   Real-time progress tracking
-   Detailed performance metrics
-   Resource usage monitoring (CPU and Memory)
-   Standalone resource usage collection
-   Color-coded output for better readability

## Usage

```
target [OPTIONS] --url <URL> | --sitemap <SITEMAP_PATH>
```

### Options

-   `--url <URL>`: URL to test
-   `--sitemap <SITEMAP_PATH>`: Path to sitemap XML file
-   `-r, --requests <REQUESTS>`: Number of requests to send (default: 100)
-   `-c, --concurrency <CONCURRENCY>`: Number of concurrent requests (default: 10)
-   `-s, --stress`: Enable stress test mode
-   `-d, --duration <DURATION>`: Duration of the stress test in seconds (default: 60)
-   `--resource-usage`: Collect and display resource usage data for 60 seconds

## Example Output

### Load Test

```
Load Test
==================================================
URLs to test: 1
Concurrency: 10
Total requests: 100
==================================================

[2023-05-01 15:30:00] https://example.com - 200 - 152.45ms
[2023-05-01 15:30:00] https://example.com - 200 - 148.32ms
...

Test Results
==================================================
Total requests: 100
Successful requests: 98
Failed requests: 2
Total time: 15.23s
Requests per second: 6.56
Average response time: 152.34ms
Minimum response time: 148.32ms
Maximum response time: 523.67ms
Median response time: 151.89ms
95th percentile response time: 201.45ms

HTTP Status Codes
==================================================
200: 98
404: 2

Resource Usage
==================================================
Average CPU Usage: 23.45%
Max CPU Usage: 35.67%
Average Memory Usage: 1.23%
Max Memory Usage: 1.45%
```

### Resource Usage Collection

```
$ target --resource-usage

Collecting resource usage data for 60 seconds...

Resource Usage
Average CPU Usage: 25.55% - 60 (number used)
Max CPU Usage: 28.33% - 60 (number used)
Average Memory Usage: 39.00% - 60 (number used)
Max Memory Usage: 40.01% - 60 (number used)
```

## Building and Running

1. Clone the repository
2. Run `cargo build --release`
3. The binary will be available in `target/release/target`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
