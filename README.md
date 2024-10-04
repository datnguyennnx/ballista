# Target Tool

Target Tool is a versatile Rust-based application for performing load testing, stress testing, and API testing. It provides detailed metrics and resource usage information to help you analyze the performance of your web applications and APIs.

## Features

-   Load testing with configurable number of requests and concurrency
-   Stress testing for a specified duration
-   API testing with custom test definitions
-   Resource usage monitoring (CPU, Memory, Network)
-   Support for testing single URLs or multiple URLs from a sitemap
-   Detailed performance metrics and statistics
-   Colorized and formatted output for easy reading

## Installation

1. Ensure you have Rust and Cargo installed on your system.
2. Clone this repository:
    ```
    git clone https://github.com/yourusername/target-tool.git
    cd target-tool
    ```
3. Build the project:
    ```
    cargo build --release
    ```

## Usage

Run the tool using the following command:

```
cargo run --release -- [OPTIONS]
```

### Options

-   `--url <URL>`: URL to test
-   `--sitemap <PATH>`: Path to sitemap XML file
-   `-r, --requests <NUMBER>`: Number of requests to send (default: 100)
-   `-c, --concurrency <NUMBER>`: Number of concurrent requests (default: 10)
-   `-s, --stress`: Enable stress test mode
-   `-d, --duration <SECONDS>`: Duration of the stress test in seconds (default: 60)
-   `--resource-usage`: Collect and display resource usage data for 60 seconds
-   `--config <PATH>`: Path to JSON configuration file
-   `--api-test <PATH>`: Path to API test JSON file

### Examples

1. Load test a single URL:

    ```
    cargo run --release -- --url https://example.com -r 1000 -c 20
    ```

2. Stress test using a sitemap:

    ```
    cargo run --release -- --sitemap path/to/sitemap.xml -s -d 300 -c 50
    ```

3. Run API tests:

    ```
    cargo run --release -- --api-test path/to/api_tests.json
    ```

4. Monitor resource usage:
    ```
    cargo run --release -- --resource-usage
    ```

## Configuration

You can use a JSON configuration file to specify test parameters. Create a file named `config.json` with the following structure:

```json
{
    "url": "https://example.com",
    "requests": 1000,
    "concurrency": 20,
    "stress": false,
    "duration": 60,
    "resource_usage": false,
    "api_test": "path/to/api_tests.json"
}
```

Then run the tool with:

```
cargo run --release -- --config config.json
```

## API Testing

To perform API tests, create a JSON file with test definitions:

```json
[
    {
        "name": "Get User",
        "url": "https://api.example.com/users/1",
        "method": "GET",
        "headers": {
            "Authorization": "Bearer token123"
        },
        "expected_status": 200
    },
    {
        "name": "Create User",
        "url": "https://api.example.com/users",
        "method": "POST",
        "headers": {
            "Content-Type": "application/json"
        },
        "body": {
            "name": "John Doe",
            "email": "john@example.com"
        },
        "expected_status": 201
    }
]
```

Then run the API tests with:

```
cargo run --release -- --api-test path/to/api_tests.json
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
