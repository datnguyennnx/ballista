use std::env;

/// Server configuration
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

/// CORS configuration
#[derive(Clone, Debug)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u64,
}

/// WebSocket configuration
#[derive(Clone, Debug)]
pub struct WebSocketConfig {
    pub ping_interval: u64,
    pub max_connections: usize,
}

/// Test runner configuration
#[derive(Clone, Debug)]
pub struct TestRunnerConfig {
    pub timeout: u64,
    pub max_concurrent: usize,
    pub queue_size: usize,
}

/// Security configuration
#[derive(Clone, Debug)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiry: u64,
}

/// Complete application configuration
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub cors: CorsConfig,
    pub websocket: WebSocketConfig,
    pub test_runner: TestRunnerConfig,
    pub security: SecurityConfig,
    pub database_url: String,
}

// Pure function to get an environment variable with a default value
pub fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

// Pure function to parse a comma-separated list from an environment variable
pub fn get_env_list(key: &str, default: &str) -> Vec<String> {
    get_env_or_default(key, default)
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// Pure function to parse a numeric value from an environment variable
pub fn get_env_number<T: std::str::FromStr + ToString>(key: &str, default: T) -> T 
where 
    T::Err: std::fmt::Debug,
{
    get_env_or_default(key, &default.to_string())
        .parse()
        .unwrap_or(default)
}

// Pure function to load server configuration
pub fn load_server_config() -> ServerConfig {
    ServerConfig {
        host: get_env_or_default("SERVER_HOST", "127.0.0.1"),
        port: get_env_number("SERVER_PORT", 3001),
        log_level: get_env_or_default("LOG_LEVEL", "info"),
    }
}

// Pure function to load CORS configuration
pub fn load_cors_config() -> CorsConfig {
    CorsConfig {
        allowed_origins: get_env_list("CORS_ALLOWED_ORIGINS", "http://localhost:3000"),
        allowed_methods: get_env_list("CORS_ALLOWED_METHODS", "GET,POST,OPTIONS"),
        allowed_headers: get_env_list("CORS_ALLOWED_HEADERS", "Content-Type,Authorization"),
        max_age: get_env_number("CORS_MAX_AGE", 86400),
    }
}

// Pure function to load WebSocket configuration
pub fn load_websocket_config() -> WebSocketConfig {
    WebSocketConfig {
        ping_interval: get_env_number("WS_PING_INTERVAL", 30),
        max_connections: get_env_number("WS_MAX_CONNECTIONS", 1000),
    }
}

// Pure function to load test runner configuration
pub fn load_test_runner_config() -> TestRunnerConfig {
    TestRunnerConfig {
        timeout: get_env_number("TEST_RUNNER_TIMEOUT", 3600),
        max_concurrent: get_env_number("TEST_RUNNER_MAX_CONCURRENT", 100),
        queue_size: get_env_number("TEST_RUNNER_QUEUE_SIZE", 1000),
    }
}

// Pure function to load security configuration
pub fn load_security_config() -> SecurityConfig {
    SecurityConfig {
        jwt_secret: get_env_or_default("JWT_SECRET", "default-secret-key-change-in-production"),
        jwt_expiry: get_env_number("JWT_EXPIRY", 86400),
    }
}

// Pure function to load the complete application configuration
pub fn load_config() -> AppConfig {
    AppConfig {
        server: load_server_config(),
        cors: load_cors_config(),
        websocket: load_websocket_config(),
        test_runner: load_test_runner_config(),
        security: load_security_config(),
        database_url: get_env_or_default("DATABASE_URL", "sqlite:ballista.db"),
    }
} 