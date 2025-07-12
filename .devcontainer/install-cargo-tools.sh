#!/bin/bash

# Fallback cargo tools installation script
set -e

echo "🔧 Installing cargo tools with network resilience..."

# Function to install a cargo tool with retry
install_cargo_tool() {
    local tool=$1
    local max_attempts=3
    
    echo "Installing $tool..."
    for attempt in $(seq 1 $max_attempts); do
        if cargo install --locked "$tool"; then
            echo "✅ Successfully installed $tool"
            return 0
        else
            echo "⚠️  Attempt $attempt failed for $tool"
            if [ $attempt -lt $max_attempts ]; then
                echo "Retrying in 5 seconds..."
                sleep 5
            fi
        fi
    done
    
    echo "❌ Failed to install $tool after $max_attempts attempts"
    return 1
}

# Ensure network is working
echo "🌐 Testing network connectivity..."
if ! nslookup index.crates.io > /dev/null 2>&1; then
    echo "⚠️  DNS issues detected, configuring alternative DNS..."
    echo "nameserver 8.8.8.8" | sudo tee /etc/resolv.conf > /dev/null
    echo "nameserver 8.8.4.4" | sudo tee -a /etc/resolv.conf > /dev/null
fi

# Install tools one by one with retry logic
tools=(
    "cargo-edit"
    "cargo-watch"
    "cargo-expand"
    "cargo-audit"
    "cargo-outdated"
    "cargo-tree"
    "cargo-deny"
    "cargo-nextest"
)

failed_tools=()

for tool in "${tools[@]}"; do
    if ! install_cargo_tool "$tool"; then
        failed_tools+=("$tool")
    fi
done

# Report results
if [ ${#failed_tools[@]} -eq 0 ]; then
    echo "✅ All cargo tools installed successfully"
else
    echo "⚠️  Some tools failed to install:"
    for tool in "${failed_tools[@]}"; do
        echo "  - $tool"
    done
    echo ""
    echo "You can try installing them manually later with:"
    echo "cargo install --locked <tool-name>"
fi

echo "🔧 Cargo tools installation complete" 