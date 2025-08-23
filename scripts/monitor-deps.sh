#!/bin/bash

# Monitor script for dependency updates
# This script checks for updates to key dependencies that could resolve security issues

echo "🔍 Checking for dependency updates..."
echo "=================================="

# Check for Pingora updates
echo "📦 Checking Pingora versions..."
CURRENT_PINGORA=$(cargo tree --depth 0 | grep pingora | head -1 | awk '{print $2}')
LATEST_PINGORA=$(cargo search pingora --limit 1 | head -1 | awk '{print $3}' | tr -d '"')

echo "   Current: $CURRENT_PINGORA"
echo "   Latest:  $LATEST_PINGORA"

if [ "$CURRENT_PINGORA" != "$LATEST_PINGORA" ]; then
    echo "   ⚠️  Update available!"
else
    echo "   ✅ Up to date"
fi

echo ""

# Check for security advisories
echo "🛡️  Running security audit..."
echo "Ignoring known accepted vulnerability RUSTSEC-2024-0437..."

if cargo audit --ignore RUSTSEC-2024-0437 > /dev/null 2>&1; then
    echo "   ✅ No new critical vulnerabilities found"
else
    echo "   ⚠️  New vulnerabilities detected!"
    echo "   Run 'cargo audit' for details"
fi

echo ""
echo "📋 Full audit output:"
cargo audit --ignore RUSTSEC-2024-0437

echo ""
echo "🏁 Monitoring complete"
echo "For security status, see SECURITY.md"
