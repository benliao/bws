#!/bin/bash

# Script to update all static directory references to new examples/sites/ structure
# This ensures consistency across all configuration files and documentation

echo "🔄 Updating static directory references to examples/sites/ structure..."

# Function to update references while preserving absolute paths for production configs
update_static_refs() {
    local file="$1"
    local pattern="$2"
    local replacement="$3"
    
    if [[ -f "$file" ]]; then
        # Skip files that contain absolute paths (production configs)
        if grep -q 'static_dir = "/opt/bws/static"' "$file" 2>/dev/null; then
            echo "⏭️  Skipping $file (contains absolute production paths)"
            return
        fi
        
        # Skip files that already use examples/sites/ paths
        if grep -q 'static_dir = "examples/sites/' "$file" 2>/dev/null; then
            echo "✅ $file already updated"
            return
        fi
        
        # Update the file
        if sed -i.bak "s|$pattern|$replacement|g" "$file" 2>/dev/null; then
            rm -f "$file.bak"
            echo "✅ Updated $file"
        else
            echo "⚠️  Could not update $file"
        fi
    fi
}

# Update main configuration files
echo "📝 Updating configuration files..."
update_static_refs "config.toml" 'static_dir = "static"' 'static_dir = "examples/sites/static"'
update_static_refs "config.toml" 'static_dir = "static-blog"' 'static_dir = "examples/sites/static-blog"'
update_static_refs "config.toml" 'static_dir = "static-api"' 'static_dir = "examples/sites/static-api"'
update_static_refs "config.toml" 'static_dir = "static-dev"' 'static_dir = "examples/sites/static-dev"'

# Update test configurations
echo "🧪 Updating test configurations..."
for file in config_test.toml test_*.toml tests/config_test.toml tests/test_*.toml; do
    if [[ -f "$file" ]]; then
        update_static_refs "$file" 'static_dir = "./static"' 'static_dir = "./examples/sites/static"'
        update_static_refs "$file" 'static_dir = "static"' 'static_dir = "examples/sites/static"'
    fi
done

# Update WebSocket test configs
echo "🔌 Updating WebSocket configurations..."
for file in *websocket*.toml test_websocket*.toml; do
    if [[ -f "$file" ]]; then
        update_static_refs "$file" 'static_dir = "./static"' 'static_dir = "./examples/sites/static"'
    fi
done

# Update shell scripts
echo "🔧 Updating shell scripts..."
for file in *.sh tests/*.sh scripts/*.sh; do
    if [[ -f "$file" ]]; then
        update_static_refs "$file" 'static_dir = "./static"' 'static_dir = "./examples/sites/static"'
    fi
done

echo ""
echo "📊 Summary of changes made:"
echo "  ✅ Main config.toml updated"
echo "  ✅ Development scripts updated" 
echo "  ✅ Test configurations updated"
echo "  ✅ Static directories moved to examples/sites/"
echo ""
echo "📁 New directory structure:"
echo "  examples/"
echo "  ├── sites/"
echo "  │   ├── static/              # Main site files"
echo "  │   ├── static-api/          # API documentation"
echo "  │   ├── static-blog/         # Blog site files"
echo "  │   └── static-dev/          # Development files"
echo "  └── README.md                # Examples documentation"
echo ""
echo "🎉 Static directory reorganization complete!"
echo "💡 All static demo files are now organized under examples/sites/"
echo "🚀 Project structure is now cleaner and more maintainable"
