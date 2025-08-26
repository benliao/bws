# Docker Deployment

BWS provides official Docker images for easy deployment.

## Quick Start

### Run with Default Configuration
```bash
# Run BWS on port 8080
docker run -p 8080:8080 ghcr.io/benliao/bws:latest
```

### Run with Custom Configuration
```bash
# Run with custom config and static files
docker run -d \
  --name bws \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  ghcr.io/benliao/bws:latest
```

## Available Images

### Docker Hub Tags
- `latest` - Latest stable release
- `v0.3.5` - Specific version (recommended for production)
- `main` - Latest development build

### Multi-Architecture Support
Images support multiple architectures:
- `linux/amd64` (x86_64)
- `linux/arm64` (ARM64/Apple Silicon)

## Configuration Examples

### Single Static Site
```bash
# Create content
mkdir static
echo "<h1>Hello from Docker!</h1>" > static/index.html

# Run container
docker run -d \
  --name bws-static \
  -p 8080:8080 \
  -v $(pwd)/static:/app/static:ro \
  ghcr.io/benliao/bws:latest \
  bws /app/static --port 8080
```

### Multi-Site with SSL
```bash
# Create configuration
cat > config.toml << 'EOF'
[server]
name = "Docker Multi-Site"

[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "/app/static"
default = true

[[sites]]
name = "secure"
hostname = "secure.localhost"
port = 8443
static_dir = "/app/static"

[sites.ssl]
enabled = true
auto_cert = true
domains = ["secure.localhost"]

[sites.ssl.acme]
enabled = true
email = "admin@example.com"
challenge_dir = "/app/acme-challenges"
EOF

# Create directories
mkdir -p static acme-challenges

# Run container
docker run -d \
  --name bws-multisite \
  -p 8080:8080 \
  -p 8443:8443 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  -v $(pwd)/acme-challenges:/app/acme-challenges \
  ghcr.io/benliao/bws:latest
```

## Docker Compose

### Basic Setup
```yaml
# docker-compose.yml
version: '3.8'

services:
  bws:
    image: ghcr.io/benliao/bws:latest
    container_name: bws
    ports:
      - "8080:8080"
      - "8443:8443"
    volumes:
      - ./config.toml:/app/config.toml:ro
      - ./static:/app/static:ro
      - ./acme-challenges:/app/acme-challenges
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Production Setup with Reverse Proxy
```yaml
version: '3.8'

services:
  bws:
    image: ghcr.io/benliao/bws:v0.3.5
    container_name: bws
    expose:
      - "8080"
    volumes:
      - ./config.toml:/app/config.toml:ro
      - ./static:/app/static:ro
      - ./acme-challenges:/app/acme-challenges
    restart: unless-stopped
    networks:
      - bws-network

  nginx:
    image: nginx:alpine
    container_name: nginx-proxy
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl-certs:/etc/ssl/certs:ro
    depends_on:
      - bws
    restart: unless-stopped
    networks:
      - bws-network

networks:
  bws-network:
    driver: bridge
```

Start with compose:
```bash
docker-compose up -d
```

## Build Custom Image

### Dockerfile
```dockerfile
FROM ghcr.io/benliao/bws:latest

# Copy custom configuration
COPY config.toml /app/config.toml
COPY static/ /app/static/

# Expose ports
EXPOSE 8080 8443

# Run BWS
CMD ["bws", "--config", "/app/config.toml"]
```

### Build and Run
```bash
# Build custom image
docker build -t my-bws .

# Run custom image
docker run -d --name my-bws-container -p 8080:8080 my-bws
```

## Volume Mounts

### Important Directories
- `/app/config.toml` - Configuration file
- `/app/static/` - Static content directory  
- `/app/acme-challenges/` - ACME challenge directory
- `/app/certs/` - Manual SSL certificates

### Example with All Mounts
```bash
docker run -d \
  --name bws-full \
  -p 80:80 \
  -p 443:443 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/static:/app/static:ro \
  -v $(pwd)/certs:/app/certs:ro \
  -v $(pwd)/acme-challenges:/app/acme-challenges \
  -v $(pwd)/logs:/app/logs \
  ghcr.io/benliao/bws:latest
```

## Environment Variables

```bash
# Override config with environment variables
docker run -d \
  --name bws-env \
  -p 8080:8080 \
  -e BWS_LOG_LEVEL=debug \
  -e BWS_WORKERS=4 \
  -v $(pwd)/static:/app/static:ro \
  ghcr.io/benliao/bws:latest \
  bws /app/static --port 8080
```

## Health Checks

### Docker Health Check
```bash
# Run with health check
docker run -d \
  --name bws-health \
  -p 8080:8080 \
  --health-cmd="curl -f http://localhost:8080/api/health || exit 1" \
  --health-interval=30s \
  --health-timeout=10s \
  --health-retries=3 \
  ghcr.io/benliao/bws:latest
```

### Check Container Health
```bash
# Check health status
docker ps
docker inspect bws-health | grep -A 10 '"Health"'
```

## Troubleshooting

### Common Issues

**Container Won't Start:**
```bash
# Check logs
docker logs bws

# Validate configuration
docker run --rm \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  ghcr.io/benliao/bws:latest \
  bws --config /app/config.toml --dry-run
```

**Permission Issues:**
```bash
# Fix volume permissions
sudo chown -R 1000:1000 static/ acme-challenges/
```

**Port Conflicts:**
```bash
# Check port usage
docker ps -a
netstat -tulpn | grep :8080
```

### Debug Commands
```bash
# Run interactive shell
docker run -it --entrypoint /bin/sh ghcr.io/benliao/bws:latest

# Exec into running container
docker exec -it bws /bin/sh

# View container logs
docker logs -f bws
```

## Security

### Best Practices
- Use specific version tags (not `latest`) in production
- Mount configuration as read-only (`:ro`)
- Run as non-root user (default in BWS image)
- Use Docker secrets for sensitive data

### Non-Root User
```bash
# BWS image runs as user 1000 by default
docker run -d \
  --name bws-secure \
  --user 1000:1000 \
  -p 8080:8080 \
  ghcr.io/benliao/bws:latest
```
