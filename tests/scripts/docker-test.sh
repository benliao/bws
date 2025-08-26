#!/bin/bash
# BWS Docker Hot Reload Test Runner
# Builds and runs hot reload tests in a Docker Linux environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DOCKER_IMAGE="bws-hot-reload-test"
CONTAINER_NAME="bws-test-container"
TEST_PORT="8080"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Clean up function
cleanup() {
    log_info "Cleaning up..."
    
    # Stop and remove container
    if docker ps -a | grep -q $CONTAINER_NAME; then
        log_info "Stopping container: $CONTAINER_NAME"
        docker stop $CONTAINER_NAME >/dev/null 2>&1 || true
        docker rm $CONTAINER_NAME >/dev/null 2>&1 || true
    fi
    
    # Clean up test directories
    rm -rf docker-test-logs docker-test-run
}

# Handle script interruption
trap cleanup EXIT INT TERM

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if Docker is installed and running
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        return 1
    fi
    
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker is not running"
        return 1
    fi
    
    log_success "Docker is available"
    return 0
}

# Build Docker image
build_image() {
    log_info "Building Docker image for hot reload testing..."
    
    if docker build -f Dockerfile.test -t $DOCKER_IMAGE .; then
        log_success "Docker image built successfully"
        return 0
    else
        log_error "Failed to build Docker image"
        return 1
    fi
}

# Run hot reload tests in container
run_tests() {
    log_info "Running hot reload tests in Docker container..."
    
    # Create test directories for volume mounts
    mkdir -p docker-test-logs docker-test-run
    
    # Run container with test script
    if docker run --rm \
        --name $CONTAINER_NAME \
        -p $TEST_PORT:8080 \
        -v "$(pwd)/tests:/app/tests" \
        -v "$(pwd)/test-configs:/app/test-configs" \
        -v "$(pwd)/docker-test-logs:/app/logs" \
        -v "$(pwd)/docker-test-run:/app/run" \
        -e RUST_LOG=info \
        $DOCKER_IMAGE \
        bash /app/tests/test_hot_reload_docker.sh; then
        
        log_success "Hot reload tests completed successfully"
        return 0
    else
        log_error "Hot reload tests failed"
        return 1
    fi
}

# Run interactive testing
run_interactive() {
    log_info "Starting interactive Docker container for manual testing..."
    
    mkdir -p docker-test-logs docker-test-run
    
    log_info "Container will be available at http://localhost:$TEST_PORT"
    log_info "Use 'exit' to stop the container"
    
    docker run -it --rm \
        --name "${CONTAINER_NAME}-interactive" \
        -p $TEST_PORT:8080 \
        -v "$(pwd)/tests:/app/tests" \
        -v "$(pwd)/test-configs:/app/test-configs" \
        -v "$(pwd)/docker-test-logs:/app/logs" \
        -v "$(pwd)/docker-test-run:/app/run" \
        -e RUST_LOG=debug \
        $DOCKER_IMAGE \
        bash
}

# Show usage
show_usage() {
    echo "BWS Docker Hot Reload Test Runner"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  test        Run automated hot reload tests (default)"
    echo "  interactive Run interactive container for manual testing"
    echo "  build       Build Docker image only"
    echo "  clean       Clean up Docker resources"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run automated tests"
    echo "  $0 test              # Run automated tests"
    echo "  $0 interactive       # Start interactive container"
    echo "  $0 build             # Build image only"
}

# Clean Docker resources
clean_docker() {
    log_info "Cleaning Docker resources..."
    
    # Remove container
    if docker ps -a | grep -q $CONTAINER_NAME; then
        docker stop $CONTAINER_NAME >/dev/null 2>&1 || true
        docker rm $CONTAINER_NAME >/dev/null 2>&1 || true
    fi
    
    # Remove image
    if docker images | grep -q $DOCKER_IMAGE; then
        docker rmi $DOCKER_IMAGE >/dev/null 2>&1 || true
    fi
    
    # Clean up directories
    rm -rf docker-test-logs docker-test-run
    
    log_success "Docker resources cleaned"
}

# Main function
main() {
    local command="${1:-test}"
    
    case $command in
        test)
            log_info "Starting automated hot reload tests in Docker..."
            
            if ! check_prerequisites; then
                exit 1
            fi
            
            if ! build_image; then
                exit 1
            fi
            
            if ! run_tests; then
                exit 1
            fi
            
            log_success "Docker hot reload testing completed successfully!"
            ;;
            
        interactive)
            log_info "Starting interactive testing environment..."
            
            if ! check_prerequisites; then
                exit 1
            fi
            
            if ! build_image; then
                exit 1
            fi
            
            run_interactive
            ;;
            
        build)
            log_info "Building Docker image..."
            
            if ! check_prerequisites; then
                exit 1
            fi
            
            if ! build_image; then
                exit 1
            fi
            
            log_success "Docker image built successfully!"
            ;;
            
        clean)
            clean_docker
            ;;
            
        help|--help|-h)
            show_usage
            ;;
            
        *)
            log_error "Unknown command: $command"
            echo ""
            show_usage
            exit 1
            ;;
    esac
}

# Execute main function
main "$@"
