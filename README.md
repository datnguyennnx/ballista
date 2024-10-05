# Target Tool

Target Tool is a versatile Rust-based application for performing load testing, stress testing, and API testing. It provides detailed metrics and resource usage information to help you analyze the performance of your web applications and APIs. The project is implemented using functional programming principles to ensure modularity, testability, and maintainability.

## Features

-   Load testing with configurable number of requests and concurrency
-   Stress testing for a specified duration
-   API testing with custom test definitions
-   Resource usage monitoring (CPU, Memory, Network)
-   Support for testing single URLs or multiple URLs from a sitemap
-   Detailed performance metrics and statistics
-   Colorized and formatted output for easy reading
-   Flexible test configurations with optional total requests

## Functional Programming Approach

Target Tool is built using functional programming principles in Rust:

-   Immutability: We use immutable data structures wherever possible to prevent unexpected state changes.
-   Pure functions: Our core logic is implemented using pure functions that don't have side effects and always return the same output for the same input.
-   Higher-order functions: We utilize functions that take other functions as arguments or return functions, enabling flexible and composable code.
-   Function composition: Complex operations are built by combining simple functions, improving code readability and maintainability.
-   Avoiding side effects: We minimize functions that modify state outside their scope, making the code easier to reason about and test.

These principles result in a more modular, testable, and maintainable codebase.

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
cargo run --release -- <COMMAND> [OPTIONS]
```

### Commands

-   `load-test`: Perform a load test
-   `stress-test`: Perform a stress test
-   `api-test`: Run API tests
-   `resource-usage`: Monitor resource usage

### Options

-   `--url <URL>`: URL to test (for load and stress tests)
-   `--sitemap <PATH>`: Path to sitemap XML file (for load and stress tests)
-   `-r, --requests <NUMBER>`: Number of requests to send (optional for stress tests)
-   `-c, --concurrency <NUMBER>`: Number of concurrent requests (default: 10)
-   `-d, --duration <SECONDS>`: Duration of the stress test in seconds (default: 60)
-   `--config <PATH>`: Path to JSON configuration file
-   `<PATH>`: Path to API test JSON file (for api-test command)

### Examples

1. Load test a single URL:

    ```
    cargo run --release -- load-test --url https://example.com -r 1000 -c 20
    ```

2. Stress test using a sitemap:

    ```
    cargo run --release -- stress-test --sitemap path/to/sitemap.xml -d 300 -c 50
    ```

3. Run API tests:

    ```
    cargo run --release -- api-test examples/sample_restfulAPI_test.json
    ```

4. Monitor resource usage:
    ```
    cargo run --release -- resource-usage
    ```

## Configuration

You can use a JSON configuration file to specify test parameters. Create a file named `config.json` with the following structure:

```json
{
    "url": "https://example.com",
    "requests": 1000,
    "concurrency": 20,
    "duration": 60,
    "api_test": "path/to/api_tests.json"
}
```

Then run the tool with:

```
cargo run --release -- load-test --config config.json
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
cargo run --release -- api-test path/to/api_tests.json
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. When contributing, please adhere to the functional programming principles used in this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
