# Ballista
Ballista - An ancient precision weapon, suggesting targeted and powerful testing

Ballista is a modern Rust-based RESTful API service for performing load testing, stress testing, and API testing. Built with Axum and following functional programming principles, it provides a robust HTTP interface for testing web applications and APIs.

## Features

- RESTful API endpoints for:
  - Load testing with configurable requests and concurrency
  - Stress testing with specified duration
  - API testing with custom test definitions
- Real-time test status tracking
- Detailed performance metrics and statistics
- CORS support for web clients
- Asynchronous test execution
- Test results history

## Functional Programming Approach

Ballista is built using functional programming principles in Rust:

- Immutability: Using immutable data structures to prevent unexpected state changes
- Pure functions: Core logic implemented using pure functions without side effects
- Higher-order functions: Functions that take other functions as arguments
- Function composition: Complex operations built by combining simple functions
- State isolation: Clear separation between state management and business logic

## Installation

1. Ensure you have Rust and Cargo installed
2. Clone this repository:
```bash
git clone https://github.com/datnguyennnx/ballista.git
cd ballista
```

3. Build the project:
```bash
cargo build --release
```

## API Usage

Start the server:
```bash
cargo run --release
```

The server will start on `http://localhost:3001`

### API Endpoints

#### Health Check
```bash
GET /api/health
```
Response:
```json
{
    "success": true,
    "message": "API is running"
}
```

#### Load Testing
```bash
POST /api/load-test
```
Request body:
```json
{
    "url": "https://example.com",
    "requests": 1000,
    "concurrency": 20
}
```

#### Stress Testing
```bash
POST /api/stress-test
```
Request body:
```json
{
    "sitemap": "https://example.com/sitemap.xml",
    "duration": 300,
    "concurrency": 50
}
```

#### API Testing
```bash
POST /api/api-test
```
Request body:
```json
{
    "path": "path/to/api_tests.json"
}
```

#### Get Test Results
```bash
GET /api/tests
```
Response:
```json
{
    "success": true,
    "message": "Test results retrieved",
    "data": [
        {
            "test_type": "Load",
            "status": "Success",
            "details": "Load test completed successfully",
            "timestamp": "2024-03-20T10:30:00Z"
        }
    ]
}
```

### API Test Definition Format

Create a JSON file with your API test definitions:

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

## Example Usage with curl

```bash
# Health check
curl http://localhost:3001/api/health

# Run load test
curl -X POST http://localhost:3001/api/load-test \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "requests": 1000, "concurrency": 20}'

# Get test results
curl http://localhost:3001/api/tests
```

## Contributing

Contributions are welcome! Please follow these guidelines:
- Adhere to functional programming principles
- Write pure functions where possible
- Include tests for new functionality
- Update documentation as needed

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
