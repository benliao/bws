# Testing

This guide covers testing strategies, tools, and best practices for BWS development and deployment.

## Testing Overview

BWS includes multiple layers of testing:
- **Unit Tests**: Test individual functions and modules
- **Integration Tests**: Test component interactions
- **End-to-End Tests**: Test complete workflows
- **Performance Tests**: Measure performance characteristics
- **Security Tests**: Validate security measures

## Running Tests

### Basic Test Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_config_parsing

# Run tests matching pattern
cargo test config

# Run ignored tests
cargo test -- --ignored

# Run tests in single thread (for debugging)
cargo test -- --test-threads=1
```

### Test Categories

```bash
# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration

# Run only documentation tests
cargo test --doc

# Run tests for specific package
cargo test -p bws-core
```

### Test with Features

```bash
# Test with all features
cargo test --all-features

# Test with specific features
cargo test --features "compression,metrics"

# Test without default features
cargo test --no-default-features
```

## Unit Testing

### Basic Unit Tests

```rust
// src/config.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.sites.len(), 0);
    }

    #[test]
    fn test_config_parsing() {
        let toml_str = r#"
            [[sites]]
            name = "test"
            hostname = "localhost"
            port = 8080
            static_dir = "static"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.sites.len(), 1);
        assert_eq!(config.sites[0].name, "test");
        assert_eq!(config.sites[0].port, 8080);
    }

    #[test]
    #[should_panic(expected = "Invalid port")]
    fn test_invalid_port() {
        let site = Site {
            name: "test".to_string(),
            hostname: "localhost".to_string(),
            port: 0, // Invalid port
            static_dir: "static".to_string(),
            ..Default::default()
        };
        site.validate().unwrap();
    }
}
```

### Testing Error Conditions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_file_not_found() {
        let result = read_config_file("nonexistent.toml");
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("No such file"));
    }

    #[test]
    fn test_invalid_toml() {
        let invalid_toml = "invalid toml content [[[";
        let result = parse_config(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_field() {
        let toml_str = r#"
            [[sites]]
            name = "test"
            # Missing hostname, port, static_dir
        "#;
        
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }
}
```

### Mocking and Test Doubles

```rust
// Use mockall for mocking
use mockall::predicate::*;
use mockall::mock;

mock! {
    FileSystem {
        fn read_file(&self, path: &str) -> Result<String>;
        fn file_exists(&self, path: &str) -> bool;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading_with_mock() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs
            .expect_read_file()
            .with(eq("config.toml"))
            .times(1)
            .returning(|_| Ok(r#"
                [[sites]]
                name = "test"
                hostname = "localhost"
                port = 8080
                static_dir = "static"
            "#.to_string()));

        let config = load_config_with_fs(&mock_fs, "config.toml").unwrap();
        assert_eq!(config.sites.len(), 1);
    }
}
```

## Integration Testing

### Test Structure

```rust
// tests/integration/server_tests.rs
use bws::{Config, Server};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_server_startup_shutdown() {
    let config = test_config();
    let server = Server::new(config).await.unwrap();
    
    // Start server in background
    let handle = tokio::spawn(async move {
        server.run().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Test server is responding
    let response = reqwest::get("http://127.0.0.1:8080/health").await.unwrap();
    assert_eq!(response.status(), 200);
    
    // Shutdown server
    handle.abort();
}

#[tokio::test]
async fn test_static_file_serving() {
    let temp_dir = setup_test_static_files().await;
    let config = Config {
        sites: vec![Site {
            name: "test".to_string(),
            hostname: "127.0.0.1".to_string(),
            port: 8081,
            static_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    
    let server = Server::new(config).await.unwrap();
    let handle = tokio::spawn(async move {
        server.run().await
    });
    
    sleep(Duration::from_millis(100)).await;
    
    // Test serving static file
    let response = reqwest::get("http://127.0.0.1:8081/test.html").await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "<h1>Test</h1>");
    
    handle.abort();
    cleanup_test_files(temp_dir).await;
}

fn test_config() -> Config {
    Config {
        sites: vec![Site {
            name: "test".to_string(),
            hostname: "127.0.0.1".to_string(),
            port: 8080,
            static_dir: "test_static".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

async fn setup_test_static_files() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    
    tokio::fs::write(
        temp_dir.path().join("test.html"),
        "<h1>Test</h1>"
    ).await.unwrap();
    
    tokio::fs::write(
        temp_dir.path().join("index.html"),
        "<h1>Index</h1>"
    ).await.unwrap();
    
    temp_dir
}
```

### HTTP Client Testing

```rust
// tests/integration/http_tests.rs
use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn test_health_endpoint() {
    let client = Client::new();
    let response = client
        .get("http://127.0.0.1:8080/health")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
    
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_custom_headers() {
    let client = Client::new();
    let response = client
        .get("http://127.0.0.1:8080/")
        .send()
        .await
        .unwrap();
    
    // Check custom headers are present
    assert!(response.headers().contains_key("x-served-by"));
    assert_eq!(response.headers()["cache-control"], "public, max-age=3600");
}

#[tokio::test]
async fn test_cors_headers() {
    let client = Client::new();
    let response = client
        .options("http://127.0.0.1:8080/")
        .header("Origin", "https://example.com")
        .header("Access-Control-Request-Method", "GET")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    assert!(response.headers().contains_key("access-control-allow-origin"));
}
```

### Database Integration Tests

```rust
// tests/integration/database_tests.rs (if BWS had database features)
use sqlx::PgPool;

#[tokio::test]
async fn test_database_connection() {
    let pool = setup_test_database().await;
    
    let config = Config {
        database_url: Some(pool.connect_options().to_url_lossy().to_string()),
        ..test_config()
    };
    
    let server = Server::new(config).await.unwrap();
    
    // Test database-dependent endpoints
    let response = reqwest::get("http://127.0.0.1:8080/api/data").await.unwrap();
    assert_eq!(response.status(), 200);
    
    cleanup_test_database(pool).await;
}

async fn setup_test_database() -> PgPool {
    // Set up test database
    PgPool::connect("postgres://test:test@localhost/bws_test")
        .await
        .unwrap()
}
```

## End-to-End Testing

### Test Scenarios

```rust
// tests/e2e/scenarios.rs
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_complete_deployment_scenario() {
    // 1. Create test configuration
    let config_content = r#"
        [daemon]
        pid_file = "/tmp/bws-test.pid"
        
        [logging]
        level = "info"
        output = "file"
        file_path = "/tmp/bws-test.log"
        
        [[sites]]
        name = "main"
        hostname = "127.0.0.1"
        port = 8080
        static_dir = "test_static"
        
        [sites.headers]
        "Cache-Control" = "public, max-age=3600"
    "#;
    
    std::fs::write("test-config.toml", config_content).unwrap();
    
    // 2. Create static files
    std::fs::create_dir_all("test_static").unwrap();
    std::fs::write("test_static/index.html", "<h1>Welcome to BWS</h1>").unwrap();
    std::fs::write("test_static/style.css", "body { color: blue; }").unwrap();
    
    // 3. Start BWS server
    let mut child = Command::new("target/release/bws")
        .arg("--config")
        .arg("test-config.toml")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    
    // 4. Wait for server to start
    sleep(Duration::from_secs(2)).await;
    
    // 5. Run tests
    test_homepage().await;
    test_static_files().await;
    test_health_check().await;
    test_performance().await;
    
    // 6. Cleanup
    child.kill().unwrap();
    std::fs::remove_file("test-config.toml").unwrap();
    std::fs::remove_dir_all("test_static").unwrap();
    std::fs::remove_file("/tmp/bws-test.log").ok();
    std::fs::remove_file("/tmp/bws-test.pid").ok();
}

async fn test_homepage() {
    let response = reqwest::get("http://127.0.0.1:8080/").await.unwrap();
    assert_eq!(response.status(), 200);
    assert!(response.text().await.unwrap().contains("Welcome to BWS"));
}

async fn test_static_files() {
    let response = reqwest::get("http://127.0.0.1:8080/style.css").await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "text/css");
    assert!(response.text().await.unwrap().contains("color: blue"));
}

async fn test_health_check() {
    let response = reqwest::get("http://127.0.0.1:8080/health").await.unwrap();
    assert_eq!(response.status(), 200);
    
    let health: serde_json::Value = response.json().await.unwrap();
    assert_eq!(health["status"], "healthy");
}

async fn test_performance() {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Make 100 concurrent requests
    let futures: Vec<_> = (0..100)
        .map(|_| reqwest::get("http://127.0.0.1:8080/"))
        .collect();
    
    let responses = futures::future::join_all(futures).await;
    let duration = start.elapsed();
    
    // All requests should succeed
    for response in responses {
        assert_eq!(response.unwrap().status(), 200);
    }
    
    // Should complete in reasonable time
    assert!(duration < Duration::from_secs(5));
    println!("100 requests completed in {:?}", duration);
}
```

### Multi-Site Testing

```rust
#[tokio::test]
async fn test_multi_site_configuration() {
    let config_content = r#"
        [[sites]]
        name = "main"
        hostname = "127.0.0.1"
        port = 8080
        static_dir = "main_static"
        
        [[sites]]
        name = "api"
        hostname = "127.0.0.1"
        port = 8081
        static_dir = "api_static"
        
        [sites.headers]
        "Content-Type" = "application/json"
    "#;
    
    // Setup and test both sites
    setup_multi_site_files();
    
    let mut child = start_bws_server("multi-site-config.toml");
    sleep(Duration::from_secs(2)).await;
    
    // Test main site
    let response = reqwest::get("http://127.0.0.1:8080/").await.unwrap();
    assert_eq!(response.status(), 200);
    
    // Test API site
    let response = reqwest::get("http://127.0.0.1:8081/").await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "application/json");
    
    cleanup_multi_site_test(child);
}
```

## Performance Testing

### Load Testing with wrk

```bash
#!/bin/bash
# scripts/load-test.sh

BWS_PID=""

setup_test_server() {
    echo "Setting up test server..."
    
    # Create test configuration
    cat > test-load-config.toml << EOF
[[sites]]
name = "load-test"
hostname = "127.0.0.1"
port = 8080
static_dir = "load_test_static"

[sites.headers]
"Cache-Control" = "public, max-age=3600"
EOF

    # Create test files
    mkdir -p load_test_static
    echo "<h1>Load Test Page</h1>" > load_test_static/index.html
    
    # Generate test files of various sizes
    dd if=/dev/zero of=load_test_static/1kb.txt bs=1024 count=1 2>/dev/null
    dd if=/dev/zero of=load_test_static/10kb.txt bs=1024 count=10 2>/dev/null
    dd if=/dev/zero of=load_test_static/100kb.txt bs=1024 count=100 2>/dev/null
    
    # Start BWS
    ./target/release/bws --config test-load-config.toml &
    BWS_PID=$!
    
    sleep 2
}

run_load_tests() {
    echo "Running load tests..."
    
    # Test 1: Basic load test
    echo "=== Basic Load Test ==="
    wrk -t4 -c50 -d30s --latency http://127.0.0.1:8080/
    
    # Test 2: High concurrency
    echo "=== High Concurrency Test ==="
    wrk -t8 -c200 -d30s --latency http://127.0.0.1:8080/
    
    # Test 3: Different file sizes
    echo "=== 1KB File Test ==="
    wrk -t4 -c50 -d15s http://127.0.0.1:8080/1kb.txt
    
    echo "=== 10KB File Test ==="
    wrk -t4 -c50 -d15s http://127.0.0.1:8080/10kb.txt
    
    echo "=== 100KB File Test ==="
    wrk -t4 -c50 -d15s http://127.0.0.1:8080/100kb.txt
    
    # Test 4: Sustained load
    echo "=== Sustained Load Test (5 minutes) ==="
    wrk -t4 -c100 -d300s --latency http://127.0.0.1:8080/
}

cleanup_test() {
    echo "Cleaning up..."
    if [ ! -z "$BWS_PID" ]; then
        kill $BWS_PID 2>/dev/null
    fi
    rm -rf load_test_static test-load-config.toml
}

# Trap cleanup on script exit
trap cleanup_test EXIT

setup_test_server
run_load_tests
```

### Benchmark Tests

```rust
// benches/server_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bws::{Config, Server};

fn bench_config_parsing(c: &mut Criterion) {
    let config_str = r#"
        [[sites]]
        name = "bench"
        hostname = "127.0.0.1"
        port = 8080
        static_dir = "static"
        
        [sites.headers]
        "Cache-Control" = "public, max-age=3600"
        "X-Content-Type-Options" = "nosniff"
    "#;
    
    c.bench_function("parse config", |b| {
        b.iter(|| {
            let _config: Config = toml::from_str(black_box(config_str)).unwrap();
        })
    });
}

fn bench_static_file_resolution(c: &mut Criterion) {
    let config = test_config();
    
    c.bench_function("resolve static file", |b| {
        b.iter(|| {
            let _path = resolve_static_file(
                black_box("/assets/css/main.css"),
                black_box(&config.sites[0])
            );
        })
    });
}

criterion_group!(benches, bench_config_parsing, bench_static_file_resolution);
criterion_main!(benches);
```

### Memory Testing

```rust
// tests/memory_tests.rs
#[test]
fn test_memory_usage() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let memory_counter = Arc::new(AtomicUsize::new(0));
    
    // Custom allocator to track memory usage
    #[global_allocator]
    static GLOBAL: TrackingAllocator = TrackingAllocator;
    
    let initial_memory = get_memory_usage();
    
    // Create large number of configs
    let configs: Vec<Config> = (0..1000)
        .map(|i| Config {
            sites: vec![Site {
                name: format!("site_{}", i),
                hostname: "127.0.0.1".to_string(),
                port: 8080 + i,
                static_dir: format!("static_{}", i),
                ..Default::default()
            }],
            ..Default::default()
        })
        .collect();
    
    let peak_memory = get_memory_usage();
    drop(configs);
    
    // Force garbage collection
    std::hint::black_box(());
    
    let final_memory = get_memory_usage();
    
    println!("Initial memory: {} KB", initial_memory / 1024);
    println!("Peak memory: {} KB", peak_memory / 1024);
    println!("Final memory: {} KB", final_memory / 1024);
    
    // Memory should be released
    assert!(final_memory < peak_memory);
}
```

## Security Testing

### Input Validation Tests

```rust
#[tokio::test]
async fn test_path_traversal_protection() {
    // Test various path traversal attempts
    let malicious_paths = vec![
        "../../../etc/passwd",
        "..%2F..%2F..%2Fetc%2Fpasswd",
        "....//....//....//etc//passwd",
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
    ];
    
    for path in malicious_paths {
        let response = reqwest::get(&format!("http://127.0.0.1:8080/{}", path))
            .await
            .unwrap();
        
        // Should return 404 or 403, not 200
        assert_ne!(response.status(), 200);
    }
}

#[tokio::test]
async fn test_request_size_limits() {
    let client = reqwest::Client::new();
    
    // Test large request body
    let large_body = "x".repeat(10 * 1024 * 1024); // 10MB
    
    let response = client
        .post("http://127.0.0.1:8080/")
        .body(large_body)
        .send()
        .await
        .unwrap();
    
    // Should reject large requests
    assert_eq!(response.status(), 413); // Payload Too Large
}

#[tokio::test]
async fn test_header_injection() {
    let client = reqwest::Client::new();
    
    // Test header injection attempts
    let response = client
        .get("http://127.0.0.1:8080/")
        .header("X-Forwarded-For", "malicious\r\nContent-Type: text/html")
        .send()
        .await
        .unwrap();
    
    // Response should not contain injected header
    assert!(!response.headers().contains_key("content-type"));
}
```

### Rate Limiting Tests

```rust
#[tokio::test]
async fn test_rate_limiting() {
    let client = reqwest::Client::new();
    
    // Make requests rapidly
    let mut success_count = 0;
    let mut rate_limited_count = 0;
    
    for _ in 0..100 {
        let response = client
            .get("http://127.0.0.1:8080/")
            .send()
            .await
            .unwrap();
        
        match response.status().as_u16() {
            200 => success_count += 1,
            429 => rate_limited_count += 1, // Too Many Requests
            _ => {}
        }
    }
    
    // Should have some rate limited responses
    assert!(rate_limited_count > 0);
    println!("Success: {}, Rate Limited: {}", success_count, rate_limited_count);
}
```

## Test Utilities and Helpers

### Test Configuration Factory

```rust
// tests/common/mod.rs
pub fn test_config() -> Config {
    Config {
        daemon: DaemonConfig::default(),
        logging: LoggingConfig {
            level: "debug".to_string(),
            output: "stdout".to_string(),
            ..Default::default()
        },
        sites: vec![Site {
            name: "test".to_string(),
            hostname: "127.0.0.1".to_string(),
            port: 8080,
            static_dir: "test_static".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn test_config_with_port(port: u16) -> Config {
    let mut config = test_config();
    config.sites[0].port = port;
    config
}

pub fn test_config_multi_site() -> Config {
    Config {
        sites: vec![
            Site {
                name: "main".to_string(),
                hostname: "127.0.0.1".to_string(),
                port: 8080,
                static_dir: "main_static".to_string(),
                ..Default::default()
            },
            Site {
                name: "api".to_string(),
                hostname: "127.0.0.1".to_string(),
                port: 8081,
                static_dir: "api_static".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
```

### Test Server Management

```rust
use std::sync::Once;
use tokio::sync::Mutex;

static INIT: Once = Once::new();
static TEST_SERVER: Mutex<Option<TestServer>> = Mutex::const_new(None);

pub struct TestServer {
    pub port: u16,
    handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    pub async fn start(config: Config) -> Self {
        let port = config.sites[0].port;
        let server = Server::new(config).await.unwrap();
        
        let handle = tokio::spawn(async move {
            server.run().await.unwrap();
        });
        
        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        TestServer { port, handle }
    }
    
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

// Global test server for shared tests
pub async fn get_test_server() -> &'static TestServer {
    let mut server = TEST_SERVER.lock().await;
    if server.is_none() {
        *server = Some(TestServer::start(test_config()).await);
    }
    server.as_ref().unwrap()
}
```

### Test File Management

```rust
use tempfile::{TempDir, NamedTempFile};

pub struct TestStaticFiles {
    pub temp_dir: TempDir,
    pub index_file: PathBuf,
    pub css_file: PathBuf,
    pub js_file: PathBuf,
}

impl TestStaticFiles {
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        
        let index_file = temp_dir.path().join("index.html");
        tokio::fs::write(&index_file, "<h1>Test Index</h1>").await.unwrap();
        
        let css_file = temp_dir.path().join("style.css");
        tokio::fs::write(&css_file, "body { color: red; }").await.unwrap();
        
        let js_file = temp_dir.path().join("script.js");
        tokio::fs::write(&js_file, "console.log('test');").await.unwrap();
        
        TestStaticFiles {
            temp_dir,
            index_file,
            css_file,
            js_file,
        }
    }
    
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}
```

## Continuous Integration Testing

### GitHub Actions Test Workflow

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
        
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Run tests
      run: cargo test --all-features --verbose
    
    - name: Run integration tests
      run: cargo test --test integration --all-features
    
    - name: Run benchmarks (check only)
      run: cargo bench --no-run
```

### Test Coverage

```yaml
# Add to GitHub Actions
- name: Install coverage tools
  run: |
    cargo install cargo-tarpaulin

- name: Generate test coverage
  run: |
    cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Xml

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    file: ./cobertura.xml
```

## Testing Best Practices

### Test Organization
- Group related tests in modules
- Use descriptive test names
- Follow AAA pattern (Arrange, Act, Assert)
- Test both happy path and error cases
- Use test fixtures for common setup

### Test Data Management
- Use temporary directories for file operations
- Clean up resources in test teardown
- Use factories for creating test objects
- Avoid hardcoded values, use constants

### Performance Testing
- Run performance tests in isolated environment
- Use consistent hardware for benchmarks
- Monitor for performance regressions
- Set reasonable performance thresholds

### Security Testing
- Test all input validation
- Check authentication and authorization
- Verify secure defaults
- Test rate limiting and DOS protection

### Test Maintenance
- Keep tests up to date with code changes
- Remove or update obsolete tests
- Refactor duplicated test code
- Document complex test scenarios

## Next Steps

- Set up [Continuous Integration](https://docs.github.com/en/actions)
- Learn about [Performance Tuning](./performance.md) for optimization
- Review [Contributing Guidelines](./contributing.md) for development workflow
- Check [Troubleshooting](./troubleshooting.md) for common issues
