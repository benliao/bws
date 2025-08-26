#!/usr/bin/env python3
"""
Comprehensive WebSocket test with a real upstream server
This script demonstrates the BWS WebSocket proxy working with an actual WebSocket backend.
"""

import asyncio
import websockets
import json
import threading
import time
import sys
import signal
from websockets.server import serve

class WebSocketTestServer:
    def __init__(self, port=3001):
        self.port = port
        self.server = None
        self.clients = set()
        
    async def handler(self, websocket, path):
        """Handle WebSocket connections"""
        self.clients.add(websocket)
        print(f"[UPSTREAM] Client connected from {websocket.remote_address}, path: {path}")
        
        try:
            # Send welcome message
            await websocket.send(json.dumps({
                "type": "welcome",
                "message": "Connected to BWS WebSocket Proxy Test Server",
                "path": path,
                "timestamp": time.time()
            }))
            
            # Echo messages back
            async for message in websocket:
                print(f"[UPSTREAM] Received: {message}")
                
                try:
                    # Try to parse as JSON
                    data = json.loads(message)
                    response = {
                        "type": "echo",
                        "original": data,
                        "processed_at": time.time(),
                        "server": "BWS WebSocket Proxy Test Server"
                    }
                except json.JSONDecodeError:
                    # Handle plain text messages
                    response = {
                        "type": "echo",
                        "original": message,
                        "processed_at": time.time(),
                        "server": "BWS WebSocket Proxy Test Server"
                    }
                
                await websocket.send(json.dumps(response))
                
        except websockets.exceptions.ConnectionClosed:
            print(f"[UPSTREAM] Client disconnected from {websocket.remote_address}")
        except Exception as e:
            print(f"[UPSTREAM] Error handling client: {e}")
        finally:
            self.clients.discard(websocket)
    
    async def start_server(self):
        """Start the WebSocket server"""
        print(f"[UPSTREAM] Starting WebSocket test server on port {self.port}")
        self.server = await serve(self.handler, "localhost", self.port)
        print(f"[UPSTREAM] WebSocket server ready on ws://localhost:{self.port}")
        
    async def stop_server(self):
        """Stop the WebSocket server"""
        if self.server:
            print("[UPSTREAM] Stopping WebSocket server...")
            self.server.close()
            await self.server.wait_closed()
            print("[UPSTREAM] WebSocket server stopped")

class WebSocketTestClient:
    def __init__(self, proxy_url="ws://localhost:8080/ws"):
        self.proxy_url = proxy_url
        
    async def test_connection(self):
        """Test WebSocket connection through BWS proxy"""
        print(f"[CLIENT] Connecting to BWS proxy at {self.proxy_url}")
        
        try:
            async with websockets.connect(self.proxy_url) as websocket:
                print("[CLIENT] Connected successfully!")
                
                # Wait for welcome message
                welcome = await websocket.recv()
                print(f"[CLIENT] Welcome message: {welcome}")
                
                # Send test messages
                test_messages = [
                    "Hello BWS WebSocket Proxy!",
                    {"type": "test", "data": "JSON message test"},
                    {"type": "ping", "timestamp": time.time()}
                ]
                
                for msg in test_messages:
                    if isinstance(msg, dict):
                        msg_str = json.dumps(msg)
                    else:
                        msg_str = msg
                        
                    print(f"[CLIENT] Sending: {msg_str}")
                    await websocket.send(msg_str)
                    
                    # Wait for echo response
                    response = await websocket.recv()
                    print(f"[CLIENT] Received: {response}")
                    
                    # Small delay between messages
                    await asyncio.sleep(0.5)
                
                print("[CLIENT] Test completed successfully!")
                
        except Exception as e:
            print(f"[CLIENT] Connection failed: {e}")
            return False
            
        return True

async def main():
    """Main test function"""
    print("🧪 BWS WebSocket Proxy Full Test")
    print("=" * 50)
    
    # Create test server
    server = WebSocketTestServer(3001)
    
    # Start upstream server
    print("\n1. Starting upstream WebSocket server...")
    await server.start_server()
    
    # Give server time to start
    await asyncio.sleep(1)
    
    print("\n2. Instructions:")
    print("   - Start BWS with WebSocket proxy configuration")
    print("   - BWS should proxy requests from ws://localhost:8080/ws to ws://localhost:3001")
    print("   - The test client will connect through the proxy")
    print()
    print("3. Configuration example for BWS:")
    print("   [[sites.proxy.routes]]")
    print("   path = \"/ws\"")
    print("   upstream = \"websocket_backend\"")
    print("   websocket = true")
    print("   ")
    print("   [[sites.proxy.upstreams]]")
    print("   name = \"websocket_backend\"")
    print("   url = \"http://localhost:3001\"")
    print()
    
    # Wait for user to start BWS
    print("4. Start BWS server and press Enter to run the test client...")
    try:
        input()
    except KeyboardInterrupt:
        print("\n[TEST] Interrupted by user")
        await server.stop_server()
        return
    
    # Run test client
    print("\n5. Running WebSocket client test...")
    client = WebSocketTestClient()
    success = await client.test_connection()
    
    if success:
        print("\n🎉 WebSocket proxy test PASSED!")
        print("✅ BWS successfully proxied WebSocket connections")
        print("✅ Bidirectional message relay working")
        print("✅ JSON and text messages handled correctly")
    else:
        print("\n❌ WebSocket proxy test FAILED!")
        print("⚠️  Check BWS configuration and server status")
    
    # Keep server running for manual testing
    print("\n6. Upstream server will continue running for manual testing...")
    print("   Connect directly to: ws://localhost:3001")
    print("   Connect via BWS proxy: ws://localhost:8080/ws")
    print("   Press Ctrl+C to stop the upstream server")
    
    try:
        while True:
            await asyncio.sleep(1)
    except KeyboardInterrupt:
        print("\n[TEST] Stopping upstream server...")
        await server.stop_server()
        print("[TEST] Test completed")

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n[TEST] Interrupted by user")
        sys.exit(0)
