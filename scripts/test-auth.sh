#!/bin/bash

# Toka Collaborative Auth - Test Script
# This script helps you test the authentication system easily

set -e

echo "🚀 Toka Collaborative Auth Test Script"
echo "======================================"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install it from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust is installed"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the Toka root directory"
    exit 1
fi

echo "✅ In the right directory"

# Check if environment variables are set
if [ -z "$GITHUB_CLIENT_ID" ] || [ -z "$GITHUB_CLIENT_SECRET" ]; then
    echo "⚠️  Environment variables not set. Checking for .env file..."
    
    if [ -f ".env" ]; then
        echo "✅ Found .env file, loading variables..."
        source .env
    else
        echo "❌ No .env file found. Please create one with your GitHub App credentials."
        echo ""
        echo "Copy the example file and fill in your values:"
        echo "cp crates/toka-collaborative-auth/env.example .env"
        echo ""
        echo "Then edit .env with your GitHub App credentials."
        exit 1
    fi
fi

# Verify required environment variables
if [ -z "$GITHUB_CLIENT_ID" ] || [ -z "$GITHUB_CLIENT_SECRET" ]; then
    echo "❌ Missing required environment variables:"
    echo "   GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET are required"
    echo ""
    echo "Please set them in your .env file or as environment variables."
    exit 1
fi

echo "✅ Environment variables configured"
echo "   GitHub Client ID: $GITHUB_CLIENT_ID"
echo "   Redirect URI: ${REDIRECT_URI:-http://localhost:3000/auth/callback}"

# Build the auth service
echo ""
echo "🔨 Building the auth service..."
cargo build --bin toka-auth-service

echo "✅ Auth service built successfully"

# Start the auth service
echo ""
echo "🌐 Starting the auth service..."
echo "📱 Visit http://localhost:3000/auth/login to test the OAuth flow"
echo "🔄 Press Ctrl+C to stop the service"
echo ""

cargo run --bin toka-auth-service 