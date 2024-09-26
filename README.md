# Load Testing CLI Tool

This is a command-line interface (CLI) tool for load testing web applications. It allows you to send multiple concurrent requests to a specified URL or a list of URLs from a sitemap, and provides detailed metrics about the performance of the target application.

## Features

-   Send a specified number of requests to a single URL or multiple URLs from a sitemap
-   Adjust concurrency level to simulate multiple users
-   Stress testing mode with a specified duration
-   Display "send package to url" message for each request
-   Support for both sitemap XML and generic XML with children
-   Detailed metrics including:
    -   Total requests sent
    -   Successful and failed requests
    -   Average response time
    -   95th percentile response time
    -   HTTP status code distribution
    -   CPU and memory usage during the test
-   Real-time updates for each request

## Prerequisites

-   Rust programming language (https://www.rust-lang.org/tools/install)
-   Cargo (Rust's package manager, typically installed with Rust)

## Installation

1. Clone this repository:

    ```
    git clone https://github.com/yourusername/target-cli.git
    cd target-cli
    ```

2. Build the project:

    ```
    cargo build --release
    ```

3. The built binary will be located at `target/release/target`. To use it from any location, you need to add it to your PATH. You can do this by:

    - Moving the binary to a directory that's already in your PATH (e.g., `/usr/local/bin` on Unix-like systems)
    - Or, adding the `target/release` directory to your PATH

    For example, on Unix-like systems, you can add the following line to your `.bashrc` or `.zshrc`:

    ```
    export PATH=$PATH:/path/to/target-cli/target/release
    ```

    Remember to replace `/path/to/target-cli` with the actual path where you cloned the repository.

## Usage

The basic syntax for running the load testing tool is:

```
target [OPTIONS]
```

### Options:

-   `--url <URL>`: The target URL to load test
-   `--sitemap <SITEMAP_URL>`: URL or path to a sitemap file (alternative to --url)
-   `--requests <NUMBER>`: Number of requests to send (default: 100)
-   `--concurrency <NUMBER>`: Number of concurrent requests (default: 10)
-   `--stress`: Enable stress testing mode
-   `--duration <SECONDS>`: Duration for stress testing in seconds (default: 60)

### Examples:

1. Test a single URL with 1000 requests and 10 concurrent users:

    ```
    target --url https://example.com --requests 1000 --concurrency 10
    ```

2. Test URLs from a sitemap with 500 requests and 5 concurrent users:

    ```
    target --sitemap ./<your files name>.xml --requests 500 --concurrency 5
    ```

3. Run a stress test on a URL for 2 minutes with 20 concurrent users:

    ```
    target --url https://example.com --stress --duration 120 --concurrency 20
    ```

4. Run a load test using cargo:

    ```
    cargo run -- --url https://example.com --requests 100 --concurrency 10
    ```

5. Run a stress test using cargo:
    ```
    cargo run -- --url https://example.com --stress --duration 60 --concurrency 10
    ```

## Output

The tool provides real-time updates for each request. After the test is complete, it displays a summary of results including:

-   Total requests sent
-   Number of successful and failed requests
-   Total test duration
-   Requests per second
-   Average response time
-   95th percentile response time
-   HTTP status code distribution
-   Average CPU and memory usage during the test
-   Sample JSON responses (if applicable)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
