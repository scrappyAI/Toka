#!/bin/bash

# Network diagnostic script for Toka dev container
set -e

echo "🌐 Testing network connectivity for Toka development environment..."

# Test DNS resolution
echo "🔍 Testing DNS resolution..."
if nslookup index.crates.io > /dev/null 2>&1; then
    echo "✅ DNS resolution working"
else
    echo "❌ DNS resolution failed"
    echo "Current DNS configuration:"
    cat /etc/resolv.conf
    echo "Trying to fix DNS..."
    echo "nameserver 8.8.8.8" | sudo tee /etc/resolv.conf > /dev/null
    echo "nameserver 8.8.4.4" | sudo tee -a /etc/resolv.conf > /dev/null
fi

# Test HTTP connectivity
echo "🌍 Testing HTTP connectivity..."
if curl -s --connect-timeout 10 https://index.crates.io > /dev/null; then
    echo "✅ HTTPS connectivity to crates.io working"
else
    echo "❌ HTTPS connectivity to crates.io failed"
fi

# Test cargo registry
echo "📦 Testing cargo registry access..."
if cargo search anyhow --limit 1 > /dev/null 2>&1; then
    echo "✅ Cargo registry access working"
else
    echo "❌ Cargo registry access failed"
fi

# Test rustup
echo "🦀 Testing rustup..."
if rustup show > /dev/null 2>&1; then
    echo "✅ Rustup working"
else
    echo "❌ Rustup failed"
fi

# Show network configuration
echo "📋 Network configuration:"
echo "DNS servers:"
cat /etc/resolv.conf
echo ""
echo "Cargo configuration:"
if [ -f ~/.cargo/config.toml ]; then
    cat ~/.cargo/config.toml
else
    echo "No user cargo config found"
fi

echo "✅ Network diagnostics complete" 