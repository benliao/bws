# REST API

BWS provides a comprehensive REST API for monitoring, management, and configuration validation.

## Endpoints

### Health Check

**GET** `/api/health`

Returns the basic server health status.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-08-26T12:00:00Z",
  "version": "0.3.4",
  "uptime_seconds": 3600
}
```

**Example:**
```bash
curl http://localhost:8080/api/health
```

### Detailed Health Information

**GET** `/api/health/detailed`

Returns comprehensive server information including system details.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-08-26T12:00:00Z",
  "version": "0.3.4",
  "uptime_seconds": 3600,
  "system": {
    "platform": "linux",
    "memory_usage": "45MB",
    "cpu_usage": "2.1%"
  },
  "sites": {
    "total": 4,
    "active": 4
  },
  "configuration": {
    "last_reload": "2025-08-26T11:30:00Z",
    "config_file": "/app/config.toml"
  }
}
```

**Example:**
```bash
curl http://localhost:8080/api/health/detailed | jq
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
curl http://localhost:8080/api/sites | jq
```

### Configuration Reload

**POST** `/api/reload`

Triggers a hot reload of the configuration file without restarting the server.

**Request:**
```bash
curl -X POST http://localhost:8080/api/reload
```

**Success Response (200 OK):**
```json
{
  "status": "success",
  "message": "Configuration reloaded successfully",
  "timestamp": "2025-08-26T12:00:00Z",
  "sites_reloaded": 4
}
```

**Error Response (400 Bad Request):**
```json
{
  "status": "error",
  "message": "Configuration validation failed: Missing required field 'hostname' for site 'main'",
  "timestamp": "2025-08-26T12:00:00Z"
}
```

**Note:** The reload endpoint validates the new configuration before applying it. If validation fails, the existing configuration remains active.

## Response Headers

All API responses include these headers:
- `Content-Type: application/json`
- `X-Powered-By: BWS/0.3.4`
- Site-specific custom headers (if configured)

## Error Responses

API endpoints return standard HTTP status codes:

- `200 OK` - Successful request
- `400 Bad Request` - Invalid request or configuration error
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

## Management API

BWS provides a separate, secure Management API for administrative operations. This API runs on a dedicated port (default: 7654) and is restricted to localhost access only.

### Security

The Management API implements security-first design:

- **Localhost Only**: Always binds to `127.0.0.1` - no external access
- **IP Validation**: Double-checks request origin is localhost
- **Optional API Key**: Additional authentication layer
- **Audit Logging**: All operations logged with client IP

### Base URL

The Management API runs on a separate port from the main web server:

```
http://127.0.0.1:7654
```

### Authentication

Authentication is optional but recommended for production:

```bash
# Without API key (localhost only)
curl -X POST http://127.0.0.1:7654/api/config/reload

# With API key
curl -X POST http://127.0.0.1:7654/api/config/reload \
  -H "X-API-Key: your-secure-api-key"
```

### Configuration Reload

**POST** `/api/config/reload`

Reloads the server configuration from the configuration file.

**Headers:**
- `X-API-Key` (optional): API key for authentication

**Response (Success):**
```json
{
  "status": "success",
  "message": "Configuration reloaded successfully",
  "config_path": "config.toml",
  "timestamp": "2025-08-26T15:27:07Z",
  "note": "Changes will apply to new connections"
}
```

**Response (Error):**
```json
{
  "error": "Configuration reload failed",
  "details": "Invalid TOML syntax at line 15"
}
```

**Example:**
```bash
# Basic reload
curl -X POST http://127.0.0.1:7654/api/config/reload

# With API key
curl -X POST http://127.0.0.1:7654/api/config/reload \
  -H "X-API-Key: your-secure-api-key"
```

### Management API Errors

The Management API returns specific error codes:

- `401 Unauthorized` - Invalid or missing API key
- `403 Forbidden` - Request not from localhost
- `404 Not Found` - Endpoint does not exist
- `500 Internal Server Error` - Configuration reload failed

**Security Error Example:**
```json
{
  "error": "Access denied: localhost only"
}
```

**Authentication Error Example:**
```json
{
  "error": "Unauthorized: invalid API key"
}
```
