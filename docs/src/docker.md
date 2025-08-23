# Docker Deployment

BWS provides comprehensive Docker support for easy deployment and containerization.

## Docker Image

### Official Docker Image
```bash
# Pull the latest image
docker pull ghcr.io/yourusername/bws:latest

# Pull a specific version
docker pull ghcr.io/yourusername/bws:0.1.5
```

### Building from Source
```bash
# Clone the repository
git clone https://github.com/yourusername/bws.git
cd bws

# Build the Docker image
docker build -t bws:local .
```

## Basic Usage

### Simple Container
```bash
# Run BWS with default configuration
docker run -p 8080:8080 ghcr.io/yourusername/bws:latest
```

### With Custom Configuration
```bash
# Run with custom config
docker run -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml \
  -v $(pwd)/static:/app/static \
  ghcr.io/yourusername/bws:latest
```

### Background Execution
```bash
# Run as daemon
docker run -d \
  --name bws-server \
  -p 8080:8080 \
  --restart unless-stopped \
  -v $(pwd)/config.toml:/app/config.toml \
  -v $(pwd)/static:/app/static \
  ghcr.io/yourusername/bws:latest
```

## Docker Compose

### Basic Setup
```yaml
# docker-compose.yml
version: '3.8'

services:
  bws:
    image: ghcr.io/yourusername/bws:latest
    ports:
      - "8080:8080"
    volumes:
      - ./config.toml:/app/config.toml
      - ./static:/app/static
      - ./logs:/app/logs
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

### Multi-Site Setup
```yaml
# docker-compose.yml
version: '3.8'

services:
  bws-main:
    image: ghcr.io/yourusername/bws:latest
    container_name: bws-main
    ports:
      - "8080:8080"
    volumes:
      - ./sites/main/config.toml:/app/config.toml
      - ./sites/main/static:/app/static
      - ./logs/main:/app/logs
    environment:
      - RUST_LOG=info
      - BWS_SITE_NAME=main
    restart: unless-stopped
    networks:
      - bws-network

  bws-api:
    image: ghcr.io/yourusername/bws:latest
    container_name: bws-api
    ports:
      - "8081:8080"
    volumes:
      - ./sites/api/config.toml:/app/config.toml
      - ./sites/api/static:/app/static
      - ./logs/api:/app/logs
    environment:
      - RUST_LOG=info
      - BWS_SITE_NAME=api
    restart: unless-stopped
    networks:
      - bws-network

networks:
  bws-network:
    driver: bridge
```

### With Reverse Proxy
```yaml
# docker-compose.yml
version: '3.8'

services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
      - ./nginx/ssl:/etc/nginx/ssl
    depends_on:
      - bws
    networks:
      - web-network

  bws:
    image: ghcr.io/yourusername/bws:latest
    expose:
      - "8080"
    volumes:
      - ./config.toml:/app/config.toml
      - ./static:/app/static
      - ./logs:/app/logs
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    networks:
      - web-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

networks:
  web-network:
    driver: bridge
```

## Environment Variables

### Configuration via Environment
```bash
# Override configuration with environment variables
docker run -p 8080:8080 \
  -e BWS_CONFIG=/app/config.toml \
  -e BWS_LOG_FILE=/app/logs/bws.log \
  -e BWS_PID_FILE=/app/bws.pid \
  -e RUST_LOG=debug \
  ghcr.io/yourusername/bws:latest
```

### Available Environment Variables
```bash
# Core settings
BWS_CONFIG=/app/config.toml          # Configuration file path
BWS_LOG_FILE=/app/logs/bws.log       # Log file path
BWS_PID_FILE=/app/bws.pid            # PID file path

# Logging
RUST_LOG=info                        # Log level
RUST_BACKTRACE=1                     # Enable backtraces

# Application
BWS_SITE_NAME=main                   # Site identifier
BWS_BIND_ADDRESS=0.0.0.0             # Bind address
BWS_PORT=8080                        # Default port
```

## Volume Mounting

### Essential Volumes
```bash
# Configuration and static files
docker run -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  -v $(pwd)/logs:/app/logs \
  ghcr.io/yourusername/bws:latest
```

### Development Setup
```bash
# Mount source for development
docker run -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml \
  -v $(pwd)/static:/app/static \
  -v $(pwd)/logs:/app/logs \
  -v $(pwd)/data:/app/data \
  ghcr.io/yourusername/bws:latest
```

### Production Setup
```bash
# Production with named volumes
docker volume create bws-config
docker volume create bws-static
docker volume create bws-logs

docker run -p 8080:8080 \
  -v bws-config:/app/config \
  -v bws-static:/app/static \
  -v bws-logs:/app/logs \
  ghcr.io/yourusername/bws:latest
```

## Health Checks

### Docker Health Check
```dockerfile
# Built into the image
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1
```

### Custom Health Check Script
```bash
#!/bin/bash
# health-check.sh
curl -f -s http://localhost:8080/health > /dev/null
if [ $? -eq 0 ]; then
    exit 0
else
    exit 1
fi
```

### Docker Compose Health Check
```yaml
services:
  bws:
    image: ghcr.io/yourusername/bws:latest
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

## Networking

### Bridge Network
```bash
# Create custom network
docker network create bws-network

# Run container on custom network
docker run --network bws-network \
  --name bws-server \
  -p 8080:8080 \
  ghcr.io/yourusername/bws:latest
```

### Host Network
```bash
# Use host networking (Linux only)
docker run --network host \
  ghcr.io/yourusername/bws:latest
```

### Internal Communication
```yaml
# docker-compose.yml
services:
  bws:
    image: ghcr.io/yourusername/bws:latest
    networks:
      - internal
    # No ports exposed externally

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    networks:
      - internal
    depends_on:
      - bws

networks:
  internal:
    driver: bridge
    internal: true  # No external access
```

## Security

### Non-Root User
```dockerfile
# Built into the image
USER bws
WORKDIR /app
```

### Read-Only Root Filesystem
```bash
# Run with read-only root filesystem
docker run --read-only \
  --tmpfs /tmp \
  --tmpfs /app/logs \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  -p 8080:8080 \
  ghcr.io/yourusername/bws:latest
```

### Security Options
```bash
# Enhanced security
docker run --security-opt=no-new-privileges \
  --cap-drop=ALL \
  --cap-add=NET_BIND_SERVICE \
  -p 8080:8080 \
  ghcr.io/yourusername/bws:latest
```

### Docker Compose Security
```yaml
services:
  bws:
    image: ghcr.io/yourusername/bws:latest
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
    tmpfs:
      - /tmp
      - /app/logs
```

## Production Deployment

### Resource Limits
```yaml
# docker-compose.yml
services:
  bws:
    image: ghcr.io/yourusername/bws:latest
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
```

### Logging Configuration
```yaml
services:
  bws:
    image: ghcr.io/yourusername/bws:latest
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
        labels: "service=bws"
```

### Monitoring Integration
```yaml
services:
  bws:
    image: ghcr.io/yourusername/bws:latest
    labels:
      - "prometheus.io/scrape=true"
      - "prometheus.io/port=8080"
      - "prometheus.io/path=/metrics"
```

## Docker Build Optimization

### Multi-Stage Build
```dockerfile
# Build stage
FROM rust:1.89-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false bws

WORKDIR /app
COPY --from=builder /app/target/release/bws /usr/local/bin/
COPY --chown=bws:bws config.toml ./
COPY --chown=bws:bws static ./static

USER bws
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["bws", "--config", "/app/config.toml"]
```

### Build Arguments
```dockerfile
ARG RUST_VERSION=1.89
ARG TARGET=x86_64-unknown-linux-gnu

FROM rust:${RUST_VERSION}-slim as builder
# ... build process ...
```

```bash
# Build with custom arguments
docker build --build-arg RUST_VERSION=1.89 -t bws:custom .
```

## Troubleshooting

### Container Won't Start
```bash
# Check container logs
docker logs bws-server

# Run interactively for debugging
docker run -it --entrypoint /bin/bash ghcr.io/yourusername/bws:latest

# Check configuration
docker run --rm -v $(pwd)/config.toml:/app/config.toml \
  ghcr.io/yourusername/bws:latest --config-check
```

### Port Binding Issues
```bash
# Check if port is already in use
netstat -tulpn | grep :8080

# Use different port
docker run -p 8081:8080 ghcr.io/yourusername/bws:latest
```

### Volume Mount Problems
```bash
# Check file permissions
ls -la config.toml static/

# Fix permissions
chmod 644 config.toml
chmod -R 644 static/
```

### Health Check Failures
```bash
# Test health endpoint manually
docker exec bws-server curl -f http://localhost:8080/health

# Check health check logs
docker inspect bws-server | jq '.[0].State.Health'
```

## Best Practices

### Image Management
- Use specific version tags, not `latest`
- Regularly update base images
- Scan images for vulnerabilities
- Use multi-stage builds to reduce image size

### Configuration
- Use external configuration files
- Store secrets securely (Docker secrets, environment variables)
- Mount configuration as read-only
- Validate configuration before deployment

### Monitoring
- Always include health checks
- Monitor resource usage
- Set up log aggregation
- Use structured logging

### Security
- Run as non-root user
- Use read-only filesystems when possible
- Drop unnecessary capabilities
- Regularly update dependencies

## Next Steps

- Configure [Production Environment](./production.md)
- Set up [Performance Monitoring](./performance.md)
- Learn about [Troubleshooting](./troubleshooting.md)
