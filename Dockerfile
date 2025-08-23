# Build stage
FROM rust:1.89 as builder

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release --bin bws-web-server

# Runtime stage
FROM debian:bookworm-slim

# Install necessary packages
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create directories
RUN mkdir -p /app/static /app/static-blog /app/static-api /app/static-dev

# Copy binary from builder stage
COPY --from=builder /app/target/release/bws-web-server /app/bws-web-server

# Copy configuration template
COPY config.toml /app/config.example.toml

# Copy static directories if they exist
COPY static /app/static
COPY static-blog /app/static-blog
COPY static-api /app/static-api
COPY static-dev /app/static-dev

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

WORKDIR /app

# Expose ports for all sites
EXPOSE 8080 8081 8082 8083

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# Default command
CMD ["./bws-web-server"]
