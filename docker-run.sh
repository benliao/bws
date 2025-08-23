#!/bin/bash

# BWS Docker Helper Script
# Usage: ./docker-run.sh [build|run|stop|logs|shell]

IMAGE_NAME="ghcr.io/benliao/bws"
CONTAINER_NAME="bws-server"
LOCAL_IMAGE="bws-web-server"

build() {
    echo "Building BWS Docker image locally..."
    docker build -t "$LOCAL_IMAGE" .
    echo "Build completed: $LOCAL_IMAGE"
}

run() {
    echo "Running BWS container..."
    
    # Stop existing container if running
    docker stop "$CONTAINER_NAME" 2>/dev/null || true
    docker rm "$CONTAINER_NAME" 2>/dev/null || true
    
    # Create necessary directories
    mkdir -p docker-logs docker-run
    
    # Run the container
    docker run -d \
        --name "$CONTAINER_NAME" \
        -p 8080:8080 \
        -p 8081:8081 \
        -p 8082:8082 \
        -p 8083:8083 \
        -v "$(pwd)/config.toml:/app/config.toml:ro" \
        -v "$(pwd)/static:/app/static:ro" \
        -v "$(pwd)/static-blog:/app/static-blog:ro" \
        -v "$(pwd)/static-api:/app/static-api:ro" \
        -v "$(pwd)/static-dev:/app/static-dev:ro" \
        -v "$(pwd)/docker-logs:/app/logs" \
        -v "$(pwd)/docker-run:/app/run" \
        -e RUST_LOG=info \
        "$LOCAL_IMAGE"
    
    echo "Container started: $CONTAINER_NAME"
    echo "Available at:"
    echo "  • Main site: http://localhost:8080"
    echo "  • Blog site: http://localhost:8081"
    echo "  • API site: http://localhost:8082"
    echo "  • Dev site: http://localhost:8083"
    echo ""
    echo "Health check: curl http://localhost:8080/api/health"
    echo "View logs: ./docker-run.sh logs"
}

run_remote() {
    echo "Running BWS container from GitHub Container Registry..."
    
    # Stop existing container if running
    docker stop "$CONTAINER_NAME" 2>/dev/null || true
    docker rm "$CONTAINER_NAME" 2>/dev/null || true
    
    # Pull latest image
    docker pull "$IMAGE_NAME:latest"
    
    # Create necessary directories
    mkdir -p docker-logs docker-run
    
    # Run the container
    docker run -d \
        --name "$CONTAINER_NAME" \
        -p 8080:8080 \
        -p 8081:8081 \
        -p 8082:8082 \
        -p 8083:8083 \
        -v "$(pwd)/config.toml:/app/config.toml:ro" \
        -v "$(pwd)/static:/app/static:ro" \
        -v "$(pwd)/static-blog:/app/static-blog:ro" \
        -v "$(pwd)/static-api:/app/static-api:ro" \
        -v "$(pwd)/static-dev:/app/static-dev:ro" \
        -v "$(pwd)/docker-logs:/app/logs" \
        -v "$(pwd)/docker-run:/app/run" \
        -e RUST_LOG=info \
        "$IMAGE_NAME:latest"
    
    echo "Container started from remote image: $CONTAINER_NAME"
    echo "Available at:"
    echo "  • Main site: http://localhost:8080"
    echo "  • Blog site: http://localhost:8081"
    echo "  • API site: http://localhost:8082"
    echo "  • Dev site: http://localhost:8083"
}

stop() {
    echo "Stopping BWS container..."
    docker stop "$CONTAINER_NAME" 2>/dev/null || echo "Container not running"
    docker rm "$CONTAINER_NAME" 2>/dev/null || echo "Container already removed"
    echo "Container stopped and removed"
}

logs() {
    echo "Showing BWS container logs (Ctrl+C to exit)..."
    docker logs -f "$CONTAINER_NAME"
}

shell() {
    echo "Opening shell in BWS container..."
    docker exec -it "$CONTAINER_NAME" /bin/bash
}

status() {
    echo "BWS Container Status:"
    docker ps -f "name=$CONTAINER_NAME" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    
    echo ""
    echo "Testing connectivity..."
    if curl -s -f http://localhost:8080/api/health > /dev/null; then
        echo "✓ Server is responding"
        curl -s http://localhost:8080/api/health | jq . 2>/dev/null || curl -s http://localhost:8080/api/health
    else
        echo "✗ Server is not responding"
    fi
}

case "$1" in
    build)
        build
        ;;
    run)
        run
        ;;
    run-remote)
        run_remote
        ;;
    stop)
        stop
        ;;
    logs)
        logs
        ;;
    shell)
        shell
        ;;
    status)
        status
        ;;
    restart)
        stop
        sleep 2
        run
        ;;
    *)
        echo "Usage: $0 {build|run|run-remote|stop|logs|shell|status|restart}"
        echo ""
        echo "Commands:"
        echo "  build      - Build Docker image locally"
        echo "  run        - Run container from local image"
        echo "  run-remote - Run container from GitHub Container Registry"
        echo "  stop       - Stop and remove container"
        echo "  logs       - Show container logs"
        echo "  shell      - Open bash shell in container"
        echo "  status     - Show container status and test connectivity"
        echo "  restart    - Stop and start container"
        echo ""
        echo "Examples:"
        echo "  $0 build && $0 run     # Build and run locally"
        echo "  $0 run-remote          # Run latest from GHCR"
        echo "  $0 status              # Check if running"
        exit 1
        ;;
esac

exit $?
