# BWS Multi-Site Shared Port Test

This test configuration demonstrates BWS virtual hosting capabilities where multiple sites share the same port (8080) but are distinguished by their hostnames.

## 🏗️ Architecture

```
Port 8080 (Shared)
├── www.local.com     → examples/sites/static/       (Main Site)
├── blog.local.com    → examples/sites/static-blog/  (Blog Site)
├── api.local.com     → examples/sites/static-api/   (API Docs)
└── dev.local.com     → examples/sites/static-dev/   (Dev Site)
```

## 🚀 Quick Start

### 1. Setup DNS Resolution

Add these entries to your `/etc/hosts` file:

```bash
sudo bash -c 'echo "127.0.0.1 www.local.com blog.local.com api.local.com dev.local.com" >> /etc/hosts'
```

### 2. Start the Test Server

```bash
# Option 1: Use the test script (recommended)
./tests/test_multisite_shared_port.sh start

# Option 2: Start manually
./target/release/bws --config tests/test_multisite_shared_port.toml
```

### 3. Test the Sites

Open these URLs in your browser:

- **Main Site**: http://www.local.com:8080
- **Blog Site**: http://blog.local.com:8080  
- **API Docs**: http://api.local.com:8080
- **Dev Site**: http://dev.local.com:8080

## 🧪 Testing Commands

### Automated Testing

```bash
# Run full test suite
./tests/test_multisite_shared_port.sh test

# Check server status
./tests/test_multisite_shared_port.sh status

# Stop the server
./tests/test_multisite_shared_port.sh stop
```

### Manual Testing with curl

```bash
# Test each site
curl -H "Host: www.local.com" http://127.0.0.1:8080
curl -H "Host: blog.local.com" http://127.0.0.1:8080
curl -H "Host: api.local.com" http://127.0.0.1:8080
curl -H "Host: dev.local.com" http://127.0.0.1:8080

# Check site-specific headers
curl -I -H "Host: www.local.com" http://127.0.0.1:8080
curl -I -H "Host: blog.local.com" http://127.0.0.1:8080
curl -I -H "Host: api.local.com" http://127.0.0.1:8080
curl -I -H "Host: dev.local.com" http://127.0.0.1:8080
```

## ⚙️ Configuration Details

### Site Configuration

Each site in `tests/test_multisite_shared_port.toml` has:

- **Unique hostname**: For virtual hosting routing
- **Shared port**: All sites use port 8080
- **Separate static_dir**: Each site serves different content
- **Custom headers**: Site-specific response headers

### Example Site Configuration

```toml
[[sites]]
name = "main"
hostname = "www.local.com"
port = 8080
static_dir = "examples/sites/static"
default = true

[sites.headers]
"X-Site-Name" = "Main Site"
"X-Site-Type" = "main"
"X-Environment" = "test"
"X-Port-Sharing" = "enabled"
```

## 🔍 What to Verify

### 1. Virtual Hosting
- Each hostname serves different content
- Same port (8080) used by all sites
- Proper routing based on Host header

### 2. Site-Specific Headers
- Check for `X-Site-Name` header differences
- Verify `X-Port-Sharing: enabled` on all sites
- API site should have CORS headers

### 3. Static Content
- Each site serves from its own directory
- Content reflects the site's purpose
- Different styling and content per site

## 🐛 Troubleshooting

### DNS Issues

If sites don't resolve:

```bash
# Check /etc/hosts
grep local.com /etc/hosts

# Test DNS resolution
nslookup www.local.com
ping www.local.com
```

### Server Issues

If server won't start:

```bash
# Check if port is in use
lsof -i :8080

# Check BWS logs
tail -f /tmp/bws_multisite_test.log

# Verify configuration
./target/release/bws --config tests/test_multisite_shared_port.toml --help
```

### Site Routing Issues

If wrong content is served:

```bash
# Test with explicit Host headers
curl -v -H "Host: www.local.com" http://127.0.0.1:8080

# Check default site behavior
curl -v http://127.0.0.1:8080

# Verify site configuration
grep -A 5 "hostname.*local.com" tests/test_multisite_shared_port.toml
```

## 📊 Expected Results

### Headers You Should See

```http
HTTP/1.1 200 OK
X-Site-Name: Main Site
X-Site-Type: main
X-Environment: test
X-Port-Sharing: enabled
Content-Type: text/html
```

### Different Content Per Site

- **www.local.com**: Virtual hosting test page
- **blog.local.com**: Blog-style content with red theme
- **api.local.com**: API documentation with dark theme
- **dev.local.com**: Development info with gradient background

## 🎯 Success Criteria

✅ All 4 sites respond on port 8080  
✅ Each site serves different content  
✅ Host-based routing works correctly  
✅ Site-specific headers are present  
✅ No port conflicts or routing errors  

This test demonstrates BWS's powerful virtual hosting capabilities, allowing you to serve multiple distinct websites from a single server instance on a single port.
