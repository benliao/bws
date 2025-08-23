import asyncio
import websockets
import json
import time

async def echo_handler(websocket, path):
    print(f'[UPSTREAM] Client connected from {websocket.remote_address}, path: {path}')
    try:
        await websocket.send(json.dumps({
            'type': 'welcome',
            'message': 'Connected to BWS WebSocket Proxy Test Server',
            'path': path,
            'timestamp': time.time()
        }))
        
        async for message in websocket:
            print(f'[UPSTREAM] Received: {message}')
            response = {
                'type': 'echo',
                'original': message,
                'processed_at': time.time(),
                'server': 'BWS WebSocket Proxy Test Server'
            }
            await websocket.send(json.dumps(response))
            
    except websockets.exceptions.ConnectionClosed:
        print(f'[UPSTREAM] Client disconnected')
    except Exception as e:
        print(f'[UPSTREAM] Error: {e}')

async def main():
    print('[UPSTREAM] Starting WebSocket server on port 3001')
    server = await websockets.serve(echo_handler, 'localhost', 3001)
    print('[UPSTREAM] WebSocket server ready on ws://localhost:3001')
    await server.wait_closed()

asyncio.run(main())