# Contributing to BWS

Thank you for your interest in contributing to BWS! This guide will help you get started with contributing to the project.

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust**: Version 1.89 or later
- **Git**: For version control
- **GitHub Account**: For submitting pull requests
- **Basic Rust Knowledge**: Understanding of Rust syntax and concepts

### Development Environment Setup

1. **Fork the Repository**
   ```bash
   # Fork the repository on GitHub, then clone your fork
   git clone https://github.com/yourusername/bws.git
   cd bws
   
   # Add upstream remote
   git remote add upstream https://github.com/benliao/bws.git
   ```

2. **Install Dependencies**
   ```bash
   # Install Rust if not already installed
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   
   # Install required tools
   cargo install cargo-fmt
   cargo install cargo-clippy
   cargo install cargo-audit
   ```

3. **Build the Project**
   ```bash
   # Build in debug mode
   cargo build
   
   # Run tests
   cargo test
   
   # Check formatting and linting
   cargo fmt --check
   cargo clippy -- -D warnings
   ```

## Development Workflow

### 1. Creating a Feature Branch

```bash
# Sync with upstream
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name
```

### 2. Making Changes

Follow these guidelines when making changes:

#### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add documentation for public APIs
- Include unit tests for new functionality

#### Commit Messages
Use conventional commit format:
```
type(scope): description

body (optional)

footer (optional)
```

Examples:
```bash
git commit -m "feat(server): add HTTP/2 support"
git commit -m "fix(config): handle missing static directory"
git commit -m "docs(readme): update installation instructions"
```

**Commit Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### 3. Testing Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration

# Check for memory leaks (if available)
cargo valgrind test
```

### 4. Code Quality Checks

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Check for security vulnerabilities
cargo audit

# Check documentation
cargo doc --no-deps
```

## Contributing Guidelines

### Code Organization

```
src/
├── lib.rs          # Main library entry point
├── bin/
│   └── main.rs     # Binary entry point
├── config/         # Configuration handling
├── server/         # Server implementation
├── handlers/       # Request handlers
├── utils/          # Utility functions
└── tests/          # Integration tests

tests/              # Integration tests
docs/               # Documentation
examples/           # Example configurations
```

### Writing Good Code

#### 1. Error Handling
```rust
use anyhow::{Context, Result};

fn read_config_file(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path))?;
    
    toml::from_str(&content)
        .with_context(|| "Failed to parse config file")
}
```

#### 2. Logging
```rust
use tracing::{info, warn, error, debug};

fn start_server(config: &Config) -> Result<()> {
    info!("Starting BWS server on {}:{}", config.hostname, config.port);
    
    match bind_server(&config) {
        Ok(server) => {
            info!("Server started successfully");
            server.run()
        }
        Err(e) => {
            error!("Failed to start server: {}", e);
            Err(e)
        }
    }
}
```

#### 3. Documentation
```rust
/// Handles HTTP requests for static file serving
/// 
/// # Arguments
/// 
/// * `request` - The incoming HTTP request
/// * `static_dir` - Path to the directory containing static files
/// 
/// # Returns
/// 
/// Returns a `Result` containing the HTTP response or an error
/// 
/// # Examples
/// 
/// ```rust
/// let response = handle_static_request(&request, "/var/www/static")?;
/// ```
pub fn handle_static_request(
    request: &HttpRequest, 
    static_dir: &str
) -> Result<HttpResponse> {
    // Implementation here
}
```

#### 4. Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_parsing() {
        let config_str = r#"
            [[sites]]
            name = "test"
            hostname = "localhost"
            port = 8080
            static_dir = "static"
        "#;
        
        let config: Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.sites.len(), 1);
        assert_eq!(config.sites[0].name, "test");
    }
    
    #[tokio::test]
    async fn test_server_startup() {
        let config = Config::default();
        let result = start_server(&config).await;
        assert!(result.is_ok());
    }
}
```

### Pull Request Process

#### 1. Before Submitting

- [ ] Code follows project style guidelines
- [ ] All tests pass (`cargo test`)
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated if needed
- [ ] CHANGELOG.md is updated for user-facing changes

#### 2. Pull Request Template

```markdown
## Description
Brief description of changes made.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Tests pass
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
```

#### 3. Review Process

1. **Automated Checks**: CI/CD pipeline runs automatically
2. **Code Review**: Maintainers review the changes
3. **Feedback**: Address any requested changes
4. **Approval**: Once approved, the PR will be merged

### Issue Reporting

#### Bug Reports

Use the bug report template:

```markdown
**Bug Description**
A clear description of the bug.

**Steps to Reproduce**
1. Step 1
2. Step 2
3. Step 3

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Environment**
- OS: [e.g., Ubuntu 20.04]
- Rust version: [e.g., 1.89.0]
- BWS version: [e.g., 0.1.5]

**Configuration**
```toml
# Include relevant configuration
```

**Logs**
```
Include relevant log output
```
```

#### Feature Requests

Use the feature request template:

```markdown
**Feature Description**
Clear description of the proposed feature.

**Use Case**
Explain why this feature would be useful.

**Proposed Solution**
Describe how you envision this feature working.

**Alternatives Considered**
Any alternative solutions you've considered.

**Additional Context**
Any other context about the feature request.
```

## Development Best Practices

### Code Review Guidelines

When reviewing code:

1. **Functionality**: Does the code work correctly?
2. **Style**: Does it follow project conventions?
3. **Performance**: Are there performance implications?
4. **Security**: Are there security concerns?
5. **Maintainability**: Is the code easy to understand and maintain?
6. **Testing**: Are there adequate tests?

### Performance Considerations

- Use `cargo bench` for performance testing
- Profile with `cargo flamegraph` when needed
- Consider memory allocation patterns
- Benchmark critical paths
- Document performance characteristics

### Security Guidelines

- Validate all user inputs
- Use secure defaults
- Follow principle of least privilege
- Regular security audits with `cargo audit`
- Handle sensitive data carefully
- Document security assumptions

## Development Tools

### Recommended Tools

```bash
# Essential tools
cargo install cargo-watch      # Auto-rebuild on changes
cargo install cargo-expand     # Expand macros
cargo install cargo-tree       # Dependency tree
cargo install cargo-outdated   # Check for outdated dependencies

# Development helpers
cargo install cargo-edit       # Add/remove dependencies easily
cargo install cargo-release    # Release management
cargo install cargo-benchcmp   # Compare benchmarks
```

### IDE Setup

#### VS Code
Recommended extensions:
- rust-analyzer
- CodeLLDB (debugging)
- Better TOML
- GitLens

#### Settings
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "editor.formatOnSave": true
}
```

### Debugging

```bash
# Debug build
cargo build

# Run with debugger
rust-gdb target/debug/bws

# Or with lldb
rust-lldb target/debug/bws

# Environment variables for debugging
RUST_BACKTRACE=1 cargo run
RUST_LOG=debug cargo run
```

## Release Process

### Version Numbering

BWS follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

1. **Update Version Numbers**
   ```bash
   # Update Cargo.toml
   version = "0.2.0"
   
   # Update documentation references
   ```

2. **Update CHANGELOG.md**
   ```markdown
   ## [0.2.0] - 2024-01-15
   
   ### Added
   - New feature descriptions
   
   ### Changed
   - Changed feature descriptions
   
   ### Fixed
   - Bug fix descriptions
   ```

3. **Run Full Test Suite**
   ```bash
   cargo test --all-features
   cargo clippy --all-targets --all-features
   cargo fmt --check
   cargo audit
   ```

4. **Update Documentation**
   ```bash
   cargo doc --no-deps
   mdbook build docs/
   ```

5. **Create Release Tag**
   ```bash
   git tag -a v0.2.0 -m "Release version 0.2.0"
   git push origin v0.2.0
   ```

## Community Guidelines

### Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct):

- Be friendly and welcoming
- Be patient
- Be respectful
- Be constructive
- Choose your words carefully

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Pull Requests**: Code contributions and reviews

### Getting Help

If you need help:

1. Check existing documentation
2. Search GitHub issues
3. Ask in GitHub Discussions
4. Ping maintainers in issues/PRs if urgent

### Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- GitHub contributor graphs
- Special thanks in documentation

## Advanced Topics

### Adding New Features

#### 1. Design Document
For significant features, create a design document:

```markdown
# Feature: HTTP/2 Support

## Overview
Add HTTP/2 support to BWS for improved performance.

## Motivation
- Better multiplexing
- Reduced latency
- Industry standard

## Design
- Use hyper's HTTP/2 implementation
- Maintain backward compatibility
- Configuration options for HTTP/2 settings

## Implementation Plan
1. Update dependencies
2. Add configuration options
3. Implement HTTP/2 handling
4. Add tests
5. Update documentation

## Testing Strategy
- Unit tests for new code
- Integration tests with HTTP/2 clients
- Performance benchmarks
- Compatibility testing

## Documentation Updates
- Configuration reference
- Performance guide
- Migration guide
```

#### 2. Implementation Steps
1. Create feature branch
2. Add configuration options
3. Implement core functionality
4. Add comprehensive tests
5. Update documentation
6. Submit pull request

### Performance Optimization

When optimizing performance:

1. **Measure First**: Use benchmarks to identify bottlenecks
2. **Profile**: Use profiling tools to understand behavior
3. **Optimize**: Make targeted improvements
4. **Verify**: Confirm improvements with benchmarks
5. **Document**: Update performance documentation

### Dependency Management

```bash
# Add dependency
cargo add tokio --features full

# Add dev dependency
cargo add --dev criterion

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated

# Audit for security issues
cargo audit
```

## Troubleshooting Development Issues

### Common Issues

#### 1. Build Failures
```bash
# Clean and rebuild
cargo clean
cargo build

# Check for dependency issues
cargo tree
cargo update
```

#### 2. Test Failures
```bash
# Run specific test
cargo test test_name -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Run tests in single thread
cargo test -- --test-threads=1
```

#### 3. Formatting Issues
```bash
# Format all code
cargo fmt

# Check specific file
rustfmt src/lib.rs
```

#### 4. Linting Issues
```bash
# Run clippy with all targets
cargo clippy --all-targets

# Allow specific lints
#[allow(clippy::too_many_arguments)]
```

## Resources

### Learning Resources
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Programming in Rust](https://rust-lang.github.io/async-book/)
- [Pingora Documentation](https://github.com/cloudflare/pingora)

### Tools and Libraries
- [tokio](https://tokio.rs/) - Async runtime
- [hyper](https://hyper.rs/) - HTTP library
- [pingora](https://github.com/cloudflare/pingora) - HTTP proxy framework
- [tracing](https://tracing.rs/) - Logging and diagnostics

### Similar Projects
- [nginx](https://nginx.org/) - High-performance web server
- [caddy](https://caddyserver.com/) - Modern web server
- [traefik](https://traefik.io/) - Cloud-native application proxy

Thank you for contributing to BWS! Your contributions help make the project better for everyone.
