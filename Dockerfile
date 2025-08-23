# Build stage
FROM rust:1.89 AS builder

WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin bws-web-server
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release --bin bws-web-server

# Runtime stage
FROM debian:bookworm-slim

# Install necessary packages including curl for health checks
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create directories with proper permissions
RUN mkdir -p /app/static /app/static-blog /app/static-api /app/static-dev \
    && mkdir -p /app/logs /app/run \
    && chown -R appuser:appuser /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/bws-web-server /app/bws-web-server

# Copy daemon management script
COPY bws-daemon.sh /app/bws-daemon.sh
RUN chmod +x /app/bws-daemon.sh /app/bws-web-server

# Copy configuration template
COPY config.toml /app/config.example.toml

# Copy static directories if they exist
COPY static* /app/
RUN find /app -name "static*" -type d -exec chown -R appuser:appuser {} \;

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

WORKDIR /app

# Set environment variables
ENV BWS_CONFIG=/app/config.toml
ENV BWS_LOG_FILE=/app/logs/bws.log
ENV BWS_PID_FILE=/app/run/bws.pid

# Expose ports for all sites
EXPOSE 8080 8081 8082 8083

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# Create startup script
RUN echo '#!/bin/bash' > /app/start.sh && \
    echo 'set -e' >> /app/start.sh && \
    echo '' >> /app/start.sh && \
    echo '# Copy example config if no config exists' >> /app/start.sh && \
    echo 'if [ ! -f "$BWS_CONFIG" ]; then' >> /app/start.sh && \
    echo '    echo "No config found at $BWS_CONFIG, copying example..."' >> /app/start.sh && \
    echo '    cp /app/config.example.toml "$BWS_CONFIG"' >> /app/start.sh && \
    echo 'fi' >> /app/start.sh && \
    echo '' >> /app/start.sh && \
    echo '# Create log and run directories' >> /app/start.sh && \
    echo 'mkdir -p /app/logs /app/run' >> /app/start.sh && \
    echo '' >> /app/start.sh && \
    echo '# Start the server' >> /app/start.sh && \
    echo 'if [ "$1" = "--daemon" ]; then' >> /app/start.sh && \
    echo '    echo "Starting BWS in daemon mode..."' >> /app/start.sh && \
    echo '    exec ./bws-web-server --config "$BWS_CONFIG" --daemon --log-file "$BWS_LOG_FILE" --pid-file "$BWS_PID_FILE"' >> /app/start.sh && \
    echo 'else' >> /app/start.sh && \
    echo '    echo "Starting BWS in foreground mode..."' >> /app/start.sh && \
    echo '    exec ./bws-web-server --config "$BWS_CONFIG" "$@"' >> /app/start.sh && \
    echo 'fi' >> /app/start.sh && \
    chmod +x /app/start.sh

# Default command - can be overridden
CMD ["./start.sh"]
