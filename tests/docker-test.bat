@echo off
REM BWS Docker Hot Reload Test Runner for Windows
REM Builds and runs hot reload tests in a Docker Linux environment

setlocal enabledelayedexpansion

REM Configuration
set DOCKER_IMAGE=bws-hot-reload-test
set CONTAINER_NAME=bws-test-container
set TEST_PORT=8080

REM Get command (default is test)
set COMMAND=%1
if "%COMMAND%"=="" set COMMAND=test

echo BWS Docker Hot Reload Test Runner
echo ====================================

REM Check if Docker is available
docker --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Docker is not installed or not in PATH
    exit /b 1
)

docker info >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Docker is not running
    exit /b 1
)

echo [INFO] Docker is available

REM Handle different commands
if "%COMMAND%"=="test" goto :run_tests
if "%COMMAND%"=="interactive" goto :run_interactive
if "%COMMAND%"=="build" goto :build_only
if "%COMMAND%"=="clean" goto :clean_docker
if "%COMMAND%"=="help" goto :show_help
if "%COMMAND%"=="--help" goto :show_help
if "%COMMAND%"=="-h" goto :show_help

echo [ERROR] Unknown command: %COMMAND%
goto :show_help

:run_tests
echo [INFO] Building Docker image for hot reload testing...
docker build -f Dockerfile.test -t %DOCKER_IMAGE% .
if errorlevel 1 (
    echo [ERROR] Failed to build Docker image
    exit /b 1
)

echo [INFO] Running hot reload tests in Docker container...

REM Create test directories
if not exist docker-test-logs mkdir docker-test-logs
if not exist docker-test-run mkdir docker-test-run

REM Run tests
docker run --rm ^
    --name %CONTAINER_NAME% ^
    -p %TEST_PORT%:8080 ^
    -v "%cd%/tests:/app/tests" ^
    -v "%cd%/docker-test-logs:/app/logs" ^
    -v "%cd%/docker-test-run:/app/run" ^
    -e RUST_LOG=info ^
    %DOCKER_IMAGE% ^
    bash /app/tests/test_hot_reload_docker.sh

if errorlevel 1 (
    echo [ERROR] Hot reload tests failed
    exit /b 1
) else (
    echo [SUCCESS] Hot reload tests completed successfully!
)
goto :cleanup_and_exit

:run_interactive
echo [INFO] Building Docker image...
docker build -f Dockerfile.test -t %DOCKER_IMAGE% .
if errorlevel 1 (
    echo [ERROR] Failed to build Docker image
    exit /b 1
)

echo [INFO] Starting interactive Docker container for manual testing...
echo [INFO] Container will be available at http://localhost:%TEST_PORT%
echo [INFO] Use 'exit' to stop the container

REM Create test directories
if not exist docker-test-logs mkdir docker-test-logs
if not exist docker-test-run mkdir docker-test-run

REM Run interactive container
docker run -it --rm ^
    --name %CONTAINER_NAME%-interactive ^
    -p %TEST_PORT%:8080 ^
    -v "%cd%/tests:/app/tests" ^
    -v "%cd%/test-configs:/app/test-configs" ^
    -v "%cd%/docker-test-logs:/app/logs" ^
    -v "%cd%/docker-test-run:/app/run" ^
    -e RUST_LOG=debug ^
    %DOCKER_IMAGE% ^
    bash

goto :cleanup_and_exit

:build_only
echo [INFO] Building Docker image...
docker build -f Dockerfile.test -t %DOCKER_IMAGE% .
if errorlevel 1 (
    echo [ERROR] Failed to build Docker image
    exit /b 1
) else (
    echo [SUCCESS] Docker image built successfully!
)
goto :cleanup_and_exit

:clean_docker
echo [INFO] Cleaning Docker resources...

REM Stop and remove container
docker stop %CONTAINER_NAME% >nul 2>&1
docker rm %CONTAINER_NAME% >nul 2>&1

REM Remove image
docker rmi %DOCKER_IMAGE% >nul 2>&1

REM Clean up directories
if exist docker-test-logs rmdir /s /q docker-test-logs
if exist docker-test-run rmdir /s /q docker-test-run

echo [SUCCESS] Docker resources cleaned
goto :cleanup_and_exit

:show_help
echo.
echo Usage: %0 [COMMAND]
echo.
echo Commands:
echo   test        Run automated hot reload tests (default)
echo   interactive Run interactive container for manual testing
echo   build       Build Docker image only
echo   clean       Clean up Docker resources
echo   help        Show this help message
echo.
echo Examples:
echo   %0                    # Run automated tests
echo   %0 test              # Run automated tests
echo   %0 interactive       # Start interactive container
echo   %0 build             # Build image only
goto :cleanup_and_exit

:cleanup_and_exit
REM Cleanup is handled automatically by Docker --rm flag
exit /b %errorlevel%
