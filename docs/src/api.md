# REST API

BWS provides a REST API for monitoring and management.

## Endpoints

### Health Check

**GET** `/api/health`

Returns the server health status.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-08-23T12:00:00Z",
  "version": "0.1.5",
  "uptime_seconds": 3600
}
```

**Example:**
```bash
curl http://localhost:8080/api/health
```

### Sites Information

**GET** `/api/sites`

Returns information about all configured sites.

**Response:**
```json
{
  "sites": [
    {
      "name": "main",
      "hostname": "localhost",
      "port": 8080,
      "static_dir": "static",
      "default": true,
      "headers": {
        "X-Site-Name": "BWS Main Site",
        "X-Powered-By": "BWS/1.0"
      }
    }
  ],
  "total_sites": 1
}
```

**Example:**
```bash
curl http://localhost:8080/api/sites
```

## Response Headers

All API responses include these headers:
- `Content-Type: application/json`
- `X-Powered-By: BWS/1.0`
- Site-specific custom headers (if configured)

## Error Responses

API endpoints return standard HTTP status codes:

- `200 OK` - Successful request
- `404 Not Found` - Endpoint not found
- `500 Internal Server Error` - Server error

**Error Format:**
```json
{
  "error": "Not Found",
  "message": "The requested endpoint does not exist",
  "available_endpoints": ["/", "/api/health", "/api/sites"]
}
```
