# Configuration

BWS uses TOML configuration files to define server behavior and site settings.

## Configuration File Location

By default, BWS looks for `config.toml` in the current directory. You can specify a different location:

```bash
bws-web-server --config /path/to/your/config.toml
```

## Basic Configuration Structure

```toml
[server]
name = "BWS Multi-Site Server"

[[sites]]
name = "example"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Custom-Header" = "value"
```

## Server Section

The `[server]` section contains global server settings:

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `name` | String | Server identification name | "BWS Server" |

```toml
[server]
name = "My Production BWS Server"
```

## Sites Configuration

Sites are defined using `[[sites]]` array tables. Each site represents a separate web service.

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Unique identifier for the site |
| `hostname` | String | Hostname to bind to |
| `port` | Integer | Port number to listen on |
| `static_dir` | String | Directory containing static files |

### Optional Fields

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `default` | Boolean | Whether this is the default site | `false` |
| `api_only` | Boolean | Only serve API endpoints, no static files | `false` |

### Example Site Configuration

```toml
[[sites]]
name = "main"
hostname = "localhost"
port = 8080
static_dir = "static"
default = true

[[sites]]
name = "api"
hostname = "api.localhost"
port = 8081
static_dir = "api-static"
api_only = true
```

## Complete Example

Here's a comprehensive configuration example:

```toml
[server]
name = "BWS Production Server"

# Main website
[[sites]]
name = "main"
hostname = "example.com"
port = 8080
static_dir = "static"
default = true

[sites.headers]
"X-Site-Name" = "Main Website"
"X-Powered-By" = "BWS/1.0"
"Cache-Control" = "public, max-age=3600"

# Blog subdomain
[[sites]]
name = "blog"
hostname = "blog.example.com"
port = 8081
static_dir = "blog-static"

[sites.headers]
"X-Site-Name" = "Blog"
"X-Content-Type" = "blog-content"
```

## Next Steps

- Learn about [Multi-Site Setup](./multi-site.md)
- Configure [Custom Headers](./headers.md)
- Set up [Health Monitoring](./health.md)
