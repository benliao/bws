#!/bin/bash

# BWS Web Server Daemon Management Script
# Usage: ./bws-daemon.sh {start|stop|restart|status}

BWS_BINARY="./target/release/bws-web-server"
BWS_CONFIG="config.toml"
PID_FILE="./bws-daemon.pid"
LOG_FILE="./bws-daemon.log"

start() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "BWS daemon is already running (PID: $PID)"
            return 1
        else
            echo "Removing stale PID file"
            rm -f "$PID_FILE"
        fi
    fi
    
    echo "Starting BWS daemon..."
    if [ ! -f "$BWS_BINARY" ]; then
        echo "Binary not found at $BWS_BINARY"
        echo "Run 'cargo build --release' first"
        return 1
    fi
    
    $BWS_BINARY --daemon --config "$BWS_CONFIG" --pid-file "$PID_FILE" --log-file "$LOG_FILE"
    
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        echo "BWS daemon started successfully (PID: $PID)"
        echo "Log file: $LOG_FILE"
        echo "PID file: $PID_FILE"
        return 0
    else
        echo "Failed to start BWS daemon"
        return 1
    fi
}

stop() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "Stopping BWS daemon (PID: $PID)..."
            kill "$PID"
            
            # Wait for process to stop
            for i in {1..10}; do
                if ! ps -p "$PID" > /dev/null 2>&1; then
                    echo "BWS daemon stopped successfully"
                    rm -f "$PID_FILE"
                    return 0
                fi
                sleep 1
            done
            
            echo "Daemon didn't stop gracefully, forcing kill..."
            kill -9 "$PID" 2>/dev/null
            rm -f "$PID_FILE"
            echo "BWS daemon forcefully stopped"
            return 0
        else
            echo "BWS daemon is not running (stale PID file)"
            rm -f "$PID_FILE"
            return 1
        fi
    else
        echo "BWS daemon is not running (no PID file)"
        return 1
    fi
}

status() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "BWS daemon is running (PID: $PID)"
            echo "Log file: $LOG_FILE"
            echo "PID file: $PID_FILE"
            
            # Show some recent log entries
            if [ -f "$LOG_FILE" ]; then
                echo "Recent log entries:"
                tail -5 "$LOG_FILE" | sed 's/^/  /'
            fi
            
            # Test if server is responding
            if command -v curl > /dev/null 2>&1; then
                echo "Testing server response..."
                if curl -s -f http://localhost:8080/api/health > /dev/null; then
                    echo "✓ Server is responding to requests"
                else
                    echo "✗ Server is not responding to requests"
                fi
            fi
            return 0
        else
            echo "BWS daemon is not running (stale PID file)"
            rm -f "$PID_FILE"
            return 1
        fi
    else
        echo "BWS daemon is not running"
        return 1
    fi
}

restart() {
    echo "Restarting BWS daemon..."
    stop
    sleep 2
    start
}

case "$1" in
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        restart
        ;;
    status)
        status
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        echo ""
        echo "Commands:"
        echo "  start   - Start the BWS daemon"
        echo "  stop    - Stop the BWS daemon"
        echo "  restart - Restart the BWS daemon"
        echo "  status  - Show daemon status and test server response"
        echo ""
        echo "Files:"
        echo "  Binary:  $BWS_BINARY"
        echo "  Config:  $BWS_CONFIG"
        echo "  PID:     $PID_FILE"
        echo "  Log:     $LOG_FILE"
        exit 1
        ;;
esac

exit $?
